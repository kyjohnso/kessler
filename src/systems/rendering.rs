use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::render::mesh::shape;
use crate::components::*;
use crate::resources::*;

/// System for handling mouse camera controls
pub fn camera_control_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    // Mouse rotation
    if mouse_buttons.pressed(MouseButton::Left) {
        for event in mouse_motion_events.read() {
            let delta = event.delta;
            
            // Horizontal rotation (around Y axis)
            camera_transform.rotate_around(
                Vec3::ZERO,
                Quat::from_rotation_y(-delta.x * 0.005),
            );
            
            // Vertical rotation (around local X axis)
            let right = camera_transform.rotation * Vec3::X;
            camera_transform.rotate_around(
                Vec3::ZERO,
                Quat::from_axis_angle(right, -delta.y * 0.005),
            );
        }
    }

    // Mouse zoom
    for event in mouse_wheel_events.read() {
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y * 0.5,
            MouseScrollUnit::Pixel => event.y * 0.01,
        };
        
        // Move camera towards/away from center
        let direction = camera_transform.translation.normalize();
        let new_distance = (camera_transform.translation.length() - scroll_amount)
            .clamp(8.0, 100.0); // Min/max zoom distances
        
        camera_transform.translation = direction * new_distance;
    }
}

/// System to render satellites as small spheres
pub fn satellite_rendering_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    satellites_without_mesh: Query<(Entity, &OrbitalState, &Satellite), (With<RenderAsSatellite>, Without<Handle<Mesh>>)>,
) {
    for (entity, orbital_state, _satellite) in satellites_without_mesh.iter() {
        // Scale down the position to make satellites visible - divide by 1000 to convert km to render units
        let scaled_position = orbital_state.position / 1000.0;
        
        // Create a visible sphere to represent the satellite
        let mesh = meshes.add(shape::UVSphere {
            radius: 0.05, // Appropriate size for scaled coordinate system
            ..default()
        }.into());
        let material = materials.add(Color::GREEN.into());
        
        commands.entity(entity).insert(PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(scaled_position),
            ..default()
        });
    }
}

/// System to render debris as small points
pub fn debris_rendering_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    debris_query: Query<(Entity, &OrbitalState, &Debris), (With<RenderAsDebris>, Without<Handle<Mesh>>)>,
) {
    for (entity, orbital_state, _debris) in debris_query.iter() {
        // Create tiny sphere for debris
        let mesh = meshes.add(shape::UVSphere {
            radius: 2.0,
            ..default()
        }.into());
        let material = materials.add(Color::RED.into());
        
        commands.entity(entity).insert(PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(orbital_state.position),
            ..default()
        });
    }
}

/// System to update positions of rendered objects
pub fn update_positions_system(
    mut query: Query<(&mut Transform, &OrbitalState), (With<Handle<Mesh>>, Changed<OrbitalState>)>,
) {
    for (mut transform, orbital_state) in query.iter_mut() {
        // Scale down position to match rendering scale (km to render units)
        transform.translation = orbital_state.position / 1000.0;
    }
}