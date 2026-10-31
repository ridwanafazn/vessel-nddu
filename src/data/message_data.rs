use crate::data::gps_data::GpsState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct MessageData {
    pub message: String,
    pub data: GpsState,
}
