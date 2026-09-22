use bevy::prelude::*;

use super::{
    biome::{BiomeType, ClimateGenerator, ClimateSample},
    caves::{CaveGenerator, CaveNoiseSample},
    strata::StrataGenerator,
};
use crate::world::{CHUNK_SIZE, CHUNK_VOLUME, Chunk, Voxel};

pub const LOGICAL_BLOCK_VOXELS: i32 = 1;

#[derive(Clone, Copy, Debug)]
pub struct TerrainColumn {
    pub terrain_height: i32,
    pub water_level: Option<i32>,
    pub biome: BiomeType,
    pub is_beach: bool,
    pub is_underground_river: bool,
    pub climate: ClimateSample,
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

                let filled_height = column
                    .water_level
                    .unwrap_or(column.terrain_height)
                    .max(column.terrain_height);

                maximum_filled_height = maximum_filled_height.max(filled_height);
            }
        }

        // Entire chunk is above terrain and water
        if chunk_min_y > maximum_filled_height {
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
                        let voxel =
                            self.voxel_at_sampled(column, world_x, world_y, world_z, cave_sample);
                        if voxel != Voxel::Air {
                            let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                            voxels[idx] = voxel;
                        }
                    }
                }
            }
        }

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
        let mut is_underground_river = false;

        if is_river {
            let initial_height = terrain_height;
            // If mountain is high, keep peak intact and let the river flow underneath!
            if initial_height > sea_level + 14 {
                is_underground_river = true;
            } else {
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
            biome: final_biome,
            is_beach,
            is_underground_river,
            climate,
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::IVec3;

    #[test]
    fn generated_chunks_produce_valid_terrain_blocks() {
        let generator = TerrainGenerator::default();
        for chunk_coordinate in [IVec3::ZERO, IVec3::NEG_ONE, IVec3::new(-1, 0, 1)] {
            let chunk = generator.generate_chunk(chunk_coordinate);
            let mut solid_count = 0;
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let voxel = chunk.get(x, y, z);
                        if !voxel.is_empty() && voxel != Voxel::Water {
                            solid_count += 1;
                        }
                    }
                }
            }
            if chunk_coordinate.y <= 0 {
                assert!(
                    solid_count > 0,
                    "Expected subterranean chunks to have solid blocks"
                );
            }
        }
    }

    #[test]
    fn subsoil_is_never_grass_and_underwater_is_never_grass() {
        let generator = TerrainGenerator::default();

        for world_z in (-100..=100).step_by(10) {
            for world_x in (-100..=100).step_by(10) {
                let column = generator.sample_column(world_x, world_z);

                // Check that submerged surface is never grass
                if column.water_level.is_some() {
                    let surface = generator.surface_material_at(
                        world_x,
                        column.terrain_height,
                        world_z,
                        column,
                    );
                    assert_ne!(
                        surface,
                        Voxel::Grass,
                        "Underwater surface must never be Grass"
                    );
                    assert_ne!(
                        surface,
                        Voxel::SnowyGrass,
                        "Underwater surface must never be SnowyGrass"
                    );
                }

                // Check that subsurface depth > 0 is never grass
                for world_y in (column.terrain_height - 6)..column.terrain_height {
                    let voxel = generator.voxel_at(column, world_x, world_y, world_z);
                    assert_ne!(voxel, Voxel::Grass, "Subsurface voxels must never be Grass");
                    assert_ne!(
                        voxel,
                        Voxel::SnowyGrass,
                        "Subsurface voxels must never be SnowyGrass"
                    );
                }
            }
        }
    }

    #[test]
    fn sea_level_aligns_with_full_logical_block() {
        let generator = TerrainGenerator::default();
        let sea_level = generator.effective_sea_level();

        assert_eq!(sea_level, 12);
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

        for z in (-1500..1500).step_by(30) {
            for x in (-1500..1500).step_by(30) {
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
                if found_deep_ocean && found_ocean {
                    break;
                }
            }
            if found_deep_ocean && found_ocean {
                break;
            }
        }

        assert!(found_deep_ocean, "expected DeepOcean biomes to generate");
        assert!(found_ocean, "expected Ocean biomes to generate");
    }

    #[test]
    fn plains_and_snowy_tundra_surfaces() {
        let generator = TerrainGenerator::default();
        let mut found_grass = false;
        let mut found_snow = false;

        for z in (-400..400).step_by(10) {
            for x in (-400..400).step_by(10) {
                let col = generator.sample_column(x, z);
                if col.biome == BiomeType::Plains {
                    let surface = generator.surface_material_at(x, col.terrain_height, z, col);
                    if surface == Voxel::Grass {
                        found_grass = true;
                    }
                } else if col.biome == BiomeType::SnowyTundra {
                    let surface = generator.surface_material_at(x, col.terrain_height, z, col);
                    if surface == Voxel::SnowyGrass {
                        found_snow = true;
                    }
                }
            }
        }

        assert!(found_grass, "Plains must generate Grass surface");
        assert!(found_snow, "Snowy Tundra must generate SnowyGrass surface");
    }

    #[test]
    fn desert_surface_generates_sand() {
        let generator = TerrainGenerator::default();
        let mut found_sand = false;

        for z in (-1500..1500).step_by(30) {
            for x in (-1500..1500).step_by(30) {
                let col = generator.sample_column(x, z);
                if col.biome == BiomeType::Desert {
                    let surface = generator.surface_material_at(x, col.terrain_height, z, col);
                    if surface == Voxel::Sand {
                        found_sand = true;
                        break;
                    }
                }
            }
            if found_sand {
                break;
            }
        }

        assert!(found_sand, "Desert must generate Sand surface");
    }

    #[test]
    fn biome_material_transition_blending_produces_organic_fringe() {
        let generator = TerrainGenerator::default();
        let mut found_snowy_in_transition = false;
        let mut found_grass_in_transition = false;

        for z in (-800..800).step_by(8) {
            for x in (-800..800).step_by(8) {
                let col = generator.sample_column(x, z);
                if col.water_level.is_none() && !col.is_beach {
                    let temp = col.climate.temperature;
                    if (-0.25..-0.15).contains(&temp) {
                        let surface = generator.surface_material_at(x, col.terrain_height, z, col);
                        if surface == Voxel::SnowyGrass {
                            found_snowy_in_transition = true;
                        } else if surface == Voxel::Grass {
                            found_grass_in_transition = true;
                        }
                    }
                }
                if found_snowy_in_transition && found_grass_in_transition {
                    break;
                }
            }
            if found_snowy_in_transition && found_grass_in_transition {
                break;
            }
        }

        assert!(
            found_snowy_in_transition && found_grass_in_transition,
            "Expected both SnowyGrass and Grass to organically coexist in transition zone"
        );
    }
}
