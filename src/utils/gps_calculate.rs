use crate::data::gps_data::GPSData;
use std::f64::consts::PI;
use time::Date;
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

/// Normalisasi nilai heading/course ke [0, 360)
fn normalize_course(mut course: f64) -> f64 {
    while course < 0.0 {
        course += 360.0;
    }
    while course >= 360.0 {
        course -= 360.0;
    }
    course
}

/// Clamp kecepatan ke batas 0..102.2 knot
fn clamp_speed(speed: f64) -> f64 {
    if speed < 0.0 {
        0.0
    } else if speed > 102.2 {
        102.2
    } else {
        speed
    }
}

/// Hitung posisi baru berdasarkan kecepatan (sog, knot) dan arah (cog, derajat)
/// update_rate dalam milisecond
/// **FUNGSI INI TELAH DIPERBAIKI**
pub fn calculate_new_position(data: &mut GPSData) -> (f64, f64) {
    let lat_rad = deg_to_rad(data.latitude);
    let lon_rad = deg_to_rad(data.longitude);
    let course_rad = deg_to_rad(data.cog);

    // Konversi speed dari knot ke meter/detik
    let speed_mps = data.sog * 0.514444;
    let distance = speed_mps * (data.update_rate as f64 / 1000.0); // ms -> s

    // Hitung jarak angular
    let angular_distance = distance / EARTH_RADIUS;

    // === PERBAIKAN: Menggunakan rumus Haversine untuk menghitung posisi baru ===
    // Rumus ini akurat secara matematis untuk permukaan bola.
    let new_lat_rad = (lat_rad.sin() * angular_distance.cos()
        + lat_rad.cos() * angular_distance.sin() * course_rad.cos())
    .asin();

    let new_lon_rad = lon_rad
        + (course_rad.sin() * angular_distance.sin() * lat_rad.cos())
            .atan2(angular_distance.cos() - lat_rad.sin() * new_lat_rad.sin());

    let mut new_lat = rad_to_deg(new_lat_rad);
    let mut new_lon = rad_to_deg(new_lon_rad);

    // Clamp latitude ke rentang valid [-90, 90]
    if new_lat > 90.0 {
        new_lat = 90.0;
    } else if new_lat < -90.0 {
        new_lat = -90.0;
    }

    // Normalisasi longitude ke [-180, 180]
    new_lon = (new_lon + 180.0).rem_euclid(360.0) - 180.0;

    // === DIHAPUS: Logika pantulan kutub tidak lagi diperlukan dengan rumus Haversine ===
    // Rumus Haversine secara alami menangani pergerakan di dekat kutub.

    // data.cog tidak perlu diubah di sini karena COG adalah arah gerak,
    // yang diasumsikan konstan selama interval waktu kecil ini.
    (new_lat, new_lon)
}

/// Update field last_update dengan timestamp UTC saat ini
pub fn update_last_update_time(data: &mut GPSData) {
    data.last_update = chrono::Utc::now();
}

/// Hitung magnetic variation (declination) berdasarkan lat, lon, dan waktu
pub fn calculate_magnetic_variation(lat: f64, lon: f64, date_str: &str) -> f64 {
    let datetime = match time::OffsetDateTime::parse(date_str, &time::format_description::well_known::Rfc3339) {
        Ok(dt) => dt,
        Err(_) => time::OffsetDateTime::now_utc(),
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

    data
}