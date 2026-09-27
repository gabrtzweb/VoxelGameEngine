use bevy::prelude::*;

use super::{
    biome::{BiomeType, ClimateGenerator, ClimateSample},
    caves::{CaveGenerator, CaveNoiseSample},
    strata::StrataGenerator,
};
use crate::world::{BlockShape, CHUNK_SIZE, CHUNK_VOLUME, Chunk, Voxel};

pub const LOGICAL_BLOCK_VOXELS: i32 = 1;

#[derive(Clone, Copy, Debug)]
pub struct TerrainColumn {
    pub terrain_height: i32,
    pub water_level: Option<i32>,
    pub biome: BiomeType,
    pub is_beach: bool,
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
            biome: BiomeType::Plains,
            is_beach: false,
            is_underground_river: false,
            climate: ClimateSample {
                continentalness: 0.0,
                temperature: 0.0,
                humidity: 0.0,
                biome: BiomeType::Plains,
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

    // Advanced Phase 6 procedural systems
    pub climate: ClimateGenerator,
    pub caves: CaveGenerator,
    pub strata: StrataGenerator,

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
            macro_amplitude: 8.0,
            macro_frequency: 0.008,

            // Local terrain relief
            detail_amplitude: 3.0,
            detail_frequency: 0.038,
            detail_octaves: 3,
            persistence: 0.5,

            // Sea level in voxels
            sea_level: 12,

            river_frequency: 0.0035,
            river_width: 0.040,

            climate: ClimateGenerator::default(),
            caves: CaveGenerator::default(),
            strata: StrataGenerator::default(),

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

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = chunk_origin.x + x as i32;
                let world_z = chunk_origin.z + z as i32;

                let column = self.sample_column(world_x, world_z);
                columns[column_index(x, z)] = column;

                let col_top = column.terrain_height;

                let filled_height = column
                    .water_level
                    .unwrap_or(col_top)
                    .max(col_top);

                maximum_filled_height = maximum_filled_height.max(filled_height);
            }
        }

        // Entire chunk is above terrain and water
        if chunk_min_y > maximum_filled_height {
            return Chunk::filled(Voxel::Air);
        }

        let mut voxels = vec![Voxel::Air; CHUNK_VOLUME];
        let mut chunk_shapes: Vec<(usize, BlockShape, u8)> = Vec::new();

        if chunk_min_y <= maximum_filled_height {
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

                            if world_y == column.terrain_height && column.surface_shape != BlockShape::Full {
                                chunk_shapes.push((idx, column.surface_shape, column.shape_orientation));
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
                    voxels_buf[lx as usize + lz as usize * CHUNK_SIZE + ly as usize * CHUNK_SIZE * CHUNK_SIZE]
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
                        if !voxel.is_collidable() || voxel == Voxel::Dreadstone {
                            continue;
                        }

                        let world_x = chunk_origin.x + x as i32;
                        let world_z = chunk_origin.z + z as i32;

                        let above = get_v(&voxels, x as i32, y as i32 + 1, z as i32);
                        if above == Voxel::Air {
                            // Cave floor / shelf: use bottom slabs on gentle 1-block steps
                            let d_west = if !get_v(&voxels, x as i32 - 1, y as i32, z as i32).is_collidable() {
                                if get_v(&voxels, x as i32 - 1, y as i32 - 1, z as i32).is_collidable() { -1 } else { -2 }
                            } else { 0 };

                            let d_east = if !get_v(&voxels, x as i32 + 1, y as i32, z as i32).is_collidable() {
                                if get_v(&voxels, x as i32 + 1, y as i32 - 1, z as i32).is_collidable() { -1 } else { -2 }
                            } else { 0 };

                            let d_north = if !get_v(&voxels, x as i32, y as i32, z as i32 - 1).is_collidable() {
                                if get_v(&voxels, x as i32, y as i32 - 1, z as i32 - 1).is_collidable() { -1 } else { -2 }
                            } else { 0 };

                            let d_south = if !get_v(&voxels, x as i32, y as i32, z as i32 + 1).is_collidable() {
                                if get_v(&voxels, x as i32, y as i32 - 1, z as i32 + 1).is_collidable() { -1 } else { -2 }
                            } else { 0 };

                            let has_cliff = d_west <= -2 || d_east <= -2 || d_north <= -2 || d_south <= -2;
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
                            let air_west = get_v(&voxels, x as i32 - 1, y as i32, z as i32) == Voxel::Air;
                            let air_east = get_v(&voxels, x as i32 + 1, y as i32, z as i32) == Voxel::Air;
                            let air_north = get_v(&voxels, x as i32, y as i32, z as i32 - 1) == Voxel::Air;
                            let air_south = get_v(&voxels, x as i32, y as i32, z as i32 + 1) == Voxel::Air;

                            let wall_openings = air_west as u8 + air_east as u8 + air_north as u8 + air_south as u8;
                            let wall_noise = crate::core::noise::gradient_noise_3d(
                                world_x as f32 * 0.18,
                                world_y as f32 * 0.18,
                                world_z as f32 * 0.18,
                                self.seed.wrapping_add(44_221),
                            );

                            if wall_openings == 1 {
                                let (air_dx, air_dz, stair_orient, inv_stair_orient, vert_slab_orient) = if air_west {
                                    (-1, 0, 0, 4, 5) // air is -X, slab attached to +X rock wall -> orient 5
                                } else if air_east {
                                    (1, 0, 1, 5, 4)  // air is +X, slab attached to -X rock wall -> orient 4
                                } else if air_north {
                                    (0, -1, 2, 6, 3) // air is -Z, slab attached to +Z rock wall -> orient 3
                                } else {
                                    (0, 1, 3, 7, 2)  // air is +Z, slab attached to -Z rock wall -> orient 2
                                };

                                let below_cave_floor = get_v(&voxels, x as i32 + air_dx, y as i32 - 1, z as i32 + air_dz).is_collidable();
                                let above_cave_roof = get_v(&voxels, x as i32 + air_dx, y as i32 + 1, z as i32 + air_dz).is_collidable();

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
                                        chunk_shapes.push((idx, BlockShape::Stair, inv_stair_orient));
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

    pub fn continuous_height_and_biome(&self, world_x: i32, world_z: i32) -> (f32, BiomeType, bool) {
        let logical_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS);
        let logical_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS);
        let sample_x = logical_block_sample_position(logical_x);
        let sample_z = logical_block_sample_position(logical_z);

        let mut climate = self.climate.sample(sample_x, sample_z, self.seed);
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
                let target_height = lerp(
                    raw_height,
                    river_bed,
                    (river_factor * 1.25).min(1.0),
                );
                raw_height = raw_height.min(target_height);
                climate.biome = BiomeType::River;
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

        // Transitions on dry land above water level (terrain at or below sea_level remains strictly full blocks)
        if water_level.is_none() && terrain_height >= sea_level + 1 {
            let get_effective_height = |(h, _, _): (f32, BiomeType, bool)| -> i32 {
                let th = h.round() as i32;
                if th < sea_level {
                    sea_level
                } else {
                    th
                }
            };

            let h_east = get_effective_height(self.continuous_height_and_biome(world_x + 1, world_z));
            let h_west = get_effective_height(self.continuous_height_and_biome(world_x - 1, world_z));
            let h_south = get_effective_height(self.continuous_height_and_biome(world_x, world_z + 1));
            let h_north = get_effective_height(self.continuous_height_and_biome(world_x, world_z - 1));

            let d_east = h_east - terrain_height;
            let d_west = h_west - terrain_height;
            let d_south = h_south - terrain_height;
            let d_north = h_north - terrain_height;

            let has_cliff = d_east <= -2 || d_west <= -2 || d_south <= -2 || d_north <= -2;

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
                        .abs() % 100) as f32 / 100.0;

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
            is_underground_river,
            climate: self.climate.sample(
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
        if matches!(
            biome,
            BiomeType::SnowyTundra | BiomeType::Highlands | BiomeType::DeepOcean
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

    fn surface_material_at(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        column: TerrainColumn,
    ) -> Voxel {
        // Submerged terrain (underwater) is NEVER Grass or SnowyGrass!
        let is_submerged = column.water_level.is_some_and(|wl| world_y <= wl);
        if is_submerged {
            return Voxel::Sand;
        }

        if column.is_beach {
            return Voxel::Sand;
        }

        // High alpine elevation snowline: mountains above y >= 48 receive snowcaps
        if column.biome != BiomeType::Desert
            && column.biome != BiomeType::Ocean
            && column.biome != BiomeType::DeepOcean
            && column.biome != BiomeType::River
        {
            let snowline_jitter = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.05,
                world_z as f32 * 0.05,
                self.seed.wrapping_add(91_111),
            ) * 3.0;
            let snowline = 48.0 + snowline_jitter;

            if world_y as f32 >= snowline {
                let slope_noise = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.15,
                    world_z as f32 * 0.15,
                    self.seed.wrapping_add(82_222),
                );
                if world_y as f32 >= snowline + 8.0 && slope_noise > 0.40 {
                    return Voxel::Stone;
                }
                return Voxel::Snow;
            }
        }

        // 1. Organic Frigid Transition (Snowy Grass <-> Grass):
        // Nominal Snowy Tundra threshold is temperature < -0.20.
        // Multi-frequency 2D noise blends snow patches and grass tongues smoothly across the transition.
        let snow_noise_macro = crate::core::noise::gradient_noise_2d(
            world_x as f32 * 0.08,
            world_z as f32 * 0.08,
            self.seed.wrapping_add(14_337),
        );
        let snow_noise_micro = crate::core::noise::gradient_noise_2d(
            world_x as f32 * 0.25,
            world_z as f32 * 0.25,
            self.seed.wrapping_add(28_991),
        );
        let snow_jitter = snow_noise_macro * 0.045 + snow_noise_micro * 0.025;
        let is_snowy = column.climate.temperature + snow_jitter < -0.20;

        if is_snowy {
            return Voxel::SnowyGrass;
        }

        // 2. Organic Arid Transition (Sand <-> Grass for Desert):
        // Nominal Desert is temperature > 0.20 && humidity < -0.05.
        // We compute distance to the desert boundary and blend with organic noise so sand dunes taper naturally.
        let temp_dist = column.climate.temperature - 0.20;
        let hum_dist = -0.05 - column.climate.humidity;
        let desert_margin = temp_dist.min(hum_dist);

        let desert_noise_macro = crate::core::noise::gradient_noise_2d(
            world_x as f32 * 0.09,
            world_z as f32 * 0.09,
            self.seed.wrapping_add(33_881),
        );
        let desert_noise_micro = crate::core::noise::gradient_noise_2d(
            world_x as f32 * 0.26,
            world_z as f32 * 0.26,
            self.seed.wrapping_add(51_223),
        );
        let desert_jitter = desert_noise_macro * 0.035 + desert_noise_micro * 0.020;
        let is_desert = (desert_margin + desert_jitter) > 0.0;

        if is_desert
            || column.biome == BiomeType::Beach
            || column.biome == BiomeType::Ocean
            || column.biome == BiomeType::DeepOcean
            || column.biome == BiomeType::River
        {
            return Voxel::Sand;
        }

        if column.biome == BiomeType::Highlands {
            let slate_noise = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.15,
                world_z as f32 * 0.15,
                self.seed.wrapping_add(45_678),
            );
            if slate_noise > 0.05 {
                return Voxel::Slate;
            } else if slate_noise > -0.25 {
                return Voxel::Cobbleslate;
            } else {
                return Voxel::Grass;
            }
        }

        Voxel::Grass
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
                    return Voxel::Water;
                } else {
                    return Voxel::Air;
                }
            }
        }

        if world_y > column.terrain_height {
            if let Some(water_level) = column.water_level
                && world_y <= water_level
            {
                return Voxel::Water;
            }

            return Voxel::Air;
        }

        // Absolute bedrock floor: bottom layers of the world are strictly solid Dreadstone
        if world_y <= self.strata.bedrock_min_block_y {
            return Voxel::Dreadstone;
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
            return Voxel::Sand;
        }

        if logical_depth <= 2 {
            let surface = self.surface_material_at(world_x, column.terrain_height, world_z, column);
            if surface == Voxel::Sand {
                return Voxel::Sand;
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
            return Voxel::Sand;
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
            (-1.00, -32.0), // Deep abyssal trench
            (-0.55, -24.0), // Deep ocean basin
            (-0.25, -16.0), // Open ocean floor
            (-0.08, -7.0),  // Continental shelf / shallow coastal waters
            (0.00, -4.0),   // Coastline / beach (aligns with sea_level = 12 at base_height = 16)
            (0.10, 4.0),    // Low coastal plains (y = 20)
            (0.22, 14.0),   // Inland rolling plains (y = 30)
            (0.35, 36.0),   // Highlands & foothills (y = 52)
            (0.50, 72.0),   // Rugged mountain chains (y = 88)
            (0.70, 118.0),  // Grand alpine peaks (y = 134)
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
        let ridge = (1.0 - macro_noise.abs()).powi(2) * 48.0 * mountain_factor;

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
        let amplitude = (self.macro_amplitude * macro_noise + self.detail_amplitude * detail_noise)
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

