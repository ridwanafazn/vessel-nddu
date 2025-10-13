use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use std::time::Duration;
use tokio::select;
use tokio::time::sleep;

/// Status koneksi MQTT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MqttState {
    Disconnected,
    Connecting,
    Connected,
}

/// Perintah yang dapat dikirim ke manager.
#[derive(Debug)]
pub enum MqttCommand {
    Reconnect,
    Stop,
}

/// Konfigurasi dasar untuk setiap service yang menggunakan MQTT manager.
#[derive(Clone)]
pub struct MqttServiceConfig {
    pub name: String,
    pub client_id: String,
    pub topic_prefix: String,
    pub keep_alive: Duration,
    pub publish_interval: Duration,
}

/// Struktur koneksi MQTT aktif.
struct MqttConnection {
    client: AsyncClient,
    eventloop: EventLoop,
}

impl MqttConnection {
    async fn connect(
        cfg: &MqttServiceConfig,
        ip: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Option<Self> {
        let mut mqttoptions = MqttOptions::new(&cfg.client_id, ip, port);
        mqttoptions.set_credentials(username, password);
        mqttoptions.set_keep_alive(cfg.keep_alive);

        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
        println!(
            "[MQTT Manager {}]: Connected to {}:{}",
            cfg.name, ip, port
        );

        Some(Self { client, eventloop })
    }

    async fn publish_message(
        &self,
        topics: &[String],
        payload: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for topic in topics {
            self.client
                .publish(topic, QoS::AtLeastOnce, false, payload.clone())
                .await?;
        }
        Ok(())
    }
}

/// 🔹 Struct baru: MqttManager
/// Tujuan: menyediakan API `publish_message` yang aman untuk dipanggil dari service
#[derive(Clone)]
pub struct MqttManager {
    client: AsyncClient,
}

impl MqttManager {
    pub fn new(client: AsyncClient) -> Self {
        Self { client }
    }

    pub async fn publish_message(
        &self,
        topics: &[String],
        payload: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for topic in topics {
            self.client
                .publish(topic, QoS::AtLeastOnce, false, payload.clone())
                .await?;
        }
        Ok(())
    }
}

/// Jalankan service manager MQTT (tanpa spawn internal).
pub async fn start_service_manager(
    cfg: MqttServiceConfig,
    ip: String,
    port: u16,
    username: String,
    password: String,
    mut command_rx: tokio::sync::mpsc::Receiver<MqttCommand>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut state = MqttState::Disconnected;
    let mut connection: Option<MqttConnection> = None;

    loop {
        select! {
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    MqttCommand::Reconnect => {
                        println!("[MQTT Manager {}]: Reconnect command received.", cfg.name);
                        state = MqttState::Disconnected;
                        connection = None;
                    }
                    MqttCommand::Stop => {
                        println!("[MQTT Manager {}]: Stopping loop.", cfg.name);
                        break;
                    }
                }
            }

            _ = sleep(cfg.publish_interval) => {}
        }

        if matches!(state, MqttState::Disconnected) {
            println!("[MQTT Manager {}]: Connecting...", cfg.name);
            match MqttConnection::connect(&cfg, &ip, port, &username, &password).await {
                Some(conn) => {
                    state = MqttState::Connected;
                    println!("[MQTT Manager {}]: Connected!", cfg.name);
                    connection = Some(conn);
                }
                None => {
                    eprintln!("[MQTT Manager {}]: Connection failed.", cfg.name);
                    continue;
                }
            }
        }

        if let Some(mut conn) = connection.take() {
            match conn.eventloop.poll().await {
                Ok(Event::Incoming(Packet::Disconnect)) => {
                    eprintln!("[MQTT Manager {}]: Disconnected.", cfg.name);
                    state = MqttState::Disconnected;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[MQTT Manager {}]: Poll error: {}", cfg.name, e);
                    state = MqttState::Disconnected;
                }
            }

            // contoh publikasi status
            let topics = vec![format!("{}/status", cfg.topic_prefix)];
            let payload = format!("{{\"status\":\"ok\",\"service\":\"{}\"}}", cfg.name);
            if let Err(e) = conn.publish_message(&topics, payload).await {
                eprintln!("[MQTT Manager {}]: Publish error: {}", cfg.name, e);
            }

            connection = Some(conn);
        }
    }

    println!("[MQTT Manager {}]: Exited loop.", cfg.name);
    Ok(())
}
