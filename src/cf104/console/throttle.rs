use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

use crate::cf104::CF104_BODY_ASSET_PATH;
use crate::{
    cf104::console::RotRange,
    player::camera::{MaskMaterials, mask_mesh},
};

#[derive(Component)]
pub struct Throttle(pub f32);

impl Default for Throttle {
    fn default() -> Self {
        Self(0.)
    }
}

pub fn spawn_throttle<const MESH: u32>(
    transform: Transform,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mask_materials: &Res<MaskMaterials>,
    parent_id: Entity,
) {
    {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", MESH));
        let material_handle = materials.add(StandardMaterial::default());

        mask_mesh::<false>(
            mask_materials,
            mesh.clone(),
            commands
                .spawn((
                    Throttle::default(),
                    RotRange {
                        min: Quat::from_xyzw(0.5193636417388916, 0., 0., 0.8545534610748291),
                        max: Quat::from_xyzw(-0.114098, 0., 0., 0.99347),
                    },
                    Name::new("Throttle"),
                    Mesh3d(mesh.clone()),
                    NoFrustumCulling,
                    MeshMaterial3d(material_handle),
                    transform,
                    ChildOf(parent_id),
                ))
                .id(),
            commands,
        )
    };
}
