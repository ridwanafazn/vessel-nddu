use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// DIUBAH: Dua tipe alias terpisah.
pub type SharedGyroState = Arc<RwLock<Option<GyroState>>>;
pub type SharedGyroConfig = Arc<RwLock<GyroConfig>>;

// DIUBAH: Struct State yang ramping, tanpa config.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GyroState {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw_rate: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    #[serde(skip)]
    pub calculation_rate_ms: u64,
}

// DIUBAH: Struct Config yang independen dengan field Option<T>.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GyroConfig {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
}

// DIUBAH: Default untuk Config adalah semua field bernilai None.
impl Default for GyroConfig {
    fn default() -> Self {
        GyroConfig {
            ip: None,
            port: None,
            username: None,
            password: None,
            update_rate: None,
            topics: None,
        }
    }
}

// Struct Request API.
#[derive(Deserialize, Debug)]
pub struct CreateGyroRequest {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw_rate: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateGyroRequest {
    pub yaw: Option<f64>,
    pub pitch: Option<f64>,
    pub roll: Option<f64>,
    pub yaw_rate: Option<f64>,
    pub is_running: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateGyroConfigRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
}