use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, ui::widget::TextShadow};

use crate::core::{AppFont, text_shadow_default};
use crate::gameplay::{BlockIcons, SelectedVoxel};
use crate::player::InspectorInteraction;
use crate::player::inventory::PlayerInventory;
use crate::world::Voxel;

pub const HOTBAR_SLOT_COUNT: usize = 9;

pub const HOTBAR_SLOT_X_COORDS: [f32; HOTBAR_SLOT_COUNT] = [
    65.0, 87.0, 109.0, 131.0, 153.0, 175.0, 197.0, 219.0, 241.0,
];
pub const HOTBAR_SLOT_Y: f32 = 14.0;
pub const HOTBAR_SCALE: f32 = 3.0;

#[derive(Resource, Debug, Clone)]
pub struct Hotbar {
    pub slots: [Option<Voxel>; HOTBAR_SLOT_COUNT],
    pub active_slot: usize,
}

impl Default for Hotbar {
    fn default() -> Self {
        Self {
            slots: [
                Some(Voxel::LoamGrass),
                Some(Voxel::LoamDirt),
                Some(Voxel::Dolomite),
                Some(Voxel::SandCommon),
                Some(Voxel::LiquidWater),
                Some(Voxel::EmitLightWarm),
                Some(Voxel::OakLog),
                Some(Voxel::OakPlanks),
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
    #[allow(dead_code)]
    pub index: usize,
}

#[derive(Component)]
pub struct HotbarSlotIcon {
    pub index: usize,
}

#[derive(Component)]
pub struct HotbarOffhandIcon;

#[derive(Component)]
pub struct HotbarUtilityIcon;

#[derive(Component)]
pub struct HotbarSelectionBox;

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
    let selection_texture = asset_server.load("textures/interfaces/containers/hotbar_selection.png");
    let shield_overlay = asset_server.load("textures/interfaces/containers/slots/shield.png");
    let quiver_overlay = asset_server.load("textures/interfaces/containers/slots/quiver.png");

    let tray_width = 320.0 * HOTBAR_SCALE;
    let tray_height = 44.0 * HOTBAR_SCALE;
    let slot_size = 16.0 * HOTBAR_SCALE;
    let icon_size = 16.0 * HOTBAR_SCALE;
    let special_slot_size = 14.0 * HOTBAR_SCALE;
    let selection_size = 20.0 * HOTBAR_SCALE;

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

            // Scaled hotbar texture (320x44 px at 3x = 960x132 px)
            // NO mouse hover / NO mouse clicks in 3D gameplay
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
                    // Static Visual Progress Bars (Placeholders per specification)
                    // Field 1: XP Progress Bar at (59, 4), 204x4 px
                    tray.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px(59.0 * HOTBAR_SCALE),
                        top: px(4.0 * HOTBAR_SCALE),
                        width: px(204.0 * HOTBAR_SCALE),
                        height: px(4.0 * HOTBAR_SCALE),
                        ..default()
                    })
                    .with_children(|xp_bar| {
                        // Static 45% level fill in ambient green
                        xp_bar.spawn((
                            Node {
                                width: Val::Percent(45.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.22, 0.78, 0.32, 0.75)),
                        ));
                    });

                    // Field 17: Health Bar (HP) at (68, 36), 82x4 px
                    tray.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px(68.0 * HOTBAR_SCALE),
                        top: px(36.0 * HOTBAR_SCALE),
                        width: px(82.0 * HOTBAR_SCALE),
                        height: px(4.0 * HOTBAR_SCALE),
                        ..default()
                    })
                    .with_children(|hp_bar| {
                        // Static full health fill in crimson
                        hp_bar.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.85, 0.18, 0.18, 0.90)),
                        ));
                    });

                    // Field 18: Mana / Stamina Bar at (172, 36), 82x4 px
                    tray.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px(172.0 * HOTBAR_SCALE),
                        top: px(36.0 * HOTBAR_SCALE),
                        width: px(82.0 * HOTBAR_SCALE),
                        height: px(4.0 * HOTBAR_SCALE),
                        ..default()
                    })
                    .with_children(|mana_bar| {
                        // Static full mana fill in azure
                        mana_bar.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.20, 0.45, 0.88, 0.90)),
                        ));
                    });

                    // Non-clickable HUD wing indicator badges (Fields 2, 3, 15, 16)
                    let badge_font = TextFont {
                        font_size: FontSize::Px(10.0 * (HOTBAR_SCALE / 3.0)),
                        ..default()
                    };

                    let badges = [
                        (18.0, 25.0, "M"), // Field 2: Map
                        (30.0, 24.0, "J"), // Field 3: Journal
                        (284.0, 24.0, "C"), // Field 15: Craft
                        (296.0, 25.0, "T"), // Field 16: Chat
                    ];

                    for (bx, by, label) in badges {
                        let mut bf = badge_font.clone();
                        if let Some(ref font) = font_handle {
                            bf.font = font.clone();
                        }
                        tray.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(bx * HOTBAR_SCALE),
                                top: px(by * HOTBAR_SCALE),
                                width: px(8.0 * HOTBAR_SCALE),
                                height: px(8.0 * HOTBAR_SCALE),
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(label),
                                bf,
                                TextColor(Color::srgba(0.68, 0.72, 0.78, 0.85)),
                                text_shadow_default(),
                            ));
                        });
                    }

                    // Field 4: Off-hand Slot display at (45, 17), 14x14 px (non-clickable visual readout)
                    tray.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px(45.0 * HOTBAR_SCALE),
                        top: px(17.0 * HOTBAR_SCALE),
                        width: px(special_slot_size),
                        height: px(special_slot_size),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|slot| {
                        slot.spawn((
                            HotbarOffhandIcon,
                            ImageNode {
                                image: shield_overlay,
                                ..default()
                            },
                            Node {
                                width: px(special_slot_size),
                                height: px(special_slot_size),
                                ..default()
                            },
                        ));
                    });

                    // Field 14: Quick Utility Slot display at (263, 17), 14x14 px (non-clickable visual readout)
                    tray.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px(263.0 * HOTBAR_SCALE),
                        top: px(17.0 * HOTBAR_SCALE),
                        width: px(special_slot_size),
                        height: px(special_slot_size),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|slot| {
                        slot.spawn((
                            HotbarUtilityIcon,
                            ImageNode {
                                image: quiver_overlay,
                                ..default()
                            },
                            Node {
                                width: px(special_slot_size),
                                height: px(special_slot_size),
                                ..default()
                            },
                        ));
                    });

                    // Fields 5 to 13: 9 Primary Hotbar Item Slots (16x16 px)
                    for index in 0..HOTBAR_SLOT_COUNT {
                        let initial_voxel = initial_hotbar.slots[index];
                        let slot_x = HOTBAR_SLOT_X_COORDS[index];

                        let icon_handle = initial_voxel
                            .map(|v| icons.get(v))
                            .unwrap_or_else(|| icons.get(Voxel::Dolomite));

                        let icon_visibility = if initial_voxel.is_some() {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        };

                        tray.spawn((
                            HotbarSlotUi { index },
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(slot_x * HOTBAR_SCALE),
                                top: px(HOTBAR_SLOT_Y * HOTBAR_SCALE),
                                width: px(slot_size),
                                height: px(slot_size),
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|slot| {
                            // Centered 2D item icon
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

                            // Slot index number (1 through 9, rendered in front with ZIndex)
                            let mut num_font = TextFont {
                                font_size: FontSize::Px(12.0 * (HOTBAR_SCALE / 3.0)),
                                ..default()
                            };
                            if let Some(ref font) = font_handle {
                                num_font.font = font.clone();
                            }

                            slot.spawn((
                                Text::new(format!("{}", index + 1)),
                                num_font,
                                TextColor(Color::srgba(0.85, 0.88, 0.92, 0.85)),
                                text_shadow_default(),
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: px(1.0 * HOTBAR_SCALE),
                                    left: px(2.0 * HOTBAR_SCALE),
                                    ..default()
                                },
                                ZIndex(10),
                            ));
                        });
                    }

                    // Active Selection Box (20x20 px bevel frame centered over active slot)
                    let active_slot_x = HOTBAR_SLOT_X_COORDS[initial_hotbar.active_slot];
                    tray.spawn((
                        HotbarSelectionBox,
                        ImageNode {
                            image: selection_texture,
                            ..default()
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            left: px((active_slot_x - 2.0) * HOTBAR_SCALE),
                            top: px((HOTBAR_SLOT_Y - 2.0) * HOTBAR_SCALE),
                            width: px(selection_size),
                            height: px(selection_size),
                            ..default()
                        },
                        ZIndex(20),
                    ));
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
        KeyCode::Digit9,
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
    player_inventory: Option<Res<PlayerInventory>>,
    icons: Res<BlockIcons>,
    asset_server: Res<AssetServer>,
    mut selection_query: Query<&mut Node, (With<HotbarSelectionBox>, Without<HotbarSlotUi>)>,
    mut icon_query: Query<(&HotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
    mut offhand_query: Query<&mut ImageNode, (With<HotbarOffhandIcon>, Without<HotbarSlotIcon>)>,
    mut utility_query: Query<
        &mut ImageNode,
        (
            With<HotbarUtilityIcon>,
            Without<HotbarSlotIcon>,
            Without<HotbarOffhandIcon>,
        ),
    >,
) {
    // Update active selection box position
    if let Ok(mut sel_node) = selection_query.single_mut() {
        let slot_x = HOTBAR_SLOT_X_COORDS[hotbar.active_slot % HOTBAR_SLOT_COUNT];
        sel_node.left = px((slot_x - 2.0) * HOTBAR_SCALE);
    }

    if hotbar.is_changed() {
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

    // Sync offhand & utility slots from player inventory if changed
    if let Some(ref inv) = player_inventory {
        if inv.is_changed() {
            if let Ok(mut offhand_img) = offhand_query.single_mut() {
                if let Some(voxel) = inv.offhand_slot {
                    offhand_img.image = icons.get(voxel);
                } else {
                    offhand_img.image = asset_server.load("textures/interfaces/containers/slots/shield.png");
                }
            }
            if let Ok(mut utility_img) = utility_query.single_mut() {
                if let Some(voxel) = inv.utility_slot {
                    utility_img.image = icons.get(voxel);
                } else {
                    utility_img.image = asset_server.load("textures/interfaces/containers/slots/quiver.png");
                }
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
        KeyCode::Digit9,
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
