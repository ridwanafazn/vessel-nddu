use crate::data::navigation_data::{DepthState, SpeedState};

/// Calculate next speed state (simulation)
/// Range: 15-55 knots
pub fn calculate_next_speed_state(state: &mut SpeedState) {
    // Random walk within bounds
    let delta = (rand::random::<f64>() - 0.5) * 1.0; // ±0.5 knots per step
    state.speed_through_water += delta;

    state.speed_through_water = state.speed_through_water.clamp(15.0, 55.0);

    state.last_update = chrono::Utc::now();
}

/// Calculate next depth state (simulation)
/// Range: 500-1000 meters from surface
pub fn calculate_next_depth_state(state: &mut DepthState) {
    // Random walk within bounds
    let delta = (rand::random::<f64>() - 0.5) * 5.0; // ±2.5 meters per step
    state.ocean_water_depth += delta;

    state.ocean_water_depth = state.ocean_water_depth.clamp(500.0, 1000.0);

    // Recalculate UKC based on new depth
    state.calculate_ukc();
    state.last_update = chrono::Utc::now();
}
