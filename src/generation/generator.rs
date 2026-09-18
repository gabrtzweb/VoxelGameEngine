use bevy::prelude::*;

use super::{
    biome::{BiomeType, ClimateGenerator, ClimateSample},
    caves::{CaveGenerator, CaveNoiseSample},
    strata::StrataGenerator,
    trees::generate_chunk_trees,
};
use crate::world::{CHUNK_SIZE, CHUNK_VOLUME, Chunk, Voxel};

pub const LOGICAL_BLOCK_VOXELS: i32 = 2;

#[derive(Clone, Copy, Debug)]
pub struct TerrainColumn {
    pub terrain_height: i32,
    pub water_level: Option<i32>,
    pub material_terrain_height: i32,
    pub biome: BiomeType,
    pub is_beach: bool,
}

impl Default for TerrainColumn {
    fn default() -> Self {
        Self {
            terrain_height: 0,
            water_level: None,
            material_terrain_height: 0,
            biome: BiomeType::Plains,
            is_beach: false,
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
            base_height: 10.0,

            // Broad continental landforms
            macro_amplitude: 8.0,
            macro_frequency: 0.008,

            // Local terrain relief
            detail_amplitude: 3.0,
            detail_frequency: 0.038,
            detail_octaves: 3,
            persistence: 0.5,

            // 9 voxels = 4.5 meters (aligns with top of logical block 4 at y=9).
            sea_level: 9,

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

                let filled_height = column
                    .water_level
                    .unwrap_or(column.terrain_height)
                    .max(column.terrain_height);

                maximum_filled_height = maximum_filled_height.max(filled_height);
            }
        }

        // Entire chunk is above terrain, water, and possible tree trunks (max trunk height 20 voxels).
        if chunk_min_y > maximum_filled_height + 24 {
            return Chunk::filled(Voxel::Air);
        }

        let mut voxels = vec![Voxel::Air; CHUNK_VOLUME];

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
                        let voxel = self.voxel_at_sampled(
                            column,
                            world_x,
                            world_y,
                            world_z,
                            cave_sample,
                        );
                        if voxel != Voxel::Air {
                            let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                            voxels[idx] = voxel;
                        }
                    }
                }
            }
        }

        // Post-terrain pass: Generate tree trunks
        generate_chunk_trees(chunk_origin, self, &mut voxels);

        Chunk::from_voxels(voxels)
    }

    pub fn effective_sea_level(&self) -> i32 {
        logical_block_top(self.sea_level)
    }

    pub fn sample_column(&self, world_x: i32, world_z: i32) -> TerrainColumn {
        let logical_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS);
        let logical_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS);
        let sample_x = logical_block_sample_position(logical_x);
        let sample_z = logical_block_sample_position(logical_z);

        let mut climate = self.climate.sample(sample_x, sample_z, self.seed);
        let sea_level = self.effective_sea_level();

        let mut terrain_height = self.terrain_height_at(world_x as f32, world_z as f32, &climate);

        let (river_factor, is_river_path) = self.river_sample(sample_x, sample_z);
        let is_inland = climate.continentalness >= -0.05;
        let is_river = is_river_path && is_inland && river_factor > 0.05;

        if is_river {
            let river_bed = sea_level - 4;
            let target_height = lerp(
                terrain_height as f32,
                river_bed as f32,
                (river_factor * 1.25).min(1.0),
            )
            .round() as i32;
            terrain_height = terrain_height.min(target_height);
            climate.biome = BiomeType::River;
        }

        let water_level = if terrain_height < sea_level {
            Some(sea_level)
        } else {
            None
        };

        let is_beach = self.is_beach_at(logical_x, logical_z, climate.biome, sea_level, &climate);
        let final_biome = if is_beach && !is_river {
            BiomeType::Beach
        } else {
            climate.biome
        };

        TerrainColumn {
            terrain_height,
            water_level,
            material_terrain_height: self.logical_terrain_height_at(logical_x, logical_z, &climate),
            biome: final_biome,
            is_beach,
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
        rep_height >= sea_level - 1 && rep_height <= sea_level + 3
    }

    fn surface_material_at(
        &self,
        world_x: i32,
        _world_y: i32,
        world_z: i32,
        column: TerrainColumn,
    ) -> Voxel {
        if column.is_beach {
            return Voxel::Sand;
        }

        let bx = world_x.div_euclid(LOGICAL_BLOCK_VOXELS) as f32 + 0.5;
        let bz = world_z.div_euclid(LOGICAL_BLOCK_VOXELS) as f32 + 0.5;
        let surface_noise = crate::core::noise::gradient_noise_2d(
            bx * 0.12,
            bz * 0.12,
            self.seed.wrapping_add(88_221),
        );

        match column.biome {
            BiomeType::Woodland => {
                // Mix Grass (50%), Mulch (30%), Packed Dirt (10%), Moss (10%)
                if surface_noise > 0.35 {
                    Voxel::Mulch
                } else if surface_noise > 0.18 {
                    Voxel::PackedDirt
                } else if surface_noise < -0.38 {
                    Voxel::Moss
                } else {
                    Voxel::Grass
                }
            }
            BiomeType::Wetlands => {
                // Mix Grass (40%), Mud (35%), Packed Mud (15%), Clay (10%)
                if surface_noise > 0.25 {
                    Voxel::Mud
                } else if surface_noise > 0.05 {
                    Voxel::PackedMud
                } else if surface_noise < -0.35 {
                    Voxel::Clay
                } else {
                    Voxel::Grass
                }
            }
            BiomeType::Plains | BiomeType::Meadow | BiomeType::PlainsForest => {
                if surface_noise > 0.46 {
                    Voxel::PackedDirt
                } else if surface_noise < -0.46 {
                    Voxel::Moss
                } else {
                    Voxel::Grass
                }
            }
            BiomeType::Beach => Voxel::Sand,
            BiomeType::Desert => {
                if surface_noise > 0.42 {
                    Voxel::RedSand
                } else {
                    Voxel::Sand
                }
            }
            BiomeType::Ocean => Voxel::Sand,
            BiomeType::DeepOcean => Voxel::Gravel,
            BiomeType::River => {
                if surface_noise > 0.20 {
                    Voxel::Gravel
                } else if surface_noise < -0.20 {
                    Voxel::Clay
                } else {
                    Voxel::Sand
                }
            }
            BiomeType::SnowyTundra => {
                if surface_noise > 0.40 {
                    Voxel::Stone
                } else {
                    Voxel::Snow
                }
            }
            BiomeType::Highlands => {
                if surface_noise > 0.35 {
                    Voxel::Stone
                } else if surface_noise < -0.35 {
                    Voxel::Cobbleslate
                } else {
                    Voxel::Slate
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
        if world_y > column.terrain_height {
            if let Some(water_level) = column.water_level
                && world_y <= water_level
            {
                return Voxel::Water;
            }

            return Voxel::Air;
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
            return self.caves.cave_voxel_sampled(
                world_y,
                cave_sample,
                self.effective_sea_level(),
            );
        }

        let logical_block_top = logical_block_top(world_y);
        let logical_depth =
            (column.material_terrain_height - logical_block_top) / LOGICAL_BLOCK_VOXELS;

        let biome_cfg = column.biome.config();

        if column.is_beach && logical_depth <= 2 {
            return Voxel::Sand;
        }

        if logical_depth <= 0 {
            return self.surface_material_at(world_x, world_y, world_z, column);
        }

        let solid_voxel = self.strata.solid_voxel_at(
            world_x,
            world_y,
            world_z,
            logical_depth,
            &biome_cfg,
            self.seed,
        );

        if solid_voxel == Voxel::Dirt
            && self.logical_block_has_exposed_dirt(column, world_x, world_y, world_z)
        {
            if column.biome == BiomeType::SnowyTundra {
                Voxel::Snow
            } else {
                Voxel::Grass
            }
        } else {
            solid_voxel
        }
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

    pub fn surface_voxel(&self, column: TerrainColumn, world_y: i32) -> Voxel {
        let logical_block_top = logical_block_top(world_y);
        let logical_depth =
            (column.material_terrain_height - logical_block_top) / LOGICAL_BLOCK_VOXELS;

        if column.is_beach && logical_depth <= 2 {
            return Voxel::Sand;
        }

        if logical_depth <= 0 {
            return self.surface_material_at(0, world_y, 0, column);
        }

        let biome_cfg = column.biome.config();
        self.strata
            .solid_voxel_at(0, world_y, 0, logical_depth, &biome_cfg, self.seed)
    }

    fn is_exposed_to_air(&self, world_x: i32, world_y: i32, world_z: i32) -> bool {
        [(0, 1, 0), (1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)]
            .into_iter()
            .any(|(offset_x, offset_y, offset_z)| {
                self.is_surface_air_at(world_x + offset_x, world_y + offset_y, world_z + offset_z)
            })
    }

    fn is_surface_air_at(&self, world_x: i32, world_y: i32, world_z: i32) -> bool {
        let column = self.sample_column(world_x, world_z);
        if world_y > column.terrain_height {
            let water_fills_voxel = column.water_level.is_some_and(|wl| world_y <= wl);
            !water_fills_voxel
        } else {
            false
        }
    }

    fn logical_block_has_exposed_dirt(
        &self,
        column: TerrainColumn,
        world_x: i32,
        world_y: i32,
        world_z: i32,
    ) -> bool {
        let block_origin_y = world_y.div_euclid(LOGICAL_BLOCK_VOXELS) * LOGICAL_BLOCK_VOXELS;
        if column.terrain_height - (block_origin_y + 1) > 2 {
            return false;
        }

        let block_origin_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS) * LOGICAL_BLOCK_VOXELS;
        let block_origin_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS) * LOGICAL_BLOCK_VOXELS;

        for local_y in (0..LOGICAL_BLOCK_VOXELS).rev() {
            let voxel_y = block_origin_y + local_y;
            for local_z in 0..LOGICAL_BLOCK_VOXELS {
                for local_x in 0..LOGICAL_BLOCK_VOXELS {
                    let voxel_x = block_origin_x + local_x;
                    let voxel_z = block_origin_z + local_z;
                    let col = if voxel_x == world_x && voxel_z == world_z {
                        column
                    } else {
                        self.sample_column(voxel_x, voxel_z)
                    };

                    if voxel_y <= col.terrain_height
                        && self.surface_voxel(col, voxel_y) == Voxel::Dirt
                        && self.is_exposed_to_air(voxel_x, voxel_y, voxel_z)
                    {
                        return true;
                    }
                }
            }
        }

        false
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
            (-1.00, -24.0), // Deep abyssal trench
            (-0.55, -20.0), // Deep ocean basin
            (-0.25, -12.0), // Open ocean floor
            (-0.08, -2.5),  // Continental shelf / shallow coastal waters
            (0.00, 1.5),    // Coastline / beach (right around sea level = 9.0)
            (0.12, 6.0),    // Low coastal plains
            (0.28, 14.0),   // Inland rolling plains
            (0.48, 26.0),   // Foothills & plateau
            (0.72, 48.0),   // Rugged mountain ranges
            (1.00, 72.0),   // Extreme alpine peaks
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

        let mountain_factor = ((climate.continentalness - 0.28) / 0.45).clamp(0.0, 1.0);
        let ridge = (1.0 - macro_noise.abs()).powi(2) * 22.0 * mountain_factor;

        let swamp_depression = if climate.continentalness > 0.02
            && climate.continentalness < 0.25
            && climate.humidity > 0.15
        {
            let wetness = ((climate.humidity - 0.15) / 0.20).clamp(0.0, 1.0);
            let inland_factor = (1.0 - ((climate.continentalness - 0.12).abs() / 0.12)).clamp(0.0, 1.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::IVec3;

    #[test]
    fn generated_logical_blocks_keep_terrain_materials_consistent_but_allow_slabs() {
        let generator = TerrainGenerator::default();
        let mut found_slab = false;

        for chunk_coordinate in [IVec3::ZERO, IVec3::NEG_ONE, IVec3::new(-1, 0, 1)] {
            let chunk = generator.generate_chunk(chunk_coordinate);

            for y in (0..CHUNK_SIZE).step_by(2) {
                for z in (0..CHUNK_SIZE).step_by(2) {
                    for x in (0..CHUNK_SIZE).step_by(2) {
                        let mut material = None;
                        let mut has_air = false;

                        for block_y in 0..2 {
                            for block_z in 0..2 {
                                for block_x in 0..2 {
                                    let voxel = chunk.get(x + block_x, y + block_y, z + block_z);

                                    if voxel == Voxel::Air {
                                        has_air = true;
                                    } else if voxel != Voxel::Water && voxel != Voxel::Occupied {
                                        assert!(
                                            material.is_none_or(|expected| expected == voxel),
                                            "mixed terrain materials at chunk {chunk_coordinate:?}, logical block ({x}, {y}, {z})"
                                        );
                                        material = Some(voxel);
                                    }
                                }
                            }
                        }

                        found_slab |= has_air && material.is_some();
                    }
                }
            }
        }

        assert!(found_slab, "expected native-resolution terrain slabs");
    }

    #[test]
    fn dirt_exposed_to_air_is_promoted_to_grass() {
        let generator = TerrainGenerator::default();

        for world_z in -32..=32 {
            for world_x in -32..=32 {
                let column = generator.sample_column(world_x, world_z);

                for world_y in (column.terrain_height - 3)..=column.terrain_height {
                    if generator.surface_voxel(column, world_y) != Voxel::Dirt
                        || !generator.is_exposed_to_air(world_x, world_y, world_z)
                    {
                        continue;
                    }

                    assert_eq!(
                        generator.voxel_at(column, world_x, world_y, world_z),
                        Voxel::Grass
                    );
                    return;
                }
            }
        }

        panic!("expected at least one exposed Dirt voxel");
    }

    #[test]
    fn sea_level_aligns_with_full_logical_block() {
        let generator = TerrainGenerator::default();
        let sea_level = generator.effective_sea_level();

        assert_eq!(sea_level % 2, 1, "sea level must be top of logical block");
    }

    #[test]
    fn caves_create_subterranean_cavities() {
        let generator = TerrainGenerator::default();
        let mut found_cave_air = false;

        'outer: for y in -40..0 {
            for z in -32..32 {
                for x in -32..32 {
                    let column = generator.sample_column(x, z);
                    if y < column.terrain_height - 6
                        && generator.caves.is_cave(
                            x,
                            y,
                            z,
                            column.terrain_height,
                            false,
                            generator.effective_sea_level(),
                            generator.seed,
                        )
                    {
                        let fill = generator.caves.cave_voxel(
                            x,
                            y,
                            z,
                            generator.effective_sea_level(),
                            generator.seed,
                        );
                        if fill == Voxel::Air {
                            found_cave_air = true;
                            break 'outer;
                        }
                    }
                }
            }
        }

        assert!(found_cave_air, "expected dry air in 3D caves");
    }

    #[test]
    fn mountain_highlands_produce_significant_elevation() {
        let generator = TerrainGenerator::default();
        let mut max_height = 0;

        for z in (-200..200).step_by(10) {
            for x in (-200..200).step_by(10) {
                let column = generator.sample_column(x, z);
                max_height = max_height.max(column.terrain_height);
            }
        }

        assert!(
            max_height >= 40,
            "expected mountain summits to reach at least 40 voxels, got {max_height}"
        );
    }

    #[test]
    fn oceans_and_deep_oceans_generate_proper_depth() {
        let generator = TerrainGenerator::default();
        let sea_level = generator.effective_sea_level();

        let mut found_deep_ocean = false;
        let mut found_ocean = false;

        for z in (-600..600).step_by(25) {
            for x in (-600..600).step_by(25) {
                let col = generator.sample_column(x, z);
                if col.biome == BiomeType::DeepOcean {
                    found_deep_ocean = true;
                    assert!(col.water_level.is_some());
                    assert!(col.terrain_height < sea_level - 10);
                } else if col.biome == BiomeType::Ocean {
                    found_ocean = true;
                    assert!(col.water_level.is_some());
                    assert!(col.terrain_height < sea_level);
                }
            }
        }

        assert!(found_deep_ocean, "expected DeepOcean biomes to generate");
        assert!(found_ocean, "expected Ocean biomes to generate");
    }

    #[test]
    fn woodland_surface_mixes_grass_with_mulch_and_dirt() {
        let generator = TerrainGenerator::default();
        let mut found_grass = false;
        let mut found_mulch = false;

        for z in (-400..400).step_by(10) {
            for x in (-400..400).step_by(10) {
                let col = generator.sample_column(x, z);
                if col.biome == BiomeType::Woodland {
                    let surface = generator.surface_material_at(x, col.terrain_height, z, col);
                    if surface == Voxel::Grass {
                        found_grass = true;
                    } else if surface == Voxel::Mulch {
                        found_mulch = true;
                    }
                }
            }
        }

        assert!(
            found_grass && found_mulch,
            "Woodland must mix Grass and Mulch"
        );
    }

    #[test]
    fn wetlands_surface_mixes_grass_with_mud() {
        let generator = TerrainGenerator::default();
        let mut found_grass = false;
        let mut found_mud = false;

        for z in (-400..400).step_by(10) {
            for x in (-400..400).step_by(10) {
                let col = generator.sample_column(x, z);
                if col.biome == BiomeType::Wetlands {
                    let surface = generator.surface_material_at(x, col.terrain_height, z, col);
                    if surface == Voxel::Grass {
                        found_grass = true;
                    } else if surface == Voxel::Mud {
                        found_mud = true;
                    }
                }
            }
        }

        assert!(found_grass && found_mud, "Wetlands must mix Grass and Mud");
    }
}
