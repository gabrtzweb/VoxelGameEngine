use bevy::prelude::*;

use crate::{core::noise::fbm_2d, world::Voxel};

/// Complete 48-biome catalog defined in `docs/world_definition.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum BiomeType {
    // 1. Forests & Woodlands (8 in total)
    AncientWeald,
    AutumnalForest,
    BirchCopse,
    BlossomGrove,
    BorealTaiga,
    DeadwoodThicket,
    TropicalRainforest,
    YewGrove,

    // 2. Plains & Open Lands (8 in total)
    AcaciaSavanna,
    Heath,
    Moorland,
    OutbackScrubland,
    PermafrostSteppe,
    SnowyTundra,
    #[default]
    Steppe,
    VolcanicPlains,

    // 3. Wetlands & Swamps (8 in total)
    CypressSwamp,
    FungalBog,
    MangroveSwamp,
    Marshland,
    PeatBog,
    SludgeWastes,
    TarPits,
    WeepingBayou,

    // 4. Arid & Warm Lands (8 in total)
    Badlands,
    DuneDesert,
    Oasis,
    PaintedDesert,
    RockyScrubland,
    ScorchedWastes,
    WhiteDesert,
    WindsweptCanyons,

    // 5. Mountain Biomes (8 in total)
    AlpineTundra,
    FrozenCaldera,
    GlacialPeaks,
    JaggedCrags,
    KarstPeaks,
    ScreeSlopes,
    ShaleBarrens,
    VolcanicFields,

    // 6. Coastal & Aquatic Biomes (8 in total)
    AbyssalTrench,
    Beach,
    BrackishEstuary,
    ChalkCliffs,
    CoastalCrags,
    DeepOcean,
    TemperateOcean,
    TidalMudflats,
}

impl BiomeType {
    #[allow(dead_code)]
    pub const ALL: [BiomeType; 48] = [
        BiomeType::AncientWeald,
        BiomeType::AutumnalForest,
        BiomeType::BirchCopse,
        BiomeType::BlossomGrove,
        BiomeType::BorealTaiga,
        BiomeType::DeadwoodThicket,
        BiomeType::TropicalRainforest,
        BiomeType::YewGrove,
        BiomeType::AcaciaSavanna,
        BiomeType::Heath,
        BiomeType::Moorland,
        BiomeType::OutbackScrubland,
        BiomeType::PermafrostSteppe,
        BiomeType::SnowyTundra,
        BiomeType::Steppe,
        BiomeType::VolcanicPlains,
        BiomeType::CypressSwamp,
        BiomeType::FungalBog,
        BiomeType::MangroveSwamp,
        BiomeType::Marshland,
        BiomeType::PeatBog,
        BiomeType::SludgeWastes,
        BiomeType::TarPits,
        BiomeType::WeepingBayou,
        BiomeType::Badlands,
        BiomeType::DuneDesert,
        BiomeType::Oasis,
        BiomeType::PaintedDesert,
        BiomeType::RockyScrubland,
        BiomeType::ScorchedWastes,
        BiomeType::WhiteDesert,
        BiomeType::WindsweptCanyons,
        BiomeType::AlpineTundra,
        BiomeType::FrozenCaldera,
        BiomeType::GlacialPeaks,
        BiomeType::JaggedCrags,
        BiomeType::KarstPeaks,
        BiomeType::ScreeSlopes,
        BiomeType::ShaleBarrens,
        BiomeType::VolcanicFields,
        BiomeType::AbyssalTrench,
        BiomeType::Beach,
        BiomeType::BrackishEstuary,
        BiomeType::ChalkCliffs,
        BiomeType::CoastalCrags,
        BiomeType::DeepOcean,
        BiomeType::TemperateOcean,
        BiomeType::TidalMudflats,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::AncientWeald => "Ancient Weald",
            Self::AutumnalForest => "Autumnal Forest",
            Self::BirchCopse => "Birch Copse",
            Self::BlossomGrove => "Blossom Grove",
            Self::BorealTaiga => "Boreal Taiga",
            Self::DeadwoodThicket => "Deadwood Thicket",
            Self::TropicalRainforest => "Tropical Rainforest",
            Self::YewGrove => "Yew Grove",
            Self::AcaciaSavanna => "Acacia Savanna",
            Self::Heath => "Heath",
            Self::Moorland => "Moorland",
            Self::OutbackScrubland => "Outback Scrubland",
            Self::PermafrostSteppe => "Permafrost Steppe",
            Self::SnowyTundra => "Snowy Tundra",
            Self::Steppe => "Steppe",
            Self::VolcanicPlains => "Volcanic Plains",
            Self::CypressSwamp => "Cypress Swamp",
            Self::FungalBog => "Fungal Bog",
            Self::MangroveSwamp => "Mangrove Swamp",
            Self::Marshland => "Marshland",
            Self::PeatBog => "Peat Bog",
            Self::SludgeWastes => "Sludge Wastes",
            Self::TarPits => "Tar Pits",
            Self::WeepingBayou => "Weeping Bayou",
            Self::Badlands => "Badlands",
            Self::DuneDesert => "Dune Desert",
            Self::Oasis => "Oasis",
            Self::PaintedDesert => "Painted Desert",
            Self::RockyScrubland => "Rocky Scrubland",
            Self::ScorchedWastes => "Scorched Wastes",
            Self::WhiteDesert => "White Desert",
            Self::WindsweptCanyons => "Windswept Canyons",
            Self::AlpineTundra => "Alpine Tundra",
            Self::FrozenCaldera => "Frozen Caldera",
            Self::GlacialPeaks => "Glacial Peaks",
            Self::JaggedCrags => "Jagged Crags",
            Self::KarstPeaks => "Karst Peaks",
            Self::ScreeSlopes => "Scree Slopes",
            Self::ShaleBarrens => "Shale Barrens",
            Self::VolcanicFields => "Volcanic Fields",
            Self::AbyssalTrench => "Abyssal Trench",
            Self::Beach => "Beach",
            Self::BrackishEstuary => "Brackish Estuary",
            Self::ChalkCliffs => "Chalk Cliffs",
            Self::CoastalCrags => "Coastal Crags",
            Self::DeepOcean => "Deep Ocean",
            Self::TemperateOcean => "Temperate Ocean",
            Self::TidalMudflats => "Tidal Mudflats",
        }
    }

    pub fn config(self) -> BiomeConfig {
        match self {
            // --- Forests & Woodlands (8) ---
            Self::AncientWeald => BiomeConfig {
                biome_type: self,
                name: "Ancient Weald",
                surface_material: Voxel::Soil_Mulch,
                subsoil_material: Voxel::Soil_Dirt,
                subsoil_depth: 3,
                base_height_offset: 4.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Rock_Stone,
                cliff_material: Voxel::Mossy_Stone,
            },
            Self::AutumnalForest => BiomeConfig {
                biome_type: self,
                name: "Autumnal Forest",
                surface_material: Voxel::Soil_Silt_Grass,
                subsoil_material: Voxel::Soil_Silt,
                subsoil_depth: 3,
                base_height_offset: 3.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Rock_Stone,
                cliff_material: Voxel::Rock_Stone,
            },
            Self::BirchCopse => BiomeConfig {
                biome_type: self,
                name: "Birch Copse",
                surface_material: Voxel::Soil_Grass,
                subsoil_material: Voxel::Soil_Dirt,
                subsoil_depth: 3,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Rock_Chalk,
                cliff_material: Voxel::Rock_Chalk,
            },
            Self::BlossomGrove => BiomeConfig {
                biome_type: self,
                name: "Blossom Grove",
                surface_material: Voxel::Soil_Grass,
                subsoil_material: Voxel::Soil_Dirt,
                subsoil_depth: 3,
                base_height_offset: 5.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::Rock_Calcite,
                cliff_material: Voxel::Rock_Calcite,
            },
            Self::BorealTaiga => BiomeConfig {
                biome_type: self,
                name: "Boreal Taiga",
                surface_material: Voxel::Soil_Snowy_Peat,
                subsoil_material: Voxel::Soil_Peat,
                subsoil_depth: 3,
                base_height_offset: 6.0,
                amplitude_multiplier: 1.4,
                primary_stone: Voxel::Rock_Stone,
                cliff_material: Voxel::Rock_Stone,
            },
            Self::DeadwoodThicket => BiomeConfig {
                biome_type: self,
                name: "Deadwood Thicket",
                surface_material: Voxel::Soil_Ash,
                subsoil_material: Voxel::Soil_Black_Sand,
                subsoil_depth: 4,
                base_height_offset: 1.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::Rock_Pitchstone,
                cliff_material: Voxel::Rock_Pitchstone,
            },
            Self::TropicalRainforest => BiomeConfig {
                biome_type: self,
                name: "Tropical Rainforest",
                surface_material: Voxel::Soil_Mud,
                subsoil_material: Voxel::Soil_Packed_Mud,
                subsoil_depth: 4,
                base_height_offset: 4.0,
                amplitude_multiplier: 1.5,
                primary_stone: Voxel::Rock_Serpentine,
                cliff_material: Voxel::Rock_Serpentine,
            },
            Self::YewGrove => BiomeConfig {
                biome_type: self,
                name: "Yew Grove",
                surface_material: Voxel::Soil_Peat_Mulch,
                subsoil_material: Voxel::Soil_Peat,
                subsoil_depth: 3,
                base_height_offset: 5.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Rock_Gabbro,
                cliff_material: Voxel::Rock_Gabbro,
            },

            // --- Plains & Open Lands (8) ---
            Self::AcaciaSavanna => BiomeConfig {
                biome_type: self,
                name: "Acacia Savanna",
                surface_material: Voxel::Soil_Grass,
                subsoil_material: Voxel::Soil_Packed_Dirt,
                subsoil_depth: 3,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::Rock_Stone,
                cliff_material: Voxel::Rock_Stone,
            },
            Self::Heath => BiomeConfig {
                biome_type: self,
                name: "Heath",
                surface_material: Voxel::Soil_Silt_Grass,
                subsoil_material: Voxel::Soil_Silt,
                subsoil_depth: 3,
                base_height_offset: 3.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::Rock_Tuffite,
                cliff_material: Voxel::Rock_Tuffite,
            },
            Self::Moorland => BiomeConfig {
                biome_type: self,
                name: "Moorland",
                surface_material: Voxel::Soil_Peat_Grass,
                subsoil_material: Voxel::Soil_Peat,
                subsoil_depth: 4,
                base_height_offset: 8.0,
                amplitude_multiplier: 1.3,
                primary_stone: Voxel::Rock_Gabbro,
                cliff_material: Voxel::Rock_Gabbro,
            },
            Self::OutbackScrubland => BiomeConfig {
                biome_type: self,
                name: "Outback Scrubland",
                surface_material: Voxel::Soil_Scorched_Red_Sand,
                subsoil_material: Voxel::Soil_Packed_Mud,
                subsoil_depth: 3,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.7,
                primary_stone: Voxel::Rock_Red_Sandstone,
                cliff_material: Voxel::Rock_Red_Sandstone,
            },
            Self::PermafrostSteppe => BiomeConfig {
                biome_type: self,
                name: "Permafrost Steppe",
                surface_material: Voxel::Soil_Snowy_Silt,
                subsoil_material: Voxel::Soil_Packed_Silt,
                subsoil_depth: 4,
                base_height_offset: 1.5,
                amplitude_multiplier: 0.7,
                primary_stone: Voxel::Rock_Slate,
                cliff_material: Voxel::Rock_Slate,
            },
            Self::SnowyTundra => BiomeConfig {
                biome_type: self,
                name: "Snowy Tundra",
                surface_material: Voxel::Soil_Snowy_Grass,
                subsoil_material: Voxel::Soil_Dirt,
                subsoil_depth: 3,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::Rock_Limestone,
                cliff_material: Voxel::Rock_Limestone,
            },
            Self::Steppe => BiomeConfig {
                biome_type: self,
                name: "Steppe",
                surface_material: Voxel::Soil_Grass,
                subsoil_material: Voxel::Soil_Dirt,
                subsoil_depth: 3,
                base_height_offset: 1.0,
                amplitude_multiplier: 0.85,
                primary_stone: Voxel::Rock_Sandstone,
                cliff_material: Voxel::Rock_Sandstone,
            },
            Self::VolcanicPlains => BiomeConfig {
                biome_type: self,
                name: "Volcanic Plains",
                surface_material: Voxel::Soil_Ash,
                subsoil_material: Voxel::Soil_Scorched_Black_Sand,
                subsoil_depth: 4,
                base_height_offset: 3.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::Rock_Basalt,
                cliff_material: Voxel::Rock_Basalt,
            },

            // --- Wetlands & Swamps (8) ---
            Self::CypressSwamp => BiomeConfig {
                biome_type: self,
                name: "Cypress Swamp",
                surface_material: Voxel::Soil_Mud,
                subsoil_material: Voxel::Soil_Packed_Mud,
                subsoil_depth: 4,
                base_height_offset: -2.0,
                amplitude_multiplier: 0.4,
                primary_stone: Voxel::Rock_Stone,
                cliff_material: Voxel::Soil_Mud,
            },
            Self::FungalBog => BiomeConfig {
                biome_type: self,
                name: "Fungal Bog",
                surface_material: Voxel::Soil_Red_Moss,
                subsoil_material: Voxel::Soil_Packed_Mud,
                subsoil_depth: 4,
                base_height_offset: -2.5,
                amplitude_multiplier: 0.4,
                primary_stone: Voxel::Rock_Serpentine,
                cliff_material: Voxel::Soil_Red_Moss,
            },
            Self::MangroveSwamp => BiomeConfig {
                biome_type: self,
                name: "Mangrove Swamp",
                surface_material: Voxel::Soil_Mud,
                subsoil_material: Voxel::Soil_Packed_Mud,
                subsoil_depth: 4,
                base_height_offset: -1.5,
                amplitude_multiplier: 0.35,
                primary_stone: Voxel::Rock_Stone,
                cliff_material: Voxel::Soil_Mud,
            },
            Self::Marshland => BiomeConfig {
                biome_type: self,
                name: "Marshland",
                surface_material: Voxel::Soil_Mud,
                subsoil_material: Voxel::Soil_Packed_Mud,
                subsoil_depth: 4,
                base_height_offset: -1.0,
                amplitude_multiplier: 0.4,
                primary_stone: Voxel::Rock_Serpentine,
                cliff_material: Voxel::Soil_Packed_Mud,
            },
            Self::PeatBog => BiomeConfig {
                biome_type: self,
                name: "Peat Bog",
                surface_material: Voxel::Soil_Peat_Grass,
                subsoil_material: Voxel::Soil_Peat,
                subsoil_depth: 5,
                base_height_offset: -1.0,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Rock_Marl,
                cliff_material: Voxel::Rock_Marl,
            },
            Self::SludgeWastes => BiomeConfig {
                biome_type: self,
                name: "Sludge Wastes",
                surface_material: Voxel::Soil_Scorched_Sand,
                subsoil_material: Voxel::Soil_Mud,
                subsoil_depth: 4,
                base_height_offset: -2.0,
                amplitude_multiplier: 0.45,
                primary_stone: Voxel::Rock_Pitchstone,
                cliff_material: Voxel::Rock_Pitchstone,
            },
            Self::TarPits => BiomeConfig {
                biome_type: self,
                name: "Tar Pits",
                surface_material: Voxel::Soil_Black_Sand,
                subsoil_material: Voxel::Soil_Black_Sand,
                subsoil_depth: 5,
                base_height_offset: -2.0,
                amplitude_multiplier: 0.4,
                primary_stone: Voxel::Rock_Pitchstone,
                cliff_material: Voxel::Rock_Pitchstone,
            },
            Self::WeepingBayou => BiomeConfig {
                biome_type: self,
                name: "Weeping Bayou",
                surface_material: Voxel::Soil_Silt_Mulch,
                subsoil_material: Voxel::Soil_Mud,
                subsoil_depth: 4,
                base_height_offset: -1.5,
                amplitude_multiplier: 0.45,
                primary_stone: Voxel::Rock_Slate,
                cliff_material: Voxel::Soil_Mud,
            },

            // --- Arid & Warm Lands (8) ---
            Self::Badlands => BiomeConfig {
                biome_type: self,
                name: "Badlands",
                surface_material: Voxel::Soil_Red_Sand,
                subsoil_material: Voxel::Rock_Red_Sandstone,
                subsoil_depth: 4,
                base_height_offset: 12.0,
                amplitude_multiplier: 2.2,
                primary_stone: Voxel::Rock_Red_Sandstone,
                cliff_material: Voxel::Rock_Terracotta,
            },
            Self::DuneDesert => BiomeConfig {
                biome_type: self,
                name: "Dune Desert",
                surface_material: Voxel::Soil_Sand,
                subsoil_material: Voxel::Soil_Sand,
                subsoil_depth: 6,
                base_height_offset: 4.0,
                amplitude_multiplier: 1.8,
                primary_stone: Voxel::Rock_Sandstone,
                cliff_material: Voxel::Rock_Sandstone,
            },
            Self::Oasis => BiomeConfig {
                biome_type: self,
                name: "Oasis",
                surface_material: Voxel::Soil_Grass,
                subsoil_material: Voxel::Soil_Dirt,
                subsoil_depth: 3,
                base_height_offset: 0.5,
                amplitude_multiplier: 0.6,
                primary_stone: Voxel::Rock_Sandstone,
                cliff_material: Voxel::Soil_Grass,
            },
            Self::PaintedDesert => BiomeConfig {
                biome_type: self,
                name: "Painted Desert",
                surface_material: Voxel::Soil_White_Sand,
                subsoil_material: Voxel::Soil_Sand,
                subsoil_depth: 4,
                base_height_offset: 6.0,
                amplitude_multiplier: 1.5,
                primary_stone: Voxel::Rock_Cinnabar,
                cliff_material: Voxel::Rock_Cinnabar,
            },
            Self::RockyScrubland => BiomeConfig {
                biome_type: self,
                name: "Rocky Scrubland",
                surface_material: Voxel::Soil_Packed_Dirt,
                subsoil_material: Voxel::Soil_Sand,
                subsoil_depth: 3,
                base_height_offset: 5.0,
                amplitude_multiplier: 1.3,
                primary_stone: Voxel::Rock_Granite,
                cliff_material: Voxel::Rock_Granite,
            },
            Self::ScorchedWastes => BiomeConfig {
                biome_type: self,
                name: "Scorched Wastes",
                surface_material: Voxel::Soil_Scorched_Sand,
                subsoil_material: Voxel::Soil_Scorched_Sand,
                subsoil_depth: 4,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Rock_Scoria,
                cliff_material: Voxel::Rock_Scoria,
            },
            Self::WhiteDesert => BiomeConfig {
                biome_type: self,
                name: "White Desert",
                surface_material: Voxel::Soil_White_Sand,
                subsoil_material: Voxel::Soil_White_Sand,
                subsoil_depth: 5,
                base_height_offset: 3.0,
                amplitude_multiplier: 1.4,
                primary_stone: Voxel::Rock_White_Sandstone,
                cliff_material: Voxel::Rock_White_Sandstone,
            },
            Self::WindsweptCanyons => BiomeConfig {
                biome_type: self,
                name: "Windswept Canyons",
                surface_material: Voxel::Soil_Sand,
                subsoil_material: Voxel::Soil_Sand,
                subsoil_depth: 3,
                base_height_offset: 16.0,
                amplitude_multiplier: 2.5,
                primary_stone: Voxel::Rock_Chert,
                cliff_material: Voxel::Rock_Porphyry,
            },

            // --- Mountain Biomes (8) ---
            Self::AlpineTundra => BiomeConfig {
                biome_type: self,
                name: "Alpine Tundra",
                surface_material: Voxel::Soil_Snowy_Grass,
                subsoil_material: Voxel::Rock_Diorite,
                subsoil_depth: 2,
                base_height_offset: 30.0,
                amplitude_multiplier: 2.6,
                primary_stone: Voxel::Rock_Diorite,
                cliff_material: Voxel::Rock_Diorite,
            },
            Self::FrozenCaldera => BiomeConfig {
                biome_type: self,
                name: "Frozen Caldera",
                surface_material: Voxel::Frost_Black_Ice,
                subsoil_material: Voxel::Rock_Obsidian,
                subsoil_depth: 3,
                base_height_offset: 38.0,
                amplitude_multiplier: 2.8,
                primary_stone: Voxel::Rock_Obsidian,
                cliff_material: Voxel::Rock_Basalt,
            },
            Self::GlacialPeaks => BiomeConfig {
                biome_type: self,
                name: "Glacial Peaks",
                surface_material: Voxel::Soil_Snow,
                subsoil_material: Voxel::Frost_Packed_Ice,
                subsoil_depth: 6,
                base_height_offset: 45.0,
                amplitude_multiplier: 3.5,
                primary_stone: Voxel::Rock_Cryolite,
                cliff_material: Voxel::Frost_Packed_Ice,
            },
            Self::JaggedCrags => BiomeConfig {
                biome_type: self,
                name: "Jagged Crags",
                surface_material: Voxel::Rock_Gabbro,
                subsoil_material: Voxel::Rock_Andesite,
                subsoil_depth: 2,
                base_height_offset: 36.0,
                amplitude_multiplier: 3.2,
                primary_stone: Voxel::Rock_Gabbro,
                cliff_material: Voxel::Rock_Gabbro,
            },
            Self::KarstPeaks => BiomeConfig {
                biome_type: self,
                name: "Karst Peaks",
                surface_material: Voxel::Rock_Karst,
                subsoil_material: Voxel::Rock_Karst,
                subsoil_depth: 2,
                base_height_offset: 34.0,
                amplitude_multiplier: 3.6,
                primary_stone: Voxel::Rock_Karst,
                cliff_material: Voxel::Rock_Karst,
            },
            Self::ScreeSlopes => BiomeConfig {
                biome_type: self,
                name: "Scree Slopes",
                surface_material: Voxel::Soil_Gravel,
                subsoil_material: Voxel::Soil_Gravel,
                subsoil_depth: 4,
                base_height_offset: 28.0,
                amplitude_multiplier: 2.8,
                primary_stone: Voxel::Rock_Andesite,
                cliff_material: Voxel::Rock_Andesite,
            },
            Self::ShaleBarrens => BiomeConfig {
                biome_type: self,
                name: "Shale Barrens",
                surface_material: Voxel::Rock_Slate,
                subsoil_material: Voxel::Cobbled_Slate,
                subsoil_depth: 3,
                base_height_offset: 26.0,
                amplitude_multiplier: 2.5,
                primary_stone: Voxel::Rock_Slate,
                cliff_material: Voxel::Rock_Slate,
            },
            Self::VolcanicFields => BiomeConfig {
                biome_type: self,
                name: "Volcanic Fields",
                surface_material: Voxel::Rock_Basalt,
                subsoil_material: Voxel::Rock_Scoria,
                subsoil_depth: 3,
                base_height_offset: 32.0,
                amplitude_multiplier: 2.7,
                primary_stone: Voxel::Rock_Magma,
                cliff_material: Voxel::Rock_Basalt,
            },

            // --- Coastal & Aquatic Biomes (8) ---
            Self::AbyssalTrench => BiomeConfig {
                biome_type: self,
                name: "Abyssal Trench",
                surface_material: Voxel::Rock_Obsidian,
                subsoil_material: Voxel::Rock_Pitchstone,
                subsoil_depth: 3,
                base_height_offset: -38.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::Rock_Obsidian,
                cliff_material: Voxel::Rock_Obsidian,
            },
            Self::Beach => BiomeConfig {
                biome_type: self,
                name: "Beach",
                surface_material: Voxel::Soil_Sand,
                subsoil_material: Voxel::Soil_Sand,
                subsoil_depth: 4,
                base_height_offset: -0.5,
                amplitude_multiplier: 0.3,
                primary_stone: Voxel::Rock_Sandstone,
                cliff_material: Voxel::Rock_Sandstone,
            },
            Self::BrackishEstuary => BiomeConfig {
                biome_type: self,
                name: "Brackish Estuary",
                surface_material: Voxel::Soil_Silt,
                subsoil_material: Voxel::Soil_Mud,
                subsoil_depth: 4,
                base_height_offset: -3.0,
                amplitude_multiplier: 0.35,
                primary_stone: Voxel::Rock_Marl,
                cliff_material: Voxel::Soil_Mud,
            },
            Self::ChalkCliffs => BiomeConfig {
                biome_type: self,
                name: "Chalk Cliffs",
                surface_material: Voxel::Soil_Grass,
                subsoil_material: Voxel::Soil_Dirt,
                subsoil_depth: 2,
                base_height_offset: 18.0,
                amplitude_multiplier: 2.2,
                primary_stone: Voxel::Rock_Chalk,
                cliff_material: Voxel::Rock_Chalk,
            },
            Self::CoastalCrags => BiomeConfig {
                biome_type: self,
                name: "Coastal Crags",
                surface_material: Voxel::Soil_Grass,
                subsoil_material: Voxel::Soil_Gravel,
                subsoil_depth: 2,
                base_height_offset: 12.0,
                amplitude_multiplier: 2.4,
                primary_stone: Voxel::Rock_Porphyry,
                cliff_material: Voxel::Rock_Porphyry,
            },
            Self::DeepOcean => BiomeConfig {
                biome_type: self,
                name: "Deep Ocean",
                surface_material: Voxel::Soil_Gravel,
                subsoil_material: Voxel::Rock_Andesite,
                subsoil_depth: 3,
                base_height_offset: -24.0,
                amplitude_multiplier: 0.6,
                primary_stone: Voxel::Rock_Andesite,
                cliff_material: Voxel::Rock_Andesite,
            },
            Self::TemperateOcean => BiomeConfig {
                biome_type: self,
                name: "Temperate Ocean",
                surface_material: Voxel::Soil_White_Sand,
                subsoil_material: Voxel::Soil_White_Sand,
                subsoil_depth: 4,
                base_height_offset: -12.0,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Rock_Limestone,
                cliff_material: Voxel::Rock_Limestone,
            },
            Self::TidalMudflats => BiomeConfig {
                biome_type: self,
                name: "Tidal Mudflats",
                surface_material: Voxel::Soil_Packed_Mud,
                subsoil_material: Voxel::Soil_Clay,
                subsoil_depth: 4,
                base_height_offset: -0.8,
                amplitude_multiplier: 0.25,
                primary_stone: Voxel::Rock_Marl,
                cliff_material: Voxel::Soil_Packed_Mud,
            },
        }
    }

    /// Calibrated grass tint color per biome.
    pub fn grass_color(self) -> [f32; 4] {
        match self {
            // Forests & Woodlands
            Self::AncientWeald => [0.48, 0.95, 0.38, 1.0],
            Self::AutumnalForest => [0.72, 0.85, 0.32, 1.0],
            Self::BirchCopse => [0.60, 1.02, 0.42, 1.0],
            Self::BlossomGrove => [0.55, 0.98, 0.45, 1.0],
            Self::BorealTaiga => [0.44, 0.78, 0.48, 1.0],
            Self::DeadwoodThicket => [0.45, 0.48, 0.42, 1.0],
            Self::TropicalRainforest => [0.32, 0.92, 0.28, 1.0],
            Self::YewGrove => [0.36, 0.65, 0.34, 1.0],

            // Plains & Open Lands
            Self::AcaciaSavanna => [0.70, 0.82, 0.32, 1.0],
            Self::Heath => [0.52, 0.78, 0.46, 1.0],
            Self::Moorland => [0.44, 0.68, 0.38, 1.0],
            Self::OutbackScrubland => [0.78, 0.68, 0.30, 1.0],
            Self::PermafrostSteppe => [0.50, 0.75, 0.65, 1.0],
            Self::SnowyTundra => [0.52, 0.82, 0.72, 1.0],
            Self::Steppe => [0.56, 0.92, 0.40, 1.0],
            Self::VolcanicPlains => [0.48, 0.52, 0.44, 1.0],

            // Wetlands & Swamps
            Self::CypressSwamp => [0.42, 0.72, 0.35, 1.0],
            Self::FungalBog => [0.85, 0.42, 0.55, 1.0],
            Self::MangroveSwamp => [0.38, 0.82, 0.32, 1.0],
            Self::Marshland => [0.45, 0.75, 0.36, 1.0],
            Self::PeatBog => [0.48, 0.70, 0.32, 1.0],
            Self::SludgeWastes => [0.60, 0.62, 0.28, 1.0],
            Self::TarPits => [0.35, 0.35, 0.35, 1.0],
            Self::WeepingBayou => [0.48, 0.85, 0.42, 1.0],

            // Arid & Warm Lands
            Self::Badlands => [0.82, 0.68, 0.32, 1.0],
            Self::DuneDesert => [0.75, 0.78, 0.35, 1.0],
            Self::Oasis => [0.45, 0.95, 0.35, 1.0],
            Self::PaintedDesert => [0.80, 0.72, 0.40, 1.0],
            Self::RockyScrubland => [0.68, 0.75, 0.36, 1.0],
            Self::ScorchedWastes => [0.72, 0.58, 0.30, 1.0],
            Self::WhiteDesert => [0.82, 0.85, 0.65, 1.0],
            Self::WindsweptCanyons => [0.74, 0.75, 0.35, 1.0],

            // Mountain Biomes
            Self::AlpineTundra => [0.48, 0.84, 0.55, 1.0],
            Self::FrozenCaldera => [0.35, 0.60, 0.75, 1.0],
            Self::GlacialPeaks => [0.65, 0.88, 1.00, 1.0],
            Self::JaggedCrags => [0.40, 0.55, 0.45, 1.0],
            Self::KarstPeaks => [0.46, 0.82, 0.48, 1.0],
            Self::ScreeSlopes => [0.50, 0.65, 0.48, 1.0],
            Self::ShaleBarrens => [0.42, 0.58, 0.55, 1.0],
            Self::VolcanicFields => [0.45, 0.48, 0.35, 1.0],

            // Coastal & Aquatic Biomes
            Self::AbyssalTrench => [0.20, 0.35, 0.50, 1.0],
            Self::Beach => [0.62, 0.88, 0.44, 1.0],
            Self::BrackishEstuary => [0.46, 0.78, 0.38, 1.0],
            Self::ChalkCliffs => [0.55, 0.92, 0.42, 1.0],
            Self::CoastalCrags => [0.48, 0.72, 0.46, 1.0],
            Self::DeepOcean => [0.28, 0.55, 0.60, 1.0],
            Self::TemperateOcean => [0.35, 0.80, 0.68, 1.0],
            Self::TidalMudflats => [0.45, 0.65, 0.42, 1.0],
        }
    }

    /// Calibrated water tint color per biome.
    pub fn water_color(self) -> [f32; 4] {
        match self {
            Self::AbyssalTrench => [0.08, 0.14, 0.38, 1.0],
            Self::DeepOcean => [0.16, 0.36, 0.72, 1.0],
            Self::TemperateOcean => [0.20, 0.68, 0.88, 1.0],
            Self::Beach => [0.26, 0.76, 0.92, 1.0],
            Self::BrackishEstuary | Self::TidalMudflats => [0.28, 0.58, 0.58, 1.0],
            Self::CypressSwamp | Self::MangroveSwamp | Self::Marshland => [0.28, 0.52, 0.42, 1.0],
            Self::FungalBog => [0.35, 0.80, 0.30, 1.0],
            Self::TarPits => [0.10, 0.10, 0.12, 1.0],
            Self::SludgeWastes => [0.28, 0.32, 0.25, 1.0],
            Self::VolcanicFields | Self::VolcanicPlains => [0.55, 0.38, 0.25, 1.0],
            Self::GlacialPeaks | Self::FrozenCaldera => [0.45, 0.78, 0.98, 1.0],
            Self::Oasis => [0.18, 0.82, 0.92, 1.0],
            Self::AncientWeald | Self::BlossomGrove | Self::BirchCopse => [0.30, 0.68, 0.95, 1.0],
            _ => [0.34, 0.65, 0.92, 1.0],
        }
    }

    /// Calibrated foliage (leaves) tint color per biome.
    pub fn foliage_color(self, voxel: Voxel) -> [f32; 4] {
        match voxel {
            Voxel::Tree_Oak_Leaves => match self {
                Self::AncientWeald | Self::BlossomGrove | Self::BirchCopse => [0.55, 1.15, 0.35, 1.0],
                Self::TropicalRainforest => [0.40, 1.05, 0.30, 1.0],
                Self::BorealTaiga | Self::SnowyTundra | Self::PermafrostSteppe => [0.45, 0.90, 0.50, 1.0],
                Self::AutumnalForest => [0.85, 0.95, 0.28, 1.0],
                Self::AcaciaSavanna | Self::OutbackScrubland | Self::DuneDesert | Self::Badlands => [0.72, 0.95, 0.30, 1.0],
                Self::CypressSwamp | Self::Marshland | Self::MangroveSwamp | Self::PeatBog => [0.44, 0.80, 0.30, 1.0],
                Self::DeadwoodThicket => [0.35, 0.40, 0.32, 1.0],
                _ => [0.55, 1.10, 0.35, 1.0],
            },
            Voxel::Tree_Birch_Leaves => [0.85, 1.25, 0.40, 1.0],
            Voxel::Tree_Pine_Leaves => match self {
                Self::GlacialPeaks | Self::AlpineTundra | Self::SnowyTundra | Self::BorealTaiga | Self::PermafrostSteppe => {
                    [0.35, 0.82, 0.60, 1.0]
                }
                _ => [0.40, 0.90, 0.55, 1.0],
            },
            Voxel::Tree_Mahogany_Leaves => [0.45, 1.00, 0.40, 1.0],
            Voxel::Tree_Acacia_Leaves => [0.72, 0.92, 0.28, 1.0],
            Voxel::Tree_Mangrove_Leaves => [0.40, 0.95, 0.42, 1.0],
            Voxel::Tree_Palm_Leaves => [0.55, 1.05, 0.30, 1.0],
            Voxel::Tree_Willow_Leaves => [0.52, 0.88, 0.48, 1.0],
            Voxel::Tree_Yew_Leaves => [0.28, 0.70, 0.38, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Primary tint color for a voxel in this biome.
    pub fn voxel_tint(self, voxel: Voxel) -> [f32; 4] {
        match voxel {
            Voxel::Soil_Grass | Voxel::Soil_Peat_Grass | Voxel::Soil_Silt_Grass => self.grass_color(),
            Voxel::Liquid_Water | Voxel::WaterOccupied => self.water_color(),
            Voxel::Tree_Oak_Leaves
            | Voxel::Tree_Birch_Leaves
            | Voxel::Tree_Pine_Leaves
            | Voxel::Tree_Mahogany_Leaves
            | Voxel::Tree_Acacia_Leaves
            | Voxel::Tree_Mangrove_Leaves
            | Voxel::Tree_Palm_Leaves
            | Voxel::Tree_Willow_Leaves
            | Voxel::Tree_Yew_Leaves => {
                self.foliage_color(voxel)
            }
            _ => [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct BiomeConfig {
    pub biome_type: BiomeType,
    pub name: &'static str,
    pub surface_material: Voxel,
    pub subsoil_material: Voxel,
    pub subsoil_depth: i32,
    pub base_height_offset: f32,
    pub amplitude_multiplier: f32,
    pub primary_stone: Voxel,
    pub cliff_material: Voxel,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct ClimateSample {
    pub continentalness: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub biome: BiomeType,
}

/// Macro climate noise generator.
#[derive(Debug, Clone, Reflect)]
pub struct ClimateGenerator {
    pub continentalness_freq: f32,
    pub temperature_freq: f32,
    pub humidity_freq: f32,
    pub warp_amplitude: f32,
    pub dither_amplitude: f32,
}

impl Default for ClimateGenerator {
    fn default() -> Self {
        Self {
            continentalness_freq: 0.0012,
            temperature_freq: 0.0012,
            humidity_freq: 0.0020,
            warp_amplitude: 36.0,
            dither_amplitude: 7.0,
        }
    }
}

impl ClimateGenerator {
    pub fn sample(&self, world_x: f32, world_z: f32, seed: u32) -> ClimateSample {
        let continentalness = fbm_2d(
            world_x,
            world_z,
            self.continentalness_freq,
            3,
            0.55,
            2.0,
            seed.wrapping_add(10_007),
        );

        let temperature = fbm_2d(
            world_x,
            world_z,
            self.temperature_freq,
            2,
            0.5,
            2.0,
            seed.wrapping_add(30_011),
        );

        let humidity = fbm_2d(
            world_x,
            world_z,
            self.humidity_freq,
            2,
            0.5,
            2.0,
            seed.wrapping_add(50_021),
        );

        let biome = Self::classify_biome(continentalness, temperature, humidity);

        ClimateSample {
            continentalness,
            temperature,
            humidity,
            biome,
        }
    }

    /// Samples climate with multi-octave domain warping and high-frequency edge dithering
    pub fn sample_dithered(&self, world_x: f32, world_z: f32, seed: u32) -> ClimateSample {
        // Macro domain warping (breaks geometric contour lines into organic tendrils)
        let warp_x = fbm_2d(world_x, world_z, 0.004, 2, 0.5, 2.0, seed.wrapping_add(12_345))
            * self.warp_amplitude;
        let warp_z = fbm_2d(world_x, world_z, 0.004, 2, 0.5, 2.0, seed.wrapping_add(67_890))
            * self.warp_amplitude;

        // Micro boundary dithering noise (interlocking feathered edge specks)
        let dither_x = crate::core::noise::gradient_noise_2d(
            world_x * 0.16,
            world_z * 0.16,
            seed.wrapping_add(88_331),
        ) * self.dither_amplitude;
        let dither_z = crate::core::noise::gradient_noise_2d(
            world_x * 0.16 + 50.0,
            world_z * 0.16 + 50.0,
            seed.wrapping_add(99_442),
        ) * self.dither_amplitude;

        self.sample(world_x + warp_x + dither_x, world_z + warp_z + dither_z, seed)
    }

    /// Organically distributes all 48 biomes across Continentalness, Temperature, and Humidity.
    pub fn classify_biome(continentalness: f32, temperature: f32, humidity: f32) -> BiomeType {
        // =========================================================================
        // 1. Aquatic Biomes (Continentalness < -0.15)
        // =========================================================================
        if continentalness < -0.48 {
            return BiomeType::AbyssalTrench;
        } else if continentalness < -0.15 {
            return if temperature > 0.10 {
                BiomeType::TemperateOcean
            } else {
                BiomeType::DeepOcean
            };
        }

        // =========================================================================
        // 2. Coastal Biomes (-0.15 <= Continentalness < 0.05)
        // =========================================================================
        if continentalness < 0.05 {
            if humidity > 0.22 {
                return if temperature > 0.10 {
                    BiomeType::MangroveSwamp
                } else {
                    BiomeType::BrackishEstuary
                };
            } else if temperature < -0.20 {
                return BiomeType::TidalMudflats;
            } else if continentalness > -0.05 && humidity < -0.18 {
                return if temperature > 0.0 {
                    BiomeType::ChalkCliffs
                } else {
                    BiomeType::CoastalCrags
                };
            } else {
                return BiomeType::Beach;
            }
        }

        // =========================================================================
        // 3. Mountain Biomes (Continentalness >= 0.35)
        // =========================================================================
        if continentalness >= 0.35 {
            if temperature < -0.25 {
                return if humidity < 0.0 {
                    BiomeType::GlacialPeaks
                } else {
                    BiomeType::FrozenCaldera
                };
            } else if temperature < -0.05 {
                return if humidity < 0.0 {
                    BiomeType::AlpineTundra
                } else {
                    BiomeType::ScreeSlopes
                };
            } else if temperature < 0.25 {
                if humidity > 0.15 {
                    return BiomeType::KarstPeaks;
                } else if humidity < -0.10 {
                    return BiomeType::ShaleBarrens;
                } else {
                    return BiomeType::JaggedCrags;
                }
            } else {
                // Hot Mountainous
                return if humidity < -0.10 {
                    BiomeType::WindsweptCanyons
                } else {
                    BiomeType::VolcanicFields
                };
            }
        }

        // =========================================================================
        // 4. Inland Continental Biomes (0.05 <= Continentalness < 0.35)
        // =========================================================================

        // --- 4A. Frigid / Polar (Temperature < -0.25) ---
        if temperature < -0.25 {
            if humidity < -0.10 {
                return BiomeType::PermafrostSteppe;
            } else if humidity > 0.15 {
                return BiomeType::BorealTaiga;
            } else {
                return BiomeType::SnowyTundra;
            }
        }

        // --- 4B. Cold / Sub-polar (-0.25 <= Temperature < -0.05) ---
        if temperature < -0.05 {
            if humidity > 0.20 {
                return BiomeType::PeatBog;
            } else if humidity > 0.05 {
                return BiomeType::Heath;
            } else if humidity < -0.15 {
                return BiomeType::DeadwoodThicket;
            } else {
                return BiomeType::Moorland;
            }
        }

        // --- 4C. Hot / Warm Lands (Temperature >= 0.25) ---
        if temperature >= 0.25 {
            if humidity < -0.25 {
                return BiomeType::DuneDesert;
            } else if humidity < -0.12 {
                return if continentalness > 0.22 {
                    BiomeType::Badlands
                } else {
                    BiomeType::WhiteDesert
                };
            } else if humidity < 0.0 {
                return if continentalness > 0.20 {
                    BiomeType::PaintedDesert
                } else {
                    BiomeType::ScorchedWastes
                };
            } else if humidity < 0.15 {
                return if continentalness < 0.15 {
                    BiomeType::Oasis
                } else {
                    BiomeType::RockyScrubland
                };
            } else if humidity > 0.30 {
                return BiomeType::TropicalRainforest;
            } else if humidity > 0.20 {
                return BiomeType::SludgeWastes;
            } else {
                return BiomeType::AcaciaSavanna;
            }
        }

        // --- 4D. Temperate Lands (-0.05 <= Temperature < 0.25) ---
        if humidity > 0.30 {
            // Very wet / inundated
            if temperature > 0.10 {
                BiomeType::CypressSwamp
            } else if continentalness < 0.15 {
                BiomeType::WeepingBayou
            } else {
                BiomeType::FungalBog
            }
        } else if humidity > 0.18 {
            // Wet / swamps
            if temperature > 0.10 {
                BiomeType::Marshland
            } else {
                BiomeType::TarPits
            }
        } else if humidity > 0.06 {
            // Rich canopy forests
            if temperature > 0.12 {
                BiomeType::BlossomGrove
            } else if temperature < 0.05 {
                BiomeType::YewGrove
            } else {
                BiomeType::AncientWeald
            }
        } else if humidity > -0.06 {
            // Woodlands & seasonal forests
            if temperature > 0.10 {
                BiomeType::BirchCopse
            } else {
                BiomeType::AutumnalForest
            }
        } else if humidity > -0.20 {
            // Open grasslands & dry plains
            if continentalness > 0.20 {
                BiomeType::OutbackScrubland
            } else if temperature > 0.10 {
                BiomeType::VolcanicPlains
            } else {
                BiomeType::Steppe
            }
        } else {
            BiomeType::Steppe
        }
    }
}

/// Smoothly blends biome colors across biome transitions using a 13-point circular Gaussian kernel
/// (covering an organic 20-block transition zone) and quantizes the result into 64 discrete steps
/// per channel to preserve high greedy meshing merges while eliminating harsh stepped borders.
pub fn sample_blended_biome_color(voxel: Voxel, world_x: f32, world_z: f32, seed: u32) -> [f32; 4] {
    if !voxel.is_tinted() {
        return [1.0, 1.0, 1.0, 1.0];
    }

    let generator = ClimateGenerator::default();

    // 13-point symmetric circular Gaussian kernel:
    // Center point (d = 0.0), inner ring (8 points at radius 5.0 in cardinal and diagonal directions),
    // and outer ring (4 points at radius 10.0 in cardinal directions).
    const R_INNER: f32 = 5.0;
    const R_DIAG: f32 = 3.5355; // 5.0 / sqrt(2)
    const R_OUTER: f32 = 10.0;

    let offsets: [(f32, f32, f32); 13] = [
        // Center
        (0.0, 0.0, 0.152),
        // Inner ring cardinal (r = 5.0)
        (R_INNER, 0.0, 0.092),
        (-R_INNER, 0.0, 0.092),
        (0.0, R_INNER, 0.092),
        (0.0, -R_INNER, 0.092),
        // Inner ring diagonal (r = 5.0)
        (R_DIAG, R_DIAG, 0.092),
        (-R_DIAG, R_DIAG, 0.092),
        (R_DIAG, -R_DIAG, 0.092),
        (-R_DIAG, -R_DIAG, 0.092),
        // Outer ring cardinal (r = 10.0)
        (R_OUTER, 0.0, 0.028),
        (-R_OUTER, 0.0, 0.028),
        (0.0, R_OUTER, 0.028),
        (0.0, -R_OUTER, 0.028),
    ];

    let mut r = 0.0f32;
    let mut g = 0.0f32;
    let mut b = 0.0f32;
    let mut a = 0.0f32;

    for (dx, dz, weight) in offsets {
        let sample = generator.sample(world_x + dx, world_z + dz, seed);
        let color = sample.biome.voxel_tint(voxel);
        r += color[0] * weight;
        g += color[1] * weight;
        b += color[2] * weight;
        a += color[3] * weight;
    }

    // Quantize to 64 steps (1/64 = 0.015625) per channel: provides smooth color transitions
    // while maximizing greedy meshing quad merges across biome transition spans.
    const QUANTIZE_STEPS: f32 = 64.0;
    [
        (r * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
        (g * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
        (b * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
        (a * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_48_biomes_classified() {
        assert_eq!(BiomeType::ALL.len(), 48);

        let mut hit = std::collections::HashSet::new();

        // Sample across a dense 3D climate grid to ensure every biome can be generated
        for c_step in -10..=10 {
            let c = c_step as f32 / 10.0;
            for t_step in -10..=10 {
                let t = t_step as f32 / 10.0;
                for h_step in -10..=10 {
                    let h = h_step as f32 / 10.0;
                    hit.insert(ClimateGenerator::classify_biome(c, t, h));
                }
            }
        }

        for &biome in &BiomeType::ALL {
            assert!(
                hit.contains(&biome),
                "Biome {:?} ({}) was never generated by classify_biome!",
                biome,
                biome.name()
            );
        }
    }
}
