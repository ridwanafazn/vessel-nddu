use rumqttc::{AsyncClient, QoS};

/// Publikasikan payload String ke beberapa topik MQTT.
pub async fn publish_mqtt_message(
    client: &AsyncClient,
    topics: &[String], // Menerima slice dari String topik
    payload: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    for topic in topics {
        client
            .publish(topic, QoS::AtLeastOnce, false, payload.clone())
            .await?;
    }

    Ok(())
}