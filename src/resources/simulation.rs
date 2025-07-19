use bevy::prelude::*;
use std::collections::HashMap;

/// Global simulation time and control
#[derive(Resource)]
pub struct SimulationTime {
    /// Current simulation time in seconds since epoch
    pub current: f64,
    /// Time speed multiplier (1.0 = real time, 3600.0 = 1 hour per second)
    pub speed_multiplier: f64,
    /// Whether the simulation is paused
    pub paused: bool,
    /// Simulation timestep in seconds
    pub timestep: f64,
}

impl Default for SimulationTime {
    fn default() -> Self {
        Self {
            current: 0.0,
            speed_multiplier: 3600.0, // Default to 1 hour per second
            paused: false,
            timestep: 1.0, // 1 second timesteps
        }
    }
}

impl SimulationTime {
    pub fn advance(&mut self, delta_time: f32) {
        if !self.paused {
            self.current += delta_time as f64 * self.speed_multiplier;
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn set_speed(&mut self, multiplier: f64) {
        self.speed_multiplier = multiplier.max(0.0);
    }
}

/// Resource for tracking energy analytics
#[derive(Resource)]
pub struct EnergyAnalytics {
    /// Altitude bins for energy averaging (in km)
    pub altitude_bins: Vec<f64>,
    /// Energy values binned by altitude
    pub energy_by_altitude: HashMap<usize, Vec<f64>>,
    /// Total number of tracked objects
    pub total_objects: usize,
    /// Number of active satellites
    pub total_satellites: usize,
    /// Number of debris objects
    pub total_debris: usize,
    /// Total system energy
    pub total_energy: f64,
}

impl Default for EnergyAnalytics {
    fn default() -> Self {
        // Create altitude bins from 200km to 2000km in 50km increments
        let mut altitude_bins = Vec::new();
        for i in 0..36 {
            altitude_bins.push(200.0 + i as f64 * 50.0);
        }

        Self {
            altitude_bins,
            energy_by_altitude: HashMap::new(),
            total_objects: 0,
            total_satellites: 0,
            total_debris: 0,
            total_energy: 0.0,
        }
    }
}

impl EnergyAnalytics {
    /// Find the appropriate bin index for a given altitude
    pub fn get_altitude_bin(&self, altitude_km: f64) -> Option<usize> {
        for (i, &bin_altitude) in self.altitude_bins.iter().enumerate() {
            if altitude_km < bin_altitude + 25.0 && altitude_km >= bin_altitude - 25.0 {
                return Some(i);
            }
        }
        None
    }

    /// Add energy measurement for a specific altitude
    pub fn add_energy_measurement(&mut self, altitude_km: f64, energy: f64) {
        if let Some(bin_index) = self.get_altitude_bin(altitude_km) {
            self.energy_by_altitude
                .entry(bin_index)
                .or_insert_with(Vec::new)
                .push(energy);
        }
    }

    /// Get average energy for a specific altitude bin
    pub fn get_average_energy(&self, bin_index: usize) -> Option<f64> {
        self.energy_by_altitude.get(&bin_index).map(|energies| {
            energies.iter().sum::<f64>() / energies.len() as f64
        })
    }

    /// Clear all measurements (called each frame)
    pub fn clear_measurements(&mut self) {
        self.energy_by_altitude.clear();
        self.total_objects = 0;
        self.total_satellites = 0;
        self.total_debris = 0;
        self.total_energy = 0.0;
    }
}