use crate::data::gps_data::GPSData;
use world_magnetic_model::GeomagneticField;
use uom::si::angle::degree;
use uom::si::length::meter;
use uom::si::f32::*;
use time::Date;
use std::f64::consts::PI;

/// Radius bumi dalam meter
const EARTH_RADIUS: f64 = 6_371_000.0;

/// Konversi derajat ke radian
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

/// Konversi radian ke derajat
pub fn rad_to_deg(rad: f64) -> f64 {
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

/// Hitung posisi baru berdasarkan kecepatan (knot) dan arah (course_over_ground)
/// update_rate dalam milisecond
pub fn calculate_new_position(data: &mut GPSData) -> (f64, f64) {
    let mut lat = data.latitude;
    let mut lon = data.longitude;
    let mut course = data.course_over_ground;

    // konversi speed dari knot ke meter/detik
    let speed_mps = data.speed_over_ground * 0.514444;
    let distance = speed_mps * (data.update_rate as f64 / 1000.0); // ms -> s

    // Delta linear pada sumbu latitude dan longitude
    let course_rad = deg_to_rad(course);
    let delta_lat = distance / EARTH_RADIUS * course_rad.cos();
    let delta_lon = if lat.abs() < 90.0 {
        distance / (EARTH_RADIUS * deg_to_rad(lat).cos()) * course_rad.sin()
    } else {
        0.0
    };

    // Tambah delta ke posisi saat ini
    lat += rad_to_deg(delta_lat);
    lon += rad_to_deg(delta_lon);

    // Pantulan di kutub
    if lat > 90.0 {
        lat = 180.0 - lat;
        course = normalize_course(course + 180.0);
    } else if lat < -90.0 {
        lat = -180.0 - lat;
        course = normalize_course(course + 180.0);
    }

    // Normalisasi longitude ke [-180, 180]
    lon = ((lon + 180.0).rem_euclid(360.0)) - 180.0;

    data.course_over_ground = course;
    (lat, lon)
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
    let year = datetime.year().clamp(2020, 2029);
    let month = datetime.month();
    let day = datetime.day();
    let date = Date::from_calendar_date(year, month, day)
        .unwrap_or(Date::from_calendar_date(2025, month, day).unwrap());

    let height_q = Length::new::<meter>(0.0);
    let lat_q = Angle::new::<degree>(lat as f32);
    let lon_q = Angle::new::<degree>(lon as f32);

    match GeomagneticField::new(height_q, lat_q, lon_q, date) {
        Ok(field) => field.declination().get::<degree>() as f64,
        Err(_) => 0.0,
    }
}

/// Fungsi utama update GPS untuk simulasi
pub fn update_gps_data(mut data: GPSData) -> GPSData {
    // Normalisasi input dulu
    data.course_over_ground = normalize_course(data.course_over_ground);
    data.speed_over_ground = clamp_speed(data.speed_over_ground);

    // Hitung posisi baru
    let (new_lat, new_lon) = calculate_new_position(&mut data);
    data.latitude = new_lat;
    data.longitude = new_lon;

    // Update waktu
    update_last_update_time(&mut data);

    // Hitung magnetic variation baru
    data.magnetic_variation = Some(calculate_magnetic_variation(
        data.latitude,
        data.longitude,
        &data.last_update.to_rfc3339(),
    ));

    data
}
