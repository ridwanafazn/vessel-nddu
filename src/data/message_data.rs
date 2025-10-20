use serde::{Serialize, Deserialize};
use crate::data::{gps_data::GpsData, gyro_data::GyroData};

// pub struct MessageData {
//     pub message: String,
//     pub data: GpsData::GyroData,
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum SensorMessage {
    GpsUpdate(GpsData),
    GyroUpdate(GyroData),
}