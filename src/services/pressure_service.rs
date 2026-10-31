use crate::data::weather_data::{SharedPressureState, SharedWeatherConfig};
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

/// Thread perhitungan Pressure (lokal, non-async)
pub fn start_pressure_calculation_thread(state: SharedPressureState) {
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(CALCULATION_INTERVAL_MS));
            let mut guard = state_clone.write().unwrap();
            if let Some(ref mut pressure_state) = *guard {
                if pressure_state.is_running {
                    // Simple simulation: vary pressure slightly (range: ~950-1050 hPa)
                    pressure_state.air_pressure += (rand::random::<f64>() - 0.5) * 2.0;
                    pressure_state.air_pressure = pressure_state.air_pressure.clamp(950.0, 1050.0);
                    pressure_state.last_update = chrono::Utc::now();
                }
            }
        }
    });
}

/// Thread publikasi Pressure ke MQTT + WebSocket
pub fn start_pressure_publication_thread(
    config_state: SharedWeatherConfig,
    data_state: SharedPressureState,
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
                            tracing::info!("[Pressure Service]: Reconnect requested.");
                        }
                        MqttCommand::Stop => {
                            tracing::info!("[Pressure Service]: Stop requested. Exiting publication loop.");
                            break;
                        }
                    }
                }

                _ = sleep(Duration::from_millis(update_rate)) => {
                    let data_opt = { data_state.read().unwrap().clone() };

                    if let Some(pressure_state) = data_opt {
                        if pressure_state.is_running {
                            let payload = match serde_json::to_string(&pressure_state) {
                                Ok(p) => p,
                                Err(e) => { eprintln!("[Pressure Service]: JSON serialize error: {}", e); continue; }
                            };
                            let topic = format!("{}/pressure", topic_prefix);

                            if let Err(e) = mqtt_manager.publish_message(std::slice::from_ref(&topic), payload.clone()).await {
                                eprintln!("[Pressure Service]: MQTT publish error to {}: {:?}", topic, e);
                            }

                            let msg = serde_json::json!({ "type": "pressure_update", "data": pressure_state });
                            let json = msg.to_string();
                            utils::net::broadcast_ws_message(&ws_clients, json).await;
                        }
                    }
                }
            }
        }

        tracing::info!("[Pressure Service]: Publication thread exited.");
    });
}
