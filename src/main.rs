mod data;
mod services;
mod controllers;
mod routes;
mod utils;

use tokio::sync::Mutex;
use std::time::Duration;
use actix_web::{App, HttpServer, web};
use actix_cors::Cors;
use actix_web::http;
use std::sync::{Arc};
use tokio::net::TcpListener;
use crate::utils::{handle_websocket_connection, handle_tcp_connection, Clients};
use crate::services::gps_service::{GPSStore, start_gps_stream};
use rumqttc::{AsyncClient, MqttOptions};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let gps_store: GPSStore = Arc::new(Mutex::new(None));
    let gps_store_data = web::Data::new(gps_store.clone());

    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    let clients_data = web::Data::new(clients.clone());

    println!("Server starting...");

    let mut mqttoptions = MqttOptions::new("vessel_client", "127.0.0.1", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    // Mulai GPS streaming periodik (tanpa MQTT client untuk sementara)
    let (mqtt_client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    tokio::spawn(async move { loop { let _ = eventloop.poll().await; } });
    start_gps_stream(gps_store.clone(), clients.clone(), Some(mqtt_client.clone()));

    // HTTP API server
    let api_server = HttpServer::new({
        let gps_store = gps_store_data.clone();
        let clients = clients_data.clone();
        move || {
            App::new()
                .app_data(gps_store.clone())
                .app_data(clients.clone())
                .wrap(
                    Cors::default()
                        .allow_any_origin()
                        .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
                        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                        .allowed_header(http::header::CONTENT_TYPE)
                        .max_age(3600),
                )
                .configure(|cfg| routes::gps_routes::init(cfg))
        }
    })
    .bind("127.0.0.1:8080")?
    .run();
    println!("API Server started on http://localhost:8080");

    // WebSocket server
    let websocket_listener = TcpListener::bind("127.0.0.1:8081").await?;
    let ws_clients = clients.clone();
    tokio::spawn(async move {
        println!("WebSocket server started on ws://localhost:8081");
        while let Ok((stream, _)) = websocket_listener.accept().await {
            let clients_clone = ws_clients.clone();
            tokio::spawn(handle_websocket_connection(stream, clients_clone));
        }
    });

    // TCP server
    let tcp_listener = TcpListener::bind("127.0.0.1:9000").await?;
    tokio::spawn(async move {
        println!("TCP server started on tcp://localhost:9000");
        while let Ok((socket, _)) = tcp_listener.accept().await {
            tokio::spawn(handle_tcp_connection(socket));
        }
    });

    // Menjalankan API server (blocking future)
    api_server.await
}