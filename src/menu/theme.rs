use bevy::prelude::*;

/// Unified pure-code Dark Iron / Forged Obsidian UI theme palette.
pub mod palette {
    use bevy::prelude::Color;

    pub const PANEL_BG: Color = Color::srgba(0.082, 0.092, 0.112, 0.96);
    pub const PANEL_HEADER_BG: Color = Color::srgba(0.112, 0.126, 0.154, 0.98);
    pub const PANEL_INNER_BG: Color = Color::srgba(0.058, 0.066, 0.082, 0.94);
    pub const PANEL_BORDER: Color = Color::srgba(0.25, 0.29, 0.36, 0.96);
    pub const PANEL_BORDER_SUBTLE: Color = Color::srgba(0.18, 0.21, 0.27, 0.90);

    pub const BAR_BG: Color = Color::srgba(0.062, 0.070, 0.088, 0.96);
    pub const BAR_BORDER: Color = Color::srgba(0.20, 0.24, 0.31, 0.90);

    pub const DRAWER_COL1_BG: Color = Color::srgba(0.055, 0.062, 0.078, 0.95);
    pub const DRAWER_COL2_BG: Color = Color::srgba(0.072, 0.082, 0.102, 0.94);
    pub const DRAWER_COL3_BG: Color = Color::srgba(0.088, 0.098, 0.122, 0.93);
    pub const DRAWER_DIVIDER: Color = Color::srgba(0.21, 0.25, 0.33, 0.85);
    pub const NAV_ITEM_HOVER_BG: Color = Color::srgba(0.16, 0.19, 0.25, 0.72);
    pub const NAV_ITEM_ACTIVE_BG: Color = Color::srgba(0.18, 0.22, 0.29, 0.92);
    pub const ACCENT_LINE: Color = Color::srgb(0.96, 0.84, 0.42);

    pub const DRAWER_COL1_WIDTH: f32 = 310.0;
    pub const DRAWER_COL2_WIDTH: f32 = 310.0;
    pub const DRAWER_COL3_WIDTH: f32 = 480.0;

    pub const SLOT_BG: Color = Color::srgba(0.110, 0.122, 0.146, 0.92);
    pub const SLOT_EMPTY_BG: Color = Color::srgba(0.085, 0.095, 0.115, 0.88);
    pub const SLOT_BORDER: Color = Color::srgba(0.22, 0.25, 0.31, 0.95);
    pub const SLOT_HOVER_BG: Color = Color::srgba(0.24, 0.30, 0.39, 0.94);
    pub const SLOT_HOVER_BORDER: Color = Color::srgba(0.58, 0.72, 0.90, 1.0);
    pub const SLOT_SELECTED_BG: Color = Color::srgba(0.22, 0.24, 0.20, 0.96);
    pub const SLOT_SELECTED_BORDER: Color = Color::srgba(0.96, 0.84, 0.42, 1.0);

    pub const BTN_NORMAL_BG: Color = Color::srgba(0.135, 0.152, 0.184, 0.96);
    pub const BTN_NORMAL_BORDER: Color = Color::srgba(0.29, 0.34, 0.42, 0.95);
    pub const BTN_HOVER_BG: Color = Color::srgba(0.235, 0.271, 0.322, 0.98);
    pub const BTN_HOVER_BORDER: Color = Color::srgba(0.55, 0.67, 0.82, 1.0);
    pub const BTN_PRESSED_BG: Color = Color::srgba(0.095, 0.110, 0.138, 1.0);
    pub const BTN_ACTIVE_BG: Color = Color::srgba(0.18, 0.27, 0.37, 0.98);
    pub const BTN_ACTIVE_BORDER: Color = Color::srgba(0.46, 0.72, 0.94, 1.0);

    pub const TAB_IDLE_BG: Color = Color::srgba(0.11, 0.124, 0.150, 0.92);
    pub const TAB_IDLE_BORDER: Color = Color::srgba(0.22, 0.26, 0.32, 0.90);
    pub const TAB_ACTIVE_BG: Color = Color::srgba(0.18, 0.21, 0.27, 0.98);
    pub const TAB_ACTIVE_BORDER: Color = Color::srgba(0.95, 0.84, 0.44, 1.0);

    pub const SCROLLER_TRACK_BG: Color = Color::srgba(0.055, 0.062, 0.078, 0.95);
    pub const SCROLLER_THUMB_BG: Color = Color::srgba(0.32, 0.38, 0.47, 0.98);
    pub const SCROLLER_THUMB_BORDER: Color = Color::srgba(0.52, 0.61, 0.74, 1.0);

    pub const HEALTH_BG: Color = Color::srgba(0.08, 0.07, 0.08, 0.95);
    pub const HEALTH_FILL: Color = Color::srgba(0.25, 0.74, 0.36, 1.0);

    pub const TEXT_PRIMARY: Color = Color::srgb(0.92, 0.94, 0.98);
    pub const TEXT_ACCENT: Color = Color::srgb(1.0, 0.90, 0.42);
    pub const TEXT_MUTED: Color = Color::srgb(0.62, 0.66, 0.74);
}

/// Pure-code button styling component replacing legacy texture-based button sprites.
#[derive(Component, Clone, Copy, Debug)]
pub struct ThemedButton {
    pub normal_bg: Color,
    pub normal_border: Color,
    pub hover_bg: Color,
    pub hover_border: Color,
    pub pressed_bg: Color,
}

impl Default for ThemedButton {
    fn default() -> Self {
        Self {
            normal_bg: palette::BTN_NORMAL_BG,
            normal_border: palette::BTN_NORMAL_BORDER,
            hover_bg: palette::BTN_HOVER_BG,
            hover_border: palette::BTN_HOVER_BORDER,
            pressed_bg: palette::BTN_PRESSED_BG,
        }
    }
}

impl ThemedButton {
    pub const fn subtle() -> Self {
        Self {
            normal_bg: palette::SLOT_BG,
            normal_border: palette::SLOT_BORDER,
            hover_bg: palette::BTN_HOVER_BG,
            hover_border: palette::BTN_HOVER_BORDER,
            pressed_bg: palette::BTN_PRESSED_BG,
        }
    }

    pub fn colors(self, hovered: bool, pressed: bool) -> (Color, Color) {
        if pressed {
            (self.pressed_bg, self.hover_border)
        } else if hovered {
            (self.hover_bg, self.hover_border)
        } else {
            (self.normal_bg, self.normal_border)
        }
    }
}
