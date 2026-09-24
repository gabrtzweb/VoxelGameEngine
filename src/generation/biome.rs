use bevy::prelude::*;

use crate::{core::noise::fbm_2d, world::Voxel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum BiomeType {
    #[default]
    Plains,
    PlainsForest,
    Meadow,
    Woodland,
    Wetlands,
    Highlands,
    SnowyTundra,
    ColdPlains,
    Savanna,
    Desert,
    Beach,
    River,
    Ocean,
    DeepOcean,
}

impl BiomeType {
    #[allow(dead_code)]
    pub const ALL: [BiomeType; 14] = [
        BiomeType::Plains,
        BiomeType::PlainsForest,
        BiomeType::Meadow,
        BiomeType::Woodland,
        BiomeType::Wetlands,
        BiomeType::Highlands,
        BiomeType::SnowyTundra,
        BiomeType::ColdPlains,
        BiomeType::Savanna,
        BiomeType::Desert,
        BiomeType::Beach,
        BiomeType::River,
        BiomeType::Ocean,
        BiomeType::DeepOcean,
    ];

    #[allow(dead_code)]
    pub const ACTIVE: [BiomeType; 14] = Self::ALL;

    pub fn name(self) -> &'static str {
        match self {
            BiomeType::Plains => "Plains",
            BiomeType::PlainsForest => "Plains Forest",
            BiomeType::Meadow => "Meadow",
            BiomeType::Woodland => "Woodland",
            BiomeType::Wetlands => "Wetlands",
            BiomeType::Highlands => "Highlands",
            BiomeType::SnowyTundra => "Snowy Tundra",
            BiomeType::ColdPlains => "Cold Plains",
            BiomeType::Savanna => "Savanna",
            BiomeType::Desert => "Desert",
            BiomeType::Beach => "Beach",
            BiomeType::River => "River",
            BiomeType::Ocean => "Ocean",
            BiomeType::DeepOcean => "Deep Ocean",
        }
    }

    pub fn config(self) -> BiomeConfig {
        match self {
            BiomeType::Plains => BiomeConfig {
                biome_type: BiomeType::Plains,
                name: "Plains",
                surface_material: Voxel::Grass,
                subsoil_material: Voxel::Dirt,
                subsoil_depth: 3,
                base_height_offset: 0.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
            BiomeType::PlainsForest => BiomeConfig {
                biome_type: BiomeType::PlainsForest,
                name: "Plains Forest",
                surface_material: Voxel::Grass,
                subsoil_material: Voxel::Dirt,
                subsoil_depth: 3,
                base_height_offset: 0.5,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
            BiomeType::Meadow => BiomeConfig {
                biome_type: BiomeType::Meadow,
                name: "Meadow",
                surface_material: Voxel::Grass,
                subsoil_material: Voxel::Dirt,
                subsoil_depth: 3,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
            BiomeType::Desert => BiomeConfig {
                biome_type: BiomeType::Desert,
                name: "Desert",
                surface_material: Voxel::Sand,
                subsoil_material: Voxel::Sand,
                subsoil_depth: 4,
                base_height_offset: 3.0,
                amplitude_multiplier: 1.5,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Sandstone,
            },
            BiomeType::SnowyTundra => BiomeConfig {
                biome_type: BiomeType::SnowyTundra,
                name: "Snowy Tundra",
                surface_material: Voxel::SnowyGrass,
                subsoil_material: Voxel::Dirt,
                subsoil_depth: 2,
                base_height_offset: 24.0,
                amplitude_multiplier: 2.6,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
            BiomeType::ColdPlains => BiomeConfig {
                biome_type: BiomeType::ColdPlains,
                name: "Cold Plains",
                surface_material: Voxel::Grass,
                subsoil_material: Voxel::Dirt,
                subsoil_depth: 3,
                base_height_offset: 1.0,
                amplitude_multiplier: 1.0,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
            BiomeType::Savanna => BiomeConfig {
                biome_type: BiomeType::Savanna,
                name: "Savanna",
                surface_material: Voxel::Grass,
                subsoil_material: Voxel::Dirt,
                subsoil_depth: 3,
                base_height_offset: 1.5,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
            BiomeType::Wetlands => BiomeConfig {
                biome_type: BiomeType::Wetlands,
                name: "Wetlands",
                surface_material: Voxel::Grass,
                subsoil_material: Voxel::PackedMud,
                subsoil_depth: 3,
                base_height_offset: -3.0,
                amplitude_multiplier: 0.35,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Clay,
            },
            BiomeType::Highlands => BiomeConfig {
                biome_type: BiomeType::Highlands,
                name: "Highlands",
                surface_material: Voxel::Slate,
                subsoil_material: Voxel::Cobbleslate,
                subsoil_depth: 2,
                base_height_offset: 32.0,
                amplitude_multiplier: 3.4,
                primary_stone: Voxel::Slate,
                cliff_material: Voxel::Slate,
            },
            BiomeType::Woodland => BiomeConfig {
                biome_type: BiomeType::Woodland,
                name: "Woodland",
                surface_material: Voxel::Grass,
                subsoil_material: Voxel::PackedDirt,
                subsoil_depth: 3,
                base_height_offset: 5.0,
                amplitude_multiplier: 1.3,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::MossyStone,
            },
            BiomeType::Beach => BiomeConfig {
                biome_type: BiomeType::Beach,
                name: "Beach",
                surface_material: Voxel::Sand,
                subsoil_material: Voxel::Sand,
                subsoil_depth: 4,
                base_height_offset: -1.0,
                amplitude_multiplier: 0.4,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Sandstone,
            },
            BiomeType::River => BiomeConfig {
                biome_type: BiomeType::River,
                name: "River",
                surface_material: Voxel::Sand,
                subsoil_material: Voxel::Dirt,
                subsoil_depth: 2,
                base_height_offset: -5.0,
                amplitude_multiplier: 0.3,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
            BiomeType::Ocean => BiomeConfig {
                biome_type: BiomeType::Ocean,
                name: "Ocean",
                surface_material: Voxel::Sand,
                subsoil_material: Voxel::Sand,
                subsoil_depth: 3,
                base_height_offset: -14.0,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
            BiomeType::DeepOcean => BiomeConfig {
                biome_type: BiomeType::DeepOcean,
                name: "Deep Ocean",
                surface_material: Voxel::Sand,
                subsoil_material: Voxel::Sand,
                subsoil_depth: 3,
                base_height_offset: -26.0,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
        }
    }

    /// Calibrated grass tint color per biome.
    pub fn grass_color(self) -> [f32; 4] {
        match self {
            Self::Plains => [0.55, 0.94, 0.42, 1.0], // Vibrant, lively emerald
            Self::PlainsForest => [0.46, 0.88, 0.38, 1.0], // Rich lush forest
            Self::Meadow => [0.52, 0.98, 0.46, 1.0], // Ultra-vivid sunny green
            Self::Woodland => [0.40, 0.82, 0.34, 1.0], // Deep canopy green
            Self::Wetlands => [0.42, 0.68, 0.36, 1.0], // Murky swamp olive
            Self::Highlands => [0.48, 0.84, 0.52, 1.0], // Alpine cool muted sage
            Self::SnowyTundra => [0.52, 0.80, 0.70, 1.0], // Glacial frosty pale blue-green
            Self::ColdPlains => [0.52, 0.84, 0.50, 1.0], // Cold subpolar muted green
            Self::Savanna => [0.68, 0.80, 0.35, 1.0], // Warm dry golden-olive
            Self::Desert => [0.72, 0.76, 0.38, 1.0], // Sun-baked arid olive
            Self::Beach => [0.65, 0.86, 0.45, 1.0],  // Sandy coastal green
            Self::River => [0.50, 0.92, 0.44, 1.0],  // Fresh lush riverbank
            Self::Ocean | Self::DeepOcean => [0.40, 0.82, 0.62, 1.0], // Aquatic muted turquoise
        }
    }

    /// Calibrated water tint color per biome.
    pub fn water_color(self) -> [f32; 4] {
        match self {
            Self::DeepOcean => [0.15, 0.32, 0.70, 1.0], // Dark marine navy abyss
            Self::Ocean => [0.24, 0.48, 0.88, 1.0],     // Deep ocean blue
            Self::Beach => [0.28, 0.74, 0.92, 1.0],     // Tropical turquoise coastal
            Self::River => [0.32, 0.68, 0.96, 1.0],     // Crystal clear river
            Self::Wetlands => [0.30, 0.55, 0.48, 1.0],  // Murky swamp teal
            Self::Desert => [0.22, 0.80, 0.88, 1.0],    // Vivid warm oasis aqua
            Self::ColdPlains => [0.42, 0.68, 0.94, 1.0], // Frigid pale blue
            Self::SnowyTundra => [0.48, 0.74, 0.98, 1.0], // Crisp glacial cyan-blue
            Self::Highlands => [0.30, 0.64, 0.95, 1.0], // Pristine alpine blue
            _ => [0.35, 0.65, 0.92, 1.0],               // Balanced temperate water
        }
    }

    /// Calibrated foliage (leaves) tint color per biome.
    pub fn foliage_color(self, voxel: Voxel) -> [f32; 4] {
        match voxel {
            Voxel::OakLeaves => match self {
                Self::Plains | Self::Meadow => [0.60, 1.15, 0.35, 1.0],
                Self::Woodland | Self::PlainsForest => [0.52, 1.05, 0.32, 1.0],
                Self::Wetlands => [0.45, 0.85, 0.30, 1.0],
                Self::Desert | Self::Savanna => [0.70, 1.00, 0.32, 1.0],
                Self::SnowyTundra | Self::ColdPlains => [0.48, 0.95, 0.50, 1.0],
                _ => [0.60, 1.15, 0.35, 1.0],
            },
            Voxel::BirchLeaves => [0.85, 1.25, 0.40, 1.0],
            Voxel::PineLeaves => match self {
                Self::SnowyTundra | Self::ColdPlains => [0.35, 0.82, 0.60, 1.0],
                _ => [0.40, 0.90, 0.55, 1.0],
            },
            Voxel::RainwoodLeaves => [0.45, 1.10, 0.65, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Primary tint color for a voxel in this biome.
    pub fn voxel_tint(self, voxel: Voxel) -> [f32; 4] {
        match voxel {
            Voxel::Grass => self.grass_color(),
            Voxel::Water | Voxel::WaterFlowing | Voxel::WaterOccupied => self.water_color(),
            Voxel::OakLeaves | Voxel::BirchLeaves | Voxel::PineLeaves | Voxel::RainwoodLeaves => {
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
}

impl Default for ClimateGenerator {
    fn default() -> Self {
        Self {
            continentalness_freq: 0.0012,
            temperature_freq: 0.0012,
            humidity_freq: 0.0016,
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

    pub fn classify_biome(continentalness: f32, temperature: f32, humidity: f32) -> BiomeType {
        // 1. Deep Oceanic Abyss
        if continentalness < -0.45 {
            BiomeType::DeepOcean
        }
        // 2. Open Ocean
        else if continentalness < -0.20 {
            BiomeType::Ocean
        }
        // 3. Coastal Beaches
        else if continentalness < -0.05 {
            BiomeType::Beach
        }
        // 4. Coastal river estuaries
        else if continentalness < 0.03 && humidity > 0.18 {
            BiomeType::River
        }
        // --- Land biomes arranged smoothly across Continentalness, Temperature & Humidity ---
        // 5. Frigid / Glacial: Snowy Tundra
        else if temperature < -0.20 {
            BiomeType::SnowyTundra
        }
        // 6. Cold sub-polar transition: Cold Plains
        else if temperature < -0.05 {
            BiomeType::ColdPlains
        }
        // 7. Mountain Highlands: inland elevated crags (high continentalness, cool/temperate)
        else if continentalness > 0.35 && temperature < 0.25 {
            BiomeType::Highlands
        }
        // 8. Warm & Arid: Desert (strictly warm and dry)
        else if temperature > 0.20 && humidity < -0.05 {
            BiomeType::Desert
        }
        // 9. Warm sub-tropical: Savanna
        else if temperature > 0.20 {
            BiomeType::Savanna
        }
        // 10. Temperate Wetlands: lowlands with high moisture
        else if humidity > 0.25 {
            BiomeType::Wetlands
        }
        // 11. Temperate Woodland: rich canopy forest with high-moderate moisture
        else if humidity > 0.10 {
            BiomeType::Woodland
        }
        // 12. Temperate Plains Forest: transitional forest-plains
        else if humidity > -0.05 {
            BiomeType::PlainsForest
        }
        // 13. Meadow: lush open upland hills with low-moderate humidity
        else if continentalness > 0.15 {
            BiomeType::Meadow
        }
        // 14. Temperate core: Plains
        else {
            BiomeType::Plains
        }
    }
}

/// Smoothly blends biome colors across biome transitions using a 13-point circular Gaussian kernel
/// (covering an organic 20-block transition zone) and quantizes the result into 128 discrete steps
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
    fn biome_classification_covers_all_variants() {
        let mut found = std::collections::HashSet::new();

        for c in -10..=10 {
            for t in -10..=10 {
                for h in -10..=10 {
                    let cont = c as f32 / 10.0;
                    let temp = t as f32 / 10.0;
                    let hum = h as f32 / 10.0;
                    let biome = ClimateGenerator::classify_biome(cont, temp, hum);
                    found.insert(biome);
                }
            }
        }

        for expected in BiomeType::ACTIVE {
            assert!(
                found.contains(&expected),
                "Missing classification for active biome: {:?}",
                expected
            );
        }
    }

    #[test]
    fn climate_generator_is_deterministic() {
        let generator = ClimateGenerator::default();
        let s1 = generator.sample(150.0, -250.0, 1337);
        let s2 = generator.sample(150.0, -250.0, 1337);
        assert_eq!(s1, s2);
    }

    #[test]
    fn biome_colors_differ_between_climates() {
        let plains_grass = BiomeType::Plains.grass_color();
        let desert_grass = BiomeType::Desert.grass_color();
        let tundra_grass = BiomeType::SnowyTundra.grass_color();

        // Desert grass is warmer and yellower (higher red, lower green) than vibrant Plains grass
        assert!(desert_grass[0] > plains_grass[0]);
        // Tundra grass is cold and desaturated (higher blue/cyan tone)
        assert!(tundra_grass[2] > plains_grass[2]);

        let ocean_water = BiomeType::Ocean.water_color();
        let desert_water = BiomeType::Desert.water_color();
        let swamp_water = BiomeType::Wetlands.water_color();

        // Desert oasis water is vibrant turquoise (high green/blue)
        assert!(desert_water[1] > ocean_water[1]);
        // Swamp water is murky/greenish
        assert!(swamp_water[1] > swamp_water[0]);
    }

    #[test]
    fn blended_biome_color_is_smooth_and_deterministic() {
        let c1 = sample_blended_biome_color(Voxel::Grass, 100.0, 200.0, 1337);
        let c2 = sample_blended_biome_color(Voxel::Grass, 100.0, 200.0, 1337);
        assert_eq!(c1, c2);

        let water = sample_blended_biome_color(Voxel::Water, 0.0, 0.0, 1337);
        assert!(water[3] > 0.99);

        let stone = sample_blended_biome_color(Voxel::Stone, 50.0, 50.0, 1337);
        assert_eq!(stone, [1.0, 1.0, 1.0, 1.0]);
    }
}
