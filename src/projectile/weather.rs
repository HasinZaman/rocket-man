use bevy::app::{Plugin, Startup, Update};
use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetApp, AssetLoader, AssetServer, Assets, Handle, LoadContext};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Res, ResMut};
use bevy::reflect::TypePath;
use ron::de::SpannedError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::projectile::util::{GAS_CONSTANT, GRAVITY, celsius_to_kelvin};

#[derive(Asset, TypePath, Debug, Serialize, Deserialize)]
pub struct WeatherData {
    pub lats: Vec<f32>,
    pub lons: Vec<f32>,
    pub temperature_2m: Vec<f32>,
    pub pressure_msl: Vec<f32>,
    pub u10: Vec<f32>,
    pub v10: Vec<f32>,
    pub u100: Vec<f32>,
    pub v100: Vec<f32>,
    pub cloud_low: Vec<f32>,
    pub cloud_mid: Vec<f32>,
    pub cloud_high: Vec<f32>,
}

#[derive(Debug, Error)]
pub enum WeatherDataLoaderError {
    #[error("IO error while reading file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse RON config: {0}")]
    Ron(#[from] SpannedError),
}

#[derive(Default)]
pub struct WeatherDataLoader;

impl AssetLoader for WeatherDataLoader {
    type Asset = WeatherData;
    type Settings = ();
    type Error = WeatherDataLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let data: WeatherData = ron::de::from_bytes(&bytes)?;

        Ok(data)
    }

    fn extensions(&self) -> &[&str] {
        &["weather"]
    }
}

#[derive(Resource, Clone)]
struct WeatherInitialized(Option<Handle<WeatherData>>, bool);
impl Default for WeatherInitialized {
    fn default() -> Self {
        Self(None, false)
    }
}

#[derive(Resource, Default)]
pub struct WeatherMeta {
    lats: Vec<f32>,
    lons: Vec<f32>,
}

#[derive(Resource, Default)]
pub struct Wind((Vec<f32>, Vec<f32>), (Vec<f32>, Vec<f32>));

#[derive(Resource, Default)]
pub struct Pressure(Vec<f32>);

#[derive(Resource, Default)]
pub struct Temperature(Vec<f32>);

#[derive(Resource, Default)]
pub struct CloudCover {
    low: Vec<f32>,
    mid: Vec<f32>,
    high: Vec<f32>,
}

pub(super) fn find(lat: f32, lon: f32, meta: &Res<WeatherMeta>, data: &[f32]) -> Result<f32, ()> {
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
    let lat = lat.clamp(lats[0], lats[n_lat - 1]);
    let lon = lon.clamp(lons[0], lons[n_lon - 1]);

    // --- Find indices for bounding box safely ---
    let lat_idx = match lats.binary_search_by(|x| x.partial_cmp(&lat).unwrap()) {
        Ok(i) => i.min(n_lat - 2),
        Err(i) => i.saturating_sub(1).min(n_lat - 2),
    };

    let lon_idx = match lons.binary_search_by(|x| x.partial_cmp(&lon).unwrap()) {
        Ok(i) => i.min(n_lon - 2),
        Err(i) => i.saturating_sub(1).min(n_lon - 2),
    };

    // --- Get surrounding lat/lon points ---
    let lat0 = lats[lat_idx];
    let lat1 = lats[lat_idx + 1];
    let lon0 = lons[lon_idx];
    let lon1 = lons[lon_idx + 1];

    // --- Prevent divide-by-zero ---
    let denom_lat = (lat1 - lat0).abs().max(f32::EPSILON);
    let denom_lon = (lon1 - lon0).abs().max(f32::EPSILON);

    // --- Compute normalized weights ---
    let t = (lat - lat0) / denom_lat;
    let u = (lon - lon0) / denom_lon;

    // --- Retrieve four corner values ---
    let idx = |i, j| i * n_lon + j;
    let f00 = data[idx(lat_idx, lon_idx)];
    let f10 = data[idx(lat_idx + 1, lon_idx)];
    let f01 = data[idx(lat_idx, lon_idx + 1)];
    let f11 = data[idx(lat_idx + 1, lon_idx + 1)];

    // --- Bilinear interpolation ---
    let f0 = f00 * (1.0 - t) + f10 * t;
    let f1 = f01 * (1.0 - t) + f11 * t;
    let value = f0 * (1.0 - u) + f1 * u;

    Ok(value)
}

pub fn get_temperature(
    lat: f32,
    lon: f32,
    altitude: f32,
    meta: &Res<WeatherMeta>,
    temperature: &Res<Temperature>,
) -> f32 {
    const DEFAULT_SURFACE_TEMP: f32 = 30.0;

    let surface_temperature: f32 =
        celsius_to_kelvin(find(lat, lon, meta, &temperature.0).unwrap_or(DEFAULT_SURFACE_TEMP));

    if altitude <= 11_000.0 {
        surface_temperature - 0.0065 * altitude
    } else {
        216.65
    }
}

pub fn get_pressure(
    lat: f32,
    lon: f32,
    altitude: f32,
    meta: &Res<WeatherMeta>,
    pressure: &Res<Pressure>,
    temperature: &f32,
) -> f32 {
    const DEFAULT_SURFACE_PRESSURE: f32 = 101_325.0;

    let surface_pressure_pa: f32 =
        find(lat, lon, meta, &pressure.0).unwrap_or(DEFAULT_SURFACE_PRESSURE);

    // let temperature: f32 = get_temperature(lat, lon, altitude, meta, temperature);

    const LAPSE_RATE: f32 = 0.0065; // K/m

    if altitude <= 11_000.0 {
        surface_pressure_pa
            * (1.0 - LAPSE_RATE * altitude / temperature)
                .powf(GRAVITY / (GAS_CONSTANT * LAPSE_RATE))
    } else {
        let pressure_at_11km: f32 = surface_pressure_pa
            * (1.0 - LAPSE_RATE * 11_000.0 / 288.15).powf(GRAVITY / (GAS_CONSTANT * LAPSE_RATE));
        pressure_at_11km * (-GRAVITY * (altitude - 11_000.0) / (GAS_CONSTANT * temperature)).exp()
    }
}

pub fn get_wind(
    lat: f32,
    lon: f32,
    altitude: f32,
    meta: &Res<WeatherMeta>,
    wind: &Res<Wind>,
) -> (f32, f32) {
    let (low_wind, high_wind) = (&wind.0, &wind.1);

    let (u_10m_data, v_10m_data) = (&low_wind.0, &low_wind.1);
    let (u_100m_data, v_100m_data) = (&high_wind.0, &high_wind.1);

    // --- Interpolate base values for 10 m and 100 m levels ---
    let u10 = find(lat, lon, meta, u_10m_data).unwrap_or(0.);
    let v10 = find(lat, lon, meta, v_10m_data).unwrap_or(0.);
    let u100 = find(lat, lon, meta, u_100m_data).unwrap_or(0.);
    let v100 = find(lat, lon, meta, v_100m_data).unwrap_or(0.);

    // --- Compute horizontal wind at requested altitude ---
    let (u, v) = if altitude <= 10.0 {
        (u10, v10)
    } else if altitude <= 100.0 {
        let t = (altitude - 10.0) / 90.0;
        (u10 + t * (u100 - u10), v10 + t * (v100 - v10))
    } else if altitude <= 2000.0 {
        let scale = (altitude / 100.0).ln() / (20.0_f32).ln();
        (u100 * (1.0 + 0.1 * scale), v100 * (1.0 + 0.1 * scale))
    } else {
        (u100 * 1.1, v100 * 1.1)
    };

    (u, v)
}

fn load_weather_data(
    asset_server: Res<AssetServer>,

    mut weather_intialized: ResMut<WeatherInitialized>,
) {
    let handle: Handle<WeatherData> = asset_server.load("weather/data.weather");

    weather_intialized.0 = Some(handle);
}

fn initialize_weather(
    mut weather_intialized: ResMut<WeatherInitialized>,

    weather_assets: Res<Assets<WeatherData>>,

    mut meta: ResMut<WeatherMeta>,
    mut wind: ResMut<Wind>,
    mut pressure: ResMut<Pressure>,
    mut temperature: ResMut<Temperature>,
    mut cloud_cover: ResMut<CloudCover>,
) {
    if weather_intialized.1 {
        return;
    }
    if weather_intialized.0.is_none() {
        return;
    }

    let handle: Handle<WeatherData> = weather_intialized.0.clone().unwrap();

    let Some(data) = weather_assets.get(handle.id()) else {
        return;
    };

    temperature.0 = data.temperature_2m.clone();
    pressure.0 = data.pressure_msl.clone();

    wind.0 = (
        data.u10.iter().cloned().collect(),
        data.v10.iter().cloned().collect(),
    );

    wind.1 = (
        data.u100.iter().cloned().collect(),
        data.v100.iter().cloned().collect(),
    );

    cloud_cover.low = data.cloud_low.clone();
    cloud_cover.mid = data.cloud_mid.clone();
    cloud_cover.high = data.cloud_high.clone();

    meta.lats = data.lats.clone();
    meta.lons = data.lons.clone();

    weather_intialized.0 = None;
    weather_intialized.1 = true;
}

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_asset::<WeatherData>()
            .init_asset_loader::<WeatherDataLoader>()
            .init_resource::<WeatherInitialized>()
            .init_resource::<WeatherMeta>()
            .init_resource::<Wind>()
            .init_resource::<Pressure>()
            .init_resource::<Temperature>()
            .init_resource::<CloudCover>()
            .add_systems(Startup, load_weather_data)
            .add_systems(Update, initialize_weather);
    }
}
