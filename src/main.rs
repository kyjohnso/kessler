use bevy::prelude::*;
use bevy::math::primitives::Sphere;
use bevy::log::LogPlugin;

mod components;
mod resources;
mod systems;
mod utils;

use components::*;
use resources::*;
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::INFO,
            filter: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,kessler_simulator=info,bevy_render=warn,bevy_ecs=warn".to_string()),
            ..default()
        }))
        .init_resource::<Constants>()
        .init_resource::<SimulationTime>()
        .init_resource::<EnergyAnalytics>()
        .init_resource::<TleDataCache>()
        .init_resource::<SpatialOctree>()
        .init_resource::<CollisionPairs>()
        .init_resource::<OptimizedPhysicsData>()
        .init_resource::<StressTestConfig>()
        // Add ambient lighting for overall scene brightness
        .insert_resource(AmbientLight {
            color: Color::srgb(0.8, 0.9, 1.0), // Slightly blue-tinted like space
            brightness: 0.15, // Soft ambient illumination
            affects_lightmapped_meshes: true,
        })
        .add_systems(Startup, (
            setup_scene,
            initialize_tle_data_system,
        ))
        .add_systems(Update, (
            camera_control_system,
            time_control_system,
            // Original physics system (disable when using optimized)
            physics_system,
        ))
        .add_systems(Update, (
            // Optimized physics systems
            prepare_optimized_physics_system,
            optimized_physics_system,
            apply_optimized_physics_system,
            optimized_physics_monitor_system,
        ))
        .add_systems(Update, (
            // Collision and debris systems
            update_spatial_octree_system,
            collision_detection_system,
            debris_generation_system,
        ))
        .add_systems(Update, (
            // Rendering and analytics systems
            satellite_rendering_system,
            debris_rendering_system,
            update_positions_system,
            energy_analytics_system,
        ))
        .add_systems(Update, (
            // Debug and stress test systems
            debug_orbital_system,
            debug_analytics_system,
            process_tle_fetch_system,
        ))
        .add_systems(Update, (
            // Stress testing systems
            stress_test_spawn_system,
            stress_test_cleanup_system,
            performance_comparison_system,
        ))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Create Earth as a sphere with bathymetry texture
    // Earth radius = 6371 km, so in scaled coordinates it should be 6.371 units
    let earth_texture = asset_server.load("textures/gebco_08_rev_bath_3600x1800_color.jpg");
    
    commands.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(earth_texture.clone()),
            base_color: Color::srgb(1.0, 1.0, 1.0), // White to let texture show through
            unlit: false, // Make sure lighting is enabled
            ..default()
        })),
        Mesh3d(meshes.add(Sphere::new(6.371).mesh().uv(32, 18))),
        Transform::default(),
    ));
    
    // Add a fallback colored sphere in case texture doesn't load
    commands.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.5, 1.0), // Fallback blue color
            unlit: true, // Unlit for debugging
            ..default()
        })),
        Mesh3d(meshes.add(Sphere::new(5.0).mesh().uv(16, 8))), // Smaller sphere for reference
        Transform::from_xyz(0.0, 0.0, 0.0), // Offset position
    ));

    // Add directional light to simulate the sun
    commands.spawn((
        DirectionalLight {
            illuminance: 100000.0, // Very bright like the sun
            shadows_enabled: true,
            ..default()
        },
        // Position the sun at an angle to create realistic lighting
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, 0.8, 0.0)),
    ));

    // Keep the original point light but reduce intensity since we have sun + ambient now
    commands.spawn((
        PointLight {
            intensity: 8000.0, // Reduced from 15000.0
            shadows_enabled: false, // Disable shadows to avoid conflicts with directional light
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Add camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 15.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}