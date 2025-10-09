use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

pub type SharedGpsState = Arc<RwLock<Option<GpsState>>>;
pub type SharedGpsConfig = Arc<RwLock<GpsConfig>>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GPSRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,                // speed over ground (knot)
    pub cog: f64,                // course over ground (derajat)
    pub is_running: bool,
    pub variation: Option<f64>,  // magnetic variation
}

/// Data GPS yang disimpan di store
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GPSData {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,
    pub cog: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    #[serde(skip)]
    pub calculation_rate_ms: u64,
}

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
        Self {
            latitude: 0.0,
            longitude: 0.0,
            sog: 0.0,
            cog: 0.0,
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

#[derive(Deserialize, Debug)]
pub struct CreateGpsRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,
    pub cog: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateGpsRequest {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub sog: Option<f64>,
    pub cog: Option<f64>,
    pub is_running: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateGpsConfigRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
}