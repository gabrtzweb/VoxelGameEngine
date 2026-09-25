use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;

use crate::world::{
    CHUNK_SIZE, ChunkHomogeneity, Voxel, VoxelWorld, WORLD_MAX_CHUNK_Y, WORLD_MIN_CHUNK_Y,
};

/// Represents the top-down surface data for a single (x, z) voxel column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapPixel {
    /// The surface voxel (topmost non-air block, or water).
    pub voxel: Voxel,
    /// World Y coordinate of the surface block.
    pub height: i16,
    /// Depth of water if the surface voxel is water, or 0 if solid ground.
    pub water_depth: u8,
    /// Precomputed RGBA surface color with biome tinting.
    pub color: [u8; 4],
}

impl Default for MapPixel {
    fn default() -> Self {
        Self {
            voxel: Voxel::Air,
            height: 0,
            water_depth: 0,
            color: [0, 0, 0, 0],
        }
    }
}

/// Cached 2D surface representation of a 16x16 chunk column.
#[derive(Clone, Debug)]
pub struct MapChunk {
    pub pixels: [MapPixel; CHUNK_SIZE * CHUNK_SIZE],
}

impl Default for MapChunk {
    fn default() -> Self {
        Self {
            pixels: [MapPixel::default(); CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl MapChunk {
    #[inline(always)]
    pub fn get(&self, x: usize, z: usize) -> MapPixel {
        debug_assert!(x < CHUNK_SIZE && z < CHUNK_SIZE);
        self.pixels[x + z * CHUNK_SIZE]
    }

    #[inline(always)]
    pub fn set(&mut self, x: usize, z: usize, pixel: MapPixel) {
        debug_assert!(x < CHUNK_SIZE && z < CHUNK_SIZE);
        self.pixels[x + z * CHUNK_SIZE] = pixel;
    }
}

/// Shared map data cache storing top-down surface terrain across all explored chunks.
#[derive(Resource, Default)]
pub struct MapCache {
    chunks: HashMap<IVec2, MapChunk>,
    dirty_columns: HashSet<IVec2>,
    /// Incremented whenever any chunk is updated, cleared, or loaded.
    pub version: u64,
}

impl MapCache {
    /// Marks a chunk column `(chunk_x, chunk_z)` as needing re-extraction.
    pub fn mark_dirty(&mut self, chunk_col: IVec2) {
        self.dirty_columns.insert(chunk_col);
    }

    /// Marks the chunk column containing a world voxel coordinate as dirty.
    pub fn mark_block_dirty(&mut self, world_voxel: IVec3) {
        let chunk_size = CHUNK_SIZE as i32;
        let col = IVec2::new(
            world_voxel.x.div_euclid(chunk_size),
            world_voxel.z.div_euclid(chunk_size),
        );
        self.mark_dirty(col);
    }

    /// Retrieves cached map data for a chunk column, if it has been explored and generated.
    pub fn get_chunk(&self, chunk_col: IVec2) -> Option<&MapChunk> {
        self.chunks.get(&chunk_col)
    }

    /// Retrieves the surface pixel for a specific world (x, z) block coordinate.
    pub fn get_pixel(&self, wx: i32, wz: i32) -> Option<MapPixel> {
        let chunk_size = CHUNK_SIZE as i32;
        let col = IVec2::new(wx.div_euclid(chunk_size), wz.div_euclid(chunk_size));
        let chunk = self.get_chunk(col)?;
        let lx = wx.rem_euclid(chunk_size) as usize;
        let lz = wz.rem_euclid(chunk_size) as usize;
        Some(chunk.get(lx, lz))
    }

    /// Clears all cached map data.
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.dirty_columns.clear();
        self.version = self.version.wrapping_add(1);
    }

    /// Updates all dirty columns by scanning top-down through loaded chunks in `VoxelWorld`.
    pub fn update_dirty_columns(&mut self, world: &VoxelWorld) {
        if self.dirty_columns.is_empty() {
            return;
        }

        let dirty_list: Vec<IVec2> = self.dirty_columns.drain().collect();
        let mut any_changed = false;

        for col in dirty_list {
            if let Some(map_chunk) = extract_column_surface(world, col) {
                self.chunks.insert(col, map_chunk);
                any_changed = true;
            }
        }

        if any_changed {
            self.version = self.version.wrapping_add(1);
        }
    }
}

/// Scans a 16x16 chunk column from top to bottom in `VoxelWorld` to determine surface terrain.
fn extract_column_surface(world: &VoxelWorld, col: IVec2) -> Option<MapChunk> {
    // 1. Fetch chunks in the column once from top to bottom (only 33 hash map lookups instead of 8,448)
    let mut col_chunks: Vec<(i32, &crate::world::Chunk)> = Vec::with_capacity(33);
    for chunk_y in (WORLD_MIN_CHUNK_Y..=WORLD_MAX_CHUNK_Y).rev() {
        if let Some(chunk) = world.get_chunk(IVec3::new(col.x, chunk_y, col.y))
            && chunk.homogeneity() != ChunkHomogeneity::Empty
        {
            col_chunks.push((chunk_y, chunk));
        }
    }

    if col_chunks.is_empty() {
        return None;
    }

    let mut map_chunk = MapChunk::default();
    let mut has_any_terrain = false;
    let chunk_size = CHUNK_SIZE as i32;

    for lx in 0..CHUNK_SIZE {
        for lz in 0..CHUNK_SIZE {
            let mut surface_voxel = Voxel::Air;
            let mut water_surface_y = i32::MIN;
            let mut ground_y = i32::MIN;

            'column_scan: for &(chunk_y, chunk) in &col_chunks {
                match chunk.homogeneity() {
                    ChunkHomogeneity::Empty => {
                        continue;
                    }
                    ChunkHomogeneity::Solid(v) => {
                        if v.is_empty() {
                            continue;
                        }

                        let top_y = chunk_y * chunk_size + (chunk_size - 1);
                        if v.is_water() {
                            if water_surface_y == i32::MIN {
                                water_surface_y = top_y;
                            }
                            continue;
                        } else {
                            ground_y = top_y;
                            surface_voxel = v;
                            break 'column_scan;
                        }
                    }
                    ChunkHomogeneity::Mixed => {
                        for ly in (0..CHUNK_SIZE).rev() {
                            let v = chunk.get(lx, ly, lz);
                            if v.is_empty() {
                                continue;
                            }

                            let world_y = chunk_y * chunk_size + ly as i32;
                            if v.is_water() {
                                if water_surface_y == i32::MIN {
                                    water_surface_y = world_y;
                                }
                            } else {
                                ground_y = world_y;
                                surface_voxel = v;
                                break 'column_scan;
                            }
                        }
                    }
                }
            }

            let world_x = col.x * chunk_size + lx as i32;
            let world_z = col.y * chunk_size + lz as i32;

            if water_surface_y != i32::MIN {
                let depth = if ground_y != i32::MIN {
                    (water_surface_y - ground_y).clamp(1, 255) as u8
                } else {
                    1
                };
                let color = super::color::voxel_map_color_at(Voxel::Water, depth, world_x, world_z);
                map_chunk.set(
                    lx,
                    lz,
                    MapPixel {
                        voxel: Voxel::Water,
                        height: water_surface_y as i16,
                        water_depth: depth,
                        color,
                    },
                );
                has_any_terrain = true;
            } else if ground_y != i32::MIN {
                let color = super::color::voxel_map_color_at(surface_voxel, 0, world_x, world_z);
                map_chunk.set(
                    lx,
                    lz,
                    MapPixel {
                        voxel: surface_voxel,
                        height: ground_y as i16,
                        water_depth: 0,
                        color,
                    },
                );
                has_any_terrain = true;
            }
        }
    }

    if has_any_terrain {
        Some(map_chunk)
    } else {
        None
    }
}
