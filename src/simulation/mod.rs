#![allow(unused_imports)]

pub mod fluid;
pub mod lighting;

pub use fluid::{
    FluidSimulationPlugin, FluidUpdateQueue, MAX_FULL_WATER_SPREAD, MAX_SINGLE_WATER_SPREAD,
    WaterInfo, compute_water_distance, compute_water_info, water_surface_height_offset,
};
pub use lighting::{
    LightState, VoxelLightRegistry, remove_chunk_lights, sync_chunk_lights, sync_voxel_light,
};

use bevy::prelude::*;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FluidSimulationPlugin);
    }
}
