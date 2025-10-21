use bevy::{ecs::{query::{With, Without}, system::{Query, Res, Single}}, math::Vec3, time::Time, transform::components::Transform};

use crate::{cf104::Joystick, player::controls::{KeyBindings, KeyState}, projectile::{AngularVelocity, Grounded, Projectile, Velocity}};




pub fn update_angular_projectile_velocity(
    joystick: Single<&Joystick>,
    keybindings: Res<KeyBindings>,
    mut query: Query<(&Velocity, &mut AngularVelocity), (With<Projectile>, Without<Grounded>)>,
) {
    const PITCH_RATE: f32 = 1.0;
    const YAW_RATE: f32 = 0.1;
    const ROLL_RATE: f32 = 2.;

    for (velocity, mut ang_vel) in &mut query {
        let input = joystick.0;

        // Joystick pitch (Y) and roll (X)
        let pitch_input: f32 = input.y;
        let roll_input: f32 = input.x;

        // Pedal yaw
        let left_pedal = keybindings.feet.left.state == KeyState::Held || keybindings.feet.left.state == KeyState::Pressed;
        let right_pedal = keybindings.feet.right.state == KeyState::Held || keybindings.feet.right.state == KeyState::Pressed;

        let yaw_input: f32 = match (left_pedal, right_pedal) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };

        // Projectile’s forward speed
        let speed: f32 = velocity.length();
        let speed_factor = 1.0 / (1.0 + speed * 0.01) + 0.01;

        ang_vel.0 = Vec3::new(
            roll_input * -ROLL_RATE * speed_factor,
            yaw_input * -YAW_RATE * speed_factor,
            pitch_input * PITCH_RATE * speed_factor,
        );

        // println!("angular velocity: {:?}", ang_vel.0);
    }
}


pub fn apply_angular_damping(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut AngularVelocity), With<Projectile>>,
) {
    for (vel, mut ang_vel) in &mut query {
        let airspeed = vel.length();
        let damping_strength = airspeed / 200.0; // stronger at high speed
        let damping = damping_strength.clamp(0.2, 3.0);

        ang_vel.0 *= 1.0 - (damping * time.delta_secs()).min(1.0);
    }
}
