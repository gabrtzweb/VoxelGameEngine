use bevy::prelude::*;

use super::chunk::{CHUNK_SIZE, Chunk, Voxel};

const SURFACE_LAYER_DEPTH: i32 = 3;
const BEACH_HEIGHT: i32 = 1;

#[derive(Resource, Clone)]
pub struct TerrainGenerator {
    pub seed: u32,
    pub base_height: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub octaves: u32,
    pub persistence: f32,

    pub sea_level: i32,
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self {
            seed: 1337,
            base_height: 7.0,
            amplitude: 5.0,
            frequency: 0.035,
            octaves: 3,
            persistence: 0.5,

            sea_level: 5,
        }
    }
}

impl TerrainGenerator {
    pub fn generate_chunk(&self, chunk_coordinate: IVec3) -> Chunk {
        let chunk_origin = chunk_coordinate * CHUNK_SIZE as i32;

        let chunk_min_y = chunk_origin.y;

        let chunk_max_y = chunk_origin.y + CHUNK_SIZE as i32 - 1;

        let mut heights = [0_i32; CHUNK_SIZE * CHUNK_SIZE];

        let mut minimum_height = i32::MAX;

        let mut maximum_height = i32::MIN;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = chunk_origin.x + x as i32;

                let world_z = chunk_origin.z + z as i32;

                let height = self.height_at(world_x, world_z);

                heights[x + z * CHUNK_SIZE] = height;

                minimum_height = minimum_height.min(height);

                maximum_height = maximum_height.max(height);
            }
        }

        // Completely above both terrain
        // and sea level.
        if chunk_min_y > maximum_height.max(self.sea_level) {
            return Chunk::filled(Voxel::Air);
        }

        // Completely underwater and also
        // above every terrain column.
        if chunk_min_y > maximum_height && chunk_max_y <= self.sea_level {
            return Chunk::filled(Voxel::Water);
        }

        // Deep enough that no surface material
        // can exist in this chunk.
        if chunk_max_y < minimum_height - SURFACE_LAYER_DEPTH {
            return Chunk::filled(Voxel::Stone);
        }

        let mut chunk = Chunk::new();

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let terrain_height = heights[x + z * CHUNK_SIZE];

                let is_beach = terrain_height <= self.sea_level + BEACH_HEIGHT;

                for y in 0..CHUNK_SIZE {
                    let world_y = chunk_origin.y + y as i32;

                    if world_y > terrain_height {
                        if world_y <= self.sea_level {
                            chunk.set(x, y, z, Voxel::Water);
                        }

                        continue;
                    }

                    let depth = terrain_height - world_y;

                    let voxel = if is_beach {
                        if depth <= SURFACE_LAYER_DEPTH {
                            Voxel::Sand
                        } else {
                            Voxel::Stone
                        }
                    } else {
                        match depth {
                            0 => Voxel::Grass,

                            1..=SURFACE_LAYER_DEPTH => Voxel::Dirt,

                            _ => Voxel::Stone,
                        }
                    };

                    chunk.set(x, y, z, voxel);
                }
            }
        }

        chunk
    }

    pub fn height_at(&self, world_x: i32, world_z: i32) -> i32 {
        let mut noise_value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut amplitude_sum = 0.0;

        for octave in 0..self.octaves {
            let x = world_x as f32 * self.frequency * frequency;

            let z = world_z as f32 * self.frequency * frequency;

            let octave_seed = self.seed.wrapping_add(octave.wrapping_mul(10_007));

            noise_value += value_noise(x, z, octave_seed) * amplitude;

            amplitude_sum += amplitude;

            amplitude *= self.persistence;

            frequency *= 2.0;
        }

        if amplitude_sum > 0.0 {
            noise_value /= amplitude_sum;
        }

        (self.base_height + noise_value * self.amplitude).round() as i32
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

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + (end - start) * amount
}
