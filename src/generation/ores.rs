use bevy::prelude::*;
use crate::world::Voxel;

/// Stub: procedural ore vein generation is temporarily bypassed to showcase pure terrain landforms.
#[inline]
pub fn populate_chunk_ores(_voxels: &mut [Voxel], _chunk_origin: IVec3, _seed: u32) {
    // Temporarily disabled: no-op stub
}

/// Stub: procedural geode generation is temporarily bypassed to showcase pure terrain landforms.
#[inline]
pub fn populate_chunk_geodes(_voxels: &mut [Voxel], _chunk_origin: IVec3, _seed: u32) {
    // Temporarily disabled: no-op stub
}
