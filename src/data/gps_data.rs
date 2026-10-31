use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// DIUBAH: Sekarang ada dua tipe alias terpisah untuk State dan Config.
pub type SharedGpsState = Arc<RwLock<Option<GpsState>>>;
pub type SharedGpsConfig = Arc<RwLock<GpsConfig>>;

#[derive(Clone, Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct GpsState {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,
    pub cog: f64,
    pub variation: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    #[serde(skip)]
    #[schema(ignore)]
    pub calculation_rate_ms: u64,
}

impl GpsState {
    /// Helper: Format time as hhmmss.ss (UTC)
    fn format_time(&self) -> String {
        let time_base = self.last_update.format("%H%M%S").to_string();
        let millisec = self.last_update.timestamp_subsec_millis() / 10; // Get centiseconds
        format!("{}.{:02}", time_base, millisec)
    }

    /// Helper: Format coordinate (latitude or longitude) to NMEA format
    /// Returns (coordinate_string, direction)
    fn format_coordinate(value: f64, is_latitude: bool) -> (String, &'static str) {
        let abs_value = value.abs();
        let degrees = abs_value.floor();
        let minutes = (abs_value - degrees) * 60.0;
        
        let coord_str = if is_latitude {
            format!("{:02}{:07.4}", degrees as i32, minutes)
        } else {
            format!("{:03}{:07.4}", degrees as i32, minutes)
        };
        
        let direction = if is_latitude {
            if value >= 0.0 { "N" } else { "S" }
        } else {
            if value >= 0.0 { "E" } else { "W" }
        };
        
        (coord_str, direction)
    }

    /// Helper: Calculate NMEA checksum (XOR of all bytes between $ and *)
    fn nmea_calculate_checksum(sentence_body: &str) -> u8 {
        sentence_body.bytes().fold(0u8, |acc, b| acc ^ b)
    }

    /// Converts GPS state to NMEA 0183 GPRMC sentence
    /// Format: $GPRMC,hhmmss.ss,Status,llll.ll,a,yyyyy.yy,a,x.x,x.x,ddmmyy,x.x,a,m*hh<CR><LF>
    pub fn to_gprmc(&self) -> String {
        let time_str = self.format_time();
        let status = if self.is_running { "A" } else { "V" };
        
        let (lat_str, lat_dir) = Self::format_coordinate(self.latitude, true);
        let (lon_str, lon_dir) = Self::format_coordinate(self.longitude, false);
        
        let speed_str = format!("{:.1}", self.sog);
        let course_str = format!("{:.1}", self.cog);
        let date_str = self.last_update.format("%d%m%y").to_string();
        
        let var_abs = self.variation.abs();
        let var_str = format!("{:.1}", var_abs);
        let var_dir = if self.variation >= 0.0 { "E" } else { "W" };
        
        let mode = if self.is_running { "A" } else { "N" };
        
        let sentence_body = format!(
            "GPRMC,{},{},{},{},{},{},{},{},{},{},{},{}",
            time_str, status, lat_str, lat_dir, lon_str, lon_dir,
            speed_str, course_str, date_str, var_str, var_dir, mode
        );
        
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Converts GPS state to NMEA 0183 GPGLL sentence
    /// Format: $GPGLL,llll.ll,a,yyyyy.yy,a,hhmmss.ss,A,m*hh<CR><LF>
    pub fn to_gpgll(&self) -> String {
        let (lat_str, lat_dir) = Self::format_coordinate(self.latitude, true);
        let (lon_str, lon_dir) = Self::format_coordinate(self.longitude, false);
        let time_str = self.format_time();
        let status = if self.is_running { "A" } else { "V" };
        let mode = if self.is_running { "A" } else { "N" };
        
        let sentence_body = format!(
            "GPGLL,{},{},{},{},{},{},{}",
            lat_str, lat_dir, lon_str, lon_dir, time_str, status, mode
        );
        
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Converts GPS state to NMEA 0183 GPVTG sentence
    /// Format: $GPVTG,cog_t,T,cog_m,M,sog_k,N,sog_kmh,K,m*hh<CR><LF>
    pub fn to_gpvtg(&self) -> String {
        // Course over ground (True)
        let cog_true = format!("{:.1}", self.cog);
        // Course over ground (Magnetic) = True course - variation
        let mut cog_magnetic = self.cog - self.variation;
        // Handle wrap-around: keep in range [0, 360)
        if cog_magnetic < 0.0 {
            cog_magnetic += 360.0;
        } else if cog_magnetic >= 360.0 {
            cog_magnetic -= 360.0;
        }
        let cog_mag = format!("{:.1}", cog_magnetic);
        // Speed over ground in knots
        let sog_knots = format!("{:.1}", self.sog);
        // Speed over ground in km/h (1 knot = 1.852 km/h)
        let sog_kmh = self.sog * 1.852;
        let sog_kmh_str = format!("{:.1}", sog_kmh);
        // Mode indicator: A = Autonomous, N = Invalid
        let mode = if self.is_running { "A" } else { "N" };
        let sentence_body = format!(
            "GPVTG,{},T,{},M,{},N,{},K,{}",
            cog_true, cog_mag, sog_knots, sog_kmh_str, mode
        );
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Converts GPS state to NMEA 0183 GPZDA sentence
    /// Format: $GPZDA,hhmmss.ss,dd,mm,yyyy,xx,xx*hh<CR><LF>
    pub fn to_gpzda(&self) -> String {
        use chrono::Timelike;
        use chrono::Datelike;
        let t = self.last_update;
        // hhmmss.ss
        let hour = t.hour();
        let min = t.minute();
        let sec = t.second();
        let centisec = t.timestamp_subsec_millis() / 10;
        let time_str = format!("{:02}{:02}{:02}.{:02}", hour, min, sec, centisec);
        // dd,mm,yyyy
        let day = t.day();
        let month = t.month();
        let year = t.year();
        // Local zone hours and minutes (always 00,00 for UTC)
        let lzh = 0;
        let lzm = 0;
        let sentence_body = format!("GPZDA,{},{:02},{:02},{:04},{:02},{:02}",
            time_str, day, month, year, lzh, lzm
        );
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct GpsConfig {
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
            enabled_sentences: None, // None means publish all sentences (backward compatible)
            tcp_enabled: Some(false), // TCP streaming disabled by default
        }
    }
}

#[derive(Deserialize, Debug, utoipa::ToSchema)]
pub struct CreateGpsRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub sog: f64,
    pub cog: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default, utoipa::ToSchema)]
pub struct UpdateGpsRequest {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub sog: Option<f64>,
    pub cog: Option<f64>,
    pub is_running: Option<bool>,
}

#[derive(Deserialize, Debug, Default, utoipa::ToSchema)]
pub struct UpdateGpsConfigRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
    pub enabled_sentences: Option<Vec<String>>,
    pub tcp_enabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::nmea;

    fn state() -> GpsState {
        GpsState {
            latitude: -6.2,
            longitude: 106.8,
            sog: 12.5,
            cog: 45.0,
            variation: 0.6,
            is_running: true,
            last_update: Utc::now(),
            calculation_rate_ms: 100,
        }
    }

    #[test]
    fn gprmc_is_valid() {
        let s = state().to_gprmc();
        assert!(s.starts_with("$GPRMC,"), "got: {}", s);
        assert!(nmea::validate(&s), "invalid: {}", s);
    }

    #[test]
    fn gpgll_is_valid() {
        let s = state().to_gpgll();
        assert!(s.starts_with("$GPGLL,"), "got: {}", s);
        assert!(nmea::validate(&s), "invalid: {}", s);
    }

    #[test]
    fn gpvtg_is_valid() {
        let s = state().to_gpvtg();
        assert!(s.starts_with("$GPVTG,"), "got: {}", s);
        assert!(nmea::validate(&s), "invalid: {}", s);
    }

    #[test]
    fn gpzda_is_valid() {
        let s = state().to_gpzda();
        assert!(s.starts_with("$GPZDA,"), "got: {}", s);
        assert!(nmea::validate(&s), "invalid: {}", s);
    }

    #[test]
    fn coordinate_format_is_nmea() {
        let s = state().to_gprmc();
        // -6.2 -> 06 degrees 12.0 minutes
        assert!(s.contains(",0612.0000,S,"), "got: {}", s);
        // 106.8 -> 106 degrees 48.0 minutes
        assert!(s.contains(",10648.0000,E,"), "got: {}", s);
    }
}