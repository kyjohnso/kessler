use bevy::prelude::*;
use bevy::render::mesh::shape;

mod components;
mod resources;
mod systems;
mod utils;

use components::*;
use resources::*;
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Constants>()
        .init_resource::<SimulationTime>()
        .init_resource::<EnergyAnalytics>()
        .init_resource::<TleDataCache>()
        .init_resource::<SpatialOctree>()
        .init_resource::<CollisionPairs>()
        .add_systems(Startup, (
            setup_scene,
            initialize_tle_data_system,
        ))
        .add_systems(Update, (
            camera_control_system,
            physics_system,
            time_control_system,
            update_spatial_octree_system,
            collision_detection_system,
            debris_generation_system,
            energy_analytics_system,
            satellite_rendering_system,
            debris_rendering_system,
            update_positions_system,
            debug_orbital_system,
            debug_analytics_system,
            process_tle_fetch_system,
        ))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create Earth as a simple sphere (properly scaled in new coordinate system)
    // Earth radius = 6371 km, so in scaled coordinates it should be 6.371 units
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::UVSphere::default().into()), // Earth sphere
        material: materials.add(Color::BLUE.into()),
        transform: Transform::from_scale(Vec3::splat(6.371)), // Earth radius in scaled units
        ..default()
    });

    // Add a light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Add camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 15.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}