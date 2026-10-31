pub mod gps_controller;
// pub mod anemo_controller;
// pub mod baro_controller;
pub mod gyro_controller;
// pub mod thermal_controller;
pub mod auth_controller;
pub mod compass_controller;
pub mod depth_controller;
pub mod humidity_controller;
pub mod mqtt_controller;
pub mod navigation_controller;
pub mod pressure_controller;
pub mod speed_controller;
pub mod temperature_controller;
pub mod weather_controller;
pub mod wind_controller;

use crate::utils::mqtt_manager::MqttCommand;
use tokio::sync::mpsc;

pub type MqttCommandTx = mpsc::Sender<MqttCommand>;
