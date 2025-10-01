use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};

/// Konfigurasi GPS (tidak keluar di API karena di-skip)
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct GPSConfig {
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub update_rate: u64,        // ms
    pub topics: Vec<String>,     // daftar topic MQTT
}

/// Request dari client untuk mulai/ubah simulasi GPS
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GPSRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,                   // speed over ground (knot)
    pub cog: f64,                   // course over ground (derajat)
    pub update_rate: Option<u64>,   // opsional, default 1000
    pub is_running: bool,
    pub variation: Option<f64>,     // magnetic variation
}

/// Data GPS yang disimpan di store
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GPSData {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,
    pub cog: f64,
    pub update_rate: u64,
    pub is_running: bool,
    pub variation: Option<f64>,
    pub last_update: DateTime<Utc>,

    #[serde(skip_serializing, skip_deserializing)]
    pub config: GPSConfig, // tetap ada di struct, tapi tidak keluar di API
}

impl Default for GPSData {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            sog: 0.0,
            cog: 0.0,
            update_rate: 1000,
            is_running: false,
            variation: None,
            last_update: Utc::now(),
            config: GPSConfig {
                ip: "127.0.0.1".to_string(),
                port: 1883,
                username: "guest".to_string(),
                password: "guest".to_string(),
                update_rate: 1000,
                topics: vec!["gps/default".to_string()],
            },
        }
    }
}

impl From<GPSRequest> for GPSData {
    fn from(req: GPSRequest) -> Self {
        GPSData {
            latitude: req.latitude,
            longitude: req.longitude,
            sog: req.sog,
            cog: req.cog,
            update_rate: req.update_rate.unwrap_or(1000),
            is_running: req.is_running,
            variation: req.variation,
            last_update: Utc::now(),
            config: GPSConfig {
                ip: "127.0.0.1".to_string(),
                port: 1883,
                username: "guest".to_string(),
                password: "guest".to_string(),
                update_rate: req.update_rate.unwrap_or(1000),
                topics: vec!["gps/default".to_string()],
            },
        }
    }
}

/// Response ke client (tidak ada config, naming singkat sog/cog)
#[derive(Clone, Serialize, Debug)]
pub struct GPSResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,
    pub cog: f64,
    pub is_running: bool,
    pub variation: Option<f64>,
    pub last_update: DateTime<Utc>,
}

impl From<GPSData> for GPSResponse {
    fn from(data: GPSData) -> Self {
        GPSResponse {
            latitude: data.latitude,
            longitude: data.longitude,
            sog: data.sog,
            cog: data.cog,
            is_running: data.is_running,
            variation: data.variation,
            last_update: data.last_update,
        }
    }
}

impl From<&GPSData> for GPSResponse {
    fn from(data: &GPSData) -> Self {
        GPSResponse {
            latitude: data.latitude,
            longitude: data.longitude,
            sog: data.sog,
            cog: data.cog,
            is_running: data.is_running,
            variation: data.variation,
            last_update: data.last_update,
        }
    }
}

