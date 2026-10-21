use crate::{generation::biome::BiomeType, world::Voxel};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeSpecies {
    Oak,
    Birch,
    Pine,
    Acacia,
    Cherry,
    Mahogany,
    Mangrove,
    Maple,
    Palm,
    Willow,
    Yew,
    Charred,
    Dead,
    Cactus,
}

impl TreeSpecies {
    pub fn log_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::OakLog,
            Self::Birch => Voxel::BirchLog,
            Self::Pine => Voxel::PineLog,
            Self::Acacia => Voxel::AcaciaLog,
            Self::Cherry => Voxel::CherryLog,
            Self::Mahogany => Voxel::MahoganyLog,
            Self::Mangrove => Voxel::MangroveLog,
            Self::Maple => Voxel::MapleLog,
            Self::Palm => Voxel::PalmLog,
            Self::Willow => Voxel::WillowLog,
            Self::Yew => Voxel::YewLog,
            Self::Charred => Voxel::RedwoodLog,
            Self::Dead => Voxel::BaobabLog,
            Self::Cactus => Voxel::BambooStalk,
        }
    }

    pub fn wood_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::OakWood,
            Self::Birch => Voxel::BirchWood,
            Self::Pine => Voxel::PineWood,
            Self::Acacia => Voxel::AcaciaWood,
            Self::Cherry => Voxel::CherryWood,
            Self::Mahogany => Voxel::MahoganyWood,
            Self::Mangrove => Voxel::MangroveWood,
            Self::Maple => Voxel::MapleWood,
            Self::Palm => Voxel::PalmWood,
            Self::Willow => Voxel::WillowWood,
            Self::Yew => Voxel::YewWood,
            Self::Charred => Voxel::RedwoodWood,
            Self::Dead => Voxel::BaobabWood,
            Self::Cactus => Voxel::BambooStalk,
        }
    }

    pub fn leaves_voxel(self) -> Option<Voxel> {
        match self {
            Self::Oak => Some(Voxel::OakLeaves),
            Self::Birch => Some(Voxel::BirchLeaves),
            Self::Pine => Some(Voxel::PineLeaves),
            Self::Acacia => Some(Voxel::AcaciaLeaves),
            Self::Cherry => Some(Voxel::CherryLeaves),
            Self::Mahogany => Some(Voxel::MahoganyLeaves),
            Self::Mangrove => Some(Voxel::MangroveLeaves),
            Self::Maple => Some(Voxel::MapleLeavesRed),
            Self::Palm => Some(Voxel::PalmLeaves),
            Self::Willow => Some(Voxel::WillowLeaves),
            Self::Yew => Some(Voxel::YewLeaves),
            Self::Charred | Self::Dead | Self::Cactus => None,
        }
    }
}

/// Selects the appropriate leaf voxel variant based on species, cell hash, and biome.
pub fn leaves_voxel_variant(species: TreeSpecies, hash: u32, _biome: BiomeType) -> Option<Voxel> {
    match species {
        TreeSpecies::Maple => match (hash >> 8) % 3 {
            0 => Some(Voxel::MapleLeavesRed),
            1 => Some(Voxel::MapleLeavesOrange),
            _ => Some(Voxel::MapleLeavesYellow),
        },
        _ => species.leaves_voxel(),
    }
}

/// Maps each of the 11 canonical biomes to its procedural tree species
/// and cell spawn probability (per 5x5 block cell).
pub fn biome_tree_profile(biome: BiomeType) -> Option<(TreeSpecies, f32)> {
    match biome {
        BiomeType::Plains => Some((TreeSpecies::Oak, 0.04)),
        BiomeType::Forest => Some((TreeSpecies::Oak, 0.65)),
        BiomeType::Desert => Some((TreeSpecies::Cactus, 0.05)),
        BiomeType::Taiga => Some((TreeSpecies::Pine, 0.55)),
        BiomeType::Highlands => Some((TreeSpecies::Pine, 0.05)),
        BiomeType::Savanna => Some((TreeSpecies::Acacia, 0.15)),
        BiomeType::CoastalBeachReef => Some((TreeSpecies::Palm, 0.08)),
        BiomeType::MangroveWetland => Some((TreeSpecies::Mangrove, 0.45)),
        BiomeType::RiversLakes => Some((TreeSpecies::Willow, 0.25)),
        BiomeType::OpenOcean | BiomeType::AbyssalTrench => None,
    }
}

/// Evaluates whether a surface material is suitable for a given tree species to root.
pub fn is_soil_valid_for_species(species: TreeSpecies, surface_voxel: Voxel) -> bool {
    match species {
        TreeSpecies::Cactus => matches!(
            surface_voxel,
            Voxel::SandCommon
                | Voxel::SandRed
                | Voxel::SandWhite
                | Voxel::CalicheCoarse
                | Voxel::CalicheDirt
        ),
        TreeSpecies::Palm => matches!(
            surface_voxel,
            Voxel::SandCommon | Voxel::SandWhite | Voxel::LoamGrass | Voxel::LoamDirt
        ),
        TreeSpecies::Dead | TreeSpecies::Charred => matches!(
            surface_voxel,
            Voxel::AshPumice
                | Voxel::AshVolcanic
                | Voxel::LoamDirt
                | Voxel::LoamMulch
                | Voxel::PeatMulch
        ),
        TreeSpecies::Mangrove | TreeSpecies::Willow => matches!(
            surface_voxel,
            Voxel::PeatMud
                | Voxel::LoamMud
                | Voxel::SiltMud
                | Voxel::ClayBlue
                | Voxel::ClayRed
                | Voxel::PeatDirt
                | Voxel::LoamDirt
                | Voxel::PeatGrass
        ),
        TreeSpecies::Pine => matches!(
            surface_voxel,
            Voxel::SiltSnow
                | Voxel::PeatSnow
                | Voxel::LoamSnow
                | Voxel::PermafrostDirt
                | Voxel::PermafrostGrass
                | Voxel::LoamGrass
                | Voxel::LoamDirt
                | Voxel::GravelSandy
        ),
        _ => {
            surface_voxel.is_grass_soil()
                || matches!(
                    surface_voxel,
                    Voxel::LoamDirt
                        | Voxel::LoamMulch
                        | Voxel::PeatMulch
                        | Voxel::SiltMulch
                        | Voxel::LoamRooted
                        | Voxel::PeatRooted
                        | Voxel::SiltRooted
                        | Voxel::PeatDirt
                        | Voxel::SiltDirt
                )
        }
    }
}

/// Fast, deterministic hash function for 2D cellular grid sampling.
#[inline]
pub fn hash_tree_cell(cell_x: i32, cell_z: i32, seed: u32) -> u32 {
    let mut h = seed
        .wrapping_add((cell_x as u32).wrapping_mul(0x85ebca6b))
        .wrapping_add((cell_z as u32).wrapping_mul(0xc2b2ae35));
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

/// Identifies trunk and structural wood blocks.
#[inline]
pub fn is_trunk(voxel: Voxel) -> bool {
    matches!(
        voxel,
        Voxel::AcaciaLog
            | Voxel::BirchLog
            | Voxel::CherryLog
            | Voxel::MahoganyLog
            | Voxel::MangroveLog
            | Voxel::MapleLog
            | Voxel::OakLog
            | Voxel::PalmLog
            | Voxel::PineLog
            | Voxel::WillowLog
            | Voxel::YewLog
            | Voxel::RedwoodLog
            | Voxel::BaobabLog
            | Voxel::BambooStalk
            | Voxel::MangroveRoots
    )
}

/// Identifies leaf voxels.
#[inline]
pub fn is_leaves(voxel: Voxel) -> bool {
    voxel.is_leaves()
}

/// Procedurally generates the voxel blocks for a tree or cactus.
pub fn generate_tree_voxels<F>(
    tx: i32,
    ty: i32,
    tz: i32,
    species: TreeSpecies,
    biome: BiomeType,
    hash: u32,
    mut emit: F,
) where
    F: FnMut(i32, i32, i32, Voxel),
{
    if species == TreeSpecies::Cactus {
        let height = 2 + ((hash >> 12) % 3) as i32;
        for dy in 1..=height {
            emit(tx, ty + dy, tz, Voxel::BambooStalk);
        }
        return;
    }

    if species == TreeSpecies::Dead || species == TreeSpecies::Charred {
        let height = 4 + ((hash >> 12) % 3) as i32;
        let (trunk_voxel, wood_voxel) = (species.log_voxel(), species.wood_voxel());

        for dy in 1..=height {
            emit(tx, ty + dy, tz, trunk_voxel);
        }

        let branch_dir = ((hash >> 15) % 4) as usize;
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (bx1, bz1) = dirs[branch_dir];
        emit(tx + bx1, ty + height - 1, tz + bz1, wood_voxel);
        let (bx2, bz2) = dirs[(branch_dir + 2) % 4];
        emit(tx + bx2, ty + height, tz + bz2, wood_voxel);
        return;
    }

    let height = 6 + ((hash >> 12) % 3) as i32;
    let log_voxel = species.log_voxel();
    let leaves_voxel = leaves_voxel_variant(species, hash, biome);

    for dy in 1..=height {
        emit(tx, ty + dy, tz, log_voxel);
    }

    if species == TreeSpecies::Mangrove {
        for &(rdx, rdz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            emit(tx + rdx, ty + 1, tz + rdz, Voxel::MangroveRoots);
        }
    }

    if let Some(leaf_voxel) = leaves_voxel {
        for dy in (height - 2)..=height {
            let is_top_layer = dy == height;
            let radius: i32 = if is_top_layer { 1 } else { 2 };

            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if dx == 0 && dz == 0 && dy <= height {
                        continue;
                    }

                    if radius == 2 && dx.abs() == 2 && dz.abs() == 2 {
                        if dy == height - 1 {
                            let corner_hash = hash.wrapping_add((dx * 31 + dz * 17) as u32);
                            if (corner_hash & 3) != 0 {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }

                    emit(tx + dx, ty + dy, tz + dz, leaf_voxel);
                }
            }
        }

        for &(dx, dz) in &[(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            emit(tx + dx, ty + height + 1, tz + dz, leaf_voxel);
        }
    }
}
