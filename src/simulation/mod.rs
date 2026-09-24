pub mod fluid;
pub mod lighting;

use bevy::prelude::*;

pub use fluid::FluidSimulationPlugin;
pub use lighting::{VoxelLightRegistry, sync_chunk_lights};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FluidSimulationPlugin);
    }
}
