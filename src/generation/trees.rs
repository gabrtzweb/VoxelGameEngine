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
            Self::Oak => Voxel::Tree_Oak_Log,
            Self::Birch => Voxel::Tree_Birch_Log,
            Self::Pine => Voxel::Tree_Pine_Log,
            Self::Acacia => Voxel::Tree_Acacia_Log,
            Self::Cherry => Voxel::Tree_Cherry_Log,
            Self::Mahogany => Voxel::Tree_Mahogany_Log,
            Self::Mangrove => Voxel::Tree_Mangrove_Log,
            Self::Maple => Voxel::Tree_Maple_Log,
            Self::Palm => Voxel::Tree_Palm_Log,
            Self::Willow => Voxel::Tree_Willow_Log,
            Self::Yew => Voxel::Tree_Yew_Log,
            Self::Charred => Voxel::Tree_Charred_Log,
            Self::Dead => Voxel::Tree_Dead_Log,
            Self::Cactus => Voxel::Tree_Cactus,
        }
    }

    pub fn wood_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::Tree_Oak_Bark,
            Self::Birch => Voxel::Tree_Birch_Bark,
            Self::Pine => Voxel::Tree_Pine_Bark,
            Self::Acacia => Voxel::Tree_Acacia_Bark,
            Self::Cherry => Voxel::Tree_Cherry_Bark,
            Self::Mahogany => Voxel::Tree_Mahogany_Bark,
            Self::Mangrove => Voxel::Tree_Mangrove_Bark,
            Self::Maple => Voxel::Tree_Maple_Bark,
            Self::Palm => Voxel::Tree_Palm_Bark,
            Self::Willow => Voxel::Tree_Willow_Bark,
            Self::Yew => Voxel::Tree_Yew_Bark,
            Self::Charred => Voxel::Tree_Charred_Bark,
            Self::Dead => Voxel::Tree_Dead_Bark,
            Self::Cactus => Voxel::Tree_Cactus,
        }
    }

    pub fn leaves_voxel(self) -> Option<Voxel> {
        match self {
            Self::Oak => Some(Voxel::Tree_Oak_Leaves),
            Self::Birch => Some(Voxel::Tree_Birch_Leaves),
            Self::Pine => Some(Voxel::Tree_Pine_Leaves),
            Self::Acacia => Some(Voxel::Tree_Acacia_Leaves),
            Self::Cherry => Some(Voxel::Tree_Cherry_Leaves),
            Self::Mahogany => Some(Voxel::Tree_Mahogany_Leaves),
            Self::Mangrove => Some(Voxel::Tree_Mangrove_Leaves),
            Self::Maple => Some(Voxel::Tree_Maple_Leaves_Red),
            Self::Palm => Some(Voxel::Tree_Palm_Leaves),
            Self::Willow => Some(Voxel::Tree_Willow_Leaves),
            Self::Yew => Some(Voxel::Tree_Yew_Leaves),
            Self::Charred | Self::Dead | Self::Cactus => None,
        }
    }
}

/// Selects the appropriate leaf voxel variant based on species, biome, and cell hash.
pub fn leaves_voxel_variant(species: TreeSpecies, hash: u32, biome: BiomeType) -> Option<Voxel> {
    match species {
        TreeSpecies::Oak => {
            if biome == BiomeType::AncientWeald {
                match (hash >> 8) % 3 {
                    0 => Some(Voxel::Tree_Oak_Leaves),
                    1 => Some(Voxel::Tree_Oak_Leaves_Lush),
                    _ => Some(Voxel::Tree_Oak_Leaves_Flowering),
                }
            } else {
                Some(Voxel::Tree_Oak_Leaves)
            }
        }
        TreeSpecies::Maple => match (hash >> 8) % 3 {
            0 => Some(Voxel::Tree_Maple_Leaves_Red),
            1 => Some(Voxel::Tree_Maple_Leaves_Orange),
            _ => Some(Voxel::Tree_Maple_Leaves_Yellow),
        },
        _ => species.leaves_voxel(),
    }
}

/// Maps each of the 48 biomes from `docs/world_definition.md` to its procedural tree species
/// and cell spawn probability (per 5x5 block cell).
pub fn biome_tree_profile(biome: BiomeType) -> Option<(TreeSpecies, f32)> {
    match biome {
        // Forests & Woodlands (8 in total)
        BiomeType::AncientWeald => Some((TreeSpecies::Oak, 0.65)),
        BiomeType::AutumnalForest => Some((TreeSpecies::Maple, 0.65)),
        BiomeType::BirchCopse => Some((TreeSpecies::Birch, 0.55)),
        BiomeType::BlossomGrove => Some((TreeSpecies::Cherry, 0.50)),
        BiomeType::BorealTaiga => Some((TreeSpecies::Pine, 0.55)),
        BiomeType::DeadwoodThicket => Some((TreeSpecies::Dead, 0.40)),
        BiomeType::TropicalRainforest => Some((TreeSpecies::Mahogany, 0.70)),
        BiomeType::YewGrove => Some((TreeSpecies::Yew, 0.60)),

        // Plains & Open Lands (8 in total)
        BiomeType::AcaciaSavanna => Some((TreeSpecies::Acacia, 0.12)),
        BiomeType::Steppe => Some((TreeSpecies::Oak, 0.02)),
        BiomeType::Heath
        | BiomeType::Moorland
        | BiomeType::OutbackScrubland
        | BiomeType::PermafrostSteppe
        | BiomeType::SnowyTundra
        | BiomeType::VolcanicPlains => None,

        // Wetlands & Swamps (8 in total)
        BiomeType::CypressSwamp => Some((TreeSpecies::Pine, 0.40)),
        BiomeType::MangroveSwamp => Some((TreeSpecies::Mangrove, 0.45)),
        BiomeType::WeepingBayou => Some((TreeSpecies::Willow, 0.45)),
        BiomeType::TarPits => Some((TreeSpecies::Charred, 0.04)),
        BiomeType::Marshland
        | BiomeType::PeatBog
        | BiomeType::FungalBog
        | BiomeType::SludgeWastes => None,

        // Arid & Warm Lands (8 in total)
        BiomeType::Oasis => Some((TreeSpecies::Palm, 0.45)),
        BiomeType::RockyScrubland => Some((TreeSpecies::Cactus, 0.16)),
        BiomeType::DuneDesert => Some((TreeSpecies::Cactus, 0.04)),
        BiomeType::PaintedDesert => Some((TreeSpecies::Cactus, 0.05)),
        BiomeType::WhiteDesert => Some((TreeSpecies::Cactus, 0.03)),
        BiomeType::Badlands => Some((TreeSpecies::Cactus, 0.03)),
        BiomeType::ScorchedWastes | BiomeType::WindsweptCanyons => None,

        // Mountain Biomes (8 in total)
        BiomeType::AlpineTundra => Some((TreeSpecies::Pine, 0.03)),
        BiomeType::FrozenCaldera
        | BiomeType::GlacialPeaks
        | BiomeType::JaggedCrags
        | BiomeType::KarstPeaks
        | BiomeType::ScreeSlopes
        | BiomeType::ShaleBarrens
        | BiomeType::VolcanicFields => None,

        // Coastal & Aquatic Biomes (8 in total)
        BiomeType::Beach => Some((TreeSpecies::Palm, 0.08)),
        BiomeType::AbyssalTrench
        | BiomeType::BrackishEstuary
        | BiomeType::ChalkCliffs
        | BiomeType::CoastalCrags
        | BiomeType::DeepOcean
        | BiomeType::TemperateOcean
        | BiomeType::TidalMudflats => None,
    }
}

/// Evaluates whether a surface material is suitable for a given tree species to root.
pub fn is_soil_valid_for_species(species: TreeSpecies, surface_voxel: Voxel) -> bool {
    match species {
        TreeSpecies::Cactus => matches!(
            surface_voxel,
            Voxel::Soil_Sand
                | Voxel::Soil_Red_Sand
                | Voxel::Soil_White_Sand
                | Voxel::Soil_Scorched_Sand
                | Voxel::Soil_Scorched_Red_Sand
                | Voxel::Soil_Packed_Dirt
        ),
        TreeSpecies::Palm => matches!(
            surface_voxel,
            Voxel::Soil_Sand
                | Voxel::Soil_White_Sand
                | Voxel::Soil_Grass
                | Voxel::Soil_Dirt
                | Voxel::Soil_Packed_Dirt
        ),
        TreeSpecies::Dead | TreeSpecies::Charred => matches!(
            surface_voxel,
            Voxel::Soil_Ash
                | Voxel::Soil_Black_Sand
                | Voxel::Soil_Scorched_Black_Sand
                | Voxel::Soil_Scorched_Sand
                | Voxel::Soil_Scorched_Red_Sand
                | Voxel::Soil_Dirt
                | Voxel::Soil_Sand
                | Voxel::Soil_Packed_Dirt
                | Voxel::Soil_Mulch
                | Voxel::Soil_Peat_Mulch
        ),
        TreeSpecies::Mangrove | TreeSpecies::Willow => matches!(
            surface_voxel,
            Voxel::Soil_Mud
                | Voxel::Soil_Packed_Mud
                | Voxel::Soil_Silt_Mulch
                | Voxel::Soil_Peat_Mulch
                | Voxel::Soil_Mulch
                | Voxel::Soil_Grass
                | Voxel::Soil_Clay
                | Voxel::Soil_Dirt
        ),
        TreeSpecies::Pine => matches!(
            surface_voxel,
            Voxel::Soil_Snowy_Peat
                | Voxel::Soil_Snowy_Grass
                | Voxel::Soil_Snowy_Silt
                | Voxel::Soil_Grass
                | Voxel::Soil_Mud
                | Voxel::Soil_Peat_Grass
                | Voxel::Soil_Dirt
                | Voxel::Soil_Mulch
        ),
        _ => matches!(
            surface_voxel,
            Voxel::Soil_Grass
                | Voxel::Soil_Peat_Grass
                | Voxel::Soil_Silt_Grass
                | Voxel::Soil_Mulch
                | Voxel::Soil_Peat_Mulch
                | Voxel::Soil_Silt_Mulch
                | Voxel::Soil_Moss
                | Voxel::Soil_Dirt
                | Voxel::Soil_Rooted_Dirt
                | Voxel::Soil_Packed_Dirt
                | Voxel::Soil_Mud
        ),
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
        Voxel::Tree_Acacia_Log
            | Voxel::Tree_Birch_Log
            | Voxel::Tree_Cherry_Log
            | Voxel::Tree_Mahogany_Log
            | Voxel::Tree_Mangrove_Log
            | Voxel::Tree_Maple_Log
            | Voxel::Tree_Oak_Log
            | Voxel::Tree_Palm_Log
            | Voxel::Tree_Pine_Log
            | Voxel::Tree_Willow_Log
            | Voxel::Tree_Yew_Log
            | Voxel::Tree_Charred_Log
            | Voxel::Tree_Dead_Log
            | Voxel::Tree_Cactus
            | Voxel::Tree_Mangrove_Roots
    )
}

/// Identifies leaf voxels.
#[inline]
pub fn is_leaves(voxel: Voxel) -> bool {
    voxel.is_leaves()
}

/// Procedurally generates the voxel blocks for a tree or cactus.
///
/// All standard trees share the simple, iconic shape of a Minecraft oak tree,
/// but taller (trunk height 6-8 blocks, leaving 3-5 blocks of clear walking space beneath the canopy).
/// Cacti are generated as simple vertical columns 2-4 blocks tall.
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
        // Minecraft-authentic cactus: vertical pillar 2 to 4 blocks tall
        let height = 2 + ((hash >> 12) % 3) as i32;
        for dy in 1..=height {
            emit(tx, ty + dy, tz, Voxel::Tree_Cactus);
        }
        return;
    }

    if species == TreeSpecies::Dead || species == TreeSpecies::Charred {
        // Lifeless/dead trees: bare trunk with short branch stubs, no leaves
        let height = 4 + ((hash >> 12) % 3) as i32;
        let (trunk_voxel, wood_voxel) = if biome == BiomeType::DeadwoodThicket {
            if (hash & 1) == 0 {
                (Voxel::Tree_Dead_Log, Voxel::Tree_Dead_Bark)
            } else {
                (Voxel::Tree_Charred_Log, Voxel::Tree_Charred_Bark)
            }
        } else {
            (species.log_voxel(), species.wood_voxel())
        };

        for dy in 1..=height {
            emit(tx, ty + dy, tz, trunk_voxel);
        }

        // Branch stubs
        let branch_dir = ((hash >> 15) % 4) as usize;
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (bx1, bz1) = dirs[branch_dir];
        emit(tx + bx1, ty + height - 1, tz + bz1, wood_voxel);
        let (bx2, bz2) = dirs[(branch_dir + 2) % 4];
        emit(tx + bx2, ty + height, tz + bz2, wood_voxel);
        return;
    }

    // Standard Tree: Minecraft oak shape, but taller (trunk height 6 to 8 blocks)
    let height = 6 + ((hash >> 12) % 3) as i32;
    let log_voxel = species.log_voxel();
    let leaves_voxel = leaves_voxel_variant(species, hash, biome);

    // Trunk column
    for dy in 1..=height {
        emit(tx, ty + dy, tz, log_voxel);
    }

    // Mangrove roots at the base
    if species == TreeSpecies::Mangrove {
        for &(rdx, rdz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            emit(tx + rdx, ty + 1, tz + rdz, Voxel::Tree_Mangrove_Roots);
        }
    }

    // Leaves canopy
    if let Some(leaf_voxel) = leaves_voxel {
        // Layers H - 2, H - 1, and H
        for dy in (height - 2)..=height {
            let is_top_layer = dy == height;
            let radius: i32 = if is_top_layer { 1 } else { 2 };

            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    // Skip center trunk
                    if dx == 0 && dz == 0 && dy <= height {
                        continue;
                    }

                    // Cut off 4 outer corners for 5x5 layers (radius 2)
                    if radius == 2 && dx.abs() == 2 && dz.abs() == 2 {
                        // On layer H-1, keep some corners with ~25% chance for subtle natural variation
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

        // Cap layer at H + 1: plus-shaped dome
        for &(dx, dz) in &[(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            emit(tx + dx, ty + height + 1, tz + dz, leaf_voxel);
        }
    }
}
