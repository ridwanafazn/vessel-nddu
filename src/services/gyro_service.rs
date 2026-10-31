use crate::data::gyro_data::{SharedGyroConfig, SharedGyroState};
use crate::utils;
use crate::utils::mqtt_manager::{MqttCommand, MqttManager};
use crate::utils::net::Clients;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

const CALCULATION_INTERVAL_MS: u64 = 100;

/// 🔹 Thread kalkulasi Gyro (lokal, non-async)
pub fn start_gyro_calculation_thread(state: SharedGyroState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
        let mut guard = state_clone.write().unwrap();
        if let Some(ref mut gyro_state) = *guard {
            if gyro_state.is_running {
                utils::gyro_calculate::calculate_next_gyro_state(gyro_state);
            }
        }
    });
}

/// 🔹 Thread publikasi Gyro ke MQTT + WebSocket
pub fn start_gyro_publication_thread(
    config_state: SharedGyroConfig,
    data_state: SharedGyroState,
    ws_clients: Clients,
    mqtt_manager: Arc<MqttManager>,
    mut command_rx: mpsc::Receiver<MqttCommand>,
    shutdown: CancellationToken,
) {
    tokio::spawn(async move {
        loop {
            // snapshot config
            let (update_rate, topic_prefix) = {
                let cfg = config_state.read().unwrap();
                let ur = cfg.update_rate.unwrap_or(1000);
                let tp = cfg
                    .topics
                    .as_ref()
                    .and_then(|v| v.first().cloned())
                    .unwrap_or_else(|| "vessel/gyro".to_string());
                (ur, tp)
            };

            select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            tracing::info!("[Gyro Service]: Reconnect requested.");
                        }
                        MqttCommand::Stop => {
                            tracing::info!("[Gyro Service]: Stop requested. Exiting publication loop.");
                            break;
                        }
                    }
                }

                _ = shutdown.cancelled() => {

                    tracing::info!("shutdown requested");

                    break;

                }


                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(gyro_state) = data_opt {
                        if gyro_state.is_running {
                            let payload = match serde_json::to_string(&gyro_state) {
                                Ok(p) => p,
                                Err(e) => { eprintln!("[Gyro Service]: JSON serialize error: {}", e); continue; }
                            };
                            let topic = topic_prefix.clone();

                            if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&topic), payload.clone()).await {
                                eprintln!("[Gyro Service]: MQTT publish error to {}: {:?}", topic, e);
                            }


                            // Helper function to check if sentence is enabled
                            let is_sentence_enabled = |sentence_type: &str| -> bool {
                                let config_guard = config_state.read().unwrap();
                                match &config_guard.enabled_sentences {
                                    None => true, // None means publish all (backward compatible)
                                    Some(sentences) if sentences.is_empty() => true, // Empty list means publish all
                                    Some(sentences) => sentences.iter().any(|s| s.eq_ignore_ascii_case(sentence_type)),
                                }
                            };

                            // Conditionally publish HEROT NMEA sentence to /rot topic
                            if is_sentence_enabled("HEROT") {
                                let rot_sentence = gyro_state.to_rot();
                                let rot_topic = format!("{}/rot", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&rot_topic), rot_sentence.clone()).await {
                                    eprintln!("[Gyro Service]: MQTT publish error to {}: {:?}", rot_topic, e);
                                }
                            }

                            // Conditionally publish HDT NMEA sentence to /hdt topic
                            if is_sentence_enabled("HDT") {
                                let hdt_sentence = gyro_state.to_hdt();
                                let hdt_topic = format!("{}/hdt", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&hdt_topic), hdt_sentence.clone()).await {
                                    eprintln!("[Gyro Service]: MQTT publish error to {}: {:?}", hdt_topic, e);
                                }
                            }

                            // Conditionally publish HDG NMEA sentence to /hdg topic
                            if is_sentence_enabled("HDG") {
                                let hdg_sentence = gyro_state.to_hdg();
                                let hdg_topic = format!("{}/hdg", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&hdg_topic), hdg_sentence.clone()).await {
                                    eprintln!("[Gyro Service]: MQTT publish error to {}: {:?}", hdg_topic, e);
                                }
                            }

                            // Conditionally publish XDR NMEA sentence to /xdr topic
                            if is_sentence_enabled("XDR") {
                                let xdr_sentence = gyro_state.to_xdr();
                                let xdr_topic = format!("{}/xdr", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&xdr_topic), xdr_sentence.clone()).await {
                                    eprintln!("[Gyro Service]: MQTT publish error to {}: {:?}", xdr_topic, e);
                                }
                            }

                            let msg = serde_json::json!({ "type": "gyro_update", "data": gyro_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        tracing::info!("[Gyro Service]: Publication thread exited.");
    });
}
