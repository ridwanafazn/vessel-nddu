// DIUBAH: Path import disesuaikan dengan struktur crate yang benar.
use crate::AppState;
use crate::utils;
use std::time::Duration;
use tokio::time::sleep;

/// Menjalankan semua background task yang berhubungan dengan GPS (kalkulasi & publikasi).
pub fn run_gps_tasks(state: AppState) {
    // Task 1: Melakukan kalkulasi data GPS secara periodik sesuai `update_rate`.
    let calc_state = state.clone();
    tokio::spawn(async move {
        loop {
            // Ambil update_rate dari config di SETIAP iterasi.
            let update_rate_ms = {
                let config = calc_state.gps_config.read().await;
                config.update_rate.unwrap_or(1000) // Default 1 detik jika tidak diset
            };
            
            // Tidur sesuai durasi yang dinamis
            sleep(Duration::from_millis(update_rate_ms)).await;

            // Lakukan kalkulasi
            let mut guard = calc_state.gps_data.write().await;
            if let Some(ref mut gps_data) = *guard {
                if gps_data.is_running {
                    // BARU: Hitung delta time (dt) dalam detik dari update_rate.
                    let dt_seconds = update_rate_ms as f64 / 1000.0;
                    
                    // DIUBAH: Panggil fungsi kalkulasi dengan argumen dt_seconds yang baru.
                    utils::gps_calculate::calculate_next_gps_state(gps_data, dt_seconds);
                    
                    let updated_data = gps_data.clone();
                    drop(guard); // Lepas lock secepatnya

                    // Kirim notifikasi update ke semua subscriber (task publikasi).
                    if let Err(e) = calc_state.gps_update_tx.send(updated_data) {
                        log::warn!("[GPS Calc Task] Gagal mengirim update, tidak ada subscriber aktif: {}", e);
                    }
                }
            }
        }
    });

    // Task 2: Mendengarkan notifikasi update dan mempublikasikannya.
    let pub_state = state.clone();
    tokio::spawn(async move {
        let mut rx = pub_state.gps_update_tx.subscribe();
        log::info!("[GPS Pub Task] Siap mendengarkan update GPS...");

        loop {
            // Task ini "tidur" sampai ada pesan baru di channel. Sangat efisien.
            match rx.recv().await {
                Ok(gps_data) => {
                    log::debug!("[GPS Pub Task] Menerima update data GPS untuk dipublikasikan.");
                    
                    // Ambil config topic
                    let topics = {
                        let config = pub_state.gps_config.read().await;
                        config.topics.clone().unwrap_or_default()
                    };

                    // Buat payload pesan sekali saja
                    let message_payload = serde_json::json!({ "type": "gps_update", "data": gps_data });
                    let message_string = message_payload.to_string();

                    // Publikasi ke WebSocket
                    utils::net::broadcast_ws_message(&pub_state.ws_clients, message_string.clone()).await;

                    // Publikasi ke MQTT
                    for topic in topics {
                         if let Err(e) = pub_state.mqtt_manager.publish(&topic, message_string.clone()).await {
                            log::error!("[GPS Pub Task] Gagal publish ke MQTT topic '{}': {:?}", topic, e);
                        }
                    }

                    log::info!("[GPS Pub Task] Berhasil mempublikasikan update GPS.");
                },
                Err(e) => {
                    log::error!("[GPS Pub Task] Error saat menerima update dari channel: {}", e);
                }
            }
        }
    });
}
