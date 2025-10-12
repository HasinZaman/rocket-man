use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{audio::Volume, camera::visibility::NoFrustumCulling, prelude::*, time::Stopwatch};

use crate::{
    player::{
        Player, Selectable,
        camera::{MaskMaterials, mask_mesh, set_up_player_camera},
    },
    projectile::PlaneBundle,
};

// CF104
#[derive(Component)]
pub struct Plane;

#[derive(Component)]
pub struct Throttle(pub f32);

impl Default for Throttle {
    fn default() -> Self {
        Self(0.)
    }
}

#[derive(Component)]
pub struct RotRange {
    pub min: Quat,
    pub max: Quat,
}

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
                false => 30.0,
            };
            let max_volume = match cockpit_closed {
                true => 40.0,
                false => 50.0,
            };

            let target_volume =
                match engine_audio.stopwatch.elapsed_secs() < engine_audio.spool_duration {
                    true => {
                        let spool_factor = (engine_audio.stopwatch.elapsed_secs()
                            / engine_audio.spool_duration)
                            .clamp(0.0, 1.0);
                        min_volume + (max_volume - min_volume) * spool_factor
                    }
                    false => {
                        min_volume + (max_volume - min_volume) * throttle_factor
                    }
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
        app.add_systems(Startup, initialize_player).add_systems(
            Update,
            (EngineAudio::start_up_engine, EngineAudio::update_sound),
        );
    }
}

fn load_cf104<const PLAYER: bool>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,

    mask_materials: &Res<MaskMaterials>,
    images: &mut ResMut<Assets<Image>>,

    tip_fuel_tanks: Option<f32>,
) -> Entity {
    const CF104_BODY_ASSET_PATH: &'static str = "cf104\\meshes.gltf";
    const CF104_DOOR_ASSET_PATH: &'static str = "cf104\\cf104_door_accessories.gltf";

    let body_id = commands
        .spawn((Player, Plane, PlaneBundle::cf_104(), Transform::default()))
        .id();

    // load body
    let body_id = {
        let parent_mesh_handle: Handle<Mesh> =
            asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 11));
        let parent_material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3::splat(0.);

        transform.rotation = Quat::from_rotation_z(FRAC_PI_2) * Quat::from_rotation_y(FRAC_PI_2);

        transform.scale = Vec3::splat(1.);

        commands
            .spawn((
                Mesh3d(parent_mesh_handle),
                NoFrustumCulling,
                MeshMaterial3d(parent_material_handle),
                transform,
                ChildOf(body_id),
            ))
            .id()
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
                transform,
            )
        };

        (
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
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
                    ChildOf(canopy_id),
                ))
                .id(),
            false => commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(materials.add(StandardMaterial::default())),
                    transform,
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
                transform,
                ChildOf(canopy_id),
            ))
            .id();

        mask_mesh::<true>(mask_materials, mesh.clone(), shell_id, commands);

        shell_id
    };

    // load console
    {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 2));
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            cull_mode: None,
            ..default()
        });

        // let material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: 2.062485694885254,
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
                    ChildOf(shell_id),
                ))
                .id();

            set_up_player_camera(commands, Transform::default(), images, Some(camera_parent));
        };

        {
            let mesh: Handle<Mesh> =
                asset_server.load(&format!("{CF104_BODY_ASSET_PATH}#Mesh{}/Primitive0", 5));
            let material_handle = materials.add(StandardMaterial::default());

            let mut transform: Transform = Transform::default();

            transform.translation = Vec3 {
                x: -0.3804253339767456,
                y: 0.2708113491535187,
                z: -0.9076950550079346,
            };

            transform.rotation = Quat::from_xyzw(0.5193636417388916, 0., 0., 0.8545534610748291);
            transform.scale = Vec3::splat(1.2716500759124756);

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
                        ChildOf(shell_id),
                    ))
                    .id(),
                commands,
            )
        };

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
    }
    body_id
}

fn initialize_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mask_materials: Res<MaskMaterials>,
    mut images: ResMut<Assets<Image>>,
) {
    load_cf104::<true>(
        &mut commands,
        &asset_server,
        &mut materials,
        &mask_materials,
        &mut images,
        Some(100.),
    );
}
