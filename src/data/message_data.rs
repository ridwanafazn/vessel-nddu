use serde::{Serialize, Deserialize};
use crate::data::gps_data::GpsState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageData {
    pub message: String,
    pub data: GpsState,
}
