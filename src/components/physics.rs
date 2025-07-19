use bevy::prelude::*;

/// Physics properties for objects that can experience forces
#[derive(Component)]
pub struct PhysicsObject {
    /// Cross-sectional area in m² for drag calculations
    pub cross_section: f64,
    /// Drag coefficient (dimensionless)
    pub drag_coefficient: f64,
    /// Radius for collision detection in meters
    pub collision_radius: f64,
}

impl PhysicsObject {
    pub fn new(cross_section: f64, drag_coefficient: f64, collision_radius: f64) -> Self {
        Self {
            cross_section,
            drag_coefficient,
            collision_radius,
        }
    }

    /// Create physics object for a typical satellite
    pub fn satellite(mass_kg: f64) -> Self {
        // Rough estimates based on satellite mass
        let radius = (mass_kg / 1000.0).powf(1.0/3.0); // Crude mass-to-size relationship
        Self::new(
            radius * radius * std::f64::consts::PI, // Cross section
            2.2, // Typical drag coefficient for satellites
            radius, // Collision radius
        )
    }

    /// Create physics object for debris
    pub fn debris(mass_kg: f64) -> Self {
        let radius = (mass_kg / 2000.0).powf(1.0/3.0); // Debris typically less dense
        Self::new(
            radius * radius * std::f64::consts::PI,
            2.5, // Higher drag coefficient for irregular debris
            radius,
        )
    }
}

/// Component to track collision events
#[derive(Component)]
pub struct CollisionEvent {
    pub id: u32,
    pub time: f64,
    pub position: Vec3,
    pub objects_involved: Vec<Entity>,
    pub debris_generated: Vec<Entity>,
}

impl CollisionEvent {
    pub fn new(id: u32, time: f64, position: Vec3, objects_involved: Vec<Entity>) -> Self {
        Self {
            id,
            time,
            position,
            objects_involved,
            debris_generated: Vec::new(),
        }
    }
}