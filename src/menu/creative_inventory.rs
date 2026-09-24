use bevy::{
    input::mouse::AccumulatedMouseScroll,
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    core::{text_shadow_default, AppFont, FontSource},
    gameplay::BlockIcons,
    player::hotbar::{HOTBAR_SLOT_COUNT, Hotbar},
    player::inventory::PlayerInventory,
    world::Voxel,
};

use super::{HeldInventoryItem, MenuState};

pub const GUI_SCALE: f32 = 3.0;

pub const INVENTORY_COLS: usize = 8;
pub const INVENTORY_VISIBLE_ROWS: usize = 4;
#[allow(dead_code)]
pub const INVENTORY_VISIBLE_SLOTS: usize = INVENTORY_COLS * INVENTORY_VISIBLE_ROWS; // 32 slots

// Left Card texture coordinates (0, 0) to (86, 122)
pub const LEFT_CARD_TEX_W: f32 = 86.0;
pub const LEFT_CARD_TEX_H: f32 = 122.0;

// Right Card (Standard) texture coordinates (90, 18) to (248, 144)
pub const RIGHT_CARD_STD_TEX_W: f32 = 158.0;
pub const RIGHT_CARD_STD_TEX_H: f32 = 126.0;

// Right Card (Creative) texture coordinates (90, 18) to (266, 144)
pub const RIGHT_CARD_CRE_TEX_W: f32 = 176.0;
pub const RIGHT_CARD_CRE_TEX_H: f32 = 126.0;

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

#[allow(dead_code)]
pub fn total_inventory_rows() -> usize {
    AVAILABLE_BLOCKS.len().div_ceil(INVENTORY_COLS)
}

#[allow(dead_code)]
pub fn max_scroll_row() -> usize {
    total_inventory_rows().saturating_sub(INVENTORY_VISIBLE_ROWS)
}

pub fn filtered_creative_blocks(query: &str) -> Vec<Voxel> {
    if query.trim().is_empty() {
        return AVAILABLE_BLOCKS.to_vec();
    }
    let q = query.trim().to_lowercase();
    AVAILABLE_BLOCKS
        .iter()
        .copied()
        .filter(|&v| format!("{:?}", v).to_lowercase().contains(&q))
        .collect()
}

pub fn total_filtered_rows(filtered_count: usize) -> usize {
    filtered_count.div_ceil(INVENTORY_COLS).max(INVENTORY_VISIBLE_ROWS)
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlotType {
    Helmet,
    ChestArmor,
    Gloves,
    Pants,
    Boots,
}

impl ArmorSlotType {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            ArmorSlotType::Helmet => "Helmet",
            ArmorSlotType::ChestArmor => "Chest Armor",
            ArmorSlotType::Gloves => "Gloves",
            ArmorSlotType::Pants => "Pants",
            ArmorSlotType::Boots => "Boots",
        }
    }

    pub fn placeholder_label(&self) -> &'static str {
        match self {
            ArmorSlotType::Helmet => "H",
            ArmorSlotType::ChestArmor => "C",
            ArmorSlotType::Gloves => "G",
            ArmorSlotType::Pants => "P",
            ArmorSlotType::Boots => "B",
        }
    }
}

pub const ARMOR_SLOT_TYPES: [ArmorSlotType; 5] = [
    ArmorSlotType::Helmet,
    ArmorSlotType::ChestArmor,
    ArmorSlotType::Gloves,
    ArmorSlotType::Pants,
    ArmorSlotType::Boots,
];

#[derive(Component)]
#[allow(dead_code)]
pub struct ArmorSlotUi {
    pub slot_type: ArmorSlotType,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct PlayerModelViewport;

#[derive(Component)]
pub struct CreativeSearchBar;

#[derive(Component)]
pub struct CreativeSearchText;

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
            .add_systems(
                OnEnter(MenuState::Inventory),
                spawn_inventory_menu,
            )
            .add_systems(
                OnExit(MenuState::Inventory),
                despawn_inventory_menu,
            )
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

    let inv_tex = asset_server.load("textures/gui/containers/inventory.png");
    let cre_tex = asset_server.load("textures/gui/containers/creative_inventory.png");
    let scr_tex = asset_server.load("textures/gui/containers/scroller.png");

    let tab1_font = make_font(12.0);
    let tab2_font = make_font(12.0);

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
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            // Top Tab Navigation Row
            backdrop
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
                                padding: UiRect::axes(px(14.0), px(5.0)),
                                border: UiRect::all(px(1.5)),
                                border_radius: BorderRadius::all(px(5.0)),
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
                                padding: UiRect::axes(px(14.0), px(5.0)),
                                border: UiRect::all(px(1.5)),
                                border_radius: BorderRadius::all(px(5.0)),
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

            // Main Dual-Card Row (Left Card + Right Card)
            backdrop
                .spawn((
                    InventoryCard,
                    Node {
                        width: px(798.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        column_gap: px(4.0 * GUI_SCALE), // 12 px gap
                        align_items: AlignItems::FlexStart,
                        justify_content: JustifyContent::FlexStart,
                        ..default()
                    },
                ))
                .with_children(|card_row| {
                    // --- LEFT CARD ---
                    card_row
                        .spawn((
                            ImageNode {
                                image: if current_tab == InventoryTab::Creative {
                                    cre_tex.clone()
                                } else {
                                    inv_tex.clone()
                                },
                                rect: Some(Rect {
                                    min: Vec2::new(0.0, 0.0),
                                    max: Vec2::new(LEFT_CARD_TEX_W, LEFT_CARD_TEX_H),
                                }),
                                ..default()
                            },
                            Node {
                                width: px(LEFT_CARD_TEX_W * GUI_SCALE),  // 258.0
                                height: px(LEFT_CARD_TEX_H * GUI_SCALE), // 366.0
                                position_type: PositionType::Relative,
                                ..default()
                            },
                        ))
                        .with_children(|left_card| {
                            // 1. Player Name Header (X=6..80, Y=6..21)
                            left_card
                                .spawn(Node {
                                    position_type: PositionType::Absolute,
                                    left: px(6.0 * GUI_SCALE),    // 18.0
                                    top: px(6.0 * GUI_SCALE),     // 18.0
                                    width: px(74.0 * GUI_SCALE),  // 222.0
                                    height: px(15.0 * GUI_SCALE), // 45.0
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::horizontal(px(10.0)),
                                    ..default()
                                })
                                .with_children(|header| {
                                    let header_font = make_font(18.0);
                                    header.spawn((
                                        Text::new("Player Name"),
                                        header_font.clone(),
                                        TextColor(Color::srgb(0.92, 0.92, 0.95)),
                                        text_shadow_default(),
                                    ));
                                    header.spawn((
                                        Text::new("Level 10"),
                                        header_font,
                                        TextColor(Color::srgb(0.85, 0.85, 0.90)),
                                        text_shadow_default(),
                                    ));
                                });

                            // 2. Player Model Viewport Space (blank for now)
                            left_card.spawn((
                                PlayerModelViewport,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(6.0 * GUI_SCALE),    // 18.0
                                    top: px(26.0 * GUI_SCALE),    // 78.0
                                    width: px(53.0 * GUI_SCALE),  // 159.0
                                    height: px(88.0 * GUI_SCALE), // 264.0
                                    ..default()
                                },
                            ));

                            // 3. 5 Armor Slots Column (X=62..80, Y=26, 44, 62, 80, 98)
                            for (i, &slot_type) in ARMOR_SLOT_TYPES.iter().enumerate() {
                                let top_y = (26.0 + i as f32 * 18.0) * GUI_SCALE;
                                left_card
                                    .spawn((
                                        Button,
                                        ArmorSlotUi { slot_type },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(62.0 * GUI_SCALE - 1.0), // 185.0 (moved 1px left)
                                            top: px(top_y - 1.0),             // (moved 1px up)
                                            width: px(18.0 * GUI_SCALE),  // 54.0
                                            height: px(18.0 * GUI_SCALE), // 54.0
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
                                        let label_font = make_font(16.0);
                                        slot.spawn((
                                            Text::new(slot_type.placeholder_label()),
                                            label_font,
                                            TextColor(Color::srgba(0.85, 0.85, 0.90, 0.35)),
                                            text_shadow_default(),
                                        ));
                                    });
                            }
                        });

                    // --- RIGHT CARD ---
                    match current_tab {
                        InventoryTab::Player => {
                            // Standard Right Card: 90, 18 to 248, 144 (158 x 126 px)
                            card_row
                                .spawn((
                                    ImageNode {
                                        image: inv_tex.clone(),
                                        rect: Some(Rect {
                                            min: Vec2::new(90.0, 18.0),
                                            max: Vec2::new(248.0, 144.0),
                                        }),
                                        ..default()
                                    },
                                    Node {
                                        width: px(RIGHT_CARD_STD_TEX_W * GUI_SCALE),  // 474.0
                                        height: px(RIGHT_CARD_STD_TEX_H * GUI_SCALE), // 378.0
                                        position_type: PositionType::Relative,
                                        margin: UiRect::top(px(18.0 * GUI_SCALE)),    // 54.0
                                        ..default()
                                    },
                                ))
                                .with_children(|right_card| {
                                    // Title "Inventory" (shifted slightly right and down)
                                    let title_font = make_font(20.0);
                                    let btn_font = make_font(18.0);
                                    right_card.spawn((
                                        Text::new("Inventory"),
                                        title_font.clone(),
                                        TextColor(Color::srgb(0.92, 0.92, 0.95)),
                                        text_shadow_default(),
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(24.0),
                                            top: px(30.0),
                                            height: px(12.0 * GUI_SCALE),
                                            display: Display::Flex,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                    ));

                                    // Button 1 "B" (Relative X=118, Y=8, W=14, H=12) - non-functional placeholder
                                    right_card
                                        .spawn((
                                            Button,
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: px(118.0 * GUI_SCALE),   // 354.0
                                                top: px(8.0 * GUI_SCALE),     // 24.0
                                                width: px(14.0 * GUI_SCALE),  // 42.0
                                                height: px(12.0 * GUI_SCALE), // 36.0
                                                display: Display::Flex,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::NONE),
                                        ))
                                        .with_children(|b1| {
                                            b1.spawn((
                                                Text::new("B"),
                                                btn_font.clone(),
                                                TextColor(Color::srgba(0.85, 0.85, 0.90, 0.70)),
                                                text_shadow_default(),
                                            ));
                                        });

                                    // Button 2 "B" (Relative X=136, Y=8, W=14, H=12) - non-functional placeholder
                                    right_card
                                        .spawn((
                                            Button,
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: px(136.0 * GUI_SCALE),   // 408.0
                                                top: px(8.0 * GUI_SCALE),     // 24.0
                                                width: px(14.0 * GUI_SCALE),  // 42.0
                                                height: px(12.0 * GUI_SCALE), // 36.0
                                                display: Display::Flex,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::NONE),
                                        ))
                                        .with_children(|b2| {
                                            b2.spawn((
                                                Text::new("B"),
                                                btn_font.clone(),
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

                                            let slot_left = (6.0 + c as f32 * 18.0) * GUI_SCALE + 1.0;
                                            let slot_top = (26.0 + r as f32 * 18.0) * GUI_SCALE - 1.0;

                                            right_card
                                                .spawn((
                                                    Button,
                                                    InventoryPaletteSlot {
                                                        slot_index: slot_idx,
                                                        voxel,
                                                    },
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        left: px(slot_left),
                                                        top: px(slot_top),
                                                        width: px(18.0 * GUI_SCALE),
                                                        height: px(18.0 * GUI_SCALE),
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

                                    // 1x8 Hotbar Row
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

                                        let slot_left = (6.0 + c as f32 * 18.0) * GUI_SCALE + 1.0;
                                        let slot_top = 102.0 * GUI_SCALE - 1.0; // 305.0

                                        right_card
                                            .spawn((
                                                Button,
                                                InventoryHotbarSlot { index: c },
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: px(slot_left),
                                                    top: px(slot_top),
                                                    width: px(18.0 * GUI_SCALE),
                                                    height: px(18.0 * GUI_SCALE),
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
                                                let num_font = make_font(11.0);
                                                slot.spawn((
                                                    Text::new(format!("{}", c + 1)),
                                                    num_font,
                                                    TextColor(Color::srgba(
                                                        0.90, 0.90, 0.90, 0.75,
                                                    )),
                                                    text_shadow_default(),
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        top: px(0.0),
                                                        left: px(7.0),
                                                        ..default()
                                                    },
                                                ));

                                                slot.spawn((
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
                                            });
                                    }
                                });
                        }
                        InventoryTab::Creative => {
                            // Creative Right Card: 90, 18 to 266, 144 (176 x 126 px)
                            card_row
                                .spawn((
                                    ImageNode {
                                        image: cre_tex.clone(),
                                        rect: Some(Rect {
                                            min: Vec2::new(90.0, 18.0),
                                            max: Vec2::new(266.0, 144.0),
                                        }),
                                        ..default()
                                    },
                                    Node {
                                        width: px(RIGHT_CARD_CRE_TEX_W * GUI_SCALE),  // 528.0
                                        height: px(RIGHT_CARD_CRE_TEX_H * GUI_SCALE), // 378.0
                                        position_type: PositionType::Relative,
                                        margin: UiRect::top(px(18.0 * GUI_SCALE)),    // 54.0
                                        ..default()
                                    },
                                ))
                                .with_children(|right_card| {
                                    // Title "Inventory" (shifted slightly right and down)
                                    let title_font = make_font(20.0);
                                    right_card.spawn((
                                        Text::new("Inventory"),
                                        title_font.clone(),
                                        TextColor(Color::srgb(0.92, 0.92, 0.95)),
                                        text_shadow_default(),
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(24.0),
                                            top: px(30.0),
                                            height: px(12.0 * GUI_SCALE),
                                            display: Display::Flex,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                    ));

                                    // Search Bar (Texture X=169..241, Y=26..38 -> rel X=79, Y=8)
                                    right_card
                                        .spawn((
                                            Button,
                                            CreativeSearchBar,
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: px(237.0),
                                                top: px(24.0),
                                                width: px(216.0),
                                                height: px(36.0),
                                                padding: UiRect::horizontal(px(8.0)),
                                                display: Display::Flex,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(px(1.5)),
                                                border_radius: BorderRadius::all(px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::NONE),
                                            BorderColor::all(if search_query.is_focused {
                                                Color::srgb(1.0, 0.85, 0.30)
                                            } else {
                                                Color::NONE
                                            }),
                                        ))
                                        .with_children(|sb| {
                                            let search_font = make_font(20.0);
                                            let is_empty = search_query.query.is_empty();
                                            sb.spawn((
                                                CreativeSearchText,
                                                Text::new(if is_empty {
                                                    "Search"
                                                } else {
                                                    &search_query.query
                                                }),
                                                search_font,
                                                TextColor(if is_empty {
                                                    Color::srgba(0.70, 0.70, 0.75, 0.60)
                                                } else {
                                                    Color::srgb(0.95, 0.95, 0.95)
                                                }),
                                                text_shadow_default(),
                                            ));
                                        });

                                    // 8x4 Grid Slots for Creative Blocks
                                    let filtered =
                                        filtered_creative_blocks(&search_query.query);
                                    let start_row = scroll_state.scroll_row;

                                    for r in 0..INVENTORY_VISIBLE_ROWS {
                                        for c in 0..INVENTORY_COLS {
                                            let slot_idx = r * INVENTORY_COLS + c;
                                            let block_idx =
                                                start_row * INVENTORY_COLS + slot_idx;
                                            let voxel = filtered.get(block_idx).copied();
                                            let icon_handle = voxel
                                                .map(|v| icons.get(v))
                                                .unwrap_or_else(|| icons.get(Voxel::Stone));
                                            let icon_vis = if voxel.is_some() {
                                                Visibility::Visible
                                            } else {
                                                Visibility::Hidden
                                            };

                                            let slot_left =
                                                (6.0 + c as f32 * 18.0) * GUI_SCALE + 1.0;
                                            let slot_top =
                                                (26.0 + r as f32 * 18.0) * GUI_SCALE - 1.0;

                                            right_card
                                                .spawn((
                                                    Button,
                                                    InventoryPaletteSlot {
                                                        slot_index: slot_idx,
                                                        voxel,
                                                    },
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        left: px(slot_left),
                                                        top: px(slot_top),
                                                        width: px(18.0 * GUI_SCALE),
                                                        height: px(18.0 * GUI_SCALE),
                                                        display: Display::Flex,
                                                        justify_content:
                                                            JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        border: UiRect::all(px(2.0)),
                                                        border_radius: BorderRadius::all(
                                                            px(2.0),
                                                        ),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::NONE),
                                                    BorderColor::all(Color::NONE),
                                                ))
                                                .with_children(|slot| {
                                                    slot.spawn((
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

                                    // 1x8 Hotbar Row
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

                                        let slot_left = (6.0 + c as f32 * 18.0) * GUI_SCALE + 1.0;
                                        let slot_top = 102.0 * GUI_SCALE - 1.0;

                                        right_card
                                            .spawn((
                                                Button,
                                                InventoryHotbarSlot { index: c },
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: px(slot_left),
                                                    top: px(slot_top),
                                                    width: px(18.0 * GUI_SCALE),
                                                    height: px(18.0 * GUI_SCALE),
                                                    display: Display::Flex,
                                                    justify_content:
                                                        JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    border: UiRect::all(px(2.0)),
                                                    border_radius: BorderRadius::all(
                                                        px(2.0),
                                                    ),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::NONE),
                                                BorderColor::all(Color::NONE),
                                            ))
                                            .with_children(|slot| {
                                                let num_font = make_font(11.0);
                                                slot.spawn((
                                                    Text::new(format!("{}", c + 1)),
                                                    num_font,
                                                    TextColor(Color::srgba(
                                                        0.90, 0.90, 0.90, 0.75,
                                                    )),
                                                    text_shadow_default(),
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        top: px(0.0),
                                                        left: px(7.0),
                                                        ..default()
                                                    },
                                                ));

                                                slot.spawn((
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
                                            });
                                    }

                                    // Scrollbar Track & Thumb (shifted 1px left and 5px down)
                                    let track_height = 305.0; // fits trough nicely with top 33.0
                                    let thumb_height = 45.0;
                                    let max_travel = track_height - thumb_height; // 260.0
                                    let max_scroll =
                                        max_filtered_scroll(filtered.len());
                                    let thumb_top = if max_scroll > 0 {
                                        (start_row as f32 / max_scroll as f32) * max_travel
                                    } else {
                                        0.0
                                    };

                                    right_card
                                        .spawn((
                                            Button,
                                            InventoryScrollTrack,
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: px(470.0), // moved 1px left from 471.0
                                                top: px(33.0),   // shifted down 5px from 28.0
                                                width: px(36.0),
                                                height: px(track_height),
                                                ..default()
                                            },
                                            BackgroundColor(Color::NONE),
                                        ))
                                        .with_children(|track| {
                                            track.spawn((
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
                                                    width: px(36.0),
                                                    height: px(thumb_height),
                                                    ..default()
                                                },
                                            ));
                                        });
                                });
                        }
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
    mut tab_state: ResMut<InventoryTab>,
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
    search_query.is_focused = false;
    *tab_state = InventoryTab::Player;

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
    search_query: Res<CreativeSearchQuery>,
    app_font: Option<Res<AppFont>>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        *tab_state = match *tab_state {
            InventoryTab::Creative => InventoryTab::Player,
            InventoryTab::Player => InventoryTab::Creative,
        };
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
    search_query: Res<CreativeSearchQuery>,
    app_font: Option<Res<AppFont>>,
    mut tab_button_query: Query<
        (&Interaction, &InventoryTabButton, &mut BackgroundColor, &mut BorderColor),
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
fn handle_creative_search_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    tab_state: Res<InventoryTab>,
    mut search_query: ResMut<CreativeSearchQuery>,
    mut scroll_state: ResMut<InventoryScrollState>,
    mut search_text_query: Query<(&mut Text, &mut TextColor), With<CreativeSearchText>>,
    mut search_bar_query: Query<
        (&Interaction, &mut BorderColor),
        With<CreativeSearchBar>,
    >,
    mut palette_query: Query<&mut InventoryPaletteSlot>,
    mut slot_icon_query: Query<(&InventoryPaletteSlotIcon, &mut ImageNode, &mut Visibility)>,
    icons: Res<BlockIcons>,
) {
    if *tab_state != InventoryTab::Creative {
        return;
    }

    // Clicking search bar focuses it; clicking outside search bar unfocuses it
    for (interaction, mut border) in &mut search_bar_query {
        if *interaction == Interaction::Pressed {
            search_query.is_focused = true;
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
        }
    }

    if mouse.just_pressed(MouseButton::Left)
        && !search_bar_query
            .iter()
            .any(|(i, _)| *i == Interaction::Hovered || *i == Interaction::Pressed)
    {
        search_query.is_focused = false;
        for (_, mut border) in &mut search_bar_query {
            *border = BorderColor::all(Color::NONE);
        }
    }

    let mut changed = false;

    if keyboard.just_pressed(KeyCode::Backspace) && search_query.query.pop().is_some() {
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::Escape) && search_query.is_focused {
        search_query.is_focused = false;
        for (_, mut border) in &mut search_bar_query {
            *border = BorderColor::all(Color::NONE);
        }
    }

    let keys = [
        (KeyCode::KeyA, 'a'),
        (KeyCode::KeyB, 'b'),
        (KeyCode::KeyC, 'c'),
        (KeyCode::KeyD, 'd'),
        (KeyCode::KeyE, 'e'),
        (KeyCode::KeyF, 'f'),
        (KeyCode::KeyG, 'g'),
        (KeyCode::KeyH, 'h'),
        (KeyCode::KeyI, 'i'),
        (KeyCode::KeyJ, 'j'),
        (KeyCode::KeyK, 'k'),
        (KeyCode::KeyL, 'l'),
        (KeyCode::KeyM, 'm'),
        (KeyCode::KeyN, 'n'),
        (KeyCode::KeyO, 'o'),
        (KeyCode::KeyP, 'p'),
        (KeyCode::KeyQ, 'q'),
        (KeyCode::KeyR, 'r'),
        (KeyCode::KeyS, 's'),
        (KeyCode::KeyT, 't'),
        (KeyCode::KeyU, 'u'),
        (KeyCode::KeyV, 'v'),
        (KeyCode::KeyW, 'w'),
        (KeyCode::KeyX, 'x'),
        (KeyCode::KeyY, 'y'),
        (KeyCode::KeyZ, 'z'),
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
        (KeyCode::Space, ' '),
    ];

    if search_query.is_focused {
        for (key, ch) in keys {
            if keyboard.just_pressed(key) && search_query.query.len() < 20 {
                search_query.query.push(ch);
                changed = true;
            }
        }
    }

    if changed {
        scroll_state.scroll_row = 0;
        let filtered = filtered_creative_blocks(&search_query.query);

        // Update search text
        for (mut text, mut color) in &mut search_text_query {
            if search_query.query.is_empty() {
                text.0 = "Search".to_string();
                color.0 = Color::srgba(0.70, 0.70, 0.75, 0.60);
            } else {
                text.0 = search_query.query.clone();
                color.0 = Color::srgb(0.95, 0.95, 0.95);
            }
        }

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
    track_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryScrollTrack>>,
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

    let thumb_height: f32 = 45.0; // 15 * 3

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
    let left_just_released = mouse.just_released(MouseButton::Left);

    for track_interaction in &track_interaction_query {
        if *track_interaction == Interaction::Pressed && left_pressed {
            scroll_state.is_dragging_thumb = true;
        }
    }

    for thumb_interaction in &thumb_query {
        if *thumb_interaction == Interaction::Pressed && left_pressed {
            scroll_state.is_dragging_thumb = true;
        }
    }

    // Direct bounding-box check to guarantee clicking on track/thumb starts drag
    if left_just_pressed
        && let (Some(window), Some((transform, computed))) =
            (window_query.iter().next(), track_query.iter().next())
        && let Some(cursor_pos) = window.cursor_position()
    {
        let center = transform.translation().truncate();
        let half = computed.size() * 0.5;
        let track_rect = Rect::from_corners(center - half, center + half);
        if track_rect.contains(cursor_pos) {
            scroll_state.is_dragging_thumb = true;
        }
    }

    if left_just_released {
        scroll_state.is_dragging_thumb = false;
    }

    let track_data = track_query.iter().next();
    let track_height = track_data
        .map(|(_, computed)| computed.size().y)
        .unwrap_or(305.0);
    let max_travel = (track_height - thumb_height).max(1.0);

    if scroll_state.is_dragging_thumb
        && let (Some(window), Some((transform, computed))) =
            (window_query.iter().next(), track_data)
        && let Some(cursor_pos) = window.cursor_position()
    {
        let track_center_y = transform.translation().y;
        let half_h = computed.size().y * 0.5;
        let track_top_screen = track_center_y - half_h;
        let track_cursor_y = cursor_pos.y - track_top_screen;

        let clamped_y = (track_cursor_y - thumb_height * 0.5).clamp(0.0, max_travel);

        // Smoothly position thumb under cursor during drag
        for mut thumb_node in &mut thumb_node_query {
            thumb_node.top = px(clamped_y);
        }

        let fraction = if max_travel > 0.0 { clamped_y / max_travel } else { 0.0 };
        let target_row = (fraction * max_scroll as f32).round() as usize;

        if target_row != scroll_state.scroll_row {
            scroll_state.scroll_row = target_row.min(max_scroll);
            row_changed = true;
        }
    } else if row_changed {
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
        (With<InventoryPaletteSlot>, Without<InventoryHotbarSlot>, Without<ArmorSlotUi>),
    >,
    mut slot_icon_query: Query<(&InventoryPaletteSlotIcon, &mut ImageNode, &mut Visibility)>,
    mut hotbar_slot_query: Query<
        (
            &Interaction,
            &InventoryHotbarSlot,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        (With<InventoryHotbarSlot>, Without<InventoryPaletteSlot>, Without<ArmorSlotUi>),
    >,
    mut armor_slot_query: Query<
        (&Interaction, &ArmorSlotUi, &mut BorderColor, &mut BackgroundColor),
        With<ArmorSlotUi>,
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
        if !card_rect.contains(cursor_pos) && let Some(held) = held_item.voxel {
            if *tab_state == InventoryTab::Player {
                player_inv.add_item(held);
            }
            held_item.voxel = None;
        }
    }

    // 2. Right-click deselect if not hovering over any hotbar slot, palette slot, or armor slot
    let any_hotbar_hovered = hotbar_slot_query
        .iter()
        .any(|(i, _, _, _)| *i == Interaction::Hovered || *i == Interaction::Pressed);
    let any_palette_hovered = palette_query
        .iter()
        .any(|(i, _, _, _)| *i == Interaction::Hovered || *i == Interaction::Pressed);
    let any_armor_hovered = armor_slot_query
        .iter()
        .any(|(i, _, _, _)| *i == Interaction::Hovered || *i == Interaction::Pressed);

    if right_just_pressed
        && !any_hotbar_hovered
        && !any_palette_hovered
        && !any_armor_hovered
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

    // 3. Process Armor Slots (visual placeholders only; reject item placement)
    for (interaction, _, mut border, mut bg) in &mut armor_slot_query {
        let is_hovered = *interaction == Interaction::Hovered;
        let is_pressed = *interaction == Interaction::Pressed;

        if is_hovered || is_pressed {
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.10));
            // Do NOT accept any item drops or placements into armor slots!
        } else {
            *border = BorderColor::all(Color::NONE);
            *bg = BackgroundColor(Color::NONE);
        }
    }

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
    fn test_filtered_blocks() {
        let stone_blocks = filtered_creative_blocks("stone");
        assert!(!stone_blocks.is_empty());
        assert!(stone_blocks.contains(&Voxel::Stone));
        assert!(stone_blocks.contains(&Voxel::Cobblestone));

        let empty = filtered_creative_blocks("");
        assert_eq!(empty.len(), AVAILABLE_BLOCKS.len());
    }

    #[test]
    fn test_armor_slot_types() {
        assert_eq!(ARMOR_SLOT_TYPES.len(), 5);
        assert_eq!(ARMOR_SLOT_TYPES[0].name(), "Helmet");
        assert_eq!(ARMOR_SLOT_TYPES[4].name(), "Boots");
    }

    #[test]
    fn test_shift_click_palette_transfer_logic() {
        let mut hotbar = Hotbar {
            slots: [None; HOTBAR_SLOT_COUNT],
            active_slot: 0,
        };

        let target_1 = hotbar.slots.iter().position(|s| s.is_none()).unwrap();
        assert_eq!(target_1, 0);
        hotbar.slots[target_1] = Some(Voxel::Grass);

        let target_2 = hotbar.slots.iter().position(|s| s.is_none()).unwrap();
        assert_eq!(target_2, 1);
        hotbar.slots[target_2] = Some(Voxel::Stone);

        for i in 2..HOTBAR_SLOT_COUNT {
            hotbar.slots[i] = Some(Voxel::Dirt);
        }

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

        assert!(card_rect.contains(center));
        assert!(card_rect.contains(Vec2::new(640.0 + half.x - 5.0, 360.0)));
        assert!(!card_rect.contains(Vec2::new(100.0, 100.0)));
        assert!(!card_rect.contains(Vec2::new(1200.0, 700.0)));
    }

    #[test]
    fn test_tab_toggle_state() {
        let mut tab = InventoryTab::default();
        assert_eq!(tab, InventoryTab::Player);

        tab = InventoryTab::Creative;
        assert_eq!(tab, InventoryTab::Creative);

        tab = InventoryTab::Player;
        assert_eq!(tab, InventoryTab::Player);
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

        let item = player_inv.get(0).unwrap();
        if let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none()) {
            hotbar.slots[empty_idx] = Some(item);
            player_inv.set(0, None);
        }
        assert_eq!(player_inv.get(0), None);
        assert_eq!(hotbar.slots[0], Some(Voxel::Cobblestone));

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

