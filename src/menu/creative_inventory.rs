use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, window::PrimaryWindow};

use crate::{
    gameplay::BlockIcons,
    player::hotbar::{HOTBAR_SLOT_COUNT, Hotbar},
    world::Voxel,
};

use super::{HeldInventoryItem, MenuState};

pub const INVENTORY_COLS: usize = 8;
pub const INVENTORY_VISIBLE_ROWS: usize = 4;
pub const INVENTORY_VISIBLE_SLOTS: usize = INVENTORY_COLS * INVENTORY_VISIBLE_ROWS; // 32 slots

pub const AVAILABLE_BLOCKS: [Voxel; 58] = [
    // Soils, Organics & Fine Sediment
    Voxel::Grass,
    Voxel::SnowyGrass,
    Voxel::Dirt,
    Voxel::PackedDirt,
    Voxel::Mud,
    Voxel::PackedMud,
    Voxel::Mulch,
    Voxel::Moss,
    Voxel::RedMoss,
    Voxel::Clay,
    Voxel::Gravel,
    Voxel::Sand,
    Voxel::RedSand,
    Voxel::Snow,
    Voxel::Ice,
    Voxel::PackedIce,
    // Stones, Rocks & Minerals
    Voxel::Stone,
    Voxel::Cobblestone,
    Voxel::MossyStone,
    Voxel::MossyCobblestone,
    Voxel::Slate,
    Voxel::Cobbleslate,
    Voxel::Blackstone,
    Voxel::Cobbleblackstone,
    Voxel::Flint,
    Voxel::Basalt,
    Voxel::Andesite,
    Voxel::Diorite,
    Voxel::Granite,
    Voxel::Tuff,
    Voxel::Sandstone,
    Voxel::RedSandstone,
    Voxel::Blueschist,
    Voxel::Calcite,
    Voxel::Dripstone,
    Voxel::Limestone,
    Voxel::Ochrestone,
    Voxel::Rhodonite,
    Voxel::Serpentinite,
    Voxel::Dreadstone,
    // Woods & Foliage
    Voxel::OakWoodLog,
    Voxel::OakWood,
    Voxel::OakLeaves,
    Voxel::BirchWoodLog,
    Voxel::BirchWood,
    Voxel::BirchLeaves,
    Voxel::PineWoodLog,
    Voxel::PineWood,
    Voxel::PineLeaves,
    Voxel::Cactus,
    // Fluids & Volcanics
    Voxel::Water,
    Voxel::Lava,
    Voxel::Magma,
    // Illumination
    Voxel::LightWarm,
    Voxel::LightCold,
    Voxel::LightRed,
    Voxel::LightGreen,
    Voxel::LightBlue,
];

pub fn total_inventory_rows() -> usize {
    AVAILABLE_BLOCKS.len().div_ceil(INVENTORY_COLS)
}

pub fn max_scroll_row() -> usize {
    total_inventory_rows().saturating_sub(INVENTORY_VISIBLE_ROWS)
}

#[derive(Resource, Default)]
pub struct InventoryScrollState {
    pub scroll_row: usize,
    pub is_dragging_thumb: bool,
}

#[derive(Component)]
struct InventoryMenuRoot;

#[derive(Component)]
struct InventoryCard;

#[derive(Component)]
struct InventoryPaletteSlot {
    slot_index: usize,
    voxel: Option<Voxel>,
}

#[derive(Component)]
struct InventoryPaletteSlotIcon {
    slot_index: usize,
}

#[derive(Component)]
struct InventoryScrollTrack;

#[derive(Component)]
struct InventoryScrollThumb;

#[derive(Component)]
struct InventoryHotbarSlot {
    index: usize,
}

#[derive(Component)]
struct InventoryHotbarSlotIcon {
    index: usize,
}

pub struct InventoryMenuPlugin;

impl Plugin for InventoryMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryScrollState>()
            .add_systems(OnEnter(MenuState::Inventory), spawn_inventory_menu)
            .add_systems(OnExit(MenuState::Inventory), despawn_inventory_menu)
            .add_systems(
                Update,
                (handle_inventory_interaction, sync_inventory_hotbar_icons)
                    .run_if(in_state(MenuState::Inventory)),
            );
    }
}

fn spawn_inventory_menu(
    mut commands: Commands,
    hotbar: Res<Hotbar>,
    icons: Res<BlockIcons>,
    scroll_state: Res<InventoryScrollState>,
) {
    let start_row = scroll_state.scroll_row;
    let total_rows = total_inventory_rows();
    let max_scroll = max_scroll_row();

    let track_height = 210.0;
    let thumb_height = (track_height * (INVENTORY_VISIBLE_ROWS as f32 / total_rows as f32))
        .clamp(40.0, track_height);
    let max_travel = (track_height - thumb_height).max(1.0);
    let thumb_top = if max_scroll > 0 {
        (start_row as f32 / max_scroll as f32) * max_travel
    } else {
        0.0
    };

    commands
        .spawn((
            InventoryMenuRoot,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    InventoryCard,
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(14.0),
                        width: px(480.0),
                        padding: UiRect::all(px(20.0)),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.11, 0.96)),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.42, 0.80)),
                ))
                .with_children(|panel| {
                    // Header Title
                    panel.spawn((
                        Text::new("CREATIVE INVENTORY"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.95, 0.98)),
                        Node {
                            margin: UiRect::bottom(px(4.0)),
                            ..default()
                        },
                    ));

                    // Palette + Scrollbar Row Container
                    panel
                        .spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(8.0),
                            ..default()
                        })
                        .with_children(|palette_row| {
                            // 8x4 Grid Container
                            palette_row
                                .spawn((
                                    Node {
                                        display: Display::Grid,
                                        grid_template_columns: RepeatedGridTrack::px(
                                            INVENTORY_COLS as u16,
                                            44.0,
                                        ),
                                        grid_template_rows: RepeatedGridTrack::px(
                                            INVENTORY_VISIBLE_ROWS as u16,
                                            44.0,
                                        ),
                                        row_gap: px(6.0),
                                        column_gap: px(6.0),
                                        padding: UiRect::all(px(8.0)),
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.85)),
                                    BorderColor::all(Color::srgba(0.22, 0.22, 0.26, 0.70)),
                                ))
                                .with_children(|grid| {
                                    for slot_idx in 0..INVENTORY_VISIBLE_SLOTS {
                                        let block_idx = start_row * INVENTORY_COLS + slot_idx;
                                        let voxel = AVAILABLE_BLOCKS.get(block_idx).copied();
                                        let icon_handle = voxel
                                            .map(|v| icons.get(v))
                                            .unwrap_or_else(|| icons.get(Voxel::Stone));
                                        let icon_vis = if voxel.is_some() {
                                            Visibility::Visible
                                        } else {
                                            Visibility::Hidden
                                        };

                                        grid.spawn((
                                            Button,
                                            InventoryPaletteSlot {
                                                slot_index: slot_idx,
                                                voxel,
                                            },
                                            Node {
                                                width: px(44.0),
                                                height: px(44.0),
                                                display: Display::Flex,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(px(1.5)),
                                                border_radius: BorderRadius::all(px(4.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.85)),
                                            BorderColor::all(Color::srgba(0.25, 0.25, 0.30, 0.60)),
                                        ))
                                        .with_children(
                                            |slot| {
                                                slot.spawn((
                                                    InventoryPaletteSlotIcon {
                                                        slot_index: slot_idx,
                                                    },
                                                    ImageNode {
                                                        image: icon_handle,
                                                        ..default()
                                                    },
                                                    Node {
                                                        width: px(32.0),
                                                        height: px(32.0),
                                                        ..default()
                                                    },
                                                    icon_vis,
                                                ));
                                            },
                                        );
                                    }
                                });

                            // Minecraft-style Scrollbar Track
                            palette_row
                                .spawn((
                                    Button,
                                    InventoryScrollTrack,
                                    Node {
                                        width: px(16.0),
                                        height: px(track_height),
                                        display: Display::Flex,
                                        position_type: PositionType::Relative,
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.04, 0.04, 0.06, 0.90)),
                                    BorderColor::all(Color::srgba(0.22, 0.22, 0.26, 0.70)),
                                ))
                                .with_children(|track| {
                                    track.spawn((
                                        Button,
                                        InventoryScrollThumb,
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(1.0),
                                            right: px(1.0),
                                            top: px(thumb_top),
                                            height: px(thumb_height),
                                            border: UiRect::all(px(1.5)),
                                            border_radius: BorderRadius::all(px(3.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.35, 0.36, 0.42, 0.95)),
                                        BorderColor::all(Color::srgba(0.55, 0.56, 0.65, 0.90)),
                                    ));
                                });
                        });

                    // Divider
                    panel.spawn(Node {
                        width: px(434.0),
                        height: px(1.5),
                        margin: UiRect::axes(px(0.0), px(6.0)),
                        border: UiRect::all(px(1.0)),
                        ..default()
                    });

                    // 8-slot Hotbar mirror row
                    panel
                        .spawn((
                            Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                column_gap: px(6.0),
                                padding: UiRect::all(px(6.0)),
                                border: UiRect::all(px(1.5)),
                                border_radius: BorderRadius::all(px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.85)),
                            BorderColor::all(Color::srgba(0.22, 0.22, 0.26, 0.70)),
                        ))
                        .with_children(|tray| {
                            for slot_idx in 0..HOTBAR_SLOT_COUNT {
                                let initial_voxel = hotbar.slots[slot_idx];
                                let icon_handle = initial_voxel
                                    .map(|v| icons.get(v))
                                    .unwrap_or_else(|| icons.get(Voxel::Stone));

                                let icon_vis = if initial_voxel.is_some() {
                                    Visibility::Visible
                                } else {
                                    Visibility::Hidden
                                };

                                tray.spawn((
                                    Button,
                                    InventoryHotbarSlot { index: slot_idx },
                                    Node {
                                        width: px(44.0),
                                        height: px(44.0),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        position_type: PositionType::Relative,
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.85)),
                                    BorderColor::all(Color::srgba(0.25, 0.25, 0.30, 0.60)),
                                ))
                                .with_children(|slot| {
                                    // Number badge 1..8
                                    slot.spawn((
                                        Text::new(format!("{}", slot_idx + 1)),
                                        TextFont {
                                            font_size: FontSize::Px(10.0),
                                            ..default()
                                        },
                                        TextColor(Color::srgba(0.85, 0.85, 0.85, 0.60)),
                                        Node {
                                            position_type: PositionType::Absolute,
                                            top: px(2.0),
                                            left: px(3.0),
                                            ..default()
                                        },
                                    ));

                                    // Item icon
                                    slot.spawn((
                                        InventoryHotbarSlotIcon { index: slot_idx },
                                        ImageNode {
                                            image: icon_handle,
                                            ..default()
                                        },
                                        Node {
                                            width: px(30.0),
                                            height: px(30.0),
                                            ..default()
                                        },
                                        icon_vis,
                                    ));
                                });
                            }
                        });
                });
        });
}

fn despawn_inventory_menu(
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    mut held_item: ResMut<HeldInventoryItem>,
    mut scroll_state: ResMut<InventoryScrollState>,
) {
    held_item.voxel = None;
    scroll_state.scroll_row = 0;
    scroll_state.is_dragging_thumb = false;
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[derive(Default)]
struct InventoryDragState {
    last_shift_palette_slot: Option<usize>,
    hotbar_drag_start_slot: Option<usize>,
    is_hotbar_dragging: bool,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_inventory_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut held_item: ResMut<HeldInventoryItem>,
    mut hotbar: ResMut<Hotbar>,
    mut scroll_state: ResMut<InventoryScrollState>,
    icons: Res<BlockIcons>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    card_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryCard>>,
    track_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryScrollTrack>>,
    thumb_query: Query<&Interaction, With<InventoryScrollThumb>>,
    mut thumb_node_query: Query<
        (&mut Node, &mut BackgroundColor, &mut BorderColor),
        (
            With<InventoryScrollThumb>,
            Without<InventoryPaletteSlot>,
            Without<InventoryHotbarSlot>,
        ),
    >,
    mut palette_query: Query<
        (
            &Interaction,
            &mut InventoryPaletteSlot,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        (
            With<InventoryPaletteSlot>,
            Without<InventoryScrollThumb>,
            Without<InventoryHotbarSlot>,
        ),
    >,
    mut slot_icon_query: Query<(&InventoryPaletteSlotIcon, &mut ImageNode, &mut Visibility)>,
    mut hotbar_slot_query: Query<
        (
            &Interaction,
            &InventoryHotbarSlot,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        (
            With<InventoryHotbarSlot>,
            Without<InventoryPaletteSlot>,
            Without<InventoryScrollThumb>,
        ),
    >,
    mut drag_state: Local<InventoryDragState>,
) {
    let is_shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let left_just_pressed = mouse.just_pressed(MouseButton::Left);
    let right_just_pressed = mouse.just_pressed(MouseButton::Right);
    let middle_just_pressed = mouse.just_pressed(MouseButton::Middle);
    let left_pressed = mouse.pressed(MouseButton::Left);

    let max_row = max_scroll_row();
    let mut row_changed = false;

    // 1. Mouse wheel scrolling
    let scroll_y = mouse_scroll.delta.y;
    if scroll_y > 0.05 {
        if scroll_state.scroll_row > 0 {
            scroll_state.scroll_row -= 1;
            row_changed = true;
        }
    } else if scroll_y < -0.05 && scroll_state.scroll_row < max_row {
        scroll_state.scroll_row += 1;
        row_changed = true;
    }

    // 2. Scrollbar Dragging / Track Clicking
    let track_height = 210.0;
    let thumb_height = (track_height
        * (INVENTORY_VISIBLE_ROWS as f32 / total_inventory_rows() as f32))
        .clamp(40.0, track_height);
    let max_travel = (track_height - thumb_height).max(1.0);

    if let (Some(window), Some((track_tf, track_computed))) =
        (window_query.iter().next(), track_query.iter().next())
    {
        let track_center = track_tf.translation().truncate();
        let track_half = track_computed.size() * 0.5;
        let track_rect = Rect::from_corners(track_center - track_half, track_center + track_half);

        if left_just_pressed
            && let Some(cursor_pos) = window.cursor_position()
            && track_rect.contains(cursor_pos)
        {
            scroll_state.is_dragging_thumb = true;
        }

        if left_pressed
            && scroll_state.is_dragging_thumb
            && let Some(cursor_pos) = window.cursor_position()
        {
            let track_top_y = track_center.y - track_half.y;
            let rel_y = (cursor_pos.y - track_top_y - thumb_height * 0.5).clamp(0.0, max_travel);
            let progress = rel_y / max_travel;
            let target_row = (progress * max_row as f32).round() as usize;
            if target_row != scroll_state.scroll_row {
                scroll_state.scroll_row = target_row.min(max_row);
                row_changed = true;
            }
        }
    }

    if mouse.just_released(MouseButton::Left) {
        scroll_state.is_dragging_thumb = false;
    }

    // 3. Deselect held item if clicking outside the central inventory card (on the backdrop)
    if (left_just_pressed || right_just_pressed)
        && held_item.voxel.is_some()
        && let (Some(window), Some((transform, computed))) =
            (window_query.iter().next(), card_query.iter().next())
        && let Some(cursor_pos) = window.cursor_position()
    {
        let center = transform.translation().truncate();
        let half = computed.size() * 0.5;
        let card_rect = Rect::from_corners(center - half, center + half);
        if !card_rect.contains(cursor_pos) {
            held_item.voxel = None;
        }
    }

    // 4. Right-click deselect if not hovering over any hotbar slot
    let any_hotbar_hovered = hotbar_slot_query
        .iter()
        .any(|(i, _, _, _)| *i == Interaction::Hovered || *i == Interaction::Pressed);

    if right_just_pressed && !any_hotbar_hovered {
        held_item.voxel = None;
    }

    // Check digit keys 1..8 for quick assignment / hotbar swap
    let digit_pressed = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some(3)
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        Some(4)
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        Some(5)
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        Some(6)
    } else if keyboard.just_pressed(KeyCode::Digit8) {
        Some(7)
    } else {
        None
    };

    // 5. Update slots if scroll row changed
    if row_changed {
        let cur_row = scroll_state.scroll_row;
        for (_, mut slot, _, _) in &mut palette_query {
            let block_idx = cur_row * INVENTORY_COLS + slot.slot_index;
            slot.voxel = AVAILABLE_BLOCKS.get(block_idx).copied();
        }

        for (icon, mut img, mut vis) in &mut slot_icon_query {
            let block_idx = cur_row * INVENTORY_COLS + icon.slot_index;
            if let Some(&voxel) = AVAILABLE_BLOCKS.get(block_idx) {
                *vis = Visibility::Visible;
                img.image = icons.get(voxel);
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }

    // Update thumb position and visual feedback
    for (mut thumb_node, mut thumb_bg, mut thumb_border) in &mut thumb_node_query {
        let progress = if max_row > 0 {
            scroll_state.scroll_row as f32 / max_row as f32
        } else {
            0.0
        };
        thumb_node.top = px(progress * max_travel);

        let is_thumb_hovered = thumb_query.iter().any(|i| *i == Interaction::Hovered);
        if scroll_state.is_dragging_thumb {
            *thumb_bg = BackgroundColor(Color::srgb(1.0, 0.85, 0.30));
            *thumb_border = BorderColor::all(Color::srgb(1.0, 1.0, 0.60));
        } else if is_thumb_hovered {
            *thumb_bg = BackgroundColor(Color::srgba(0.50, 0.52, 0.60, 1.0));
            *thumb_border = BorderColor::all(Color::srgba(0.70, 0.72, 0.80, 1.0));
        } else {
            *thumb_bg = BackgroundColor(Color::srgba(0.35, 0.36, 0.42, 0.95));
            *thumb_border = BorderColor::all(Color::srgba(0.55, 0.56, 0.65, 0.90));
        }
    }

    // 6. Process Palette Slots
    for (interaction, slot, mut border, mut bg) in &mut palette_query {
        let is_hovered = *interaction == Interaction::Hovered;
        let is_pressed = *interaction == Interaction::Pressed;

        if is_hovered || is_pressed {
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            *bg = BackgroundColor(Color::srgba(0.20, 0.20, 0.25, 0.95));

            if let Some(v) = slot.voxel {
                // Quick assign via keys 1..8
                if let Some(target_slot) = digit_pressed {
                    hotbar.slots[target_slot] = Some(v);
                }

                // Mouse Tweaks: Shift + Click / Shift + Drag quick-transfer into hotbar
                if is_shift {
                    if (left_just_pressed || left_pressed)
                        && drag_state.last_shift_palette_slot != Some(slot.slot_index)
                    {
                        drag_state.last_shift_palette_slot = Some(slot.slot_index);
                        if let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none()) {
                            hotbar.slots[empty_idx] = Some(v);
                        } else {
                            let active_slot = hotbar.active_slot;
                            hotbar.slots[active_slot] = Some(v);
                        }
                    }
                } else {
                    // Middle click picks up block directly
                    if middle_just_pressed {
                        held_item.voxel = Some(v);
                    }

                    // Left-click interaction
                    if left_just_pressed && is_pressed {
                        if held_item.voxel == Some(v) {
                            // Clicking same block again deselects it / returns to palette
                            held_item.voxel = None;
                        } else {
                            // Pick up onto cursor
                            held_item.voxel = Some(v);
                        }
                    }
                }
            }
        } else {
            *border = BorderColor::all(Color::srgba(0.25, 0.25, 0.30, 0.60));
            *bg = BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.85));
        }
    }

    // 7. Process Hotbar Slots
    for (interaction, hotbar_slot, mut border, mut bg) in &mut hotbar_slot_query {
        let is_hovered = *interaction == Interaction::Hovered;
        let is_pressed = *interaction == Interaction::Pressed;
        let idx = hotbar_slot.index;

        if is_hovered || is_pressed {
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            *bg = BackgroundColor(Color::srgba(0.20, 0.20, 0.25, 0.95));

            // Q key clears hovered hotbar slot
            if keyboard.just_pressed(KeyCode::KeyQ) {
                hotbar.slots[idx] = None;
            }

            // Middle click clears hotbar slot
            if middle_just_pressed {
                hotbar.slots[idx] = None;
            }

            // Quick swap / move via keys 1..8
            if let Some(target_slot) = digit_pressed
                && target_slot != idx
            {
                hotbar.slots.swap(idx, target_slot);
            }

            // Mouse Tweaks: Shift + Click / Shift + Drag clears hovered hotbar slots
            if is_shift {
                if left_just_pressed || (left_pressed && is_hovered) {
                    hotbar.slots[idx] = None;
                }
            } else {
                // Right click on hotbar slot: place/stamp held item into slot
                if right_just_pressed && let Some(held) = held_item.voxel {
                    hotbar.slots[idx] = Some(held);
                }

                // Left click or Left-drag (Mouse Tweaks LMB Tweak)
                if let Some(held) = held_item.voxel {
                    if left_pressed && (is_hovered || is_pressed) {
                        if left_just_pressed {
                            // Clicked on this slot with held item
                            drag_state.hotbar_drag_start_slot = Some(idx);
                            drag_state.is_hotbar_dragging = false;

                            if let Some(existing) = hotbar.slots[idx] {
                                if existing != held {
                                    // Swap items
                                    hotbar.slots[idx] = Some(held);
                                    held_item.voxel = Some(existing);
                                    drag_state.hotbar_drag_start_slot = None;
                                } else {
                                    // Same item: place into slot and clear held item
                                    held_item.voxel = None;
                                }
                            } else {
                                // Empty slot: place item into slot
                                hotbar.slots[idx] = Some(held);
                            }
                        } else if drag_state.hotbar_drag_start_slot.is_some()
                            && drag_state.hotbar_drag_start_slot != Some(idx)
                        {
                            // Dragged over another slot with LMB held down
                            drag_state.is_hotbar_dragging = true;
                            hotbar.slots[idx] = Some(held);
                        }
                    }
                } else {
                    // Not holding item: Left click picks up item from hotbar slot
                    if left_just_pressed && is_pressed && hotbar.slots[idx].is_some() {
                        held_item.voxel = hotbar.slots[idx];
                        hotbar.slots[idx] = None;
                    }
                }
            }
        } else {
            *border = BorderColor::all(Color::srgba(0.25, 0.25, 0.30, 0.60));
            *bg = BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.85));
        }
    }

    // 8. Mouse release cleanup
    if mouse.just_released(MouseButton::Left) {
        if drag_state.hotbar_drag_start_slot.is_some() {
            // Placing into hotbar slot clears held item from cursor
            held_item.voxel = None;
        }
        drag_state.last_shift_palette_slot = None;
        drag_state.is_hotbar_dragging = false;
        drag_state.hotbar_drag_start_slot = None;
    }
}

fn sync_inventory_hotbar_icons(
    hotbar: Res<Hotbar>,
    icons: Res<BlockIcons>,
    mut icons_query: Query<(&InventoryHotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    if !hotbar.is_changed() {
        return;
    }

    for (icon, mut img, mut vis) in &mut icons_query {
        let slot_voxel = hotbar.slots[icon.index];
        if let Some(voxel) = slot_voxel {
            *vis = Visibility::Visible;
            img.image = icons.get(voxel);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_dimensions_and_blocks() {
        assert_eq!(
            INVENTORY_COLS * INVENTORY_VISIBLE_ROWS,
            INVENTORY_VISIBLE_SLOTS
        );
        assert_eq!(INVENTORY_VISIBLE_ROWS, 4);
        assert_eq!(AVAILABLE_BLOCKS.len(), 58);
        assert_eq!(total_inventory_rows(), 8);
        assert_eq!(max_scroll_row(), 4);
    }

    #[test]
    fn test_inventory_scroll_bounds() {
        let max_row = max_scroll_row();
        assert_eq!(max_row, 4);

        // At scroll_row 0: items 0..32
        let first_block = AVAILABLE_BLOCKS[0];
        assert_eq!(first_block, Voxel::Grass);

        // At scroll_row 4 (bottom): items 32..57
        let last_row_first_block_idx = max_row * INVENTORY_COLS;
        assert_eq!(last_row_first_block_idx, 32);
        assert!(last_row_first_block_idx < AVAILABLE_BLOCKS.len());

        let last_block_row = (AVAILABLE_BLOCKS.len() - 1) / INVENTORY_COLS;
        assert_eq!(last_block_row, 7);
        assert!(last_block_row < total_inventory_rows());
    }

    #[test]
    fn test_shift_click_palette_transfer_logic() {
        let mut hotbar = Hotbar {
            slots: [None; HOTBAR_SLOT_COUNT],
            active_slot: 0,
        };

        // First empty slot should be 0
        let target_1 = hotbar.slots.iter().position(|s| s.is_none()).unwrap();
        assert_eq!(target_1, 0);
        hotbar.slots[target_1] = Some(Voxel::Grass);

        // Next empty slot should be 1
        let target_2 = hotbar.slots.iter().position(|s| s.is_none()).unwrap();
        assert_eq!(target_2, 1);
        hotbar.slots[target_2] = Some(Voxel::Stone);

        // Fill remaining slots
        for i in 2..HOTBAR_SLOT_COUNT {
            hotbar.slots[i] = Some(Voxel::Dirt);
        }

        // When all slots are full, it falls back to active_slot
        hotbar.active_slot = 3;
        let empty_idx = hotbar.slots.iter().position(|s| s.is_none());
        assert!(empty_idx.is_none());
        let fallback_idx = hotbar.active_slot;
        hotbar.slots[fallback_idx] = Some(Voxel::Sand);
        assert_eq!(hotbar.slots[3], Some(Voxel::Sand));
    }

    #[test]
    fn test_hotbar_swap_logic() {
        let mut hotbar = Hotbar {
            slots: [None; HOTBAR_SLOT_COUNT],
            active_slot: 0,
        };
        hotbar.slots[0] = Some(Voxel::Grass);
        hotbar.slots[1] = Some(Voxel::Stone);

        hotbar.slots.swap(0, 1);
        assert_eq!(hotbar.slots[0], Some(Voxel::Stone));
        assert_eq!(hotbar.slots[1], Some(Voxel::Grass));
    }

    #[test]
    fn test_card_bounds_detection() {
        let center = Vec2::new(640.0, 360.0);
        let size = Vec2::new(480.0, 300.0);
        let half = size * 0.5;
        let card_rect = Rect::from_corners(center - half, center + half);

        // Center is inside
        assert!(card_rect.contains(center));
        // Point just inside corner is inside
        assert!(card_rect.contains(Vec2::new(640.0 + half.x - 5.0, 360.0)));
        // Point outside on the backdrop is outside (deselects held item)
        assert!(!card_rect.contains(Vec2::new(100.0, 100.0)));
        assert!(!card_rect.contains(Vec2::new(1200.0, 700.0)));
    }
}
