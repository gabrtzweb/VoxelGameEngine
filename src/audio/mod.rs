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
            .add_systems(Update, sync_mute_on_unfocus)
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

/// Globally mutes all audio sinks (music, ambience, weather, footsteps, SFX, UI) when the game
/// window loses focus and `mute_on_unfocus` is enabled in Audio Settings.
fn sync_mute_on_unfocus(
    window_q: Query<&Window, With<bevy::window::PrimaryWindow>>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
    mut audio_settings: ResMut<AudioSettings>,
    mut global_volume: ResMut<GlobalVolume>,
    mut sinks_q: Query<&mut AudioSink>,
    mut was_muted: Local<bool>,
) {
    let mute_enabled = game_settings
        .as_deref()
        .is_some_and(|s| s.mute_on_unfocus);
    let is_focused = window_q.single().map_or(true, |w| w.focused);
    let should_mute = mute_enabled && !is_focused;

    if should_mute != *was_muted {
        *was_muted = should_mute;
        audio_settings.unfocused_muted = should_mute;
        global_volume.volume = if should_mute {
            bevy::audio::Volume::Linear(0.0)
        } else {
            bevy::audio::Volume::Linear(1.0)
        };
        for mut sink in &mut sinks_q {
            if should_mute {
                sink.mute();
            } else {
                sink.unmute();
            }
        }
    } else if should_mute {
        // Also mute any newly spawned AudioSinks that appear while unfocused
        for mut sink in &mut sinks_q {
            if !sink.is_muted() {
                sink.mute();
            }
        }
    }
}
