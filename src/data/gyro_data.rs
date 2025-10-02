use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type GyroStore = Arc<Mutex<Option<GyroData>>>;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct GyroConfig {
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub update_rate: u64, 
    pub topics: Vec<String>, 
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GyroRequest {
    pub heading_true: f64,          // derajat
    pub pitch: f64,                 // derajat
    pub roll: f64,                  // derajat
    pub heading_rate: f64,          // derajat per detik
    pub update_rate: Option<u64>,   // default 1000
    pub is_running: bool,
}

/// Data Gyro yang disimpan di store
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GyroData {
    pub heading_true: f64,
    pub pitch: f64,
    pub roll: f64,
    pub heading_rate: f64,
    pub update_rate: u64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,

    #[serde(skip_serializing, skip_deserializing)]
    pub config: GyroConfig,
}

impl Default for GyroData {
    fn default() -> Self {
        Self {
            heading_true: 0.0,
            pitch: 0.0,
            roll: 0.0,
            heading_rate: 0.0,
            update_rate: 1000,
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
            heading_true: req.heading_true,
            pitch: req.pitch,
            roll: req.roll,
            heading_rate: req.heading_rate,
            update_rate: req.update_rate.unwrap_or(1000),
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

#[derive(Clone, Serialize, Debug)]
pub struct GyroResponse {
    pub heading_true: f64,
    pub pitch: f64,
    pub roll: f64,
    pub heading_rate: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
}

impl From<GyroData> for GyroResponse {
    fn from(data: GyroData) -> Self {
        GyroResponse {
            heading_true: data.heading_true,
            pitch: data.pitch,
            roll: data.roll,
            heading_rate: data.heading_rate,
            is_running: data.is_running,
            last_update: data.last_update,
        }
    }
}

impl From<&GyroData> for GyroResponse {
    fn from(data: &GyroData) -> Self {
        GyroResponse {
            heading_true: data.heading_true,
            pitch: data.pitch,
            roll: data.roll,
            heading_rate: data.heading_rate,
            is_running: data.is_running,
            last_update: data.last_update,
        }
    }
}