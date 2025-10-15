use bevy::prelude::*;

use crate::cf104::console::{
    altimeter::update_altimeter, clock::update_clock, gyro_compass::update_compass_gyro,
    speedometer::update_speedometer,
};

pub mod altimeter;
pub mod clock;
pub mod gyro_compass;
pub mod radio;
pub mod speedometer;
pub mod throttle;

#[derive(Component)]
pub struct RotRange {
    pub min: Quat,
    pub max: Quat,
}

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_clock,
                update_compass_gyro,
                update_altimeter,
                update_speedometer,
            ),
        );
    }
}
