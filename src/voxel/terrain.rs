use bevy::prelude::*;

use super::{
    biome::{BiomeType, ClimateGenerator, ClimateSample},
    blocks::Voxel,
    caves::CaveGenerator,
    chunk::{CHUNK_SIZE, Chunk},
    strata::StrataGenerator,
};

pub const LOGICAL_BLOCK_VOXELS: i32 = 2;

const BEACH_HEIGHT: i32 = 2;

const LAKE_WATER_THRESHOLD: f32 = 0.20;
const LAKE_MATERIAL_THRESHOLD: f32 = 0.05;

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

    // Lakes.
    pub sea_level: i32,
    pub lake_frequency: f32,
    pub lake_threshold: f32,
    pub lake_transition: f32,
    pub lake_max_depth: f32,

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

            // 13 voxels = 6.5 meters baseline.
            base_height: 13.0,

            // Broad hills and plains.
            macro_amplitude: 7.0,
            macro_frequency: 0.010,

            // Smaller local variation.
            detail_amplitude: 2.5,
            detail_frequency: 0.045,
            detail_octaves: 3,
            persistence: 0.5,

            // 9 voxels = 4.5 meters (aligns with top of logical block 4 at y=9).
            sea_level: 9,

            lake_frequency: 0.008,
            lake_threshold: 0.10,
            lake_transition: 0.45,
            lake_max_depth: 10.0,

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

        // Entire chunk is above both terrain and any possible water surface.
        if chunk_min_y > maximum_filled_height {
            return Chunk::filled(Voxel::Air);
        }

        let mut chunk = Chunk::new();

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let column = columns[column_index(x, z)];

                for y in 0..CHUNK_SIZE {
                    let world_y = chunk_origin.y + y as i32;
                    let world_x = chunk_origin.x + x as i32;
                    let world_z = chunk_origin.z + z as i32;

                    let voxel = self.voxel_at(column, world_x, world_y, world_z);
                    if voxel != Voxel::Air {
                        chunk.set(x, y, z, voxel);
                    }
                }
            }
        }

        chunk
    }

    pub fn effective_sea_level(&self) -> i32 {
        logical_block_top(self.sea_level)
    }

    pub fn sample_column(&self, world_x: i32, world_z: i32) -> TerrainColumn {
        let logical_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS);
        let logical_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS);
        let sample_x = logical_block_sample_position(logical_x);
        let sample_z = logical_block_sample_position(logical_z);

        let climate = self.climate.sample(sample_x, sample_z, self.seed);
        let terrain_height = self.terrain_height_at(world_x as f32, world_z as f32, &climate);
        let lake_strength = self.lake_strength_at(world_x as f32, world_z as f32);
        let sea_level = self.effective_sea_level();

        let water_level = if terrain_height < sea_level && lake_strength >= LAKE_WATER_THRESHOLD {
            Some(sea_level)
        } else {
            None
        };

        let is_beach = self.is_beach_at(world_x, world_z, climate.biome);

        TerrainColumn {
            terrain_height,
            water_level,
            material_terrain_height: self.logical_terrain_height_at(logical_x, logical_z, &climate),
            biome: climate.biome,
            is_beach,
        }
    }

    fn is_beach_at(&self, world_x: i32, world_z: i32, biome: BiomeType) -> bool {
        // High mountain peaks and frozen tundras do not generate sandy beaches
        if matches!(biome, BiomeType::SnowyTundra | BiomeType::Highlands) {
            return false;
        }

        let logical_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS);
        let logical_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS);

        let sample_x = logical_block_sample_position(logical_x);
        let sample_z = logical_block_sample_position(logical_z);

        let lake_strength = self.lake_strength_at(sample_x, sample_z);
        let climate = self.climate.sample(sample_x, sample_z, self.seed);
        let representative_height = self.terrain_height_at(sample_x, sample_z, &climate);

        let near_lake = lake_strength > LAKE_MATERIAL_THRESHOLD;
        near_lake && representative_height <= self.effective_sea_level() + BEACH_HEIGHT
    }

    fn voxel_at(&self, column: TerrainColumn, world_x: i32, world_y: i32, world_z: i32) -> Voxel {
        if world_y > column.terrain_height {
            if let Some(water_level) = column.water_level
                && world_y <= water_level
            {
                return Voxel::Water;
            }

            return Voxel::Air;
        }

        // 3D Cave carving: check if this subterranean position is hollowed out by caves
        if self
            .caves
            .is_cave(world_x, world_y, world_z, column.terrain_height, self.seed)
        {
            return self.caves.cave_voxel(
                world_x,
                world_y,
                world_z,
                self.effective_sea_level(),
                self.seed,
            );
        }

        // Solid subterranean ground: evaluate geological strata and surface layers
        let logical_block_top = logical_block_top(world_y);
        let logical_depth =
            (column.material_terrain_height - logical_block_top) / LOGICAL_BLOCK_VOXELS;

        let biome_cfg = column.biome.config();

        if column.is_beach && logical_depth <= 2 {
            return Voxel::Sand;
        }

        let solid_voxel = self.strata.solid_voxel_at(
            world_x,
            world_y,
            world_z,
            logical_depth,
            &biome_cfg,
            self.seed,
        );

        // Check if exposed subsoil (e.g. Dirt) on hill slopes should be promoted to surface grass / snow
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

    pub fn surface_voxel(&self, column: TerrainColumn, world_y: i32) -> Voxel {
        let logical_block_top = logical_block_top(world_y);
        let logical_depth =
            (column.material_terrain_height - logical_block_top) / LOGICAL_BLOCK_VOXELS;
        let biome_cfg = column.biome.config();

        if column.is_beach && logical_depth <= 2 {
            return Voxel::Sand;
        }

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
        // If the top of this logical block is deeper than 2 voxels below the column surface,
        // it cannot be exposed to surface air.
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

        let biome_cfg = climate.biome.config();

        // Non-linear continentalness lift for towering highlands and peaks
        let continental_factor = (climate.continentalness - 0.10).max(0.0) / 0.90;
        let mountain_lift = continental_factor.powf(1.3) * 32.0;

        // Sharp jagged mountain ridges when in high terrain
        let ridge = (1.0 - macro_noise.abs()).powi(2) * 18.0 * continental_factor;

        let base = self.base_height + biome_cfg.base_height_offset + mountain_lift;
        let amplitude = (self.macro_amplitude * macro_noise + self.detail_amplitude * detail_noise)
            * biome_cfg.amplitude_multiplier
            + ridge;

        base + amplitude
    }

    fn terrain_height_at(&self, world_x: f32, world_z: f32, climate: &ClimateSample) -> i32 {
        let natural_height = self.natural_height_at(world_x, world_z, climate);
        let lake_strength = self.lake_strength_at(world_x, world_z);

        if lake_strength > 0.0 {
            let deepest_floor = self.effective_sea_level() as f32 - self.lake_max_depth;
            lerp(natural_height, deepest_floor, lake_strength).round() as i32
        } else {
            natural_height.round() as i32
        }
    }

    fn lake_strength_at(&self, world_x: f32, world_z: f32) -> f32 {
        let lake_noise = fractal_noise(
            world_x,
            world_z,
            self.lake_frequency,
            2,
            0.55,
            self.seed.wrapping_add(420_911),
        );

        smooth_range(
            lake_noise,
            self.lake_threshold,
            self.lake_threshold + self.lake_transition,
        )
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
                                    } else if voxel != Voxel::Water {
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

        // Must be an odd index so both bottom (sea_level - 1) and top (sea_level) are within the 1m block
        assert_eq!(sea_level % 2, 1, "sea level must be top of logical block");
    }

    #[test]
    fn caves_create_subterranean_cavities() {
        let generator = TerrainGenerator::default();
        let mut found_cave_air = false;

        // Sample subterranean coordinates across several chunks below sea level
        'outer: for y in -40..0 {
            for z in -32..32 {
                for x in -32..32 {
                    let column = generator.sample_column(x, z);
                    if y < column.terrain_height - 6
                        && generator
                            .caves
                            .is_cave(x, y, z, column.terrain_height, generator.seed)
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

        // Sample a wide area to find mountain summits
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
}
