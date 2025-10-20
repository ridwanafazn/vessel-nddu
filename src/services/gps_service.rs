use crate::AppState;
use crate::utils;
use std::time::Duration;
use tokio::time::sleep;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::task::JoinHandle;
use crate::ConfigUpdate;


pub fn run_gps_tasks(state: AppState) {
    let calc_state = state.clone();
    tokio::spawn(async move {
        loop {
            let update_rate_ms = {
                let config = calc_state.gps_config.read().await;
                config.update_rate.unwrap_or(1000)
            };
            
            sleep(Duration::from_millis(update_rate_ms)).await;

            let mut guard = calc_state.gps_data.write().await;
            if let Some(ref mut gps_data) = *guard {
                if gps_data.is_running {
                    let dt_seconds = update_rate_ms as f64 / 1000.0;
                    utils::gps_calculate::calculate_next_gps_state(gps_data, dt_seconds);
                    
                    let updated_data = gps_data.clone();
                    drop(guard); 

                    if let Err(e) = calc_state.gps_update_tx.send(updated_data) {
                        log::warn!("[GPS Calc Task] Gagal mengirim update, tidak ada subscriber aktif: {}", e);
                    }
                }
            }
        }
    });

    let pub_state = state;
    tokio::spawn(async move {
        let mut active_client: Option<AsyncClient> = None;
        let mut eventloop_handle: Option<JoinHandle<()>> = None;

        let mut config_rx = pub_state.config_update_tx.subscribe();
        let mut data_rx = pub_state.gps_update_tx.subscribe();

        log::info!("[GPS Manager Task] Service dimulai, menunggu konfigurasi dan data...");

        loop {
            if active_client.is_none() {
                let config = pub_state.gps_config.read().await;
                if let (Some(ip), Some(port)) = (config.ip.as_deref(), config.port) {
                    log::info!("[GPS Manager Task] Mencoba konek ke MQTT broker di {}:{}", ip, port);
                    let mut mqttoptions = MqttOptions::new("vessel-gps-service", ip, port);
                    mqttoptions.set_keep_alive(Duration::from_secs(5));
                    if let (Some(user), Some(pass)) = (config.username.as_ref(), config.password.as_ref()) {
                        mqttoptions.set_credentials(user, pass);
                    }

                    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
                    
                    let handle = tokio::spawn(async move {
                        loop {
                            if let Err(e) = eventloop.poll().await {
                                log::error!("[GPS MQTT EventLoop] Error koneksi: {}. Mencoba reconnect...", e);
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    });

                    log::info!("[GPS Manager Task] Berhasil konek ke MQTT broker.");
                    eventloop_handle = Some(handle);
                    active_client = Some(client);
                }
            }
            
            tokio::select! {
                Ok(update_type) = config_rx.recv() => {
                    if let ConfigUpdate::Gps = update_type {
                        log::info!("[GPS Manager Task] Menerima sinyal update config. Akan melakukan reconnect...");
                        if let Some(handle) = eventloop_handle.take() {
                            handle.abort();
                        }
                        active_client = None;
                    }
                },

                Ok(gps_data) = data_rx.recv() => {
                    if let Some(client) = &active_client {
                        let topics = {
                            let config = pub_state.gps_config.read().await;
                            config.topics.clone().unwrap_or_default()
                        };

                        let message_payload = serde_json::json!({ "type": "gps_update", "data": &gps_data });
                        let message_string = message_payload.to_string();

                        utils::net::broadcast_ws_message(&pub_state.ws_clients, message_string.clone()).await;

                        for topic in topics {
                            log::debug!("[GPS Manager Task] Publikasi ke MQTT topic '{}'", topic);
                            if let Err(e) = client.publish(&topic, QoS::AtLeastOnce, false, message_string.clone().into_bytes()).await {
                                log::error!("[GPS Manager Task] Gagal publish ke MQTT topic '{}': {:?}", topic, e);
                            }
                        }
                    } else {
                        log::warn!("[GPS Manager Task] Menerima data, tetapi tidak ada koneksi MQTT aktif untuk publikasi.");
                    }
                }
            }
        }
    });
}