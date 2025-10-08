use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::data::{gps_data::GPSConfig, gyro_data::GyroData};
use tracing::{error, info};

lazy_static::lazy_static! {
    static ref MQTT_CLIENT: Arc<Mutex<Option<AsyncClient>>> = Arc::new(Mutex::new(None));
}

/// Membuat koneksi MQTT baru berdasarkan konfigurasi GPS (IP, port, username, password)
pub async fn connect_mqtt(config: &GPSConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut mqttoptions = MqttOptions::new(
        "vessel_client",
        config.ip.clone(),
        config.port,
    );

    mqttoptions.set_credentials(config.username.clone(), config.password.clone());
    mqttoptions.set_keep_alive(std::time::Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    {
        let mut global_client = MQTT_CLIENT.lock().await;
        *global_client = Some(client.clone());
    }

    // Jalankan event loop di background
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(_) => {}
                Err(e) => {
                    error!("[MQTT] EventLoop error: {:?}", e);
                    break;
                }
            }
        }
        info!("[MQTT] EventLoop stopped.");
    });

    info!("[MQTT] Connected to {}:{}", config.ip, config.port);
    Ok(())
}

/// Menutup koneksi MQTT (reset global client)
pub async fn disconnect_mqtt() {
    let mut client = MQTT_CLIENT.lock().await;
    *client = None;
    info!("[MQTT] Disconnected.");
}

/// Reconnect MQTT ketika konfigurasi berubah
pub async fn reconnect_if_needed(config: &GPSConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("[MQTT] Reconnecting with new configuration...");
    disconnect_mqtt().await;
    connect_mqtt(config).await?;
    Ok(())
}

/// Fungsi generik untuk publish data (bisa GPS, Gyro, dsb)
pub async fn publish_mqtt<T: Serialize>(
    topic: &str,
    data: &T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let payload = serde_json::to_vec(data)?;

    let client_opt = MQTT_CLIENT.lock().await;
    if let Some(client) = &*client_opt {
        client.publish(topic, QoS::AtLeastOnce, false, payload).await?;
        Ok(())
    } else {
        Err("MQTT client not connected".into())
    }
}

/// Khusus publish data GPS (helper untuk service GPS)
pub async fn publish_gps_mqtt(
    data: &crate::data::gps_data::GPSData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    publish_mqtt("gps/default", data).await
}

/// Khusus publish data Gyro (helper untuk service Gyro)
pub async fn publish_gyro_mqtt(
    data: &GyroData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    publish_mqtt("gyro/default", data).await
}
