use bevy::prelude::*;

use super::chunk::{CHUNK_SIZE, Chunk, Voxel};

#[derive(Resource)]
pub struct TerrainGenerator {
    pub seed: u32,
    pub base_height: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub octaves: u32,
    pub persistence: f32,
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
        }
    }
}

impl TerrainGenerator {
    pub fn generate_chunk(&self, chunk_coordinate: IVec3) -> Chunk {
        let mut chunk = Chunk::new();

        let chunk_origin = chunk_coordinate * CHUNK_SIZE as i32;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = chunk_origin.x + x as i32;
                let world_z = chunk_origin.z + z as i32;

                let terrain_height = self.height_at(world_x, world_z);

                for y in 0..CHUNK_SIZE {
                    let world_y = chunk_origin.y + y as i32;

                    if world_y <= terrain_height {
                        chunk.set(x, y, z, Voxel::Solid);
                    }
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
