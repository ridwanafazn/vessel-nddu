use crate::data::gyro_data::{SharedGyroConfig, SharedGyroState};
use crate::utils::mqtt_manager::{MqttManager, MqttCommand};
use crate::utils;
use crate::utils::net::Clients;
use std::sync::Arc;
use std::time::Duration;
use std::thread;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio::select;

const CALCULATION_INTERVAL_MS: u64 = 100;

/// 🔹 Thread kalkulasi Gyro (lokal, non-async)
pub fn start_gyro_calculation_thread(state: SharedGyroState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
            let mut guard = state_clone.write().unwrap();
            if let Some(ref mut gyro_state) = *guard {
                if gyro_state.is_running {
                    utils::gyro_calculate::calculate_next_gyro_state(gyro_state);
                }
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
) {
    tokio::spawn(async move {
        loop {
            // snapshot config
            let (update_rate, topic_prefix) = {
                let cfg = config_state.read().unwrap();
                let ur = cfg.update_rate.unwrap_or(1000);
                let tp = cfg.topics.first().cloned().unwrap_or_else(|| "vessel/gyro".to_string());
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

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(gyro_state) = data_opt {
                        if gyro_state.is_running {
                            let payload = match serde_json::to_string(&gyro_state) {
                                Ok(p) => p,
                                Err(e) => { eprintln!("[Gyro Service]: JSON serialize error: {}", e); continue; }
                            };
                            let topic = format!("{}/data", topic_prefix);

                            if let Err(e) = mqtt_manager.publish_message(&[topic.clone()], payload.clone()).await {
                                eprintln!("[Gyro Service]: MQTT publish error to {}: {:?}", topic, e);
                            }

                            let msg = serde_json::json!({ "type": "gyro_update", "data": gyro_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await.ok();
                        }
                    }
                }
            }
        }

        tracing::info!("[Gyro Service]: Publication thread exited.");
    });
}
