pub mod block;
pub mod chunk;
pub mod modifications;
pub mod storage;
pub mod streaming;

pub use block::{BlockShape, Voxel};
pub use chunk::{CHUNK_SIZE, CHUNK_VOLUME, Chunk, ChunkHomogeneity, VOXEL_SIZE};
pub use modifications::WorldModificationStore;
pub use storage::{CHUNK_WORLD_SIZE, ChunkNeighborhood, VoxelAccess, VoxelWorld, affected_chunks};
pub use streaming::{
    ChunkStreamingPlugin, ChunkStreamingQueues, ChunkStreamingSettings, NEIGHBOR_DIRECTIONS,
    WORLD_MAX_CHUNK_Y, WORLD_MIN_CHUNK_Y,
};

use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkStreamingPlugin);
    }
}
