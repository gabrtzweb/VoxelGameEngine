use bevy::prelude::*;

use crate::{core::noise::fbm_2d, world::Voxel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum BiomeType {
    #[default]
    Plains,
    Meadow,
    Woodland,
    Wetlands,
    Highlands,
    SnowyTundra,
    Desert,
    Beach,
    River,
    Ocean,
    DeepOcean,
}

impl BiomeType {
    #[allow(dead_code)]
    pub const ALL: [BiomeType; 11] = [
        BiomeType::Plains,
        BiomeType::Meadow,
        BiomeType::Woodland,
        BiomeType::Wetlands,
        BiomeType::Highlands,
        BiomeType::SnowyTundra,
        BiomeType::Desert,
        BiomeType::Beach,
        BiomeType::River,
        BiomeType::Ocean,
        BiomeType::DeepOcean,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BiomeType::Plains => "Plains",
            BiomeType::Meadow => "Meadow",
            BiomeType::Woodland => "Woodland",
            BiomeType::Wetlands => "Wetlands",
            BiomeType::Highlands => "Highlands",
            BiomeType::SnowyTundra => "Snowy Tundra",
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
                surface_material: Voxel::Snow,
                subsoil_material: Voxel::Dirt,
                subsoil_depth: 2,
                base_height_offset: 24.0,
                amplitude_multiplier: 2.6,
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
                subsoil_material: Voxel::Gravel,
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
                subsoil_material: Voxel::Gravel,
                subsoil_depth: 3,
                base_height_offset: -14.0,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Stone,
                cliff_material: Voxel::Stone,
            },
            BiomeType::DeepOcean => BiomeConfig {
                biome_type: BiomeType::DeepOcean,
                name: "Deep Ocean",
                surface_material: Voxel::Gravel,
                subsoil_material: Voxel::Blackstone,
                subsoil_depth: 3,
                base_height_offset: -26.0,
                amplitude_multiplier: 0.5,
                primary_stone: Voxel::Blackstone,
                cliff_material: Voxel::Blackstone,
            },
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
            continentalness_freq: 0.0025,
            temperature_freq: 0.0018,
            humidity_freq: 0.0022,
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
        // 5. Extreme cold or high frozen peaks
        else if temperature < -0.22 || (continentalness > 0.65 && temperature < 0.10) {
            BiomeType::SnowyTundra
        }
        // 6. High continentalness creates rugged mountain highlands
        else if continentalness > 0.40 {
            BiomeType::Highlands
        }
        // 7. Hot and dry creates desert dunes
        else if humidity < -0.18 && temperature > 0.15 {
            BiomeType::Desert
        }
        // 8. Low inland elevation with high moisture creates wetlands/swamps
        else if continentalness < 0.15 && humidity > 0.25 {
            BiomeType::Wetlands
        }
        // 9. High moisture inland creates rich woodland
        else if humidity > 0.18 {
            BiomeType::Woodland
        }
        // 10. Gentle transition meadow
        else if humidity > 0.02 {
            BiomeType::Meadow
        }
        // 11. Default temperate rolling plains
        else {
            BiomeType::Plains
        }
    }
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

        for expected in BiomeType::ALL {
            assert!(
                found.contains(&expected),
                "Missing classification for biome: {:?}",
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
}
