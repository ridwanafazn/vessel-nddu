use crate::data::gps_data::{SharedGpsConfig, SharedGpsState};
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

/// 🔹 Thread perhitungan GPS (lokal, non-async)
pub fn start_gps_calculation_thread(state: SharedGpsState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
        let mut guard = state_clone.write().unwrap();
        if let Some(ref mut gps_state) = *guard {
            if gps_state.is_running {
                utils::gps_calculate::calculate_next_gps_state(gps_state);
            }
        }
    });
}

/// 🔹 Thread publikasi GPS ke MQTT + WebSocket
pub fn start_gps_publication_thread(
    config_state: SharedGpsConfig,
    data_state: SharedGpsState,
    ws_clients: Clients,
    mqtt_manager: Arc<MqttManager>,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        loop {
            // snapshot config
            let (update_rate, topic_prefix) = {
                let cfg = config_state.read().unwrap();
                let ur = cfg.update_rate.unwrap_or(1000);
                // ambil topic pertama atau default
                let tp = cfg
                    .topics
                    .as_ref()
                    .and_then(|v| v.first())
                    .cloned()
                    .unwrap_or_else(|| "vessel/gps".to_string());
                (ur, tp)
            };
            select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            tracing::info!("[GPS Service]: Reconnect requested.");
                        }
                        MqttCommand::Stop => {
                            tracing::info!("[GPS Service]: Stop requested. Exiting publication loop.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(gps_state) = data_opt {
                        if gps_state.is_running {
                            // Helper function to check if sentence is enabled
                            let is_sentence_enabled = |sentence_type: &str| -> bool {
                                let config_guard = config_state.read().unwrap();
                                match &config_guard.enabled_sentences {
                                    None => true, // None means publish all (backward compatible)
                                    Some(sentences) if sentences.is_empty() => true, // Empty list means publish all
                                    Some(sentences) => sentences.iter().any(|s| s.eq_ignore_ascii_case(sentence_type)),
                                }
                            };

                            // 1. Publish JSON format to default topic (vessel/gps)
                            let json_data = serde_json::to_string(&gps_state).unwrap_or_default();

                            if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&topic_prefix), json_data.clone()).await {
                                eprintln!("[GPS Service]: MQTT publish error to {}: {:?}", topic_prefix, e);
                            }

                            // 2. Conditionally publish GPRMC NMEA sentence to /gprmc topic
                            let gprmc_sentence = gps_state.to_gprmc();
                            if is_sentence_enabled("GPRMC") {
                                let gprmc_topic = format!("{}/gprmc", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&gprmc_topic), gprmc_sentence.clone()).await {
                                    eprintln!("[GPS Service]: MQTT publish error to {}: {:?}", gprmc_topic, e);
                                }
                            }

                            // 3. Conditionally publish GPGLL NMEA sentence to /gpgll topic
                            let gpgll_sentence = gps_state.to_gpgll();
                            if is_sentence_enabled("GPGLL") {
                                let gpgll_topic = format!("{}/gpgll", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&gpgll_topic), gpgll_sentence.clone()).await {
                                    eprintln!("[GPS Service]: MQTT publish error to {}: {:?}", gpgll_topic, e);
                                }
                            }

                            // 4. Conditionally publish GPVTG NMEA sentence to /gpvtg topic
                            let gpvtg_sentence = gps_state.to_gpvtg();
                            if is_sentence_enabled("GPVTG") {
                                let gpvtg_topic = format!("{}/gpvtg", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&gpvtg_topic), gpvtg_sentence.clone()).await {
                                    eprintln!("[GPS Service]: MQTT publish error to {}: {:?}", gpvtg_topic, e);
                                }
                            }

                            // 4b. Conditionally publish GPZDA NMEA sentence to /gpzda topic
                            let gpzda_sentence = gps_state.to_gpzda();
                            if is_sentence_enabled("GPZDA") {
                                let gpzda_topic = format!("{}/gpzda", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&gpzda_topic), gpzda_sentence.clone()).await {
                                    eprintln!("[GPS Service]: MQTT publish error to {}: {:?}", gpzda_topic, e);
                                }
                            }

                            // 5. For WebSocket, send all formats
                            let ws_msg = serde_json::json!({
                                "type": "gps_update",
                                "json": gps_state,
                                "gprmc": gprmc_sentence.trim_end(),
                                "gpgll": gpgll_sentence.trim_end(),
                                "gpvtg": gpvtg_sentence.trim_end(),
                                "gpzda": gpzda_sentence.trim_end()
                            });
                            let ws_json = ws_msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, ws_json).await;
                        }
                    }
                }
            }
        }

        tracing::info!("[GPS Service]: Publication thread exited.");
    });
}
