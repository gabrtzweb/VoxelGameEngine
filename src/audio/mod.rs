pub mod music;
pub mod settings;
pub mod sfx;
pub mod weather_audio;

use bevy::prelude::*;

pub use music::GameplayMusicPlugin;
pub use settings::AudioSettings;
pub use sfx::{PlaySfxEvent, PlaySpatialSfxEvent, SfxPlugin};
pub use weather_audio::WeatherAudioPlugin;

/// Central game engine audio plugin handling settings, spatial SFX, atmospheric weather audio, and soundtrack.
pub struct EngineAudioPlugin;

impl Plugin for EngineAudioPlugin {
    fn build(&self, app: &mut App) {
        let initial_settings = app
            .world()
            .get_resource::<crate::core::LocalConfig>()
            .map(AudioSettings::from_local_config)
            .unwrap_or_default();

        app.insert_resource(initial_settings)
            .register_type::<AudioSettings>()
            .add_plugins((SfxPlugin, WeatherAudioPlugin, GameplayMusicPlugin));
    }
}
