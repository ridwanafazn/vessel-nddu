use crate::data::gyro_data::GyroData;
// PERBAIKAN: Tambahkan `use` untuk thread_rng
use rand::{thread_rng, Rng}; 
use rand_distr::{Distribution, Normal};
use std::f64::consts::PI;

/// Normalisasi yaw ke [0, 360)
fn normalize_yaw(mut yaw: f64) -> f64 {
    while yaw < 0.0 {
        yaw += 360.0;
    }
    while yaw >= 360.0 {
        yaw -= 360.0;
    }
    yaw
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

/// Fungsi utama untuk memperbarui data Gyro (simulasi rotasi & noise)
/// **FUNGSI INI TELAH DIPERBAIKI**
pub fn update_gyro_data(mut data: GyroData) -> GyroData {
    // Δt dalam detik (dari update_rate dalam ms)
    let dt = data.update_rate as f64 / 1000.0;

    // PERBAIKAN: Menyesuaikan nama variabel untuk konsistensi.
    // Pastikan field di struct GyroData juga diubah dari `yaw_rate` ke `heading_rate`.
    let heading_rate = clamp(data.yaw_rate, -50.0, 50.0);

    // Integrasi yaw (rotasi)
    let mut yaw = data.yaw + heading_rate * dt;
    yaw = normalize_yaw(yaw);
    data.yaw = yaw;

    // Simulasi dinamika Pitch & Roll yang realistis
    let t = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;

    // PERBAIKAN: Menggunakan rand::thread_rng() untuk generator acak yang lebih aman.
    let mut rng = thread_rng();
    
    // Gaussian noise kecil (stddev 0.05°)
    let normal = Normal::new(0.0, 0.05).unwrap();
    let noise: f64 = normal.sample(&mut rng);

    // === PERBAIKAN: Logika Pitch & Roll ===
    // Dihitung sebagai osilasi sinusoidal murni berpusat di nol, bukan integrasi.
    // Ini mensimulasikan gerakan berayun yang kembali ke posisi setimbang.

    // Roll: osilasi ±2° dengan periode ~8 detik
    let roll_wave = 2.0 * (2.0 * PI * t / 8.0).sin();
    // Nilai roll di-set langsung, bukan ditambahkan ke nilai sebelumnya.
    data.roll = clamp(roll_wave + noise, -60.0, 60.0);

    // Pitch: osilasi ±1° dengan periode ~10 detik
    let pitch_wave = 1.0 * (2.0 * PI * t / 10.0).sin();
    // Nilai pitch di-set langsung, bukan ditambahkan ke nilai sebelumnya.
    data.pitch = clamp(pitch_wave + noise, -30.0, 30.0);

    // Perbarui waktu terakhir
    update_last_update_time(&mut data);

    data
}