// DIUBAH: Path import disesuaikan.
use crate::AppState;
use crate::utils;
use std::time::Duration;
use tokio::time::sleep;

/// Menjalankan semua background task yang berhubungan dengan Gyro.
pub fn run_gyro_tasks(state: AppState) {
    // Task 1: Kalkulasi data Gyro sesuai `update_rate`.
    let calc_state = state.clone();
    tokio::spawn(async move {
        loop {
            let update_rate_ms = {
                let config = calc_state.gyro_config.read().await;
                config.update_rate.unwrap_or(1000)
            };
            sleep(Duration::from_millis(update_rate_ms)).await;

            let mut guard = calc_state.gyro_data.write().await;
            if let Some(ref mut gyro_data) = *guard {
                if gyro_data.is_running {
                    // BARU: Hitung delta time (dt) dalam detik dari update_rate.
                    let dt_seconds = update_rate_ms as f64 / 1000.0;
                    
                    // DIUBAH: Panggil fungsi kalkulasi dengan argumen dt_seconds.
                    utils::gyro_calculate::calculate_next_gyro_state(gyro_data, dt_seconds);

                    let updated_data = gyro_data.clone();
                    drop(guard);
                    
                    // Kirim notifikasi update
                    if let Err(e) = calc_state.gyro_update_tx.send(updated_data) {
                         log::warn!("[Gyro Calc Task] Gagal mengirim update, tidak ada subscriber aktif: {}", e);
                    }
                }
            }
        }
    });

    // Task 2: Mendengarkan dan mempublikasikan update Gyro.
    let pub_state = state.clone();
    tokio::spawn(async move {
        let mut rx = pub_state.gyro_update_tx.subscribe();
        log::info!("[Gyro Pub Task] Siap mendengarkan update Gyro...");
        loop {
            match rx.recv().await {
                 Ok(gyro_data) => {
                    let topics = {
                        let config = pub_state.gyro_config.read().await;
                        config.topics.clone().unwrap_or_default()
                    };
                    
                    let message_payload = serde_json::json!({ "type": "gyro_update", "data": gyro_data });
                    let message_string = message_payload.to_string();

                    utils::net::broadcast_ws_message(&pub_state.ws_clients, message_string.clone()).await;

                    for topic in topics {
                        if let Err(e) = pub_state.mqtt_manager.publish(&topic, message_string.clone()).await {
                            log::error!("[Gyro Pub Task] Gagal publish ke MQTT topic '{}': {:?}", topic, e);
                        }
                    }
                    log::info!("[Gyro Pub Task] Berhasil mempublikasikan update Gyro.");
                },
                Err(e) => {
                    log::error!("[Gyro Pub Task] Error saat menerima update dari channel: {}", e);
                }
            }
        }
    });
}
