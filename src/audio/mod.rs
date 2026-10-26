pub mod ambience;
pub mod block_audio;
pub mod footsteps;
pub mod material;
pub mod music;
pub mod settings;
pub mod sfx;
pub mod ui;
pub mod weather_audio;

use bevy::prelude::*;

pub use ambience::AmbientAudioPlugin;
pub use block_audio::BlockAudioPlugin;
pub use footsteps::FootstepAudioPlugin;
pub use material::SoundMaterial;
pub use music::GameplayMusicPlugin;
pub use settings::AudioSettings;
pub use sfx::{PlaySfxEvent, PlaySpatialSfxEvent, SfxPlugin};
pub use ui::{PlayUiSoundEvent, UiAudioPlugin, UiSound};
pub use weather_audio::WeatherAudioPlugin;

/// Central game engine audio plugin handling settings, spatial SFX, block acoustics, footsteps, UI, ambient biomes, and weather audio.
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
            .register_type::<SoundMaterial>()
            .add_plugins((
                SfxPlugin,
                BlockAudioPlugin,
                FootstepAudioPlugin,
                UiAudioPlugin,
                AmbientAudioPlugin,
                WeatherAudioPlugin,
                GameplayMusicPlugin,
            ));
    }
}
