use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

use crate::player::camera::{MaskMaterials, mask_mesh};

use crate::cf104::CF104_CONSOLE_ASSET_PATH;

#[derive(Component, Debug)]
pub struct RadioFxSelector(u8, f32);
#[derive(Component, Debug)]
pub struct RadioVolume(f32);

pub fn spawn_radio<const FRAME_MESH: u32, const CHANNEL_MESH: u32, const VOLUME_MESH: u32>(
    transform: Transform,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mask_materials: &Res<MaskMaterials>,
    console_material: &Handle<StandardMaterial>,
    parent_id: Entity,
) {
    // mesh frame
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        FRAME_MESH
    ));
    let material_handle = console_material.clone();

    let radio_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
            ChildOf(parent_id),
        ))
        .id();

    //
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        CHANNEL_MESH
    ));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        emissive_texture: Some(asset_server.load("cf104/radio_dial.png")),
        emissive: LinearRgba {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        }, // intensity multiplier
        ..default()
    });
    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: 0.,
        y: -0.022010857239365578,
        z: -0.01099395751953125,
    };
    transform.rotation = Quat::from_xyzw(0.7071068286895752, 0., 0., 0.7071068286895752);
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material_handle.clone()),
        RadioFxSelector(0, 0.),
        // Visibility::Visible,
        NoFrustumCulling,
        transform,
        ChildOf(radio_id),
    ));
    mask_mesh::<false>(mask_materials, mesh.clone(), radio_id, commands);
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        VOLUME_MESH
    ));
    let material_handle = console_material.clone();
    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: 0.038504600524902344,
        y: -0.011331111192703247,
        z: 0.0514528751373291,
    };
    let fz_dial = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            RadioVolume(100.),
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
            ChildOf(radio_id),
        ))
        .id();
    mask_mesh::<false>(mask_materials, mesh.clone(), fz_dial, commands);
    let mesh: Handle<Mesh> =
        asset_server.load(&format!("{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0", 23));
    let material_handle = console_material.clone();
    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: 0.05943775177001953,
        y: 0.001281827688217163,
        z: -0.017145991325378418,
    };
    let volume = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
            ChildOf(radio_id),
        ))
        .id();
    mask_mesh::<false>(mask_materials, mesh.clone(), volume, commands);
}
