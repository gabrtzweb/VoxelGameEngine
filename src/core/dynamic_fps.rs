use std::time::{Duration, Instant};

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    window::PrimaryWindow,
};

/// High-level dynamic FPS throttling state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum DynamicFpsStateKind {
    #[default]
    Normal,
    Unfocused,
    Idle,
    Battery,
}

impl DynamicFpsStateKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Active",
            Self::Unfocused => "Unfocused (Throttled)",
            Self::Idle => "Idle (Power Saving)",
            Self::Battery => "Battery Saving",
        }
    }
}

/// User-configurable dynamic FPS and power throttling settings.
#[derive(Resource, Debug, Clone, Reflect)]
pub struct DynamicFpsSettings {
    pub enabled: bool,
    pub unfocused_target_fps: u32,
    pub idle_target_fps: u32,
    pub battery_target_fps: u32,
    pub idle_timeout_secs: f32,
}

impl Default for DynamicFpsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            unfocused_target_fps: 15,
            idle_target_fps: 30,
            battery_target_fps: 60,
            idle_timeout_secs: 60.0,
        }
    }
}

/// Runtime power, input, and frame-pacing tracking state.
#[derive(Resource, Debug)]
pub struct DynamicFpsState {
    pub is_focused: bool,
    pub is_idle: bool,
    pub on_battery: bool,
    pub state_kind: DynamicFpsStateKind,
    pub target_fps: Option<u32>,
    pub last_input_instant: Instant,
    pub last_frame_instant: Instant,
    pub last_battery_check: Instant,
}

impl Default for DynamicFpsState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            is_focused: true,
            is_idle: false,
            on_battery: false,
            state_kind: DynamicFpsStateKind::Normal,
            target_fps: None,
            last_input_instant: now,
            last_frame_instant: now,
            last_battery_check: now - Duration::from_secs(10),
        }
    }
}

#[cfg(target_os = "windows")]
mod sys_power {
    #[repr(C)]
    struct SystemPowerStatus {
        ac_line_status: u8,
        battery_flag: u8,
        battery_life_percent: u8,
        system_status_flag: u8,
        battery_life_time: u32,
        battery_full_life_time: u32,
    }

    unsafe extern "system" {
        fn GetSystemPowerStatus(lp_system_power_status: *mut SystemPowerStatus) -> i32;
    }

    pub fn is_running_on_battery() -> bool {
        let mut status = SystemPowerStatus {
            ac_line_status: 255,
            battery_flag: 255,
            battery_life_percent: 255,
            system_status_flag: 0,
            battery_life_time: 0,
            battery_full_life_time: 0,
        };
        unsafe {
            if GetSystemPowerStatus(&mut status) != 0 {
                // ac_line_status: 0 = Offline (running on battery), 1 = Online (AC power), 255 = Unknown
                return status.ac_line_status == 0;
            }
        }
        false
    }
}

#[cfg(not(target_os = "windows"))]
mod sys_power {
    pub fn is_running_on_battery() -> bool {
        false
    }
}

/// Tracks keyboard, mouse clicks, motion, and scroll events to register active user presence.
pub fn dynamic_fps_input_tracker(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut state: ResMut<DynamicFpsState>,
) {
    let has_input = keyboard.get_just_pressed().next().is_some()
        || keyboard.get_pressed().next().is_some()
        || mouse_buttons.get_just_pressed().next().is_some()
        || mouse_buttons.get_pressed().next().is_some()
        || mouse_motion.delta != Vec2::ZERO
        || mouse_scroll.delta != Vec2::ZERO;

    if has_input {
        state.last_input_instant = Instant::now();
        state.is_idle = false;
    }
}

/// Updates window focus, battery status, and evaluates the active power state and target FPS.
pub fn dynamic_fps_state_update(
    settings: Res<DynamicFpsSettings>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<DynamicFpsState>,
) {
    let now = Instant::now();

    // Check battery status periodically every 5 seconds to eliminate OS query overhead
    if now.duration_since(state.last_battery_check).as_secs_f32() > 5.0 {
        state.on_battery = sys_power::is_running_on_battery();
        state.last_battery_check = now;
    }

    // Window focus status
    if let Ok(window) = window_query.single() {
        state.is_focused = window.focused;
    }

    // Idle / AFK status
    let idle_elapsed = now.duration_since(state.last_input_instant).as_secs_f32();
    state.is_idle = idle_elapsed >= settings.idle_timeout_secs;

    if !settings.enabled {
        state.state_kind = DynamicFpsStateKind::Normal;
        state.target_fps = None;
        return;
    }

    // Priority 1: Unfocused / Background window
    if !state.is_focused {
        state.state_kind = DynamicFpsStateKind::Unfocused;
        state.target_fps = Some(settings.unfocused_target_fps);
    }
    // Priority 2: Idle / AFK with zero player activity
    else if state.is_idle {
        state.state_kind = DynamicFpsStateKind::Idle;
        state.target_fps = Some(settings.idle_target_fps);
    }
    // Priority 3: Laptop running on battery power
    else if state.on_battery {
        state.state_kind = DynamicFpsStateKind::Battery;
        state.target_fps = Some(settings.battery_target_fps);
    }
    // Priority 4: Normal active gameplay
    else {
        state.state_kind = DynamicFpsStateKind::Normal;
        state.target_fps = None;
    }
}

/// Precision frame pacer that yields CPU/GPU execution when throttled.
pub fn dynamic_fps_pacer(mut state: ResMut<DynamicFpsState>) {
    let now = Instant::now();
    let elapsed = now.duration_since(state.last_frame_instant);

    if let Some(target_fps) = state.target_fps
        && target_fps > 0
    {
        let target_frame_duration = Duration::from_secs_f64(1.0 / target_fps as f64);
        if elapsed < target_frame_duration {
            let sleep_duration = target_frame_duration - elapsed;
            std::thread::sleep(sleep_duration);
        }
    }

    state.last_frame_instant = Instant::now();
}

pub struct DynamicFpsPlugin;

impl Plugin for DynamicFpsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DynamicFpsSettings>()
            .init_resource::<DynamicFpsState>()
            .register_type::<DynamicFpsSettings>()
            .register_type::<DynamicFpsStateKind>()
            .add_systems(
                PreUpdate,
                (dynamic_fps_input_tracker, dynamic_fps_state_update).chain(),
            )
            .add_systems(PostUpdate, dynamic_fps_pacer);
    }
}
