use bevy::prelude::*;
use nalgebra::{Vector3, Unit};

/// Core orbital state component containing position and velocity vectors
#[derive(Component)]
pub struct OrbitalState {
    /// Position vector in kilometers from Earth center
    pub position: Vec3,
    /// Velocity vector in km/s
    pub velocity: Vec3,
    /// Object mass in kilograms
    pub mass: f64,
}

impl OrbitalState {
    pub fn new(position: Vec3, velocity: Vec3, mass: f64) -> Self {
        Self {
            position,
            velocity,
            mass,
        }
    }

    /// Calculate distance from Earth center in km
    pub fn altitude(&self) -> f64 {
        self.position.length() as f64
    }

    /// Calculate orbital speed in km/s
    pub fn speed(&self) -> f64 {
        self.velocity.length() as f64
    }

    /// Calculate kinetic energy in Joules
    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass * (self.speed() * 1000.0).powi(2) // Convert km/s to m/s
    }

    /// Calculate gravitational potential energy in Joules
    pub fn potential_energy(&self, gm: f64) -> f64 {
        let r_meters = self.altitude() * 1000.0; // Convert km to meters
        -gm * self.mass / r_meters
    }

    /// Calculate total orbital energy in Joules
    pub fn total_energy(&self, gm: f64) -> f64 {
        self.kinetic_energy() + self.potential_energy(gm)
    }
}

/// Component to store the original TLE data for reference
#[derive(Component)]
pub struct TleData {
    pub norad_id: u32,
    pub name: String,
    pub line1: String,
    pub line2: String,
    pub epoch: f64,
}

impl TleData {
    pub fn new(norad_id: u32, name: String, line1: String, line2: String, epoch: f64) -> Self {
        Self {
            norad_id,
            name,
            line1,
            line2,
            epoch,
        }
    }
}