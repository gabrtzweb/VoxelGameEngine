use bevy::{prelude::*, window::PrimaryWindow};
use crate::menu::{GameSettings, MenuState, ScreenMode};

#[cfg(target_os = "windows")]
pub fn confine_mouse_to_window(enable: bool) {
    #[repr(C)]
    struct RECT {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }
    unsafe extern "system" {
        fn ClipCursor(lpRect: *const RECT) -> i32;
        fn GetClientRect(hWnd: *mut core::ffi::c_void, lpRect: *mut RECT) -> i32;
        fn ClientToScreen(hWnd: *mut core::ffi::c_void, lpPoint: *mut POINT) -> i32;
        fn GetForegroundWindow() -> *mut core::ffi::c_void;
    }

    unsafe {
        if !enable {
            ClipCursor(std::ptr::null());
            return;
        }

        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return;
        }

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetClientRect(hwnd, &mut rect) == 0 {
            return;
        }

        let mut top_left = POINT {
            x: rect.left,
            y: rect.top,
        };
        let mut bottom_right = POINT {
            x: rect.right,
            y: rect.bottom,
        };

        ClientToScreen(hwnd, &mut top_left);
        ClientToScreen(hwnd, &mut bottom_right);

        let screen_rect = RECT {
            left: top_left.x,
            top: top_left.y,
            right: bottom_right.x,
            bottom: bottom_right.y,
        };

        ClipCursor(&screen_rect);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn confine_mouse_to_window(_enable: bool) {}

/// System monitoring window focus and screen mode to trap/confine cursor within the screen.
pub fn manage_cursor_confinement(
    window_query: Query<&Window, With<PrimaryWindow>>,
    game_settings: Option<Res<GameSettings>>,
    menu_state: Option<Res<State<MenuState>>>,
) {
    let Ok(window) = window_query.single() else {
        confine_mouse_to_window(false);
        return;
    };

    if !window.focused {
        confine_mouse_to_window(false);
        return;
    }

    let is_fullscreen_or_borderless = game_settings.as_ref().is_some_and(|s| {
        s.screen_mode == ScreenMode::BorderlessFullscreen
            || s.screen_mode == ScreenMode::ExclusiveFullscreen
    });

    // In borderless fullscreen or exclusive fullscreen, ALWAYS confine mouse to window so it never escapes to second monitor.
    // In windowed mode during gameplay (MenuState::None), also confine so mouse look doesn't slip out.
    let is_gameplay = menu_state.as_ref().is_some_and(|m| *m.get() == MenuState::None);

    if is_fullscreen_or_borderless || is_gameplay {
        confine_mouse_to_window(true);
    } else {
        confine_mouse_to_window(false);
    }
}

pub struct WindowClipPlugin;

impl Plugin for WindowClipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, manage_cursor_confinement);
    }
}
