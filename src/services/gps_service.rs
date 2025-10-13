use crate::data::gps_data::{SharedGpsConfig, SharedGpsState};
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

/// 🔹 Thread perhitungan GPS (lokal, non-async)
pub fn start_gps_calculation_thread(state: SharedGpsState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
            let mut guard = state_clone.write().unwrap();
            if let Some(ref mut gps_state) = *guard {
                if gps_state.is_running {
                    utils::gps_calculate::calculate_next_gps_state(gps_state);
                }
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
                let tp = cfg.topics.first().cloned().unwrap_or_else(|| "vessel/gps".to_string());
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
                            let payload = match serde_json::to_string(&gps_state) {
                                Ok(p) => p,
                                Err(e) => { eprintln!("[GPS Service]: JSON serialize error: {}", e); continue; }
                            };
                            let topic = format!("{}/data", topic_prefix);

                            if let Err(e) = mqtt_manager.publish_message(&[topic.clone()], payload.clone()).await {
                                eprintln!("[GPS Service]: MQTT publish error to {}: {:?}", topic, e);
                            }

                            let msg = serde_json::json!({ "type": "gps_update", "data": gps_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await.ok();
                        }
                    }
                }
            }
        }

        tracing::info!("[GPS Service]: Publication thread exited.");
    });
}
