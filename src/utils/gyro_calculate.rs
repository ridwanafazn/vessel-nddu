use crate::data::gyro_data::GyroData;
use rand_distr::{Distribution, Normal};
use std::f64::consts::PI;

/// Normalisasi heading ke [0, 360)
fn normalize_heading(mut heading: f64) -> f64 {
    while heading < 0.0 {
        heading += 360.0;
    }
    while heading >= 360.0 {
        heading -= 360.0;
    }
    heading
}

/// Clamp nilai ke rentang tertentu
fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Update field last_update dengan timestamp UTC saat ini
fn update_last_update_time(data: &mut GyroData) {
    data.last_update = chrono::Utc::now();
}

/// Fungsi utama update Gyro untuk simulasi
pub fn update_gyro_data(mut data: GyroData) -> GyroData {
    // Δt dalam detik
    let dt = data.update_rate as f64 / 1000.0;

    // Clamp laju perubahan heading_rate (misalnya ±50 deg/s)
    let heading_rate = clamp(data.heading_rate, -50.0, 50.0);

    // Integrasi heading
    let mut heading = data.heading_true + heading_rate * dt;
    heading = normalize_heading(heading);
    data.heading_true = heading;

    // Pitch & Roll sederhana (gunakan nilai user + sedikit noise sinusoidal/random)
    // Misalnya simulasi ombak ±2° untuk roll, ±1° untuk pitch
    let t = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;

    // Gaussian noise kecil
    let normal = Normal::new(0.0, 0.05).unwrap(); // stddev 0.05°
    let noise: f64 = normal.sample(&mut rand::rng());

    // Roll simulasi: baseline roll + osilasi kecil
    let roll_wave = 2.0 * (2.0 * PI * t / 8.0).sin(); // amplitudo 2°, periode 8s
    data.roll = clamp(data.roll + roll_wave + noise, -60.0, 60.0);

    // Pitch simulasi: baseline pitch + osilasi kecil
    let pitch_wave = 1.0 * (2.0 * PI * t / 10.0).sin(); // amplitudo 1°, periode 10s
    data.pitch = clamp(data.pitch + pitch_wave + noise, -30.0, 30.0);

    // Update waktu terakhir
    update_last_update_time(&mut data);

    data
}
