use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};

/// Request dari client untuk mulai/ubah simulasi.
/// Tidak ada `last_update` karena diisi otomatis oleh server.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GPSRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub speed_over_ground: f64,   // knot
    pub course_over_ground: f64,  // derajat
    pub update_rate: u64,         // ms
    pub running: bool,
    pub magnetic_variation: Option<f64>,
}

/// State internal GPS yang digunakan simulator.
/// Termasuk `last_update` otomatis dari server.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GPSData {
    pub latitude: f64,
    pub longitude: f64,
    pub speed_over_ground: f64,   // knot
    pub course_over_ground: f64,  // derajat
    pub update_rate: u64,         // ms
    pub running: bool,
    pub magnetic_variation: Option<f64>,
    pub last_update: DateTime<Utc>, // otomatis RFC3339
}

impl Default for GPSData {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            speed_over_ground: 0.0,
            course_over_ground: 0.0,
            update_rate: 1000,
            running: false,
            magnetic_variation: None,
            last_update: Utc::now(),
        }
    }
}

impl From<GPSRequest> for GPSData {
    fn from(req: GPSRequest) -> Self {
        GPSData {
            latitude: req.latitude,
            longitude: req.longitude,
            speed_over_ground: req.speed_over_ground,
            course_over_ground: req.course_over_ground,
            update_rate: req.update_rate,
            running: req.running,
            magnetic_variation: req.magnetic_variation,
            last_update: Utc::now(),
        }
    }
}
