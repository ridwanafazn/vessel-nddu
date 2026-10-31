use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// Shared types for weather config
pub type SharedWeatherConfig = Arc<RwLock<WeatherConfig>>;

// Individual state types for each weather sensor
pub type SharedTemperatureState = Arc<RwLock<Option<TemperatureState>>>;
pub type SharedHumidityState = Arc<RwLock<Option<HumidityState>>>;
pub type SharedWindState = Arc<RwLock<Option<WindState>>>;
pub type SharedPressureState = Arc<RwLock<Option<PressureState>>>;

// Old combined WeatherState (kept for backward compatibility if needed)
pub type SharedWeatherState = Arc<RwLock<Option<WeatherState>>>;

// Individual sensor states
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TemperatureState {
    pub air_temperature: f64, // °C
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    
    #[serde(skip)]
    #[allow(dead_code)]
    pub calculation_rate_ms: u64,
}

impl WindState {
    /// Converts Wind state to NMEA 0183 WIMWV sentence (Wind Speed and Angle)
    /// Format: $WIMWV,angle,R,speed,N,A*hh<CR><LF>
    pub fn to_wimwv(&self) -> String {
        let angle = self.wind_direction_relative;
        let speed = self.wind_velocity;
        let ref_type = "R"; // Relative to ship
        let unit = "N"; // Knots
        let status = if self.is_running { "A" } else { "V" };
        let sentence_body = format!("WIMWV,{:.1},{},{:.1},{},{}", angle, ref_type, speed, unit, status);
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Converts Wind state to NMEA 0183 WIMWD sentence (Wind Direction and Speed)
    /// Format: $WIMWD,dir_true,T,dir_mag,M,speed_kn,N,speed_ms,M*hh<CR><LF>
    pub fn to_wimwd(&self) -> String {
        let dir_true = self.wind_direction_relative;
        let dir_mag = dir_true; // If no magnetic, use same as true
        let speed_kn = self.wind_velocity;
        let speed_ms = self.wind_velocity * 0.514444; // knots to m/s
        let sentence_body = format!("WIMWD,{:.1},T,{:.1},M,{:.1},N,{:.1},M", dir_true, dir_mag, speed_kn, speed_ms);
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Helper: Calculate NMEA checksum (XOR of all bytes between $ and *)
    fn nmea_calculate_checksum(sentence_body: &str) -> u8 {
        sentence_body.bytes().fold(0u8, |acc, b| acc ^ b)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HumidityState {
    pub air_humidity: f64, // %
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    
    #[serde(skip)]
    #[allow(dead_code)]
    pub calculation_rate_ms: u64,
}

impl TemperatureState {
    /// Converts Temperature state to NMEA 0183 WIMTW sentence (Water Temperature)
    /// Format: $WIMTW,xx.x,C*hh<CR><LF>
    pub fn to_wimtw(&self) -> String {
        let temp = self.air_temperature;
        let sentence_body = format!("WIMTW,{:.1},C", temp);
        let checksum = Self::nmea_calculate_checksum(&sentence_body);
        format!("${}*{:02X}\r\n", sentence_body, checksum)
    }

    /// Helper: Calculate NMEA checksum (XOR of all bytes between $ and *)
    fn nmea_calculate_checksum(sentence_body: &str) -> u8 {
        sentence_body.bytes().fold(0u8, |acc, b| acc ^ b)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WindState {
    pub wind_direction_relative: f64, // ° relative to ship heading
    pub wind_velocity: f64,           // knots
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    
    #[serde(skip)]
    #[allow(dead_code)]
    pub calculation_rate_ms: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PressureState {
    pub air_pressure: f64, // mbar or hPa
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    
    #[serde(skip)]
    #[allow(dead_code)]
    pub calculation_rate_ms: u64,
}

// Combined WeatherState (for old /api/weather endpoint if still used)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WeatherState {
    // Weather Sensor Data
    pub air_temperature: f64,        // °C (from thermometer)
    pub air_humidity: f64,           // % (from hygrometer)
    pub wind_direction_relative: f64, // ° (from anemometer, relative to ship heading)
    pub wind_velocity: f64,    // knots (from anemometer)
    pub air_pressure: f64,           // mbar or hPa (from barometer)
    
    // Metadata
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
    
    #[serde(skip)]
    pub calculation_rate_ms: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WeatherConfig {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
    pub enabled_sentences: Option<Vec<String>>,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        WeatherConfig {
            ip: None,
            port: None,
            username: None,
            password: None,
            update_rate: None,
            topics: None,
            enabled_sentences: None, // None means publish all
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateWeatherRequest {
    pub air_temperature: f64,
    pub air_humidity: f64,
    pub wind_direction_relative: f64,
    pub wind_velocity: f64,
    pub air_pressure: f64,
    pub is_running: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateWeatherRequest {
    pub air_temperature: Option<f64>,
    pub air_humidity: Option<f64>,
    pub wind_direction_relative: Option<f64>,
    pub wind_velocity: Option<f64>,
    pub air_pressure: Option<f64>,
    pub is_running: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateWeatherConfigRequest {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub update_rate: Option<u64>,
    pub topics: Option<Vec<String>>,
    pub enabled_sentences: Option<Vec<String>>,
}