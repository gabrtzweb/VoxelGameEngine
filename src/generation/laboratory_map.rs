use std::f32::consts::TAU;

use crate::{
    core::noise::fractal_noise_2d,
    generation::{BiomeType, MacroBiome, biome::ClimateSample},
};

/// Sea level in Laboratory Mode (matches Adventure Mode for crisp coastal & river visuals).
pub const LAB_SEA_LEVEL: i32 = 12;

/// Total world half-extent (`2880 x 2880` blocks) — compact enough to fly across in ~35s,
/// yet spacious enough for every terrestrial biome (~280 x 150..215 blocks) to express full
/// mountain peaks, canyons, plateaus, valleys, and lakes.
pub const LAB_WORLD_RADIUS: f32 = 1440.0;

const R_HUB_LAND: f32 = 82.0;
const R_MOAT_OUTER: f32 = 114.0;
const R_RING1_END: f32 = 394.0;
const R_RING2_END: f32 = 674.0;
const R_TERRESTRIAL_END: f32 = 954.0;
const R_COAST_END: f32 = 1066.0;
const R_ISLAND_ORBIT: f32 = 1265.0;
const R_ISLAND_RADIUS: f32 = 92.0;

/// 6 terrestrial families x 9 biomes each = all 54 terrestrial biomes!
/// Ordered `[Ring1 (2), Ring2 (3), Ring3 (4)]` so large mountain/cliff/canyon biomes sit in
/// Rings 2 & 3 where each cell is ~280 x 185..215 blocks wide.
const TERRESTRIAL_SLICES: [[BiomeType; 9]; 6] = [
    // Slice 0: Lowlands & Wetlands (9)
    [
        BiomeType::Grasslands,
        BiomeType::FlowerMeadow,
        BiomeType::RollingSteppe,
        BiomeType::FertileValley,
        BiomeType::LavenderValley,
        BiomeType::SakuraGrove,
        BiomeType::WillowMarsh,
        BiomeType::MangroveSwamp,
        BiomeType::OrchidSwamp,
    ],
    // Slice 1: Forests (9)
    [
        BiomeType::VerdantWoods,
        BiomeType::AutumnForest,
        BiomeType::MistyWoods,
        BiomeType::PineHighlands,
        BiomeType::BleakTaiga,
        BiomeType::ShieldForest,
        BiomeType::DenseRainforest,
        BiomeType::BambooGrove,
        BiomeType::JungleWetlands,
    ],
    // Slice 2: Mountains & Relief (9)
    [
        BiomeType::EmeraldFoothills,
        BiomeType::WindyPass,
        BiomeType::GranitePlateau,
        BiomeType::MistyCrags,
        BiomeType::AmethystCanyon,
        BiomeType::AlpinePeaks,
        BiomeType::JaggedPeaks,
        BiomeType::MonolithCliffs,
        BiomeType::DeepGorge,
    ],
    // Slice 3: Frozen & Polar (9)
    [
        BiomeType::SnowyPlains,
        BiomeType::ColdShrubland,
        BiomeType::PermafrostSteppe,
        BiomeType::WintryForest,
        BiomeType::FrostPinewoods,
        BiomeType::GlacialChasm,
        BiomeType::IceSheet,
        BiomeType::FrozenCliffs,
        BiomeType::IceboundRidge,
    ],
    // Slice 4: Special & Dark Fantasy (9)
    [
        BiomeType::ScorchedFlats,
        BiomeType::AshenWastes,
        BiomeType::WitheredWoods,
        BiomeType::ObsidianMire,
        BiomeType::AcidPools,
        BiomeType::SulfurVents,
        BiomeType::VolcanicJungle,
        BiomeType::Caldera,
        BiomeType::VolcanicPeaks,
    ],
    // Slice 5: Arid (9)
    [
        BiomeType::AcaciaPlains,
        BiomeType::DesertOasis,
        BiomeType::DuneSea,
        BiomeType::AncientSands,
        BiomeType::FracturedSavanna,
        BiomeType::SavannaBadlands,
        BiomeType::RedrockCanyon,
        BiomeType::PaintedMesas,
        BiomeType::DesertSpires,
    ],
];

/// All 9 Coastal biomes arranged around the outer coastal ring (`954..1066`).
const COASTAL_RING_BIOMES: [BiomeType; 9] = [
    BiomeType::SandyShores,
    BiomeType::DuneBeach,
    BiomeType::GravelBeach,
    BiomeType::CraggyBeach,
    BiomeType::CoastalCliffs,
    BiomeType::TidePools,
    BiomeType::Mudflats,
    BiomeType::SwampyShore,
    BiomeType::ReedyShore,
];

/// All 6 Ocean biomes + matching 6 Island biomes floating in the outer ocean ring (`1066..1440`).
const OCEAN_AND_ISLAND_SECTORS: [(BiomeType, BiomeType); 6] = [
    (BiomeType::WarmOcean, BiomeType::TropicalIsland),
    (BiomeType::CoralReefs, BiomeType::CoralIsland),
    (BiomeType::SeagrassMeadows, BiomeType::TemperateIsland),
    (BiomeType::KelpForests, BiomeType::RockyIsland),
    (BiomeType::ColdOcean, BiomeType::TundraIsland),
    (BiomeType::OceanTrench, BiomeType::FrozenIsland),
];

/// Purely procedural radial "Laboratory" world showcasing all 78 surface biomes in a compact,
/// symmetric circular atlas with meandering radial rivers separating terrestrial slices,
/// a central hub island + moat ring, scenic inland lakes, a coastal ring, and an outer ocean
/// ring with floating island biomes.
pub struct LaboratoryWorldMap {
    pub sea_level: i32,
    pub world_radius: f32,
}

impl Default for LaboratoryWorldMap {
    fn default() -> Self {
        Self {
            sea_level: LAB_SEA_LEVEL,
            world_radius: LAB_WORLD_RADIUS,
        }
    }
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns `(temperature, humidity, continentalness, erosion, ridges)` tailored to each biome
/// so foliage tinting, snow/rain weather, trees, and flora behave authentically.
fn lab_climate_for(biome: BiomeType) -> (f32, f32, f32, f32, f32) {
    match biome {
        // Islands
        BiomeType::TropicalIsland | BiomeType::CoralIsland => (0.82, 0.72, 0.08, 0.25, 0.05),
        BiomeType::TemperateIsland => (0.22, 0.35, 0.08, 0.25, 0.05),
        BiomeType::TundraIsland => (-0.65, -0.10, 0.08, 0.25, 0.05),
        BiomeType::FrozenIsland => (-0.90, -0.35, 0.08, 0.25, 0.05),
        BiomeType::RockyIsland => (-0.05, 0.15, 0.12, -0.15, 0.25),

        // Aquatic
        BiomeType::WarmOcean | BiomeType::CoralReefs | BiomeType::SeagrassMeadows => {
            (0.55, 0.85, -0.45, 0.30, 0.0)
        }
        BiomeType::ColdOcean | BiomeType::KelpForests | BiomeType::OceanTrench => {
            (-0.35, 0.80, -0.55, 0.30, 0.0)
        }
        BiomeType::Rapids | BiomeType::CalmLake | BiomeType::Estuary => {
            (0.18, 0.85, 0.05, 0.35, 0.0)
        }

        // Coastal
        BiomeType::SandyShores | BiomeType::DuneBeach => (0.55, 0.20, 0.02, 0.30, 0.0),
        BiomeType::GravelBeach | BiomeType::CraggyBeach | BiomeType::CoastalCliffs => {
            (-0.10, 0.25, 0.04, -0.10, 0.20)
        }
        BiomeType::TidePools => (0.25, 0.55, 0.01, 0.35, 0.0),
        BiomeType::Mudflats | BiomeType::SwampyShore | BiomeType::ReedyShore => {
            (0.30, 0.85, 0.02, 0.40, 0.0)
        }

        // Special & Dark Fantasy
        BiomeType::Caldera | BiomeType::VolcanicPeaks | BiomeType::ScorchedFlats => {
            (0.90, -0.60, 0.35, -0.35, 0.45)
        }
        BiomeType::VolcanicJungle => (0.85, 0.80, 0.20, 0.10, 0.15),
        BiomeType::AcidPools | BiomeType::SulfurVents => (0.75, 0.25, 0.16, 0.20, 0.10),
        BiomeType::WitheredWoods | BiomeType::AshenWastes | BiomeType::ObsidianMire => {
            (-0.15, 0.05, 0.15, 0.15, 0.10)
        }

        // Lowlands & Wetlands
        BiomeType::Grasslands | BiomeType::FlowerMeadow | BiomeType::LavenderValley => {
            (0.25, 0.30, 0.14, 0.35, 0.05)
        }
        BiomeType::RollingSteppe => (0.45, -0.25, 0.16, 0.30, 0.08),
        BiomeType::FertileValley | BiomeType::SakuraGrove => (0.22, 0.50, 0.14, 0.32, 0.06),
        BiomeType::WillowMarsh | BiomeType::OrchidSwamp => (0.35, 0.90, 0.08, 0.45, 0.0),
        BiomeType::MangroveSwamp => (0.72, 0.95, 0.06, 0.45, 0.0),

        // Forests
        BiomeType::VerdantWoods | BiomeType::AutumnForest | BiomeType::MistyWoods => {
            (0.18, 0.45, 0.16, 0.25, 0.08)
        }
        BiomeType::PineHighlands | BiomeType::BleakTaiga | BiomeType::ShieldForest => {
            (-0.30, 0.35, 0.22, 0.10, 0.15)
        }
        BiomeType::DenseRainforest | BiomeType::BambooGrove | BiomeType::JungleWetlands => {
            (0.82, 0.90, 0.18, 0.25, 0.08)
        }

        // Mountains & Relief
        BiomeType::AlpinePeaks | BiomeType::JaggedPeaks | BiomeType::MistyCrags => {
            (-0.55, 0.15, 0.45, -0.45, 0.65)
        }
        BiomeType::EmeraldFoothills | BiomeType::WindyPass => (-0.15, 0.30, 0.30, -0.20, 0.35),
        BiomeType::GranitePlateau | BiomeType::DeepGorge | BiomeType::MonolithCliffs => {
            (-0.20, 0.05, 0.38, -0.35, 0.50)
        }
        BiomeType::AmethystCanyon => (-0.30, 0.20, 0.32, -0.30, 0.40),

        // Arid
        BiomeType::DuneSea | BiomeType::AncientSands => (0.90, -0.85, 0.18, 0.25, 0.08),
        BiomeType::RedrockCanyon | BiomeType::PaintedMesas | BiomeType::DesertSpires => {
            (0.88, -0.80, 0.32, -0.30, 0.45)
        }
        BiomeType::DesertOasis => (0.80, 0.45, 0.12, 0.35, 0.0),
        BiomeType::AcaciaPlains | BiomeType::FracturedSavanna | BiomeType::SavannaBadlands => {
            (0.75, -0.45, 0.20, 0.10, 0.18)
        }

        // Frozen & Polar
        BiomeType::SnowyPlains | BiomeType::ColdShrubland | BiomeType::PermafrostSteppe => {
            (-0.72, -0.15, 0.16, 0.28, 0.06)
        }
        BiomeType::GlacialChasm | BiomeType::IceSheet | BiomeType::FrozenCliffs => {
            (-0.95, -0.30, 0.32, -0.25, 0.45)
        }
        BiomeType::WintryForest | BiomeType::FrostPinewoods => (-0.62, 0.10, 0.18, 0.22, 0.10),
        BiomeType::IceboundRidge => (-0.82, -0.10, 0.42, -0.40, 0.60),
    }
}

/// Evaluates the authentic local terrain height above `sea_level` for a given biome, modulated
/// by `interior_t` (`0.0` at river/coast edges -> `1.0` inside the biome's core) so mountains,
/// canyons, mesas, and peaks rise dramatically inside their cells without creating harsh walls
/// at slice borders!
fn biome_relief_height(
    biome: BiomeType,
    wx: f32,
    wz: f32,
    seed: u32,
    interior_t: f32,
    sea_f: f32,
) -> f32 {
    let n_broad = fractal_noise_2d(wx, wz, 0.0085, 3, 0.5, seed ^ 0x1A2B_3C4D);
    let n_med = fractal_noise_2d(wx, wz, 0.022, 3, 0.5, seed ^ 0x5E6F_7A8B);
    let n_fine = fractal_noise_2d(wx, wz, 0.055, 2, 0.5, seed ^ 0x9C0D_1E2F);
    let ridge = (1.0 - fractal_noise_2d(wx, wz, 0.012, 3, 0.5, seed ^ 0x3F4E_5D6C).abs()).powi(2);

    let (base_above_sea, relief) = match biome {
        // Towering Alpine & Volcanic Peaks (`Y ~ 65..112` at summit!)
        BiomeType::AlpinePeaks | BiomeType::JaggedPeaks | BiomeType::IceboundRidge => {
            let peak = 28.0 + ridge * 58.0 + n_broad * 14.0 + n_med * 7.0;
            (6.0, peak)
        }
        BiomeType::VolcanicPeaks | BiomeType::Caldera => {
            let crater = if biome == BiomeType::Caldera && n_broad > 0.25 {
                -18.0 * ((n_broad - 0.25) / 0.75)
            } else {
                0.0
            };
            let peak = 24.0 + ridge * 52.0 + n_broad * 12.0 + crater;
            (6.0, peak)
        }
        BiomeType::MonolithCliffs | BiomeType::FrozenCliffs | BiomeType::MistyCrags => {
            let terraced = ((n_broad * 0.5 + 0.5) * 4.0).floor() * 11.0;
            (6.0, 16.0 + terraced + ridge * 26.0 + n_med * 5.0)
        }
        BiomeType::GranitePlateau | BiomeType::PaintedMesas => {
            let plat = smoothstep(-0.15, 0.25, n_broad) * 38.0;
            (5.0, 8.0 + plat + n_med * 3.5 + n_fine * 1.5)
        }
        BiomeType::DeepGorge | BiomeType::AmethystCanyon | BiomeType::RedrockCanyon => {
            let canyon_cut = smoothstep(0.15, 0.45, n_broad.abs()) * 34.0;
            (5.0, 6.0 + canyon_cut + n_med * 6.0)
        }
        BiomeType::DesertSpires | BiomeType::GlacialChasm => {
            let spires = ridge * ridge * 46.0;
            (5.0, 8.0 + spires + n_broad * 10.0 + n_med * 5.0)
        }
        BiomeType::EmeraldFoothills | BiomeType::WindyPass | BiomeType::PineHighlands => {
            (5.0, 12.0 + ridge * 24.0 + n_broad * 11.0 + n_med * 4.5)
        }
        // Wetlands & Low Marshes (low, damp, near water level)
        BiomeType::WillowMarsh
        | BiomeType::MangroveSwamp
        | BiomeType::OrchidSwamp
        | BiomeType::JungleWetlands
        | BiomeType::AcidPools
        | BiomeType::ObsidianMire => (1.8, 1.5 + n_broad * 2.2 + n_med * 1.4),
        // Dunes & Arid Flats
        BiomeType::DuneSea | BiomeType::AncientSands => {
            let dune = ((wx * 0.045 + wz * 0.025 + n_broad * 3.0).sin() * 0.5 + 0.5) * 9.5;
            (4.0, 4.0 + dune + n_med * 2.5)
        }
        BiomeType::SavannaBadlands | BiomeType::FracturedSavanna => {
            let step = smoothstep(-0.1, 0.3, n_broad) * 18.0;
            (4.0, 5.0 + step + n_med * 3.5)
        }
        // Standard Plains, Valleys, Forests, Tundra
        _ => (4.5, 6.5 + n_broad * 9.0 + n_med * 3.5 + n_fine * 1.5),
    };

    sea_f + base_above_sea + relief * interior_t
}

/// Internal classification for a world `(wx, wz)` point in the Laboratory disc.
struct LabPoint {
    biome: BiomeType,
    /// `Some(water_y)` if submerged in moat, radial river, estuary, inland lake, or ocean.
    water_level: Option<i32>,
    /// `Some((is_river_bank, is_lake_bank, factor))` on dry bank transition strips.
    bank: Option<(bool, bool, f32)>,
    /// `0.0` at river/moat/coast edge -> `1.0` inside biome core.
    interior_t: f32,
    /// Target height if water/coastal/island overrides standard terrestrial relief.
    override_height: Option<f32>,
}

impl LaboratoryWorldMap {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn half_extents(&self) -> (f32, f32) {
        (self.world_radius, self.world_radius)
    }

    #[inline]
    pub fn world_dimensions(&self) -> (f32, f32) {
        (self.world_radius * 2.0, self.world_radius * 2.0)
    }

    #[inline]
    pub fn fingerprint(&self) -> u64 {
        0x1AB0_7A70_7100_0001u64 ^ ((self.world_radius.to_bits() as u64) << 12)
    }

    #[inline]
    pub fn spawn_world_xz(&self) -> (i32, i32) {
        (0, 0)
    }

    /// Evaluates the exact polar zone, river divider, inland lake, coastal arc, ocean sector,
    /// or floating island at `(world_x, world_z)`.
    fn classify(&self, world_x: i32, world_z: i32) -> LabPoint {
        const MEANDER_SEED: u32 = 0x1AB0_4E11;
        const SHAPE_SEED: u32 = 0x1AB0_9C33;

        let wx = world_x as f32;
        let wz = world_z as f32;
        let sea_f = self.sea_level as f32;

        let r = wx.hypot(wz);
        let raw_angle = wz.atan2(wx).rem_euclid(TAU);

        // Gentle organic wobble on concentric boundaries (~+-10 blocks)
        let ring_wobble = fractal_noise_2d(wx, wz, 0.015, 2, 0.5, SHAPE_SEED) * 10.5;
        let r_org = r + ring_wobble;

        // 1. Central Hub Island (spawn at 0, 0)
        if r_org < R_HUB_LAND {
            let edge_t = smoothstep(0.0, 22.0, R_HUB_LAND - r_org);
            let hub_biome = if fractal_noise_2d(wx, wz, 0.035, 2, 0.5, SHAPE_SEED ^ 0x44) > 0.15 {
                BiomeType::FlowerMeadow
            } else {
                BiomeType::Grasslands
            };
            let bank = if R_HUB_LAND - r_org < 7.0 {
                Some((false, true, 1.0 - (R_HUB_LAND - r_org) / 7.0))
            } else {
                None
            };
            let h = sea_f + 1.4 + edge_t * 5.0
                + fractal_noise_2d(wx, wz, 0.03, 2, 0.5, SHAPE_SEED ^ 0x11) * 1.6 * edge_t;
            return LabPoint {
                biome: hub_biome,
                water_level: None,
                bank,
                interior_t: edge_t,
                override_height: Some(h),
            };
        }

        // 2. Central Circular Moat Ring (`CalmLake`, like Image 3!)
        if r_org < R_MOAT_OUTER {
            let mid = (R_HUB_LAND + R_MOAT_OUTER) * 0.5;
            let half_w = (R_MOAT_OUTER - R_HUB_LAND) * 0.5;
            let dist_from_center = (r_org - mid).abs() / half_w;
            let depth = 1.2 + (1.0 - dist_from_center * dist_from_center).max(0.0) * 5.5;
            return LabPoint {
                biome: BiomeType::CalmLake,
                water_level: Some(self.sea_level),
                bank: None,
                interior_t: 0.0,
                override_height: Some(sea_f - depth),
            };
        }

        // 3. Six Terrestrial Radial Slices (`114..954`) separated by meandering `Rapids` rivers
        if r_org < R_TERRESTRIAL_END {
            let slice_span = TAU / 6.0; // 60 degrees
            // Gentle meandering river centerline along each 60° slice seam
            let meander_blocks = fractal_noise_2d(wx, wz, 0.011, 2, 0.5, MEANDER_SEED) * 20.0;
            let meander_angle = meander_blocks / r.max(110.0);
            let adj_angle = (raw_angle + meander_angle).rem_euclid(TAU);

            let slice_f = adj_angle / slice_span;
            let slice_idx = (slice_f.floor() as usize).min(5);
            let local_u = slice_f - slice_f.floor(); // 0.0..1.0 within this 60° slice

            // Arc distance in blocks to the nearest slice-dividing river seam
            let angle_to_seam = local_u.min(1.0 - local_u) * slice_span;
            let dist_to_river = angle_to_seam * r;

            // River widens smoothly from ~11 blocks half-width inland to ~22 blocks at the estuary mouth
            let estuary_t = smoothstep(R_TERRESTRIAL_END - 85.0, R_TERRESTRIAL_END, r_org);
            let river_half_w = 10.5 + estuary_t * 12.0;

            if dist_to_river < river_half_w {
                let norm = (dist_to_river / river_half_w).clamp(0.0, 1.0);
                let depth = 1.0 + (1.0 - norm * norm) * (3.2 + estuary_t * 2.2);
                let river_biome = if estuary_t > 0.35 {
                    BiomeType::Estuary
                } else {
                    BiomeType::Rapids
                };
                return LabPoint {
                    biome: river_biome,
                    water_level: Some(self.sea_level),
                    bank: None,
                    interior_t: 0.0,
                    override_height: Some(sea_f - depth),
                };
            }

            // Pick the micro-biome inside this slice:
            // Ring 1 (`114..394`): 2 sub-sectors (indices 0..2)
            // Ring 2 (`394..674`): 3 sub-sectors (indices 2..5)
            // Ring 3 (`674..954`): 4 sub-sectors (indices 5..9)
            let biome_wobble =
                fractal_noise_2d(wx, wz, 0.025, 2, 0.5, SHAPE_SEED ^ 0x77) * 0.04;
            let u_org = (local_u + biome_wobble).clamp(0.0, 0.9999);
            let sub_idx = if r_org < R_RING1_END {
                (u_org * 2.0).floor() as usize
            } else if r_org < R_RING2_END {
                2 + (u_org * 3.0).floor() as usize
            } else {
                5 + (u_org * 4.0).floor() as usize
            };
            let biome = TERRESTRIAL_SLICES[slice_idx][sub_idx.min(8)];

            // Scenic CalmLake pocket nestled in the middle of Slices 0 (Lowlands) & 1 (Forests)
            // around `(r = 394, local_u = 0.5)` so freshwater lakes are showcased right inside land!
            if slice_idx == 0 || slice_idx == 1 {
                let lake_angle = (slice_idx as f32 + 0.5) * slice_span;
                let lx = lake_angle.cos() * 394.0;
                let lz = lake_angle.sin() * 394.0;
                let lake_wob = fractal_noise_2d(wx, wz, 0.04, 2, 0.5, SHAPE_SEED ^ 0x33) * 12.0;
                let ldist = (wx - lx).hypot(wz - lz) + lake_wob;
                if ldist < 42.0 {
                    let norm = (ldist / 42.0).clamp(0.0, 1.0);
                    let depth = 1.1 + (1.0 - norm * norm) * 7.8;
                    return LabPoint {
                        biome: BiomeType::CalmLake,
                        water_level: Some(self.sea_level),
                        bank: None,
                        interior_t: 0.0,
                        override_height: Some(sea_f - depth),
                    };
                }
            }

            // Smooth distance-to-water interior factor so peaks rise cleanly away from rivers/moat/coast
            let d_moat = r_org - R_MOAT_OUTER;
            let d_coast = R_TERRESTRIAL_END - r_org;
            let d_riv = dist_to_river - river_half_w;
            let min_water_d = d_moat.min(d_coast).min(d_riv);
            let interior_t = smoothstep(0.0, 38.0, min_water_d);

            let bank = if d_riv < 6.0 {
                Some((true, false, (1.0 - d_riv / 6.0).clamp(0.0, 1.0)))
            } else if d_moat < 6.0 {
                Some((false, true, (1.0 - d_moat / 6.0).clamp(0.0, 1.0)))
            } else {
                None
            };

            return LabPoint {
                biome,
                water_level: None,
                bank,
                interior_t,
                override_height: None,
            };
        }

        // 4. Outer Coastal Ring (`954..1066`): all 9 Coastal biomes in 9 equal 40° arcs!
        if r_org < R_COAST_END {
            let coast_span = TAU / 9.0;
            let coast_idx = ((raw_angle / coast_span).floor() as usize).min(8);
            let biome = COASTAL_RING_BIOMES[coast_idx];

            // Keep the 6 radial estuaries flowing cleanly through the coastal ring into the ocean!
            let slice_span = TAU / 6.0;
            let meander_blocks = fractal_noise_2d(wx, wz, 0.011, 2, 0.5, MEANDER_SEED) * 20.0;
            let adj_angle = (raw_angle + meander_blocks / r.max(110.0)).rem_euclid(TAU);
            let local_u = (adj_angle / slice_span).fract();
            let dist_to_river = local_u.min(1.0 - local_u) * slice_span * r;
            let mouth_w = 22.0 + smoothstep(R_TERRESTRIAL_END, R_COAST_END, r_org) * 18.0;
            if dist_to_river < mouth_w {
                let norm = (dist_to_river / mouth_w).clamp(0.0, 1.0);
                let depth = 1.2 + (1.0 - norm * norm) * 4.8;
                return LabPoint {
                    biome: BiomeType::Estuary,
                    water_level: Some(self.sea_level),
                    bank: None,
                    interior_t: 0.0,
                    override_height: Some(sea_f - depth),
                };
            }

            // Smooth coastal slope down from inland height to sea level at `R_COAST_END`
            let t_sea = ((r_org - R_TERRESTRIAL_END) / (R_COAST_END - R_TERRESTRIAL_END))
                .clamp(0.0, 1.0);
            let h = if biome == BiomeType::CoastalCliffs {
                let cliff = (1.0 - smoothstep(0.65, 0.92, t_sea)) * 22.0;
                sea_f - 1.5 + cliff
                    + fractal_noise_2d(wx, wz, 0.035, 2, 0.5, SHAPE_SEED ^ 0x55) * 3.5
            } else if matches!(
                biome,
                BiomeType::TidePools | BiomeType::Mudflats | BiomeType::SwampyShore
            ) {
                let low = (1.0 - t_sea) * 2.8 - 1.2;
                sea_f + low + fractal_noise_2d(wx, wz, 0.05, 2, 0.5, SHAPE_SEED ^ 0x66) * 1.6
            } else {
                let beach = (1.0 - smoothstep(0.0, 0.92, t_sea)) * 7.5 - 1.5;
                sea_f + beach + fractal_noise_2d(wx, wz, 0.03, 2, 0.5, SHAPE_SEED ^ 0x77) * 1.8
            };

            let water_level = if h < sea_f {
                Some(self.sea_level)
            } else {
                None
            };
            return LabPoint {
                biome,
                water_level,
                bank: None,
                interior_t: 1.0 - t_sea,
                override_height: Some(h),
            };
        }

        // 5. Outer Ocean Ring (`1066..1440`) + 6 Floating Island Biomes!
        let sector_span = TAU / 6.0;
        let sector_idx = ((raw_angle / sector_span).floor() as usize).min(5);
        let (ocean_biome, island_biome) = OCEAN_AND_ISLAND_SECTORS[sector_idx];

        // Check distance to this sector's floating Island centered at `(R_ISLAND_ORBIT, center_angle)`
        let island_angle = (sector_idx as f32 + 0.5) * sector_span;
        let ix = island_angle.cos() * R_ISLAND_ORBIT;
        let iz = island_angle.sin() * R_ISLAND_ORBIT;
        let island_wobble = fractal_noise_2d(wx, wz, 0.025, 3, 0.5, SHAPE_SEED ^ 0x99) * 18.0;
        let idist = (wx - ix).hypot(wz - iz) + island_wobble;

        if idist < R_ISLAND_RADIUS {
            let norm = (idist / R_ISLAND_RADIUS).clamp(0.0, 1.0);
            let dome = (1.0 - norm * norm).powf(1.25);
            let peak_h = if island_biome == BiomeType::RockyIsland {
                24.0
            } else {
                13.0
            };
            let h = sea_f - 0.8 + dome * peak_h
                + fractal_noise_2d(wx, wz, 0.04, 2, 0.5, SHAPE_SEED ^ 0xAA) * 2.5 * dome;
            let water_level = if h < sea_f {
                Some(self.sea_level)
            } else {
                None
            };
            return LabPoint {
                biome: island_biome,
                water_level,
                bank: None,
                interior_t: dome,
                override_height: Some(h),
            };
        }

        // Open ocean floor depth tailored per ocean biome
        let sea_t = smoothstep(R_COAST_END, R_COAST_END + 65.0, r_org);
        let floor_noise = fractal_noise_2d(wx, wz, 0.025, 2, 0.5, SHAPE_SEED ^ 0xBB) * 3.0;
        let base_depth = match ocean_biome {
            BiomeType::CoralReefs | BiomeType::SeagrassMeadows => 6.5,
            BiomeType::KelpForests => 11.5,
            BiomeType::WarmOcean | BiomeType::ColdOcean => 16.5,
            BiomeType::OceanTrench => 36.0,
            _ => 14.0,
        };
        let depth = 2.0 + sea_t * base_depth + floor_noise;
        LabPoint {
            biome: ocean_biome,
            water_level: Some(self.sea_level),
            bank: None,
            interior_t: 0.0,
            override_height: Some(sea_f - depth),
        }
    }

    #[inline]
    pub fn sample_water_level(&self, world_x: i32, world_z: i32) -> Option<i32> {
        self.classify(world_x, world_z).water_level
    }

    #[inline]
    pub fn sample_inland_bank(&self, world_x: i32, world_z: i32) -> Option<(bool, bool, f32)> {
        self.classify(world_x, world_z).bank
    }

    #[inline]
    pub fn sample_biome_and_macro(&self, world_x: i32, world_z: i32) -> (BiomeType, MacroBiome) {
        let b = self.classify(world_x, world_z).biome;
        (b, b.parent_macro_biome().unwrap_or_default())
    }

    pub fn sample_column_data(
        &self,
        world_x: i32,
        world_z: i32,
        seed: u32,
    ) -> (f32, BiomeType, bool, ClimateSample) {
        let pt = self.classify(world_x, world_z);
        let sea_f = self.sea_level as f32;
        let raw_height = match pt.override_height {
            Some(h) => h,
            None => biome_relief_height(
                pt.biome,
                world_x as f32,
                world_z as f32,
                seed,
                pt.interior_t,
                sea_f,
            ),
        };
        let (temp, humid, cont, eros, ridges) = lab_climate_for(pt.biome);
        let climate = ClimateSample {
            continentalness: cont,
            erosion: eros,
            ridges,
            temperature: temp,
            humidity: humid,
            depth: (sea_f - raw_height).max(0.0),
            biome: pt.biome,
        };
        (raw_height, pt.biome, false, climate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn laboratory_map_contains_all_78_biomes() {
        let mut seen = HashSet::new();
        for slice in TERRESTRIAL_SLICES {
            for b in slice {
                seen.insert(b);
            }
        }
        for b in COASTAL_RING_BIOMES {
            seen.insert(b);
        }
        for (o, i) in OCEAN_AND_ISLAND_SECTORS {
            seen.insert(o);
            seen.insert(i);
        }
        seen.insert(BiomeType::Rapids);
        seen.insert(BiomeType::CalmLake);
        seen.insert(BiomeType::Estuary);

        for b in BiomeType::ALL {
            assert!(
                seen.contains(&b),
                "Biome {b:?} is missing from LaboratoryWorldMap!"
            );
        }
        assert_eq!(seen.len(), BiomeType::ALL.len());
    }
}
