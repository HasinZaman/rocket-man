use std::f32::consts::TAU;

use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

use crate::{cf104::CF104_CONSOLE_ASSET_PATH, world::GlobalPosition};

#[derive(Component)]
pub struct Altimeter(Entity, Entity, Entity, Entity, Entity);

#[derive(Component)]
pub struct AltimeterTmp;

pub fn update_altimeter(
    parent_query: Query<&GlobalPosition>,
    altimeter_query: Query<(&Altimeter)>,
    mut transform_query: Query<&mut Transform, (With<AltimeterTmp>, Without<Altimeter>)>,
) {
    let wheel_offset: f32 = 200.0_f32.to_radians();

    for altimeter in &altimeter_query {
        let (target, wheel_10k, wheel_1k, wheel_100, needle) = (
            altimeter.0,
            altimeter.1,
            altimeter.2,
            altimeter.3,
            altimeter.4,
        );

        let Ok(parent) = parent_query.get(target) else {
            panic!("Invalid state")
        };

        let altitude: f32 = (parent.y as f32 + 156.) * 3.28084;

        if let Ok(mut transform) = transform_query.get_mut(wheel_10k) {
            let angle = -(altitude / 10_000_00.0) * TAU + wheel_offset;
            transform.rotation = Quat::from_rotation_x(angle);
        }

        if let Ok(mut transform) = transform_query.get_mut(wheel_1k) {
            let angle = -(altitude / 1_000_00.0) * TAU + wheel_offset;
            transform.rotation = Quat::from_rotation_x(angle);
        }

        if let Ok(mut transform) = transform_query.get_mut(wheel_100) {
            let angle = -(altitude / 100_00.0) * TAU + wheel_offset;
            transform.rotation = Quat::from_rotation_x(angle);
        }

        if let Ok(mut transform) = transform_query.get_mut(needle) {
            let angle = (altitude / 100_00.0) * TAU;
            transform.rotation = Quat::from_rotation_y(angle);
        }
    }
}

pub fn spawn_altimeter<
    const WHEEL_1: usize,
    const WHEEL_2: usize,
    const WHEEL_3: usize,
    const NEEDLE: usize,
    const FRAME: usize,
>(
    plane_id: Entity,
    parent_transform: Transform,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    needle_material_handle: &Handle<StandardMaterial>,
    parent_id: Entity,
) {
    // 10000ft marker
    let mesh_10000: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        WHEEL_1
    ));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        emissive_texture: Some(asset_server.load("cf104/circle_dial_1.png")),
        emissive: LinearRgba {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        }, // intensity multiplier
        ..default()
    });
    let mut transform_10000 = Transform::default();
    transform_10000.scale = Vec3::splat(0.6485239863395691);
    transform_10000.translation = Vec3 {
        x: -0.030985355377197266,
        y: 0.014090169221162796,
        z: 0.,
    };
    transform_10000.rotation = Quat::from_array([-0.9859495759010315, 0., 0., 0.1670437753200531]);
    let wheel_1 = commands
        .spawn((
            Mesh3d(mesh_10000),
            MeshMaterial3d(material_handle.clone()),
            AltimeterTmp,
            NoFrustumCulling,
            transform_10000,
        ))
        .id();
    // 1000ft marker
    let mesh_1000: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        WHEEL_2
    ));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        emissive_texture: Some(asset_server.load("cf104/circle_dial_1.png")),
        emissive: LinearRgba {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        }, // intensity multiplier
        ..default()
    });
    let mut transform_1000 = Transform::default();
    transform_1000.scale = Vec3::splat(0.6485239863395691);
    transform_1000.translation = Vec3 {
        x: -0.02058267593383789,
        y: 0.014090169221162796,
        z: 0.,
    };
    transform_1000.rotation = Quat::from_array([0.9855281114578247, 0., 0., 0.16951194405555725]);
    let wheel_2 = commands
        .spawn((
            Mesh3d(mesh_1000),
            MeshMaterial3d(material_handle.clone()),
            AltimeterTmp,
            NoFrustumCulling,
            transform_1000,
        ))
        .id();

    // 100ft marker
    let mesh_100: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        WHEEL_3
    ));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        emissive_texture: Some(asset_server.load("cf104/circle_dial_2.png")),
        emissive: LinearRgba {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        }, // intensity multiplier
        ..default()
    });
    let mut transform_100 = Transform::default();
    transform_100.translation = Vec3 {
        x: -0.0078887939453125,
        y: 0.021565333008766174,
        z: 0.,
    };
    transform_100.rotation = Quat::from_array([0.1538494974374771, 0., 0., 0.9880943298339844]);
    let wheel_3 = commands
        .spawn((
            Mesh3d(mesh_100),
            MeshMaterial3d(material_handle.clone()),
            AltimeterTmp,
            NoFrustumCulling,
            transform_100,
        ))
        .id();
    // Needle
    let mesh_needle: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        NEEDLE
    ));
    let mut transform_needle = Transform::default();
    transform_needle.translation = Vec3 {
        x: 0.,
        y: -0.007229913026094437,
        z: -0.004800081253051758,
    };
    let needle = commands
        .spawn((
            Mesh3d(mesh_needle),
            MeshMaterial3d(needle_material_handle.clone()),
            AltimeterTmp, // reuse as rotating needle marker
            NoFrustumCulling,
            transform_needle,
        ))
        .id();
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        FRAME
    ));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        emissive_texture: Some(asset_server.load("cf104/altimeter.png")),
        emissive: LinearRgba {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        }, // intensity multiplier
        ..default()
    });

    let altimeter_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            Altimeter(plane_id, wheel_1, wheel_2, wheel_3, needle),
            NoFrustumCulling,
            parent_transform,
            ChildOf(parent_id),
        ))
        .id();
    for id in [wheel_1, wheel_2, wheel_3, needle] {
        commands.entity(id).insert(ChildOf(altimeter_id));
    }
}
