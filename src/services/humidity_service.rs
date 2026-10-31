use crate::data::weather_data::{SharedHumidityState, SharedWeatherConfig};
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

/// Thread perhitungan Humidity (lokal, non-async)
pub fn start_humidity_calculation_thread(state: SharedHumidityState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
            let mut guard = state_clone.write().unwrap();
            if let Some(ref mut humidity_state) = *guard {
                if humidity_state.is_running {
                    // Simple simulation: vary humidity slightly
                    humidity_state.air_humidity += (rand::random::<f64>() - 0.5) * 2.0;
                    humidity_state.air_humidity = humidity_state.air_humidity.clamp(0.0, 100.0);
                    humidity_state.last_update = chrono::Utc::now();
                }
            }
        }
    });
}

/// Thread publikasi Humidity ke MQTT + WebSocket
pub fn start_humidity_publication_thread(
    config_state: SharedWeatherConfig,
    data_state: SharedHumidityState,
    ws_clients: Clients,
    mqtt_manager: Arc<MqttManager>,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        loop {
            let (update_rate, topic_prefix) = {
                let cfg = config_state.read().unwrap();
                let ur = cfg.update_rate.unwrap_or(1000);
                let tp = cfg
                    .topics
                    .as_ref()
                    .and_then(|v| v.first())
                    .cloned()
                    .unwrap_or_else(|| "vessel/weather".to_string());
                (ur, tp)
            };

            select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            tracing::info!("[Humidity Service]: Reconnect requested.");
                        }
                        MqttCommand::Stop => {
                            tracing::info!("[Humidity Service]: Stop requested. Exiting publication loop.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(humidity_state) = data_opt {
                        if humidity_state.is_running {
                            let payload = match serde_json::to_string(&humidity_state) {
                                Ok(p) => p,
                                Err(e) => { eprintln!("[Humidity Service]: JSON serialize error: {}", e); continue; }
                            };
                            let topic = format!("{}/humidity", topic_prefix);

                            if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&topic), payload.clone()).await {
                                eprintln!("[Humidity Service]: MQTT publish error to {}: {:?}", topic, e);
                            }

                            let msg = serde_json::json!({ "type": "humidity_update", "data": humidity_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        tracing::info!("[Humidity Service]: Publication thread exited.");
    });
}
