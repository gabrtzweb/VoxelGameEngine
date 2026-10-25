use bevy::prelude::*;

use crate::{
    core::noise::{Simd4f, fbm_2d, fbm_2d_x4, gradient_noise_2d_x4},
    world::Voxel,
};

/// Canonical 11-biome catalog (6 Terrestrial, 5 Aquatic) according to `block_replacement_plan.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum BiomeType {
    // Terrestrial Biomes (6)
    #[default]
    Plains,
    Forest,
    Desert,
    Taiga,
    Highlands,
    Savanna,

    // Aquatic & Marine Biomes (5)
    CoastalBeachReef,
    OpenOcean,
    AbyssalTrench,
    RiversLakes,
    MangroveWetland,
}

impl BiomeType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Plains => "Temperate Plains",
            Self::Forest => "Dense Temperate Forest",
            Self::Desert => "Arid Desert & Badlands",
            Self::Taiga => "Snowy Coniferous Taiga",
            Self::Highlands => "Rocky Highlands / Mountain Tundra",
            Self::Savanna => "Tropical Savanna",
            Self::CoastalBeachReef => "Shallow Coastal Beach & Reef",
            Self::OpenOcean => "Open Temperate Ocean",
            Self::AbyssalTrench => "Abyssal Ocean Trench",
            Self::RiversLakes => "Inland Rivers & Lakes",
            Self::MangroveWetland => "Mangrove Swamp & Wetland",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::Plains => "biome.temperate_plains",
            Self::Forest => "biome.dense_temperate_forest",
            Self::Desert => "biome.arid_desert_badlands",
            Self::Taiga => "biome.snowy_coniferous_taiga",
            Self::Highlands => "biome.rocky_highlands_mountain_tundra",
            Self::Savanna => "biome.tropical_savanna",
            Self::CoastalBeachReef => "biome.shallow_coastal_beach_reef",
            Self::OpenOcean => "biome.open_temperate_ocean",
            Self::AbyssalTrench => "biome.abyssal_ocean_trench",
            Self::RiversLakes => "biome.inland_rivers_lakes",
            Self::MangroveWetland => "biome.mangrove_swamp_wetland",
        }
    }

    pub fn config(self) -> BiomeConfig {
        match self {
            Self::Plains => BiomeConfig {
                biome_type: self,
                name: "Temperate Plains",
                surface_material: Voxel::LoamGrass,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 3,
                base_height_offset: 2.0,
                amplitude_multiplier: 0.9,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::Forest => BiomeConfig {
                biome_type: self,
                name: "Dense Temperate Forest",
                surface_material: Voxel::LoamMulch,
                subsoil_material: Voxel::LoamDirt,
                subsoil_depth: 4,
                base_height_offset: 4.0,
                amplitude_multiplier: 1.2,
                primary_stone: Voxel::Granite,
                cliff_material: Voxel::CobbledGranite,
            },
            Self::Desert => BiomeConfig {
                biome_type: self,
                name: "Arid Desert & Badlands",
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 4,
                base_height_offset: 3.0,
                amplitude_multiplier: 1.1,
                primary_stone: Voxel::SandstoneCommon,
                cliff_material: Voxel::SandstoneCommon,
            },
            Self::Taiga => BiomeConfig {
                biome_type: self,
                name: "Snowy Coniferous Taiga",
                surface_material: Voxel::SiltSnow,
                subsoil_material: Voxel::PermafrostDirt,
                subsoil_depth: 4,
                base_height_offset: 6.0,
                amplitude_multiplier: 1.3,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::FrostIcePacked,
            },
            Self::Highlands => BiomeConfig {
                biome_type: self,
                name: "Rocky Highlands / Mountain Tundra",
                surface_material: Voxel::GravelSandy,
                subsoil_material: Voxel::Andesite,
                subsoil_depth: 2,
                base_height_offset: 32.0,
                amplitude_multiplier: 3.2,
                primary_stone: Voxel::Andesite,
                cliff_material: Voxel::CobbledAndesite,
            },
            Self::Savanna => BiomeConfig {
                biome_type: self,
                name: "Tropical Savanna",
                surface_material: Voxel::LateriteGrass,
                subsoil_material: Voxel::LateriteDirt,
                subsoil_depth: 3,
                base_height_offset: 3.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::Basalt,
                cliff_material: Voxel::Basalt,
            },
            Self::CoastalBeachReef => BiomeConfig {
                biome_type: self,
                name: "Shallow Coastal Beach & Reef",
                surface_material: Voxel::SandWhite,
                subsoil_material: Voxel::SandCommon,
                subsoil_depth: 4,
                base_height_offset: -0.5,
                amplitude_multiplier: 0.3,
                primary_stone: Voxel::SandstoneWhite,
                cliff_material: Voxel::SandstoneWhite,
            },
            Self::OpenOcean => BiomeConfig {
                biome_type: self,
                name: "Open Temperate Ocean",
                surface_material: Voxel::SandCommon,
                subsoil_material: Voxel::GravelSandy,
                subsoil_depth: 4,
                base_height_offset: -18.0,
                amplitude_multiplier: 0.6,
                primary_stone: Voxel::Gabbro,
                cliff_material: Voxel::Gabbro,
            },
            Self::AbyssalTrench => BiomeConfig {
                biome_type: self,
                name: "Abyssal Ocean Trench",
                surface_material: Voxel::GravelAshy,
                subsoil_material: Voxel::Basalt,
                subsoil_depth: 3,
                base_height_offset: -38.0,
                amplitude_multiplier: 0.8,
                primary_stone: Voxel::Obsidian,
                cliff_material: Voxel::Obsidian,
            },
            Self::RiversLakes => BiomeConfig {
                biome_type: self,
                name: "Inland Rivers & Lakes",
                surface_material: Voxel::SiltMud,
                subsoil_material: Voxel::ClayBlue,
                subsoil_depth: 3,
                base_height_offset: -2.0,
                amplitude_multiplier: 0.4,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::Argillite,
            },
            Self::MangroveWetland => BiomeConfig {
                biome_type: self,
                name: "Mangrove Swamp & Wetland",
                surface_material: Voxel::PeatMud,
                subsoil_material: Voxel::PeatDirt,
                subsoil_depth: 4,
                base_height_offset: -0.2,
                amplitude_multiplier: 0.35,
                primary_stone: Voxel::Argillite,
                cliff_material: Voxel::LoamMud,
            },
        }
    }

    /// Calibrated grass tint color per biome.
    pub fn grass_color(self) -> [f32; 4] {
        match self {
            Self::Plains => [0.55, 0.94, 0.42, 1.0],
            Self::Forest => [0.45, 0.90, 0.35, 1.0],
            Self::Desert => [0.78, 0.72, 0.35, 1.0],
            Self::Taiga => [0.48, 0.80, 0.65, 1.0],
            Self::Highlands => [0.50, 0.78, 0.52, 1.0],
            Self::Savanna => [0.72, 0.85, 0.30, 1.0],
            Self::CoastalBeachReef => [0.60, 0.92, 0.44, 1.0],
            Self::OpenOcean => [0.35, 0.75, 0.65, 1.0],
            Self::AbyssalTrench => [0.20, 0.35, 0.50, 1.0],
            Self::RiversLakes => [0.52, 0.90, 0.40, 1.0],
            Self::MangroveWetland => [0.40, 0.82, 0.32, 1.0],
        }
    }

    /// Calibrated water tint color per biome.
    pub fn water_color(self) -> [f32; 4] {
        match self {
            Self::CoastalBeachReef => [0.20, 0.82, 0.94, 1.0],
            Self::OpenOcean => [0.14, 0.42, 0.78, 1.0],
            Self::AbyssalTrench => [0.06, 0.12, 0.32, 1.0],
            Self::RiversLakes => [0.25, 0.65, 0.75, 1.0],
            Self::MangroveWetland => [0.22, 0.50, 0.38, 1.0],
            Self::Taiga => [0.35, 0.72, 0.95, 1.0],
            Self::Highlands => [0.38, 0.75, 0.98, 1.0],
            Self::Desert => [0.22, 0.75, 0.95, 1.0],
            Self::Savanna => [0.28, 0.68, 0.85, 1.0],
            Self::Forest => [0.26, 0.62, 0.88, 1.0],
            Self::Plains => [0.24, 0.65, 0.92, 1.0],
        }
    }

    /// Calibrated foliage (leaves) tint color per biome.
    pub fn foliage_color(self, _voxel: Voxel) -> [f32; 4] {
        match self {
            Self::Plains => [0.55, 1.10, 0.35, 1.0],
            Self::Forest => [0.42, 1.00, 0.30, 1.0],
            Self::Desert => [0.72, 0.95, 0.30, 1.0],
            Self::Taiga => [0.38, 0.85, 0.55, 1.0],
            Self::Highlands => [0.45, 0.88, 0.50, 1.0],
            Self::Savanna => [0.70, 0.92, 0.28, 1.0],
            Self::CoastalBeachReef => [0.52, 1.05, 0.32, 1.0],
            Self::OpenOcean => [0.35, 0.80, 0.50, 1.0],
            Self::AbyssalTrench => [0.25, 0.60, 0.40, 1.0],
            Self::RiversLakes => [0.48, 1.02, 0.36, 1.0],
            Self::MangroveWetland => [0.40, 0.88, 0.30, 1.0],
        }
    }

    /// Primary tint color for a voxel in this biome.
    pub fn voxel_tint(self, voxel: Voxel) -> [f32; 4] {
        if !voxel.is_tinted() {
            [1.0, 1.0, 1.0, 1.0]
        } else if voxel.is_grass_soil() {
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

        let temperature = fbm_2d_x4(
            world_x,
            world_z,
            self.temperature_freq,
            2,
            0.5,
            2.0,
            seed.wrapping_add(30_011),
        );

        let humidity = fbm_2d_x4(
            world_x,
            world_z,
            self.humidity_freq,
            2,
            0.5,
            2.0,
            seed.wrapping_add(50_021),
        );

        std::array::from_fn(|i| {
            let c = continentalness.0[i];
            let t = temperature.0[i];
            let h = humidity.0[i];
            ClimateSample {
                continentalness: c,
                temperature: t,
                humidity: h,
                biome: Self::classify_biome(c, t, h),
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

    /// Organically distributes all 11 biomes across Continentalness, Temperature, and Humidity.
    pub fn classify_biome(continentalness: f32, temperature: f32, humidity: f32) -> BiomeType {
        // 1. Deep Aquatic (Continentalness < -0.15)
        if continentalness < -0.42 {
            return BiomeType::AbyssalTrench;
        } else if continentalness < -0.15 {
            return BiomeType::OpenOcean;
        }

        // 2. Coastal / Transitional (-0.15 <= Continentalness < 0.05)
        if continentalness < 0.05 {
            if humidity > 0.15 {
                return BiomeType::MangroveWetland;
            } else {
                return BiomeType::CoastalBeachReef;
            }
        }

        // 3. Mountain / Highlands (Continentalness >= 0.35)
        if continentalness >= 0.35 {
            return BiomeType::Highlands;
        }

        // 4. Inland Continental Biomes (0.05 <= Continentalness < 0.35)
        if humidity > 0.35 && continentalness < 0.15 {
            return BiomeType::RiversLakes;
        }

        if temperature < -0.15 {
            BiomeType::Taiga
        } else if temperature >= 0.22 {
            if humidity < -0.05 {
                BiomeType::Desert
            } else {
                BiomeType::Savanna
            }
        } else if humidity > 0.05 {
            BiomeType::Forest
        } else {
            BiomeType::Plains
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

    const R_INNER: f32 = 5.0;
    const R_DIAG: f32 = 3.5355; // 5.0 / sqrt(2)
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
