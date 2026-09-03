use bevy::prelude::*;

use super::chunk::{CHUNK_SIZE, Chunk, Voxel};

const GRASS_DIRT_DEPTH: i32 = 3;
const SAND_DEPTH: i32 = 4;
const BEACH_HEIGHT: i32 = 2;

const MAX_SURFACE_LAYER_DEPTH: i32 = SAND_DEPTH;

#[derive(Clone, Copy)]
struct TerrainColumn {
    terrain_height: i32,
    water_level: Option<i32>,
    lake_strength: f32,
}

impl Default for TerrainColumn {
    fn default() -> Self {
        Self {
            terrain_height: 0,
            water_level: None,
            lake_strength: 0.0,
        }
    }
}

#[derive(Resource, Clone)]
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

    // Maximum depth below the water surface,
    // expressed in 0.5 m voxels.
    pub lake_max_depth: f32,
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self {
            seed: 1337,

            // 13 voxels = 6.5 meters.
            base_height: 13.0,

            // Broad hills and plains.
            macro_amplitude: 7.0,
            macro_frequency: 0.010,

            // Smaller local variation.
            detail_amplitude: 2.5,
            detail_frequency: 0.045,
            detail_octaves: 3,
            persistence: 0.5,

            // 8 voxels = 4 meters.
            sea_level: 8,

            // Very low frequency so lakes become
            // wide features instead of tiny puddles.
            lake_frequency: 0.008,

            // Lake noise begins carving here.
            lake_threshold: 0.10,

            // Controls how gradually the shore
            // transitions into the deep basin.
            lake_transition: 0.45,

            // 10 voxels = up to roughly 5 meters
            // below the water surface.
            lake_max_depth: 10.0,
        }
    }
}

impl TerrainGenerator {
    pub fn generate_chunk(&self, chunk_coordinate: IVec3) -> Chunk {
        let chunk_origin = chunk_coordinate * CHUNK_SIZE as i32;

        let chunk_min_y = chunk_origin.y;

        let chunk_max_y = chunk_origin.y + CHUNK_SIZE as i32 - 1;

        let mut columns = [TerrainColumn::default(); CHUNK_SIZE * CHUNK_SIZE];

        let mut minimum_terrain_height = i32::MAX;

        let mut maximum_filled_height = i32::MIN;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = chunk_origin.x + x as i32;

                let world_z = chunk_origin.z + z as i32;

                let column = self.sample_column(world_x, world_z);

                columns[column_index(x, z)] = column;

                minimum_terrain_height = minimum_terrain_height.min(column.terrain_height);

                let filled_height = column
                    .water_level
                    .unwrap_or(column.terrain_height)
                    .max(column.terrain_height);

                maximum_filled_height = maximum_filled_height.max(filled_height);
            }
        }

        // Entire chunk is above both terrain
        // and any possible lake surface.
        if chunk_min_y > maximum_filled_height {
            return Chunk::filled(Voxel::Air);
        }

        // Entire chunk is sufficiently deep below
        // every surface layer, so it can be filled
        // directly with Stone.
        if chunk_max_y < minimum_terrain_height - MAX_SURFACE_LAYER_DEPTH {
            return Chunk::filled(Voxel::Stone);
        }

        let mut chunk = Chunk::new();

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let column = columns[column_index(x, z)];

                for y in 0..CHUNK_SIZE {
                    let world_y = chunk_origin.y + y as i32;

                    let voxel = self.voxel_at(column, world_y);

                    if voxel != Voxel::Air {
                        chunk.set(x, y, z, voxel);
                    }
                }
            }
        }

        chunk
    }

    fn sample_column(&self, world_x: i32, world_z: i32) -> TerrainColumn {
        let natural_height = self.natural_height_at(world_x, world_z);

        let lake_strength = self.lake_strength_at(world_x, world_z);

        if lake_strength <= 0.0 {
            return TerrainColumn {
                terrain_height: natural_height.round() as i32,

                water_level: None,

                lake_strength: 0.0,
            };
        }

        // At the center of a lake, the desired
        // floor approaches:
        //
        // sea_level - lake_max_depth
        //
        // Near the shore, the natural terrain and
        // lake floor are smoothly blended.
        let deepest_floor = self.sea_level as f32 - self.lake_max_depth;

        let carved_height = lerp(natural_height, deepest_floor, lake_strength);

        let terrain_height = carved_height.round() as i32;

        // Water appears only once the carved terrain
        // has actually gone below the lake surface.
        //
        // This naturally leaves dry sloped margins
        // around the lake.
        let water_level = if terrain_height < self.sea_level && lake_strength >= 0.20 {
            Some(self.sea_level)
        } else {
            None
        };

        TerrainColumn {
            terrain_height,
            water_level,
            lake_strength,
        }
    }

    fn voxel_at(&self, column: TerrainColumn, world_y: i32) -> Voxel {
        if world_y > column.terrain_height {
            if let Some(water_level) = column.water_level
                && world_y <= water_level
            {
                return Voxel::Water;
            }

            return Voxel::Air;
        }

        let depth = column.terrain_height - world_y;

        let underwater = column.water_level.is_some();

        let near_lake = column.lake_strength > 0.05;

        let beach = near_lake && column.terrain_height <= self.sea_level + BEACH_HEIGHT;

        if underwater || beach {
            if depth <= SAND_DEPTH {
                return Voxel::Sand;
            }

            return Voxel::Stone;
        }

        match depth {
            0 => Voxel::Grass,

            1..=GRASS_DIRT_DEPTH => Voxel::Dirt,

            _ => Voxel::Stone,
        }
    }

    fn natural_height_at(&self, world_x: i32, world_z: i32) -> f32 {
        let macro_noise = fractal_noise(
            world_x as f32,
            world_z as f32,
            self.macro_frequency,
            3,
            0.55,
            self.seed.wrapping_add(31_337),
        );

        let detail_noise = fractal_noise(
            world_x as f32,
            world_z as f32,
            self.detail_frequency,
            self.detail_octaves,
            self.persistence,
            self.seed.wrapping_add(81_731),
        );

        self.base_height + macro_noise * self.macro_amplitude + detail_noise * self.detail_amplitude
    }

    fn lake_strength_at(&self, world_x: i32, world_z: i32) -> f32 {
        let lake_noise = fractal_noise(
            world_x as f32,
            world_z as f32,
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
