use crate::data::navigation_data::{SharedDepthState, SharedNavigationConfig};
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

/// Thread perhitungan Depth (lokal, non-async)
pub fn start_depth_calculation_thread(state: SharedDepthState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
        let mut guard = state_clone.write().unwrap();
        if let Some(ref mut depth_state) = *guard {
            if depth_state.is_running {
                utils::navigation_calculate::calculate_next_depth_state(depth_state);
            }
        }
    });
}

/// Thread publikasi Depth ke MQTT + WebSocket
pub fn start_depth_publication_thread(
    config_state: SharedNavigationConfig,
    data_state: SharedDepthState,
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
                    .unwrap_or_else(|| "vessel/navigation".to_string());
                (ur, tp)
            };

            select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            tracing::info!("[Depth Service]: Reconnect requested.");
                        }
                        MqttCommand::Stop => {
                            tracing::info!("[Depth Service]: Stop requested. Exiting publication loop.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(depth_state) = data_opt {
                        if depth_state.is_running {
                            let payload = match serde_json::to_string(&depth_state) {
                                Ok(p) => p,
                                Err(e) => { eprintln!("[Depth Service]: JSON serialize error: {}", e); continue; }
                            };
                            let topic = format!("{}/depth", topic_prefix);

                            if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&topic), payload.clone()).await {
                                eprintln!("[Depth Service]: MQTT publish error to {}: {:?}", topic, e);
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

                            // Conditionally publish DBT NMEA sentence
                            if is_sentence_enabled("DBT") {
                                let dbt_sentence = depth_state.to_dbt();
                                let dbt_topic = format!("{}/dbt", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&dbt_topic), dbt_sentence.clone()).await {
                                    eprintln!("[Depth Service]: MQTT publish error to {}: {:?}", dbt_topic, e);
                                }
                            }
                            // Conditionally publish DPT NMEA sentence
                            if is_sentence_enabled("DPT") {
                                let dpt_sentence = depth_state.to_dpt();
                                let dpt_topic = format!("{}/dpt", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&dpt_topic), dpt_sentence.clone()).await {
                                    eprintln!("[Depth Service]: MQTT publish error to {}: {:?}", dpt_topic, e);
                                }
                            }
                            // Conditionally publish DBK NMEA sentence
                            if is_sentence_enabled("DBK") {
                                let dbk_sentence = depth_state.to_dbk();
                                let dbk_topic = format!("{}/dbk", topic_prefix);
                                if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&dbk_topic), dbk_sentence.clone()).await {
                                    eprintln!("[Depth Service]: MQTT publish error to {}: {:?}", dbk_topic, e);
                                }
                            }

                            let msg = serde_json::json!({ "type": "depth_update", "data": depth_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        tracing::info!("[Depth Service]: Publication thread exited.");
    });
}
