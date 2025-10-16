mod data;
mod services;
mod controllers;
mod routes;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, http};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::data::gps_data::{GpsData, GpsConfig};
use crate::data::gyro_data::{GyroData, GyroConfig};
use crate::utils::net::{Clients, handle_websocket_connection};

// DIUBAH: Dibuat `pub` agar bisa diakses dari modul lain di dalam crate.
#[derive(Clone, Debug)]
pub enum ConfigUpdate {
    Gps,
    Gyro,
}

#[derive(Clone)]
pub struct AppState {
    pub gps_data: Arc<RwLock<Option<GpsData>>>,
    pub gyro_data: Arc<RwLock<Option<GyroData>>>,
    pub gps_config: Arc<RwLock<GpsConfig>>,
    pub gyro_config: Arc<RwLock<GyroConfig>>,
    pub ws_clients: Clients,
    pub gps_update_tx: broadcast::Sender<GpsData>,
    pub gyro_update_tx: broadcast::Sender<GyroData>,
    pub config_update_tx: broadcast::Sender<ConfigUpdate>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Server starting...");

    let (gps_update_tx, _) = broadcast::channel::<GpsData>(32);
    let (gyro_update_tx, _) = broadcast::channel::<GyroData>(32);
    let (config_update_tx, _) = broadcast::channel::<ConfigUpdate>(8);

    let app_state = AppState {
        gps_data: Arc::new(RwLock::new(None)),
        gyro_data: Arc::new(RwLock::new(None)),
        gps_config: Arc::new(RwLock::new(GpsConfig::default())),
        gyro_config: Arc::new(RwLock::new(GyroConfig::default())),
        ws_clients: Arc::new(RwLock::new(Vec::new())),
        gps_update_tx,
        gyro_update_tx,
        config_update_tx,
    };

    log::info!("Starting background services...");

    services::gps_service::run_gps_tasks(app_state.clone());
    services::gyro_service::run_gyro_tasks(app_state.clone());

    log::info!("Background services running.");

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

    log::info!("API Server started on http://127.0.0.1:8080");
    
    let ws_state = app_state.clone();
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8081").await.unwrap();
        log::info!("WebSocket server started on ws://127.0.0.1:8081");
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(handle_websocket_connection(stream, ws_state.ws_clients.clone()));
            }
        }
    });

    api_server.await
}

