use serde::{Serialize, Deserialize};
use crate::data::gps_data::GPSData;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageData {
    pub message: String,
    pub data: GPSData,
}
