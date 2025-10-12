use bevy::{ecs::{bundle::Bundle, component::Component, query::With, system::{Query, Res}}, math::{Quat, Vec2, Vec3}, prelude::{Deref, DerefMut}, time::Time, transform::components::Transform};

use crate::cf104::Joystick;

#[derive(Component)]
pub struct Projectile;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec3);

#[derive(Component, Deref, DerefMut)]
pub struct AngularVelocity(pub Vec3);

#[derive(Component)]
pub struct MomentOfInertia(pub f32);

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct DragCoefficient(pub f32);

#[derive(Component)]
pub struct CrossSectionArea(pub f32);

#[derive(Component)]
pub struct GravityScale(pub f32);

#[derive(Component)]
pub struct Engine {
    pub max_thrust: f32,
    pub ramp_time: f32,
    pub elapsed: f32,
    pub direction: Quat,
    pub throttle: f32,
}

impl Engine {
    pub fn engine_thrust(&self) -> f32 {
        let ramp_factor = if self.ramp_time > 0.0 {
            (self.elapsed / self.ramp_time).clamp(0.0, 1.0)
        } else {
            1.0
        };

        self.max_thrust * self.throttle.clamp(0.0, 1.0) * ramp_factor
    }
    
    pub fn thrust_vector(&self, transform: &Transform) -> Vec3 {
        let thrust_magnitude: f32 = self.engine_thrust();
        let world_dir = transform.rotation * self.direction;

        world_dir * thrust_magnitude * Vec3::X
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
            engine: Engine {
                max_thrust: 44_000.0,
                ramp_time: 5.0,
                elapsed: 0.0,
                direction: Quat::IDENTITY,
                throttle: 0.0,
            },
        }
    }
}

pub fn update_rotation(
    time: Res<Time>,
    mut query: Query<(&Joystick, &Velocity, &mut AngularVelocity)>,
) {
    for (joystick, velocity, mut ang_vel) in &mut query {
        let input = joystick.0;
        if input == Vec2::ZERO {
            continue;
        }

        let delta_time = time.delta_secs();

        // --- Airspeed-based control authority ---
        let airspeed = velocity.length();
        const MIN_EFFECTIVE_SPEED: f32 = 20.0;   // below this, little control
        const MAX_EFFECTIVE_SPEED: f32 = 300.0;  // full control authority
        const MAX_ANGULAR_ACCEL: f32 = 1.5;      // radians/s²

        let control_effectiveness = ((airspeed - MIN_EFFECTIVE_SPEED)
            / (MAX_EFFECTIVE_SPEED - MIN_EFFECTIVE_SPEED))
            .clamp(0.0, 1.0);

        // --- Compute angular acceleration ---
        let pitch_accel = -input.y * control_effectiveness * MAX_ANGULAR_ACCEL;
        let roll_accel = -input.x * control_effectiveness * MAX_ANGULAR_ACCEL;

        // --- Update angular velocity ---
        ang_vel.x += pitch_accel * delta_time;
        ang_vel.z += roll_accel * delta_time;
    }
}

pub fn apply_angular_damping(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut AngularVelocity)>,
) {
    for (vel, mut ang_vel) in &mut query {
        let airspeed = vel.length();
        let damping_strength = airspeed / 200.0; // stronger at high speed
        let damping = damping_strength.clamp(0.2, 3.0);

        ang_vel.0 *= 1.0 - (damping * time.delta_secs()).min(1.0);
    }
}

pub fn update_projectile(
    time: Res<Time>,
    mut query: Query<(
        &mut Transform,
        &mut Velocity,
        &mut AngularVelocity,
        &Mass,
        &MomentOfInertia,
        &DragCoefficient,
        &CrossSectionArea,
        &Engine,
        Option<&GravityScale>,
    ), With<Projectile>>,
) {
    const AIR_DENSITY: f32 = 1.225; // kg/m³ at sea level
    let dt = time.delta_secs();

    for (
        mut transform,
        mut velocity,
        mut angular_velocity,
        mass,
        inertia,
        drag,
        area,
        engine,
        gravity_scale,
    ) in &mut query
    {
        // --- ENGINE THRUST ---
        let thrust_mag = engine.max_thrust * engine.elapsed;
        let thrust_dir = transform.rotation * Vec3::Z; // forward in world space
        let thrust_force = thrust_dir * thrust_mag;

        // --- DRAG FORCE ---
        let speed = velocity.0.length();
        let drag_force = if speed > 0.0 {
            -0.5 * AIR_DENSITY * speed * speed * drag.0 * area.0 * velocity.0.normalize()
        } else {
            Vec3::ZERO
        };

        // --- GRAVITY ---
        let gravity_force = Vec3::new(0.0, -9.81, 0.0)
            * gravity_scale.map_or(1.0, |g| g.0)
            * mass.0;

        // --- SUM FORCES ---
        let total_force = thrust_force + drag_force + gravity_force;
        let acceleration = total_force / mass.0;

        // --- LINEAR INTEGRATION ---
        velocity.0 += acceleration * dt;
        transform.translation += velocity.0 * dt;

        // =====================
        //  ANGULAR DYNAMICS
        // =====================

        // --- ENGINE TORQUE (simple yaw/pitch control) ---
        // let torque = engine.torque_dir * engine.torque_strength;

        // // --- ROTATIONAL DRAG (aerodynamic stability) ---
        // let angular_drag = -angular_velocity.0 * 0.5 * AIR_DENSITY * area.0 * drag.0;

        // // --- SUM TORQUES ---
        // let total_torque = torque + angular_drag;

        // // --- ROTATIONAL ACCELERATION ---
        // let angular_accel = total_torque / inertia.0;

        // // --- UPDATE ANGULAR VELOCITY ---
        // angular_velocity.0 += angular_accel * dt;

        // // --- UPDATE ROTATION ---
        // let delta_rot = Quat::from_scaled_axis(angular_velocity.0 * dt);
        // transform.rotation = (transform.rotation * delta_rot).normalize();
    }
}