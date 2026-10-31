use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// Shared types for navigation sensors (Speed & Depth)
pub type SharedNavigationConfig = Arc<RwLock<NavigationConfig>>;
pub type SharedSpeedState = Arc<RwLock<Option<SpeedState>>>;
pub type SharedDepthState = Arc<RwLock<Option<DepthState>>>;

// Speed State
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SpeedState {
    pub speed_through_water: f64, // knots
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    
    #[serde(skip)]
    #[allow(dead_code)]
    pub calculation_rate_ms: u64,
}

impl SpeedState {
    /// Converts Speed state to NMEA 0183 VHW sentence (Water speed and heading)
    /// Format: $VHW,heading,T,heading,M,speed_kn,N,speed_kmh,K*hh<CR><LF>
    pub fn to_vhw(&self) -> String {
        let heading_true = 0.0; // If no heading, set 0.0 or use from compass if available
        let heading_mag = 0.0; // If no magnetic, set 0.0
        let speed_kn = self.speed_through_water;
        let speed_kmh = self.speed_through_water * 1.852;
        let sentence_body = format!("VHW,{:.1},T,{:.1},M,{:.1},N,{:.1},K", heading_true, heading_mag, speed_kn, speed_kmh);
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Helper: Calculate NMEA checksum (XOR of all bytes between $ and *)
    fn nmea_calculate_checksum(sentence_body: &str) -> u8 {
        sentence_body.bytes().fold(0u8, |acc, b| acc ^ b)
    }
}

// Depth State
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DepthState {
    pub ocean_water_depth: f64,  // meters from surface (Current Depth)
    pub constant_clearance: f64,  // meters (Set Constant Clearance)
    pub under_keel_clearance: f64, // meters (UKC = ocean_water_depth - constant_clearance)
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    
    #[serde(skip)]
    #[allow(dead_code)]
    pub calculation_rate_ms: u64,
}

impl DepthState {
    /// Calculate UKC from current depth and constant clearance
    pub fn calculate_ukc(&mut self) {
        self.under_keel_clearance = self.ocean_water_depth - self.constant_clearance;
    }

    /// Converts Depth state to NMEA 0183 DBT sentence (Depth Below Transducer)
    /// Format: $--DBT,xx.x,f,yy.y,M,zz.z,F*hh<CR><LF>
    pub fn to_dbt(&self) -> String {
        let depth_m = self.ocean_water_depth;
        let depth_ft = depth_m * 3.28084;
        let depth_fa = depth_m * 0.546807;
        let sentence_body = format!("DBT,{:.1},f,{:.1},M,{:.1},F", depth_ft, depth_m, depth_fa);
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Converts Depth state to NMEA 0183 DPT sentence (Depth)
    /// Format: $--DPT,xx.x,yy.y*hh<CR><LF>
    pub fn to_dpt(&self) -> String {
        let depth_m = self.ocean_water_depth;
        let offset = self.constant_clearance;
        let sentence_body = format!("DPT,{:.1},{:.1}", depth_m, offset);
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Converts Depth state to NMEA 0183 DBK sentence (Depth Below Keel)
    /// Format: $--DBK,xx.x,f,yy.y,M,zz.z,F*hh<CR><LF>
    pub fn to_dbk(&self) -> String {
        let depth_m = self.under_keel_clearance;
        let depth_ft = depth_m * 3.28084;
        let depth_fa = depth_m * 0.546807;
        let sentence_body = format!("DBK,{:.1},f,{:.1},M,{:.1},F", depth_ft, depth_m, depth_fa);
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Helper: Calculate NMEA checksum (XOR of all bytes between $ and *)
    fn nmea_calculate_checksum(sentence_body: &str) -> u8 {
        sentence_body.bytes().fold(0u8, |acc, b| acc ^ b)
    }
}

// Shared Navigation Config (for both Speed & Depth)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NavigationConfig {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
    pub enabled_sentences: Option<Vec<String>>, // Added for dynamic NMEA selection
}

impl Default for NavigationConfig {
    fn default() -> Self {
        NavigationConfig {
            ip: None,
            port: None,
            username: None,
            password: None,
            update_rate: None,
            topics: None,
            enabled_sentences: None,
        }
    }
}

// Speed Request Types
#[derive(Deserialize, Debug)]
pub struct CreateSpeedRequest {
    pub speed_through_water: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateSpeedRequest {
    pub speed_through_water: Option<f64>,
    pub is_running: Option<bool>,
}

// Depth Request Types
#[derive(Deserialize, Debug)]
pub struct CreateDepthRequest {
    pub ocean_water_depth: f64,
    pub constant_clearance: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateDepthRequest {
    pub ocean_water_depth: Option<f64>,
    pub constant_clearance: Option<f64>,
    pub is_running: Option<bool>,
}

// Shared Config Request
#[derive(Deserialize, Debug, Default)]
pub struct UpdateNavigationConfigRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
    pub enabled_sentences: Option<Vec<String>>, // Added for dynamic NMEA selection
}
