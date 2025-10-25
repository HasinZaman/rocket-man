use bevy::{
    asset::{AssetServer, Assets, Handle},
    camera::visibility::Visibility,
    ecs::{
        entity::Entity,
        hierarchy::ChildOf,
        system::{Commands, Res, ResMut},
    },
    math::{Quat, Vec3},
    mesh::{Mesh, Mesh3d},
    pbr::{MeshMaterial3d, StandardMaterial},
    transform::components::Transform,
};

use crate::world::{props::Prop, GlobalPosition};

pub fn spawn_lahr_airbase(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,

    transform: Transform,
) -> Entity {
    const ASSET_PATHS: &'static str = "lahrs_airfeild/assets.gltf";
    let air_base: Entity = commands.spawn((transform, Visibility::Inherited, Prop)).id();
    // runway
    {
        let body_id = {
            let parent_mesh_handle: Handle<Mesh> =
                asset_server.load(&format!("{ASSET_PATHS}#Mesh{}/Primitive0", 0));
            let parent_material_handle = materials.add(StandardMaterial::default());

            let mut transform = Transform::default();

            transform.translation = Vec3::splat(0.);

            commands
                .spawn((
                    Mesh3d(parent_mesh_handle),
                    MeshMaterial3d(parent_material_handle),
                    transform,
                    ChildOf(air_base),
                    Visibility::Inherited,
                ))
                .id()
        };
    }

    // runway
    {
        let body_id = {
            let parent_mesh_handle: Handle<Mesh> =
                asset_server.load(&format!("{ASSET_PATHS}#Mesh{}/Primitive0", 1));
            let parent_material_handle = materials.add(StandardMaterial::default());

            let mut transform = Transform::default();

            transform.translation = Vec3 {
                x: 0.,
                y: 0.,
                z: 83.74634552001953,
            };

            commands
                .spawn((
                    Mesh3d(parent_mesh_handle),
                    MeshMaterial3d(parent_material_handle),
                    transform,
                    ChildOf(air_base),
                    Visibility::Inherited,
                ))
                .id()
        };
    }

    // hangers
    for i in 0..4 {
        let hanger = {
            let parent_mesh_handle: Handle<Mesh> =
                asset_server.load(&format!("{ASSET_PATHS}#Mesh{}/Primitive0", 2));
            let parent_material_handle = materials.add(StandardMaterial::default());

            let mut transform = Transform::default();

            transform.translation = Vec3 {
                x: 39.25777053833008 + i as f32 * 50.,
                y: 1.,
                z: 169.4016571044922,
            };

            transform.rotation = Quat::from_xyzw(0.7071068286895752, 0., 0., 0.7071068286895752);

            commands
                .spawn((
                    Mesh3d(parent_mesh_handle),
                    MeshMaterial3d(parent_material_handle),
                    transform,
                    ChildOf(air_base),
                    Visibility::Inherited,
                ))
                .id()
        };
    }

    air_base
}
