use rumqttc::{AsyncClient, QoS};
use crate::data::gps_data::GPSData;

/// Publish GPSData ke MQTT
pub async fn publish_gps_mqtt(
    client: &AsyncClient,
    gps: &GPSData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // println!("[DEBUG] Publishing GPS to MQTT: {:?}", gps);

    let payload = serde_json::to_vec(gps)?;
    client
        .publish("gps/default", QoS::AtLeastOnce, false, payload)
        .await?;
    Ok(())
}
