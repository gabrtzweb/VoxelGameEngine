use bevy::{
    input::mouse::AccumulatedMouseScroll, picking::hover::Hovered, prelude::*, ui::Pressed,
    ui_widgets::Button, window::PrimaryWindow,
};

use crate::{
    core::{AppFont, FontSource, text_shadow_default},
    gameplay::BlockIcons,
    player::hotbar::{HOTBAR_SLOT_COUNT, Hotbar},
    player::inventory::{
        PLAYER_INVENTORY_COLS, PLAYER_INVENTORY_ROWS, PLAYER_INVENTORY_SLOTS, PlayerInventory,
    },
    world::Voxel,
};

use super::{HeldInventoryItem, MenuState};

pub const GUI_SCALE: f32 = 3.0;

pub const WINDOW_WIDTH: f32 = 416.0 * GUI_SCALE; // 1248 px
pub const WINDOW_HEIGHT: f32 = 256.0 * GUI_SCALE; // 768 px

pub const CATALOG_COLS: usize = 9;
pub const CATALOG_VISIBLE_ROWS: usize = 10;
#[allow(dead_code)]
pub const CATALOG_VISIBLE_SLOTS: usize = CATALOG_COLS * CATALOG_VISIBLE_ROWS; // 90 slots

pub const INVENTORY_SLOT_X_COORDS: [f32; 9] = [
    124.0, 143.0, 162.0, 181.0, 200.0, 219.0, 238.0, 257.0, 276.0,
];
pub const INVENTORY_ROW_Y_COORDS: [f32; 4] = [151.0, 169.0, 187.0, 205.0];
pub const HOTBAR_ROW_Y: f32 = 227.0;

pub const CATALOG_ROW_Y_COORDS: [f32; 10] = [
    43.0, 61.0, 79.0, 97.0, 115.0, 133.0, 151.0, 169.0, 187.0, 205.0,
];

pub const AVAILABLE_BLOCKS: [Voxel; 720] = Voxel::ALL;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreativeCategory {
    #[default]
    All, // B2 (136, 7)
    Rocks,       // B3
    Soils,       // B4
    Trees,       // B5
    Ores,        // B6
    Oceans,      // B7
    Vegetation,  // B8
    Organics,    // B9
    Decorations, // B10
    Lighting,    // B11
    Fluids,      // B12
}

impl CreativeCategory {
    pub const ALL: [CreativeCategory; 11] = [
        CreativeCategory::All,
        CreativeCategory::Rocks,
        CreativeCategory::Soils,
        CreativeCategory::Trees,
        CreativeCategory::Ores,
        CreativeCategory::Oceans,
        CreativeCategory::Vegetation,
        CreativeCategory::Organics,
        CreativeCategory::Decorations,
        CreativeCategory::Lighting,
        CreativeCategory::Fluids,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::All => "All Blocks",
            Self::Rocks => "Rocks & Geology",
            Self::Soils => "Soils & Terrain",
            Self::Trees => "Trees & Timber",
            Self::Ores => "Ores, Gems & Crystals",
            Self::Oceans => "Oceans & Corals",
            Self::Vegetation => "Vegetation & Fungi",
            Self::Organics => "Organics & Fauna",
            Self::Decorations => "Decorations",
            Self::Lighting => "Lighting & Utility",
            Self::Fluids => "Fluids & Ice",
        }
    }

    pub fn icon_voxel(self) -> Voxel {
        match self {
            Self::All => Voxel::LoamGrass,
            Self::Rocks => Voxel::Dolomite,
            Self::Soils => Voxel::LoamDirt,
            Self::Trees => Voxel::OakLog,
            Self::Ores => Voxel::OreCoalRaw,
            Self::Oceans => Voxel::BrainCoral,
            Self::Vegetation => Voxel::FungiAmanita,
            Self::Organics => Voxel::FleshBlock,
            Self::Decorations => Voxel::Bricks,
            Self::Lighting => Voxel::EmitTorchFire,
            Self::Fluids => Voxel::LiquidWater,
        }
    }

    /// Classifies a voxel primarily by its texture-name prefix (`rock_`, `soil_`, `tree_`,
    /// `aqua_`, `vege_`, `fungi_`, `bio_`, `organic_`, `nest_`, `deco_`, `emit_`, `util_`,
    /// `liquid_`, `frost_`), with a few id-based overrides for blocks whose texture is
    /// borrowed from their host rock (ores/gems) or that have no texture (speleothems).
    pub fn matches(self, voxel: Voxel) -> bool {
        let id = voxel.id();
        let tex = voxel.texture_name().unwrap_or("");

        // Ores & gems embedded in rocks reuse the host rock texture, so detect them by id.
        let is_ore = id.starts_with("ore_") || id.starts_with("gem_") || id.starts_with("crystal_");
        // Animal burrows reuse soil textures but belong with fauna.
        let is_burrow = id.ends_with("_burrow");
        // Paths are terrain even though their texture lives under deco_.
        let is_path = id.starts_with("path_");
        let is_moss = id.starts_with("moss_");
        // Marine flora (kelp, seagrass, algae...) — vege_ textures that are not coral fans.
        let is_marine_flora =
            tex.starts_with("vege_") && !id.contains("coral") && id != "bone_worms";

        match self {
            Self::All => true,
            Self::Rocks => {
                !is_ore
                    && (tex.starts_with("rock_")
                        || id.starts_with("pebble_")
                        || id.starts_with("dripstone_")
                        || id.starts_with("speleothem_")
                        || id.starts_with("fossil_"))
            }
            Self::Soils => (tex.starts_with("soil_") && !is_burrow) || is_path,
            Self::Trees => tex.starts_with("tree_"),
            Self::Ores => is_ore,
            Self::Oceans => tex.starts_with("aqua_") || tex.starts_with("vege_"),
            Self::Vegetation => tex.starts_with("fungi_") || is_moss || is_marine_flora,
            Self::Organics => {
                tex.starts_with("bio_")
                    || tex.starts_with("organic_")
                    || tex.starts_with("nest_")
                    || is_burrow
            }
            Self::Decorations => tex.starts_with("deco_") && !is_path,
            Self::Lighting => {
                tex.starts_with("emit_")
                    || tex.starts_with("util_")
                    || tex.starts_with("debug_")
                    || tex.starts_with("null_")
            }
            Self::Fluids => tex.starts_with("liquid_") || tex.starts_with("frost_"),
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
pub struct ActionBtnHover;

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

fn despawn_inventory_menu(
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    mut held_item: ResMut<HeldInventoryItem>,
    mut player_inv: ResMut<PlayerInventory>,
    mut hotbar: ResMut<Hotbar>,
    selected_voxel: Option<ResMut<crate::gameplay::SelectedVoxel>>,
) {
    if let Some(held) = held_item.voxel.take()
        && !player_inv.add_item(held)
        && let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none())
    {
        hotbar.slots[empty_idx] = Some(held);
    }
    if let Some(mut sel) = selected_voxel {
        sel.0 = hotbar.slots[hotbar.active_slot];
    }
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

    let closed_tex =
        asset_server.load("textures/interfaces/containers/inventory_creative_closed.png");
    let open_tex = asset_server.load("textures/interfaces/containers/inventory_creative_open.png");
    let scroller_tex =
        asset_server.load("textures/interfaces/containers/inventory_creative_scroller.png");
    let tab_select_tex =
        asset_server.load("textures/interfaces/containers/inventory_creative_tab_selection.png");
    let slot_hover_tex =
        asset_server.load("textures/interfaces/containers/inventory_slot_hover.png");
    let special_hover_tex =
        asset_server.load("textures/interfaces/containers/inventory_slot_special_hover.png");
    let btn_util_hover_tex =
        asset_server.load("textures/interfaces/containers/button_util_hover.png");

    let title_font = make_font(4.5 * GUI_SCALE);
    let num_font = make_font(3.5 * GUI_SCALE);

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
            // Main Central Panel (416 x 256 px scaled to 1248 x 768 px)
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

                            // Creative Toggle Tab B1: (118, 117), 10x10 px
                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                Hovered::default(),
                                InventoryToggleTabButton,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(118.0 * GUI_SCALE),
                                    top: px(117.0 * GUI_SCALE),
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
                                    top: px(135.0 * GUI_SCALE),
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

                            // Action Buttons B3..B6: (251, 263, 275, 287, 135), 8x8 px
                            // The normal state is cleanly pre-rendered into the background plate.
                            // Only button_util_hover is displayed on hover.
                            let actions = [
                                (251.0, InventoryAction::Sort),
                                (263.0, InventoryAction::Trash),
                                (275.0, InventoryAction::Options),
                                (287.0, InventoryAction::Lock),
                            ];

                            for (btn_x, action) in actions {
                                card.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    Hovered::default(),
                                    action,
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(btn_x * GUI_SCALE),
                                        top: px(135.0 * GUI_SCALE),
                                        width: px(8.0 * GUI_SCALE),
                                        height: px(8.0 * GUI_SCALE),
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        InventoryMenuEntity,
                                        ActionBtnHover,
                                        ImageNode {
                                            image: btn_util_hover_tex.clone(),
                                            ..default()
                                        },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(0.0),
                                            top: px(0.0),
                                            width: px(8.0 * GUI_SCALE),
                                            height: px(8.0 * GUI_SCALE),
                                            ..default()
                                        },
                                        Visibility::Hidden,
                                    ));
                                });
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
                                        image: asset_server.load(format!(
                                            "textures/interfaces/containers/slots/{overlay_name}"
                                        )),
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
                            for (row, &slot_y) in INVENTORY_ROW_Y_COORDS
                                .iter()
                                .enumerate()
                                .take(PLAYER_INVENTORY_ROWS)
                            {
                                for (col, &slot_x) in INVENTORY_SLOT_X_COORDS
                                    .iter()
                                    .enumerate()
                                    .take(PLAYER_INVENTORY_COLS)
                                {
                                    let slot_idx = row * PLAYER_INVENTORY_COLS + col;

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
                                        Hovered::default(),
                                        PlayerStorageSlot {
                                            slot_index: slot_idx,
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
                                        // Item Icon
                                        slot.spawn((
                                            InventoryMenuEntity,
                                            PlayerStorageSlotIcon {
                                                slot_index: slot_idx,
                                            },
                                            ImageNode {
                                                image: icon_handle,
                                                ..default()
                                            },
                                            Node {
                                                width: px(16.0 * GUI_SCALE),
                                                height: px(16.0 * GUI_SCALE),
                                                ..default()
                                            },
                                            ZIndex(5),
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
                                            ZIndex(1),
                                            Visibility::Hidden,
                                        ));
                                    });
                                }
                            }

                            // Card B: Off-hand Slot B43 at (104, 228), 14x14 px
                            let offhand_voxel = player_inv.offhand_slot;
                            let offhand_img = offhand_voxel
                                .map(|v| icons.get(v))
                                .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                            let offhand_vis = if offhand_voxel.is_some() {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };

                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                Hovered::default(),
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
                                    ZIndex(5),
                                    offhand_vis,
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
                                    ZIndex(1),
                                    Visibility::Hidden,
                                ));
                            });

                            // Card B: Quick Utility Slot B53 at (298, 228), 14x14 px
                            let util_voxel = player_inv.utility_slot;
                            let util_img = util_voxel
                                .map(|v| icons.get(v))
                                .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                            let util_vis = if util_voxel.is_some() {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };

                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                Hovered::default(),
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
                                    ZIndex(5),
                                    util_vis,
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
                                    ZIndex(1),
                                    Visibility::Hidden,
                                ));
                            });

                            // Card B: Mirrored 9 Hotbar Slots B44..B52 at (124..276, 227), 16x16 px
                            for (c, &slot_x) in INVENTORY_SLOT_X_COORDS
                                .iter()
                                .enumerate()
                                .take(HOTBAR_SLOT_COUNT)
                            {
                                let slot_voxel = hotbar.slots[c];
                                let icon_handle = slot_voxel
                                    .map(|v| icons.get(v))
                                    .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                                let icon_vis = if slot_voxel.is_some() {
                                    Visibility::Visible
                                } else {
                                    Visibility::Hidden
                                };

                                card.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    Hovered::default(),
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
                                        ZIndex(5),
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
                                        ZIndex(1),
                                        Visibility::Hidden,
                                    ));

                                    // Slot index number (1 through 9)
                                    slot.spawn((
                                        InventoryMenuEntity,
                                        Text::new(format!("{}", c + 1)),
                                        num_font.clone(),
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
                                        image: asset_server.load(format!(
                                            "textures/interfaces/containers/slots/{overlay_name}"
                                        )),
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

                            // Creative Toggle Tab B1: (117, 8), 12x11 px (click to collapse)
                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                Hovered::default(),
                                InventoryToggleTabButton,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(117.0 * GUI_SCALE),
                                    top: px(8.0 * GUI_SCALE),
                                    width: px(12.0 * GUI_SCALE),
                                    height: px(11.0 * GUI_SCALE),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ));

                            // Active Tab Selection Dock: docked over active category tab
                            let active_idx = CreativeCategory::ALL
                                .iter()
                                .position(|&c| c == active_cat)
                                .unwrap_or(0);
                            let active_tab_x = 136.0 + active_idx as f32 * 15.0;

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
                                    top: px(5.0 * GUI_SCALE),
                                    width: px(16.0 * GUI_SCALE),
                                    height: px(16.0 * GUI_SCALE),
                                    ..default()
                                },
                                ZIndex(15),
                            ));

                            // 11 Category Tabs B2..B12: (136..286, 7), 12x12 px with 8x8 block icons
                            for (idx, &cat) in CreativeCategory::ALL.iter().enumerate() {
                                let tab_x = 136.0 + idx as f32 * 15.0;
                                let tab_icon = icons.get(cat.icon_voxel());

                                card.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    Hovered::default(),
                                    CategoryTabButton { category: cat },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(tab_x * GUI_SCALE),
                                        top: px(7.0 * GUI_SCALE),
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
                                            width: px(8.0 * GUI_SCALE),
                                            height: px(8.0 * GUI_SCALE),
                                            ..default()
                                        },
                                        ZIndex(21),
                                    ));
                                });
                            }

                            // Category Title Header Plate B13: (121, 27), 98x8 px
                            card.spawn((
                                InventoryMenuEntity,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(121.0 * GUI_SCALE),
                                    top: px(27.0 * GUI_SCALE),
                                    width: px(98.0 * GUI_SCALE),
                                    height: px(8.0 * GUI_SCALE),
                                    display: Display::Flex,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::left(px(2.0 * GUI_SCALE)),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                            ))
                            .with_children(|header| {
                                header.spawn((
                                    InventoryMenuEntity,
                                    Text::new(active_cat.title()),
                                    make_font(3.8 * GUI_SCALE),
                                    TextLayout::no_wrap(),
                                    TextColor(Color::srgb(0.92, 0.92, 0.96)),
                                    text_shadow_default(),
                                ));
                            });

                            // Live Search Box B14: (223, 27), 72x8 px
                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                Hovered::default(),
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
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|sb| {
                                let search_font = make_font(3.8 * GUI_SCALE);
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
                                    format!(
                                        "{}|{}",
                                        &search_query.query[..cur],
                                        &search_query.query[cur..]
                                    )
                                };

                                sb.spawn((
                                    InventoryMenuEntity,
                                    CreativeSearchText,
                                    Text::new(display_text),
                                    search_font,
                                    TextLayout::no_wrap(),
                                    TextColor(if !search_query.is_focused && is_empty {
                                        Color::srgba(0.65, 0.68, 0.75, 0.60)
                                    } else {
                                        Color::srgb(0.95, 0.95, 0.98)
                                    }),
                                    text_shadow_default(),
                                ));
                            });

                            // 90 Creative Catalog Slots B16..B105: 10 rows of 9 (16x16 px)
                            let filtered =
                                filtered_creative_blocks(active_cat, &search_query.query);
                            let start_row = scroll_state.scroll_row;

                            for (row, &slot_y) in CATALOG_ROW_Y_COORDS
                                .iter()
                                .enumerate()
                                .take(CATALOG_VISIBLE_ROWS)
                            {
                                for (col, &slot_x) in INVENTORY_SLOT_X_COORDS
                                    .iter()
                                    .enumerate()
                                    .take(CATALOG_COLS)
                                {
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

                                    card.spawn((
                                        InventoryMenuEntity,
                                        Button,
                                        Hovered::default(),
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
                                            InventoryCatalogSlotIcon {
                                                slot_index: slot_idx,
                                            },
                                            ImageNode {
                                                image: icon_handle,
                                                ..default()
                                            },
                                            Node {
                                                width: px(16.0 * GUI_SCALE),
                                                height: px(16.0 * GUI_SCALE),
                                                ..default()
                                            },
                                            ZIndex(5),
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
                                            ZIndex(1),
                                            Visibility::Hidden,
                                        ));
                                    });
                                }
                            }

                            // Scrollbar Track & Thumb B15: (298, 40..205), 7x166 px
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
                                Hovered::default(),
                                InventoryScrollTrack,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(298.0 * GUI_SCALE),
                                    top: px(40.0 * GUI_SCALE),
                                    width: px(7.0 * GUI_SCALE),
                                    height: px(track_height),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|track| {
                                track.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    Hovered::default(),
                                    InventoryScrollThumb,
                                    ImageNode {
                                        image: scroller_tex.clone(),
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(-2.5 * GUI_SCALE),
                                        top: px(thumb_top),
                                        width: px(12.0 * GUI_SCALE),
                                        height: px(thumb_height),
                                        ..default()
                                    },
                                    ZIndex(10),
                                ));
                            });

                            // Bottom Hotbar Row B106..B116 (mirrored hotbar accessible in creative)
                            // Off-hand at (104, 228), 14x14 px
                            let offhand_voxel = player_inv.offhand_slot;
                            let offhand_img = offhand_voxel
                                .map(|v| icons.get(v))
                                .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                            let offhand_vis = if offhand_voxel.is_some() {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };

                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                Hovered::default(),
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
                                    ZIndex(5),
                                    offhand_vis,
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
                                    ZIndex(1),
                                    Visibility::Hidden,
                                ));
                            });

                            // 9 Hotbar slots at (124..276, 227), 16x16 px
                            for (c, &slot_x) in INVENTORY_SLOT_X_COORDS
                                .iter()
                                .enumerate()
                                .take(HOTBAR_SLOT_COUNT)
                            {
                                let slot_voxel = hotbar.slots[c];
                                let icon_handle = slot_voxel
                                    .map(|v| icons.get(v))
                                    .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                                let icon_vis = if slot_voxel.is_some() {
                                    Visibility::Visible
                                } else {
                                    Visibility::Hidden
                                };

                                card.spawn((
                                    InventoryMenuEntity,
                                    Button,
                                    Hovered::default(),
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
                                        ZIndex(5),
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
                                        ZIndex(1),
                                        Visibility::Hidden,
                                    ));

                                    slot.spawn((
                                        InventoryMenuEntity,
                                        Text::new(format!("{}", c + 1)),
                                        num_font.clone(),
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
                                .unwrap_or_else(|| icons.get(Voxel::Dolomite));
                            let util_vis = if util_voxel.is_some() {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };

                            card.spawn((
                                InventoryMenuEntity,
                                Button,
                                Hovered::default(),
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
                                    ZIndex(5),
                                    util_vis,
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
                                    ZIndex(1),
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
    toggle_query: Query<(&Hovered, Has<Pressed>), With<InventoryToggleTabButton>>,
) {
    for (hovered, is_pressed) in &toggle_query {
        if is_pressed || (mouse.just_pressed(MouseButton::Left) && hovered.get()) {
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
    cat_query: Query<(&Hovered, &CategoryTabButton)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    for (hovered, cat_btn) in &cat_query {
        if hovered.get() && active_cat.0 != cat_btn.category {
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

fn handle_inventory_action_buttons(
    mut player_inv: ResMut<PlayerInventory>,
    mut held_item: ResMut<HeldInventoryItem>,
    action_query: Query<(&Hovered, &InventoryAction, &Children)>,
    mut hover_query: Query<&mut Visibility, With<ActionBtnHover>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let left_click = mouse.just_pressed(MouseButton::Left);

    for (hovered, action, children) in &action_query {
        let is_hovered = hovered.get();
        for child in children.iter() {
            if let Ok(mut vis) = hover_query.get_mut(child) {
                let target = if is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                if *vis != target {
                    *vis = target;
                }
            }
        }

        if is_hovered && left_click {
            match action {
                InventoryAction::Sort => {
                    let mut items: Vec<Voxel> =
                        player_inv.slots.iter().filter_map(|&s| s).collect();
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

#[derive(bevy::ecs::system::SystemParam)]
struct RebuildUiParams<'w, 's> {
    commands: Commands<'w, 's>,
    query: Query<'w, 's, Entity, With<InventoryMenuRoot>>,
    asset_server: Res<'w, AssetServer>,
    hotbar: Res<'w, Hotbar>,
    player_inv: Res<'w, PlayerInventory>,
    icons: Res<'w, BlockIcons>,
    app_font: Option<Res<'w, AppFont>>,
}

fn handle_creative_search_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut search_query: ResMut<CreativeSearchQuery>,
    mut scroll_state: ResMut<InventoryScrollState>,
    search_bar_query: Query<&Hovered, With<CreativeSearchBar>>,
    mut text_query: Query<&mut Text, With<CreativeSearchText>>,
    tab_state: Res<InventoryTab>,
    active_cat: Res<ActiveCreativeCategory>,
    mut ui: RebuildUiParams,
) {
    if *tab_state != InventoryTab::Creative {
        return;
    }

    let mut focus_changed = false;
    let mut query_changed = false;

    if mouse.just_pressed(MouseButton::Left) {
        let is_over_search = search_bar_query.iter().any(|h| h.get());
        if is_over_search {
            if !search_query.is_focused {
                search_query.is_focused = true;
                search_query.cursor = search_query.query.len();
                focus_changed = true;
            }
        } else if search_query.is_focused {
            search_query.is_focused = false;
            focus_changed = true;
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
                        query_changed = true;
                    }
                    None
                }
                KeyCode::Escape | KeyCode::Enter => {
                    search_query.is_focused = false;
                    focus_changed = true;
                    None
                }
                _ => None,
            };

            if let Some(c) = ch
                && search_query.query.len() < 20
            {
                search_query.query.push(c);
                search_query.cursor = search_query.query.len();
                scroll_state.scroll_row = 0;
                query_changed = true;
            }
        }
    }

    if focus_changed || query_changed {
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
    }

    if query_changed {
        // Rebuild catalog grid slots
        for entity in &ui.query {
            ui.commands.entity(entity).despawn();
        }
        build_inventory_ui(
            &mut ui.commands,
            &ui.asset_server,
            &ui.hotbar,
            &ui.player_inv,
            &ui.icons,
            *tab_state,
            active_cat.0,
            &scroll_state,
            &search_query,
            ui.app_font.as_ref().map(|f| f.source()),
        );
    }
}

fn handle_inventory_scroll(
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    tab_state: Res<InventoryTab>,
    active_cat: Res<ActiveCreativeCategory>,
    search_query: Res<CreativeSearchQuery>,
    mut scroll_state: ResMut<InventoryScrollState>,
    icons: Res<BlockIcons>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    card_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryCard>>,
    track_interaction_query: Query<Has<Pressed>, With<InventoryScrollTrack>>,
    thumb_interaction_query: Query<Has<Pressed>, With<InventoryScrollThumb>>,
    mut thumb_node_query: Query<&mut Node, With<InventoryScrollThumb>>,
    mut catalog_slot_query: Query<&mut InventoryCatalogSlot>,
    mut catalog_icon_query: Query<(&InventoryCatalogSlotIcon, &mut ImageNode, &mut Visibility)>,
    ui_scale: Option<Res<bevy::ui::UiScale>>,
) {
    if *tab_state != InventoryTab::Creative {
        return;
    }

    let scale = ui_scale.as_ref().map_or(1.0, |s| s.0);
    let filtered = filtered_creative_blocks(active_cat.0, &search_query.query);
    let max_scroll = max_filtered_scroll(filtered.len());
    let mut row_changed = false;

    let track_height_logical = 166.0 * GUI_SCALE;
    let thumb_height_logical = 16.0 * GUI_SCALE;
    let max_travel_logical = (track_height_logical - thumb_height_logical).max(1.0);

    // 1. Mouse wheel scrolling
    let scroll_y = mouse_scroll.delta.y;
    if scroll_y > 0.05 && scroll_state.scroll_row > 0 {
        scroll_state.scroll_row = scroll_state.scroll_row.saturating_sub(1);
        row_changed = true;
    } else if scroll_y < -0.05 && scroll_state.scroll_row < max_scroll {
        scroll_state.scroll_row = (scroll_state.scroll_row + 1).min(max_scroll);
        row_changed = true;
    }

    // 2. Track & thumb mouse hit detection
    let left_pressed = mouse.pressed(MouseButton::Left);
    let left_just_pressed = mouse.just_pressed(MouseButton::Left);
    let cursor_pos = window_query.iter().next().and_then(|w| w.cursor_position());

    let (top_left, card_w, card_h) = if let Some((card_tf, card_node)) = card_query.iter().next() {
        let size = card_node.size();
        if size.x > 10.0 && size.y > 10.0 {
            let center = card_tf.translation().truncate();
            (center - size * 0.5, size.x, size.y)
        } else if let Some(w) = window_query.iter().next() {
            let w_size = Vec2::new(WINDOW_WIDTH * scale, WINDOW_HEIGHT * scale);
            (
                Vec2::new((w.width() - w_size.x) * 0.5, (w.height() - w_size.y) * 0.5),
                w_size.x,
                w_size.y,
            )
        } else {
            (Vec2::ZERO, WINDOW_WIDTH, WINDOW_HEIGHT)
        }
    } else if let Some(w) = window_query.iter().next() {
        let w_size = Vec2::new(WINDOW_WIDTH * scale, WINDOW_HEIGHT * scale);
        (
            Vec2::new((w.width() - w_size.x) * 0.5, (w.height() - w_size.y) * 0.5),
            w_size.x,
            w_size.y,
        )
    } else {
        (Vec2::ZERO, WINDOW_WIDTH, WINDOW_HEIGHT)
    };

    let eff_scale_x = card_w / 416.0;
    let eff_scale_y = card_h / 256.0;
    let track_screen_x = top_left.x + 298.0 * eff_scale_x;
    let track_screen_y = top_left.y + 40.0 * eff_scale_y;
    let track_screen_w = 7.0 * eff_scale_x;
    let track_screen_h = 166.0 * eff_scale_y;
    let thumb_screen_h = 16.0 * eff_scale_y;
    let max_travel_screen = (track_screen_h - thumb_screen_h).max(1.0);

    let track_hit_rect = Rect::new(
        track_screen_x - 12.0 * scale,
        track_screen_y - 6.0 * scale,
        track_screen_x + track_screen_w + 12.0 * scale,
        track_screen_y + track_screen_h + 6.0 * scale,
    );

    let is_inside_track = cursor_pos.is_some_and(|pos| track_hit_rect.contains(pos));
    let track_pressed =
        track_interaction_query.iter().any(|p| p) || thumb_interaction_query.iter().any(|p| p);

    if left_just_pressed && (is_inside_track || track_pressed) {
        scroll_state.is_dragging_thumb = true;
    }

    if !left_pressed {
        scroll_state.is_dragging_thumb = false;
    }

    if scroll_state.is_dragging_thumb && left_pressed {
        if let Some(pos) = cursor_pos {
            let cursor_rel_y = pos.y - track_screen_y;
            let clamped_y = (cursor_rel_y - thumb_screen_h * 0.5).clamp(0.0, max_travel_screen);
            let fraction = clamped_y / max_travel_screen;
            let thumb_top_logical = fraction * max_travel_logical;

            for mut thumb_node in &mut thumb_node_query {
                thumb_node.top = px(thumb_top_logical);
            }

            let target_row = ((fraction * max_scroll as f32).round() as usize).min(max_scroll);
            if target_row != scroll_state.scroll_row {
                scroll_state.scroll_row = target_row;
                row_changed = true;
            }
        }
    } else {
        // Sync thumb position whenever not actively dragging
        let cur_row = scroll_state.scroll_row;
        let thumb_top_logical = if max_scroll > 0 {
            (cur_row as f32 / max_scroll as f32) * max_travel_logical
        } else {
            0.0
        };

        for mut thumb_node in &mut thumb_node_query {
            thumb_node.top = px(thumb_top_logical);
        }
    }

    // 3. Update catalog slot icons and voxel data directly
    if row_changed {
        let cur_row = scroll_state.scroll_row;
        for mut slot in &mut catalog_slot_query {
            let block_idx = cur_row * CATALOG_COLS + slot.slot_index;
            slot.voxel = filtered.get(block_idx).copied();
        }

        for (icon, mut img, mut vis) in &mut catalog_icon_query {
            let block_idx = cur_row * CATALOG_COLS + icon.slot_index;
            if let Some(&v) = filtered.get(block_idx) {
                *vis = Visibility::Visible;
                img.image = icons.get(v);
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
struct SlotInteractionQueries<'w, 's> {
    storage_slot_query: Query<
        'w,
        's,
        (
            &'static Hovered,
            &'static PlayerStorageSlot,
            &'static Children,
        ),
        (
            With<PlayerStorageSlot>,
            Without<InventoryHotbarSlot>,
            Without<InventoryCatalogSlot>,
        ),
    >,
    hotbar_slot_query: Query<
        'w,
        's,
        (
            &'static Hovered,
            &'static InventoryHotbarSlot,
            &'static Children,
        ),
        (
            With<InventoryHotbarSlot>,
            Without<PlayerStorageSlot>,
            Without<InventoryCatalogSlot>,
        ),
    >,
    catalog_slot_query: Query<
        'w,
        's,
        (
            &'static Hovered,
            &'static InventoryCatalogSlot,
            &'static Children,
        ),
        (
            With<InventoryCatalogSlot>,
            Without<PlayerStorageSlot>,
            Without<InventoryHotbarSlot>,
        ),
    >,
    offhand_query: Query<
        'w,
        's,
        (&'static Hovered, &'static Children),
        (With<InventoryOffhandSlot>, Without<InventoryUtilitySlot>),
    >,
    utility_query: Query<
        'w,
        's,
        (&'static Hovered, &'static Children),
        (With<InventoryUtilitySlot>, Without<InventoryOffhandSlot>),
    >,
    hover_overlay_query: Query<'w, 's, &'static mut Visibility, With<SlotHoverHighlight>>,
}

fn handle_inventory_slot_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut held_item: ResMut<HeldInventoryItem>,
    mut hotbar: ResMut<Hotbar>,
    mut player_inv: ResMut<PlayerInventory>,
    selected_voxel: Option<ResMut<crate::gameplay::SelectedVoxel>>,
    tab_state: Res<InventoryTab>,
    scroll_state: Res<InventoryScrollState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    card_query: Query<(&GlobalTransform, &ComputedNode), With<InventoryCard>>,
    mut dragged_slots: Local<std::collections::HashSet<(u8, usize)>>,
    slots: SlotInteractionQueries,
) {
    let is_shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let left_click = mouse.just_pressed(MouseButton::Left);
    let left_held = mouse.pressed(MouseButton::Left);
    let left_released = mouse.just_released(MouseButton::Left);
    let right_click = mouse.just_pressed(MouseButton::Right);

    if !left_held && !left_released {
        dragged_slots.clear();
    }

    let digit_pressed: Option<usize> = [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ]
    .into_iter()
    .find_map(|(k, idx)| {
        if keyboard.just_pressed(k) {
            Some(idx)
        } else {
            None
        }
    });

    let mut any_interacted = false;

    let SlotInteractionQueries {
        mut storage_slot_query,
        mut hotbar_slot_query,
        mut catalog_slot_query,
        mut offhand_query,
        mut utility_query,
        mut hover_overlay_query,
    } = slots;

    // 1. Main Player Storage Slots
    for (hovered, slot, children) in &mut storage_slot_query {
        let is_hovered = hovered.get();
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                let target = if is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                if *vis != target {
                    *vis = target;
                }
            }
        }

        if is_hovered {
            let slot_idx = slot.slot_index;

            // Number key quick swap with hotbar
            if let Some(h_idx) = digit_pressed {
                any_interacted = true;
                std::mem::swap(&mut hotbar.slots[h_idx], &mut player_inv.slots[slot_idx]);
            } else if is_shift
                && (left_click || (left_held && !dragged_slots.contains(&(0, slot_idx))))
            {
                any_interacted = true;
                dragged_slots.insert((0, slot_idx));
                let current_item = player_inv.slots[slot_idx];
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
                any_interacted = true;
                dragged_slots.insert((0, slot_idx));
                let current_item = player_inv.slots[slot_idx];
                let temp = held_item.voxel;
                held_item.voxel = current_item;
                player_inv.slots[slot_idx] = temp;
            } else if left_released
                && held_item.voxel.is_some()
                && !dragged_slots.contains(&(0, slot_idx))
            {
                any_interacted = true;
                let current_item = player_inv.slots[slot_idx];
                let temp = held_item.voxel;
                held_item.voxel = current_item;
                player_inv.slots[slot_idx] = temp;
            } else if right_click {
                any_interacted = true;
                let current_item = player_inv.slots[slot_idx];
                if held_item.voxel.is_none() && current_item.is_some() {
                    held_item.voxel = current_item;
                    player_inv.slots[slot_idx] = None;
                } else if let Some(held) = held_item.voxel {
                    if current_item.is_none() {
                        player_inv.slots[slot_idx] = Some(held);
                        held_item.voxel = None;
                    } else {
                        let temp = held_item.voxel;
                        held_item.voxel = current_item;
                        player_inv.slots[slot_idx] = temp;
                    }
                }
            }
        }
    }

    // 2. Mirrored Hotbar Slots
    for (hovered, slot, children) in &mut hotbar_slot_query {
        let is_hovered = hovered.get();
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                let target = if is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                if *vis != target {
                    *vis = target;
                }
            }
        }

        if is_hovered {
            let h_idx = slot.index;
            if is_shift
                && *tab_state == InventoryTab::Player
                && (left_click || (left_held && !dragged_slots.contains(&(1, h_idx))))
            {
                any_interacted = true;
                dragged_slots.insert((1, h_idx));
                let current_item = hotbar.slots[h_idx];
                if let Some(item) = current_item {
                    // Transfer to storage
                    if player_inv.add_item(item) {
                        hotbar.slots[h_idx] = None;
                    }
                }
            } else if is_shift
                && *tab_state == InventoryTab::Creative
                && (left_click || (left_held && !dragged_slots.contains(&(1, h_idx))))
            {
                // In Creative tab, Shift+Click or Shift+Drag discards/deletes hotbar item
                any_interacted = true;
                dragged_slots.insert((1, h_idx));
                hotbar.slots[h_idx] = None;
            } else if left_click {
                any_interacted = true;
                dragged_slots.insert((1, h_idx));
                let current_item = hotbar.slots[h_idx];
                let temp = held_item.voxel;
                held_item.voxel = current_item;
                hotbar.slots[h_idx] = temp;
            } else if left_released
                && held_item.voxel.is_some()
                && !dragged_slots.contains(&(1, h_idx))
            {
                any_interacted = true;
                let current_item = hotbar.slots[h_idx];
                let temp = held_item.voxel;
                held_item.voxel = current_item;
                hotbar.slots[h_idx] = temp;
            } else if right_click {
                any_interacted = true;
                let current_item = hotbar.slots[h_idx];
                if held_item.voxel.is_none() && current_item.is_some() {
                    held_item.voxel = current_item;
                    hotbar.slots[h_idx] = None;
                } else if let Some(held) = held_item.voxel {
                    if current_item.is_none() {
                        hotbar.slots[h_idx] = Some(held);
                        held_item.voxel = None;
                    } else {
                        let temp = held_item.voxel;
                        held_item.voxel = current_item;
                        hotbar.slots[h_idx] = temp;
                    }
                }
            }
        }
    }

    // 3. Off-hand Slot
    for (hovered, children) in &mut offhand_query {
        let is_hovered = hovered.get();
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                let target = if is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                if *vis != target {
                    *vis = target;
                }
            }
        }

        if is_hovered && left_click {
            any_interacted = true;
            std::mem::swap(&mut held_item.voxel, &mut player_inv.offhand_slot);
        }
    }

    // 4. Quick Utility Slot
    for (hovered, children) in &mut utility_query {
        let is_hovered = hovered.get();
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                let target = if is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                if *vis != target {
                    *vis = target;
                }
            }
        }

        if is_hovered && left_click {
            any_interacted = true;
            std::mem::swap(&mut held_item.voxel, &mut player_inv.utility_slot);
        }
    }

    // 5. Creative Catalog Slots
    for (hovered, slot, children) in &mut catalog_slot_query {
        let is_hovered = hovered.get();
        for child in children.iter() {
            if let Ok(mut vis) = hover_overlay_query.get_mut(child) {
                let target = if is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                if *vis != target {
                    *vis = target;
                }
            }
        }

        if is_hovered {
            let c_idx = slot.slot_index;
            if let Some(h_idx) = digit_pressed {
                any_interacted = true;
                if let Some(voxel) = slot.voxel {
                    hotbar.slots[h_idx] = Some(voxel);
                }
            } else if is_shift
                && (left_click || (left_held && !dragged_slots.contains(&(2, c_idx))))
            {
                any_interacted = true;
                dragged_slots.insert((2, c_idx));
                if let Some(voxel) = slot.voxel {
                    if let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none()) {
                        hotbar.slots[empty_idx] = Some(voxel);
                    } else {
                        let active = hotbar.active_slot;
                        hotbar.slots[active] = Some(voxel);
                    }
                }
            } else if left_click || right_click {
                any_interacted = true;
                if let Some(voxel) = slot.voxel {
                    held_item.voxel = Some(voxel);
                } else {
                    // Empty catalog slot: clicking with held item discards it
                    held_item.voxel = None;
                }
            } else if left_released {
                // Releasing mouse over an empty catalog slot discards held item
                if slot.voxel.is_none() && held_item.voxel.is_some() {
                    any_interacted = true;
                    held_item.voxel = None;
                }
            }
        }
    }

    // 6. Sync active held voxel with active hotbar slot
    if let Some(mut sel) = selected_voxel {
        sel.0 = hotbar.slots[hotbar.active_slot];
    }

    // 7. Deselect held item if clicking or releasing outside the card (and no slot was clicked)
    let outside_trigger = !any_interacted && (left_click || right_click || left_released);
    if outside_trigger
        && !scroll_state.is_dragging_thumb
        && let (Some(window), Some((transform, computed))) =
            (window_query.iter().next(), card_query.iter().next())
        && let Some(cursor_pos) = window.cursor_position()
    {
        let center = transform.translation().truncate();
        let half = computed.size() * 0.5;
        let card_rect = Rect::from_corners(center - half, center + half);
        if !card_rect.contains(cursor_pos) {
            if *tab_state == InventoryTab::Creative {
                held_item.voxel = None;
            } else if let Some(held) = held_item.voxel.take()
                && !player_inv.add_item(held)
                && let Some(empty_idx) = hotbar.slots.iter().position(|s| s.is_none())
            {
                hotbar.slots[empty_idx] = Some(held);
            }
        }
    }
}

fn sync_inventory_hotbar_icons(
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    icons: Res<BlockIcons>,
    mut hotbar_icons: Query<(&InventoryHotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
    mut storage_icons: Query<
        (&PlayerStorageSlotIcon, &mut ImageNode, &mut Visibility),
        Without<InventoryHotbarSlotIcon>,
    >,
    mut offhand_icons: Query<
        (&mut ImageNode, &mut Visibility),
        (
            With<InventoryOffhandIcon>,
            Without<InventoryHotbarSlotIcon>,
            Without<PlayerStorageSlotIcon>,
        ),
    >,
    mut utility_icons: Query<
        (&mut ImageNode, &mut Visibility),
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

        for (mut img, mut vis) in &mut offhand_icons {
            if let Some(voxel) = player_inv.offhand_slot {
                img.image = icons.get(voxel);
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }

        for (mut img, mut vis) in &mut utility_icons {
            if let Some(voxel) = player_inv.utility_slot {
                img.image = icons.get(voxel);
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn update_inventory_tooltip(
    window_query: Query<&Window, With<PrimaryWindow>>,
    hotbar: Res<Hotbar>,
    player_inv: Res<PlayerInventory>,
    storage_query: Query<(&Hovered, &PlayerStorageSlot)>,
    hotbar_query: Query<(&Hovered, &InventoryHotbarSlot)>,
    catalog_query: Query<(&Hovered, &InventoryCatalogSlot)>,
    offhand_query: Query<&Hovered, With<InventoryOffhandSlot>>,
    utility_query: Query<&Hovered, With<InventoryUtilitySlot>>,
    action_btn_query: Query<(&Hovered, &InventoryAction)>,
    ui_scale: Option<Res<bevy::ui::UiScale>>,
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

    let mut hovered_text: Option<String> = None;

    for (hovered, slot) in &storage_query {
        if hovered.get() {
            if let Some(voxel) = player_inv.slots.get(slot.slot_index).copied().flatten() {
                hovered_text = Some(voxel.label().to_string());
            }
            break;
        }
    }

    if hovered_text.is_none() {
        for (hovered, slot) in &hotbar_query {
            if hovered.get() {
                if let Some(voxel) = hotbar.slots.get(slot.index).copied().flatten() {
                    hovered_text = Some(voxel.label().to_string());
                }
                break;
            }
        }
    }

    if hovered_text.is_none() {
        for (hovered, slot) in &catalog_query {
            if hovered.get() {
                if let Some(voxel) = slot.voxel {
                    hovered_text = Some(voxel.label().to_string());
                }
                break;
            }
        }
    }

    if hovered_text.is_none() {
        for hovered in &offhand_query {
            if hovered.get() {
                if let Some(voxel) = player_inv.offhand_slot {
                    hovered_text = Some(voxel.label().to_string());
                }
                break;
            }
        }
    }

    if hovered_text.is_none() {
        for hovered in &utility_query {
            if hovered.get() {
                if let Some(voxel) = player_inv.utility_slot {
                    hovered_text = Some(voxel.label().to_string());
                }
                break;
            }
        }
    }

    if hovered_text.is_none() {
        for (hovered, action) in &action_btn_query {
            if hovered.get() {
                hovered_text = Some(match action {
                    InventoryAction::Sort => "Auto-Sort Inventory".to_string(),
                    InventoryAction::Trash => "Quick Discard Held Item".to_string(),
                    InventoryAction::Options => "Preferences".to_string(),
                    InventoryAction::Lock => "Lock Slots".to_string(),
                });
                break;
            }
        }
    }

    if let (Some(label), Some(cursor_pos)) = (hovered_text, window.cursor_position()) {
        let scale = ui_scale.map_or(1.0, |s| s.0);
        text.0 = label;
        box_node.left = px((cursor_pos.x / scale) + 16.0);
        box_node.top = px((cursor_pos.y / scale) - 12.0);
        *box_vis = Visibility::Visible;
    } else {
        *box_vis = Visibility::Hidden;
    }
}
