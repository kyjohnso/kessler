// Energy analytics system

use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// System to calculate and track energy vs altitude analytics
pub fn energy_analytics_system(
    mut analytics: ResMut<EnergyAnalytics>,
    orbital_query: Query<&OrbitalState>,
    satellite_query: Query<&Satellite>,
    debris_query: Query<&Debris>,
    constants: Res<Constants>,
) {
    // Clear previous frame's measurements
    analytics.clear_measurements();

    let mut total_energy = 0.0;
    let mut total_objects = 0;

    for orbital_state in orbital_query.iter() {
        let altitude = orbital_state.altitude() - constants.earth_radius;
        let energy = orbital_state.total_energy(constants.gravitational_parameter);
        
        analytics.add_energy_measurement(altitude, energy);
        total_energy += energy;
        total_objects += 1;
    }

    // Count satellites and debris separately
    analytics.total_satellites = satellite_query.iter().count();
    analytics.total_debris = debris_query.iter().count();
    analytics.total_objects = total_objects;
    analytics.total_energy = total_energy;
}

/// Debug system to print analytics information
pub fn debug_analytics_system(
    analytics: Res<EnergyAnalytics>,
    sim_time: Res<SimulationTime>,
    mut last_print: Local<f64>,
) {
    // Print analytics every 10 seconds
    if sim_time.current - *last_print > 10.0 {
        *last_print = sim_time.current;
        
        println!("=== Energy Analytics ===");
        println!("Total Objects: {}", analytics.total_objects);
        println!("Satellites: {}", analytics.total_satellites);
        println!("Debris: {}", analytics.total_debris);
        println!("Total System Energy: {:.2e} J", analytics.total_energy);
        
        // Print energy by altitude bins
        for (bin_idx, &altitude) in analytics.altitude_bins.iter().enumerate() {
            if let Some(avg_energy) = analytics.get_average_energy(bin_idx) {
                println!("Altitude {:.0}km: Avg Energy {:.2e} J", altitude, avg_energy);
            }
        }
        println!("========================");
    }
}