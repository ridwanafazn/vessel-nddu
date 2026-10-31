use crate::data::navigation_data::{SharedNavigationConfig, SharedSpeedState};
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

/// Thread perhitungan Speed (lokal, non-async)
pub fn start_speed_calculation_thread(state: SharedSpeedState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
            let mut guard = state_clone.write().unwrap();
            if let Some(ref mut speed_state) = *guard {
                if speed_state.is_running {
                    utils::navigation_calculate::calculate_next_speed_state(speed_state);
                }
            }
        }
    });
}

/// Thread publikasi Speed ke MQTT + WebSocket
pub fn start_speed_publication_thread(
    config_state: SharedNavigationConfig,
    data_state: SharedSpeedState,
    ws_clients: Clients,
    mqtt_manager: Arc<MqttManager>,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        loop {
            let (update_rate, topic_prefix) = {
                let cfg = config_state.read().unwrap();
                let ur = cfg.update_rate.unwrap_or(1000);
                let tp = cfg.topics.as_ref().and_then(|v| v.first()).cloned().unwrap_or_else(|| "vessel/navigation".to_string());
                (ur, tp)
            };
            
            select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            tracing::info!("[Speed Service]: Reconnect requested.");
                        }
                        MqttCommand::Stop => {
                            tracing::info!("[Speed Service]: Stop requested. Exiting publication loop.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(speed_state) = data_opt {
                        if speed_state.is_running {
                            let payload = match serde_json::to_string(&speed_state) {
                                Ok(p) => p,
                                Err(e) => { eprintln!("[Speed Service]: JSON serialize error: {}", e); continue; }
                            };
                            let topic = format!("{}/speed", topic_prefix);

                            if let Err(e) = mqtt_manager.publish_message(&[topic.clone()], payload.clone()).await {
                                eprintln!("[Speed Service]: MQTT publish error to {}: {:?}", topic, e);
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

                            // Conditionally publish VHW NMEA sentence
                            if is_sentence_enabled("VHW") {
                                let vhw_sentence = speed_state.to_vhw();
                                let vhw_topic = format!("{}/vhw", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(&[vhw_topic.clone()], vhw_sentence.clone()).await {
                                    eprintln!("[Speed Service]: MQTT publish error to {}: {:?}", vhw_topic, e);
                                }
                            }

                            let msg = serde_json::json!({ "type": "speed_update", "data": speed_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        tracing::info!("[Speed Service]: Publication thread exited.");
    });
}
