use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, ui::widget::TextShadow};

use crate::core::{AppFont, text_shadow_default};
use crate::gameplay::{BlockIcons, SelectedVoxel};
use crate::player::InspectorInteraction;
use crate::player::inventory::PlayerInventory;
use crate::world::Voxel;

pub const HOTBAR_SLOT_COUNT: usize = 9;

pub const HOTBAR_SLOT_X_COORDS: [f32; HOTBAR_SLOT_COUNT] = [
    156.0, 175.0, 194.0, 213.0, 232.0, 251.0, 270.0, 289.0, 308.0,
];
pub const HOTBAR_SLOT_Y: f32 = 228.0;
pub const HOTBAR_SCALE: f32 = 4.0;
pub const HOTBAR_FRAME_WIDTH: f32 = 480.0 * HOTBAR_SCALE; // 1920.0 px
pub const HOTBAR_FRAME_HEIGHT: f32 = 270.0 * HOTBAR_SCALE; // 1080.0 px

#[derive(Resource, Debug, Clone)]
pub struct Hotbar {
    pub slots: [Option<Voxel>; HOTBAR_SLOT_COUNT],
    pub counts: [u16; HOTBAR_SLOT_COUNT],
    pub active_slot: usize,
}

impl Default for Hotbar {
    fn default() -> Self {
        let mut slots = [None; HOTBAR_SLOT_COUNT];
        let mut counts = [0u16; HOTBAR_SLOT_COUNT];
        slots[0] = Some(Voxel::ToolFlintPickaxe);
        counts[0] = 1;
        slots[1] = Some(Voxel::ToolFlintAxe);
        counts[1] = 1;
        slots[2] = Some(Voxel::ToolFlintShovel);
        counts[2] = 1;
        slots[HOTBAR_SLOT_COUNT - 1] = Some(Voxel::EmitTorchFire);
        counts[HOTBAR_SLOT_COUNT - 1] = 8;
        Self {
            slots,
            counts,
            active_slot: 0,
        }
    }
}

impl Hotbar {
    pub fn try_stack_existing(&mut self, voxel: Voxel, mut amount: u16) -> u16 {
        let max_stack = voxel.max_stack_size();
        for i in 0..HOTBAR_SLOT_COUNT {
            if amount == 0 {
                break;
            }
            if self.slots[i] == Some(voxel) && self.counts[i] < max_stack {
                let space = max_stack - self.counts[i];
                let add = amount.min(space);
                self.counts[i] += add;
                amount -= add;
            }
        }
        amount
    }

    pub fn add_to_empty(&mut self, voxel: Voxel, mut amount: u16) -> u16 {
        let max_stack = voxel.max_stack_size();
        for i in 0..HOTBAR_SLOT_COUNT {
            if amount == 0 {
                break;
            }
            if self.slots[i].is_none() {
                let add = amount.min(max_stack);
                self.slots[i] = Some(voxel);
                self.counts[i] = add;
                amount -= add;
            }
        }
        amount
    }

    pub fn consume_active(&mut self, amount: u16) {
        let active = self.active_slot % HOTBAR_SLOT_COUNT;
        if self.slots[active].is_some() {
            if self.counts[active] <= amount {
                self.slots[active] = None;
                self.counts[active] = 0;
            } else {
                self.counts[active] -= amount;
            }
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
pub struct HotbarSlotCount {
    pub index: usize,
}

#[derive(Component)]
pub struct HotbarOffhandIcon;

#[derive(Component)]
pub struct HotbarOffhandCount;

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

#[derive(Component)]
pub struct HotbarRoot;

#[derive(Component)]
pub struct HotbarTray;

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
                (
                    handle_hotbar_input,
                    sync_hotbar_ui,
                    sync_hotbar_scale,
                    update_hotbar_tooltip,
                    manage_hotbar_visibility,
                )
                    .chain(),
            );
    }
}

fn setup_hotbar_ui(mut commands: Commands, icons: Res<BlockIcons>, app_font: Option<Res<AppFont>>) {
    use crate::menu::theme::palette;

    let initial_hotbar = Hotbar::default();
    let font_handle = app_font.as_ref().map(|f| f.source());

    let slot_size = 16.0 * HOTBAR_SCALE;
    let icon_size = 16.0 * HOTBAR_SCALE;
    let selection_size = 20.0 * HOTBAR_SCALE;

    // Fullscreen 16:9 gameplay hotbar container
    commands
        .spawn((
            HotbarRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(150),
        ))
        .with_children(|parent| {
            // Scaled 16:9 hotbar coordinate frame (480x270 at 4x = 1920x1080 px)
            // NO mouse hover / NO mouse clicks in 3D gameplay
            parent
                .spawn((
                    HotbarTray,
                    UiTransform::IDENTITY,
                    Node {
                        width: px(HOTBAR_FRAME_WIDTH),
                        height: px(HOTBAR_FRAME_HEIGHT),
                        ..default()
                    },
                ))
                .with_children(|tray| {
                    // Pure-code Dark Iron hotbar chassis framing the 9 primary slots (clean, no XP bar)
                    tray.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(151.0 * HOTBAR_SCALE),
                            top: px(224.0 * HOTBAR_SCALE),
                            width: px(178.0 * HOTBAR_SCALE),
                            height: px(24.0 * HOTBAR_SCALE),
                            border: UiRect::all(px(2.0)),
                            border_radius: BorderRadius::all(px(5.0)),
                            ..default()
                        },
                        BackgroundColor(palette::BAR_BG),
                        BorderColor::all(palette::BAR_BORDER),
                    ));

                    // Secondary right-side chassis for the 2 auxiliary slots (Off-hand + Quick Utility), separated from the 9-slot bar
                    tray.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(339.0 * HOTBAR_SCALE),
                            top: px(224.0 * HOTBAR_SCALE),
                            width: px(45.0 * HOTBAR_SCALE),
                            height: px(24.0 * HOTBAR_SCALE),
                            border: UiRect::all(px(2.0)),
                            border_radius: BorderRadius::all(px(5.0)),
                            ..default()
                        },
                        BackgroundColor(palette::BAR_BG),
                        BorderColor::all(palette::BAR_BORDER),
                    ));

                    // Held item name tooltip (centered text floating above hotbar)
                    let mut tooltip_font = TextFont {
                        font_size: FontSize::Px(20.0),
                        ..default()
                    };
                    if let Some(ref font) = font_handle {
                        tooltip_font.font = font.clone();
                    }

                    let mut num_font = TextFont {
                        font_size: FontSize::Px(3.0 * HOTBAR_SCALE),
                        ..default()
                    };
                    if let Some(ref font) = font_handle {
                        num_font.font = font.clone();
                    }

                    tray.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        right: px(0.0),
                        top: px(200.0 * HOTBAR_SCALE),
                        width: px(HOTBAR_FRAME_WIDTH),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|wrapper| {
                        wrapper.spawn((
                            HotbarTooltipText,
                            Text::new(""),
                            TextLayout::justify(Justify::Center),
                            tooltip_font,
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                            text_shadow_default(),
                            Visibility::Hidden,
                        ));
                    });

                    // Off-hand Slot display at (344, 228), 16x16 px (grouped on the right, separated from main 9)
                    tray.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(344.0 * HOTBAR_SCALE),
                            top: px(HOTBAR_SLOT_Y * HOTBAR_SCALE),
                            width: px(slot_size),
                            height: px(slot_size),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(px(1.5)),
                            border_radius: BorderRadius::all(px(3.0)),
                            ..default()
                        },
                        BackgroundColor(palette::SLOT_EMPTY_BG),
                        BorderColor::all(palette::SLOT_BORDER),
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            HotbarOffhandIcon,
                            ImageNode::default(),
                            Node {
                                width: px(slot_size),
                                height: px(slot_size),
                                ..default()
                            },
                            Visibility::Hidden,
                        ));
                        slot.spawn((
                            HotbarOffhandCount,
                            Text::new(""),
                            num_font.clone(),
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.96)),
                            text_shadow_default(),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: px(1.0 * HOTBAR_SCALE),
                                right: px(2.0 * HOTBAR_SCALE),
                                ..default()
                            },
                            ZIndex(11),
                        ));
                    });

                    // Quick Utility Slot display at (363, 228), 16x16 px (beside Off-hand on the right)
                    tray.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(363.0 * HOTBAR_SCALE),
                            top: px(HOTBAR_SLOT_Y * HOTBAR_SCALE),
                            width: px(slot_size),
                            height: px(slot_size),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(px(1.5)),
                            border_radius: BorderRadius::all(px(3.0)),
                            ..default()
                        },
                        BackgroundColor(palette::SLOT_EMPTY_BG),
                        BorderColor::all(palette::SLOT_BORDER),
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            HotbarUtilityIcon,
                            ImageNode::default(),
                            Node {
                                width: px(slot_size),
                                height: px(slot_size),
                                ..default()
                            },
                            Visibility::Hidden,
                        ));
                    });

                    // Fields 3 to 11: 9 Primary Hotbar Item Slots (16x16 px)
                    for (index, &slot_x) in HOTBAR_SLOT_X_COORDS
                        .iter()
                        .enumerate()
                        .take(HOTBAR_SLOT_COUNT)
                    {
                        let initial_voxel = initial_hotbar.slots[index];

                        let icon_handle = initial_voxel
                            .map(|v| icons.get(v))
                            .unwrap_or_else(|| icons.get(Voxel::Dolomite));

                        let icon_visibility = if initial_voxel.is_some() {
                            Visibility::Inherited
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
                                border: UiRect::all(px(1.5)),
                                border_radius: BorderRadius::all(px(3.0)),
                                ..default()
                            },
                            BackgroundColor(palette::SLOT_BG),
                            BorderColor::all(palette::SLOT_BORDER),
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

                            // Slot index number (1 through 9)
                            slot.spawn((
                                Text::new(format!("{}", index + 1)),
                                num_font.clone(),
                                TextColor(Color::srgba(0.85, 0.88, 0.92, 0.80)),
                                text_shadow_default(),
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: px(1.0 * HOTBAR_SCALE),
                                    left: px(2.0 * HOTBAR_SCALE),
                                    ..default()
                                },
                                ZIndex(10),
                            ));

                            // Stack count readout (bottom-right, visible when > 1)
                            let init_count = initial_hotbar.counts[index];
                            let count_str = if init_count > 1 {
                                format!("{}", init_count)
                            } else {
                                String::new()
                            };
                            slot.spawn((
                                HotbarSlotCount { index },
                                Text::new(count_str),
                                num_font.clone(),
                                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.96)),
                                text_shadow_default(),
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: px(1.0 * HOTBAR_SCALE),
                                    right: px(2.0 * HOTBAR_SCALE),
                                    ..default()
                                },
                                ZIndex(11),
                            ));
                        });
                    }

                    // Active Selection Box (20x20 px gold accent border-only frame centered over active slot)
                    let active_slot_x = HOTBAR_SLOT_X_COORDS[initial_hotbar.active_slot];
                    tray.spawn((
                        HotbarSelectionBox,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px((active_slot_x - 2.0) * HOTBAR_SCALE),
                            top: px((HOTBAR_SLOT_Y - 2.0) * HOTBAR_SCALE),
                            width: px(selection_size),
                            height: px(selection_size),
                            border: UiRect::all(px(3.0)),
                            border_radius: BorderRadius::all(px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        BorderColor::all(palette::SLOT_SELECTED_BORDER),
                        ZIndex(20),
                    ));
                });
        });
}

fn handle_hotbar_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    control_settings: Option<Res<crate::core::ControlSettings>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<crate::player::Player>)>,
    mut hotbar: ResMut<Hotbar>,
    mut player_inv: Option<ResMut<PlayerInventory>>,
    mut selected: ResMut<SelectedVoxel>,
) {
    if menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }

    let hotbar_actions = [
        crate::core::InputAction::Hotbar1,
        crate::core::InputAction::Hotbar2,
        crate::core::InputAction::Hotbar3,
        crate::core::InputAction::Hotbar4,
        crate::core::InputAction::Hotbar5,
        crate::core::InputAction::Hotbar6,
        crate::core::InputAction::Hotbar7,
        crate::core::InputAction::Hotbar8,
        crate::core::InputAction::Hotbar9,
    ];

    if let Some(ref c) = control_settings {
        for (idx, action) in hotbar_actions.iter().enumerate() {
            if c.is_action_just_pressed(*action, &keyboard, &mouse) {
                hotbar.active_slot = idx;
            }
        }
    } else {
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
    }

    // When holding Zoom, mouse wheel scroll adjusts zoom level instead of cycling hotbar
    let is_zooming = control_settings
        .as_ref()
        .map_or(keyboard.pressed(KeyCode::KeyZ), |c| {
            c.is_action_pressed(crate::core::InputAction::Zoom, &keyboard, &mouse)
        });

    if !is_zooming {
        let scroll = mouse_scroll.delta.y;
        if scroll > 0.05 {
            // Scroll up = previous slot
            hotbar.active_slot = (hotbar.active_slot + HOTBAR_SLOT_COUNT - 1) % HOTBAR_SLOT_COUNT;
        } else if scroll < -0.05 {
            // Scroll down = next slot
            hotbar.active_slot = (hotbar.active_slot + 1) % HOTBAR_SLOT_COUNT;
        }
    }

    // Drop item key clears the active hotbar slot and tosses a 3D item drop into the world
    let is_drop = control_settings
        .as_ref()
        .map_or(keyboard.just_pressed(KeyCode::KeyQ), |c| {
            c.is_action_just_pressed(crate::core::InputAction::DropItem, &keyboard, &mouse)
        });
    if is_drop {
        let active = hotbar.active_slot;
        if let Some(dropped_voxel) = hotbar.slots[active] {
            hotbar.consume_active(1);
            let spawn_pos = player_query
                .iter()
                .next()
                .map_or(Vec3::new(0.0, 20.0, 0.0), |t| {
                    t.translation + Vec3::Y * 1.38
                });
            let forward = camera_query
                .iter()
                .next()
                .map_or(Vec3::NEG_Z, |c| *c.forward());
            let throw_dir =
                Vec3::new(forward.x, forward.y.max(-0.15) + 0.22, forward.z).normalize_or_zero();
            commands.trigger(crate::gameplay::SpawnItemDropEvent {
                position: spawn_pos + throw_dir * 0.45,
                velocity: throw_dir * 5.8 + Vec3::Y * 1.4,
                voxel: dropped_voxel,
                pickup_delay: 1.35,
            });
        }
    }

    // Quick-swap active main-hand slot with off-hand slot (default E in gameplay, disabled in menus)
    let is_swap_hands = control_settings
        .as_ref()
        .map_or(keyboard.just_pressed(KeyCode::KeyE), |c| {
            c.is_action_just_pressed(crate::core::InputAction::SwapHands, &keyboard, &mouse)
        });
    if is_swap_hands && let Some(ref mut inv) = player_inv {
        let active = hotbar.active_slot;
        let hb = &mut *hotbar;
        let inv_ref = &mut **inv;
        std::mem::swap(&mut hb.slots[active], &mut inv_ref.offhand_slot);
        std::mem::swap(&mut hb.counts[active], &mut inv_ref.offhand_count);
    }

    let active = hotbar.active_slot;
    selected.0 = hotbar.slots[active];
}

fn sync_hotbar_ui(
    hotbar: Res<Hotbar>,
    player_inventory: Option<Res<PlayerInventory>>,
    icons: Res<BlockIcons>,
    _asset_server: Res<AssetServer>,
    mut selection_query: Query<&mut Node, (With<HotbarSelectionBox>, Without<HotbarSlotUi>)>,
    mut slot_bg_query: Query<(&HotbarSlotUi, &mut BackgroundColor)>,
    mut icon_query: Query<(&HotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
    mut count_query: Query<(&HotbarSlotCount, &mut Text), Without<HotbarOffhandCount>>,
    mut offhand_count_query: Query<&mut Text, (With<HotbarOffhandCount>, Without<HotbarSlotCount>)>,
    mut offhand_query: Query<
        (&mut ImageNode, &mut Visibility),
        (With<HotbarOffhandIcon>, Without<HotbarSlotIcon>),
    >,
    mut utility_query: Query<
        (&mut ImageNode, &mut Visibility),
        (
            With<HotbarUtilityIcon>,
            Without<HotbarSlotIcon>,
            Without<HotbarOffhandIcon>,
        ),
    >,
) {
    use crate::menu::theme::palette;

    // Update active selection box position
    let active_idx = hotbar.active_slot % HOTBAR_SLOT_COUNT;
    if let Ok(mut sel_node) = selection_query.single_mut() {
        let slot_x = HOTBAR_SLOT_X_COORDS[active_idx];
        sel_node.left = px((slot_x - 2.0) * HOTBAR_SCALE);
        sel_node.top = px((HOTBAR_SLOT_Y - 2.0) * HOTBAR_SCALE);
    }

    for (slot_ui, mut bg) in &mut slot_bg_query {
        let target = if slot_ui.index == active_idx {
            Color::srgba(0.18, 0.16, 0.12, 0.92)
        } else {
            palette::SLOT_BG
        };
        if bg.0 != target {
            bg.0 = target;
        }
    }

    if hotbar.is_changed() {
        for (icon, mut img_node, mut visibility) in &mut icon_query {
            match hotbar.slots[icon.index] {
                Some(voxel) => {
                    img_node.image = icons.get(voxel);
                    *visibility = Visibility::Inherited;
                }
                None => {
                    *visibility = Visibility::Hidden;
                }
            }
        }
        for (cnt_ui, mut txt) in &mut count_query {
            let count = if hotbar.slots[cnt_ui.index].is_some() {
                hotbar.counts[cnt_ui.index]
            } else {
                0
            };
            let next = if count > 1 {
                format!("{}", count)
            } else {
                String::new()
            };
            if txt.0 != next {
                txt.0 = next;
            }
        }
    }

    // Sync offhand & utility slots from player inventory if changed
    if let Some(ref inv) = player_inventory
        && inv.is_changed()
    {
        if let Ok((mut offhand_img, mut offhand_vis)) = offhand_query.single_mut() {
            if let Some(voxel) = inv.offhand_slot {
                offhand_img.image = icons.get(voxel);
                *offhand_vis = Visibility::Inherited;
            } else {
                *offhand_vis = Visibility::Hidden;
            }
        }
        if let Ok(mut off_txt) = offhand_count_query.single_mut() {
            let cnt = if inv.offhand_slot.is_some() {
                inv.offhand_count
            } else {
                0
            };
            let next = if cnt > 1 {
                format!("{}", cnt)
            } else {
                String::new()
            };
            if off_txt.0 != next {
                off_txt.0 = next;
            }
        }
        if let Ok((mut utility_img, mut utility_vis)) = utility_query.single_mut() {
            if let Some(voxel) = inv.utility_slot {
                utility_img.image = icons.get(voxel);
                *utility_vis = Visibility::Inherited;
            } else {
                *utility_vis = Visibility::Hidden;
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
    loc: Option<Res<crate::core::Localization>>,
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
            let label = if let Some(ref l) = loc {
                l.t_block(voxel)
            } else {
                voxel.label()
            };
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

fn manage_hotbar_visibility(
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    mut root_query: Query<(&mut Visibility, &mut Node), With<HotbarRoot>>,
) {
    let Some(menu_state) = menu_state else {
        return;
    };

    let hide = *menu_state.get() != crate::menu::MenuState::None;
    let target_vis = if hide {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
    let target_display = if hide { Display::None } else { Display::Flex };

    for (mut vis, mut node) in &mut root_query {
        if *vis != target_vis {
            *vis = target_vis;
        }
        if node.display != target_display {
            node.display = target_display;
        }
    }
}

fn sync_hotbar_scale(
    game_settings: Option<Res<crate::menu::GameSettings>>,
    mut query: Query<&mut UiTransform, With<HotbarTray>>,
) {
    let Some(settings) = game_settings else {
        return;
    };
    if !settings.is_changed() {
        return;
    }
    let factor = (settings.gui_scale as f32 / 4.0).clamp(0.5, 1.0);
    let y_offset = 540.0 * (1.0 - factor);
    for mut transform in &mut query {
        transform.scale = Vec2::splat(factor);
        transform.translation = Val2 {
            x: Val::Px(0.0),
            y: Val::Px(y_offset),
        };
    }
}
