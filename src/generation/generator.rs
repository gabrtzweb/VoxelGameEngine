use bevy::prelude::*;

use super::{
    biome::{BiomeType, ClimateGenerator, ClimateSample},
    caves::{CaveGenerator, CaveNoiseSample},
    flora, ores,
    strata::StrataGenerator,
};
use crate::core::math::lerp;
use crate::core::noise::{Simd4f, fractal_noise_2d as fractal_noise, fractal_noise_2d_x4};
use crate::world::{BlockShape, CHUNK_SIZE, CHUNK_VOLUME, Chunk, Voxel};

pub const LOGICAL_BLOCK_VOXELS: i32 = 1;

pub const SEED_OFFSET_BEACH_NOISE: u32 = 88_411;
pub const SEED_OFFSET_SURFACE_DITHER: u32 = 45_678;
pub const SEED_OFFSET_SNOWLINE_JITTER: u32 = 91_111;
pub const SEED_OFFSET_SNOW_SLOPE: u32 = 82_222;
pub const SEED_OFFSET_OUTCROP_TURF: u32 = 73_333;

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
            biome: BiomeType::default(),
            is_beach: false,
            is_cliff: false,
            is_underground_river: false,
            climate: ClimateSample {
                continentalness: 0.0,
                erosion: 0.0,
                ridges: 0.0,
                temperature: 0.0,
                humidity: 0.0,
                depth: 0.0,
                biome: BiomeType::default(),
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
            macro_amplitude: 4.0,
            macro_frequency: 0.0260,

            // Local terrain relief
            detail_amplitude: 5.4,
            detail_frequency: 0.032,
            detail_octaves: 3,
            persistence: 0.5,

            // Mountain ridges and rolling hills
            mountain_ridge_height: 54.0,
            rolling_hills_amplitude: 10.0,

            // Sea level in voxels
            sea_level: 12,

            river_frequency: 0.0048,
            river_width: 0.064,

            climate: ClimateGenerator::default(),
            caves: CaveGenerator::default(),
            strata: StrataGenerator::default(),

            // Tree generation disabled for initial terrain validation
            tree_density: 0.0,

            version: 0,
        }
    }
}

impl TerrainGenerator {
    /// Fast 5-point bounding evaluation (4 corners + center) providing a safe upper-bound
    /// on the maximum solid terrain, water, and tree canopy height anywhere within chunk column (chunk_x, chunk_z).
    /// Used for O(1) early rejection of empty sky chunks.
    pub fn estimate_chunk_max_height(&self, chunk_x: i32, chunk_z: i32) -> i32 {
        let chunk_origin_x = chunk_x * CHUNK_SIZE as i32;
        let chunk_origin_z = chunk_z * CHUNK_SIZE as i32;

        let samples = [
            (chunk_origin_x, chunk_origin_z),
            (chunk_origin_x + (CHUNK_SIZE as i32 - 1), chunk_origin_z),
            (chunk_origin_x, chunk_origin_z + (CHUNK_SIZE as i32 - 1)),
            (
                chunk_origin_x + (CHUNK_SIZE as i32 - 1),
                chunk_origin_z + (CHUNK_SIZE as i32 - 1),
            ),
            (
                chunk_origin_x + (CHUNK_SIZE as i32 / 2),
                chunk_origin_z + (CHUNK_SIZE as i32 / 2),
            ),
        ];

        let mut max_h = f32::MIN;
        for (sx, sz) in samples {
            let (raw_h, _, _) = self.continuous_height_and_biome(sx, sz);
            max_h = max_h.max(raw_h);
        }

        let sea_level = self.effective_sea_level() as f32;
        // 24.0 blocks buffer safely accounts for intra-chunk noise variance, local cliff tops, and tallest tree canopies (8 blocks)
        (max_h.max(sea_level) + 24.0).ceil() as i32
    }

    /// Fast 5-point bounding evaluation providing a safe lower-bound
    /// on the minimum solid terrain or seabed height anywhere within chunk column (chunk_x, chunk_z).
    /// Used to ensure visible surface terrain and water chunks stay loaded when player is flying high.
    pub fn estimate_chunk_surface_min_height(&self, chunk_x: i32, chunk_z: i32) -> i32 {
        let chunk_origin_x = chunk_x * CHUNK_SIZE as i32;
        let chunk_origin_z = chunk_z * CHUNK_SIZE as i32;

        let samples = [
            (chunk_origin_x, chunk_origin_z),
            (chunk_origin_x + (CHUNK_SIZE as i32 - 1), chunk_origin_z),
            (chunk_origin_x, chunk_origin_z + (CHUNK_SIZE as i32 - 1)),
            (
                chunk_origin_x + (CHUNK_SIZE as i32 - 1),
                chunk_origin_z + (CHUNK_SIZE as i32 - 1),
            ),
            (
                chunk_origin_x + (CHUNK_SIZE as i32 / 2),
                chunk_origin_z + (CHUNK_SIZE as i32 / 2),
            ),
        ];

        let mut min_h = f32::MAX;
        for (sx, sz) in samples {
            let (raw_h, _, _) = self.continuous_height_and_biome(sx, sz);
            min_h = min_h.min(raw_h);
        }

        let sea_level = self.effective_sea_level() as f32;
        // 16.0 blocks buffer safely accounts for intra-chunk noise variance and local seabed dips
        (min_h.min(sea_level) - 16.0).floor() as i32
    }

    pub fn generate_chunk(&self, chunk_coordinate: IVec3) -> Chunk {
        let chunk_origin = chunk_coordinate * CHUNK_SIZE as i32;
        let chunk_min_y = chunk_origin.y;
        let chunk_max_y = chunk_min_y + CHUNK_SIZE as i32 - 1;

        // 1. Bedrock floor fast path: bottom layers below bedrock are 100% solid Dreadstone
        if chunk_max_y <= self.strata.bedrock_min_block_y {
            let mut chunk = Chunk::filled(Voxel::Mantle);
            chunk.set_subterranean(true);
            return chunk;
        }

        // 2. Sky fast path: if chunk is strictly above estimated max terrain height + canopy, return Air in O(1)
        let est_max = self.estimate_chunk_max_height(chunk_coordinate.x, chunk_coordinate.z);
        if chunk_min_y > est_max {
            return Chunk::filled(Voxel::Air);
        }

        let mut columns = [TerrainColumn::default(); CHUNK_SIZE * CHUNK_SIZE];
        let mut maximum_filled_height = i32::MIN;
        let mut minimum_filled_height = i32::MAX;

        for z in 0..CHUNK_SIZE {
            for x_start in (0..CHUNK_SIZE).step_by(4) {
                let wx = [
                    chunk_origin.x + x_start as i32,
                    chunk_origin.x + x_start as i32 + 1,
                    chunk_origin.x + x_start as i32 + 2,
                    chunk_origin.x + x_start as i32 + 3,
                ];
                let wz = [chunk_origin.z + z as i32; 4];

                let cols = self.sample_columns_x4(wx, wz);
                for (lane, col) in cols.into_iter().enumerate() {
                    let x = x_start + lane;
                    columns[column_index(x, z)] = col;

                    let col_top = col.terrain_height;
                    let filled_height = col.water_level.unwrap_or(col_top).max(col_top) + 16;

                    maximum_filled_height = maximum_filled_height.max(filled_height);
                    minimum_filled_height = minimum_filled_height.min(col_top);
                }
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

            // Subterranean Ores & Crystal Geodes Generation
            ores::populate_chunk_ores(&mut voxels, chunk_origin, self.seed);
            ores::populate_chunk_geodes(&mut voxels, chunk_origin, self.seed);

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
                        if !voxel.is_collidable() || voxel.is_unbreakable() {
                            continue;
                        }

                        let world_x = chunk_origin.x + x as i32;
                        let world_z = chunk_origin.z + z as i32;

                        let above = get_v(&voxels, x as i32, y as i32 + 1, z as i32);
                        let has_air_neighbor = above == Voxel::Air
                            || get_v(&voxels, x as i32 - 1, y as i32, z as i32) == Voxel::Air
                            || get_v(&voxels, x as i32 + 1, y as i32, z as i32) == Voxel::Air
                            || get_v(&voxels, x as i32, y as i32, z as i32 - 1) == Voxel::Air
                            || get_v(&voxels, x as i32, y as i32, z as i32 + 1) == Voxel::Air;

                        // Unique Subterranean Cavern Chamber Lining (Cave Karst & Cave Calcite)
                        if has_air_neighbor {
                            let cavern_noise = crate::core::noise::gradient_noise_3d(
                                world_x as f32 * 0.024,
                                world_y as f32 * 0.032,
                                world_z as f32 * 0.024,
                                self.seed.wrapping_add(88_112),
                            );

                            if cavern_noise > 0.45
                                && matches!(
                                    voxel,
                                    Voxel::Dolomite | Voxel::Limestone | Voxel::Marble
                                )
                            {
                                let rubble_roll = (world_x ^ world_y ^ world_z) & 7;
                                voxels[idx] = if rubble_roll == 0 {
                                    Voxel::FracturedCaveKarst
                                } else {
                                    Voxel::CaveKarst
                                };
                            } else {
                                let calcite_noise = crate::core::noise::gradient_noise_3d(
                                    world_x as f32 * 0.026,
                                    world_y as f32 * 0.036,
                                    world_z as f32 * 0.026,
                                    self.seed.wrapping_add(66_331),
                                );
                                if calcite_noise > 0.52
                                    && matches!(
                                        voxel,
                                        Voxel::Dolomite | Voxel::Limestone | Voxel::Andesite
                                    )
                                {
                                    let rubble_roll = (world_x ^ world_y ^ world_z) & 7;
                                    voxels[idx] = if rubble_roll == 0 {
                                        Voxel::FracturedCaveCalcite
                                    } else {
                                        Voxel::CaveCalcite
                                    };
                                }
                            }
                        }

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

                                if let Some(orient) = corner_orient
                                    && wall_noise > 0.15
                                {
                                    chunk_shapes.push((idx, BlockShape::Column, orient));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Tree and vegetation generation is bypassed during terrain overhaul.

        // Phase 6: Procedural Flora Generation (Wildgrass, Ferns, Bushes, Flowers & Cave Growths)
        if has_solid_voxels {
            flora::populate_chunk_flora(
                &mut voxels,
                &columns,
                chunk_origin,
                self.seed,
                |wx, wy, wz, col| self.voxel_at(col, wx, wy, wz),
            );
        }

        if !has_solid_voxels {
            return Chunk::filled(Voxel::Air);
        }

        let mut chunk = Chunk::from_voxels(voxels);
        for (idx, shape, orientation) in chunk_shapes {
            let (lx, ly, lz) = Chunk::index_to_xyz(idx);
            chunk.set_shape(lx, ly, lz, shape, orientation);
        }

        let is_subterranean = chunk_max_y < minimum_filled_height;
        chunk.set_subterranean(is_subterranean);

        chunk
    }

    pub fn effective_sea_level(&self) -> i32 {
        logical_block_top(self.sea_level)
    }

    pub fn continuous_height_and_biome_with_climate(
        &self,
        world_x: i32,
        world_z: i32,
    ) -> (f32, BiomeType, bool, ClimateSample) {
        let logical_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS);
        let logical_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS);
        let sample_x = logical_block_sample_position(logical_x);
        let sample_z = logical_block_sample_position(logical_z);

        let mut climate = self.climate.sample_dithered(sample_x, sample_z, self.seed);

        let mut raw_height = self.natural_height_at(world_x as f32, world_z as f32, &climate);

        let (river_factor, is_river_path) = self.river_sample(sample_x, sample_z);
        let is_inland = climate.continentalness >= -0.04;
        let is_river = is_river_path && is_inland && river_factor > 0.05;
        let mut is_underground_river = false;
        let sea_level = self.effective_sea_level();

        if is_river {
            let initial_height = raw_height.round() as i32;
            if initial_height > 95 {
                is_underground_river = true;
            } else {
                let carve_t = river_factor * river_factor * (3.0 - 2.0 * river_factor);
                let blend_to_sea = ((0.12 - climate.continentalness) / 0.16).clamp(0.0, 1.0);
                let river_depth = 2.5 + river_factor * 2.0;
                let target_bed = lerp(raw_height - river_depth, (sea_level - 2) as f32, blend_to_sea);
                let carved_bed = lerp(raw_height, target_bed, (carve_t * 1.35).min(1.0));
                raw_height = raw_height.min(carved_bed);

                if climate.continentalness < 0.18
                    && climate.humidity > 0.35
                    && raw_height <= (sea_level + 2) as f32
                {
                    climate.biome = BiomeType::OrchidSwamp;
                }
            }
        } else if let Some((pond_water_y, carve_depth)) =
            self.sample_flat_pond(world_x, world_z, raw_height, &climate)
        {
            raw_height = (pond_water_y as f32 - carve_depth).min(raw_height);
        }

        (raw_height, climate.biome, is_underground_river, climate)
    }

    pub fn continuous_height_and_biome(
        &self,
        world_x: i32,
        world_z: i32,
    ) -> (f32, BiomeType, bool) {
        let (raw_height, final_biome, is_underground_river, _) =
            self.continuous_height_and_biome_with_climate(world_x, world_z);
        (raw_height, final_biome, is_underground_river)
    }

    /// Evaluates 4 terrain column heights, biomes, and climates simultaneously using 4-wide SIMD noise.
    pub fn continuous_height_and_biome_x4(
        &self,
        world_xs: [i32; 4],
        world_zs: [i32; 4],
    ) -> [(f32, BiomeType, bool, ClimateSample); 4] {
        let logical_x = [
            world_xs[0].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_xs[1].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_xs[2].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_xs[3].div_euclid(LOGICAL_BLOCK_VOXELS),
        ];
        let logical_z = [
            world_zs[0].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_zs[1].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_zs[2].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_zs[3].div_euclid(LOGICAL_BLOCK_VOXELS),
        ];

        let sample_x = Simd4f([
            logical_block_sample_position(logical_x[0]),
            logical_block_sample_position(logical_x[1]),
            logical_block_sample_position(logical_x[2]),
            logical_block_sample_position(logical_x[3]),
        ]);
        let sample_z = Simd4f([
            logical_block_sample_position(logical_z[0]),
            logical_block_sample_position(logical_z[1]),
            logical_block_sample_position(logical_z[2]),
            logical_block_sample_position(logical_z[3]),
        ]);

        let climates = self
            .climate
            .sample_dithered_x4(sample_x, sample_z, self.seed);
        let sea_level = self.effective_sea_level();

        let wx_f32 = Simd4f([
            world_xs[0] as f32,
            world_xs[1] as f32,
            world_xs[2] as f32,
            world_xs[3] as f32,
        ]);
        let wz_f32 = Simd4f([
            world_zs[0] as f32,
            world_zs[1] as f32,
            world_zs[2] as f32,
            world_zs[3] as f32,
        ]);

        let raw_heights = self.natural_height_at_x4(wx_f32, wz_f32, &climates);
        let (river_factors, is_river_paths) = self.river_sample_x4(sample_x, sample_z);

        let mut res = [(0.0f32, BiomeType::default(), false, climates[0]); 4];

        for i in 0..4 {
            let mut raw_height = raw_heights.0[i];
            let mut climate = climates[i];
            let river_factor = river_factors[i];
            let is_river_path = is_river_paths[i];

            let is_inland = climate.continentalness >= -0.04;
            let is_river = is_river_path && is_inland && river_factor > 0.04;
            let mut is_underground_river = false;

            if is_river {
                let initial_height = raw_height.round() as i32;
                if initial_height > 95 {
                    is_underground_river = true;
                } else {
                    let carve_t = river_factor * river_factor * (3.0 - 2.0 * river_factor);
                    let blend_to_sea = ((0.12 - climate.continentalness) / 0.16).clamp(0.0, 1.0);
                    let river_depth = 2.5 + river_factor * 2.0;
                    let target_bed = lerp(raw_height - river_depth, (sea_level - 2) as f32, blend_to_sea);
                    let carved_bed = lerp(raw_height, target_bed, (carve_t * 1.35).min(1.0));
                    raw_height = raw_height.min(carved_bed);

                    if climate.continentalness < 0.18
                        && climate.humidity > 0.35
                        && raw_height <= (sea_level + 2) as f32
                    {
                        climate.biome = BiomeType::OrchidSwamp;
                    }
                }
            } else if let Some((pond_water_y, carve_depth)) =
                self.sample_flat_pond(world_xs[i], world_zs[i], raw_height, &climate)
            {
                raw_height = (pond_water_y as f32 - carve_depth).min(raw_height);
            }

            res[i] = (raw_height, climate.biome, is_underground_river, climate);
        }

        res
    }

    pub fn build_terrain_column(
        &self,
        world_x: i32,
        world_z: i32,
        raw_height: f32,
        final_biome: BiomeType,
        is_underground_river: bool,
        climate: ClimateSample,
    ) -> TerrainColumn {
        let sea_level = self.effective_sea_level();
        let terrain_height = raw_height.round() as i32;
        let mut surface_shape = BlockShape::Full;
        let mut shape_orientation = 0u8;

        let logical_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS);
        let logical_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS);
        let sample_x = logical_block_sample_position(logical_x);
        let sample_z = logical_block_sample_position(logical_z);

        let (river_factor, is_river_path) = self.river_sample(sample_x, sample_z);
        let is_inland = climate.continentalness >= -0.04;
        let is_river = is_river_path && is_inland && river_factor > 0.05 && !is_underground_river;

        let water_level = if terrain_height < sea_level {
            Some(sea_level)
        } else if is_river {
            let blend_to_sea = ((0.12 - climate.continentalness) / 0.16).clamp(0.0, 1.0);
            let uncarved_height = self.natural_height_at(world_x as f32, world_z as f32, &climate);
            let target_bed = lerp(uncarved_height - (2.5 + river_factor * 2.0), (sea_level - 2) as f32, blend_to_sea);
            let raw_water = target_bed + 1.8;
            let terraced_water = ((raw_water / 2.0).floor() * 2.0) as i32;
            let r_water = terraced_water.min((uncarved_height - 1.0).round() as i32).max(sea_level);
            if terrain_height < r_water {
                Some(r_water)
            } else {
                None
            }
        } else if let Some((pond_water_y, _)) =
            self.sample_flat_pond(world_x, world_z, raw_height, &climate)
        {
            if terrain_height < pond_water_y {
                Some(pond_water_y)
            } else {
                None
            }
        } else {
            None
        };

        let mut is_cliff = false;

        // Transitions on dry land above water level (terrain at or below sea_level remains strictly full blocks)
        if water_level.is_none() && terrain_height > sea_level {
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

            let max_delta = d_east
                .abs()
                .max(d_west.abs())
                .max(d_south.abs())
                .max(d_north.abs());

            let is_cliff_biome = matches!(
                final_biome,
                BiomeType::BasaltCliffs
                    | BiomeType::GraniteCliffs
                    | BiomeType::WhiteCliffs
                    | BiomeType::YosemiteCliffs
                    | BiomeType::FrozenCliffs
                    | BiomeType::BryceCanyon
                    | BiomeType::AmethystCanyon
                    | BiomeType::DesertSpires
                    | BiomeType::EmeraldPeaks
                    | BiomeType::AlpineHighlands
                    | BiomeType::VolcanicPeaks
                    | BiomeType::RockyMountains
                    | BiomeType::ScarletMountains
            );

            is_cliff = if is_cliff_biome {
                max_delta >= 3
            } else {
                max_delta >= 5
            };

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

            if !is_cliff && !keep_solid {
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

        // Horizontal beach width: coastal transition strip directly bordering water
        let shore_noise = crate::core::noise::gradient_noise_2d(
            world_x as f32 * 0.12,
            world_z as f32 * 0.12,
            self.seed.wrapping_add(SEED_OFFSET_BEACH_NOISE),
        );
        let max_beach_y = sea_level + 5 + (shore_noise * 3.0).round() as i32;

        let is_coastal_continentalness = climate.continentalness >= -0.18 && climate.continentalness <= 0.05;
        let is_coastal_zone = matches!(final_biome, BiomeType::GravelBeach | BiomeType::MirageIsles)
            || is_coastal_continentalness;

        let is_beach = !is_cliff
            && is_coastal_zone
            && terrain_height >= sea_level - 2
            && terrain_height <= max_beach_y;

        TerrainColumn {
            terrain_height,
            water_level,
            biome: final_biome,
            is_beach,
            is_cliff,
            is_underground_river,
            climate,
            surface_shape,
            shape_orientation,
        }
    }

    pub fn sample_column(&self, world_x: i32, world_z: i32) -> TerrainColumn {
        let (raw_height, final_biome, is_underground_river, climate) =
            self.continuous_height_and_biome_with_climate(world_x, world_z);
        self.build_terrain_column(
            world_x,
            world_z,
            raw_height,
            final_biome,
            is_underground_river,
            climate,
        )
    }

    /// Evaluates 4 terrain columns simultaneously using 4-wide SIMD noise.
    pub fn sample_columns_x4(&self, world_xs: [i32; 4], world_zs: [i32; 4]) -> [TerrainColumn; 4] {
        let samples = self.continuous_height_and_biome_x4(world_xs, world_zs);
        let mut cols = [TerrainColumn::default(); 4];
        for i in 0..4 {
            let (raw_height, final_biome, is_underground_river, climate) = samples[i];
            cols[i] = self.build_terrain_column(
                world_xs[i],
                world_zs[i],
                raw_height,
                final_biome,
                is_underground_river,
                climate,
            );
        }
        cols
    }

    /// Evaluates whether a coordinate is inside a flat alpine/meadow pond.
    /// Returns Option<(pond_water_y, pond_bed_carve_depth)>.
    pub fn sample_flat_pond(
        &self,
        world_x: i32,
        world_z: i32,
        natural_height: f32,
        climate: &ClimateSample,
    ) -> Option<(i32, f32)> {
        let sea_level = self.effective_sea_level();
        if climate.continentalness < 0.08 || climate.erosion < 0.08 || natural_height <= (sea_level + 4) as f32 {
            return None;
        }

        let cell_size = 110;
        let cx = world_x.div_euclid(cell_size);
        let cz = world_z.div_euclid(cell_size);

        let h = crate::core::noise::hash_2d(cx, cz, self.seed.wrapping_add(444_222));
        if h % 4 != 0 {
            return None;
        }

        let offset_x = 25 + ((h >> 8) % (cell_size as u32 - 50)) as i32;
        let offset_z = 25 + ((h >> 16) % (cell_size as u32 - 50)) as i32;
        let center_x = cx * cell_size + offset_x;
        let center_z = cz * cell_size + offset_z;

        let radius = 10.0 + ((h >> 24) % 6) as f32; // 10 to 15 blocks radius

        let dx = (world_x - center_x) as f32;
        let dz = (world_z - center_z) as f32;
        let dist = (dx * dx + dz * dz).sqrt();

        if dist < radius {
            let center_climate = self.climate.sample_dithered(center_x as f32, center_z as f32, self.seed);
            let center_h = self.natural_height_at(center_x as f32, center_z as f32, &center_climate);

            // Pond water is sampled ONCE at the fixed center of the pond:
            let pond_water_y = (center_h - 1.2).round() as i32;
            let t = dist / radius;
            // Carve basin deeper in the middle (up to 3.0 blocks below water)
            let carve_depth = (1.0 - t * t) * 3.0;
            Some((pond_water_y, carve_depth))
        } else {
            None
        }
    }

    /// Evaluates a Voronoi-based non-looping river network.
    /// Geometrically partitions the world using Voronoi bisectors that branch and merge
    /// into natural confluences without ever forming closed circular loops around hills.
    pub fn voronoi_river_sample(&self, world_x: f32, world_z: f32) -> (f32, bool) {
        let warp_x = crate::core::noise::gradient_noise_2d(
            world_x * 0.0035,
            world_z * 0.0035,
            self.seed.wrapping_add(101_333),
        ) * 45.0;
        let warp_z = crate::core::noise::gradient_noise_2d(
            world_x * 0.0035 + 127.0,
            world_z * 0.0035 + 127.0,
            self.seed.wrapping_add(202_444),
        ) * 45.0;

        let wx = world_x + warp_x;
        let wz = world_z + warp_z;

        let cell_size = 280.0;
        let cell_x = (wx / cell_size).floor() as i32;
        let cell_z = (wz / cell_size).floor() as i32;

        let mut min_d1_sq = f32::MAX;
        let mut min_d2_sq = f32::MAX;
        let mut p1 = [0.0f32; 2];
        let mut p2 = [0.0f32; 2];
        let mut p1_cell = (0, 0);
        let mut p2_cell = (0, 0);

        for dx in -1..=1 {
            for dz in -1..=1 {
                let cx = cell_x + dx;
                let cz = cell_z + dz;

                let h1 = crate::core::noise::hash_2d(cx, cz, self.seed.wrapping_add(333_555));
                let h2 = crate::core::noise::hash_2d(cx, cz, self.seed.wrapping_add(777_999));
                let jx = 0.20 + (h1 as f32 / u32::MAX as f32) * 0.60;
                let jz = 0.20 + (h2 as f32 / u32::MAX as f32) * 0.60;

                let pt_x = (cx as f32 + jx) * cell_size;
                let pt_z = (cz as f32 + jz) * cell_size;

                let d_sq = (wx - pt_x) * (wx - pt_x) + (wz - pt_z) * (wz - pt_z);
                if d_sq < min_d1_sq {
                    min_d2_sq = min_d1_sq;
                    p2 = p1;
                    p2_cell = p1_cell;

                    min_d1_sq = d_sq;
                    p1 = [pt_x, pt_z];
                    p1_cell = (cx, cz);
                } else if d_sq < min_d2_sq {
                    min_d2_sq = d_sq;
                    p2 = [pt_x, pt_z];
                    p2_cell = (cx, cz);
                }
            }
        }

        let diff_x = p2[0] - p1[0];
        let diff_z = p2[1] - p1[1];
        let edge_len = (diff_x * diff_x + diff_z * diff_z).sqrt();
        if edge_len < 1.0 {
            return (0.0, false);
        }

        let (c_min, c_max) = if p1_cell < p2_cell { (p1_cell, p2_cell) } else { (p2_cell, p1_cell) };
        let edge_hash = crate::core::noise::hash_2d(
            c_min.0.wrapping_mul(31) ^ c_max.0,
            c_min.1.wrapping_mul(31) ^ c_max.1,
            self.seed.wrapping_add(888_111),
        );
        if edge_hash % 3 == 0 {
            return (0.0, false);
        }

        let mid_x = 0.5 * (p1[0] + p2[0]);
        let mid_z = 0.5 * (p1[1] + p2[1]);
        let dist_to_edge = ((wx - mid_x) * diff_x + (wz - mid_z) * diff_z).abs() / edge_len;

        let width_noise = crate::core::noise::gradient_noise_2d(
            world_x * 0.0025,
            world_z * 0.0025,
            self.seed.wrapping_add(999_222),
        );
        let base_width = 11.5 + width_noise * 3.5;

        if dist_to_edge < base_width {
            let depth_factor = 1.0 - (dist_to_edge / base_width);
            (depth_factor, true)
        } else {
            (0.0, false)
        }
    }

    pub fn river_sample(&self, world_x: f32, world_z: f32) -> (f32, bool) {
        self.voronoi_river_sample(world_x, world_z)
    }

    pub fn river_sample_x4(&self, world_x: Simd4f, world_z: Simd4f) -> ([f32; 4], [bool; 4]) {
        let mut depth_factors = [0.0f32; 4];
        let mut is_river = [false; 4];
        for i in 0..4 {
            let (df, ir) = self.voronoi_river_sample(world_x.0[i], world_z.0[i]);
            depth_factors[i] = df;
            is_river[i] = ir;
        }
        (depth_factors, is_river)
    }

    /// Evaluates 3D volumetric density for surface rock arches and overhangs.
    fn sample_surface_overhang(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        column: TerrainColumn,
        dy: i32,
    ) -> Option<Voxel> {
        let is_rugged = column.climate.ridges > 0.20
            || matches!(
                column.biome,
                BiomeType::BasaltCliffs
                    | BiomeType::GraniteCliffs
                    | BiomeType::WhiteCliffs
                    | BiomeType::YosemiteCliffs
                    | BiomeType::FrozenCliffs
                    | BiomeType::DesertSpires
                    | BiomeType::BryceCanyon
                    | BiomeType::EmeraldPeaks
                    | BiomeType::AlpineHighlands
                    | BiomeType::VolcanicPeaks
                    | BiomeType::RockyMountains
                    | BiomeType::FracturedSavanna
                    | BiomeType::PaintedMountains
            );

        if !is_rugged || dy > 16 {
            return None;
        }

        let noise_a = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.05,
            world_y as f32 * 0.07,
            world_z as f32 * 0.05,
            self.seed.wrapping_add(88_871),
        );
        let noise_b = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.09,
            world_y as f32 * 0.09,
            world_z as f32 * 0.09,
            self.seed.wrapping_add(99_931),
        );

        let density = noise_a + noise_b * 0.35 - (dy as f32 * 0.075);
        if density > 0.32 {
            if matches!(
                column.biome,
                BiomeType::BryceCanyon
                    | BiomeType::PaintedMountains
                    | BiomeType::SavannaBadlands
                    | BiomeType::DesertSpires
            ) {
                Some(crate::generation::strata::canyon_strata_voxel(
                    world_y, world_x, world_z, self.seed,
                ))
            } else {
                Some(column.biome.config().cliff_material)
            }
        } else {
            None
        }
    }

    /// Evaluates if an arch opening should hollow out under a mountain cliff surface.
    fn sample_surface_arch_opening(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        column: TerrainColumn,
        depth: i32,
    ) -> bool {
        let is_rugged = column.climate.ridges > 0.25
            || matches!(
                column.biome,
                BiomeType::BasaltCliffs
                    | BiomeType::GraniteCliffs
                    | BiomeType::WhiteCliffs
                    | BiomeType::YosemiteCliffs
                    | BiomeType::FrozenCliffs
                    | BiomeType::DesertSpires
                    | BiomeType::BryceCanyon
                    | BiomeType::EmeraldPeaks
                    | BiomeType::AlpineHighlands
                    | BiomeType::VolcanicPeaks
                    | BiomeType::RockyMountains
            );

        if !is_rugged || depth < 1 || depth > 6 {
            return false;
        }

        let noise_a = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.05,
            world_y as f32 * 0.07,
            world_z as f32 * 0.05,
            self.seed.wrapping_add(88_871),
        );
        let noise_b = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.09,
            world_y as f32 * 0.09,
            world_z as f32 * 0.09,
            self.seed.wrapping_add(99_931),
        );

        noise_a > 0.30 && noise_b < -0.15
    }

    /// Converts a mother rock into its corresponding surface outcrop turf block.
    pub fn rock_to_outcrop_grass(stone: Voxel) -> Voxel {
        match stone {
            Voxel::Andesite => Voxel::AndesiteGrass,
            Voxel::Argillite => Voxel::ArgilliteGrass,
            Voxel::Basalt => Voxel::BasaltGrass,
            Voxel::Chalk => Voxel::ChalkGrass,
            Voxel::Dolomite => Voxel::DolomiteGrass,
            Voxel::Granite => Voxel::GraniteGrass,
            Voxel::Limestone => Voxel::LimestoneGrass,
            _ => Voxel::GraniteGrass,
        }
    }

    /// Converts a mother rock into its corresponding snow outcrop block for alpine transitions.
    pub fn rock_to_outcrop_snow(stone: Voxel) -> Voxel {
        match stone {
            Voxel::Andesite => Voxel::AndesiteSnow,
            Voxel::Argillite => Voxel::ArgilliteSnow,
            Voxel::Basalt => Voxel::BasaltSnow,
            Voxel::Chalk => Voxel::ChalkSnow,
            Voxel::Dolomite => Voxel::DolomiteSnow,
            Voxel::Granite => Voxel::GraniteSnow,
            Voxel::Limestone => Voxel::LimestoneSnow,
            _ => Voxel::GraniteSnow,
        }
    }

    pub fn surface_material_at(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        column: TerrainColumn,
    ) -> Voxel {
        let cfg = column.biome.config();
        let dither_noise = crate::core::noise::gradient_noise_2d(
            world_x as f32 * 0.15,
            world_z as f32 * 0.15,
            self.seed.wrapping_add(SEED_OFFSET_SURFACE_DITHER),
        );

        // 1. Submerged terrain (underwater) floor materials
        let is_submerged = column.water_level.is_some_and(|wl| world_y <= wl);
        if is_submerged {
            return match column.biome {
                BiomeType::DeepWarmOcean => {
                    if dither_noise > 0.0 {
                        Voxel::SandCommon
                    } else {
                        Voxel::ClayBlue
                    }
                }
                BiomeType::ColdOcean => {
                    if dither_noise > 0.35 {
                        Voxel::GravelSandy
                    } else if dither_noise > -0.20 {
                        Voxel::ClayBlue
                    } else {
                        Voxel::GravelDirty
                    }
                }
                BiomeType::CoralReef => {
                    if dither_noise > 0.40 {
                        Voxel::BrainCoral
                    } else if dither_noise > 0.15 {
                        Voxel::BubbleCoral
                    } else if dither_noise > -0.15 {
                        Voxel::FireCoral
                    } else if dither_noise > -0.40 {
                        Voxel::HeartCoral
                    } else {
                        Voxel::Reefstone
                    }
                }
                BiomeType::OceanTrench => {
                    if dither_noise > 0.20 {
                        Voxel::Basalt
                    } else if dither_noise > -0.20 {
                        Voxel::Gabbro
                    } else {
                        Voxel::ClayBlue
                    }
                }
                BiomeType::Caldera => Voxel::Basalt,
                BiomeType::OrchidSwamp => Voxel::PeatMud,
                BiomeType::GlacialChasm | BiomeType::FrozenCliffs | BiomeType::IceCaves => Voxel::FrostIcePacked,
                _ => {
                    if column.climate.temperature < -0.15 {
                        if dither_noise > 0.0 {
                            Voxel::GravelSandy
                        } else {
                            Voxel::GravelDirty
                        }
                    } else if column.climate.humidity > 0.40 {
                        Voxel::SiltMud
                    } else {
                        Voxel::SandCommon
                    }
                }
            };
        }

        // 2. Coastal Shoreline & Beach Surface (predominantly sand with organic scattered gravel patches)
        if column.is_beach {
            let beach_noise = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.16,
                world_z as f32 * 0.16,
                self.seed.wrapping_add(SEED_OFFSET_BEACH_NOISE),
            );

            // Dedicated gravel beach biome or extreme subarctic cold coast:
            // ~70% sand with natural gravel pockets (not 99% solid gravel!)
            let is_gravel_specialist = column.biome == BiomeType::GravelBeach
                || (column.climate.temperature < -0.40 && column.biome != BiomeType::FrozenCliffs);

            if is_gravel_specialist {
                return if beach_noise > 0.35 {
                    if dither_noise > 0.0 {
                        Voxel::GravelSandy
                    } else {
                        Voxel::GravelDirty
                    }
                } else {
                    Voxel::SandCommon
                };
            }

            // Red sand with gravel patches on arid canyon / badlands shores
            if matches!(
                column.biome,
                BiomeType::BryceCanyon
                    | BiomeType::PaintedMountains
                    | BiomeType::AncientSands
                    | BiomeType::DesertSpires
                    | BiomeType::SavannaBadlands
            ) {
                return if beach_noise > 0.48 {
                    Voxel::GravelSandy
                } else {
                    Voxel::SandRed
                };
            }

            // Standard beaches: predominantly sand (~80%) with natural scattered patches of gravel (~20%)
            let has_gravel_patch = beach_noise > 0.48;
            if has_gravel_patch {
                return if dither_noise > 0.0 {
                    Voxel::GravelSandy
                } else {
                    Voxel::GravelDirty
                };
            }

            // Clean beach sand: white coral sand in tropics, golden common sand in temperate zones
            return if column.climate.temperature > 0.40 {
                Voxel::SandWhite
            } else {
                Voxel::SandCommon
            };
        }

        // 3. High alpine elevation snowline: cold/temperate mountains above y >= 50 receive snowcaps
        let is_warm = matches!(
            column.biome,
            BiomeType::DesertSpires
                | BiomeType::AncientSands
                | BiomeType::DesertOasis
                | BiomeType::FracturedSavanna
                | BiomeType::SavannaBadlands
                | BiomeType::AshenSavanna
                | BiomeType::DuneSea
                | BiomeType::BryceCanyon
                | BiomeType::VolcanicPeaks
                | BiomeType::GravelBeach
                | BiomeType::MirageIsles
        );

        if !is_warm && column.climate.temperature < 0.10 {
            let snowline_jitter = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.05,
                world_z as f32 * 0.05,
                self.seed.wrapping_add(SEED_OFFSET_SNOWLINE_JITTER),
            ) * 4.0;
            let snowline = 75.0 + snowline_jitter;

            if world_y as f32 >= snowline {
                let slope_noise = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.15,
                    world_z as f32 * 0.15,
                    self.seed.wrapping_add(SEED_OFFSET_SNOW_SLOPE),
                );
                if world_y as f32 >= snowline + 8.0 && slope_noise > 0.35 {
                    return column.biome.config().primary_stone;
                }
                return Voxel::SnowBlock;
            } else if world_y as f32 >= snowline - 10.0 {
                // Organic Alpine Transition Zone: gradual snow patches and snow-dusted rock outcrops
                let trans_t = (world_y as f32 - (snowline - 10.0)) / 10.0;
                let trans_noise = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.18,
                    world_z as f32 * 0.18,
                    self.seed.wrapping_add(SEED_OFFSET_SNOW_SLOPE),
                );
                let snow_threshold = 0.65 - trans_t * 0.50;
                if trans_noise > snow_threshold {
                    if trans_noise > snow_threshold + 0.18 {
                        return Voxel::SnowBlock;
                    } else {
                        return Self::rock_to_outcrop_snow(cfg.primary_stone);
                    }
                }
            }
        }

        // 4. Biome Surface Block Selection with organic dual-surface dithering
        if column.is_cliff {
            if matches!(
                column.biome,
                BiomeType::BryceCanyon
                    | BiomeType::PaintedMountains
                    | BiomeType::SavannaBadlands
                    | BiomeType::DesertSpires
            ) {
                crate::generation::strata::canyon_strata_voxel(world_y, world_x, world_z, self.seed)
            } else {
                cfg.cliff_material
            }
        } else {
            // Natural complementary surface patches:
            // Mulch in forests/jungles as accent litter, dry dirt in savannas
            if matches!(
                column.biome,
                BiomeType::CloudForest
                    | BiomeType::SakuraGrove
                    | BiomeType::YosemiteLowlands
                    | BiomeType::TropicalJungle
            ) && dither_noise > 0.58 {
                return Voxel::LoamMulch;
            } else if column.biome == BiomeType::Shield && dither_noise > 0.62 {
                return Voxel::PeatMulch;
            } else if matches!(
                column.biome,
                BiomeType::SavannaBadlands | BiomeType::FracturedSavanna
            ) && dither_noise > 0.60 {
                return Voxel::LateriteDirt;
            }

            // Small scattered rock outcrops on flat terrain using organic high-frequency noise
            let outcrop_noise = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.16,
                world_z as f32 * 0.16,
                self.seed.wrapping_add(SEED_OFFSET_OUTCROP_TURF),
            );
            if outcrop_noise > 0.68 {
                Self::rock_to_outcrop_grass(cfg.primary_stone)
            } else {
                cfg.surface_material
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
                    return Voxel::LiquidWater;
                } else {
                    return Voxel::Air;
                }
            }
        }

        if world_y > column.terrain_height {
            if let Some(water_level) = column.water_level
                && world_y <= water_level
            {
                return Voxel::LiquidWater;
            }

            // Surface rock arches and overhangs strictly generate on dry land above water
            if column.water_level.is_none() && column.terrain_height > self.effective_sea_level() + 2 {
                let dy = world_y - column.terrain_height;
                if dy <= 16 {
                    if let Some(overhang_voxel) =
                        self.sample_surface_overhang(world_x, world_y, world_z, column, dy)
                    {
                        return overhang_voxel;
                    }
                }
            }

            return Voxel::Air;
        }

        // Absolute bedrock floor: bottom layers of the world are strictly solid Mantle
        if world_y <= self.strata.bedrock_min_block_y {
            return Voxel::Mantle;
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

        // 3D Rock Arch Openings (hollow window under mountain cliff)
        if self.sample_surface_arch_opening(world_x, world_y, world_z, column, depth) {
            if let Some(water_level) = column.water_level
                && world_y <= water_level
            {
                return Voxel::LiquidWater;
            }
            return Voxel::Air;
        }

        let logical_depth = depth / LOGICAL_BLOCK_VOXELS;
        let biome_cfg = column.biome.config();

        if column.is_beach && logical_depth <= 5 {
            let beach_noise = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.16,
                world_z as f32 * 0.16,
                self.seed.wrapping_add(SEED_OFFSET_BEACH_NOISE),
            );

            let is_gravel_specialist = column.biome == BiomeType::GravelBeach
                || (column.climate.temperature < -0.40 && column.biome != BiomeType::FrozenCliffs);

            if is_gravel_specialist {
                return if beach_noise > 0.40 {
                    Voxel::SandCommon
                } else {
                    Voxel::GravelDirty
                };
            }

            if matches!(
                column.biome,
                BiomeType::BryceCanyon
                    | BiomeType::PaintedMountains
                    | BiomeType::AncientSands
                    | BiomeType::DesertSpires
                    | BiomeType::SavannaBadlands
            ) {
                return if beach_noise > 0.32 {
                    Voxel::GravelSandy
                } else {
                    Voxel::SandRed
                };
            }

            let has_gravel_patch = beach_noise > 0.28;
            if has_gravel_patch {
                return Voxel::GravelDirty;
            }

            return if column.climate.temperature > 0.40 {
                Voxel::SandWhite
            } else {
                Voxel::SandCommon
            };
        }

        if logical_depth <= 2 {
            let surface = self.surface_material_at(world_x, column.terrain_height, world_z, column);
            if matches!(
                surface,
                Voxel::SandCommon
                    | Voxel::SandDune
                    | Voxel::SandRed
                    | Voxel::SandWhite
                    | Voxel::GravelSandy
                    | Voxel::GravelDirty
                    | Voxel::GravelAshy
                    | Voxel::GravelSnowy
                    | Voxel::LoamMud
                    | Voxel::PeatMud
                    | Voxel::SiltMud
                    | Voxel::SnowBlock
                    | Voxel::FrostIce
                    | Voxel::FrostIcePacked
                    | Voxel::FrostIceBlack
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

    fn voxel_at(&self, column: TerrainColumn, world_x: i32, world_y: i32, world_z: i32) -> Voxel {
        let sample = self.caves.sample_noise_point(
            world_x as f32,
            world_y as f32,
            world_z as f32,
            self.seed,
        );
        self.voxel_at_sampled(column, world_x, world_y, world_z, sample)
    }

    pub fn continental_elevation(continentalness: f32) -> f32 {
        const SPLINE_NODES: [(f32, f32); 10] = [
            (-1.00, -48.0), // Deep abyssal ocean trench
            (-0.65, -30.0), // Deep ocean basin
            (-0.35, -20.0), // Open ocean floor
            (-0.15, -12.0), // Outer continental shelf
            (-0.04, -6.0),  // Shallow coastal shelf / beach transition
            (0.10, 5.0),    // Low coastal plains (y = 21)
            (0.22, 16.0),   // Inland rolling plains (y = 32)
            (0.35, 42.0),   // Highlands & foothills (y = 58)
            (0.50, 84.0),   // Rugged mountain chains (y = 100)
            (0.70, 140.0),  // Grand alpine peaks (y = 156)
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

        let steep_variation = ((climate.ridges - 0.10) / 0.45).clamp(0.0, 1.0);
        let abrupt_peak = if steep_variation > 0.0 && mountain_factor > 0.0 {
            let peak_form = (1.0 - (macro_noise * 1.35).abs().min(1.0)).powf(1.6);
            peak_form * 16.0 * steep_variation * mountain_factor
        } else {
            0.0
        };

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
        let amplitude = (self.macro_amplitude * macro_noise
            + rolling_hills
            + self.detail_amplitude * detail_noise)
            * roughness
            + ridge
            + abrupt_peak;

        base + amplitude
    }

    fn natural_height_at_x4(
        &self,
        world_x: Simd4f,
        world_z: Simd4f,
        climates: &[ClimateSample; 4],
    ) -> Simd4f {
        let macro_noise = fractal_noise_2d_x4(
            world_x,
            world_z,
            self.macro_frequency,
            3,
            0.55,
            self.seed.wrapping_add(31_337),
        );

        let rolling_hills = fractal_noise_2d_x4(
            world_x,
            world_z,
            0.012,
            2,
            0.5,
            self.seed.wrapping_add(45_117),
        )
        .scale(self.rolling_hills_amplitude);

        let detail_noise = fractal_noise_2d_x4(
            world_x,
            world_z,
            self.detail_frequency,
            self.detail_octaves,
            self.persistence,
            self.seed.wrapping_add(81_731),
        );

        let mut res = [0.0f32; 4];
        for i in 0..4 {
            let m_n = macro_noise.0[i];
            let r_h = rolling_hills.0[i];
            let d_n = detail_noise.0[i];
            let climate = &climates[i];

            let cont_base = Self::continental_elevation(climate.continentalness);
            let roughness = Self::continental_roughness(climate.continentalness);

            let mountain_factor = ((climate.continentalness - 0.22) / 0.35).clamp(0.0, 1.0);
            let ridge = (1.0 - m_n.abs()).powi(2) * self.mountain_ridge_height * mountain_factor;

            let steep_variation = ((climate.ridges - 0.10) / 0.45).clamp(0.0, 1.0);
            let abrupt_peak = if steep_variation > 0.0 && mountain_factor > 0.0 {
                let peak_form = (1.0 - (m_n * 1.35).abs().min(1.0)).powf(1.6);
                peak_form * 16.0 * steep_variation * mountain_factor
            } else {
                0.0
            };

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
            let amplitude = (self.macro_amplitude * m_n + r_h + self.detail_amplitude * d_n)
                * roughness
                + ridge
                + abrupt_peak;

            res[i] = base + amplitude;
        }

        Simd4f(res)
    }

    pub fn terrain_height_at(&self, world_x: f32, world_z: f32, climate: &ClimateSample) -> i32 {
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
