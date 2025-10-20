use crate::data::gyro_data::GyroData;
use chrono::Utc;
use rand_distr::{Distribution, Normal};
use std::f64::consts::PI;

const DAMPING_FACTOR: f64 = 0.995;

fn normalize_yaw(yaw: f64) -> f64 {
    (yaw % 360.0 + 360.0) % 360.0
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

#[allow(deprecated)]
pub fn calculate_next_gyro_state(gyro_data: &mut GyroData, dt_seconds: f64) {
    let new_yaw = gyro_data.yaw + gyro_data.yaw_rate * dt_seconds;
    gyro_data.yaw = normalize_yaw(new_yaw);
    
    let t = Utc::now().timestamp_millis() as f64 / 1000.0;
    
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 0.05).unwrap();
    let noise: f64 = normal.sample(&mut rng);

    let damped_roll = gyro_data.roll * DAMPING_FACTOR;
    let roll_wave = 2.0 * (2.0 * PI * t / 8.0).sin();
    gyro_data.roll = clamp(damped_roll + roll_wave + noise, -60.0, 60.0);
    
    let damped_pitch = gyro_data.pitch * DAMPING_FACTOR;
    let pitch_wave = 1.0 * (2.0 * PI * t / 10.0).sin();
    gyro_data.pitch = clamp(damped_pitch + pitch_wave + noise, -30.0, 30.0);
    
    gyro_data.last_update = Utc::now();
}