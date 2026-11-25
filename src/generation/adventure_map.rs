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
    /// elevated `CalmLake` basins, stepped `Rapids` rivers, and `Estuary` channels.
    water_level_i16: Vec<i16>,
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

/// Computes per-pixel water surface elevation for oceans, estuaries, elevated `CalmLake` basins,
/// and stepped inland `Rapids` rivers.
fn compute_water_levels(
    width: usize,
    height: usize,
    heights_u16: &[u16],
    macro_u8: &[u8],
    micro_u8: &[u8],
    landmass_u8: &[u8],
) -> Vec<i16> {
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
    let river_lake_macro_idx = MacroBiome::ALL
        .iter()
        .position(|&m| m == MacroBiome::RiverAndLake)
        .unwrap_or(0) as u8;

    // 1. Mark open oceans, coastal seas, estuaries, and non-river water pixels at ADVENTURE_SEA_LEVEL.
    for i in 0..n {
        let micro = BiomeType::ALL
            .get(micro_u8[i] as usize)
            .copied()
            .unwrap_or_default();
        if micro_u8[i] == estuary_idx
            || is_ocean_biome(micro)
            || (landmass_u8[i] == 0 && macro_u8[i] != river_lake_macro_idx)
        {
            water_level[i] = ADVENTURE_SEA_LEVEL as i16;
        }
    }

    // 2. BFS flood-fill connected `CalmLake` basins to find each lake's natural rim elevation.
    let mut visited = vec![false; n];
    let mut queue = VecDeque::new();
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
                } else if water_level[n_idx] == ADVENTURE_SEA_LEVEL as i16 {
                    touches_sea = true;
                } else if landmass_u8[n_idx] == 1 && macro_u8[n_idx] != river_lake_macro_idx {
                    let rim_y = raw_u16_to_y(heights_u16[n_idx]).floor() as i32;
                    rim_heights.push(rim_y);
                }
            }
        }

        let lake_y = if touches_sea || rim_heights.is_empty() {
            ADVENTURE_SEA_LEVEL
        } else {
            rim_heights.sort_unstable();
            // Use the 10th percentile rim elevation minus 1 block so the lake sits flat inside its basin
            // without overflowing low banks or collapsing due to a single stray pixel.
            let p10 = rim_heights[(rim_heights.len() / 10).min(rim_heights.len() - 1)];
            (p10 - 1).max(ADVENTURE_SEA_LEVEL)
        } as i16;

        for &idx in &cluster {
            water_level[idx] = lake_y;
        }
    }

    // 3. Initial elevation for inland `Rapids` river pixels from surrounding dry-land banks.
    let mut rapids_indices = Vec::new();
    for i in 0..n {
        if micro_u8[i] != rapids_idx {
            continue;
        }
        rapids_indices.push(i);
        let x = (i % width) as i32;
        let y = (i / width) as i32;

        let mut bank_sum = 0i32;
        let mut bank_count = 0i32;
        let mut min_bank = i32::MAX;
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
                let n_idx = nz as usize * width + nx as usize;
                if landmass_u8[n_idx] == 1 && macro_u8[n_idx] != river_lake_macro_idx {
                    let h = raw_u16_to_y(heights_u16[n_idx]).floor() as i32;
                    bank_sum += h;
                    bank_count += 1;
                    min_bank = min_bank.min(h);
                }
            }
        }

        let initial_y = if bank_count > 0 {
            let avg = bank_sum / bank_count;
            let blended = (min_bank * 2 + avg) / 3;
            (blended - 1).max(ADVENTURE_SEA_LEVEL)
        } else {
            (raw_u16_to_y(heights_u16[i]).floor() as i32).max(ADVENTURE_SEA_LEVEL)
        };
        water_level[i] = initial_y as i16;
    }

    // 4. Smooth `Rapids` river water surface across 6 relaxation passes so steps descend smoothly
    //    into connected lakes and estuaries.
    let mut next_rapids = vec![0i16; rapids_indices.len()];
    for _ in 0..6 {
        for (k, &idx) in rapids_indices.iter().enumerate() {
            let x = idx % width;
            let y = idx / width;
            let cur = water_level[idx] as i32;
            let mut min_nbr = cur;
            let mut sum = cur;
            let mut cnt = 1;

            let neighbors = [
                (x > 0).then(|| idx - 1),
                (x + 1 < width).then(|| idx + 1),
                (y > 0).then(|| idx - width),
                (y + 1 < height).then(|| idx + width),
            ];
            for n_idx in neighbors.into_iter().flatten() {
                let nw = water_level[n_idx];
                if nw != i16::MIN {
                    let nw_i = nw as i32;
                    min_nbr = min_nbr.min(nw_i);
                    sum += nw_i;
                    cnt += 1;
                }
            }

            let avg = sum / cnt;
            // Clamp step height to at most +2 blocks above the lowest adjacent water pixel
            let stepped = avg.min(min_nbr + 2).max(ADVENTURE_SEA_LEVEL);
            next_rapids[k] = stepped as i16;
        }
        for (k, &idx) in rapids_indices.iter().enumerate() {
            water_level[idx] = next_rapids[k];
        }
    }

    water_level
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
            let water_level_i16 = compute_water_levels(
                width as usize,
                height as usize,
                &heights_u16,
                &macro_u8,
                &micro_u8,
                &landmass_u8,
            );

            let mem_mb = (pixel_count * (2 + 1 + 1 + 1 + 1 + 2)) as f32 / (1024.0 * 1024.0);
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

        // Subterranean fallback
        BiomeType::ThermalCaves => (0.85, -0.40),
        BiomeType::UndergroundJungle => (0.65, 0.90),
        BiomeType::IceCaves => (-0.90, 0.10),
        BiomeType::MushroomCaves => (0.10, 0.85),
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

    /// Returns the local water surface elevation `Y` at `(world_x, world_z)` if this column is
    /// covered by ocean, estuary, elevated `CalmLake`, or stepped `Rapids` river water.
    #[inline]
    pub fn sample_water_level(&self, world_x: i32, world_z: i32) -> Option<i32> {
        let Some((fx, fz, _)) = self.world_to_pixel_f(world_x, world_z) else {
            return Some(self.sea_level);
        };
        let idx = self.pixel_index(fx.round() as i32, fz.round() as i32);
        let wl = self.raw.water_level_i16[idx];
        if wl == i16::MIN {
            None
        } else {
            Some(wl as i32)
        }
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
        let local_water_y = self.raw.water_level_i16[base_idx];
        let is_river_or_lake = base_macro == MacroBiome::RiverAndLake;

        // Subtle organic domain-warp (~3-5 blocks) strictly for inland dry-land micro-biome borders,
        // keeping coastlines, oceans, lakes, and river channels razor-sharp.
        let mut biome = base_micro;
        if base_land && !is_river_or_lake {
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
        if base_land && !is_river_or_lake && raw_height > sea_f && raw_height < sea_f + 16.0 {
            let t = (raw_height - sea_f) / 16.0;
            let eased = t * t * 0.65 + t * 0.35;
            raw_height = sea_f + eased * 16.0;
        }

        // Controlled high-frequency micro-detail (attenuated near coasts & rivers)
        let coast_atten = ((raw_height - sea_f).abs() / 6.5).clamp(0.0, 1.0);
        let river_atten = if is_river_or_lake { 0.15 } else { 1.0 };
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
        raw_height += micro * detail_amp * coast_atten * river_atten;

        // Stage 4: Carve elevated CalmLake basins, stepped Rapids rivers, and Estuaries
        // strictly relative to their true local water surface elevation `local_water_y`.
        if is_river_or_lake && local_water_y != i16::MIN {
            let water_y = local_water_y as f32;
            let bed_noise =
                fractal_noise_2d(wx_f, wz_f, 0.08, 2, 0.5, seed ^ 0x71B3_2C4D) * 0.85;
            let target_bed = match base_micro {
                BiomeType::CalmLake => water_y - 4.8 + bed_noise,
                BiomeType::Rapids => water_y - 3.0 + bed_noise * 0.6,
                BiomeType::Estuary => sea_f - 3.2 + bed_noise * 0.6,
                _ => water_y - 3.5 + bed_noise * 0.6,
            };
            raw_height = target_bed.min(water_y - 1.5);
        } else if !base_land && raw_height >= sea_f - 0.5 {
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
            elevated_water_pixels > 100,
            "Expected elevated lakes/rivers above sea level"
        );
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
