use crate::data::gps_data::GpsData;
use chrono::{DateTime, Utc, Datelike};
use std::f64::consts::PI;
use time::{Date, Month};
use uom::si::angle::degree;
use uom::si::f32::*;
use uom::si::length::meter;
use world_magnetic_model::GeomagneticField;

const EARTH_RADIUS_M: f64 = 6_371_000.0;

#[inline]
fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

#[inline]
fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / PI
}

/// Normalisasi heading/course agar selalu di [0, 360)
fn normalize_course(course: f64) -> f64 {
    ((course % 360.0) + 360.0) % 360.0
}

/// Batasi kecepatan (knots)
fn clamp_speed(sog: f64) -> f64 {
    sog.clamp(0.0, 102.2)
}

/// --- Magnetic Variation ---
pub fn calculate_magnetic_variation(lat: f64, lon: f64, date_time: &DateTime<Utc>) -> f64 {
    let month = Month::try_from(date_time.month() as u8).unwrap_or(Month::January);
    let day = date_time.day() as u8;
    let date = Date::from_calendar_date(date_time.year(), month, day)
        .unwrap_or_else(|_| Date::from_calendar_date(2025, Month::January, 1).unwrap());

    let height_q = Length::new::<meter>(0.0);
    let lat_q = Angle::new::<degree>(lat as f32);
    let lon_q = Angle::new::<degree>(lon as f32);

    match GeomagneticField::new(height_q, lat_q, lon_q, date) {
        Ok(field) => field.declination().get::<degree>() as f64,
        Err(_) => 0.0,
    }
}

pub fn calculate_next_gps_state(gps_data: &mut GpsData, dt_seconds: f64) {
    gps_data.sog = clamp_speed(gps_data.sog);
    gps_data.cog = normalize_course(gps_data.cog);

    let speed_mps = gps_data.sog * 0.514444;
    let distance = speed_mps * dt_seconds;

    let course_rad = deg_to_rad(gps_data.cog);

    let delta_lat = (distance / EARTH_RADIUS_M) * course_rad.cos();
    let delta_lon = if gps_data.latitude.abs() < 90.0 {
        (distance / (EARTH_RADIUS_M * deg_to_rad(gps_data.latitude).cos())) * course_rad.sin()
    } else {
        0.0
    };

    let mut new_lat = gps_data.latitude + rad_to_deg(delta_lat);
    let mut new_lon = gps_data.longitude + rad_to_deg(delta_lon);

    if new_lat > 90.0 {
        new_lat = 180.0 - new_lat;
        gps_data.cog = normalize_course(gps_data.cog + 180.0);
        new_lon = (new_lon + 180.0).rem_euclid(360.0) - 180.0;
        println!("[POLE] Crossed North Pole → Flip COG {:.2}", gps_data.cog);
    } else if new_lat < -90.0 {
        new_lat = -180.0 - new_lat;
        gps_data.cog = normalize_course(gps_data.cog + 180.0);
        new_lon = (new_lon + 180.0).rem_euclid(360.0) - 180.0;
        println!("[POLE] Crossed South Pole → Flip COG {:.2}", gps_data.cog);
    }

    new_lon = ((new_lon + 180.0).rem_euclid(360.0)) - 180.0;

    gps_data.latitude = new_lat.clamp(-90.0, 90.0);
    gps_data.longitude = new_lon;
    gps_data.last_update = Utc::now();
    gps_data.variation =
        calculate_magnetic_variation(gps_data.latitude, gps_data.longitude, &gps_data.last_update);
}