pub const GRAVITY: f32 = 9.80907; //m/s^2 (wolfram alpha)

#[inline]
pub fn altitude(y: f32) -> f32 {
    y + 156.
}

const LAHR_LAT: f32 = 48.123;
const LAHR_LON: f32 = 7.873;

pub fn get_lat(x: f32) -> f32 {
    LAHR_LAT + x / 111_320.0
}

pub fn get_lon(y: f32) -> f32 {
    LAHR_LON + y / (111_320.0 * LAHR_LAT.to_radians().cos())
}

pub const GAS_CONSTANT: f32 = 287.05;
#[inline]
pub fn air_density(air_pressure: f32, temperature: f32) -> f32 {
    air_pressure / (GAS_CONSTANT * temperature)
}

#[inline]
pub fn speed_of_sound(temperature: f32) -> f32 {
    (1.4 * GAS_CONSTANT * temperature).sqrt()
}

#[inline]
pub fn celsius_to_kelvin(temp_c: f32) -> f32 {
    temp_c + 273.15
}
