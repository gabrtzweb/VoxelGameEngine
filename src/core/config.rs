use bevy::prelude::Resource;
use bevy::render::settings::Backends;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const CONFIG_PATH: &str = "local_config.json";

/// Performance presets tailored for different hardware profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformancePreset {
    LowSpec,
    Medium,
    High,
    Ultra,
}

impl PerformancePreset {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            Self::LowSpec => "Low-Spec (Laptop)",
            Self::Medium => "Medium (Balanced)",
            Self::High => "High",
            Self::Ultra => "Ultra (Desktop)",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::LowSpec => "Low-Spec",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::LowSpec => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Ultra,
            Self::Ultra => Self::LowSpec,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::LowSpec => Self::Ultra,
            Self::Medium => Self::LowSpec,
            Self::High => Self::Medium,
            Self::Ultra => Self::High,
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// Preferred graphics backend: "auto" (default, uses PRIMARY/Vulkan), "dx12", "vulkan", "gl"
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Active performance preset (if any was selected).
    #[serde(default)]
    pub preset: Option<PerformancePreset>,

    /// Chunk render distance in full 16m chunks (e.g. 2..=16).
    #[serde(default = "default_render_distance")]
    pub render_distance: i32,

    /// Chunk LOD render distance in full 16m chunk columns (e.g. 0..=32).
    #[serde(default = "default_lod_render_distance")]
    pub lod_render_distance: i32,

    /// Chunk simulation distance in full 16m chunks (e.g. 2..=8).
    #[serde(default = "default_simulation_distance")]
    pub simulation_distance: i32,

    /// Vertical field of view in degrees (60.0 ..= 110.0).
    #[serde(default = "default_fov_degrees")]
    pub fov_degrees: f32,

    /// Whether distance horizon fog is enabled.
    #[serde(default = "default_fog_enabled")]
    pub fog_enabled: bool,

    /// Head bobbing during walking/sprinting.
    #[serde(default = "default_view_bobbing")]
    pub view_bobbing: bool,

    /// Render side texture on grass as full grass instead of dual-layer outcrop.
    #[serde(default = "default_full_grass")]
    pub full_grass: bool,

    /// VSync presentation mode.
    #[serde(default = "default_vsync_enabled")]
    pub vsync_enabled: bool,

    /// Dynamic frame throttling (15 FPS unfocused, 30 FPS idle, unconstrained active).
    #[serde(default = "default_dynamic_fps_enabled")]
    pub dynamic_fps_enabled: bool,

    /// Display screen mode: "windowed", "borderless", "fullscreen".
    #[serde(default = "default_screen_mode")]
    pub screen_mode: String,
}

fn default_backend() -> String {
    "auto".to_string()
}

fn default_render_distance() -> i32 {
    8
}

fn default_lod_render_distance() -> i32 {
    8
}

fn default_simulation_distance() -> i32 {
    8
}

fn default_fov_degrees() -> f32 {
    90.0
}

fn default_fog_enabled() -> bool {
    false
}

fn default_view_bobbing() -> bool {
    true
}

fn default_full_grass() -> bool {
    false
}

fn default_vsync_enabled() -> bool {
    false
}

fn default_dynamic_fps_enabled() -> bool {
    true
}

fn default_screen_mode() -> String {
    "windowed".to_string()
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            preset: Some(PerformancePreset::Medium),
            render_distance: default_render_distance(),
            lod_render_distance: default_lod_render_distance(),
            simulation_distance: default_simulation_distance(),
            fov_degrees: default_fov_degrees(),
            fog_enabled: default_fog_enabled(),
            view_bobbing: default_view_bobbing(),
            full_grass: default_full_grass(),
            vsync_enabled: default_vsync_enabled(),
            dynamic_fps_enabled: default_dynamic_fps_enabled(),
            screen_mode: default_screen_mode(),
        }
    }
}

impl LocalConfig {
    /// Loads `local_config.json` if it exists, otherwise creates and saves default settings.
    pub fn load_or_default() -> Self {
        if Path::new(CONFIG_PATH).exists()
            && let Ok(content) = fs::read_to_string(CONFIG_PATH)
            && let Ok(config) = serde_json::from_str::<LocalConfig>(&content)
        {
            return config;
        }
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    /// Saves the current configuration to `local_config.json`.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let serialized = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(CONFIG_PATH, serialized)
    }

    /// Applies a predefined performance preset to the configuration.
    pub fn apply_preset(&mut self, preset: PerformancePreset) {
        self.preset = Some(preset);
        match preset {
            PerformancePreset::LowSpec => {
                self.render_distance = 2;
                self.lod_render_distance = 6;
                self.simulation_distance = 4;
                self.fog_enabled = false;
                self.view_bobbing = true;
                self.full_grass = false;
                self.vsync_enabled = false;
                self.dynamic_fps_enabled = true;
            }
            PerformancePreset::Medium => {
                self.render_distance = 8;
                self.lod_render_distance = 8;
                self.simulation_distance = 8;
                self.fog_enabled = false;
                self.view_bobbing = true;
                self.full_grass = false;
                self.vsync_enabled = false;
                self.dynamic_fps_enabled = true;
            }
            PerformancePreset::High => {
                self.render_distance = 12;
                self.lod_render_distance = 16;
                self.simulation_distance = 8;
                self.fog_enabled = true;
                self.view_bobbing = true;
                self.full_grass = true;
                self.vsync_enabled = false;
                self.dynamic_fps_enabled = true;
            }
            PerformancePreset::Ultra => {
                self.render_distance = 16;
                self.lod_render_distance = 32;
                self.simulation_distance = 8;
                self.fog_enabled = true;
                self.view_bobbing = true;
                self.full_grass = true;
                self.vsync_enabled = true;
                self.dynamic_fps_enabled = false;
            }
        }
    }

    /// Resolves the WGPU graphics backends based on:
    /// 1. `backend` in `local_config.json` (if explicitly set to "dx12", "vulkan", etc.)
    /// 2. `WGPU_BACKEND` environment variable
    /// 3. Default fallback: `Backends::PRIMARY` (Vulkan on desktop/Linux)
    pub fn resolve_backends(&self) -> Backends {
        match self.backend.to_lowercase().trim() {
            "dx12" | "directx12" | "d3d12" => Backends::DX12,
            "vulkan" | "vk" => Backends::VULKAN,
            "gl" | "opengl" => Backends::GL,
            "metal" => Backends::METAL,
            _ => Backends::from_env().unwrap_or(Backends::PRIMARY),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LocalConfig::default();
        assert_eq!(config.backend, "auto");
        assert_eq!(config.render_distance, 8);
        assert_eq!(config.lod_render_distance, 8);
        assert_eq!(config.simulation_distance, 8);
    }

    #[test]
    fn test_presets() {
        let mut config = LocalConfig::default();
        config.apply_preset(PerformancePreset::LowSpec);
        assert_eq!(config.render_distance, 2);
        assert_eq!(config.lod_render_distance, 6);
        assert_eq!(config.simulation_distance, 4);
        assert!(!config.fog_enabled);

        config.apply_preset(PerformancePreset::Ultra);
        assert_eq!(config.render_distance, 16);
        assert_eq!(config.lod_render_distance, 32);
        assert!(config.fog_enabled);
        assert!(config.vsync_enabled);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut config = LocalConfig::default();
        config.apply_preset(PerformancePreset::LowSpec);
        let json = serde_json::to_string(&config).expect("Serialize failed");
        let parsed: LocalConfig = serde_json::from_str(&json).expect("Deserialize failed");
        assert_eq!(parsed.render_distance, 2);
        assert_eq!(parsed.preset, Some(PerformancePreset::LowSpec));
    }
}
