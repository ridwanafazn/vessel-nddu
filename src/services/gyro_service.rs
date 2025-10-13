use crate::data::gyro_data::{SharedGyroConfig, SharedGyroState};
use crate::utils::mqtt_manager::{MqttManager, MqttCommand, MqttServiceConfig};
use crate::utils;
use crate::utils::net::Clients;
use std::sync::Arc;
use std::time::Duration;
use std::thread;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio::select;

const CALCULATION_INTERVAL_MS: u64 = 100;

/// Thread kalkulasi gyro — tetap sama.
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

/// Thread publikasi gyro — sinkron ke `MqttManager`.
pub fn start_gyro_publication_thread(
    config_state: SharedGyroConfig,
    data_state: SharedGyroState,
    ws_clients: Clients,
    mqtt_manager: Arc<MqttManager>,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        let update_rate = {
            let config = config_state.read().unwrap();
            config.update_rate.unwrap_or(1000)
        };

        let topic_prefix = "vessel/gyro".to_string();

        loop {
            select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            println!("[Gyro Service]: Reconnect requested.");
                            // handled externally oleh mqtt_manager
                        }
                        MqttCommand::Stop => {
                            println!("[Gyro Service]: Stopping publication.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = {
                        let guard = data_state.read().unwrap();
                        guard.clone()
                    };

                    if let Some(gyro_state) = data_opt {
                        if gyro_state.is_running {
                            // Publish ke MQTT
                            let payload = serde_json::to_string(&gyro_state).unwrap_or_default();
                            let topic = format!("{}/data", topic_prefix);
                            let _ = mqtt_manager.publish_message(&[topic], payload).await;

                            // Broadcast ke WebSocket
                            let msg = serde_json::json!({ "type": "gyro_update", "data": gyro_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        println!("[Gyro Service]: Publication thread exited.");
    });
}
