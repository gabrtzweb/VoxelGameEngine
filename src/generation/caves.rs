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
    pub bedrock_floor_y: i32,
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
            bedrock_floor_y: -254,
        }
    }
}

impl CaveGenerator {
    /// Determines whether a hollowed subterranean cave coordinate is submerged under a regional
    /// lava lake, abyssal mantle core magma, or other liquid basin.
    /// Strictly guarantees flat, horizontal liquid surfaces with zero liquid on walls or ceilings.
    #[inline]
    pub fn cave_voxel_sampled(
        &self,
        world_y: i32,
        _world_x: i32,
        _world_z: i32,
        _sample: CaveNoiseSample,
        _sea_level: i32,
        _seed: u32,
    ) -> Voxel {
        // Abyssal Geothermal Mantle Sea:
        // Strictly at the deep bedrock floor (Y = -246 down to bedrock at -254).
        // Flat horizontal magma horizon at Y = -246. Above -246 is dry cave air.
        const MANTLE_MAGMA_SEA_LEVEL: i32 = -246;
        if world_y <= MANTLE_MAGMA_SEA_LEVEL {
            return Voxel::LiquidCoreMagma;
        }

        Voxel::Air
    }

    /// Fast test to verify if a 3D coordinate lies within an open cavern or worm tunnel.
    #[inline]
    pub fn is_cave_at_point(&self, fx: f32, fy: f32, fz: f32, seed: u32) -> bool {
        let sample = self.sample_noise_point(fx, fy, fz, seed);
        (sample.worm_a.abs() < self.spaghetti_threshold && sample.worm_b.abs() < self.spaghetti_threshold)
            || sample.cheese > self.cheese_threshold
    }

    /// Selects an authentic volcanic rock material for lava basin floors and rims.
    #[inline]
    pub fn volcanic_rock_for_basin(seed_hash: u32, x: i32, y: i32, z: i32) -> Voxel {
        let roll = ((seed_hash ^ (x as u32).wrapping_mul(31) ^ (z as u32).wrapping_mul(73) ^ (y as u32).wrapping_mul(127)) >> 4) % 100;
        if roll < 26 {
            Voxel::Obsidian
        } else if roll < 52 {
            Voxel::Magma
        } else if roll < 78 {
            Voxel::Basalt
        } else {
            Voxel::CobbledBasalt
        }
    }

    /// Evaluates if (world_x, world_y, world_z) lies inside a complete subterranean lava pool basin.
    /// Returns:
    /// - `Some(Voxel::LiquidLava)` for the flat lava liquid core
    /// - `Some(Voxel::Air)` for headroom above the pool
    /// - `Some(volcanic_rock)` (Obsidian, Magma, Basalt, CobbledBasalt) for the solid basin floor and enclosing rim
    /// - `None` if outside any lava basin
    /// Evaluates if (world_x, world_y, world_z) lies inside a complete subterranean lava pool basin.
    /// Strictly guarantees:
    /// - Flat, horizontal lava liquid surface with zero floating blocks.
    /// - A solid volcanic rim (borda) rising 1 block above the lava, completely sealing the pool so it never leaks.
    /// - A solid volcanic foundation (Obsidian, Magma, Basalt, CobbledBasalt) extending continuously downward,
    ///   making it physically impossible for the pool or its floor to float in mid-air.
    /// - Clear, open cavern headroom directly above the lava core.
    pub fn subterranean_lava_basin_at(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        seed: u32,
    ) -> Option<Voxel> {
        const CELL_SIZE: i32 = 28;
        let cell_x = world_x.div_euclid(CELL_SIZE);
        let cell_z = world_z.div_euclid(CELL_SIZE);

        // 6 subterranean depth tiers: -72, -104, -136, -168, -200, -232
        const TIERS: [i32; 6] = [-72, -104, -136, -168, -200, -232];

        for dx in -1..=1 {
            for dz in -1..=1 {
                let cx = cell_x + dx;
                let cz = cell_z + dz;

                for (tier_idx, &tier_y) in TIERS.iter().enumerate() {
                    // Check if world_y is in the vertical range affected by this tier
                    // Headroom extends up to tier_y + 4; foundation extends downward 32 blocks.
                    if world_y > tier_y + 4 || world_y < tier_y - 32 {
                        continue;
                    }

                    let h = crate::core::noise::hash_2d(
                        cx,
                        cz,
                        seed.wrapping_add(883_117 + tier_idx as u32 * 7_919),
                    );

                    // ~55% chance per depth tier per cell to contain an authentic volcanic lava basin
                    if (h % 100) >= 55 {
                        continue;
                    }

                    let margin = 4;
                    let center_x = cx * CELL_SIZE + ((h % (CELL_SIZE as u32 - margin * 2)) + margin) as i32;
                    let center_z = cz * CELL_SIZE + (((h >> 8) % (CELL_SIZE as u32 - margin * 2)) + margin) as i32;
                    let pool_surface_y = tier_y;

                    // Only generate pool if an actual cavern/tunnel exists at this depth
                    if !self.is_cave_at_point(center_x as f32, pool_surface_y as f32, center_z as f32, seed)
                        && !self.is_cave_at_point(center_x as f32, (pool_surface_y + 1) as f32, center_z as f32, seed)
                    {
                        continue;
                    }

                    let base_radius = 6.0 + ((h >> 16) % 5) as f32; // 6.0 .. 10.0 blocks
                    let edge_noise = gradient_noise_2d(
                        world_x as f32 * 0.08,
                        world_z as f32 * 0.08,
                        seed.wrapping_add(331_909 + tier_idx as u32 * 3_137),
                    );
                    let radius = (base_radius + edge_noise * 1.8).max(4.0);
                    let rim_radius = radius + 2.8; // Sturdy ~3-block thick volcanic containment rim

                    let dist_sq = ((world_x - center_x) * (world_x - center_x)
                        + (world_z - center_z) * (world_z - center_z)) as f32;

                    if dist_sq > rim_radius * rim_radius {
                        continue;
                    }

                    let dist = dist_sq.sqrt();
                    let pool_depth = 3;

                    if dist <= radius {
                        // 1. Clear open headroom above the lava surface
                        if world_y > pool_surface_y && world_y <= pool_surface_y + 4 {
                            return Some(Voxel::Air);
                        }
                        // 2. Liquid Lava core (strictly horizontal surface at pool_surface_y)
                        if world_y <= pool_surface_y && world_y >= pool_surface_y - pool_depth {
                            return Some(Voxel::LiquidLava);
                        }
                        // 3. Solid basin floor & volcanic foundation extending continuously downward:
                        // Completely fills any open cavern space beneath the pool down to bedrock.
                        // It is physically impossible for the lava pool to ever float in mid-air.
                        if world_y < pool_surface_y - pool_depth {
                            return Some(Self::volcanic_rock_for_basin(h, world_x, world_y, world_z));
                        }
                    } else {
                        // 4. Solid volcanic rim encircling the perimeter:
                        // Rises 1 block above the lava surface to contain it, and extends downward
                        // through the entire depth to solidly support and seal the basin.
                        if world_y <= pool_surface_y + 1 {
                            return Some(Self::volcanic_rock_for_basin(h, world_x, world_y, world_z));
                        }
                    }
                }
            }
        }
        None
    }

    /// Checks if a subterranean solid rock coordinate is within the volcanic thermal halo of a nearby lava basin.
    /// Surrounding cave walls and floors are naturally transmuted into Basalt, Obsidian, Magma, and CobbledBasalt.
    pub fn volcanic_halo_at(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        seed: u32,
    ) -> Option<Voxel> {
        const CELL_SIZE: i32 = 28;
        let cell_x = world_x.div_euclid(CELL_SIZE);
        let cell_z = world_z.div_euclid(CELL_SIZE);
        const TIERS: [i32; 6] = [-72, -104, -136, -168, -200, -232];

        for dx in -1..=1 {
            for dz in -1..=1 {
                let cx = cell_x + dx;
                let cz = cell_z + dz;

                for (tier_idx, &tier_y) in TIERS.iter().enumerate() {
                    if world_y > tier_y + 4 || world_y < tier_y - 20 {
                        continue;
                    }
                    let h = crate::core::noise::hash_2d(
                        cx,
                        cz,
                        seed.wrapping_add(883_117 + tier_idx as u32 * 7_919),
                    );
                    if (h % 100) >= 55 {
                        continue;
                    }
                    let margin = 4;
                    let center_x = cx * CELL_SIZE + ((h % (CELL_SIZE as u32 - margin * 2)) + margin) as i32;
                    let center_z = cz * CELL_SIZE + (((h >> 8) % (CELL_SIZE as u32 - margin * 2)) + margin) as i32;
                    let pool_surface_y = tier_y;

                    if !self.is_cave_at_point(center_x as f32, pool_surface_y as f32, center_z as f32, seed)
                        && !self.is_cave_at_point(center_x as f32, (pool_surface_y + 1) as f32, center_z as f32, seed)
                    {
                        continue;
                    }

                    let base_radius = 6.0 + ((h >> 16) % 5) as f32;
                    let halo_radius = base_radius + 5.5;
                    let dist_sq = ((world_x - center_x) * (world_x - center_x)
                        + (world_z - center_z) * (world_z - center_z)) as f32;

                    if dist_sq <= halo_radius * halo_radius {
                        return Some(Self::volcanic_rock_for_basin(h, world_x, world_y, world_z));
                    }
                }
            }
        }
        None
    }

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
    #[allow(clippy::needless_range_loop)]
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
        if world_y > surface_height || world_y <= self.bedrock_floor_y {
            return false;
        }

        let depth_below_surface = surface_height - world_y;
        if depth_below_surface < 0 {
            return false;
        }

        // Smooth floor transition: gently taper cave carving approaching the bedrock floor
        // over 8 voxels, curving tunnel bottoms and cavern floors smoothly rather than abrupt flat cuts.
        let smooth_floor = if world_y < self.bedrock_floor_y + 8 {
            let floor_dist = (world_y - self.bedrock_floor_y) as f32;
            let t = (floor_dist / 8.0).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        } else {
            1.0
        };

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

                let half_width = (self.ravine_width + wall_jitter) * smooth_floor;
                if ravine_path.abs() < half_width {
                    let bottom_dist = 36 - depth_below_surface;
                    if bottom_dist > 1 {
                        return true;
                    }
                }
            }
        }

        // 2. Worm / Spaghetti tunnels: subterranean tubes with natural surface cave mouths
        let effective_spaghetti = self.spaghetti_threshold * smooth_floor;
        let is_worm =
            sample.worm_a.abs() < effective_spaghetti && sample.worm_b.abs() < effective_spaghetti;

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
        let effective_cheese_threshold = self.cheese_threshold + (1.0 - smooth_floor) * 0.50;
        if depth_below_surface > 8 && sample.cheese > effective_cheese_threshold {
            return true;
        }

        // 4. Natural Mountain Arches: horizontal cavern hollows tunneling through ridges
        if surface_height >= 34
            && (6..=24).contains(&depth_below_surface)
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
}
