use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, window::PrimaryWindow};

use crate::{
    core::{AppFont, FontSource, text_shadow_default},
    gameplay::BlockIcons,
    player::hotbar::{HOTBAR_SLOT_COUNT, Hotbar},
    player::inventory::{PLAYER_INVENTORY_COLS, PLAYER_INVENTORY_ROWS, PLAYER_INVENTORY_SLOTS, PlayerInventory},
    world::Voxel,
};

use super::{HeldInventoryItem, MenuState};

pub const GUI_SCALE: f32 = 2.0;

pub const WINDOW_WIDTH: f32 = 416.0 * GUI_SCALE;   // 832 px
pub const WINDOW_HEIGHT: f32 = 256.0 * GUI_SCALE;  // 512 px

pub const CATALOG_COLS: usize = 9;
pub const CATALOG_VISIBLE_ROWS: usize = 10;
#[allow(dead_code)]
pub const CATALOG_VISIBLE_SLOTS: usize = CATALOG_COLS * CATALOG_VISIBLE_ROWS; // 90 slots

pub const INVENTORY_SLOT_X_COORDS: [f32; 9] = [
    124.0, 143.0, 162.0, 181.0, 200.0, 219.0, 238.0, 257.0, 276.0,
];
pub const INVENTORY_ROW_Y_COORDS: [f32; 4] = [148.0, 166.0, 184.0, 202.0];
pub const HOTBAR_ROW_Y: f32 = 227.0;

pub const CATALOG_ROW_Y_COORDS: [f32; 10] = [
    43.0, 61.0, 79.0, 97.0, 115.0, 133.0, 151.0, 169.0, 187.0, 205.0,
];

pub const AVAILABLE_BLOCKS: [Voxel; 720] = Voxel::ALL;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreativeCategory {
    #[default]
    Rocks,        // B2 (132, 8)
    Trees,        // B3 (146, 8)
    Soils,        // B4 (160, 8)
    Ores,         // B5 (174, 8)
    Caverns,      // B6 (188, 8)
    Oceans,       // B7 (202, 8)
    Organics,     // B8 (216, 8)
    Decorations,  // B9 (230, 8)
    Mushrooms,    // B10 (244, 8)
    Vegetation,   // B11 (258, 8)
    Fluids,       // B12 (272, 8)
}

impl CreativeCategory {
    pub const ALL: [CreativeCategory; 11] = [
        CreativeCategory::Rocks,
        CreativeCategory::Trees,
        CreativeCategory::Soils,
        CreativeCategory::Ores,
        CreativeCategory::Caverns,
        CreativeCategory::Oceans,
        CreativeCategory::Organics,
        CreativeCategory::Decorations,
        CreativeCategory::Mushrooms,
        CreativeCategory::Vegetation,
        CreativeCategory::Fluids,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Rocks => "Rocks & Geology",
            Self::Trees => "Trees & Timber",
            Self::Soils => "Soils & Terrain",
            Self::Ores => "Ores & Minerals",
            Self::Caverns => "Caverns & Crystals",
            Self::Oceans => "Oceans & Marine",
            Self::Organics => "Organics & Fauna",
            Self::Decorations => "Decorations & Architecture",
            Self::Mushrooms => "Mycology & Fungi",
            Self::Vegetation => "Vegetation & Flora",
            Self::Fluids => "Fluids & Systems",
        }
    }

    pub fn icon_voxel(self) -> Voxel {
        match self {
            Self::Rocks => Voxel::Dolomite,
            Self::Trees => Voxel::OakLog,
            Self::Soils => Voxel::LoamDirt,
            Self::Ores => Voxel::OreCoalRaw,
            Self::Caverns => Voxel::Limestone,
            Self::Oceans => Voxel::SandCommon,
            Self::Organics => Voxel::BoneBlock,
            Self::Decorations => Voxel::BrickRed,
            Self::Mushrooms => Voxel::FungiAmanita,
            Self::Vegetation => Voxel::Seagrass,
            Self::Fluids => Voxel::LiquidWater,
        }
    }

    pub fn matches(self, voxel: Voxel) -> bool {
        let id = voxel.id();
        match self {
            Self::Rocks => {
                id.starts_with("rock_")
                    || id.starts_with("stone_")
                    || id.contains("dolomite")
                    || id.contains("granite")
                    || id.contains("basalt")
                    || id.contains("andesite")
                    || id.contains("diorite")
                    || id.contains("slate")
                    || id.contains("sandstone")
                    || id.contains("limestone")
                    || id.contains("marble")
                    || id.contains("obsidian")
                    || id.contains("cobblestone")
            }
            Self::Trees => {
                id.starts_with("wood_")
                    || id.starts_with("leaves_")
                    || id.contains("log")
                    || id.contains("plank")
                    || id.contains("bark")
                    || id.contains("branch")
                    || id.contains("sapling")
            }
            Self::Soils => {
                id.starts_with("soil_")
                    || id.starts_with("sand_")
                    || id.starts_with("gravel_")
                    || id.contains("dirt")
                    || id.contains("grass")
                    || id.contains("mud")
                    || id.contains("clay")
                    || id.contains("peat")
                    || id.contains("loam")
                    || id.contains("laterite")
            }
            Self::Ores => {
                id.starts_with("ore_")
                    || id.starts_with("metal_")
                    || id.contains("coal")
                    || id.contains("copper")
                    || id.contains("iron")
                    || id.contains("gold")
                    || id.contains("gem_")
                    || id.contains("crystal")
            }
            Self::Caverns => {
                id.starts_with("cavern_")
                    || id.starts_with("dripstone_")
                    || id.starts_with("frost_")
                    || id.contains("stalactite")
                    || id.contains("stalagmite")
                    || id.contains("permafrost")
            }
            Self::Oceans => {
                id.starts_with("ocean_")
                    || id.starts_with("coral_")
                    || id.starts_with("sponge_")
                    || id.contains("reef")
                    || id.contains("seaweed")
                    || id.contains("kelp")
                    || id.contains("marine")
            }
            Self::Organics => {
                id.starts_with("organic_")
                    || id.starts_with("bone_")
                    || id.contains("flesh")
                    || id.contains("bale")
                    || id.contains("carcass")
                    || id.contains("amber")
                    || id.contains("slime")
                    || id.contains("honey")
            }
            Self::Decorations => {
                id.starts_with("deco_")
                    || id.starts_with("brick_")
                    || id.starts_with("glass_")
                    || id.starts_with("lantern_")
                    || id.starts_with("torch_")
                    || id.contains("carpet")
                    || id.contains("fence")
                    || id.contains("gate")
                    || id.contains("chain")
                    || id.contains("pillar")
                    || id.contains("barrel")
                    || id.contains("pot")
            }
            Self::Mushrooms => {
                id.starts_with("fungi_")
                    || id.starts_with("mushroom_")
                    || id.starts_with("spore_")
                    || id.contains("mycelium")
                    || id.contains("shroom")
            }
            Self::Vegetation => {
                id.starts_with("plant_")
                    || id.starts_with("flower_")
                    || id.starts_with("crop_")
                    || id.contains("flora")
                    || id.contains("fern")
                    || id.contains("vine")
                    || id.contains("bush")
                    || id.contains("wildflower")
                    || id.contains("reed")
                    || id.contains("bamboo")
                    || id.contains("cactus")
            }
            Self::Fluids => {
                id.starts_with("liquid_")
                    || id.starts_with("fluid_")
                    || id.starts_with("debug_")
                    || id.starts_with("null_")
                    || id.contains("water")
                    || id.contains("lava")
                    || id.contains("acid")
                    || id.contains("magma")
            }
        }
    }
}

pub fn filtered_creative_blocks(category: CreativeCategory, query: &str) -> Vec<Voxel> {
    let q = query.trim().to_lowercase();
    AVAILABLE_BLOCKS
        .iter()
        .copied()
        .filter(|&v| {
            if v.is_empty() || v == Voxel::Occupied || v == Voxel::WaterOccupied {
                return false;
            }
            if !q.is_empty() {
                format!("{:?}", v).to_lowercase().contains(&q)
                    || v.label().to_lowercase().contains(&q)
                    || v.id().to_lowercase().contains(&q)
            } else {
                category.matches(v)
            }
        })
        .collect()
}

pub fn total_filtered_rows(filtered_count: usize) -> usize {
    filtered_count
        .div_ceil(CATALOG_COLS)
        .max(CATALOG_VISIBLE_ROWS)
}

pub fn max_filtered_scroll(filtered_count: usize) -> usize {
    total_filtered_rows(filtered_count).saturating_sub(CATALOG_VISIBLE_ROWS)
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
    #[allow(dead_code)]
    pub is_dragging_thumb: bool,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CreativeSearchQuery {
    pub query: String,
    pub is_focused: bool,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveCreativeCategory(pub CreativeCategory);

#[derive(Component)]
pub struct CreativeSearchBar;

#[derive(Component)]
pub struct CreativeSearchText;

#[derive(Component)]
pub struct InventoryMenuEntity;

#[derive(Component)]
pub struct InventoryMenuRoot;

#[derive(Component)]
pub struct InventoryCard;

#[derive(Component)]
pub struct InventoryToggleTabButton;

#[derive(Component)]
pub struct CategoryTabButton {
    pub category: CreativeCategory,
}

#[derive(Component)]
pub struct CategoryTabSelectionDock;

#[derive(Component)]
pub struct InventoryCatalogSlot {
    #[allow(dead_code)]
    pub slot_index: usize,
    pub voxel: Option<Voxel>,
}

#[derive(Component)]
pub struct InventoryCatalogSlotIcon {
    #[allow(dead_code)]
    pub slot_index: usize,
}

#[derive(Component)]
pub struct PlayerStorageSlot {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct PlayerStorageSlotIcon {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct InventoryOffhandSlot;

#[derive(Component)]
pub struct InventoryUtilitySlot;

#[derive(Component)]
pub struct InventoryOffhandIcon;

#[derive(Component)]
pub struct InventoryUtilityIcon;

#[derive(Component)]
pub struct InventoryHotbarSlot {
    pub index: usize,
}

#[derive(Component)]
pub struct InventoryHotbarSlotIcon {
    pub index: usize,
}

#[derive(Component)]
pub struct SlotHoverHighlight;

#[derive(Component)]
pub struct InventoryScrollTrack;

#[derive(Component)]
pub struct InventoryScrollThumb;

#[derive(Component)]
pub struct InventoryTooltipBox;

#[derive(Component)]
pub struct InventoryTooltipText;

#[derive(Component)]
pub enum InventoryAction {
    Sort,
    Trash,
    Options,
    Lock,
}

pub struct InventoryMenuPlugin;

impl Plugin for InventoryMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryScrollState>()
            .init_resource::<InventoryTab>()
            .init_resource::<CreativeSearchQuery>()
            .init_resource::<ActiveCreativeCategory>()
            .add_systems(OnEnter(MenuState::Inventory), spawn_inventory_menu)
            .add_systems(OnExit(MenuState::Inventory), despawn_inventory_menu)
            .add_systems(
                Update,
                (
                    handle_inventory_tab_key,
                    handle_creative_toggle_interaction,
                    handle_category_tab_interaction,
                    handle_creative_search_input,
                    handle_inventory_scroll,
                    handle_inventory_action_buttons,
                    handle_inventory_slot_interaction,
                    update_inventory_tooltip,
                    sync_inventory_hotbar_icons,
                )
                    .chain()
                    .run_if(in_state(MenuState::Inventory)),
            );
    }
}

fn spawn_inventory_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    tab_state: Res<InventoryTab>,
    active_cat: Res<ActiveCreativeCategory>,
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
        active_cat.0,
        &scroll_state,
        &search_query,
        app_font.as_ref().map(|f| f.source()),
    );
}

fn despawn_inventory_menu(mut commands: Commands, query: Query<Entity, With<InventoryMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn build_inventory_ui(
    commands: &mut Commands,
    asset_server: &AssetServer,
    hotbar: &Hotbar,
    player_inv: &PlayerInventory,
    icons: &BlockIcons,
    current_tab: InventoryTab,
    active_cat: CreativeCategory,
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

    let closed_tex = asset_server.load("textures/interfaces/containers/inventory_creative_closed.png");
    let open_tex = asset_server.load("textures/interfaces/containers/inventory_creative_open.png");
    let scroller_tex = asset_server.load("textures/interfaces/containers/inventory_creative_scroller.png");
    let tab_select_tex = asset_server.load("textures/interfaces/containers/inventory_creative_tab_selection.png");
    let slot_hover_tex = asset_server.load("textures/interfaces/containers/inventory_slot_hover.png");
    let special_hover_tex = asset_server.load("textures/interfaces/containers/inventory_slot_special_hover.png");
    let btn_util_tex = asset_server.load("textures/interfaces/containers/button_util.png");

    let shield_overlay = asset_server.load("textures/interfaces/containers/slots/shield.png");
    let quiver_overlay = asset_server.load("textures/interfaces/containers/slots/quiver.png");

    let title_font = make_font(12.0 * GUI_SCALE);

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.60)),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            // Main Central Panel (416 x 256 px scaled to 832 x 512 px)
            backdrop
                .spawn((
                    InventoryMenuEntity,
                    InventoryCard,
                    ImageNode {
                        image: if current_tab == InventoryTab::Creative {
                            open_tex.clone()
                        } else {
                            closed_tex.clone()
                        },
                        ..default()
                    },
                    Node {
                        width: px(WINDOW_WIDTH),
                        height: px(WINDOW_HEIGHT),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                ))
                .with_children(|card| {
                    match current_tab {
                        InventoryTab::Player => {
                            // ==========================================
                            // 1. INVENTORY_CREATIVE_CLOSED
                            // ==========================================

                            // Creative Toggle Tab B1: (118, 116), 10x10 px
                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                InventoryToggleTabButton,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(118.0 * GUI_SCALE),
                                    top: px(116.0 * GUI_SCALE),
                                    width: px(10.0 * GUI_SCALE),
                                    height: px(10.0 * GUI_SCALE),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ));

                            // Title Header Plate B2: (121, 135), 60x8 px
                            card.spawn((
                                InventoryMenuEntity,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(121.0 * GUI_SCALE),
                                    top: px(134.0 * GUI_SCALE),
                                    width: px(60.0 * GUI_SCALE),
                                    height: px(8.0 * GUI_SCALE),
                                    display: Display::Flex,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ))
                            .with_children(|header| {
                                header.spawn((
                                    InventoryMenuEntity,
                                    Text::new("Inventory"),
                                    title_font.clone(),
                                    TextColor(Color::srgb(0.90, 0.90, 0.95)),
                                    text_shadow_default(),
                                ));
                            });

                            // Action Buttons B3..B6: (268, 278, 288, 298, 135), 8x8 px
                            let actions = [
                                (268.0, InventoryAction::Sort),
                                (278.0, InventoryAction::Trash),
                                (288.0, InventoryAction::Options),
                                (298.0, InventoryAction::Lock),
                            ];

                            for (btn_x, action) in actions {
                                card.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    action,
                                    ImageNode {
                                        image: btn_util_tex.clone(),
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(btn_x * GUI_SCALE),
                                        top: px(135.0 * GUI_SCALE),
                                        width: px(8.0 * GUI_SCALE),
                                        height: px(8.0 * GUI_SCALE),
                                        ..default()
                                    },
                                ));
                            }

                            // Card A: Player Character & Armor Slots (Visual Placeholders)
                            // Armor slots A3..A8 (14x14 px)
                            let armor_slots = [
                                (25.0, 210.0, "helmet.png"),
                                (42.0, 210.0, "chestplate.png"),
                                (59.0, 210.0, "leggings.png"),
                                (25.0, 226.0, "boots.png"),
                                (42.0, 226.0, "gloves.png"),
                                (59.0, 226.0, "necklace.png"),
                            ];

                            for (ax, ay, overlay_name) in armor_slots {
                                card.spawn((
                                    InventoryMenuEntity,
                                    ImageNode {
                                        image: asset_server.load(format!("textures/interfaces/containers/slots/{overlay_name}")),
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(ax * GUI_SCALE),
                                        top: px(ay * GUI_SCALE),
                                        width: px(14.0 * GUI_SCALE),
                                        height: px(14.0 * GUI_SCALE),
                                        ..default()
                                    },
                                ));
                            }

                            // Card B: 36 Main Inventory Slots B7..B42 (4 rows of 9)
                            for row in 0..PLAYER_INVENTORY_ROWS {
                                for col in 0..PLAYER_INVENTORY_COLS {
                                    let slot_idx = row * PLAYER_INVENTORY_COLS + col;
                                    let slot_x = INVENTORY_SLOT_X_COORDS[col];
                                    let slot_y = INVENTORY_ROW_Y_COORDS[row];

                                    let voxel = player_inv.slots.get(slot_idx).copied().flatten();
                                    let icon_handle = voxel
                                        .map(|v| icons.get(v))
                                        .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                                    let icon_vis = if voxel.is_some() {
                                        Visibility::Visible
                                    } else {
                                        Visibility::Hidden
                                    };

                                    card.spawn((
                                        InventoryMenuEntity,
                                        Button,
                                        PlayerStorageSlot { slot_index: slot_idx },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(slot_x * GUI_SCALE),
                                            top: px(slot_y * GUI_SCALE),
                                            width: px(16.0 * GUI_SCALE),
                                            height: px(16.0 * GUI_SCALE),
                                            display: Display::Flex,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                    ))
                                    .with_children(|slot| {
                                        // Item Icon
                                        slot.spawn((
                                            InventoryMenuEntity,
                                            PlayerStorageSlotIcon { slot_index: slot_idx },
                                            ImageNode {
                                                image: icon_handle,
                                                ..default()
                                            },
                                            Node {
                                                width: px(16.0 * GUI_SCALE),
                                                height: px(16.0 * GUI_SCALE),
                                                ..default()
                                            },
                                            icon_vis,
                                        ));

                                        // Slot Hover Highlight Overlay (16x16 px)
                                        slot.spawn((
                                            InventoryMenuEntity,
                                            SlotHoverHighlight,
                                            ImageNode {
                                                image: slot_hover_tex.clone(),
                                                ..default()
                                            },
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: px(0.0),
                                                top: px(0.0),
                                                width: px(16.0 * GUI_SCALE),
                                                height: px(16.0 * GUI_SCALE),
                                                ..default()
                                            },
                                            ZIndex(10),
                                            Visibility::Hidden,
                                        ));
                                    });
                                }
                            }

                            // Card B: Off-hand Slot B43 at (104, 228), 14x14 px
                            let offhand_voxel = player_inv.offhand_slot;
                            let offhand_img = offhand_voxel
                                .map(|v| icons.get(v))
                                .unwrap_or_else(|| shield_overlay.clone());

                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                InventoryOffhandSlot,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(104.0 * GUI_SCALE),
                                    top: px(228.0 * GUI_SCALE),
                                    width: px(14.0 * GUI_SCALE),
                                    height: px(14.0 * GUI_SCALE),
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|slot| {
                                slot.spawn((
                                    InventoryMenuEntity,
                                    InventoryOffhandIcon,
                                    ImageNode {
                                        image: offhand_img,
                                        ..default()
                                    },
                                    Node {
                                        width: px(14.0 * GUI_SCALE),
                                        height: px(14.0 * GUI_SCALE),
                                        ..default()
                                    },
                                ));

                                slot.spawn((
                                    InventoryMenuEntity,
                                    SlotHoverHighlight,
                                    ImageNode {
                                        image: special_hover_tex.clone(),
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(0.0),
                                        top: px(0.0),
                                        width: px(14.0 * GUI_SCALE),
                                        height: px(14.0 * GUI_SCALE),
                                        ..default()
                                    },
                                    ZIndex(10),
                                    Visibility::Hidden,
                                ));
                            });

                            // Card B: Quick Utility Slot B53 at (298, 228), 14x14 px
                            let util_voxel = player_inv.utility_slot;
                            let util_img = util_voxel
                                .map(|v| icons.get(v))
                                .unwrap_or_else(|| quiver_overlay.clone());

                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                InventoryUtilitySlot,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(298.0 * GUI_SCALE),
                                    top: px(228.0 * GUI_SCALE),
                                    width: px(14.0 * GUI_SCALE),
                                    height: px(14.0 * GUI_SCALE),
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|slot| {
                                slot.spawn((
                                    InventoryMenuEntity,
                                    InventoryUtilityIcon,
                                    ImageNode {
                                        image: util_img,
                                        ..default()
                                    },
                                    Node {
                                        width: px(14.0 * GUI_SCALE),
                                        height: px(14.0 * GUI_SCALE),
                                        ..default()
                                    },
                                ));

                                slot.spawn((
                                    InventoryMenuEntity,
                                    SlotHoverHighlight,
                                    ImageNode {
                                        image: special_hover_tex.clone(),
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(0.0),
                                        top: px(0.0),
                                        width: px(14.0 * GUI_SCALE),
                                        height: px(14.0 * GUI_SCALE),
                                        ..default()
                                    },
                                    ZIndex(10),
                                    Visibility::Hidden,
                                ));
                            });

                            // Card B: Mirrored 9 Hotbar Slots B44..B52 at (124..276, 227), 16x16 px
                            for c in 0..HOTBAR_SLOT_COUNT {
                                let slot_voxel = hotbar.slots[c];
                                let icon_handle = slot_voxel
                                    .map(|v| icons.get(v))
                                    .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                                let icon_vis = if slot_voxel.is_some() {
                                    Visibility::Visible
                                } else {
                                    Visibility::Hidden
                                };

                                let slot_x = INVENTORY_SLOT_X_COORDS[c];

                                card.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    InventoryHotbarSlot { index: c },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(slot_x * GUI_SCALE),
                                        top: px(HOTBAR_ROW_Y * GUI_SCALE),
                                        width: px(16.0 * GUI_SCALE),
                                        height: px(16.0 * GUI_SCALE),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                ))
                                .with_children(|slot| {
                                    slot.spawn((
                                        InventoryMenuEntity,
                                        InventoryHotbarSlotIcon { index: c },
                                        ImageNode {
                                            image: icon_handle,
                                            ..default()
                                        },
                                        Node {
                                            width: px(16.0 * GUI_SCALE),
                                            height: px(16.0 * GUI_SCALE),
                                            ..default()
                                        },
                                        icon_vis,
                                    ));

                                    slot.spawn((
                                        InventoryMenuEntity,
                                        SlotHoverHighlight,
                                        ImageNode {
                                            image: slot_hover_tex.clone(),
                                            ..default()
                                        },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(0.0),
                                            top: px(0.0),
                                            width: px(16.0 * GUI_SCALE),
                                            height: px(16.0 * GUI_SCALE),
                                            ..default()
                                        },
                                        ZIndex(10),
                                        Visibility::Hidden,
                                    ));

                                    // Slot index number (1 through 9)
                                    let num_font = make_font(8.0 * GUI_SCALE);
                                    slot.spawn((
                                        InventoryMenuEntity,
                                        Text::new(format!("{}", c + 1)),
                                        num_font,
                                        TextColor(Color::srgba(0.85, 0.88, 0.92, 0.80)),
                                        text_shadow_default(),
                                        Node {
                                            position_type: PositionType::Absolute,
                                            top: px(1.0 * GUI_SCALE),
                                            left: px(2.0 * GUI_SCALE),
                                            ..default()
                                        },
                                        ZIndex(15),
                                    ));
                                });
                            }
                        }
                        InventoryTab::Creative => {
                            // ==========================================
                            // 2. INVENTORY_CREATIVE_OPEN
                            // ==========================================

                            // Creative Toggle Tab B1: (118, 8), 10x10 px (click to collapse)
                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                InventoryToggleTabButton,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(118.0 * GUI_SCALE),
                                    top: px(8.0 * GUI_SCALE),
                                    width: px(10.0 * GUI_SCALE),
                                    height: px(10.0 * GUI_SCALE),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ));

                            // Active Tab Selection Dock: docked over active category tab
                            let active_idx = CreativeCategory::ALL
                                .iter()
                                .position(|&c| c == active_cat)
                                .unwrap_or(0);
                            let active_tab_x = 132.0 + active_idx as f32 * 14.0;

                            card.spawn((
                                InventoryMenuEntity,
                                CategoryTabSelectionDock,
                                ImageNode {
                                    image: tab_select_tex.clone(),
                                    ..default()
                                },
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px((active_tab_x - 2.0) * GUI_SCALE),
                                    top: px(6.0 * GUI_SCALE),
                                    width: px(16.0 * GUI_SCALE),
                                    height: px(16.0 * GUI_SCALE),
                                    ..default()
                                },
                                ZIndex(15),
                            ));

                            // 11 Category Tabs B2..B12: (132..272, 8), 12x12 px with 3D block icons
                            for (idx, &cat) in CreativeCategory::ALL.iter().enumerate() {
                                let tab_x = 132.0 + idx as f32 * 14.0;
                                let tab_icon = icons.get(cat.icon_voxel());

                                card.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    CategoryTabButton { category: cat },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(tab_x * GUI_SCALE),
                                        top: px(8.0 * GUI_SCALE),
                                        width: px(12.0 * GUI_SCALE),
                                        height: px(12.0 * GUI_SCALE),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                    ZIndex(20),
                                ))
                                .with_children(|tab| {
                                    tab.spawn((
                                        InventoryMenuEntity,
                                        ImageNode {
                                            image: tab_icon,
                                            ..default()
                                        },
                                        Node {
                                            width: px(12.0 * GUI_SCALE),
                                            height: px(12.0 * GUI_SCALE),
                                            ..default()
                                        },
                                    ));
                                });
                            }

                            // Category Title Header Plate B13: (121, 27), 60x8 px
                            card.spawn((
                                InventoryMenuEntity,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(121.0 * GUI_SCALE),
                                    top: px(27.0 * GUI_SCALE),
                                    width: px(70.0 * GUI_SCALE),
                                    height: px(8.0 * GUI_SCALE),
                                    display: Display::Flex,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::left(px(2.0 * GUI_SCALE)),
                                    ..default()
                                },
                            ))
                            .with_children(|header| {
                                header.spawn((
                                    InventoryMenuEntity,
                                    Text::new(active_cat.title()),
                                    make_font(7.5 * GUI_SCALE),
                                    TextColor(Color::srgb(0.92, 0.92, 0.96)),
                                    text_shadow_default(),
                                ));
                            });

                            // Live Search Box B14: (223, 27), 72x8 px
                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                CreativeSearchBar,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(223.0 * GUI_SCALE),
                                    top: px(27.0 * GUI_SCALE),
                                    width: px(72.0 * GUI_SCALE),
                                    height: px(8.0 * GUI_SCALE),
                                    display: Display::Flex,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::left(px(2.0 * GUI_SCALE)),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|sb| {
                                let search_font = make_font(7.0 * GUI_SCALE);
                                let is_empty = search_query.query.is_empty();
                                let display_text = if !search_query.is_focused {
                                    if is_empty {
                                        "Search...".to_string()
                                    } else {
                                        search_query.query.clone()
                                    }
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
                                        Color::srgba(0.65, 0.68, 0.75, 0.60)
                                    } else {
                                        Color::srgb(0.95, 0.95, 0.98)
                                    }),
                                    text_shadow_default(),
                                ));
                            });

                            // 90 Creative Catalog Slots B16..B105: 10 rows of 9 (16x16 px)
                            let filtered = filtered_creative_blocks(active_cat, &search_query.query);
                            let start_row = scroll_state.scroll_row;

                            for row in 0..CATALOG_VISIBLE_ROWS {
                                for col in 0..CATALOG_COLS {
                                    let slot_idx = row * CATALOG_COLS + col;
                                    let block_idx = (start_row + row) * CATALOG_COLS + col;
                                    let voxel = filtered.get(block_idx).copied();

                                    let icon_handle = voxel
                                        .map(|v| icons.get(v))
                                        .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                                    let icon_vis = if voxel.is_some() {
                                        Visibility::Visible
                                    } else {
                                        Visibility::Hidden
                                    };

                                    let slot_x = INVENTORY_SLOT_X_COORDS[col];
                                    let slot_y = CATALOG_ROW_Y_COORDS[row];

                                    card.spawn((
                                        InventoryMenuEntity,
                                        Button,
                                        InventoryCatalogSlot {
                                            slot_index: slot_idx,
                                            voxel,
                                        },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(slot_x * GUI_SCALE),
                                            top: px(slot_y * GUI_SCALE),
                                            width: px(16.0 * GUI_SCALE),
                                            height: px(16.0 * GUI_SCALE),
                                            display: Display::Flex,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                    ))
                                    .with_children(|slot| {
                                        slot.spawn((
                                            InventoryMenuEntity,
                                            InventoryCatalogSlotIcon { slot_index: slot_idx },
                                            ImageNode {
                                                image: icon_handle,
                                                ..default()
                                            },
                                            Node {
                                                width: px(16.0 * GUI_SCALE),
                                                height: px(16.0 * GUI_SCALE),
                                                ..default()
                                            },
                                            icon_vis,
                                        ));

                                        slot.spawn((
                                            InventoryMenuEntity,
                                            SlotHoverHighlight,
                                            ImageNode {
                                                image: slot_hover_tex.clone(),
                                                ..default()
                                            },
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: px(0.0),
                                                top: px(0.0),
                                                width: px(16.0 * GUI_SCALE),
                                                height: px(16.0 * GUI_SCALE),
                                                ..default()
                                            },
                                            ZIndex(10),
                                            Visibility::Hidden,
                                        ));
                                    });
                                }
                            }

                            // Scrollbar Track & Thumb B15: (298, 40..205), 6x166 px
                            let track_height = 166.0 * GUI_SCALE;
                            let thumb_height = 16.0 * GUI_SCALE;
                            let max_travel = track_height - thumb_height;
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
                                    left: px(296.0 * GUI_SCALE),
                                    top: px(40.0 * GUI_SCALE),
                                    width: px(10.0 * GUI_SCALE),
                                    height: px(track_height),
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
                                        image: scroller_tex.clone(),
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(0.0),
                                        top: px(thumb_top),
                                        width: px(10.0 * GUI_SCALE),
                                        height: px(thumb_height),
                                        ..default()
                                    },
                                ));
                            });

                            // Bottom Hotbar Row B106..B116 (mirrored hotbar accessible in creative)
                            // Off-hand at (104, 228), 14x14 px
                            let offhand_voxel = player_inv.offhand_slot;
                            let offhand_img = offhand_voxel
                                .map(|v| icons.get(v))
                                .unwrap_or_else(|| shield_overlay.clone());

                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                InventoryOffhandSlot,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(104.0 * GUI_SCALE),
                                    top: px(228.0 * GUI_SCALE),
                                    width: px(14.0 * GUI_SCALE),
                                    height: px(14.0 * GUI_SCALE),
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|slot| {
                                slot.spawn((
                                    InventoryMenuEntity,
                                    InventoryOffhandIcon,
                                    ImageNode {
                                        image: offhand_img,
                                        ..default()
                                    },
                                    Node {
                                        width: px(14.0 * GUI_SCALE),
                                        height: px(14.0 * GUI_SCALE),
                                        ..default()
                                    },
                                ));

                                slot.spawn((
                                    InventoryMenuEntity,
                                    SlotHoverHighlight,
                                    ImageNode {
                                        image: special_hover_tex.clone(),
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(0.0),
                                        top: px(0.0),
                                        width: px(14.0 * GUI_SCALE),
                                        height: px(14.0 * GUI_SCALE),
                                        ..default()
                                    },
                                    ZIndex(10),
                                    Visibility::Hidden,
                                ));
                            });

                            // 9 Hotbar slots at (124..276, 227), 16x16 px
                            for c in 0..HOTBAR_SLOT_COUNT {
                                let slot_voxel = hotbar.slots[c];
                                let icon_handle = slot_voxel
                                    .map(|v| icons.get(v))
                                    .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                                let icon_vis = if slot_voxel.is_some() {
                                    Visibility::Visible
                                } else {
                                    Visibility::Hidden
                                };

                                let slot_x = INVENTORY_SLOT_X_COORDS[c];

                                card.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    InventoryHotbarSlot { index: c },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(slot_x * GUI_SCALE),
                                        top: px(HOTBAR_ROW_Y * GUI_SCALE),
                                        width: px(16.0 * GUI_SCALE),
                                        height: px(16.0 * GUI_SCALE),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                ))
                                .with_children(|slot| {
                                    slot.spawn((
                                        InventoryMenuEntity,
                                        InventoryHotbarSlotIcon { index: c },
                                        ImageNode {
                                            image: icon_handle,
                                            ..default()
                                        },
                                        Node {
                                            width: px(16.0 * GUI_SCALE),
                                            height: px(16.0 * GUI_SCALE),
                                            ..default()
                                        },
                                        icon_vis,
                                    ));

                                    slot.spawn((
                                        InventoryMenuEntity,
                                        SlotHoverHighlight,
                                        ImageNode {
                                            image: slot_hover_tex.clone(),
                                            ..default()
                                        },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(0.0),
                                            top: px(0.0),
                                            width: px(16.0 * GUI_SCALE),
                                            height: px(16.0 * GUI_SCALE),
                                            ..default()
                                        },
                                        ZIndex(10),
                                        Visibility::Hidden,
                                    ));

                                    let num_font = make_font(8.0 * GUI_SCALE);
                                    slot.spawn((
                                        InventoryMenuEntity,
                                        Text::new(format!("{}", c + 1)),
                                        num_font,
                                        TextColor(Color::srgba(0.85, 0.88, 0.92, 0.80)),
                                        text_shadow_default(),
                                        Node {
                                            position_type: PositionType::Absolute,
                                            top: px(1.0 * GUI_SCALE),
                                            left: px(2.0 * GUI_SCALE),
                                            ..default()
                                        },
                                        ZIndex(15),
                                    ));
                                });
                            }

                            // Quick Utility Slot at (298, 228), 14x14 px
                            let util_voxel = player_inv.utility_slot;
                            let util_img = util_voxel
                                .map(|v| icons.get(v))
                                .unwrap_or_else(|| quiver_overlay.clone());

                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                InventoryUtilitySlot,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(298.0 * GUI_SCALE),
                                    top: px(228.0 * GUI_SCALE),
                                    width: px(14.0 * GUI_SCALE),
                                    height: px(14.0 * GUI_SCALE),
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|slot| {
                                slot.spawn((
                                    InventoryMenuEntity,
                                    InventoryUtilityIcon,
                                    ImageNode {
                                        image: util_img,
                                        ..default()
                                    },
                                    Node {
                                        width: px(14.0 * GUI_SCALE),
                                        height: px(14.0 * GUI_SCALE),
                                        ..default()
                                    },
                                ));

                                slot.spawn((
                                    InventoryMenuEntity,
                                    SlotHoverHighlight,
                                    ImageNode {
                                        image: special_hover_tex.clone(),
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(0.0),
                                        top: px(0.0),
                                        width: px(14.0 * GUI_SCALE),
                                        height: px(14.0 * GUI_SCALE),
                                        ..default()
                                    },
                                    ZIndex(10),
                                    Visibility::Hidden,
                                ));
                            });
                        }
                    }
                });

            // Floating hover tooltip for items
            let tip_font = make_font(14.0);
            backdrop
                .spawn((
                    InventoryMenuEntity,
                    InventoryTooltipBox,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        padding: UiRect::axes(px(8.0), px(4.0)),
                        border: UiRect::all(px(1.5)),
                        border_radius: BorderRadius::all(px(3.0)),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.07, 0.12, 0.96)),
                    BorderColor::all(Color::srgba(0.38, 0.42, 0.58, 0.90)),
                    ZIndex(850),
                    Visibility::Hidden,
                ))
                .with_children(|tip| {
                    tip.spawn((
                        InventoryMenuEntity,
                        InventoryTooltipText,
                        Text::new(""),
                        tip_font,
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        text_shadow_default(),
                    ));
                });
        });
}

fn handle_inventory_tab_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tab_state: ResMut<InventoryTab>,
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    asset_server: Res<AssetServer>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    active_cat: Res<ActiveCreativeCategory>,
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
            active_cat.0,
            &scroll_state,
            &search_query,
            app_font.as_ref().map(|f| f.source()),
        );
    }
}

fn handle_creative_toggle_interaction(
    mouse: Res<ButtonInput<MouseButton>>,
    mut tab_state: ResMut<InventoryTab>,
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    asset_server: Res<AssetServer>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    active_cat: Res<ActiveCreativeCategory>,
    scroll_state: Res<InventoryScrollState>,
    mut search_query: ResMut<CreativeSearchQuery>,
    app_font: Option<Res<AppFont>>,
    toggle_query: Query<&Interaction, (With<InventoryToggleTabButton>, Changed<Interaction>)>,
) {
    for interaction in &toggle_query {
        if *interaction == Interaction::Pressed || mouse.just_pressed(MouseButton::Left) && *interaction == Interaction::Hovered {
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
                active_cat.0,
                &scroll_state,
                &search_query,
                app_font.as_ref().map(|f| f.source()),
            );
            break;
        }
    }
}

fn handle_category_tab_interaction(
    mouse: Res<ButtonInput<MouseButton>>,
    mut active_cat: ResMut<ActiveCreativeCategory>,
    mut scroll_state: ResMut<InventoryScrollState>,
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    asset_server: Res<AssetServer>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    tab_state: Res<InventoryTab>,
    search_query: Res<CreativeSearchQuery>,
    app_font: Option<Res<AppFont>>,
    cat_query: Query<(&Interaction, &CategoryTabButton)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    for (interaction, cat_btn) in &cat_query {
        if *interaction == Interaction::Pressed || *interaction == Interaction::Hovered {
            if active_cat.0 != cat_btn.category {
                active_cat.0 = cat_btn.category;
                scroll_state.scroll_row = 0;
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
                    active_cat.0,
                    &scroll_state,
                    &search_query,
                    app_font.as_ref().map(|f| f.source()),
                );
                break;
            }
        }
    }
}

fn handle_inventory_action_buttons(
    mut player_inv: ResMut<PlayerInventory>,
    mut held_item: ResMut<HeldInventoryItem>,
    action_query: Query<(&Interaction, &InventoryAction)>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    for (interaction, action) in &action_query {
        if *interaction == Interaction::Pressed || *interaction == Interaction::Hovered {
            match action {
                InventoryAction::Sort => {
                    let mut items: Vec<Voxel> = player_inv.slots.iter().filter_map(|&s| s).collect();
                    items.sort_by_key(|v| v.id());
                    player_inv.slots = [None; PLAYER_INVENTORY_SLOTS];
                    for (i, v) in items.into_iter().enumerate() {
                        if i < PLAYER_INVENTORY_SLOTS {
                            player_inv.slots[i] = Some(v);
                        }
                    }
                }
                InventoryAction::Trash => {
                    held_item.voxel = None;
                }
                InventoryAction::Options => {}
                InventoryAction::Lock => {}
            }
        }
    }
}

fn handle_creative_search_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut search_query: ResMut<CreativeSearchQuery>,
    mut scroll_state: ResMut<InventoryScrollState>,
    search_bar_query: Query<&Interaction, With<CreativeSearchBar>>,
    mut text_query: Query<&mut Text, With<CreativeSearchText>>,
    mut commands: Commands,
    menu_query: Query<Entity, With<InventoryMenuRoot>>,
    asset_server: Res<AssetServer>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    tab_state: Res<InventoryTab>,
    active_cat: Res<ActiveCreativeCategory>,
    app_font: Option<Res<AppFont>>,
) {
    let mut changed = false;

    if mouse.just_pressed(MouseButton::Left) {
        for interaction in &search_bar_query {
            if *interaction == Interaction::Pressed || *interaction == Interaction::Hovered {
                search_query.is_focused = true;
                search_query.cursor = search_query.query.len();
                changed = true;
            } else {
                search_query.is_focused = false;
                changed = true;
            }
        }
    }

    if search_query.is_focused {
        for key in keyboard.get_just_pressed() {
            let ch = match key {
                KeyCode::KeyA => Some('a'),
                KeyCode::KeyB => Some('b'),
                KeyCode::KeyC => Some('c'),
                KeyCode::KeyD => Some('d'),
                KeyCode::KeyE => Some('e'),
                KeyCode::KeyF => Some('f'),
                KeyCode::KeyG => Some('g'),
                KeyCode::KeyH => Some('h'),
                KeyCode::KeyI => Some('i'),
                KeyCode::KeyJ => Some('j'),
                KeyCode::KeyK => Some('k'),
                KeyCode::KeyL => Some('l'),
                KeyCode::KeyM => Some('m'),
                KeyCode::KeyN => Some('n'),
                KeyCode::KeyO => Some('o'),
                KeyCode::KeyP => Some('p'),
                KeyCode::KeyQ => Some('q'),
                KeyCode::KeyR => Some('r'),
                KeyCode::KeyS => Some('s'),
                KeyCode::KeyT => Some('t'),
                KeyCode::KeyU => Some('u'),
                KeyCode::KeyV => Some('v'),
                KeyCode::KeyW => Some('w'),
                KeyCode::KeyX => Some('x'),
                KeyCode::KeyY => Some('y'),
                KeyCode::KeyZ => Some('z'),
                KeyCode::Space => Some(' '),
                KeyCode::Backspace => {
                    if !search_query.query.is_empty() {
                        search_query.query.pop();
                        search_query.cursor = search_query.query.len();
                        scroll_state.scroll_row = 0;
                        changed = true;
                    }
                    None
                }
                KeyCode::Escape | KeyCode::Enter => {
                    search_query.is_focused = false;
                    changed = true;
                    None
                }
                _ => None,
            };

            if let Some(c) = ch {
                if search_query.query.len() < 20 {
                    search_query.query.push(c);
                    search_query.cursor = search_query.query.len();
                    scroll_state.scroll_row = 0;
                    changed = true;
                }
            }
        }
    }

    if changed {
        for mut text in &mut text_query {
            let display_text = if !search_query.is_focused && search_query.query.is_empty() {
                "Search...".to_string()
            } else if search_query.is_focused && search_query.query.is_empty() {
                "|".to_string()
            } else {
                search_query.query.clone()
            };
            text.0 = display_text;
        }

        // Rebuild catalog grid slots
        for entity in &menu_query {
            commands.entity(entity).despawn();
        }
        build_inventory_ui(
            &mut commands,
            &asset_server,
            &hotbar,
            &player_inv,
            &icons,
            *tab_state,
            active_cat.0,
            &scroll_state,
            &search_query,
            app_font.as_ref().map(|f| f.source()),
        );
    }
}

fn handle_inventory_scroll(
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut scroll_state: ResMut<InventoryScrollState>,
    tab_state: Res<InventoryTab>,
    active_cat: Res<ActiveCreativeCategory>,
    search_query: Res<CreativeSearchQuery>,
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    asset_server: Res<AssetServer>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    app_font: Option<Res<AppFont>>,
) {
    if *tab_state != InventoryTab::Creative {
        return;
    }

    let delta = mouse_scroll.delta.y;
    if delta.abs() > 0.05 {
        let filtered = filtered_creative_blocks(active_cat.0, &search_query.query);
        let max_scroll = max_filtered_scroll(filtered.len());
        let old_row = scroll_state.scroll_row;

        if delta > 0.0 {
            scroll_state.scroll_row = scroll_state.scroll_row.saturating_sub(1);
        } else {
            scroll_state.scroll_row = (scroll_state.scroll_row + 1).min(max_scroll);
        }

        if scroll_state.scroll_row != old_row {
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
                active_cat.0,
                &scroll_state,
                &search_query,
                app_font.as_ref().map(|f| f.source()),
            );
        }
    }
}

fn handle_inventory_slot_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut held_item: ResMut<HeldInventoryItem>,
    mut hotbar: ResMut<Hotbar>,
    mut player_inv: ResMut<PlayerInventory>,
    tab_state: Res<InventoryTab>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    card_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryCard>>,
    mut storage_slot_query: Query<
        (&Interaction, &PlayerStorageSlot, &Children),
        (With<PlayerStorageSlot>, Without<InventoryHotbarSlot>, Without<InventoryCatalogSlot>),
    >,
    mut hotbar_slot_query: Query<
        (&Interaction, &InventoryHotbarSlot, &Children),
        (With<InventoryHotbarSlot>, Without<PlayerStorageSlot>, Without<InventoryCatalogSlot>),
    >,
    mut catalog_slot_query: Query<
        (&Interaction, &InventoryCatalogSlot, &Children),
        (With<InventoryCatalogSlot>, Without<PlayerStorageSlot>, Without<InventoryHotbarSlot>),
    >,
    mut offhand_query: Query<(&Interaction, &Children), (With<InventoryOffhandSlot>, Without<InventoryUtilitySlot>)>,
    mut utility_query: Query<(&Interaction, &Children), (With<InventoryUtilitySlot>, Without<InventoryOffhandSlot>)>,
    mut hover_overlay_query: Query<&mut Visibility, With<SlotHoverHighlight>>,
) {
    let is_shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let left_click = mouse.just_pressed(MouseButton::Left);
    let right_click = mouse.just_pressed(MouseButton::Right);

    // 1. Deselect held item if clicking outside the card
    if (left_click || right_click)
        && let (Some(window), Some((transform, computed))) =
            (window_query.iter().next(), card_query.iter().next())
        && let Some(cursor_pos) = window.cursor_position()
    {
        let center = transform.translation().truncate();
        let half = computed.size() * 0.5;
        let card_rect = Rect::from_corners(center - half, center + half);
        if !card_rect.contains(cursor_pos) {
            if let Some(held) = held_item.voxel {
                player_inv.add_item(held);
                held_item.voxel = None;
            }
        }
    }

    // 2. Main Player Storage Slots
    for (interaction, slot, children) in &mut storage_slot_query {
        let is_hovered = *interaction == Interaction::Hovered;
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                *vis = if is_hovered { Visibility::Visible } else { Visibility::Hidden };
            }
        }

        if is_hovered && (left_click || right_click) {
            let slot_idx = slot.slot_index;
            let current_item = player_inv.slots[slot_idx];

            if is_shift && left_click {
                if let Some(item) = current_item {
                    // Transfer to hotbar
                    if let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none()) {
                        hotbar.slots[empty_idx] = Some(item);
                        player_inv.slots[slot_idx] = None;
                    } else {
                        let active = hotbar.active_slot;
                        let displaced = hotbar.slots[active];
                        hotbar.slots[active] = Some(item);
                        player_inv.slots[slot_idx] = displaced;
                    }
                }
            } else if left_click {
                let temp = held_item.voxel;
                held_item.voxel = current_item;
                player_inv.slots[slot_idx] = temp;
            } else if right_click {
                if held_item.voxel.is_none() && current_item.is_some() {
                    held_item.voxel = current_item;
                    player_inv.slots[slot_idx] = None;
                } else if let Some(held) = held_item.voxel {
                    if current_item.is_none() {
                        player_inv.slots[slot_idx] = Some(held);
                    } else {
                        let temp = held_item.voxel;
                        held_item.voxel = current_item;
                        player_inv.slots[slot_idx] = temp;
                    }
                }
            }
        }
    }

    // 3. Mirrored Hotbar Slots
    for (interaction, slot, children) in &mut hotbar_slot_query {
        let is_hovered = *interaction == Interaction::Hovered;
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                *vis = if is_hovered { Visibility::Visible } else { Visibility::Hidden };
            }
        }

        if is_hovered && (left_click || right_click) {
            let h_idx = slot.index;
            let current_item = hotbar.slots[h_idx];

            if is_shift && left_click && *tab_state == InventoryTab::Player {
                if let Some(item) = current_item {
                    // Transfer to storage
                    if player_inv.add_item(item) {
                        hotbar.slots[h_idx] = None;
                    }
                }
            } else if left_click {
                let temp = held_item.voxel;
                held_item.voxel = current_item;
                hotbar.slots[h_idx] = temp;
            } else if right_click {
                if held_item.voxel.is_none() && current_item.is_some() {
                    held_item.voxel = current_item;
                    hotbar.slots[h_idx] = None;
                } else if held_item.voxel.is_some() {
                    let temp = held_item.voxel;
                    held_item.voxel = current_item;
                    hotbar.slots[h_idx] = temp;
                }
            }
        }
    }

    // 4. Off-hand Slot
    for (interaction, children) in &mut offhand_query {
        let is_hovered = *interaction == Interaction::Hovered;
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                *vis = if is_hovered { Visibility::Visible } else { Visibility::Hidden };
            }
        }

        if is_hovered && left_click {
            let temp = held_item.voxel;
            held_item.voxel = player_inv.offhand_slot;
            player_inv.offhand_slot = temp;
        }
    }

    // 5. Quick Utility Slot
    for (interaction, children) in &mut utility_query {
        let is_hovered = *interaction == Interaction::Hovered;
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                *vis = if is_hovered { Visibility::Visible } else { Visibility::Hidden };
            }
        }

        if is_hovered && left_click {
            let temp = held_item.voxel;
            held_item.voxel = player_inv.utility_slot;
            player_inv.utility_slot = temp;
        }
    }

    // 6. Creative Catalog Slots
    for (interaction, slot, children) in &mut catalog_slot_query {
        let is_hovered = *interaction == Interaction::Hovered;
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                *vis = if is_hovered { Visibility::Visible } else { Visibility::Hidden };
            }
        }

        if is_hovered && (left_click || right_click) {
            if let Some(voxel) = slot.voxel {
                if is_shift {
                    // Shift click directly inserts into hotbar
                    if let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none()) {
                        hotbar.slots[empty_idx] = Some(voxel);
                    } else {
                        let active = hotbar.active_slot;
                        hotbar.slots[active] = Some(voxel);
                    }
                } else {
                    held_item.voxel = Some(voxel);
                }
            }
        }
    }
}

fn sync_inventory_hotbar_icons(
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    asset_server: Res<AssetServer>,
    mut hotbar_icons: Query<(&InventoryHotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
    mut storage_icons: Query<
        (&PlayerStorageSlotIcon, &mut ImageNode, &mut Visibility),
        Without<InventoryHotbarSlotIcon>,
    >,
    mut offhand_icons: Query<
        &mut ImageNode,
        (With<InventoryOffhandIcon>, Without<InventoryHotbarSlotIcon>, Without<PlayerStorageSlotIcon>),
    >,
    mut utility_icons: Query<
        &mut ImageNode,
        (
            With<InventoryUtilityIcon>,
            Without<InventoryHotbarSlotIcon>,
            Without<PlayerStorageSlotIcon>,
            Without<InventoryOffhandIcon>,
        ),
    >,
) {
    if hotbar.is_changed() {
        for (icon, mut img, mut vis) in &mut hotbar_icons {
            if let Some(voxel) = hotbar.slots[icon.index] {
                img.image = icons.get(voxel);
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }

    if player_inv.is_changed() {
        for (icon, mut img, mut vis) in &mut storage_icons {
            if let Some(voxel) = player_inv.slots.get(icon.slot_index).copied().flatten() {
                img.image = icons.get(voxel);
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }

        for mut img in &mut offhand_icons {
            if let Some(voxel) = player_inv.offhand_slot {
                img.image = icons.get(voxel);
            } else {
                img.image = asset_server.load("textures/interfaces/containers/slots/shield.png");
            }
        }

        for mut img in &mut utility_icons {
            if let Some(voxel) = player_inv.utility_slot {
                img.image = icons.get(voxel);
            } else {
                img.image = asset_server.load("textures/interfaces/containers/slots/quiver.png");
            }
        }
    }
}

fn update_inventory_tooltip(
    window_query: Query<&Window, With<PrimaryWindow>>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    storage_query: Query<(&Interaction, &PlayerStorageSlot)>,
    hotbar_query: Query<(&Interaction, &InventoryHotbarSlot)>,
    catalog_query: Query<(&Interaction, &InventoryCatalogSlot)>,
    offhand_query: Query<&Interaction, With<InventoryOffhandSlot>>,
    utility_query: Query<&Interaction, With<InventoryUtilitySlot>>,
    mut tooltip_box: Query<(&mut Node, &mut Visibility), With<InventoryTooltipBox>>,
    mut tooltip_text: Query<&mut Text, With<InventoryTooltipText>>,
) {
    let Ok((mut box_node, mut box_vis)) = tooltip_box.single_mut() else {
        return;
    };
    let Ok(mut text) = tooltip_text.single_mut() else {
        return;
    };
    let Ok(window) = window_query.single() else {
        return;
    };

    let mut hovered_voxel = None;

    for (interaction, slot) in &storage_query {
        if *interaction == Interaction::Hovered {
            hovered_voxel = player_inv.slots.get(slot.slot_index).copied().flatten();
            break;
        }
    }

    if hovered_voxel.is_none() {
        for (interaction, slot) in &hotbar_query {
            if *interaction == Interaction::Hovered {
                hovered_voxel = hotbar.slots.get(slot.index).copied().flatten();
                break;
            }
        }
    }

    if hovered_voxel.is_none() {
        for (interaction, slot) in &catalog_query {
            if *interaction == Interaction::Hovered {
                hovered_voxel = slot.voxel;
                break;
            }
        }
    }

    if hovered_voxel.is_none() {
        for interaction in &offhand_query {
            if *interaction == Interaction::Hovered {
                hovered_voxel = player_inv.offhand_slot;
                break;
            }
        }
    }

    if hovered_voxel.is_none() {
        for interaction in &utility_query {
            if *interaction == Interaction::Hovered {
                hovered_voxel = player_inv.utility_slot;
                break;
            }
        }
    }

    if let (Some(voxel), Some(cursor_pos)) = (hovered_voxel, window.cursor_position()) {
        text.0 = voxel.label().to_string();
        box_node.left = px(cursor_pos.x + 16.0);
        box_node.top = px(cursor_pos.y - 12.0);
        *box_vis = Visibility::Visible;
    } else {
        *box_vis = Visibility::Hidden;
    }
}
