use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

use crate::cf104::CF104_CONSOLE_ASSET_PATH;

#[derive(Component)]
pub struct CompassGyro;

pub fn update_compass_gyro(
    compass_query: Query<(&ChildOf, &mut Transform), With<CompassGyro>>,
    parent_query: Query<&GlobalTransform, Without<CompassGyro>>,
) {
    let global_north_rot = Quat::from_rotation_y(PI); // adjust as needed
    // let global_north = Quat::from_rotation_y(0.0);;
    // let global_up = Vec3::Y;

    let offset_rot =
        Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_y(PI) * Quat::from_rotation_x(PI);

    for (ChildOf(parent), mut local) in compass_query {
        if let Ok(parent_global) = parent_query.get(*parent) {
            let parent_rot = parent_global.rotation();

            // Target rotation in world space
            let target_rot = global_north_rot * offset_rot;

            // Convert to local rotation relative to parent
            local.rotation = parent_rot.conjugate() * target_rot;
        }
    }
}

pub fn spawn_gyro_compass<const FRAME: usize, const SPHERE: usize, const SCREEN: usize>(
    parent_transform: Transform,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    console_material: &Handle<StandardMaterial>,
    glass_material: &Handle<StandardMaterial>,
    parent_id: Entity,
) {
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        FRAME
    ));
    let material_handle = console_material.clone();

    let compass_ball_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            // Visibility::Visible,
            NoFrustumCulling,
            parent_transform,
            ChildOf(parent_id),
        ))
        .id();

    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        SPHERE
    ));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        emissive_texture: Some(asset_server.load("cf104/compass_ball.png")),
        emissive: LinearRgba {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        }, // intensity multiplier
        ..default()
    });
    let mut transform = Transform::default();
    transform.scale = Vec3::splat(0.9211342334747314);
    transform.translation = Vec3 {
        x: 0.,
        y: 0.014294596388936043,
        z: 0.,
    };
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material_handle.clone()),
        CompassGyro,
        // Visibility::Visible,
        NoFrustumCulling,
        transform,
        ChildOf(compass_ball_id),
    ));
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        SCREEN
    ));
    let material_handle = glass_material.clone();
    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: 0.,
        y: -0.01874825544655323,
        z: 0.,
    };
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material_handle.clone()),
        // Visibility::Visible,
        NoFrustumCulling,
        transform,
        ChildOf(compass_ball_id),
    ));
}
