use bevy::prelude::Resource;
use bevy::render::settings::Backends;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const CONFIG_PATH: &str = "local_config.json";

/// Performance presets tailored for different hardware profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformancePreset {
    Potato,
    Low,
    Medium,
    High,
    Ultra,
}

impl PerformancePreset {
    pub fn name(self) -> &'static str {
        match self {
            Self::Potato => "Potato",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
        }
    }

    pub fn short_name(self) -> &'static str {
        self.name()
    }

    pub fn next(self) -> Self {
        match self {
            Self::Potato => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Ultra,
            Self::Ultra => Self::Potato,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Potato => Self::Ultra,
            Self::Low => Self::Potato,
            Self::Medium => Self::Low,
            Self::High => Self::Medium,
            Self::Ultra => Self::High,
        }
    }
}

/// Anti-Aliasing technique for edge smoothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AntiAliasingMode {
    Off,
    #[default]
    Fxaa,
    Msaa2x,
    Msaa4x,
}

impl AntiAliasingMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Fxaa => "FXAA",
            Self::Msaa2x => "MSAA 2x",
            Self::Msaa4x => "MSAA 4x",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Fxaa,
            Self::Fxaa => Self::Msaa2x,
            Self::Msaa2x => Self::Msaa4x,
            Self::Msaa4x => Self::Off,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Off => Self::Msaa4x,
            Self::Fxaa => Self::Off,
            Self::Msaa2x => Self::Fxaa,
            Self::Msaa4x => Self::Msaa2x,
        }
    }
}

/// Directional shadow quality and cascade fidelity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ShadowQuality {
    Off,
    Low,
    #[default]
    Medium,
    High,
    Ultra,
}

impl ShadowQuality {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Ultra => "Ultra",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Ultra,
            Self::Ultra => Self::Off,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Off => Self::Ultra,
            Self::Low => Self::Off,
            Self::Medium => Self::Low,
            Self::High => Self::Medium,
            Self::Ultra => Self::High,
        }
    }
}

/// Cloud rendering quality: Off, 2D flat planar, 3D volumetric blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CloudQuality {
    Off,
    Fast2D,
    #[default]
    Fancy3D,
}

impl CloudQuality {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Fast2D => "Fast (2D)",
            Self::Fancy3D => "Fancy (3D)",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Fast2D,
            Self::Fast2D => Self::Fancy3D,
            Self::Fancy3D => Self::Off,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Off => Self::Fancy3D,
            Self::Fast2D => Self::Off,
            Self::Fancy3D => Self::Fast2D,
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

    /// Dynamic FOV kick on sprint (can be disabled for motion-sensitive players).
    #[serde(default = "default_dynamic_fov_enabled")]
    pub dynamic_fov_enabled: bool,

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

    /// Whether screen-space transmission is enabled (disabled on Low-Spec / Medium to prevent costly framebuffer blits).
    #[serde(default = "default_transmission_enabled")]
    pub transmission_enabled: bool,

    /// Directional shadow quality (Off, Low, Medium, High, Ultra).
    #[serde(default = "default_shadow_quality")]
    pub shadow_quality: ShadowQuality,

    /// Whether directional shadows are enabled (kept for compatibility with shadow_quality != Off).
    #[serde(default = "default_shadows_enabled")]
    pub shadows_enabled: bool,

    /// Cloud rendering fidelity (Off, Fast2D, Fancy3D).
    #[serde(default = "default_cloud_quality")]
    pub cloud_quality: CloudQuality,

    /// Anti-aliasing mode (Off, FXAA, MSAA 2x, MSAA 4x).
    #[serde(default = "default_anti_aliasing")]
    pub anti_aliasing: AntiAliasingMode,

    /// Whether volumetric god rays / crepuscular rays post-processing is enabled.
    #[serde(default = "default_god_rays_enabled")]
    pub god_rays_enabled: bool,

    /// Render scale factor for 3D world rendering (0.50 ..= 1.00).
    /// 1.00 = Native resolution (100%)
    /// 0.85 = Ultra Quality (85%)
    /// 0.75 = Quality (75%)
    /// 0.66 = Balanced (66%)
    /// 0.50 = Performance (50%)
    #[serde(default = "default_render_scale")]
    pub render_scale: f32,
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

fn default_dynamic_fov_enabled() -> bool {
    true
}

fn default_fog_enabled() -> bool {
    true
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

fn default_transmission_enabled() -> bool {
    false
}

fn default_shadow_quality() -> ShadowQuality {
    ShadowQuality::Medium
}

fn default_shadows_enabled() -> bool {
    true
}

fn default_cloud_quality() -> CloudQuality {
    CloudQuality::Fancy3D
}

fn default_anti_aliasing() -> AntiAliasingMode {
    AntiAliasingMode::Fxaa
}

fn default_god_rays_enabled() -> bool {
    true
}

fn default_render_scale() -> f32 {
    1.0
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
            dynamic_fov_enabled: default_dynamic_fov_enabled(),
            fog_enabled: default_fog_enabled(),
            view_bobbing: default_view_bobbing(),
            full_grass: default_full_grass(),
            vsync_enabled: default_vsync_enabled(),
            dynamic_fps_enabled: default_dynamic_fps_enabled(),
            screen_mode: default_screen_mode(),
            transmission_enabled: default_transmission_enabled(),
            shadow_quality: default_shadow_quality(),
            shadows_enabled: default_shadows_enabled(),
            cloud_quality: default_cloud_quality(),
            anti_aliasing: default_anti_aliasing(),
            god_rays_enabled: default_god_rays_enabled(),
            render_scale: default_render_scale(),
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
            PerformancePreset::Potato => {
                self.render_distance = 2;
                self.lod_render_distance = 2;
                self.simulation_distance = 2;
                self.fog_enabled = false;
                self.view_bobbing = true;
                self.full_grass = true;
                self.vsync_enabled = false;
                self.dynamic_fps_enabled = true;
                self.transmission_enabled = false;
                self.shadow_quality = ShadowQuality::Off;
                self.shadows_enabled = false;
                self.cloud_quality = CloudQuality::Off;
                self.anti_aliasing = AntiAliasingMode::Off;
                self.god_rays_enabled = false;
                self.render_scale = 0.50;
            }
            PerformancePreset::Low => {
                self.render_distance = 4;
                self.lod_render_distance = 4;
                self.simulation_distance = 4;
                self.fog_enabled = false;
                self.view_bobbing = true;
                self.full_grass = true;
                self.vsync_enabled = false;
                self.dynamic_fps_enabled = true;
                self.transmission_enabled = false;
                self.shadow_quality = ShadowQuality::Off;
                self.shadows_enabled = false;
                self.cloud_quality = CloudQuality::Fast2D;
                self.anti_aliasing = AntiAliasingMode::Off;
                self.god_rays_enabled = false;
                self.render_scale = 0.75;
            }
            PerformancePreset::Medium => {
                self.render_distance = 8;
                self.lod_render_distance = 8;
                self.simulation_distance = 8;
                self.fog_enabled = true;
                self.view_bobbing = true;
                self.full_grass = false;
                self.vsync_enabled = false;
                self.dynamic_fps_enabled = true;
                self.transmission_enabled = false;
                self.shadow_quality = ShadowQuality::Medium;
                self.shadows_enabled = true;
                self.cloud_quality = CloudQuality::Fancy3D;
                self.anti_aliasing = AntiAliasingMode::Fxaa;
                self.god_rays_enabled = true;
                self.render_scale = 1.00;
            }
            PerformancePreset::High => {
                self.render_distance = 8;
                self.lod_render_distance = 16;
                self.simulation_distance = 8;
                self.fog_enabled = true;
                self.view_bobbing = true;
                self.full_grass = false;
                self.vsync_enabled = false;
                self.dynamic_fps_enabled = true;
                self.transmission_enabled = true;
                self.shadow_quality = ShadowQuality::High;
                self.shadows_enabled = true;
                self.cloud_quality = CloudQuality::Fancy3D;
                self.anti_aliasing = AntiAliasingMode::Msaa2x;
                self.god_rays_enabled = true;
                self.render_scale = 1.00;
            }
            PerformancePreset::Ultra => {
                self.render_distance = 16;
                self.lod_render_distance = 32;
                self.simulation_distance = 16;
                self.fog_enabled = true;
                self.view_bobbing = true;
                self.full_grass = false;
                self.vsync_enabled = false;
                self.dynamic_fps_enabled = true;
                self.transmission_enabled = true;
                self.shadow_quality = ShadowQuality::Ultra;
                self.shadows_enabled = true;
                self.cloud_quality = CloudQuality::Fancy3D;
                self.anti_aliasing = AntiAliasingMode::Msaa4x;
                self.god_rays_enabled = true;
                self.render_scale = 1.00;
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

pub const RENDER_SCALE_PRESETS: [f32; 5] = [0.50, 0.66, 0.75, 0.85, 1.00];

pub fn next_render_scale(current: f32) -> f32 {
    for &step in &RENDER_SCALE_PRESETS {
        if step > current + 0.02 {
            return step;
        }
    }
    1.00
}

pub fn prev_render_scale(current: f32) -> f32 {
    for &step in RENDER_SCALE_PRESETS.iter().rev() {
        if step < current - 0.02 {
            return step;
        }
    }
    0.50
}

pub fn format_render_scale(scale: f32) -> String {
    if (scale - 1.00).abs() < 0.02 {
        "100% (Native)".to_string()
    } else if (scale - 0.85).abs() < 0.02 {
        "85% (Ultra Quality)".to_string()
    } else if (scale - 0.75).abs() < 0.02 {
        "75% (Quality)".to_string()
    } else if (scale - 0.66).abs() < 0.02 {
        "66% (Balanced)".to_string()
    } else if (scale - 0.50).abs() < 0.02 {
        "50% (Performance)".to_string()
    } else {
        format!("{:.0}%", scale * 100.0)
    }
}
