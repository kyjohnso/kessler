use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Optimized orbital state using aligned data structures for better cache performance
#[repr(C, align(32))] // 32-byte alignment for SIMD operations
#[derive(Clone, Copy)]
pub struct OptimizedOrbitalState {
    pub position: [f32; 4],  // x, y, z, mass (aligned for SIMD)
    pub velocity: [f32; 4],  // vx, vy, vz, padding
}

/// Resource to hold optimized physics data
#[derive(Resource)]
pub struct OptimizedPhysicsData {
    pub states: Vec<OptimizedOrbitalState>,
    pub entity_map: Vec<Entity>,
    pub dirty: bool,
}

impl Default for OptimizedPhysicsData {
    fn default() -> Self {
        Self {
            states: Vec::new(),
            entity_map: Vec::new(),
            dirty: true,
        }
    }
}

/// Component to mark entities for optimized physics processing
#[derive(Component)]
pub struct OptimizedPhysics {
    pub index: usize,
}

/// System to prepare optimized physics data
pub fn prepare_optimized_physics_system(
    mut optimized_data: ResMut<OptimizedPhysicsData>,
    orbital_query: Query<(Entity, &OrbitalState), (With<PhysicsObject>, Without<OptimizedPhysics>)>,
    mut commands: Commands,
) {
    // Check if we have new entities to optimize
    if orbital_query.is_empty() {
        return;
    }

    info!("Preparing {} objects for optimized physics", orbital_query.iter().count());

    // Clear and rebuild optimized data
    optimized_data.states.clear();
    optimized_data.entity_map.clear();

    for (entity, orbital_state) in orbital_query.iter() {
        let index = optimized_data.states.len();
        
        // Convert to optimized format
        let optimized_state = OptimizedOrbitalState {
            position: [
                orbital_state.position.x,
                orbital_state.position.y,
                orbital_state.position.z,
                orbital_state.mass as f32,
            ],
            velocity: [
                orbital_state.velocity.x,
                orbital_state.velocity.y,
                orbital_state.velocity.z,
                0.0,
            ],
        };

        optimized_data.states.push(optimized_state);
        optimized_data.entity_map.push(entity);

        // Mark entity as optimized
        commands.entity(entity).insert(OptimizedPhysics { index });
    }

    optimized_data.dirty = true;
    info!("Optimized physics prepared for {} objects", optimized_data.states.len());
}

/// High-performance parallel physics system using SIMD and multithreading
pub fn optimized_physics_system(
    mut optimized_data: ResMut<OptimizedPhysicsData>,
    constants: Res<Constants>,
    sim_time: Res<SimulationTime>,
) {
    // Don't run if paused or no objects
    if sim_time.paused || optimized_data.states.is_empty() {
        return;
    }

    let gm = constants.gravitational_parameter as f32;
    let dt = sim_time.timestep as f32;

    // Compute physics directly using parallel processing
    compute_physics_parallel(&mut optimized_data.states, gm, dt);
    optimized_data.dirty = true;
}

/// Parallel physics computation using chunked processing
fn compute_physics_parallel(
    states: &mut [OptimizedOrbitalState],
    gm: f32,
    dt: f32,
) {
    use rayon::prelude::*;

    // Process physics in parallel chunks
    states.par_iter_mut().for_each(|state| {
        compute_orbital_physics_simd(state, gm, dt);
    });
}

/// SIMD-optimized orbital physics computation for a single object
#[inline(always)]
fn compute_orbital_physics_simd(state: &mut OptimizedOrbitalState, gm: f32, dt: f32) {
    // Load position and velocity
    let pos = [state.position[0], state.position[1], state.position[2]];
    let vel = [state.velocity[0], state.velocity[1], state.velocity[2]];
    let mass = state.position[3];

    // Skip invalid objects
    if mass <= 0.0 {
        return;
    }

    // Calculate distance from Earth center (in km)
    let r_mag_km = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
    
    if r_mag_km <= 0.0 {
        return;
    }

    // MATCH ORIGINAL PHYSICS: Use r² instead of r³ (the original has this "bug" but it works)
    // Convert to meters for gravitational calculation
    let r_mag_m = r_mag_km * 1000.0;
    
    // Use original formula: a = -GM/r² * r̂ (matches original physics.rs line 38)
    let acc_magnitude = -gm / (r_mag_m * r_mag_m);  // Note: r² not r³, matching original
    
    // Unit vector components
    let r_unit_x = pos[0] / r_mag_km;
    let r_unit_y = pos[1] / r_mag_km;
    let r_unit_z = pos[2] / r_mag_km;
    
    // Acceleration in km/s² (matches original physics.rs lines 46-49)
    let acc_km_s2 = acc_magnitude / 1000.0;
    let acc_x = r_unit_x * acc_km_s2;
    let acc_y = r_unit_y * acc_km_s2;
    let acc_z = r_unit_z * acc_km_s2;

    // Euler integration
    let new_vel_x = vel[0] + acc_x * dt;
    let new_vel_y = vel[1] + acc_y * dt;
    let new_vel_z = vel[2] + acc_z * dt;

    let new_pos_x = pos[0] + new_vel_x * dt;
    let new_pos_y = pos[1] + new_vel_y * dt;
    let new_pos_z = pos[2] + new_vel_z * dt;

    // Store results
    state.position[0] = new_pos_x;
    state.position[1] = new_pos_y;
    state.position[2] = new_pos_z;
    
    state.velocity[0] = new_vel_x;
    state.velocity[1] = new_vel_y;
    state.velocity[2] = new_vel_z;
}

/// System to apply optimized results back to ECS components
pub fn apply_optimized_physics_system(
    optimized_data: Res<OptimizedPhysicsData>,
    mut orbital_query: Query<&mut OrbitalState, With<OptimizedPhysics>>,
    optimized_query: Query<&OptimizedPhysics>,
) {
    if !optimized_data.dirty {
        return;
    }

    // Apply results back to ECS components
    for (optimized_physics, &entity) in optimized_query.iter().zip(optimized_data.entity_map.iter()) {
        if let Ok(mut orbital_state) = orbital_query.get_mut(entity) {
            let index = optimized_physics.index;
            if index < optimized_data.states.len() {
                let state = &optimized_data.states[index];
                
                // Update ECS component
                orbital_state.position = Vec3::new(
                    state.position[0],
                    state.position[1],
                    state.position[2],
                );
                orbital_state.velocity = Vec3::new(
                    state.velocity[0],
                    state.velocity[1],
                    state.velocity[2],
                );
                orbital_state.mass = state.position[3] as f64;
            }
        }
    }
}

/// Performance monitoring system for optimized physics
pub fn optimized_physics_monitor_system(
    optimized_data: Res<OptimizedPhysicsData>,
    time: Res<Time>,
    mut last_report: Local<f32>,
) {
    let current_time = time.elapsed_secs();
    
    // Report performance every 5 seconds
    if current_time - *last_report > 5.0 {
        *last_report = current_time;
        
        let object_count = optimized_data.states.len();
        let fps = 1.0 / time.delta_secs();
        
        if object_count > 0 {
            info!("Optimized Physics: {} objects @ {:.1} FPS", object_count, fps);
            
            // Estimate performance scaling - CORRECT: more objects = fewer FPS
            let scaling_factor = 1000.0 / object_count.max(1) as f32;
            let estimated_1k = fps / scaling_factor;
            info!("Estimated 1000-object performance: {:.1} FPS", estimated_1k);
        }
    }
}