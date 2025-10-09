use crate::data::gps_data::{SharedGpsConfig, SharedGpsState};
use crate::services::MqttCommand;
use crate::utils;
use crate::utils::net::Clients;
// DIUBAH: Hapus import yang tidak digunakan (Event, Packet, EventLoop)
use rumqttc::{AsyncClient, MqttOptions}; 
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;

const CALCULATION_INTERVAL_MS: u64 = 100;

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

pub fn start_gps_publication_thread(
    config_state: SharedGpsConfig,
    data_state: SharedGpsState,
    ws_clients: Clients,
    mut command_rx: mpsc::Receiver<MqttCommand>,
) {
    tokio::spawn(async move {
        let mut mqtt_client: Option<AsyncClient> = None;
        let mut eventloop_handle: Option<JoinHandle<()>> = None;

        loop {
            let update_rate = config_state.read().unwrap().update_rate.unwrap_or(1000);

            select! {
                Some(command) = command_rx.recv() => {
                    match command {
                        MqttCommand::Reconnect => {
                            println!("[MQTT Manager GPS]: Reconnect command received. Resetting connection.");
                            if let Some(handle) = eventloop_handle.take() {
                                handle.abort();
                            }
                            mqtt_client = None;
                        }
                    }
                }
                _ = sleep(Duration::from_millis(update_rate)) => {}
            }

            // DIUBAH: Logika koneksi ulang dengan pola Lock -> Salin -> Lepas Lock -> Await
            if mqtt_client.is_none() {
                println!("[MQTT Manager GPS]: Attempting to connect...");
                
                let connection_details = { // Scope pendek untuk lock
                    let config = config_state.read().unwrap();
                    if let (Some(ip), Some(port), Some(username), Some(password)) =
                        (&config.ip, config.port, &config.username, &config.password)
                    {
                        Some((ip.clone(), port, username.clone(), password.clone()))
                    } else {
                        None
                    }
                }; // Lock dilepas di sini

                if let Some((ip, port, username, password)) = connection_details {
                    let mut mqttoptions = MqttOptions::new("vessel_gps_publisher", ip, port);
                    mqttoptions.set_credentials(username, password);
                    mqttoptions.set_keep_alive(Duration::from_secs(5));

                    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
                    
                    // Operasi .await sekarang aman karena tidak ada lock yang dipegang
                    match eventloop.poll().await {
                        Ok(_) => {
                             println!("[MQTT Manager GPS]: Connection successful!");
                             mqtt_client = Some(client);
                             let handle = tokio::spawn(async move {
                                 loop {
                                     if eventloop.poll().await.is_err() { break; }
                                 }
                             });
                             eventloop_handle = Some(handle);
                        }
                        Err(e) => {
                             eprintln!("[MQTT Manager GPS]: Failed to connect: {}. Will retry later.", e);
                        }
                    }
                } else {
                    println!("[MQTT Manager GPS]: Config is incomplete. Skipping connection attempt.");
                }
            }
            
            if let Some(client) = &mqtt_client {
                if eventloop_handle.as_ref().map_or(true, |h| h.is_finished()) {
                    eprintln!("[MQTT Manager GPS]: Disconnected from broker. Will attempt to reconnect.");
                    mqtt_client = None;
                    eventloop_handle = None;
                    continue;
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