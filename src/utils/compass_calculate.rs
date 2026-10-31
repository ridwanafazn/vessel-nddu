use crate::data::compass_data::CompassState;
use rand::Rng;

pub fn calculate_compass(state: &mut CompassState) {
    let mut rng = rand::rng();
    
    // Random walk for heading: ±5 degrees per step
    let delta: f64 = rng.random_range(-5.0..=5.0);
    state.heading += delta;
    
    // Normalize heading to 0-360 range
    if state.heading < 0.0 {
        state.heading += 360.0;
    } else if state.heading >= 360.0 {
        state.heading -= 360.0;
    }
}
