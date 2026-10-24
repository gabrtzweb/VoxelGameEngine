pub mod buffer;
pub mod emitters;
pub mod renderer;

use bevy::prelude::*;

pub use buffer::{
    MAX_WEATHER_PARTICLES, ParticleFadeMode, ParticleRotationMode, WeatherParticle,
    WeatherParticleKind, WeatherParticlePool,
};
pub use emitters::WeatherRng;
pub use renderer::WeatherMeshHandle;

use emitters::emit_weather_particles;
use renderer::{setup_weather_renderer, sync_weather_particle_mesh};

pub struct WeatherParticlePlugin;

impl Plugin for WeatherParticlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherParticlePool>()
            .register_type::<WeatherParticlePool>()
            .init_resource::<WeatherRng>()
            .add_systems(Startup, setup_weather_renderer)
            .add_systems(
                Update,
                (
                    emit_weather_particles,
                    sync_weather_particle_mesh,
                )
                    .chain()
                    .in_set(WeatherParticleSet),
            );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WeatherParticleSet;
