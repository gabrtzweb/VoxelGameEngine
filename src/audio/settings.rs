use bevy::prelude::*;
use crate::core::LocalConfig;

/// Central volume settings configuration across the engine.
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct AudioSettings {
    /// Global master volume multiplier [0.0..=1.0].
    pub master_volume: f32,
    /// Environmental and meteorological phenomena volume [0.0..=1.0].
    pub weather_volume: f32,
    /// Sound effects volume (block breaks, footsteps, combat, interactions) [0.0..=1.0].
    pub sfx_volume: f32,
    /// Background music score volume [0.0..=1.0].
    pub music_volume: f32,
    /// Ambient biome background murmurs and nature loops [0.0..=1.0].
    pub ambient_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.85,
            weather_volume: 0.75,
            sfx_volume: 0.85,
            music_volume: 0.50,
            ambient_volume: 0.65,
        }
    }
}

impl AudioSettings {
    pub fn from_local_config(config: &LocalConfig) -> Self {
        Self {
            master_volume: config.master_volume,
            weather_volume: config.weather_volume,
            sfx_volume: config.sfx_volume,
            music_volume: config.music_volume,
            ambient_volume: config.ambient_volume,
        }
    }
}
