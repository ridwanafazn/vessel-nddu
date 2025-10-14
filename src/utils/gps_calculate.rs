use crate::data::gps_data::GpsData;
use chrono::{Datelike, Utc};
use std::f64::consts::PI;
use time::{Date, Month};
use uom::si::angle::degree;
use uom::si::f32::*;
use uom::si::length::meter;
use world_magnetic_model::GeomagneticField;

const EARTH_RADIUS: f64 = 6_371_000.0;

fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / PI
}

pub fn calculate_magnetic_variation(lat: f64, lon: f64, date_time: &chrono::DateTime<Utc>) -> f64 {
    let month = match Month::try_from(date_time.month() as u8) {
        Ok(m) => m,
        Err(_) => return 0.0,
    };
    let day = date_time.day() as u8;

    let date = match Date::from_calendar_date(date_time.year(), month, day) {
        Ok(d) => d,
        Err(_) => return 0.0,
    };

    let height_q = Length::new::<meter>(0.0);
    let lat_q = Angle::new::<degree>(lat as f32);
    let lon_q = Angle::new::<degree>(lon as f32);

    match GeomagneticField::new(height_q, lat_q, lon_q, date) {
        Ok(field) => field.declination().get::<degree>() as f64,
        Err(_) => 0.0,
    }
}

// DIUBAH: Fungsi sekarang menerima `dt_seconds` (delta time in seconds) sebagai argumen.
pub fn calculate_next_gps_state(gps_data: &mut GpsData, dt_seconds: f64) {
    // DIPERBAIKI: Semua variabel `data` diganti menjadi `gps_data`.
    let speed_mps = gps_data.sog * 0.514444; // Konversi knot ke m/s
    let distance = speed_mps * dt_seconds;

    let lat_rad = deg_to_rad(gps_data.latitude);
    let lon_rad = deg_to_rad(gps_data.longitude);
    let course_rad = deg_to_rad(gps_data.cog);
    let angular_distance = distance / EARTH_RADIUS;

    let new_lat_rad = (lat_rad.sin() * angular_distance.cos()
        + lat_rad.cos() * angular_distance.sin() * course_rad.cos())
    .asin();

    let new_lon_rad = lon_rad
        + (course_rad.sin() * angular_distance.sin() * lat_rad.cos())
            .atan2(angular_distance.cos() - lat_rad.sin() * new_lat_rad.sin());

    gps_data.latitude = rad_to_deg(new_lat_rad).clamp(-90.0, 90.0);
    gps_data.longitude = (rad_to_deg(new_lon_rad) + 180.0).rem_euclid(360.0) - 180.0;
    gps_data.last_update = Utc::now();
    gps_data.variation = calculate_magnetic_variation(gps_data.latitude, gps_data.longitude, &gps_data.last_update);
}