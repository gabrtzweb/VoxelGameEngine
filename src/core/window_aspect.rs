use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};

pub struct WindowAspectPlugin;

impl Plugin for WindowAspectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowAspectState>()
            .add_systems(Update, enforce_aspect_ratio_16_9);
    }
}

pub const TARGET_ASPECT_RATIO: f32 = 16.0 / 9.0;
pub const MIN_WINDOW_WIDTH: f32 = 640.0;
pub const MIN_WINDOW_HEIGHT: f32 = 360.0;

/// Tracks window size history and whether 16:9 aspect ratio locking is active.
#[derive(Resource)]
pub struct WindowAspectState {
    pub last_width: f32,
    pub last_height: f32,
    pub is_locked: bool,
    pub initial_frames: u32,
}

impl Default for WindowAspectState {
    fn default() -> Self {
        Self {
            last_width: 1280.0,
            last_height: 720.0,
            is_locked: true,
            initial_frames: 5,
        }
    }
}

/// Enforces 16:9 aspect ratio in windowed mode by snapping dimensions in real-time when resized.
pub fn enforce_aspect_ratio_16_9(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut state: ResMut<WindowAspectState>,
) {
    if !state.is_locked {
        return;
    }

    let Ok(mut window) = primary_window.single_mut() else {
        return;
    };

    // Only enforce 16:9 snapping in Windowed mode.
    // Fullscreen and borderless modes must match the monitor's native display mode.
    if window.mode != WindowMode::Windowed {
        state.last_width = window.width();
        state.last_height = window.height();
        return;
    }

    let current_w = window.width();
    let current_h = window.height();

    if current_w <= 0.0 || current_h <= 0.0 || !current_w.is_finite() || !current_h.is_finite() {
        return;
    }

    // Allow window to initialize cleanly during startup frames
    if state.initial_frames > 0 {
        state.initial_frames -= 1;
        state.last_width = current_w;
        state.last_height = current_h;
        return;
    }

    let dw = (current_w - state.last_width).abs();
    let dh = (current_h - state.last_height).abs();

    // If dimensions have not changed by at least 1 logical pixel, do nothing.
    if dw < 1.0 && dh < 1.0 {
        return;
    }

    let current_aspect = current_w / current_h;
    // If the aspect ratio is already within a small margin of 16:9, update state without setting.
    if (current_aspect - TARGET_ASPECT_RATIO).abs() < 0.015 {
        state.last_width = current_w;
        state.last_height = current_h;
        return;
    }

    // Determine dominant resize direction.
    let (target_w, target_h) = if dw >= dh {
        let adjusted_w = current_w.max(MIN_WINDOW_WIDTH);
        let adjusted_h = (adjusted_w / TARGET_ASPECT_RATIO)
            .round()
            .max(MIN_WINDOW_HEIGHT);
        (adjusted_w, adjusted_h)
    } else {
        let adjusted_h = current_h.max(MIN_WINDOW_HEIGHT);
        let adjusted_w = (adjusted_h * TARGET_ASPECT_RATIO)
            .round()
            .max(MIN_WINDOW_WIDTH);
        (adjusted_w, adjusted_h)
    };

    state.last_width = target_w;
    state.last_height = target_h;
    window.resolution.set(target_w, target_h);
}
