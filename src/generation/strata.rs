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
            mid_crust_y: -120,
            deep_crust_y: -200,
            bedrock_min_block_y: -254,
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

        // Bedrock (Dreadstone) strictly in bottom 3-4 layers of the world
        if block_y <= self.bedrock_min_block_y {
            return Voxel::Dreadstone;
        } else if block_y <= self.bedrock_min_block_y + 3 {
            let bedrock_noise =
                gradient_noise_3d(bx * 0.35, by * 0.35, bz * 0.35, seed.wrapping_add(234_567));
            if block_y == self.bedrock_min_block_y + 1 {
                if bedrock_noise > -0.40 {
                    return Voxel::Dreadstone;
                }
            } else if block_y == self.bedrock_min_block_y + 2 {
                if bedrock_noise > 0.05 {
                    return Voxel::Dreadstone;
                }
            } else if block_y == self.bedrock_min_block_y + 3 && bedrock_noise > 0.40 {
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
            return biome.subsoil_material;
        }

        // 3. Subterranean stone:
        // - Highlands biome uniquely features Slate and Cobbleslate
        // - Standard biomes switch from Stone to Blackstone halfway down the world (mid_crust_y)
        if biome.biome_type == crate::generation::BiomeType::Highlands {
            let cobbleslate_noise =
                gradient_noise_3d(bx * 0.20, by * 0.20, bz * 0.20, seed.wrapping_add(88_222));
            if cobbleslate_noise > 0.35 {
                Voxel::Cobbleslate
            } else {
                Voxel::Slate
            }
        } else {
            let blackstone_transition =
                gradient_noise_3d(bx * 0.15, by * 0.15, bz * 0.15, seed.wrapping_add(77_889)) * 4.0;
            if (block_y as f32) < (self.mid_crust_y as f32 + blackstone_transition) {
                Voxel::Blackstone
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

        let mid = generator.solid_voxel_at(0, -20, 0, 15, &plains, seed);
        assert_eq!(mid, Voxel::Stone);

        // Below mid_crust_y (-120), subterranean stone transitions to Blackstone
        let deep = generator.solid_voxel_at(0, -150, 0, 40, &plains, seed);
        assert_eq!(deep, Voxel::Blackstone);

        // Highlands biome keeps Slate and Cobbleslate exclusive
        let highlands = BiomeType::Highlands.config();
        let highlands_rock = generator.solid_voxel_at(0, 40, 0, 5, &highlands, seed);
        assert!(highlands_rock == Voxel::Slate || highlands_rock == Voxel::Cobbleslate);

        // Bottom layer of blocks must always be unbreakable Dreadstone
        let bedrock_bottom = generator.solid_voxel_at(0, -254, 0, 80, &plains, seed);
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
                let bottom = generator.solid_voxel_at(x, -254, z, 90, &plains, seed);
                assert_eq!(
                    bottom,
                    Voxel::Dreadstone,
                    "Layer -254 must be 100% Dreadstone"
                );
            }
        }
    }
}
