use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// DIUBAH: Sekarang ada dua tipe alias terpisah untuk State dan Config.
pub type SharedGpsState = Arc<RwLock<Option<GpsState>>>;
pub type SharedGpsConfig = Arc<RwLock<GpsConfig>>;

// DIUBAH: Struct ini sekarang ramping dan HANYA berisi data sensor.
// Field `config` telah dihapus.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GpsState {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,
    pub cog: f64,
    pub variation: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    #[serde(skip)]
    pub calculation_rate_ms: u64,
}

// DIUBAH: Struct ini sekarang independen dan semua field-nya adalah Option<T>.
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

// DIUBAH: Default untuk Config sekarang adalah semua field bernilai None.
impl Default for GpsConfig {
    fn default() -> Self {
        GpsConfig {
            ip: None,
            port: None,
            username: None,
            password: None,
            update_rate: None,
            topics: None,
        }
    }
}

// Struct untuk request API di bawah ini sebagian besar tetap sama,
// karena sudah dirancang dengan baik.

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