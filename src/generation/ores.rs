use bevy::prelude::*;

use crate::{
    core::noise::hash_3d,
    world::{CHUNK_SIZE, Voxel},
};

/// The 10 procedural mineral vein types that spawn inside planetary rock matrices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineralType {
    Coal,
    Copper,
    Iron,
    Gold,
    Silver,
    Uranium,
    Sulfur,
    Diamond,
    Emerald,
    GemVein,
}

/// Generation parameters for procedural ore veins.
pub struct OreVeinProfile {
    pub mineral: MineralType,
    pub min_y: i32,
    pub max_y: i32,
    pub veins_per_chunk: u32,
    pub vein_radius: f32,
}

pub const ORE_PROFILES: [OreVeinProfile; 10] = [
    // Base & Industrial Metals
    OreVeinProfile { mineral: MineralType::Iron, min_y: -210, max_y: 90, veins_per_chunk: 5, vein_radius: 1.6 },
    OreVeinProfile { mineral: MineralType::Copper, min_y: -90, max_y: 80, veins_per_chunk: 4, vein_radius: 1.5 },
    // Carbon & Chemicals
    OreVeinProfile { mineral: MineralType::Coal, min_y: -70, max_y: 110, veins_per_chunk: 6, vein_radius: 1.7 },
    OreVeinProfile { mineral: MineralType::Sulfur, min_y: -245, max_y: -150, veins_per_chunk: 2, vein_radius: 1.3 },
    // Precious & Strategic Metals
    OreVeinProfile { mineral: MineralType::Gold, min_y: -220, max_y: -20, veins_per_chunk: 2, vein_radius: 1.3 },
    OreVeinProfile { mineral: MineralType::Silver, min_y: -180, max_y: -10, veins_per_chunk: 2, vein_radius: 1.4 },
    OreVeinProfile { mineral: MineralType::Uranium, min_y: -245, max_y: -110, veins_per_chunk: 1, vein_radius: 1.2 },
    // Precious Gemstones
    OreVeinProfile { mineral: MineralType::Diamond, min_y: -245, max_y: -130, veins_per_chunk: 1, vein_radius: 1.2 },
    OreVeinProfile { mineral: MineralType::Emerald, min_y: -190, max_y: -40, veins_per_chunk: 1, vein_radius: 1.2 },
    OreVeinProfile { mineral: MineralType::GemVein, min_y: -210, max_y: 20, veins_per_chunk: 2, vein_radius: 1.4 },
];

/// Maps a mineral vein type and host rock to its raw ore or rough gemstone voxel block.
#[inline]
pub fn host_to_ore_vein(mineral: MineralType, host: Voxel) -> Option<Voxel> {
    match mineral {
        MineralType::Coal => match host {
            Voxel::SandstoneCommon
            | Voxel::Granite
            | Voxel::Dolomite
            | Voxel::Limestone
            | Voxel::Chalk
            | Voxel::Argillite => Some(Voxel::OreCoalRaw),
            _ => None,
        },
        MineralType::Copper => match host {
            Voxel::SandstoneCommon
            | Voxel::SandstoneRed
            | Voxel::SandstoneWhite
            | Voxel::Granite
            | Voxel::Andesite
            | Voxel::Dolomite => Some(Voxel::OreCopperRaw),
            _ => None,
        },
        MineralType::Iron => match host {
            Voxel::Granite
            | Voxel::Andesite
            | Voxel::Dolomite
            | Voxel::Slate
            | Voxel::Gabbro
            | Voxel::Basalt => Some(Voxel::OreIronRaw),
            _ => None,
        },
        MineralType::Gold => match host {
            Voxel::SandstoneRed
            | Voxel::Granite
            | Voxel::Slate
            | Voxel::Gabbro => Some(Voxel::OreGoldRaw),
            _ => None,
        },
        MineralType::Silver => match host {
            Voxel::Andesite
            | Voxel::Slate
            | Voxel::Marble => Some(Voxel::OreSilverRaw),
            _ => None,
        },
        MineralType::Uranium => match host {
            Voxel::Slate
            | Voxel::Gabbro => Some(Voxel::OreUraniumRaw),
            _ => None,
        },
        MineralType::Diamond => match host {
            Voxel::Gabbro
            | Voxel::Basalt => Some(Voxel::GemDiamondRough),
            _ => None,
        },
        MineralType::Emerald => match host {
            Voxel::Slate
            | Voxel::Marble => Some(Voxel::GemEmeraldRough),
            _ => None,
        },
        MineralType::Sulfur => match host {
            Voxel::Basalt => Some(Voxel::OreSulfurRaw),
            _ => None,
        },
        MineralType::GemVein => match host {
            Voxel::Dolomite | Voxel::Granite => Some(Voxel::GemTopazRough),
            Voxel::Andesite | Voxel::Slate => Some(Voxel::GemJadeRough),
            Voxel::Gabbro | Voxel::Basalt => Some(Voxel::GemSapphireRough),
            Voxel::Marble => Some(Voxel::GemRubyRough),
            _ => None,
        },
    }
}

/// Populates procedural ore veins within a chunk, replacing only valid host rocks.
pub fn populate_chunk_ores(voxels: &mut [Voxel], chunk_origin: IVec3, seed: u32) {
    let chunk_min_y = chunk_origin.y;
    let chunk_max_y = chunk_origin.y + CHUNK_SIZE as i32 - 1;

    for (spec_idx, profile) in ORE_PROFILES.iter().enumerate() {
        if chunk_max_y < profile.min_y || chunk_min_y > profile.max_y {
            continue;
        }

        let ore_seed = seed.wrapping_add((spec_idx as u32 + 1) * 7919);

        for v in 0..profile.veins_per_chunk {
            let vein_hash = hash_3d(chunk_origin.x, chunk_origin.y, chunk_origin.z, ore_seed.wrapping_add(v * 67));
            let center_x = (vein_hash & 0x0F) as i32;
            let center_y = ((vein_hash >> 4) & 0x0F) as i32;
            let center_z = ((vein_hash >> 8) & 0x0F) as i32;

            let world_y = chunk_origin.y + center_y;
            if world_y < profile.min_y || world_y > profile.max_y {
                continue;
            }

            let r = profile.vein_radius;
            let r_sq = r * r;
            let r_ceil = r.ceil() as i32;

            for dz in -r_ceil..=r_ceil {
                let lz = center_z + dz;
                if !(0..CHUNK_SIZE as i32).contains(&lz) {
                    continue;
                }
                for dy in -r_ceil..=r_ceil {
                    let ly = center_y + dy;
                    if !(0..CHUNK_SIZE as i32).contains(&ly) {
                        continue;
                    }
                    for dx in -r_ceil..=r_ceil {
                        let lx = center_x + dx;
                        if !(0..CHUNK_SIZE as i32).contains(&lx) {
                            continue;
                        }

                        let dist_sq = (dx * dx + dy * dy + dz * dz) as f32;
                        if dist_sq > r_sq {
                            continue;
                        }

                        let idx = lx as usize + lz as usize * CHUNK_SIZE + ly as usize * CHUNK_SIZE * CHUNK_SIZE;
                        let host = voxels[idx];

                        if let Some(ore_voxel) = host_to_ore_vein(profile.mineral, host) {
                            voxels[idx] = ore_voxel;
                        }
                    }
                }
            }
        }
    }
}

/// The 6 mineral crystal geode types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrystalGeodeType {
    Amethyst,
    Aquamarine,
    Citrine,
    Fluorite,
    PinkSalt,
    Selenite,
}

impl CrystalGeodeType {
    pub fn shell_voxel(self) -> Voxel {
        match self {
            Self::Amethyst | Self::Citrine => Voxel::Basalt,
            Self::Aquamarine | Self::Fluorite | Self::PinkSalt | Self::Selenite => Voxel::CaveCalcite,
        }
    }

    pub fn crystal_voxel(self) -> Voxel {
        match self {
            Self::Amethyst => Voxel::CrystalAmethystBlock,
            Self::Aquamarine => Voxel::CrystalAquamarineBlock,
            Self::Citrine => Voxel::CrystalCitrineBlock,
            Self::Fluorite => Voxel::CrystalFluoriteBlock,
            Self::PinkSalt => Voxel::CrystalPinkSaltBlock,
            Self::Selenite => Voxel::CrystalSeleniteBlock,
        }
    }
}

/// Generates naturally occurring spherical crystal geodes in the underground (Minecraft style).
pub fn populate_chunk_geodes(voxels: &mut [Voxel], chunk_origin: IVec3, seed: u32) {
    let chunk_min_y = chunk_origin.y;
    let chunk_max_y = chunk_origin.y + CHUNK_SIZE as i32 - 1;

    // Geodes generate strictly in the deep underground
    if chunk_max_y < -230 || chunk_min_y > -35 {
        return;
    }

    // Grid spacing for geode candidates: 48 voxels (3 chunks)
    const GRID_SPACING: i32 = 48;
    let cell_min_x = (chunk_origin.x - 8).div_euclid(GRID_SPACING);
    let cell_max_x = (chunk_origin.x + CHUNK_SIZE as i32 + 8).div_euclid(GRID_SPACING);
    let cell_min_y = (chunk_origin.y - 8).div_euclid(GRID_SPACING);
    let cell_max_y = (chunk_origin.y + CHUNK_SIZE as i32 + 8).div_euclid(GRID_SPACING);
    let cell_min_z = (chunk_origin.z - 8).div_euclid(GRID_SPACING);
    let cell_max_z = (chunk_origin.z + CHUNK_SIZE as i32 + 8).div_euclid(GRID_SPACING);

    let geode_seed = seed.wrapping_add(987_654);

    for cz in cell_min_z..=cell_max_z {
        for cy in cell_min_y..=cell_max_y {
            for cx in cell_min_x..=cell_max_x {
                let cell_hash = hash_3d(cx, cy, cz, geode_seed);

                // ~25% chance of a geode per 48x48x48 cell
                if (cell_hash % 100) >= 25 {
                    continue;
                }

                let geode_gx = cx * GRID_SPACING + 8 + ((cell_hash & 0x1F) as i32);
                let geode_gy = cy * GRID_SPACING + 8 + (((cell_hash >> 5) & 0x1F) as i32);
                let geode_gz = cz * GRID_SPACING + 8 + (((cell_hash >> 10) & 0x1F) as i32);

                if !(-225..=-40).contains(&geode_gy) {
                    continue;
                }

                let crystal_type = match (cell_hash >> 15) % 6 {
                    0 => CrystalGeodeType::Amethyst,
                    1 => CrystalGeodeType::Aquamarine,
                    2 => CrystalGeodeType::Citrine,
                    3 => CrystalGeodeType::Fluorite,
                    4 => CrystalGeodeType::PinkSalt,
                    _ => CrystalGeodeType::Selenite,
                };

                let shell_mat = crystal_type.shell_voxel();
                let crystal_mat = crystal_type.crystal_voxel();

                // Geode radius: ~4.2 blocks
                let radius = 3.9 + (((cell_hash >> 18) & 3) as f32) * 0.35;
                let r_outer_sq = radius * radius;
                let r_inner_crystal = radius - 1.0;
                let r_crystal_sq = r_inner_crystal * r_inner_crystal;
                let r_hollow = radius - 2.0;
                let r_hollow_sq = r_hollow * r_hollow;

                let r_ceil = (radius + 1.0).ceil() as i32;

                // Bounding box intersection check with current chunk
                if geode_gx + r_ceil < chunk_origin.x
                    || geode_gx - r_ceil >= chunk_origin.x + CHUNK_SIZE as i32
                    || geode_gy + r_ceil < chunk_origin.y
                    || geode_gy - r_ceil >= chunk_origin.y + CHUNK_SIZE as i32
                    || geode_gz + r_ceil < chunk_origin.z
                    || geode_gz - r_ceil >= chunk_origin.z + CHUNK_SIZE as i32
                {
                    continue;
                }

                for dz in -r_ceil..=r_ceil {
                    let wz = geode_gz + dz;
                    let lz = wz - chunk_origin.z;
                    if !(0..CHUNK_SIZE as i32).contains(&lz) {
                        continue;
                    }
                    for dy in -r_ceil..=r_ceil {
                        let wy = geode_gy + dy;
                        let ly = wy - chunk_origin.y;
                        if !(0..CHUNK_SIZE as i32).contains(&ly) {
                            continue;
                        }
                        for dx in -r_ceil..=r_ceil {
                            let wx = geode_gx + dx;
                            let lx = wx - chunk_origin.x;
                            if !(0..CHUNK_SIZE as i32).contains(&lx) {
                                continue;
                            }

                            let dist_sq = (dx * dx + dy * dy + dz * dz) as f32;
                            if dist_sq > r_outer_sq {
                                continue;
                            }

                            let idx = lx as usize + lz as usize * CHUNK_SIZE + ly as usize * CHUNK_SIZE * CHUNK_SIZE;
                            let current = voxels[idx];

                            // Do not carve through mantle or surface
                            if current == Voxel::Mantle || current == Voxel::CobbledMantle {
                                continue;
                            }

                            if dist_sq <= r_hollow_sq {
                                // Hollow crystalline center
                                voxels[idx] = Voxel::Air;
                            } else if dist_sq <= r_crystal_sq {
                                // Solid crystal wall
                                voxels[idx] = crystal_mat;
                            } else {
                                // Protective outer shell (Basalt or CaveCalcite)
                                voxels[idx] = shell_mat;
                            }
                        }
                    }
                }
            }
        }
    }
}
