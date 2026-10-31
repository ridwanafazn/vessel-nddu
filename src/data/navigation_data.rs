use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// Shared types for navigation sensors (Speed & Depth)
pub type SharedNavigationConfig = Arc<RwLock<NavigationConfig>>;
pub type SharedSpeedState = Arc<RwLock<Option<SpeedState>>>;
pub type SharedDepthState = Arc<RwLock<Option<DepthState>>>;

// Speed State
#[derive(Clone, Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct SpeedState {
    pub speed_through_water: f64, // knots
    pub is_running: bool,
    pub last_update: DateTime<Utc>,

    #[serde(skip)]
    #[schema(ignore)]
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
        // Talker II (integrated instrumentation) wajib ada agar valid NMEA 0183.
        let sentence_body = format!(
            "IIVHW,{:.1},T,{:.1},M,{:.1},N,{:.1},K",
            heading_true, heading_mag, speed_kn, speed_kmh
        );
        crate::utils::nmea::build(&sentence_body)
    }
}

// Depth State
#[derive(Clone, Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct DepthState {
    pub ocean_water_depth: f64,    // meters from surface (Current Depth)
    pub constant_clearance: f64,   // meters (Set Constant Clearance)
    pub under_keel_clearance: f64, // meters (UKC = ocean_water_depth - constant_clearance)
    pub is_running: bool,
    pub last_update: DateTime<Utc>,

    #[serde(skip)]
    #[schema(ignore)]
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
        // Talker SD (sounder depth).
        let sentence_body = format!("SDDBT,{:.1},f,{:.1},M,{:.1},F", depth_ft, depth_m, depth_fa);
        crate::utils::nmea::build(&sentence_body)
    }

    /// Converts Depth state to NMEA 0183 DPT sentence (Depth)
    /// Format: $--DPT,xx.x,yy.y*hh<CR><LF>
    pub fn to_dpt(&self) -> String {
        let depth_m = self.ocean_water_depth;
        let offset = self.constant_clearance;
        // Talker SD (sounder depth).
        let sentence_body = format!("SDDPT,{:.1},{:.1}", depth_m, offset);
        crate::utils::nmea::build(&sentence_body)
    }

    /// Converts Depth state to NMEA 0183 DBK sentence (Depth Below Keel)
    /// Format: $--DBK,xx.x,f,yy.y,M,zz.z,F*hh<CR><LF>
    pub fn to_dbk(&self) -> String {
        let depth_m = self.under_keel_clearance;
        let depth_ft = depth_m * 3.28084;
        let depth_fa = depth_m * 0.546807;
        // Talker SD (sounder depth).
        let sentence_body = format!("SDDBK,{:.1},f,{:.1},M,{:.1},F", depth_ft, depth_m, depth_fa);
        crate::utils::nmea::build(&sentence_body)
    }
}

// Shared Navigation Config (for both Speed & Depth)
#[derive(Clone, Serialize, Deserialize, Debug, utoipa::ToSchema, Default)]
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

// Speed Request Types
#[derive(Deserialize, Debug, utoipa::ToSchema)]
pub struct CreateSpeedRequest {
    pub speed_through_water: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default, utoipa::ToSchema)]
pub struct UpdateSpeedRequest {
    pub speed_through_water: Option<f64>,
    pub is_running: Option<bool>,
}

// Depth Request Types
#[derive(Deserialize, Debug, utoipa::ToSchema)]
pub struct CreateDepthRequest {
    pub ocean_water_depth: f64,
    pub constant_clearance: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default, utoipa::ToSchema)]
pub struct UpdateDepthRequest {
    pub ocean_water_depth: Option<f64>,
    pub constant_clearance: Option<f64>,
    pub is_running: Option<bool>,
}

// Shared Config Request
#[derive(Deserialize, Debug, Default, utoipa::ToSchema)]
pub struct UpdateNavigationConfigRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
    pub enabled_sentences: Option<Vec<String>>, // Added for dynamic NMEA selection
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::nmea;

    fn speed() -> SpeedState {
        SpeedState {
            speed_through_water: 12.0,
            is_running: true,
            last_update: chrono::Utc::now(),
            calculation_rate_ms: 100,
        }
    }

    fn depth() -> DepthState {
        DepthState {
            ocean_water_depth: 600.0,
            constant_clearance: 10.0,
            under_keel_clearance: 590.0,
            is_running: true,
            last_update: chrono::Utc::now(),
            calculation_rate_ms: 100,
        }
    }

    #[test]
    fn vhw_has_talker_and_valid_checksum() {
        let s = speed().to_vhw();
        assert!(s.starts_with("$IIVHW,"), "got: {}", s);
        assert!(nmea::validate(&s), "invalid: {}", s);
        assert_eq!(nmea::talker(&s), Some("II"));
        assert_eq!(nmea::formatter(&s), Some("VHW"));
    }

    #[test]
    fn dbt_has_sd_talker_and_valid_checksum() {
        let s = depth().to_dbt();
        assert!(s.starts_with("$SDDBT,"), "got: {}", s);
        assert!(nmea::validate(&s), "invalid: {}", s);
    }

    #[test]
    fn dpt_has_sd_talker_and_valid_checksum() {
        let s = depth().to_dpt();
        assert!(s.starts_with("$SDDPT,"), "got: {}", s);
        assert!(nmea::validate(&s), "invalid: {}", s);
    }

    #[test]
    fn dbk_has_sd_talker_and_valid_checksum() {
        let s = depth().to_dbk();
        assert!(s.starts_with("$SDDBK,"), "got: {}", s);
        assert!(nmea::validate(&s), "invalid: {}", s);
    }

    #[test]
    fn vhw_uses_knots_and_kmh() {
        let s = speed().to_vhw();
        // 12 knots = 22.2 km/h
        assert!(s.contains(",12.0,N,22.2,K"), "got: {}", s);
    }
}
