pub mod config;
pub mod dev_stats;
pub mod dynamic_fps;
pub mod font;
pub mod localization;
pub mod math;
pub mod noise;
pub mod render_scale;
pub mod ui_scale;
pub mod window_aspect;

pub use config::{
    AntiAliasingMode, CloudQuality, LocalConfig, PerformancePreset, ShadowQuality, TargetFpsLimit,
    UpscalingFilter,
};
pub use dev_stats::DevStatsPlugin;
pub use dynamic_fps::{DynamicFpsPlugin, DynamicFpsSettings, DynamicFpsState};
pub use font::{AppFont, FontPlugin, FontSource, text_shadow_default};
pub use localization::{Locale, Localization, LocalizationPlugin};
pub use render_scale::{RenderScalePlugin, RenderScaleState, RenderScaleUiCamera};
pub use ui_scale::AdaptiveUiPlugin;
pub use window_aspect::{WindowAspectPlugin, WindowAspectState};
