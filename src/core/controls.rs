use std::collections::HashMap;

use bevy::prelude::*;

/// High-level engine input action classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum InputAction {
    // Movement
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Crouch,
    Sprint,

    // Gameplay & Interaction
    PrimaryAction,
    SecondaryAction,
    PickBlock,
    DropItem,
    ShapeCycle,
    ShapeInvert,
    Zoom,

    // Menus & Interfaces
    Inventory,
    WorldMap,
    Journal,
    MenuTabPrev,
    MenuTabNext,

    // Hotbar Quick Slots
    Hotbar1,
    Hotbar2,
    Hotbar3,
    Hotbar4,
    Hotbar5,
    Hotbar6,
    Hotbar7,
    Hotbar8,
    Hotbar9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    Movement,
    Gameplay,
    Interfaces,
    Hotbar,
}

impl InputAction {
    pub const ALL: &'static [InputAction] = &[
        InputAction::MoveForward,
        InputAction::MoveBackward,
        InputAction::MoveLeft,
        InputAction::MoveRight,
        InputAction::Jump,
        InputAction::Crouch,
        InputAction::Sprint,
        InputAction::PrimaryAction,
        InputAction::SecondaryAction,
        InputAction::PickBlock,
        InputAction::DropItem,
        InputAction::ShapeCycle,
        InputAction::ShapeInvert,
        InputAction::Zoom,
        InputAction::Inventory,
        InputAction::WorldMap,
        InputAction::Journal,
        InputAction::MenuTabPrev,
        InputAction::MenuTabNext,
        InputAction::Hotbar1,
        InputAction::Hotbar2,
        InputAction::Hotbar3,
        InputAction::Hotbar4,
        InputAction::Hotbar5,
        InputAction::Hotbar6,
        InputAction::Hotbar7,
        InputAction::Hotbar8,
        InputAction::Hotbar9,
    ];

    pub fn category(self) -> ActionCategory {
        match self {
            Self::MoveForward
            | Self::MoveBackward
            | Self::MoveLeft
            | Self::MoveRight
            | Self::Jump
            | Self::Crouch
            | Self::Sprint => ActionCategory::Movement,

            Self::PrimaryAction
            | Self::SecondaryAction
            | Self::PickBlock
            | Self::DropItem
            | Self::ShapeCycle
            | Self::ShapeInvert
            | Self::Zoom => ActionCategory::Gameplay,

            Self::Inventory
            | Self::WorldMap
            | Self::Journal
            | Self::MenuTabPrev
            | Self::MenuTabNext => ActionCategory::Interfaces,

            Self::Hotbar1
            | Self::Hotbar2
            | Self::Hotbar3
            | Self::Hotbar4
            | Self::Hotbar5
            | Self::Hotbar6
            | Self::Hotbar7
            | Self::Hotbar8
            | Self::Hotbar9 => ActionCategory::Hotbar,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::MoveForward => "controls.action.move_forward",
            Self::MoveBackward => "controls.action.move_backward",
            Self::MoveLeft => "controls.action.move_left",
            Self::MoveRight => "controls.action.move_right",
            Self::Jump => "controls.action.jump",
            Self::Crouch => "controls.action.crouch",
            Self::Sprint => "controls.action.sprint",

            Self::PrimaryAction => "controls.action.primary",
            Self::SecondaryAction => "controls.action.secondary",
            Self::PickBlock => "controls.action.pick_block",
            Self::DropItem => "controls.action.drop_item",
            Self::ShapeCycle => "controls.action.shape_cycle",
            Self::ShapeInvert => "controls.action.shape_invert",
            Self::Zoom => "controls.action.zoom",

            Self::Inventory => "controls.action.inventory",
            Self::WorldMap => "controls.action.world_map",
            Self::Journal => "controls.action.journal",
            Self::MenuTabPrev => "controls.action.tab_prev",
            Self::MenuTabNext => "controls.action.tab_next",

            Self::Hotbar1 => "controls.action.hotbar_1",
            Self::Hotbar2 => "controls.action.hotbar_2",
            Self::Hotbar3 => "controls.action.hotbar_3",
            Self::Hotbar4 => "controls.action.hotbar_4",
            Self::Hotbar5 => "controls.action.hotbar_5",
            Self::Hotbar6 => "controls.action.hotbar_6",
            Self::Hotbar7 => "controls.action.hotbar_7",
            Self::Hotbar8 => "controls.action.hotbar_8",
            Self::Hotbar9 => "controls.action.hotbar_9",
        }
    }

    pub fn config_key(self) -> &'static str {
        match self {
            Self::MoveForward => "move_forward",
            Self::MoveBackward => "move_backward",
            Self::MoveLeft => "move_left",
            Self::MoveRight => "move_right",
            Self::Jump => "jump",
            Self::Crouch => "crouch",
            Self::Sprint => "sprint",

            Self::PrimaryAction => "primary_action",
            Self::SecondaryAction => "secondary_action",
            Self::PickBlock => "pick_block",
            Self::DropItem => "drop_item",
            Self::ShapeCycle => "shape_cycle",
            Self::ShapeInvert => "shape_invert",
            Self::Zoom => "zoom",

            Self::Inventory => "inventory",
            Self::WorldMap => "world_map",
            Self::Journal => "journal",
            Self::MenuTabPrev => "menu_tab_prev",
            Self::MenuTabNext => "menu_tab_next",

            Self::Hotbar1 => "hotbar_1",
            Self::Hotbar2 => "hotbar_2",
            Self::Hotbar3 => "hotbar_3",
            Self::Hotbar4 => "hotbar_4",
            Self::Hotbar5 => "hotbar_5",
            Self::Hotbar6 => "hotbar_6",
            Self::Hotbar7 => "hotbar_7",
            Self::Hotbar8 => "hotbar_8",
            Self::Hotbar9 => "hotbar_9",
        }
    }

    pub fn from_config_key(key: &str) -> Option<Self> {
        match key {
            "move_forward" => Some(Self::MoveForward),
            "move_backward" => Some(Self::MoveBackward),
            "move_left" => Some(Self::MoveLeft),
            "move_right" => Some(Self::MoveRight),
            "jump" => Some(Self::Jump),
            "crouch" => Some(Self::Crouch),
            "sprint" => Some(Self::Sprint),

            "primary_action" => Some(Self::PrimaryAction),
            "secondary_action" => Some(Self::SecondaryAction),
            "pick_block" => Some(Self::PickBlock),
            "drop_item" => Some(Self::DropItem),
            "shape_cycle" => Some(Self::ShapeCycle),
            "shape_invert" => Some(Self::ShapeInvert),
            "zoom" => Some(Self::Zoom),

            "inventory" => Some(Self::Inventory),
            "world_map" => Some(Self::WorldMap),
            "journal" => Some(Self::Journal),
            "menu_tab_prev" => Some(Self::MenuTabPrev),
            "menu_tab_next" => Some(Self::MenuTabNext),

            "hotbar_1" => Some(Self::Hotbar1),
            "hotbar_2" => Some(Self::Hotbar2),
            "hotbar_3" => Some(Self::Hotbar3),
            "hotbar_4" => Some(Self::Hotbar4),
            "hotbar_5" => Some(Self::Hotbar5),
            "hotbar_6" => Some(Self::Hotbar6),
            "hotbar_7" => Some(Self::Hotbar7),
            "hotbar_8" => Some(Self::Hotbar8),
            "hotbar_9" => Some(Self::Hotbar9),
            _ => None,
        }
    }
}

/// Generic input binding supporting either a keyboard key or a mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl InputBinding {
    #[inline]
    pub fn is_pressed(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Key(k) => keyboard.pressed(*k),
            Self::Mouse(m) => mouse.pressed(*m),
        }
    }

    #[inline]
    pub fn just_pressed(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Key(k) => keyboard.just_pressed(*k),
            Self::Mouse(m) => mouse.just_pressed(*m),
        }
    }

    #[inline]
    pub fn just_released(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Key(k) => keyboard.just_released(*k),
            Self::Mouse(m) => mouse.just_released(*m),
        }
    }

    /// User-facing display text for the binding, with localized mouse names where appropriate.
    pub fn display_name(&self, loc: Option<&crate::core::Localization>) -> String {
        match self {
            Self::Mouse(MouseButton::Left) => {
                loc.map_or("Left Click", |l| l.t("controls.mouse.left")).to_string()
            }
            Self::Mouse(MouseButton::Right) => {
                loc.map_or("Right Click", |l| l.t("controls.mouse.right")).to_string()
            }
            Self::Mouse(MouseButton::Middle) => {
                loc.map_or("Middle Click", |l| l.t("controls.mouse.middle")).to_string()
            }
            Self::Mouse(MouseButton::Back) => "Mouse 4".to_string(),
            Self::Mouse(MouseButton::Forward) => "Mouse 5".to_string(),
            Self::Mouse(MouseButton::Other(n)) => format!("Mouse {n}"),

            Self::Key(k) => key_code_display_name(*k),
        }
    }

    /// Formats the binding into a stable string representation for `local_config.json`.
    pub fn to_config_str(&self) -> String {
        match self {
            Self::Key(k) => format!("Key:{:?}", k),
            Self::Mouse(m) => format!("Mouse:{:?}", m),
        }
    }

    /// Parses a binding string from `local_config.json`.
    pub fn from_config_str(s: &str) -> Option<Self> {
        let (kind, val) = s.split_once(':')?;
        match kind {
            "Key" => parse_key_code(val).map(Self::Key),
            "Mouse" => match val {
                "Left" => Some(Self::Mouse(MouseButton::Left)),
                "Right" => Some(Self::Mouse(MouseButton::Right)),
                "Middle" => Some(Self::Mouse(MouseButton::Middle)),
                "Back" => Some(Self::Mouse(MouseButton::Back)),
                "Forward" => Some(Self::Mouse(MouseButton::Forward)),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Helper converting a Bevy KeyCode into clean human-readable text.
fn key_code_display_name(k: KeyCode) -> String {
    match k {
        KeyCode::Space => "Space".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Escape => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::ShiftLeft => "L-Shift".to_string(),
        KeyCode::ShiftRight => "R-Shift".to_string(),
        KeyCode::ControlLeft => "L-Ctrl".to_string(),
        KeyCode::ControlRight => "R-Ctrl".to_string(),
        KeyCode::AltLeft => "L-Alt".to_string(),
        KeyCode::AltRight => "R-Alt".to_string(),
        KeyCode::ArrowUp => "Up".to_string(),
        KeyCode::ArrowDown => "Down".to_string(),
        KeyCode::ArrowLeft => "Left".to_string(),
        KeyCode::ArrowRight => "Right".to_string(),

        KeyCode::KeyA => "A".to_string(),
        KeyCode::KeyB => "B".to_string(),
        KeyCode::KeyC => "C".to_string(),
        KeyCode::KeyD => "D".to_string(),
        KeyCode::KeyE => "E".to_string(),
        KeyCode::KeyF => "F".to_string(),
        KeyCode::KeyG => "G".to_string(),
        KeyCode::KeyH => "H".to_string(),
        KeyCode::KeyI => "I".to_string(),
        KeyCode::KeyJ => "J".to_string(),
        KeyCode::KeyK => "K".to_string(),
        KeyCode::KeyL => "L".to_string(),
        KeyCode::KeyM => "M".to_string(),
        KeyCode::KeyN => "N".to_string(),
        KeyCode::KeyO => "O".to_string(),
        KeyCode::KeyP => "P".to_string(),
        KeyCode::KeyQ => "Q".to_string(),
        KeyCode::KeyR => "R".to_string(),
        KeyCode::KeyS => "S".to_string(),
        KeyCode::KeyT => "T".to_string(),
        KeyCode::KeyU => "U".to_string(),
        KeyCode::KeyV => "V".to_string(),
        KeyCode::KeyW => "W".to_string(),
        KeyCode::KeyX => "X".to_string(),
        KeyCode::KeyY => "Y".to_string(),
        KeyCode::KeyZ => "Z".to_string(),

        KeyCode::Digit1 => "1".to_string(),
        KeyCode::Digit2 => "2".to_string(),
        KeyCode::Digit3 => "3".to_string(),
        KeyCode::Digit4 => "4".to_string(),
        KeyCode::Digit5 => "5".to_string(),
        KeyCode::Digit6 => "6".to_string(),
        KeyCode::Digit7 => "7".to_string(),
        KeyCode::Digit8 => "8".to_string(),
        KeyCode::Digit9 => "9".to_string(),
        KeyCode::Digit0 => "0".to_string(),

        KeyCode::F1 => "F1".to_string(),
        KeyCode::F2 => "F2".to_string(),
        KeyCode::F3 => "F3".to_string(),
        KeyCode::F4 => "F4".to_string(),
        KeyCode::F5 => "F5".to_string(),
        KeyCode::F6 => "F6".to_string(),
        KeyCode::F7 => "F7".to_string(),
        KeyCode::F8 => "F8".to_string(),
        KeyCode::F9 => "F9".to_string(),
        KeyCode::F10 => "F10".to_string(),
        KeyCode::F11 => "F11".to_string(),
        KeyCode::F12 => "F12".to_string(),

        _ => format!("{k:?}"),
    }
}

/// Helper parsing KeyCode from debug-formatted string.
fn parse_key_code(s: &str) -> Option<KeyCode> {
    match s {
        "Space" => Some(KeyCode::Space),
        "Enter" => Some(KeyCode::Enter),
        "Escape" => Some(KeyCode::Escape),
        "Backspace" => Some(KeyCode::Backspace),
        "Tab" => Some(KeyCode::Tab),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "ControlLeft" => Some(KeyCode::ControlLeft),
        "ControlRight" => Some(KeyCode::ControlRight),
        "AltLeft" => Some(KeyCode::AltLeft),
        "AltRight" => Some(KeyCode::AltRight),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),

        "KeyA" => Some(KeyCode::KeyA),
        "KeyB" => Some(KeyCode::KeyB),
        "KeyC" => Some(KeyCode::KeyC),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyE" => Some(KeyCode::KeyE),
        "KeyF" => Some(KeyCode::KeyF),
        "KeyG" => Some(KeyCode::KeyG),
        "KeyH" => Some(KeyCode::KeyH),
        "KeyI" => Some(KeyCode::KeyI),
        "KeyJ" => Some(KeyCode::KeyJ),
        "KeyK" => Some(KeyCode::KeyK),
        "KeyL" => Some(KeyCode::KeyL),
        "KeyM" => Some(KeyCode::KeyM),
        "KeyN" => Some(KeyCode::KeyN),
        "KeyO" => Some(KeyCode::KeyO),
        "KeyP" => Some(KeyCode::KeyP),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyR" => Some(KeyCode::KeyR),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyT" => Some(KeyCode::KeyT),
        "KeyU" => Some(KeyCode::KeyU),
        "KeyV" => Some(KeyCode::KeyV),
        "KeyW" => Some(KeyCode::KeyW),
        "KeyX" => Some(KeyCode::KeyX),
        "KeyY" => Some(KeyCode::KeyY),
        "KeyZ" => Some(KeyCode::KeyZ),

        "Digit1" => Some(KeyCode::Digit1),
        "Digit2" => Some(KeyCode::Digit2),
        "Digit3" => Some(KeyCode::Digit3),
        "Digit4" => Some(KeyCode::Digit4),
        "Digit5" => Some(KeyCode::Digit5),
        "Digit6" => Some(KeyCode::Digit6),
        "Digit7" => Some(KeyCode::Digit7),
        "Digit8" => Some(KeyCode::Digit8),
        "Digit9" => Some(KeyCode::Digit9),
        "Digit0" => Some(KeyCode::Digit0),

        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        _ => None,
    }
}

/// Global resource holding all active player control settings and input bindings.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ControlSettings {
    /// Mouse sensitivity multiplier applied to camera rotation (0.2x to 3.0x, default 1.0).
    pub mouse_sensitivity: f32,
    /// Invert the vertical Y-axis for mouse look (default false).
    pub invert_mouse_y: bool,
    /// Active binding mapping per action.
    pub bindings: HashMap<InputAction, InputBinding>,
}

impl Default for ControlSettings {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // Movement
        bindings.insert(InputAction::MoveForward, InputBinding::Key(KeyCode::KeyW));
        bindings.insert(InputAction::MoveBackward, InputBinding::Key(KeyCode::KeyS));
        bindings.insert(InputAction::MoveLeft, InputBinding::Key(KeyCode::KeyA));
        bindings.insert(InputAction::MoveRight, InputBinding::Key(KeyCode::KeyD));
        bindings.insert(InputAction::Jump, InputBinding::Key(KeyCode::Space));
        bindings.insert(InputAction::Crouch, InputBinding::Key(KeyCode::ControlLeft));
        bindings.insert(InputAction::Sprint, InputBinding::Key(KeyCode::ShiftLeft));

        // Gameplay
        bindings.insert(InputAction::PrimaryAction, InputBinding::Mouse(MouseButton::Left));
        bindings.insert(InputAction::SecondaryAction, InputBinding::Mouse(MouseButton::Right));
        bindings.insert(InputAction::PickBlock, InputBinding::Mouse(MouseButton::Middle));
        bindings.insert(InputAction::DropItem, InputBinding::Key(KeyCode::KeyQ));
        bindings.insert(InputAction::ShapeCycle, InputBinding::Key(KeyCode::KeyR));
        bindings.insert(InputAction::ShapeInvert, InputBinding::Key(KeyCode::KeyT));
        bindings.insert(InputAction::Zoom, InputBinding::Key(KeyCode::KeyZ));

        // Interfaces
        bindings.insert(InputAction::Inventory, InputBinding::Key(KeyCode::KeyI));
        bindings.insert(InputAction::WorldMap, InputBinding::Key(KeyCode::KeyM));
        bindings.insert(InputAction::Journal, InputBinding::Key(KeyCode::KeyJ));
        bindings.insert(InputAction::MenuTabPrev, InputBinding::Key(KeyCode::KeyQ));
        bindings.insert(InputAction::MenuTabNext, InputBinding::Key(KeyCode::KeyE));

        // Hotbar
        bindings.insert(InputAction::Hotbar1, InputBinding::Key(KeyCode::Digit1));
        bindings.insert(InputAction::Hotbar2, InputBinding::Key(KeyCode::Digit2));
        bindings.insert(InputAction::Hotbar3, InputBinding::Key(KeyCode::Digit3));
        bindings.insert(InputAction::Hotbar4, InputBinding::Key(KeyCode::Digit4));
        bindings.insert(InputAction::Hotbar5, InputBinding::Key(KeyCode::Digit5));
        bindings.insert(InputAction::Hotbar6, InputBinding::Key(KeyCode::Digit6));
        bindings.insert(InputAction::Hotbar7, InputBinding::Key(KeyCode::Digit7));
        bindings.insert(InputAction::Hotbar8, InputBinding::Key(KeyCode::Digit8));
        bindings.insert(InputAction::Hotbar9, InputBinding::Key(KeyCode::Digit9));

        Self {
            mouse_sensitivity: 1.0,
            invert_mouse_y: false,
            bindings,
        }
    }
}

impl ControlSettings {
    #[inline]
    pub fn get_binding(&self, action: InputAction) -> InputBinding {
        self.bindings.get(&action).copied().unwrap_or_else(|| {
            Self::default()
                .bindings
                .get(&action)
                .copied()
                .unwrap_or(InputBinding::Key(KeyCode::Space))
        })
    }

    pub fn set_binding(&mut self, action: InputAction, binding: InputBinding) {
        self.bindings.insert(action, binding);
    }

    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    #[inline]
    pub fn is_action_pressed(
        &self,
        action: InputAction,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.get_binding(action).is_pressed(keyboard, mouse)
    }

    #[inline]
    pub fn is_action_just_pressed(
        &self,
        action: InputAction,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.get_binding(action).just_pressed(keyboard, mouse)
    }

    #[inline]
    pub fn is_action_just_released(
        &self,
        action: InputAction,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.get_binding(action).just_released(keyboard, mouse)
    }

    /// Checks if a given action's current binding conflicts with any other action in the same category.
    pub fn has_conflict(&self, action: InputAction) -> bool {
        let binding = self.get_binding(action);
        let category = action.category();

        for &other_action in InputAction::ALL {
            if other_action == action {
                continue;
            }
            if other_action.category() == category && self.get_binding(other_action) == binding {
                return true;
            }
        }
        false
    }
}
