use crate::data::gyro_data::GyroData;
use crate::utils::gyro_calculate::update_gyro_data;
use crate::utils::net::{dispatch_gyro, Clients};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio::time::{sleep, Duration};
use rumqttc::AsyncClient;

/// Tipe untuk menyimpan Gyro saat ini
pub type GyroStore = Arc<Mutex<Option<GyroData>>>;

/// ===== CRUD Gyro =====
pub async fn create_gyro(
    store: &GyroStore,
    data: GyroData,
    clients: Option<&Clients>,
    mqtt: Option<&AsyncClient>,
) {
    let gyro = update_gyro_data(data);

    {
        let mut lock = store.lock().await;
        *lock = Some(gyro.clone());
    }

    if let Some(clients) = clients {
        let mqtt = mqtt.cloned();
        let gyro_clone = gyro.clone();
        let clients_clone = clients.clone();
        tokio::spawn(async move {
            dispatch_gyro(&clients_clone, &gyro_clone, mqtt.as_ref()).await;
        });
    }
}

pub async fn get_gyro(store: &GyroStore) -> Option<GyroData> {
    let lock = store.lock().await;
    lock.clone()
}

pub async fn update_gyro(
    store: &GyroStore,
    update_fn: impl FnOnce(&mut GyroData) + Send + 'static,
    clients: Option<&Clients>,
    mqtt: Option<&AsyncClient>,
) -> Option<GyroData> {
    let mut lock = store.lock().await;
    if let Some(ref mut gyro) = *lock {
        update_fn(gyro);
        let updated = update_gyro_data(gyro.clone());
        *gyro = updated.clone();

        if let Some(clients) = clients {
            let mqtt = mqtt.cloned();
            let gyro_clone = updated.clone();
            let clients_clone = clients.clone();
            tokio::spawn(async move {
                dispatch_gyro(&clients_clone, &gyro_clone, mqtt.as_ref()).await;
            });
        }

        Some(updated)
    } else {
        None
    }
}

pub async fn delete_gyro(store: &GyroStore, clients: Option<&Clients>) -> bool {
    let mut lock = store.lock().await;
    if lock.is_some() {
        // Hapus data gyro
        *lock = None;

        // Broadcast ke semua client WebSocket
        if let Some(clients) = clients {
            let msg = Message::Text(
                r#"{"type": "gyro_delete", "message": "Success to delete Gyro live tracking."}"#
                    .to_string()
                    .into(),
            );

            let clients_lock = clients.lock().await;
            for client in clients_lock.iter() {
                if let Err(e) = client.send(msg.clone()) {
                    eprintln!("Failed to send message to client: {:?}", e);
                }
            }
        }
        true
    } else {
        false
    }
}

/// ===== Streaming Gyro periodik =====
pub fn start_gyro_stream(store: GyroStore, clients: Clients, mqtt: Option<AsyncClient>) {
    tokio::spawn(async move {
        loop {
            let gyro_opt = {
                let lock = store.lock().await;
                lock.clone()
            };

            if let Some(mut gyro) = gyro_opt {
                if gyro.is_running {
                    gyro = update_gyro_data(gyro.clone());

                    {
                        let mut lock = store.lock().await;
                        *lock = Some(gyro.clone());
                    }

                    let gyro_clone = gyro.clone();
                    let clients_clone = clients.clone();
                    let mqtt_clone = mqtt.clone();
                    tokio::spawn(async move {
                        dispatch_gyro(&clients_clone, &gyro_clone, mqtt_clone.as_ref()).await;
                    });

                    let wait_ms = if gyro.update_rate == 0 { 1000 } else { gyro.update_rate };
                    sleep(Duration::from_millis(wait_ms)).await;
                } else {
                    sleep(Duration::from_millis(500)).await;
                }
            } else {
                sleep(Duration::from_millis(500)).await;
            }
        }
    });
}
