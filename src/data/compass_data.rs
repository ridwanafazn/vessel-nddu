use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use chrono::Local;

// Constants for magnetic compass
pub const COMPASS_DEVIATION: f64 = 1.0;  // degrees
pub const COMPASS_VARIATION: f64 = 3.0;  // degrees

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompassConfig {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub topics: Option<Vec<String>>,
    pub update_rate: Option<u32>,
}

impl CompassConfig {
    pub fn is_complete(&self) -> bool {
        self.ip.is_some()
            && self.port.is_some()
            && self.username.is_some()
            && self.password.is_some()
            && self.update_rate.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompassState {
    pub heading: f64,  // Dynamic value (0-360 degrees)
    pub is_running: bool,
    pub last_update: String,
    
    #[serde(skip)]
    #[allow(dead_code)]
    pub calculation_rate_ms: u64,
}

impl CompassState {
    #[allow(dead_code)]
    pub fn new() -> Self {
        CompassState {
            heading: 0.0,
            is_running: false,
            last_update: Local::now().format("%d %b %Y, %H:%M:%S").to_string(),
            calculation_rate_ms: 100,
        }
    }
}

pub type SharedCompassConfig = Arc<RwLock<Option<CompassConfig>>>;
pub type SharedCompassState = Arc<RwLock<Option<CompassState>>>;
