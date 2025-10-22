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
        system::{Commands, Query, Res, ResMut, Single},
    },
    math::{Dir3, EulerRot, Quat, Vec2, Vec3},
    prelude::{Deref, DerefMut},
    time::Time,
    transform::components::Transform,
};

use crate::{
    cf104::{Joystick, console::throttle::Throttle},
    projectile::{
        control_surfaces::{apply_angular_damping, update_angular_projectile_velocity},
        drag::{CrossSectionArea, Drag, drag_force, update_cross_section},
        engine::Engine,
        lift::lift_force,
        mass::{
            ExternalTank, Mass, MassBundle, MassComponent, MassData, Tank, get_weight,
            update_fuel_mass_system, update_tank_flow_rate,
        },
        util::{GRAVITY, air_density, altitude, get_lat, get_lon},
        weather::{
            Pressure, Temperature, WeatherMeta, WeatherPlugin, Wind, get_pressure, get_temperature,
            get_wind,
        },
    },
    world::{GlobalPosition, MovingOrigin},
};

pub mod control_surfaces;
pub(crate) mod drag;
pub mod engine;
pub(crate) mod lift;
pub mod mass;
pub mod util;

pub mod weather;

#[derive(Component, Debug)]
pub struct Grounded;

#[derive(Component, Debug)]
pub struct Projectile;

#[derive(Component, Default, Debug)]
pub struct GForceCache {
    pub net_force: Vec3,
    pub mass: f32,
}

#[derive(Component, Deref, DerefMut, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Component, Deref, DerefMut, Debug)]
pub struct AngularVelocity(pub Vec3);

#[derive(Component, Debug)]
pub struct DragCoefficient(pub f32);

#[derive(Component, Debug)]
pub struct GravityScale(pub f32);

#[derive(Component, Debug)]
pub struct WingArea(pub f32); //m^2

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
    pub position: GlobalPosition,
    pub net_force: GForceCache,
    pub projectile: Projectile,
    pub velocity: Velocity,
    pub angular_velocity: AngularVelocity,
    pub mass: Mass,
    pub wing_area: WingArea,
    pub engine: Engine,
    pub drag: Drag,
    pub cross_section_area: CrossSectionArea,
}

impl PlaneBundle {
    pub fn cf_104(position: Vec3) -> Self {
        Self {
            position: GlobalPosition {
                x: position.x as f64,
                y: position.y as f64,
                z: position.z as f64,
            },
            net_force: GForceCache::default(),
            projectile: Projectile,
            velocity: Velocity(Vec3::ZERO),
            angular_velocity: AngularVelocity(Vec3::ZERO),
            mass: Mass::default(),
            wing_area: WingArea(18.2),
            engine: Engine::cf104(),
            drag: Drag::new(),
            cross_section_area: CrossSectionArea::default(),
        }
    }
}

fn vec3_fmt(v: Vec3) -> String {
    format!("({:.2}, {:.2}, {:.2})", v.x, v.y, v.z)
}

pub fn update_engine_thrust(
    time: Res<Time>,
    throttle: Single<&Throttle>,
    mut engine_query: Query<&mut Engine>,
) {
    for mut engine in &mut engine_query {
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

pub fn update_projectile_velocity(
    time: Res<Time>,

    //weather data
    weather_meta: Res<WeatherMeta>,
    wind: Res<Wind>,
    temperature: Res<Temperature>,
    pressure: Res<Pressure>,

    mut query: Query<
        (
            &mut Velocity,
            &mut GForceCache,
            &GlobalPosition,
            &Transform,
            &Mass,
            &CrossSectionArea,
            &WingArea,
            &Engine,
        ),
        (With<Projectile>, Without<Grounded>),
    >,
    mass_components: Query<&MassData, With<MassComponent>>,
) {
    for (
        mut velocity,
        mut g_force_cache,
        position,
        transform,
        masses,
        cross_section,
        wing_area,
        engine,
    ) in &mut query
    {
        let dt = time.delta_secs();

        let forward = transform.rotation * Vec3::X;
        let up = transform.rotation * Vec3::Y;

        let speed = velocity.length();

        // mass
        let mass: f32 = get_weight(masses, &mass_components);

        // positional_data
        let lat: f32 = get_lat(position.x as f32); // get lat and lon takes in f64
        let lon: f32 = get_lon(position.z as f32);
        let altitude: f32 = altitude(position.y as f32);

        // weather data
        let temperature: f32 = get_temperature(lat, lon, altitude, &weather_meta, &temperature);
        let pressure: f32 =
            get_pressure(lat, lon, altitude, &weather_meta, &pressure, &temperature);
        let wind = get_wind(lat, lon, altitude, &weather_meta, &wind);

        // --- Forces ---
        let thrust = engine.thrust_vector(transform);

        let drag_force = drag_force(cross_section.area, &velocity, temperature, pressure);

        let lift_force = lift_force(&forward, &velocity, &up, air_density(pressure, temperature));

        let gravity_force = Vec3::new(0.0, -mass * GRAVITY, 0.0);

        let total_force = thrust + drag_force + lift_force + gravity_force;
        let acceleration = total_force / mass;

        velocity.0 += acceleration * dt;

        g_force_cache.net_force = total_force.clone();
        g_force_cache.mass = mass;

        // velocity.x+= wind.0;
        // velocity.z+= wind.1;

        // let max_speed = 590.0;
        // if vel.0.length() > max_speed {
        //     vel.0 = vel.0.normalize() * max_speed;
        // }

        if velocity.0.x.is_nan() {
            panic!();
        }

        println!(
            "pos: {:#?} rot: {:#?}\n lat:{:?} lon: {:?} altitude: {:?}",
            &transform.translation,
            &transform.rotation.to_euler(EulerRot::XYZ),
            lat,
            lon,
            altitude
        );
        println!(
            "Velocity: {}, Total Force: {}",
            vec3_fmt(velocity.0),
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
    mut moving_center: ResMut<MovingOrigin>,

    //weather data
    weather_meta: Res<WeatherMeta>,
    wind: Res<Wind>,
    temperature: Res<Temperature>,
    pressure: Res<Pressure>,

    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Velocity,
            &BrakeForce,
            &SteeringWheel,
            &Transform,
            &mut GlobalPosition,
            &Mass,
            &CrossSectionArea,
            &WingArea,
            &Engine,
        ),
        (With<Projectile>, With<Grounded>),
    >,
    mass_components: Query<&MassData, With<MassComponent>>,
) {
    const ROLLING_RESISTANCE: f32 = 0.8;

    for (
        entity,
        mut velocity,
        brake,
        wheel,
        transform,
        mut position,
        masses,
        cross_section,
        wing_area,
        engine,
    ) in &mut query
    {
        let dt = time.delta_secs();

        let forward = transform.rotation * Vec3::X;
        let right = transform.rotation * Vec3::Z;
        let up = transform.rotation * Vec3::Y;

        let speed = velocity.length();
        let velocity_dir = if speed > 0.001 {
            velocity.normalize()
        } else {
            forward
        };
        // positional_data
        let lat: f32 = get_lat(position.x as f32); // get lat and lon takes in f64
        let lon: f32 = get_lon(position.z as f32);
        let altitude: f32 = altitude(position.y as f32);

        // weather data
        let temperature: f32 = get_temperature(lat, lon, altitude, &weather_meta, &temperature);
        let pressure: f32 =
            get_pressure(lat, lon, altitude, &weather_meta, &pressure, &temperature);
        let wind = get_wind(lat, lon, altitude, &weather_meta, &wind);

        // mass
        let mass: f32 = get_weight(masses, &mass_components);

        // --- Forces ---
        let thrust = engine.thrust_vector(transform);

        let drag_force = drag_force(cross_section.area, &velocity, temperature, pressure);
        // let drag_force =
        //     -velocity_dir * 0.5 * AIR_DENSITY * speed * speed * drag.0 * cross_section.0;

        let brake_force = if brake.1 {
            -velocity_dir * brake.0
        } else {
            Vec3::ZERO
        };

        let rolling_resistance = -velocity_dir * speed * ROLLING_RESISTANCE;

        let lift_force = lift_force(&forward, &velocity, &up, air_density(pressure, temperature));

        let gravity_force = Vec3::new(0.0, -mass * GRAVITY, 0.0);

        // total force (no lateral friction)
        let total_force =
            thrust + drag_force + brake_force + rolling_resistance + lift_force + gravity_force;

        let acceleration = total_force / mass;
        velocity.0 += acceleration * dt;

        // --- Remove lateral velocity ---
        // Project velocity onto forward vector and discard sideways (right) component
        let forward_vel = forward.normalize() * velocity.0.dot(forward.normalize());
        velocity.0 = forward_vel + Vec3::Y * velocity.0.dot(Vec3::Y);

        // Prevent negative vertical velocity while grounded
        velocity.y = velocity.y.max(0.0);

        // velocity.x+= wind.0;
        // velocity.z+= wind.1;

        // let max_speed = 590.0;
        // if velocity.0.length() > max_speed {
        //     velocity.0 = vel.0.normalize() * max_speed;
        // }

        println!(
            "\nVelocity: {} | Total: {}",
            vec3_fmt(velocity.0),
            vec3_fmt(total_force)
        );
        println!(
            "Thrust: {} | Brake: {} | RollRes: {} | Lift: {} | Drag: {} | Gravity: {}",
            vec3_fmt(thrust),
            vec3_fmt(brake_force),
            vec3_fmt(rolling_resistance),
            vec3_fmt(lift_force),
            vec3_fmt(drag_force),
            vec3_fmt(gravity_force),
        );

        if velocity.y > 0.0 {
            position.x = transform.translation.x as f64;
            position.y = transform.translation.y as f64;
            position.z = transform.translation.z as f64;

            // TODO! - only do this if the entity is the player
            moving_center.0 = Some(entity);

            commands.entity(entity).remove::<Grounded>();
            println!("✈️  Takeoff! The CF-104 is airborne.");
        }
    }
}

pub fn update_transform(
    time: Res<Time>,
    center: Res<MovingOrigin>,
    mut query: Query<(
        &mut Transform,
        &mut GlobalPosition,
        &Velocity,
        &AngularVelocity,
    )>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut position, velocity, angular_velocity) in &mut query {
        position.x += (velocity.x * dt) as f64;
        position.y += (velocity.y * dt) as f64;
        position.z += (velocity.z * dt) as f64;

        if center.0.is_none() {
            transform.translation += velocity.0 * dt;
        }
        println!("position: {position:?}");

        // transform.translation += velocity.0 * dt;

        let omega = angular_velocity.0;
        if omega.length_squared() > 1e-8 {
            let roll_angle: f32 = omega.x * dt;
            let pitch_angle: f32 = omega.z * dt;
            let yaw_angle: f32 = omega.y * dt;

            let right: Dir3 = transform.forward();
            let forward: Dir3 = transform.right();
            let up: Dir3 = transform.up();

            transform.rotate_axis(forward, roll_angle);
            transform.rotate_axis(right, pitch_angle);
            transform.rotate_axis(up, yaw_angle);
        }
    }
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(WeatherPlugin).add_systems(
            FixedUpdate,
            (
                update_cross_section,
                update_tank_flow_rate,
                update_fuel_mass_system,
                update_engine_thrust,
                update_angular_projectile_velocity,
                apply_angular_damping,
                update_grounded_turn,
                update_projectile_velocity,
                update_grounded_velocity,
                update_transform,
            ),
        );
    }
}
