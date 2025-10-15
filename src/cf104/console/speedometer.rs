use std::f32::consts::PI;

use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

use crate::cf104::CF104_CONSOLE_ASSET_PATH;
use crate::projectile::Velocity;

#[derive(Component)]
pub struct Speedometer(Entity);
#[derive(Component)]
pub struct SpeedometerNeedle;

pub fn update_speedometer(
    query: Query<&Speedometer>,
    mut needle_query: Query<&mut Transform, With<SpeedometerNeedle>>,
    velocity: Single<(&GlobalTransform, &Velocity)>,
) {
    let deg_to_rad = PI / 180.0;

    for Speedometer(needle) in query {
        let forward = *velocity.0.right();
        let forward_speed = velocity.1.0.project_onto(forward).length();

        let mach: f32 = forward_speed / 343.0;

        let angle = if mach <= 1.0 {
            41.0 * deg_to_rad * mach
        } else if mach <= 2.0 {
            let start = 41.0;
            let end = 120.0;
            start * deg_to_rad + ((end - start) * (mach - 1.0) / 1.0) * deg_to_rad
        } else if mach <= 4.0 {
            let start = 120.0;
            let end = 242.0;
            start * deg_to_rad + ((end - start) * (mach - 2.0) / 2.0) * deg_to_rad
        } else {
            let start = 242.0;
            let end = 320.0;
            start * deg_to_rad + ((end - start) * (mach - 4.0) / 4.0) * deg_to_rad
        };

        if let Ok(mut transform) = needle_query.get_mut(*needle) {
            transform.rotation = Quat::from_rotation_y(angle);
        }
    }
}

pub fn spawn_speedometer<
    const FRAME: usize,
    const SCREEN: usize,
    const NEEDLE: usize,
    const DIAL_CENTER: usize,
>(
    parent_transform: Transform,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    console_material: Handle<StandardMaterial>,
    glass_material: Handle<StandardMaterial>,
    needle_material_handle: Handle<StandardMaterial>,
    parent_id: Entity,
) {
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        DIAL_CENTER
    ));
    let material_handle = console_material.clone();
    let mut transform = Transform::default();
    transform.scale = Vec3::splat(0.010637586);
    transform.translation = Vec3 {
        x: 0.0,
        y: 3.1705946e-05,
        z: 0.0,
    };
    transform.rotation = Quat::from_array([-0.7071068286895752, 0.0, 0.0, 0.7071068286895752]);
    let dial_center = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material_handle.clone()),
            NoFrustumCulling,
            transform,
        ))
        .id();
    // --- Needle ---
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        NEEDLE
    ));
    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: 0.00000333786,
        y: -0.007229913,
        z: 0.0,
    };
    let speed_needle = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(needle_material_handle.clone()),
            SpeedometerNeedle,
            NoFrustumCulling,
            transform,
        ))
        .id();
    // --- Screen (decorative element) ---
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        SCREEN
    ));
    let material = glass_material.clone();
    let screen = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            NoFrustumCulling,
            Transform::default(),
        ))
        .id();
    // --- Speedometer Base ---
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        FRAME
    ));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        emissive_texture: Some(asset_server.load("cf104/speedometer.png")),
        emissive: LinearRgba {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        },
        ..default()
    });
    // The main Speedometer entity
    let speedometer_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Speedometer(speed_needle),
            NoFrustumCulling,
            parent_transform,
            ChildOf(parent_id),
        ))
        .id();
    // Attach all parts as children
    for id in [dial_center, screen, speed_needle] {
        commands.entity(id).insert(ChildOf(speedometer_id));
    }
}
