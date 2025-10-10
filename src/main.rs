mod data;
mod services;
mod controllers;
mod routes;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, http};
use std::sync::Arc;

// DIUBAH: Menggunakan path lengkap `std::sync::RwLock` untuk state sensor.
use std::sync::RwLock;

// DIUBAH: Import semua komponen yang diperlukan untuk arsitektur baru.
use crate::data::gps_data::{SharedGpsConfig, SharedGpsState, GpsConfig};
use crate::data::gyro_data::{SharedGyroConfig, SharedGyroState, GyroConfig};
use crate::services::MqttCommand;
use crate::utils::net::{Clients, handle_websocket_connection, handle_tcp_connection};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting...");

    // === 1. INISIALISASI STATE TERPISAH (4 STATE) ===
    let shared_gps_config: SharedGpsConfig = Arc::new(RwLock::new(GpsConfig::default()));
    let shared_gyro_config: SharedGyroConfig = Arc::new(RwLock::new(GyroConfig::default()));
    let shared_gps_state: SharedGpsState = Arc::new(RwLock::new(None));
    let shared_gyro_state: SharedGyroState = Arc::new(RwLock::new(None));
    
    // Menggunakan `tokio::sync::RwLock` secara eksplisit untuk state websocket.
    let ws_clients: Clients = Arc::new(tokio::sync::RwLock::new(Vec::new()));

    // === 2. PEMBUATAN COMMAND CHANNEL ===
    let (gps_command_tx, gps_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (gyro_command_tx, gyro_command_rx) = mpsc::channel::<MqttCommand>(10);

    // === 3. MENJALANKAN SEMUA SERVICE BACKGROUND ===
    println!("Starting background services...");
    services::gps_service::start_gps_calculation_thread(shared_gps_state.clone());
    services::gps_service::start_gps_publication_thread(
        shared_gps_config.clone(),
        shared_gps_state.clone(),
        ws_clients.clone(), // Clone #1 untuk service GPS
        gps_command_rx,
    );
    
    services::gyro_service::start_gyro_calculation_thread(shared_gyro_state.clone());
    services::gyro_service::start_gyro_publication_thread(
        shared_gyro_config.clone(),
        shared_gyro_state.clone(),
        ws_clients.clone(), // Clone #2 untuk service Gyro
        gyro_command_rx,
    );
    println!("Background services running.");

    // DIUBAH: Buat clone untuk API server SEBELUM blok `move`. Ini menyelesaikan error ownership.
    let ws_clients_for_api = ws_clients.clone();

    // === 4. KONFIGURASI DAN JALANKAN HTTP API SERVER ===
    let api_server = HttpServer::new(move || {
        App::new()
            // Suntikkan semua state dan PENGIRIM perintah ke dalam aplikasi
            .app_data(web::Data::new(shared_gps_config.clone()))
            .app_data(web::Data::new(shared_gps_state.clone()))
            .app_data(web::Data::new(shared_gyro_config.clone()))
            .app_data(web::Data::new(shared_gyro_state.clone()))
            // Gunakan clone yang sudah dibuat khusus untuk API
            .app_data(web::Data::new(ws_clients_for_api.clone())) 
            .app_data(web::Data::new(gps_command_tx.clone()))
            .app_data(web::Data::new(gyro_command_tx.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE])
                    .max_age(3600),
            )
            .configure(routes::gps_routes::init)
            .configure(routes::gyro_routes::init)
    })
    .bind("127.0.0.1:8080")?
    .run();
    
    println!("API Server started on http://127.0.0.1:8080");

    // === 5. JALANKAN SERVER LAIN (WebSocket & TCP) ===
    // `ws_clients` yang asli masih bisa digunakan di sini karena tidak pernah dipindahkan.
    let websocket_listener = TcpListener::bind("127.0.0.1:8081").await?;
    tokio::spawn(async move {
        println!("WebSocket server started on ws://127.0.0.1:8081");
        while let Ok((stream, _)) = websocket_listener.accept().await {
            // Gunakan `ws_clients` yang asli di sini
            tokio::spawn(handle_websocket_connection(stream, ws_clients.clone()));
        }
    });

    let tcp_listener = TcpListener::bind("127.0.0.1:9000").await?;
    tokio::spawn(async move {
        println!("TCP server started on tcp://127.0.0.1:9000");
        while let Ok((socket, _)) = tcp_listener.accept().await {
            tokio::spawn(handle_tcp_connection(socket));
        }
    });

    api_server.await
}