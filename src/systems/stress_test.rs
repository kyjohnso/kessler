use bevy::prelude::*;
use rand::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Component to mark stress test objects
#[derive(Component)]
pub struct StressTestObject;

/// Resource to control stress test parameters
#[derive(Resource)]
pub struct StressTestConfig {
    pub target_objects: usize,
    pub current_objects: usize,
    pub spawn_rate: usize, // Objects to spawn per frame
    pub enabled: bool,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            target_objects: 1000, // Start with 1000 objects
            current_objects: 0,
            spawn_rate: 50, // Spawn 50 objects per frame
            enabled: false,
        }
    }
}

/// System to create stress test objects for performance testing
pub fn stress_test_spawn_system(
    mut commands: Commands,
    mut config: ResMut<StressTestConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
    existing_objects: Query<&StressTestObject>,
) {
    // Toggle stress test with 'T' key
    if keyboard.just_pressed(KeyCode::KeyT) {
        config.enabled = !config.enabled;
        if config.enabled {
            info!("Stress test ENABLED - spawning up to {} objects", config.target_objects);
        } else {
            info!("Stress test DISABLED");
        }
    }

    // Adjust target object count
    if keyboard.just_pressed(KeyCode::Digit5) {
        config.target_objects = 500;
        info!("Target objects: {}", config.target_objects);
    }
    if keyboard.just_pressed(KeyCode::Digit6) {
        config.target_objects = 1000;
        info!("Target objects: {}", config.target_objects);
    }
    if keyboard.just_pressed(KeyCode::Digit7) {
        config.target_objects = 2000;
        info!("Target objects: {}", config.target_objects);
    }
    if keyboard.just_pressed(KeyCode::Digit8) {
        config.target_objects = 5000;
        info!("Target objects: {}", config.target_objects);
    }

    if !config.enabled {
        return;
    }

    // Count existing stress test objects
    config.current_objects = existing_objects.iter().count();

    // Spawn objects if we haven't reached the target
    if config.current_objects < config.target_objects {
        let to_spawn = (config.target_objects - config.current_objects).min(config.spawn_rate);
        
        for _ in 0..to_spawn {
            spawn_random_orbital_object(&mut commands);
        }

        if to_spawn > 0 {
            info!("Spawned {} objects ({}/{})", to_spawn, config.current_objects + to_spawn, config.target_objects);
        }
    }
}

/// Create a random orbital object for stress testing
fn spawn_random_orbital_object(commands: &mut Commands) {
    let mut rng = thread_rng();
    
    // Generate random orbital parameters
    let altitude_range = (200.0..50000.0); // 200km to 50,000km altitude
    let altitude = rng.gen_range(altitude_range);
    let earth_radius = 6371.0; // km
    let orbital_radius = earth_radius + altitude;
    
    // Random orbital inclination (0 to 180 degrees)
    let inclination = rng.gen_range(0.0..180.0_f32).to_radians();
    
    // Random right ascension of ascending node
    let raan = rng.gen_range(0.0..360.0_f32).to_radians();
    
    // Random argument of perigee
    let arg_perigee = rng.gen_range(0.0..360.0_f32).to_radians();
    
    // Random true anomaly (position in orbit)
    let true_anomaly = rng.gen_range(0.0..360.0_f32).to_radians();
    
    // Calculate position in orbital plane
    let r_orbital = Vec3::new(
        orbital_radius * true_anomaly.cos(),
        orbital_radius * true_anomaly.sin(),
        0.0,
    );
    
    // Apply orbital rotations
    let position = apply_orbital_rotations(r_orbital, inclination, raan, arg_perigee);
    
    // Calculate orbital velocity (circular orbit approximation)
    let gm = 3.986004418e14; // Earth's gravitational parameter
    let orbital_speed = (gm / (orbital_radius * 1000.0)).sqrt() / 1000.0; // km/s
    
    // Velocity perpendicular to position in orbital plane
    let v_orbital = Vec3::new(
        -orbital_speed * true_anomaly.sin(),
        orbital_speed * true_anomaly.cos(),
        0.0,
    );
    
    let velocity = apply_orbital_rotations(v_orbital, inclination, raan, arg_perigee);
    
    // Random mass (small debris to large satellites)
    let mass = rng.gen_range(1.0..10000.0); // 1kg to 10 tons
    
    // Spawn the entity
    commands.spawn((
        OrbitalState::new(position, velocity, mass),
        PhysicsObject::debris(mass),
        StressTestObject,
        RenderAsDebris, // Render as red debris
    ));
}

/// Apply orbital rotations to convert from orbital plane to Earth-fixed coordinates
fn apply_orbital_rotations(vec: Vec3, inclination: f32, raan: f32, arg_perigee: f32) -> Vec3 {
    // Rotation matrices for orbital mechanics
    // This is a simplified version - full implementation would use proper rotation matrices
    
    // Rotate by argument of perigee (in orbital plane)
    let cos_w = arg_perigee.cos();
    let sin_w = arg_perigee.sin();
    let rotated_w = Vec3::new(
        vec.x * cos_w - vec.y * sin_w,
        vec.x * sin_w + vec.y * cos_w,
        vec.z,
    );
    
    // Rotate by inclination
    let cos_i = inclination.cos();
    let sin_i = inclination.sin();
    let rotated_i = Vec3::new(
        rotated_w.x,
        rotated_w.y * cos_i - rotated_w.z * sin_i,
        rotated_w.y * sin_i + rotated_w.z * cos_i,
    );
    
    // Rotate by RAAN (right ascension of ascending node)
    let cos_raan = raan.cos();
    let sin_raan = raan.sin();
    Vec3::new(
        rotated_i.x * cos_raan - rotated_i.y * sin_raan,
        rotated_i.x * sin_raan + rotated_i.y * cos_raan,
        rotated_i.z,
    )
}

/// System to clean up stress test objects
pub fn stress_test_cleanup_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    stress_objects: Query<Entity, With<StressTestObject>>,
    mut config: ResMut<StressTestConfig>,
) {
    // Clean up with 'C' key
    if keyboard.just_pressed(KeyCode::KeyC) {
        let count = stress_objects.iter().count();
        for entity in stress_objects.iter() {
            commands.entity(entity).despawn();
        }
        config.current_objects = 0;
        config.enabled = false;
        info!("Cleaned up {} stress test objects", count);
    }
}

/// Performance comparison system
pub fn performance_comparison_system(
    stress_config: Res<StressTestConfig>,
    time: Res<Time>,
    mut last_report: Local<f32>,
) {
    let current_time = time.elapsed_secs();
    
    // Report every 2 seconds when stress testing
    if stress_config.enabled && current_time - *last_report > 2.0 {
        *last_report = current_time;
        
        let fps = 1.0 / time.delta_secs();
        let frame_time_ms = time.delta_secs() * 1000.0;
        
        info!("PERFORMANCE: {} objects @ {:.1} FPS ({:.2}ms/frame)", 
              stress_config.current_objects, fps, frame_time_ms);
        
        // Performance thresholds
        if fps < 30.0 {
            warn!("Performance warning: FPS below 30 with {} objects", stress_config.current_objects);
        } else if fps > 60.0 {
            info!("Excellent performance: {} objects running at {:.1} FPS", stress_config.current_objects, fps);
        }
    }
}