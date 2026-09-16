#![allow(unused_imports)]

pub mod async_mesher;
pub mod greedy;
pub mod pipeline;
pub mod shapes;
pub mod textures;

pub use async_mesher::{
    AsyncMesherPlugin, ChunkMeshingTask, CompletedChunkMesh, collect_meshing_tasks,
    start_meshing_tasks,
};
pub use greedy::{ChunkMesher, ChunkMeshes, FaceDirection, FaceKey, MeshBuffers};
pub use pipeline::{
    ChunkMaterial, ChunkMeshRegistry, ChunkRenderData, ChunkRenderPart, VOXEL_SHADER_PATH,
    VoxelMaterial, VoxelMaterialExtension, apply_chunk_mesh, remove_chunk_render,
    setup_chunk_material, sync_chunk_render,
};
pub use shapes::{
    get_chunk_local_centered_material, is_chunk_local_centered_layer, is_chunk_local_isolated_voxel,
    mesh_centered_voxels, push_centered_quad, push_water_quad_both_sides, push_water_side_quad,
};
pub use textures::{
    LoadedTexture, MAX_VOXEL_VARIANTS, TEXTURE_RESOLUTION, VoxelTextureMapping,
    VoxelTextureRegistry, build_voxel_texture_array,
};

use bevy::prelude::*;

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default())
            .add_plugins(AsyncMesherPlugin);
    }
}
