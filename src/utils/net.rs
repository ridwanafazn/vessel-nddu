use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use serde_json::json;

use crate::data::gps_data::{GPSData, GPSResponse};
use crate::data::gyro_data::{GyroData, GyroResponse};
use crate::utils::mqtt::{publish_gps_mqtt, publish_gyro_mqtt};
use rumqttc::AsyncClient;

/// Type alias untuk WebSocket clients
pub type Tx = mpsc::UnboundedSender<Message>;
pub type Clients = Arc<Mutex<Vec<Tx>>>;

/// ==== HELPER FUNCTIONS ====

// Broadcast JSON message ke semua WebSocket clients
async fn broadcast_json(clients: &Clients, message: Message) {
    let clients_lock = clients.lock().await;
    for client in clients_lock.iter() {
        let _ = client.send(message.clone());
    }
}

/// ==== GPS SECTION ====

// Broadcast GPSData ke semua WebSocket client
pub async fn broadcast_gps(clients: &Clients, gps: &GPSData) {
    let resp = GPSResponse::from(gps.clone());
    let message = Message::Text(
        json!({
            "type": "gps_update",
            "data": resp
        })
        .to_string()
        .into(),
    );
    broadcast_json(clients, message).await;
}

/// Dispatch GPS ke WebSocket dan MQTT
/// - Jika `mqtt` tersedia, publish ke MQTT.
/// - Jika `mqtt` None, hanya WS broadcast.
pub async fn dispatch_gps(clients: &Clients, gps: &GPSData, mqtt: Option<&AsyncClient>) {
    broadcast_gps(clients, gps).await;

    if let Some(client) = mqtt {
        if let Err(e) = publish_gps_mqtt(client, gps).await {
            eprintln!("MQTT publish error (gps): {:?}", e);
        }
    } else {
        // tidak ada mqtt client => skip publish
        // log debug supaya mudah dilacak saat testing
        tracing::debug!("dispatch_gps: no mqtt client provided, skipping MQTT publish");
    }
}

/// ==== GYRO SECTION ====

// Broadcast GyroData ke semua WebSocket client
pub async fn broadcast_gyro(clients: &Clients, gyro: &GyroData) {
    let resp = GyroResponse::from(gyro.clone());
    let message = Message::Text(
        json!({
            "type": "gyro_update",
            "data": resp
        })
        .to_string()
        .into(),
    );
    broadcast_json(clients, message).await;
}

/// Dispatch Gyro ke WebSocket dan MQTT
/// - jika `mqtt` tersedia, publish ke MQTT
/// - jika tidak ada, hanya WS broadcast
pub async fn dispatch_gyro(clients: &Clients, gyro: &GyroData, mqtt: Option<&AsyncClient>) {
    broadcast_gyro(clients, gyro).await;

    if let Some(client) = mqtt {
        if let Err(e) = publish_gyro_mqtt(client, gyro).await {
            eprintln!("MQTT publish error (gyro): {:?}", e);
        }
    } else {
        tracing::debug!("dispatch_gyro: no mqtt client provided, skipping MQTT publish");
    }
}

/// ==== WEBSOCKET HANDLER ====

// Handle koneksi WebSocket
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
        // Task kirim pesan ke client
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

        // Task baca pesan dari client
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    // echo ke semua client
                    let msg = Message::Text(text.clone());
                    broadcast_json(&clients, msg).await;
                }
                Err(_) => break,
                _ => {}
            }
        }
    }
}

/// ==== TCP HANDLER ====

// Handle koneksi TCP sederhana (untuk debug/test)
pub async fn handle_tcp_connection(mut socket: TcpStream) {
    let mut buffer = [0; 1024];
    match socket.read(&mut buffer).await {
        Ok(n) if n > 0 => {
            let _ = socket.write_all(b"TCP received").await;
        }
        _ => {}
    }
}
