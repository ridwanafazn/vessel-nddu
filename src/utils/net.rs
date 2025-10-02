use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::data::gps_data::{GPSData, GPSResponse};
use serde_json::json;
use crate::data::gyro_data::GyroData;

// tambahan untuk MQTT
use crate::utils::mqtt::publish_gps_mqtt;
use rumqttc::AsyncClient;

/// Type alias untuk WebSocket clients
pub type Tx = mpsc::UnboundedSender<Message>;
pub type Clients = Arc<Mutex<Vec<Tx>>>;

/// Broadcast GPSData ke semua client WS
pub async fn broadcast_gps(clients: &Clients, gps: &GPSData) {
    let resp = GPSResponse::from(gps.clone());

    // kirim JSON string ke semua client
    let message = Message::Text(
        json!({
            "type": "gps_update",
            "data": resp
        })
        .to_string().into(),
    );

    let clients_lock = clients.lock().await;
    for client in clients_lock.iter() {
        let _ = client.send(message.clone());
    }
}

/// Dispatch GPS ke semua saluran (WS + MQTT)
pub async fn dispatch_gps(clients: &Clients, gps: &GPSData, mqtt: Option<&AsyncClient>) {
    broadcast_gps(clients, gps).await;

    if let Some(client) = mqtt {
        if let Err(e) = publish_gps_mqtt(client, gps).await {
            eprintln!("MQTT publish error: {:?}", e);
        }
    }
}

pub async fn dispatch_gyro(clients: &Clients, data: &GyroData, mqtt: Option<&AsyncClient>) {
    let json = serde_json::json!({
        "type": "gyro_update",
        "data": data
    }).to_string();

    // broadcast ke WebSocket
    let msg = Message::Text(json.clone().into());
    let clients_lock = clients.lock().await;
    for client in clients_lock.iter() {
        let _ = client.send(msg.clone());
    }
    drop(clients_lock); // lepas lock lebih cepat

    // publish ke MQTT
    if let Some(mqtt) = mqtt {
        let _ = mqtt
            .publish("gyro/default", rumqttc::QoS::AtMostOnce, false, json)
            .await;
    }
}

/// Handle koneksi WebSocket
pub async fn handle_websocket_connection(stream: TcpStream, clients: Clients) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        {
            let mut clients_lock = clients.lock().await;
            clients_lock.push(tx);
        }

        let clients_clone = clients.clone();
        // task kirim pesan ke client
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            }
            let mut clients_lock = clients_clone.lock().await;
            clients_lock.retain(|c| !c.is_closed());
        });

        // task baca pesan dari client
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    // echo ke semua client
                    let message = Message::Text(text.clone());
                    let clients_lock = clients.lock().await;
                    for client in clients_lock.iter() {
                        let _ = client.send(message.clone());
                    }
                }
                Err(_) => break,
                _ => {}
            }
        }
    }
}

/// Handle koneksi TCP sederhana
pub async fn handle_tcp_connection(mut socket: TcpStream) {
    let mut buffer = [0; 1024];
    match socket.read(&mut buffer).await {
        Ok(n) if n > 0 => {
            let _ = socket.write_all(b"TCP received").await;
        }
        _ => {}
    }
}
