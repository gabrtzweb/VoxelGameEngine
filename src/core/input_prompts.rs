use bevy::{picking::Pickable, prelude::*};

use super::controls::{ControlSettings, InputAction, InputBinding};

const TILEMAP_PATH: &str = "textures/interfaces/atlases/inputs_tilemap.png";
const TILEMAP_COLS: u32 = 34;
const TILEMAP_ROWS: u32 = 24;
const TILE_SIZE: u32 = 16;
const TILE_PADDING: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputPromptStyle {
    #[default]
    DarkSteel,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputPromptDevice {
    #[default]
    KeyboardMouse,
    Gamepad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputPromptTiles {
    Single(usize),
    Wide(usize, usize),
}

#[derive(Resource, Clone)]
pub struct InputPromptAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub style: InputPromptStyle,
    pub device: InputPromptDevice,
}

impl InputPromptAtlas {
    #[inline]
    const fn idx(col: usize, row: usize) -> usize {
        row * (TILEMAP_COLS as usize) + col
    }

    pub fn tiles_for_key(&self, key: KeyCode) -> InputPromptTiles {
        let row_offset = match self.style {
            InputPromptStyle::DarkSteel => 8,
            InputPromptStyle::Light => 0,
        };
        let r = |base_row: usize| base_row + row_offset;

        match key {
            // Function row (base row 0 -> dark row 8)
            KeyCode::Escape => InputPromptTiles::Single(Self::idx(17, r(0))),
            KeyCode::F1 => InputPromptTiles::Single(Self::idx(18, r(0))),
            KeyCode::F2 => InputPromptTiles::Single(Self::idx(19, r(0))),
            KeyCode::F3 => InputPromptTiles::Single(Self::idx(20, r(0))),
            KeyCode::F4 => InputPromptTiles::Single(Self::idx(21, r(0))),
            KeyCode::F5 => InputPromptTiles::Single(Self::idx(22, r(0))),
            KeyCode::F6 => InputPromptTiles::Single(Self::idx(23, r(0))),
            KeyCode::F7 => InputPromptTiles::Single(Self::idx(24, r(0))),
            KeyCode::F8 => InputPromptTiles::Single(Self::idx(25, r(0))),
            KeyCode::F9 => InputPromptTiles::Single(Self::idx(26, r(0))),
            KeyCode::F10 => InputPromptTiles::Single(Self::idx(27, r(0))),
            KeyCode::F11 => InputPromptTiles::Single(Self::idx(28, r(0))),
            KeyCode::F12 => InputPromptTiles::Single(Self::idx(29, r(0))),
            KeyCode::Backquote => InputPromptTiles::Single(Self::idx(30, r(0))),

            // Number row (base row 1 -> dark row 9)
            KeyCode::Digit1 | KeyCode::Numpad1 => InputPromptTiles::Single(Self::idx(17, r(1))),
            KeyCode::Digit2 | KeyCode::Numpad2 => InputPromptTiles::Single(Self::idx(18, r(1))),
            KeyCode::Digit3 | KeyCode::Numpad3 => InputPromptTiles::Single(Self::idx(19, r(1))),
            KeyCode::Digit4 | KeyCode::Numpad4 => InputPromptTiles::Single(Self::idx(20, r(1))),
            KeyCode::Digit5 | KeyCode::Numpad5 => InputPromptTiles::Single(Self::idx(21, r(1))),
            KeyCode::Digit6 | KeyCode::Numpad6 => InputPromptTiles::Single(Self::idx(22, r(1))),
            KeyCode::Digit7 | KeyCode::Numpad7 => InputPromptTiles::Single(Self::idx(23, r(1))),
            KeyCode::Digit8 | KeyCode::Numpad8 => InputPromptTiles::Single(Self::idx(24, r(1))),
            KeyCode::Digit9 | KeyCode::Numpad9 => InputPromptTiles::Single(Self::idx(25, r(1))),
            KeyCode::Digit0 | KeyCode::Numpad0 => InputPromptTiles::Single(Self::idx(26, r(1))),
            KeyCode::Minus | KeyCode::NumpadSubtract => {
                InputPromptTiles::Single(Self::idx(27, r(1)))
            }
            KeyCode::NumpadAdd => InputPromptTiles::Single(Self::idx(28, r(1))),
            KeyCode::Equal => InputPromptTiles::Single(Self::idx(29, r(1))),
            KeyCode::Backspace => InputPromptTiles::Wide(Self::idx(32, r(1)), Self::idx(33, r(1))),

            // QWERTY row (base row 2 -> dark row 10)
            KeyCode::KeyQ => InputPromptTiles::Single(Self::idx(17, r(2))),
            KeyCode::KeyW => InputPromptTiles::Single(Self::idx(18, r(2))),
            KeyCode::KeyE => InputPromptTiles::Single(Self::idx(19, r(2))),
            KeyCode::KeyR => InputPromptTiles::Single(Self::idx(20, r(2))),
            KeyCode::KeyT => InputPromptTiles::Single(Self::idx(21, r(2))),
            KeyCode::KeyY => InputPromptTiles::Single(Self::idx(22, r(2))),
            KeyCode::KeyU => InputPromptTiles::Single(Self::idx(23, r(2))),
            KeyCode::KeyI => InputPromptTiles::Single(Self::idx(24, r(2))),
            KeyCode::KeyO => InputPromptTiles::Single(Self::idx(25, r(2))),
            KeyCode::KeyP => InputPromptTiles::Single(Self::idx(26, r(2))),
            KeyCode::BracketLeft => InputPromptTiles::Single(Self::idx(27, r(2))),
            KeyCode::BracketRight => InputPromptTiles::Single(Self::idx(28, r(2))),
            KeyCode::Backslash => InputPromptTiles::Single(Self::idx(31, r(2))),

            // ASDF row (base row 3 -> dark row 11)
            KeyCode::KeyA => InputPromptTiles::Single(Self::idx(18, r(3))),
            KeyCode::KeyS => InputPromptTiles::Single(Self::idx(19, r(3))),
            KeyCode::KeyD => InputPromptTiles::Single(Self::idx(20, r(3))),
            KeyCode::KeyF => InputPromptTiles::Single(Self::idx(21, r(3))),
            KeyCode::KeyG => InputPromptTiles::Single(Self::idx(22, r(3))),
            KeyCode::KeyH => InputPromptTiles::Single(Self::idx(23, r(3))),
            KeyCode::KeyJ => InputPromptTiles::Single(Self::idx(24, r(3))),
            KeyCode::KeyK => InputPromptTiles::Single(Self::idx(25, r(3))),
            KeyCode::KeyL => InputPromptTiles::Single(Self::idx(26, r(3))),
            KeyCode::Quote => InputPromptTiles::Single(Self::idx(27, r(3))),
            KeyCode::Semicolon => InputPromptTiles::Single(Self::idx(30, r(3))),
            KeyCode::NumpadMultiply => InputPromptTiles::Single(Self::idx(31, r(3))),
            KeyCode::Enter | KeyCode::NumpadEnter => {
                InputPromptTiles::Wide(Self::idx(32, r(3)), Self::idx(33, r(3)))
            }

            // ZXCV + Arrows row (base row 4 -> dark row 12)
            KeyCode::Space => InputPromptTiles::Single(Self::idx(17, r(4))),
            KeyCode::KeyZ => InputPromptTiles::Single(Self::idx(19, r(4))),
            KeyCode::KeyX => InputPromptTiles::Single(Self::idx(20, r(4))),
            KeyCode::KeyC => InputPromptTiles::Single(Self::idx(21, r(4))),
            KeyCode::KeyV => InputPromptTiles::Single(Self::idx(22, r(4))),
            KeyCode::KeyB => InputPromptTiles::Single(Self::idx(23, r(4))),
            KeyCode::KeyN => InputPromptTiles::Single(Self::idx(24, r(4))),
            KeyCode::KeyM => InputPromptTiles::Single(Self::idx(25, r(4))),
            KeyCode::Comma => InputPromptTiles::Single(Self::idx(27, r(6))),
            KeyCode::Period => InputPromptTiles::Single(Self::idx(27, r(5))),
            KeyCode::Slash | KeyCode::NumpadDivide => InputPromptTiles::Single(Self::idx(29, r(4))),
            KeyCode::ArrowUp => InputPromptTiles::Single(Self::idx(30, r(4))),
            KeyCode::ArrowRight => InputPromptTiles::Single(Self::idx(31, r(4))),
            KeyCode::ArrowDown => InputPromptTiles::Single(Self::idx(32, r(4))),
            KeyCode::ArrowLeft => InputPromptTiles::Single(Self::idx(33, r(4))),

            // Wide modifier & navigation keys (base rows 5..=7 -> dark rows 13..=15)
            KeyCode::AltLeft | KeyCode::AltRight => {
                InputPromptTiles::Wide(Self::idx(17, r(5)), Self::idx(18, r(5)))
            }
            KeyCode::Tab => InputPromptTiles::Wide(Self::idx(19, r(5)), Self::idx(20, r(5))),
            KeyCode::Delete => InputPromptTiles::Wide(Self::idx(21, r(5)), Self::idx(22, r(5))),
            KeyCode::End => InputPromptTiles::Wide(Self::idx(23, r(5)), Self::idx(24, r(5))),
            KeyCode::NumLock => InputPromptTiles::Wide(Self::idx(25, r(5)), Self::idx(26, r(5))),

            KeyCode::ControlLeft | KeyCode::ControlRight => {
                InputPromptTiles::Wide(Self::idx(17, r(6)), Self::idx(18, r(6)))
            }
            KeyCode::CapsLock => InputPromptTiles::Wide(Self::idx(19, r(6)), Self::idx(20, r(6))),
            KeyCode::Home => InputPromptTiles::Wide(Self::idx(21, r(6)), Self::idx(22, r(6))),
            KeyCode::PageUp => InputPromptTiles::Wide(Self::idx(23, r(6)), Self::idx(24, r(6))),
            KeyCode::PageDown => InputPromptTiles::Wide(Self::idx(25, r(6)), Self::idx(26, r(6))),

            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                InputPromptTiles::Wide(Self::idx(17, r(7)), Self::idx(18, r(7)))
            }
            KeyCode::Insert => InputPromptTiles::Wide(Self::idx(19, r(7)), Self::idx(20, r(7))),
            KeyCode::PrintScreen => {
                InputPromptTiles::Wide(Self::idx(21, r(7)), Self::idx(22, r(7)))
            }
            KeyCode::ScrollLock => InputPromptTiles::Wide(Self::idx(23, r(7)), Self::idx(24, r(7))),
            KeyCode::Pause => InputPromptTiles::Wide(Self::idx(25, r(7)), Self::idx(26, r(7))),

            // Blank dark/light keycap fallback
            _ => InputPromptTiles::Single(Self::idx(29, r(6))),
        }
    }

    pub fn tiles_for_mouse(&self, button: MouseButton) -> InputPromptTiles {
        let row = match self.style {
            InputPromptStyle::DarkSteel => 3,
            InputPromptStyle::Light => 2,
        };
        let col = match button {
            MouseButton::Left => 9,
            MouseButton::Right => 10,
            MouseButton::Middle => 11,
            _ => 8,
        };
        InputPromptTiles::Single(Self::idx(col, row))
    }

    pub fn tiles_for_binding(&self, binding: InputBinding) -> InputPromptTiles {
        match binding {
            InputBinding::Key(key) => self.tiles_for_key(key),
            InputBinding::Mouse(btn) => self.tiles_for_mouse(btn),
        }
    }

    pub fn tiles_for_action(
        &self,
        action: InputAction,
        controls: Option<&ControlSettings>,
    ) -> InputPromptTiles {
        let binding = controls
            .map(|c| c.get_binding(action))
            .unwrap_or_else(|| ControlSettings::default().get_binding(action));
        self.tiles_for_binding(binding)
    }

    pub fn tiles_for_prompt(
        &self,
        prompt: DynamicInputPrompt,
        controls: Option<&ControlSettings>,
    ) -> InputPromptTiles {
        match prompt {
            DynamicInputPrompt::Action(action) => self.tiles_for_action(action, controls),
            DynamicInputPrompt::Key(key) => self.tiles_for_key(key),
            DynamicInputPrompt::Mouse(btn) => self.tiles_for_mouse(btn),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicInputPrompt {
    Action(InputAction),
    Key(KeyCode),
    Mouse(MouseButton),
}

#[derive(Component)]
struct InputPromptPrimaryTile;

#[derive(Component)]
struct InputPromptSecondaryTile;

pub fn spawn_input_prompt(
    parent: &mut ChildSpawnerCommands,
    atlas: &InputPromptAtlas,
    prompt: DynamicInputPrompt,
    controls: Option<&ControlSettings>,
    tile_px: f32,
) {
    let tiles = atlas.tiles_for_prompt(prompt, controls);
    let (primary_idx, secondary_idx) = match tiles {
        InputPromptTiles::Single(i) => (i, None),
        InputPromptTiles::Wide(a, b) => (a, Some(b)),
    };

    parent
        .spawn((
            prompt,
            Pickable::IGNORE,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(0.0),
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                InputPromptPrimaryTile,
                Pickable::IGNORE,
                ImageNode::from_atlas_image(
                    atlas.texture.clone(),
                    TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: primary_idx,
                    },
                ),
                Node {
                    width: px(tile_px),
                    height: px(tile_px),
                    ..default()
                },
            ));

            row.spawn((
                InputPromptSecondaryTile,
                Pickable::IGNORE,
                ImageNode::from_atlas_image(
                    atlas.texture.clone(),
                    TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: secondary_idx.unwrap_or(primary_idx),
                    },
                ),
                Node {
                    width: px(tile_px),
                    height: px(tile_px),
                    display: if secondary_idx.is_some() {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    ..default()
                },
            ));
        });
}

pub struct InputPromptsPlugin;

impl Plugin for InputPromptsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_input_prompt_atlas)
            .add_systems(Update, sync_dynamic_input_prompts);
    }
}

fn setup_input_prompt_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load(TILEMAP_PATH);
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE),
        TILEMAP_COLS,
        TILEMAP_ROWS,
        Some(UVec2::splat(TILE_PADDING)),
        None,
    ));

    commands.insert_resource(InputPromptAtlas {
        texture,
        layout,
        style: InputPromptStyle::DarkSteel,
        device: InputPromptDevice::KeyboardMouse,
    });
}

fn sync_dynamic_input_prompts(
    atlas: Option<Res<InputPromptAtlas>>,
    controls: Option<Res<ControlSettings>>,
    prompt_q: Query<(&DynamicInputPrompt, &Children)>,
    mut primary_q: Query<
        &mut ImageNode,
        (
            With<InputPromptPrimaryTile>,
            Without<InputPromptSecondaryTile>,
        ),
    >,
    mut secondary_q: Query<
        (&mut ImageNode, &mut Node),
        (
            With<InputPromptSecondaryTile>,
            Without<InputPromptPrimaryTile>,
        ),
    >,
) {
    let Some(atlas) = atlas else {
        return;
    };
    let controls_changed = controls.as_ref().is_some_and(|c| c.is_changed());
    if !controls_changed && !atlas.is_changed() {
        return;
    }

    for (&prompt, children) in &prompt_q {
        let tiles = atlas.tiles_for_prompt(prompt, controls.as_deref());
        let (primary_idx, secondary_idx) = match tiles {
            InputPromptTiles::Single(i) => (i, None),
            InputPromptTiles::Wide(a, b) => (a, Some(b)),
        };

        for child in children.iter() {
            if let Ok(mut img) = primary_q.get_mut(child)
                && let Some(ref mut ta) = img.texture_atlas
                && ta.index != primary_idx
            {
                ta.index = primary_idx;
            }
            if let Ok((mut img, mut node)) = secondary_q.get_mut(child) {
                if let Some(sec_idx) = secondary_idx {
                    if let Some(ref mut ta) = img.texture_atlas
                        && ta.index != sec_idx
                    {
                        ta.index = sec_idx;
                    }
                    if node.display != Display::Flex {
                        node.display = Display::Flex;
                    }
                } else if node.display != Display::None {
                    node.display = Display::None;
                }
            }
        }
    }
}
