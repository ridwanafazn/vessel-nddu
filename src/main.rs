mod data;
mod services;
mod controllers;
mod routes;
mod utils;
mod config;
mod telemetry;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, http};
use std::sync::{Arc, RwLock};
use crate::config::AppConfig;
use crate::data::gps_data::{SharedGpsConfig, SharedGpsState, GpsConfig};
use crate::data::gyro_data::{SharedGyroConfig, SharedGyroState, GyroConfig};
use crate::data::weather_data::{
    SharedWeatherConfig, SharedWeatherState, WeatherConfig,
    SharedTemperatureState, SharedHumidityState, SharedWindState, SharedPressureState
};
use crate::data::navigation_data::{SharedNavigationConfig, SharedSpeedState, SharedDepthState, NavigationConfig};
use crate::data::compass_data::{SharedCompassConfig, SharedCompassState};
use crate::utils::mqtt_manager::{MqttCommand, MqttManager};
use crate::utils::net::{Clients, handle_websocket_connection, handle_tcp_connection};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app_config = AppConfig::from_env();
    telemetry::init(&app_config.log_level);

    let http_addr = app_config.http_addr;
    let ws_addr = app_config.ws_addr;
    let tcp_addr = app_config.tcp_addr;

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        http = %http_addr,
        ws = %ws_addr,
        tcp = %tcp_addr,
        mqtt = %format!("{}:{}", app_config.mqtt.host, app_config.mqtt.port),
        "vessel backend starting"
    );

    // Shared states
    let shared_gps_config: SharedGpsConfig = Arc::new(RwLock::new(GpsConfig::default()));
    let shared_gyro_config: SharedGyroConfig = Arc::new(RwLock::new(GyroConfig::default()));
    let shared_weather_config: SharedWeatherConfig = Arc::new(RwLock::new(WeatherConfig::default()));
    let shared_navigation_config: SharedNavigationConfig = Arc::new(RwLock::new(NavigationConfig::default()));
    let shared_compass_config: SharedCompassConfig = Arc::new(RwLock::new(None));
    let shared_gps_state: SharedGpsState = Arc::new(RwLock::new(None));
    let shared_gyro_state: SharedGyroState = Arc::new(RwLock::new(None));
    let shared_weather_state: SharedWeatherState = Arc::new(RwLock::new(None));
    
    // Individual weather sensor states
    let shared_temperature_state: SharedTemperatureState = Arc::new(RwLock::new(None));
    let shared_humidity_state: SharedHumidityState = Arc::new(RwLock::new(None));
    let shared_wind_state: SharedWindState = Arc::new(RwLock::new(None));
    let shared_pressure_state: SharedPressureState = Arc::new(RwLock::new(None));
    
    // Navigation sensor states
    let shared_speed_state: SharedSpeedState = Arc::new(RwLock::new(None));
    let shared_depth_state: SharedDepthState = Arc::new(RwLock::new(None));
    
    // Compass sensor state
    let shared_compass_state: SharedCompassState = Arc::new(RwLock::new(None));
    let ws_clients: Clients = Arc::new(tokio::sync::RwLock::new(Vec::new()));

    // Channels
    let (gps_command_tx, gps_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (gyro_command_tx, gyro_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (weather_command_tx, weather_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (temperature_command_tx, temperature_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (humidity_command_tx, humidity_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (wind_command_tx, wind_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (pressure_command_tx, pressure_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (speed_command_tx, speed_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (depth_command_tx, depth_command_rx) = mpsc::channel::<MqttCommand>(10);
    let (compass_command_tx, compass_command_rx) = mpsc::channel::<MqttCommand>(10);

    // MQTT global (fail-safe, bisa diubah runtime lewat /api/mqtt/config)
    let mqtt_manager = Arc::new(MqttManager::spawn(app_config.mqtt.clone()));

    tracing::info!("starting background services");

    // Jalankan kalkulasi + publikasi
    services::gps_service::start_gps_calculation_thread(shared_gps_state.clone());
    services::gps_service::start_gps_publication_thread(
        shared_gps_config.clone(),
        shared_gps_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        gps_command_rx,
    );

    // Start GPS TCP client for streaming NMEA data to remote target
    tokio::spawn(services::tcp_client::start_tcp_client(
        shared_gps_state.clone(),
        shared_gyro_state.clone(),
        shared_wind_state.clone(),
        shared_temperature_state.clone(),
        shared_speed_state.clone(),
        shared_depth_state.clone(),
        shared_gps_config.clone(),
    ));

    services::gyro_service::start_gyro_calculation_thread(shared_gyro_state.clone());
    services::gyro_service::start_gyro_publication_thread(
        shared_gyro_config.clone(),
        shared_gyro_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        gyro_command_rx,
    );

    services::weather_service::start_weather_calculation_thread(shared_weather_state.clone());
    services::weather_service::start_weather_publication_thread(
        shared_weather_config.clone(),
        shared_weather_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        weather_command_rx,
    );

    // Individual weather sensor services
    services::temperature_service::start_temperature_calculation_thread(shared_temperature_state.clone());
    services::temperature_service::start_temperature_publication_thread(
        shared_weather_config.clone(),
        shared_temperature_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        temperature_command_rx,
    );

    services::humidity_service::start_humidity_calculation_thread(shared_humidity_state.clone());
    services::humidity_service::start_humidity_publication_thread(
        shared_weather_config.clone(),
        shared_humidity_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        humidity_command_rx,
    );

    services::wind_service::start_wind_calculation_thread(shared_wind_state.clone());
    services::wind_service::start_wind_publication_thread(
        shared_weather_config.clone(),
        shared_wind_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        wind_command_rx,
    );

    services::pressure_service::start_pressure_calculation_thread(shared_pressure_state.clone());
    services::pressure_service::start_pressure_publication_thread(
        shared_weather_config.clone(),
        shared_pressure_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        pressure_command_rx,
    );

    services::speed_service::start_speed_calculation_thread(shared_speed_state.clone());
    services::speed_service::start_speed_publication_thread(
        shared_navigation_config.clone(),
        shared_speed_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        speed_command_rx,
    );

    services::depth_service::start_depth_calculation_thread(shared_depth_state.clone());
    services::depth_service::start_depth_publication_thread(
        shared_navigation_config.clone(),
        shared_depth_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        depth_command_rx,
    );

    services::compass_service::start_compass_calculation_thread(
        shared_compass_state.clone(),
        compass_command_rx,
    );
    services::compass_service::start_compass_publication_thread(
        shared_compass_config.clone(),
        shared_compass_state.clone(),
        mqtt_manager.clone(),
        ws_clients.clone(),
    );

    tracing::info!("background services running");

    // API server
    let ws_clients_for_api = ws_clients.clone();
    let mqtt_manager_for_api = mqtt_manager.clone();

    let api_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_gps_config.clone()))
            .app_data(web::Data::new(shared_gps_state.clone()))
            .app_data(web::Data::new(shared_gyro_config.clone()))
            .app_data(web::Data::new(shared_gyro_state.clone()))
            .app_data(web::Data::new(shared_weather_config.clone()))
            .app_data(web::Data::new(shared_weather_state.clone()))
            .app_data(web::Data::new(shared_temperature_state.clone()))
            .app_data(web::Data::new(shared_humidity_state.clone()))
            .app_data(web::Data::new(shared_wind_state.clone()))
            .app_data(web::Data::new(shared_pressure_state.clone()))
            .app_data(web::Data::new(shared_navigation_config.clone()))
            .app_data(web::Data::new(shared_speed_state.clone()))
            .app_data(web::Data::new(shared_depth_state.clone()))
            .app_data(web::Data::new(shared_compass_config.clone()))
            .app_data(web::Data::new(shared_compass_state.clone()))
            .app_data(web::Data::new(ws_clients_for_api.clone()))
            .app_data(web::Data::new(gps_command_tx.clone()))
            .app_data(web::Data::new(gyro_command_tx.clone()))
            .app_data(web::Data::new(weather_command_tx.clone()))
            .app_data(web::Data::new(temperature_command_tx.clone()))
            .app_data(web::Data::new(humidity_command_tx.clone()))
            .app_data(web::Data::new(wind_command_tx.clone()))
            .app_data(web::Data::new(pressure_command_tx.clone()))
            .app_data(web::Data::new(speed_command_tx.clone()))
            .app_data(web::Data::new(depth_command_tx.clone()))
            .app_data(web::Data::new(compass_command_tx.clone()))
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
            .configure(routes::weather_routes::init)
            .configure(routes::temperature_routes::init)
            .configure(routes::humidity_routes::init)
            .configure(routes::wind_routes::init)
            .configure(routes::pressure_routes::init)
            .configure(routes::navigation_routes::init)
            .configure(routes::speed_routes::init)
            .configure(routes::depth_routes::init)
            .configure(routes::compass_routes::init)
            .configure(routes::auth_routes::init)
            .configure(routes::mqtt_routes::init)

    })
    .bind(http_addr)?
    .run();

    tracing::info!(addr = %http_addr, "API server started");

    // WebSocket
    let websocket_listener = TcpListener::bind(ws_addr).await?;
    tokio::spawn(async move {
        tracing::info!(addr = %ws_addr, "WebSocket server started");
        while let Ok((stream, _)) = websocket_listener.accept().await {
            tokio::spawn(handle_websocket_connection(stream, ws_clients.clone()));
        }
    });

    // TCP
    let tcp_listener = TcpListener::bind(tcp_addr).await?;
    tokio::spawn(async move {
        tracing::info!(addr = %tcp_addr, "TCP server started");
        while let Ok((socket, _)) = tcp_listener.accept().await {
            tokio::spawn(handle_tcp_connection(socket));
        }
    });

    api_server.await
}
