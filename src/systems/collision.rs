// Collision detection system with octree spatial partitioning

use bevy::prelude::*;
use crate::components::*;
use std::collections::HashMap;

/// Octree node for spatial partitioning
#[derive(Debug, Clone)]
pub struct OctreeNode {
    /// Center of this octree node (km)
    pub center: Vec3,
    /// Half the width/height/depth of this node (km)
    pub half_size: f32,
    /// Maximum depth for subdivision
    pub max_depth: u32,
    /// Current depth of this node
    pub depth: u32,
    /// Objects contained in this node
    pub objects: Vec<Entity>,
    /// Child nodes (8 octants)
    pub children: Option<Box<[OctreeNode; 8]>>,
}

impl OctreeNode {
    /// Create a new octree node
    pub fn new(center: Vec3, half_size: f32, max_depth: u32, depth: u32) -> Self {
        Self {
            center,
            half_size,
            max_depth,
            depth,
            objects: Vec::new(),
            children: None,
        }
    }

    /// Insert an object into the octree
    pub fn insert(&mut self, entity: Entity, position: Vec3) -> bool {
        // Check if point is within this node
        if !self.contains_point(position) {
            return false;
        }

        // If we can subdivide and have too many objects, subdivide
        const MAX_OBJECTS_PER_NODE: usize = 4;
        if self.objects.len() >= MAX_OBJECTS_PER_NODE && self.depth < self.max_depth && self.children.is_none() {
            self.subdivide();
        }

        // Try to insert into children first
        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                if child.insert(entity, position) {
                    return true;
                }
            }
        }

        // If no child could contain it, or we have no children, add to this node
        self.objects.push(entity);
        true
    }

    /// Check if a point is within this node's boundaries
    fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.center.x - self.half_size && point.x <= self.center.x + self.half_size &&
        point.y >= self.center.y - self.half_size && point.y <= self.center.y + self.half_size &&
        point.z >= self.center.z - self.half_size && point.z <= self.center.z + self.half_size
    }

    /// Subdivide this node into 8 children
    fn subdivide(&mut self) {
        let quarter_size = self.half_size / 2.0;
        let new_depth = self.depth + 1;

        self.children = Some(Box::new([
            // Bottom layer (z - quarter_size)
            OctreeNode::new(self.center + Vec3::new(-quarter_size, -quarter_size, -quarter_size), quarter_size, self.max_depth, new_depth),
            OctreeNode::new(self.center + Vec3::new(quarter_size, -quarter_size, -quarter_size), quarter_size, self.max_depth, new_depth),
            OctreeNode::new(self.center + Vec3::new(-quarter_size, quarter_size, -quarter_size), quarter_size, self.max_depth, new_depth),
            OctreeNode::new(self.center + Vec3::new(quarter_size, quarter_size, -quarter_size), quarter_size, self.max_depth, new_depth),
            // Top layer (z + quarter_size)
            OctreeNode::new(self.center + Vec3::new(-quarter_size, -quarter_size, quarter_size), quarter_size, self.max_depth, new_depth),
            OctreeNode::new(self.center + Vec3::new(quarter_size, -quarter_size, quarter_size), quarter_size, self.max_depth, new_depth),
            OctreeNode::new(self.center + Vec3::new(-quarter_size, quarter_size, quarter_size), quarter_size, self.max_depth, new_depth),
            OctreeNode::new(self.center + Vec3::new(quarter_size, quarter_size, quarter_size), quarter_size, self.max_depth, new_depth),
        ]));
    }

    /// Get all objects within a sphere (for collision detection)
    pub fn query_sphere(&self, center: Vec3, radius: f32, results: &mut Vec<Entity>) {
        // Early exit if sphere doesn't intersect with this node
        if !self.sphere_intersects_cube(center, radius) {
            return;
        }

        // Add objects from this node that are within the sphere
        for &entity in &self.objects {
            results.push(entity);
        }

        // Recursively check children
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_sphere(center, radius, results);
            }
        }
    }

    /// Check if a sphere intersects with this cube
    fn sphere_intersects_cube(&self, sphere_center: Vec3, sphere_radius: f32) -> bool {
        // Find the closest point on the cube to the sphere center
        let closest = Vec3::new(
            sphere_center.x.clamp(self.center.x - self.half_size, self.center.x + self.half_size),
            sphere_center.y.clamp(self.center.y - self.half_size, self.center.y + self.half_size),
            sphere_center.z.clamp(self.center.z - self.half_size, self.center.z + self.half_size),
        );

        // Check if the distance to the closest point is less than the sphere radius
        (closest - sphere_center).length() <= sphere_radius
    }

    /// Clear all objects from this node and its children
    pub fn clear(&mut self) {
        self.objects.clear();
        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                child.clear();
            }
        }
    }
}

/// Resource to hold the octree for spatial partitioning
#[derive(Resource)]
pub struct SpatialOctree {
    pub root: OctreeNode,
}

impl Default for SpatialOctree {
    fn default() -> Self {
        // Create octree covering Earth orbit space
        // Center at origin, half-size of 50,000 km (covers LEO to GEO)
        Self {
            root: OctreeNode::new(Vec3::ZERO, 50000.0, 6, 0),
        }
    }
}

/// Collision pairs to check
#[derive(Resource, Default)]
pub struct CollisionPairs {
    pub pairs: Vec<(Entity, Entity)>,
}

/// System to update octree with current object positions
pub fn update_spatial_octree_system(
    mut octree: ResMut<SpatialOctree>,
    orbital_query: Query<(Entity, &OrbitalState), With<PhysicsObject>>,
) {
    // Clear the octree for fresh population
    octree.root.clear();
    
    // Insert all objects into the octree
    for (entity, orbital_state) in orbital_query.iter() {
        octree.root.insert(entity, orbital_state.position);
    }
}

/// Collision detection system using octree spatial partitioning
pub fn collision_detection_system(
    octree: Res<SpatialOctree>,
    mut collision_pairs: ResMut<CollisionPairs>,
    orbital_query: Query<(Entity, &OrbitalState, &PhysicsObject)>,
) {
    collision_pairs.pairs.clear();
    let mut checked_pairs = std::collections::HashSet::new();
    
    for (entity, orbital_state, physics_object) in orbital_query.iter() {
        // Query octree for nearby objects
        let mut nearby_objects = Vec::new();
        let search_radius = physics_object.collision_radius as f32 * 2.0; // Search within 2x collision radius
        
        octree.root.query_sphere(orbital_state.position, search_radius, &mut nearby_objects);
        
        // Check collisions with nearby objects
        for &other_entity in &nearby_objects {
            if entity == other_entity {
                continue;
            }
            
            // Ensure we don't check the same pair twice
            let pair = if entity.index() < other_entity.index() {
                (entity, other_entity)
            } else {
                (other_entity, entity)
            };
            
            if checked_pairs.contains(&pair) {
                continue;
            }
            
            checked_pairs.insert(pair);
            
            // Get other object's data
            if let Ok((_, other_orbital, other_physics)) = orbital_query.get(other_entity) {
                // Check if objects are close enough to collide
                let distance = (orbital_state.position - other_orbital.position).length();
                let combined_radius = (physics_object.collision_radius + other_physics.collision_radius) as f32;
                
                if distance <= combined_radius {
                    collision_pairs.pairs.push((entity, other_entity));
                    println!("COLLISION DETECTED! Distance: {:.2}km, Combined radius: {:.2}km",
                            distance, combined_radius);
                }
            }
        }
    }
    
    // Debug output for collision detection
    if !collision_pairs.pairs.is_empty() {
        println!("Detected {} collision pairs this frame", collision_pairs.pairs.len());
    }
}

/// Debris generation system
pub fn debris_generation_system(
    mut commands: Commands,
    collision_pairs: Res<CollisionPairs>,
    orbital_query: Query<(Entity, &OrbitalState, &PhysicsObject, Option<&Satellite>)>,
    mut debris_count: Local<u32>,
) {
    for &(entity1, entity2) in &collision_pairs.pairs {
        if let (Ok((_, orbital1, physics1, sat1)), Ok((_, orbital2, physics2, sat2))) =
            (orbital_query.get(entity1), orbital_query.get(entity2)) {
            
            // Calculate collision properties
            let collision_point = (orbital1.position + orbital2.position) / 2.0;
            let relative_velocity = orbital2.velocity - orbital1.velocity;
            let collision_energy = 0.5 * (orbital1.mass + orbital2.mass) as f32 * relative_velocity.length_squared();
            
            println!("COLLISION EVENT:");
            if let Some(sat1) = sat1 {
                println!("  Object 1: {} (Mass: {:.0}kg)", sat1.name, orbital1.mass);
            }
            if let Some(sat2) = sat2 {
                println!("  Object 2: {} (Mass: {:.0}kg)", sat2.name, orbital2.mass);
            }
            println!("  Collision energy: {:.2e} J", collision_energy);
            println!("  Location: ({:.1}, {:.1}, {:.1}) km", collision_point.x, collision_point.y, collision_point.z);
            
            // Generate debris based on collision energy
            // NASA standard breakup model: more energy = more debris
            let debris_pieces = calculate_debris_count(collision_energy, orbital1.mass + orbital2.mass);
            
            for i in 0..debris_pieces {
                *debris_count += 1;
                
                // Generate debris with random velocity distribution around collision point
                let debris_velocity = generate_debris_velocity(
                    orbital1.velocity,
                    orbital2.velocity,
                    relative_velocity.length()
                );
                
                // Create debris entity
                let debris_mass = (orbital1.mass + orbital2.mass) / debris_pieces as f64 * 0.1; // Smaller fragments
                
                commands.spawn((
                    Debris::from_collision(*debris_count, 0.0), // collision_id, creation_time
                    OrbitalState::new(collision_point, debris_velocity, debris_mass),
                    PhysicsObject::debris(debris_mass),
                    RenderAsDebris,
                ));
            }
            
            println!("  Generated {} debris pieces", debris_pieces);
            
            // Remove collided objects (they've been destroyed)
            commands.entity(entity1).despawn();
            commands.entity(entity2).despawn();
        }
    }
}

/// Calculate number of debris pieces from collision energy (NASA breakup model)
fn calculate_debris_count(collision_energy: f32, total_mass: f64) -> u32 {
    // Simplified NASA standard breakup model
    // More massive objects and higher energy create more debris
    let base_debris = (total_mass / 1000.0).sqrt() as u32; // Base on mass
    let energy_multiplier = (collision_energy / 1e12).sqrt().min(10.0) as u32; // Energy scaling
    
    (base_debris + energy_multiplier).clamp(2, 50) // Minimum 2, maximum 50 pieces
}

/// Generate debris velocity with realistic distribution
fn generate_debris_velocity(vel1: Vec3, vel2: Vec3, relative_speed: f32) -> Vec3 {
    use rand::prelude::*;
    
    // Average velocity of colliding objects
    let avg_velocity = (vel1 + vel2) / 2.0;
    
    // Generate random direction for debris
    let mut rng = thread_rng();
    let theta = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
    let phi = rng.gen::<f32>() * std::f32::consts::PI;
    
    let random_dir = Vec3::new(
        phi.sin() * theta.cos(),
        phi.sin() * theta.sin(),
        phi.cos(),
    );
    
    // Debris gets kicked with fraction of relative collision speed
    let debris_kick_speed = relative_speed * rng.gen_range(0.1..0.5);
    
    avg_velocity + random_dir * debris_kick_speed
}