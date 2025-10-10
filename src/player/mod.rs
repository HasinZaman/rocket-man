

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    color::palettes::css, core_pipeline::tonemapping::Tonemapping, math::vec3,
    picking::backend::ray::RayMap, post_process::bloom::Bloom, prelude::*,
};

use crate::{player::{camera::look_camera, ui::{center_cursor, fullscreen_startup, hide_cursor}}};

pub mod camera;

pub mod ui;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (hide_cursor, fullscreen_startup, setup_focus_material))
            .add_systems(Update, (look_camera, center_cursor, check_camera_selection));
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

#[derive(Resource)]
pub struct FocusMaterial {
    pub handle: Handle<StandardMaterial>,
}

fn setup_focus_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let highlight = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0), // yellow
        emissive: LinearRgba::new(1., 1., 1., 1.),
        unlit: true,
        ..default()
    });

    // Store it as a resource for reuse
    commands.insert_resource(FocusMaterial { handle: highlight });
}

fn check_camera_selection(
    focus_material: Res<FocusMaterial>,
    camera_transform: Single<&GlobalTransform, With<Camera3d>>,

    mut commands: Commands,
    
    mut raycast: MeshRayCast,
    
    selectable_query: Query<(Entity, &Transform, &Mesh3d, &Name), (With<Selectable>, Without<Selected>, Without<Focused>)>,
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
        // remove focus
        for (focused_entity, children) in remove_focus_query.iter() {
            commands.entity(focused_entity).remove::<Focused>();

            // Clean up any outline meshes that were attached as children
            for child_entity in children.iter() {
                commands.entity(child_entity).despawn();
            }
        }

        if let Ok((_, transform,  mesh, name)) = selectable_query.get(entity) {
            let mesh_handle = &mesh.0;
            let mut outline_transform = Transform::default();

            outline_transform.translation = Vec3::ZERO;

            outline_transform.scale = 1.005 * transform.scale;

            outline_transform.scale.x *= -1.;

            commands.spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(focus_material.handle.clone()),
                outline_transform,
                ChildOf(entity)
            ));
            commands.entity(entity)
                .insert(Focused);
            
            println!(
                "Camera is looking at entity {:?} named '{}' at {:?}",
                entity, name.as_str(), transform
            );
        } else {
            for (focused_entity, children) in remove_focus_query.iter() {
                commands.entity(focused_entity).remove::<Focused>();
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }
        }
    }
}

