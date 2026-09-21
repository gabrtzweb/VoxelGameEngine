use bevy::prelude::*;

use super::biome::BiomeConfig;
use crate::{core::noise::gradient_noise_3d, world::Voxel};

/// Generates depth-based geological layers (strata) and localized mineral/sedimentary veins.
#[derive(Debug, Clone, Reflect)]
pub struct StrataGenerator {
    pub mid_crust_y: i32,
    pub deep_crust_y: i32,
    pub bedrock_min_block_y: i32,
    pub vein_frequency: f32,
}

impl Default for StrataGenerator {
    fn default() -> Self {
        Self {
            mid_crust_y: -16,
            deep_crust_y: -64,
            bedrock_min_block_y: -80,
            vein_frequency: 0.085,
        }
    }
}

impl StrataGenerator {
    /// Determines the solid subterranean voxel type at the given coordinate.
    pub fn solid_voxel_at(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        logical_depth: i32,
        biome: &BiomeConfig,
        seed: u32,
    ) -> Voxel {
        let block_y = world_y;
        let bx = world_x as f32 + 0.5;
        let by = ly_coord(world_y);
        let bz = world_z as f32 + 0.5;

        // Bedrock (Dreadstone) bottom 3 layers with natural blend
        if block_y <= self.bedrock_min_block_y {
            return Voxel::Dreadstone;
        } else if block_y <= self.bedrock_min_block_y + 2 {
            let bedrock_noise =
                gradient_noise_3d(bx * 0.35, by * 0.35, bz * 0.35, seed.wrapping_add(234_567));
            if block_y == self.bedrock_min_block_y + 1 {
                if bedrock_noise > -0.30 {
                    return Voxel::Dreadstone;
                }
            } else if block_y == self.bedrock_min_block_y + 2 && bedrock_noise > 0.20 {
                return Voxel::Dreadstone;
            }
        }

        // 1. Surface layer (top logical block)
        if logical_depth <= 0 {
            return biome.surface_material;
        }

        // 2. Subsoil layer with natural dithered transition to stone
        let transition_noise =
            gradient_noise_3d(bx * 0.22, by * 0.22, bz * 0.22, seed.wrapping_add(99_111));

        let dithered_subsoil_depth =
            (biome.subsoil_depth as f32 + transition_noise * 1.6).round() as i32;

        if logical_depth <= dithered_subsoil_depth.max(1) {
            // Check for clay veins in wetlands subsoil
            if biome.surface_material == Voxel::Mud {
                let clay_noise = gradient_noise_3d(
                    bx * self.vein_frequency,
                    by * self.vein_frequency,
                    bz * self.vein_frequency,
                    seed.wrapping_add(104_729),
                );
                if clay_noise > 0.45 {
                    return Voxel::Clay;
                }
            }
            return biome.subsoil_material;
        }

        // 3. Deep geological strata & veins based on modulated Y level
        let vein = gradient_noise_3d(
            bx * self.vein_frequency,
            by * self.vein_frequency,
            bz * self.vein_frequency,
            seed.wrapping_add(120_007),
        );

        let block_top_y = block_y * 2 + 1;
        let strata_dither = transition_noise * 6.0;

        // A. Deep Crust (Blackstone, Magma, Cobbleblackstone)
        if (block_top_y as f32 + strata_dither) <= self.deep_crust_y as f32 {
            if vein > 0.65 {
                Voxel::Magma
            } else if vein > 0.45 {
                Voxel::Cobbleblackstone
            } else {
                Voxel::Blackstone
            }
        }
        // B. Mid Crust (Slate, Cobbleslate, Flint)
        else if (block_top_y as f32 + strata_dither) <= self.mid_crust_y as f32 {
            if vein > 0.72 {
                Voxel::Flint
            } else if vein > 0.45 {
                Voxel::Cobbleslate
            } else {
                Voxel::Slate
            }
        }
        // C. Upper Crust (Standard Stone or Biome Primary Stone, Gravel, Flint, Cobblestone)
        else {
            if vein > 0.78 {
                Voxel::Flint
            } else if vein > 0.58 {
                Voxel::Gravel
            } else if vein > 0.42 {
                Voxel::Cobblestone
            } else {
                biome.primary_stone
            }
        }
    }
}

#[inline]
fn ly_coord(world_y: i32) -> f32 {
    world_y as f32 + 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::biome::BiomeType;

    #[test]
    fn strata_materials_change_with_depth() {
        let generator = StrataGenerator::default();
        let plains = BiomeType::Plains.config();
        let seed = 1337;

        let surface = generator.solid_voxel_at(0, 10, 0, 0, &plains, seed);
        assert_eq!(surface, Voxel::Grass);

        let subsoil = generator.solid_voxel_at(0, 8, 0, 2, &plains, seed);
        assert_eq!(subsoil, Voxel::Dirt);

        let mid = generator.solid_voxel_at(0, -30, 0, 15, &plains, seed);
        assert!(matches!(
            mid,
            Voxel::Slate | Voxel::Cobbleslate | Voxel::Flint
        ));

        let deep = generator.solid_voxel_at(0, -70, 0, 40, &plains, seed);
        assert!(matches!(
            deep,
            Voxel::Blackstone | Voxel::Cobbleblackstone | Voxel::Magma
        ));

        // Bottom layer of blocks must always be unbreakable Dreadstone
        let bedrock_bottom = generator.solid_voxel_at(0, -80, 0, 80, &plains, seed);
        assert_eq!(bedrock_bottom, Voxel::Dreadstone);
        assert!(bedrock_bottom.is_unbreakable());
    }

    #[test]
    fn dreadstone_bedrock_spawns_in_bottom_layers() {
        let generator = StrataGenerator::default();
        let plains = BiomeType::Plains.config();
        let seed = 42;

        for x in -5..=5 {
            for z in -5..=5 {
                let bottom = generator.solid_voxel_at(x, -80, z, 90, &plains, seed);
                assert_eq!(
                    bottom,
                    Voxel::Dreadstone,
                    "Layer -80 must be 100% Dreadstone"
                );
            }
        }
    }
}
