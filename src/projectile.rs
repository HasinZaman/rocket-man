use std::{
    f32::consts::{FRAC_PI_4, PI},
    process::Command,
};

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res, Single},
    },
    math::{EulerRot, Quat, Vec2, Vec3},
    prelude::{Deref, DerefMut},
    time::Time,
    transform::components::Transform,
};

use crate::cf104::{Joystick, console::throttle::Throttle};

#[derive(Component, Debug)]
pub struct Grounded;

#[derive(Component, Debug)]
pub struct Projectile;

#[derive(Component, Deref, DerefMut, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Component, Deref, DerefMut, Debug)]
pub struct AngularVelocity(pub Vec3);

#[derive(Component, Debug)]
pub struct MomentOfInertia(pub f32);

#[derive(Component, Debug)]
pub struct Mass(pub f32);

#[derive(Component, Debug)]
pub struct DragCoefficient(pub f32);

#[derive(Component, Debug)]
pub struct CrossSectionArea(pub f32);

#[derive(Component, Debug)]
pub struct GravityScale(pub f32);

#[derive(Component, Debug)]
pub struct WingArea(pub f32); //m^2

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

#[derive(Component, Debug)]
pub struct BrakeForce(pub f32, pub bool);

#[derive(Component, Debug)]
pub struct SteeringWheel {
    pub max_angle: f32,
    pub input_dir: f32,
    pub current_angle: f32,
    pub delta_speed: f32,
}

#[derive(Bundle)]
pub struct GroundedBundle {
    pub grounded: Grounded,
    pub brake_force: BrakeForce,
    pub turn_radius: SteeringWheel,
}

impl GroundedBundle {
    pub fn cf_104() -> Self {
        GroundedBundle {
            grounded: Grounded,
            brake_force: BrakeForce(8_000.0, false),
            turn_radius: SteeringWheel {
                max_angle: FRAC_PI_4,
                input_dir: 0.0,
                current_angle: 0.0,
                delta_speed: 2.0,
            },
        }
    }
}

#[derive(Bundle)]
pub struct PlaneBundle {
    pub projectile: Projectile,
    pub velocity: Velocity,
    pub angular_velocity: AngularVelocity,
    pub moment_of_inertia: MomentOfInertia,
    pub mass: Mass,
    pub drag: DragCoefficient,
    pub cross_section: CrossSectionArea,
    pub wing_area: WingArea,
    pub engine: Engine,
}

impl PlaneBundle {
    pub fn cf_104() -> Self {
        Self {
            projectile: Projectile,
            velocity: Velocity(Vec3::ZERO),
            angular_velocity: AngularVelocity(Vec3::ZERO),
            moment_of_inertia: MomentOfInertia(0.0),
            mass: Mass(13_200.0),
            drag: DragCoefficient(0.75),
            cross_section: CrossSectionArea(3.0),
            wing_area: WingArea(18.2),
            engine: Engine::cf104(),
        }
    }
}

fn vec3_fmt(v: Vec3) -> String {
    format!("({:.2}, {:.2}, {:.2})", v.x, v.y, v.z)
}

pub fn update_engine_thrust(
    time: Res<Time>,
    throttle: Single<&Throttle>,
    mut query: Query<&mut Engine>,
) {
    for mut engine in &mut query {
        // ramping
        if throttle.0 > 0.05 {
            engine.elapsed += time.delta_secs();
        } else {
            engine.elapsed = (engine.elapsed - time.delta_secs() * 2.0).max(0.0);
        }

        let ramp_factor = if engine.ramp_time > 0.0 {
            (engine.elapsed / engine.ramp_time).clamp(0.0, 1.0)
        } else {
            1.0
        };

        engine.current_thrust = engine.max_thrust * (throttle.0 / 100.) * ramp_factor;
    }
}

pub fn update_angular_projectile_velocity(
    time: Res<Time>,
    joystick: Single<&Joystick>,
    mut query: Query<(&Velocity, &mut AngularVelocity), (With<Projectile>, Without<Grounded>)>,
) {
    for (velocity, mut ang_vel) in &mut query {
        let input = joystick.0;
        if input == Vec2::ZERO {
            continue;
        }

        let delta_time = time.delta_secs();

        // --- Airspeed-based control authority ---
        let airspeed = velocity.length();
        const MIN_EFFECTIVE_SPEED: f32 = 20.0; // below this, little control
        const MAX_EFFECTIVE_SPEED: f32 = 300.0; // full control authority
        const MAX_ANGULAR_ACCEL: f32 = 2.0; // radians/s²

        let control_effectiveness = ((airspeed - MIN_EFFECTIVE_SPEED)
            / (MAX_EFFECTIVE_SPEED - MIN_EFFECTIVE_SPEED))
            .clamp(0.0, 1.0);

        // --- Compute angular acceleration ---
        let pitch_accel = input.y * control_effectiveness * MAX_ANGULAR_ACCEL;
        let roll_accel = -input.x * control_effectiveness * MAX_ANGULAR_ACCEL;

        // --- Update angular velocity ---
        ang_vel.x += pitch_accel * delta_time;
        ang_vel.z += roll_accel * delta_time;
    }
}

pub fn update_grounded_turn(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform, &mut SteeringWheel), (With<Grounded>)>,
) {
    for (vel, mut transform, mut wheel) in &mut query {
        let dt = time.delta_secs();
        let speed = vel.length();

        let speed_factor = ((speed - 10.0) / (40.0 - 10.0)).clamp(0.0, 1.0);
        let speed_effectiveness = 1.0 - speed_factor;

        // Target steering angle (reversed direction for correct turning feel)
        let target_angle = -wheel.max_angle * wheel.input_dir * speed_effectiveness;

        let angle_diff = target_angle - wheel.current_angle;
        let max_delta = wheel.delta_speed * dt;
        let applied_delta = angle_diff.clamp(-max_delta, max_delta);
        wheel.current_angle += applied_delta;

        let yaw_rate = (wheel.current_angle * (1.0 - speed_factor)) * dt * 0.25;
        let yaw_rotation = Quat::from_rotation_y(yaw_rate);
        transform.rotation = yaw_rotation * transform.rotation;

        // println!(
        //     "Wheel Debug → speed: {:.1} m/s | eff: {:.2} | target: {:.2} rad | current: {:.2} rad | applied: {:.3}",
        //     speed,
        //     speed_effectiveness,
        //     target_angle,
        //     wheel.current_angle,
        //     applied_delta
        // );
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
pub fn update_projectile_velocity(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Velocity,
            &Transform,
            &Mass,
            &DragCoefficient,
            &CrossSectionArea,
            &WingArea,
            &Engine,
        ),
        (With<Projectile>, Without<Grounded>),
    >,
) {
    const AIR_DENSITY: f32 = 1.225; // kg/m³
    const GRAVITY: f32 = 9.81;
    const LIFT_COEFFICIENT: f32 = 0.8;

    for (mut vel, transform, mass, drag, cross_section, wing_area, engine) in &mut query {
        let dt = time.delta_secs();

        let forward = transform.rotation * Vec3::X;
        let up = transform.rotation * Vec3::Y;

        let speed = vel.length();
        let velocity_dir = if speed > 0.001 {
            vel.normalize()
        } else {
            forward
        };

        // --- Forces ---
        let thrust = engine.thrust_vector(transform);
        let drag_force =
            -velocity_dir * 0.5 * AIR_DENSITY * speed * speed * drag.0 * cross_section.0;

        let lift_magnitude = 0.5 * AIR_DENSITY * speed * speed * LIFT_COEFFICIENT * wing_area.0;
        let lift_force = up * lift_magnitude;

        let gravity_force = Vec3::new(0.0, -mass.0 * GRAVITY, 0.0);

        let total_force = thrust + drag_force + lift_force + gravity_force;
        let acceleration = total_force / mass.0;

        vel.0 += acceleration * dt;

        let max_speed = 590.0;
        if vel.0.length() > max_speed {
            vel.0 = vel.0.normalize() * max_speed;
        }

        if vel.0.x.is_nan() {
            panic!();
        }

        println!(
            "pos: {:#?} rot: {:#?}",
            &transform.translation,
            &transform.rotation.to_euler(EulerRot::XYZ)
        );
        println!(
            "Velocity: {}, Total Force: {}",
            vec3_fmt(vel.0),
            vec3_fmt(total_force)
        );
        println!(
            "Engine Thrust: {}, Drag: {}, Gravity: {}, Lift: {}",
            vec3_fmt(thrust),
            vec3_fmt(drag_force),
            vec3_fmt(gravity_force),
            vec3_fmt(lift_force)
        );
    }
}

pub fn update_grounded_velocity(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Velocity,
            &BrakeForce,
            &SteeringWheel,
            &Transform,
            &Mass,
            &DragCoefficient,
            &CrossSectionArea,
            &WingArea,
            &Engine,
        ),
        (With<Projectile>, With<Grounded>),
    >,
) {
    const AIR_DENSITY: f32 = 1.225;
    const GRAVITY: f32 = 9.81;
    const LIFT_COEFFICIENT: f32 = 0.8;
    const ROLLING_RESISTANCE: f32 = 5.0;

    for (entity, mut vel, brake, wheel, transform, mass, drag, cross_section, wing_area, engine) in
        &mut query
    {
        let dt = time.delta_secs();

        let forward = transform.rotation * Vec3::X;
        let right = transform.rotation * Vec3::Z;
        let up = transform.rotation * Vec3::Y;

        let speed = vel.length();
        let velocity_dir = if speed > 0.001 {
            vel.normalize()
        } else {
            forward
        };

        // --- Forces ---
        let thrust = engine.thrust_vector(transform);
        let drag_force =
            -velocity_dir * 0.5 * AIR_DENSITY * speed * speed * drag.0 * cross_section.0;

        let brake_force = if brake.1 {
            -velocity_dir * brake.0
        } else {
            Vec3::ZERO
        };

        let rolling_resistance = -velocity_dir * speed * ROLLING_RESISTANCE;

        let lift_magnitude = 0.5 * AIR_DENSITY * speed * speed * LIFT_COEFFICIENT * wing_area.0;
        let lift_force = up * lift_magnitude;

        let gravity_force = Vec3::new(0.0, -mass.0 * GRAVITY, 0.0);

        // total force (no lateral friction)
        let total_force =
            thrust + drag_force + brake_force + rolling_resistance + lift_force + gravity_force;

        let acceleration = total_force / mass.0;
        vel.0 += acceleration * dt;

        // --- Remove lateral velocity ---
        // Project velocity onto forward vector and discard sideways (right) component
        let forward_vel = forward.normalize() * vel.0.dot(forward.normalize());
        vel.0 = forward_vel + Vec3::Y * vel.0.dot(Vec3::Y);

        // Prevent negative vertical velocity while grounded
        vel.y = vel.y.max(0.0);

        let max_speed = 590.0;
        if vel.0.length() > max_speed {
            vel.0 = vel.0.normalize() * max_speed;
        }

        // println!(
        //     "\nVelocity: {} | Total: {}",
        //     vec3_fmt(vel.0),
        //     vec3_fmt(total_force)
        // );
        // println!(
        //     "Thrust: {} | Brake: {} | RollRes: {} | Lift: {} | Gravity: {}",
        //     vec3_fmt(thrust),
        //     vec3_fmt(brake_force),
        //     vec3_fmt(rolling_resistance),
        //     vec3_fmt(lift_force),
        //     vec3_fmt(gravity_force),
        // );

        if vel.y > 0.0 {
            commands.entity(entity).remove::<Grounded>();
            println!("✈️  Takeoff! The CF-104 is airborne.");
        }
    }
}

pub fn update_transform(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity, &AngularVelocity)>,
) {
    let dt = time.delta_secs();

    for (mut transform, velocity, angular_velocity) in &mut query {
        // --- Translation ---
        transform.translation += velocity.0 * dt;

        // --- Rotation integration ---
        let angular_speed = angular_velocity.length();
        if angular_speed > 0.0001 {
            let axis = angular_velocity.normalize();
            let delta_rot = Quat::from_axis_angle(axis, angular_speed * dt);
            transform.rotation = delta_rot * transform.rotation;
        }
    }
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedUpdate,
            (
                update_engine_thrust,
                update_angular_projectile_velocity,
                apply_angular_damping,
                update_grounded_turn,
                update_projectile_velocity,
                update_projectile_velocity,
                update_grounded_velocity,
                update_transform,
            ),
        );
    }
}
