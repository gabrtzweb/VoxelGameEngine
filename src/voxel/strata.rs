use bevy::prelude::*;

use super::{biome::BiomeConfig, blocks::Voxel, noise::gradient_noise_3d};

/// Generates depth-based geological layers (strata) and localized mineral/sedimentary veins.
#[derive(Debug, Clone, Reflect)]
pub struct StrataGenerator {
    pub mid_crust_y: i32,
    pub deep_crust_y: i32,
    pub vein_frequency: f32,
}

impl Default for StrataGenerator {
    fn default() -> Self {
        Self {
            // Transition from standard stone to Slate at y = -16 voxels (-8m)
            mid_crust_y: -16,
            // Transition to Blackstone / Magma depths at y = -64 voxels (-32m)
            deep_crust_y: -64,
            // Higher frequency 3D noise for compact ore/mineral vein clusters
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
        // 1. Surface layer (top logical block)
        if logical_depth <= 0 {
            return biome.surface_material;
        }

        // 2. Subsoil layer (depth 1 to subsoil_depth logical blocks)
        let lx = world_x.div_euclid(2) as f32 + 0.5;
        let ly = world_y.div_euclid(2) as f32 + 0.5;
        let lz = world_z.div_euclid(2) as f32 + 0.5;

        if logical_depth <= biome.subsoil_depth {
            // Check for clay veins in wetlands subsoil
            if biome.surface_material == Voxel::Mud {
                let clay_noise = gradient_noise_3d(
                    lx * self.vein_frequency,
                    ly * self.vein_frequency,
                    lz * self.vein_frequency,
                    seed.wrapping_add(104_729),
                );
                if clay_noise > 0.45 {
                    return Voxel::Clay;
                }
            }
            return biome.subsoil_material;
        }

        // 3. Deep geological strata & veins based on absolute Y level
        let vein = gradient_noise_3d(
            lx * self.vein_frequency,
            ly * self.vein_frequency,
            lz * self.vein_frequency,
            seed.wrapping_add(120_007),
        );

        let block_top_y = world_y.div_euclid(2) * 2 + 1;

        // A. Deep Crust (Blackstone, Magma, Cobbleblackstone)
        if block_top_y <= self.deep_crust_y {
            if vein > 0.65 {
                Voxel::Magma
            } else if vein > 0.45 {
                Voxel::Cobbleblackstone
            } else {
                Voxel::Blackstone
            }
        }
        // B. Mid Crust (Slate, Cobbleslate, Flint)
        else if block_top_y <= self.mid_crust_y {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::biome::BiomeType;

    #[test]
    fn strata_materials_change_with_depth() {
        let generator = StrataGenerator::default();
        let plains = BiomeType::Plains.config();
        let seed = 1337;

        // Surface is grass
        let surface = generator.solid_voxel_at(0, 10, 0, 0, &plains, seed);
        assert_eq!(surface, Voxel::Grass);

        // Subsoil is dirt
        let subsoil = generator.solid_voxel_at(0, 8, 0, 2, &plains, seed);
        assert_eq!(subsoil, Voxel::Dirt);

        // Mid crust includes slate/cobbleslate/flint
        let mid = generator.solid_voxel_at(0, -30, 0, 15, &plains, seed);
        assert!(matches!(
            mid,
            Voxel::Slate | Voxel::Cobbleslate | Voxel::Flint
        ));

        // Deep crust includes blackstone/cobbleblackstone/magma
        let deep = generator.solid_voxel_at(0, -80, 0, 40, &plains, seed);
        assert!(matches!(
            deep,
            Voxel::Blackstone | Voxel::Cobbleblackstone | Voxel::Magma
        ));
    }
}
