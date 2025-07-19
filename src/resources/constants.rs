use bevy::prelude::*;

/// Physical and mathematical constants for the simulation
#[derive(Resource)]
pub struct Constants {
    /// Earth's mass in kg
    pub earth_mass: f64,
    /// Earth's gravitational parameter GM in m³/s²
    pub gravitational_parameter: f64,
    /// Earth's radius in km
    pub earth_radius: f64,
    /// Earth's radius in meters
    pub earth_radius_m: f64,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            earth_mass: 5.972e24,              // kg
            gravitational_parameter: 3.986004418e14, // m³/s²
            earth_radius: 6371.0,              // km
            earth_radius_m: 6.371e6,           // m
        }
    }
}

impl Constants {
    /// Calculate gravitational acceleration at distance r (in km) from Earth center
    pub fn gravity_acceleration(&self, r_km: f64) -> f64 {
        let r_m = r_km * 1000.0; // Convert to meters
        self.gravitational_parameter / (r_m * r_m)
    }

    /// Calculate orbital velocity for circular orbit at altitude h (km above surface)
    pub fn circular_velocity(&self, altitude_km: f64) -> f64 {
        let r_km = self.earth_radius + altitude_km;
        let r_m = r_km * 1000.0;
        (self.gravitational_parameter / r_m).sqrt() / 1000.0 // Return in km/s
    }

    /// Calculate escape velocity at distance r (km from center)
    pub fn escape_velocity(&self, r_km: f64) -> f64 {
        let r_m = r_km * 1000.0;
        (2.0 * self.gravitational_parameter / r_m).sqrt() / 1000.0 // Return in km/s
    }
}