use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, window::PrimaryWindow};

use crate::{
    core::{AppFont, FontSource, text_shadow_default},
    gameplay::BlockIcons,
    player::hotbar::{HOTBAR_SLOT_COUNT, Hotbar},
    player::inventory::PlayerInventory,
    world::Voxel,
};

use super::{HeldInventoryItem, MenuState};

pub const GUI_SCALE: f32 = 3.5;

pub const INVENTORY_COLS: usize = 8;
pub const INVENTORY_VISIBLE_ROWS: usize = 4;

// Central Inventory Panel base texture size (190 x 152 px)
pub const INVENTORY_PANEL_TEX_W: f32 = 190.0;
pub const INVENTORY_PANEL_TEX_H: f32 = 152.0;

pub const AVAILABLE_BLOCKS: [Voxel; 67] = [
    // Soils, Organics & Fine Sediment
    Voxel::Grass,
    Voxel::SnowyGrass,
    Voxel::Dirt,
    Voxel::PackedDirt,
    Voxel::RootedDirt,
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
    // Woods, Foliage & Planks
    Voxel::OakWoodLog,
    Voxel::OakWood,
    Voxel::OakPlanks,
    Voxel::OakLeaves,
    Voxel::BirchWoodLog,
    Voxel::BirchWood,
    Voxel::BirchPlanks,
    Voxel::BirchLeaves,
    Voxel::PineWoodLog,
    Voxel::PineWood,
    Voxel::PinePlanks,
    Voxel::PineLeaves,
    Voxel::RainwoodWoodLog,
    Voxel::RainwoodWood,
    Voxel::RainwoodPlanks,
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

pub fn filtered_creative_blocks(query: &str) -> Vec<Voxel> {
    if query.trim().is_empty() {
        return AVAILABLE_BLOCKS.to_vec();
    }
    let q = query.trim().to_lowercase();
    AVAILABLE_BLOCKS
        .iter()
        .copied()
        .filter(|&v| format!("{:?}", v).to_lowercase().contains(&q) || v.label().to_lowercase().contains(&q))
        .collect()
}

pub fn total_filtered_rows(filtered_count: usize) -> usize {
    filtered_count
        .div_ceil(INVENTORY_COLS)
        .max(INVENTORY_VISIBLE_ROWS)
}

pub fn max_filtered_scroll(filtered_count: usize) -> usize {
    total_filtered_rows(filtered_count).saturating_sub(INVENTORY_VISIBLE_ROWS)
}

/// The active inventory view mode.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InventoryTab {
    Creative,
    #[default]
    Player,
}

#[derive(Resource, Default)]
pub struct InventoryScrollState {
    pub scroll_row: usize,
    pub is_dragging_thumb: bool,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CreativeSearchQuery {
    pub query: String,
    pub is_focused: bool,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
}

#[derive(Component)]
pub struct CreativeSearchBar;

#[derive(Component)]
pub struct CreativeSearchText;

#[derive(Component)]
pub struct InventoryMenuEntity;

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
            .init_resource::<CreativeSearchQuery>()
            .add_systems(OnEnter(MenuState::Inventory), spawn_inventory_menu)
            .add_systems(OnExit(MenuState::Inventory), despawn_inventory_menu)
            .add_systems(
                Update,
                (
                    handle_inventory_tab_key,
                    handle_inventory_tab_interaction,
                    handle_creative_search_input,
                    handle_inventory_scroll,
                    handle_inventory_slot_interaction,
                    sync_inventory_hotbar_icons,
                )
                    .chain()
                    .run_if(in_state(MenuState::Inventory)),
            );
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_inventory_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    tab_state: Res<InventoryTab>,
    scroll_state: Res<InventoryScrollState>,
    search_query: Res<CreativeSearchQuery>,
    app_font: Option<Res<AppFont>>,
) {
    build_inventory_ui(
        &mut commands,
        &asset_server,
        &hotbar,
        &player_inv,
        &icons,
        *tab_state,
        &scroll_state,
        &search_query,
        app_font.as_ref().map(|f| f.source()),
    );
}

#[allow(clippy::too_many_arguments)]
fn build_inventory_ui(
    commands: &mut Commands,
    asset_server: &AssetServer,
    hotbar: &Hotbar,
    player_inv: &PlayerInventory,
    icons: &BlockIcons,
    current_tab: InventoryTab,
    scroll_state: &InventoryScrollState,
    search_query: &CreativeSearchQuery,
    font_source: Option<FontSource>,
) {
    let make_font = |size: f32| -> TextFont {
        let mut tf = TextFont {
            font_size: FontSize::Px(size),
            ..default()
        };
        if let Some(ref fs) = font_source {
            tf.font = fs.clone();
        }
        tf
    };

    let inv_tex = asset_server.load("textures/interfaces/containers/inventory.png");
    let cre_tex = asset_server.load("textures/interfaces/containers/inventory_creative.png");
    let scr_tex = asset_server.load("textures/interfaces/containers/inventory_scroller.png");

    let tab_font = make_font(16.0);
    let title_font = make_font(16.0);
    let btn_font = make_font(16.0);

    commands
        .spawn((
            InventoryMenuEntity,
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
            // Main Central Panel (190 x 152 px base image)
            backdrop
                .spawn((
                    InventoryMenuEntity,
                    InventoryCard,
                    ImageNode {
                        image: if current_tab == InventoryTab::Creative {
                            cre_tex.clone()
                        } else {
                            inv_tex.clone()
                        },
                        ..default()
                    },
                    Node {
                        width: px(INVENTORY_PANEL_TEX_W * GUI_SCALE),  // 570.0 px
                        height: px(INVENTORY_PANEL_TEX_H * GUI_SCALE), // 456.0 px
                        position_type: PositionType::Relative,
                        ..default()
                    },
                ))
                .with_children(|card| {
                    // 1. Two Top Toggle Buttons (Mode Switcher above both interfaces)
                    // Button 1: Personal Mode (X=21..92, Y=1..14)
                    card.spawn((
                        InventoryMenuEntity,
                        Button,
                        InventoryTabButton {
                            tab: InventoryTab::Player,
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(21.0 * GUI_SCALE),  // 63.0
                            top: px(1.0 * GUI_SCALE),    // 3.0
                            width: px(72.0 * GUI_SCALE), // 216.0
                            height: px(14.0 * GUI_SCALE),// 42.0
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        BorderColor::all(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            InventoryMenuEntity,
                            InventoryTabButtonText {
                                tab: InventoryTab::Player,
                            },
                            Text::new("Personal"),
                            tab_font.clone(),
                            TextColor(if current_tab == InventoryTab::Player {
                                Color::srgb(1.0, 0.90, 0.40)
                            } else {
                                Color::srgb(0.70, 0.72, 0.78)
                            }),
                            text_shadow_default(),
                        ));
                    });

                    // Button 2: Creative Mode (X=97..168, Y=1..14)
                    card.spawn((
                        InventoryMenuEntity,
                        Button,
                        InventoryTabButton {
                            tab: InventoryTab::Creative,
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(97.0 * GUI_SCALE),  // 291.0
                            top: px(1.0 * GUI_SCALE),    // 3.0
                            width: px(72.0 * GUI_SCALE), // 216.0
                            height: px(14.0 * GUI_SCALE),// 42.0
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        BorderColor::all(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            InventoryMenuEntity,
                            InventoryTabButtonText {
                                tab: InventoryTab::Creative,
                            },
                            Text::new("Creative"),
                            tab_font,
                            TextColor(if current_tab == InventoryTab::Creative {
                                Color::srgb(1.0, 0.90, 0.40)
                            } else {
                                Color::srgb(0.70, 0.72, 0.78)
                            }),
                            text_shadow_default(),
                        ));
                    });

                    // 2. Title Area (X=22..90, Y=32..43)
                    card.spawn((
                        InventoryMenuEntity,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(22.0 * GUI_SCALE), // 66.0
                            top: px(32.0 * GUI_SCALE),  // 96.0
                            width: px(75.0 * GUI_SCALE), // 225.0
                            height: px(12.0 * GUI_SCALE),// 36.0
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            padding: UiRect::left(px(6.0)),
                            ..default()
                        },
                    ))
                    .with_children(|title_box| {
                        let title_text = match current_tab {
                            InventoryTab::Player => "Personal Inventory",
                            InventoryTab::Creative => "Creative Inventory",
                        };
                        title_box.spawn((
                            InventoryMenuEntity,
                            Text::new(title_text),
                            title_font,
                            TextColor(Color::srgb(0.92, 0.92, 0.95)),
                            text_shadow_default(),
                        ));
                    });

                    match current_tab {
                        InventoryTab::Player => {
                            // Section for two buttons (currently non-functional)
                            // Button A: X=138..149, Y=32..43
                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(138.0 * GUI_SCALE), // 414.0
                                    top: px(32.0 * GUI_SCALE),   // 96.0
                                    width: px(12.0 * GUI_SCALE), // 36.0
                                    height: px(12.0 * GUI_SCALE),// 36.0
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|b1| {
                                b1.spawn((
                                    InventoryMenuEntity,
                                    Text::new("B"),
                                    btn_font.clone(),
                                    TextColor(Color::srgba(0.85, 0.85, 0.90, 0.70)),
                                    text_shadow_default(),
                                ));
                            });

                            // Button B: X=156..167, Y=32..43
                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(156.0 * GUI_SCALE), // 468.0
                                    top: px(32.0 * GUI_SCALE),   // 96.0
                                    width: px(12.0 * GUI_SCALE), // 36.0
                                    height: px(12.0 * GUI_SCALE),// 36.0
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|b2| {
                                b2.spawn((
                                    InventoryMenuEntity,
                                    Text::new("B"),
                                    btn_font,
                                    TextColor(Color::srgba(0.85, 0.85, 0.90, 0.70)),
                                    text_shadow_default(),
                                ));
                            });

                            // 8x4 Grid Slots for PlayerInventory
                            for r in 0..INVENTORY_VISIBLE_ROWS {
                                for c in 0..INVENTORY_COLS {
                                    let slot_idx = r * INVENTORY_COLS + c;
                                    let voxel = player_inv.get(slot_idx);
                                    let icon_handle = voxel
                                        .map(|v| icons.get(v))
                                        .unwrap_or_else(|| icons.get(Voxel::Stone));
                                    let icon_vis = if voxel.is_some() {
                                        Visibility::Visible
                                    } else {
                                        Visibility::Hidden
                                    };

                                    let slot_left = (24.0 + c as f32 * 18.0) * GUI_SCALE;
                                    let slot_top = (52.0 + r as f32 * 18.0) * GUI_SCALE;

                                    card.spawn((
                                        InventoryMenuEntity,
                                        Button,
                                        InventoryPaletteSlot {
                                            slot_index: slot_idx,
                                            voxel,
                                        },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(slot_left),
                                            top: px(slot_top),
                                            width: px(16.0 * GUI_SCALE),
                                            height: px(16.0 * GUI_SCALE),
                                            display: Display::Flex,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(px(2.0)),
                                            border_radius: BorderRadius::all(px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                        BorderColor::all(Color::NONE),
                                    ))
                                    .with_children(|slot| {
                                        slot.spawn((
                                            InventoryMenuEntity,
                                            InventoryPaletteSlotIcon {
                                                slot_index: slot_idx,
                                            },
                                            ImageNode {
                                                image: icon_handle,
                                                ..default()
                                            },
                                            Node {
                                                width: px(36.0),
                                                height: px(36.0),
                                                ..default()
                                            },
                                            icon_vis,
                                        ));
                                    });
                                }
                            }
                        }
                        InventoryTab::Creative => {
                            // Search Bar: X=98..168, Y=32..43
                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                CreativeSearchBar,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(98.0 * GUI_SCALE),  // 294.0
                                    top: px(32.0 * GUI_SCALE),   // 96.0
                                    width: px(71.0 * GUI_SCALE), // 213.0
                                    height: px(12.0 * GUI_SCALE),// 36.0
                                    padding: UiRect::horizontal(px(6.0)),
                                    display: Display::Flex,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::all(px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(if search_query.selection.is_some() {
                                    Color::srgba(0.20, 0.50, 0.95, 0.35)
                                } else {
                                    Color::NONE
                                }),
                                BorderColor::all(if search_query.is_focused {
                                    Color::srgb(1.0, 0.85, 0.30)
                                } else {
                                    Color::NONE
                                }),
                            ))
                            .with_children(|sb| {
                                let search_font = make_font(15.0);
                                let is_empty = search_query.query.is_empty();
                                let has_selection = search_query.selection.is_some() && !is_empty;
                                let display_text = if !search_query.is_focused {
                                    if is_empty {
                                        "Search".to_string()
                                    } else {
                                        search_query.query.clone()
                                    }
                                } else if has_selection {
                                    search_query.query.clone()
                                } else if is_empty {
                                    "|".to_string()
                                } else {
                                    let cur = search_query.cursor.min(search_query.query.len());
                                    format!("{}|{}", &search_query.query[..cur], &search_query.query[cur..])
                                };
                                sb.spawn((
                                    InventoryMenuEntity,
                                    CreativeSearchText,
                                    Text::new(display_text),
                                    search_font,
                                    TextColor(if !search_query.is_focused && is_empty {
                                        Color::srgba(0.70, 0.70, 0.75, 0.60)
                                    } else if has_selection {
                                        Color::srgb(0.40, 0.85, 1.0)
                                    } else {
                                        Color::srgb(0.95, 0.95, 0.95)
                                    }),
                                    text_shadow_default(),
                                ));
                            });

                            // 8x4 Grid Slots for Creative Blocks
                            let filtered = filtered_creative_blocks(&search_query.query);
                            let start_row = scroll_state.scroll_row;

                            for r in 0..INVENTORY_VISIBLE_ROWS {
                                for c in 0..INVENTORY_COLS {
                                    let slot_idx = r * INVENTORY_COLS + c;
                                    let block_idx = start_row * INVENTORY_COLS + slot_idx;
                                    let voxel = filtered.get(block_idx).copied();
                                    let icon_handle = voxel
                                        .map(|v| icons.get(v))
                                        .unwrap_or_else(|| icons.get(Voxel::Stone));
                                    let icon_vis = if voxel.is_some() {
                                        Visibility::Visible
                                    } else {
                                        Visibility::Hidden
                                    };

                                    let slot_left = (24.0 + c as f32 * 18.0) * GUI_SCALE;
                                    let slot_top = (52.0 + r as f32 * 18.0) * GUI_SCALE;

                                    card.spawn((
                                        InventoryMenuEntity,
                                        Button,
                                        InventoryPaletteSlot {
                                            slot_index: slot_idx,
                                            voxel,
                                        },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(slot_left),
                                            top: px(slot_top),
                                            width: px(16.0 * GUI_SCALE),
                                            height: px(16.0 * GUI_SCALE),
                                            display: Display::Flex,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(px(2.0)),
                                            border_radius: BorderRadius::all(px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                        BorderColor::all(Color::NONE),
                                    ))
                                    .with_children(|slot| {
                                        slot.spawn((
                                            InventoryMenuEntity,
                                            InventoryPaletteSlotIcon {
                                                slot_index: slot_idx,
                                            },
                                            ImageNode {
                                                image: icon_handle,
                                                ..default()
                                            },
                                            Node {
                                                width: px(36.0),
                                                height: px(36.0),
                                                ..default()
                                            },
                                            icon_vis,
                                        ));
                                    });
                                }
                            }

                            // Scrollbar Track & Thumb (X=178..183, Y=33..142)
                            let track_height = 110.0 * GUI_SCALE; // 330.0
                            let thumb_height = 15.0 * GUI_SCALE;  // 45.0
                            let max_travel = track_height - thumb_height; // 285.0
                            let max_scroll = max_filtered_scroll(filtered.len());
                            let thumb_top = if max_scroll > 0 {
                                (start_row as f32 / max_scroll as f32) * max_travel
                            } else {
                                0.0
                            };

                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                InventoryScrollTrack,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(175.0 * GUI_SCALE), // 525.0
                                    top: px(33.0 * GUI_SCALE),   // 99.0
                                    width: px(12.0 * GUI_SCALE), // 36.0
                                    height: px(track_height),    // 330.0
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|track| {
                                track.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    InventoryScrollThumb,
                                    ImageNode {
                                        image: scr_tex.clone(),
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(0.0),
                                        top: px(thumb_top),
                                        width: px(12.0 * GUI_SCALE),
                                        height: px(thumb_height),
                                        ..default()
                                    },
                                ));
                            });
                        }
                    }

                    // 1x8 Hotbar Row (Y=128..143)
                    for c in 0..HOTBAR_SLOT_COUNT {
                        let slot_voxel = hotbar.slots[c];
                        let icon_handle = slot_voxel
                            .map(|v| icons.get(v))
                            .unwrap_or_else(|| icons.get(Voxel::Stone));
                        let icon_vis = if slot_voxel.is_some() {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        };

                        let slot_left = (24.0 + c as f32 * 18.0) * GUI_SCALE;
                        let slot_top = 128.0 * GUI_SCALE; // 384.0

                        card.spawn((
                            InventoryMenuEntity,
                            Button,
                            InventoryHotbarSlot { index: c },
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(slot_left),
                                top: px(slot_top),
                                width: px(16.0 * GUI_SCALE),
                                height: px(16.0 * GUI_SCALE),
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(2.0)),
                                border_radius: BorderRadius::all(px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(Color::NONE),
                        ))
                        .with_children(|slot| {
                            // Centered 2D item icon (rendered first, below slot number)
                            slot.spawn((
                                InventoryMenuEntity,
                                InventoryHotbarSlotIcon { index: c },
                                ImageNode {
                                    image: icon_handle,
                                    ..default()
                                },
                                Node {
                                    width: px(36.0),
                                    height: px(36.0),
                                    ..default()
                                },
                                icon_vis,
                            ));

                            // Slot index number (1 through 8, rendered in front with ZIndex)
                            let num_font = make_font(13.0);
                            slot.spawn((
                                InventoryMenuEntity,
                                Text::new(format!("{}", c + 1)),
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

#[allow(clippy::too_many_arguments)]
fn despawn_inventory_menu(
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    mut held_item: ResMut<HeldInventoryItem>,
    mut scroll_state: ResMut<InventoryScrollState>,
    mut search_query: ResMut<CreativeSearchQuery>,
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
    search_query.query.clear();
    search_query.cursor = 0;
    search_query.is_focused = false;
    search_query.selection = None;

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

#[allow(clippy::too_many_arguments)]
fn handle_inventory_tab_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tab_state: ResMut<InventoryTab>,
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    asset_server: Res<AssetServer>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    scroll_state: Res<InventoryScrollState>,
    mut search_query: ResMut<CreativeSearchQuery>,
    app_font: Option<Res<AppFont>>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        *tab_state = match *tab_state {
            InventoryTab::Creative => InventoryTab::Player,
            InventoryTab::Player => InventoryTab::Creative,
        };
        search_query.is_focused = false;
        search_query.cursor = 0;
        search_query.selection = None;
        for entity in &query {
            commands.entity(entity).despawn();
        }
        build_inventory_ui(
            &mut commands,
            &asset_server,
            &hotbar,
            &player_inv,
            &icons,
            *tab_state,
            &scroll_state,
            &search_query,
            app_font.as_ref().map(|f| f.source()),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_inventory_tab_interaction(
    mouse: Res<ButtonInput<MouseButton>>,
    mut tab_state: ResMut<InventoryTab>,
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    asset_server: Res<AssetServer>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    scroll_state: Res<InventoryScrollState>,
    mut search_query: ResMut<CreativeSearchQuery>,
    app_font: Option<Res<AppFont>>,
    mut tab_button_query: Query<
        (
            &Interaction,
            &InventoryTabButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<InventoryTabButton>,
    >,
    mut tab_text_query: Query<(&InventoryTabButtonText, &mut TextColor)>,
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
            *bg = BackgroundColor(Color::NONE);
            *border = BorderColor::all(Color::NONE);
        } else if is_hovered {
            *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08));
            *border = BorderColor::all(Color::NONE);
        } else {
            *bg = BackgroundColor(Color::NONE);
            *border = BorderColor::all(Color::NONE);
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
        search_query.is_focused = false;
        search_query.cursor = 0;
        search_query.selection = None;
        for entity in &query {
            commands.entity(entity).despawn();
        }
        build_inventory_ui(
            &mut commands,
            &asset_server,
            &hotbar,
            &player_inv,
            &icons,
            *tab_state,
            &scroll_state,
            &search_query,
            app_font.as_ref().map(|f| f.source()),
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum KeyAction {
    Char(char),
    Backspace,
    Delete,
    Left,
    Right,
}

#[derive(Default)]
struct SearchInputState {
    active_repeat: Option<(KeyCode, KeyAction)>,
    repeat_timer: f32,
    last_click_time: f32,
    drag_start_cursor: Option<usize>,
}

fn apply_search_key_action(
    action: KeyAction,
    search_query: &mut CreativeSearchQuery,
    changed: &mut bool,
) {
    match action {
        KeyAction::Char(ch) => {
            if let Some((start, end)) = search_query.selection {
                let start = start.min(search_query.query.len());
                let end = end.min(search_query.query.len());
                if start < end {
                    search_query.query.replace_range(start..end, &ch.to_string());
                    search_query.cursor = start + 1;
                    search_query.selection = None;
                    *changed = true;
                }
            } else if search_query.query.len() < 24 {
                let cur = search_query.cursor.min(search_query.query.len());
                search_query.query.insert(cur, ch);
                search_query.cursor = cur + 1;
                *changed = true;
            }
        }
        KeyAction::Backspace => {
            if let Some((start, end)) = search_query.selection {
                let start = start.min(search_query.query.len());
                let end = end.min(search_query.query.len());
                if start < end {
                    search_query.query.drain(start..end);
                    search_query.cursor = start;
                    search_query.selection = None;
                    *changed = true;
                }
            } else if search_query.cursor > 0 && !search_query.query.is_empty() {
                let cur = search_query.cursor.min(search_query.query.len());
                search_query.query.remove(cur - 1);
                search_query.cursor = cur - 1;
                *changed = true;
            }
        }
        KeyAction::Delete => {
            if let Some((start, end)) = search_query.selection {
                let start = start.min(search_query.query.len());
                let end = end.min(search_query.query.len());
                if start < end {
                    search_query.query.drain(start..end);
                    search_query.cursor = start;
                    search_query.selection = None;
                    *changed = true;
                }
            } else {
                let cur = search_query.cursor.min(search_query.query.len());
                if cur < search_query.query.len() {
                    search_query.query.remove(cur);
                    *changed = true;
                }
            }
        }
        KeyAction::Left => {
            if let Some((start, _)) = search_query.selection {
                search_query.cursor = start.min(search_query.query.len());
                search_query.selection = None;
            } else {
                search_query.cursor = search_query.cursor.saturating_sub(1);
            }
        }
        KeyAction::Right => {
            if let Some((_, end)) = search_query.selection {
                search_query.cursor = end.min(search_query.query.len());
                search_query.selection = None;
            } else {
                search_query.cursor = (search_query.cursor + 1).min(search_query.query.len());
            }
        }
    }
}

fn get_card_top_left(
    card_query: &Query<(&GlobalTransform, &ComputedNode), With<InventoryCard>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    if let Some((card_tf, card_node)) = card_query.iter().next() {
        let center = card_tf.translation().truncate();
        let half = card_node.size() * 0.5;
        if half.x > 10.0 && half.y > 10.0 {
            return Some(center - half);
        }
    }
    if let Some(w) = window_query.iter().next() {
        return Some(Vec2::new(
            (w.width() - INVENTORY_PANEL_TEX_W * GUI_SCALE) * 0.5,
            (w.height() - INVENTORY_PANEL_TEX_H * GUI_SCALE) * 0.5,
        ));
    }
    None
}

#[cfg(target_os = "windows")]
fn is_caps_lock_on() -> bool {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetKeyState(n_virt_key: i32) -> i16;
    }
    unsafe { (GetKeyState(0x14) & 1) != 0 }
}

#[cfg(not(target_os = "windows"))]
fn is_caps_lock_on() -> bool {
    false
}

#[allow(clippy::too_many_arguments)]
fn handle_creative_search_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    tab_state: Res<InventoryTab>,
    mut search_query: ResMut<CreativeSearchQuery>,
    mut scroll_state: ResMut<InventoryScrollState>,
    mut search_text_query: Query<(&mut Text, &mut TextColor), With<CreativeSearchText>>,
    mut search_bar_query: Query<
        (
            &Interaction,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        With<CreativeSearchBar>,
    >,
    mut palette_query: Query<&mut InventoryPaletteSlot>,
    mut slot_icon_query: Query<(&InventoryPaletteSlotIcon, &mut ImageNode, &mut Visibility)>,
    card_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryCard>>,
    icons: Res<BlockIcons>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut input_state: Local<SearchInputState>,
) {
    if *tab_state != InventoryTab::Creative {
        return;
    }

    let cursor_pos = window_query.iter().next().and_then(|w| w.cursor_position());
    let card_top_left = get_card_top_left(&card_query, &window_query);

    let (search_bar_rect, text_start_x) = if let Some(top_left) = card_top_left {
        let min_x = top_left.x + 98.0 * GUI_SCALE;
        let min_y = top_left.y + 32.0 * GUI_SCALE;
        let max_x = min_x + 71.0 * GUI_SCALE;
        let max_y = min_y + 12.0 * GUI_SCALE;
        (
            Some(Rect::new(min_x - 4.0, min_y - 4.0, max_x + 4.0, max_y + 4.0)),
            min_x + 6.0,
        )
    } else {
        (None, 0.0)
    };

    let left_just_pressed = mouse.just_pressed(MouseButton::Left);
    let left_pressed = mouse.pressed(MouseButton::Left);

    let is_inside_search_bar = cursor_pos.is_some_and(|pos| {
        search_bar_rect.is_some_and(|r| r.contains(pos))
    });

    let is_search_pressed = search_bar_query
        .iter()
        .any(|(i, _, _)| *i == Interaction::Pressed)
        || (left_just_pressed && is_inside_search_bar);

    let is_search_hovered = search_bar_query
        .iter()
        .any(|(i, _, _)| *i == Interaction::Hovered || *i == Interaction::Pressed)
        || is_inside_search_bar;

    if left_just_pressed {
        if is_inside_search_bar || is_search_pressed {
            search_query.is_focused = true;
            let now = time.elapsed_secs();
            let is_double_click = (now - input_state.last_click_time) < 0.35
                && (now - input_state.last_click_time) > 0.0;

            if is_double_click && !search_query.query.is_empty() {
                search_query.selection = Some((0, search_query.query.len()));
                search_query.cursor = search_query.query.len();
                input_state.last_click_time = 0.0;
                input_state.drag_start_cursor = None;
            } else {
                input_state.last_click_time = now;
                search_query.selection = None;
                if let Some(pos) = cursor_pos {
                    let click_x = (pos.x - text_start_x).max(0.0);
                    let char_width = 8.5;
                    let approx_idx = (click_x / char_width).round() as usize;
                    search_query.cursor = approx_idx.min(search_query.query.len());
                    input_state.drag_start_cursor = Some(search_query.cursor);
                } else {
                    search_query.cursor = search_query.query.len();
                }
            }
        } else {
            search_query.is_focused = false;
            search_query.selection = None;
            input_state.active_repeat = None;
            input_state.drag_start_cursor = None;
        }
    }

    // Drag selection while holding left mouse
    if left_pressed && search_query.is_focused {
        if let (Some(anchor), Some(pos)) = (input_state.drag_start_cursor, cursor_pos) {
            let click_x = (pos.x - text_start_x).max(0.0);
            let char_width = 8.5;
            let cur_idx = ((click_x / char_width).round() as usize).min(search_query.query.len());
            if cur_idx != anchor && !search_query.query.is_empty() {
                let start = anchor.min(cur_idx);
                let end = anchor.max(cur_idx);
                search_query.selection = Some((start, end));
                search_query.cursor = cur_idx;
            }
        }
    }

    if mouse.just_released(MouseButton::Left) {
        input_state.drag_start_cursor = None;
    }

    let target_border = if search_query.is_focused {
        BorderColor::all(Color::srgb(1.0, 0.85, 0.30))
    } else if is_search_hovered {
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.40))
    } else {
        BorderColor::all(Color::NONE)
    };
    if let Some((_, mut border, _)) = search_bar_query.iter_mut().next() {
        if *border != target_border {
            *border = target_border;
        }
    }

    let target_bg = if search_query.is_focused
        && search_query.selection.is_some()
        && !search_query.query.is_empty()
    {
        BackgroundColor(Color::srgba(0.20, 0.50, 0.95, 0.35))
    } else {
        BackgroundColor(Color::NONE)
    };
    if let Some((_, _, mut bg)) = search_bar_query.iter_mut().next() {
        if *bg != target_bg {
            *bg = target_bg;
        }
    }

    let mut changed = false;

    if keyboard.just_pressed(KeyCode::Escape) && search_query.is_focused {
        search_query.is_focused = false;
        search_query.selection = None;
        input_state.active_repeat = None;
        input_state.drag_start_cursor = None;
    }

    let char_keys: &[(KeyCode, char, char)] = &[
        (KeyCode::KeyA, 'a', 'A'),
        (KeyCode::KeyB, 'b', 'B'),
        (KeyCode::KeyC, 'c', 'C'),
        (KeyCode::KeyD, 'd', 'D'),
        (KeyCode::KeyE, 'e', 'E'),
        (KeyCode::KeyF, 'f', 'F'),
        (KeyCode::KeyG, 'g', 'G'),
        (KeyCode::KeyH, 'h', 'H'),
        (KeyCode::KeyI, 'i', 'I'),
        (KeyCode::KeyJ, 'j', 'J'),
        (KeyCode::KeyK, 'k', 'K'),
        (KeyCode::KeyL, 'l', 'L'),
        (KeyCode::KeyM, 'm', 'M'),
        (KeyCode::KeyN, 'n', 'N'),
        (KeyCode::KeyO, 'o', 'O'),
        (KeyCode::KeyP, 'p', 'P'),
        (KeyCode::KeyQ, 'q', 'Q'),
        (KeyCode::KeyR, 'r', 'R'),
        (KeyCode::KeyS, 's', 'S'),
        (KeyCode::KeyT, 't', 'T'),
        (KeyCode::KeyU, 'u', 'U'),
        (KeyCode::KeyV, 'v', 'V'),
        (KeyCode::KeyW, 'w', 'W'),
        (KeyCode::KeyX, 'x', 'X'),
        (KeyCode::KeyY, 'y', 'Y'),
        (KeyCode::KeyZ, 'z', 'Z'),
        (KeyCode::Digit0, '0', '0'),
        (KeyCode::Digit1, '1', '1'),
        (KeyCode::Digit2, '2', '2'),
        (KeyCode::Digit3, '3', '3'),
        (KeyCode::Digit4, '4', '4'),
        (KeyCode::Digit5, '5', '5'),
        (KeyCode::Digit6, '6', '6'),
        (KeyCode::Digit7, '7', '7'),
        (KeyCode::Digit8, '8', '8'),
        (KeyCode::Digit9, '9', '9'),
        (KeyCode::Numpad0, '0', '0'),
        (KeyCode::Numpad1, '1', '1'),
        (KeyCode::Numpad2, '2', '2'),
        (KeyCode::Numpad3, '3', '3'),
        (KeyCode::Numpad4, '4', '4'),
        (KeyCode::Numpad5, '5', '5'),
        (KeyCode::Numpad6, '6', '6'),
        (KeyCode::Numpad7, '7', '7'),
        (KeyCode::Numpad8, '8', '8'),
        (KeyCode::Numpad9, '9', '9'),
        (KeyCode::Space, ' ', ' '),
        (KeyCode::Minus, '-', '_'),
    ];

    if search_query.is_focused {
        let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
        let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
        let caps = is_caps_lock_on();

        let mut triggered_action: Option<(KeyCode, KeyAction)> = None;

        if ctrl && keyboard.just_pressed(KeyCode::KeyA) {
            if !search_query.query.is_empty() {
                search_query.selection = Some((0, search_query.query.len()));
                search_query.cursor = search_query.query.len();
            }
        } else if !ctrl {
            if keyboard.just_pressed(KeyCode::Backspace) {
                triggered_action = Some((KeyCode::Backspace, KeyAction::Backspace));
            } else if keyboard.just_pressed(KeyCode::Delete) {
                triggered_action = Some((KeyCode::Delete, KeyAction::Delete));
            } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
                triggered_action = Some((KeyCode::ArrowLeft, KeyAction::Left));
            } else if keyboard.just_pressed(KeyCode::ArrowRight) {
                triggered_action = Some((KeyCode::ArrowRight, KeyAction::Right));
            } else if keyboard.just_pressed(KeyCode::Home) {
                search_query.cursor = 0;
                search_query.selection = None;
            } else if keyboard.just_pressed(KeyCode::End) {
                search_query.cursor = search_query.query.len();
                search_query.selection = None;
            } else {
                for &(key, normal_ch, shift_ch) in char_keys {
                    if keyboard.just_pressed(key) {
                        let is_letter = normal_ch.is_ascii_alphabetic();
                        let ch = if is_letter {
                            if shift ^ caps {
                                shift_ch
                            } else {
                                normal_ch
                            }
                        } else if shift {
                            shift_ch
                        } else {
                            normal_ch
                        };
                        triggered_action = Some((key, KeyAction::Char(ch)));
                        break;
                    }
                }
            }
        }

        if let Some((k, act)) = triggered_action {
            apply_search_key_action(act, &mut search_query, &mut changed);
            input_state.active_repeat = Some((k, act));
            input_state.repeat_timer = 0.40;
        }

        // Key repeat processing for holding any key down
        if let Some((active_k, active_act)) = input_state.active_repeat {
            if keyboard.pressed(active_k) {
                if triggered_action.map(|(k, _)| k) != Some(active_k) {
                    input_state.repeat_timer -= time.delta_secs();
                    while input_state.repeat_timer <= 0.0 {
                        apply_search_key_action(active_act, &mut search_query, &mut changed);
                        input_state.repeat_timer += 0.04;
                    }
                }
            } else {
                input_state.active_repeat = None;
            }
        }
    } else {
        input_state.active_repeat = None;
    }

    if changed {
        scroll_state.scroll_row = 0;
        let filtered = filtered_creative_blocks(&search_query.query);

        // Update slots
        for mut slot in &mut palette_query {
            let block_idx = slot.slot_index;
            slot.voxel = filtered.get(block_idx).copied();
        }

        for (icon, mut img, mut vis) in &mut slot_icon_query {
            let block_idx = icon.slot_index;
            if let Some(&v) = filtered.get(block_idx) {
                *vis = Visibility::Visible;
                img.image = icons.get(v);
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }

    // Always update search text rendering to reflect query, cursor, and selection
    let has_selection = search_query.is_focused && search_query.selection.is_some() && !search_query.query.is_empty();
    for (mut text, mut color) in &mut search_text_query {
        let target_str = if !search_query.is_focused {
            if search_query.query.is_empty() {
                "Search".to_string()
            } else {
                search_query.query.clone()
            }
        } else if has_selection {
            search_query.query.clone()
        } else if search_query.query.is_empty() {
            "|".to_string()
        } else {
            let cur = search_query.cursor.min(search_query.query.len());
            format!("{}|{}", &search_query.query[..cur], &search_query.query[cur..])
        };

        if text.0 != target_str {
            text.0 = target_str;
        }

        let target_color = if !search_query.is_focused && search_query.query.is_empty() {
            Color::srgba(0.70, 0.70, 0.75, 0.60)
        } else if has_selection {
            Color::srgb(0.40, 0.85, 1.0)
        } else {
            Color::srgb(0.95, 0.95, 0.95)
        };
        if color.0 != target_color {
            color.0 = target_color;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_inventory_scroll(
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    tab_state: Res<InventoryTab>,
    search_query: Res<CreativeSearchQuery>,
    mut scroll_state: ResMut<InventoryScrollState>,
    icons: Res<BlockIcons>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    card_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryCard>>,
    track_interaction_query: Query<&Interaction, With<InventoryScrollTrack>>,
    thumb_query: Query<&Interaction, With<InventoryScrollThumb>>,
    mut thumb_node_query: Query<&mut Node, With<InventoryScrollThumb>>,
    mut palette_query: Query<&mut InventoryPaletteSlot>,
    mut slot_icon_query: Query<(&InventoryPaletteSlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    if *tab_state != InventoryTab::Creative {
        return;
    }

    let filtered = filtered_creative_blocks(&search_query.query);
    let max_scroll = max_filtered_scroll(filtered.len());
    let mut row_changed = false;

    let track_height = 110.0 * GUI_SCALE; // 330.0 px
    let thumb_height = 15.0 * GUI_SCALE;  // 45.0 px
    let max_travel = (track_height - thumb_height).max(1.0); // 285.0 px

    // 1. Mouse wheel scrolling
    let scroll_y = mouse_scroll.delta.y;
    if scroll_y > 0.05 && scroll_state.scroll_row > 0 {
        scroll_state.scroll_row -= 1;
        row_changed = true;
    } else if scroll_y < -0.05 && scroll_state.scroll_row < max_scroll {
        scroll_state.scroll_row += 1;
        row_changed = true;
    }

    // 2. Dragging the thumb or clicking on the track
    let left_pressed = mouse.pressed(MouseButton::Left);
    let left_just_pressed = mouse.just_pressed(MouseButton::Left);
    let cursor_pos = window_query.iter().next().and_then(|w| w.cursor_position());
    let card_top_left = get_card_top_left(&card_query, &window_query);

    let (track_screen_x, track_screen_y) = if let Some(top_left) = card_top_left {
        (
            top_left.x + 175.0 * GUI_SCALE,
            top_left.y + 33.0 * GUI_SCALE,
        )
    } else {
        (0.0, 0.0)
    };

    let track_hit_rect = Rect::new(
        track_screen_x - 16.0,
        track_screen_y - 6.0,
        track_screen_x + 12.0 * GUI_SCALE + 16.0,
        track_screen_y + track_height + 6.0,
    );

    let is_inside_track = cursor_pos.is_some_and(|pos| track_hit_rect.contains(pos));

    let track_pressed = track_interaction_query
        .iter()
        .any(|i| *i == Interaction::Pressed)
        || thumb_query.iter().any(|i| *i == Interaction::Pressed);

    if left_just_pressed && (is_inside_track || track_pressed) {
        scroll_state.is_dragging_thumb = true;
    }

    if !left_pressed {
        scroll_state.is_dragging_thumb = false;
    }

    if scroll_state.is_dragging_thumb && left_pressed {
        if let Some(pos) = cursor_pos {
            let cursor_rel_y = pos.y - track_screen_y;
            let clamped_y = (cursor_rel_y - thumb_height * 0.5).clamp(0.0, max_travel);

            for mut thumb_node in &mut thumb_node_query {
                thumb_node.top = px(clamped_y);
            }

            let fraction = clamped_y / max_travel;
            let target_row = ((fraction * max_scroll as f32).round() as usize).min(max_scroll);

            if target_row != scroll_state.scroll_row {
                scroll_state.scroll_row = target_row;
                row_changed = true;
            }
        }
    } else {
        // Sync thumb position whenever not actively dragging
        let cur_row = scroll_state.scroll_row;
        let thumb_top = if max_scroll > 0 {
            (cur_row as f32 / max_scroll as f32) * max_travel
        } else {
            0.0
        };

        for mut thumb_node in &mut thumb_node_query {
            thumb_node.top = px(thumb_top);
        }
    }

    // 3. Update slot icons when row changes
    if row_changed {
        let cur_row = scroll_state.scroll_row;
        for mut slot in &mut palette_query {
            let block_idx = cur_row * INVENTORY_COLS + slot.slot_index;
            slot.voxel = filtered.get(block_idx).copied();
        }

        for (icon, mut img, mut vis) in &mut slot_icon_query {
            let block_idx = cur_row * INVENTORY_COLS + icon.slot_index;
            if let Some(&v) = filtered.get(block_idx) {
                *vis = Visibility::Visible;
                img.image = icons.get(v);
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
        (
            With<InventoryPaletteSlot>,
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
        ),
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

    // 4. Process Grid Slots (Creative vs Player mode)
    for (interaction, mut slot, mut border, mut bg) in &mut palette_query {
        let is_hovered = *interaction == Interaction::Hovered;
        let is_pressed = *interaction == Interaction::Pressed;
        let s_idx = slot.slot_index;

        if is_hovered || is_pressed {
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12));

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
                        if (left_just_pressed || (left_pressed && is_hovered))
                            && drag_state.last_shift_palette_slot != Some(s_idx)
                            && let Some(item) = player_inv.get(s_idx)
                        {
                            drag_state.last_shift_palette_slot = Some(s_idx);
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
            *border = BorderColor::all(Color::NONE);
            *bg = BackgroundColor(Color::NONE);
        }
    }

    // 5. Process Hotbar Slots
    for (interaction, hotbar_slot, mut border, mut bg) in &mut hotbar_slot_query {
        let is_hovered = *interaction == Interaction::Hovered;
        let is_pressed = *interaction == Interaction::Pressed;
        let idx = hotbar_slot.index;

        if is_hovered || is_pressed {
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12));

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
                    // In Personal tab, Shift-click or Shift-drag transfers hotbar item into Player Inventory!
                    if (left_just_pressed || (left_pressed && is_hovered))
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
            *border = BorderColor::all(Color::NONE);
            *bg = BackgroundColor(Color::NONE);
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

    // 6. Mouse release cleanup
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
