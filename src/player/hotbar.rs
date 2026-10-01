use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, ui::widget::TextShadow};

use crate::core::{AppFont, text_shadow_default};
use crate::gameplay::{BlockIcons, SelectedVoxel};
use crate::player::InspectorInteraction;
use crate::world::Voxel;

pub const HOTBAR_SLOT_COUNT: usize = 8;

#[derive(Resource, Debug, Clone)]
pub struct Hotbar {
    pub slots: [Option<Voxel>; HOTBAR_SLOT_COUNT],
    pub active_slot: usize,
}

impl Default for Hotbar {
    fn default() -> Self {
        Self {
            slots: [
                Some(Voxel::Soil_Grass),
                Some(Voxel::Soil_Dirt),
                Some(Voxel::Rock_Stone),
                Some(Voxel::Soil_Sand),
                Some(Voxel::Liquid_Water),
                Some(Voxel::Emit_Warm_Light),
                None,
                None,
            ],
            active_slot: 0,
        }
    }
}

pub const TOOLTIP_HOLD_SECS: f32 = 1.4;
pub const TOOLTIP_FADE_SECS: f32 = 0.8;
pub const TOOLTIP_TOTAL_SECS: f32 = TOOLTIP_HOLD_SECS + TOOLTIP_FADE_SECS;

#[derive(Component)]
pub struct HotbarSlotUi {
    pub index: usize,
}

#[derive(Component)]
pub struct HotbarSlotIcon {
    pub index: usize,
}

#[derive(Resource, Debug, Clone)]
pub struct HotbarTooltipState {
    pub text: String,
    pub timer: f32,
    pub last_slot: usize,
    pub last_voxel: Option<Voxel>,
    pub initialized: bool,
}

impl Default for HotbarTooltipState {
    fn default() -> Self {
        Self {
            text: String::new(),
            timer: 0.0,
            last_slot: 0,
            last_voxel: None,
            initialized: false,
        }
    }
}

#[derive(Component)]
pub struct HotbarTooltipText;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hotbar>()
            .init_resource::<HotbarTooltipState>()
            .add_systems(
                Startup,
                setup_hotbar_ui.after(crate::gameplay::setup_block_icons),
            )
            .add_systems(
                Update,
                (handle_hotbar_input, sync_hotbar_ui, update_hotbar_tooltip).chain(),
            );
    }
}

fn setup_hotbar_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icons: Res<BlockIcons>,
    app_font: Option<Res<AppFont>>,
) {
    let initial_hotbar = Hotbar::default();
    let font_handle = app_font.as_ref().map(|f| f.source());
    let hotbar_texture = asset_server.load("textures/interfaces/containers/hotbar.png");

    // Scale multiplier for hotbar UI rendering (3x integer scaling = 768x96 px)
    const HOTBAR_SCALE: f32 = 3.0;
    let tray_width = 256.0 * HOTBAR_SCALE;
    let tray_height = 32.0 * HOTBAR_SCALE;
    let slot_stride = 20.0 * HOTBAR_SCALE;
    let slot_left_offset = 49.0 * HOTBAR_SCALE;
    let slot_top = 7.0 * HOTBAR_SCALE;
    let slot_size = 18.0 * HOTBAR_SCALE;
    let icon_size = 16.0 * HOTBAR_SCALE;
    let border_width = 1.0 * HOTBAR_SCALE;

    // Centered bottom container spanning screen width
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(16.0),
                left: px(0.0),
                right: px(0.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(150),
        ))
        .with_children(|parent| {
            // Held item name tooltip (Minecraft-style centered text floating above hotbar)
            let mut tooltip_font = TextFont {
                font_size: FontSize::Px(20.0),
                ..default()
            };
            if let Some(ref font) = font_handle {
                tooltip_font.font = font.clone();
            }

            parent.spawn((
                HotbarTooltipText,
                Text::new(""),
                tooltip_font,
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                text_shadow_default(),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(tray_height + 14.0),
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Visibility::Hidden,
            ));

            // Scaled hotbar texture (256x32 px at 3x = 768x96 px)
            parent
                .spawn((
                    ImageNode {
                        image: hotbar_texture,
                        ..default()
                    },
                    Node {
                        width: px(tray_width),
                        height: px(tray_height),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                ))
                .with_children(|tray| {
                    for index in 0..HOTBAR_SLOT_COUNT {
                        let is_active = index == initial_hotbar.active_slot;
                        let initial_voxel = initial_hotbar.slots[index];

                        let border_color = if is_active {
                            Color::srgb(1.0, 0.85, 0.30)
                        } else {
                            Color::NONE
                        };

                        let bg_color = if is_active {
                            Color::srgba(1.0, 1.0, 1.0, 0.12)
                        } else {
                            Color::NONE
                        };

                        let icon_handle = initial_voxel
                            .map(|v| icons.get(v))
                            .unwrap_or_else(|| icons.get(Voxel::Rock_Stone));

                        let icon_visibility = if initial_voxel.is_some() {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        };

                        tray.spawn((
                            HotbarSlotUi { index },
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(slot_left_offset + index as f32 * slot_stride),
                                top: px(slot_top),
                                width: px(slot_size),
                                height: px(slot_size),
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(border_width)),
                                border_radius: BorderRadius::all(px(border_width)),
                                ..default()
                            },
                            BackgroundColor(bg_color),
                            BorderColor::all(border_color),
                        ))
                        .with_children(|slot| {
                            // Centered 2D item icon (rendered first, below slot number)
                            slot.spawn((
                                HotbarSlotIcon { index },
                                ImageNode {
                                    image: icon_handle,
                                    ..default()
                                },
                                Node {
                                    width: px(icon_size),
                                    height: px(icon_size),
                                    ..default()
                                },
                                icon_visibility,
                            ));

                            // Slot index number (1 through 8, rendered in front with ZIndex)
                            let mut num_font = TextFont {
                                font_size: FontSize::Px(16.0),
                                ..default()
                            };
                            if let Some(ref font) = font_handle {
                                num_font.font = font.clone();
                            }

                            slot.spawn((
                                Text::new(format!("{}", index + 1)),
                                num_font,
                                TextColor(Color::srgba(0.95, 0.95, 0.95, 0.90)),
                                text_shadow_default(),
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: px(2.0),
                                    left: px(4.0),
                                    ..default()
                                },
                                ZIndex(10),
                            ));
                        });
                    }
                });
        });
}

fn handle_hotbar_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    mut hotbar: ResMut<Hotbar>,
    mut selected: ResMut<SelectedVoxel>,
) {
    if menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }
    let digit_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ];

    for (idx, key) in digit_keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            hotbar.active_slot = idx;
        }
    }

    // When holding Z (camera zoom), mouse wheel scroll adjusts zoom level instead of cycling hotbar
    if !keyboard.pressed(KeyCode::KeyZ) {
        let scroll = mouse_scroll.delta.y;
        if scroll > 0.05 {
            // Scroll up = previous slot
            hotbar.active_slot = (hotbar.active_slot + HOTBAR_SLOT_COUNT - 1) % HOTBAR_SLOT_COUNT;
        } else if scroll < -0.05 {
            // Scroll down = next slot
            hotbar.active_slot = (hotbar.active_slot + 1) % HOTBAR_SLOT_COUNT;
        }
    }

    // Q key clears the active hotbar slot (giving the player an empty hand)
    if keyboard.just_pressed(KeyCode::KeyQ) {
        let active = hotbar.active_slot;
        hotbar.slots[active] = None;
    }

    let active = hotbar.active_slot;
    selected.0 = hotbar.slots[active];
}

fn sync_hotbar_ui(
    hotbar: Res<Hotbar>,
    icons: Res<BlockIcons>,
    mut slot_query: Query<(&HotbarSlotUi, &mut BorderColor, &mut BackgroundColor)>,
    mut icon_query: Query<(&HotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    if !hotbar.is_changed() {
        return;
    }

    for (slot, mut border_color, mut bg_color) in &mut slot_query {
        if slot.index == hotbar.active_slot {
            *border_color = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            bg_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.12);
        } else {
            *border_color = BorderColor::all(Color::NONE);
            bg_color.0 = Color::NONE;
        }
    }

    for (icon, mut img_node, mut visibility) in &mut icon_query {
        match hotbar.slots[icon.index] {
            Some(voxel) => {
                img_node.image = icons.get(voxel);
                *visibility = Visibility::Visible;
            }
            None => {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn update_hotbar_tooltip(
    time: Res<Time>,
    hotbar: Res<Hotbar>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut tooltip_state: ResMut<HotbarTooltipState>,
    mut tooltip_query: Query<
        (
            &mut Text,
            &mut TextColor,
            Option<&mut TextShadow>,
            &mut Visibility,
        ),
        With<HotbarTooltipText>,
    >,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
) {
    let Ok((mut text, mut color, mut shadow, mut vis)) = tooltip_query.single_mut() else {
        return;
    };

    let in_menu = menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active);

    let active_slot = hotbar.active_slot;
    let current_voxel = hotbar.slots[active_slot];

    if !tooltip_state.initialized {
        tooltip_state.initialized = true;
        tooltip_state.last_slot = active_slot;
        tooltip_state.last_voxel = current_voxel;
        *vis = Visibility::Hidden;
        color.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
        return;
    }

    let digit_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ];

    let explicit_key_press = digit_keys
        .get(active_slot)
        .is_some_and(|k| keyboard.just_pressed(*k));
    let explicit_scroll = !keyboard.pressed(KeyCode::KeyZ) && mouse_scroll.delta.y.abs() > 0.05;
    let slot_changed = active_slot != tooltip_state.last_slot;
    let voxel_changed = current_voxel != tooltip_state.last_voxel;

    if (!in_menu && (explicit_key_press || explicit_scroll)) || slot_changed || voxel_changed {
        tooltip_state.last_slot = active_slot;
        tooltip_state.last_voxel = current_voxel;

        if let Some(voxel) = current_voxel {
            let label = voxel.label();
            tooltip_state.text = label.to_string();
            tooltip_state.timer = TOOLTIP_TOTAL_SECS;
            text.0 = tooltip_state.text.clone();
        } else {
            tooltip_state.timer = 0.0;
            text.0.clear();
        }
    }

    if tooltip_state.timer > 0.0 {
        tooltip_state.timer = (tooltip_state.timer - time.delta_secs()).max(0.0);
    }

    if in_menu || tooltip_state.timer <= 0.0 {
        *vis = Visibility::Hidden;
        color.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
        if let Some(ref mut s) = shadow {
            s.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
        }
    } else {
        let alpha = if tooltip_state.timer > TOOLTIP_FADE_SECS {
            1.0
        } else {
            (tooltip_state.timer / TOOLTIP_FADE_SECS).clamp(0.0, 1.0)
        };

        color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
        if let Some(ref mut s) = shadow {
            s.color = Color::srgba(0.0, 0.0, 0.0, 0.85 * alpha);
        }
        *vis = Visibility::Inherited;
    }
}
