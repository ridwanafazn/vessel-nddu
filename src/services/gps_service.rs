use crate::data::gps_data::{SharedGpsConfig, SharedGpsState};
use crate::services::MqttCommand; // Import perintah yang baru kita buat
use crate::utils;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet};
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::sleep;
// Import yang diperlukan untuk publikasi
use crate::utils::net::Clients;

const CALCULATION_INTERVAL_MS: u64 = 100;

// Fungsi ini tidak banyak berubah, hanya penyesuaian tipe data.
pub fn start_gps_calculation_thread(state: SharedGpsState) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(CALCULATION_INTERVAL_MS)).await;
            {
                let mut guard = state.write().unwrap();
                if let Some(ref mut gps_state) = *guard {
                    if gps_state.is_running {
                        utils::gps_calculate::calculate_next_gps_state(gps_state);
                    }
                }
            }
        }
    });
}

// === MANAJER KONEKSI BARU ===
// Fungsi ini ditulis ulang sepenuhnya.
pub fn start_gps_publication_thread(
    config_state: SharedGpsConfig,
    data_state: SharedGpsState,
    ws_clients: Clients,
    mut command_rx: mpsc::Receiver<MqttCommand>, // Penerima perintah
) {
    tokio::spawn(async move {
        let mut mqtt_connection: Option<(AsyncClient, EventLoop)> = None;

        loop {
            // Cek apakah ada perintah baru atau waktunya publish
            let update_rate = {
                config_state.read().unwrap().update_rate.unwrap_or(1000)
            };

            select! {
                // 1. Menunggu perintah dari controller
                Some(command) = command_rx.recv() => {
                    match command {
                        MqttCommand::Reconnect => {
                            println!("[MQTT Manager GPS]: Reconnect command received. Resetting connection.");
                            mqtt_connection = None; // Putuskan koneksi lama
                        }
                    }
                }

                // 2. Menunggu timer untuk publikasi
                _ = sleep(Duration::from_millis(update_rate)) => {
                    // Lanjutkan ke logika publikasi di bawah
                }
            }
            
            // --- Logika Koneksi dan Publikasi ---

            // Jika tidak terhubung, coba hubungkan ulang
            if mqtt_connection.is_none() {
                println!("[MQTT Manager GPS]: Attempting to connect...");
                let config = config_state.read().unwrap();
                if let (Some(ip), Some(port), Some(username), Some(password)) = 
                    (&config.ip, config.port, &config.username, &config.password) {
                    
                    let mut mqttoptions = MqttOptions::new("vessel_gps_publisher", ip, port);
                    mqttoptions.set_credentials(username, password);
                    mqttoptions.set_keep_alive(Duration::from_secs(5));

                    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
                    println!("[MQTT Manager GPS]: Connection successful!");
                    mqtt_connection = Some((client, eventloop));
                } else {
                    println!("[MQTT Manager GPS]: Config is incomplete. Skipping connection attempt.");
                }
            }

            // Jika sudah terhubung, lakukan publikasi
            if let Some((client, eventloop)) = &mut mqtt_connection {
                // Poll eventloop untuk menjaga koneksi tetap hidup
                if let Ok(Event::Incoming(Packet::Disconnect)) = eventloop.poll().await {
                        eprintln!("[MQTT Manager GPS]: Disconnected from broker. Will attempt to reconnect.");
                        mqtt_connection = None;
                        continue; // Langsung coba reconnect di iterasi berikutnya
                }

                let data_to_publish = {
                    let guard = data_state.read().unwrap();
                    guard.as_ref().filter(|s| s.is_running).cloned()
                };

                if let Some(gps_state) = data_to_publish {
                    let message_payload = serde_json::json!({ "type": "gps_update", "data": gps_state });
                    let message_string = message_payload.to_string();
                    println!("[GPS PUBLISH]: {}", message_string);

                    utils::net::broadcast_ws_message(&ws_clients, message_string.clone()).await;
                    
                    let topics = config_state.read().unwrap().topics.clone().unwrap_or_default();
                    if let Err(e) = utils::mqtt::publish_mqtt_message(client, &topics, message_string).await {
                        eprintln!("Failed to publish GPS data to MQTT: {}", e);
                    }
                }
            }
        }
    });
}