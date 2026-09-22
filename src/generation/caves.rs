use bevy::prelude::*;

use crate::{
    core::noise::{fbm_3d, gradient_noise_2d, gradient_noise_3d},
    world::{CHUNK_SIZE, CHUNK_VOLUME, Voxel},
};

/// 4-component continuous density noise vector evaluated at 3D coordinates.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
#[repr(C)]
pub struct CaveNoiseSample {
    pub worm_a: f32,
    pub worm_b: f32,
    pub cheese: f32,
    pub aquifer: f32,
}

#[inline(always)]
fn lerp_sample(a: CaveNoiseSample, b: CaveNoiseSample, t: f32) -> CaveNoiseSample {
    CaveNoiseSample {
        worm_a: a.worm_a + t * (b.worm_a - a.worm_a),
        worm_b: a.worm_b + t * (b.worm_b - a.worm_b),
        cheese: a.cheese + t * (b.cheese - a.cheese),
        aquifer: a.aquifer + t * (b.aquifer - a.aquifer),
    }
}

/// Pre-interpolated 16x16x16 cave density sampler for a single chunk.
#[derive(Clone)]
pub struct ChunkCaveSampler {
    samples: Vec<CaveNoiseSample>,
}

impl ChunkCaveSampler {
    #[inline]
    pub fn sample(&self, x: usize, y: usize, z: usize) -> CaveNoiseSample {
        self.samples[x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE]
    }
}

/// 3D Cave, ravine, and cavern generator using dual-noise worms, chasms, and cheese chambers.
#[derive(Debug, Clone, Reflect)]
pub struct CaveGenerator {
    pub spaghetti_freq: f32,
    pub spaghetti_threshold: f32,
    pub cheese_freq: f32,
    pub cheese_threshold: f32,
    pub surface_buffer_voxels: i32,
    pub deep_lava_y: i32,
    pub ravine_freq: f32,
    pub ravine_width: f32,
    pub ravine_abundance: f32,
}

impl Default for CaveGenerator {
    fn default() -> Self {
        Self {
            spaghetti_freq: 0.024,
            spaghetti_threshold: 0.14,
            cheese_freq: 0.016,
            cheese_threshold: 0.44,
            surface_buffer_voxels: 4,
            deep_lava_y: -60,
            ravine_freq: 0.003,
            ravine_width: 0.022,
            ravine_abundance: 0.15,
        }
    }
}

impl CaveGenerator {
    /// Evaluates the 4 continuous 3D noise fields at a specific floating-point coordinate.
    #[inline]
    pub fn sample_noise_point(&self, fx: f32, fy: f32, fz: f32, seed: u32) -> CaveNoiseSample {
        let worm_a = gradient_noise_3d(
            fx * self.spaghetti_freq,
            fy * self.spaghetti_freq,
            fz * self.spaghetti_freq,
            seed.wrapping_add(70_001),
        );

        let worm_b = gradient_noise_3d(
            fx * self.spaghetti_freq,
            fy * self.spaghetti_freq,
            fz * self.spaghetti_freq,
            seed.wrapping_add(80_009),
        );

        let cheese = fbm_3d(
            fx,
            fy,
            fz,
            self.cheese_freq,
            2,
            0.5,
            2.0,
            seed.wrapping_add(90_013),
        );

        let aquifer = gradient_noise_3d(
            fx * 0.035,
            fy * 0.035,
            fz * 0.035,
            seed.wrapping_add(108_888),
        );

        CaveNoiseSample {
            worm_a,
            worm_b,
            cheese,
            aquifer,
        }
    }

    /// Evaluates 3D noise on a 5x5x5 lattice (125 samples) and up-samples across
    /// all 4,096 voxels in the chunk using SIMD-friendly trilinear interpolation.
    pub fn build_chunk_sampler(&self, chunk_origin: IVec3, seed: u32) -> ChunkCaveSampler {
        const LATTICE_DIM: usize = 5;
        const CELL_SIZE: usize = 4;

        let mut lattice = [[[CaveNoiseSample::default(); LATTICE_DIM]; LATTICE_DIM]; LATTICE_DIM];

        for lz in 0..LATTICE_DIM {
            let fz = (chunk_origin.z + (lz * CELL_SIZE) as i32) as f32;
            for ly in 0..LATTICE_DIM {
                let fy = (chunk_origin.y + (ly * CELL_SIZE) as i32) as f32;
                for lx in 0..LATTICE_DIM {
                    let fx = (chunk_origin.x + (lx * CELL_SIZE) as i32) as f32;
                    lattice[lx][ly][lz] = self.sample_noise_point(fx, fy, fz, seed);
                }
            }
        }

        let mut samples = vec![CaveNoiseSample::default(); CHUNK_VOLUME];

        for cz in 0..4 {
            for cy in 0..4 {
                for cx in 0..4 {
                    let c000 = lattice[cx][cy][cz];
                    let c100 = lattice[cx + 1][cy][cz];
                    let c010 = lattice[cx][cy + 1][cz];
                    let c110 = lattice[cx + 1][cy + 1][cz];
                    let c001 = lattice[cx][cy][cz + 1];
                    let c101 = lattice[cx + 1][cy][cz + 1];
                    let c011 = lattice[cx][cy + 1][cz + 1];
                    let c111 = lattice[cx + 1][cy + 1][cz + 1];

                    for dz in 0..CELL_SIZE {
                        let tz = dz as f32 * 0.25;
                        let c00 = lerp_sample(c000, c001, tz);
                        let c10 = lerp_sample(c100, c101, tz);
                        let c01 = lerp_sample(c010, c011, tz);
                        let c11 = lerp_sample(c110, c111, tz);

                        let z = cz * CELL_SIZE + dz;

                        for dy in 0..CELL_SIZE {
                            let ty = dy as f32 * 0.25;
                            let c0 = lerp_sample(c00, c01, ty);
                            let c1 = lerp_sample(c10, c11, ty);

                            let y = cy * CELL_SIZE + dy;

                            for dx in 0..CELL_SIZE {
                                let tx = dx as f32 * 0.25;
                                let sample = lerp_sample(c0, c1, tx);

                                let x = cx * CELL_SIZE + dx;
                                samples[x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE] = sample;
                            }
                        }
                    }
                }
            }
        }

        ChunkCaveSampler { samples }
    }

    /// Determines whether a subterranean or ravine coordinate should be hollowed out using a precomputed sample.
    #[allow(clippy::too_many_arguments)]
    pub fn is_cave_sampled(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        sample: CaveNoiseSample,
        surface_height: i32,
        is_underwater: bool,
        sea_level: i32,
        seed: u32,
    ) -> bool {
        if world_y > surface_height {
            return false;
        }

        let depth_below_surface = surface_height - world_y;
        if depth_below_surface < 0 {
            return false;
        }

        let fx = world_x as f32;
        let fy = world_y as f32;
        let fz = world_z as f32;

        // 1. 3D Ravines / Chasms (dramatic fissures, strictly prohibited underwater in rivers/oceans)
        if !is_underwater
            && surface_height > sea_level + 2
            && depth_below_surface <= 36
            && self.ravine_abundance > 0.001
        {
            let active_threshold = (1.0 - self.ravine_abundance).clamp(0.50, 0.99);
            let ravine_active =
                gradient_noise_2d(fx * 0.0020, fz * 0.0020, seed.wrapping_add(44_444));

            if ravine_active > active_threshold {
                let ravine_path = gradient_noise_2d(
                    fx * self.ravine_freq,
                    fz * self.ravine_freq,
                    seed.wrapping_add(33_333),
                );

                let wall_jitter = gradient_noise_3d(
                    fx * 0.045,
                    fy * 0.045,
                    fz * 0.045,
                    seed.wrapping_add(55_555),
                ) * 0.008;

                let half_width = self.ravine_width + wall_jitter;
                if ravine_path.abs() < half_width {
                    let bottom_dist = 36 - depth_below_surface;
                    if bottom_dist > 1 {
                        return true;
                    }
                }
            }
        }

        // 2. Worm / Spaghetti tunnels: subterranean tubes with natural surface cave mouths
        let is_worm = sample.worm_a.abs() < self.spaghetti_threshold
            && sample.worm_b.abs() < self.spaghetti_threshold;

        if depth_below_surface <= self.surface_buffer_voxels {
            // Surface buffer protects hillsides from craters/gullies.
            // Natural cave mouths breach the surface where a subterranean worm tunnel reaches
            // the surface within designated entrance zones.
            if !is_underwater && is_worm {
                let entrance_mask =
                    gradient_noise_2d(fx * 0.010, fz * 0.010, seed.wrapping_add(77_777));
                if entrance_mask > 0.18 {
                    return true;
                }
            }
        } else if is_worm {
            return true;
        }

        // 3. Cheese Caverns (large chambers deep underground)
        if depth_below_surface > 8 && sample.cheese > self.cheese_threshold {
            return true;
        }

        // 4. Natural Mountain Arches: horizontal cavern hollows tunneling through ridges
        if surface_height >= 34
            && depth_below_surface >= 6
            && depth_below_surface <= 24
            && world_y > sea_level + 10
        {
            let arch_noise_a = gradient_noise_3d(
                fx * 0.025,
                fy * 0.040,
                fz * 0.025,
                seed.wrapping_add(61_234),
            );
            let arch_noise_b = gradient_noise_3d(
                fx * 0.025,
                fy * 0.040,
                fz * 0.025,
                seed.wrapping_add(71_567),
            );
            if arch_noise_a.abs() < 0.13 && arch_noise_b.abs() < 0.22 {
                return true;
            }
        }

        false
    }

    /// Determines whether a subterranean or ravine coordinate should be hollowed out by cave generation.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn is_cave(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        surface_height: i32,
        is_underwater: bool,
        sea_level: i32,
        seed: u32,
    ) -> bool {
        let sample = self.sample_noise_point(world_x as f32, world_y as f32, world_z as f32, seed);
        self.is_cave_sampled(
            world_x,
            world_y,
            world_z,
            sample,
            surface_height,
            is_underwater,
            sea_level,
            seed,
        )
    }

    /// Returns the filler voxel for a hollowed cave position using an up-sampled noise sample.
    #[inline]
    pub fn cave_voxel_sampled(
        &self,
        _world_y: i32,
        _sample: CaveNoiseSample,
        _sea_level: i32,
    ) -> Voxel {
        Voxel::Air
    }

    /// Returns the filler voxel for a hollowed cave position.
    #[allow(dead_code)]
    pub fn cave_voxel(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        sea_level: i32,
        seed: u32,
    ) -> Voxel {
        let sample = self.sample_noise_point(world_x as f32, world_y as f32, world_z as f32, seed);
        self.cave_voxel_sampled(world_y, sample, sea_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caves_do_not_carve_above_surface() {
        let generator = CaveGenerator::default();
        assert!(!generator.is_cave(0, 20, 0, 15, false, 9, 1337));
        assert!(!generator.is_cave(0, 16, 0, 15, false, 9, 1337));
    }

    #[test]
    fn cave_filler_types_by_depth() {
        let generator = CaveGenerator::default();
        let sea_level = 9;

        assert_eq!(generator.cave_voxel(0, 5, 0, sea_level, 1337), Voxel::Air);
        assert_eq!(generator.cave_voxel(0, -10, 0, sea_level, 1337), Voxel::Air);
        assert_eq!(generator.cave_voxel(0, -60, 0, sea_level, 1337), Voxel::Air);
        assert_eq!(generator.cave_voxel(0, -68, 0, sea_level, 1337), Voxel::Air);
    }

    #[test]
    fn ravines_are_suppressed_underwater() {
        let generator = CaveGenerator::default();
        let sea_level = 9;

        // Submerged surface (surface_height <= sea_level) must never trigger ravine carving
        let at_sea = generator.is_cave(100, 8, 100, 8, true, sea_level, 1337);
        assert!(!at_sea);
    }

    #[test]
    fn trilinear_chunk_sampler_continuity_and_dimensions() {
        let generator = CaveGenerator::default();
        let chunk_origin = IVec3::new(16, -32, 16);
        let sampler = generator.build_chunk_sampler(chunk_origin, 1337);

        assert_eq!(sampler.samples.len(), CHUNK_VOLUME);

        // Check continuity across cell boundary (e.g. at x = 3 and x = 4)
        let s_3 = sampler.sample(3, 0, 0);
        let s_4 = sampler.sample(4, 0, 0);

        // The values must be close (continuous smoothly varying scalar field)
        assert!((s_3.worm_a - s_4.worm_a).abs() < 0.25);
        assert!((s_3.worm_b - s_4.worm_b).abs() < 0.25);
        assert!((s_3.cheese - s_4.cheese).abs() < 0.25);

        // Exact lattice point comparison: at (0, 0, 0), sample must exactly equal sample_noise_point
        let exact = generator.sample_noise_point(
            chunk_origin.x as f32,
            chunk_origin.y as f32,
            chunk_origin.z as f32,
            1337,
        );
        let sampled = sampler.sample(0, 0, 0);
        assert!((exact.worm_a - sampled.worm_a).abs() < 1e-5);
        assert!((exact.worm_b - sampled.worm_b).abs() < 1e-5);
        assert!((exact.cheese - sampled.cheese).abs() < 1e-5);
        assert!((exact.aquifer - sampled.aquifer).abs() < 1e-5);
    }
}
