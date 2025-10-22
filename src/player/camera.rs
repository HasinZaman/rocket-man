use crate::cf104::Plane;
use crate::player::controls::{KeyBindings, KeyState};
use crate::player::ui::BlackoutRedout;
use crate::player::{Player, Selectable};
use crate::projectile::util::GRAVITY;
use crate::projectile::{AngularVelocity, GForceCache};
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use std::f32::consts::{FRAC_PI_3, PI};

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
pub struct HeadSetSpeaker;

#[derive(Component)]
pub struct SpeakerSink;

pub fn spawn_headset_with_speakers(commands: &mut Commands, parent: Entity) {
    commands.spawn((
        HeadSetSpeaker,
        Transform::IDENTITY,
        ChildOf(parent),
    ));
}

#[derive(Component)]
pub struct CameraSensitivity(Vec2);

#[derive(Component)]
pub struct FOVMaxRange(f32, f32);

#[derive(Component)]
pub struct FOVMinRange(f32, f32);

#[derive(Component)]
pub struct FOVGoal(f32);
#[derive(Component)]
pub struct FOVSpeed(f32);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002))
    }
}

#[derive(Component, Default)]
pub struct CameraShake(Vec3);

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
                FOVMaxRange(FRAC_PI_3, PI / 10.),
                FOVMinRange(FRAC_PI_3, PI / 10.),
                FOVGoal(0.),
                FOVSpeed(15.),
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

    spawn_headset_with_speakers(commands, camera_id);
    // create player headset for radio chatter + music

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
        FOVMaxRange(FRAC_PI_3, PI / 10.),
        FOVMinRange(FRAC_PI_3, PI / 10.),
        FOVGoal(0.),
        FOVSpeed(15.),
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
    fov_query: Query<&mut FOVGoal>,
) {
    let Ok((mut cam_transform, mut projection, sensitivity)) = cam_query.single_mut() else {
        return;
    };

    let zoom_in =
        key_bindings.zoom.state == KeyState::Held || key_bindings.zoom.state == KeyState::Pressed;

    // put this into a res and let a diffrent sytem handle FOV
    if let Projection::Perspective(ref mut perspective) = *projection {
        if zoom_in {
            for mut goal in fov_query {
                goal.0 = 100.;
            }
            // perspective.fov = ZOOMED_FOV;
        } else {
            for mut goal in fov_query {
                goal.0 = 0.;
            }
            // perspective.fov = DEFAULT_FOV;
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

pub fn visualize_gs(
    camera: Single<&GlobalTransform, (With<Player>, With<Camera3d>)>,
    plane: Single<(&GForceCache, &AngularVelocity, &GlobalTransform), (With<Player>, With<Plane>)>,

    mut fov_query: Query<(&mut FOVMinRange, &FOVMaxRange, &mut FOVSpeed)>,
    mut black_out: Single<&mut BackgroundGradient, With<BlackoutRedout>>,
) {
    let vertical_g_force: f32 = {
        let (g_force_cache, angular_velocity, plane_global_transform) = plane.into_inner();

        let (camera_global_transform) = camera.into_inner();
        let pilot_up_vector: Vec3 = plane_global_transform.up().normalize();

        let linear_acceleration: Vec3 = match g_force_cache.net_force.length() <= 0.00001 {
            true => Vec3::ZERO,
            false => -1. * g_force_cache.net_force / g_force_cache.mass, // - Vec3::new(0.0, GRAVITY, 0.0),
        };

        let rotational_acceleration: f32 = match angular_velocity.0.length_squared() <= 1e-12 {
            true => 0.,
            false => {
                let relative_position =
                    camera_global_transform.translation() - plane_global_transform.translation();

                let forward_dist = relative_position
                    .project_onto(*plane_global_transform.right())
                    .length()
                    .abs();
                let vertical_dist = relative_position
                    .project_onto(*plane_global_transform.up())
                    .length()
                    .abs();
                (0.5 * angular_velocity.x.powf(2.) * vertical_dist).abs()
                    + -1.
                        * angular_velocity.z.signum()
                        * 0.25
                        * angular_velocity.z.powf(2.)
                        * forward_dist
                        * 0.75
            }
        };
        let total_acceleration: Vec3 = linear_acceleration * 0.9;

        let projected_vertical_acceleration: Vec3 =
            total_acceleration.project_onto(pilot_up_vector);

        (projected_vertical_acceleration.length()
            * projected_vertical_acceleration
                .dot(pilot_up_vector)
                .signum()
            + rotational_acceleration)
            / GRAVITY
    };

    // println!(
    //     "Total G-force experienced by pilot: {:.2} g",
    //     vertical_g_force
    // );

    for gradient in black_out.0.iter_mut() {
        if let Gradient::Radial(RadialGradient { stops, .. }) = gradient {
            if vertical_g_force <= 0. {
                *stops = vec![
                    ColorStop::new(
                        Color::srgba(
                            0.0,
                            0.0,
                            0.0,
                            ((vertical_g_force.abs() - 7.0) / 2.0).clamp(0., 1.),
                        ),
                        Val::Percent(0.0),
                    ),
                    ColorStop::new(
                        Color::srgba(
                            0.0,
                            0.0,
                            0.0,
                            ((vertical_g_force.abs() - 6.) / 2.0).clamp(0., 1.),
                        ),
                        Val::Percent(25.0),
                    ),
                    ColorStop::new(
                        Color::srgba(
                            0.0,
                            0.0,
                            0.0,
                            ((vertical_g_force.abs() - 4.5) / 2.0).clamp(0., 1.),
                        ),
                        Val::Percent(50.0),
                    ),
                    ColorStop::new(
                        Color::srgba(
                            0.0,
                            0.0,
                            0.0,
                            ((vertical_g_force.abs() - 4.1) / 1.0).clamp(0., 1.),
                        ),
                        Val::Percent(75.0),
                    ),
                    ColorStop::new(
                        Color::srgba(
                            0.0,
                            0.0,
                            0.0,
                            ((vertical_g_force.abs() - 4.) / 0.5).clamp(0., 1.),
                        ),
                        Val::Percent(100.0),
                    ),
                ];

                
                for (mut min, max, mut speed) in fov_query.iter_mut() {
                    update_fov_from_gs(
                        vertical_g_force.abs(),
                        &mut min,
                        &max,
                        &mut speed
                    )
                }
            } else {
                const RED: f32 = 0.1;
                *stops = vec![
                    ColorStop::new(
                        Color::srgba(
                            RED,
                            0.0,
                            0.0,
                            ((vertical_g_force.abs() - 2.) / 1.0).clamp(0., 1.),
                        ),
                        Val::Percent(0.0),
                    ),
                    // ColorStop::new(
                    //     Color::srgba(
                    //         RED,
                    //         0.0,
                    //         0.0,
                    //         ((vertical_g_force.abs() - 1.0) / 2.0).clamp(0., 1.),
                    //     ),
                    //     Val::Percent(25.0),
                    // ),
                    ColorStop::new(
                        Color::srgba(
                            RED,
                            0.0,
                            0.0,
                            ((vertical_g_force.abs() - 1.0) / 1.0).clamp(0., 1.),
                        ),
                        Val::Percent(50.0),
                    ),
                    ColorStop::new(
                        Color::srgba(
                            RED,
                            0.0,
                            0.0,
                            ((vertical_g_force.abs() - 0.5) / 1.).clamp(0., 1.),
                        ),
                        Val::Percent(75.0),
                    ),
                    ColorStop::new(
                        Color::srgba(
                            RED,
                            0.0,
                            0.0,
                            ((vertical_g_force.abs() - 0.5) / 0.5).clamp(0., 1.),
                        ),
                        Val::Percent(100.0),
                    ),
                ];

                for (mut min, max, mut speed) in fov_query.iter_mut() {
                    update_fov_from_gs(
                        0.,
                        &mut min,
                        &max,
                        &mut speed
                    )
                }
            }
        }
    }
}

pub fn update_fov_from_gs(
    vertical_gs: f32,
    min_range: &mut FOVMinRange,
    max_range: &FOVMaxRange,
    speed: &mut FOVSpeed,
) {
    let gs = vertical_gs.abs();

    const START: f32 = 3.0;
    const END: f32 = 5.0;

    let half: f32 = max_range.1 + (max_range.0 - max_range.1) * 0.5;

    min_range.0 = (-1. * half / (END - START) * (gs - START) + max_range.0).clamp(max_range.1, max_range.0);
    
    let half: f32 = 1. + (15. - 1.) * 0.5;
    speed.0 = (-1. * half/(END-START)*(gs-START) + 15.).clamp(1., 15.);
}

pub fn update_fov(
    time: Res<Time>,
    camera_query: Query<(&mut Projection, &FOVGoal, &FOVMaxRange, &FOVMinRange, &FOVSpeed)>,
) {
    let delta_time = time.delta_secs();
    for (mut projection, FOVGoal(goal), FOVMaxRange(min, max), FOVMinRange(inf, sup), FOVSpeed(speed)) in
        camera_query
    {
        if let Projection::Perspective(ref mut perspective) = *projection {
            let current_fov: f32 = perspective.fov;

            let target_fov: f32 = if *goal >= 100.0 {
                *max
            } else {
                *inf + (*max - *inf) * (*goal / 100.0)
            };

            let new_fov: f32 = current_fov + (target_fov - current_fov) * speed * delta_time;

            perspective.fov = new_fov.clamp(*sup, *inf);
        }
    }
}
