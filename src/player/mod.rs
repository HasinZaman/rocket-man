use bevy::prelude::*;

use crate::player::{
    camera::{MaskMaterials, OutlineCamera, OutlineTexture, look_camera, setup_mask_materials},
    controls::{
        Arms, KeyBindings, canopy_door_controller, grounded_controller, joystick_controller,
        radio_fx_controller, radio_volume_controller, select_tool, throttle_controller,
        update_key_bindings,
    },
    ui::{center_cursor, fullscreen_startup, hide_cursor},
};

pub mod camera;
pub mod sobel; // should be moved to camera

pub mod controls;

pub mod ui;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(SobelPlugin)
            .init_resource::<MaskMaterials>()
            .init_resource::<OutlineTexture>()
            .init_resource::<KeyBindings>()
            .init_resource::<Arms>()
            .add_systems(
                Startup,
                (hide_cursor, fullscreen_startup, setup_mask_materials),
            )
            .add_systems(
                Update,
                (
                    look_camera,
                    center_cursor,
                    check_camera_selection,
                    select_tool,
                    update_key_bindings,
                    grounded_controller,
                    throttle_controller,
                    joystick_controller,
                    canopy_door_controller,
                    radio_fx_controller,
                    radio_volume_controller,
                ),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Selectable;

#[derive(Component)]
pub struct Focused;

#[derive(Component)]
pub struct Selected;

fn check_camera_selection(
    mask_materials: Res<MaskMaterials>,
    camera_transform: Single<
        &GlobalTransform,
        (With<Camera3d>, With<Player>, Without<OutlineCamera>),
    >,

    mut commands: Commands,

    mut raycast: MeshRayCast,

    mut selectable_query: Query<
        (Entity, &mut MeshMaterial3d<StandardMaterial>, &Transform),
        (With<Selectable>, Without<Selected>, Without<Focused>),
    >,
    remove_focus_query: Query<(Entity, &Children), (With<Selectable>, With<Focused>)>,
) {
    let ray: Ray3d = Ray3d::new(camera_transform.translation(), camera_transform.forward());

    if let Some((entity, hit)) = raycast
        .cast_ray(ray, &MeshRayCastSettings::default())
        .iter()
        .find_map(|(e, h)| {
            if selectable_query.get(*e).is_ok() {
                Some((*e, h.clone()))
            } else {
                None
            }
        })
    {
        if let Ok((_, mut material, transform)) = selectable_query.get_mut(entity) {
            material.0 = mask_materials.white.clone();

            commands.entity(entity).insert(Focused);

            println!(
                "Camera is looking at entity {:?} at {:?}",
                entity, transform
            );
        } else {
            // remove focus from other selectables
        }
    }
}
