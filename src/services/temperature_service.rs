use crate::data::weather_data::{SharedWeatherConfig, SharedTemperatureState};
use crate::utils::mqtt_manager::{MqttManager, MqttCommand};
use crate::utils;
use crate::utils::net::Clients;
use std::sync::Arc;
use std::time::Duration;
use std::thread;
use tokio::sync::mpsc;
use tokio::select;
use tokio::time::sleep;

const CALCULATION_INTERVAL_MS: u64 = 100;

/// Thread perhitungan Temperature (lokal, non-async)
pub fn start_temperature_calculation_thread(state: SharedTemperatureState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
            let mut guard = state_clone.write().unwrap();
            if let Some(ref mut temp_state) = *guard {
                if temp_state.is_running {
                    // Simple simulation: vary temperature slightly
                    temp_state.air_temperature += (rand::random::<f64>() - 0.5) * 0.5;
                    temp_state.last_update = chrono::Utc::now();
                }
            }
        }
    });
}

/// Thread publikasi Temperature ke MQTT + WebSocket
pub fn start_temperature_publication_thread(
    config_state: SharedWeatherConfig,
    data_state: SharedTemperatureState,
    ws_clients: Clients,
    mqtt_manager: Arc<MqttManager>,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        loop {
            let (update_rate, topic_prefix) = {
                let cfg = config_state.read().unwrap();
                let ur = cfg.update_rate.unwrap_or(1000);
                let tp = cfg.topics.as_ref().and_then(|v| v.first()).cloned().unwrap_or_else(|| "vessel/weather".to_string());
                (ur, tp)
            };
            
            select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            tracing::info!("[Temperature Service]: Reconnect requested.");
                        }
                        MqttCommand::Stop => {
                            tracing::info!("[Temperature Service]: Stop requested. Exiting publication loop.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(temp_state) = data_opt {
                        if temp_state.is_running {
                            let payload = match serde_json::to_string(&temp_state) {
                                Ok(p) => p,
                                Err(e) => { eprintln!("[Temperature Service]: JSON serialize error: {}", e); continue; }
                            };
                            let topic = format!("{}/temperature", topic_prefix);

                            if let Err(e) = mqtt_manager.publish_message(&[topic.clone()], payload.clone()).await {
                                eprintln!("[Temperature Service]: MQTT publish error to {}: {:?}", topic, e);
                            }

                            // Helper: check if sentence enabled
                            let is_sentence_enabled = |sentence_type: &str| -> bool {
                                let config_guard = config_state.read().unwrap();
                                match &config_guard.enabled_sentences {
                                    None => true,
                                    Some(sentences) if sentences.is_empty() => true,
                                    Some(sentences) => sentences.iter().any(|s| s.eq_ignore_ascii_case(sentence_type)),
                                }
                            };

                            // Conditionally publish WIMTW NMEA sentence
                            if is_sentence_enabled("WIMTW") {
                                let wimtw_sentence = temp_state.to_wimtw();
                                let wimtw_topic = format!("{}/wimtw", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(&[wimtw_topic.clone()], wimtw_sentence.clone()).await {
                                    eprintln!("[Temperature Service]: MQTT publish error to {}: {:?}", wimtw_topic, e);
                                }
                            }

                            let msg = serde_json::json!({ "type": "temperature_update", "data": temp_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        tracing::info!("[Temperature Service]: Publication thread exited.");
    });
}
