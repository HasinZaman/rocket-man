use bevy::prelude::*;

use crate::{
    cf104::CF104Plugin,
    player::{
        Player, PlayerPlugin,
        camera::OutlineCamera,
        controls::{KeyBindings, KeyState},
    },
    projectile::ProjectilePlugin,
    world::WorldPlugin,
};

pub mod cf104;
pub mod player;
pub mod world;

pub mod projectile;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(CF104Plugin)
        .add_plugins(ProjectilePlugin)
        // .add_systems(Update, debug_camera_control)
        .run();
}

pub fn debug_camera_control(
    time: Res<Time>,
    keybindings: Res<KeyBindings>,
    mut cam_query: Query<&mut Transform, With<Player>>,
) {
    let dt = time.delta_secs();
    let move_speed = 25.0;
    let rotate_speed = 1.5;

    for mut transform in &mut cam_query {
        let mut translation = Vec3::ZERO;
        let mut rotation = transform.rotation;

        // --- Rotation (Q / E) ---
        if keybindings.left_arm.alt_1.state == KeyState::Held {
            rotation *= Quat::from_rotation_y(rotate_speed * dt);
        }
        if keybindings.left_arm.alt_2.state == KeyState::Held {
            rotation *= Quat::from_rotation_y(-rotate_speed * dt);
        }

        // --- Movement ---
        let forward = rotation * Vec3::Z;
        let right = rotation * Vec3::X;

        if keybindings.left_arm.up.state == KeyState::Held {
            translation -= forward * move_speed * dt;
        }
        if keybindings.left_arm.down.state == KeyState::Held {
            translation += forward * move_speed * dt;
        }
        if keybindings.left_arm.left.state == KeyState::Held {
            translation -= right * move_speed * dt;
        }
        if keybindings.left_arm.right.state == KeyState::Held {
            translation += right * move_speed * dt;
        }

        // --- Elevation (feet bindings) ---
        if keybindings.feet.left.state == KeyState::Held {
            translation.y += move_speed * dt;
        }
        if keybindings.feet.right.state == KeyState::Held {
            translation.y -= move_speed * dt;
        }

        transform.translation += translation;
        transform.rotation = rotation;
    }
}
