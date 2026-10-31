use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};
use crate::utils::{handle_websocket_connection, Clients};

pub async fn start_websocket_server(clients: Clients) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8081").await?;
    println!("WebSocket server running on ws://localhost:8081");

    while let Ok((stream, _)) = listener.accept().await {
        let clients_clone = clients.clone();
        tokio::spawn(handle_websocket_connection(stream, clients_clone));
    }

    Ok(())
}
