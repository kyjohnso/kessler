// SGP4 wrapper - Simplified implementation for TLE to state vector conversion
// For Phase 2, we'll implement a working version using basic orbital mechanics
// This can be enhanced with full SGP4 later

use crate::utils::TleRecord;
use bevy::prelude::Vec3;
use std::f64::consts::PI;

/// Convert TLE data to initial position/velocity state vectors
/// Uses simplified orbital mechanics (can be upgraded to full SGP4 later)
pub fn tle_to_state_vectors(tle: &TleRecord) -> Result<(Vec3, Vec3), String> {
    // Convert orbital elements to Cartesian coordinates
    // This is a simplified implementation - full SGP4 would be more accurate
    
    // Convert angles from degrees to radians
    let inclination = tle.inclination.to_radians();
    let raan = tle.right_ascension.to_radians(); // Right Ascension of Ascending Node
    let arg_perigee = tle.argument_of_perigee.to_radians();
    let mean_anomaly = tle.mean_anomaly.to_radians();
    let eccentricity = tle.eccentricity;
    
    // Earth's gravitational parameter (km³/s²)
    let mu = 398600.4418;
    
    // Calculate semi-major axis from mean motion
    // n = sqrt(mu/a³) => a = (mu/n²)^(1/3)
    let mean_motion_rad_per_sec = tle.mean_motion * 2.0 * PI / 86400.0; // Convert rev/day to rad/s
    let semi_major_axis = (mu / (mean_motion_rad_per_sec * mean_motion_rad_per_sec)).powf(1.0/3.0);
    
    // Solve Kepler's equation: E - e*sin(E) = M
    // Using Newton's method for eccentric anomaly
    let eccentric_anomaly = solve_keplers_equation(mean_anomaly, eccentricity)?;
    
    // Calculate true anomaly
    let true_anomaly = 2.0 * ((1.0 + eccentricity)/(1.0 - eccentricity)).sqrt().atan() *
                       (eccentric_anomaly / 2.0).tan();
    
    // Calculate distance from Earth center
    let radius = semi_major_axis * (1.0 - eccentricity * eccentric_anomaly.cos());
    
    // Position in orbital plane
    let cos_true_anom = true_anomaly.cos();
    let sin_true_anom = true_anomaly.sin();
    
    let pos_orbital = Vec3::new(
        (radius * cos_true_anom) as f32,
        (radius * sin_true_anom) as f32,
        0.0
    );
    
    // Velocity in orbital plane (km/s)
    let h = (mu * semi_major_axis * (1.0 - eccentricity * eccentricity)).sqrt(); // Angular momentum
    let vel_orbital = Vec3::new(
        (-mu / h * sin_true_anom) as f32,
        (mu / h * (eccentricity + cos_true_anom)) as f32,
        0.0
    );
    
    // Rotation matrices to convert from orbital plane to ECI coordinates
    let cos_raan = raan.cos();
    let sin_raan = raan.sin();
    let cos_inc = inclination.cos();
    let sin_inc = inclination.sin();
    let cos_arg = arg_perigee.cos();
    let sin_arg = arg_perigee.sin();
    
    // Combined rotation matrix elements
    let p11 = cos_raan * cos_arg - sin_raan * sin_arg * cos_inc;
    let p12 = -cos_raan * sin_arg - sin_raan * cos_arg * cos_inc;
    let p13 = sin_raan * sin_inc;
    
    let p21 = sin_raan * cos_arg + cos_raan * sin_arg * cos_inc;
    let p22 = -sin_raan * sin_arg + cos_raan * cos_arg * cos_inc;
    let p23 = -cos_raan * sin_inc;
    
    let p31 = sin_arg * sin_inc;
    let p32 = cos_arg * sin_inc;
    let p33 = cos_inc;
    
    // Transform position to ECI coordinates
    let position = Vec3::new(
        (p11 * pos_orbital.x as f64 + p12 * pos_orbital.y as f64 + p13 * pos_orbital.z as f64) as f32,
        (p21 * pos_orbital.x as f64 + p22 * pos_orbital.y as f64 + p23 * pos_orbital.z as f64) as f32,
        (p31 * pos_orbital.x as f64 + p32 * pos_orbital.y as f64 + p33 * pos_orbital.z as f64) as f32,
    );
    
    // Transform velocity to ECI coordinates
    let velocity = Vec3::new(
        (p11 * vel_orbital.x as f64 + p12 * vel_orbital.y as f64 + p13 * vel_orbital.z as f64) as f32,
        (p21 * vel_orbital.x as f64 + p22 * vel_orbital.y as f64 + p23 * vel_orbital.z as f64) as f32,
        (p31 * vel_orbital.x as f64 + p32 * vel_orbital.y as f64 + p33 * vel_orbital.z as f64) as f32,
    );
    
    Ok((position, velocity))
}

/// Solve Kepler's equation using Newton's method
fn solve_keplers_equation(mean_anomaly: f64, eccentricity: f64) -> Result<f64, String> {
    let mut eccentric_anomaly = mean_anomaly; // Initial guess
    const MAX_ITERATIONS: u32 = 50;
    const TOLERANCE: f64 = 1e-12;
    
    for _i in 0..MAX_ITERATIONS {
        let f = eccentric_anomaly - eccentricity * eccentric_anomaly.sin() - mean_anomaly;
        let f_prime = 1.0 - eccentricity * eccentric_anomaly.cos();
        
        if f_prime.abs() < TOLERANCE {
            return Err("Division by zero in Kepler's equation solution".to_string());
        }
        
        let delta = f / f_prime;
        eccentric_anomaly -= delta;
        
        if delta.abs() < TOLERANCE {
            return Ok(eccentric_anomaly);
        }
    }
    
    Err("Failed to converge in Kepler's equation solution".to_string())
}

/// Get current Julian day number for epoch calculations
pub fn current_julian_day() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    
    // Convert Unix timestamp to Julian Day Number
    // Unix epoch (1970-01-01) is JD 2440587.5
    2440587.5 + unix_time / 86400.0
}

/// Calculate time difference between TLE epoch and current time in days
pub fn days_since_epoch(tle: &TleRecord) -> f64 {
    let current_jd = current_julian_day();
    
    // Convert TLE epoch to Julian Day
    let year = if tle.epoch_year < 57 {
        tle.epoch_year + 2000
    } else {
        tle.epoch_year + 1900
    } as f64;
    
    // Calculate Julian Day for TLE epoch
    // Approximate calculation - good enough for this phase
    let tle_jd = 365.25 * (year - 1.0) + tle.epoch_day + 1721425.5;
    
    current_jd - tle_jd
}

/// Propagate satellite using simplified orbital mechanics
/// This advances the satellite from its TLE epoch to the current time
pub fn propagate_to_current_time(tle: &TleRecord) -> Result<(Vec3, Vec3), String> {
    // Get position/velocity at TLE epoch
    let (mut pos, mut vel) = tle_to_state_vectors(tle)?;
    
    // Calculate time elapsed since TLE epoch
    let days_elapsed = days_since_epoch(tle);
    let seconds_elapsed = days_elapsed * 86400.0;
    
    // For now, use simple Keplerian propagation
    // This could be enhanced with perturbation models later
    
    // Calculate mean motion in rad/s
    let mean_motion_rad_per_sec = tle.mean_motion * 2.0 * PI / 86400.0;
    
    // Advance mean anomaly
    let delta_mean_anomaly = mean_motion_rad_per_sec * seconds_elapsed;
    
    // Create updated TLE with advanced mean anomaly
    let mut updated_tle = tle.clone();
    updated_tle.mean_anomaly = (tle.mean_anomaly + delta_mean_anomaly.to_degrees()) % 360.0;
    
    // Recalculate position and velocity
    tle_to_state_vectors(&updated_tle)
}

/// Convert TLE data to simplified orbital state (for immediate use)
/// This provides basic orbital mechanics without full SGP4 complexity
pub fn tle_to_simple_orbit(tle: &TleRecord) -> Result<(Vec3, Vec3, f64), String> {
    let (pos, vel) = tle_to_state_vectors(tle)?;
    
    // Calculate orbital period in seconds
    let mean_motion_rad_per_sec = tle.mean_motion * 2.0 * PI / 86400.0;
    let orbital_period = 2.0 * PI / mean_motion_rad_per_sec;
    
    Ok((pos, vel, orbital_period))
}