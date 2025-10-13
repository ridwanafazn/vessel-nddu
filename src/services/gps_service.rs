use crate::data::gps_data::{SharedGpsConfig, SharedGpsState};
use crate::utils::mqtt_manager::{MqttManager, MqttCommand, MqttServiceConfig};
use crate::utils;
use crate::utils::net::Clients;
use std::sync::Arc;
use std::time::Duration;
use std::thread;
use tokio::sync::mpsc;
use tokio::select;
use tokio::time::sleep;

const CALCULATION_INTERVAL_MS: u64 = 100;

/// Thread perhitungan GPS — tetap sama seperti sebelumnya.
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

/// Thread publikasi GPS yang kini menggunakan `MqttManager` (bukan langsung eventloop).
pub fn start_gps_publication_thread(
    config_state: SharedGpsConfig,
    data_state: SharedGpsState,
    ws_clients: Clients,
    mqtt_manager: Arc<MqttManager>,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        // Ambil konfigurasi dasar dari shared state
        let update_rate = {
            let config = config_state.read().unwrap();
            config.update_rate.unwrap_or(1000)
        };

        let topic_prefix = "vessel/gps".to_string();

        // Loop utama publikasi data GPS
        loop {
            select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        MqttCommand::Reconnect => {
                            println!("[GPS Service]: Reconnect requested.");
                            // handled externally oleh mqtt_manager
                        }
                        MqttCommand::Stop => {
                            println!("[GPS Service]: Stopping publication.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    // Ambil data dari state
                    let data_opt = {
                        let guard = data_state.read().unwrap();
                        guard.clone()
                    };

                    if let Some(gps_state) = data_opt {
                        if gps_state.is_running {
                            // Publish ke MQTT
                            let payload = serde_json::to_string(&gps_state).unwrap_or_default();
                            let topic = format!("{}/data", topic_prefix);
                            let _ = mqtt_manager.publish_message(&[topic], payload).await;

                            // Broadcast ke WebSocket
                            let msg = serde_json::json!({ "type": "gps_update", "data": gps_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        println!("[GPS Service]: Publication thread exited.");
    });
}
