use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use std::time::Duration;
use tokio::time::sleep;
use tokio::select;

/// Status koneksi MQTT
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MqttState {
    Disconnected,
    Connecting,
    Connected,
}

/// Perintah antar service
#[derive(Debug)]
pub enum MqttCommand {
    Reconnect,
    Stop,
}

/// Konfigurasi dasar MQTT per service
#[derive(Clone)]
pub struct MqttServiceConfig {
    pub name: String,
    pub client_id: String,
    pub topic_prefix: String,
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub keep_alive: Duration,
    pub publish_interval: Duration,
}

struct MqttConnection {
    client: AsyncClient,
    eventloop: EventLoop,
}

impl MqttConnection {
    async fn connect(cfg: &MqttServiceConfig) -> Option<Self> {
        let mut mqttoptions = MqttOptions::new(&cfg.client_id, &cfg.ip, cfg.port);
        mqttoptions.set_credentials(&cfg.username, &cfg.password);
        mqttoptions.set_keep_alive(cfg.keep_alive);

        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
        println!(
            "[MQTT Manager {}]: Connected to {}:{}",
            cfg.name, cfg.ip, cfg.port
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

/// MqttManager — digunakan oleh semua service (GPS, Gyro, dsb)
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

/// Optional: loop manajerial MQTT (opsional, bisa dipakai untuk reconnect otomatis)
#[allow(dead_code)]
pub async fn start_service_manager(
    cfg: MqttServiceConfig,
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
                        println!("[MQTT Manager {}]: Stopping MQTT loop.", cfg.name);
                        break;
                    }
                }
            }
            _ = sleep(cfg.publish_interval) => {}
        }

        // Jika belum terhubung, coba koneksi baru
        if matches!(state, MqttState::Disconnected) {
            println!("[MQTT Manager {}]: Connecting...", cfg.name);
            match MqttConnection::connect(&cfg).await {
                Some(conn) => {
                    state = MqttState::Connected;
                    connection = Some(conn);
                    println!("[MQTT Manager {}]: Connected!", cfg.name);
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

            // heartbeat status ke topic status
            let topics = vec![format!("{}/status", cfg.topic_prefix)];
            let payload = format!("{{\"status\":\"ok\",\"service\":\"{}\"}}", cfg.name);
            if let Err(e) = conn.publish_message(&topics, payload).await {
                eprintln!("[MQTT Manager {}]: Publish error: {}", cfg.name, e);
            }

            connection = Some(conn);
        }
    }

    println!("[MQTT Manager {}]: Exited MQTT loop.", cfg.name);
    Ok(())
}
