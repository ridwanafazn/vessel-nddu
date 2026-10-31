mod config;
mod controllers;
mod data;
mod docs;
mod routes;
mod services;
mod telemetry;
mod utils;

use crate::config::AppConfig;
use crate::data::compass_data::{SharedCompassConfig, SharedCompassState};
use crate::data::gps_data::{GpsConfig, SharedGpsConfig, SharedGpsState};
use crate::data::gyro_data::{GyroConfig, SharedGyroConfig, SharedGyroState};
use crate::data::navigation_data::{
    NavigationConfig, SharedDepthState, SharedNavigationConfig, SharedSpeedState,
};
use crate::data::weather_data::{
    SharedHumidityState, SharedPressureState, SharedTemperatureState, SharedWeatherConfig,
    SharedWeatherState, SharedWindState, WeatherConfig,
};
use crate::utils::mqtt_manager::{MqttCommand, MqttManager};
use crate::utils::net::{handle_tcp_connection, handle_websocket_connection, Clients};
use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    let shared_weather_config: SharedWeatherConfig =
        Arc::new(RwLock::new(WeatherConfig::default()));
    let shared_navigation_config: SharedNavigationConfig =
        Arc::new(RwLock::new(NavigationConfig::default()));
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
    let shutdown = CancellationToken::new();

    tracing::info!("starting background services");

    // Jalankan kalkulasi + publikasi
    services::gps_service::start_gps_calculation_thread(shared_gps_state.clone());
    services::gps_service::start_gps_publication_thread(
        shared_gps_config.clone(),
        shared_gps_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        gps_command_rx,
        shutdown.clone(),
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
        shutdown.clone(),
    ));

    services::gyro_service::start_gyro_calculation_thread(shared_gyro_state.clone());
    services::gyro_service::start_gyro_publication_thread(
        shared_gyro_config.clone(),
        shared_gyro_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        gyro_command_rx,
        shutdown.clone(),
    );

    services::weather_service::start_weather_calculation_thread(shared_weather_state.clone());
    services::weather_service::start_weather_publication_thread(
        shared_weather_config.clone(),
        shared_weather_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        weather_command_rx,
        shutdown.clone(),
    );

    // Individual weather sensor services
    services::temperature_service::start_temperature_calculation_thread(
        shared_temperature_state.clone(),
    );
    services::temperature_service::start_temperature_publication_thread(
        shared_weather_config.clone(),
        shared_temperature_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        temperature_command_rx,
        shutdown.clone(),
    );

    services::humidity_service::start_humidity_calculation_thread(shared_humidity_state.clone());
    services::humidity_service::start_humidity_publication_thread(
        shared_weather_config.clone(),
        shared_humidity_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        humidity_command_rx,
        shutdown.clone(),
    );

    services::wind_service::start_wind_calculation_thread(shared_wind_state.clone());
    services::wind_service::start_wind_publication_thread(
        shared_weather_config.clone(),
        shared_wind_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        wind_command_rx,
        shutdown.clone(),
    );

    services::pressure_service::start_pressure_calculation_thread(shared_pressure_state.clone());
    services::pressure_service::start_pressure_publication_thread(
        shared_weather_config.clone(),
        shared_pressure_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        pressure_command_rx,
        shutdown.clone(),
    );

    services::speed_service::start_speed_calculation_thread(shared_speed_state.clone());
    services::speed_service::start_speed_publication_thread(
        shared_navigation_config.clone(),
        shared_speed_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        speed_command_rx,
        shutdown.clone(),
    );

    services::depth_service::start_depth_calculation_thread(shared_depth_state.clone());
    services::depth_service::start_depth_publication_thread(
        shared_navigation_config.clone(),
        shared_depth_state.clone(),
        ws_clients.clone(),
        mqtt_manager.clone(),
        depth_command_rx,
        shutdown.clone(),
    );

    services::compass_service::start_compass_calculation_thread(
        shared_compass_state.clone(),
        compass_command_rx,
        shutdown.clone(),
    );
    services::compass_service::start_compass_publication_thread(
        shared_compass_config.clone(),
        shared_compass_state.clone(),
        mqtt_manager.clone(),
        ws_clients.clone(),
        shutdown.clone(),
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
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", docs::openapi::ApiDoc::openapi()),
            )
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::CONTENT_TYPE,
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
    let ws_shutdown = shutdown.clone();
    tokio::spawn(async move {
        tracing::info!(addr = %ws_addr, "WebSocket server started");
        loop {
            tokio::select! {
                _ = ws_shutdown.cancelled() => break,
                accepted = websocket_listener.accept() => {
                    match accepted {
                        Ok((stream, _)) => {
                            tokio::spawn(handle_websocket_connection(stream, ws_clients.clone()));
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "WebSocket accept error");
                            break;
                        }
                    }
                }
            }
        }
        tracing::info!("WebSocket server stopped");
    });

    // TCP
    let tcp_listener = TcpListener::bind(tcp_addr).await?;
    let tcp_shutdown = shutdown.clone();
    tokio::spawn(async move {
        tracing::info!(addr = %tcp_addr, "TCP server started");
        loop {
            tokio::select! {
                _ = tcp_shutdown.cancelled() => break,
                accepted = tcp_listener.accept() => {
                    match accepted {
                        Ok((socket, _)) => {
                            tokio::spawn(handle_tcp_connection(socket));
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "TCP accept error");
                            break;
                        }
                    }
                }
            }
        }
        tracing::info!("TCP server stopped");
    });

    let server_handle = api_server.handle();
    tokio::select! {
        result = api_server => result?,
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("shutdown signal received; stopping gracefully");
            shutdown.cancel();
            server_handle.stop(true).await;
        }
    }

    Ok(())
}
