pub mod async_mesher;
pub mod greedy;
pub mod pipeline;
pub mod shapes;
pub mod textures;

pub use async_mesher::ChunkMeshingTask;
pub use pipeline::{ChunkMaterial, ChunkMeshRegistry, remove_chunk_render, sync_chunk_render};
pub use textures::VoxelTextureRegistry;

use async_mesher::AsyncMesherPlugin;
use bevy::prelude::*;
use pipeline::VoxelMaterial;

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default())
            .add_plugins(AsyncMesherPlugin);
    }
}
