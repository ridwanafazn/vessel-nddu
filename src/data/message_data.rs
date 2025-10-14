use serde::{Serialize, Deserialize};
use crate::data::{gps_data::GpsData, gyro_data::GyroData};

// DIHAPUS: Struct MessageData yang lama dan salah.
// pub struct MessageData {
//     pub message: String,
//     pub data: GpsData::GyroData, // INI PENYEBAB ERROR
// }

// BARU: Menggunakan enum untuk merepresentasikan berbagai jenis data sensor.
// Ini adalah cara yang type-safe dan benar di Rust.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")] // Trik Serde untuk membuat JSON yang Anda inginkan!
#[serde(rename_all = "snake_case")] // Membuat "GpsUpdate" menjadi "gps_update" di JSON
pub enum SensorMessage {
    GpsUpdate(GpsData),
    GyroUpdate(GyroData),
}