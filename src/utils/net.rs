use std::sync::Arc;
use tokio::net::TcpStream;
// DIUBAH: Menggunakan RwLock dari Tokio karena digunakan dalam konteks async
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

// DIUBAH: Menggunakan tokio::sync::RwLock
pub type Tx = mpsc::UnboundedSender<Message>;
pub type Clients = Arc<RwLock<Vec<Tx>>>;

pub async fn broadcast_ws_message(clients: &Clients, message_string: String) {
    // DIUBAH: Menambahkan .into() sesuai petunjuk compiler
    let message = Message::Text(message_string.into());

    let clients_guard = clients.read().await;
    for client_tx in clients_guard.iter() {
        // Kirim pesan, abaikan jika ada error (client disconnect)
        let _ = client_tx.send(message.clone());
    }
}

pub async fn handle_websocket_connection(stream: TcpStream, clients: Clients) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        
        // Tambahkan client baru ke daftar
        clients.write().await.push(tx);

        // Task untuk mengirim pesan dari channel ke client
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Loop untuk mendeteksi disconnect
        while read.next().await.is_some() {}

        // Client disconnect, bersihkan daftar
        clients.write().await.retain(|c| !c.is_closed());
        println!("A client disconnected.");
    }
}

// Fungsi TCP tidak berubah
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// pub async fn handle_tcp_connection(mut socket: TcpStream) {
//     let mut buffer = [0; 1024];
//     match socket.read(&mut buffer).await {
//         Ok(n) if n > 0 => {
//             let _ = socket.write_all(b"TCP received").await;
//         }
//         _ => {}
//     }
// }