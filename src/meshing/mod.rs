pub mod async_mesher;
pub mod culling;
pub mod greedy;
pub mod lod;
pub mod pipeline;
pub mod shapes;
pub mod textures;

pub use async_mesher::ChunkMeshingTask;
#[allow(unused_imports)]
pub use culling::{CaveCullingState, ChunkCoordinate, SubterraneanChunkMesh};
#[allow(unused_imports)]
pub use lod::{ChunkLod, build_lod_mesh};
#[allow(unused_imports)]
pub use pipeline::{
    ChunkMaterial, ChunkMeshRegistry, LodMeshRegistry, OpaqueChunkMaterial, OpaqueVoxelMaterial,
    TransparentChunkMaterial, TransparentVoxelMaterial, apply_lod_mesh, remove_chunk_render,
    remove_lod_render, sync_chunk_render,
};
pub use textures::VoxelTextureRegistry;

use async_mesher::AsyncMesherPlugin;
use bevy::prelude::*;

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LodMeshRegistry>()
            .init_resource::<culling::CaveCullingState>()
            .add_systems(PostUpdate, culling::update_cave_culling_system)
            .add_plugins((
                MaterialPlugin::<OpaqueChunkMaterial>::default(),
                MaterialPlugin::<TransparentChunkMaterial>::default(),
            ))
            .add_plugins(AsyncMesherPlugin);
    }
}
