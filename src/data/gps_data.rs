use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// DIHAPUS: Tipe alias Shared... tidak lagi dibutuhkan karena kita pakai AppState.

// DIUBAH: Ganti nama GpsState menjadi GpsData agar lebih intuitif sebagai data transfer object (DTO).
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GpsData {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,
    pub cog: f64,
    pub variation: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    // DIHAPUS: `calculation_rate_ms` tidak perlu disimpan di sini, karena sudah ada di GpsConfig.
}

// Struct GpsConfig sudah bagus, tidak perlu diubah.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GpsConfig {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
}

impl Default for GpsConfig {
    fn default() -> Self {
        GpsConfig {
            ip: None,
            port: None,
            username: None,
            password: None,
            update_rate: Some(1000), // Default 1 detik
            topics: Some(vec!["vessel/gps/data".to_string()]), // Default topic
        }
    }
}

// Struct untuk request API, diubah agar lebih sesuai dengan ekspektasi API Anda.
#[derive(Deserialize, Debug)]
pub struct CreateGpsPayload {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,
    pub cog: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateGpsPayload {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub sog: Option<f64>,
    pub cog: Option<f64>,
    pub is_running: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateGpsConfigPayload {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
}