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
}

impl Default for MapPixel {
    fn default() -> Self {
        Self {
            voxel: Voxel::Air,
            height: 0,
            water_depth: 0,
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

    /// Returns true if the chunk column has been explored.
    #[allow(dead_code)]
    pub fn contains_chunk(&self, chunk_col: IVec2) -> bool {
        self.chunks.contains_key(&chunk_col)
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

    /// Returns total number of explored chunk columns currently cached.
    #[allow(dead_code)]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
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
    let mut map_chunk = MapChunk::default();
    let mut has_any_terrain = false;

    for lx in 0..CHUNK_SIZE {
        for lz in 0..CHUNK_SIZE {
            let mut surface_voxel = Voxel::Air;
            let mut water_surface_y = i32::MIN;
            let mut ground_y = i32::MIN;

            'column_scan: for chunk_y in (WORLD_MIN_CHUNK_Y..=WORLD_MAX_CHUNK_Y).rev() {
                let chunk_coord = IVec3::new(col.x, chunk_y, col.y);
                let Some(chunk) = world.get_chunk(chunk_coord) else {
                    continue;
                };

                match chunk.homogeneity() {
                    ChunkHomogeneity::Empty => {
                        // All air, continue down
                        continue;
                    }
                    ChunkHomogeneity::Solid(v) => {
                        if v.is_empty() {
                            continue;
                        }

                        let top_y = chunk_y * CHUNK_SIZE as i32 + (CHUNK_SIZE - 1) as i32;
                        if v.is_water() {
                            if water_surface_y == i32::MIN {
                                water_surface_y = top_y;
                            }
                            // Solid water chunk, continue downward to find sea floor
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

                            let world_y = chunk_y * CHUNK_SIZE as i32 + ly as i32;
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

            if water_surface_y != i32::MIN {
                let depth = if ground_y != i32::MIN {
                    (water_surface_y - ground_y).clamp(1, 255) as u8
                } else {
                    1
                };
                map_chunk.set(
                    lx,
                    lz,
                    MapPixel {
                        voxel: Voxel::Water,
                        height: water_surface_y as i16,
                        water_depth: depth,
                    },
                );
                has_any_terrain = true;
            } else if ground_y != i32::MIN {
                map_chunk.set(
                    lx,
                    lz,
                    MapPixel {
                        voxel: surface_voxel,
                        height: ground_y as i16,
                        water_depth: 0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Chunk;

    #[test]
    fn test_map_chunk_get_set() {
        let mut chunk = MapChunk::default();
        let pixel = MapPixel {
            voxel: Voxel::Grass,
            height: 64,
            water_depth: 0,
        };
        chunk.set(5, 10, pixel);
        assert_eq!(chunk.get(5, 10), pixel);
        assert_eq!(chunk.get(0, 0), MapPixel::default());
    }

    #[test]
    fn test_map_cache_dirty_and_update() {
        let mut cache = MapCache::default();
        let mut world = VoxelWorld::default();

        // Place a solid stone chunk at (0, 0, 0)
        let stone_chunk = Chunk::filled(Voxel::Stone);
        world.insert_chunk(IVec3::new(0, 0, 0), stone_chunk);

        // Mark column (0, 0) dirty
        cache.mark_dirty(IVec2::new(0, 0));
        assert_eq!(cache.chunk_count(), 0);

        // Update dirty columns
        cache.update_dirty_columns(&world);
        assert_eq!(cache.chunk_count(), 1);

        // Pixel at (4, 4) should be Stone at height 15
        let p = cache.get_pixel(4, 4).expect("pixel should exist");
        assert_eq!(p.voxel, Voxel::Stone);
        assert_eq!(p.height, 15);
        assert_eq!(p.water_depth, 0);
    }

    #[test]
    fn test_map_cache_water_detection() {
        let mut cache = MapCache::default();
        let mut world = VoxelWorld::default();

        // Create a mixed chunk at (0, 0, 0) with Sand at y=5 and Water at y=6..10
        let mut chunk = Chunk::new();
        for x in 0..16 {
            for z in 0..16 {
                chunk.set(x, 5, z, Voxel::Sand);
                for y in 6..=10 {
                    chunk.set(x, y, z, Voxel::Water);
                }
            }
        }
        world.insert_chunk(IVec3::new(0, 0, 0), chunk);

        cache.mark_dirty(IVec2::new(0, 0));
        cache.update_dirty_columns(&world);

        let p = cache.get_pixel(8, 8).expect("pixel should exist");
        assert_eq!(p.voxel, Voxel::Water);
        assert_eq!(p.height, 10);
        assert_eq!(p.water_depth, 5); // 10 - 5 = 5
    }

    #[test]
    fn test_map_cache_clear() {
        let mut cache = MapCache::default();
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::new(0, 0, 0), Chunk::filled(Voxel::Grass));

        cache.mark_dirty(IVec2::new(0, 0));
        cache.update_dirty_columns(&world);
        assert_eq!(cache.chunk_count(), 1);

        cache.clear();
        assert_eq!(cache.chunk_count(), 0);
        assert!(cache.get_pixel(0, 0).is_none());
    }
}
