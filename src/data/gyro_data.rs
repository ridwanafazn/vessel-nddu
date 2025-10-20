use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GyroData {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw_rate: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
}

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

#[derive(Deserialize, Debug)]
pub struct CreateGyroPayload {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw_rate: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateGyroPayload {
    pub yaw: Option<f64>,
    pub pitch: Option<f64>,
    pub roll: Option<f64>,
    pub yaw_rate: Option<f64>,
    pub is_running: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateGyroConfigPayload {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
}