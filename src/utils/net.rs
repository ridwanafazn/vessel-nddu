use tokio::net::TcpStream; 
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use crate::data::gps_data::GPSData;
use crate::data::gps_data::GPSResponse;
use serde_json::json;

/// Type alias untuk WebSocket clients
pub type Tx = mpsc::UnboundedSender<Message>;
pub type Clients = Arc<Mutex<Vec<Tx>>>;

/// Broadcast GPSData ke semua client WS
pub fn broadcast_gps(clients: &Clients, gps: &GPSData) {
    let resp = GPSResponse::from(gps.clone());
    let message = Message::Text(json!({
        "type": "gps_update",
        "data": resp
    }).to_string().into());

    // lock sekali, clone semua sender, lalu lepas
    let clients_clone = {
        let clients_lock = clients.lock().unwrap();
        clients_lock.clone()
    };

    for client in clients_clone.iter() {
        let _ = client.send(message.clone());
    }
}

/// Handle koneksi WebSocket
pub async fn handle_websocket_connection(stream: TcpStream, clients: Clients) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        {
            // simpan tx ke daftar clients
            let mut clients_lock = clients.lock().unwrap();
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
            // jika koneksi client mati, hapus tx yang invalid
            let mut clients_lock = clients_clone.lock().unwrap();
            clients_lock.retain(|c| !c.is_closed());
        });

        // task baca pesan dari client
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    // broadcast balik ke semua client
                    let message = Message::Text(text.clone());
                    let clients_clone = {
                        let clients_lock = clients.lock().unwrap();
                        clients_lock.clone()
                    };
                    for client in clients_clone.iter() {
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
    if let Ok(n) = socket.read(&mut buffer).await {
        if n > 0 {
            let _ = socket.write_all(b"TCP received").await;
        }
    }
}
