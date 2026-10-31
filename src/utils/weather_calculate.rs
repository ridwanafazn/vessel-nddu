use crate::data::weather_data::WeatherState;
use chrono::Utc;
use rand::rng;
use rand_distr::{Distribution, Normal};

/// Simulasi perubahan weather state
/// Menggunakan realistic weather patterns dengan noise
#[allow(deprecated)]
pub fn calculate_next_weather_state(state: &mut WeatherState) {
    let dt_seconds = state.calculation_rate_ms as f64 / 1000.0;
    let mut rng = rng();
    
    // ============ AIR TEMPERATURE (°C) ============
    // Simulasi dengan slow drift + small noise
    // Range normal: -10°C to 40°C
    let temp_noise = Normal::new(0.0, 0.05).unwrap(); // Small variance
    let temp_drift = temp_noise.sample(&mut rng) * dt_seconds;
    state.air_temperature = (state.air_temperature + temp_drift).clamp(-10.0, 45.0);
    
    // ============ AIR HUMIDITY (%) ============
    // Range: 0% to 100%
    let humidity_noise = Normal::new(0.0, 0.3).unwrap();
    let humidity_drift = humidity_noise.sample(&mut rng) * dt_seconds;
    state.air_humidity = (state.air_humidity + humidity_drift).clamp(0.0, 100.0);
    
    // ============ AIR PRESSURE (mbar/hPa) ============
    // Normal atmospheric pressure: 980 - 1050 mbar
    // 1013.25 mbar = standard sea level pressure
    let pressure_noise = Normal::new(0.0, 0.1).unwrap();
    let pressure_drift = pressure_noise.sample(&mut rng) * dt_seconds;
    state.air_pressure = (state.air_pressure + pressure_drift).clamp(950.0, 1060.0);
    
    // ============ WIND SPEED RELATIVE (knots) ============
    // Simulasi dengan sinusoidal wave + noise
    // Range: 0 - 50 knots (typical for navigation)
    let t = Utc::now().timestamp_millis() as f64 / 1000.0;
    
    // Base wind dengan periode ~60 detik
    let wind_base = 15.0 + 8.0 * (2.0 * std::f64::consts::PI * t / 60.0).sin();
    
    //noise untuk simulasi gusting
    let wind_noise = Normal::new(0.0, 1.5).unwrap();
    let wind_gust = wind_noise.sample(&mut rng);
    
    state.wind_velocity = (wind_base + wind_gust).max(0.0).min(50.0);
    
    // ============ WIND DIRECTION RELATIVE (°) ============
    // Relative to ship's heading (0-360°)
    // Slowly changing direction dengan noise
    let dir_noise = Normal::new(0.0, 2.0).unwrap();
    let dir_change = dir_noise.sample(&mut rng) * dt_seconds;
    
    state.wind_direction_relative = (state.wind_direction_relative + dir_change) % 360.0;
    if state.wind_direction_relative < 0.0 {
        state.wind_direction_relative += 360.0;
    }
    
    // Update timestamp
    state.last_update = Utc::now();
}

// ============ HELPER FUNCTIONS ============

/// Calculate dew point dari temperature dan humidity
#[allow(dead_code)]
pub fn calculate_dew_point(temp_celsius: f64, humidity_percent: f64) -> f64 {
    let a = 17.27;
    let b = 237.7;
    
    let alpha = ((a * temp_celsius) / (b + temp_celsius)) + (humidity_percent / 100.0).ln();
    (b * alpha) / (a - alpha)
}

/// Calculate heat index 
#[allow(dead_code)]
pub fn calculate_heat_index(temp_celsius: f64, humidity_percent: f64) -> f64 {
    if temp_celsius < 27.0 {
        return temp_celsius;
    }
    
    let t = temp_celsius;
    let rh = humidity_percent;
    
    // Heat index formula (NOAA)
    -8.78469475556
        + 1.61139411 * t
        + 2.33854883889 * rh
        + -0.14611605 * t * rh
        + -0.012308094 * t.powi(2)
        + -0.0164248277778 * rh.powi(2)
        + 0.002211732 * t.powi(2) * rh
        + 0.00072546 * t * rh.powi(2)
        + -0.000003582 * t.powi(2) * rh.powi(2)
}

/// Convert pressure units
#[allow(dead_code)]
pub fn mbar_to_atm(mbar: f64) -> f64 {
    mbar / 1013.25
}

#[allow(dead_code)]
pub fn mbar_to_mmhg(mbar: f64) -> f64 {
    mbar * 0.750062
}
