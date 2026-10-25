use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default, Reflect)]
pub enum GrassStyle {
    #[default]
    Tapered,
    Segmented,
}

impl GrassStyle {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Tapered => "Tapered",
            Self::Segmented => "Segmented",
        }
    }
}

/// Runtime configuration knobs for procedural voxel foliage (Grassier Grass).
#[derive(Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct GrassConfig {
    pub enabled: bool,
    pub blades_per_block: u32,
    pub grass_style: GrassStyle,
    pub blade_height: f32,
    pub height_variation: f32,
    pub blade_width: f32,
    pub grass_sparsity: f32,
    pub render_radius: u32,
    pub render_lod: bool,
    pub dense_flowers: bool,
    pub blade_particles: bool,
    pub grass_through_snow: bool,
    pub grass_brightness: f32,
    pub blade_hue_jitter: f32,
    pub blade_gradient_bottom: f32,
    pub blade_gradient_top: f32,
    pub blade_gradient_curve: f32,
    pub dynamic_wind: bool,
    pub dynamic_wind_speed_limit: f32,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            blades_per_block: 64,
            grass_style: GrassStyle::Tapered,
            blade_height: 0.30,
            height_variation: 1.35,
            blade_width: 0.95,
            grass_sparsity: 0.10,
            render_radius: 128,
            render_lod: true,
            dense_flowers: true,
            blade_particles: true,
            grass_through_snow: false,
            grass_brightness: 1.0,
            blade_hue_jitter: 8.0,
            blade_gradient_bottom: 0.8,
            blade_gradient_top: 1.0,
            blade_gradient_curve: 1.9,
            dynamic_wind: true,
            dynamic_wind_speed_limit: 0.8,
        }
    }
}
