impl GyroState {
        /// Converts Gyro state to NMEA 0183 HDT sentence
        /// Format: $HEHDT,xxx.x,T*hh<CR><LF>
        pub fn to_hdt(&self) -> String {
            let heading = self.yaw;
            let sentence_body = format!("HEHDT,{:.2},T", heading);
            let checksum = Self::nmea_calculate_checksum(&sentence_body);
            format!("${}*{:02X}\r\n", sentence_body, checksum)
        }

        /// Converts Gyro state to NMEA 0183 HDG sentence
        /// Format: $HEHDG,xxx.x,x.x,a,x.x,a*hh<CR><LF>
        pub fn to_hdg(&self) -> String {
            let heading = self.yaw;
            // Simulasi deviation dan variation
            let deviation = 0.0; // bisa di-random atau tetap
            let deviation_dir = "E";
            let variation = 0.0; // bisa diisi dari config jika ada
            let variation_dir = "E";
            let sentence_body = format!("HEHDG,{:.2},{:.1},{},{:.1},{}", heading, deviation, deviation_dir, variation, variation_dir);
            let checksum = Self::nmea_calculate_checksum(&sentence_body);
            format!("${}*{:02X}\r\n", sentence_body, checksum)
        }

        /// Converts Gyro state to NMEA 0183 XDR sentence (pitch, roll, yaw)
        /// Format: $HEXDR,A,x.x,D,PTCH,A,x.x,D,ROLL,A,x.x,D,YAW*hh<CR><LF>
        pub fn to_xdr(&self) -> String {
            let pitch = self.pitch;
            let roll = self.roll;
            let yaw = self.yaw;
            let sentence_body = format!("HEXDR,A,{:.2},D,PTCH,A,{:.2},D,ROLL,A,{:.2},D,YAW", pitch, roll, yaw);
            let checksum = Self::nmea_calculate_checksum(&sentence_body);
            format!("${}*{:02X}\r\n", sentence_body, checksum)
        }
    /// Converts Gyro state to NMEA 0183 ROT sentence
    /// Format: $HEROT,x.x,A*hh<CR><LF>
    pub fn to_rot(&self) -> String {
        // x.x = rate of turn, degrees/minute
        let rot_str = format!("{:.2}", self.yaw_rate);
        // Status: A = valid, V = invalid
        let status = if self.is_running { "A" } else { "V" };
        // Sentence body (HEROT is common for heading/gyro)
        let sentence_body = format!("HEROT,{},{}", rot_str, status);
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Helper: Calculate NMEA checksum (XOR of all bytes between $ and *)
    fn nmea_calculate_checksum(sentence_body: &str) -> u8 {
        sentence_body.bytes().fold(0u8, |acc, b| acc ^ b)
    }
}
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// DIUBAH: Dua tipe alias terpisah.
pub type SharedGyroState = Arc<RwLock<Option<GyroState>>>;
pub type SharedGyroConfig = Arc<RwLock<GyroConfig>>;

// DIUBAH: Struct State yang ramping, tanpa config.
#[derive(Clone, Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct GyroState {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw_rate: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    #[serde(skip)]
    #[schema(ignore)]
    pub calculation_rate_ms: u64,
}

// DIUBAH: Struct Config yang independen dengan field Option<T>.
#[derive(Clone, Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct GyroConfig {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
    pub enabled_sentences: Option<Vec<String>>,
    pub tcp_enabled: Option<bool>,
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
            enabled_sentences: None, // None means publish all sentences (backward compatible)
            tcp_enabled: Some(false), // TCP streaming disabled by default
        }
    }
}

// Struct Request API.
#[derive(Deserialize, Debug, utoipa::ToSchema)]
pub struct CreateGyroRequest {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw_rate: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default, utoipa::ToSchema)]
pub struct UpdateGyroRequest {
    pub yaw: Option<f64>,
    pub pitch: Option<f64>,
    pub roll: Option<f64>,
    pub yaw_rate: Option<f64>,
    pub is_running: Option<bool>,
}

#[derive(Deserialize, Debug, Default, utoipa::ToSchema)]
pub struct UpdateGyroConfigRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
    pub enabled_sentences: Option<Vec<String>>,
    pub tcp_enabled: Option<bool>,
}