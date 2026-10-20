//! Pure domain models and simulation logic for agricultural crops and plant life cycles.
//! Decoupled from rendering and GPU systems for deterministic execution and testing.

/// Supported agricultural crop types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CropType {
    Wheat,
    Barley,
    Carrot,
    Potato,
    Beetroot,
    Herb,
}

impl CropType {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Wheat => "wheat",
            Self::Barley => "barley",
            Self::Carrot => "carrot",
            Self::Potato => "potato",
            Self::Beetroot => "beetroot",
            Self::Herb => "herb",
        }
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Wheat => "Wheat",
            Self::Barley => "Barley",
            Self::Carrot => "Carrot",
            Self::Potato => "Potato",
            Self::Beetroot => "Beetroot",
            Self::Herb => "Medicinal Herb",
        }
    }

    /// Total distinct growth stages from Seed (0) to Harvestable/Ripe.
    pub const fn total_stages(&self) -> u8 {
        match self {
            Self::Wheat | Self::Barley => 8,  // 0..=7 (classic 8 stages)
            Self::Carrot | Self::Potato => 4, // 0..=3 (4 stages)
            Self::Beetroot => 4,
            Self::Herb => 3,
        }
    }

    /// Maximum stage index (inclusive).
    #[inline(always)]
    pub const fn max_stage(&self) -> u8 {
        self.total_stages().saturating_sub(1)
    }

    /// Returns standard growth requirements for this crop.
    pub const fn growth_config(&self) -> CropGrowthConfig {
        match self {
            Self::Wheat | Self::Barley => CropGrowthConfig {
                min_light_level: 9,
                min_moisture: SoilMoisture::Moist,
                growth_chance_hydrated: 0.75,
                growth_chance_moist: 0.35,
            },
            Self::Carrot | Self::Potato => CropGrowthConfig {
                min_light_level: 8,
                min_moisture: SoilMoisture::Moist,
                growth_chance_hydrated: 0.80,
                growth_chance_moist: 0.40,
            },
            Self::Beetroot => CropGrowthConfig {
                min_light_level: 9,
                min_moisture: SoilMoisture::Moist,
                growth_chance_hydrated: 0.70,
                growth_chance_moist: 0.30,
            },
            Self::Herb => CropGrowthConfig {
                min_light_level: 7,
                min_moisture: SoilMoisture::Dry, // Herbs are resilient and can grow in dry soil
                growth_chance_hydrated: 0.85,
                growth_chance_moist: 0.60,
            },
        }
    }
}

/// Represents the current growth stage of a crop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CropStage {
    pub stage: u8,
    pub max_stage: u8,
}

impl CropStage {
    #[inline(always)]
    pub const fn new(stage: u8, max_stage: u8) -> Self {
        let clamped = if stage > max_stage { max_stage } else { stage };
        Self {
            stage: clamped,
            max_stage,
        }
    }

    #[inline(always)]
    pub const fn seed(crop: CropType) -> Self {
        Self {
            stage: 0,
            max_stage: crop.max_stage(),
        }
    }

    #[inline(always)]
    pub const fn is_seed(&self) -> bool {
        self.stage == 0
    }

    #[inline(always)]
    pub const fn is_sprout(&self) -> bool {
        self.stage == 1
    }

    #[inline(always)]
    pub const fn is_ripe(&self) -> bool {
        self.stage >= self.max_stage
    }

    /// Progresses crop to the next stage, capping at `max_stage`.
    #[inline(always)]
    pub const fn advance(self) -> Self {
        if self.stage < self.max_stage {
            Self {
                stage: self.stage + 1,
                max_stage: self.max_stage,
            }
        } else {
            self
        }
    }

    /// Returns the progress ratio from 0.0 (just planted) to 1.0 (fully ripe).
    #[inline(always)]
    pub fn progress_ratio(&self) -> f32 {
        if self.max_stage == 0 {
            1.0
        } else {
            self.stage as f32 / self.max_stage as f32
        }
    }
}

/// Soil hydration state affecting plant growth rates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SoilMoisture {
    #[default]
    Dry = 0,
    Moist = 1,
    Hydrated = 2,
}

impl SoilMoisture {
    /// Pure function computing soil moisture based on the horizontal Chebyshev distance
    /// to the nearest open water source (standard 4-block radius) and weather state.
    pub fn calculate_hydration(distance_to_water: u32, is_raining: bool) -> Self {
        if distance_to_water <= 4 {
            Self::Hydrated
        } else if is_raining || distance_to_water <= 6 {
            Self::Moist
        } else {
            Self::Dry
        }
    }
}

/// Parameters governing growth evaluation for a crop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CropGrowthConfig {
    pub min_light_level: u8,
    pub min_moisture: SoilMoisture,
    pub growth_chance_hydrated: f32,
    pub growth_chance_moist: f32,
}

/// Resulting items dropped upon harvesting a crop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CropHarvestResult {
    pub produce_count: u32,
    pub seed_count: u32,
}

/// Pure simulator evaluating crop state progression.
pub struct CropSimulator;

impl CropSimulator {
    /// Determines whether a crop can attempt to advance under current environmental conditions.
    pub fn can_grow(
        crop: CropType,
        stage: CropStage,
        moisture: SoilMoisture,
        light_level: u8,
    ) -> bool {
        if stage.is_ripe() {
            return false;
        }

        let config = crop.growth_config();

        if light_level < config.min_light_level {
            return false;
        }

        moisture >= config.min_moisture
    }

    /// Evaluates a growth tick. If conditions are met and the random roll (0.0..1.0)
    /// satisfies the probability threshold, advances the crop stage.
    pub fn evaluate_tick(
        crop: CropType,
        current_stage: CropStage,
        moisture: SoilMoisture,
        light_level: u8,
        roll: f32,
    ) -> CropStage {
        if !Self::can_grow(crop, current_stage, moisture, light_level) {
            return current_stage;
        }

        let config = crop.growth_config();
        let chance = match moisture {
            SoilMoisture::Hydrated => config.growth_chance_hydrated,
            SoilMoisture::Moist => config.growth_chance_moist,
            SoilMoisture::Dry => 0.10,
        };

        if roll <= chance {
            current_stage.advance()
        } else {
            current_stage
        }
    }

    /// Determines the harvest yield when breaking or interacting with a crop.
    pub fn harvest(crop: CropType, stage: CropStage) -> CropHarvestResult {
        if !stage.is_ripe() {
            // Immature crops drop only the 1 original planted seed, no produce
            return CropHarvestResult {
                produce_count: 0,
                seed_count: 1,
            };
        }

        match crop {
            CropType::Wheat | CropType::Barley => CropHarvestResult {
                produce_count: 1,
                seed_count: 2, // Replant 1 + 1 extra seed
            },
            CropType::Carrot | CropType::Potato => CropHarvestResult {
                produce_count: 3, // Multi-crop yield
                seed_count: 1,
            },
            CropType::Beetroot => CropHarvestResult {
                produce_count: 1,
                seed_count: 2,
            },
            CropType::Herb => CropHarvestResult {
                produce_count: 2,
                seed_count: 1,
            },
        }
    }
}
