use bevy::prelude::*;

/// Component for active satellites
#[derive(Component)]
pub struct Satellite {
    pub name: String,
    pub norad_id: u32,
    pub active: bool,
}

impl Satellite {
    pub fn new(name: String, norad_id: u32, active: bool) -> Self {
        Self {
            name,
            norad_id,
            active,
        }
    }
}

/// Component for debris objects
#[derive(Component)]
pub struct Debris {
    /// ID of the collision event that created this debris
    pub parent_collision: Option<u32>,
    /// Generation number (0 = original object, 1 = first-gen debris, etc.)
    pub generation: u32,
    /// When this debris was created (simulation time)
    pub creation_time: f64,
}

impl Debris {
    pub fn new(parent_collision: Option<u32>, generation: u32, creation_time: f64) -> Self {
        Self {
            parent_collision,
            generation,
            creation_time,
        }
    }

    /// Create debris from an original satellite collision
    pub fn from_collision(collision_id: u32, creation_time: f64) -> Self {
        Self::new(Some(collision_id), 1, creation_time)
    }

    /// Create higher generation debris from existing debris
    pub fn from_debris(parent: &Debris, collision_id: u32, creation_time: f64) -> Self {
        Self::new(Some(collision_id), parent.generation + 1, creation_time)
    }
}

/// Marker component for objects that should be rendered as satellites
#[derive(Component)]
pub struct RenderAsSatellite;

/// Marker component for objects that should be rendered as debris
#[derive(Component)]
pub struct RenderAsDebris;