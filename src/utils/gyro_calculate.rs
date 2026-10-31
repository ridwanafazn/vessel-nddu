use crate::data::gyro_data::GyroState;
use chrono::Utc;
use rand::rng;
use rand_distr::{Distribution, Normal};
use std::f64::consts::PI;

fn normalize_yaw(yaw: f64) -> f64 {
    (yaw % 360.0 + 360.0) % 360.0
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

// DIUBAH: Menambahkan atribut #[allow(deprecated)] untuk menyembunyikan warning
#[allow(deprecated)]
pub fn calculate_next_gyro_state(state: &mut GyroState) {
    let dt_seconds = state.calculation_rate_ms as f64 / 1000.0;
    let new_yaw = state.yaw + state.yaw_rate * dt_seconds;
    state.yaw = normalize_yaw(new_yaw);
    let t = Utc::now().timestamp_millis() as f64 / 1000.0;
    
    // Panggilan ini yang menyebabkan warning, sekarang akan diabaikan oleh compiler.
    let mut rng = rng();
    let normal = Normal::new(0.0, 0.05).unwrap();
    let noise: f64 = normal.sample(&mut rng);

    let roll_wave = 2.0 * (2.0 * PI * t / 8.0).sin();
    state.roll = clamp(roll_wave + noise, -60.0, 60.0);
    let pitch_wave = 1.0 * (2.0 * PI * t / 10.0).sin();
    state.pitch = clamp(pitch_wave + noise, -30.0, 30.0);
    state.last_update = Utc::now();
}