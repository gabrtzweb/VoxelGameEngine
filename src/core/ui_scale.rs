use bevy::{prelude::*, ui::UiScale, window::PrimaryWindow};

pub struct AdaptiveUiPlugin;

impl Plugin for AdaptiveUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiScale>()
            .add_systems(PreUpdate, update_adaptive_ui_scale);
    }
}

/// Baseline reference resolution for the HUD layout (1080p).
/// At 1920x1080, UiScale is 1.0.
/// On smaller windows (e.g. 720p, 540p), UiScale scales down so the minimap,
/// hotbar, and menus maintain consistent screen proportions without overcrowding.
/// On higher resolution screens (1440p, 4K), UiScale scales up proportionally.
pub const REFERENCE_WIDTH: f32 = 1920.0;
pub const REFERENCE_HEIGHT: f32 = 1080.0;

pub fn update_adaptive_ui_scale(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
) {
    let Ok(window) = primary_window.single() else {
        return;
    };

    let w = window.width();
    let h = window.height();
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let scale_w = w / REFERENCE_WIDTH;
    let scale_h = h / REFERENCE_HEIGHT;

    // Use minimum of horizontal and vertical scaling to ensure elements never overflow
    // horizontally or vertically on non-16:9 aspect ratios.
    let target_scale = scale_w.min(scale_h).clamp(0.40, 3.00);

    if (ui_scale.0 - target_scale).abs() > 0.002 {
        ui_scale.0 = target_scale;
    }
}
