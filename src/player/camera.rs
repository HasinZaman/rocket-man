use std::f32::consts::PI;
use crate::cf104::Plane;
use crate::player::Player;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::{prelude::*};

#[derive(Component)]
pub struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002))
    }
}

pub fn set_up_player_camera() -> (Camera3d, CameraSensitivity) {
    (Camera3d::default(), CameraSensitivity::default())
}

pub fn look_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    plane_transform: Single<
        &Transform,
        (
            With<Player>,
            With<Plane>,
            Without<Camera3d>,
            Without<CameraSensitivity>,
        ),
    >,
    mut cam_query: Query<(&mut Transform, &CameraSensitivity), With<Player>>,
) {
    let Ok((mut cam_transform, sensitivity)) = cam_query.single_mut() else {
        return;
    };

    let delta = accumulated_mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    let delta_yaw = -delta.x * sensitivity.0.x;
    let delta_pitch = -delta.y * sensitivity.0.y;

    let _plane_forward = plane_transform.back();
    let plane_up = plane_transform.down();
    let plane_right = plane_transform.left();

    let yaw_rotation = Quat::from_axis_angle(*plane_right, delta_yaw);
    let pitch_rotation = Quat::from_axis_angle(*plane_up, delta_pitch);

    cam_transform.rotation = yaw_rotation * pitch_rotation * cam_transform.rotation;

    let (mut yaw, mut pitch, _) = cam_transform.rotation.to_euler(EulerRot::XYZ);

    pitch = pitch.clamp(-1., 1.);

    yaw = yaw.clamp(0.5, 3. * PI / 4.);

    cam_transform.rotation = Quat::from_euler(EulerRot::XYZ, yaw, pitch, 0.);
}
