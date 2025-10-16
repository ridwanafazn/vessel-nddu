use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

pub type Tx = mpsc::UnboundedSender<Message>;
pub type Clients = Arc<RwLock<Vec<Tx>>>;

pub async fn broadcast_ws_message(clients: &Clients, message_string: String) {
    let message = Message::Text(message_string.into());

    let clients_guard = clients.read().await;
    for client_tx in clients_guard.iter() {
        let _ = client_tx.send(message.clone());
    }
}

pub async fn handle_websocket_connection(stream: TcpStream, clients: Clients) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        
        clients.write().await.push(tx);

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            }
        });

        while read.next().await.is_some() {}
        clients.write().await.retain(|c| !c.is_closed());
        println!("A client disconnected.");
    }
}

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