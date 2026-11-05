//! Procedural tree generation (temporarily bypassed during terrain & landform overhaul).

use crate::world::Voxel;

/// Identifies trunk and structural wood blocks.
#[inline]
pub fn is_trunk(voxel: Voxel) -> bool {
    voxel.is_wood_block()
}

/// Identifies leaf voxels.
#[inline]
pub fn is_leaves(voxel: Voxel) -> bool {
    voxel.is_leaves()
}

/// Fast, deterministic hash function for 2D cellular grid sampling.
#[inline]
pub fn hash_tree_cell(cell_x: i32, cell_z: i32, seed: u32) -> u32 {
    let mut h = seed
        .wrapping_add((cell_x as u32).wrapping_mul(0x85ebca6b))
        .wrapping_add((cell_z as u32).wrapping_mul(0xc2b2ae35));
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}
