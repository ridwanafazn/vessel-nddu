use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use serde_json::json;
use chrono::Utc;

use crate::data::gyro_data::{GyroData, GyroResponse, GyroStore, GyroRequest};
use crate::utils::mqtt::publish_gyro_mqtt;
use rumqttc::AsyncClient;

/// Type alias untuk WebSocket clients
pub type Tx = mpsc::UnboundedSender<Message>;
pub type Clients = Arc<Mutex<Vec<Tx>>>;

/// ==== GYRO SERVICE STRUCT ====

/// Inisialisasi store gyro kosong
pub fn init_gyro_store() -> GyroStore {
    Arc::new(Mutex::new(Some(GyroData::default())))
}

/// Ambil data gyro terakhir
pub async fn get_latest(store: &GyroStore) -> Option<GyroData> {
    store.lock().await.clone()
}

/// Update data gyro dari request baru
pub async fn update_data(store: &GyroStore, req: GyroRequest) -> GyroData {
    let mut gyro = GyroData::from(req);
    gyro.last_update = Utc::now();

    let mut guard = store.lock().await;
    *guard = Some(gyro.clone());
    gyro
}

/// Update konfigurasi gyro (misalnya MQTT, rate, dll)
pub async fn update_config(store: &GyroStore, new_config: &crate::data::gyro_data::GyroConfig) {
    let mut guard = store.lock().await;
    if let Some(ref mut gyro) = *guard {
        gyro.config = new_config.clone();
    }
}

/// ==== HELPER: Broadcast JSON ke semua WebSocket client ====
async fn broadcast_json(clients: &Clients, message: Message) {
    let clients_lock = clients.lock().await;
    for client in clients_lock.iter() {
        let _ = client.send(message.clone());
    }
}

/// ==== GYRO SECTION ====

/// Kirim data Gyro ke semua WebSocket client
pub async fn broadcast_gyro(clients: &Clients, gyro: &GyroData) {
    let resp = GyroResponse::from(gyro.clone());
    let message = Message::Text(
        json!({
            "type": "gyro_update",
            "data": resp
        })
        .to_string(),
    );
    broadcast_json(clients, Message::Text(message)).await;
}

/// Dispatch data Gyro ke WebSocket dan MQTT
pub async fn dispatch_gyro(clients: &Clients, gyro: &GyroData, mqtt: Option<&AsyncClient>) {
    // 1️⃣ Kirim ke WebSocket client
    broadcast_gyro(clients, gyro).await;

    // 2️⃣ Publish ke MQTT global
    if let Some(client) = mqtt {
        if let Err(e) = publish_gyro_mqtt(client, gyro).await {
            eprintln!("[Gyro MQTT] Publish error: {:?}", e);
        }
    }
}

/// ==== WEBSOCKET HANDLER ====

/// Menerima koneksi WebSocket (biasanya untuk dashboard)
pub async fn handle_websocket_connection(stream: TcpStream, clients: Clients) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        // Simpan TX channel client baru
        {
            let mut clients_lock = clients.lock().await;
            clients_lock.push(tx);
        }

        let clients_clone = clients.clone();

        // Task untuk mengirim data ke client
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            }

            // Hapus client yang sudah tutup
            let mut clients_lock = clients_clone.lock().await;
            clients_lock.retain(|c| !c.is_closed());
        });

        // Task untuk menerima pesan dari client
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    // echo balik ke semua client
                    broadcast_json(&clients, Message::Text(text.clone())).await;
                }
                Err(_) => break,
                _ => {}
            }
        }
    }
}

/// ==== TCP HANDLER (debug only) ====
pub async fn handle_tcp_connection(mut socket: TcpStream) {
    let mut buffer = [0; 1024];
    match socket.read(&mut buffer).await {
        Ok(n) if n > 0 => {
            let _ = socket.write_all(b"Gyro TCP received").await;
        }
        _ => {}
    }
}

pub fn start_gyro_publication_thread(
    config_state: SharedGyroConfig,
    data_state: SharedGyroState,
    ws_clients: Clients,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        let mut mqtt_client: Option<AsyncClient> = None;
        let mut eventloop_handle: Option<JoinHandle<()>> = None;

        loop {
            let update_rate = config_state.read().unwrap().update_rate.unwrap_or(1000);

            select! {
                Some(command) = command_rx.recv() => {
                    match command {
                        MqttCommand::Reconnect => {
                            println!("[MQTT Manager Gyro]: Reconnect command received. Resetting connection.");
                            if let Some(handle) = eventloop_handle.take() {
                                handle.abort();
                            }
                            mqtt_client = None;
                        }
                    }
                }
                _ = sleep(Duration::from_millis(update_rate)) => {}
            }

            // DIUBAH: Logika koneksi ulang dengan pola Lock -> Salin -> Lepas Lock -> Await
            if mqtt_client.is_none() {
                println!("[MQTT Manager Gyro]: Attempting to connect...");

                let connection_details = { // Scope pendek untuk lock
                    let config = config_state.read().unwrap();
                    if let (Some(ip), Some(port), Some(username), Some(password)) =
                        (&config.ip, config.port, &config.username, &config.password)
                    {
                        Some((ip.clone(), port, username.clone(), password.clone()))
                    } else {
                        None
                    }
                }; // Lock dilepas di sini

                if let Some((ip, port, username, password)) = connection_details {
                    let mut mqttoptions = MqttOptions::new("vessel_gyro_publisher", ip, port);
                    mqttoptions.set_credentials(username, password);
                    mqttoptions.set_keep_alive(Duration::from_secs(5));
                    
                    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
                    
                    // Operasi .await sekarang aman karena tidak ada lock yang dipegang
                    match eventloop.poll().await {
                        Ok(_) => {
                             println!("[MQTT Manager Gyro]: Connection successful!");
                             mqtt_client = Some(client);
                             let handle = tokio::spawn(async move {
                                 loop {
                                     if eventloop.poll().await.is_err() { break; }
                                 }
                             });
                             eventloop_handle = Some(handle);
                        }
                        Err(e) => {
                             eprintln!("[MQTT Manager Gyro]: Failed to connect: {}. Will retry later.", e);
                        }
                    }
                } else {
                    println!("[MQTT Manager Gyro]: Config is incomplete. Skipping connection attempt.");
                }
            }
            
            if let Some(client) = &mqtt_client {
                if eventloop_handle.as_ref().map_or(true, |h| h.is_finished()) {
                    eprintln!("[MQTT Manager Gyro]: Disconnected from broker. Will attempt to reconnect.");
                    mqtt_client = None;
                    eventloop_handle = None;
                    continue;
                }

                let data_to_publish = {
                    let guard = data_state.read().unwrap();
                    guard.as_ref().filter(|s| s.is_running).cloned()
                };

                if let Some(gyro_state) = data_to_publish {
                    let message_payload = serde_json::json!({ "type": "gyro_update", "data": gyro_state });
                    let message_string = message_payload.to_string();
                    println!("[GYRO PUBLISH]: {}", message_string);

                    utils::net::broadcast_ws_message(&ws_clients, message_string.clone()).await;
                    
                    let topics = config_state.read().unwrap().topics.clone().unwrap_or_default();
                    if let Err(e) = utils::mqtt::publish_mqtt_message(client, &topics, message_string).await {
                        eprintln!("Failed to publish Gyro data to MQTT: {}", e);
                    }
                }
            }
        }
    });
}