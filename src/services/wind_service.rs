use crate::data::weather_data::{SharedWeatherConfig, SharedWindState};
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

/// Thread perhitungan Wind (lokal, non-async)
pub fn start_wind_calculation_thread(state: SharedWindState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
            let mut guard = state_clone.write().unwrap();
            if let Some(ref mut wind_state) = *guard {
                if wind_state.is_running {
                    // Simple simulation: vary wind slightly
                    wind_state.wind_velocity += (rand::random::<f64>() - 0.5) * 2.0;
                    if wind_state.wind_velocity < 0.0 {
                        wind_state.wind_velocity = 0.0;
                    }

                    wind_state.wind_direction_relative += (rand::random::<f64>() - 0.5) * 10.0;
                    // Normalize to 0-360 degrees
                    if wind_state.wind_direction_relative < 0.0 {
                        wind_state.wind_direction_relative += 360.0;
                    } else if wind_state.wind_direction_relative >= 360.0 {
                        wind_state.wind_direction_relative -= 360.0;
                    }

                    wind_state.last_update = chrono::Utc::now();
                }
            }
        }
    });
}

/// Thread publikasi Wind ke MQTT + WebSocket
pub fn start_wind_publication_thread(
    config_state: SharedWeatherConfig,
    data_state: SharedWindState,
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
                            tracing::info!("[Wind Service]: Reconnect requested.");
                        }
                        MqttCommand::Stop => {
                            tracing::info!("[Wind Service]: Stop requested. Exiting publication loop.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(wind_state) = data_opt {
                        if wind_state.is_running {
                            let payload = match serde_json::to_string(&wind_state) {
                                Ok(p) => p,
                                Err(e) => { eprintln!("[Wind Service]: JSON serialize error: {}", e); continue; }
                            };
                            let topic = format!("{}/wind", topic_prefix);

                            if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&topic), payload.clone()).await {
                                eprintln!("[Wind Service]: MQTT publish error to {}: {:?}", topic, e);
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

                            // Conditionally publish WIMWV NMEA sentence
                            if is_sentence_enabled("WIMWV") {
                                let wimwv_sentence = wind_state.to_wimwv();
                                let wimwv_topic = format!("{}/wimwv", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&wimwv_topic), wimwv_sentence.clone()).await {
                                    eprintln!("[Wind Service]: MQTT publish error to {}: {:?}", wimwv_topic, e);
                                }
                            }

                            // Conditionally publish WIMWD NMEA sentence
                            if is_sentence_enabled("WIMWD") {
                                let wimwd_sentence = wind_state.to_wimwd();
                                let wimwd_topic = format!("{}/wimwd", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&wimwd_topic), wimwd_sentence.clone()).await {
                                    eprintln!("[Wind Service]: MQTT publish error to {}: {:?}", wimwd_topic, e);
                                }
                            }

                            let msg = serde_json::json!({ "type": "wind_update", "data": wind_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        tracing::info!("[Wind Service]: Publication thread exited.");
    });
}
