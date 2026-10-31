use crate::data::gps_data::GpsState;
use chrono::{Datelike, Utc}; // Menggunakan trait Datelike dari chrono
use std::f64::consts::PI;
use time::{Date, Month}; // Mengimpor Month dari time
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

// DIUBAH: Dibuat `pub` agar bisa diakses controller. Logika tanggal diperbaiki.
pub fn calculate_magnetic_variation(lat: f64, lon: f64, date_time: &chrono::DateTime<Utc>) -> f64 {
    // Konversi u32 month dari chrono ke enum Month dari time
    let month = match Month::try_from(date_time.month() as u8) {
        Ok(m) => m,
        Err(_) => return 0.0, // Return default jika konversi gagal
    };
    // Konversi u32 day dari chrono ke u8
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

pub fn calculate_next_gps_state(state: &mut GpsState) {
    let dt_seconds = state.calculation_rate_ms as f64 / 1000.0;
    let speed_mps = state.sog * 0.514444;
    let distance = speed_mps * dt_seconds;

    let lat_rad = deg_to_rad(state.latitude);
    let lon_rad = deg_to_rad(state.longitude);
    let course_rad = deg_to_rad(state.cog);
    let angular_distance = distance / EARTH_RADIUS;

    let new_lat_rad = (lat_rad.sin() * angular_distance.cos()
        + lat_rad.cos() * angular_distance.sin() * course_rad.cos())
    .asin();

    let new_lon_rad = lon_rad
        + (course_rad.sin() * angular_distance.sin() * lat_rad.cos())
            .atan2(angular_distance.cos() - lat_rad.sin() * new_lat_rad.sin());

    state.latitude = rad_to_deg(new_lat_rad).clamp(-90.0, 90.0);
    state.longitude = (rad_to_deg(new_lon_rad) + 180.0).rem_euclid(360.0) - 180.0;
    state.last_update = Utc::now();
    state.variation = calculate_magnetic_variation(state.latitude, state.longitude, &state.last_update);
}