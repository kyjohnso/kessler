use bevy::prelude::*;
use rand::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Component to mark stress test objects
#[derive(Component)]
pub struct StressTestObject {
    pub orbit_type: OrbitType,
}

impl StressTestObject {
    pub fn new(orbit_type: OrbitType) -> Self {
        Self { orbit_type }
    }
}

/// Orbital types for satellite distribution
#[derive(Clone, Copy, Debug)]
pub enum OrbitType {
    LEO,  // Low Earth Orbit: 160-2000km
    MEO,  // Medium Earth Orbit: 2000-35,786km
    GEO,  // Geostationary Earth Orbit: ~35,786km
}

impl OrbitType {
    fn altitude_range(&self) -> (f32, f32) {
        match self {
            OrbitType::LEO => (160.0, 2000.0),
            OrbitType::MEO => (2000.0, 35786.0),
            OrbitType::GEO => (35786.0, 35786.0), // Fixed altitude for GEO
        }
    }
}

/// Resource to control stress test parameters
#[derive(Resource)]
pub struct StressTestConfig {
    pub target_objects: usize,
    pub current_objects: usize,
    pub spawn_rate: usize, // Objects to spawn per frame
    pub enabled: bool,
    // Orbital distribution targets
    pub target_leo: usize,
    pub target_meo: usize,
    pub target_geo: usize,
    // Current counts by orbit type
    pub current_leo: usize,
    pub current_meo: usize,
    pub current_geo: usize,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            target_objects: 800, // Total satellites
            current_objects: 0,
            spawn_rate: 50, // Spawn 50 objects per frame
            enabled: true,  // Enable by default to start with 1000 satellites
            // Distribution: 800 LEO, 100 MEO, 100 GEO
            target_leo: 600,
            target_meo: 0,
            target_geo: 200,
            current_leo: 0,
            current_meo: 0,
            current_geo: 0,
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
            info!("Stress test ENABLED - spawning {} satellites (LEO: {}, MEO: {}, GEO: {})",
                  config.target_objects, config.target_leo, config.target_meo, config.target_geo);
        } else {
            info!("Stress test DISABLED");
        }
    }

    // Adjust target object count - keep the same distribution ratios
    if keyboard.just_pressed(KeyCode::Digit5) {
        config.target_objects = 500;
        config.target_leo = 400;  // 80%
        config.target_meo = 50;   // 10%
        config.target_geo = 50;   // 10%
        info!("Target objects: {} (LEO: {}, MEO: {}, GEO: {})",
              config.target_objects, config.target_leo, config.target_meo, config.target_geo);
    }
    if keyboard.just_pressed(KeyCode::Digit6) {
        config.target_objects = 1000;
        config.target_leo = 800;  // 80%
        config.target_meo = 100;  // 10%
        config.target_geo = 100;  // 10%
        info!("Target objects: {} (LEO: {}, MEO: {}, GEO: {})",
              config.target_objects, config.target_leo, config.target_meo, config.target_geo);
    }
    if keyboard.just_pressed(KeyCode::Digit7) {
        config.target_objects = 2000;
        config.target_leo = 1600; // 80%
        config.target_meo = 200;  // 10%
        config.target_geo = 200;  // 10%
        info!("Target objects: {} (LEO: {}, MEO: {}, GEO: {})",
              config.target_objects, config.target_leo, config.target_meo, config.target_geo);
    }
    if keyboard.just_pressed(KeyCode::Digit8) {
        config.target_objects = 5000;
        config.target_leo = 4000; // 80%
        config.target_meo = 500;  // 10%
        config.target_geo = 500;  // 10%
        info!("Target objects: {} (LEO: {}, MEO: {}, GEO: {})",
              config.target_objects, config.target_leo, config.target_meo, config.target_geo);
    }

    if !config.enabled {
        return;
    }

    // Count existing objects by orbit type
    config.current_leo = 0;
    config.current_meo = 0;
    config.current_geo = 0;
    
    for obj in existing_objects.iter() {
        match obj.orbit_type {
            OrbitType::LEO => config.current_leo += 1,
            OrbitType::MEO => config.current_meo += 1,
            OrbitType::GEO => config.current_geo += 1,
        }
    }
    
    config.current_objects = config.current_leo + config.current_meo + config.current_geo;

    // Spawn objects if we haven't reached targets
    let mut spawned = 0;
    
    // Spawn LEO satellites
    if config.current_leo < config.target_leo {
        let to_spawn = (config.target_leo - config.current_leo).min(config.spawn_rate / 3);
        for _ in 0..to_spawn {
            spawn_orbital_satellite(&mut commands, OrbitType::LEO);
            spawned += 1;
        }
    }
    
    // Spawn MEO satellites
    if config.current_meo < config.target_meo {
        let to_spawn = (config.target_meo - config.current_meo).min(config.spawn_rate / 3);
        for _ in 0..to_spawn {
            spawn_orbital_satellite(&mut commands, OrbitType::MEO);
            spawned += 1;
        }
    }
    
    // Spawn GEO satellites
    if config.current_geo < config.target_geo {
        let to_spawn = (config.target_geo - config.current_geo).min(config.spawn_rate / 3);
        for _ in 0..to_spawn {
            spawn_orbital_satellite(&mut commands, OrbitType::GEO);
            spawned += 1;
        }
    }

    if spawned > 0 {
        info!("Spawned {} satellites - LEO: {}/{}, MEO: {}/{}, GEO: {}/{}",
              spawned, config.current_leo, config.target_leo,
              config.current_meo, config.target_meo, config.current_geo, config.target_geo);
    }
}

/// Create a satellite in the specified orbital type
fn spawn_orbital_satellite(commands: &mut Commands, orbit_type: OrbitType) {
    let mut rng = thread_rng();
    
    // Get altitude range for this orbit type
    let (min_alt, max_alt) = orbit_type.altitude_range();
    let altitude = if min_alt == max_alt {
        // GEO - fixed altitude
        min_alt
    } else {
        // LEO or MEO - random within range
        rng.gen_range(min_alt..max_alt)
    };
    
    let earth_radius = 6371.0; // km
    let orbital_radius = earth_radius + altitude;
    
    // Orbital inclination based on orbit type
    let inclination = match orbit_type {
        OrbitType::LEO => rng.gen_range(0.0..180.0_f32).to_radians(), // Any inclination
        OrbitType::MEO => rng.gen_range(55.0..65.0_f32).to_radians(), // Common MEO inclinations
        OrbitType::GEO => 0.0, // Equatorial orbit
    };
    
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
    
    // Satellite mass based on orbit type
    let mass = match orbit_type {
        OrbitType::LEO => rng.gen_range(100.0..2000.0), // Small to medium satellites
        OrbitType::MEO => rng.gen_range(500.0..3000.0), // GPS-like satellites
        OrbitType::GEO => rng.gen_range(2000.0..8000.0), // Large geostationary satellites
    };
    
    // Spawn the satellite entity
    commands.spawn((
        OrbitalState::new(position, velocity, mass),
        PhysicsObject::satellite(mass),
        Satellite::new(format!("{:?} Satellite", orbit_type), 0, true),
        StressTestObject::new(orbit_type),
        RenderAsSatellite, // Render as green satellite
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