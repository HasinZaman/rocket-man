use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{audio::Volume, camera::visibility::NoFrustumCulling, prelude::*, time::Stopwatch};

use crate::{
    cf104::console::{
        ConsolePlugin, RotRange,
        altimeter::spawn_altimeter,
        clock::spawn_clock,
        gyro_compass::spawn_gyro_compass,
        radio::spawn_radio,
        speedometer::spawn_speedometer,
        throttle::{Throttle, spawn_throttle},
    },
    player::{
        Player,
        camera::{CameraShake, MaskMaterials, mask_mesh, set_up_player_camera},
    },
    projectile::{
        GroundedBundle, PlaneBundle,
        drag::DragTarget,
        mass::{ExternalFuelTankBundle, InternalFuelTankBundle, MassBundle},
    },
};

pub mod console;

// CF104
#[derive(Component)]
pub struct Plane;

#[derive(Component)]
pub struct Joystick(pub Vec2);

impl Default for Joystick {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}

#[derive(Component, Debug)]
pub struct CanopyDoor(pub f32);

impl CanopyDoor {
    pub fn open() -> Self {
        CanopyDoor(100.)
    }

    pub fn close() -> Self {
        CanopyDoor(0.)
    }
}

#[derive(Component, Debug)]
pub struct CanopyDoorHandle;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct RotRange2D {
    pub center: Quat,
    pub radius: Vec2,
}

impl RotRange2D {
    pub fn new(center: Quat, radius: Vec2) -> Self {
        Self { center, radius }
    }

    pub fn to_quat(&self, input: Vec2) -> Quat {
        let clamped = input.clamp(Vec2::splat(-1.0), Vec2::splat(1.0));

        let yaw = self.radius.x * clamped.x;
        let pitch = self.radius.y * clamped.y;

        let offset = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);

        self.center * offset
    }
}

#[derive(Component, Debug)]
pub struct EngineAudio {
    pub spool_up: Handle<AudioSource>,
    pub running_loop: Handle<AudioSource>,
    pub loop_instance: Option<Handle<AudioSource>>,
    pub stopwatch: Stopwatch,
    pub spool_duration: f32,
    running: bool,
}

impl EngineAudio {
    pub fn new(asset_server: &Res<AssetServer>) -> Self {
        Self {
            spool_up: asset_server.load("cf104/spool_up.ogg"),
            running_loop: asset_server.load("cf104/running.ogg"),
            loop_instance: None,
            stopwatch: Stopwatch::default(),
            spool_duration: 17.0,
            running: false,
        }
    }
    pub fn start_up_engine(
        mut commands: Commands,
        throttle: Single<&Throttle>,
        mut query: Query<(Entity, &mut EngineAudio), Without<AudioPlayer>>,
    ) {
        for (entity, mut engine_audio) in &mut query {
            // Start engine only when throttle applied
            if throttle.0 > 0.05 {
                println!("Starting engine spool-up sound...");

                let audio: AudioPlayer = AudioPlayer::new(engine_audio.spool_up.clone());

                engine_audio.loop_instance = Some(engine_audio.spool_up.clone());
                engine_audio.stopwatch.reset();

                commands.entity(entity).insert(audio);
            }
        }
    }
    pub fn update_sound(
        time: Res<Time>,
        mut commands: Commands,
        throttle: Single<&Throttle>,
        canopy_door: Single<&CanopyDoor>,
        mut query: Query<(Entity, &mut EngineAudio, &mut SpatialAudioSink)>,
    ) {
        let cockpit_closed: bool = canopy_door.0 <= 0.00001;

        for (entity, mut engine_audio, mut audio_sink) in &mut query {
            engine_audio.stopwatch.tick(time.delta());

            let throttle_factor = (throttle.0 / 100.0).clamp(0.0, 1.0);

            let min_volume = match cockpit_closed {
                true => 20.0,
                false => 40.0,
            };
            let max_volume = match cockpit_closed {
                true => 40.0,
                false => 60.0,
            };

            let target_volume =
                match engine_audio.stopwatch.elapsed_secs() < engine_audio.spool_duration {
                    true => {
                        let spool_factor = (engine_audio.stopwatch.elapsed_secs()
                            / engine_audio.spool_duration)
                            .clamp(0.0, 1.0);
                        min_volume + (max_volume - min_volume) * spool_factor
                    }
                    false => min_volume + (max_volume - min_volume) * throttle_factor,
                };

            if audio_sink.volume() != Volume::Linear(target_volume) {
                audio_sink.set_volume(Volume::Linear(target_volume));
            }

            if engine_audio.stopwatch.elapsed_secs() >= engine_audio.spool_duration
                && !engine_audio.running
            {
                commands.entity(entity).remove::<AudioPlayer>();
                commands.entity(entity).remove::<SpatialAudioSink>();

                commands
                    .entity(entity)
                    .insert(AudioPlayer::new(engine_audio.running_loop.clone()));
                engine_audio.loop_instance = Some(engine_audio.running_loop.clone());
                engine_audio.running = true;
            }
        }
    }
}

pub struct CF104Plugin;
impl Plugin for CF104Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin)
            .add_systems(Startup, initialize_player)
            .add_systems(
                Update,
                (EngineAudio::start_up_engine, EngineAudio::update_sound),
            );
    }
}

pub(crate) const CF104_BODY_ASSET_PATH: &'static str = "cf104\\meshes.gltf";
pub(crate) const CF104_CONSOLE_ASSET_PATH: &'static str = "cf104\\cf104_console_accessories.gltf";
pub(crate) const CF104_DOOR_ASSET_PATH: &'static str = "cf104\\cf104_door_accessories.gltf";

fn load_cf104<const PLAYER: bool>(
    transform: Transform,

    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,

    mask_materials: &Res<MaskMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    images: &mut ResMut<Assets<Image>>,

    tip_fuel_tanks: Option<f32>,
) -> Entity {
    let parent_id = commands
        .spawn((
            Player,
            Plane,
            GroundedBundle::cf_104(),
            PlaneBundle::cf_104(transform.translation.clone()),
            transform,
        ))
        .id();
    // commands.entity(body_id).insert();

    // load body
    let (body_id, internal_tank) = {
        let parent_mesh_handle: Handle<Mesh> =
            asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 11));
        let parent_material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3::splat(0.);

        transform.rotation = Quat::from_rotation_z(FRAC_PI_2) * Quat::from_rotation_y(FRAC_PI_2);

        transform.scale = Vec3::splat(1.);

        let id = commands
            .spawn((
                Mesh3d(parent_mesh_handle.clone()),
                NoFrustumCulling,
                MeshMaterial3d(parent_material_handle),
                transform,
                MassBundle::empty_cf_104(parent_id),
                ChildOf(parent_id),
            ))
            .id();

        // add internal fuel
        let internal_fuel_tank = commands
            .spawn((InternalFuelTankBundle::new(2_608.0, parent_id), ChildOf(id)))
            .id();

        let nuke = commands
            .spawn((MassBundle::nuke(parent_id), ChildOf(id)))
            .id();

        (id, internal_fuel_tank)
    };
    // engine_exhaust
    {
        let mut transform = Transform::default();
        transform.translation = Vec3 {
            x: 0.,
            y: -16.,
            z: 0.33,
        };

        commands.spawn((
            transform,
            EngineAudio::new(asset_server),
            PlaybackSettings::LOOP.with_spatial(true),
            ChildOf(body_id),
        ));
    }

    // load canopy shell
    let canopy_bundle = {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 9));
        let material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: -0.46190130710601807,
            z: 1.4541336297988892,
        };

        let canopy_window_bundle = {
            let mesh: Handle<Mesh> =
                asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 0));
            let material_handle = materials.add(StandardMaterial {
                base_color: Color::srgba(0.8, 0.8, 1.0, 0.25),
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            });

            let transform = Transform::default();

            (
                Mesh3d(mesh),
                NoFrustumCulling,
                MeshMaterial3d(material_handle),
                DragTarget(parent_id),
                transform,
            )
        };

        (
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            DragTarget(parent_id),
            NoFrustumCulling,
            transform,
            children![(canopy_window_bundle)],
            ChildOf(body_id),
        )
    };
    let canopy_id = commands.spawn(canopy_bundle).id(); //.set_parent_in_place(body_id);

    // load canopy door
    let canopy_door_bundle = {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 8));

        let mut transform = Transform::default();
        transform.rotation = Quat::from_xyzw(
            0.007375705521553755,
            -0.4225538969039917,
            0.015817251056432724,
            0.9061697721481323,
        );

        transform.translation = Vec3 {
            x: -0.5362579822540283,
            y: 0.,
            z: -0.4400066137313843,
        };

        let door_id = match PLAYER {
            true => commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(materials.add(StandardMaterial::default())),
                    NoFrustumCulling,
                    RotRange {
                        max: Quat::from_xyzw(
                            0.007375705521553755,
                            -0.4225538969039917,
                            0.015817251056432724,
                            0.9061697721481323,
                        ),
                        min: Quat::from_xyzw(0., 0., 0., 1.),
                    },
                    CanopyDoor::open(),
                    transform,
                    DragTarget(parent_id),
                    ChildOf(canopy_id),
                ))
                .id(),
            false => commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(materials.add(StandardMaterial::default())),
                    transform,
                    DragTarget(parent_id),
                    ChildOf(canopy_id),
                ))
                .id(),
        };

        match PLAYER {
            true => commands.spawn((
                Mesh3d(asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 7))),
                NoFrustumCulling,
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.8, 0.8, 1.0, 0.25),
                    alpha_mode: AlphaMode::Blend,
                    cull_mode: None,
                    ..default()
                })),
                DragTarget(parent_id),
                Transform::default(),
                ChildOf(door_id),
            )),
            false => commands.spawn((
                Mesh3d(asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 7))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.8, 0.8, 1.0, 0.25),
                    alpha_mode: AlphaMode::Blend,
                    cull_mode: None,
                    ..default()
                })),
                DragTarget(parent_id),
                Transform::default(),
                ChildOf(door_id),
            )),
        };

        if PLAYER {
            // handle
            {
                let mut transform = Transform::default();
                let mesh =
                    asset_server.load(&format!("{CF104_DOOR_ASSET_PATH}#Mesh{}/Primitive0", 1));
                transform.translation = Vec3 {
                    x: 0.9124946594238281,
                    y: -0.9854511022567749,
                    z: 0.5844357013702393,
                };
                let handle = commands
                    .spawn((
                        Mesh3d(mesh.clone()),
                        NoFrustumCulling,
                        MeshMaterial3d(materials.add(StandardMaterial::default())),
                        transform,
                        CanopyDoorHandle,
                        ChildOf(door_id),
                    ))
                    .id();

                mask_mesh::<false>(mask_materials, mesh.clone(), handle, commands);
            }
            // mirror
            let mut transform = Transform::default();
            transform.translation = Vec3 {
                x: 0.5201082229614258,
                y: 1.035123586654663,
                z: 0.6033310890197754,
            };
            commands.spawn((
                Mesh3d(asset_server.load(&format!("{CF104_DOOR_ASSET_PATH}#Mesh{}/Primitive0", 2))),
                NoFrustumCulling,
                MeshMaterial3d(materials.add(StandardMaterial::default())),
                transform,
                ChildOf(door_id),
            ));
        }
    };

    // load cockpit shell
    let shell_id = {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 6));
        let material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: 0.,
            z: 0.0027964115142822266,
        };

        let shell_id = commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material_handle),
                NoFrustumCulling,
                DragTarget(parent_id),
                transform,
                ChildOf(canopy_id),
            ))
            .id();

        mask_mesh::<true>(mask_materials, mesh.clone(), shell_id, commands);

        shell_id
    };

    // load console
    {
        let console_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            cull_mode: None,
            ..default()
        });
        let glass_material = materials.add(StandardMaterial {
            base_color: Color::linear_rgba(
                0.800000011920929,
                0.800000011920929,
                0.800000011920929,
                0.05,
            ),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let needle_material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 1.0),
            emissive: LinearRgba {
                red: 1.,
                green: 1.,
                blue: 1.,
                alpha: 1.,
            }, // intensity multiplier
            ..default()
        });

        let console_id = {
            let mesh: Handle<Mesh> =
                asset_server.load(&format!("{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0", 31));
            let material_handle = console_material.clone();

            // let material_handle = materials.add(StandardMaterial::default());

            let mut transform = Transform::default();

            transform.translation = Vec3 {
                x: 0.,
                y: 2.062485694885254,
                z: -1.4541336297988892,
            };

            let console_id = commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material_handle.clone()),
                    // Visibility::Visible,
                    NoFrustumCulling,
                    transform,
                    ChildOf(shell_id),
                ))
                .id();

            mask_mesh::<true>(mask_materials, mesh.clone(), console_id, commands);

            console_id
        };
        if PLAYER {
            // radio
            {
                let mut transform = Transform::default();
                transform.translation = Vec3 {
                    x: -0.35561704635620117,
                    y: -1.5650274753570557,
                    z: 0.910442054271698,
                };

                spawn_radio::<26, 24, 25, 23>(
                    transform,
                    commands,
                    asset_server,
                    materials,
                    mask_materials,
                    &console_material,
                    console_id,
                );
            }

            // clock
            {
                let mut transform = Transform::default();
                transform.translation = Vec3 {
                    x: 0.,
                    y: -0.0012316405773162842,
                    z: 0.,
                };

                spawn_clock::<7, 8, 10, 11, 6, 9>(
                    transform,
                    commands,
                    asset_server,
                    materials,
                    &console_material,
                    &glass_material,
                    &needle_material_handle,
                    console_id,
                );
            }

            // compass gyro ball
            {
                let mut transform = Transform::default();
                transform.scale = Vec3::splat(1.5779876708984375);
                transform.translation = Vec3 {
                    x: -0.03270721435546875,
                    y: -1.5688923597335815,
                    z: 1.020938754081726,
                };

                spawn_gyro_compass::<14, 13, 12>(
                    transform,
                    commands,
                    asset_server,
                    materials,
                    &console_material,
                    &glass_material,
                    console_id,
                );
            }

            // altimeter
            {
                let mut transform = Transform::default();
                transform.scale = Vec3::splat(0.8027474284172058);
                transform.translation = Vec3 {
                    x: -0.21370697021484375,
                    y: -1.5688923597335815,
                    z: 0.9261799454689026,
                };
                spawn_altimeter::<0, 1, 2, 3, 5>(
                    parent_id,
                    transform,
                    commands,
                    asset_server,
                    materials,
                    &needle_material_handle,
                    console_id,
                );
            }

            // speedometer
            {
                let mut transform = Transform::default();
                transform.scale = Vec3::splat(0.8027474284172058);
                transform.translation = Vec3 {
                    x: -0.21204900741577148,
                    y: -1.5688923597335815,
                    z: 1.0485676527023315,
                };

                spawn_speedometer::<30, 29, 28, 27>(
                    transform,
                    commands,
                    asset_server,
                    materials,
                    console_material,
                    glass_material,
                    needle_material_handle,
                    console_id,
                );
            }
        }

        let tmp = Vec3 {
            x: -0.2562694549560547,
            y: -1.5931899547576904,
            z: 1.1498485803604126,
        };
        commands.spawn((
            PointLight {
                intensity: 500.0,
                color: Color::Srgba(Srgba {
                    red: 1.,
                    green: 0.,
                    blue: 0.,
                    alpha: 1.,
                }),
                shadows_enabled: true,
                ..default()
            },
            {
                let mut transform = Transform::default();

                transform.translation = tmp.clone();

                transform
            },
            ChildOf(console_id),
        ));
    };

    // load seat
    {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 4));
        // let material_handle = materials.add(StandardMaterial::default());

        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.1, 0.1),
            cull_mode: None,
            ..default()
        });

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: -0.736794650554657,
            z: -1.4541336297988892,
        };

        mask_mesh::<true>(
            mask_materials,
            mesh.clone(),
            commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material_handle),
                    NoFrustumCulling,
                    transform,
                    ChildOf(shell_id),
                ))
                .id(),
            commands,
        )
    };

    let seat_back_bundle = {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 1));
        let material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: -2.4246456623077393,
            z: -1.4541336297988892,
        };

        (
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            NoFrustumCulling,
            transform,
            ChildOf(shell_id),
        )
    };

    commands.spawn(seat_back_bundle);

    if let Some(fuel_level) = tip_fuel_tanks {
        for i in 0..2 {
            let fuel_tank_bundle = {
                let mesh: Handle<Mesh> =
                    asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 10));
                let material_handle = materials.add(StandardMaterial::default());

                let mut transform = Transform::default();

                transform.translation = Vec3 {
                    x: (i * 2 - 1) as f32 * -7.403514862060547,
                    y: -6.743416786193848,
                    z: -0.3721950650215149,
                };

                (
                    Mesh3d(mesh),
                    MeshMaterial3d(material_handle),
                    NoFrustumCulling,
                    ExternalFuelTankBundle::new(454.0 * fuel_level, parent_id, internal_tank),
                    transform,
                    ChildOf(body_id),
                )
            };
            commands.spawn(fuel_tank_bundle);
        }
    }

    if PLAYER {
        {
            let camera_parent = commands
                .spawn((
                    {
                        let mut transform: Transform = Transform::default();

                        transform.translation = Vec3 {
                            x: 0.,
                            y: -0.65,
                            z: 0.,
                        };
                        transform.rotation = Quat::from_euler(EulerRot::XYZ, FRAC_PI_2, 0., 0.);

                        transform
                    },
                    CameraShake::default(),
                    ChildOf(shell_id),
                ))
                .id();

            set_up_player_camera(
                commands,
                Transform::default(),
                &asset_server,
                images,
                Some(camera_parent),
            );
        };

        {
            let mut transform: Transform = Transform::default();

            transform.translation = Vec3 {
                x: -0.3804253339767456,
                y: 0.2708113491535187,
                z: -0.9076950550079346,
            };

            transform.rotation = Quat::from_xyzw(0.5193636417388916, 0., 0., 0.8545534610748291);
            transform.scale = Vec3::splat(1.2716500759124756);

            spawn_throttle::<5>(
                transform,
                commands,
                asset_server,
                materials,
                mask_materials,
                shell_id,
            );
        }

        // player
        let joystick_bundle = {
            let mesh: Handle<Mesh> =
                asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 3));
            let material_handle = materials.add(StandardMaterial::default());

            let mut transform: Transform = Transform::default();

            transform.translation = Vec3 {
                x: 0.0,
                y: 0.5056626200675964,
                z: -1.7769677639007568,
            };
            transform.rotation = Quat::from_xyzw(0.1549355387687683, 0., 0., 0.9879246950149536);
            transform.scale = Vec3::splat(1.2716500759124756);

            mask_mesh::<false>(
                mask_materials,
                mesh.clone(),
                commands
                    .spawn((
                        Joystick::default(),
                        RotRange2D::new(
                            Quat::from_xyzw(0.1549355387687683, 0., 0., 0.9879246950149536),
                            Vec2::new(PI / 12., PI / 14.),
                        ),
                        Name::new("Joystick"),
                        Mesh3d(mesh),
                        MeshMaterial3d(material_handle),
                        transform,
                        ChildOf(shell_id),
                    ))
                    .id(),
                commands,
            )
        };

        // console dials
        {
            //CF104_CONSOLE_ASSET_PATH
        }
    }
    body_id
}

fn initialize_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mask_materials: Res<MaskMaterials>,
    mut images: ResMut<Assets<Image>>,
) {
    load_cf104::<true>(
        {
            let mut transform = Transform::default();
            transform.translation = Vec3 {
                x: 39.25777053833008,
                y: 2.,
                z: 169.4016571044922,
            };

            transform.rotation = Quat::from_euler(EulerRot::XYZ, 0., -PI / 2., 0.);

            transform
        },
        &mut commands,
        &asset_server,
        &mut materials,
        &mask_materials,
        meshes,
        &mut images,
        Some(1.),
    );
}
