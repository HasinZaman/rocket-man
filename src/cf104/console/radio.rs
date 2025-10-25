use std::f32::consts::TAU;
use std::time::Duration;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::audio::Volume;
use bevy::{camera::visibility::NoFrustumCulling, prelude::*};
use lewton::VorbisError;
use lewton::inside_ogg::OggStreamReader;
use rand::seq::SliceRandom;
use rand::{SeedableRng, thread_rng};
use rand_chacha::ChaCha8Rng;
use ron::de::SpannedError;
use serde::Deserialize;
use thiserror::Error;

use crate::player::camera::{HeadSetSpeaker, MaskMaterials, SpeakerSink, mask_mesh};

use crate::cf104::CF104_CONSOLE_ASSET_PATH;

#[derive(Component, Debug)]
pub struct RadioFxSelector(pub u8);

#[derive(Message, Debug)]
pub struct UpdateRadioFx(pub u8);

#[derive(Component, Debug)]
pub struct RadioVolume(pub f32);

#[derive(Message, Debug)]
pub struct UpdateVolume(pub f32);

pub fn spawn_radio<
    const FRAME_MESH: u32,
    const CHANNEL_MESH: u32,
    const VOLUME_MESH: u32,
    const SELECTOR_MESH: u32,
>(
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
    let radio_dial = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            RadioFxSelector(0),
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
            ChildOf(radio_id),
        ))
        .id();
    mask_mesh::<false>(mask_materials, mesh.clone(), radio_dial, commands);

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
    let volume_id = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            RadioVolume(5.),
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
            ChildOf(radio_id),
        ))
        .id();
    mask_mesh::<false>(mask_materials, mesh.clone(), volume_id, commands);
    let mesh: Handle<Mesh> = asset_server.load(&format!(
        "{CF104_CONSOLE_ASSET_PATH}#Mesh{}/Primitive0",
        SELECTOR_MESH
    ));
    let material_handle = console_material.clone();
    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: 0.05943775177001953,
        y: 0.001281827688217163,
        z: -0.017145991325378418,
    };
    let selector = commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            // Visibility::Visible,
            NoFrustumCulling,
            transform,
            ChildOf(radio_id),
        ))
        .id();
    mask_mesh::<true>(mask_materials, mesh.clone(), selector, commands);
}

pub fn update_fx_selector(mut query: Query<(&mut Transform, &RadioFxSelector)>) {
    for (mut transform, selector) in &mut query {
        transform.rotation = Quat::from_rotation_y(TAU / 28. * selector.0 as f32);
    }
}

pub fn update_volume_knob(
    mut volume_message: MessageReader<UpdateVolume>,
    mut transform: Single<&mut Transform, With<RadioVolume>>,
    head_set_emitters: Query<&mut SpatialAudioSink, With<SpeakerSink>>,
) {
    let Some(volume) = volume_message.read().last() else {
        return;
    };

    println!("{volume:?}");

    transform.rotation = Quat::from_rotation_y(TAU * volume.0 / 100.);

    for mut speaker in head_set_emitters {
        speaker.set_volume(Volume::Linear(volume.0 / 100. * 3.));
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Playable {
    audio: String,
    duration: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub enum LoadOrder {
    Default,
    Random(u64),
    TimeRandom,
}
#[derive(Debug, Clone, Deserialize, TypePath, Asset)]
pub struct RadioChannelConfig {
    source: Option<(Vec3, f32)>,
    playables: Vec<Playable>,
    load_order: LoadOrder,
}

#[derive(Debug, Error)]
pub enum RadioChannelLoaderError {
    #[error("IO error while reading file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse RON config: {0}")]
    Ron(#[from] SpannedError),

    #[error("Vorbis decoding error: {0}")]
    VorbisError(#[from] VorbisError),
}

#[derive(Default)]
pub struct RadioChannelLoader;
impl AssetLoader for RadioChannelLoader {
    type Asset = RadioChannelConfig;
    type Settings = ();
    type Error = RadioChannelLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let mut config: RadioChannelConfig = ron::de::from_bytes(&bytes)?;

        let asset_root = std::path::Path::new("assets");
        let dir_path = load_context.path().parent().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to get directory")
        })?;

        // Prepend ./assets so the OS can find the files
        let full_dir_path = asset_root.join(dir_path);

        for entry in std::fs::read_dir(&full_dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|s| s == "ogg").unwrap_or(false) {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();

                if config.playables.iter().any(|p| p.audio == filename) {
                    continue;
                }

                match std::fs::File::open(&path) {
                    Ok(file) => {
                        match OggStreamReader::new(file) {
                            Ok(mut ogg_reader) => {
                                let sample_rate = ogg_reader.ident_hdr.audio_sample_rate;
                                let channels = ogg_reader.ident_hdr.audio_channels;

                                let mut total_samples = 0usize;
                                // let mut packet_count = 0usize;

                                while let Some(pck) = ogg_reader.read_dec_packet_itl()? {
                                    total_samples += pck.len() / channels as usize;
                                    // packet_count += 1;
                                }

                                let duration: f32 = total_samples as f32 / sample_rate as f32;

                                let full_path = path.canonicalize()?;
                                let asset_path = full_path
                                    .to_string_lossy()
                                    .replace("\\", "/") // normalize Windows paths
                                    .split("assets/")
                                    .nth(1) // take everything after "assets/"
                                    .ok_or_else(|| {
                                        std::io::Error::new(
                                            std::io::ErrorKind::Other,
                                            "Failed to strip assets prefix",
                                        )
                                    })?
                                    .to_string();

                                config.playables.push(Playable {
                                    audio: asset_path,
                                    duration,
                                });
                            }
                            Err(e) => {
                                println!("❌ Failed to decode {}: {:?}", filename, e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to open {}: {:?}", filename, e);
                    }
                }
            }
        }

        config.playables.sort_by_key(|p| p.audio.clone());

        match config.load_order {
            LoadOrder::Default => {}
            LoadOrder::Random(seed) => {
                let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(seed);
                config.playables.shuffle(&mut rng);
            }
            LoadOrder::TimeRandom => {
                let mut rng = rand::rng();
                config.playables.shuffle(&mut rng);
            }
        }

        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["radio_config"]
    }
}

#[derive(Resource, Debug, Default)]
pub struct RadioChannels([Option<Handle<RadioChannelConfig>>; 28]);

pub fn load_channels(mut channels: ResMut<RadioChannels>, asset_server: Res<AssetServer>) {
    channels.0[6] = Some(asset_server.load("audio\\channels\\channel_6\\.radio_config"));
    channels.0[10] = Some(asset_server.load("audio\\channels\\channel_10\\.radio_config"));
}

#[derive(Resource, Debug, Default)]
pub struct Radio {
    surpassed_time: Duration,
    playable_duration: Timer,
    handle: Option<RadioChannelConfig>,
    idx: usize,
}

pub fn set_up_radio_audio(
    mut radio_fx_writer: MessageWriter<UpdateRadioFx>,
    start_fx: Single<&RadioFxSelector>,
) {
    radio_fx_writer.write(UpdateRadioFx(start_fx.0));
}

#[derive(Message)]
pub struct DeferredFxChange(u8);

pub fn update_radio(
    time: Res<Time>,
    mut radio_fx_reader: MessageReader<UpdateRadioFx>,
    mut radio_fx_writer: MessageWriter<DeferredFxChange>,

    mut radio_volume_write: MessageWriter<UpdateVolume>,

    radio_channels: Res<RadioChannels>,
    radio_channel_configs: Res<Assets<RadioChannelConfig>>,
    asset_server: Res<AssetServer>,
    mut radio: ResMut<Radio>,

    mut commands: Commands,
    head_sets_speakers_query: Query<(Entity, Option<&Children>), With<HeadSetSpeaker>>,
    volume: Single<&RadioVolume>, // sinks: Query<Entity, (With<SpatialAudioSink>, With<SpeakerSink>)>,
) {
    radio.surpassed_time += time.delta();

    'change_channel: {
        if let Some(UpdateRadioFx(idx)) = radio_fx_reader.read().last() {
            // println!("New channel");
            let Some(radio_channel_config) = radio_channels.0[*idx as usize].clone() else {
                println!("remove sinks and playing static");
                // remove all sinks
                for (_, children) in head_sets_speakers_query.iter() {
                    let Some(children) = children else {
                        continue;
                    };
                    for child in children {
                        println!("de-spawning: {child:?}");
                        commands.entity(*child).despawn();
                    }
                }
                // replace sinks with static
                for (entity, _) in head_sets_speakers_query.iter() {
                    println!("spawning in: {entity:?}");
                    commands.spawn((
                        AudioPlayer::new(asset_server.load("audio/radio_static.ogg")),
                        PlaybackSettings::LOOP
                            .with_spatial(true)
                            .with_volume(Volume::Linear(volume.0 / 100. * 3.)),
                        SpeakerSink,
                        Transform::IDENTITY,
                        ChildOf(entity),
                    ));
                }
                radio.handle = None;

                radio_volume_write.write(UpdateVolume(volume.0));
                return;
            };

            let Some(new_channel_config) = radio_channel_configs.get(radio_channel_config.id())
            else {
                // println!("try again");
                // try again once the config is loaded
                radio_fx_writer.write(DeferredFxChange(*idx));
                break 'change_channel;
            };

            println!("{new_channel_config:?}");
            radio.handle = Some(new_channel_config.clone());

            let (start_idx, skip_duration) = {
                let start_sec: f32 = radio.surpassed_time.as_secs_f32();
                let loop_duration: f32 = new_channel_config
                    .playables
                    .iter()
                    .map(|p| p.duration)
                    .sum();

                let loops: f32 = (start_sec / loop_duration).floor();

                let start_sec: f32 = start_sec - loops * loop_duration;

                let mut start_idx: usize = new_channel_config.playables.len() - 1;
                let mut skip_duration: f32 = 0.;

                let mut sum = 0.;

                for (i1, Playable { duration, .. }) in
                    new_channel_config.playables.iter().enumerate()
                {
                    if sum + duration > start_sec {
                        start_idx = i1;
                        skip_duration = start_sec - sum;
                        break;
                    }
                    sum += duration;
                }

                (start_idx, skip_duration)
            };
            // println!("total_time: {:?}\tstart idx: {start_idx:?}\tskip: {skip_duration:?}\ttotal:{:?}",
            // radio.surpassed_time, new_channel_config.playables[start_idx].duration);

            radio.idx = start_idx;

            // remove all sinks
            for (_, children) in head_sets_speakers_query.iter() {
                let Some(children) = children else {
                    continue;
                };
                for child in children {
                    commands.entity(*child).despawn();
                }
            }
            // replace sinks with new audio
            for (entity, _) in head_sets_speakers_query.iter() {
                commands.spawn((
                    AudioPlayer::new(
                        asset_server.load(new_channel_config.playables[start_idx].audio.clone()),
                    ),
                    PlaybackSettings::LOOP
                        .with_spatial(true)
                        .with_volume(Volume::Linear(volume.0 / 100. * 3.))
                        .with_start_position(Duration::from_secs_f32(
                            skip_duration - time.delta_secs(),
                        )),
                    SpeakerSink,
                    Transform::IDENTITY,
                    ChildOf(entity),
                ));
            }

            radio.playable_duration = Timer::new(
                Duration::from_secs_f32(new_channel_config.playables[start_idx].duration),
                TimerMode::Once,
            );
            radio
                .playable_duration
                .tick(Duration::from_secs_f32(skip_duration));

            // println!("{:?} | {:?} | {:?}", radio.playable_duration.elapsed(), radio.playable_duration.fraction_remaining(), radio.playable_duration.duration());

            radio_volume_write.write(UpdateVolume(volume.0));
            return;
        }
    }

    // update audio
    if let Some(channel_config) = radio.handle.clone() {
        radio.playable_duration.tick(time.delta());

        if !radio.playable_duration.is_finished() {
            return;
        };
        println!("New audio");
        radio.idx = (radio.idx + 1) % channel_config.playables.len();
        let idx: usize = radio.idx;

        for (_, children) in head_sets_speakers_query.iter() {
            let Some(children) = children else {
                continue;
            };
            for child in children {
                commands.entity(*child).despawn();
            }
        }
        for (entity, _) in head_sets_speakers_query.iter() {
            commands.spawn((
                AudioPlayer::new(asset_server.load(channel_config.playables[idx].audio.clone())),
                PlaybackSettings::LOOP
                    .with_spatial(true)
                    .with_volume(Volume::Linear(volume.0 / 100. * 3.)),
                SpeakerSink,
                Transform::IDENTITY,
                ChildOf(entity),
            ));
        }

        radio.playable_duration = Timer::new(
            Duration::from_secs_f32(channel_config.playables[idx].duration),
            TimerMode::Once,
        );
    };
}

pub fn deferred_fx_change(
    mut in_message: MessageReader<DeferredFxChange>,
    mut out_message: MessageWriter<UpdateRadioFx>,
) {
    for DeferredFxChange(idx) in in_message.read() {
        out_message.write(UpdateRadioFx(*idx));
    }
}
