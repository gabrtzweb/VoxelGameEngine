use bevy::prelude::*;

use super::{
    biome::{BiomeType, ClimateGenerator, ClimateSample},
    caves::{CaveGenerator, CaveNoiseSample},
    strata::StrataGenerator,
    trees::{self, TreeSpecies},
};
use crate::world::{BlockShape, CHUNK_SIZE, CHUNK_VOLUME, Chunk, Voxel};

pub const LOGICAL_BLOCK_VOXELS: i32 = 1;

#[derive(Clone, Copy, Debug)]
pub struct TerrainColumn {
    pub terrain_height: i32,
    pub water_level: Option<i32>,
    pub biome: BiomeType,
    pub is_beach: bool,
    pub is_cliff: bool,
    pub is_underground_river: bool,
    pub climate: ClimateSample,
    pub surface_shape: BlockShape,
    pub shape_orientation: u8,
}

impl Default for TerrainColumn {
    fn default() -> Self {
        Self {
            terrain_height: 0,
            water_level: None,
            biome: BiomeType::Steppe,
            is_beach: false,
            is_cliff: false,
            is_underground_river: false,
            climate: ClimateSample {
                continentalness: 0.0,
                temperature: 0.0,
                humidity: 0.0,
                biome: BiomeType::Steppe,
            },
            surface_shape: BlockShape::Full,
            shape_orientation: 0,
        }
    }
}

#[derive(Resource, Clone, Reflect)]
pub struct TerrainGenerator {
    pub seed: u32,

    // Large-scale terrain.
    pub base_height: f32,
    pub macro_amplitude: f32,
    pub macro_frequency: f32,

    // Smaller terrain details.
    pub detail_amplitude: f32,
    pub detail_frequency: f32,
    pub detail_octaves: u32,
    pub persistence: f32,

    // Water, rivers & oceans.
    pub sea_level: i32,
    pub river_frequency: f32,
    pub river_width: f32,

    // Mountain ridges and rolling hills
    pub mountain_ridge_height: f32,
    pub rolling_hills_amplitude: f32,

    // Advanced Phase 6 procedural systems
    pub climate: ClimateGenerator,
    pub caves: CaveGenerator,
    pub strata: StrataGenerator,

    // Phase 12 procedural vegetation
    pub tree_density: f32,

    // Generation version tracker to signal runtime remeshing when tweaked in inspector
    pub version: u32,
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self {
            seed: 1337,

            // Baseline elevation in voxels
            base_height: 16.0,

            // Broad continental landforms
            macro_amplitude: 14.0,
            macro_frequency: 0.0260,

            // Local terrain relief
            detail_amplitude: 5.4,
            detail_frequency: 0.032,
            detail_octaves: 3,
            persistence: 0.5,

            // Mountain ridges and rolling hills
            mountain_ridge_height: 54.0,
            rolling_hills_amplitude: 5.0,

            // Sea level in voxels
            sea_level: 12,

            river_frequency: 0.0035,
            river_width: 0.040,

            climate: ClimateGenerator::default(),
            caves: CaveGenerator::default(),
            strata: StrataGenerator::default(),

            // Standard tree spawn density multiplier
            tree_density: 1.0,

            version: 0,
        }
    }
}

impl TerrainGenerator {
    pub fn generate_chunk(&self, chunk_coordinate: IVec3) -> Chunk {
        let chunk_origin = chunk_coordinate * CHUNK_SIZE as i32;
        let chunk_min_y = chunk_origin.y;

        let mut columns = [TerrainColumn::default(); CHUNK_SIZE * CHUNK_SIZE];
        let mut maximum_filled_height = i32::MIN;
        let mut minimum_filled_height = i32::MAX;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = chunk_origin.x + x as i32;
                let world_z = chunk_origin.z + z as i32;

                let column = self.sample_column(world_x, world_z);
                columns[column_index(x, z)] = column;

                let col_top = column.terrain_height;

                let filled_height = column.water_level.unwrap_or(col_top).max(col_top);

                maximum_filled_height = maximum_filled_height.max(filled_height);
                minimum_filled_height = minimum_filled_height.min(col_top);
            }
        }

        // Entire chunk is above terrain, water, AND any potential tree canopies
        if chunk_min_y > maximum_filled_height + 16 {
            return Chunk::filled(Voxel::Air);
        }

        let mut voxels = vec![Voxel::Air; CHUNK_VOLUME];
        let mut chunk_shapes: Vec<(usize, BlockShape, u8)> = Vec::new();
        let mut has_solid_voxels = false;

        if chunk_min_y <= maximum_filled_height {
            has_solid_voxels = true;
            let chunk_caves = self.caves.build_chunk_sampler(chunk_origin, self.seed);

            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let column = columns[column_index(x, z)];

                    for y in 0..CHUNK_SIZE {
                        let world_y = chunk_origin.y + y as i32;
                        let world_x = chunk_origin.x + x as i32;
                        let world_z = chunk_origin.z + z as i32;

                        let cave_sample = chunk_caves.sample(x, y, z);
                        let voxel =
                            self.voxel_at_sampled(column, world_x, world_y, world_z, cave_sample);
                        if voxel != Voxel::Air {
                            let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                            voxels[idx] = voxel;

                            if world_y == column.terrain_height
                                && column.surface_shape != BlockShape::Full
                            {
                                chunk_shapes.push((
                                    idx,
                                    column.surface_shape,
                                    column.shape_orientation,
                                ));
                            }
                        }
                    }
                }
            }

            // Cave interior shaping: natural ledges, slabs, and wall transitions
            let get_v = |voxels_buf: &[Voxel], lx: i32, ly: i32, lz: i32| -> Voxel {
                if lx >= 0
                    && lx < CHUNK_SIZE as i32
                    && ly >= 0
                    && ly < CHUNK_SIZE as i32
                    && lz >= 0
                    && lz < CHUNK_SIZE as i32
                {
                    voxels_buf[lx as usize
                        + lz as usize * CHUNK_SIZE
                        + ly as usize * CHUNK_SIZE * CHUNK_SIZE]
                } else {
                    let wx = chunk_origin.x + lx;
                    let wy = chunk_origin.y + ly;
                    let wz = chunk_origin.z + lz;
                    let col = self.sample_column(wx, wz);
                    self.voxel_at(col, wx, wy, wz)
                }
            };

            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let column = columns[column_index(x, z)];

                    for y in 0..CHUNK_SIZE {
                        let world_y = chunk_origin.y + y as i32;
                        // Subterranean spaces strictly below surface terrain height
                        if world_y >= column.terrain_height {
                            continue;
                        }

                        let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                        let voxel = voxels[idx];
                        if !voxel.is_collidable() || voxel == Voxel::Rock_Dreadstone {
                            continue;
                        }

                        let world_x = chunk_origin.x + x as i32;
                        let world_z = chunk_origin.z + z as i32;

                        let above = get_v(&voxels, x as i32, y as i32 + 1, z as i32);
                        if above == Voxel::Air {
                            // Cave floor / shelf: use bottom slabs on gentle 1-block steps
                            let d_west = if !get_v(&voxels, x as i32 - 1, y as i32, z as i32)
                                .is_collidable()
                            {
                                if get_v(&voxels, x as i32 - 1, y as i32 - 1, z as i32)
                                    .is_collidable()
                                {
                                    -1
                                } else {
                                    -2
                                }
                            } else {
                                0
                            };

                            let d_east = if !get_v(&voxels, x as i32 + 1, y as i32, z as i32)
                                .is_collidable()
                            {
                                if get_v(&voxels, x as i32 + 1, y as i32 - 1, z as i32)
                                    .is_collidable()
                                {
                                    -1
                                } else {
                                    -2
                                }
                            } else {
                                0
                            };

                            let d_north = if !get_v(&voxels, x as i32, y as i32, z as i32 - 1)
                                .is_collidable()
                            {
                                if get_v(&voxels, x as i32, y as i32 - 1, z as i32 - 1)
                                    .is_collidable()
                                {
                                    -1
                                } else {
                                    -2
                                }
                            } else {
                                0
                            };

                            let d_south = if !get_v(&voxels, x as i32, y as i32, z as i32 + 1)
                                .is_collidable()
                            {
                                if get_v(&voxels, x as i32, y as i32 - 1, z as i32 + 1)
                                    .is_collidable()
                                {
                                    -1
                                } else {
                                    -2
                                }
                            } else {
                                0
                            };

                            let has_cliff =
                                d_west <= -2 || d_east <= -2 || d_north <= -2 || d_south <= -2;
                            if !has_cliff {
                                let lower_count = (if d_west == -1 { 1 } else { 0 })
                                    + (if d_east == -1 { 1 } else { 0 })
                                    + (if d_north == -1 { 1 } else { 0 })
                                    + (if d_south == -1 { 1 } else { 0 });

                                if lower_count == 1 || lower_count == 2 {
                                    // Slabs for cave floor transitions (avoids excessive staircases)
                                    chunk_shapes.push((idx, BlockShape::Slab, 0));
                                }
                            }
                        } else {
                            // Cave wall shaping: soften wall bases, arched ceiling junctions, vertical wall recesses, and chamfered corners
                            let air_west =
                                get_v(&voxels, x as i32 - 1, y as i32, z as i32) == Voxel::Air;
                            let air_east =
                                get_v(&voxels, x as i32 + 1, y as i32, z as i32) == Voxel::Air;
                            let air_north =
                                get_v(&voxels, x as i32, y as i32, z as i32 - 1) == Voxel::Air;
                            let air_south =
                                get_v(&voxels, x as i32, y as i32, z as i32 + 1) == Voxel::Air;

                            let wall_openings =
                                air_west as u8 + air_east as u8 + air_north as u8 + air_south as u8;
                            let wall_noise = crate::core::noise::gradient_noise_3d(
                                world_x as f32 * 0.18,
                                world_y as f32 * 0.18,
                                world_z as f32 * 0.18,
                                self.seed.wrapping_add(44_221),
                            );

                            if wall_openings == 1 {
                                let (
                                    air_dx,
                                    air_dz,
                                    stair_orient,
                                    inv_stair_orient,
                                    vert_slab_orient,
                                ) = if air_west {
                                    (-1, 0, 0, 4, 5) // air is -X, slab attached to +X rock wall -> orient 5
                                } else if air_east {
                                    (1, 0, 1, 5, 4) // air is +X, slab attached to -X rock wall -> orient 4
                                } else if air_north {
                                    (0, -1, 2, 6, 3) // air is -Z, slab attached to +Z rock wall -> orient 3
                                } else {
                                    (0, 1, 3, 7, 2) // air is +Z, slab attached to -Z rock wall -> orient 2
                                };

                                let below_cave_floor = get_v(
                                    &voxels,
                                    x as i32 + air_dx,
                                    y as i32 - 1,
                                    z as i32 + air_dz,
                                )
                                .is_collidable();
                                let above_cave_roof = get_v(
                                    &voxels,
                                    x as i32 + air_dx,
                                    y as i32 + 1,
                                    z as i32 + air_dz,
                                )
                                .is_collidable();

                                if below_cave_floor && wall_noise > 0.10 {
                                    // Wall base: rock footing using stairs or slabs
                                    if wall_noise > 0.45 {
                                        chunk_shapes.push((idx, BlockShape::Stair, stair_orient));
                                    } else {
                                        chunk_shapes.push((idx, BlockShape::Slab, 0));
                                    }
                                } else if above_cave_roof && wall_noise > 0.25 {
                                    // Wall top: arched ceiling overhang
                                    if wall_noise > 0.50 {
                                        chunk_shapes.push((
                                            idx,
                                            BlockShape::Stair,
                                            inv_stair_orient,
                                        ));
                                    } else {
                                        chunk_shapes.push((idx, BlockShape::Slab, 1));
                                    }
                                } else if (0.35..0.68).contains(&wall_noise) {
                                    // Mid-wall: vertical slabs create rocky recesses and wall relief in caves & ravines!
                                    chunk_shapes.push((idx, BlockShape::Slab, vert_slab_orient));
                                }
                            } else if wall_openings == 2 {
                                // Outer turning corners in caves and ravines: side-aligned column slabs chamfer sharp 90-degree rock edges
                                let corner_orient = if air_west && air_north {
                                    Some(4) // Solid corner at MaxX, MaxZ (+X, +Z)
                                } else if air_east && air_north {
                                    Some(3) // Solid corner at MinX, MaxZ (-X, +Z)
                                } else if air_west && air_south {
                                    Some(2) // Solid corner at MaxX, MinZ (+X, -Z)
                                } else if air_east && air_south {
                                    Some(1) // Solid corner at MinX, MinZ (-X, -Z)
                                } else {
                                    None // Opposite walls (corridor)
                                };

                                if let Some(orient) = corner_orient {
                                    if wall_noise > 0.15 {
                                        chunk_shapes.push((idx, BlockShape::Column, orient));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Phase 12: Procedural Tree & Cactus Generation
        // Only evaluate trees if this chunk intersects the surface layer where trees can exist
        let chunk_max_y = chunk_min_y + CHUNK_SIZE as i32 - 1;
        if self.tree_density > 0.0 && chunk_max_y >= minimum_filled_height - 6 {
            let min_cell_x = (chunk_origin.x - 5).div_euclid(5);
            let max_cell_x = (chunk_origin.x + 16).div_euclid(5);
            let min_cell_z = (chunk_origin.z - 5).div_euclid(5);
            let max_cell_z = (chunk_origin.z + 16).div_euclid(5);

            for cell_z in min_cell_z..=max_cell_z {
                for cell_x in min_cell_x..=max_cell_x {
                    let hash = trees::hash_tree_cell(cell_x, cell_z, self.seed);
                    let offset_x = (hash % 3) as i32 + 1;
                    let offset_z = ((hash >> 2) % 3) as i32 + 1;
                    let tx = cell_x * 5 + offset_x;
                    let tz = cell_z * 5 + offset_z;

                    // Quick horizontal bounds check: does canopy touch this chunk?
                    if tx + 2 < chunk_origin.x
                        || tx - 2 >= chunk_origin.x + CHUNK_SIZE as i32
                        || tz + 2 < chunk_origin.z
                        || tz - 2 >= chunk_origin.z + CHUNK_SIZE as i32
                    {
                        continue;
                    }

                    let col = self.sample_column(tx, tz);
                    let Some((species, base_prob)) = trees::biome_tree_profile(col.biome) else {
                        continue;
                    };

                    let roll = ((hash >> 4) & 0xFFFF) as f32 / 65535.0;
                    if roll >= base_prob * self.tree_density {
                        continue;
                    }

                    if col.is_cliff || col.surface_shape == BlockShape::Stair {
                        continue;
                    }

                    let is_swamp_tree = species == TreeSpecies::Mangrove
                        || (species == TreeSpecies::Pine && col.biome == BiomeType::CypressSwamp);

                    let ty = if let Some(water_level) = col.water_level {
                        if !is_swamp_tree {
                            continue;
                        }
                        if water_level - col.terrain_height > 2 {
                            continue;
                        }
                        col.terrain_height
                    } else {
                        col.terrain_height
                    };

                    // Vertical bounds check: tree occupies y in [ty + 1, ty + 10]
                    if ty + 1 > chunk_max_y || ty + 10 < chunk_min_y {
                        continue;
                    }

                    let surface_mat = self.surface_material_at(tx, col.terrain_height, tz, col);
                    if !trees::is_soil_valid_for_species(species, surface_mat) {
                        continue;
                    }

                    // Cactus specific clearance: no solid blocks directly adjacent to base
                    if species == TreeSpecies::Cactus {
                        let col_e = self.sample_column(tx + 1, tz);
                        let col_w = self.sample_column(tx - 1, tz);
                        let col_s = self.sample_column(tx, tz + 1);
                        let col_n = self.sample_column(tx, tz - 1);
                        if col_e.terrain_height > ty
                            || col_w.terrain_height > ty
                            || col_s.terrain_height > ty
                            || col_n.terrain_height > ty
                        {
                            continue;
                        }
                    }

                    trees::generate_tree_voxels(
                        tx,
                        ty,
                        tz,
                        species,
                        col.biome,
                        hash,
                        |bx, by, bz, voxel| {
                            if bx >= chunk_origin.x
                                && bx < chunk_origin.x + CHUNK_SIZE as i32
                                && by >= chunk_origin.y
                                && by < chunk_origin.y + CHUNK_SIZE as i32
                                && bz >= chunk_origin.z
                                && bz < chunk_origin.z + CHUNK_SIZE as i32
                            {
                                let lx = (bx - chunk_origin.x) as usize;
                                let ly = (by - chunk_origin.y) as usize;
                                let lz = (bz - chunk_origin.z) as usize;
                                let idx = lx + lz * CHUNK_SIZE + ly * CHUNK_SIZE * CHUNK_SIZE;

                                let current = voxels[idx];
                                let is_trunk_or_cactus = trees::is_trunk(voxel);

                                if is_trunk_or_cactus {
                                    if current == Voxel::Air
                                        || current == Voxel::Liquid_Water
                                        || trees::is_leaves(current)
                                    {
                                        voxels[idx] = voxel;
                                        has_solid_voxels = true;
                                    }
                                } else if current == Voxel::Air {
                                    voxels[idx] = voxel;
                                    has_solid_voxels = true;
                                }
                            }
                        },
                    );

                    // Convert surface soil directly underneath the trunk to dirt if grass
                    if ty >= chunk_min_y
                        && ty <= chunk_max_y
                        && tx >= chunk_origin.x
                        && tx < chunk_origin.x + CHUNK_SIZE as i32
                        && tz >= chunk_origin.z
                        && tz < chunk_origin.z + CHUNK_SIZE as i32
                    {
                        let lx = (tx - chunk_origin.x) as usize;
                        let ly = (ty - chunk_origin.y) as usize;
                        let lz = (tz - chunk_origin.z) as usize;
                        let idx = lx + lz * CHUNK_SIZE + ly * CHUNK_SIZE * CHUNK_SIZE;
                        if matches!(
                            voxels[idx],
                            Voxel::Soil_Grass | Voxel::Soil_Peat_Grass | Voxel::Soil_Silt_Grass
                        ) {
                            voxels[idx] = Voxel::Soil_Dirt;
                        }
                    }
                }
            }
        }

        if !has_solid_voxels {
            return Chunk::filled(Voxel::Air);
        }

        let mut chunk = Chunk::from_voxels(voxels);
        for (idx, shape, orientation) in chunk_shapes {
            let (lx, ly, lz) = Chunk::index_to_xyz(idx);
            chunk.set_shape(lx, ly, lz, shape, orientation);
        }

        chunk
    }

    pub fn effective_sea_level(&self) -> i32 {
        logical_block_top(self.sea_level)
    }

    pub fn continuous_height_and_biome(
        &self,
        world_x: i32,
        world_z: i32,
    ) -> (f32, BiomeType, bool) {
        let logical_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS);
        let logical_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS);
        let sample_x = logical_block_sample_position(logical_x);
        let sample_z = logical_block_sample_position(logical_z);

        let mut climate = self.climate.sample_dithered(sample_x, sample_z, self.seed);
        let sea_level = self.effective_sea_level();

        let mut raw_height = self.natural_height_at(world_x as f32, world_z as f32, &climate);

        let (river_factor, is_river_path) = self.river_sample(sample_x, sample_z);
        let is_inland = climate.continentalness >= -0.05;
        let is_river = is_river_path && is_inland && river_factor > 0.05;
        let mut is_underground_river = false;

        if is_river {
            let initial_height = raw_height.round() as i32;
            if initial_height > sea_level + 14 {
                is_underground_river = true;
            } else {
                let river_bed = (sea_level - 4) as f32;
                let target_height = lerp(raw_height, river_bed, (river_factor * 1.25).min(1.0));
                raw_height = raw_height.min(target_height);
                if climate.continentalness < 0.15 {
                    climate.biome = BiomeType::BrackishEstuary;
                }
            }
        }

        let is_beach = self.is_beach_at(logical_x, logical_z, climate.biome, sea_level, &climate);
        let final_biome = if is_beach && !is_river {
            BiomeType::Beach
        } else {
            climate.biome
        };

        (raw_height, final_biome, is_underground_river)
    }

    pub fn sample_column(&self, world_x: i32, world_z: i32) -> TerrainColumn {
        let sea_level = self.effective_sea_level();
        let (raw_height, final_biome, is_underground_river) =
            self.continuous_height_and_biome(world_x, world_z);

        let terrain_height = raw_height.round() as i32;
        let mut surface_shape = BlockShape::Full;
        let mut shape_orientation = 0u8;

        let water_level = if terrain_height < sea_level {
            Some(sea_level)
        } else {
            None
        };

        let is_beach = final_biome == BiomeType::Beach;
        let mut is_cliff = false;

        // Transitions on dry land above water level (terrain at or below sea_level remains strictly full blocks)
        if water_level.is_none() && terrain_height >= sea_level + 1 {
            let get_effective_height = |(h, _, _): (f32, BiomeType, bool)| -> i32 {
                let th = h.round() as i32;
                if th < sea_level { sea_level } else { th }
            };

            let h_east =
                get_effective_height(self.continuous_height_and_biome(world_x + 1, world_z));
            let h_west =
                get_effective_height(self.continuous_height_and_biome(world_x - 1, world_z));
            let h_south =
                get_effective_height(self.continuous_height_and_biome(world_x, world_z + 1));
            let h_north =
                get_effective_height(self.continuous_height_and_biome(world_x, world_z - 1));

            let d_east = h_east - terrain_height;
            let d_west = h_west - terrain_height;
            let d_south = h_south - terrain_height;
            let d_north = h_north - terrain_height;

            let has_cliff = d_east <= -2 || d_west <= -2 || d_south <= -2 || d_north <= -2;
            let max_delta = d_east.abs().max(d_west.abs()).max(d_south.abs()).max(d_north.abs());
            is_cliff = has_cliff || max_delta >= 2;

            // Retain solid blocks on sections of higher terrain for blocky cliffs and stepped terraces
            let edge_noise = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.08,
                world_z as f32 * 0.08,
                self.seed.wrapping_add(88_991),
            );

            let is_high_terrain = terrain_height >= sea_level + 7;
            let keep_solid = if is_high_terrain {
                edge_noise > 0.05
            } else {
                edge_noise > 0.42
            };

            if !has_cliff && !keep_solid {
                let lower_count = (if d_west == -1 { 1 } else { 0 })
                    + (if d_east == -1 { 1 } else { 0 })
                    + (if d_north == -1 { 1 } else { 0 })
                    + (if d_south == -1 { 1 } else { 0 });

                if lower_count == 1 {
                    // Surface stairs restored at a natural frequency (~35%), slabs for remainder
                    let stair_hash = ((world_x.wrapping_mul(374_761_393)
                        ^ world_z.wrapping_mul(668_265_263)
                        ^ (self.seed as i32))
                        .abs()
                        % 100) as f32
                        / 100.0;

                    if stair_hash < 0.35 {
                        surface_shape = BlockShape::Stair;
                        // Orient step towards the higher side so it ascends away from the lower neighbor
                        shape_orientation = if d_east == -1 {
                            1 // Drop to East (+X): step is at -X
                        } else if d_west == -1 {
                            0 // Drop to West (-X): step is at +X
                        } else if d_south == -1 {
                            3 // Drop to South (+Z): step is at -Z
                        } else {
                            2 // Drop to North (-Z): step is at +Z
                        };
                    } else {
                        surface_shape = BlockShape::Slab;
                        shape_orientation = 0; // Bottom Slab
                    }
                } else if lower_count == 2 {
                    // Corner drops use clean bottom slabs
                    surface_shape = BlockShape::Slab;
                    shape_orientation = 0; // Bottom Slab
                }
            }
        }

        TerrainColumn {
            terrain_height,
            water_level,
            biome: final_biome,
            is_beach,
            is_cliff,
            is_underground_river,
            climate: self.climate.sample_dithered(
                logical_block_sample_position(world_x.div_euclid(LOGICAL_BLOCK_VOXELS)),
                logical_block_sample_position(world_z.div_euclid(LOGICAL_BLOCK_VOXELS)),
                self.seed,
            ),
            surface_shape,
            shape_orientation,
        }
    }

    pub fn river_sample(&self, world_x: f32, world_z: f32) -> (f32, bool) {
        let river_noise = fractal_noise(
            world_x,
            world_z,
            self.river_frequency,
            3,
            0.5,
            self.seed.wrapping_add(555_123),
        );
        let dist = river_noise.abs();
        if dist < self.river_width {
            let depth_factor = 1.0 - (dist / self.river_width);
            (depth_factor, true)
        } else {
            (0.0, false)
        }
    }

    fn is_beach_at(
        &self,
        logical_x: i32,
        logical_z: i32,
        biome: BiomeType,
        sea_level: i32,
        climate: &ClimateSample,
    ) -> bool {
        // Exclude sheer cliffs, ocean trenches, rocky crags, tidal flats, and frozen/karst mountain peaks
        if matches!(
            biome,
            BiomeType::ChalkCliffs
                | BiomeType::CoastalCrags
                | BiomeType::AbyssalTrench
                | BiomeType::DeepOcean
                | BiomeType::TemperateOcean
                | BiomeType::BrackishEstuary
                | BiomeType::TidalMudflats
                | BiomeType::GlacialPeaks
                | BiomeType::FrozenCaldera
                | BiomeType::KarstPeaks
                | BiomeType::JaggedCrags
                | BiomeType::ShaleBarrens
                | BiomeType::VolcanicFields
        ) {
            return false;
        }

        let rep_height = self.logical_terrain_height_at(logical_x, logical_z, climate);
        let beach_noise = crate::core::noise::gradient_noise_2d(
            logical_x as f32 * 0.12,
            logical_z as f32 * 0.12,
            self.seed.wrapping_add(88_411),
        ) * 1.5;
        let max_beach_height = (sea_level as f32 + 2.5 + beach_noise).round() as i32;
        rep_height >= sea_level - 6 && rep_height <= max_beach_height
    }

    pub fn surface_material_at(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        column: TerrainColumn,
    ) -> Voxel {
        let dither_noise = crate::core::noise::gradient_noise_2d(
            world_x as f32 * 0.15,
            world_z as f32 * 0.15,
            self.seed.wrapping_add(45_678),
        );

        // 1. Submerged terrain (underwater) floor materials
        let is_submerged = column.water_level.is_some_and(|wl| world_y <= wl);
        if is_submerged {
            return match column.biome {
                BiomeType::AbyssalTrench => {
                    if dither_noise > 0.0 {
                        Voxel::Rock_Obsidian
                    } else {
                        Voxel::Rock_Pitchstone
                    }
                }
                BiomeType::DeepOcean => {
                    if dither_noise > 0.05 {
                        Voxel::Soil_Gravel
                    } else {
                        Voxel::Rock_Andesite
                    }
                }
                BiomeType::TemperateOcean => Voxel::Soil_White_Sand,
                BiomeType::BrackishEstuary => {
                    if dither_noise > 0.0 {
                        Voxel::Soil_Silt
                    } else {
                        Voxel::Soil_Mud
                    }
                }
                BiomeType::TidalMudflats => {
                    if dither_noise > 0.0 {
                        Voxel::Soil_Packed_Mud
                    } else {
                        Voxel::Soil_Clay
                    }
                }
                BiomeType::CypressSwamp
                | BiomeType::MangroveSwamp
                | BiomeType::Marshland
                | BiomeType::FungalBog
                | BiomeType::WeepingBayou => Voxel::Soil_Mud,
                BiomeType::TarPits => Voxel::Soil_Black_Sand,
                BiomeType::SludgeWastes => Voxel::Soil_Scorched_Sand,
                BiomeType::VolcanicPlains | BiomeType::VolcanicFields => Voxel::Rock_Basalt,
                BiomeType::FrozenCaldera => Voxel::Frost_Black_Ice,
                BiomeType::Beach => {
                    if column.climate.temperature > 0.15 {
                        Voxel::Soil_White_Sand
                    } else {
                        Voxel::Soil_Sand
                    }
                }
                _ => {
                    if column.climate.temperature < -0.15 {
                        Voxel::Soil_Gravel
                    } else {
                        Voxel::Soil_Sand
                    }
                }
            };
        }

        // 2. Coastal Beach Surface
        if column.is_beach {
            let beach_warmth = column.climate.temperature + dither_noise * 0.10;
            return if beach_warmth > 0.15 {
                Voxel::Soil_White_Sand
            } else {
                Voxel::Soil_Sand
            };
        }

        // 3. High alpine elevation snowline: cold/temperate mountains above y >= 50 receive snowcaps
        let is_warm_or_volcanic = matches!(
            column.biome,
            BiomeType::ScorchedWastes
                | BiomeType::VolcanicFields
                | BiomeType::VolcanicPlains
                | BiomeType::Badlands
                | BiomeType::DuneDesert
                | BiomeType::WhiteDesert
                | BiomeType::PaintedDesert
                | BiomeType::TropicalRainforest
                | BiomeType::Oasis
                | BiomeType::TarPits
                | BiomeType::SludgeWastes
        );

        if !is_warm_or_volcanic && column.climate.temperature < 0.10 {
            let snowline_jitter = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.05,
                world_z as f32 * 0.05,
                self.seed.wrapping_add(91_111),
            ) * 3.0;
            let snowline = 50.0 + snowline_jitter;

            if world_y as f32 >= snowline {
                let slope_noise = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.15,
                    world_z as f32 * 0.15,
                    self.seed.wrapping_add(82_222),
                );
                if world_y as f32 >= snowline + 8.0 && slope_noise > 0.35 {
                    return column.biome.config().primary_stone;
                }
                return Voxel::Soil_Snow;
            }
        }

        // 4. Biome Surface Block Selection with organic dual-surface dithering
        match column.biome {
            // Forests & Woodlands
            BiomeType::AncientWeald => {
                if dither_noise > 0.05 {
                    Voxel::Soil_Mulch
                } else {
                    Voxel::Soil_Grass
                }
            }
            BiomeType::AutumnalForest => Voxel::Soil_Silt_Grass,
            BiomeType::BirchCopse => Voxel::Soil_Grass,
            BiomeType::BlossomGrove => Voxel::Soil_Grass,
            BiomeType::BorealTaiga => {
                if dither_noise > 0.0 {
                    Voxel::Soil_Snowy_Peat
                } else {
                    Voxel::Soil_Snowy_Grass
                }
            }
            BiomeType::DeadwoodThicket => {
                if dither_noise > 0.0 {
                    Voxel::Soil_Ash
                } else {
                    Voxel::Soil_Black_Sand
                }
            }
            BiomeType::TropicalRainforest => {
                if dither_noise > -0.10 {
                    Voxel::Soil_Moss
                } else {
                    Voxel::Soil_Mud
                }
            }
            BiomeType::YewGrove => Voxel::Soil_Peat_Mulch,

            // Plains & Open Lands
            BiomeType::AcaciaSavanna => {
                if dither_noise > 0.15 {
                    Voxel::Soil_Packed_Dirt
                } else {
                    Voxel::Soil_Grass
                }
            }
            BiomeType::Heath => Voxel::Soil_Silt_Grass,
            BiomeType::Moorland => Voxel::Soil_Peat_Grass,
            BiomeType::OutbackScrubland => {
                if dither_noise > 0.0 {
                    Voxel::Soil_Scorched_Red_Sand
                } else {
                    Voxel::Soil_Packed_Mud
                }
            }
            BiomeType::PermafrostSteppe => Voxel::Soil_Snowy_Silt,
            BiomeType::SnowyTundra => Voxel::Soil_Snowy_Grass,
            BiomeType::Steppe => Voxel::Soil_Grass,
            BiomeType::VolcanicPlains => {
                if dither_noise > 0.0 {
                    Voxel::Soil_Ash
                } else {
                    Voxel::Soil_Scorched_Black_Sand
                }
            }

            // Wetlands & Swamps
            BiomeType::CypressSwamp => Voxel::Soil_Mud,
            BiomeType::FungalBog => Voxel::Soil_Red_Moss,
            BiomeType::MangroveSwamp => Voxel::Soil_Mud,
            BiomeType::Marshland => {
                if dither_noise > 0.10 {
                    Voxel::Soil_Packed_Mud
                } else {
                    Voxel::Soil_Mud
                }
            }
            BiomeType::PeatBog => Voxel::Soil_Peat_Grass,
            BiomeType::SludgeWastes => Voxel::Soil_Scorched_Sand,
            BiomeType::TarPits => Voxel::Soil_Black_Sand,
            BiomeType::WeepingBayou => Voxel::Soil_Silt_Mulch,

            BiomeType::Badlands => {
                if column.is_cliff {
                    if (world_y.rem_euclid(8)) < 3 {
                        Voxel::Rock_Terracotta
                    } else {
                        Voxel::Rock_Red_Sandstone
                    }
                } else {
                    Voxel::Soil_Red_Sand
                }
            },
            BiomeType::DuneDesert => Voxel::Soil_Sand,
            BiomeType::Oasis => Voxel::Soil_Grass,
            BiomeType::PaintedDesert => {
                if dither_noise > 0.0 {
                    Voxel::Soil_White_Sand
                } else {
                    Voxel::Soil_Sand
                }
            }
            BiomeType::RockyScrubland => {
                if dither_noise > 0.05 {
                    Voxel::Soil_Packed_Dirt
                } else {
                    Voxel::Soil_Sand
                }
            }
            BiomeType::ScorchedWastes => Voxel::Soil_Scorched_Sand,
            BiomeType::WhiteDesert => Voxel::Soil_White_Sand,
            BiomeType::WindsweptCanyons => Voxel::Soil_Sand,

            // Mountain Biomes
            BiomeType::AlpineTundra => {
                if dither_noise > 0.30 {
                    Voxel::Rock_Diorite
                } else {
                    Voxel::Soil_Snowy_Grass
                }
            }
            BiomeType::FrozenCaldera => Voxel::Frost_Black_Ice,
            BiomeType::GlacialPeaks => Voxel::Soil_Snow,
            BiomeType::JaggedCrags => {
                if column.is_cliff {
                    if dither_noise > 0.0 {
                        Voxel::Rock_Gabbro
                    } else {
                        Voxel::Rock_Andesite
                    }
                } else if dither_noise > 0.25 {
                    Voxel::Rock_Gabbro
                } else {
                    Voxel::Soil_Grass
                }
            }
            BiomeType::KarstPeaks => {
                if column.is_cliff || dither_noise > 0.15 {
                    Voxel::Rock_Karst
                } else {
                    Voxel::Soil_Grass
                }
            }
            BiomeType::ScreeSlopes => Voxel::Soil_Gravel,
            BiomeType::ShaleBarrens => {
                if column.is_cliff {
                    if dither_noise > 0.15 {
                        Voxel::Rock_Slate
                    } else {
                        Voxel::Cobbled_Slate
                    }
                } else if dither_noise > 0.20 {
                    Voxel::Rock_Slate
                } else {
                    Voxel::Soil_Silt_Grass
                }
            }
            BiomeType::VolcanicFields => {
                if dither_noise > 0.05 {
                    Voxel::Rock_Basalt
                } else {
                    Voxel::Rock_Scoria
                }
            }

            // Coastal & Aquatic Biomes
            BiomeType::AbyssalTrench => {
                if dither_noise > 0.0 {
                    Voxel::Rock_Obsidian
                } else {
                    Voxel::Rock_Pitchstone
                }
            }
            BiomeType::Beach => {
                if column.climate.temperature > 0.15 {
                    Voxel::Soil_White_Sand
                } else {
                    Voxel::Soil_Sand
                }
            }
            BiomeType::BrackishEstuary => {
                if dither_noise > 0.0 {
                    Voxel::Soil_Silt
                } else {
                    Voxel::Soil_Mud
                }
            }
            BiomeType::ChalkCliffs => {
                if column.is_cliff {
                    Voxel::Rock_Chalk
                } else {
                    Voxel::Soil_Grass
                }
            }
            BiomeType::CoastalCrags => {
                if column.is_cliff {
                    Voxel::Rock_Porphyry
                } else if dither_noise > 0.10 {
                    Voxel::Soil_Grass
                } else if dither_noise > -0.15 {
                    Voxel::Soil_Gravel
                } else {
                    Voxel::Cobbled_Porphyry
                }
            }
            BiomeType::DeepOcean => {
                if dither_noise > 0.0 {
                    Voxel::Soil_Gravel
                } else {
                    Voxel::Rock_Andesite
                }
            }
            BiomeType::TemperateOcean => Voxel::Soil_White_Sand,
            BiomeType::TidalMudflats => {
                if dither_noise > 0.0 {
                    Voxel::Soil_Packed_Mud
                } else {
                    Voxel::Soil_Clay
                }
            }
        }
    }

    fn voxel_at_sampled(
        &self,
        column: TerrainColumn,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        cave_sample: CaveNoiseSample,
    ) -> Voxel {
        // Underground mountain river tunnel
        if column.is_underground_river {
            let river_floor = self.effective_sea_level() - 4;
            let river_roof = self.effective_sea_level() + 5;
            if world_y >= river_floor && world_y <= river_roof {
                if world_y <= self.effective_sea_level() {
                    return Voxel::Liquid_Water;
                } else {
                    return Voxel::Air;
                }
            }
        }

        if world_y > column.terrain_height {
            if let Some(water_level) = column.water_level
                && world_y <= water_level
            {
                return Voxel::Liquid_Water;
            }

            return Voxel::Air;
        }

        // Absolute bedrock floor: bottom layers of the world are strictly solid Dreadstone
        if world_y <= self.strata.bedrock_min_block_y {
            return Voxel::Rock_Dreadstone;
        }

        let is_underwater =
            column.water_level.is_some() || column.terrain_height <= self.effective_sea_level() + 2;

        if self.caves.is_cave_sampled(
            world_x,
            world_y,
            world_z,
            cave_sample,
            column.terrain_height,
            is_underwater,
            self.effective_sea_level(),
            self.seed,
        ) {
            return self
                .caves
                .cave_voxel_sampled(world_y, cave_sample, self.effective_sea_level());
        }

        let depth = column.terrain_height - world_y;
        if depth <= 0 {
            return self.surface_material_at(world_x, world_y, world_z, column);
        }

        let logical_depth = depth / LOGICAL_BLOCK_VOXELS;
        let biome_cfg = column.biome.config();

        if column.is_beach && logical_depth <= 3 {
            return if column.climate.temperature > 0.15 {
                Voxel::Soil_White_Sand
            } else {
                Voxel::Soil_Sand
            };
        }

        if logical_depth <= 2 {
            let surface = self.surface_material_at(world_x, column.terrain_height, world_z, column);
            if matches!(
                surface,
                Voxel::Soil_Sand
                    | Voxel::Soil_Red_Sand
                    | Voxel::Soil_White_Sand
                    | Voxel::Soil_Black_Sand
                    | Voxel::Soil_Scorched_Sand
                    | Voxel::Soil_Scorched_Red_Sand
                    | Voxel::Soil_Scorched_Black_Sand
                    | Voxel::Soil_Gravel
                    | Voxel::Soil_Mud
                    | Voxel::Soil_Silt
                    | Voxel::Soil_Ash
                    | Voxel::Soil_Snow
                    | Voxel::Frost_Black_Ice
            ) {
                return surface;
            }
        }

        self.strata.solid_voxel_at(
            world_x,
            world_y,
            world_z,
            logical_depth,
            &biome_cfg,
            self.seed,
        )
    }

    #[allow(dead_code)]
    fn voxel_at(&self, column: TerrainColumn, world_x: i32, world_y: i32, world_z: i32) -> Voxel {
        let sample = self.caves.sample_noise_point(
            world_x as f32,
            world_y as f32,
            world_z as f32,
            self.seed,
        );
        self.voxel_at_sampled(column, world_x, world_y, world_z, sample)
    }

    #[allow(dead_code)]
    pub fn surface_voxel(&self, column: TerrainColumn, world_y: i32) -> Voxel {
        let depth = column.terrain_height - world_y;
        if depth <= 0 {
            return self.surface_material_at(0, world_y, 0, column);
        }

        let logical_depth = depth / LOGICAL_BLOCK_VOXELS;
        if column.is_beach && logical_depth <= 3 {
            return Voxel::Soil_Sand;
        }

        let biome_cfg = column.biome.config();
        self.strata
            .solid_voxel_at(0, world_y, 0, logical_depth, &biome_cfg, self.seed)
    }

    fn logical_terrain_height_at(
        &self,
        logical_x: i32,
        logical_z: i32,
        climate: &ClimateSample,
    ) -> i32 {
        let sample_x = logical_block_sample_position(logical_x);
        let sample_z = logical_block_sample_position(logical_z);

        logical_block_top(self.terrain_height_at(sample_x, sample_z, climate))
    }

    pub fn continental_elevation(continentalness: f32) -> f32 {
        const SPLINE_NODES: [(f32, f32); 10] = [
            (-1.00, -36.0), // Deep abyssal trench
            (-0.55, -26.0), // Deep ocean basin
            (-0.25, -18.0), // Open ocean floor
            (-0.08, -8.0),  // Continental shelf / shallow coastal waters
            (0.00, -4.0),   // Coastline / beach (aligns with sea_level = 12 at base_height = 16)
            (0.10, 5.0),    // Low coastal plains (y = 21)
            (0.22, 16.0),   // Inland rolling plains (y = 32)
            (0.35, 42.0),   // Highlands & foothills (y = 58)
            (0.50, 84.0),   // Rugged mountain chains (y = 100)
            (0.70, 134.0),  // Grand alpine peaks (y = 150)
        ];

        let c = continentalness.clamp(-1.0, 1.0);

        for i in 0..(SPLINE_NODES.len() - 1) {
            let (c0, y0) = SPLINE_NODES[i];
            let (c1, y1) = SPLINE_NODES[i + 1];
            if c <= c1 {
                let t = (c - c0) / (c1 - c0);
                let t_smooth = t * t * (3.0 - 2.0 * t);
                return y0 + (y1 - y0) * t_smooth;
            }
        }

        SPLINE_NODES[SPLINE_NODES.len() - 1].1
    }

    pub fn continental_roughness(continentalness: f32) -> f32 {
        if continentalness < 0.0 {
            0.55
        } else if continentalness < 0.35 {
            let t = continentalness / 0.35;
            let t_smooth = t * t * (3.0 - 2.0 * t);
            0.55 + 0.50 * t_smooth
        } else {
            let t = ((continentalness - 0.35) / 0.50).min(1.0);
            let t_smooth = t * t * (3.0 - 2.0 * t);
            1.05 + 1.75 * t_smooth
        }
    }

    fn natural_height_at(&self, world_x: f32, world_z: f32, climate: &ClimateSample) -> f32 {
        let macro_noise = fractal_noise(
            world_x,
            world_z,
            self.macro_frequency,
            3,
            0.55,
            self.seed.wrapping_add(31_337),
        );

        let rolling_hills = fractal_noise(
            world_x,
            world_z,
            0.012,
            2,
            0.5,
            self.seed.wrapping_add(45_117),
        ) * self.rolling_hills_amplitude;

        let detail_noise = fractal_noise(
            world_x,
            world_z,
            self.detail_frequency,
            self.detail_octaves,
            self.persistence,
            self.seed.wrapping_add(81_731),
        );

        let cont_base = Self::continental_elevation(climate.continentalness);
        let roughness = Self::continental_roughness(climate.continentalness);

        let mountain_factor = ((climate.continentalness - 0.22) / 0.35).clamp(0.0, 1.0);
        let ridge = (1.0 - macro_noise.abs()).powi(2) * self.mountain_ridge_height * mountain_factor;

        let swamp_depression = if climate.continentalness > 0.02
            && climate.continentalness < 0.25
            && climate.humidity > 0.15
        {
            let wetness = ((climate.humidity - 0.15) / 0.20).clamp(0.0, 1.0);
            let inland_factor =
                (1.0 - ((climate.continentalness - 0.12).abs() / 0.12)).clamp(0.0, 1.0);
            wetness * inland_factor * 3.5
        } else {
            0.0
        };

        let base = self.base_height + cont_base - swamp_depression;
        let amplitude = (self.macro_amplitude * macro_noise + rolling_hills + self.detail_amplitude * detail_noise)
            * roughness
            + ridge;

        base + amplitude
    }

    fn terrain_height_at(&self, world_x: f32, world_z: f32, climate: &ClimateSample) -> i32 {
        self.natural_height_at(world_x, world_z, climate).round() as i32
    }
}

pub fn logical_block_sample_position(logical_coordinate: i32) -> f32 {
    logical_coordinate as f32 * LOGICAL_BLOCK_VOXELS as f32 + 0.5
}

pub fn logical_block_bottom(world_y: i32) -> i32 {
    world_y.div_euclid(LOGICAL_BLOCK_VOXELS) * LOGICAL_BLOCK_VOXELS
}

pub fn logical_block_top(world_y: i32) -> i32 {
    logical_block_bottom(world_y) + LOGICAL_BLOCK_VOXELS - 1
}

fn column_index(x: usize, z: usize) -> usize {
    x + z * CHUNK_SIZE
}

fn fractal_noise(
    world_x: f32,
    world_z: f32,
    base_frequency: f32,
    octaves: u32,
    persistence: f32,
    seed: u32,
) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut amplitude_sum = 0.0;

    for octave in 0..octaves {
        let x = world_x * base_frequency * frequency;
        let z = world_z * base_frequency * frequency;

        let octave_seed = seed.wrapping_add(octave.wrapping_mul(10_007));
        value += value_noise(x, z, octave_seed) * amplitude;

        amplitude_sum += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    if amplitude_sum > 0.0 {
        value / amplitude_sum
    } else {
        0.0
    }
}

fn value_noise(x: f32, z: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;

    let x1 = x0 + 1;
    let z1 = z0 + 1;

    let tx = smoothstep(x - x0 as f32);
    let tz = smoothstep(z - z0 as f32);

    let v00 = hash_value(x0, z0, seed);
    let v10 = hash_value(x1, z0, seed);
    let v01 = hash_value(x0, z1, seed);
    let v11 = hash_value(x1, z1, seed);

    let top = lerp(v00, v10, tx);
    let bottom = lerp(v01, v11, tx);

    lerp(top, bottom, tz)
}

fn hash_value(x: i32, z: i32, seed: u32) -> f32 {
    let mut hash = seed;
    hash ^= (x as u32).wrapping_mul(0x27D4_EB2D);
    hash ^= (z as u32).wrapping_mul(0x1656_67B1);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x85EB_CA6B);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xC2B2_AE35);
    hash ^= hash >> 16;

    let normalized = hash as f32 / u32::MAX as f32;
    normalized * 2.0 - 1.0
}

#[allow(dead_code)]
fn smooth_range(value: f32, start: f32, end: f32) -> f32 {
    if end <= start {
        return if value >= start { 1.0 } else { 0.0 };
    }

    let normalized = ((value - start) / (end - start)).clamp(0.0, 1.0);
    smoothstep(normalized)
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + (end - start) * amount
}
