use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    app::{Plugin, Startup},
    asset::{AssetServer, Assets},
    camera::{Camera, ClearColor},
    color::{Color, LinearRgba, Srgba},
    ecs::{
        hierarchy::ChildOf,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut, Single},
    },
    light::{AmbientLight, DirectionalLight},
    math::{
        Quat, Vec3,
        primitives::{Circle, Plane3d},
    },
    mesh::{Mesh, Mesh3d, Meshable},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::Component,
    transform::components::{GlobalTransform, Transform},
    utils::default,
};

use crate::{player::Player, world::lahr::spawn_lahr_airbase};

mod lahr;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.08)))
            .add_systems(
                Startup,
                (setup_world, spawn_lahr_airbase, sky_box_follow_camera),
            );
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

    commands.spawn((
        Mesh3d(meshes.add(Circle::new(100_000.0))),
        MeshMaterial3d(materials.add(Color::Srgba(Srgba {
            red: 0.,
            green: 0.75,
            blue: 0.,
            alpha: 1.,
        }))),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}

pub fn sky_box_follow_camera(
    camera_q: Single<&GlobalTransform, (With<Camera>, With<Player>)>,
    mut skybox_q: Single<&mut Transform, (With<Skybox>, Without<Camera>)>,
) {
    skybox_q.translation = camera_q.translation().clone() - Vec3::splat(50.);
}
