use bevy::prelude::*;

use crate::cf104::console::{
    altimeter::update_altimeter, clock::update_clock, gyro_compass::update_compass_gyro, radio::{deferred_fx_change, load_channels, set_up_radio_audio, update_fx_selector, update_radio, update_volume_knob, DeferredFxChange, Radio, RadioChannelConfig, RadioChannelLoader, RadioChannels, UpdateRadioFx, UpdateVolume}, speedometer::update_speedometer
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
        app.init_asset::<RadioChannelConfig>()
            .init_asset_loader::<RadioChannelLoader>()
            .init_resource::<RadioChannels>()
            .init_resource::<Radio>()
            .add_message::<UpdateVolume>()
            .add_message::<UpdateRadioFx>()
            .add_message::<DeferredFxChange>()
            .add_systems(
                Update,
                (
                    update_clock,
                    update_compass_gyro,
                    update_altimeter,
                    update_speedometer,
                    update_fx_selector,
                    update_volume_knob,
                    update_radio,
                    deferred_fx_change
                ),
            )
            .add_systems(Startup, load_channels)
            .add_systems(PostStartup, set_up_radio_audio);
    }
}
