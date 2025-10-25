use std::process::Command;

use bevy::{
    app::{Plugin, Startup, Update},
    asset::{
        io::Reader, Asset, AssetApp, AssetLoader, AssetServer, Assets, Handle, LoadContext, RenderAssetUsages
    },
    camera::visibility::Visibility,
    color::Color,
    ecs::{
        component::Component, entity::Entity, hierarchy::Children, message::MessageWriter, query::{With, Without}, resource::Resource, system::{Commands, Query, Res, ResMut}
    },
    math::Vec3,
    mesh::{Indices, Mesh, Mesh3d, PrimitiveTopology},
    pbr::{MeshMaterial3d, StandardMaterial},
    platform::collections::HashSet,
    reflect::TypePath,
    transform::components::{GlobalTransform, Transform},
};
use ron::de::SpannedError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{cf104::Plane, player::Player, world::{
    props::{Prop, SpawnPropsMessage}, util::{get_lat_f64, get_lon_f64}, GlobalPosition, MovingOrigin
}};

const GRID_SIZE: f64 = 5_000.;

const MAX_VISION: f64 = 40_000.;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LandCover {
    ContinuousUrbanFabric = 111,
    DiscontinuousUrbanFabric,
    IndustrialOrCommercialUnits = 121,
    RoadAndRailNetworksAndAssociatedLand,
    PortAreas,
    Airports,
    MineralExtractionSites = 131,
    DumpSites,
    ConstructionSites,
    GreenUrbanAreas = 141,
    SportAndLeisureFacilities,
    NonIrrigatedArableLand = 211,
    PermanentlyIrrigatedLand,
    RiceFields,
    Vineyards = 221,
    FruitTreesAndBerryPlantations,
    OliveGroves,
    Pastures = 231,
    AnnualCropsAssociatedWithPermanentCrops = 241,
    ComplexCultivationPatterns,
    LandPrincipallyOccupiedByAgricultureWithSignificantAreasOfNaturalVegetation,
    AgroForestryAreas,
    BroadLeavedForest = 311,
    ConiferousForest,
    MixedForest,
    NaturalGrasslands = 321,
    MoorsAndHeathland,
    SclerophyllousVegetation,
    TransitionalWoodlandShrub,
    BeachesDunesSands = 331,
    BareRocks,
    SparselyVegetatedAreas,
    BurntAreas,
    GlaciersAndPerpetualSnow,
    InlandMarshes = 411,
    PeatBogs,
    SaltMarshes = 421,
    Salines,
    IntertidalFlats,
    WaterCourses = 511,
    WaterBodies,
    CoastalLagoons = 521,
    Estuaries,
    SeaAndOcean,
    Nodata = 999,
    UnclassifiedLandSurface = 990,
    UnclassifiedWaterBodies = 995,
}

#[derive(Default, Serialize, Deserialize, TypePath, Asset)]
pub struct GroundData {
    pub lats: Vec<f64>,
    pub lons: Vec<f64>,
    pub height: Vec<f32>,
    pub land_use: Vec<LandCover>,
}

#[derive(Debug, Error)]
pub enum GroundDataLoaderError {
    #[error("IO error while reading file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse RON config: {0}")]
    Ron(#[from] SpannedError),
}

#[derive(Default)]
pub struct GroundDataLoader;
impl AssetLoader for GroundDataLoader {
    type Asset = GroundData;
    type Settings = ();
    type Error = GroundDataLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let ground_data: GroundData = ron::de::from_bytes(&bytes)?;

        Ok(ground_data)
    }
    fn extensions(&self) -> &[&str] {
        &["ground"]
    }
}

#[derive(Resource, Debug, Default)]
pub struct WorldDataInitialized(Option<Handle<GroundData>>, bool);

#[derive(Resource, Debug, Default)]
pub struct GroundMeta {
    pub lats: Vec<f64>,
    pub lons: Vec<f64>,
}

#[derive(Resource, Debug, Default)]
pub struct HeightData(Vec<f32>);

#[derive(Resource, Debug, Default)]
pub struct LandCoverData(Vec<LandCover>);

#[derive(Resource, Debug, Default)]
pub struct FreeGroundChunks(Vec<Entity>);

#[derive(Component, Debug, Default)]
pub struct GroundChunk(i32, i32);

pub struct GroundPlugin;
impl Plugin for GroundPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_asset::<GroundData>()
            .init_asset_loader::<GroundDataLoader>()
            .init_resource::<WorldDataInitialized>()
            .init_resource::<GroundMeta>()
            .init_resource::<HeightData>()
            .init_resource::<LandCoverData>()
            .init_resource::<FreeGroundChunks>()
            .add_systems(Startup, load_ground_data)
            .add_systems(Update, (
                initialize_ground_data,
                update_ground,
                // update_ground_visibility
            ));
        // .add_systems(Update, update_ground);
    }
}

pub fn load_ground_data(
    asset_server: Res<AssetServer>,
    mut world_data_initialize: ResMut<WorldDataInitialized>,
) {
    let handle: Handle<GroundData> = asset_server.load("world\\europe.ground");

    world_data_initialize.0 = Some(handle);
}

pub fn initialize_ground_data(
    mut world_data_initialize: ResMut<WorldDataInitialized>,

    ground_assets: Res<Assets<GroundData>>,

    mut meta: ResMut<GroundMeta>,
    mut height_data: ResMut<HeightData>,
    mut land_cover: ResMut<LandCoverData>,
) {
    if world_data_initialize.1 {
        return;
    }

    if world_data_initialize.0.is_none() {
        return;
    }

    let handle: Handle<GroundData> = world_data_initialize.0.clone().unwrap();

    let Some(data) = ground_assets.get(handle.id()) else {
        return;
    };

    meta.lats = data.lats.clone();
    meta.lons = data.lons.clone();

    height_data.0 = data.height.iter().cloned().map(|x| x.max(0.0)).collect();
    land_cover.0 = data.land_use.clone();

    world_data_initialize.1 = true;
    world_data_initialize.0 = None;
}

fn update_ground_visibility(
    plane_query: Query<&GlobalTransform, (With<Plane>, With<Player>)>,
    mut ground_query: Query<(&GlobalTransform, &mut Visibility), With<GroundChunk>>
) {
    let Ok(plane_transform) = plane_query.single() else { return; };

    let forward: Vec3 = *plane_transform.right();

    let plane_pos: Vec3 = plane_transform.translation();

    const FOV_COS_THRESHOLD: f32 = 0.0;

    for (chunk_transform, mut visibility) in ground_query.iter_mut() {
        let chunk_pos: Vec3 = chunk_transform.translation();

        let dir_to_chunk: Vec3 = (chunk_pos - plane_pos).normalize_or_zero();

        let dot = forward.dot(dir_to_chunk);

        if dot > FOV_COS_THRESHOLD {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn update_ground(
    moving_origin: Res<MovingOrigin>,
    centered_entity: Query<&GlobalPosition, Without<GroundChunk>>,

    world_data_initialize: Res<WorldDataInitialized>,
    ground_meta: Res<GroundMeta>,
    height_data: Res<HeightData>,
    land_cover: Res<LandCoverData>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut free_chunks: ResMut<FreeGroundChunks>,

    mut ground_chunks_query: Query<(Entity, &mut GlobalPosition, &mut GroundChunk, &mut Mesh3d, &Children)>,
    prop_query: Query<Entity, With<Prop>>,

    mut spawn_prop_writer: MessageWriter<SpawnPropsMessage>,
) {
    // check if the data is loaded
    if !world_data_initialize.1 {
        return;
    }

    let mut center: GlobalPosition = match moving_origin.0 {
        Some(entity) => {
            let Ok(center) = centered_entity.get(entity) else {
                panic!("Invalid state");
            };

            center.clone()
        }
        None => GlobalPosition::zero(),
    };
    center.y = 0.;

    let radius: f64 = (MAX_VISION * MAX_VISION).sqrt();
    let min_x: i32 = (round_to_nearest_grid_coord(center.x - radius, GRID_SIZE) / GRID_SIZE) as i32;
    let max_x: i32 = (round_to_nearest_grid_coord(center.x + radius, GRID_SIZE) / GRID_SIZE) as i32;
    let min_y: i32 = (round_to_nearest_grid_coord(center.z - radius, GRID_SIZE) / GRID_SIZE) as i32;
    let max_y: i32 = (round_to_nearest_grid_coord(center.z + radius, GRID_SIZE) / GRID_SIZE) as i32;

    // remove values
    for i in (0..free_chunks.0.len()).rev() {
        let (.., GroundChunk(x, y), _, _) = ground_chunks_query.get(free_chunks.0[i]).unwrap();
        let x_cond: bool = min_x <= *x && *x <= max_x;
        let y_cond: bool = min_y <= *y && *y <= max_y;
        if x_cond && y_cond {
            free_chunks.0.remove(i);
        }
    }

    // add values to free_chunks
    let mut active_chunks: HashSet<(i32, i32)> = HashSet::new();
    for (entity, _, ground_chunk, ..) in ground_chunks_query.iter_mut() {
        active_chunks.insert((ground_chunk.0, ground_chunk.1));

        let x_cond: bool = min_x < ground_chunk.0 && ground_chunk.0 < max_x;
        let y_cond: bool = min_y < ground_chunk.1 && ground_chunk.1 < max_y;
        if (!x_cond || !y_cond) && !free_chunks.0.contains(&entity) {
            free_chunks.0.push(entity);
        }
    }



    // println!("active chunks{:?}", active_chunks.len());

    {
        // println!("{:?}->{:?}", (min_x, min_y), (max_x, max_y));
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if active_chunks.contains(&(x, y)) {
                    continue;
                }

                let height_00: f32 = find(
                    get_lat_f64(x as f64 * GRID_SIZE) as f64,
                    get_lon_f64(y as f64 * GRID_SIZE) as f64,
                    &ground_meta,
                    &height_data.0,
                )
                .unwrap();
                let height_10: f32 = find(
                    get_lat_f64((x + 1) as f64 * GRID_SIZE) as f64,
                    get_lon_f64(y as f64 * GRID_SIZE) as f64,
                    &ground_meta,
                    &height_data.0,
                )
                .unwrap();
                let height_01: f32 = find(
                    get_lat_f64(x as f64 * GRID_SIZE) as f64,
                    get_lon_f64((y + 1) as f64 * GRID_SIZE) as f64,
                    &ground_meta,
                    &height_data.0,
                )
                .unwrap();
                let height_11: f32 = find(
                    get_lat_f64((x + 1) as f64 * GRID_SIZE) as f64,
                    get_lon_f64((y + 1) as f64 * GRID_SIZE) as f64,
                    &ground_meta,
                    &height_data.0,
                )
                .unwrap();

                let land_use: [LandCover; 4] = [
                    find_nearest_land_cover(
                        get_lat_f64(x as f64 * GRID_SIZE) as f64,
                        get_lon_f64(y as f64 * GRID_SIZE) as f64,
                        &ground_meta,
                        &land_cover.0,
                    )
                    .unwrap(),
                    find_nearest_land_cover(
                        get_lat_f64((x + 1) as f64 * GRID_SIZE) as f64,
                        get_lon_f64(y as f64 * GRID_SIZE) as f64,
                        &ground_meta,
                        &land_cover.0,
                    )
                    .unwrap(),
                    find_nearest_land_cover(
                        get_lat_f64(x as f64 * GRID_SIZE) as f64,
                        get_lon_f64((y + 1) as f64 * GRID_SIZE) as f64,
                        &ground_meta,
                        &land_cover.0,
                    )
                    .unwrap(),
                    find_nearest_land_cover(
                        get_lat_f64((x + 1) as f64 * GRID_SIZE) as f64,
                        get_lon_f64((y + 1) as f64 * GRID_SIZE) as f64,
                        &ground_meta,
                        &land_cover.0,
                    )
                    .unwrap(),
                ];

                // if free_chunks.0.len() > 0 {
                //     println!("{free_chunks:?}");
                // }
                let min_height: f32 = height_00.min(height_01).min(height_10).min(height_11);
                match free_chunks.0.pop() {
                    Some(chunk_entity) => {
                        // println!("Old chunk");
                        let (_, mut position, mut ground_chunk, mut mesh, children) =
                            ground_chunks_query.get_mut(chunk_entity).unwrap();

                        ground_chunk.0 = x;
                        ground_chunk.1 = y;

                        *position = GlobalPosition {
                            x: x as f64 * GRID_SIZE,
                            y: min_height as f64,
                            z: y as f64 * GRID_SIZE,
                        };
                        // for child in children {
                        //     commands.get_entity(*child).unwrap().despawn();
                        // }

                        // commands.entity(chunk_entity)
                        //     .insert(
                        //         Mesh3d(meshes.add(create_height_quad_mesh(
                        //         height_00,
                        //         height_01,
                        //         height_10,
                        //         height_11,
                        //         5000.,
                        //         5000.,
                        //     )))
                        // );
                        mesh.0 = meshes.add(create_height_quad_mesh(
                            height_00, height_01, height_10, height_11, 5000., 5000.,
                        ));

                        // spawn_prop_writer.write(SpawnPropsMessage {
                        //     entity: chunk_entity,
                        //     u: Vec3::new(GRID_SIZE as f32, height_10, 0.).normalize(),
                        //     v: Vec3::new(0., height_01, GRID_SIZE as f32).normalize(),
                        //     chunk_id: (x, y),
                        //     width: GRID_SIZE as f32,
                        //     length: GRID_SIZE as f32,
                        //     heights: [height_00, height_01, height_10, height_11],
                        //     land_use,
                        // });
                    }
                    None => {
                        // println!("New Chunk");
                        let id: Entity = commands
                            .spawn((
                                Visibility::Visible,
                                Transform::from_translation(Vec3::new(
                                    x as f32 * GRID_SIZE as f32,
                                    min_height,
                                    y as f32 * GRID_SIZE as f32,
                                )),
                                Mesh3d(meshes.add(create_height_quad_mesh(
                                    height_00, height_01, height_10, height_11, 5000., 5000.,
                                ))),
                                MeshMaterial3d(materials.add(Color::srgb(0., 0.75, 0.))),
                                GroundChunk(x, y),
                                GlobalPosition {
                                    x: x as f64 * GRID_SIZE,
                                    y: min_height as f64,
                                    z: y as f64 * GRID_SIZE,
                                },
                            ))
                            .id();
                        spawn_prop_writer.write(SpawnPropsMessage {
                            entity: id,
                            u: Vec3::new(GRID_SIZE as f32, height_10, 0.).normalize(),
                            v: Vec3::new(0., height_01, GRID_SIZE as f32).normalize(),
                            chunk_id: (x, y),
                            width: GRID_SIZE as f32,
                            length: GRID_SIZE as f32,
                            heights: [height_00, height_01, height_10, height_11],
                            land_use,
                        });

                        // send Event to spawn buildings and trees?
                    }
                }
            }
        }
    }

    // new values are inserted into cache
    // update MeshData for new values
    // new chunks should trigger an event to spawn buildings
    // trees
}

#[inline]
fn round_to_nearest_grid_coord(pos: f64, grid_size: f64) -> f64 {
    (pos / grid_size).round() * grid_size
}

fn create_height_quad_mesh(
    height_00: f32,
    height_01: f32,
    height_10: f32,
    height_11: f32,
    length: f32,
    width: f32,
) -> Mesh {
    let length = length;
    let width = width;

    let min_height = height_00.min(height_01).min(height_10).min(height_11);

    let positions: Vec<[f32; 3]> = vec![
        [0.0, height_00 - min_height, 0.0],
        [width, height_10 - min_height, 0.0],
        [0.0, height_01 - min_height, length],
        [width, height_11 - min_height, length],
    ];

    let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];

    // Counter-clockwise winding when viewed from above
    let indices: Vec<u32> = vec![0, 2, 1, 1, 2, 3];

    // Compute approximate face normal
    let v0: Vec3 = Vec3::from(positions[0]);
    let v1: Vec3 = Vec3::from(positions[1]);
    let v2: Vec3 = Vec3::from(positions[2]);
    let normal: Vec3 = (v1 - v0).cross(v2 - v0).normalize();

    let normals: Vec<[f32; 3]> = vec![
        [0., 1., 0.].into(),
        [0., 1., 0.].into(),
        [0., 1., 0.].into(),
        [0., 1., 0.].into(),
    ];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn find(lat: f64, lon: f64, meta: &Res<GroundMeta>, data: &[f32]) -> Result<f32, ()> {
    let lats = &meta.lats;
    let lons = &meta.lons;

    let n_lat = lats.len();
    let n_lon = lons.len();

    if n_lat < 2 || n_lon < 2 {
        return Err(()); // not enough data to interpolate
    }

    if data.len() != n_lat * n_lon {
        return Err(()); // mismatched data grid
    }

    // --- Clamp lat/lon to grid range ---
    let lat: f64 = lat.clamp(lats[0], lats[n_lat - 1]);
    let lon: f64 = lon.clamp(lons[0], lons[n_lon - 1]);

    // --- Find indices for bounding box safely ---
    let lat_idx: usize = match lats.binary_search_by(|x| x.partial_cmp(&lat).unwrap()) {
        Ok(i) => i.min(n_lat - 2),
        Err(i) => i.saturating_sub(1).min(n_lat - 2),
    };

    let lon_idx: usize = match lons.binary_search_by(|x| x.partial_cmp(&lon).unwrap()) {
        Ok(i) => i.min(n_lon - 2),
        Err(i) => i.saturating_sub(1).min(n_lon - 2),
    };

    // --- Get surrounding lat/lon points ---
    let lat0: f64 = lats[lat_idx];
    let lat1: f64 = lats[lat_idx + 1];
    let lon0: f64 = lons[lon_idx];
    let lon1: f64 = lons[lon_idx + 1];

    // --- Prevent divide-by-zero ---
    let denom_lat = (lat1 - lat0).abs().max(f64::EPSILON);
    let denom_lon = (lon1 - lon0).abs().max(f64::EPSILON);

    // --- Compute normalized weights ---
    let t: f64 = (lat - lat0) / denom_lat;
    let u: f64 = (lon - lon0) / denom_lon;

    // --- Retrieve four corner values ---
    let idx = |i, j| i * n_lon + j;
    let f00: f32 = data[idx(lat_idx, lon_idx)];
    let f10: f32 = data[idx(lat_idx + 1, lon_idx)];
    let f01: f32 = data[idx(lat_idx, lon_idx + 1)];
    let f11: f32 = data[idx(lat_idx + 1, lon_idx + 1)];

    // --- Bilinear interpolation ---
    let f0: f32 = f00 * (1.0 - t as f32) + f10 * t as f32;
    let f1: f32 = f01 * (1.0 - t as f32) + f11 * t as f32;
    let value: f32 = f0 * (1.0 - u as f32) + f1 * u as f32;

    Ok(value)
}

pub fn find_nearest_land_cover(
    lat: f64,
    lon: f64,
    meta: &GroundMeta,
    data: &[LandCover],
) -> Result<LandCover, ()> {
    let lats = &meta.lats;
    let lons = &meta.lons;
    let n_lat = lats.len();
    let n_lon = lons.len();

    if n_lat == 0 || n_lon == 0 || data.len() != n_lat * n_lon {
        return Err(()); // Invalid grid
    }

    // Clamp within bounds
    let lat = lat.clamp(lats[0], lats[n_lat - 1]);
    let lon = lon.clamp(lons[0], lons[n_lon - 1]);

    // --- Find closest latitude index ---
    let lat_idx = match lats.binary_search_by(|x| x.partial_cmp(&lat).unwrap()) {
        Ok(i) => i,
        Err(i) => {
            if i == 0 {
                0
            } else if i >= n_lat {
                n_lat - 1
            } else {
                let d0 = (lats[i - 1] - lat).abs();
                let d1 = (lats[i] - lat).abs();
                if d0 < d1 { i - 1 } else { i }
            }
        }
    };

    // --- Find closest longitude index ---
    let lon_idx = match lons.binary_search_by(|x| x.partial_cmp(&lon).unwrap()) {
        Ok(i) => i,
        Err(i) => {
            if i == 0 {
                0
            } else if i >= n_lon {
                n_lon - 1
            } else {
                let d0 = (lons[i - 1] - lon).abs();
                let d1 = (lons[i] - lon).abs();
                if d0 < d1 { i - 1 } else { i }
            }
        }
    };

    // Flatten 2D index into 1D
    let idx = lat_idx * n_lon + lon_idx;

    Ok(data[idx])
}
