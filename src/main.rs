mod data;
mod services;
mod controllers;
mod routes;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, http};
use std::sync::{Arc, RwLock};
use crate::data::gps_data::{SharedGpsConfig, SharedGpsState, GpsConfig};
use crate::data::gyro_data::{SharedGyroConfig, SharedGyroState, GyroConfig};
use crate::utils::mqtt_manager::{MqttCommand, MqttManager};
use crate::utils::net::{Clients, handle_websocket_connection, handle_tcp_connection};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use rumqttc::{AsyncClient, MqttOptions};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Server starting...");

    // Shared states
    let shared_gps_config: SharedGpsConfig = Arc::new(RwLock::new(GpsConfig::default()));
    let shared_gyro_config: SharedGyroConfig = Arc::new(RwLock::new(GyroConfig::default()));
    let shared_gps_state: SharedGpsState = Arc::new(RwLock::new(None));
    let shared_gyro_state: SharedGyroState = Arc::new(RwLock::new(None));
    let ws_clients: Clients = Arc::new(tokio::sync::RwLock::new(Vec::new()));

    // Channels
    let (gps_command_tx, gps_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (gyro_command_tx, gyro_command_rx) = mpsc::channel::<MqttCommand>(10);

    // MQTT Client (dihubungkan dari config)
    let mut mqttoptions = MqttOptions::new("vessel-client", "127.0.0.1", 1883);
    mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));
    let (client, _eventloop) = AsyncClient::new(mqttoptions, 10);
    let mqtt_manager = Arc::new(MqttManager::new(client));

    println!("🧠 Starting background services...");

    // Jalankan kalkulasi + publikasi
    services::gps_service::start_gps_calculation_thread(shared_gps_state.clone());
    services::gps_service::start_gps_publication_thread(
        shared_gps_config.clone(),
        shared_gps_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        gps_command_rx,
    );

    services::gyro_service::start_gyro_calculation_thread(shared_gyro_state.clone());
    services::gyro_service::start_gyro_publication_thread(
        shared_gyro_config.clone(),
        shared_gyro_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        gyro_command_rx,
    );

    println!("✅ Background services running.");

    // API server
    let ws_clients_for_api = ws_clients.clone();
    let mqtt_manager_for_api = mqtt_manager.clone();

    let api_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_gps_config.clone()))
            .app_data(web::Data::new(shared_gps_state.clone()))
            .app_data(web::Data::new(shared_gyro_config.clone()))
            .app_data(web::Data::new(shared_gyro_state.clone()))
            .app_data(web::Data::new(ws_clients_for_api.clone()))
            .app_data(web::Data::new(gps_command_tx.clone()))
            .app_data(web::Data::new(gyro_command_tx.clone()))
            .app_data(web::Data::new(mqtt_manager_for_api.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::CONTENT_TYPE
                    ])
                    .max_age(3600),
            )
            .configure(routes::gps_routes::init)
            .configure(routes::gyro_routes::init)
    })
    .bind("127.0.0.1:8080")?
    .run();

    println!("🌐 API Server started on http://127.0.0.1:8080");

    // WebSocket
    let websocket_listener = TcpListener::bind("127.0.0.1:8081").await?;
    tokio::spawn(async move {
        println!("🔌 WebSocket server started on ws://127.0.0.1:8081");
        while let Ok((stream, _)) = websocket_listener.accept().await {
            tokio::spawn(handle_websocket_connection(stream, ws_clients.clone()));
        }
    });

    // TCP
    let tcp_listener = TcpListener::bind("127.0.0.1:9000").await?;
    tokio::spawn(async move {
        println!("📡 TCP server started on tcp://127.0.0.1:9000");
        while let Ok((socket, _)) = tcp_listener.accept().await {
            tokio::spawn(handle_tcp_connection(socket));
        }
    });

    api_server.await
}
