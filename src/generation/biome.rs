use bevy::prelude::*;

use crate::{
    core::noise::{Simd4f, fbm_2d, fbm_2d_x4, gradient_noise_2d_x4},
    world::Voxel,
};

/// 36 diverse biomes spanning all 6 climate dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum BiomeType {
    // Group A: Extreme Peaks & Mountain Ranges (Low Erosion, High Ridges)
    #[default]
    AlpineHighlands,
    EmeraldPeaks,
    VolcanicPeaks,
    PaintedMountains,
    RockyMountains,
    ScarletMountains,

    // Group B: Canyons & Dramatic Cliffs
    BryceCanyon,
    AmethystCanyon,
    BasaltCliffs,
    GraniteCliffs,
    WhiteCliffs,
    YosemiteCliffs,

    // Group C: Deserts, Savannas & Arid Badlands
    DesertSpires,
    AncientSands,
    DesertOasis,
    FracturedSavanna,
    SavannaBadlands,
    AshenSavanna,

    // Group D: Forests, Lush Valleys & Plains
    BloomingValley,
    CloudForest,
    LavenderValley,
    SakuraGrove,
    YosemiteLowlands,
    Shield,
    TropicalJungle,
    OrchidSwamp,

    // Group E: Glacial & Subarctic Tundra
    GlacialChasm,
    SiberianTaiga,
    FrozenCliffs,
    WintryForest,
    ColdShrubland,

    // Group F: Coastal, Aquatic & Subterranean Environments
    GravelBeach,
    MirageIsles,
    Caldera,
    DeepWarmOcean,
    ColdOcean,
    CoralReef,
    OceanTrench,
    ThermalCaves,
    UndergroundJungle,
    IceCaves,
    MushroomCaves,

    // Group G: Surface & Montane Biomes
    DuneSea,
    PineHighlands,
}

impl BiomeType {
    pub fn name(self) -> &'static str {
        match self {
            Self::AlpineHighlands => "Alpine Highlands",
            Self::EmeraldPeaks => "Emerald Peaks",
            Self::VolcanicPeaks => "Volcanic Peaks",
            Self::PaintedMountains => "Painted Mountains",
            Self::RockyMountains => "Rocky Mountains",
            Self::ScarletMountains => "Scarlet Mountains",
            Self::BryceCanyon => "Bryce Canyon",
            Self::AmethystCanyon => "Amethyst Canyon",
            Self::BasaltCliffs => "Basalt Cliffs",
            Self::GraniteCliffs => "Granite Cliffs",
            Self::WhiteCliffs => "White Cliffs",
            Self::YosemiteCliffs => "Yosemite Cliffs",
            Self::DesertSpires => "Desert Spires",
            Self::AncientSands => "Ancient Sands",
            Self::DesertOasis => "Desert Oasis",
            Self::FracturedSavanna => "Fractured Savanna",
            Self::SavannaBadlands => "Savanna Badlands",
            Self::AshenSavanna => "Ashen Savanna",
            Self::BloomingValley => "Blooming Valley",
            Self::CloudForest => "Cloud Forest",
            Self::LavenderValley => "Lavender Valley",
            Self::SakuraGrove => "Sakura Grove",
            Self::YosemiteLowlands => "Yosemite Lowlands",
            Self::Shield => "Shield Taiga",
            Self::TropicalJungle => "Tropical Jungle",
            Self::OrchidSwamp => "Orchid Swamp",
            Self::GlacialChasm => "Glacial Chasm",
            Self::SiberianTaiga => "Siberian Taiga",
            Self::FrozenCliffs => "Frozen Cliffs",
            Self::WintryForest => "Wintry Forest",
            Self::ColdShrubland => "Cold Shrubland",
            Self::GravelBeach => "Gravel Beach",
            Self::MirageIsles => "Mirage Isles",
            Self::Caldera => "Volcanic Caldera",
            Self::DeepWarmOcean => "Deep Warm Ocean",
            Self::ColdOcean => "Cold Ocean",
            Self::CoralReef => "Coral Reef",
            Self::OceanTrench => "Ocean Trench",
            Self::ThermalCaves => "Thermal Caves",
            Self::UndergroundJungle => "Underground Jungle",
            Self::IceCaves => "Ice Caves",
            Self::MushroomCaves => "Mushroom Caves",
            Self::DuneSea => "Dune Sea",
            Self::PineHighlands => "Pine Highlands",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::AlpineHighlands => "biome.alpine_highlands",
            Self::EmeraldPeaks => "biome.emerald_peaks",
            Self::VolcanicPeaks => "biome.volcanic_peaks",
            Self::PaintedMountains => "biome.painted_mountains",
            Self::RockyMountains => "biome.rocky_mountains",
            Self::ScarletMountains => "biome.scarlet_mountains",
            Self::BryceCanyon => "biome.bryce_canyon",
            Self::AmethystCanyon => "biome.amethyst_canyon",
            Self::BasaltCliffs => "biome.basalt_cliffs",
            Self::GraniteCliffs => "biome.granite_cliffs",
            Self::WhiteCliffs => "biome.white_cliffs",
            Self::YosemiteCliffs => "biome.yosemite_cliffs",
            Self::DesertSpires => "biome.desert_spires",
            Self::AncientSands => "biome.ancient_sands",
            Self::DesertOasis => "biome.desert_oasis",
            Self::FracturedSavanna => "biome.fractured_savanna",
            Self::SavannaBadlands => "biome.savanna_badlands",
            Self::AshenSavanna => "biome.ashen_savanna",
            Self::BloomingValley => "biome.blooming_valley",
            Self::CloudForest => "biome.cloud_forest",
            Self::LavenderValley => "biome.lavender_valley",
            Self::SakuraGrove => "biome.sakura_grove",
            Self::YosemiteLowlands => "biome.yosemite_lowlands",
            Self::Shield => "biome.shield",
            Self::TropicalJungle => "biome.tropical_jungle",
            Self::OrchidSwamp => "biome.orchid_swamp",
            Self::GlacialChasm => "biome.glacial_chasm",
            Self::SiberianTaiga => "biome.siberian_taiga",
            Self::FrozenCliffs => "biome.frozen_cliffs",
            Self::WintryForest => "biome.wintry_forest",
            Self::ColdShrubland => "biome.cold_shrubland",
            Self::GravelBeach => "biome.gravel_beach",
            Self::MirageIsles => "biome.mirage_isles",
            Self::Caldera => "biome.caldera",
            Self::DeepWarmOcean => "biome.deep_warm_ocean",
            Self::ColdOcean => "biome.cold_ocean",
            Self::CoralReef => "biome.coral_reef",
            Self::OceanTrench => "biome.ocean_trench",
            Self::ThermalCaves => "biome.thermal_caves",
            Self::UndergroundJungle => "biome.underground_jungle",
            Self::IceCaves => "biome.ice_caves",
            Self::MushroomCaves => "biome.mushroom_caves",
            Self::DuneSea => "biome.dune_sea",
            Self::PineHighlands => "biome.pine_highlands",
        }
    }

    pub fn config(self) -> BiomeConfig {
        match self {
            // Group A: Extreme Peaks & Mountain Ranges
            Self::AlpineHighlands => BiomeConfig {
                biome_type: self,
                name: "Alpine Highlands",
                surface_material: Voxel::SiltSnow,
                subsoil_material: Voxel::GravelSnowy,
                subsoil_depth: 3,
                base_height_offset: 75.0,
                amplitude_multiplier: 2.4,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::EmeraldPeaks => BiomeConfig {
                biome_type: self,
                name: "Emerald Peaks",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 3,
                base_height_offset: 65.0,
                amplitude_multiplier: 2.2,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::VolcanicPeaks => BiomeConfig {
                biome_type: self,
                name: "Volcanic Peaks",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 80.0,
                amplitude_multiplier: 2.5,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::PaintedMountains => BiomeConfig {
                biome_type: self,
                name: "Painted Mountains",
                surface_material: Voxel::ClayRed,
                subsoil_material: Voxel::ClayYellow,
                subsoil_depth: 5,
                base_height_offset: 55.0,
                amplitude_multiplier: 1.9,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::ClayWhite,
            },
            Self::RockyMountains => BiomeConfig {
                biome_type: self,
                name: "Rocky Mountains",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 3,
                base_height_offset: 70.0,
                amplitude_multiplier: 2.3,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::ScarletMountains => BiomeConfig {
                biome_type: self,
                name: "Scarlet Mountains",
                surface_material: Voxel::LateriteHardpan,
                subsoil_material: Voxel::LateriteDirt,
                subsoil_depth: 4,
                base_height_offset: 60.0,
                amplitude_multiplier: 2.0,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::LateriteCoarse,
            },

            // Group B: Canyons & Dramatic Cliffs
            Self::BryceCanyon => BiomeConfig {
                biome_type: self,
                name: "Bryce Canyon",
                surface_material: Voxel::SandRed,
                subsoil_material: Voxel::ClayRed,
                subsoil_depth: 6,
                base_height_offset: 40.0,
                amplitude_multiplier: 2.1,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::ClayRed,
            },
            Self::AmethystCanyon => BiomeConfig {
                biome_type: self,
                name: "Amethyst Canyon",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 35.0,
                amplitude_multiplier: 2.0,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::Basalt,
            },
            Self::BasaltCliffs => BiomeConfig {
                biome_type: self,
                name: "Basalt Cliffs",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 3,
                base_height_offset: 45.0,
                amplitude_multiplier: 2.2,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::GraniteCliffs => BiomeConfig {
                biome_type: self,
                name: "Granite Cliffs",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 3,
                base_height_offset: 50.0,
                amplitude_multiplier: 2.3,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::WhiteCliffs => BiomeConfig {
                biome_type: self,
                name: "White Cliffs",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::SiltDirt,
                subsoil_depth: 3,
                base_height_offset: 42.0,
                amplitude_multiplier: 2.0,
                primary_stone: Voxel::Chalk,
                cliff_material: Voxel::CobbledChalk,
            },
            Self::YosemiteCliffs => BiomeConfig {
                biome_type: self,
                name: "Yosemite Cliffs",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamMulch,
                subsoil_depth: 4,
                base_height_offset: 55.0,
                amplitude_multiplier: 2.4,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },

            // Group C: Deserts, Savannas & Arid Badlands
            Self::DesertSpires => BiomeConfig {
                biome_type: self,
                name: "Desert Spires",
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 5,
                base_height_offset: 25.0,
                amplitude_multiplier: 1.8,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::SandstoneCommon,
            },
            Self::AncientSands => BiomeConfig {
                biome_type: self,
                name: "Ancient Sands",
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 6,
                base_height_offset: 18.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::SandstoneCommon,
            },
            Self::DesertOasis => BiomeConfig {
                biome_type: self,
                name: "Desert Oasis",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamMud,
                subsoil_depth: 3,
                base_height_offset: 8.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::SandCommon,
            },
            Self::FracturedSavanna => BiomeConfig {
                biome_type: self,
                name: "Fractured Savanna",
                surface_material: Voxel::LateriteGrass,
                subsoil_material: Voxel::LateriteDirt,
                subsoil_depth: 4,
                base_height_offset: 30.0,
                amplitude_multiplier: 1.7,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::LateriteHardpan,
            },
            Self::SavannaBadlands => BiomeConfig {
                biome_type: self,
                name: "Savanna Badlands",
                surface_material: Voxel::LateriteGrass,
                subsoil_material: Voxel::CalicheCoarse,
                subsoil_depth: 4,
                base_height_offset: 20.0,
                amplitude_multiplier: 1.3,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::LateriteHardpan,
            },
            Self::AshenSavanna => BiomeConfig {
                biome_type: self,
                name: "Ashen Savanna",
                surface_material: Voxel::LateriteGrass,
                subsoil_material: Voxel::LateriteDirt,
                subsoil_depth: 3,
                base_height_offset: 22.0,
                amplitude_multiplier: 1.4,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::Basalt,
            },

            // Group D: Forests, Lush Valleys & Plains
            Self::BloomingValley => BiomeConfig {
                biome_type: self,
                name: "Blooming Valley",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 12.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::CloudForest => BiomeConfig {
                biome_type: self,
                name: "Cloud Forest",
                surface_material: Voxel::MossForest,
                subsoil_material: Voxel::LoamMulch,
                subsoil_depth: 5,
                base_height_offset: 45.0,
                amplitude_multiplier: 1.6,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::LavenderValley => BiomeConfig {
                biome_type: self,
                name: "Lavender Valley",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 15.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::SakuraGrove => BiomeConfig {
                biome_type: self,
                name: "Sakura Grove",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamMulch,
                subsoil_depth: 4,
                base_height_offset: 20.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::YosemiteLowlands => BiomeConfig {
                biome_type: self,
                name: "Yosemite Lowlands",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 16.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::Shield => BiomeConfig {
                biome_type: self,
                name: "Shield Taiga",
                surface_material: Voxel::PeatGrass,
                subsoil_material: Voxel::PeatDirt,
                subsoil_depth: 3,
                base_height_offset: 25.0,
                amplitude_multiplier: 1.4,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::TropicalJungle => BiomeConfig {
                biome_type: self,
                name: "Tropical Jungle",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamRooted,
                subsoil_depth: 5,
                base_height_offset: 22.0,
                amplitude_multiplier: 1.5,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::LoamMud,
            },
            Self::OrchidSwamp => BiomeConfig {
                biome_type: self,
                name: "Orchid Swamp",
                surface_material: Voxel::PeatMud,
                subsoil_material: Voxel::PeatDirt,
                subsoil_depth: 4,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.4,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::PeatMud,
            },

            // Group E: Glacial & Subarctic Tundra
            Self::GlacialChasm => BiomeConfig {
                biome_type: self,
                name: "Glacial Chasm",
                surface_material: Voxel::FrostIcePacked,
                subsoil_material: Voxel::FrostBlueIce,
                subsoil_depth: 6,
                base_height_offset: 35.0,
                amplitude_multiplier: 2.1,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::FrostBlueIce,
            },
            Self::SiberianTaiga => BiomeConfig {
                biome_type: self,
                name: "Siberian Taiga",
                surface_material: Voxel::PermafrostGrass,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 4,
                base_height_offset: 28.0,
                amplitude_multiplier: 1.3,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::PermafrostIcy,
            },
            Self::FrozenCliffs => BiomeConfig {
                biome_type: self,
                name: "Frozen Cliffs",
                surface_material: Voxel::SiltSnow,
                subsoil_material: Voxel::FrostIce,
                subsoil_depth: 5,
                base_height_offset: 52.0,
                amplitude_multiplier: 2.3,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::FrostBlueIce,
            },
            Self::WintryForest => BiomeConfig {
                biome_type: self,
                name: "Wintry Forest",
                surface_material: Voxel::LoamSnow,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 4,
                base_height_offset: 24.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::ColdShrubland => BiomeConfig {
                biome_type: self,
                name: "Cold Shrubland",
                surface_material: Voxel::PermafrostCoarse,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 3,
                base_height_offset: 18.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::CobbledGranite,
            },

            // Group F: Coastal, Aquatic & Subterranean Environments
            Self::GravelBeach => BiomeConfig {
                biome_type: self,
                name: "Gravel Beach",
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::GravelDirty,
                subsoil_depth: 4,
                base_height_offset: 1.0,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::MirageIsles => BiomeConfig {
                biome_type: self,
                name: "Mirage Isles",
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 4,
                base_height_offset: 3.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::SandCommon,
            },
            Self::Caldera => BiomeConfig {
                biome_type: self,
                name: "Volcanic Caldera",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 5,
                base_height_offset: 30.0,
                amplitude_multiplier: 2.0,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::DeepWarmOcean => BiomeConfig {
                biome_type: self,
                name: "Deep Warm Ocean",
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::ClayBlue,
                subsoil_depth: 4,
                base_height_offset: -35.0,
                amplitude_multiplier: 0.7,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::Argillite,
            },
            Self::ThermalCaves => BiomeConfig {
                biome_type: self,
                name: "Thermal Caves",
                surface_material: Voxel::Basalt,
                subsoil_material: Voxel::Basalt,
                subsoil_depth: 4,
                base_height_offset: -60.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::UndergroundJungle => BiomeConfig {
                biome_type: self,
                name: "Underground Jungle",
                surface_material: Voxel::MossForest,
                subsoil_material: Voxel::LoamRooted,
                subsoil_depth: 3,
                base_height_offset: -40.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::ClayBlue,
            },
            Self::ColdOcean => BiomeConfig {
                biome_type: self,
                name: "Cold Ocean",
                surface_material: Voxel::GravelSandy,
                subsoil_material: Voxel::ClayBlue,
                subsoil_depth: 4,
                base_height_offset: -32.0,
                amplitude_multiplier: 0.6,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::Argillite,
            },
            Self::CoralReef => BiomeConfig {
                biome_type: self,
                name: "Coral Reef",
                surface_material: Voxel::Reefstone,
                subsoil_material: Voxel::Coralstone,
                subsoil_depth: 3,
                base_height_offset: -12.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::SandstoneWhite,
                cliff_material: Voxel::Reefstone,
            },
            Self::OceanTrench => BiomeConfig {
                biome_type: self,
                name: "Ocean Trench",
                surface_material: Voxel::Basalt,
                subsoil_material: Voxel::Gabbro,
                subsoil_depth: 5,
                base_height_offset: -52.0,
                amplitude_multiplier: 1.4,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::IceCaves => BiomeConfig {
                biome_type: self,
                name: "Ice Caves",
                surface_material: Voxel::FrostIcePacked,
                subsoil_material: Voxel::FrostBlueIce,
                subsoil_depth: 6,
                base_height_offset: -50.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::FrostIcePacked,
            },
            Self::MushroomCaves => BiomeConfig {
                biome_type: self,
                name: "Mushroom Caves",
                surface_material: Voxel::PeatMud,
                subsoil_material: Voxel::LoamRooted,
                subsoil_depth: 4,
                base_height_offset: -45.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::PeatMud,
            },
            Self::DuneSea => BiomeConfig {
                biome_type: self,
                name: "Dune Sea",
                surface_material: Voxel::SandDune,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 8,
                base_height_offset: 24.0,
                amplitude_multiplier: 1.8,
                primary_stone: Voxel::SandstoneDune,
                cliff_material: Voxel::SandstoneCommon,
            },
            Self::PineHighlands => BiomeConfig {
                biome_type: self,
                name: "Pine Highlands",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 52.0,
                amplitude_multiplier: 1.8,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
        }
    }

    /// Grass tint color calibrated for each biome.
    pub fn grass_color(self) -> [f32; 4] {
        match self {
            Self::AlpineHighlands => [0.45, 0.75, 0.55, 1.0],
            Self::EmeraldPeaks => [0.25, 0.95, 0.35, 1.0],
            Self::VolcanicPeaks => [0.35, 0.45, 0.30, 1.0],
            Self::PaintedMountains => [0.70, 0.65, 0.35, 1.0],
            Self::RockyMountains => [0.48, 0.70, 0.45, 1.0],
            Self::ScarletMountains => [0.65, 0.60, 0.30, 1.0],

            Self::BryceCanyon => [0.75, 0.60, 0.25, 1.0],
            Self::AmethystCanyon => [0.45, 0.75, 0.50, 1.0],
            Self::BasaltCliffs => [0.30, 0.50, 0.35, 1.0],
            Self::GraniteCliffs => [0.50, 0.85, 0.40, 1.0],
            Self::WhiteCliffs => [0.60, 0.88, 0.48, 1.0],
            Self::YosemiteCliffs => [0.42, 0.88, 0.38, 1.0],

            Self::DesertSpires => [0.80, 0.75, 0.35, 1.0],
            Self::AncientSands => [0.78, 0.70, 0.30, 1.0],
            Self::DesertOasis => [0.40, 0.96, 0.35, 1.0],
            Self::FracturedSavanna => [0.70, 0.82, 0.28, 1.0],
            Self::SavannaBadlands => [0.72, 0.78, 0.25, 1.0],
            Self::AshenSavanna => [0.55, 0.60, 0.35, 1.0],

            Self::BloomingValley => [0.35, 0.98, 0.45, 1.0],
            Self::CloudForest => [0.20, 0.90, 0.50, 1.0],
            Self::LavenderValley => [0.55, 0.70, 0.85, 1.0],
            Self::SakuraGrove => [0.60, 0.92, 0.65, 1.0],
            Self::YosemiteLowlands => [0.38, 0.90, 0.36, 1.0],
            Self::Shield => [0.40, 0.75, 0.50, 1.0],
            Self::TropicalJungle => [0.15, 0.92, 0.25, 1.0],
            Self::OrchidSwamp => [0.38, 0.80, 0.35, 1.0],

            Self::GlacialChasm => [0.50, 0.80, 0.90, 1.0],
            Self::SiberianTaiga => [0.40, 0.75, 0.60, 1.0],
            Self::FrozenCliffs => [0.45, 0.70, 0.75, 1.0],
            Self::WintryForest => [0.42, 0.72, 0.65, 1.0],
            Self::ColdShrubland => [0.55, 0.70, 0.50, 1.0],

            Self::GravelBeach => [0.58, 0.85, 0.45, 1.0],
            Self::MirageIsles => [0.30, 0.96, 0.50, 1.0],
            Self::Caldera => [0.35, 0.40, 0.30, 1.0],
            Self::DeepWarmOcean => [0.25, 0.85, 0.60, 1.0],
            Self::ColdOcean => [0.40, 0.70, 0.65, 1.0],
            Self::CoralReef => [0.20, 0.96, 0.65, 1.0],
            Self::OceanTrench => [0.20, 0.40, 0.50, 1.0],
            Self::ThermalCaves => [0.40, 0.35, 0.30, 1.0],
            Self::UndergroundJungle => [0.20, 0.95, 0.40, 1.0],
            Self::IceCaves => [0.55, 0.85, 0.95, 1.0],
            Self::MushroomCaves => [0.45, 0.65, 0.50, 1.0],
            Self::DuneSea => [0.82, 0.72, 0.28, 1.0],
            Self::PineHighlands => [0.32, 0.80, 0.44, 1.0],
        }
    }

    /// Water tint color calibrated for each biome.
    pub fn water_color(self) -> [f32; 4] {
        match self {
            Self::DeepWarmOcean | Self::MirageIsles => [0.10, 0.85, 0.88, 1.0],
            Self::CoralReef => [0.08, 0.92, 0.92, 1.0],
            Self::ColdOcean => [0.15, 0.55, 0.82, 1.0],
            Self::OceanTrench => [0.05, 0.16, 0.42, 1.0],
            Self::OrchidSwamp => [0.20, 0.45, 0.35, 1.0],
            Self::GlacialChasm | Self::FrozenCliffs | Self::IceCaves => [0.30, 0.75, 0.98, 1.0],
            Self::VolcanicPeaks | Self::Caldera | Self::ThermalCaves => [0.65, 0.25, 0.15, 1.0],
            Self::BloomingValley | Self::LavenderValley => [0.22, 0.70, 0.95, 1.0],
            Self::TropicalJungle | Self::UndergroundJungle | Self::MushroomCaves => [0.18, 0.75, 0.70, 1.0],
            _ => [0.22, 0.60, 0.90, 1.0],
        }
    }

    /// Foliage tint color for leaves.
    pub fn foliage_color(self, _voxel: Voxel) -> [f32; 4] {
        match self {
            Self::SakuraGrove => [0.95, 0.65, 0.78, 1.0],
            Self::LavenderValley => [0.70, 0.55, 0.85, 1.0],
            Self::TropicalJungle => [0.12, 0.95, 0.22, 1.0],
            Self::SiberianTaiga | Self::WintryForest | Self::PineHighlands => [0.35, 0.75, 0.55, 1.0],
            Self::VolcanicPeaks | Self::AshenSavanna => [0.50, 0.45, 0.35, 1.0],
            _ => self.grass_color(),
        }
    }

    /// Primary tint color for a voxel in this biome.
    pub fn voxel_tint(self, voxel: Voxel) -> [f32; 4] {
        if !voxel.is_tinted() {
            [1.0, 1.0, 1.0, 1.0]
        } else if voxel.is_grass_soil() || voxel.is_tinted_flora() {
            self.grass_color()
        } else if matches!(voxel, Voxel::LiquidWater | Voxel::WaterOccupied) {
            self.water_color()
        } else if voxel.is_leaves() {
            self.foliage_color(voxel)
        } else {
            [1.0, 1.0, 1.0, 1.0]
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

/// A 6-dimensional climate sample replicating Minecraft/worldgen noise parameters.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct ClimateSample {
    pub continentalness: f32,
    pub erosion: f32,
    pub ridges: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub depth: f32,
    pub biome: BiomeType,
}

/// Target coordinate in 6D Multi-Noise space.
struct BiomePoint {
    biome: BiomeType,
    c: f32,
    e: f32,
    r: f32,
    t: f32,
    h: f32,
    d: f32,
}

static BIOME_POINTS: &[BiomePoint] = &[
    // Group A: Extreme Peaks & Mountain Ranges (Inland mountain ridges, low erosion)
    BiomePoint { biome: BiomeType::AlpineHighlands, c: 0.35, e: -0.55, r: 0.60, t: -0.65, h: 0.10, d: 0.0 },
    BiomePoint { biome: BiomeType::EmeraldPeaks,    c: 0.30, e: -0.50, r: 0.65, t: -0.05, h: 0.55, d: 0.0 },
    BiomePoint { biome: BiomeType::VolcanicPeaks,   c: 0.40, e: -0.45, r: 0.75, t: 0.75,  h: -0.45, d: 0.0 },
    BiomePoint { biome: BiomeType::PaintedMountains,c: 0.32, e: -0.40, r: 0.50, t: 0.55,  h: -0.55, d: 0.0 },
    BiomePoint { biome: BiomeType::RockyMountains,  c: 0.36, e: -0.50, r: 0.55, t: -0.20, h: -0.15, d: 0.0 },
    BiomePoint { biome: BiomeType::ScarletMountains,c: 0.34, e: -0.45, r: 0.60, t: 0.65,  h: -0.35, d: 0.0 },

    // Group B: Canyons & Dramatic Cliffs
    BiomePoint { biome: BiomeType::BryceCanyon,    c: 0.28, e: 0.05,  r: 0.50, t: 0.65,  h: -0.65, d: 0.0 },
    BiomePoint { biome: BiomeType::AmethystCanyon, c: 0.25, e: -0.10, r: -0.55,t: 0.15,  h: 0.15,  d: 0.0 },
    BiomePoint { biome: BiomeType::BasaltCliffs,   c: 0.05, e: -0.50, r: 0.35, t: 0.40,  h: -0.10, d: 0.0 },
    BiomePoint { biome: BiomeType::GraniteCliffs,  c: 0.25, e: -0.45, r: 0.45, t: 0.05,  h: 0.25,  d: 0.0 },
    BiomePoint { biome: BiomeType::WhiteCliffs,    c: 0.05, e: -0.45, r: 0.25, t: 0.00,  h: 0.20,  d: 0.0 },
    BiomePoint { biome: BiomeType::YosemiteCliffs, c: 0.30, e: -0.50, r: 0.55, t: 0.15,  h: 0.35,  d: 0.0 },

    // Group C: Deserts, Savannas & Arid Badlands (High temperature, low humidity)
    BiomePoint { biome: BiomeType::DesertSpires,    c: 0.30, e: -0.25, r: 0.50, t: 0.75,  h: -0.55, d: 0.0 },
    BiomePoint { biome: BiomeType::AncientSands,    c: 0.22, e: 0.30,  r: -0.20,t: 0.65,  h: -0.55, d: 0.0 },
    BiomePoint { biome: BiomeType::AncientSands,    c: 0.32, e: 0.10,  r: 0.00, t: 0.70,  h: -0.60, d: 0.0 },
    BiomePoint { biome: BiomeType::DesertOasis,     c: 0.18, e: 0.45,  r: -0.40,t: 0.65,  h: -0.20, d: 0.0 },
    BiomePoint { biome: BiomeType::FracturedSavanna,c: 0.26, e: -0.20, r: 0.35, t: 0.55,  h: -0.30, d: 0.0 },
    BiomePoint { biome: BiomeType::SavannaBadlands, c: 0.24, e: 0.10,  r: 0.10, t: 0.55,  h: -0.45, d: 0.0 },
    BiomePoint { biome: BiomeType::AshenSavanna,    c: 0.35, e: 0.05,  r: 0.15, t: 0.80,  h: -0.35, d: 0.0 },

    // Group D: Forests, Lush Valleys & Plains (Temperate and subtropical core)
    BiomePoint { biome: BiomeType::BloomingValley,  c: 0.10, e: 0.35,  r: -0.25,t: 0.10,  h: 0.30,  d: 0.0 },
    BiomePoint { biome: BiomeType::CloudForest,     c: 0.25, e: -0.15, r: 0.30, t: 0.10,  h: 0.70,  d: 0.0 },
    BiomePoint { biome: BiomeType::LavenderValley,  c: 0.08, e: 0.30,  r: -0.20,t: -0.05, h: 0.25,  d: 0.0 },
    BiomePoint { biome: BiomeType::SakuraGrove,     c: 0.15, e: 0.20,  r: 0.05, t: 0.15,  h: 0.40,  d: 0.0 },
    BiomePoint { biome: BiomeType::YosemiteLowlands,c: 0.12, e: 0.25,  r: -0.15,t: 0.05,  h: 0.15,  d: 0.0 },
    BiomePoint { biome: BiomeType::Shield,          c: 0.18, e: 0.10,  r: 0.10, t: -0.15, h: 0.10,  d: 0.0 },
    BiomePoint { biome: BiomeType::TropicalJungle,  c: 0.12, e: 0.15,  r: -0.10,t: 0.65,  h: 0.75,  d: 0.0 },
    BiomePoint { biome: BiomeType::OrchidSwamp,     c: 0.03, e: 0.55,  r: -0.50,t: 0.45,  h: 0.85,  d: 0.0 },

    // Group E: Glacial & Subarctic Tundra (Cold climate)
    BiomePoint { biome: BiomeType::GlacialChasm,   c: 0.35, e: -0.50, r: -0.65,t: -0.85, h: 0.30,  d: 0.0 },
    BiomePoint { biome: BiomeType::SiberianTaiga,  c: 0.15, e: 0.15,  r: 0.05, t: -0.55, h: 0.25,  d: 0.0 },
    BiomePoint { biome: BiomeType::FrozenCliffs,   c: 0.05, e: -0.55, r: 0.45, t: -0.70, h: 0.10,  d: 0.0 },
    BiomePoint { biome: BiomeType::WintryForest,   c: 0.12, e: 0.25,  r: -0.05,t: -0.65, h: 0.50,  d: 0.0 },
    BiomePoint { biome: BiomeType::ColdShrubland,  c: 0.20, e: 0.35,  r: -0.20,t: -0.45, h: -0.25, d: 0.0 },

    // Group F: Coastal, Aquatic & Subterranean Environments
    BiomePoint { biome: BiomeType::GravelBeach,       c: -0.08,e: 0.40,  r: -0.30,t: -0.25, h: -0.05, d: 0.0 },
    BiomePoint { biome: BiomeType::MirageIsles,       c: -0.08,e: 0.35,  r: -0.25,t: 0.65,  h: 0.40,  d: 0.0 },
    BiomePoint { biome: BiomeType::Caldera,           c: 0.45, e: -0.20, r: -0.75,t: 0.80,  h: 0.00,  d: 0.0 },
    BiomePoint { biome: BiomeType::DeepWarmOcean,     c: -0.40,e: 0.00,  r: 0.00, t: 0.35,  h: 0.20,  d: 0.0 },
    BiomePoint { biome: BiomeType::CoralReef,         c: -0.22,e: 0.20,  r: -0.20,t: 0.70,  h: 0.45,  d: 0.0 },
    BiomePoint { biome: BiomeType::ColdOcean,         c: -0.45,e: 0.00,  r: 0.00, t: -0.65, h: 0.10,  d: 0.0 },
    BiomePoint { biome: BiomeType::OceanTrench,       c: -0.80,e: -0.30, r: 0.20, t: -0.10, h: 0.10,  d: 0.0 },
    BiomePoint { biome: BiomeType::ThermalCaves,      c: 0.25, e: 0.00,  r: 0.00, t: 0.70,  h: -0.50, d: -0.70 },
    BiomePoint { biome: BiomeType::UndergroundJungle, c: 0.15, e: 0.00,  r: 0.00, t: 0.50,  h: 0.80,  d: -0.70 },
    BiomePoint { biome: BiomeType::IceCaves,          c: 0.25, e: 0.00,  r: 0.00, t: -0.70, h: 0.20,  d: -0.75 },
    BiomePoint { biome: BiomeType::MushroomCaves,     c: 0.10, e: 0.00,  r: 0.00, t: 0.10,  h: 0.65,  d: -0.80 },

    // Group G: Surface & Montane Biomes
    BiomePoint { biome: BiomeType::DuneSea,           c: 0.38, e: 0.45,  r: 0.25, t: 0.85,  h: -0.80, d: 0.0 },
    BiomePoint { biome: BiomeType::PineHighlands,     c: 0.28, e: -0.30, r: 0.40, t: -0.35, h: 0.20,  d: 0.0 },
];

/// Macro 6D climate noise generator.
#[derive(Debug, Clone, Reflect)]
pub struct ClimateGenerator {
    pub continentalness_freq: f32,
    pub erosion_freq: f32,
    pub ridges_freq: f32,
    pub temperature_freq: f32,
    pub humidity_freq: f32,
    pub warp_amplitude: f32,
    pub dither_amplitude: f32,
}

impl Default for ClimateGenerator {
    fn default() -> Self {
        Self {
            continentalness_freq: 0.0006,
            erosion_freq: 0.0014,
            ridges_freq: 0.0018,
            temperature_freq: 0.0006,
            humidity_freq: 0.0007,
            warp_amplitude: 32.0,
            dither_amplitude: 4.0,
        }
    }
}

impl ClimateGenerator {
    /// Samples the 6D climate at world coordinates (x, z) with depth d.
    pub fn sample_at_depth(&self, world_x: f32, world_z: f32, depth: f32, seed: u32) -> ClimateSample {
        let continentalness = fbm_2d(
            world_x,
            world_z,
            self.continentalness_freq,
            3,
            0.55,
            2.0,
            seed.wrapping_add(10_007),
        );

        let erosion = fbm_2d(
            world_x,
            world_z,
            self.erosion_freq,
            3,
            0.50,
            2.0,
            seed.wrapping_add(20_011),
        );

        let ridges = fbm_2d(
            world_x,
            world_z,
            self.ridges_freq,
            3,
            0.50,
            2.0,
            seed.wrapping_add(30_013),
        );

        let temperature = fbm_2d(
            world_x,
            world_z,
            self.temperature_freq,
            2,
            0.50,
            2.0,
            seed.wrapping_add(40_017),
        );

        let humidity = fbm_2d(
            world_x,
            world_z,
            self.humidity_freq,
            2,
            0.50,
            2.0,
            seed.wrapping_add(50_021),
        );

        let biome = Self::classify_biome(continentalness, erosion, ridges, temperature, humidity, depth);

        ClimateSample {
            continentalness,
            erosion,
            ridges,
            temperature,
            humidity,
            depth,
            biome,
        }
    }

    /// Samples surface climate (depth = 0.0).
    pub fn sample(&self, world_x: f32, world_z: f32, seed: u32) -> ClimateSample {
        self.sample_at_depth(world_x, world_z, 0.0, seed)
    }

    /// Samples climate with multi-octave domain warping and high-frequency edge dithering.
    pub fn sample_dithered(&self, world_x: f32, world_z: f32, seed: u32) -> ClimateSample {
        let warp_x = fbm_2d(
            world_x,
            world_z,
            0.004,
            2,
            0.5,
            2.0,
            seed.wrapping_add(12_345),
        ) * self.warp_amplitude;
        let warp_z = fbm_2d(
            world_x,
            world_z,
            0.004,
            2,
            0.5,
            2.0,
            seed.wrapping_add(67_890),
        ) * self.warp_amplitude;

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

        self.sample(
            world_x + warp_x + dither_x,
            world_z + warp_z + dither_z,
            seed,
        )
    }

    /// Evaluates 4 climate samples simultaneously using 4-wide SIMD noise.
    pub fn sample_x4(&self, world_x: Simd4f, world_z: Simd4f, seed: u32) -> [ClimateSample; 4] {
        let continentalness = fbm_2d_x4(
            world_x,
            world_z,
            self.continentalness_freq,
            3,
            0.55,
            2.0,
            seed.wrapping_add(10_007),
        );

        let erosion = fbm_2d_x4(
            world_x,
            world_z,
            self.erosion_freq,
            3,
            0.50,
            2.0,
            seed.wrapping_add(20_011),
        );

        let ridges = fbm_2d_x4(
            world_x,
            world_z,
            self.ridges_freq,
            3,
            0.50,
            2.0,
            seed.wrapping_add(30_013),
        );

        let temperature = fbm_2d_x4(
            world_x,
            world_z,
            self.temperature_freq,
            2,
            0.50,
            2.0,
            seed.wrapping_add(40_017),
        );

        let humidity = fbm_2d_x4(
            world_x,
            world_z,
            self.humidity_freq,
            2,
            0.50,
            2.0,
            seed.wrapping_add(50_021),
        );

        std::array::from_fn(|i| {
            let c = continentalness.0[i];
            let e = erosion.0[i];
            let r = ridges.0[i];
            let t = temperature.0[i];
            let h = humidity.0[i];
            let d = 0.0;
            ClimateSample {
                continentalness: c,
                erosion: e,
                ridges: r,
                temperature: t,
                humidity: h,
                depth: d,
                biome: Self::classify_biome(c, e, r, t, h, d),
            }
        })
    }

    /// Samples climate for 4 coordinates simultaneously with SIMD domain warping and edge dithering.
    pub fn sample_dithered_x4(
        &self,
        world_x: Simd4f,
        world_z: Simd4f,
        seed: u32,
    ) -> [ClimateSample; 4] {
        let warp_x = fbm_2d_x4(
            world_x,
            world_z,
            0.004,
            2,
            0.5,
            2.0,
            seed.wrapping_add(12_345),
        )
        .scale(self.warp_amplitude);

        let warp_z = fbm_2d_x4(
            world_x,
            world_z,
            0.004,
            2,
            0.5,
            2.0,
            seed.wrapping_add(67_890),
        )
        .scale(self.warp_amplitude);

        let dither_x = gradient_noise_2d_x4(
            world_x.scale(0.16),
            world_z.scale(0.16),
            seed.wrapping_add(88_331),
        )
        .scale(self.dither_amplitude);

        let dither_z = gradient_noise_2d_x4(
            world_x.scale(0.16).add_scalar(50.0),
            world_z.scale(0.16).add_scalar(50.0),
            seed.wrapping_add(99_442),
        )
        .scale(self.dither_amplitude);

        self.sample_x4(
            world_x.add(warp_x).add(dither_x),
            world_z.add(warp_z).add(dither_z),
            seed,
        )
    }

    /// Classifies the biome using 6D Multi-Noise Voronoi matching with weighted Euclidean distance.
    #[inline(always)]
    pub fn classify_biome(c: f32, e: f32, r: f32, t: f32, h: f32, d: f32) -> BiomeType {
        let mut best_biome = BiomeType::AlpineHighlands;
        let mut min_dist_sq = f32::MAX;

        // Weights: Temperature (3.6) and humidity (2.2) strictly enforce thermal/climate banding,
        // preventing hot biomes from ever bordering cold/frozen biomes.
        const W_C: f32 = 2.0;
        const W_E: f32 = 1.0;
        const W_R: f32 = 1.0;
        const W_T: f32 = 3.6;
        const W_H: f32 = 2.2;
        const W_D: f32 = 3.0;

        for point in BIOME_POINTS {
            let dc = (c - point.c) * W_C;
            let de = (e - point.e) * W_E;
            let dr = (r - point.r) * W_R;
            let dt = (t - point.t) * W_T;
            let dh = (h - point.h) * W_H;
            let dd = (d - point.d) * W_D;

            let dist_sq = dc * dc + de * de + dr * dr + dt * dt + dh * dh + dd * dd;
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_biome = point.biome;
            }
        }

        best_biome
    }
}

/// Smoothly blends biome colors across biome transitions using a 13-point circular Gaussian kernel
/// and quantizes the result into 64 discrete steps per channel.
pub fn sample_blended_biome_color(voxel: Voxel, world_x: f32, world_z: f32, seed: u32) -> [f32; 4] {
    if !voxel.is_tinted() {
        return [1.0, 1.0, 1.0, 1.0];
    }

    let generator = ClimateGenerator::default();

    const R_INNER: f32 = 5.0;
    const R_DIAG: f32 = 3.5355;
    const R_OUTER: f32 = 10.0;

    let offsets: [(f32, f32, f32); 13] = [
        (0.0, 0.0, 0.152),
        (R_INNER, 0.0, 0.092),
        (-R_INNER, 0.0, 0.092),
        (0.0, R_INNER, 0.092),
        (0.0, -R_INNER, 0.092),
        (R_DIAG, R_DIAG, 0.092),
        (-R_DIAG, R_DIAG, 0.092),
        (R_DIAG, -R_DIAG, 0.092),
        (-R_DIAG, -R_DIAG, 0.092),
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

    const QUANTIZE_STEPS: f32 = 64.0;
    [
        (r * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
        (g * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
        (b * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
        (a * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
    ]
}
