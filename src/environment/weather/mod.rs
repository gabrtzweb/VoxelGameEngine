pub mod lightning;
pub mod particles;
pub mod state;
pub mod wind;

use bevy::prelude::*;

pub use lightning::{LightningPlugin, LightningState, TriggerThunderEvent};
pub use particles::{WeatherParticlePool, WeatherParticlePlugin};
pub use state::{WeatherManager, WeatherType};
pub use wind::WindState;

use state::{advance_weather_simulation, handle_weather_input};
use wind::update_wind_field;

/// Main Weather Plugin configuring dynamic meteorological phenomena.
pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherManager>()
            .register_type::<WeatherManager>()
            .init_resource::<WindState>()
            .register_type::<WindState>()
            .add_plugins((particles::WeatherParticlePlugin, lightning::LightningPlugin))
            .add_systems(
                Update,
                (
                    handle_weather_input,
                    advance_weather_simulation,
                    update_wind_field,
                )
                    .chain()
                    .in_set(WeatherSystemSet),
            );
    }
}

/// System set ensuring deterministic execution order for weather subsystems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WeatherSystemSet;
