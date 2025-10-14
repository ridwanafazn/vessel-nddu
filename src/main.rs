mod data;
mod services;
mod controllers;
mod routes;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, http};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock}; // DIUBAH: Menggunakan RwLock dari Tokio secara konsisten
use rumqttc::{AsyncClient, MqttOptions};

// Impor struct data dan config Anda
use crate::data::gps_data::{GpsData, GpsConfig};
use crate::data::gyro_data::{GyroData, GyroConfig};
use crate::utils::mqtt_manager::MqttManager; // DIUBAH: Ganti nama file mqtt.rs -> mqtt_manager.rs
use crate::utils::net::{Clients, handle_websocket_connection};

// BARU: AppState terpusat untuk menampung semua state yang dibagikan.
// Ini adalah "Single Source of Truth" untuk aplikasi Anda.
#[derive(Clone)]
pub struct AppState {
    pub gps_data: Arc<RwLock<Option<GpsData>>>,
    pub gyro_data: Arc<RwLock<Option<GyroData>>>,
    pub gps_config: Arc<RwLock<GpsConfig>>,
    pub gyro_config: Arc<RwLock<GyroConfig>>,
    pub ws_clients: Clients,
    pub mqtt_manager: Arc<MqttManager>,
    // Channel untuk "berteriak" ketika ada data baru. Ini adalah kunci arsitektur baru.
    pub gps_update_tx: broadcast::Sender<GpsData>,
    pub gyro_update_tx: broadcast::Sender<GyroData>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Inisialisasi logger sangat disarankan untuk debugging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("🚀 Server starting...");

    // BARU: Setup broadcast channels untuk komunikasi antar-task.
    let (gps_update_tx, _) = broadcast::channel::<GpsData>(32);
    let (gyro_update_tx, _) = broadcast::channel::<GyroData>(32);

    // BARU: Setup SATU KONEKSI MQTT TERPUSAT.
    // Konfigurasi awal bisa dari env var atau file. Akan di-override oleh API.
    let mut mqttoptions = MqttOptions::new("vessel-nddu-client", "127.0.0.1", 1883);
    mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // PENTING: EventLoop MQTT harus di-poll agar koneksi tetap hidup dan bisa reconnect otomatis.
    tokio::spawn(async move {
        loop {
            // eventloop.poll() menangani PING dan reconnect secara otomatis.
            if let Err(e) = eventloop.poll().await {
                log::error!("[MQTT EventLoop] Connection error: {}. Reconnecting...", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    });

    // BARU: Inisialisasi AppState terpusat
    let app_state = AppState {
        gps_data: Arc::new(RwLock::new(None)),
        gyro_data: Arc::new(RwLock::new(None)),
        gps_config: Arc::new(RwLock::new(GpsConfig::default())),
        gyro_config: Arc::new(RwLock::new(GyroConfig::default())),
        ws_clients: Arc::new(RwLock::new(Vec::new())),
        mqtt_manager: Arc::new(MqttManager::new(client)), // DIUBAH: Pass client ke MqttManager
        gps_update_tx,
        gyro_update_tx,
    };

    log::info!("🧠 Starting background services...");

    // BARU: Jalankan semua background task dari satu tempat dengan arsitektur baru
    services::gps_service::run_gps_tasks(app_state.clone());
    services::gyro_service::run_gyro_tasks(app_state.clone());

    log::info!("✅ Background services running.");

    // DIUBAH: Konfigurasi server API menjadi lebih bersih, hanya inject satu AppState
    let api_server_state = app_state.clone();
    let api_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(api_server_state.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
                    .allowed_headers(vec![http::header::CONTENT_TYPE])
                    .max_age(3600),
            )
            .configure(routes::gps_routes::init)
            .configure(routes::gyro_routes::init)
    })
    .bind("127.0.0.1:8080")?
    .run();

    log::info!("🌐 API Server started on http://127.0.0.1:8080");
    
    // DIUBAH: Server lain juga menggunakan AppState
    let ws_state = app_state.clone();
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8081").await.unwrap();
        log::info!("🔌 WebSocket server started on ws://127.0.0.1:8081");
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(handle_websocket_connection(stream, ws_state.ws_clients.clone()));
            }
        }
    });

    // Hapus server TCP jika tidak digunakan untuk menyederhanakan kode.
    
    api_server.await
}