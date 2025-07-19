// GPU Compute Shader for Orbital Physics
// Implements 2-body gravitational mechanics in parallel

// Orbital state structure matching the Rust GPU structure
struct OrbitalState {
    position: vec4<f32>,  // xyz = position (km), w = mass (kg)
    velocity: vec4<f32>,  // xyz = velocity (km/s), w = unused
}

// Physics parameters structure
struct PhysicsParams {
    gm: f32,           // Gravitational parameter (m³/s²)
    dt: f32,           // Time step (seconds)
    object_count: u32, // Number of objects
    _padding: u32,     // Alignment padding
}

// Bind groups
@group(0) @binding(0) var<storage, read_write> orbital_states: array<OrbitalState>;
@group(0) @binding(1) var<uniform> physics_params: PhysicsParams;

// Compute shader main function - processes orbital mechanics for each object
@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    
    // Bounds check
    if (index >= physics_params.object_count) {
        return;
    }
    
    // Load current orbital state
    let current_state = orbital_states[index];
    let position = current_state.position.xyz;
    let velocity = current_state.velocity.xyz;
    let mass = current_state.position.w;
    
    // Skip if mass is invalid
    if (mass <= 0.0) {
        return;
    }
    
    // Calculate gravitational acceleration: a = -GM * r / |r|³
    let r_magnitude_km = length(position);
    
    // Skip if position is invalid (at origin or negative)
    if (r_magnitude_km <= 0.0) {
        return;
    }
    
    // Convert to meters for calculation
    let r_magnitude_m = r_magnitude_km * 1000.0;
    let r_magnitude_cubed = r_magnitude_m * r_magnitude_m * r_magnitude_m;
    
    // Gravitational acceleration magnitude in m/s²
    let acc_magnitude = -physics_params.gm / r_magnitude_cubed;
    
    // Unit vector (direction)
    let r_unit = position / r_magnitude_km;
    
    // Acceleration vector in km/s² (convert from m/s²)
    let acceleration = r_unit * (acc_magnitude / 1000.0);
    
    // Euler integration
    let new_velocity = velocity + acceleration * physics_params.dt;
    let new_position = position + new_velocity * physics_params.dt;
    
    // Store updated orbital state
    orbital_states[index].position = vec4<f32>(new_position, mass);
    orbital_states[index].velocity = vec4<f32>(new_velocity, 0.0);
}

// Alternative integration methods for future optimization
//
// Velocity Verlet integration (more accurate):
// fn velocity_verlet_integration(pos: vec3<f32>, vel: vec3<f32>, acc: vec3<f32>, dt: f32) -> vec2<vec3<f32>> {
//     let new_pos = pos + vel * dt + 0.5 * acc * dt * dt;
//     let new_acc = calculate_acceleration(new_pos);
//     let new_vel = vel + 0.5 * (acc + new_acc) * dt;
//     return vec2<vec3<f32>>(new_pos, new_vel);
// }
//
// Leapfrog integration (symplectic, energy-conserving):
// fn leapfrog_integration(pos: vec3<f32>, vel: vec3<f32>, acc: vec3<f32>, dt: f32) -> vec2<vec3<f32>> {
//     let new_vel = vel + acc * dt;
//     let new_pos = pos + new_vel * dt;
//     return vec2<vec3<f32>>(new_pos, new_vel);
// }