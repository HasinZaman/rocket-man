use std::{
    f32::consts::FRAC_PI_2,
    ops::{Add, Sub},
};

use bevy::{
    app::{FixedUpdate, Plugin, Startup},
    asset::{AssetServer, Assets},
    camera::{Camera, ClearColor},
    color::{Color, Srgba},
    ecs::{
        entity::Entity,
        query::{With, Without},
        resource::Resource,
        system::{Commands, Query, Res, ResMut, Single},
    },
    light::{AmbientLight, DirectionalLight},
    math::{Quat, Vec3, primitives::Circle},
    mesh::{Mesh, Mesh3d},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::Component,
    transform::components::{GlobalTransform, Transform},
    utils::default,
};

use crate::{
    player::Player,
    world::{
        ground::GroundPlugin,
        props::{PropPlugin, SpawnPropsMessage},
    },
};

mod ground;
mod props;
pub mod util;

#[derive(Resource, Default)]
pub struct MovingOrigin(pub Option<Entity>);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct GlobalPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Sub<GlobalPosition> for GlobalPosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        GlobalPosition {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl Add<GlobalPosition> for GlobalPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        GlobalPosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl GlobalPosition {
    pub fn zero() -> Self {
        GlobalPosition {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
    pub fn splat(val: f64) -> Self {
        GlobalPosition {
            x: val,
            y: val,
            z: val,
        }
    }

    pub fn dist(&self, other: &Self) -> f64 {
        let (x_delta, y_delta, z_delta) = (self.x - other.x, self.y - other.y, self.z - other.z);

        (x_delta * x_delta + y_delta * y_delta + z_delta * z_delta).sqrt()
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((GroundPlugin, PropPlugin))
            .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.08)))
            .init_resource::<MovingOrigin>()
            .add_systems(Startup, (setup_world, sky_box_follow_camera))
            .add_systems(FixedUpdate, moving_origin);
    }
}

fn moving_origin(
    center: Res<MovingOrigin>,
    query: Query<(Entity, &mut Transform, &GlobalPosition)>,
) {
    let Some(center_entity) = center.0 else {
        return;
    };

    let center = {
        let Ok((_, _, center)) = query.get(center_entity) else {
            panic!("Invalid state");
            // return;
        };

        center.clone()
    };

    for (entity, mut transform, global_position) in query {
        let new_translation: Vec3 = Vec3 {
            x: (global_position.x - center.x) as f32,
            y: (global_position.y - center.y) as f32,
            z: (global_position.z - center.z) as f32,
        };

        // println!("({entity:?}) := {global_position:?} - {center:?} = {new_translation:?}");

        transform.translation = new_translation;
    }
}

#[derive(Component)]
pub struct Skybox;

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.05, 0.08, 0.12),
        brightness: 0.05,
        affects_lightmapped_meshes: true,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 100., //1000.0,
            shadows_enabled: true,
            color: Color::srgb(0.6, 0.65, 0.85),
            ..default()
        },
        Transform::from_xyz(-10.0, 20.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // commands.spawn((
    //     Mesh3d(meshes.add(Circle::new(100_000.0))),
    //     MeshMaterial3d(materials.add(Color::Srgba(Srgba {
    //         red: 0.,
    //         green: 0.75,
    //         blue: 0.,
    //         alpha: 1.,
    //     }))),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    //     GlobalPosition {
    //         x: 0.,
    //         y: 0.,
    //         z: 0.,
    //     },
    // ));
}

pub fn sky_box_follow_camera(
    camera_q: Single<&GlobalTransform, (With<Camera>, With<Player>)>,
    mut skybox_q: Single<&mut Transform, (With<Skybox>, Without<Camera>)>,
) {
    skybox_q.translation = camera_q.translation().clone() - Vec3::splat(50.);
}
