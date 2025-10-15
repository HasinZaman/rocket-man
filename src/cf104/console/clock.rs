use std::f32::consts::TAU;

use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

use crate::cf104::CF104_CONSOLE_ASSET_PATH;

#[derive(Component, Debug)]
pub struct Clock(pub f32, Entity, Entity, Entity);

#[derive(Component, Debug)]
pub struct ClockHand;

pub fn update_clock(
    time: Res<Time>,
    mut clocks: Query<&mut Clock>,
    mut transforms: Query<&mut Transform, With<ClockHand>>,
) {
    for mut clock in clocks.iter_mut() {
        const MAX_SEC: f32 = 60. * 60. * 12.;
        clock.0 = (clock.0 + time.delta_secs()) % MAX_SEC;

        let total_seconds = clock.0;

        let hours: f32 = (total_seconds / 3600.0) % 12.0;
        let minutes: f32 = (total_seconds / 60.0) % 60.0;
        let seconds: f32 = total_seconds % 60.0;

        let hour_angle: f32 = TAU * (hours / 12.0);
        let minute_angle: f32 = TAU * (minutes / 60.0);
        let second_angle: f32 = TAU * (seconds / 60.0);

        if let Ok(mut t) = transforms.get_mut(clock.1) {
            t.rotation = Quat::from_rotation_y(hour_angle);
        }
        if let Ok(mut t) = transforms.get_mut(clock.2) {
            t.rotation = Quat::from_rotation_y(minute_angle);
        }
        if let Ok(mut t) = transforms.get_mut(clock.3) {
            t.rotation = Quat::from_rotation_y(second_angle);
        }
    }
}

pub fn spawn_clock<
    const HOUR_HAND: usize,
    const MINUTE_HAND: usize,
    const SECOND_HAND: usize,
    const CLOCK_FRAME: usize,
    const CLOCK_CENTER: usize,
    const CLOCK_SCREEN: usize,
>(
    transform: Transform,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    console_material: &Handle<StandardMaterial>,
    glass_material: &Handle<StandardMaterial>,
    needle_material_handle: &Handle<StandardMaterial>,
    parent_id: Entity,
) {
    // hand material
    // hours hand
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        HOUR_HAND
    ));

    let hour_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(needle_material_handle.clone()),
            ClockHand,
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
        ))
        .id();
    // minutes hand
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        MINUTE_HAND
    ));
    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: 0.,
        y: -0.0012316405773162842,
        z: 0.,
    };
    let minute_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(needle_material_handle.clone()),
            ClockHand,
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
        ))
        .id();
    // seconds hand
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        SECOND_HAND
    ));
    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: 0.,
        y: -0.0042786262929439545,
        z: 0.,
    };
    let seconds_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(needle_material_handle.clone()),
            ClockHand,
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
        ))
        .id();
    // clock frame
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        CLOCK_FRAME
    ));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        emissive_texture: Some(asset_server.load("cf104/clock.png")),
        emissive: LinearRgba {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        },
        ..default()
    });
    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: -0.3253922462463379,
        y: -1.5688923597335815,
        z: 1.0362449884414673,
    };
    transform.scale = Vec3::splat(0.693862795829773);
    let clock_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            Clock(38185.0, hour_id, minute_id, seconds_id),
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
            ChildOf(parent_id),
        ))
        .id();
    for id in [hour_id, minute_id, seconds_id] {
        commands.entity(id).insert(ChildOf(clock_id));
    }
    // clock center
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        CLOCK_CENTER
    ));
    let material_handle = console_material.clone();
    let mut transform = Transform::default();
    transform.scale = Vec3::splat(0.010637586005032063);
    transform.rotation = Quat::from_xyzw(-0.7071068286895752, 0., 0., 0.7071068286895752);
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material_handle.clone()),
        // Visibility::Visible,
        NoFrustumCulling,
        transform,
        ChildOf(clock_id),
    ));
    // screen
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        CLOCK_SCREEN
    ));
    let transform = Transform::default();
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(glass_material.clone()),
        // Visibility::Visible,
        NoFrustumCulling,
        transform,
        ChildOf(clock_id),
    ));
}
