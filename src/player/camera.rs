use crate::cf104::Plane;
use crate::player::controls::{KeyBindings, KeyState};
use crate::player::{Player, Selectable};
use bevy::camera::RenderTarget;
use bevy::camera::primitives::CubeMapFace;
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::Skybox;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseWheel};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use std::default;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, PI};
use std::ops::DerefMut;

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
    asset_server: &Res<AssetServer>,
    images: &mut ResMut<Assets<Image>>,
    parent: Option<Entity>,
) -> Entity {
    let (camera, sensitivity) = (Camera3d::default(), CameraSensitivity::default());

    // let cube_handle = images.add(cubemap);

    // let skybox_handle: Handle<Image> = asset_server.load("sky_box/Ryfjallet_cubemap_astc4x4.ktx2");

    let audio_listener = SpatialListener::new(0.18);
    let camera_id = match parent {
        Some(parent_id) => commands
            .spawn((
                Player,
                camera,
                // Skybox {
                //     image: skybox_handle.clone(),
                //     brightness: 1000.0,
                //     ..default()
                // },
                audio_listener,
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
                // Skybox {
                //     image: skybox_handle.clone(),
                //     brightness: 1000.0,
                //     ..default()
                // },
                audio_listener,
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
    key_bindings: Res<KeyBindings>,
    mut cam_query: Query<(&mut Transform, &mut Projection, &CameraSensitivity), With<Player>>,
) {
    let Ok((mut cam_transform, mut projection, sensitivity)) = cam_query.single_mut() else {
        return;
    };

    let zoom_in =
        key_bindings.zoom.state == KeyState::Held || key_bindings.zoom.state == KeyState::Pressed;

    // put this into a res and let a diffrent sytem handle FOV
    const DEFAULT_FOV: f32 = FRAC_PI_3;
    const ZOOMED_FOV: f32 = PI / 10.;
    if let Projection::Perspective(ref mut perspective) = *projection {
        if zoom_in {
            perspective.fov = ZOOMED_FOV;
        } else {
            perspective.fov = DEFAULT_FOV;
        }
    }

    let delta = accumulated_mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    let delta_yaw = -delta.x * sensitivity.0.x;
    let delta_pitch = -delta.y * sensitivity.0.y;

    let (mut pitch, mut yaw, _) = cam_transform.rotation.to_euler(EulerRot::XYZ);

    yaw += delta_yaw;
    pitch += delta_pitch;

    pitch = pitch.clamp(-PI / 3., PI / 3.);

    yaw = yaw.clamp(-PI / 2. - 0.5, PI / 2. + 0.5);

    cam_transform.rotation = Quat::from_euler(EulerRot::XYZ, pitch, yaw, 0.);

    let mut pos = cam_transform.translation;

    const MAX_ANGLE: f32 = PI / 2. + 0.5;
    const MIN_ANGLE: f32 = PI / 3.;
    const RANGE: f32 = (MAX_ANGLE - MIN_ANGLE);

    const DX: f32 = -0.3 / RANGE;
    const DY: f32 = 0.3 / RANGE;
    pos.x = match yaw.abs() > PI / 3. {
        false => 0.,
        true => {
            let sign = yaw.signum();
            let delta = yaw.abs() - MIN_ANGLE;
            let offset = delta.clamp(0.0, RANGE);

            DX * offset * sign
        }
    };

    pos.z = match yaw.abs() > PI / 3. {
        false => 0.,
        true => {
            let sign = yaw.signum();
            let delta = yaw.abs() - MIN_ANGLE;
            let offset = delta.clamp(0.0, RANGE);

            DY * offset * sign
        }
    };

    cam_transform.translation = pos;
}
