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
    #[allow(dead_code)]
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

    /// Inserts or updates an LOD-generated chunk column surface on the world map.
    pub fn insert_lod_chunk(&mut self, col: IVec2, map_chunk: MapChunk) {
        if !self.chunks.contains_key(&col) {
            self.chunks.insert(col, map_chunk);
            self.version = self.version.wrapping_add(1);
        }
    }
}

/// Generates a 2D surface representation of an LOD chunk column directly from procedural terrain and trees.
pub fn generate_lod_map_chunk(
    col: IVec2,
    generator: &crate::generation::TerrainGenerator,
) -> MapChunk {
    let mut map_chunk = MapChunk::default();
    let chunk_size = CHUNK_SIZE as i32;
    let chunk_origin_x = col.x * chunk_size;
    let chunk_origin_z = col.y * chunk_size;

    for lx in 0..CHUNK_SIZE {
        for lz in 0..CHUNK_SIZE {
            let wx = chunk_origin_x + lx as i32;
            let wz = chunk_origin_z + lz as i32;

            let col_data = generator.sample_column(wx, wz);
            let surface_mat =
                generator.surface_material_at(wx, col_data.terrain_height, wz, col_data);

            if let Some(wl) = col_data.water_level
                && col_data.terrain_height < wl
            {
                let depth = (wl - col_data.terrain_height).clamp(1, 255) as u8;
                let color =
                    super::color::voxel_map_color_at(Voxel::Liquid_Water, depth, wx, wz);
                map_chunk.set(
                    lx,
                    lz,
                    MapPixel {
                        voxel: Voxel::Liquid_Water,
                        height: wl as i16,
                        water_depth: depth,
                        color,
                    },
                );
            } else {
                let color = super::color::voxel_map_color_at(surface_mat, 0, wx, wz);
                map_chunk.set(
                    lx,
                    lz,
                    MapPixel {
                        voxel: surface_mat,
                        height: col_data.terrain_height as i16,
                        water_depth: 0,
                        color,
                    },
                );
            }
        }
    }

    // Include procedural trees on the map
    if generator.tree_density > 0.0 {
        let min_cell_x = (chunk_origin_x - 3).div_euclid(5);
        let max_cell_x = (chunk_origin_x + 18).div_euclid(5);
        let min_cell_z = (chunk_origin_z - 3).div_euclid(5);
        let max_cell_z = (chunk_origin_z + 18).div_euclid(5);

        for cell_z in min_cell_z..=max_cell_z {
            for cell_x in min_cell_x..=max_cell_x {
                let hash =
                    crate::generation::trees::hash_tree_cell(cell_x, cell_z, generator.seed);
                let offset_x = (hash % 3) as i32 + 1;
                let offset_z = ((hash >> 2) % 3) as i32 + 1;
                let tx = cell_x * 5 + offset_x;
                let tz = cell_z * 5 + offset_z;

                let col_data = generator.sample_column(tx, tz);
                let Some((species, base_prob)) =
                    crate::generation::trees::biome_tree_profile(col_data.biome)
                else {
                    continue;
                };

                let roll = ((hash >> 4) & 0xFFFF) as f32 / 65535.0;
                if roll >= base_prob * generator.tree_density {
                    continue;
                }
                if col_data.is_cliff || col_data.surface_shape == crate::world::BlockShape::Stair {
                    continue;
                }

                let is_swamp_tree = species == crate::generation::trees::TreeSpecies::Mangrove
                    || (species == crate::generation::trees::TreeSpecies::Pine
                        && col_data.biome == crate::generation::BiomeType::CypressSwamp);

                let ty = if let Some(water_level) = col_data.water_level {
                    if !is_swamp_tree || water_level - col_data.terrain_height > 2 {
                        continue;
                    }
                    col_data.terrain_height
                } else {
                    col_data.terrain_height
                };

                let surface_mat = generator.surface_material_at(tx, ty, tz, col_data);
                if !crate::generation::trees::is_soil_valid_for_species(species, surface_mat) {
                    continue;
                }

                let leaf_opt = crate::generation::trees::leaves_voxel_variant(
                    species,
                    hash,
                    col_data.biome,
                );
                let display_voxel = leaf_opt.unwrap_or_else(|| species.log_voxel());

                let height = 6 + ((hash >> 12) % 3) as i32;
                let tree_top_y = (ty + height + 1) as i16;

                let r: i32 = if species == crate::generation::trees::TreeSpecies::Cactus {
                    0
                } else {
                    1
                };
                for dx in -r..=r {
                    for dz in -r..=r {
                        let wx = tx + dx;
                        let wz = tz + dz;
                        let lx = wx - chunk_origin_x;
                        let lz = wz - chunk_origin_z;
                        if lx >= 0 && lx < chunk_size && lz >= 0 && lz < chunk_size {
                            let color =
                                super::color::voxel_map_color_at(display_voxel, 0, wx, wz);
                            map_chunk.set(
                                lx as usize,
                                lz as usize,
                                MapPixel {
                                    voxel: display_voxel,
                                    height: tree_top_y,
                                    water_depth: 0,
                                    color,
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    map_chunk
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
                let color = super::color::voxel_map_color_at(Voxel::Liquid_Water, depth, world_x, world_z);
                map_chunk.set(
                    lx,
                    lz,
                    MapPixel {
                        voxel: Voxel::Liquid_Water,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::TerrainGenerator;

    #[test]
    fn test_generate_lod_map_chunk() {
        let generator = TerrainGenerator::default();
        let map_chunk = generate_lod_map_chunk(IVec2::new(0, 0), &generator);
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pixel = map_chunk.get(x, z);
                assert!(!pixel.voxel.is_empty(), "LOD map pixel must have a valid voxel material");
                assert!(pixel.color[3] > 0, "LOD map pixel must have non-zero alpha");
            }
        }
    }

    #[test]
    fn test_insert_lod_chunk_does_not_overwrite_real_chunks() {
        let mut cache = MapCache::default();
        let col = IVec2::new(5, 5);
        let real_pixel = MapPixel {
            voxel: Voxel::Rock_Basalt,
            height: 100,
            water_depth: 0,
            color: [100, 100, 100, 255],
        };
        let mut real_chunk = MapChunk::default();
        real_chunk.set(0, 0, real_pixel);
        cache.chunks.insert(col, real_chunk);

        let lod_pixel = MapPixel {
            voxel: Voxel::Soil_Grass,
            height: 50,
            water_depth: 0,
            color: [50, 200, 50, 255],
        };
        let mut lod_chunk = MapChunk::default();
        lod_chunk.set(0, 0, lod_pixel);

        // Attempt inserting LOD chunk for already existing real column
        cache.insert_lod_chunk(col, lod_chunk);
        assert_eq!(cache.get_chunk(col).unwrap().get(0, 0).voxel, Voxel::Rock_Basalt);

        // Inserting into empty column should succeed
        let empty_col = IVec2::new(10, 10);
        let mut lod_chunk2 = MapChunk::default();
        lod_chunk2.set(0, 0, lod_pixel);
        cache.insert_lod_chunk(empty_col, lod_chunk2);
        assert_eq!(cache.get_chunk(empty_col).unwrap().get(0, 0).voxel, Voxel::Soil_Grass);
    }
}

