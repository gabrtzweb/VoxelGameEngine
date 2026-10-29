use bevy::prelude::Resource;
use bevy::render::settings::Backends;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const CONFIG_PATH: &str = "local_config.json";

/// Performance presets tailored for different hardware profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformancePreset {
    Custom,
    Potato,
    Low,
    Medium,
    High,
    Ultra,
}

impl PerformancePreset {
    pub fn name(self) -> &'static str {
        match self {
            Self::Custom => "Custom",
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
            Self::Custom => Self::Potato,
            Self::Potato => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Ultra,
            Self::Ultra => Self::Potato,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Custom => Self::Ultra,
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

/// Target frame rate limit for active gameplay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TargetFpsLimit {
    Fps30,
    #[default]
    Fps60,
    Fps120,
    Fps144,
    Unlocked,
}

impl TargetFpsLimit {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fps30 => "30 FPS",
            Self::Fps60 => "60 FPS",
            Self::Fps120 => "120 FPS",
            Self::Fps144 => "144 FPS",
            Self::Unlocked => "Unlocked",
        }
    }

    pub fn to_fps(self) -> Option<u32> {
        match self {
            Self::Fps30 => Some(30),
            Self::Fps60 => Some(60),
            Self::Fps120 => Some(120),
            Self::Fps144 => Some(144),
            Self::Unlocked => None,
        }
    }

    pub fn from_fps(fps: u32) -> Self {
        match fps {
            30 => Self::Fps30,
            60 => Self::Fps60,
            120 => Self::Fps120,
            144 => Self::Fps144,
            _ => Self::Unlocked,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Fps30 => Self::Fps60,
            Self::Fps60 => Self::Fps120,
            Self::Fps120 => Self::Fps144,
            Self::Fps144 => Self::Unlocked,
            Self::Unlocked => Self::Fps30,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Fps30 => Self::Unlocked,
            Self::Fps60 => Self::Fps30,
            Self::Fps120 => Self::Fps60,
            Self::Fps144 => Self::Fps120,
            Self::Unlocked => Self::Fps144,
        }
    }
}

/// 3D Upscaling reconstruction filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UpscalingFilter {
    #[default]
    Bilinear,
    Fsr1,
}

impl UpscalingFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bilinear => "Bilinear",
            Self::Fsr1 => "FSR 1.0",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Bilinear => Self::Fsr1,
            Self::Fsr1 => Self::Bilinear,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Bilinear => Self::Fsr1,
            Self::Fsr1 => Self::Bilinear,
        }
    }
}

/// Coated texture tactile micro-variation intensity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CoatedTextureMode {
    Off,
    Low,
    #[default]
    Medium,
    High,
    Ultra,
}

impl CoatedTextureMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Low => "Low (15%)",
            Self::Medium => "Medium (28%)",
            Self::High => "High (42%)",
            Self::Ultra => "Ultra (58%)",
        }
    }

    pub fn strength(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Low => 0.15,
            Self::Medium => 0.28,
            Self::High => 0.42,
            Self::Ultra => 0.58,
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

/// Menu background blur depth of field intensity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MenuBlurIntensity {
    Off,
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

impl MenuBlurIntensity {
    pub const ALL: [Self; 5] = [
        Self::Off,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::Ultra,
    ];

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

    pub fn params(self) -> (f32, f32) {
        match self {
            Self::Off => (0.0, 32.0),
            Self::Low => (16.0, 1.4),
            Self::Medium => (32.0, 1.0),
            Self::High => (48.0, 0.7),
            Self::Ultra => (64.0, 0.5),
        }
    }
}

/// Menu background dimming (darkening opacity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BackgroundDimming {
    Off,
    Light,
    Medium,
    #[default]
    Dark,
    Heavy,
}

impl BackgroundDimming {
    pub const ALL: [Self; 5] = [
        Self::Off,
        Self::Light,
        Self::Medium,
        Self::Dark,
        Self::Heavy,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Light,
            Self::Light => Self::Medium,
            Self::Medium => Self::Dark,
            Self::Dark => Self::Heavy,
            Self::Heavy => Self::Off,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Off => Self::Heavy,
            Self::Light => Self::Off,
            Self::Medium => Self::Light,
            Self::Dark => Self::Medium,
            Self::Heavy => Self::Dark,
        }
    }

    pub fn alpha(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Light => 0.35,
            Self::Medium => 0.55,
            Self::Dark => 0.75,
            Self::Heavy => 0.90,
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

    /// Target frame rate limiter during active gameplay (Fps30, Fps60, Fps120, Fps144, Unlocked).
    #[serde(default = "default_target_fps")]
    pub target_fps: TargetFpsLimit,

    /// Per-frame limit on chunk mesh buffer uploads to the GPU.
    #[serde(default = "default_mesh_upload_budget")]
    pub mesh_upload_budget: usize,

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

    /// 3D Upscaling reconstruction filter (Bilinear or FSR 1.0).
    #[serde(default = "default_upscaling_filter")]
    pub upscaling_filter: UpscalingFilter,

    /// Whether natural in-game time flow is paused.
    #[serde(default = "default_is_time_paused", alias = "time_paused")]
    pub is_time_paused: bool,

    /// Master volume level (0.0 ..= 1.0).
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,

    /// Weather audio volume level (0.0 ..= 1.0).
    #[serde(default = "default_weather_volume")]
    pub weather_volume: f32,

    /// SFX audio volume level (0.0 ..= 1.0).
    #[serde(default = "default_sfx_volume")]
    pub sfx_volume: f32,

    /// Gameplay and soundtrack music volume level (0.0 ..= 1.0).
    #[serde(default = "default_music_volume")]
    pub music_volume: f32,

    /// Ambient nature murmurs volume level (0.0 ..= 1.0).
    #[serde(default = "default_ambient_volume")]
    pub ambient_volume: f32,

    /// Selected UI and gameplay language locale (e.g. "en_us", "pt_br").
    #[serde(default = "default_language")]
    pub language: String,

    /// GUI Scale factor for game interfaces (2x, 3x, 4x).
    #[serde(default = "default_gui_scale")]
    pub gui_scale: u32,

    /// Background blur intensity applied while in menus.
    #[serde(default = "default_menu_blur_intensity")]
    pub menu_blur_intensity: MenuBlurIntensity,

    /// Background darkness/dimming opacity applied while in menus.
    #[serde(default = "default_background_dimming")]
    pub background_dimming: BackgroundDimming,

    /// Mouse sensitivity multiplier (0.2 ..= 3.0, default: 1.0).
    #[serde(default = "default_mouse_sensitivity")]
    pub mouse_sensitivity: f32,

    /// Invert mouse look Y-axis (default: false).
    #[serde(default = "default_invert_mouse_y")]
    pub invert_mouse_y: bool,

    /// Custom keybindings overrides map: action_name -> "Key:KeyCode" / "Mouse:MouseButton".
    #[serde(default)]
    pub custom_keybindings: std::collections::BTreeMap<String, String>,

    /// Whether procedural foliage / grass blades are enabled.
    #[serde(default = "default_grass_enabled")]
    pub grass_enabled: bool,

    /// Procedural grass visual style: "tapered" or "segmented".
    #[serde(default = "default_grass_style")]
    pub grass_style: String,

    /// Procedural grass density (blades per block: 16, 32, 48, 64).
    #[serde(default = "default_grass_density")]
    pub grass_density: u32,

    /// Dynamic wind-blown loose grass blade particles.
    #[serde(default = "default_grass_particles")]
    pub grass_particles: bool,

    /// Coated texture strength (Off, Low, Medium, High, Ultra).
    #[serde(default = "default_coated_textures")]
    pub coated_textures: CoatedTextureMode,

    /// Procedural wind waving on flora, crops, and tree leaves.
    #[serde(default = "default_waving_foliage")]
    pub waving_foliage: bool,

    /// Pixelated shadows snapped to block texel grid.
    #[serde(default = "default_pixelated_shadows")]
    pub pixelated_shadows: bool,

    /// HDR bloom optical glow for torches, lava, and glowing blocks.
    #[serde(default = "default_hdr_bloom")]
    pub hdr_bloom: bool,
}

fn default_coated_textures() -> CoatedTextureMode {
    CoatedTextureMode::Medium
}

fn default_waving_foliage() -> bool {
    true
}

fn default_pixelated_shadows() -> bool {
    true
}

fn default_hdr_bloom() -> bool {
    true
}

fn default_grass_enabled() -> bool {
    true
}

fn default_grass_style() -> String {
    "tapered".to_string()
}

fn default_grass_density() -> u32 {
    32
}

fn default_grass_particles() -> bool {
    true
}

pub fn next_grass_density(density: u32) -> u32 {
    match density {
        0..=16 => 32,
        17..=32 => 48,
        33..=48 => 64,
        _ => 16,
    }
}

pub fn prev_grass_density(density: u32) -> u32 {
    match density {
        0..=16 => 64,
        17..=32 => 16,
        33..=48 => 32,
        _ => 48,
    }
}

pub fn cycle_grass_style(style: &str) -> String {
    match style {
        "segmented" | "Segmented" => "tapered".to_string(),
        _ => "segmented".to_string(),
    }
}

fn default_mouse_sensitivity() -> f32 {
    1.0
}

fn default_invert_mouse_y() -> bool {
    false
}

fn default_menu_blur_intensity() -> MenuBlurIntensity {
    MenuBlurIntensity::High
}

fn default_background_dimming() -> BackgroundDimming {
    BackgroundDimming::Dark
}

fn default_gui_scale() -> u32 {
    4
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
    "borderless".to_string()
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

fn default_target_fps() -> TargetFpsLimit {
    TargetFpsLimit::Fps60
}

fn default_mesh_upload_budget() -> usize {
    12
}

fn default_upscaling_filter() -> UpscalingFilter {
    UpscalingFilter::Bilinear
}

fn default_is_time_paused() -> bool {
    false
}

fn default_master_volume() -> f32 {
    0.85
}

fn default_weather_volume() -> f32 {
    0.75
}

fn default_sfx_volume() -> f32 {
    0.85
}

fn default_music_volume() -> f32 {
    0.50
}

fn default_ambient_volume() -> f32 {
    0.65
}

fn default_language() -> String {
    "en_us".to_string()
}

pub fn format_volume(volume: f32) -> String {
    format!("{:.0}%", (volume.clamp(0.0, 1.0) * 100.0).round())
}

pub fn step_volume_up(volume: f32) -> f32 {
    let steps = (volume * 20.0).round() + 1.0;
    (steps / 20.0).clamp(0.0, 1.0)
}

pub fn step_volume_down(volume: f32) -> f32 {
    let steps = (volume * 20.0).round() - 1.0;
    (steps / 20.0).clamp(0.0, 1.0)
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
            target_fps: default_target_fps(),
            mesh_upload_budget: default_mesh_upload_budget(),
            screen_mode: default_screen_mode(),
            transmission_enabled: default_transmission_enabled(),
            shadow_quality: default_shadow_quality(),
            shadows_enabled: default_shadows_enabled(),
            cloud_quality: default_cloud_quality(),
            anti_aliasing: default_anti_aliasing(),
            god_rays_enabled: default_god_rays_enabled(),
            render_scale: default_render_scale(),
            upscaling_filter: default_upscaling_filter(),
            is_time_paused: default_is_time_paused(),
            master_volume: default_master_volume(),
            weather_volume: default_weather_volume(),
            sfx_volume: default_sfx_volume(),
            music_volume: default_music_volume(),
            ambient_volume: default_ambient_volume(),
            language: default_language(),
            gui_scale: default_gui_scale(),
            menu_blur_intensity: default_menu_blur_intensity(),
            background_dimming: default_background_dimming(),
            mouse_sensitivity: default_mouse_sensitivity(),
            invert_mouse_y: default_invert_mouse_y(),
            custom_keybindings: std::collections::BTreeMap::new(),
            grass_enabled: default_grass_enabled(),
            grass_style: default_grass_style(),
            grass_density: default_grass_density(),
            grass_particles: default_grass_particles(),
            coated_textures: default_coated_textures(),
            waving_foliage: default_waving_foliage(),
            pixelated_shadows: default_pixelated_shadows(),
            hdr_bloom: default_hdr_bloom(),
        }
    }
}

impl LocalConfig {
    /// Constructs a `ControlSettings` resource reflecting persisted bindings and mouse options.
    pub fn to_control_settings(&self) -> crate::core::ControlSettings {
        let mut controls = crate::core::ControlSettings::default();
        controls.mouse_sensitivity = self.mouse_sensitivity;
        controls.invert_mouse_y = self.invert_mouse_y;

        for (action_key, binding_str) in &self.custom_keybindings {
            if let Some(action) = crate::core::InputAction::from_config_key(action_key) {
                if let Some(binding) = crate::core::InputBinding::from_config_str(binding_str) {
                    controls.set_binding(action, binding);
                }
            }
        }
        controls
    }

    /// Updates internal config properties from active `ControlSettings`.
    pub fn update_from_control_settings(&mut self, controls: &crate::core::ControlSettings) {
        self.mouse_sensitivity = controls.mouse_sensitivity;
        self.invert_mouse_y = controls.invert_mouse_y;

        let defaults = crate::core::ControlSettings::default();
        self.custom_keybindings.clear();

        for &action in crate::core::InputAction::ALL {
            let current_binding = controls.get_binding(action);
            let default_binding = defaults.get_binding(action);
            if current_binding != default_binding {
                self.custom_keybindings.insert(
                    action.config_key().to_string(),
                    current_binding.to_config_str(),
                );
            }
        }
    }

    /// Loads `local_config.json` if it exists, otherwise creates and saves default settings.
    pub fn load_or_default() -> Self {
        if Path::new(CONFIG_PATH).exists()
            && let Ok(content) = fs::read_to_string(CONFIG_PATH)
            && let Ok(mut config) = serde_json::from_str::<LocalConfig>(&content)
        {
            config.gui_scale = config.gui_scale.clamp(2, 4);
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
            PerformancePreset::Custom => {}
            PerformancePreset::Potato => {
                self.render_distance = 2;
                self.lod_render_distance = 2;
                self.simulation_distance = 2;
                self.fog_enabled = false;
                self.view_bobbing = true;
                self.full_grass = true;
                self.vsync_enabled = false;
                self.dynamic_fps_enabled = true;
                self.target_fps = TargetFpsLimit::Fps30;
                self.mesh_upload_budget = 3;
                self.transmission_enabled = false;
                self.shadow_quality = ShadowQuality::Off;
                self.shadows_enabled = false;
                self.cloud_quality = CloudQuality::Off;
                self.anti_aliasing = AntiAliasingMode::Off;
                self.god_rays_enabled = false;
                self.render_scale = 0.50;
                self.upscaling_filter = UpscalingFilter::Bilinear;
                self.grass_enabled = false;
                self.grass_density = 16;
                self.grass_particles = false;
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
                self.target_fps = TargetFpsLimit::Fps60;
                self.mesh_upload_budget = 6;
                self.transmission_enabled = false;
                self.shadow_quality = ShadowQuality::Off;
                self.shadows_enabled = false;
                self.cloud_quality = CloudQuality::Fast2D;
                self.anti_aliasing = AntiAliasingMode::Off;
                self.god_rays_enabled = false;
                self.render_scale = 0.75;
                self.upscaling_filter = UpscalingFilter::Bilinear;
                self.grass_enabled = false;
                self.grass_density = 16;
                self.grass_particles = false;
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
                self.target_fps = TargetFpsLimit::Fps60;
                self.mesh_upload_budget = 12;
                self.transmission_enabled = false;
                self.shadow_quality = ShadowQuality::Medium;
                self.shadows_enabled = true;
                self.cloud_quality = CloudQuality::Fancy3D;
                self.anti_aliasing = AntiAliasingMode::Fxaa;
                self.god_rays_enabled = true;
                self.render_scale = 1.00;
                self.upscaling_filter = UpscalingFilter::Fsr1;
                self.grass_enabled = true;
                self.grass_density = 32;
                self.grass_particles = true;
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
                self.target_fps = TargetFpsLimit::Fps144;
                self.mesh_upload_budget = 24;
                self.transmission_enabled = true;
                self.shadow_quality = ShadowQuality::High;
                self.shadows_enabled = true;
                self.cloud_quality = CloudQuality::Fancy3D;
                self.anti_aliasing = AntiAliasingMode::Msaa2x;
                self.god_rays_enabled = true;
                self.render_scale = 1.00;
                self.upscaling_filter = UpscalingFilter::Fsr1;
                self.grass_enabled = true;
                self.grass_density = 48;
                self.grass_particles = true;
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
                self.target_fps = TargetFpsLimit::Unlocked;
                self.mesh_upload_budget = 64;
                self.transmission_enabled = true;
                self.shadow_quality = ShadowQuality::Ultra;
                self.shadows_enabled = true;
                self.cloud_quality = CloudQuality::Fancy3D;
                self.anti_aliasing = AntiAliasingMode::Msaa4x;
                self.god_rays_enabled = true;
                self.render_scale = 1.00;
                self.upscaling_filter = UpscalingFilter::Fsr1;
                self.grass_enabled = true;
                self.grass_density = 64;
                self.grass_particles = true;
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

pub const MESH_UPLOAD_BUDGET_STEPS: [usize; 5] = [3, 6, 12, 24, 64];

pub fn next_mesh_upload_budget(current: usize) -> usize {
    for &step in &MESH_UPLOAD_BUDGET_STEPS {
        if step > current {
            return step;
        }
    }
    64
}

pub fn prev_mesh_upload_budget(current: usize) -> usize {
    for &step in MESH_UPLOAD_BUDGET_STEPS.iter().rev() {
        if step < current {
            return step;
        }
    }
    3
}
