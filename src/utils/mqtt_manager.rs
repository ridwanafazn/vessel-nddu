use rumqttc::{AsyncClient, QoS};
use std::error::Error;

/// BARU: MqttManager yang jauh lebih sederhana.
/// Fungsinya hanya sebagai wrapper untuk mempermudah publikasi.
/// Logika koneksi sudah ditangani oleh eventloop di main.rs.
#[derive(Clone)]
pub struct MqttManager {
    client: AsyncClient,
}

impl MqttManager {
    pub fn new(client: AsyncClient) -> Self {
        Self { client }
    }

    /// Mempublikasikan pesan ke satu topic.
    pub async fn publish(
        &self,
        topic: &str,
        payload: String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload.into_bytes())
            .await?;
        Ok(())
    }
}