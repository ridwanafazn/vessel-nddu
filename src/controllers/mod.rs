pub mod gps_controller;
// pub mod anemo_controller;
// pub mod baro_controller;
pub mod gyro_controller;
// pub mod thermal_controller;

use crate::utils::mqtt_manager::MqttCommand;
use tokio::sync::mpsc;

pub type MqttCommandTx = mpsc::Sender<MqttCommand>;
