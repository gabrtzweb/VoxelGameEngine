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
        let col = IVec2::new(world_voxel.x >> 4, world_voxel.z >> 4);
        self.mark_dirty(col);
    }

    /// Retrieves cached map data for a chunk column, if it has been explored and generated.
    pub fn get_chunk(&self, chunk_col: IVec2) -> Option<&MapChunk> {
        self.chunks.get(&chunk_col)
    }

    /// Clears all cached map data.
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.dirty_columns.clear();
        self.version = self.version.wrapping_add(1);
    }

    /// Updates up to `max_per_frame` dirty columns by scanning top-down through loaded chunks in `VoxelWorld`.
    /// Budgeted so bulk chunk arrivals never hitch the main thread.
    pub fn update_dirty_columns(&mut self, world: &VoxelWorld) {
        if self.dirty_columns.is_empty() {
            return;
        }

        const MAX_DIRTY_COLUMNS_PER_FRAME: usize = 2;
        let dirty_list: Vec<IVec2> = self
            .dirty_columns
            .iter()
            .take(MAX_DIRTY_COLUMNS_PER_FRAME)
            .copied()
            .collect();
        let mut any_changed = false;

        for col in dirty_list {
            self.dirty_columns.remove(&col);
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

    // 1. Sample terrain at 2x2 stride (64 samples instead of 256 -> 4x faster background LOD generation)
    let mut grid_cols = Vec::with_capacity(64);
    let mut grid_mats = [Voxel::Air; 64];

    for gx in 0..8usize {
        for gz in 0..8usize {
            let wx = chunk_origin_x + (gx as i32) * 2;
            let wz = chunk_origin_z + (gz as i32) * 2;

            let col_data = generator.sample_column(wx, wz);
            let (eff_height, surface_mat) =
                generator.lod_surface_height_and_material(wx, wz, col_data);
            grid_cols.push(col_data);
            grid_mats[gx * 8 + gz] = surface_mat;

            let pixel = if let Some(wl) = col_data.water_level
                && col_data.terrain_height < wl
            {
                let depth = (wl - col_data.terrain_height).clamp(1, 255) as u8;
                let color = super::color::voxel_map_color_at(Voxel::LiquidWater, depth, wx, wz);
                MapPixel {
                    voxel: Voxel::LiquidWater,
                    height: wl as i16,
                    water_depth: depth,
                    color,
                }
            } else {
                let color = super::color::voxel_map_color_at(surface_mat, 0, wx, wz);
                MapPixel {
                    voxel: surface_mat,
                    height: eff_height as i16,
                    water_depth: 0,
                    color,
                }
            };

            let lx0 = gx * 2;
            let lz0 = gz * 2;
            map_chunk.set(lx0, lz0, pixel);
            map_chunk.set(lx0 + 1, lz0, pixel);
            map_chunk.set(lx0, lz0 + 1, pixel);
            map_chunk.set(lx0 + 1, lz0 + 1, pixel);
        }
    }

    // 2. Stamp procedural tree canopies onto the LOD map tile (with 4-block overhang margin)
    const MARGIN: i32 = 4;
    let lod_trees = crate::generation::trees::sample_lod_trees_in_region(
        chunk_origin_x - MARGIN,
        chunk_origin_z - MARGIN,
        chunk_origin_x + chunk_size - 1 + MARGIN,
        chunk_origin_z + chunk_size - 1 + MARGIN,
        generator.seed,
        |wx, wz| {
            let lx = wx - chunk_origin_x;
            let lz = wz - chunk_origin_z;
            if (0..chunk_size).contains(&lx) && (0..chunk_size).contains(&lz) {
                grid_cols[(lx as usize / 2) * 8 + (lz as usize / 2)]
            } else {
                generator.sample_column(wx, wz)
            }
        },
        |wx, wy, wz, col_data| {
            let lx = wx - chunk_origin_x;
            let lz = wz - chunk_origin_z;
            let g = if (0..chunk_size).contains(&lx) && (0..chunk_size).contains(&lz) {
                grid_mats[(lx as usize / 2) * 8 + (lz as usize / 2)]
            } else {
                generator.surface_material_at(wx, wy, wz, col_data)
            };
            if g.is_solid_opaque() && !g.is_fluid() && !g.is_empty() {
                Some(g)
            } else {
                None
            }
        },
    );

    for tree in lod_trees {
        let canopy_voxel = tree.leaf_voxel.unwrap_or(tree.log_voxel);
        let radius = if tree.leaf_voxel.is_some() {
            (tree.canopy_radius * 0.85).round().clamp(1.0, 4.0) as i32
        } else {
            0
        };
        let r2 = radius * radius;
        let top_h = (tree.root_y + tree.trunk_height.round() as i32) as i16;
        let color = super::color::voxel_map_color_at(canopy_voxel, 0, tree.root_x, tree.root_z);

        for dz in -radius..=radius {
            let lz = tree.root_z + dz - chunk_origin_z;
            if !(0..chunk_size).contains(&lz) {
                continue;
            }
            for dx in -radius..=radius {
                let lx = tree.root_x + dx - chunk_origin_x;
                if !(0..chunk_size).contains(&lx) {
                    continue;
                }
                let dist2 = dx * dx + dz * dz;
                if dist2 > r2 {
                    continue;
                }
                // Soften outer corners for organic circular canopies on the map
                if radius >= 2 && dist2 == r2 && ((dx ^ dz ^ tree.root_x) & 1) == 0 {
                    continue;
                }
                let curr = map_chunk.get(lx as usize, lz as usize);
                if top_h >= curr.height {
                    map_chunk.set(
                        lx as usize,
                        lz as usize,
                        MapPixel {
                            voxel: canopy_voxel,
                            height: top_h,
                            water_depth: 0,
                            color,
                        },
                    );
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
                let color =
                    super::color::voxel_map_color_at(Voxel::LiquidWater, depth, world_x, world_z);
                map_chunk.set(
                    lx,
                    lz,
                    MapPixel {
                        voxel: Voxel::LiquidWater,
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
