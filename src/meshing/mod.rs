pub mod async_mesher;
pub mod greedy;
pub mod pipeline;
pub mod shapes;
pub mod textures;

pub use async_mesher::ChunkMeshingTask;
#[allow(unused_imports)]
pub use pipeline::{
    ChunkMaterial, ChunkMeshRegistry, OpaqueChunkMaterial, OpaqueVoxelMaterial,
    TransparentChunkMaterial, TransparentVoxelMaterial, VoxelMaterial, remove_chunk_render,
    sync_chunk_render,
};
pub use textures::VoxelTextureRegistry;

use async_mesher::AsyncMesherPlugin;
use bevy::prelude::*;

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<OpaqueChunkMaterial>::default(),
            MaterialPlugin::<TransparentChunkMaterial>::default(),
        ))
        .add_plugins(AsyncMesherPlugin);
    }
}
