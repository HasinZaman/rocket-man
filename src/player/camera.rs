use crate::cf104::Plane;
use crate::player::{Player, Selectable};
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use std::default;
use std::f32::consts::PI;

#[derive(Component)]
pub struct OutlineCamera;

#[derive(Resource, Default)]
pub struct OutlineTexture(pub Handle<Image>);

#[derive(Resource, Default)]
pub struct MaskMaterials {
    pub black: Handle<StandardMaterial>,
    pub white: Handle<StandardMaterial>,
}

pub fn mask_mesh<const BACKGROUND: bool>(
    mask_materials: &Res<MaskMaterials>,
    mesh: Handle<Mesh>,
    parent_entity: Entity,
    commands: &mut Commands,
) {
    if BACKGROUND {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mask_materials.black.clone()),
            Transform::default(),
            RenderLayers::layer(1),
            ChildOf(parent_entity),
        ));
    } else {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mask_materials.black.clone()),
            Transform::default(),
            RenderLayers::layer(1),
            Selectable,
            // SobelSettings{ threshold: 0.05 },
            ChildOf(parent_entity),
        ));
    }
}

pub fn setup_mask_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let black = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        ..default()
    });
    let white = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });
    commands.insert_resource(MaskMaterials { black, white });
}

#[derive(Component)]
pub struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002))
    }
}

pub fn set_up_player_camera(
    commands: &mut Commands,
    transform: Transform,
    images: &mut ResMut<Assets<Image>>,
    parent: Option<Entity>,
) -> Entity {
    let (camera, sensitivity) = (Camera3d::default(), CameraSensitivity::default());

    let camera_id = match parent {
        Some(parent_id) => commands
            .spawn((
                Player,
                camera,
                sensitivity,
                transform,
                RenderLayers::layer(0),
                ChildOf(parent_id),
            ))
            .id(),
        None => commands
            .spawn((
                Player,
                camera,
                sensitivity,
                transform,
                RenderLayers::layer(0),
            ))
            .id(),
    };

    let size = Extent3d {
        width: 1920,
        height: 1080,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("outline_render_texture"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[TextureFormat::Rgba8UnormSrgb],
        },
        ..default()
    };
    image.resize(size);

    let outline_texture = images.add(image);

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            target: RenderTarget::Image(outline_texture.clone().into()),
            ..default()
        },
        OutlineCamera,
        RenderLayers::layer(1),
        Transform::IDENTITY,
        ChildOf(camera_id),
    ));

    commands.insert_resource(OutlineTexture(outline_texture));

    camera_id
}

pub fn look_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    plane_transform: Single<
        &Transform,
        (
            With<Player>,
            With<Plane>,
            Without<Camera3d>,
            Without<CameraSensitivity>,
        ),
    >,
    mut cam_query: Query<(&mut Transform, &CameraSensitivity), With<Player>>,
) {
    let Ok((mut cam_transform, sensitivity)) = cam_query.single_mut() else {
        return;
    };

    let delta = accumulated_mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    let delta_yaw = -delta.x * sensitivity.0.x;
    let delta_pitch = -delta.y * sensitivity.0.y;

    let _plane_forward = plane_transform.back();
    let plane_up = plane_transform.down();
    let plane_right = plane_transform.left();

    let yaw_rotation = Quat::from_axis_angle(*plane_right, delta_yaw);
    let pitch_rotation = Quat::from_axis_angle(*plane_up, delta_pitch);

    cam_transform.rotation = yaw_rotation * pitch_rotation * cam_transform.rotation;

    let (mut yaw, mut pitch, _) = cam_transform.rotation.to_euler(EulerRot::XYZ);

    pitch = pitch.clamp(-1., 1.);

    yaw = yaw.clamp(0.5, 3. * PI / 4.);

    cam_transform.rotation = Quat::from_euler(EulerRot::XYZ, yaw, pitch, 0.);
}
