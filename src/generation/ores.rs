use bevy::prelude::*;

use crate::{
    core::noise::hash_3d,
    world::{CHUNK_SIZE, Voxel},
};

/// The 17 procedural mineral vein types that embed inside planetary rock matrices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineralType {
    Coal,
    Copper,
    Tin,
    Iron,
    Zinc,
    Gold,
    Silver,
    Platinum,
    Uranium,
    Sulfur,
    Cinnabar,
    Lazurite,
    Diamond,
    Emerald,
    Opal,
    Onyx,
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

pub const ORE_PROFILES: [OreVeinProfile; 17] = [
    // Base & Industrial Metals (reduced by 55-60%)
    OreVeinProfile { mineral: MineralType::Iron, min_y: -210, max_y: 90, veins_per_chunk: 5, vein_radius: 1.6 },
    OreVeinProfile { mineral: MineralType::Copper, min_y: -90, max_y: 80, veins_per_chunk: 4, vein_radius: 1.5 },
    OreVeinProfile { mineral: MineralType::Tin, min_y: -100, max_y: 70, veins_per_chunk: 3, vein_radius: 1.4 },
    OreVeinProfile { mineral: MineralType::Zinc, min_y: -150, max_y: 40, veins_per_chunk: 3, vein_radius: 1.4 },
    // Carbon & Chemicals (Coal reduced from 16 to 6, others to 2)
    OreVeinProfile { mineral: MineralType::Coal, min_y: -70, max_y: 110, veins_per_chunk: 6, vein_radius: 1.7 },
    OreVeinProfile { mineral: MineralType::Sulfur, min_y: -245, max_y: -150, veins_per_chunk: 2, vein_radius: 1.3 },
    OreVeinProfile { mineral: MineralType::Cinnabar, min_y: -245, max_y: -160, veins_per_chunk: 2, vein_radius: 1.3 },
    OreVeinProfile { mineral: MineralType::Lazurite, min_y: -130, max_y: 30, veins_per_chunk: 2, vein_radius: 1.3 },
    // Precious & Strategic Metals
    OreVeinProfile { mineral: MineralType::Gold, min_y: -220, max_y: -20, veins_per_chunk: 2, vein_radius: 1.3 },
    OreVeinProfile { mineral: MineralType::Silver, min_y: -180, max_y: -10, veins_per_chunk: 2, vein_radius: 1.4 },
    OreVeinProfile { mineral: MineralType::Platinum, min_y: -245, max_y: -140, veins_per_chunk: 1, vein_radius: 1.2 },
    OreVeinProfile { mineral: MineralType::Uranium, min_y: -245, max_y: -110, veins_per_chunk: 1, vein_radius: 1.2 },
    // Precious Gemstones
    OreVeinProfile { mineral: MineralType::Diamond, min_y: -245, max_y: -130, veins_per_chunk: 1, vein_radius: 1.2 },
    OreVeinProfile { mineral: MineralType::Emerald, min_y: -190, max_y: -40, veins_per_chunk: 1, vein_radius: 1.2 },
    OreVeinProfile { mineral: MineralType::Opal, min_y: -60, max_y: 40, veins_per_chunk: 2, vein_radius: 1.3 },
    OreVeinProfile { mineral: MineralType::Onyx, min_y: -210, max_y: -100, veins_per_chunk: 2, vein_radius: 1.3 },
    OreVeinProfile { mineral: MineralType::GemVein, min_y: -210, max_y: 20, veins_per_chunk: 2, vein_radius: 1.4 },
];

/// Maps a mineral vein type and host rock to the exact procedural ore-in-stone voxel block.
#[inline]
pub fn host_to_ore_vein(mineral: MineralType, host: Voxel) -> Option<Voxel> {
    match mineral {
        MineralType::Coal => match host {
            Voxel::SandstoneCommon => Some(Voxel::OreCoalSandstoneCommon),
            Voxel::Granite => Some(Voxel::OreCoalGranite),
            Voxel::Dolomite => Some(Voxel::OreCoalDolomite),
            Voxel::Limestone => Some(Voxel::OreCoalLimestone),
            Voxel::Chalk => Some(Voxel::OreCoalChalk),
            Voxel::Argillite => Some(Voxel::OreCoalArgillite),
            _ => None,
        },
        MineralType::Copper => match host {
            Voxel::SandstoneCommon => Some(Voxel::OreCopperSandstoneCommon),
            Voxel::SandstoneRed => Some(Voxel::OreCopperSandstoneRed),
            Voxel::SandstoneWhite => Some(Voxel::OreCopperSandstoneWhite),
            Voxel::Granite => Some(Voxel::OreCopperGranite),
            Voxel::Andesite => Some(Voxel::OreCopperAndesite),
            Voxel::Dolomite => Some(Voxel::OreCopperDolomite),
            _ => None,
        },
        MineralType::Tin => match host {
            Voxel::Granite => Some(Voxel::OreTinGranite),
            Voxel::Andesite => Some(Voxel::OreTinAndesite),
            Voxel::Dolomite => Some(Voxel::OreTinDolomite),
            Voxel::Chalk => Some(Voxel::OreTinChalk),
            _ => None,
        },
        MineralType::Iron => match host {
            Voxel::Granite => Some(Voxel::OreIronGranite),
            Voxel::Andesite => Some(Voxel::OreIronAndesite),
            Voxel::Dolomite => Some(Voxel::OreIronDolomite),
            Voxel::Slate => Some(Voxel::OreIronSlate),
            Voxel::Gabbro => Some(Voxel::OreIronGabbro),
            Voxel::Basalt => Some(Voxel::OreIronBasalt),
            _ => None,
        },
        MineralType::Zinc => match host {
            Voxel::Andesite => Some(Voxel::OreZincAndesite),
            Voxel::Dolomite => Some(Voxel::OreZincDolomite),
            Voxel::Argillite => Some(Voxel::OreZincArgillite),
            Voxel::Slate => Some(Voxel::OreZincSlate),
            _ => None,
        },
        MineralType::Gold => match host {
            Voxel::SandstoneRed => Some(Voxel::OreGoldSandstoneRed),
            Voxel::Granite => Some(Voxel::OreGoldGranite),
            Voxel::Slate => Some(Voxel::OreGoldSlate),
            Voxel::Gabbro => Some(Voxel::OreGoldGabbro),
            _ => None,
        },
        MineralType::Silver => match host {
            Voxel::Andesite => Some(Voxel::OreSilverAndesite),
            Voxel::Slate => Some(Voxel::OreSilverSlate),
            Voxel::Marble => Some(Voxel::OreSilverMarble),
            _ => None,
        },
        MineralType::Platinum => match host {
            Voxel::Gabbro => Some(Voxel::OrePlatinumGabbro),
            Voxel::Basalt => Some(Voxel::OrePlatinumBasalt),
            _ => None,
        },
        MineralType::Uranium => match host {
            Voxel::Slate => Some(Voxel::OreUraniumSlate),
            Voxel::Gabbro => Some(Voxel::OreUraniumGabbro),
            _ => None,
        },
        MineralType::Lazurite => match host {
            Voxel::Limestone => Some(Voxel::OreLazuriteLimestone),
            Voxel::Marble => Some(Voxel::OreLazuriteMarble),
            _ => None,
        },
        MineralType::Diamond => match host {
            Voxel::Gabbro => Some(Voxel::GemDiamondGabbro),
            Voxel::Basalt => Some(Voxel::GemDiamondBasalt),
            _ => None,
        },
        MineralType::Emerald => match host {
            Voxel::Slate => Some(Voxel::GemEmeraldSlate),
            Voxel::Marble => Some(Voxel::GemEmeraldMarble),
            _ => None,
        },
        MineralType::Sulfur => match host {
            Voxel::Basalt => Some(Voxel::OreSulfurBasalt),
            _ => None,
        },
        MineralType::Cinnabar => match host {
            Voxel::Basalt => Some(Voxel::OreCinnabarBasalt),
            _ => None,
        },
        MineralType::Opal => match host {
            Voxel::SandstoneWhite => Some(Voxel::GemOpalSandstoneWhite),
            _ => None,
        },
        MineralType::Onyx => match host {
            Voxel::Slate => Some(Voxel::GemOnyxSlate),
            _ => None,
        },
        MineralType::GemVein => match host {
            Voxel::Dolomite => Some(Voxel::GemVeinDolomite),
            Voxel::Granite => Some(Voxel::GemVeinGranite),
            Voxel::Andesite => Some(Voxel::GemVeinAndesite),
            Voxel::Slate => Some(Voxel::GemVeinSlate),
            Voxel::Gabbro => Some(Voxel::GemVeinGabbro),
            Voxel::Basalt => Some(Voxel::GemVeinBasalt),
            Voxel::Marble => Some(Voxel::GemVeinMarble),
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
