use bevy::prelude::*;

use crate::{
    core::noise::{Simd4f, fbm_2d, fbm_2d_x4, gradient_noise_2d_x4},
    world::Voxel,
};

/// 27 official Macro-Biomes of the world architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum MacroBiome {
    // I. Islands (01..=03)
    CoastalIslands,
    OceanicIslands,
    ContinentalIslands,

    // II. Aquatic (04..=06)
    Ocean,
    CoastalSea,
    RiverAndLake,

    // III. Coastal (07..=09)
    OceanicBeach,
    RockyCoast,
    FreshwaterShores,

    // IV. Special & Dark Fantasy (10..=12)
    VolcanicLands,
    ShadowLands,
    BrimstoneBasin,

    // V. Lowlands & Wetlands (13..=15)
    #[default]
    Plains,
    Valley,
    Swamp,

    // VI. Forests (16..=18)
    TemperateForest,
    BorealForest,
    TropicalForest,

    // VII. Mountains & Relief (19..=21)
    MountainRange,
    Highlands,
    MountainGorges,

    // VIII. Arid (22..=24)
    Desert,
    Savanna,
    Canyons,

    // IX. Frozen & Polar (25..=27)
    Tundra,
    Glacier,
    SnowyWoods,
}

impl MacroBiome {
    pub const ALL: [MacroBiome; 27] = [
        Self::CoastalIslands,
        Self::OceanicIslands,
        Self::ContinentalIslands,
        Self::Ocean,
        Self::CoastalSea,
        Self::RiverAndLake,
        Self::OceanicBeach,
        Self::RockyCoast,
        Self::FreshwaterShores,
        Self::VolcanicLands,
        Self::ShadowLands,
        Self::BrimstoneBasin,
        Self::Plains,
        Self::Valley,
        Self::Swamp,
        Self::TemperateForest,
        Self::BorealForest,
        Self::TropicalForest,
        Self::MountainRange,
        Self::Highlands,
        Self::MountainGorges,
        Self::Desert,
        Self::Savanna,
        Self::Canyons,
        Self::Tundra,
        Self::Glacier,
        Self::SnowyWoods,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::CoastalIslands => "Coastal Islands",
            Self::OceanicIslands => "Oceanic Islands",
            Self::ContinentalIslands => "Continental Islands",
            Self::Ocean => "Ocean",
            Self::CoastalSea => "Coastal Sea",
            Self::RiverAndLake => "River and Lake",
            Self::OceanicBeach => "Oceanic Beach",
            Self::RockyCoast => "Rocky Coast",
            Self::FreshwaterShores => "Freshwater Shores",
            Self::VolcanicLands => "Volcanic Lands",
            Self::ShadowLands => "Shadow Lands",
            Self::BrimstoneBasin => "Brimstone Basin",
            Self::Plains => "Plains",
            Self::Valley => "Valley",
            Self::Swamp => "Swamp",
            Self::TemperateForest => "Temperate Forest",
            Self::BorealForest => "Boreal Forest",
            Self::TropicalForest => "Tropical Forest",
            Self::MountainRange => "Mountain Range",
            Self::Highlands => "Highlands",
            Self::MountainGorges => "Mountain Gorges",
            Self::Desert => "Desert",
            Self::Savanna => "Savanna",
            Self::Canyons => "Canyons",
            Self::Tundra => "Tundra",
            Self::Glacier => "Glacier",
            Self::SnowyWoods => "Snowy Woods",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::CoastalIslands => "macro_biome.coastal_islands",
            Self::OceanicIslands => "macro_biome.oceanic_islands",
            Self::ContinentalIslands => "macro_biome.continental_islands",
            Self::Ocean => "macro_biome.ocean",
            Self::CoastalSea => "macro_biome.coastal_sea",
            Self::RiverAndLake => "macro_biome.river_and_lake",
            Self::OceanicBeach => "macro_biome.oceanic_beach",
            Self::RockyCoast => "macro_biome.rocky_coast",
            Self::FreshwaterShores => "macro_biome.freshwater_shores",
            Self::VolcanicLands => "macro_biome.volcanic_lands",
            Self::ShadowLands => "macro_biome.shadow_lands",
            Self::BrimstoneBasin => "macro_biome.brimstone_basin",
            Self::Plains => "macro_biome.plains",
            Self::Valley => "macro_biome.valley",
            Self::Swamp => "macro_biome.swamp",
            Self::TemperateForest => "macro_biome.temperate_forest",
            Self::BorealForest => "macro_biome.boreal_forest",
            Self::TropicalForest => "macro_biome.tropical_forest",
            Self::MountainRange => "macro_biome.mountain_range",
            Self::Highlands => "macro_biome.highlands",
            Self::MountainGorges => "macro_biome.mountain_gorges",
            Self::Desert => "macro_biome.desert",
            Self::Savanna => "macro_biome.savanna",
            Self::Canyons => "macro_biome.canyons",
            Self::Tundra => "macro_biome.tundra",
            Self::Glacier => "macro_biome.glacier",
            Self::SnowyWoods => "macro_biome.snowy_woods",
        }
    }

    /// Official flat RGB legend color from `assets/maps/adventure/macro_biomes.png`.
    pub const fn legend_rgb(self) -> [u8; 3] {
        match self {
            Self::CoastalIslands => [92, 224, 196],
            Self::OceanicIslands => [26, 178, 168],
            Self::ContinentalIslands => [122, 208, 168],
            Self::Ocean => [20, 58, 110],
            Self::CoastalSea => [46, 144, 168],
            Self::RiverAndLake => [44, 132, 214],
            Self::OceanicBeach => [244, 220, 152],
            Self::RockyCoast => [128, 138, 148],
            Self::FreshwaterShores => [154, 178, 118],
            Self::VolcanicLands => [138, 44, 36],
            Self::ShadowLands => [72, 42, 92],
            Self::BrimstoneBasin => [186, 170, 42],
            Self::Plains => [132, 194, 78],
            Self::Valley => [92, 168, 68],
            Self::Swamp => [76, 112, 68],
            Self::TemperateForest => [56, 134, 62],
            Self::BorealForest => [38, 96, 82],
            Self::TropicalForest => [22, 94, 38],
            Self::MountainRange => [156, 156, 166],
            Self::Highlands => [130, 116, 102],
            Self::MountainGorges => [98, 86, 112],
            Self::Desert => [242, 192, 108],
            Self::Savanna => [204, 156, 72],
            Self::Canyons => [198, 102, 58],
            Self::Tundra => [224, 236, 248],
            Self::Glacier => [82, 236, 252],
            Self::SnowyWoods => [154, 194, 214],
        }
    }

    pub const fn from_rgb(rgb: [u8; 3]) -> Option<Self> {
        match rgb {
            [92, 224, 196] => Some(Self::CoastalIslands),
            [26, 178, 168] => Some(Self::OceanicIslands),
            [122, 208, 168] => Some(Self::ContinentalIslands),
            [20, 58, 110] => Some(Self::Ocean),
            [46, 144, 168] => Some(Self::CoastalSea),
            [44, 132, 214] => Some(Self::RiverAndLake),
            [244, 220, 152] => Some(Self::OceanicBeach),
            [128, 138, 148] => Some(Self::RockyCoast),
            [154, 178, 118] => Some(Self::FreshwaterShores),
            [138, 44, 36] => Some(Self::VolcanicLands),
            [72, 42, 92] => Some(Self::ShadowLands),
            [186, 170, 42] => Some(Self::BrimstoneBasin),
            [132, 194, 78] => Some(Self::Plains),
            [92, 168, 68] => Some(Self::Valley),
            [76, 112, 68] => Some(Self::Swamp),
            [56, 134, 62] => Some(Self::TemperateForest),
            [38, 96, 82] => Some(Self::BorealForest),
            [22, 94, 38] => Some(Self::TropicalForest),
            [156, 156, 166] => Some(Self::MountainRange),
            [130, 116, 102] => Some(Self::Highlands),
            [98, 86, 112] => Some(Self::MountainGorges),
            [242, 192, 108] => Some(Self::Desert),
            [204, 156, 72] => Some(Self::Savanna),
            [198, 102, 58] => Some(Self::Canyons),
            [224, 236, 248] => Some(Self::Tundra),
            [82, 236, 252] => Some(Self::Glacier),
            [154, 194, 214] => Some(Self::SnowyWoods),
            _ => None,
        }
    }
}

/// 78 surface Micro-Biomes + 4 Subterranean Biomes (82 total).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum BiomeType {
    // I. Islands (01..=06)
    TropicalIsland,
    TemperateIsland,
    TundraIsland,
    FrozenIsland,
    RockyIsland,
    CoralIsland,

    // II. Aquatic (07..=15)
    WarmOcean,
    ColdOcean,
    OceanTrench,
    CoralReefs,
    SeagrassMeadows,
    KelpForests,
    Rapids,
    CalmLake,
    Estuary,

    // III. Coastal (16..=24)
    SandyShores,
    GravelBeach,
    DuneBeach,
    CraggyBeach,
    TidePools,
    CoastalCliffs,
    Mudflats,
    SwampyShore,
    ReedyShore,

    // IV. Special & Dark Fantasy (25..=33)
    Caldera,
    VolcanicPeaks,
    VolcanicJungle,
    WitheredWoods,
    AshenWastes,
    ObsidianMire,
    AcidPools,
    SulfurVents,
    ScorchedFlats,

    // V. Lowlands & Wetlands (34..=42)
    #[default]
    Grasslands,
    FlowerMeadow,
    RollingSteppe,
    FertileValley,
    LavenderValley,
    SakuraGrove,
    WillowMarsh,
    MangroveSwamp,
    OrchidSwamp,

    // VI. Forests (43..=51)
    VerdantWoods,
    AutumnForest,
    MistyWoods,
    PineHighlands,
    BleakTaiga,
    ShieldForest,
    DenseRainforest,
    BambooGrove,
    JungleWetlands,

    // VII. Mountains & Relief (52..=60)
    AlpinePeaks,
    JaggedPeaks,
    MonolithCliffs,
    EmeraldFoothills,
    GranitePlateau,
    MistyCrags,
    AmethystCanyon,
    DeepGorge,
    WindyPass,

    // VIII. Arid (61..=69)
    DuneSea,
    AncientSands,
    DesertOasis,
    AcaciaPlains,
    FracturedSavanna,
    SavannaBadlands,
    RedrockCanyon,
    PaintedMesas,
    DesertSpires,

    // IX. Frozen & Polar (70..=78)
    SnowyPlains,
    ColdShrubland,
    PermafrostSteppe,
    GlacialChasm,
    IceSheet,
    FrozenCliffs,
    WintryForest,
    FrostPinewoods,
    IceboundRidge,

    // X. Subterranean (79..=82 - untouched)
    ThermalCaves,
    UndergroundJungle,
    IceCaves,
    MushroomCaves,
}

impl BiomeType {
    pub const ALL: [BiomeType; 82] = [
        // I. Islands
        Self::TropicalIsland,
        Self::TemperateIsland,
        Self::TundraIsland,
        Self::FrozenIsland,
        Self::RockyIsland,
        Self::CoralIsland,
        // II. Aquatic
        Self::WarmOcean,
        Self::ColdOcean,
        Self::OceanTrench,
        Self::CoralReefs,
        Self::SeagrassMeadows,
        Self::KelpForests,
        Self::Rapids,
        Self::CalmLake,
        Self::Estuary,
        // III. Coastal
        Self::SandyShores,
        Self::GravelBeach,
        Self::DuneBeach,
        Self::CraggyBeach,
        Self::TidePools,
        Self::CoastalCliffs,
        Self::Mudflats,
        Self::SwampyShore,
        Self::ReedyShore,
        // IV. Special & Dark Fantasy
        Self::Caldera,
        Self::VolcanicPeaks,
        Self::VolcanicJungle,
        Self::WitheredWoods,
        Self::AshenWastes,
        Self::ObsidianMire,
        Self::AcidPools,
        Self::SulfurVents,
        Self::ScorchedFlats,
        // V. Lowlands & Wetlands
        Self::Grasslands,
        Self::FlowerMeadow,
        Self::RollingSteppe,
        Self::FertileValley,
        Self::LavenderValley,
        Self::SakuraGrove,
        Self::WillowMarsh,
        Self::MangroveSwamp,
        Self::OrchidSwamp,
        // VI. Forests
        Self::VerdantWoods,
        Self::AutumnForest,
        Self::MistyWoods,
        Self::PineHighlands,
        Self::BleakTaiga,
        Self::ShieldForest,
        Self::DenseRainforest,
        Self::BambooGrove,
        Self::JungleWetlands,
        // VII. Mountains & Relief
        Self::AlpinePeaks,
        Self::JaggedPeaks,
        Self::MonolithCliffs,
        Self::EmeraldFoothills,
        Self::GranitePlateau,
        Self::MistyCrags,
        Self::AmethystCanyon,
        Self::DeepGorge,
        Self::WindyPass,
        // VIII. Arid
        Self::DuneSea,
        Self::AncientSands,
        Self::DesertOasis,
        Self::AcaciaPlains,
        Self::FracturedSavanna,
        Self::SavannaBadlands,
        Self::RedrockCanyon,
        Self::PaintedMesas,
        Self::DesertSpires,
        // IX. Frozen & Polar
        Self::SnowyPlains,
        Self::ColdShrubland,
        Self::PermafrostSteppe,
        Self::GlacialChasm,
        Self::IceSheet,
        Self::FrozenCliffs,
        Self::WintryForest,
        Self::FrostPinewoods,
        Self::IceboundRidge,
        // X. Subterranean
        Self::ThermalCaves,
        Self::UndergroundJungle,
        Self::IceCaves,
        Self::MushroomCaves,
    ];

    pub const fn parent_macro_biome(self) -> Option<MacroBiome> {
        match self {
            Self::TropicalIsland | Self::CoralIsland => Some(MacroBiome::CoastalIslands),
            Self::TemperateIsland | Self::RockyIsland => Some(MacroBiome::OceanicIslands),
            Self::TundraIsland | Self::FrozenIsland => Some(MacroBiome::ContinentalIslands),
            Self::WarmOcean | Self::ColdOcean | Self::OceanTrench => Some(MacroBiome::Ocean),
            Self::CoralReefs | Self::SeagrassMeadows | Self::KelpForests => {
                Some(MacroBiome::CoastalSea)
            }
            Self::Rapids | Self::CalmLake | Self::Estuary => Some(MacroBiome::RiverAndLake),
            Self::SandyShores | Self::GravelBeach | Self::DuneBeach => {
                Some(MacroBiome::OceanicBeach)
            }
            Self::CraggyBeach | Self::TidePools | Self::CoastalCliffs => {
                Some(MacroBiome::RockyCoast)
            }
            Self::Mudflats | Self::SwampyShore | Self::ReedyShore => {
                Some(MacroBiome::FreshwaterShores)
            }
            Self::Caldera | Self::VolcanicPeaks | Self::VolcanicJungle => {
                Some(MacroBiome::VolcanicLands)
            }
            Self::WitheredWoods | Self::AshenWastes | Self::ObsidianMire => {
                Some(MacroBiome::ShadowLands)
            }
            Self::AcidPools | Self::SulfurVents | Self::ScorchedFlats => {
                Some(MacroBiome::BrimstoneBasin)
            }
            Self::Grasslands | Self::FlowerMeadow | Self::RollingSteppe => Some(MacroBiome::Plains),
            Self::FertileValley | Self::LavenderValley | Self::SakuraGrove => {
                Some(MacroBiome::Valley)
            }
            Self::WillowMarsh | Self::MangroveSwamp | Self::OrchidSwamp => Some(MacroBiome::Swamp),
            Self::VerdantWoods | Self::AutumnForest | Self::MistyWoods => {
                Some(MacroBiome::TemperateForest)
            }
            Self::PineHighlands | Self::BleakTaiga | Self::ShieldForest => {
                Some(MacroBiome::BorealForest)
            }
            Self::DenseRainforest | Self::BambooGrove | Self::JungleWetlands => {
                Some(MacroBiome::TropicalForest)
            }
            Self::AlpinePeaks | Self::JaggedPeaks | Self::MonolithCliffs => {
                Some(MacroBiome::MountainRange)
            }
            Self::EmeraldFoothills | Self::GranitePlateau | Self::MistyCrags => {
                Some(MacroBiome::Highlands)
            }
            Self::AmethystCanyon | Self::DeepGorge | Self::WindyPass => {
                Some(MacroBiome::MountainGorges)
            }
            Self::DuneSea | Self::AncientSands | Self::DesertOasis => Some(MacroBiome::Desert),
            Self::AcaciaPlains | Self::FracturedSavanna | Self::SavannaBadlands => {
                Some(MacroBiome::Savanna)
            }
            Self::RedrockCanyon | Self::PaintedMesas | Self::DesertSpires => {
                Some(MacroBiome::Canyons)
            }
            Self::SnowyPlains | Self::ColdShrubland | Self::PermafrostSteppe => {
                Some(MacroBiome::Tundra)
            }
            Self::GlacialChasm | Self::IceSheet | Self::FrozenCliffs => Some(MacroBiome::Glacier),
            Self::WintryForest | Self::FrostPinewoods | Self::IceboundRidge => {
                Some(MacroBiome::SnowyWoods)
            }
            Self::ThermalCaves
            | Self::UndergroundJungle
            | Self::IceCaves
            | Self::MushroomCaves => None,
        }
    }

    /// Official flat RGB legend color from `assets/maps/adventure/micro_biomes.png`.
    pub const fn legend_rgb(self) -> [u8; 3] {
        match self {
            Self::TropicalIsland => [24, 196, 156],
            Self::TemperateIsland => [88, 214, 141],
            Self::TundraIsland => [200, 220, 232],
            Self::FrozenIsland => [92, 238, 255],
            Self::RockyIsland => [140, 155, 166],
            Self::CoralIsland => [56, 232, 216],
            Self::WarmOcean => [27, 79, 140],
            Self::ColdOcean => [21, 53, 94],
            Self::OceanTrench => [11, 29, 56],
            Self::CoralReefs => [50, 184, 200],
            Self::SeagrassMeadows => [42, 150, 158],
            Self::KelpForests => [30, 114, 130],
            Self::Rapids => [72, 156, 232],
            Self::CalmLake => [36, 116, 196],
            Self::Estuary => [52, 164, 184],
            Self::SandyShores => [246, 224, 160],
            Self::GravelBeach => [200, 190, 158],
            Self::DuneBeach => [234, 184, 106],
            Self::CraggyBeach => [150, 160, 168],
            Self::TidePools => [116, 140, 148],
            Self::CoastalCliffs => [94, 102, 110],
            Self::Mudflats => [142, 124, 94],
            Self::SwampyShore => [104, 140, 84],
            Self::ReedyShore => [164, 188, 120],
            Self::Caldera => [212, 88, 42],
            Self::VolcanicPeaks => [122, 36, 32],
            Self::VolcanicJungle => [74, 56, 40],
            Self::WitheredWoods => [78, 50, 100],
            Self::AshenWastes => [56, 38, 68],
            Self::ObsidianMire => [36, 24, 48],
            Self::AcidPools => [196, 212, 40],
            Self::SulfurVents => [184, 158, 32],
            Self::ScorchedFlats => [140, 114, 36],
            Self::Grasslands => [132, 196, 78],
            Self::FlowerMeadow => [158, 216, 92],
            Self::RollingSteppe => [168, 184, 82],
            Self::FertileValley => [92, 168, 68],
            Self::LavenderValley => [156, 120, 184],
            Self::SakuraGrove => [228, 136, 168],
            Self::WillowMarsh => [78, 118, 72],
            Self::MangroveSwamp => [60, 98, 68],
            Self::OrchidSwamp => [100, 140, 110],
            Self::VerdantWoods => [56, 134, 62],
            Self::AutumnForest => [196, 122, 50],
            Self::MistyWoods => [46, 116, 84],
            Self::PineHighlands => [46, 110, 86],
            Self::BleakTaiga => [32, 80, 68],
            Self::ShieldForest => [58, 120, 104],
            Self::DenseRainforest => [22, 94, 38],
            Self::BambooGrove => [68, 140, 54],
            Self::JungleWetlands => [30, 112, 68],
            Self::AlpinePeaks => [212, 220, 228],
            Self::JaggedPeaks => [148, 150, 160],
            Self::MonolithCliffs => [116, 118, 130],
            Self::EmeraldFoothills => [104, 140, 94],
            Self::GranitePlateau => [138, 124, 110],
            Self::MistyCrags => [110, 106, 116],
            Self::AmethystCanyon => [126, 84, 148],
            Self::DeepGorge => [88, 80, 96],
            Self::WindyPass => [122, 114, 132],
            Self::DuneSea => [244, 196, 100],
            Self::AncientSands => [226, 174, 88],
            Self::DesertOasis => [56, 184, 104],
            Self::AcaciaPlains => [208, 162, 76],
            Self::FracturedSavanna => [184, 134, 60],
            Self::SavannaBadlands => [166, 110, 52],
            Self::RedrockCanyon => [200, 90, 50],
            Self::PaintedMesas => [216, 126, 74],
            Self::DesertSpires => [168, 74, 40],
            Self::SnowyPlains => [230, 240, 250],
            Self::ColdShrubland => [184, 200, 206],
            Self::PermafrostSteppe => [202, 216, 228],
            Self::GlacialChasm => [40, 184, 216],
            Self::IceSheet => [100, 238, 252],
            Self::FrozenCliffs => [66, 208, 232],
            Self::WintryForest => [156, 194, 214],
            Self::FrostPinewoods => [126, 168, 190],
            Self::IceboundRidge => [180, 212, 228],
            Self::ThermalCaves => [160, 60, 30],
            Self::UndergroundJungle => [40, 160, 80],
            Self::IceCaves => [120, 210, 245],
            Self::MushroomCaves => [140, 90, 180],
        }
    }

    pub const fn from_rgb(rgb: [u8; 3]) -> Option<Self> {
        match rgb {
            [24, 196, 156] => Some(Self::TropicalIsland),
            [88, 214, 141] => Some(Self::TemperateIsland),
            [200, 220, 232] => Some(Self::TundraIsland),
            [92, 238, 255] => Some(Self::FrozenIsland),
            [140, 155, 166] => Some(Self::RockyIsland),
            [56, 232, 216] => Some(Self::CoralIsland),
            [27, 79, 140] => Some(Self::WarmOcean),
            [21, 53, 94] => Some(Self::ColdOcean),
            [11, 29, 56] => Some(Self::OceanTrench),
            [50, 184, 200] => Some(Self::CoralReefs),
            [42, 150, 158] => Some(Self::SeagrassMeadows),
            [30, 114, 130] => Some(Self::KelpForests),
            [72, 156, 232] => Some(Self::Rapids),
            [36, 116, 196] => Some(Self::CalmLake),
            [52, 164, 184] => Some(Self::Estuary),
            [246, 224, 160] => Some(Self::SandyShores),
            [200, 190, 158] => Some(Self::GravelBeach),
            [234, 184, 106] => Some(Self::DuneBeach),
            [150, 160, 168] => Some(Self::CraggyBeach),
            [116, 140, 148] => Some(Self::TidePools),
            [94, 102, 110] => Some(Self::CoastalCliffs),
            [142, 124, 94] => Some(Self::Mudflats),
            [104, 140, 84] => Some(Self::SwampyShore),
            [164, 188, 120] => Some(Self::ReedyShore),
            [212, 88, 42] => Some(Self::Caldera),
            [122, 36, 32] => Some(Self::VolcanicPeaks),
            [74, 56, 40] => Some(Self::VolcanicJungle),
            [78, 50, 100] => Some(Self::WitheredWoods),
            [56, 38, 68] => Some(Self::AshenWastes),
            [36, 24, 48] => Some(Self::ObsidianMire),
            [196, 212, 40] => Some(Self::AcidPools),
            [184, 158, 32] => Some(Self::SulfurVents),
            [140, 114, 36] => Some(Self::ScorchedFlats),
            [132, 196, 78] => Some(Self::Grasslands),
            [158, 216, 92] => Some(Self::FlowerMeadow),
            [168, 184, 82] => Some(Self::RollingSteppe),
            [92, 168, 68] => Some(Self::FertileValley),
            [156, 120, 184] => Some(Self::LavenderValley),
            [228, 136, 168] => Some(Self::SakuraGrove),
            [78, 118, 72] => Some(Self::WillowMarsh),
            [60, 98, 68] => Some(Self::MangroveSwamp),
            [100, 140, 110] => Some(Self::OrchidSwamp),
            [56, 134, 62] => Some(Self::VerdantWoods),
            [196, 122, 50] => Some(Self::AutumnForest),
            [46, 116, 84] => Some(Self::MistyWoods),
            [46, 110, 86] => Some(Self::PineHighlands),
            [32, 80, 68] => Some(Self::BleakTaiga),
            [58, 120, 104] => Some(Self::ShieldForest),
            [22, 94, 38] => Some(Self::DenseRainforest),
            [68, 140, 54] => Some(Self::BambooGrove),
            [30, 112, 68] => Some(Self::JungleWetlands),
            [212, 220, 228] => Some(Self::AlpinePeaks),
            [148, 150, 160] => Some(Self::JaggedPeaks),
            [116, 118, 130] => Some(Self::MonolithCliffs),
            [104, 140, 94] => Some(Self::EmeraldFoothills),
            [138, 124, 110] => Some(Self::GranitePlateau),
            [110, 106, 116] => Some(Self::MistyCrags),
            [126, 84, 148] => Some(Self::AmethystCanyon),
            [88, 80, 96] => Some(Self::DeepGorge),
            [122, 114, 132] => Some(Self::WindyPass),
            [244, 196, 100] => Some(Self::DuneSea),
            [226, 174, 88] => Some(Self::AncientSands),
            [56, 184, 104] => Some(Self::DesertOasis),
            [208, 162, 76] => Some(Self::AcaciaPlains),
            [184, 134, 60] => Some(Self::FracturedSavanna),
            [166, 110, 52] => Some(Self::SavannaBadlands),
            [200, 90, 50] => Some(Self::RedrockCanyon),
            [216, 126, 74] => Some(Self::PaintedMesas),
            [168, 74, 40] => Some(Self::DesertSpires),
            [230, 240, 250] => Some(Self::SnowyPlains),
            [184, 200, 206] => Some(Self::ColdShrubland),
            [202, 216, 228] => Some(Self::PermafrostSteppe),
            [40, 184, 216] => Some(Self::GlacialChasm),
            [100, 238, 252] => Some(Self::IceSheet),
            [66, 208, 232] => Some(Self::FrozenCliffs),
            [156, 194, 214] => Some(Self::WintryForest),
            [126, 168, 190] => Some(Self::FrostPinewoods),
            [180, 212, 228] => Some(Self::IceboundRidge),
            _ => None,
        }
    }

    pub fn category_key(self) -> &'static str {
        match self {
            Self::TropicalIsland
            | Self::TemperateIsland
            | Self::TundraIsland
            | Self::FrozenIsland
            | Self::RockyIsland
            | Self::CoralIsland => "journal.biome_cat.islands",

            Self::WarmOcean
            | Self::ColdOcean
            | Self::OceanTrench
            | Self::CoralReefs
            | Self::SeagrassMeadows
            | Self::KelpForests
            | Self::Rapids
            | Self::CalmLake
            | Self::Estuary => "journal.biome_cat.aquatic",

            Self::SandyShores
            | Self::GravelBeach
            | Self::DuneBeach
            | Self::CraggyBeach
            | Self::TidePools
            | Self::CoastalCliffs
            | Self::Mudflats
            | Self::SwampyShore
            | Self::ReedyShore => "journal.biome_cat.coastal",

            Self::Caldera
            | Self::VolcanicPeaks
            | Self::VolcanicJungle
            | Self::WitheredWoods
            | Self::AshenWastes
            | Self::ObsidianMire
            | Self::AcidPools
            | Self::SulfurVents
            | Self::ScorchedFlats => "journal.biome_cat.special",

            Self::Grasslands
            | Self::FlowerMeadow
            | Self::RollingSteppe
            | Self::FertileValley
            | Self::LavenderValley
            | Self::SakuraGrove
            | Self::WillowMarsh
            | Self::MangroveSwamp
            | Self::OrchidSwamp => "journal.biome_cat.lowlands",

            Self::VerdantWoods
            | Self::AutumnForest
            | Self::MistyWoods
            | Self::PineHighlands
            | Self::BleakTaiga
            | Self::ShieldForest
            | Self::DenseRainforest
            | Self::BambooGrove
            | Self::JungleWetlands => "journal.biome_cat.forests",

            Self::AlpinePeaks
            | Self::JaggedPeaks
            | Self::MonolithCliffs
            | Self::EmeraldFoothills
            | Self::GranitePlateau
            | Self::MistyCrags
            | Self::AmethystCanyon
            | Self::DeepGorge
            | Self::WindyPass => "journal.biome_cat.peaks",

            Self::DuneSea
            | Self::AncientSands
            | Self::DesertOasis
            | Self::AcaciaPlains
            | Self::FracturedSavanna
            | Self::SavannaBadlands
            | Self::RedrockCanyon
            | Self::PaintedMesas
            | Self::DesertSpires => "journal.biome_cat.arid",

            Self::SnowyPlains
            | Self::ColdShrubland
            | Self::PermafrostSteppe
            | Self::GlacialChasm
            | Self::IceSheet
            | Self::FrozenCliffs
            | Self::WintryForest
            | Self::FrostPinewoods
            | Self::IceboundRidge => "journal.biome_cat.glacial",

            Self::ThermalCaves | Self::UndergroundJungle | Self::IceCaves | Self::MushroomCaves => {
                "journal.biome_cat.caves"
            }
        }
    }

    pub fn icon_asset_path(self) -> &'static str {
        match self {
            // I. Islands
            Self::TropicalIsland => "textures/interfaces/icons/biomes/biome_tropical_island.png",
            Self::TemperateIsland => "textures/interfaces/icons/biomes/biome_temperate_island.png",
            Self::TundraIsland => "textures/interfaces/icons/biomes/biome_tundra_island.png",
            Self::FrozenIsland => "textures/interfaces/icons/biomes/biome_frozen_island.png",
            Self::RockyIsland => "textures/interfaces/icons/biomes/biome_rocky_island.png",
            Self::CoralIsland => "textures/interfaces/icons/biomes/biome_coral_island.png",

            // II. Aquatic
            Self::WarmOcean => "textures/interfaces/icons/biomes/biome_warm_ocean.png",
            Self::ColdOcean => "textures/interfaces/icons/biomes/biome_cold_ocean.png",
            Self::OceanTrench => "textures/interfaces/icons/biomes/biome_ocean_trench.png",
            Self::CoralReefs => "textures/interfaces/icons/biomes/biome_coral_reefs.png",
            Self::SeagrassMeadows => "textures/interfaces/icons/biomes/biome_seagrass_meadows.png",
            Self::KelpForests => "textures/interfaces/icons/biomes/biome_kelp_forests.png",
            Self::Rapids => "textures/interfaces/icons/biomes/biome_rapids.png",
            Self::CalmLake => "textures/interfaces/icons/biomes/biome_calm_lake.png",
            Self::Estuary => "textures/interfaces/icons/biomes/biome_estuary.png",

            // III. Coastal
            Self::SandyShores => "textures/interfaces/icons/biomes/biome_sandy_shores.png",
            Self::GravelBeach => "textures/interfaces/icons/biomes/biome_gravel_beach.png",
            Self::DuneBeach => "textures/interfaces/icons/biomes/biome_dune_beach.png",
            Self::CraggyBeach => "textures/interfaces/icons/biomes/biome_craggy_beach.png",
            Self::TidePools => "textures/interfaces/icons/biomes/biome_tide_pools.png",
            Self::CoastalCliffs => "textures/interfaces/icons/biomes/biome_coastal_cliffs.png",
            Self::Mudflats => "textures/interfaces/icons/biomes/biome_mudflats.png",
            Self::SwampyShore => "textures/interfaces/icons/biomes/biome_swampy_shore.png",
            Self::ReedyShore => "textures/interfaces/icons/biomes/biome_reedy_shore.png",

            // IV. Special & Dark Fantasy
            Self::Caldera => "textures/interfaces/icons/biomes/biome_caldera.png",
            Self::VolcanicPeaks => "textures/interfaces/icons/biomes/biome_volcanic_peaks.png",
            Self::VolcanicJungle => "textures/interfaces/icons/biomes/biome_volcanic_jungle.png",
            Self::WitheredWoods => "textures/interfaces/icons/biomes/biome_withered_woods.png",
            Self::AshenWastes => "textures/interfaces/icons/biomes/biome_ashen_wastes.png",
            Self::ObsidianMire => "textures/interfaces/icons/biomes/biome_obsidian_mire.png",
            Self::AcidPools => "textures/interfaces/icons/biomes/biome_acid_pools.png",
            Self::SulfurVents => "textures/interfaces/icons/biomes/biome_sulfur_vents.png",
            Self::ScorchedFlats => "textures/interfaces/icons/biomes/biome_scorched_flats.png",

            // V. Lowlands & Wetlands
            Self::Grasslands => "textures/interfaces/icons/biomes/biome_grasslands.png",
            Self::FlowerMeadow => "textures/interfaces/icons/biomes/biome_flower_meadow.png",
            Self::RollingSteppe => "textures/interfaces/icons/biomes/biome_rolling_steppe.png",
            Self::FertileValley => "textures/interfaces/icons/biomes/biome_fertile_valley.png",
            Self::LavenderValley => "textures/interfaces/icons/biomes/biome_lavender_valley.png",
            Self::SakuraGrove => "textures/interfaces/icons/biomes/biome_sakura_grove.png",
            Self::WillowMarsh => "textures/interfaces/icons/biomes/biome_willow_marsh.png",
            Self::MangroveSwamp => "textures/interfaces/icons/biomes/biome_mangrove_swamp.png",
            Self::OrchidSwamp => "textures/interfaces/icons/biomes/biome_orchid_swamp.png",

            // VI. Forests
            Self::VerdantWoods => "textures/interfaces/icons/biomes/biome_verdant_woods.png",
            Self::AutumnForest => "textures/interfaces/icons/biomes/biome_autumn_forest.png",
            Self::MistyWoods => "textures/interfaces/icons/biomes/biome_misty_woods.png",
            Self::PineHighlands => "textures/interfaces/icons/biomes/biome_pine_highlands.png",
            Self::BleakTaiga => "textures/interfaces/icons/biomes/biome_bleak_taiga.png",
            Self::ShieldForest => "textures/interfaces/icons/biomes/biome_shield_forest.png",
            Self::DenseRainforest => "textures/interfaces/icons/biomes/biome_dense_rainforest.png",
            Self::BambooGrove => "textures/interfaces/icons/biomes/biome_bamboo_grove.png",
            Self::JungleWetlands => "textures/interfaces/icons/biomes/biome_jungle_wetlands.png",

            // VII. Mountains & Relief
            Self::AlpinePeaks => "textures/interfaces/icons/biomes/biome_alpine_peaks.png",
            Self::JaggedPeaks => "textures/interfaces/icons/biomes/biome_jagged_peaks.png",
            Self::MonolithCliffs => "textures/interfaces/icons/biomes/biome_monolith_cliffs.png",
            Self::EmeraldFoothills => {
                "textures/interfaces/icons/biomes/biome_emerald_foothills.png"
            }
            Self::GranitePlateau => "textures/interfaces/icons/biomes/biome_granite_plateau.png",
            Self::MistyCrags => "textures/interfaces/icons/biomes/biome_misty_crags.png",
            Self::AmethystCanyon => "textures/interfaces/icons/biomes/biome_amethyst_canyon.png",
            Self::DeepGorge => "textures/interfaces/icons/biomes/biome_deep_gorge.png",
            Self::WindyPass => "textures/interfaces/icons/biomes/biome_windy_pass.png",

            // VIII. Arid
            Self::DuneSea => "textures/interfaces/icons/biomes/biome_dune_sea.png",
            Self::AncientSands => "textures/interfaces/icons/biomes/biome_ancient_sands.png",
            Self::DesertOasis => "textures/interfaces/icons/biomes/biome_desert_oasis.png",
            Self::AcaciaPlains => "textures/interfaces/icons/biomes/biome_acacia_plains.png",
            Self::FracturedSavanna => {
                "textures/interfaces/icons/biomes/biome_fractured_savanna.png"
            }
            Self::SavannaBadlands => "textures/interfaces/icons/biomes/biome_savanna_badlands.png",
            Self::RedrockCanyon => "textures/interfaces/icons/biomes/biome_redrock_canyon.png",
            Self::PaintedMesas => "textures/interfaces/icons/biomes/biome_painted_mesas.png",
            Self::DesertSpires => "textures/interfaces/icons/biomes/biome_desert_spires.png",

            // IX. Frozen & Polar
            Self::SnowyPlains => "textures/interfaces/icons/biomes/biome_snowy_plains.png",
            Self::ColdShrubland => "textures/interfaces/icons/biomes/biome_cold_shrubland.png",
            Self::PermafrostSteppe => {
                "textures/interfaces/icons/biomes/biome_permafrost_steppe.png"
            }
            Self::GlacialChasm => "textures/interfaces/icons/biomes/biome_glacial_chasm.png",
            Self::IceSheet => "textures/interfaces/icons/biomes/biome_ice_sheet.png",
            Self::FrozenCliffs => "textures/interfaces/icons/biomes/biome_frozen_cliffs.png",
            Self::WintryForest => "textures/interfaces/icons/biomes/biome_wintry_forest.png",
            Self::FrostPinewoods => "textures/interfaces/icons/biomes/biome_frost_pinewoods.png",
            Self::IceboundRidge => "textures/interfaces/icons/biomes/biome_icebound_ridge.png",

            // X. Subterranean
            Self::ThermalCaves => "textures/interfaces/icons/biomes/biome_thermal_caves.png",
            Self::UndergroundJungle => {
                "textures/interfaces/icons/biomes/biome_underground_jungle.png"
            }
            Self::IceCaves => "textures/interfaces/icons/biomes/biome_ice_caves.png",
            Self::MushroomCaves => "textures/interfaces/icons/biomes/biome_mushroom_caves.png",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::TropicalIsland => "Tropical Island",
            Self::TemperateIsland => "Temperate Island",
            Self::TundraIsland => "Tundra Island",
            Self::FrozenIsland => "Frozen Island",
            Self::RockyIsland => "Rocky Island",
            Self::CoralIsland => "Coral Island",
            Self::WarmOcean => "Warm Ocean",
            Self::ColdOcean => "Cold Ocean",
            Self::OceanTrench => "Ocean Trench",
            Self::CoralReefs => "Coral Reefs",
            Self::SeagrassMeadows => "Seagrass Meadows",
            Self::KelpForests => "Kelp Forests",
            Self::Rapids => "Rapids",
            Self::CalmLake => "Calm Lake",
            Self::Estuary => "Estuary",
            Self::SandyShores => "Sandy Shores",
            Self::GravelBeach => "Gravel Beach",
            Self::DuneBeach => "Dune Beach",
            Self::CraggyBeach => "Craggy Beach",
            Self::TidePools => "Tide Pools",
            Self::CoastalCliffs => "Coastal Cliffs",
            Self::Mudflats => "Mudflats",
            Self::SwampyShore => "Swampy Shore",
            Self::ReedyShore => "Reedy Shore",
            Self::Caldera => "Caldera",
            Self::VolcanicPeaks => "Volcanic Peaks",
            Self::VolcanicJungle => "Volcanic Jungle",
            Self::WitheredWoods => "Withered Woods",
            Self::AshenWastes => "Ashen Wastes",
            Self::ObsidianMire => "Obsidian Mire",
            Self::AcidPools => "Acid Pools",
            Self::SulfurVents => "Sulfur Vents",
            Self::ScorchedFlats => "Scorched Flats",
            Self::Grasslands => "Grasslands",
            Self::FlowerMeadow => "Flower Meadow",
            Self::RollingSteppe => "Rolling Steppe",
            Self::FertileValley => "Fertile Valley",
            Self::LavenderValley => "Lavender Valley",
            Self::SakuraGrove => "Sakura Grove",
            Self::WillowMarsh => "Willow Marsh",
            Self::MangroveSwamp => "Mangrove Swamp",
            Self::OrchidSwamp => "Orchid Swamp",
            Self::VerdantWoods => "Verdant Woods",
            Self::AutumnForest => "Autumn Forest",
            Self::MistyWoods => "Misty Woods",
            Self::PineHighlands => "Pine Highlands",
            Self::BleakTaiga => "Bleak Taiga",
            Self::ShieldForest => "Shield Forest",
            Self::DenseRainforest => "Dense Rainforest",
            Self::BambooGrove => "Bamboo Grove",
            Self::JungleWetlands => "Jungle Wetlands",
            Self::AlpinePeaks => "Alpine Peaks",
            Self::JaggedPeaks => "Jagged Peaks",
            Self::MonolithCliffs => "Monolith Cliffs",
            Self::EmeraldFoothills => "Emerald Foothills",
            Self::GranitePlateau => "Granite Plateau",
            Self::MistyCrags => "Misty Crags",
            Self::AmethystCanyon => "Amethyst Canyon",
            Self::DeepGorge => "Deep Gorge",
            Self::WindyPass => "Windy Pass",
            Self::DuneSea => "Dune Sea",
            Self::AncientSands => "Ancient Sands",
            Self::DesertOasis => "Desert Oasis",
            Self::AcaciaPlains => "Acacia Plains",
            Self::FracturedSavanna => "Fractured Savanna",
            Self::SavannaBadlands => "Savanna Badlands",
            Self::RedrockCanyon => "Redrock Canyon",
            Self::PaintedMesas => "Painted Mesas",
            Self::DesertSpires => "Desert Spires",
            Self::SnowyPlains => "Snowy Plains",
            Self::ColdShrubland => "Cold Shrubland",
            Self::PermafrostSteppe => "Permafrost Steppe",
            Self::GlacialChasm => "Glacial Chasm",
            Self::IceSheet => "Ice Sheet",
            Self::FrozenCliffs => "Frozen Cliffs",
            Self::WintryForest => "Wintry Forest",
            Self::FrostPinewoods => "Frost Pinewoods",
            Self::IceboundRidge => "Icebound Ridge",
            Self::ThermalCaves => "Thermal Caves",
            Self::UndergroundJungle => "Underground Jungle",
            Self::IceCaves => "Ice Caves",
            Self::MushroomCaves => "Mushroom Caves",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::TropicalIsland => "biome.tropical_island",
            Self::TemperateIsland => "biome.temperate_island",
            Self::TundraIsland => "biome.tundra_island",
            Self::FrozenIsland => "biome.frozen_island",
            Self::RockyIsland => "biome.rocky_island",
            Self::CoralIsland => "biome.coral_island",
            Self::WarmOcean => "biome.warm_ocean",
            Self::ColdOcean => "biome.cold_ocean",
            Self::OceanTrench => "biome.ocean_trench",
            Self::CoralReefs => "biome.coral_reefs",
            Self::SeagrassMeadows => "biome.seagrass_meadows",
            Self::KelpForests => "biome.kelp_forests",
            Self::Rapids => "biome.rapids",
            Self::CalmLake => "biome.calm_lake",
            Self::Estuary => "biome.estuary",
            Self::SandyShores => "biome.sandy_shores",
            Self::GravelBeach => "biome.gravel_beach",
            Self::DuneBeach => "biome.dune_beach",
            Self::CraggyBeach => "biome.craggy_beach",
            Self::TidePools => "biome.tide_pools",
            Self::CoastalCliffs => "biome.coastal_cliffs",
            Self::Mudflats => "biome.mudflats",
            Self::SwampyShore => "biome.swampy_shore",
            Self::ReedyShore => "biome.reedy_shore",
            Self::Caldera => "biome.caldera",
            Self::VolcanicPeaks => "biome.volcanic_peaks",
            Self::VolcanicJungle => "biome.volcanic_jungle",
            Self::WitheredWoods => "biome.withered_woods",
            Self::AshenWastes => "biome.ashen_wastes",
            Self::ObsidianMire => "biome.obsidian_mire",
            Self::AcidPools => "biome.acid_pools",
            Self::SulfurVents => "biome.sulfur_vents",
            Self::ScorchedFlats => "biome.scorched_flats",
            Self::Grasslands => "biome.grasslands",
            Self::FlowerMeadow => "biome.flower_meadow",
            Self::RollingSteppe => "biome.rolling_steppe",
            Self::FertileValley => "biome.fertile_valley",
            Self::LavenderValley => "biome.lavender_valley",
            Self::SakuraGrove => "biome.sakura_grove",
            Self::WillowMarsh => "biome.willow_marsh",
            Self::MangroveSwamp => "biome.mangrove_swamp",
            Self::OrchidSwamp => "biome.orchid_swamp",
            Self::VerdantWoods => "biome.verdant_woods",
            Self::AutumnForest => "biome.autumn_forest",
            Self::MistyWoods => "biome.misty_woods",
            Self::PineHighlands => "biome.pine_highlands",
            Self::BleakTaiga => "biome.bleak_taiga",
            Self::ShieldForest => "biome.shield_forest",
            Self::DenseRainforest => "biome.dense_rainforest",
            Self::BambooGrove => "biome.bamboo_grove",
            Self::JungleWetlands => "biome.jungle_wetlands",
            Self::AlpinePeaks => "biome.alpine_peaks",
            Self::JaggedPeaks => "biome.jagged_peaks",
            Self::MonolithCliffs => "biome.monolith_cliffs",
            Self::EmeraldFoothills => "biome.emerald_foothills",
            Self::GranitePlateau => "biome.granite_plateau",
            Self::MistyCrags => "biome.misty_crags",
            Self::AmethystCanyon => "biome.amethyst_canyon",
            Self::DeepGorge => "biome.deep_gorge",
            Self::WindyPass => "biome.windy_pass",
            Self::DuneSea => "biome.dune_sea",
            Self::AncientSands => "biome.ancient_sands",
            Self::DesertOasis => "biome.desert_oasis",
            Self::AcaciaPlains => "biome.acacia_plains",
            Self::FracturedSavanna => "biome.fractured_savanna",
            Self::SavannaBadlands => "biome.savanna_badlands",
            Self::RedrockCanyon => "biome.redrock_canyon",
            Self::PaintedMesas => "biome.painted_mesas",
            Self::DesertSpires => "biome.desert_spires",
            Self::SnowyPlains => "biome.snowy_plains",
            Self::ColdShrubland => "biome.cold_shrubland",
            Self::PermafrostSteppe => "biome.permafrost_steppe",
            Self::GlacialChasm => "biome.glacial_chasm",
            Self::IceSheet => "biome.ice_sheet",
            Self::FrozenCliffs => "biome.frozen_cliffs",
            Self::WintryForest => "biome.wintry_forest",
            Self::FrostPinewoods => "biome.frost_pinewoods",
            Self::IceboundRidge => "biome.icebound_ridge",
            Self::ThermalCaves => "biome.thermal_caves",
            Self::UndergroundJungle => "biome.underground_jungle",
            Self::IceCaves => "biome.ice_caves",
            Self::MushroomCaves => "biome.mushroom_caves",
        }
    }

    pub fn config(self) -> BiomeConfig {
        let name = self.name();
        match self {
            // I. Islands
            Self::TropicalIsland => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 4,
                base_height_offset: 4.0,
                amplitude_multiplier: 0.85,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::SandstoneCommon,
            },
            Self::TemperateIsland => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 8.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::TundraIsland => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatTundraMoss,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 3,
                base_height_offset: 6.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::CobbledAndesite,
            },
            Self::FrozenIsland => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltSnow,
                subsoil_material: Voxel::FrostIcePacked,
                subsoil_depth: 4,
                base_height_offset: 7.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::FrostBlueIce,
            },
            Self::RockyIsland => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::AndesiteGrass,
                subsoil_material: Voxel::GravelDirty,
                subsoil_depth: 3,
                base_height_offset: 14.0,
                amplitude_multiplier: 1.4,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::CoralIsland => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SandWhite,
                subsoil_material: Voxel::Coralstone,
                subsoil_depth: 4,
                base_height_offset: 2.5,
                amplitude_multiplier: 0.65,
                primary_stone: Voxel::SandstoneWhite,
                cliff_material: Voxel::Reefstone,
            },

            // II. Aquatic
            Self::WarmOcean => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::ClayBlue,
                subsoil_depth: 4,
                base_height_offset: -32.0,
                amplitude_multiplier: 0.7,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::Argillite,
            },
            Self::ColdOcean => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::GravelSandy,
                subsoil_material: Voxel::ClayBlue,
                subsoil_depth: 4,
                base_height_offset: -34.0,
                amplitude_multiplier: 0.65,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::Argillite,
            },
            Self::OceanTrench => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::Basalt,
                subsoil_material: Voxel::Gabbro,
                subsoil_depth: 5,
                base_height_offset: -54.0,
                amplitude_multiplier: 1.4,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::CoralReefs => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::Reefstone,
                subsoil_material: Voxel::Coralstone,
                subsoil_depth: 3,
                base_height_offset: -11.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::SandstoneWhite,
                cliff_material: Voxel::Reefstone,
            },
            Self::SeagrassMeadows => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SeagrassSand,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 4,
                base_height_offset: -13.0,
                amplitude_multiplier: 0.55,
                primary_stone: Voxel::Limestone,
                cliff_material: Voxel::SandstoneCommon,
            },
            Self::KelpForests => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::GravelSandy,
                subsoil_material: Voxel::SiltMud,
                subsoil_depth: 4,
                base_height_offset: -18.0,
                amplitude_multiplier: 0.65,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::CobbledAndesite,
            },
            Self::Rapids => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::GravelDirty,
                subsoil_material: Voxel::CobbledAndesite,
                subsoil_depth: 3,
                base_height_offset: -4.0,
                amplitude_multiplier: 0.7,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::CobbledAndesite,
            },
            Self::CalmLake => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltMud,
                subsoil_material: Voxel::ClayBlue,
                subsoil_depth: 4,
                base_height_offset: -8.0,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Limestone,
                cliff_material: Voxel::CobbledLimestone,
            },
            Self::Estuary => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatMud,
                subsoil_material: Voxel::SiltMud,
                subsoil_depth: 4,
                base_height_offset: -5.0,
                amplitude_multiplier: 0.45,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::SiltMud,
            },

            // III. Coastal
            Self::SandyShores => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 4,
                base_height_offset: 1.5,
                amplitude_multiplier: 0.45,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::SandstoneCommon,
            },
            Self::GravelBeach => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::GravelSandy,
                subsoil_material: Voxel::GravelDirty,
                subsoil_depth: 4,
                base_height_offset: 1.5,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::DuneBeach => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SandDune,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 5,
                base_height_offset: 3.0,
                amplitude_multiplier: 0.75,
                primary_stone: Voxel::SandstoneDune,
                cliff_material: Voxel::SandstoneCommon,
            },
            Self::CraggyBeach => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::GravelDirty,
                subsoil_material: Voxel::CobbledBasalt,
                subsoil_depth: 3,
                base_height_offset: 4.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::TidePools => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::MossyBasalt,
                subsoil_material: Voxel::Basalt,
                subsoil_depth: 3,
                base_height_offset: 1.0,
                amplitude_multiplier: 0.55,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::CoastalCliffs => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::ChalkGrass,
                subsoil_material: Voxel::SiltDirt,
                subsoil_depth: 3,
                base_height_offset: 38.0,
                amplitude_multiplier: 1.9,
                primary_stone: Voxel::Chalk,
                cliff_material: Voxel::CobbledChalk,
            },
            Self::Mudflats => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltMud,
                subsoil_material: Voxel::ClayBlue,
                subsoil_depth: 4,
                base_height_offset: 1.0,
                amplitude_multiplier: 0.35,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::SiltMud,
            },
            Self::SwampyShore => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatGrass,
                subsoil_material: Voxel::PeatMud,
                subsoil_depth: 4,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.45,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::PeatMud,
            },
            Self::ReedyShore => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltGrass,
                subsoil_material: Voxel::SiltMud,
                subsoil_depth: 4,
                base_height_offset: 2.5,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Limestone,
                cliff_material: Voxel::SiltDirt,
            },

            // IV. Special & Dark Fantasy
            Self::Caldera => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::GravelAshy,
                subsoil_material: Voxel::CobbledMagma,
                subsoil_depth: 4,
                base_height_offset: 32.0,
                amplitude_multiplier: 2.0,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::VolcanicPeaks => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::Basalt,
                subsoil_material: Voxel::GravelAshy,
                subsoil_depth: 4,
                base_height_offset: 80.0,
                amplitude_multiplier: 2.5,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::VolcanicJungle => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::BasaltGrass,
                subsoil_material: Voxel::LoamRooted,
                subsoil_depth: 4,
                base_height_offset: 24.0,
                amplitude_multiplier: 1.5,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::MossyBasalt,
            },
            Self::WitheredWoods => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatGrass,
                subsoil_material: Voxel::PeatCoarse,
                subsoil_depth: 4,
                base_height_offset: 18.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Slate,
                cliff_material: Voxel::CobbledSlate,
            },
            Self::AshenWastes => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::GravelAshy,
                subsoil_material: Voxel::PeatCoarse,
                subsoil_depth: 4,
                base_height_offset: 16.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::ObsidianMire => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatMud,
                subsoil_material: Voxel::Obsidian,
                subsoil_depth: 4,
                base_height_offset: 4.0,
                amplitude_multiplier: 0.6,
                primary_stone: Voxel::Obsidian,
                cliff_material: Voxel::Basalt,
            },
            Self::AcidPools => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::ClayYellow,
                subsoil_material: Voxel::CalicheCracked,
                subsoil_depth: 4,
                base_height_offset: 12.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Dolomite,
                cliff_material: Voxel::ClayYellow,
            },
            Self::SulfurVents => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::CalicheCracked,
                subsoil_material: Voxel::ClayYellow,
                subsoil_depth: 4,
                base_height_offset: 20.0,
                amplitude_multiplier: 1.35,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::ClayYellow,
            },
            Self::ScorchedFlats => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LateriteHardpan,
                subsoil_material: Voxel::GravelAshy,
                subsoil_depth: 4,
                base_height_offset: 16.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::LateriteCoarse,
            },

            // V. Lowlands & Wetlands
            Self::Grasslands => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 14.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Limestone,
                cliff_material: Voxel::CobbledLimestone,
            },
            Self::FlowerMeadow => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 15.0,
                amplitude_multiplier: 0.95,
                primary_stone: Voxel::Limestone,
                cliff_material: Voxel::CobbledLimestone,
            },
            Self::RollingSteppe => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltGrass,
                subsoil_material: Voxel::SiltCoarse,
                subsoil_depth: 4,
                base_height_offset: 18.0,
                amplitude_multiplier: 1.05,
                primary_stone: Voxel::Dolomite,
                cliff_material: Voxel::CobbledDolomite,
            },
            Self::FertileValley => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 12.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::LavenderValley => BiomeConfig {
                biome_type: self,
                name,
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
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamMulch,
                subsoil_depth: 4,
                base_height_offset: 20.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::WillowMarsh => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatGrass,
                subsoil_material: Voxel::PeatMud,
                subsoil_depth: 4,
                base_height_offset: 2.5,
                amplitude_multiplier: 0.45,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::PeatMud,
            },
            Self::MangroveSwamp => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatMud,
                subsoil_material: Voxel::LoamMud,
                subsoil_depth: 5,
                base_height_offset: 1.5,
                amplitude_multiplier: 0.4,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::PeatMud,
            },
            Self::OrchidSwamp => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatMud,
                subsoil_material: Voxel::PeatDirt,
                subsoil_depth: 4,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.4,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::PeatMud,
            },

            // VI. Forests
            Self::VerdantWoods => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 20.0,
                amplitude_multiplier: 1.15,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::AutumnForest => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamMulch,
                subsoil_depth: 4,
                base_height_offset: 22.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::MistyWoods => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::MossForest,
                subsoil_material: Voxel::LoamMulch,
                subsoil_depth: 5,
                base_height_offset: 36.0,
                amplitude_multiplier: 1.5,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::MossyGranite,
            },
            Self::PineHighlands => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 48.0,
                amplitude_multiplier: 1.75,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::BleakTaiga => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PermafrostGrass,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 4,
                base_height_offset: 28.0,
                amplitude_multiplier: 1.3,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::PermafrostIcy,
            },
            Self::ShieldForest => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatGrass,
                subsoil_material: Voxel::PeatDirt,
                subsoil_depth: 3,
                base_height_offset: 25.0,
                amplitude_multiplier: 1.4,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::DenseRainforest => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamRooted,
                subsoil_depth: 5,
                base_height_offset: 22.0,
                amplitude_multiplier: 1.5,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::LoamMud,
            },
            Self::BambooGrove => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamMulch,
                subsoil_depth: 4,
                base_height_offset: 24.0,
                amplitude_multiplier: 1.35,
                primary_stone: Voxel::Limestone,
                cliff_material: Voxel::MossyLimestone,
            },
            Self::JungleWetlands => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamMud,
                subsoil_depth: 5,
                base_height_offset: 6.0,
                amplitude_multiplier: 0.7,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::LoamMud,
            },

            // VII. Mountains & Relief
            Self::AlpinePeaks => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltSnow,
                subsoil_material: Voxel::GravelSnowy,
                subsoil_depth: 3,
                base_height_offset: 75.0,
                amplitude_multiplier: 2.4,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::JaggedPeaks => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::AndesiteGrass,
                subsoil_material: Voxel::GravelDirty,
                subsoil_depth: 3,
                base_height_offset: 72.0,
                amplitude_multiplier: 2.35,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::CobbledAndesite,
            },
            Self::MonolithCliffs => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::GraniteGrass,
                subsoil_material: Voxel::LoamMulch,
                subsoil_depth: 3,
                base_height_offset: 60.0,
                amplitude_multiplier: 2.4,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::EmeraldFoothills => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 3,
                base_height_offset: 56.0,
                amplitude_multiplier: 2.0,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::GranitePlateau => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::GraniteGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 3,
                base_height_offset: 52.0,
                amplitude_multiplier: 1.85,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::MistyCrags => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::Slate,
                subsoil_material: Voxel::GravelDirty,
                subsoil_depth: 3,
                base_height_offset: 58.0,
                amplitude_multiplier: 2.1,
                primary_stone: Voxel::Slate,
                cliff_material: Voxel::CobbledSlate,
            },
            Self::AmethystCanyon => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 35.0,
                amplitude_multiplier: 2.0,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::Basalt,
            },
            Self::DeepGorge => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::BasaltGrass,
                subsoil_material: Voxel::GravelDirty,
                subsoil_depth: 3,
                base_height_offset: 32.0,
                amplitude_multiplier: 2.1,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::CobbledBasalt,
            },
            Self::WindyPass => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltGrass,
                subsoil_material: Voxel::GravelDirty,
                subsoil_depth: 3,
                base_height_offset: 42.0,
                amplitude_multiplier: 1.8,
                primary_stone: Voxel::Dolomite,
                cliff_material: Voxel::CobbledDolomite,
            },

            // VIII. Arid
            Self::DuneSea => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SandDune,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 8,
                base_height_offset: 24.0,
                amplitude_multiplier: 1.8,
                primary_stone: Voxel::SandstoneDune,
                cliff_material: Voxel::SandstoneCommon,
            },
            Self::AncientSands => BiomeConfig {
                biome_type: self,
                name,
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
                name,
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamMud,
                subsoil_depth: 3,
                base_height_offset: 8.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::SandCommon,
            },
            Self::AcaciaPlains => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::CalicheGrass,
                subsoil_material: Voxel::CalicheDirt,
                subsoil_depth: 4,
                base_height_offset: 18.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::CalicheCoarse,
            },
            Self::FracturedSavanna => BiomeConfig {
                biome_type: self,
                name,
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
                name,
                surface_material: Voxel::LateriteGrass,
                subsoil_material: Voxel::CalicheCoarse,
                subsoil_depth: 4,
                base_height_offset: 20.0,
                amplitude_multiplier: 1.3,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::LateriteHardpan,
            },
            Self::RedrockCanyon => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SandRed,
                subsoil_material: Voxel::ClayRed,
                subsoil_depth: 6,
                base_height_offset: 40.0,
                amplitude_multiplier: 2.1,
                primary_stone: Voxel::SandstoneRed,
                cliff_material: Voxel::ClayRed,
            },
            Self::PaintedMesas => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::ClayRed,
                subsoil_material: Voxel::ClayYellow,
                subsoil_depth: 5,
                base_height_offset: 50.0,
                amplitude_multiplier: 1.9,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::ClayWhite,
            },
            Self::DesertSpires => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SandRed,
                subsoil_material: Voxel::SandstoneRed,
                subsoil_depth: 5,
                base_height_offset: 28.0,
                amplitude_multiplier: 1.85,
                primary_stone: Voxel::SandstoneRed,
                cliff_material: Voxel::SandstoneRed,
            },

            // IX. Frozen & Polar (Featuring the new Tundra Moss soils!)
            Self::SnowyPlains => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltSnow,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 4,
                base_height_offset: 16.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::PermafrostIcy,
            },
            Self::ColdShrubland => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::LoamTundraMoss,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 3,
                base_height_offset: 18.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::PermafrostSteppe => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltTundraMoss,
                subsoil_material: Voxel::PermafrostCoarse,
                subsoil_depth: 4,
                base_height_offset: 20.0,
                amplitude_multiplier: 1.05,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::PermafrostIcy,
            },
            Self::GlacialChasm => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::FrostIcePacked,
                subsoil_material: Voxel::FrostBlueIce,
                subsoil_depth: 6,
                base_height_offset: 35.0,
                amplitude_multiplier: 2.1,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::FrostBlueIce,
            },
            Self::IceSheet => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SnowBlock,
                subsoil_material: Voxel::FrostIcePacked,
                subsoil_depth: 6,
                base_height_offset: 22.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::FrostIcePacked,
            },
            Self::FrozenCliffs => BiomeConfig {
                biome_type: self,
                name,
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
                name,
                surface_material: Voxel::LoamSnow,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 4,
                base_height_offset: 24.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::FrostPinewoods => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::PeatSnow,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 4,
                base_height_offset: 28.0,
                amplitude_multiplier: 1.35,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::PermafrostIcy,
            },
            Self::IceboundRidge => BiomeConfig {
                biome_type: self,
                name,
                surface_material: Voxel::SiltSnow,
                subsoil_material: Voxel::GravelSnowy,
                subsoil_depth: 4,
                base_height_offset: 44.0,
                amplitude_multiplier: 1.75,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::FrostIcePacked,
            },

            // X. Subterranean (untouched)
            Self::ThermalCaves => BiomeConfig {
                biome_type: self,
                name,
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
                name,
                surface_material: Voxel::MossForest,
                subsoil_material: Voxel::LoamRooted,
                subsoil_depth: 3,
                base_height_offset: -40.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::ClayBlue,
            },
            Self::IceCaves => BiomeConfig {
                biome_type: self,
                name,
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
                name,
                surface_material: Voxel::PeatMud,
                subsoil_material: Voxel::LoamRooted,
                subsoil_depth: 4,
                base_height_offset: -45.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::PeatMud,
            },
        }
    }

    /// Grass tint color calibrated for each micro-biome.
    pub fn grass_color(self) -> [f32; 4] {
        match self {
            // I. Islands
            Self::TropicalIsland | Self::CoralIsland => [0.24, 0.94, 0.36, 1.0],
            Self::TemperateIsland => [0.38, 0.88, 0.38, 1.0],
            Self::TundraIsland => [0.54, 0.68, 0.46, 1.0],
            Self::FrozenIsland => [0.46, 0.72, 0.78, 1.0],
            Self::RockyIsland => [0.42, 0.68, 0.44, 1.0],

            // II. Aquatic
            Self::WarmOcean | Self::CoralReefs => [0.22, 0.92, 0.58, 1.0],
            Self::SeagrassMeadows => [0.28, 0.88, 0.46, 1.0],
            Self::ColdOcean | Self::KelpForests => [0.38, 0.72, 0.60, 1.0],
            Self::OceanTrench => [0.20, 0.42, 0.52, 1.0],
            Self::Rapids | Self::CalmLake => [0.38, 0.88, 0.42, 1.0],
            Self::Estuary => [0.42, 0.78, 0.38, 1.0],

            // III. Coastal
            Self::SandyShores | Self::DuneBeach => [0.64, 0.84, 0.40, 1.0],
            Self::GravelBeach | Self::CraggyBeach | Self::TidePools => [0.52, 0.78, 0.44, 1.0],
            Self::CoastalCliffs => [0.58, 0.86, 0.46, 1.0],
            Self::Mudflats | Self::SwampyShore => [0.40, 0.76, 0.36, 1.0],
            Self::ReedyShore => [0.48, 0.84, 0.40, 1.0],

            // IV. Special & Dark Fantasy
            Self::Caldera | Self::VolcanicPeaks => [0.35, 0.44, 0.30, 1.0],
            Self::VolcanicJungle => [0.22, 0.78, 0.28, 1.0],
            Self::WitheredWoods => [0.44, 0.36, 0.52, 1.0],
            Self::AshenWastes | Self::ObsidianMire => [0.36, 0.32, 0.42, 1.0],
            Self::AcidPools | Self::SulfurVents => [0.68, 0.74, 0.24, 1.0],
            Self::ScorchedFlats => [0.58, 0.52, 0.28, 1.0],

            // V. Lowlands & Wetlands
            Self::Grasslands => [0.42, 0.90, 0.38, 1.0],
            Self::FlowerMeadow => [0.36, 0.96, 0.44, 1.0],
            Self::RollingSteppe => [0.58, 0.82, 0.36, 1.0],
            Self::FertileValley => [0.35, 0.95, 0.42, 1.0],
            Self::LavenderValley => [0.56, 0.82, 0.46, 1.0],
            Self::SakuraGrove => [0.60, 0.92, 0.65, 1.0],
            Self::WillowMarsh | Self::MangroveSwamp | Self::OrchidSwamp => [0.38, 0.78, 0.34, 1.0],

            // VI. Forests
            Self::VerdantWoods => [0.32, 0.88, 0.36, 1.0],
            Self::AutumnForest => [0.68, 0.74, 0.32, 1.0],
            Self::MistyWoods => [0.22, 0.88, 0.48, 1.0],
            Self::PineHighlands => [0.32, 0.80, 0.44, 1.0],
            Self::BleakTaiga => [0.38, 0.72, 0.56, 1.0],
            Self::ShieldForest => [0.40, 0.75, 0.50, 1.0],
            Self::DenseRainforest | Self::JungleWetlands => [0.15, 0.92, 0.25, 1.0],
            Self::BambooGrove => [0.26, 0.94, 0.32, 1.0],

            // VII. Mountains & Relief
            Self::AlpinePeaks => [0.45, 0.75, 0.55, 1.0],
            Self::JaggedPeaks => [0.48, 0.70, 0.45, 1.0],
            Self::MonolithCliffs => [0.42, 0.86, 0.38, 1.0],
            Self::EmeraldFoothills => [0.25, 0.94, 0.36, 1.0],
            Self::GranitePlateau => [0.50, 0.82, 0.40, 1.0],
            Self::MistyCrags | Self::DeepGorge | Self::WindyPass => [0.44, 0.72, 0.46, 1.0],
            Self::AmethystCanyon => [0.45, 0.75, 0.50, 1.0],

            // VIII. Arid
            Self::DuneSea => [0.82, 0.72, 0.28, 1.0],
            Self::AncientSands => [0.78, 0.70, 0.30, 1.0],
            Self::DesertOasis => [0.40, 0.96, 0.35, 1.0],
            Self::AcaciaPlains | Self::FracturedSavanna => [0.70, 0.82, 0.28, 1.0],
            Self::SavannaBadlands => [0.72, 0.78, 0.25, 1.0],
            Self::RedrockCanyon => [0.75, 0.60, 0.25, 1.0],
            Self::PaintedMesas => [0.70, 0.65, 0.35, 1.0],
            Self::DesertSpires => [0.80, 0.75, 0.35, 1.0],

            // IX. Frozen & Polar
            Self::SnowyPlains | Self::IceboundRidge => [0.46, 0.74, 0.62, 1.0],
            Self::ColdShrubland | Self::PermafrostSteppe => [0.55, 0.70, 0.50, 1.0],
            Self::GlacialChasm | Self::IceSheet => [0.50, 0.80, 0.90, 1.0],
            Self::FrozenCliffs => [0.45, 0.70, 0.75, 1.0],
            Self::WintryForest | Self::FrostPinewoods => [0.42, 0.72, 0.65, 1.0],

            // X. Subterranean
            Self::ThermalCaves => [0.40, 0.35, 0.30, 1.0],
            Self::UndergroundJungle => [0.20, 0.95, 0.40, 1.0],
            Self::IceCaves => [0.55, 0.85, 0.95, 1.0],
            Self::MushroomCaves => [0.45, 0.65, 0.50, 1.0],
        }
    }

    /// Water tint color calibrated for each micro-biome.
    pub fn water_color(self) -> [f32; 4] {
        match self {
            Self::WarmOcean | Self::TropicalIsland | Self::SeagrassMeadows => {
                [0.10, 0.85, 0.88, 1.0]
            }
            Self::CoralReefs | Self::CoralIsland => [0.08, 0.92, 0.92, 1.0],
            Self::ColdOcean | Self::KelpForests | Self::TundraIsland => [0.15, 0.55, 0.82, 1.0],
            Self::OceanTrench => [0.05, 0.16, 0.42, 1.0],
            Self::WillowMarsh
            | Self::MangroveSwamp
            | Self::OrchidSwamp
            | Self::SwampyShore
            | Self::Estuary => [0.20, 0.48, 0.36, 1.0],
            Self::WitheredWoods | Self::AshenWastes | Self::ObsidianMire => [0.26, 0.18, 0.38, 1.0],
            Self::AcidPools | Self::SulfurVents | Self::ScorchedFlats => [0.58, 0.72, 0.16, 1.0],
            Self::GlacialChasm
            | Self::IceSheet
            | Self::FrozenCliffs
            | Self::FrozenIsland
            | Self::IceCaves => [0.30, 0.75, 0.98, 1.0],
            Self::VolcanicPeaks | Self::Caldera | Self::ThermalCaves => [0.65, 0.25, 0.15, 1.0],
            Self::FlowerMeadow | Self::FertileValley | Self::LavenderValley | Self::SakuraGrove => {
                [0.22, 0.70, 0.95, 1.0]
            }
            Self::DenseRainforest
            | Self::BambooGrove
            | Self::JungleWetlands
            | Self::VolcanicJungle
            | Self::UndergroundJungle
            | Self::MushroomCaves => [0.18, 0.75, 0.70, 1.0],
            _ => [0.22, 0.60, 0.90, 1.0],
        }
    }

    /// Foliage tint color for leaves.
    pub fn foliage_color(self, voxel: Voxel) -> [f32; 4] {
        match voxel {
            Voxel::PalmLeaves => return [0.28, 0.88, 0.34, 1.0],
            Voxel::PineLeaves | Voxel::YewLeaves | Voxel::RedwoodLeaves => {
                return [0.30, 0.72, 0.44, 1.0];
            }
            Voxel::MahoganyLeaves => return [0.16, 0.90, 0.24, 1.0],
            Voxel::AcaciaLeaves => return [0.64, 0.76, 0.28, 1.0],
            _ => {}
        }
        match self {
            Self::SakuraGrove => [0.95, 0.65, 0.78, 1.0],
            Self::LavenderValley => [0.70, 0.55, 0.85, 1.0],
            Self::AutumnForest => [0.88, 0.54, 0.22, 1.0],
            Self::DenseRainforest | Self::BambooGrove | Self::JungleWetlands => {
                [0.12, 0.95, 0.22, 1.0]
            }
            Self::BleakTaiga
            | Self::ShieldForest
            | Self::WintryForest
            | Self::FrostPinewoods
            | Self::PineHighlands => [0.35, 0.75, 0.55, 1.0],
            Self::VolcanicPeaks
            | Self::WitheredWoods
            | Self::AshenWastes
            | Self::ObsidianMire
            | Self::ScorchedFlats => [0.46, 0.40, 0.36, 1.0],
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

/// A 6-dimensional climate sample storing multi-noise worldgen parameters.
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

/// Target coordinate in 6D Multi-Noise space (for Sandbox Mode procedural generation).
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
    // I. Islands & Coastal Archipelagos
    BiomePoint {
        biome: BiomeType::TropicalIsland,
        c: -0.10,
        e: 0.35,
        r: -0.20,
        t: 0.70,
        h: 0.45,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::TemperateIsland,
        c: -0.10,
        e: 0.30,
        r: -0.15,
        t: 0.10,
        h: 0.25,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::TundraIsland,
        c: -0.10,
        e: 0.30,
        r: -0.15,
        t: -0.50,
        h: -0.10,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::FrozenIsland,
        c: -0.12,
        e: 0.25,
        r: -0.10,
        t: -0.80,
        h: 0.15,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::RockyIsland,
        c: -0.12,
        e: -0.30,
        r: 0.30,
        t: -0.10,
        h: 0.05,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::CoralIsland,
        c: -0.14,
        e: 0.40,
        r: -0.30,
        t: 0.80,
        h: 0.55,
        d: 0.0,
    },
    // II. Aquatic
    BiomePoint {
        biome: BiomeType::WarmOcean,
        c: -0.45,
        e: 0.00,
        r: 0.00,
        t: 0.45,
        h: 0.20,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::ColdOcean,
        c: -0.45,
        e: 0.00,
        r: 0.00,
        t: -0.60,
        h: 0.10,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::OceanTrench,
        c: -0.80,
        e: -0.30,
        r: 0.20,
        t: -0.10,
        h: 0.10,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::CoralReefs,
        c: -0.22,
        e: 0.25,
        r: -0.20,
        t: 0.72,
        h: 0.45,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::SeagrassMeadows,
        c: -0.24,
        e: 0.30,
        r: -0.25,
        t: 0.25,
        h: 0.30,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::KelpForests,
        c: -0.26,
        e: 0.15,
        r: -0.15,
        t: -0.30,
        h: 0.20,
        d: 0.0,
    },
    // III. Coastal
    BiomePoint {
        biome: BiomeType::SandyShores,
        c: -0.05,
        e: 0.40,
        r: -0.30,
        t: 0.30,
        h: 0.10,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::GravelBeach,
        c: -0.06,
        e: 0.38,
        r: -0.28,
        t: -0.30,
        h: -0.05,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::DuneBeach,
        c: -0.04,
        e: 0.42,
        r: -0.25,
        t: 0.65,
        h: -0.45,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::CraggyBeach,
        c: -0.04,
        e: -0.25,
        r: 0.20,
        t: -0.15,
        h: 0.05,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::TidePools,
        c: -0.06,
        e: -0.20,
        r: 0.10,
        t: 0.15,
        h: 0.35,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::CoastalCliffs,
        c: 0.04,
        e: -0.45,
        r: 0.30,
        t: 0.05,
        h: 0.20,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::Mudflats,
        c: -0.03,
        e: 0.50,
        r: -0.40,
        t: 0.20,
        h: 0.50,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::SwampyShore,
        c: -0.02,
        e: 0.52,
        r: -0.45,
        t: 0.45,
        h: 0.75,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::ReedyShore,
        c: -0.02,
        e: 0.48,
        r: -0.35,
        t: 0.10,
        h: 0.45,
        d: 0.0,
    },
    // IV. Special & Dark Fantasy
    BiomePoint {
        biome: BiomeType::Caldera,
        c: 0.45,
        e: -0.20,
        r: -0.75,
        t: 0.85,
        h: -0.10,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::VolcanicPeaks,
        c: 0.42,
        e: -0.48,
        r: 0.75,
        t: 0.80,
        h: -0.45,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::VolcanicJungle,
        c: 0.36,
        e: -0.15,
        r: 0.35,
        t: 0.78,
        h: 0.55,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::WitheredWoods,
        c: 0.32,
        e: 0.15,
        r: 0.10,
        t: 0.25,
        h: -0.20,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::AshenWastes,
        c: 0.36,
        e: 0.25,
        r: 0.05,
        t: 0.75,
        h: -0.40,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::ObsidianMire,
        c: 0.28,
        e: 0.45,
        r: -0.45,
        t: 0.35,
        h: 0.55,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::AcidPools,
        c: 0.38,
        e: 0.35,
        r: -0.50,
        t: 0.70,
        h: -0.25,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::SulfurVents,
        c: 0.40,
        e: -0.10,
        r: 0.30,
        t: 0.76,
        h: -0.35,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::ScorchedFlats,
        c: 0.35,
        e: 0.30,
        r: -0.10,
        t: 0.82,
        h: -0.50,
        d: 0.0,
    },
    // V. Lowlands & Wetlands
    BiomePoint {
        biome: BiomeType::Grasslands,
        c: 0.14,
        e: 0.32,
        r: -0.15,
        t: 0.08,
        h: 0.10,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::FlowerMeadow,
        c: 0.12,
        e: 0.35,
        r: -0.20,
        t: 0.12,
        h: 0.32,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::RollingSteppe,
        c: 0.20,
        e: 0.28,
        r: -0.05,
        t: 0.22,
        h: -0.20,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::FertileValley,
        c: 0.10,
        e: 0.30,
        r: -0.25,
        t: 0.06,
        h: 0.22,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::LavenderValley,
        c: 0.08,
        e: 0.30,
        r: -0.20,
        t: -0.05,
        h: 0.25,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::SakuraGrove,
        c: 0.15,
        e: 0.20,
        r: 0.05,
        t: 0.15,
        h: 0.40,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::WillowMarsh,
        c: 0.04,
        e: 0.52,
        r: -0.45,
        t: 0.18,
        h: 0.78,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::MangroveSwamp,
        c: 0.02,
        e: 0.55,
        r: -0.50,
        t: 0.58,
        h: 0.85,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::OrchidSwamp,
        c: 0.03,
        e: 0.55,
        r: -0.50,
        t: 0.42,
        h: 0.82,
        d: 0.0,
    },
    // VI. Forests
    BiomePoint {
        biome: BiomeType::VerdantWoods,
        c: 0.16,
        e: 0.15,
        r: 0.05,
        t: 0.08,
        h: 0.35,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::AutumnForest,
        c: 0.18,
        e: 0.12,
        r: 0.10,
        t: -0.08,
        h: 0.30,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::MistyWoods,
        c: 0.25,
        e: -0.15,
        r: 0.30,
        t: 0.10,
        h: 0.70,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::PineHighlands,
        c: 0.28,
        e: -0.30,
        r: 0.40,
        t: -0.35,
        h: 0.20,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::BleakTaiga,
        c: 0.15,
        e: 0.15,
        r: 0.05,
        t: -0.55,
        h: 0.25,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::ShieldForest,
        c: 0.18,
        e: 0.10,
        r: 0.10,
        t: -0.20,
        h: 0.15,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::DenseRainforest,
        c: 0.14,
        e: 0.15,
        r: -0.10,
        t: 0.68,
        h: 0.78,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::BambooGrove,
        c: 0.18,
        e: 0.05,
        r: 0.15,
        t: 0.55,
        h: 0.72,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::JungleWetlands,
        c: 0.06,
        e: 0.42,
        r: -0.35,
        t: 0.65,
        h: 0.88,
        d: 0.0,
    },
    // VII. Mountains & Relief
    BiomePoint {
        biome: BiomeType::AlpinePeaks,
        c: 0.35,
        e: -0.55,
        r: 0.60,
        t: -0.65,
        h: 0.10,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::JaggedPeaks,
        c: 0.36,
        e: -0.50,
        r: 0.58,
        t: -0.20,
        h: -0.15,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::MonolithCliffs,
        c: 0.30,
        e: -0.50,
        r: 0.55,
        t: 0.15,
        h: 0.35,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::EmeraldFoothills,
        c: 0.30,
        e: -0.45,
        r: 0.55,
        t: -0.05,
        h: 0.55,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::GranitePlateau,
        c: 0.26,
        e: -0.40,
        r: 0.42,
        t: 0.05,
        h: 0.22,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::MistyCrags,
        c: 0.32,
        e: -0.42,
        r: 0.48,
        t: -0.15,
        h: 0.48,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::AmethystCanyon,
        c: 0.25,
        e: -0.10,
        r: -0.55,
        t: 0.15,
        h: 0.15,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::DeepGorge,
        c: 0.28,
        e: -0.25,
        r: -0.60,
        t: -0.05,
        h: 0.10,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::WindyPass,
        c: 0.30,
        e: -0.30,
        r: -0.40,
        t: -0.25,
        h: -0.10,
        d: 0.0,
    },
    // VIII. Arid
    BiomePoint {
        biome: BiomeType::DuneSea,
        c: 0.38,
        e: 0.45,
        r: 0.25,
        t: 0.85,
        h: -0.80,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::AncientSands,
        c: 0.26,
        e: 0.25,
        r: -0.10,
        t: 0.68,
        h: -0.58,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::AcaciaPlains,
        c: 0.22,
        e: 0.20,
        r: -0.05,
        t: 0.52,
        h: -0.32,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::FracturedSavanna,
        c: 0.26,
        e: -0.20,
        r: 0.35,
        t: 0.55,
        h: -0.30,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::SavannaBadlands,
        c: 0.24,
        e: 0.10,
        r: 0.10,
        t: 0.55,
        h: -0.45,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::RedrockCanyon,
        c: 0.28,
        e: 0.05,
        r: 0.50,
        t: 0.65,
        h: -0.65,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::PaintedMesas,
        c: 0.32,
        e: -0.38,
        r: 0.50,
        t: 0.58,
        h: -0.55,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::DesertSpires,
        c: 0.30,
        e: -0.25,
        r: 0.50,
        t: 0.75,
        h: -0.55,
        d: 0.0,
    },
    // IX. Frozen & Polar
    BiomePoint {
        biome: BiomeType::SnowyPlains,
        c: 0.18,
        e: 0.38,
        r: -0.20,
        t: -0.68,
        h: 0.15,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::ColdShrubland,
        c: 0.20,
        e: 0.35,
        r: -0.20,
        t: -0.45,
        h: -0.25,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::PermafrostSteppe,
        c: 0.24,
        e: 0.30,
        r: -0.10,
        t: -0.55,
        h: -0.15,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::GlacialChasm,
        c: 0.35,
        e: -0.50,
        r: -0.65,
        t: -0.85,
        h: 0.30,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::IceSheet,
        c: 0.30,
        e: 0.35,
        r: -0.15,
        t: -0.88,
        h: 0.20,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::FrozenCliffs,
        c: 0.05,
        e: -0.55,
        r: 0.45,
        t: -0.72,
        h: 0.10,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::WintryForest,
        c: 0.12,
        e: 0.25,
        r: -0.05,
        t: -0.65,
        h: 0.50,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::FrostPinewoods,
        c: 0.18,
        e: 0.10,
        r: 0.15,
        t: -0.62,
        h: 0.40,
        d: 0.0,
    },
    BiomePoint {
        biome: BiomeType::IceboundRidge,
        c: 0.28,
        e: -0.35,
        r: 0.45,
        t: -0.70,
        h: 0.35,
        d: 0.0,
    },
    // X. Subterranean
    BiomePoint {
        biome: BiomeType::ThermalCaves,
        c: 0.25,
        e: 0.00,
        r: 0.00,
        t: 0.70,
        h: -0.50,
        d: -0.70,
    },
    BiomePoint {
        biome: BiomeType::UndergroundJungle,
        c: 0.15,
        e: 0.00,
        r: 0.00,
        t: 0.50,
        h: 0.80,
        d: -0.70,
    },
    BiomePoint {
        biome: BiomeType::IceCaves,
        c: 0.25,
        e: 0.00,
        r: 0.00,
        t: -0.70,
        h: 0.20,
        d: -0.75,
    },
    BiomePoint {
        biome: BiomeType::MushroomCaves,
        c: 0.10,
        e: 0.00,
        r: 0.00,
        t: 0.10,
        h: 0.65,
        d: -0.80,
    },
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
    pub fn sample_at_depth(
        &self,
        world_x: f32,
        world_z: f32,
        depth: f32,
        seed: u32,
    ) -> ClimateSample {
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

        let biome = Self::classify_biome(
            continentalness,
            erosion,
            ridges,
            temperature,
            humidity,
            depth,
        );

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
        let mut best_biome = BiomeType::Grasslands;
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

#[derive(Clone, Copy)]
struct TintCacheEntry {
    key_x: i32,
    key_z: i32,
    seed: u32,
    valid: bool,
    grass: [f32; 4],
    water: [f32; 4],
    foliage: [f32; 4],
}

impl Default for TintCacheEntry {
    fn default() -> Self {
        Self {
            key_x: i32::MIN,
            key_z: i32::MIN,
            seed: 0,
            valid: false,
            grass: [1.0; 4],
            water: [1.0; 4],
            foliage: [1.0; 4],
        }
    }
}

const TINT_CACHE_SIZE: usize = 4096;

std::thread_local! {
    static BLENDED_TINT_CACHE: std::cell::RefCell<Box<[TintCacheEntry]>> = std::cell::RefCell::new(
        vec![TintCacheEntry::default(); TINT_CACHE_SIZE].into_boxed_slice()
    );
}

#[inline]
fn compute_blended_tints_cell(cx: f32, cz: f32, seed: u32) -> ([f32; 4], [f32; 4], [f32; 4]) {
    let generator = ClimateGenerator::default();

    const R_INNER: f32 = 5.0;
    const R_DIAG: f32 = 3.5355;
    const R_OUTER: f32 = 10.0;
    const QUANTIZE_STEPS: f32 = 64.0;

    let mut grass = [0.0f32; 4];
    let mut water = [0.0f32; 4];
    let mut foliage = [0.0f32; 4];

    let mut accum = |biome: BiomeType, w: f32| {
        let g = biome.grass_color();
        let wa = biome.water_color();
        let f = biome.foliage_color(Voxel::OakLeaves);
        for c in 0..4 {
            grass[c] += g[c] * w;
            water[c] += wa[c] * w;
            foliage[c] += f[c] * w;
        }
    };

    // Center point (weight 0.152)
    let center_sample = generator.sample(cx, cz, seed);
    accum(center_sample.biome, 0.152);

    // 4 inner cardinal points evaluated in 1 SIMD x4 call
    let inner_samples = generator.sample_x4(
        Simd4f([cx + R_INNER, cx - R_INNER, cx, cx]),
        Simd4f([cz, cz, cz + R_INNER, cz - R_INNER]),
        seed,
    );
    for s in inner_samples {
        accum(s.biome, 0.092);
    }

    // 4 diagonal points evaluated in 1 SIMD x4 call
    let diag_samples = generator.sample_x4(
        Simd4f([cx + R_DIAG, cx - R_DIAG, cx + R_DIAG, cx - R_DIAG]),
        Simd4f([cz + R_DIAG, cz + R_DIAG, cz - R_DIAG, cz - R_DIAG]),
        seed,
    );
    for s in diag_samples {
        accum(s.biome, 0.092);
    }

    // 4 outer cardinal points evaluated in 1 SIMD x4 call
    let outer_samples = generator.sample_x4(
        Simd4f([cx + R_OUTER, cx - R_OUTER, cx, cx]),
        Simd4f([cz, cz, cz + R_OUTER, cz - R_OUTER]),
        seed,
    );
    for s in outer_samples {
        accum(s.biome, 0.028);
    }

    let quantize = |arr: [f32; 4]| -> [f32; 4] {
        [
            (arr[0] * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
            (arr[1] * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
            (arr[2] * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
            (arr[3] * QUANTIZE_STEPS).round() / QUANTIZE_STEPS,
        ]
    };

    (quantize(grass), quantize(water), quantize(foliage))
}

/// Smoothly blends biome colors across biome transitions using a 13-point circular Gaussian kernel
/// quantized into 64 discrete steps, accelerated by a thread-local 2x2m spatial SIMD cache.
pub fn sample_blended_biome_color(voxel: Voxel, world_x: f32, world_z: f32, seed: u32) -> [f32; 4] {
    if !voxel.is_tinted() {
        return [1.0, 1.0, 1.0, 1.0];
    }

    // Early-out for biome-invariant foliage palettes (Palms, Conifers, Mahogany, Acacia)
    match voxel {
        Voxel::PalmLeaves => return [0.28, 0.88, 0.34, 1.0],
        Voxel::PineLeaves | Voxel::YewLeaves | Voxel::RedwoodLeaves => {
            return [0.30, 0.72, 0.44, 1.0];
        }
        Voxel::MahoganyLeaves => return [0.16, 0.90, 0.24, 1.0],
        Voxel::AcaciaLeaves => return [0.64, 0.76, 0.28, 1.0],
        _ => {}
    }

    let key_x = (world_x.floor() as i32) >> 1;
    let key_z = (world_z.floor() as i32) >> 1;
    let slot_idx = (((key_x as u32).wrapping_mul(73_856_093)
        ^ (key_z as u32).wrapping_mul(19_349_663)
        ^ seed) as usize)
        & (TINT_CACHE_SIZE - 1);

    let (grass, water, foliage) = BLENDED_TINT_CACHE.with(|cell| {
        let mut cache = cell.borrow_mut();
        let entry = &mut cache[slot_idx];
        if entry.valid && entry.key_x == key_x && entry.key_z == key_z && entry.seed == seed {
            (entry.grass, entry.water, entry.foliage)
        } else {
            let cx = (key_x * 2 + 1) as f32;
            let cz = (key_z * 2 + 1) as f32;
            let (g, w, f) = compute_blended_tints_cell(cx, cz, seed);
            *entry = TintCacheEntry {
                key_x,
                key_z,
                seed,
                valid: true,
                grass: g,
                water: w,
                foliage: f,
            };
            (g, w, f)
        }
    });

    if voxel == Voxel::VegeLeafLitter {
        // Dry foliage tint: warm golden-brown / russet / olive-ochre modulated by biome foliage
        let r = (0.58 + foliage[0] * 0.24).clamp(0.45, 0.86);
        let g = (0.40 + foliage[1] * 0.20).clamp(0.32, 0.68);
        let b = (0.20 + foliage[2] * 0.14).clamp(0.16, 0.42);
        [r, g, b, 1.0]
    } else if voxel.is_grass_soil() || voxel.is_tinted_flora() {
        grass
    } else if matches!(voxel, Voxel::LiquidWater | Voxel::WaterOccupied) {
        water
    } else if voxel.is_leaves() {
        foliage
    } else {
        [1.0, 1.0, 1.0, 1.0]
    }
}
