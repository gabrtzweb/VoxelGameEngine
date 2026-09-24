use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, window::PrimaryWindow};

use crate::{
    core::{text_shadow_default, AppFont},
    gameplay::BlockIcons,
    player::hotbar::{HOTBAR_SLOT_COUNT, Hotbar},
    player::inventory::PlayerInventory,
    world::Voxel,
};

use super::{HeldInventoryItem, MenuState};

pub const INVENTORY_COLS: usize = 8;
pub const INVENTORY_VISIBLE_ROWS: usize = 4;
pub const INVENTORY_VISIBLE_SLOTS: usize = INVENTORY_COLS * INVENTORY_VISIBLE_ROWS; // 32 slots

pub const AVAILABLE_BLOCKS: [Voxel; 62] = [
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
    Voxel::Terracotta,
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
    Voxel::RainwoodWoodLog,
    Voxel::RainwoodWood,
    Voxel::RainwoodLeaves,
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

/// The active inventory view mode.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InventoryTab {
    #[default]
    Creative,
    Player,
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
struct InventoryTabButton {
    tab: InventoryTab,
}

#[derive(Component)]
struct InventoryTabButtonText {
    tab: InventoryTab,
}

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
            .init_resource::<InventoryTab>()
            .add_systems(OnEnter(MenuState::Inventory), spawn_inventory_menu)
            .add_systems(OnExit(MenuState::Inventory), despawn_inventory_menu)
            .add_systems(
                Update,
                (
                    handle_inventory_tab_interaction,
                    handle_inventory_scroll,
                    handle_inventory_slot_interaction,
                    sync_inventory_hotbar_icons,
                )
                    .chain()
                    .run_if(in_state(MenuState::Inventory)),
            );
    }
}

fn spawn_inventory_menu(
    mut commands: Commands,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    tab_state: Res<InventoryTab>,
    scroll_state: Res<InventoryScrollState>,
    app_font: Option<Res<AppFont>>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source());
    let current_tab = *tab_state;
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

    let (track_display, track_vis) = match current_tab {
        InventoryTab::Creative => (Display::Flex, Visibility::Visible),
        InventoryTab::Player => (Display::None, Visibility::Hidden),
    };

    let mut tab1_font = TextFont {
        font_size: FontSize::Px(12.0),
        ..default()
    };
    let mut tab2_font = TextFont {
        font_size: FontSize::Px(12.0),
        ..default()
    };
    if let Some(ref font) = font_handle {
        tab1_font.font = font.clone();
        tab2_font.font = font.clone();
    }

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
                        row_gap: px(12.0),
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
                    // Top Tab Navigation Bar
                    panel
                        .spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: px(10.0),
                            margin: UiRect::bottom(px(4.0)),
                            ..default()
                        })
                        .with_children(|tab_row| {
                            // Tab 1: Creative Inventory
                            tab_row
                                .spawn((
                                    Button,
                                    InventoryTabButton {
                                        tab: InventoryTab::Creative,
                                    },
                                    Node {
                                        padding: UiRect::axes(px(14.0), px(6.0)),
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(6.0)),
                                        display: Display::Flex,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    if current_tab == InventoryTab::Creative {
                                        BackgroundColor(Color::srgba(0.24, 0.24, 0.32, 0.95))
                                    } else {
                                        BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.70))
                                    },
                                    if current_tab == InventoryTab::Creative {
                                        BorderColor::all(Color::srgb(1.0, 0.85, 0.30))
                                    } else {
                                        BorderColor::all(Color::srgba(0.28, 0.28, 0.35, 0.60))
                                    },
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        InventoryTabButtonText {
                                            tab: InventoryTab::Creative,
                                        },
                                        Text::new("CREATIVE INVENTORY"),
                                        tab1_font,
                                        TextColor(if current_tab == InventoryTab::Creative {
                                            Color::srgb(1.0, 0.90, 0.40)
                                        } else {
                                            Color::srgb(0.70, 0.72, 0.78)
                                        }),
                                        text_shadow_default(),
                                    ));
                                });

                            // Tab 2: Personal Inventory
                            tab_row
                                .spawn((
                                    Button,
                                    InventoryTabButton {
                                        tab: InventoryTab::Player,
                                    },
                                    Node {
                                        padding: UiRect::axes(px(14.0), px(6.0)),
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(6.0)),
                                        display: Display::Flex,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    if current_tab == InventoryTab::Player {
                                        BackgroundColor(Color::srgba(0.24, 0.24, 0.32, 0.95))
                                    } else {
                                        BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.70))
                                    },
                                    if current_tab == InventoryTab::Player {
                                        BorderColor::all(Color::srgb(1.0, 0.85, 0.30))
                                    } else {
                                        BorderColor::all(Color::srgba(0.28, 0.28, 0.35, 0.60))
                                    },
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        InventoryTabButtonText {
                                            tab: InventoryTab::Player,
                                        },
                                        Text::new("PERSONAL INVENTORY"),
                                        tab2_font,
                                        TextColor(if current_tab == InventoryTab::Player {
                                            Color::srgb(1.0, 0.90, 0.40)
                                        } else {
                                            Color::srgb(0.70, 0.72, 0.78)
                                        }),
                                        text_shadow_default(),
                                    ));
                                });
                        });

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
                                        let voxel = match current_tab {
                                            InventoryTab::Creative => {
                                                let block_idx =
                                                    start_row * INVENTORY_COLS + slot_idx;
                                                AVAILABLE_BLOCKS.get(block_idx).copied()
                                            }
                                            InventoryTab::Player => player_inv.get(slot_idx),
                                        };

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

                            // Minecraft-style Scrollbar Track (visible in Creative tab, hidden in Player tab)
                            palette_row
                                .spawn((
                                    Button,
                                    InventoryScrollTrack,
                                    Node {
                                        width: if current_tab == InventoryTab::Creative {
                                            px(16.0)
                                        } else {
                                            px(0.0)
                                        },
                                        height: px(track_height),
                                        display: track_display,
                                        position_type: PositionType::Relative,
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.04, 0.04, 0.06, 0.90)),
                                    BorderColor::all(Color::srgba(0.22, 0.22, 0.26, 0.70)),
                                    track_vis,
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
                                    let mut num_font = TextFont {
                                        font_size: FontSize::Px(10.0),
                                        ..default()
                                    };
                                    if let Some(ref font) = font_handle {
                                        num_font.font = font.clone();
                                    }

                                    slot.spawn((
                                        Text::new(format!("{}", slot_idx + 1)),
                                        num_font,
                                        TextColor(Color::srgba(0.85, 0.85, 0.85, 0.60)),
                                        text_shadow_default(),
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
    mut player_inv: ResMut<PlayerInventory>,
    mut hotbar: ResMut<Hotbar>,
) {
    if let Some(held) = held_item.voxel {
        if !player_inv.add_item(held)
            && let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none())
        {
            hotbar.slots[empty_idx] = Some(held);
        }
        held_item.voxel = None;
    }

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
fn handle_inventory_tab_interaction(
    mouse: Res<ButtonInput<MouseButton>>,
    mut tab_state: ResMut<InventoryTab>,
    scroll_state: Res<InventoryScrollState>,
    icons: Res<BlockIcons>,
    player_inv: Res<PlayerInventory>,
    mut tab_button_query: Query<
        (
            &Interaction,
            &InventoryTabButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (
            With<InventoryTabButton>,
            Without<InventoryScrollThumb>,
            Without<InventoryScrollTrack>,
        ),
    >,
    mut tab_text_query: Query<(&InventoryTabButtonText, &mut TextColor)>,
    mut track_node_query: Query<
        (&mut Node, &mut Visibility),
        (
            With<InventoryScrollTrack>,
            Without<InventoryPaletteSlotIcon>,
        ),
    >,
    mut palette_query: Query<
        &mut InventoryPaletteSlot,
        (
            Without<InventoryTabButton>,
            Without<InventoryScrollThumb>,
            Without<InventoryScrollTrack>,
        ),
    >,
    mut slot_icon_query: Query<
        (&InventoryPaletteSlotIcon, &mut ImageNode, &mut Visibility),
        Without<InventoryScrollTrack>,
    >,
) {
    let left_just_pressed = mouse.just_pressed(MouseButton::Left);
    let mut tab_switched = false;

    for (interaction, tab_btn, mut bg, mut border) in &mut tab_button_query {
        let is_active = *tab_state == tab_btn.tab;
        let is_hovered = *interaction == Interaction::Hovered;
        let is_pressed = *interaction == Interaction::Pressed;

        if left_just_pressed && is_pressed && !is_active {
            *tab_state = tab_btn.tab;
            tab_switched = true;
        }

        if is_active {
            *bg = BackgroundColor(Color::srgba(0.24, 0.24, 0.32, 0.95));
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
        } else if is_hovered {
            *bg = BackgroundColor(Color::srgba(0.18, 0.18, 0.24, 0.85));
            *border = BorderColor::all(Color::srgba(0.45, 0.45, 0.55, 0.80));
        } else {
            *bg = BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.70));
            *border = BorderColor::all(Color::srgba(0.28, 0.28, 0.35, 0.60));
        }
    }

    for (tab_text, mut text_color) in &mut tab_text_query {
        if *tab_state == tab_text.tab {
            text_color.0 = Color::srgb(1.0, 0.90, 0.40);
        } else {
            text_color.0 = Color::srgb(0.70, 0.72, 0.78);
        }
    }

    if tab_switched {
        for (mut track_node, mut track_vis) in &mut track_node_query {
            match *tab_state {
                InventoryTab::Creative => {
                    track_node.display = Display::Flex;
                    track_node.width = px(16.0);
                    *track_vis = Visibility::Visible;
                }
                InventoryTab::Player => {
                    track_node.display = Display::None;
                    track_node.width = px(0.0);
                    *track_vis = Visibility::Hidden;
                }
            }
        }

        let cur_row = scroll_state.scroll_row;
        for mut slot in &mut palette_query {
            slot.voxel = match *tab_state {
                InventoryTab::Creative => {
                    let block_idx = cur_row * INVENTORY_COLS + slot.slot_index;
                    AVAILABLE_BLOCKS.get(block_idx).copied()
                }
                InventoryTab::Player => player_inv.get(slot.slot_index),
            };
        }

        for (icon, mut img, mut vis) in &mut slot_icon_query {
            let voxel = match *tab_state {
                InventoryTab::Creative => {
                    let block_idx = cur_row * INVENTORY_COLS + icon.slot_index;
                    AVAILABLE_BLOCKS.get(block_idx).copied()
                }
                InventoryTab::Player => player_inv.get(icon.slot_index),
            };
            if let Some(v) = voxel {
                *vis = Visibility::Visible;
                img.image = icons.get(v);
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_inventory_scroll(
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    tab_state: Res<InventoryTab>,
    mut scroll_state: ResMut<InventoryScrollState>,
    icons: Res<BlockIcons>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    track_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryScrollTrack>>,
    thumb_query: Query<&Interaction, With<InventoryScrollThumb>>,
    mut thumb_node_query: Query<
        (&mut Node, &mut BackgroundColor, &mut BorderColor),
        (With<InventoryScrollThumb>, Without<InventoryScrollTrack>),
    >,
    mut palette_query: Query<&mut InventoryPaletteSlot>,
    mut slot_icon_query: Query<(&InventoryPaletteSlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    if *tab_state != InventoryTab::Creative {
        return;
    }

    let left_just_pressed = mouse.just_pressed(MouseButton::Left);
    let left_pressed = mouse.pressed(MouseButton::Left);
    let max_row = max_scroll_row();
    let mut row_changed = false;

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

    if mouse.just_released(MouseButton::Left) {
        scroll_state.is_dragging_thumb = false;
    }

    if row_changed {
        let cur_row = scroll_state.scroll_row;
        for mut slot in &mut palette_query {
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
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_inventory_slot_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut held_item: ResMut<HeldInventoryItem>,
    mut hotbar: ResMut<Hotbar>,
    mut player_inv: ResMut<PlayerInventory>,
    tab_state: Res<InventoryTab>,
    icons: Res<BlockIcons>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    card_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryCard>>,
    mut palette_query: Query<
        (
            &Interaction,
            &mut InventoryPaletteSlot,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        (With<InventoryPaletteSlot>, Without<InventoryHotbarSlot>),
    >,
    mut slot_icon_query: Query<(&InventoryPaletteSlotIcon, &mut ImageNode, &mut Visibility)>,
    mut hotbar_slot_query: Query<
        (
            &Interaction,
            &InventoryHotbarSlot,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        (With<InventoryHotbarSlot>, Without<InventoryPaletteSlot>),
    >,
    mut drag_state: Local<InventoryDragState>,
) {
    let is_shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let left_just_pressed = mouse.just_pressed(MouseButton::Left);
    let right_just_pressed = mouse.just_pressed(MouseButton::Right);
    let middle_just_pressed = mouse.just_pressed(MouseButton::Middle);
    let left_pressed = mouse.pressed(MouseButton::Left);

    // 1. Deselect held item if clicking outside the central inventory card (on the backdrop)
    if (left_just_pressed || right_just_pressed)
        && let (Some(window), Some((transform, computed))) =
            (window_query.iter().next(), card_query.iter().next())
        && let Some(cursor_pos) = window.cursor_position()
    {
        let center = transform.translation().truncate();
        let half = computed.size() * 0.5;
        let card_rect = Rect::from_corners(center - half, center + half);
        if !card_rect.contains(cursor_pos)
            && let Some(held) = held_item.voxel
        {
            if *tab_state == InventoryTab::Player {
                player_inv.add_item(held);
            }
            held_item.voxel = None;
        }
    }

    // 2. Right-click deselect if not hovering over any hotbar slot or palette slot
    let any_hotbar_hovered = hotbar_slot_query
        .iter()
        .any(|(i, _, _, _)| *i == Interaction::Hovered || *i == Interaction::Pressed);
    let any_palette_hovered = palette_query
        .iter()
        .any(|(i, _, _, _)| *i == Interaction::Hovered || *i == Interaction::Pressed);

    if right_just_pressed
        && !any_hotbar_hovered
        && !any_palette_hovered
        && let Some(held) = held_item.voxel
    {
        if *tab_state == InventoryTab::Player {
            player_inv.add_item(held);
        }
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

    let mut sync_player_grid_icons = false;

    // 3. Process Grid Slots (Creative vs Player mode)
    for (interaction, mut slot, mut border, mut bg) in &mut palette_query {
        let is_hovered = *interaction == Interaction::Hovered;
        let is_pressed = *interaction == Interaction::Pressed;
        let s_idx = slot.slot_index;

        if is_hovered || is_pressed {
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            *bg = BackgroundColor(Color::srgba(0.20, 0.20, 0.25, 0.95));

            match *tab_state {
                InventoryTab::Creative => {
                    if let Some(v) = slot.voxel {
                        if let Some(target_slot) = digit_pressed {
                            hotbar.slots[target_slot] = Some(v);
                        }

                        if is_shift {
                            if (left_just_pressed || left_pressed)
                                && drag_state.last_shift_palette_slot != Some(s_idx)
                            {
                                drag_state.last_shift_palette_slot = Some(s_idx);
                                if let Some(empty_idx) =
                                    hotbar.slots.iter().position(|s| s.is_none())
                                {
                                    hotbar.slots[empty_idx] = Some(v);
                                } else {
                                    let active_slot = hotbar.active_slot;
                                    hotbar.slots[active_slot] = Some(v);
                                }
                            }
                        } else {
                            if middle_just_pressed {
                                held_item.voxel = Some(v);
                            }

                            if left_just_pressed && is_pressed {
                                if held_item.voxel == Some(v) {
                                    held_item.voxel = None;
                                } else {
                                    held_item.voxel = Some(v);
                                }
                            }
                        }
                    }
                }
                InventoryTab::Player => {
                    // Quick swap / move with hotbar via digit keys 1..8
                    if let Some(target_slot) = digit_pressed {
                        let inv_item = player_inv.get(s_idx);
                        let hotbar_item = hotbar.slots[target_slot];
                        player_inv.set(s_idx, hotbar_item);
                        hotbar.slots[target_slot] = inv_item;
                        slot.voxel = player_inv.get(s_idx);
                        sync_player_grid_icons = true;
                    }

                    // Shift + Click: transfer from Personal Inventory to Hotbar
                    if is_shift {
                        if left_just_pressed && let Some(item) = player_inv.get(s_idx) {
                            if let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none()) {
                                hotbar.slots[empty_idx] = Some(item);
                                player_inv.set(s_idx, None);
                            } else {
                                let active_slot = hotbar.active_slot;
                                let existing = hotbar.slots[active_slot];
                                hotbar.slots[active_slot] = Some(item);
                                player_inv.set(s_idx, existing);
                            }
                            slot.voxel = player_inv.get(s_idx);
                            sync_player_grid_icons = true;
                        }
                    } else if let Some(held) = held_item.voxel {
                        // Left-click with held item on cursor
                        if left_just_pressed && is_pressed {
                            if let Some(existing) = player_inv.get(s_idx) {
                                if existing == held {
                                    held_item.voxel = None;
                                } else {
                                    player_inv.set(s_idx, Some(held));
                                    held_item.voxel = Some(existing);
                                }
                            } else {
                                player_inv.set(s_idx, Some(held));
                                held_item.voxel = None;
                            }
                            slot.voxel = player_inv.get(s_idx);
                            sync_player_grid_icons = true;
                        } else if right_just_pressed {
                            player_inv.set(s_idx, Some(held));
                            slot.voxel = player_inv.get(s_idx);
                            sync_player_grid_icons = true;
                        }
                    } else {
                        // Left-click with empty cursor picks up item
                        if left_just_pressed
                            && is_pressed
                            && let Some(item) = player_inv.get(s_idx)
                        {
                            held_item.voxel = Some(item);
                            player_inv.set(s_idx, None);
                            slot.voxel = None;
                            sync_player_grid_icons = true;
                        } else if (middle_just_pressed || keyboard.just_pressed(KeyCode::KeyQ))
                            && player_inv.get(s_idx).is_some()
                        {
                            player_inv.set(s_idx, None);
                            slot.voxel = None;
                            sync_player_grid_icons = true;
                        }
                    }
                }
            }
        } else {
            *border = BorderColor::all(Color::srgba(0.25, 0.25, 0.30, 0.60));
            *bg = BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.85));
        }
    }

    // 4. Process Hotbar Slots
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

            // Shift + Click handling
            if is_shift {
                if *tab_state == InventoryTab::Player {
                    // In Personal tab, Shift-click transfers hotbar item into Player Inventory!
                    if left_just_pressed
                        && let Some(item) = hotbar.slots[idx]
                        && player_inv.add_item(item)
                    {
                        hotbar.slots[idx] = None;
                        sync_player_grid_icons = true;
                    }
                } else {
                    // In Creative tab, Shift-click clears hotbar slot
                    if left_just_pressed || (left_pressed && is_hovered) {
                        hotbar.slots[idx] = None;
                    }
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
                            drag_state.hotbar_drag_start_slot = Some(idx);
                            drag_state.is_hotbar_dragging = false;

                            if let Some(existing) = hotbar.slots[idx] {
                                if existing != held {
                                    hotbar.slots[idx] = Some(held);
                                    held_item.voxel = Some(existing);
                                    drag_state.hotbar_drag_start_slot = None;
                                } else {
                                    held_item.voxel = None;
                                }
                            } else {
                                hotbar.slots[idx] = Some(held);
                            }
                        } else if drag_state.hotbar_drag_start_slot.is_some()
                            && drag_state.hotbar_drag_start_slot != Some(idx)
                        {
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

    if sync_player_grid_icons && *tab_state == InventoryTab::Player {
        for (_, mut slot, _, _) in &mut palette_query {
            slot.voxel = player_inv.get(slot.slot_index);
        }
        for (icon, mut img, mut vis) in &mut slot_icon_query {
            if let Some(v) = player_inv.get(icon.slot_index) {
                *vis = Visibility::Visible;
                img.image = icons.get(v);
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }

    // 5. Mouse release cleanup
    if mouse.just_released(MouseButton::Left) {
        if drag_state.hotbar_drag_start_slot.is_some() {
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
    use crate::player::inventory::PLAYER_INVENTORY_SLOTS;

    #[test]
    fn test_inventory_dimensions_and_blocks() {
        assert_eq!(
            INVENTORY_COLS * INVENTORY_VISIBLE_ROWS,
            INVENTORY_VISIBLE_SLOTS
        );
        assert_eq!(INVENTORY_VISIBLE_ROWS, 4);
        assert_eq!(AVAILABLE_BLOCKS.len(), 62);
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

    #[test]
    fn test_tab_toggle_state() {
        let mut tab = InventoryTab::default();
        assert_eq!(tab, InventoryTab::Creative);

        tab = InventoryTab::Player;
        assert_eq!(tab, InventoryTab::Player);

        tab = InventoryTab::Creative;
        assert_eq!(tab, InventoryTab::Creative);
    }

    #[test]
    fn test_player_inventory_to_hotbar_shift_transfer() {
        let mut player_inv = PlayerInventory::default();
        let mut hotbar = Hotbar {
            slots: [None; HOTBAR_SLOT_COUNT],
            active_slot: 0,
        };

        player_inv.set(0, Some(Voxel::Cobblestone));
        player_inv.set(1, Some(Voxel::Sand));

        // Shift-click slot 0 moves to hotbar slot 0
        let item = player_inv.get(0).unwrap();
        if let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none()) {
            hotbar.slots[empty_idx] = Some(item);
            player_inv.set(0, None);
        }
        assert_eq!(player_inv.get(0), None);
        assert_eq!(hotbar.slots[0], Some(Voxel::Cobblestone));

        // Shift-click slot 1 moves to hotbar slot 1
        let item2 = player_inv.get(1).unwrap();
        if let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none()) {
            hotbar.slots[empty_idx] = Some(item2);
            player_inv.set(1, None);
        }
        assert_eq!(player_inv.get(1), None);
        assert_eq!(hotbar.slots[1], Some(Voxel::Sand));
    }

    #[test]
    fn test_hotbar_to_player_inventory_shift_transfer() {
        let mut player_inv = PlayerInventory::default();
        let mut hotbar = Hotbar {
            slots: [None; HOTBAR_SLOT_COUNT],
            active_slot: 0,
        };

        hotbar.slots[3] = Some(Voxel::Granite);

        // Shift-click hotbar slot 3 in Personal Tab transfers to player inventory
        let item = hotbar.slots[3].unwrap();
        assert!(player_inv.add_item(item));
        hotbar.slots[3] = None;

        assert_eq!(hotbar.slots[3], None);
        assert_eq!(player_inv.get(0), Some(Voxel::Granite));
    }

    #[test]
    fn test_player_inventory_number_key_swap() {
        let mut player_inv = PlayerInventory::default();
        let mut hotbar = Hotbar {
            slots: [None; HOTBAR_SLOT_COUNT],
            active_slot: 0,
        };

        player_inv.set(5, Some(Voxel::Dirt));
        hotbar.slots[2] = Some(Voxel::Stone);

        // Pressing digit 3 (hotbar index 2) while hovering slot 5 swaps them
        let s_idx = 5;
        let target_slot = 2;
        let inv_item = player_inv.get(s_idx);
        let hotbar_item = hotbar.slots[target_slot];
        player_inv.set(s_idx, hotbar_item);
        hotbar.slots[target_slot] = inv_item;

        assert_eq!(player_inv.get(5), Some(Voxel::Stone));
        assert_eq!(hotbar.slots[2], Some(Voxel::Dirt));
    }

    #[test]
    fn test_close_inventory_returns_held_item() {
        let mut player_inv = PlayerInventory::default();
        let mut hotbar = Hotbar {
            slots: [None; HOTBAR_SLOT_COUNT],
            active_slot: 0,
        };
        let mut held_item = HeldInventoryItem {
            voxel: Some(Voxel::Basalt),
        };

        // When player inventory has space:
        if let Some(held) = held_item.voxel.take() {
            if !player_inv.add_item(held) {
                if let Some(empty) = hotbar.slots.iter().position(|s| s.is_none()) {
                    hotbar.slots[empty] = Some(held);
                } else {
                    let active = hotbar.active_slot;
                    hotbar.slots[active] = Some(held);
                }
            }
        }
        assert_eq!(held_item.voxel, None);
        assert_eq!(player_inv.get(0), Some(Voxel::Basalt));

        // When player inventory is full: returns to hotbar
        for i in 0..PLAYER_INVENTORY_SLOTS {
            player_inv.set(i, Some(Voxel::Dirt));
        }
        held_item.voxel = Some(Voxel::Stone);
        if let Some(held) = held_item.voxel.take() {
            if !player_inv.add_item(held) {
                if let Some(empty) = hotbar.slots.iter().position(|s| s.is_none()) {
                    hotbar.slots[empty] = Some(held);
                } else {
                    let active = hotbar.active_slot;
                    hotbar.slots[active] = Some(held);
                }
            }
        }
        assert_eq!(held_item.voxel, None);
        assert_eq!(hotbar.slots[0], Some(Voxel::Stone));
    }
}
