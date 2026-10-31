//! Skema response untuk OpenAPI.
//!
//! Handler saat ini membangun response dengan `serde_json::json!`, sehingga
//! tipe-tipe ini dipakai **hanya** untuk dokumentasi. Bentuk field-nya
//! disamakan dengan response aktual.

use crate::data::gps_data::{GpsConfig, GpsState};
use crate::data::gyro_data::{GyroConfig, GyroState};
use crate::data::navigation_data::{DepthState, NavigationConfig, SpeedState};
use crate::data::weather_data::{WeatherConfig, WeatherState};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct GpsResponse {
    pub message: String,
    pub data: GpsState,
}

#[derive(Serialize, ToSchema)]
pub struct GpsConfigResponse {
    pub message: String,
    pub data: GpsConfig,
}

#[derive(Serialize, ToSchema)]
pub struct GyroResponse {
    pub message: String,
    pub data: GyroState,
}

#[derive(Serialize, ToSchema)]
pub struct GyroConfigResponse {
    pub message: String,
    pub data: GyroConfig,
}

#[derive(Serialize, ToSchema)]
pub struct WeatherResponse {
    pub message: String,
    pub data: WeatherState,
}

#[derive(Serialize, ToSchema)]
pub struct WeatherConfigResponse {
    pub message: String,
    pub data: WeatherConfig,
}

#[derive(Serialize, ToSchema)]
pub struct TemperatureData {
    pub air_temperature: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct TemperatureResponse {
    pub message: String,
    pub data: TemperatureData,
}

#[derive(Serialize, ToSchema)]
pub struct HumidityData {
    pub air_humidity: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct HumidityResponse {
    pub message: String,
    pub data: HumidityData,
}

#[derive(Serialize, ToSchema)]
pub struct WindData {
    pub wind_direction_relative: f64,
    pub wind_velocity: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct WindResponse {
    pub message: String,
    pub data: WindData,
}

#[derive(Serialize, ToSchema)]
pub struct PressureData {
    pub air_pressure: f64,
    pub is_running: bool,
    pub last_update: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct PressureResponse {
    pub message: String,
    pub data: PressureData,
}

#[derive(Serialize, ToSchema)]
pub struct SpeedResponse {
    pub message: String,
    pub data: SpeedState,
}

#[derive(Serialize, ToSchema)]
pub struct DepthResponse {
    pub message: String,
    pub data: DepthState,
}

#[derive(Serialize, ToSchema)]
pub struct NavigationConfigResponse {
    pub message: String,
    pub data: NavigationConfig,
}

#[derive(Serialize, ToSchema)]
pub struct CompassData {
    pub heading: f64,
    pub deviation: f64,
    pub variation: f64,
    pub is_running: bool,
    pub last_update: String,
}

#[derive(Serialize, ToSchema)]
pub struct CompassResponse {
    pub message: String,
    pub data: CompassData,
}

#[derive(Serialize, ToSchema)]
pub struct CompassConfigResponse {
    pub message: String,
    pub data: Option<crate::data::compass_data::CompassConfig>,
}

#[derive(Serialize, ToSchema)]
pub struct MqttConfigData {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub keep_alive_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dropped: Option<u64>,
}

#[derive(Serialize, ToSchema)]
pub struct MqttConfigResponse {
    pub message: String,
    pub data: MqttConfigData,
}
