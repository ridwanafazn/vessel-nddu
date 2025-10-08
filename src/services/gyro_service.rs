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
