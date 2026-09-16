use bevy::prelude::*;

use super::{
    blocks::Voxel,
    noise::{fbm_3d, gradient_noise_2d, gradient_noise_3d},
};

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
}

impl Default for CaveGenerator {
    fn default() -> Self {
        Self {
            // Dual 3D noise frequency for worm tunnels (spaghetti caves)
            spaghetti_freq: 0.028,
            spaghetti_threshold: 0.075,

            // Low frequency 3D FBM for expansive cavern chambers
            cheese_freq: 0.016,
            cheese_threshold: 0.46,

            // Distance below surface before full side-carving expands
            surface_buffer_voxels: 4,

            // Magma and lava pools in deep underground crust (chunk y <= -4, y <= -60 voxels)
            deep_lava_y: -60,

            // 3D Ravines / Canyons cutting down from the surface
            ravine_freq: 0.007,
            ravine_width: 0.024,
        }
    }
}

impl CaveGenerator {
    /// Determines whether a subterranean or ravine coordinate should be hollowed out by cave generation.
    pub fn is_cave(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        surface_height: i32,
        seed: u32,
    ) -> bool {
        // Caves only carve into solid ground at or beneath surface
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

        // 1. 3D Ravines / Chasms (dramatic surface-breaching fissures down to 40 voxels deep)
        if depth_below_surface <= 40 {
            let ravine_path = gradient_noise_2d(
                fx * self.ravine_freq,
                fz * self.ravine_freq,
                seed.wrapping_add(33_333),
            );

            let ravine_active =
                gradient_noise_2d(fx * 0.0025, fz * 0.0025, seed.wrapping_add(44_444));

            // Ravines generate in regions where ravine_active > 0.10
            if ravine_active > 0.10 {
                let wall_jitter = gradient_noise_3d(
                    fx * 0.045,
                    fy * 0.045,
                    fz * 0.045,
                    seed.wrapping_add(55_555),
                ) * 0.008;

                let half_width = self.ravine_width + wall_jitter;
                if ravine_path.abs() < half_width {
                    // Slight tapering at the very bottom floor of the ravine
                    let bottom_dist = 40 - depth_below_surface;
                    if bottom_dist > 1 {
                        return true;
                    }
                }
            }
        }

        // 2. Worm / Spaghetti tunnels: intersection of two zero-crossings
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

        // Core of the tunnel punctures the surface creating natural cave mouths and openings
        let is_tunnel_core = worm_a.abs() < 0.042 && worm_b.abs() < 0.042;
        if depth_below_surface <= self.surface_buffer_voxels {
            if is_tunnel_core {
                return true;
            }
        } else {
            let is_worm =
                worm_a.abs() < self.spaghetti_threshold && worm_b.abs() < self.spaghetti_threshold;
            if is_worm {
                return true;
            }
        }

        // 3. Cheese Caverns (large chambers deep underground)
        if depth_below_surface > 8 {
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

            if cheese > self.cheese_threshold {
                return true;
            }
        }

        false
    }

    /// Returns the filler voxel for a hollowed cave position.
    /// Caves are dry Air by default, with localized subterranean water aquifers and deep magma/lava.
    pub fn cave_voxel(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        sea_level: i32,
        seed: u32,
    ) -> Voxel {
        // 1. Deep volcanic magma & lava layer
        if world_y <= self.deep_lava_y {
            if world_y <= self.deep_lava_y - 4 {
                Voxel::Lava
            } else {
                Voxel::Magma
            }
        }
        // 2. Localized underground aquifers (flooded subterranean lakes)
        else if world_y <= sea_level - 10 {
            let aquifer_noise = gradient_noise_3d(
                world_x as f32 * 0.035,
                world_y as f32 * 0.035,
                world_z as f32 * 0.035,
                seed.wrapping_add(108_888),
            );

            // Only specific flooded chambers contain water (roughly 12% of deep caves)
            if aquifer_noise > 0.48 {
                Voxel::Water
            } else {
                Voxel::Air
            }
        }
        // 3. Dry, walkable/flyable air cave
        else {
            Voxel::Air
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caves_do_not_carve_above_surface() {
        let generator = CaveGenerator::default();
        assert!(!generator.is_cave(0, 20, 0, 15, 1337));
        assert!(!generator.is_cave(0, 16, 0, 15, 1337));
    }

    #[test]
    fn cave_filler_types_by_depth() {
        let generator = CaveGenerator::default();
        let sea_level = 9;

        // Normal underground caves are Air
        assert_eq!(generator.cave_voxel(0, 5, 0, sea_level, 1337), Voxel::Air);
        assert_eq!(generator.cave_voxel(0, -10, 0, sea_level, 1337), Voxel::Air);

        // Deep subterranean is magma / lava
        assert_eq!(
            generator.cave_voxel(0, -60, 0, sea_level, 1337),
            Voxel::Magma
        );
        assert_eq!(
            generator.cave_voxel(0, -68, 0, sea_level, 1337),
            Voxel::Lava
        );
    }
}
