use bevy::prelude::*;

use crate::{
    cf104::CF104Plugin, player::PlayerPlugin,
};

pub mod player;
pub mod cf104;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((CF104Plugin, PlayerPlugin))
        .add_systems(Update, (debug_camera_movement))
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    // commands.spawn((
    //     Mesh3d(meshes.add(Circle::new(4.0))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    // ));
    // // cube
    // commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
    //     Transform::from_xyz(0.0, 0.5, 0.0),
    // ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(-10.5, 10.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    // ));
}

fn debug_camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    let mut transform = query.single_mut().unwrap();

    let move_speed = 10.0 * time.delta_secs();
    // let rot_speed = 1.5 * time.delta_secs();

    // // --- Rotation (global) ---
    // if keyboard.pressed(KeyCode::ArrowLeft) {
    //     transform.rotate_axis(Dir3::Y, rot_speed);
    // }
    // if keyboard.pressed(KeyCode::ArrowRight) {
    //     transform.rotate_axis(Dir3::Y, -rot_speed);
    // }
    // if keyboard.pressed(KeyCode::KeyQ) {
    //     transform.rotate_axis(Dir3::Y, rot_speed);
    // }
    // if keyboard.pressed(KeyCode::KeyE) {
    //     transform.rotate_axis(Dir3::Y, -rot_speed);
    // }

    // // Pitch (around camera's local right axis, but inverted for natural control)
    // let right = transform.right();
    // if keyboard.pressed(KeyCode::ArrowUp) {
    //     // Look down
    //     transform.rotate_axis(right, rot_speed);
    // }
    // if keyboard.pressed(KeyCode::ArrowDown) {
    //     // Look up
    //     transform.rotate_axis(right, -rot_speed);
    // }

    // --- Movement (local orientation) ---
    let forward = transform.forward();
    let right = transform.right();
    let up = transform.up();

    if keyboard.pressed(KeyCode::KeyW) {
        transform.translation += forward * move_speed;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        transform.translation -= forward * move_speed;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        transform.translation -= right * move_speed;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        transform.translation += right * move_speed;
    }
    if keyboard.pressed(KeyCode::Space) {
        transform.translation += up * move_speed;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        transform.translation -= up * move_speed;
    }

    // println!("{:?}\n{:?}", transform.translation, transform.rotation.to_euler(EulerRot::XYZ));
}
