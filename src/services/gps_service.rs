use crate::data::gps_data::GPSData;
use crate::utils::gps_calculate::update_gps_data;
use crate::utils::net::{dispatch_gps, Clients};
use std::sync::{Arc, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tokio::time::{sleep, Duration};
use rumqttc::AsyncClient;

/// Tipe untuk menyimpan GPS saat ini
pub type GPSStore = Arc<Mutex<Option<GPSData>>>;

/// ===== CRUD GPS =====
pub fn create_gps(
    store: &GPSStore,
    data: GPSData,
    clients: Option<&Clients>,
    mqtt: Option<&AsyncClient>,
) {
    let gps = update_gps_data(data);

    let mut lock = store.lock().unwrap();
    *lock = Some(gps.clone());

    if let Some(clients) = clients {
        let mqtt = mqtt.cloned();
        let gps_clone = gps.clone();
        let clients_clone = clients.clone();
        tokio::spawn(async move {
            dispatch_gps(&clients_clone, &gps_clone, mqtt.as_ref()).await;
        });
    }
}

pub fn get_gps(store: &GPSStore) -> Option<GPSData> {
    let lock = store.lock().unwrap();
    lock.clone()
}

pub fn update_gps(
    store: &GPSStore,
    update_fn: impl FnOnce(&mut GPSData),
    clients: Option<&Clients>,
    mqtt: Option<&AsyncClient>,
) -> Option<GPSData> {
    let mut lock = store.lock().unwrap();
    if let Some(ref mut gps) = *lock {
        update_fn(gps);
        let updated = update_gps_data(gps.clone());
        *gps = updated.clone();

        if let Some(clients) = clients {
            let mqtt = mqtt.cloned();
            let gps_clone = updated.clone();
            let clients_clone = clients.clone();
            tokio::spawn(async move {
                dispatch_gps(&clients_clone, &gps_clone, mqtt.as_ref()).await;
            });
        }

        Some(updated)
    } else {
        None
    }
}

pub fn delete_gps(store: &GPSStore, clients: Option<&Clients>) -> bool {
    let mut lock = store.lock().unwrap();
    if lock.is_some() {
        *lock = None;
        if let Some(clients) = clients {
            let msg = Message::Text(
                r#"{"type": "gps_delete", "message": "Success to delete GPS live tracking."}"#.into(),
            );
            let clients_lock = clients.lock().unwrap();
            for client in clients_lock.iter() {
                let _ = client.send(msg.clone());
            }
        }
        true
    } else {
        false
    }
}

/// ===== Streaming GPS periodik =====
/// Akan mengirim update ke semua client setiap `update_rate` ms jika `is_running == true`
pub fn start_gps_stream(store: GPSStore, clients: Clients, mqtt: Option<AsyncClient>) {
    tokio::spawn(async move {
        loop {
            let gps_opt = {
                let lock = store.lock().unwrap();
                lock.clone()
            };

            if let Some(mut gps) = gps_opt {
                if gps.is_running {
                    gps = update_gps_data(gps.clone());

                    {
                        let mut lock = store.lock().unwrap();
                        *lock = Some(gps.clone());
                    }

                    let gps_clone = gps.clone();
                    let clients_clone = clients.clone();
                    let mqtt_clone = mqtt.clone();
                    tokio::spawn(async move {
                        dispatch_gps(&clients_clone, &gps_clone, mqtt_clone.as_ref()).await;
                    });

                    let wait_ms = if gps.update_rate == 0 { 1000 } else { gps.update_rate };
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
