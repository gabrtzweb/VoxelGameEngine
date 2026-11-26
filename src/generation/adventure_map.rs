use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, OnceLock};

use bevy::prelude::*;

use crate::core::noise::fractal_noise_2d;

use super::biome::{BiomeType, ClimateSample, MacroBiome};

const HEIGHTMAP_PATH: &str = "assets/maps/adventure/heightmap.png";
const MACRO_BIOMES_PATH: &str = "assets/maps/adventure/macro_biomes.png";
const MICRO_BIOMES_PATH: &str = "assets/maps/adventure/micro_biomes.png";
const CONTINENTS_PATH: &str = "assets/maps/adventure/world_continents.png";
const REGIONS_PATH: &str = "assets/maps/adventure/world_regions.png";
const LANDMASS_PATH: &str = "assets/maps/adventure/world_landmass.png";

pub const FULL_ADVENTURE_WIDTH: f32 = 12_800.0;
pub const FULL_ADVENTURE_HEIGHT: f32 = 7_200.0;
pub const ADVENTURE_SEA_LEVEL: i32 = 12;
pub const ADVENTURE_MIN_Y: f32 = -56.0;
pub const ADVENTURE_MAX_Y: f32 = 180.0;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdventureWorldScale {
    /// 50% scale (6,400 x 3,600 blocks) — faster exploration & testing scale.
    Medium,
    /// 100% scale (12,800 x 7,200 blocks) — full ~92 km^2 handcrafted world.
    #[default]
    Full,
}

impl AdventureWorldScale {
    pub const ALL: [Self; 2] = [Self::Medium, Self::Full];

    pub fn factor(self) -> f32 {
        match self {
            Self::Medium => 0.50,
            Self::Full => 1.0,
        }
    }

    pub fn dimensions(self) -> (f32, f32) {
        let f = self.factor();
        (FULL_ADVENTURE_WIDTH * f, FULL_ADVENTURE_HEIGHT * f)
    }

    pub fn loc_key(self) -> &'static str {
        match self {
            Self::Medium => "menu.main.adventure.scale.medium",
            Self::Full => "menu.main.adventure.scale.full",
        }
    }

    pub fn fallback_label(self) -> &'static str {
        match self {
            Self::Medium => "Medium (6,400 x 3,600)",
            Self::Full => "Full (12,800 x 7,200)",
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdventureSpawnChoice {
    NorthEstuary,
    #[default]
    CentralBay,
    SouthCoast,
}

impl AdventureSpawnChoice {
    pub const ALL: [Self; 3] = [Self::NorthEstuary, Self::CentralBay, Self::SouthCoast];

    /// Normalized `(u, v)` coordinates on the 16:9 (3200x1800) Adventure map.
    pub fn normalized_uv(self) -> (f32, f32) {
        match self {
            Self::NorthEstuary => (0.327, 0.290),
            Self::CentralBay => (0.410, 0.495),
            Self::SouthCoast => (0.303, 0.712),
        }
    }

    /// Normalized `(u, v)` coordinates on the 16:9 menu preview artwork.
    pub fn preview_uv(self) -> (f32, f32) {
        self.normalized_uv()
    }

    pub fn title_key(self) -> &'static str {
        match self {
            Self::NorthEstuary => "menu.main.adventure.spawn.north.title",
            Self::CentralBay => "menu.main.adventure.spawn.central.title",
            Self::SouthCoast => "menu.main.adventure.spawn.south.title",
        }
    }

    pub fn desc_key(self) -> &'static str {
        match self {
            Self::NorthEstuary => "menu.main.adventure.spawn.north.desc",
            Self::CentralBay => "menu.main.adventure.spawn.central.desc",
            Self::SouthCoast => "menu.main.adventure.spawn.south.desc",
        }
    }

    pub fn fallback_title(self) -> &'static str {
        match self {
            Self::NorthEstuary => "Northern River Estuary",
            Self::CentralBay => "Central-East Bay",
            Self::SouthCoast => "Southern Coast",
        }
    }

    pub fn fallback_desc(self) -> &'static str {
        match self {
            Self::NorthEstuary => "Cool northern river mouth near boreal woods and arctic snow.",
            Self::CentralBay => "Sheltered temperate bay with lush valleys and easy sea access.",
            Self::SouthCoast => "Rugged southern shore bordering cool forests and snowy peaks.",
        }
    }
}

/// 5 official tectonic continents / water zones from `world_continents.png`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum ContinentId {
    #[default]
    OceanWater = 0,
    WesternContinent = 1,
    CentralContinent = 2,
    EasternContinent = 3,
    ArchipelagoIslands = 4,
}

impl ContinentId {
    pub const ALL: [Self; 5] = [
        Self::OceanWater,
        Self::WesternContinent,
        Self::CentralContinent,
        Self::EasternContinent,
        Self::ArchipelagoIslands,
    ];

    pub const fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::WesternContinent,
            2 => Self::CentralContinent,
            3 => Self::EasternContinent,
            4 => Self::ArchipelagoIslands,
            _ => Self::OceanWater,
        }
    }

    pub const fn legend_rgb(self) -> [u8; 3] {
        match self {
            Self::OceanWater => [34, 86, 146],
            Self::WesternContinent => [66, 168, 92],
            Self::CentralContinent => [228, 174, 58],
            Self::EasternContinent => [214, 96, 72],
            Self::ArchipelagoIslands => [152, 92, 206],
        }
    }

    pub const fn from_rgb(rgb: [u8; 3]) -> Option<Self> {
        match rgb {
            [34, 86, 146] => Some(Self::OceanWater),
            [66, 168, 92] => Some(Self::WesternContinent),
            [228, 174, 58] => Some(Self::CentralContinent),
            [214, 96, 72] => Some(Self::EasternContinent),
            [152, 92, 206] => Some(Self::ArchipelagoIslands),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::OceanWater => "Ocean & Waters",
            Self::WesternContinent => "Western Continent",
            Self::CentralContinent => "Central Continent",
            Self::EasternContinent => "Eastern Continent",
            Self::ArchipelagoIslands => "Archipelago & Islands",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::OceanWater => "map.continent.ocean_water",
            Self::WesternContinent => "map.continent.western",
            Self::CentralContinent => "map.continent.central",
            Self::EasternContinent => "map.continent.eastern",
            Self::ArchipelagoIslands => "map.continent.archipelago",
        }
    }
}

/// 12 official climatic/geographic regions from `world_regions.png`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum WorldRegionId {
    #[default]
    OpenOcean = 0,
    CoastalSea = 1,
    RiversAndLakes = 2,
    PlainsAndValleys = 3,
    ForestRegion = 4,
    DenseForestRegion = 5,
    MountainRegion = 6,
    AridRegion = 7,
    DesertRegion = 8,
    SnowyRegion = 9,
    FrozenRegion = 10,
    SpecialRegion = 11,
}

impl WorldRegionId {
    pub const ALL: [Self; 12] = [
        Self::OpenOcean,
        Self::CoastalSea,
        Self::RiversAndLakes,
        Self::PlainsAndValleys,
        Self::ForestRegion,
        Self::DenseForestRegion,
        Self::MountainRegion,
        Self::AridRegion,
        Self::DesertRegion,
        Self::SnowyRegion,
        Self::FrozenRegion,
        Self::SpecialRegion,
    ];

    pub const fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::CoastalSea,
            2 => Self::RiversAndLakes,
            3 => Self::PlainsAndValleys,
            4 => Self::ForestRegion,
            5 => Self::DenseForestRegion,
            6 => Self::MountainRegion,
            7 => Self::AridRegion,
            8 => Self::DesertRegion,
            9 => Self::SnowyRegion,
            10 => Self::FrozenRegion,
            11 => Self::SpecialRegion,
            _ => Self::OpenOcean,
        }
    }

    pub const fn legend_rgb(self) -> [u8; 3] {
        match self {
            Self::OpenOcean => [24, 72, 128],
            Self::CoastalSea => [54, 154, 174],
            Self::RiversAndLakes => [48, 138, 198],
            Self::PlainsAndValleys => [114, 168, 72],
            Self::ForestRegion => [52, 124, 58],
            Self::DenseForestRegion => [24, 96, 40],
            Self::MountainRegion => [138, 136, 142],
            Self::AridRegion => [194, 156, 82],
            Self::DesertRegion => [236, 186, 114],
            Self::SnowyRegion => [228, 238, 248],
            Self::FrozenRegion => [73, 250, 255],
            Self::SpecialRegion => [70, 58, 60],
        }
    }

    pub const fn from_rgb(rgb: [u8; 3]) -> Option<Self> {
        match rgb {
            [24, 72, 128] => Some(Self::OpenOcean),
            [54, 154, 174] => Some(Self::CoastalSea),
            [48, 138, 198] => Some(Self::RiversAndLakes),
            [114, 168, 72] => Some(Self::PlainsAndValleys),
            [52, 124, 58] => Some(Self::ForestRegion),
            [24, 96, 40] => Some(Self::DenseForestRegion),
            [138, 136, 142] => Some(Self::MountainRegion),
            [194, 156, 82] => Some(Self::AridRegion),
            [236, 186, 114] => Some(Self::DesertRegion),
            [228, 238, 248] => Some(Self::SnowyRegion),
            [73, 250, 255] => Some(Self::FrozenRegion),
            [70, 58, 60] | [184, 113, 45] => Some(Self::SpecialRegion),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::OpenOcean => "Open Ocean",
            Self::CoastalSea => "Coastal Sea",
            Self::RiversAndLakes => "Rivers & Lakes",
            Self::PlainsAndValleys => "Plains & Valleys",
            Self::ForestRegion => "Forest Region",
            Self::DenseForestRegion => "Dense Forest Region",
            Self::MountainRegion => "Mountain Region",
            Self::AridRegion => "Arid Region",
            Self::DesertRegion => "Desert Region",
            Self::SnowyRegion => "Snowy Region",
            Self::FrozenRegion => "Frozen Region",
            Self::SpecialRegion => "Special Region",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::OpenOcean => "map.region.open_ocean",
            Self::CoastalSea => "map.region.coastal_sea",
            Self::RiversAndLakes => "map.region.rivers_and_lakes",
            Self::PlainsAndValleys => "map.region.plains_and_valleys",
            Self::ForestRegion => "map.region.forest",
            Self::DenseForestRegion => "map.region.dense_forest",
            Self::MountainRegion => "map.region.mountain",
            Self::AridRegion => "map.region.arid",
            Self::DesertRegion => "map.region.desert",
            Self::SnowyRegion => "map.region.snowy",
            Self::FrozenRegion => "map.region.frozen",
            Self::SpecialRegion => "map.region.special",
        }
    }
}

/// Overlay modes selectable on the `[M]` World Map in Adventure Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdventureOverlayMode {
    #[default]
    Terrain,
    Continents,
    Regions,
    MacroBiomes,
    MicroBiomes,
}

impl AdventureOverlayMode {
    pub fn next(self) -> Self {
        match self {
            Self::Terrain => Self::Continents,
            Self::Continents => Self::Regions,
            Self::Regions => Self::MacroBiomes,
            Self::MacroBiomes => Self::MicroBiomes,
            Self::MicroBiomes => Self::Terrain,
        }
    }

    pub fn loc_key(self) -> &'static str {
        match self {
            Self::Terrain => "map.overlay.terrain",
            Self::Continents => "map.overlay.continents",
            Self::Regions => "map.overlay.regions",
            Self::MacroBiomes => "map.overlay.macro_biomes",
            Self::MicroBiomes => "map.overlay.micro_biomes",
        }
    }

    pub fn fallback_label(self) -> &'static str {
        match self {
            Self::Terrain => "Terrain",
            Self::Continents => "Continents",
            Self::Regions => "Regions",
            Self::MacroBiomes => "Macro-Biomes",
            Self::MicroBiomes => "Micro-Biomes",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AdventureRegionInfo {
    pub continent: ContinentId,
    pub region: WorldRegionId,
    pub macro_biome: MacroBiome,
    pub micro_biome: BiomeType,
}

/// Decoded compact pixel buffers shared across scale switches so changing scale is instant.
struct RawAdventureMapBuffers {
    width: u32,
    height: u32,
    heights_u16: Vec<u16>,
    macro_u8: Vec<u8>,
    micro_u8: Vec<u8>,
    /// High 4 bits = `WorldRegionId as u8`, Low 4 bits = `ContinentId as u8`.
    continent_region_u8: Vec<u8>,
    /// `0` = water, `1` = dry land.
    landmass_u8: Vec<u8>,
    /// `i16::MIN` = dry land; otherwise exact water surface elevation `Y` for oceans,
    /// flat elevated `CalmLake` basins, foothill ponds, stepped `Rapids` reaches, and `Estuary`.
    water_level_i16: Vec<i16>,
    /// `0` = on water or far inland (`> 6 px`); `1..=6` = pixel distance on dry land to nearest
    /// inland river/lake, paired with `bank_water_y_i16` for smooth shoreline/valley grading.
    bank_dist_u8: Vec<u8>,
    bank_water_y_i16: Vec<i16>,
    /// `0` = dry land or open ocean; `1..=16` = interior BFS distance (in pixels) from dry shore
    /// inside `RiverAndLake` water bodies so lake centers deepen smoothly into natural concave basins.
    dist_in_water_u8: Vec<u8>,
    /// Pre-blurred (`0..=255`) continuous `RiverAndLake` coverage mask (3-pass Gaussian blur)
    /// that rounds off sharp 4x4-block pixel corners into silky-smooth organic shorelines.
    water_mask_u8: Vec<u8>,
}

static RAW_MAP_CACHE: OnceLock<Result<Arc<RawAdventureMapBuffers>, String>> = OnceLock::new();

#[inline]
fn rgb_dist_sq(a: [u8; 3], b: [u8; 3]) -> i32 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    dr * dr + dg * dg + db * db
}

#[inline]
fn raw_u16_to_y(h: u16) -> f32 {
    ADVENTURE_MIN_Y + (h as f32 / 65535.0) * (ADVENTURE_MAX_Y - ADVENTURE_MIN_Y)
}

#[inline]
fn is_ocean_biome(b: BiomeType) -> bool {
    matches!(
        b,
        BiomeType::WarmOcean
            | BiomeType::ColdOcean
            | BiomeType::OceanTrench
            | BiomeType::CoralReefs
            | BiomeType::SeagrassMeadows
            | BiomeType::KelpForests
    )
}

/// Computes complete hydrology for Adventure Mode:
/// 1. Retires dead-end rivers that climb too high onto mountains (`Y > 78`) back to solid terrain
///    while preserving any high-altitude river/waterfall that bridges a lake to a lower river/fjord,
///    and seeds small flat foothill spring ponds right below retired mountain headwaters.
/// 2. Only forces `Estuary` to sea level if it is 4-connected to the open ocean; enclosed coastal
///    crater/caldera lakes are promoted to elevated `CalmLake`s at their true rim elevation.
/// 3. Assigns each `CalmLake` / foothill pond a single 100% flat elevation matching its surrounding basin.
/// 4. Builds monotonically descending `Rapids` rivers quantized into long flat reaches (~52 blocks)
///    on gentle slopes and crisp 3-block pool-and-drop waterfalls on steep cliffs/escarpments.
/// 5. Precomputes dry shoreline distance (`bank_dist_u8`, `bank_water_y_i16`) AND interior water
///    distance (`dist_in_water_u8`) for smooth block-resolution curved banks and deep concave lakes.
fn compute_water_levels(
    width: usize,
    height: usize,
    heights_u16: &[u16],
    macro_u8: &mut [u8],
    micro_u8: &mut [u8],
    landmass_u8: &mut [u8],
) -> (Vec<i16>, Vec<u8>, Vec<i16>, Vec<u8>, Vec<u8>) {
    let n = width * height;
    let mut water_level = vec![i16::MIN; n];

    let calm_lake_idx = BiomeType::ALL
        .iter()
        .position(|&b| b == BiomeType::CalmLake)
        .unwrap_or(0) as u8;
    let rapids_idx = BiomeType::ALL
        .iter()
        .position(|&b| b == BiomeType::Rapids)
        .unwrap_or(0) as u8;
    let estuary_idx = BiomeType::ALL
        .iter()
        .position(|&b| b == BiomeType::Estuary)
        .unwrap_or(0) as u8;
    let foothills_idx = BiomeType::ALL
        .iter()
        .position(|&b| b == BiomeType::EmeraldFoothills)
        .unwrap_or(0) as u8;
    let river_lake_macro_idx = MacroBiome::ALL
        .iter()
        .position(|&m| m == MacroBiome::RiverAndLake)
        .unwrap_or(0) as u8;
    let highlands_macro_idx = MacroBiome::ALL
        .iter()
        .position(|&m| m == MacroBiome::Highlands)
        .unwrap_or(0) as u8;

    // 1. Retire dead-end high-altitude mountain river tips (`Y > 78.0`) back to solid mountain terrain,
    //    while PRESERVING any high-altitude river/waterfall channel that connects to a `CalmLake`
    //    (e.g. snowy plateau outlets cascading down fjord cliffs!).
    let mut retired_mountain = vec![false; n];
    let mut high_visited = vec![false; n];
    let mut queue = VecDeque::new();
    let mut high_comp = Vec::new();
    for start in 0..n {
        if high_visited[start]
            || macro_u8[start] != river_lake_macro_idx
            || micro_u8[start] != rapids_idx
            || raw_u16_to_y(heights_u16[start]) <= 78.0
        {
            continue;
        }
        high_visited[start] = true;
        queue.clear();
        high_comp.clear();
        queue.push_back(start);
        let mut touches_lake_or_sea = false;
        while let Some(idx) = queue.pop_front() {
            high_comp.push(idx);
            let x = idx % width;
            let y = idx / width;
            let neighbors = [
                (x > 0).then(|| idx - 1),
                (x + 1 < width).then(|| idx + 1),
                (y > 0).then(|| idx - width),
                (y + 1 < height).then(|| idx + width),
            ];
            for n_idx in neighbors.into_iter().flatten() {
                if micro_u8[n_idx] == calm_lake_idx || micro_u8[n_idx] == estuary_idx {
                    touches_lake_or_sea = true;
                } else if macro_u8[n_idx] == river_lake_macro_idx
                    && micro_u8[n_idx] == rapids_idx
                    && !high_visited[n_idx]
                    && raw_u16_to_y(heights_u16[n_idx]) > 78.0
                {
                    high_visited[n_idx] = true;
                    queue.push_back(n_idx);
                }
            }
        }
        if !touches_lake_or_sea {
            for &idx in &high_comp {
                retired_mountain[idx] = true;
                landmass_u8[idx] = 1;
            }
        }
    }
    // Assign retired pixels the nearest dry neighbor's macro/micro biome
    for i in 0..n {
        if !retired_mountain[i] {
            continue;
        }
        let x = (i % width) as i32;
        let y = (i / width) as i32;
        let mut found = None;
        'search: for r in 1..=6i32 {
            for dz in -r..=r {
                for dx in -r..=r {
                    let nx = x + dx;
                    let nz = y + dz;
                    if nx >= 0 && nx < width as i32 && nz >= 0 && nz < height as i32 {
                        let ni = nz as usize * width + nx as usize;
                        if landmass_u8[ni] == 1
                            && macro_u8[ni] != river_lake_macro_idx
                            && !retired_mountain[ni]
                        {
                            found = Some((macro_u8[ni], micro_u8[ni]));
                            break 'search;
                        }
                    }
                }
            }
        }
        let (mac, mic) = found.unwrap_or((highlands_macro_idx, foothills_idx));
        macro_u8[i] = mac;
        micro_u8[i] = mic;
    }

    // Seed small Mountain Foothill Ponds (`CalmLake` radius 2 px = ~8 blocks) at retired river headwaters
    let mut pond_seeds = Vec::new();
    for i in 0..n {
        if macro_u8[i] == river_lake_macro_idx && micro_u8[i] == rapids_idx {
            let y_f = raw_u16_to_y(heights_u16[i]);
            if (60.0..=78.0).contains(&y_f) {
                let x = i % width;
                let y = i / width;
                let touches_retired = (x > 0 && retired_mountain[i - 1])
                    || (x + 1 < width && retired_mountain[i + 1])
                    || (y > 0 && retired_mountain[i - width])
                    || (y + 1 < height && retired_mountain[i + width]);
                if touches_retired {
                    pond_seeds.push(i);
                }
            }
        }
    }
    for seed_idx in pond_seeds {
        let sx = (seed_idx % width) as i32;
        let sy = (seed_idx / width) as i32;
        for dz in -2..=2i32 {
            for dx in -2..=2i32 {
                if dx * dx + dz * dz > 4 {
                    continue;
                }
                let nx = sx + dx;
                let nz = sy + dz;
                if nx >= 0 && nx < width as i32 && nz >= 0 && nz < height as i32 {
                    let ni = nz as usize * width + nx as usize;
                    if !retired_mountain[ni] && raw_u16_to_y(heights_u16[ni]) <= 79.0 {
                        macro_u8[ni] = river_lake_macro_idx;
                        micro_u8[ni] = calm_lake_idx;
                        landmass_u8[ni] = 0;
                    }
                }
            }
        }
    }

    // 2. Mark open oceans and coastal seas at ADVENTURE_SEA_LEVEL, then flood-fill ONLY
    //    `Estuary` pixels that are truly 4-connected to the open sea. Any enclosed coastal
    //    `Estuary` basin (e.g. Volcano Island caldera lake) is reclassified to `CalmLake` so it
    //    receives its true elevated rim height instead of plunging to sea level!
    let mut is_sea = vec![false; n];
    queue.clear();
    for i in 0..n {
        let micro = BiomeType::ALL
            .get(micro_u8[i] as usize)
            .copied()
            .unwrap_or_default();
        if is_ocean_biome(micro)
            || (landmass_u8[i] == 0 && macro_u8[i] != river_lake_macro_idx)
        {
            water_level[i] = ADVENTURE_SEA_LEVEL as i16;
            is_sea[i] = true;
            queue.push_back(i);
        }
    }
    while let Some(idx) = queue.pop_front() {
        let x = idx % width;
        let y = idx / width;
        let neighbors = [
            (x > 0).then(|| idx - 1),
            (x + 1 < width).then(|| idx + 1),
            (y > 0).then(|| idx - width),
            (y + 1 < height).then(|| idx + width),
        ];
        for n_idx in neighbors.into_iter().flatten() {
            if !is_sea[n_idx] && micro_u8[n_idx] == estuary_idx {
                is_sea[n_idx] = true;
                water_level[n_idx] = ADVENTURE_SEA_LEVEL as i16;
                queue.push_back(n_idx);
            }
        }
    }
    for i in 0..n {
        if micro_u8[i] == estuary_idx && !is_sea[i] {
            micro_u8[i] = calm_lake_idx;
            macro_u8[i] = river_lake_macro_idx;
        }
    }

    // 3. BFS flood-fill connected `CalmLake` basins (and foothill ponds) -> ONE 100% flat elevation per lake.
    let mut visited = vec![false; n];
    let mut cluster = Vec::new();
    let mut rim_heights = Vec::new();

    for start in 0..n {
        if micro_u8[start] != calm_lake_idx || visited[start] {
            continue;
        }
        visited[start] = true;
        queue.clear();
        cluster.clear();
        rim_heights.clear();
        queue.push_back(start);

        let mut touches_sea = false;

        while let Some(idx) = queue.pop_front() {
            cluster.push(idx);
            let x = idx % width;
            let y = idx / width;

            let neighbors = [
                (x > 0).then(|| idx - 1),
                (x + 1 < width).then(|| idx + 1),
                (y > 0).then(|| idx - width),
                (y + 1 < height).then(|| idx + width),
            ];

            for n_idx in neighbors.into_iter().flatten() {
                if micro_u8[n_idx] == calm_lake_idx {
                    if !visited[n_idx] {
                        visited[n_idx] = true;
                        queue.push_back(n_idx);
                    }
                } else if is_sea[n_idx] {
                    touches_sea = true;
                } else if landmass_u8[n_idx] == 1 && macro_u8[n_idx] != river_lake_macro_idx {
                    let rim_y = raw_u16_to_y(heights_u16[n_idx]).round() as i32;
                    rim_heights.push(rim_y);
                }
            }
        }

        let lake_y = if touches_sea {
            ADVENTURE_SEA_LEVEL
        } else if rim_heights.is_empty() {
            let avg_in = cluster
                .iter()
                .map(|&i| raw_u16_to_y(heights_u16[i]).round() as i32)
                .sum::<i32>()
                / (cluster.len().max(1) as i32);
            (avg_in - 1).max(ADVENTURE_SEA_LEVEL)
        } else {
            rim_heights.sort_unstable();
            let p20 = rim_heights[(rim_heights.len() / 5).min(rim_heights.len() - 1)];
            (p20 - 1).max(ADVENTURE_SEA_LEVEL)
        } as i16;

        for &idx in &cluster {
            water_level[idx] = lake_y;
        }
    }

    // 4. Stepped `Rapids` rivers with long flat reaches (~48–68 blocks) + cascades/waterfalls!
    //    Compute geodesic river distance upstream from seas & lakes via BFS.
    let mut river_dist = vec![u32::MAX; n];
    let mut outlet_y = vec![ADVENTURE_SEA_LEVEL as i16; n];
    let mut bfs_order = Vec::new();
    queue.clear();

    for i in 0..n {
        if micro_u8[i] != rapids_idx {
            continue;
        }
        let x = i % width;
        let y = i / width;
        let neighbors = [
            (x > 0).then(|| i - 1),
            (x + 1 < width).then(|| i + 1),
            (y > 0).then(|| i - width),
            (y + 1 < height).then(|| i + width),
        ];
        let mut min_anchor: Option<i16> = None;
        for n_idx in neighbors.into_iter().flatten() {
            if water_level[n_idx] != i16::MIN && micro_u8[n_idx] != rapids_idx {
                min_anchor = Some(
                    min_anchor.map_or(water_level[n_idx], |cur| cur.min(water_level[n_idx])),
                );
            }
        }
        if let Some(anchor) = min_anchor {
            river_dist[i] = 1;
            outlet_y[i] = anchor;
            queue.push_back(i);
        }
    }

    while let Some(idx) = queue.pop_front() {
        bfs_order.push(idx);
        let d = river_dist[idx];
        let out = outlet_y[idx];
        let x = idx % width;
        let y = idx / width;
        let neighbors = [
            (x > 0).then(|| idx - 1),
            (x + 1 < width).then(|| idx + 1),
            (y > 0).then(|| idx - width),
            (y + 1 < height).then(|| idx + width),
        ];
        for n_idx in neighbors.into_iter().flatten() {
            if micro_u8[n_idx] == rapids_idx && river_dist[n_idx] == u32::MAX {
                river_dist[n_idx] = d + 1;
                outlet_y[n_idx] = out;
                queue.push_back(n_idx);
            }
        }
    }

    // Any disconnected inland rapids components (draining into an endorheic basin)
    for i in 0..n {
        if micro_u8[i] == rapids_idx && river_dist[i] == u32::MAX {
            river_dist[i] = 1;
            outlet_y[i] =
                ((raw_u16_to_y(heights_u16[i]).round() as i32) - 1).max(ADVENTURE_SEA_LEVEL) as i16;
            bfs_order.push(i);
        }
    }

    // Raw local bank elevation for each river pixel
    let mut river_float = vec![0.0f32; n];
    for &idx in &bfs_order {
        let x = (idx % width) as i32;
        let y = (idx / width) as i32;
        let mut sum = 0.0f32;
        let mut cnt = 0.0f32;
        for dz in -3..=3 {
            let nz = y + dz;
            if nz < 0 || nz >= height as i32 {
                continue;
            }
            for dx in -3..=3 {
                let nx = x + dx;
                if nx < 0 || nx >= width as i32 {
                    continue;
                }
                let ni = nz as usize * width + nx as usize;
                if landmass_u8[ni] == 1 && macro_u8[ni] != river_lake_macro_idx {
                    sum += raw_u16_to_y(heights_u16[ni]);
                    cnt += 1.0;
                }
            }
        }
        let target = if cnt > 0.0 {
            (sum / cnt) - 1.4
        } else {
            raw_u16_to_y(heights_u16[idx]) - 1.2
        };
        river_float[idx] = target.max(outlet_y[idx] as f32).max(ADVENTURE_SEA_LEVEL as f32);
    }

    // Enforce downstream monotonicity along `bfs_order` (downstream never higher than upstream)
    for &idx in &bfs_order {
        let d = river_dist[idx];
        let x = idx % width;
        let y = idx / width;
        let mut max_down = outlet_y[idx] as f32;
        let neighbors = [
            (x > 0).then(|| idx - 1),
            (x + 1 < width).then(|| idx + 1),
            (y > 0).then(|| idx - width),
            (y + 1 < height).then(|| idx + width),
        ];
        for n_idx in neighbors.into_iter().flatten() {
            if micro_u8[n_idx] == rapids_idx && river_dist[n_idx] < d {
                max_down = max_down.max(river_float[n_idx]);
            } else if water_level[n_idx] != i16::MIN && micro_u8[n_idx] != rapids_idx && d <= 2 {
                max_down = max_down.max(water_level[n_idx] as f32);
            }
        }
        river_float[idx] = river_float[idx].max(max_down);
    }

    // Smooth along-river profile over 5 passes before quantizing into flat terraced reaches
    let mut smooth_buf = river_float.clone();
    for _ in 0..5 {
        for &idx in &bfs_order {
            let x = idx % width;
            let y = idx / width;
            let mut s = river_float[idx] * 2.0;
            let mut w_sum = 2.0;
            let neighbors = [
                (x > 0).then(|| idx - 1),
                (x + 1 < width).then(|| idx + 1),
                (y > 0).then(|| idx - width),
                (y + 1 < height).then(|| idx + width),
            ];
            for n_idx in neighbors.into_iter().flatten() {
                if micro_u8[n_idx] == rapids_idx {
                    s += river_float[n_idx];
                    w_sum += 1.0;
                } else if water_level[n_idx] != i16::MIN {
                    s += water_level[n_idx] as f32 * 1.5;
                    w_sum += 1.5;
                }
            }
            smooth_buf[idx] = (s / w_sum).max(outlet_y[idx] as f32);
        }
        for &idx in &bfs_order {
            river_float[idx] = smooth_buf[idx];
        }
    }

    // Group connected river segments into ~13-pixel (~52-block) flat reaches on gentle terrain,
    // while allowing steeper 2-pixel waterfall steps across cliffs/escarpments!
    let mut reach_visited = vec![false; n];
    let mut reach_cluster = Vec::new();
    for &start in &bfs_order {
        if reach_visited[start] {
            continue;
        }
        let reach_bucket = river_dist[start] / 13;
        reach_visited[start] = true;
        queue.clear();
        reach_cluster.clear();
        queue.push_back(start);

        let mut min_f = f32::MAX;
        let mut max_f = f32::MIN;
        while let Some(idx) = queue.pop_front() {
            reach_cluster.push(idx);
            min_f = min_f.min(river_float[idx]);
            max_f = max_f.max(river_float[idx]);
            let x = idx % width;
            let y = idx / width;
            let neighbors = [
                (x > 0).then(|| idx - 1),
                (x + 1 < width).then(|| idx + 1),
                (y > 0).then(|| idx - width),
                (y + 1 < height).then(|| idx + width),
            ];
            for n_idx in neighbors.into_iter().flatten() {
                if micro_u8[n_idx] == rapids_idx
                    && !reach_visited[n_idx]
                    && (river_dist[n_idx] / 13) == reach_bucket
                {
                    reach_visited[n_idx] = true;
                    queue.push_back(n_idx);
                }
            }
        }

        if max_f - min_f <= 3.2 {
            // Gentle lowland/valley gradient: make the entire ~52-block reach 100% flat!
            let flat_y = (((min_f + max_f) * 0.5).round() as i32).max(ADVENTURE_SEA_LEVEL) as i16;
            for &idx in &reach_cluster {
                water_level[idx] = flat_y.max(outlet_y[idx]);
            }
        } else {
            // Steep escarpment / waterfall reach: quantize into crisp 3-block pool-and-drop waterfalls!
            for &idx in &reach_cluster {
                let fy = river_float[idx];
                let terraced = (((fy / 3.0).floor() * 3.0) as i32).max(ADVENTURE_SEA_LEVEL) as i16;
                water_level[idx] = terraced.max(outlet_y[idx]);
            }
        }
    }

    // Re-verify cross-section flatness (adjacent river pixels at similar river_dist share flat Y)
    for &idx in &bfs_order {
        let d = river_dist[idx];
        let x = idx % width;
        let y = idx / width;
        let mut max_down = outlet_y[idx];
        let neighbors = [
            (x > 0).then(|| idx - 1),
            (x + 1 < width).then(|| idx + 1),
            (y > 0).then(|| idx - width),
            (y + 1 < height).then(|| idx + width),
        ];
        for n_idx in neighbors.into_iter().flatten() {
            if micro_u8[n_idx] == rapids_idx && river_dist[n_idx] < d {
                max_down = max_down.max(water_level[n_idx]);
            }
        }
        if water_level[idx] < max_down {
            water_level[idx] = max_down;
        }
    }

    // 5. Compute `1..=6 px` shoreline bank distance & nearest water Y on dry land bordering
    //    inland lakes & rivers for seamless valley/basin bank grading.
    let mut bank_dist = vec![0u8; n];
    let mut bank_water_y = vec![i16::MIN; n];
    queue.clear();

    for i in 0..n {
        if water_level[i] != i16::MIN && !is_sea[i] {
            bank_water_y[i] = water_level[i];
            queue.push_back(i);
        }
    }
    while let Some(idx) = queue.pop_front() {
        let cur_d = bank_dist[idx];
        if cur_d >= 6 {
            continue;
        }
        let wy = bank_water_y[idx];
        let x = idx % width;
        let y = idx / width;
        let neighbors = [
            (x > 0).then(|| idx - 1),
            (x + 1 < width).then(|| idx + 1),
            (y > 0).then(|| idx - width),
            (y + 1 < height).then(|| idx + width),
        ];
        for n_idx in neighbors.into_iter().flatten() {
            if landmass_u8[n_idx] == 1 && water_level[n_idx] == i16::MIN && bank_dist[n_idx] == 0 {
                bank_dist[n_idx] = cur_d + 1;
                bank_water_y[n_idx] = wy;
                queue.push_back(n_idx);
            }
        }
    }

    // 6. Compute interior BFS distance from dry shore inside `RiverAndLake` water bodies (`1..=16`)
    //    so lake beds deepen organically from shallow shores down into deep concave centers.
    let mut dist_in_water = vec![0u8; n];
    queue.clear();
    for i in 0..n {
        if macro_u8[i] != river_lake_macro_idx || water_level[i] == i16::MIN {
            continue;
        }
        let x = i % width;
        let y = i / width;
        let touches_dry = (x > 0 && landmass_u8[i - 1] == 1 && water_level[i - 1] == i16::MIN)
            || (x + 1 < width && landmass_u8[i + 1] == 1 && water_level[i + 1] == i16::MIN)
            || (y > 0 && landmass_u8[i - width] == 1 && water_level[i - width] == i16::MIN)
            || (y + 1 < height && landmass_u8[i + width] == 1 && water_level[i + width] == i16::MIN);
        if touches_dry {
            dist_in_water[i] = 1;
            queue.push_back(i);
        }
    }
    while let Some(idx) = queue.pop_front() {
        let cur_d = dist_in_water[idx];
        if cur_d >= 16 {
            continue;
        }
        let x = idx % width;
        let y = idx / width;
        let neighbors = [
            (x > 0).then(|| idx - 1),
            (x + 1 < width).then(|| idx + 1),
            (y > 0).then(|| idx - width),
            (y + 1 < height).then(|| idx + width),
        ];
        for n_idx in neighbors.into_iter().flatten() {
            if macro_u8[n_idx] == river_lake_macro_idx
                && water_level[n_idx] != i16::MIN
                && dist_in_water[n_idx] == 0
            {
                dist_in_water[n_idx] = cur_d + 1;
                queue.push_back(n_idx);
            }
        }
    }
    for i in 0..n {
        if macro_u8[i] == river_lake_macro_idx
            && water_level[i] != i16::MIN
            && dist_in_water[i] == 0
        {
            dist_in_water[i] = 16;
        }
    }

    // 7. Compute 3-pass separable Gaussian-blurred `water_mask_u8` (`0..=255`) for `RiverAndLake`
    //    so all 4x4-block pixel corners and ruler-straight edges round off into organic curves.
    let mut mask_a = vec![0u8; n];
    for i in 0..n {
        if macro_u8[i] == river_lake_macro_idx && water_level[i] != i16::MIN {
            mask_a[i] = 255;
        }
    }
    let mut mask_b = vec![0u8; n];
    for _pass in 0..3 {
        // Horizontal 3-tap [1, 2, 1] / 4 pass
        for y in 0..height {
            let row = y * width;
            for x in 0..width {
                let xm = x.saturating_sub(1);
                let xp = (x + 1).min(width - 1);
                let v = (mask_a[row + xm] as u16
                    + (mask_a[row + x] as u16) * 2
                    + mask_a[row + xp] as u16
                    + 2)
                    >> 2;
                mask_b[row + x] = v as u8;
            }
        }
        // Vertical 3-tap [1, 2, 1] / 4 pass
        for y in 0..height {
            let ym = y.saturating_sub(1) * width;
            let yc = y * width;
            let yp = (y + 1).min(height - 1) * width;
            for x in 0..width {
                let v = (mask_b[ym + x] as u16
                    + (mask_b[yc + x] as u16) * 2
                    + mask_b[yp + x] as u16
                    + 2)
                    >> 2;
                mask_a[yc + x] = v as u8;
            }
        }
    }
    // Guarantee narrow 1-pixel `Rapids` river centerlines stay connected after blurring
    for i in 0..n {
        if macro_u8[i] == river_lake_macro_idx
            && micro_u8[i] == rapids_idx
            && water_level[i] != i16::MIN
            && mask_a[i] < 168
        {
            mask_a[i] = 168;
        }
    }

    (water_level, bank_dist, bank_water_y, dist_in_water, mask_a)
}

fn load_raw_buffers() -> Result<Arc<RawAdventureMapBuffers>, String> {
    RAW_MAP_CACHE
        .get_or_init(|| {
            let h_img = image::open(HEIGHTMAP_PATH)
                .map_err(|e| format!("Failed to load {HEIGHTMAP_PATH}: {e}"))?
                .into_luma16();
            let (width, height) = h_img.dimensions();

            let load_rgb = |path: &str| -> Result<Vec<u8>, String> {
                let img = image::open(path)
                    .map_err(|e| format!("Failed to load {path}: {e}"))?
                    .into_rgb8();
                if img.dimensions() != (width, height) {
                    return Err(format!(
                        "{path} dimensions {:?} do not match heightmap {:?}",
                        img.dimensions(),
                        (width, height)
                    ));
                }
                Ok(img.into_raw())
            };

            let macro_rgb = load_rgb(MACRO_BIOMES_PATH)?;
            let micro_rgb = load_rgb(MICRO_BIOMES_PATH)?;
            let continents_rgb = load_rgb(CONTINENTS_PATH)?;
            let regions_rgb = load_rgb(REGIONS_PATH)?;

            let land_img = image::open(LANDMASS_PATH)
                .map_err(|e| format!("Failed to load {LANDMASS_PATH}: {e}"))?
                .into_luma8();
            if land_img.dimensions() != (width, height) {
                return Err(format!(
                    "{LANDMASS_PATH} dimensions {:?} do not match heightmap {:?}",
                    land_img.dimensions(),
                    (width, height)
                ));
            }
            let land_raw = land_img.into_raw();

            // Build exact RGB lookup tables for fast O(1) decoding
            let mut macro_lut: HashMap<[u8; 3], u8> = HashMap::with_capacity(MacroBiome::ALL.len());
            for (idx, &m) in MacroBiome::ALL.iter().enumerate() {
                macro_lut.insert(m.legend_rgb(), idx as u8);
            }

            let mut micro_lut: HashMap<[u8; 3], u8> = HashMap::with_capacity(BiomeType::ALL.len());
            for (idx, &b) in BiomeType::ALL.iter().enumerate() {
                micro_lut.insert(b.legend_rgb(), idx as u8);
            }

            let pixel_count = (width as usize) * (height as usize);
            let mut macro_u8 = vec![0u8; pixel_count];
            let mut micro_u8 = vec![0u8; pixel_count];
            let mut continent_region_u8 = vec![0u8; pixel_count];
            let mut landmass_u8 = vec![0u8; pixel_count];

            let mut macro_fallback_cache: HashMap<[u8; 3], u8> = HashMap::new();
            let mut micro_fallback_cache: HashMap<[u8; 3], u8> = HashMap::new();

            for i in 0..pixel_count {
                let off = i * 3;

                // 1. Macro biome
                let m_rgb = [macro_rgb[off], macro_rgb[off + 1], macro_rgb[off + 2]];
                macro_u8[i] = if let Some(&idx) = macro_lut.get(&m_rgb) {
                    idx
                } else {
                    *macro_fallback_cache.entry(m_rgb).or_insert_with(|| {
                        MacroBiome::ALL
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, m)| rgb_dist_sq(m_rgb, m.legend_rgb()))
                            .map_or(0, |(idx, _)| idx as u8)
                    })
                };

                // 2. Micro biome
                let b_rgb = [micro_rgb[off], micro_rgb[off + 1], micro_rgb[off + 2]];
                micro_u8[i] = if let Some(&idx) = micro_lut.get(&b_rgb) {
                    idx
                } else {
                    *micro_fallback_cache.entry(b_rgb).or_insert_with(|| {
                        BiomeType::ALL
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, b)| rgb_dist_sq(b_rgb, b.legend_rgb()))
                            .map_or(0, |(idx, _)| idx as u8)
                    })
                };

                // 3. Continent + Region packed byte
                let c_rgb = [
                    continents_rgb[off],
                    continents_rgb[off + 1],
                    continents_rgb[off + 2],
                ];
                let cont = ContinentId::from_rgb(c_rgb).unwrap_or_else(|| {
                    ContinentId::ALL
                        .iter()
                        .copied()
                        .min_by_key(|c| rgb_dist_sq(c_rgb, c.legend_rgb()))
                        .unwrap_or_default()
                });

                let r_rgb = [regions_rgb[off], regions_rgb[off + 1], regions_rgb[off + 2]];
                let reg = WorldRegionId::from_rgb(r_rgb).unwrap_or_else(|| {
                    WorldRegionId::ALL
                        .iter()
                        .copied()
                        .min_by_key(|r| rgb_dist_sq(r_rgb, r.legend_rgb()))
                        .unwrap_or_default()
                });
                continent_region_u8[i] = ((reg as u8) << 4) | ((cont as u8) & 0x0F);

                // 4. Landmass mask
                landmass_u8[i] = if land_raw[i] >= 128 { 1 } else { 0 };
            }

            let heights_u16 = h_img.into_raw();
            let (water_level_i16, bank_dist_u8, bank_water_y_i16, dist_in_water_u8, water_mask_u8) =
                compute_water_levels(
                    width as usize,
                    height as usize,
                    &heights_u16,
                    &mut macro_u8,
                    &mut micro_u8,
                    &mut landmass_u8,
                );

            let mem_mb =
                (pixel_count * (2 + 1 + 1 + 1 + 1 + 2 + 1 + 2 + 1 + 1)) as f32 / (1024.0 * 1024.0);
            info!(
                "Loaded 6-layer Adventure Mode map assets ({width}x{height}): ~{mem_mb:.1} MB"
            );

            Ok(Arc::new(RawAdventureMapBuffers {
                width,
                height,
                heights_u16,
                macro_u8,
                micro_u8,
                continent_region_u8,
                landmass_u8,
                water_level_i16,
                bank_dist_u8,
                bank_water_y_i16,
                dist_in_water_u8,
                water_mask_u8,
            }))
        })
        .clone()
}

/// Derives synthetic `(temperature, humidity)` in `[-1.0, 1.0]` for any surface micro-biome,
/// gently modulated by latitude and altitude so foliage tinting, snow/rain weather, and ambience
/// behave authentically without a separate `climate_map.png`.
fn synthetic_climate(biome: BiomeType, v: f32, height: f32) -> (f32, f32) {
    let (base_t, base_h): (f32, f32) = match biome {
        // I. Islands
        BiomeType::TropicalIsland | BiomeType::CoralIsland => (0.82, 0.72),
        BiomeType::TemperateIsland => (0.20, 0.35),
        BiomeType::TundraIsland => (-0.65, -0.10),
        BiomeType::FrozenIsland => (-0.90, -0.35),
        BiomeType::RockyIsland => (-0.05, 0.15),

        // II. Aquatic
        BiomeType::WarmOcean | BiomeType::CoralReefs | BiomeType::SeagrassMeadows => (0.55, 0.85),
        BiomeType::ColdOcean | BiomeType::KelpForests | BiomeType::OceanTrench => (-0.35, 0.80),
        BiomeType::Rapids | BiomeType::CalmLake | BiomeType::Estuary => (0.15, 0.85),

        // III. Coastal
        BiomeType::SandyShores | BiomeType::DuneBeach => (0.55, 0.20),
        BiomeType::GravelBeach | BiomeType::CraggyBeach | BiomeType::CoastalCliffs => (-0.10, 0.25),
        BiomeType::TidePools => (0.25, 0.55),
        BiomeType::Mudflats | BiomeType::SwampyShore | BiomeType::ReedyShore => (0.30, 0.85),

        // IV. Special & Dark Fantasy
        BiomeType::Caldera | BiomeType::VolcanicPeaks | BiomeType::ScorchedFlats => (0.90, -0.60),
        BiomeType::VolcanicJungle => (0.85, 0.80),
        BiomeType::AcidPools | BiomeType::SulfurVents => (0.75, 0.25),
        BiomeType::WitheredWoods | BiomeType::AshenWastes | BiomeType::ObsidianMire => (-0.15, 0.05),

        // V. Lowlands & Wetlands
        BiomeType::Grasslands | BiomeType::FlowerMeadow | BiomeType::LavenderValley => (0.25, 0.30),
        BiomeType::RollingSteppe => (0.45, -0.25),
        BiomeType::FertileValley | BiomeType::SakuraGrove => (0.22, 0.50),
        BiomeType::WillowMarsh | BiomeType::OrchidSwamp => (0.35, 0.90),
        BiomeType::MangroveSwamp => (0.72, 0.95),

        // VI. Forests
        BiomeType::VerdantWoods | BiomeType::AutumnForest | BiomeType::MistyWoods => (0.18, 0.45),
        BiomeType::PineHighlands | BiomeType::BleakTaiga | BiomeType::ShieldForest => (-0.30, 0.35),
        BiomeType::DenseRainforest | BiomeType::BambooGrove | BiomeType::JungleWetlands => {
            (0.82, 0.90)
        }

        // VII. Mountains & Relief
        BiomeType::AlpinePeaks | BiomeType::JaggedPeaks | BiomeType::MistyCrags => (-0.55, 0.15),
        BiomeType::EmeraldFoothills | BiomeType::WindyPass => (-0.15, 0.30),
        BiomeType::GranitePlateau | BiomeType::DeepGorge | BiomeType::MonolithCliffs => {
            (-0.20, 0.05)
        }
        BiomeType::AmethystCanyon => (-0.30, 0.20),

        // VIII. Arid
        BiomeType::DuneSea
        | BiomeType::AncientSands
        | BiomeType::RedrockCanyon
        | BiomeType::PaintedMesas
        | BiomeType::DesertSpires => (0.90, -0.85),
        BiomeType::DesertOasis => (0.80, 0.45),
        BiomeType::AcaciaPlains | BiomeType::FracturedSavanna | BiomeType::SavannaBadlands => {
            (0.75, -0.45)
        }

        // IX. Frozen & Polar
        BiomeType::SnowyPlains | BiomeType::ColdShrubland | BiomeType::PermafrostSteppe => {
            (-0.72, -0.15)
        }
        BiomeType::GlacialChasm | BiomeType::IceSheet | BiomeType::FrozenCliffs => (-0.95, -0.30),
        BiomeType::WintryForest | BiomeType::FrostPinewoods | BiomeType::IceboundRidge => {
            (-0.62, 0.10)
        }
    };

    // Subtle latitude & altitude cooling so transitions feel natural
    let pole_factor = ((v - 0.5).abs() * 2.0).clamp(0.0, 1.0);
    let alt_cool = ((height - ADVENTURE_SEA_LEVEL as f32).max(0.0) / 170.0) * 0.25;
    let temp = (base_t - pole_factor * 0.12 - alt_cool).clamp(-1.0, 1.0);
    let humid = base_h.clamp(-1.0, 1.0);
    (temp, humid)
}

pub struct AdventureWorldMap {
    raw: Arc<RawAdventureMapBuffers>,
    pub scale: AdventureWorldScale,
    pub world_width: f32,
    pub world_height: f32,
    pub sea_level: i32,
    pub min_y: f32,
    pub max_y: f32,
}

impl AdventureWorldMap {
    pub fn load_with_scale(scale: AdventureWorldScale) -> Result<Self, String> {
        let raw = load_raw_buffers()?;
        let (world_width, world_height) = scale.dimensions();
        Ok(Self {
            raw,
            scale,
            world_width,
            world_height,
            sea_level: ADVENTURE_SEA_LEVEL,
            min_y: ADVENTURE_MIN_Y,
            max_y: ADVENTURE_MAX_Y,
        })
    }

    #[inline]
    pub fn half_extents(&self) -> (f32, f32) {
        (self.world_width * 0.5, self.world_height * 0.5)
    }

    #[inline]
    pub fn spawn_world_xz(&self, choice: AdventureSpawnChoice) -> (i32, i32) {
        let (u, v) = choice.normalized_uv();
        let wx = ((u - 0.5) * self.world_width).round() as i32;
        let wz = ((v - 0.5) * self.world_height).round() as i32;
        (wx, wz)
    }

    #[inline]
    pub fn fingerprint(&self) -> u64 {
        let scale_tag = match self.scale {
            AdventureWorldScale::Medium => 2u64,
            AdventureWorldScale::Full => 3u64,
        };
        0xAD7E_3200_1800_0000u64 ^ scale_tag.wrapping_mul(0x9E37_79B9_7F4A_7C15)
    }

    #[inline]
    fn world_to_pixel_f(&self, world_x: i32, world_z: i32) -> Option<(f32, f32, f32)> {
        let half_w = self.world_width * 0.5;
        let half_h = self.world_height * 0.5;
        let wx_f = world_x as f32;
        let wz_f = world_z as f32;
        if wx_f < -half_w || wx_f >= half_w || wz_f < -half_h || wz_f >= half_h {
            return None;
        }
        let u = ((wx_f + half_w) / self.world_width).clamp(0.0, 1.0);
        let v = ((wz_f + half_h) / self.world_height).clamp(0.0, 1.0);
        let fx = u * (self.raw.width - 1) as f32;
        let fz = v * (self.raw.height - 1) as f32;
        Some((fx, fz, v))
    }

    #[inline]
    fn pixel_index(&self, px: i32, pz: i32) -> usize {
        let x = px.clamp(0, self.raw.width as i32 - 1) as usize;
        let z = pz.clamp(0, self.raw.height as i32 - 1) as usize;
        z * self.raw.width as usize + x
    }

    #[inline]
    fn sample_height_bilinear(&self, fx: f32, fz: f32) -> f32 {
        let w = self.raw.width as i32;
        let h = self.raw.height as i32;
        let x0 = (fx.floor() as i32).clamp(0, w - 1);
        let z0 = (fz.floor() as i32).clamp(0, h - 1);
        let x1 = (x0 + 1).min(w - 1);
        let z1 = (z0 + 1).min(h - 1);
        let tx = (fx - x0 as f32).clamp(0.0, 1.0);
        let tz = (fz - z0 as f32).clamp(0.0, 1.0);

        let w_u = self.raw.width as usize;
        let h00 = self.raw.heights_u16[z0 as usize * w_u + x0 as usize] as f32;
        let h10 = self.raw.heights_u16[z0 as usize * w_u + x1 as usize] as f32;
        let h01 = self.raw.heights_u16[z1 as usize * w_u + x0 as usize] as f32;
        let h11 = self.raw.heights_u16[z1 as usize * w_u + x1 as usize] as f32;

        let top = h00 + (h10 - h00) * tx;
        let bot = h01 + (h11 - h01) * tx;
        let raw_u16 = top + (bot - top) * tz;
        self.min_y + (raw_u16 / 65535.0) * (self.max_y - self.min_y)
    }

    #[inline]
    fn sample_micro_at(&self, idx: usize) -> BiomeType {
        BiomeType::ALL
            .get(self.raw.micro_u8[idx] as usize)
            .copied()
            .unwrap_or_default()
    }

    #[inline]
    fn sample_macro_at(&self, idx: usize) -> MacroBiome {
        MacroBiome::ALL
            .get(self.raw.macro_u8[idx] as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Bilinearly-interpolated, multi-octave organically domain-warped water coverage and bank
    /// distance sample using the 3-pass Gaussian-blurred `water_mask_u8`. Used identically by
    /// `sample_water_level` and `sample_column_data` so Adventure Mode rivers and lakes have
    /// completely organic, meandering, non-square shorelines at 1-block resolution.
    #[inline]
    fn sample_water_field(
        &self,
        world_x: i32,
        world_z: i32,
        fx: f32,
        fz: f32,
    ) -> (f32, f32, f32, Option<i32>, BiomeType) {
        const WATER_WARP_SEED_X: u32 = 0x5EA1_4E7D;
        const WATER_WARP_SEED_Z: u32 = 0x9B2C_61A3;
        const WATER_EDGE_SEED: u32 = 0x3C7E_D14B;

        let wx_f = world_x as f32;
        let wz_f = world_z as f32;
        // Two-scale organic warp: broad natural coves/bends (~35-block wavelength, +-1.15 px)
        // plus fine 1-2 block shoreline irregularity (~12-block wavelength, +-0.32 px).
        let warp_x = fractal_noise_2d(wx_f, wz_f, 0.028, 2, 0.5, WATER_WARP_SEED_X) * 1.15
            + fractal_noise_2d(wx_f + 19.1, wz_f - 11.7, 0.085, 2, 0.5, WATER_WARP_SEED_X ^ 0x55AA)
                * 0.32;
        let warp_z = fractal_noise_2d(wx_f + 41.3, wz_f + 23.7, 0.028, 2, 0.5, WATER_WARP_SEED_Z)
            * 1.15
            + fractal_noise_2d(wx_f - 13.4, wz_f + 29.9, 0.085, 2, 0.5, WATER_WARP_SEED_Z ^ 0xAA55)
                * 0.32;

        let max_x = (self.raw.width - 1) as f32;
        let max_z = (self.raw.height - 1) as f32;
        let wfx = (fx + warp_x).clamp(0.0, max_x);
        let wfz = (fz + warp_z).clamp(0.0, max_z);

        let w = self.raw.width as i32;
        let h = self.raw.height as i32;
        let x0 = (wfx.floor() as i32).clamp(0, w - 1);
        let z0 = (wfz.floor() as i32).clamp(0, h - 1);
        let x1 = (x0 + 1).min(w - 1);
        let z1 = (z0 + 1).min(h - 1);
        let tx = (wfx - x0 as f32).clamp(0.0, 1.0);
        let tz = (wfz - z0 as f32).clamp(0.0, 1.0);

        let w_u = self.raw.width as usize;
        let corners = [
            (z0 as usize * w_u + x0 as usize, (1.0 - tx) * (1.0 - tz)),
            (z0 as usize * w_u + x1 as usize, tx * (1.0 - tz)),
            (z1 as usize * w_u + x0 as usize, (1.0 - tx) * tz),
            (z1 as usize * w_u + x1 as usize, tx * tz),
        ];

        let mut raw_mask_cov = 0.0f32;
        let mut in_dist_sum = 0.0f32;
        let mut in_dist_w = 0.0f32;
        let mut bank_dist_sum = 0.0f32;
        let mut bank_w_sum = 0.0f32;
        let mut best_water_w = -1.0f32;
        let mut best_water_y: Option<i32> = None;
        let mut best_water_micro = BiomeType::CalmLake;
        let mut best_bank_w = -1.0f32;
        let mut best_bank_y: Option<i32> = None;

        for (idx, wc) in corners {
            raw_mask_cov += wc * (self.raw.water_mask_u8[idx] as f32 / 255.0);
            let is_inland_water = self.sample_macro_at(idx) == MacroBiome::RiverAndLake
                && self.raw.water_level_i16[idx] != i16::MIN;
            if is_inland_water {
                in_dist_sum += wc * self.raw.dist_in_water_u8[idx] as f32;
                in_dist_w += wc;
                if wc > best_water_w {
                    best_water_w = wc;
                    best_water_y = Some(self.raw.water_level_i16[idx] as i32);
                    best_water_micro = self.sample_micro_at(idx);
                }
            } else if self.raw.landmass_u8[idx] == 1 {
                let bd = self.raw.bank_dist_u8[idx];
                if bd > 0 && self.raw.bank_water_y_i16[idx] != i16::MIN {
                    bank_dist_sum += wc * bd as f32;
                    bank_w_sum += wc;
                    if wc > best_bank_w {
                        best_bank_w = wc;
                        best_bank_y = Some(self.raw.bank_water_y_i16[idx] as i32);
                    }
                } else {
                    bank_dist_sum += wc * 6.5;
                    bank_w_sum += wc;
                }
            }
        }

        // Subtle fine shoreline perturbation so no edge ever looks ruler-straight
        let edge_jitter =
            fractal_noise_2d(wx_f, wz_f, 0.14, 2, 0.5, WATER_EDGE_SEED) * 0.045;
        let inland_cov = if raw_mask_cov > 0.02 {
            (raw_mask_cov + edge_jitter).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // If the warped cove falls slightly outside the original 4 corners, find nearest parent
        // `RiverAndLake` pixel within radius 2 px so `water_y` and `water_micro` stay exact.
        if inland_cov > 0.08 && best_water_y.is_none() {
            let cx = wfx.round() as i32;
            let cz = wfz.round() as i32;
            'find_water: for r in 1..=2i32 {
                for dz in -r..=r {
                    for dx in -r..=r {
                        let ni = self.pixel_index(cx + dx, cz + dz);
                        if self.sample_macro_at(ni) == MacroBiome::RiverAndLake
                            && self.raw.water_level_i16[ni] != i16::MIN
                        {
                            best_water_y = Some(self.raw.water_level_i16[ni] as i32);
                            best_water_micro = self.sample_micro_at(ni);
                            break 'find_water;
                        }
                    }
                }
            }
        }

        let in_dist = if in_dist_w > 0.001 {
            in_dist_sum / in_dist_w
        } else {
            0.0
        };
        let bank_dist = if bank_w_sum > 0.001 {
            bank_dist_sum / bank_w_sum
        } else {
            6.5
        };
        let water_y = best_water_y.or(best_bank_y);

        (inland_cov, in_dist, bank_dist, water_y, best_water_micro)
    }

    /// Returns the local water surface elevation `Y` at `(world_x, world_z)` if this column is
    /// covered by ocean, estuary, elevated `CalmLake`, or stepped `Rapids` river water.
    #[inline]
    pub fn sample_water_level(&self, world_x: i32, world_z: i32) -> Option<i32> {
        let Some((fx, fz, _)) = self.world_to_pixel_f(world_x, world_z) else {
            return Some(self.sea_level);
        };
        let (inland_cov, _, _, water_y, _) = self.sample_water_field(world_x, world_z, fx, fz);
        if inland_cov >= 0.46 {
            return water_y;
        }
        let idx = self.pixel_index(fx.round() as i32, fz.round() as i32);
        if self.sample_macro_at(idx) == MacroBiome::RiverAndLake {
            // Outside the smooth curved shoreline of an inland river/lake -> dry bank!
            return None;
        }
        let wl = self.raw.water_level_i16[idx];
        if wl == i16::MIN {
            None
        } else {
            Some(wl as i32)
        }
    }

    /// Returns `Some((is_river_bank, is_lake_bank, bank_factor))` when `(world_x, world_z)` sits
    /// on the dry shoreline transition strip (2–6 blocks wide) bordering an inland river or lake.
    #[inline]
    pub fn sample_inland_bank(&self, world_x: i32, world_z: i32) -> Option<(bool, bool, f32)> {
        let (fx, fz, _) = self.world_to_pixel_f(world_x, world_z)?;
        let (inland_cov, _, _, water_y, water_micro) =
            self.sample_water_field(world_x, world_z, fx, fz);
        if water_y.is_none() || inland_cov >= 0.46 || inland_cov < 0.14 {
            return None;
        }
        let bank_factor = ((inland_cov - 0.14) / (0.46 - 0.14)).clamp(0.0, 1.0);
        let is_lake = water_micro == BiomeType::CalmLake;
        let is_river = !is_lake;
        Some((is_river, is_lake, bank_factor))
    }

    /// Samples Continent, Region, Macro-Biome, and Micro-Biome metadata for World Map hover & HUD.
    #[inline]
    pub fn sample_region_info(&self, world_x: i32, world_z: i32) -> Option<AdventureRegionInfo> {
        let (fx, fz, _) = self.world_to_pixel_f(world_x, world_z)?;
        let idx = self.pixel_index(fx.round() as i32, fz.round() as i32);
        let packed = self.raw.continent_region_u8[idx];
        Some(AdventureRegionInfo {
            continent: ContinentId::from_u8(packed & 0x0F),
            region: WorldRegionId::from_u8(packed >> 4),
            macro_biome: self.sample_macro_at(idx),
            micro_biome: self.sample_micro_at(idx),
        })
    }

    /// Samples the official flat RGB overlay color for the `[M]` World Map filter layers.
    #[inline]
    pub fn sample_overlay_rgb(
        &self,
        world_x: i32,
        world_z: i32,
        mode: AdventureOverlayMode,
    ) -> Option<[u8; 3]> {
        let info = self.sample_region_info(world_x, world_z)?;
        match mode {
            AdventureOverlayMode::Terrain => None,
            AdventureOverlayMode::Continents => Some(info.continent.legend_rgb()),
            AdventureOverlayMode::Regions => Some(info.region.legend_rgb()),
            AdventureOverlayMode::MacroBiomes => Some(info.macro_biome.legend_rgb()),
            AdventureOverlayMode::MicroBiomes => Some(info.micro_biome.legend_rgb()),
        }
    }

    /// Samples height, micro-biome, and climate for a world column in Adventure Mode.
    pub fn sample_column_data(
        &self,
        world_x: i32,
        world_z: i32,
        seed: u32,
    ) -> (f32, BiomeType, bool, ClimateSample) {
        let half_h = self.world_height * 0.5;
        let wx_f = world_x as f32;
        let wz_f = world_z as f32;

        // Smooth open ocean outside the Adventure world boundaries
        let Some((fx, fz, v)) = self.world_to_pixel_f(world_x, world_z) else {
            let lat_cold = ((wz_f / half_h).abs()).clamp(0.0, 1.0);
            let temp = (0.45 - lat_cold * 1.15).clamp(-1.0, 1.0);
            let biome = if temp < -0.15 {
                BiomeType::ColdOcean
            } else {
                BiomeType::WarmOcean
            };
            let climate = ClimateSample {
                continentalness: -0.75,
                erosion: 0.25,
                ridges: 0.0,
                temperature: temp,
                humidity: 0.70,
                depth: 0.0,
                biome,
            };
            return (self.sea_level as f32 - 44.0, biome, false, climate);
        };

        let mut raw_height = self.sample_height_bilinear(fx, fz);
        let base_idx = self.pixel_index(fx.round() as i32, fz.round() as i32);
        let base_macro = self.sample_macro_at(base_idx);
        let base_micro = self.sample_micro_at(base_idx);
        let base_land = self.raw.landmass_u8[base_idx] == 1;
        let (inland_cov, in_dist, smooth_bank_dist, field_water_y, field_water_micro) =
            self.sample_water_field(world_x, world_z, fx, fz);
        let is_inland_water = inland_cov >= 0.46 && field_water_y.is_some();
        let is_river_or_lake = is_inland_water;

        // Subtle organic domain-warp (~3-5 blocks) for dry-land micro-biome borders,
        // while keeping open oceans and smooth curved inland water bodies crisp.
        let mut biome = if is_inland_water {
            field_water_micro
        } else if base_macro == MacroBiome::RiverAndLake {
            // Smoothly pulled outside the curved bank of a river/lake -> inherit nearest dry biome!
            let px_i = fx.round() as i32;
            let pz_i = fz.round() as i32;
            let mut dry_b = BiomeType::Grasslands;
            'find_dry: for r in 1..=2i32 {
                for dz in -r..=r {
                    for dx in -r..=r {
                        let ni = self.pixel_index(px_i + dx, pz_i + dz);
                        if self.raw.landmass_u8[ni] == 1
                            && self.sample_macro_at(ni) != MacroBiome::RiverAndLake
                        {
                            dry_b = self.sample_micro_at(ni);
                            break 'find_dry;
                        }
                    }
                }
            }
            dry_b
        } else {
            base_micro
        };

        let is_dry_land = !is_inland_water && (base_land || base_macro == MacroBiome::RiverAndLake);
        if is_dry_land {
            let warp_px = 1.05;
            let warp_x =
                fractal_noise_2d(wx_f, wz_f, 0.045, 2, 0.5, seed ^ 0x517C_C1B7) * warp_px;
            let warp_z = fractal_noise_2d(
                wx_f + 31.7,
                wz_f + 17.3,
                0.045,
                2,
                0.5,
                seed ^ 0x9E37_79B9,
            ) * warp_px;
            let warped_idx =
                self.pixel_index((fx + warp_x).round() as i32, (fz + warp_z).round() as i32);
            if self.raw.landmass_u8[warped_idx] == 1
                && self.sample_macro_at(warped_idx) != MacroBiome::RiverAndLake
            {
                biome = self.sample_micro_at(warped_idx);
            }
        }

        let sea_f = self.sea_level as f32;

        // Gently ease the low coastal band (sea_level..sea_level+16) so beaches slope gradually
        if is_dry_land && raw_height > sea_f && raw_height < sea_f + 16.0 {
            let t = (raw_height - sea_f) / 16.0;
            let eased = t * t * 0.65 + t * 0.35;
            raw_height = sea_f + eased * 16.0;
        }

        // Continuous effective distance from curved water's edge (`0.0` at shoreline -> `6.5` inland)
        let eff_bank_dist = if is_inland_water {
            0.0
        } else if inland_cov > 0.02 {
            ((0.46 - inland_cov) / 0.46).clamp(0.0, 1.0)
        } else {
            (0.95 + smooth_bank_dist).min(6.5)
        };

        // Controlled high-frequency micro-detail (attenuated near coasts, lakeshores & riverbanks)
        let coast_atten = ((raw_height - sea_f).abs() / 6.5).clamp(0.0, 1.0);
        let bank_atten = if is_inland_water {
            0.12
        } else if eff_bank_dist < 6.0 && field_water_y.is_some() {
            (eff_bank_dist / 6.0).clamp(0.18, 1.0)
        } else {
            1.0
        };
        let detail_amp = match biome {
            BiomeType::AlpinePeaks
            | BiomeType::JaggedPeaks
            | BiomeType::MonolithCliffs
            | BiomeType::EmeraldFoothills
            | BiomeType::GranitePlateau
            | BiomeType::MistyCrags
            | BiomeType::VolcanicPeaks
            | BiomeType::PaintedMesas => 5.5,
            BiomeType::DuneSea | BiomeType::AncientSands | BiomeType::DesertSpires => 2.6,
            b if is_ocean_biome(b) => 1.8,
            _ => 2.2,
        };
        let micro_scale = (0.045 / self.scale.factor().sqrt()).min(0.11);
        let micro = fractal_noise_2d(wx_f, wz_f, micro_scale, 3, 0.5, seed ^ 0xA24B_AED4);
        raw_height += micro * detail_amp * coast_atten * bank_atten;

        // Smooth curved shoreline & riverbank grading on dry land (`eff_bank_dist in 0.0..6.0`),
        // eliminating 4x4 blocky corners and flat concrete curbs while gently sealing the bank.
        if is_dry_land && eff_bank_dist < 6.0 {
            if let Some(wy_i) = field_water_y {
                let wy = wy_i as f32;
                let shore_target = wy + 0.45 + (eff_bank_dist.min(1.0)) * 0.85;
                let t = (eff_bank_dist / 6.0).clamp(0.0, 1.0);
                let smooth_t = t * t * (3.0 - 2.0 * t);
                let graded = shore_target + (raw_height - shore_target) * smooth_t;
                raw_height = if eff_bank_dist <= 1.2 {
                    graded.max(wy + 0.65)
                } else {
                    graded
                };
            }
        }

        // Stage 4: Carve smooth curved elevated CalmLake basins (deep concave center!), foothill
        // ponds, stepped Rapids reaches, and Estuaries strictly relative to `field_water_y`.
        if is_inland_water {
            let water_y = field_water_y.unwrap_or(self.sea_level) as f32;
            let bed_noise =
                fractal_noise_2d(wx_f, wz_f, 0.075, 2, 0.5, seed ^ 0x71B3_2C4D) * 0.75;
            // Gentle continuous ramp right from the shoreline (`edge_t = 0.0` -> shallow 1-block wading)
            // down into the deeper channel/basin center so there is NEVER a sheer vertical cliff at the bank!
            let edge_t = ((inland_cov - 0.46) / 0.54).clamp(0.0, 1.0);
            let smooth_edge = edge_t * edge_t * (3.0 - 2.0 * edge_t);

            let depth = match field_water_micro {
                BiomeType::CalmLake => {
                    // Shallow wading shore (~0.8..2.2) smoothly deepening into a deep
                    // concave center (~8.5..11.5 blocks deep in large lakes!)
                    let deep_t = ((in_dist - 0.85) / 5.5).clamp(0.0, 1.0);
                    let concave = deep_t * (2.0 - deep_t);
                    0.85 + smooth_edge * 1.65 + concave * 8.8
                }
                BiomeType::Rapids => {
                    let center_t = ((in_dist - 0.65) / 2.2).clamp(0.0, 1.0);
                    0.80 + smooth_edge * 1.55 + center_t * 1.65
                }
                BiomeType::Estuary => {
                    let deep_t = ((in_dist - 0.65) / 4.0).clamp(0.0, 1.0);
                    0.90 + smooth_edge * 1.70 + deep_t * 3.8
                }
                _ => 0.90 + smooth_edge * 2.0,
            };
            raw_height = (water_y - depth + bed_noise * 0.55).min(water_y - 0.75);
        } else if !is_dry_land && raw_height >= sea_f - 0.5 {
            // Keep open sea / coastal sea pixels safely submerged
            raw_height = sea_f - 1.8;
        }

        let (temp, humid) = synthetic_climate(biome, v, raw_height);

        // Keep ocean/land biome classification consistent when height crosses sea level
        if !is_river_or_lake && raw_height < sea_f - 0.5 && !is_ocean_biome(biome) {
            biome = if raw_height < sea_f - 36.0 {
                BiomeType::OceanTrench
            } else if temp > 0.15 && raw_height > sea_f - 10.0 {
                BiomeType::CoralReefs
            } else if temp < -0.15 {
                BiomeType::ColdOcean
            } else {
                BiomeType::WarmOcean
            };
        } else if !is_river_or_lake && raw_height >= sea_f - 0.5 && is_ocean_biome(biome) {
            biome = if temp < -0.55 {
                BiomeType::GlacialChasm
            } else if temp > 0.55 && humid < -0.25 {
                BiomeType::SandyShores
            } else {
                BiomeType::GravelBeach
            };
        }

        let continentalness = if raw_height >= sea_f - 2.0 && raw_height <= sea_f + 9.0 {
            ((raw_height - sea_f) / 185.0).clamp(-1.0, 0.049)
        } else {
            ((raw_height - sea_f) / 65.0).clamp(-1.0, 1.0)
        };
        let ridges = ((raw_height - (sea_f + 35.0)) / 95.0).clamp(0.0, 1.0);
        let erosion = (1.0 - ridges).clamp(-1.0, 1.0);

        let climate = ClimateSample {
            continentalness,
            erosion,
            ridges,
            temperature: temp,
            humidity: humid,
            depth: 0.0,
            biome,
        };

        (raw_height, biome, false, climate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adventure_map_loads_all_six_layers() {
        let map = AdventureWorldMap::load_with_scale(AdventureWorldScale::Full)
            .expect("All 6 3200x1800 Adventure Mode PNGs should load cleanly");
        assert_eq!(map.raw.width, 3200);
        assert_eq!(map.raw.height, 1800);
        assert_eq!(map.world_width, 12_800.0);
        assert_eq!(map.world_height, 7_200.0);

        for spawn in AdventureSpawnChoice::ALL {
            let (sx, sz) = map.spawn_world_xz(spawn);
            let info = map
                .sample_region_info(sx, sz)
                .expect("Spawn point must lie inside Adventure world bounds");
            let (h, biome, _, _) = map.sample_column_data(sx, sz, 42);
            assert!(h.is_finite());
            assert!(
                h >= ADVENTURE_SEA_LEVEL as f32,
                "Spawn {:?} should be on dry land, got h={h:.1}",
                spawn
            );
            println!(
                "Spawn {:?} -> ({sx}, {sz}) h={h:.1} biome={:?} macro={:?} region={:?} continent={:?}",
                spawn, biome, info.macro_biome, info.region, info.continent
            );
        }

        // Verify Stage 4 elevated lakes & rivers computed above sea level
        let elevated_water_pixels = map
            .raw
            .water_level_i16
            .iter()
            .filter(|&&wl| wl != i16::MIN && wl > ADVENTURE_SEA_LEVEL as i16)
            .count();
        println!("Elevated inland water pixels (Y > {ADVENTURE_SEA_LEVEL}): {elevated_water_pixels}");
        assert!(
            elevated_water_pixels > 25_000,
            "Expected majority of inland lakes & rivers above sea level, got {elevated_water_pixels}"
        );

        // Verify the exact 5 inland lakes/rivers from user screenshots are elevated (NOT at Y=12!)
        for (label, cx, cz, min_expected_y) in [
            ("Plains River", -1756, 247, 26),
            ("Oasis Lake", -1822, -175, 28),
            ("Sulfur Lake", -3946, -2252, 54),
            ("Jungle Lake", -2808, 642, 40),
        ] {
            let mut max_wl = ADVENTURE_SEA_LEVEL;
            for dz in -80..=80 {
                for dx in -80..=80 {
                    if let Some(wl) = map.sample_water_level(cx + dx * 2, cz + dz * 2) {
                        max_wl = max_wl.max(wl);
                    }
                }
            }
            println!("{label} near ({cx}, {cz}): local water_level Y = {max_wl}");
            assert!(
                max_wl >= min_expected_y,
                "{label} should sit elevated >= Y={min_expected_y}, got Y={max_wl}"
            );
        }
    }

    /// Renders an authentic in-game top-down World Map preview (`1280x720`) directly from
    /// `TerrainGenerator` + `AdventureWorldMap` and saves it to `assets/maps/adventure/map_preview.png`.
    /// Run anytime after adjusting biomes/terrain:
    /// `cargo test render_adventure_map_preview --lib -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn render_adventure_map_preview() {
        use crate::generation::TerrainGenerator;
        use crate::map::color::{apply_relief_shading, voxel_map_color_at};
        use crate::world::Voxel;

        const OUT_W: u32 = 1280;
        const OUT_H: u32 = 720;

        let map = Arc::new(
            AdventureWorldMap::load_with_scale(AdventureWorldScale::Full)
                .expect("Failed to load Adventure map"),
        );
        let mut terrain_gen = TerrainGenerator::default();
        terrain_gen.adventure_map = Some(map.clone());

        let half_w = map.world_width * 0.5;
        let half_h = map.world_height * 0.5;
        let mut raw = vec![0u8; (OUT_W * OUT_H * 4) as usize];
        let mut prev_heights = vec![0i16; OUT_W as usize];

        for py in 0..OUT_H {
            let v = (py as f32 + 0.5) / OUT_H as f32;
            let wz = ((v - 0.5) * map.world_height).floor() as i32;
            let mut cur_heights = vec![0i16; OUT_W as usize];

            for px in 0..OUT_W {
                let u = (px as f32 + 0.5) / OUT_W as f32;
                let wx = ((u - 0.5) * map.world_width).floor() as i32;

                let col = terrain_gen.sample_column(wx, wz);
                let (eff_h, surface_mat) =
                    terrain_gen.lod_surface_height_and_material(wx, wz, col);

                let (base_col, height, is_water) =
                    if let Some(wl) = col.water_level && col.terrain_height < wl {
                        let depth = (wl - col.terrain_height).clamp(1, 255) as u8;
                        (
                            voxel_map_color_at(Voxel::LiquidWater, depth, wx, wz),
                            wl as i16,
                            true,
                        )
                    } else {
                        (
                            voxel_map_color_at(surface_mat, 0, wx, wz),
                            eff_h as i16,
                            false,
                        )
                    };

                cur_heights[px as usize] = height;
                let north_h = if py > 0 {
                    Some(prev_heights[px as usize])
                } else {
                    None
                };
                let mut c = apply_relief_shading(base_col, height, north_h);

                // Subtle topographic contour lines matching the in-game World Map
                if !is_water && north_h.is_some_and(|nh| (height >> 3) != (nh >> 3)) {
                    c[0] = (c[0] as f32 * 0.90) as u8;
                    c[1] = (c[1] as f32 * 0.90) as u8;
                    c[2] = (c[2] as f32 * 0.90) as u8;
                }

                // Clamp inside exact bounds check
                let _ = (half_w, half_h);
                let idx = ((py * OUT_W + px) * 4) as usize;
                raw[idx] = c[0];
                raw[idx + 1] = c[1];
                raw[idx + 2] = c[2];
                raw[idx + 3] = 255;
            }
            prev_heights = cur_heights;
        }

        let img = image::RgbaImage::from_raw(OUT_W, OUT_H, raw)
            .expect("Valid RGBA buffer for map_preview.png");
        img.save("assets/maps/adventure/map_preview.png")
            .expect("Failed to save assets/maps/adventure/map_preview.png");
        println!("Rendered authentic in-game preview to assets/maps/adventure/map_preview.png ({OUT_W}x{OUT_H})");
    }
}
