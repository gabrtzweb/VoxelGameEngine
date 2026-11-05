use crate::{
    generation::generator::TerrainColumn,
    world::{CHUNK_SIZE, Voxel},
};

/// Stub: flora population is temporarily bypassed to showcase pure terrain landforms.
#[inline]
pub fn populate_chunk_flora<F>(
    _voxels: &mut [Voxel],
    _columns: &[TerrainColumn; CHUNK_SIZE * CHUNK_SIZE],
    _chunk_origin: bevy::prelude::IVec3,
    _seed: u32,
    _voxel_at: F,
) where
    F: Fn(i32, i32, i32, TerrainColumn) -> Voxel,
{
    // Temporarily disabled: no-op stub
}
