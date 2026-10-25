use bevy::prelude::*;

use super::biome::BiomeConfig;
use crate::{core::noise::gradient_noise_3d, world::Voxel};

/// Generates depth-based geological layers (strata), intrusive rock pockets, and the mantle floor.
#[derive(Debug, Clone, Reflect)]
pub struct StrataGenerator {
    pub upper_crust_y: i32,
    pub mid_crust_y: i32,
    pub deep_crust_y: i32,
    pub abyssal_crust_y: i32,
    pub bedrock_min_block_y: i32,
    pub vein_frequency: f32,
}

impl Default for StrataGenerator {
    fn default() -> Self {
        Self {
            upper_crust_y: -30,
            mid_crust_y: -110,
            deep_crust_y: -190,
            abyssal_crust_y: -248,
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

        // 1. Bedrock Floor (Primordial Mantle & Fractured Seams)
        // Strictly at the bottom of the world
        if block_y <= self.bedrock_min_block_y {
            return Voxel::Mantle;
        } else if block_y <= self.abyssal_crust_y {
            let mantle_noise =
                gradient_noise_3d(bx * 0.28, by * 0.28, bz * 0.28, seed.wrapping_add(234_567));

            if block_y <= self.bedrock_min_block_y + 2 {
                // Impenetrable mantle floor with rare molten cracks
                if mantle_noise > 0.48 {
                    return Voxel::Magma;
                }
                return Voxel::Mantle;
            } else if block_y <= self.bedrock_min_block_y + 4 {
                // Upper mantle transition: fractured cobbled mantle and magma veins
                if mantle_noise > 0.40 {
                    return Voxel::Magma;
                } else if mantle_noise > -0.15 {
                    return Voxel::Mantle;
                } else {
                    return Voxel::CobbledMantle;
                }
            } else {
                // Transition zone into mantle
                if mantle_noise > 0.42 {
                    return Voxel::Magma;
                } else if mantle_noise > 0.05 {
                    return Voxel::CobbledMantle;
                }
            }
        }

        // 2. Surface layer (top logical block)
        if logical_depth <= 0 {
            return biome.surface_material;
        }

        // 3. Subsoil layer with natural dithered transition to stone
        let transition_noise =
            gradient_noise_3d(bx * 0.22, by * 0.22, bz * 0.22, seed.wrapping_add(99_111));

        let dithered_subsoil_depth =
            (biome.subsoil_depth as f32 + transition_noise * 1.6).round() as i32;

        if logical_depth <= dithered_subsoil_depth.max(1) {
            return biome.subsoil_material;
        }

        // 4. Surface Crust (Crosta da Superfície): strictly 4 to 8 blocks of Mother Rock below subsoil
        let crust_noise =
            gradient_noise_3d(bx * 0.16, by * 0.16, bz * 0.16, seed.wrapping_add(33_445));
        let crust_depth =
            dithered_subsoil_depth.max(1) + 4 + ((crust_noise + 1.0) * 2.0).round() as i32;

        if logical_depth <= crust_depth {
            let mother_rock = match biome.subsoil_material {
                Voxel::PermafrostDirt | Voxel::PermafrostCoarse | Voxel::PermafrostIcy => {
                    Voxel::Andesite
                }
                Voxel::LoamDirt | Voxel::LoamCoarse | Voxel::LoamRooted | Voxel::LoamMud => {
                    Voxel::Granite
                }
                Voxel::PeatDirt | Voxel::PeatCoarse | Voxel::PeatRooted | Voxel::PeatMud => {
                    Voxel::Argillite
                }
                Voxel::SiltDirt | Voxel::SiltCoarse | Voxel::SiltRooted | Voxel::SiltMud => {
                    Voxel::Chalk
                }
                Voxel::CalicheDirt | Voxel::CalicheCoarse => Voxel::Limestone,
                Voxel::LateriteDirt | Voxel::LateriteCoarse => Voxel::Basalt,
                Voxel::SandRed => Voxel::SandstoneRed,
                Voxel::SandDune => Voxel::SandstoneDune,
                Voxel::SandWhite => Voxel::SandstoneWhite,
                Voxel::SandCommon => Voxel::SandstoneCommon,
                _ => match biome.surface_material {
                    Voxel::PermafrostGrass
                    | Voxel::PermafrostDirt
                    | Voxel::PermafrostCoarse
                    | Voxel::PermafrostIcy => Voxel::Andesite,
                    Voxel::LoamGrass
                    | Voxel::LoamMulch
                    | Voxel::LoamDirt
                    | Voxel::LoamCoarse
                    | Voxel::LoamRooted
                    | Voxel::LoamMud => Voxel::Granite,
                    Voxel::PeatGrass
                    | Voxel::PeatDirt
                    | Voxel::PeatCoarse
                    | Voxel::PeatRooted
                    | Voxel::PeatMud => Voxel::Argillite,
                    Voxel::SiltGrass
                    | Voxel::SiltSnow
                    | Voxel::SiltDirt
                    | Voxel::SiltCoarse
                    | Voxel::SiltRooted
                    | Voxel::SiltMud => Voxel::Chalk,
                    Voxel::CalicheGrass | Voxel::CalicheDirt | Voxel::CalicheCoarse => {
                        Voxel::Limestone
                    }
                    Voxel::LateriteGrass | Voxel::LateriteDirt | Voxel::LateriteCoarse => {
                        Voxel::Basalt
                    }
                    Voxel::SandRed => Voxel::SandstoneRed,
                    Voxel::SandDune => Voxel::SandstoneDune,
                    Voxel::SandWhite => Voxel::SandstoneWhite,
                    Voxel::SandCommon => Voxel::SandstoneCommon,
                    _ => biome.primary_stone,
                },
            };
            return mother_rock;
        }

        // 5. Intrusive Rock Pockets ("Bolsões" / Plutons / Lenses)
        // Independent 3D ellipsoidal intrusions across the planetary crust (only beneath surface crust):
        if block_y > self.abyssal_crust_y {
            // A. Marble Pockets (Metamorphic noble marble: depth -30 to -170)
            if (-170..=-30).contains(&block_y) {
                let marble_noise =
                    gradient_noise_3d(bx * 0.030, by * 0.040, bz * 0.030, seed.wrapping_add(112_358));
                if marble_noise > 0.60 {
                    return Voxel::Marble;
                }
            }

            // B. Granite Plutons (Continental batholiths: depth -10 to -100)
            if (-100..=-10).contains(&block_y) {
                let granite_noise =
                    gradient_noise_3d(bx * 0.024, by * 0.032, bz * 0.024, seed.wrapping_add(271_828));
                if granite_noise > 0.62 {
                    return Voxel::Granite;
                }
            }

            // C. Basalt Volcanic Columns / Conduits (depth -60 to -240)
            if (-240..=-60).contains(&block_y) {
                let basalt_noise =
                    gradient_noise_3d(bx * 0.032, by * 0.018, bz * 0.032, seed.wrapping_add(314_159));
                if basalt_noise > 0.64 {
                    return Voxel::Basalt;
                }
            }

            // D. Argillite Sedimentary Lenses (dense claystone: depth -15 to -80)
            // Smaller, discreet pockets with lower spawn frequency
            if (-80..=-15).contains(&block_y) {
                let argillite_noise =
                    gradient_noise_3d(bx * 0.055, by * 0.075, bz * 0.055, seed.wrapping_add(141_421));
                if argillite_noise > 0.72 {
                    return Voxel::Argillite;
                }
            }
        }

        // 6. Geological Depth Stratification (Macro-Estratos)
        // Gentle undulating wave warping the vertical boundaries
        let wave = gradient_noise_3d(bx * 0.012, by * 0.018, bz * 0.012, seed.wrapping_add(77_889))
            * 10.0;
        let eff_y = block_y as f32 + wave;

        // Macro-Estrato 1: Upper & Middle Crust (down to ~-110)
        // Dominated by Dolomite (the classic continental stone baseline), with stratified bands of Andesite and Limestone
        if eff_y > self.mid_crust_y as f32 {
            let band_noise =
                gradient_noise_3d(bx * 0.020, by * 0.045, bz * 0.020, seed.wrapping_add(55_667));
            if band_noise > 0.38 {
                Voxel::Andesite
            } else if band_noise < -0.38 {
                Voxel::Limestone
            } else {
                Voxel::Dolomite
            }
        }
        // Macro-Estrato 2: Deep Crust (~-110 to ~-190)
        // Dominated by Slate (metamorphic foundation) with intrusive Gabbro massifs
        else if eff_y > self.deep_crust_y as f32 {
            let gabbro_noise =
                gradient_noise_3d(bx * 0.024, by * 0.028, bz * 0.024, seed.wrapping_add(88_991));
            if gabbro_noise > 0.22 {
                Voxel::Gabbro
            } else {
                Voxel::Slate
            }
        }
        // Macro-Estrato 3: Abyssal Geothermal Crust (~-190 down to Mantle)
        // Extreme depth: Gabbro, Slate, Volcanic Basalt, Obsidian, and Magma
        else {
            let deep_noise =
                gradient_noise_3d(bx * 0.025, by * 0.035, bz * 0.025, seed.wrapping_add(42_069));
            if eff_y < -230.0 && deep_noise > 0.42 {
                Voxel::Magma
            } else if deep_noise < -0.52 && eff_y < -210.0 {
                Voxel::Obsidian
            } else if deep_noise > 0.28 {
                Voxel::Basalt
            } else if deep_noise < -0.15 {
                Voxel::Slate
            } else {
                Voxel::Gabbro
            }
        }
    }
}

#[inline]
fn ly_coord(world_y: i32) -> f32 {
    world_y as f32 + 0.5
}
