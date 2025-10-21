use std::f32::consts::PI;

use bevy::{
    ecs::component::Component,
    math::{Quat, Vec3},
    transform::components::Transform,
};

#[derive(Component, Debug)]
pub struct Engine {
    pub max_thrust: f32,
    pub ramp_time: f32,
    pub elapsed: f32,
    pub direction: Quat,
    pub current_thrust: f32,
}

impl Engine {
    pub fn cf104() -> Self {
        Self {
            max_thrust: 44_000.0,
            ramp_time: 5.0,
            elapsed: 0.0,
            direction: Quat::from_rotation_y(PI),
            current_thrust: 0.0,
        }
    }
    pub fn thrust_vector(&self, transform: &Transform) -> Vec3 {
        let world_dir = transform.rotation * self.direction * Vec3::X;
        world_dir * self.current_thrust
    }
}
