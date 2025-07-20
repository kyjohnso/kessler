use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Main physics system implementing 2-body orbital mechanics
pub fn physics_system(
    mut orbital_query: Query<&mut OrbitalState>,
    constants: Res<Constants>,
    mut sim_time: ResMut<SimulationTime>,
    time: Res<Time>,
) {
    // Update simulation time
    sim_time.advance(time.delta_secs());

    // Don't run physics if paused
    if sim_time.paused {
        return;
    }

    let dt = sim_time.timestep;
    let gm = constants.gravitational_parameter;

    for mut orbital_state in orbital_query.iter_mut() {
        // Work with f64 precision for physics calculations
        let pos_x = orbital_state.position.x as f64;
        let pos_y = orbital_state.position.y as f64;
        let pos_z = orbital_state.position.z as f64;
        
        let vel_x = orbital_state.velocity.x as f64;
        let vel_y = orbital_state.velocity.y as f64;
        let vel_z = orbital_state.velocity.z as f64;

        // Calculate gravitational acceleration: a = -GM * r / |r|³
        let r_magnitude_km = (pos_x * pos_x + pos_y * pos_y + pos_z * pos_z).sqrt();
        let r_magnitude_m = r_magnitude_km * 1000.0; // Convert km to m
        
        if r_magnitude_m > 0.0 {
            let acc_magnitude = -gm / (r_magnitude_m * r_magnitude_m);
            
            // Unit vector components
            let r_unit_x = pos_x / r_magnitude_km;
            let r_unit_y = pos_y / r_magnitude_km;
            let r_unit_z = pos_z / r_magnitude_km;
            
            // Acceleration in km/s²
            let acc_km_s2 = acc_magnitude / 1000.0;
            let acc_x = r_unit_x * acc_km_s2;
            let acc_y = r_unit_y * acc_km_s2;
            let acc_z = r_unit_z * acc_km_s2;

            // Simple Euler integration
            let new_vel_x = vel_x + acc_x * dt;
            let new_vel_y = vel_y + acc_y * dt;
            let new_vel_z = vel_z + acc_z * dt;
            
            let new_pos_x = pos_x + new_vel_x * dt;
            let new_pos_y = pos_y + new_vel_y * dt;
            let new_pos_z = pos_z + new_vel_z * dt;

            // Update orbital state
            orbital_state.velocity = Vec3::new(
                new_vel_x as f32,
                new_vel_y as f32,
                new_vel_z as f32,
            );
            orbital_state.position = Vec3::new(
                new_pos_x as f32,
                new_pos_y as f32,
                new_pos_z as f32,
            );
        }
    }
}

/// System to handle simulation time controls
pub fn time_control_system(
    mut sim_time: ResMut<SimulationTime>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if sim_time.paused {
            sim_time.resume();
        } else {
            sim_time.pause();
        }
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        sim_time.set_speed(1.0); // Real time
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        sim_time.set_speed(60.0); // 1 minute per second
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        sim_time.set_speed(3600.0); // 1 hour per second
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        sim_time.set_speed(86400.0); // 1 day per second
    }
}

/// Debug system to print orbital information summary
pub fn debug_orbital_system(
    orbital_query: Query<(&OrbitalState, &Satellite)>,
    constants: Res<Constants>,
    sim_time: Res<SimulationTime>,
    mut last_print: Local<f64>,
) {
    // Print debug info every 30 seconds (reduced frequency)
    if sim_time.current - *last_print > 30.0 {
        *last_print = sim_time.current;
        
        let total_satellites = orbital_query.iter().count();
        if total_satellites > 0 {
            debug!("Orbital System: {} active satellites in simulation", total_satellites);
            
            // Only show detailed info for first few satellites in debug builds
            #[cfg(debug_assertions)]
            {
                let mut count = 0;
                for (orbital_state, satellite) in orbital_query.iter() {
                    if count >= 3 { // Limit to first 3 satellites
                        break;
                    }
                    let altitude = orbital_state.altitude() - constants.earth_radius;
                    let speed = orbital_state.speed();
                    let energy = orbital_state.total_energy(constants.gravitational_parameter);
                    
                    trace!(
                        "{}: Alt={:.1}km, Speed={:.2}km/s, Energy={:.2e}J",
                        satellite.name, altitude, speed, energy
                    );
                    count += 1;
                }
                if total_satellites > 3 {
                    trace!("... and {} more satellites", total_satellites - 3);
                }
            }
        }
    }
}