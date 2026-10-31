use crate::data::weather_data::{SharedWeatherConfig, SharedWeatherState};
use crate::utils;
use crate::utils::mqtt_manager::{MqttCommand, MqttManager};
use crate::utils::net::Clients;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::sleep;

const CALCULATION_INTERVAL_MS: u64 = 100;

pub fn start_weather_calculation_thread(state: SharedWeatherState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
        let mut guard = state_clone.write().unwrap();
        if let Some(ref mut weather_state) = *guard {
            if weather_state.is_running {
                utils::weather_calculate::calculate_next_weather_state(weather_state);
            }
        }
    });
}

pub fn start_weather_publication_thread(
    config_state: SharedWeatherConfig,
    data_state: SharedWeatherState,
    ws_clients: Clients,
    mqtt_manager: Arc<MqttManager>,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        loop {
            // Snapshot config
            let (update_rate, topic_prefix) = {
                let cfg = config_state.read().unwrap();
                let ur = cfg.update_rate.unwrap_or(1000);
                let tp = cfg
                    .topics
                    .as_ref()
                    .and_then(|topics| topics.first())
                    .cloned()
                    .unwrap_or_else(|| "vessel/weather".to_string());
                (ur, tp)
            };

            select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            tracing::info!("[Weather Service]: Reconnect requested.");
                        }
                        MqttCommand::Stop => {
                            tracing::info!("[Weather Service]: Stop requested. Exiting publication loop.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(weather_state) = data_opt {
                        if weather_state.is_running {

                            let payload = match serde_json::to_string(&weather_state) {
                                Ok(p) => p,
                                Err(e) => {
                                    eprintln!("[Weather Service]: JSON serialize error: {}", e);
                                    continue;
                                }
                            };

                            let topic = format!("{}/data", topic_prefix);
                            if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&topic), payload.clone()).await {
                                eprintln!("[Weather Service]: MQTT publish error to {}: {:?}", topic, e);
                            }

                            let msg = serde_json::json!({
                                "type": "weather_update",
                                "data": weather_state
                            });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        tracing::info!("[Weather Service]: Publication thread exited.");
    });
}
