use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared Gyro Store (async safe)
pub type GyroStore = Arc<Mutex<Option<GyroData>>>;

/// Konfigurasi koneksi dan publikasi Gyro
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct GyroConfig {
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub update_rate: u64,
    pub topics: Vec<String>,
}

/// Request dari API untuk update data gyro
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GyroRequest {
    /// Sudut arah (yaw) dalam derajat [0–360)
    pub yaw: f64,
    /// Sudut pitch dalam derajat [-90–90)
    pub pitch: f64,
    /// Sudut roll dalam derajat [-180–180)
    pub roll: f64,
    /// Laju perubahan yaw (deg/s)
    pub yaw_rate: f64,
    /// Status apakah gyro aktif berjalan
    pub is_running: bool,
}

/// Data Gyro aktif yang disimpan dan dikirim
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GyroData {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw_rate: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,

    /// Disimpan secara lokal untuk kebutuhan MQTT, tidak dikirim ke client
    #[serde(skip_serializing, skip_deserializing)]
    pub config: GyroConfig,
}

impl Default for GyroData {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            yaw_rate: 0.0,
            is_running: false,
            last_update: Utc::now(),
            config: GyroConfig {
                ip: "127.0.0.1".to_string(),
                port: 1883,
                username: "guest".to_string(),
                password: "guest".to_string(),
                update_rate: 1000,
                topics: vec!["gyro/default".to_string()],
            },
        }
    }
}

impl From<GyroRequest> for GyroData {
    fn from(req: GyroRequest) -> Self {
        GyroData {
            yaw: req.yaw,
            pitch: req.pitch,
            roll: req.roll,
            yaw_rate: req.yaw_rate,
            is_running: req.is_running,
            last_update: Utc::now(),
            config: GyroConfig {
                ip: "127.0.0.1".to_string(),
                port: 1883,
                username: "guest".to_string(),
                password: "guest".to_string(),
                update_rate: req.update_rate.unwrap_or(1000),
                topics: vec!["gyro/default".to_string()],
            },
        }
    }
}

/// Response API ke client
#[derive(Clone, Serialize, Debug)]
pub struct GyroResponse {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw_rate: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
}

impl From<GyroData> for GyroResponse {
    fn from(data: GyroData) -> Self {
        GyroResponse {
            yaw: data.yaw,
            pitch: data.pitch,
            roll: data.roll,
            yaw_rate: data.yaw_rate,
            is_running: data.is_running,
            last_update: data.last_update,
        }
    }
}

impl From<&GyroData> for GyroResponse {
    fn from(data: &GyroData) -> Self {
        GyroResponse {
            yaw: data.yaw,
            pitch: data.pitch,
            roll: data.roll,
            yaw_rate: data.yaw_rate,
            is_running: data.is_running,
            last_update: data.last_update,
        }
    }
}