const LAHR_LAT_F32: f32 = 48.123;
const LAHR_LON_F32: f32 = 7.873;

pub fn get_lat_f32(x: f32) -> f32 {
    LAHR_LAT_F32 + x / 111_320.0
}

pub fn get_lon_f32(y: f32) -> f32 {
    LAHR_LON_F32 + y / (111_320.0 * LAHR_LAT_F32.to_radians().cos())
}

const LAHR_LAT_F64: f64 = 48.123;
const LAHR_LON_F64: f64 = 7.873;

pub fn get_lat_f64(x: f64) -> f64 {
    LAHR_LAT_F64 + x / 111_320.0
}

pub fn get_lon_f64(y: f64) -> f64 {
    LAHR_LON_F64 + y / (111_320.0 * LAHR_LAT_F64.to_radians().cos())
}
