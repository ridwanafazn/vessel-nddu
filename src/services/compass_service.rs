use crate::data::compass_data::{SharedCompassConfig, SharedCompassState, COMPASS_DEVIATION, COMPASS_VARIATION};
use crate::utils::compass_calculate::calculate_compass;
use crate::utils::mqtt_manager::{MqttCommand, MqttManager};
use crate::utils;
use crate::utils::net::Clients;
use chrono::Local;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub fn start_compass_calculation_thread(
    state: SharedCompassState,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = sleep(Duration::from_millis(100)) => {
                    let mut guard = state.write().unwrap();
                    if let Some(ref mut compass) = *guard {
                        if compass.is_running {
                            calculate_compass(compass);
                            compass.last_update = Local::now().format("%d %b %Y, %H:%M:%S").to_string();
                        }
                    }
                }
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            println!("[Compass Calc] Received reconnect command");
                        }
                        MqttCommand::Stop => {
                            println!("[Compass Calc] Stop requested");
                            break;
                        }
                    }
                }
            }
        }
    });
}

pub fn start_compass_publication_thread(
    config: SharedCompassConfig,
    state: SharedCompassState,
    mqtt_manager: Arc<MqttManager>,
    ws_clients: Clients,
) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(1000)).await;

            let (cfg_opt, compass_opt) = {
                let cfg_guard = config.read().unwrap();
                let state_guard = state.read().unwrap();
                (cfg_guard.clone(), state_guard.clone())
            };

            if let (Some(_cfg), Some(compass_state)) = (cfg_opt, compass_opt) {
                if compass_state.is_running {
                    let payload = serde_json::json!({
                        "heading": compass_state.heading,
                        "deviation": COMPASS_DEVIATION,
                        "variation": COMPASS_VARIATION,
                        "last_update": compass_state.last_update,
                    });

                    // Publish to MQTT
                    let topic = "vessel/compass/heading";
                    if let Err(e) = mqtt_manager.publish_message(&[topic.to_string()], payload.to_string()).await {
                        eprintln!("[Compass Pub] MQTT publish error: {}", e);
                    }

                    // Broadcast to WebSocket clients
                    let msg = serde_json::json!({ "type": "compass_update", "data": payload });
                    let json = msg.to_string();
                    utils::net::broadcast_ws_message(&ws_clients, json).await;
                }
            }
        }
    });
}
