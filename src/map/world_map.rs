use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
        mouse::AccumulatedMouseScroll,
    },
    math::Rot2,
    picking::{Pickable, hover::Hovered},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    text::FontSize,
    ui::UiTransform,
    ui_widgets::Button,
    window::PrimaryWindow,
};

use crate::{
    core::{
        AppFont, ControlSettings, DynamicInputPrompt, InputAction, InputPromptAtlas, Localization,
        spawn_input_prompt, text_shadow_default,
    },
    environment::EnvironmentState,
    menu::{CursorMode, MenuState, ThemedButton, menu_bar::MENU_BAR_HEIGHT, theme::palette},
    player::{GameMode, Player, PlayerCamera, PlayerMotion},
    world::{VOXEL_SIZE, Voxel},
};

use super::{
    cache::MapCache,
    color::{apply_relief_shading, unexplored_color},
};

pub const WORLD_MAP_WIDTH: u32 = 900;
pub const WORLD_MAP_HEIGHT: u32 = 440;
pub const WORLD_MAP_VIEWPORT_WIDTH: f32 = 1800.0;
pub const WORLD_MAP_VIEWPORT_HEIGHT: f32 = 880.0;
pub const WORLD_MARKER_SIZE: f32 = 28.0;
const WAYPOINT_PIN_SIZE: f32 = 22.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WaypointKind {
    #[default]
    Target,
    Home,
    Mark,
    Camp,
    Mine,
    Danger,
    Star,
}

impl WaypointKind {
    pub const ALL: [Self; 7] = [
        Self::Target,
        Self::Home,
        Self::Mark,
        Self::Camp,
        Self::Mine,
        Self::Danger,
        Self::Star,
    ];

    pub fn icon_path(self) -> &'static str {
        match self {
            Self::Target => "textures/interfaces/atlases/decorations/wp_target.png",
            Self::Home => "textures/interfaces/atlases/decorations/wp_home.png",
            Self::Mark => "textures/interfaces/atlases/decorations/x_mark.png",
            Self::Camp => "textures/interfaces/atlases/decorations/wp_camp.png",
            Self::Mine => "textures/interfaces/atlases/decorations/wp_mine.png",
            Self::Danger => "textures/interfaces/atlases/decorations/wp_skull.png",
            Self::Star => "textures/interfaces/atlases/decorations/wp_star.png",
        }
    }

    pub fn name_loc_key(self) -> &'static str {
        match self {
            Self::Target => "map.wp_kind.target",
            Self::Home => "map.wp_kind.home",
            Self::Mark => "map.wp_kind.mark",
            Self::Camp => "map.wp_kind.camp",
            Self::Mine => "map.wp_kind.mine",
            Self::Danger => "map.wp_kind.danger",
            Self::Star => "map.wp_kind.star",
        }
    }

    pub fn default_name(self) -> &'static str {
        match self {
            Self::Target => "Target",
            Self::Home => "Home",
            Self::Mark => "Mark",
            Self::Camp => "Camp",
            Self::Mine => "Mine",
            Self::Danger => "Danger",
            Self::Star => "Star",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MapWaypoint {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub z: i32,
    pub kind: WaypointKind,
}

#[derive(Resource)]
pub struct WorldWaypoints {
    pub waypoints: Vec<MapWaypoint>,
    pub next_id: u32,
    pub selected_kind: WaypointKind,
    pub editing_id: Option<u32>,
    pub placing_mode: bool,
    pub ui_dirty: bool,
}

impl Default for WorldWaypoints {
    fn default() -> Self {
        Self {
            waypoints: Vec::new(),
            next_id: 1,
            selected_kind: WaypointKind::Target,
            editing_id: None,
            placing_mode: false,
            ui_dirty: true,
        }
    }
}

impl WorldWaypoints {
    pub fn add(&mut self, loc: Option<&Localization>, x: i32, z: i32) {
        let id = self.next_id;
        self.next_id += 1;
        let kind = self.selected_kind;
        let kind_count = self.waypoints.iter().filter(|w| w.kind == kind).count() + 1;
        let base_name = loc.map_or(kind.default_name(), |l| l.t(kind.name_loc_key()));
        self.waypoints.push(MapWaypoint {
            id,
            name: format!("{base_name} {kind_count}"),
            x,
            z,
            kind,
        });
        self.ui_dirty = true;
    }

    pub fn remove(&mut self, id: u32) {
        let prev_len = self.waypoints.len();
        self.waypoints.retain(|w| w.id != id);
        if self.editing_id == Some(id) {
            self.editing_id = None;
        }
        if self.waypoints.len() != prev_len {
            self.ui_dirty = true;
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldMapLayers {
    pub show_player_marker: bool,
    pub show_waypoints: bool,
    pub show_chunk_grid: bool,
    pub follow_day_night: bool,
    pub show_elevation: bool,
}

impl Default for WorldMapLayers {
    fn default() -> Self {
        Self {
            show_player_marker: true,
            show_waypoints: true,
            show_chunk_grid: false,
            follow_day_night: true,
            show_elevation: true,
        }
    }
}

#[derive(Resource, Default)]
struct WorldMapContextMenu {
    open: bool,
    viewport_pos: Vec2,
    world_x: i32,
    world_z: i32,
}

#[derive(Component)]
pub struct WorldMapRoot;

#[derive(Component)]
pub struct WorldMapFrame;

#[derive(Component)]
pub struct WorldMapViewport;

#[derive(Component)]
pub struct WorldMapPlayerMarker;

#[derive(Component)]
pub struct WorldMapInfoText;

#[derive(Component)]
pub struct WorldMapCursorText;

#[derive(Component)]
struct WorldMapWaypointPinsContainer;

#[derive(Component)]
struct WorldMapWaypointPin {
    world_x: i32,
    world_z: i32,
}

#[derive(Component)]
struct WorldMapPlacementGhost;

#[derive(Component)]
struct WorldMapWaypointListContainer;

#[derive(Component, Clone, Copy)]
struct WaypointIconPickerButton(WaypointKind);

#[derive(Component, Clone, Copy)]
struct WaypointFocusButton {
    x: i32,
    z: i32,
}

#[derive(Component, Clone, Copy)]
struct WaypointRenameButton(u32);

#[derive(Component, Clone, Copy)]
struct WaypointDeleteButton(u32);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum WorldMapLayerKind {
    PlayerMarker,
    Waypoints,
    ChunkGrid,
    DayNight,
    Elevation,
}

#[derive(Component)]
struct LayerStateText(WorldMapLayerKind);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum WorldMapAction {
    CenterPlayer,
    CreateWaypoint,
}

#[derive(Component)]
struct MapContextMenuRoot;

#[derive(Component)]
struct MapContextMenuCoordsText;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MapContextAction {
    AddWaypoint,
    Teleport,
    Cancel,
}

#[derive(Resource)]
pub struct WorldMapState {
    pub center: Vec2,
    pub zoom: f32,
    pub is_dragging: bool,
    pub last_cursor_screen_pos: Option<Vec2>,
    pub terrain_image: Handle<Image>,
    pub marker_image: Handle<Image>,
    pub last_drawn_center: Vec2,
    pub last_drawn_zoom: f32,
    pub last_drawn_aspect: f32,
    pub last_cache_version: u64,
    pub last_drawn_layers: WorldMapLayers,
    pub last_drawn_time_bucket: i32,
}

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldWaypoints>()
            .init_resource::<WorldMapLayers>()
            .init_resource::<WorldMapContextMenu>()
            .add_systems(Startup, setup_world_map_resources)
            .add_systems(OnEnter(MenuState::WorldMap), spawn_world_map_ui)
            .add_systems(OnExit(MenuState::WorldMap), despawn_world_map_ui)
            .add_systems(
                Update,
                (
                    handle_waypoint_rename_typing,
                    handle_world_map_input,
                    handle_world_map_sidebar_buttons,
                    handle_context_menu_buttons,
                    rebuild_waypoint_ui_if_dirty,
                    sync_world_map_terrain,
                    update_world_map_ui,
                )
                    .chain()
                    .run_if(in_state(MenuState::WorldMap)),
            );
    }
}

fn setup_world_map_resources(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let terrain_pixels =
        [18u8, 19u8, 24u8, 255u8].repeat((WORLD_MAP_WIDTH * WORLD_MAP_HEIGHT) as usize);

    let mut terrain_img = Image::new_fill(
        Extent3d {
            width: WORLD_MAP_WIDTH,
            height: WORLD_MAP_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &terrain_pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    terrain_img.sampler = ImageSampler::nearest();
    let terrain_handle = images.add(terrain_img);

    let marker_handle: Handle<Image> =
        asset_server.load("textures/interfaces/atlases/decorations/marker_red.png");

    commands.insert_resource(WorldMapState {
        center: Vec2::ZERO,
        zoom: 1.0,
        is_dragging: false,
        last_cursor_screen_pos: None,
        terrain_image: terrain_handle,
        marker_image: marker_handle,
        last_drawn_center: Vec2::splat(999_999.0),
        last_drawn_zoom: -1.0,
        last_drawn_aspect: -1.0,
        last_cache_version: u64::MAX,
        last_drawn_layers: WorldMapLayers::default(),
        last_drawn_time_bucket: -1,
    });
}

fn daylight_tint(time_of_day: f32) -> [f32; 3] {
    // 0.00 = dawn (06:00), 0.25 = noon (12:00), 0.50 = dusk (18:00), 0.75 = midnight (00:00)
    let sun = (time_of_day * std::f32::consts::TAU).sin().clamp(-1.0, 1.0);
    if sun >= 0.2 {
        let t = ((sun - 0.2) / 0.8).clamp(0.0, 1.0);
        [0.92 + 0.08 * t, 0.88 + 0.12 * t, 0.84 + 0.16 * t]
    } else if sun >= -0.2 {
        let t = ((sun + 0.2) / 0.4).clamp(0.0, 1.0);
        [0.54 + 0.38 * t, 0.44 + 0.44 * t, 0.48 + 0.36 * t]
    } else {
        let t = ((sun + 1.0) / 0.8).clamp(0.0, 1.0);
        [0.24 + 0.30 * t, 0.28 + 0.16 * t, 0.44 + 0.04 * t]
    }
}

fn spawn_world_map_ui(
    mut commands: Commands,
    mut map_state: ResMut<WorldMapState>,
    mut waypoints: ResMut<WorldWaypoints>,
    mut ctx_menu: ResMut<WorldMapContextMenu>,
    layers: Res<WorldMapLayers>,
    game_mode: Option<Res<GameMode>>,
    player_query: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
    app_font: Option<Res<AppFont>>,
    loc: Option<Res<Localization>>,
    input_prompts: Option<Res<InputPromptAtlas>>,
    controls: Option<Res<ControlSettings>>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source()).unwrap_or_default();
    let make_font = |size: f32| TextFont {
        font: font_handle.clone(),
        font_size: FontSize::Px(size),
        ..default()
    };
    let t = |key: &str, fallback: &str| -> String {
        loc.as_ref()
            .map_or_else(|| fallback.to_string(), |l| l.t(key).to_string())
    };

    if let Ok(player_transform) = player_query.single() {
        let p = player_transform.translation;
        map_state.center = Vec2::new(p.x / VOXEL_SIZE, p.z / VOXEL_SIZE);
    }
    map_state.last_drawn_center = Vec2::splat(999_999.0);
    waypoints.ui_dirty = true;
    waypoints.placing_mode = false;
    ctx_menu.open = false;

    let is_creative = game_mode
        .as_ref()
        .is_some_and(|m| **m == GameMode::Creative);
    let on_str = t("map.state.on", "ON");
    let off_str = t("map.state.off", "OFF");

    commands
        .spawn((
            WorldMapRoot,
            crate::menu::MenuBackdrop,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(MENU_BAR_HEIGHT),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                ..default()
            },
            BackgroundColor(palette::PANEL_INNER_BG),
            ZIndex(250),
        ))
        .with_children(|root| {
            // Left / Main Interactive Full-Height World Map Viewport
            root.spawn((
                WorldMapFrame,
                WorldMapViewport,
                Node {
                    flex_grow: 1.0,
                    height: percent(100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ZIndex(5),
            ))
            .with_children(|viewport| {
                // Terrain Render Texture
                viewport.spawn((
                    Pickable::IGNORE,
                    ImageNode {
                        image: map_state.terrain_image.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    ZIndex(1),
                ));

                // Waypoint Pins Container (populated dynamically by rebuild_waypoint_ui_if_dirty)
                viewport.spawn((
                    WorldMapWaypointPinsContainer,
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    ZIndex(20),
                ));

                // Player Directional Red Marker
                let half_m = WORLD_MARKER_SIZE / 2.0;
                viewport.spawn((
                    WorldMapPlayerMarker,
                    Pickable::IGNORE,
                    ImageNode {
                        image: map_state.marker_image.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(WORLD_MARKER_SIZE),
                        height: px(WORLD_MARKER_SIZE),
                        left: percent(50.0),
                        top: percent(50.0),
                        margin: UiRect {
                            left: px(-half_m),
                            top: px(-half_m),
                            ..default()
                        },
                        ..default()
                    },
                    UiTransform::IDENTITY,
                    Visibility::Visible,
                    ZIndex(30),
                ));

                // Cursor-following Waypoint Placement Ghost Pin
                let ghost_size = 26.0;
                viewport.spawn((
                    WorldMapPlacementGhost,
                    Pickable::IGNORE,
                    ImageNode {
                        image: asset_server.load(waypoints.selected_kind.icon_path()),
                        image_mode: NodeImageMode::Stretch,
                        color: Color::srgba(1.0, 1.0, 1.0, 0.90),
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(ghost_size),
                        height: px(ghost_size),
                        left: px(0.0),
                        top: px(0.0),
                        margin: UiRect {
                            left: px(-ghost_size * 0.5),
                            top: px(-ghost_size * 0.5),
                            ..default()
                        },
                        ..default()
                    },
                    Visibility::Hidden,
                    ZIndex(55),
                ));

                // Floating Right-Click Context Menu
                viewport
                    .spawn((
                        MapContextMenuRoot,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0.0),
                            top: px(0.0),
                            width: px(220.0),
                            padding: UiRect::all(px(8.0)),
                            border: UiRect::all(px(1.5)),
                            border_radius: BorderRadius::all(px(6.0)),
                            display: Display::None,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(5.0),
                            ..default()
                        },
                        BackgroundColor(palette::DRAWER_COL1_BG),
                        BorderColor::all(palette::ACCENT_LINE),
                        ZIndex(60),
                    ))
                    .with_children(|ctx| {
                        ctx.spawn((
                            MapContextMenuCoordsText,
                            Pickable::IGNORE,
                            Text::new("X: 0  Z: 0"),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                            Node {
                                margin: UiRect::axes(px(6.0), px(2.0)),
                                ..default()
                            },
                        ));

                        let ctx_actions: Vec<(MapContextAction, String, Color)> = if is_creative {
                            vec![
                                (
                                    MapContextAction::AddWaypoint,
                                    t("map.ctx.add_waypoint", "+ Add Waypoint Here"),
                                    palette::TEXT_ACCENT,
                                ),
                                (
                                    MapContextAction::Teleport,
                                    t("map.ctx.teleport", "Teleport Here (Creative)"),
                                    Color::srgb(0.45, 0.88, 0.98),
                                ),
                                (
                                    MapContextAction::Cancel,
                                    t("map.ctx.cancel", "Cancel"),
                                    palette::TEXT_MUTED,
                                ),
                            ]
                        } else {
                            vec![
                                (
                                    MapContextAction::AddWaypoint,
                                    t("map.ctx.add_waypoint", "+ Add Waypoint Here"),
                                    palette::TEXT_ACCENT,
                                ),
                                (
                                    MapContextAction::Cancel,
                                    t("map.ctx.cancel", "Cancel"),
                                    palette::TEXT_MUTED,
                                ),
                            ]
                        };

                        for (act, label, col) in ctx_actions {
                            let theme = ThemedButton::subtle();
                            ctx.spawn((
                                Button,
                                Hovered::default(),
                                act,
                                theme,
                                Node {
                                    width: percent(100.0),
                                    height: px(32.0),
                                    padding: UiRect::horizontal(px(10.0)),
                                    border: UiRect::all(px(1.0)),
                                    border_radius: BorderRadius::all(px(4.0)),
                                    display: Display::Flex,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::FlexStart,
                                    ..default()
                                },
                                BackgroundColor(theme.normal_bg),
                                BorderColor::all(theme.normal_border),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Pickable::IGNORE,
                                    Text::new(label),
                                    make_font(13.0),
                                    TextColor(col),
                                    text_shadow_default(),
                                ));
                            });
                        }
                    });

                // Subtle bottom-left controls hint with Kenney pixel-art input prompts
                let pan_lbl = t("map.hint.pan", "Move");
                let actions_lbl = t("map.hint.actions", "Actions");
                let center_lbl = t("map.hint.center", "Center");
                let close_lbl = t("map.hint.close", "Close");
                viewport
                    .spawn((
                        Pickable::IGNORE,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(18.0),
                            bottom: px(14.0),
                            padding: UiRect::axes(px(12.0), px(6.0)),
                            border_radius: BorderRadius::all(px(4.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(14.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.82)),
                        ZIndex(40),
                    ))
                    .with_children(|pill| {
                        if let Some(ref atlas) = input_prompts {
                            let items: [(DynamicInputPrompt, &str); 4] = [
                                (DynamicInputPrompt::Mouse(MouseButton::Left), &pan_lbl),
                                (DynamicInputPrompt::Mouse(MouseButton::Right), &actions_lbl),
                                (DynamicInputPrompt::Key(KeyCode::Space), &center_lbl),
                                (
                                    DynamicInputPrompt::Action(InputAction::WorldMap),
                                    &close_lbl,
                                ),
                            ];
                            for (prompt, label) in items {
                                pill.spawn((
                                    Pickable::IGNORE,
                                    Node {
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: px(6.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|grp| {
                                    spawn_input_prompt(
                                        grp,
                                        atlas,
                                        prompt,
                                        controls.as_deref(),
                                        20.0,
                                    );
                                    grp.spawn((
                                        Pickable::IGNORE,
                                        Text::new(label),
                                        make_font(13.0),
                                        TextColor(palette::TEXT_MUTED),
                                        text_shadow_default(),
                                    ));
                                });
                            }
                        } else {
                            let hint_text = t(
                                "map.controls_hint",
                                "LMB Drag: Pan  |  RMB: Actions  |  Scroll: Zoom  |  Space: Center",
                            );
                            pill.spawn((
                                Pickable::IGNORE,
                                Text::new(hint_text),
                                make_font(13.0),
                                TextColor(palette::TEXT_MUTED),
                                text_shadow_default(),
                            ));
                        }
                    });
            });

            // Right-Docked Full-Height Map Sidebar (standardized 310px width)
            root.spawn((
                Node {
                    width: px(palette::DRAWER_COL1_WIDTH),
                    min_width: px(palette::DRAWER_COL1_WIDTH),
                    flex_shrink: 0.0,
                    height: percent(100.0),
                    border: UiRect::left(px(1.5)),
                    padding: UiRect::all(px(20.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(14.0),
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL1_BG),
                BorderColor::all(palette::DRAWER_DIVIDER),
                ZIndex(10),
            ))
            .with_children(|sidebar| {
                // Quick Actions
                sidebar
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8.0),
                        ..default()
                    })
                    .with_children(|actions| {
                        actions.spawn((
                            Text::new(t("map.tools", "MAP TOOLS").to_uppercase()),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        let btn_theme = ThemedButton::subtle();

                        actions
                            .spawn((
                                Button,
                                Hovered::default(),
                                WorldMapAction::CreateWaypoint,
                                btn_theme,
                                BackgroundColor(btn_theme.normal_bg),
                                BorderColor::all(palette::ACCENT_LINE),
                                Node {
                                    width: percent(100.0),
                                    height: px(36.0),
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::all(px(4.0)),
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Pickable::IGNORE,
                                    Text::new(t("map.new_waypoint", "+ New Waypoint")),
                                    make_font(14.0),
                                    TextColor(palette::TEXT_ACCENT),
                                    text_shadow_default(),
                                ));
                            });

                        actions
                            .spawn((
                                Button,
                                Hovered::default(),
                                WorldMapAction::CenterPlayer,
                                btn_theme,
                                BackgroundColor(btn_theme.normal_bg),
                                BorderColor::all(btn_theme.normal_border),
                                Node {
                                    width: percent(100.0),
                                    height: px(34.0),
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::all(px(4.0)),
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Pickable::IGNORE,
                                    Text::new(t("map.center_player", "Center on Player")),
                                    make_font(13.0),
                                    TextColor(palette::TEXT_PRIMARY),
                                    text_shadow_default(),
                                ));
                            });
                    });

                // Waypoint Icon Picker Row (7 icon types)
                sidebar
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|picker_sec| {
                        picker_sec.spawn((
                            Text::new(t("map.waypoint_icon", "WAYPOINT ICON").to_uppercase()),
                            make_font(11.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        picker_sec
                            .spawn(Node {
                                width: percent(100.0),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|row| {
                                for kind in WaypointKind::ALL {
                                    let is_sel = kind == waypoints.selected_kind;
                                    let icon = asset_server.load(kind.icon_path());
                                    row.spawn((
                                        Button,
                                        Hovered::default(),
                                        WaypointIconPickerButton(kind),
                                        Node {
                                            width: px(34.0),
                                            height: px(34.0),
                                            border: UiRect::all(px(1.5)),
                                            border_radius: BorderRadius::all(px(4.0)),
                                            display: Display::Flex,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        BackgroundColor(if is_sel {
                                            palette::BTN_HOVER_BG
                                        } else {
                                            palette::PANEL_INNER_BG
                                        }),
                                        BorderColor::all(if is_sel {
                                            palette::ACCENT_LINE
                                        } else {
                                            palette::PANEL_BORDER_SUBTLE
                                        }),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Pickable::IGNORE,
                                            ImageNode {
                                                image: icon,
                                                image_mode: NodeImageMode::Stretch,
                                                ..default()
                                            },
                                            Node {
                                                width: px(20.0),
                                                height: px(20.0),
                                                ..default()
                                            },
                                        ));
                                    });
                                }
                            });
                    });

                // Waypoints List (dynamic & scrollable)
                sidebar
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6.0),
                        flex_grow: 1.0,
                        min_height: px(120.0),
                        max_height: px(280.0),
                        ..default()
                    })
                    .with_children(|wps| {
                        wps.spawn((
                            Text::new(t("map.waypoints", "WAYPOINTS").to_uppercase()),
                            make_font(11.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        wps.spawn((
                            WorldMapWaypointListContainer,
                            Hovered::default(),
                            ScrollPosition::default(),
                            Node {
                                width: percent(100.0),
                                flex_grow: 1.0,
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(5.0),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                        ));
                    });

                // Spacer pushing Layers & Coordinates to the bottom of the sidebar
                sidebar.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                // Functional Map Layers & Filters (docked above coordinates box)
                sidebar
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|layer_sec| {
                        layer_sec.spawn((
                            Text::new(t("map.layers", "LAYERS & FILTERS").to_uppercase()),
                            make_font(11.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        let layer_defs = [
                            (
                                WorldMapLayerKind::PlayerMarker,
                                t("map.layer.player_marker", "Player Marker"),
                                layers.show_player_marker,
                            ),
                            (
                                WorldMapLayerKind::Waypoints,
                                t("map.layer.waypoints", "Waypoints"),
                                layers.show_waypoints,
                            ),
                            (
                                WorldMapLayerKind::ChunkGrid,
                                t("map.layer.chunk_grid", "Chunk Grid (16x16)"),
                                layers.show_chunk_grid,
                            ),
                            (
                                WorldMapLayerKind::DayNight,
                                t("map.layer.day_night", "Day / Night Lighting"),
                                layers.follow_day_night,
                            ),
                            (
                                WorldMapLayerKind::Elevation,
                                t("map.layer.elevation", "Elevation Contours"),
                                layers.show_elevation,
                            ),
                        ];

                        for (kind, name, enabled) in layer_defs {
                            layer_sec
                                .spawn((
                                    Button,
                                    Hovered::default(),
                                    kind,
                                    Node {
                                        width: percent(100.0),
                                        height: px(30.0),
                                        padding: UiRect::horizontal(px(10.0)),
                                        border: UiRect::all(px(1.0)),
                                        border_radius: BorderRadius::all(px(4.0)),
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(palette::PANEL_INNER_BG),
                                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                                ))
                                .with_children(|row| {
                                    row.spawn((
                                        Pickable::IGNORE,
                                        Text::new(name),
                                        make_font(12.0),
                                        TextColor(palette::TEXT_PRIMARY),
                                    ));
                                    row.spawn((
                                        Pickable::IGNORE,
                                        LayerStateText(kind),
                                        Text::new(if enabled {
                                            on_str.clone()
                                        } else {
                                            off_str.clone()
                                        }),
                                        make_font(11.0),
                                        TextColor(if enabled {
                                            palette::TEXT_ACCENT
                                        } else {
                                            palette::TEXT_MUTED
                                        }),
                                    ));
                                });
                        }
                    });

                sidebar
                    .spawn((
                        Node {
                            width: percent(100.0),
                            padding: UiRect::all(px(12.0)),
                            border: UiRect::all(px(1.0)),
                            border_radius: BorderRadius::all(px(5.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6.0),
                            ..default()
                        },
                        BackgroundColor(palette::PANEL_INNER_BG),
                        BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                    ))
                    .with_children(|coords| {
                        coords.spawn((
                            WorldMapInfoText,
                            Text::new("Player: X: 0 Y: 0 Z: 0"),
                            make_font(13.0),
                            TextColor(palette::TEXT_PRIMARY),
                            text_shadow_default(),
                        ));
                        coords.spawn((
                            WorldMapCursorText,
                            Text::new("Cursor: X: 0 Z: 0 | Zoom: 1.00x"),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                            text_shadow_default(),
                        ));
                    });
            });
        });
}

fn rebuild_waypoint_ui_if_dirty(
    mut commands: Commands,
    mut waypoints: ResMut<WorldWaypoints>,
    asset_server: Res<AssetServer>,
    app_font: Option<Res<AppFont>>,
    loc: Option<Res<Localization>>,
    pins_q: Query<(Entity, Option<&Children>), With<WorldMapWaypointPinsContainer>>,
    list_q: Query<(Entity, Option<&Children>), With<WorldMapWaypointListContainer>>,
) {
    if !waypoints.ui_dirty {
        return;
    }
    waypoints.ui_dirty = false;

    let font_handle = app_font.as_ref().map(|f| f.source()).unwrap_or_default();
    let make_font = |size: f32| TextFont {
        font: font_handle.clone(),
        font_size: FontSize::Px(size),
        ..default()
    };

    // 1. Rebuild pins on the map viewport
    let half_pin = WAYPOINT_PIN_SIZE * 0.5;
    for (container, children) in &pins_q {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        commands.entity(container).with_children(|pins| {
            for wp in &waypoints.waypoints {
                let icon = asset_server.load(wp.kind.icon_path());
                pins.spawn((
                    WorldMapWaypointPin {
                        world_x: wp.x,
                        world_z: wp.z,
                    },
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50.0),
                        top: percent(50.0),
                        margin: UiRect {
                            left: px(-half_pin),
                            top: px(-half_pin),
                            ..default()
                        },
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(2.0),
                        ..default()
                    },
                    Visibility::Hidden,
                ))
                .with_children(|pin| {
                    pin.spawn((
                        Pickable::IGNORE,
                        ImageNode {
                            image: icon,
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        Node {
                            width: px(WAYPOINT_PIN_SIZE),
                            height: px(WAYPOINT_PIN_SIZE),
                            ..default()
                        },
                    ));
                    pin.spawn((
                        Pickable::IGNORE,
                        Text::new(wp.name.clone()),
                        make_font(10.0),
                        TextColor(palette::TEXT_PRIMARY),
                        text_shadow_default(),
                    ));
                });
            }
        });
    }

    // 2. Rebuild waypoint list in the sidebar
    let empty_text = loc
        .as_ref()
        .map_or("No waypoints yet. Right-click the map to add one.", |l| {
            l.t("map.no_waypoints")
        });

    for (container, children) in &list_q {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        commands.entity(container).with_children(|list| {
            if waypoints.waypoints.is_empty() {
                list.spawn((
                    Text::new(empty_text),
                    make_font(11.0),
                    TextColor(palette::TEXT_MUTED),
                ));
                return;
            }

            for wp in &waypoints.waypoints {
                let icon = asset_server.load(wp.kind.icon_path());
                let is_editing = waypoints.editing_id == Some(wp.id);
                let display_title = if is_editing {
                    format!("{}_", wp.name)
                } else {
                    wp.name.clone()
                };

                list.spawn((
                    Node {
                        width: percent(100.0),
                        flex_shrink: 0.0,
                        padding: UiRect::axes(px(8.0), px(6.0)),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(4.0)),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: px(6.0),
                        ..default()
                    },
                    BackgroundColor(if is_editing {
                        palette::BTN_HOVER_BG
                    } else {
                        palette::PANEL_INNER_BG
                    }),
                    BorderColor::all(if is_editing {
                        palette::ACCENT_LINE
                    } else {
                        palette::PANEL_BORDER_SUBTLE
                    }),
                ))
                .with_children(|row| {
                    row.spawn((
                        Button,
                        Hovered::default(),
                        WaypointFocusButton { x: wp.x, z: wp.z },
                        Node {
                            flex_grow: 1.0,
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            column_gap: px(6.0),
                            ..default()
                        },
                    ))
                    .with_children(|focus| {
                        focus
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(6.0),
                                ..default()
                            })
                            .with_children(|left| {
                                left.spawn((
                                    Pickable::IGNORE,
                                    ImageNode {
                                        image: icon,
                                        image_mode: NodeImageMode::Stretch,
                                        ..default()
                                    },
                                    Node {
                                        width: px(16.0),
                                        height: px(16.0),
                                        ..default()
                                    },
                                ));
                                left.spawn((
                                    Pickable::IGNORE,
                                    Text::new(display_title),
                                    make_font(12.0),
                                    TextColor(if is_editing {
                                        palette::TEXT_ACCENT
                                    } else {
                                        palette::TEXT_PRIMARY
                                    }),
                                ));
                            });

                        focus.spawn((
                            Pickable::IGNORE,
                            Text::new(format!("{}, {}", wp.x, wp.z)),
                            make_font(11.0),
                            TextColor(palette::TEXT_MUTED),
                        ));
                    });

                    // Rename button
                    row.spawn((
                        Button,
                        Hovered::default(),
                        WaypointRenameButton(wp.id),
                        Node {
                            width: px(20.0),
                            height: px(20.0),
                            border_radius: BorderRadius::all(px(3.0)),
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(if is_editing {
                            palette::ACCENT_LINE
                        } else {
                            Color::srgba(0.18, 0.22, 0.30, 0.75)
                        }),
                    ))
                    .with_children(|ren| {
                        ren.spawn((
                            Pickable::IGNORE,
                            Text::new("E"),
                            make_font(10.0),
                            TextColor(if is_editing {
                                Color::srgb(0.08, 0.08, 0.10)
                            } else {
                                palette::TEXT_PRIMARY
                            }),
                        ));
                    });

                    // Delete button
                    row.spawn((
                        Button,
                        Hovered::default(),
                        WaypointDeleteButton(wp.id),
                        Node {
                            width: px(20.0),
                            height: px(20.0),
                            border_radius: BorderRadius::all(px(3.0)),
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.28, 0.12, 0.14, 0.70)),
                    ))
                    .with_children(|del| {
                        del.spawn((
                            Pickable::IGNORE,
                            Text::new("x"),
                            make_font(11.0),
                            TextColor(Color::srgb(0.96, 0.65, 0.65)),
                        ));
                    });
                });
            }
        });
    }
}

fn handle_world_map_sidebar_buttons(
    mut buttons_query: Query<
        (
            &Hovered,
            Has<bevy::ui::Pressed>,
            &WorldMapAction,
            &ThemedButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (
            With<Button>,
            Without<WaypointIconPickerButton>,
            Without<WorldMapLayerKind>,
        ),
    >,
    mut picker_query: Query<
        (
            &Hovered,
            &WaypointIconPickerButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (
            With<Button>,
            Without<WorldMapAction>,
            Without<WorldMapLayerKind>,
        ),
    >,
    mut layer_query: Query<
        (&Hovered, &WorldMapLayerKind, &mut BackgroundColor),
        (
            With<Button>,
            Without<WorldMapAction>,
            Without<WaypointIconPickerButton>,
        ),
    >,
    mut layer_text_query: Query<(&LayerStateText, &mut Text, &mut TextColor)>,
    focus_query: Query<(&Hovered, &WaypointFocusButton), With<Button>>,
    rename_query: Query<(&Hovered, &WaypointRenameButton), With<Button>>,
    delete_query: Query<(&Hovered, &WaypointDeleteButton), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, With<Player>>,
    loc: Option<Res<Localization>>,
    mut local_config: Option<ResMut<crate::core::config::LocalConfig>>,
    mut map_state: ResMut<WorldMapState>,
    mut waypoints: ResMut<WorldWaypoints>,
    mut layers: ResMut<WorldMapLayers>,
) {
    let clicked = mouse.just_pressed(MouseButton::Left);

    // 1. Primary action buttons
    for (hovered, is_pressed, action, theme, mut bg, mut border) in &mut buttons_query {
        let (next_bg, next_border) = theme.colors(hovered.get(), is_pressed);
        if *action == WorldMapAction::CreateWaypoint && waypoints.placing_mode {
            bg.0 = Color::srgba(0.32, 0.26, 0.10, 0.92);
            *border = BorderColor::all(palette::TEXT_ACCENT);
        } else {
            bg.0 = next_bg;
            if *action != WorldMapAction::CreateWaypoint {
                *border = BorderColor::all(next_border);
            } else {
                *border = BorderColor::all(palette::ACCENT_LINE);
            }
        }
        if hovered.get() && clicked {
            match action {
                WorldMapAction::CenterPlayer => {
                    if let Ok(t) = player_query.single() {
                        let p = t.translation;
                        map_state.center = Vec2::new(p.x / VOXEL_SIZE, p.z / VOXEL_SIZE);
                    }
                }
                WorldMapAction::CreateWaypoint => {
                    waypoints.placing_mode = !waypoints.placing_mode;
                }
            }
        }
    }

    // 2. Waypoint icon picker
    for (hovered, picker, mut bg, mut border) in &mut picker_query {
        if hovered.get() && clicked {
            waypoints.selected_kind = picker.0;
        }
        let is_sel = picker.0 == waypoints.selected_kind;
        bg.0 = if is_sel || hovered.get() {
            palette::BTN_HOVER_BG
        } else {
            palette::PANEL_INNER_BG
        };
        *border = BorderColor::all(if is_sel || hovered.get() {
            palette::ACCENT_LINE
        } else {
            palette::PANEL_BORDER_SUBTLE
        });
    }

    // 3. Waypoint rename, focus & delete
    if clicked {
        for (hovered, ren) in &rename_query {
            if hovered.get() {
                waypoints.editing_id = if waypoints.editing_id == Some(ren.0) {
                    None
                } else {
                    Some(ren.0)
                };
                waypoints.ui_dirty = true;
                return;
            }
        }
        for (hovered, del) in &delete_query {
            if hovered.get() {
                waypoints.remove(del.0);
                return;
            }
        }
        for (hovered, focus) in &focus_query {
            if hovered.get() {
                map_state.center = Vec2::new(focus.x as f32, focus.z as f32);
                return;
            }
        }
    }

    // 4. Functional layer toggles
    let mut layer_changed = false;
    for (hovered, kind, mut bg) in &mut layer_query {
        bg.0 = if hovered.get() {
            palette::BTN_HOVER_BG
        } else {
            palette::PANEL_INNER_BG
        };
        if hovered.get() && clicked {
            match kind {
                WorldMapLayerKind::PlayerMarker => {
                    layers.show_player_marker = !layers.show_player_marker
                }
                WorldMapLayerKind::Waypoints => layers.show_waypoints = !layers.show_waypoints,
                WorldMapLayerKind::ChunkGrid => layers.show_chunk_grid = !layers.show_chunk_grid,
                WorldMapLayerKind::DayNight => layers.follow_day_night = !layers.follow_day_night,
                WorldMapLayerKind::Elevation => layers.show_elevation = !layers.show_elevation,
            }
            layer_changed = true;
        }
    }

    if layer_changed {
        if let Some(ref mut cfg) = local_config {
            cfg.map_show_player_marker = layers.show_player_marker;
            cfg.map_show_waypoints = layers.show_waypoints;
            cfg.map_show_chunk_grid = layers.show_chunk_grid;
            cfg.map_follow_day_night = layers.follow_day_night;
            cfg.map_show_elevation = layers.show_elevation;
            let _ = cfg.save();
        }
        let on_str = loc.as_ref().map_or("ON", |l| l.t("map.state.on"));
        let off_str = loc.as_ref().map_or("OFF", |l| l.t("map.state.off"));
        for (tag, mut text, mut col) in &mut layer_text_query {
            let enabled = match tag.0 {
                WorldMapLayerKind::PlayerMarker => layers.show_player_marker,
                WorldMapLayerKind::Waypoints => layers.show_waypoints,
                WorldMapLayerKind::ChunkGrid => layers.show_chunk_grid,
                WorldMapLayerKind::DayNight => layers.follow_day_night,
                WorldMapLayerKind::Elevation => layers.show_elevation,
            };
            text.0 = if enabled {
                on_str.to_string()
            } else {
                off_str.to_string()
            };
            col.0 = if enabled {
                palette::TEXT_ACCENT
            } else {
                palette::TEXT_MUTED
            };
        }
    }
}

fn handle_context_menu_buttons(
    mut ctx_menu: ResMut<WorldMapContextMenu>,
    mut ctx_root_q: Query<&mut Node, With<MapContextMenuRoot>>,
    mut ctx_coords_q: Query<&mut Text, With<MapContextMenuCoordsText>>,
    mut ctx_buttons_q: Query<
        (
            &Hovered,
            Has<bevy::ui::Pressed>,
            &MapContextAction,
            &ThemedButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mouse: Res<ButtonInput<MouseButton>>,
    game_mode: Option<Res<GameMode>>,
    map_cache: Res<MapCache>,
    loc: Option<Res<Localization>>,
    mut waypoints: ResMut<WorldWaypoints>,
    mut player_q: Query<(&mut Transform, &mut PlayerMotion), With<Player>>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    let clicked = mouse.just_pressed(MouseButton::Left);

    if ctx_menu.open {
        let mut any_ctx_hovered = false;
        for (hovered, is_pressed, action, theme, mut bg, mut border) in &mut ctx_buttons_q {
            if hovered.get() {
                any_ctx_hovered = true;
            }
            let (next_bg, next_border) = theme.colors(hovered.get(), is_pressed);
            bg.0 = next_bg;
            *border = BorderColor::all(next_border);

            if hovered.get() && clicked {
                match action {
                    MapContextAction::AddWaypoint => {
                        waypoints.add(loc.as_deref(), ctx_menu.world_x, ctx_menu.world_z);
                        ctx_menu.open = false;
                    }
                    MapContextAction::Teleport => {
                        if game_mode
                            .as_ref()
                            .is_some_and(|m| **m == GameMode::Creative)
                            && let Ok((mut tf, mut motion)) = player_q.single_mut()
                        {
                            let wx = ctx_menu.world_x;
                            let wz = ctx_menu.world_z;
                            let col = IVec2::new(wx >> 4, wz >> 4);
                            let lx = (wx & 15) as usize;
                            let lz = (wz & 15) as usize;
                            let target_y = map_cache
                                .get_chunk(col)
                                .map(|c| c.get(lx, lz))
                                .filter(|p| p.voxel != Voxel::Air)
                                .map_or(tf.translation.y, |p| (p.height as f32 + 2.0) * VOXEL_SIZE);

                            tf.translation = Vec3::new(
                                (wx as f32 + 0.5) * VOXEL_SIZE,
                                target_y,
                                (wz as f32 + 0.5) * VOXEL_SIZE,
                            );
                            motion.velocity = Vec3::ZERO;
                            ctx_menu.open = false;
                            next_state.set(MenuState::None);
                        }
                    }
                    MapContextAction::Cancel => {
                        ctx_menu.open = false;
                    }
                }
                break;
            }
        }

        if clicked && !any_ctx_hovered {
            ctx_menu.open = false;
        }
    }

    for mut node in &mut ctx_root_q {
        if ctx_menu.open {
            node.display = Display::Flex;
            node.left = px(ctx_menu.viewport_pos.x);
            node.top = px(ctx_menu.viewport_pos.y);
        } else {
            node.display = Display::None;
        }
    }
    if ctx_menu.open {
        for mut txt in &mut ctx_coords_q {
            txt.0 = format!("X: {}   Z: {}", ctx_menu.world_x, ctx_menu.world_z);
        }
    }
}

fn despawn_world_map_ui(mut commands: Commands, query: Query<Entity, With<WorldMapRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_waypoint_rename_typing(
    mut key_events: MessageReader<KeyboardInput>,
    mut waypoints: ResMut<WorldWaypoints>,
) {
    let Some(edit_id) = waypoints.editing_id else {
        key_events.clear();
        return;
    };

    let mut changed = false;
    let mut stop_editing = false;

    for ev in key_events.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }
        match &ev.logical_key {
            Key::Enter | Key::Escape => {
                stop_editing = true;
            }
            Key::Backspace => {
                if let Some(wp) = waypoints.waypoints.iter_mut().find(|w| w.id == edit_id) {
                    wp.name.pop();
                    changed = true;
                }
            }
            Key::Space => {
                if let Some(wp) = waypoints.waypoints.iter_mut().find(|w| w.id == edit_id)
                    && wp.name.chars().count() < 20
                {
                    wp.name.push(' ');
                    changed = true;
                }
            }
            Key::Character(s) => {
                if let Some(wp) = waypoints.waypoints.iter_mut().find(|w| w.id == edit_id) {
                    for ch in s.chars() {
                        if !ch.is_control() && wp.name.chars().count() < 20 {
                            wp.name.push(ch);
                            changed = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if stop_editing {
        if let Some(wp) = waypoints.waypoints.iter_mut().find(|w| w.id == edit_id)
            && wp.name.trim().is_empty()
        {
            wp.name = format!("Waypoint #{}", wp.id);
        }
        waypoints.editing_id = None;
        waypoints.ui_dirty = true;
    } else if changed {
        waypoints.ui_dirty = true;
    }
}

fn handle_world_map_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    player_query: Query<&Transform, With<Player>>,
    loc: Option<Res<Localization>>,
    mut waypoints: ResMut<WorldWaypoints>,
    mut map_state: ResMut<WorldMapState>,
    mut ctx_menu: ResMut<WorldMapContextMenu>,
    mut cursor_mode: ResMut<CursorMode>,
    viewport_query: Query<(&UiGlobalTransform, &ComputedNode), With<WorldMapViewport>>,
    mut wp_list_query: Query<
        (&Hovered, &ComputedNode, &mut ScrollPosition),
        With<WorldMapWaypointListContainer>,
    >,
) {
    let Some(window) = window else {
        return;
    };

    let win_scale = window.scale_factor();
    let cursor_pos = window.cursor_position();

    // Placement Mode: LMB inside viewport drops waypoint at exact cursor coords; RMB cancels
    if waypoints.placing_mode {
        ctx_menu.open = false;
        if mouse_button.just_pressed(MouseButton::Right) {
            waypoints.placing_mode = false;
            return;
        }
        if mouse_button.just_pressed(MouseButton::Left)
            && let (Some(cursor), Ok((vp_tf, vp_node))) = (cursor_pos, viewport_query.single())
        {
            let inv = vp_node.inverse_scale_factor();
            let ui_cursor = cursor * win_scale * inv;
            let vp_size = vp_node.size() * inv;
            let vp_pos = vp_tf.translation * inv - vp_size * 0.5;
            let in_x = ui_cursor.x - vp_pos.x;
            let in_y = ui_cursor.y - vp_pos.y;
            if in_x >= 0.0 && in_x <= vp_size.x && in_y >= 0.0 && in_y <= vp_size.y {
                let half_w = (WORLD_MAP_WIDTH as f32) * 0.5;
                let half_h = (WORLD_MAP_HEIGHT as f32) * 0.5;
                let tex_aspect = (WORLD_MAP_WIDTH as f32) / (WORLD_MAP_HEIGHT as f32);
                let vp_aspect = vp_size.x / vp_size.y.max(1.0);
                let aspect_scale_x = (vp_aspect / tex_aspect).clamp(0.25, 4.0);
                let safe_zoom = map_state.zoom.max(0.05);
                let zoom_x = safe_zoom / aspect_scale_x;

                let tex_x = in_x / (vp_size.x / WORLD_MAP_WIDTH as f32);
                let tex_y = in_y / (vp_size.y / WORLD_MAP_HEIGHT as f32);
                let wx = (map_state.center.x + (tex_x - half_w) / zoom_x).floor() as i32;
                let wz = (map_state.center.y + (tex_y - half_h) / safe_zoom).floor() as i32;

                waypoints.add(loc.as_deref(), wx, wz);
                waypoints.placing_mode = false;
                return;
            }
        }
    }

    // Scroll waypoints list when hovering over it (and consume scroll so map doesn't zoom)
    let mut scrolled_wp_list = false;
    if let Ok((hovered, list_node, mut scroll_pos)) = wp_list_query.single_mut()
        && hovered.get()
        && scroll.delta.y.abs() > 0.001
    {
        let inv = list_node.inverse_scale_factor();
        let max_scroll = ((list_node.content_size().y - list_node.size().y) * inv).max(0.0);
        let step = match scroll.unit {
            bevy::input::mouse::MouseScrollUnit::Line => 36.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => 1.0,
        };
        scroll_pos.y = (scroll_pos.y - scroll.delta.y * step).clamp(0.0, max_scroll);
        scrolled_wp_list = true;
    }

    // Right-click inside viewport opens context menu at exact UI-scaled cursor coordinate
    if mouse_button.just_pressed(MouseButton::Right)
        && let (Some(cursor), Ok((vp_tf, vp_node))) = (cursor_pos, viewport_query.single())
    {
        let inv = vp_node.inverse_scale_factor();
        let ui_cursor = cursor * win_scale * inv;
        let vp_size = vp_node.size() * inv;
        let vp_pos = vp_tf.translation * inv - vp_size * 0.5;
        let in_x = ui_cursor.x - vp_pos.x;
        let in_y = ui_cursor.y - vp_pos.y;
        if in_x >= 0.0 && in_x <= vp_size.x && in_y >= 0.0 && in_y <= vp_size.y {
            let half_w = (WORLD_MAP_WIDTH as f32) * 0.5;
            let half_h = (WORLD_MAP_HEIGHT as f32) * 0.5;
            let tex_aspect = (WORLD_MAP_WIDTH as f32) / (WORLD_MAP_HEIGHT as f32);
            let vp_aspect = vp_size.x / vp_size.y.max(1.0);
            let aspect_scale_x = (vp_aspect / tex_aspect).clamp(0.25, 4.0);
            let safe_zoom = map_state.zoom.max(0.05);
            let zoom_x = safe_zoom / aspect_scale_x;

            let tex_x = in_x / (vp_size.x / WORLD_MAP_WIDTH as f32);
            let tex_y = in_y / (vp_size.y / WORLD_MAP_HEIGHT as f32);
            let wx = (map_state.center.x + (tex_x - half_w) / zoom_x).floor() as i32;
            let wz = (map_state.center.y + (tex_y - half_h) / safe_zoom).floor() as i32;

            ctx_menu.open = true;
            ctx_menu.viewport_pos = Vec2::new(
                in_x.clamp(8.0, (vp_size.x - 230.0).max(8.0)),
                in_y.clamp(8.0, (vp_size.y - 160.0).max(8.0)),
            );
            ctx_menu.world_x = wx;
            ctx_menu.world_z = wz;
            return;
        }
    }

    // Pause keyboard panning/centering while renaming a waypoint
    if waypoints.editing_id.is_none() {
        if keyboard.just_pressed(KeyCode::Space)
            && let Ok(player_transform) = player_query.single()
        {
            let p = player_transform.translation;
            map_state.center = Vec2::new(p.x / VOXEL_SIZE, p.z / VOXEL_SIZE);
        }

        let mut pan_dir = Vec2::ZERO;
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            pan_dir.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            pan_dir.x += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            pan_dir.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            pan_dir.y += 1.0;
        }
        let zoom = map_state.zoom;
        if pan_dir != Vec2::ZERO {
            map_state.center += pan_dir * (4.0 / zoom);
            ctx_menu.open = false;
        }
    }

    // 3. Mouse Zoom (only when not scrolling sidebar waypoint list)
    let scroll_y = scroll.delta.y;
    if !scrolled_wp_list && scroll_y.abs() > 0.001 {
        let zoom_factor = if scroll_y > 0.0 { 1.15 } else { 1.0 / 1.15 };
        map_state.zoom = (map_state.zoom * zoom_factor).clamp(0.20, 4.0);
        ctx_menu.open = false;
    }

    // 4. Mouse Left Click Drag Pan (disabled while context menu or placement mode is active)
    if mouse_button.pressed(MouseButton::Left) && !ctx_menu.open && !waypoints.placing_mode {
        if let (Some(curr), Some(prev)) = (cursor_pos, map_state.last_cursor_screen_pos) {
            let (in_vp, vp_scale_y) = if let Ok((vp_transform, computed)) = viewport_query.single()
            {
                let inv = computed.inverse_scale_factor();
                let ui_cursor = curr * win_scale * inv;
                let logical_size = computed.size() * inv;
                let vp_center = vp_transform.translation * inv;
                let vp_pos = vp_center - logical_size * 0.5;
                let in_x = ui_cursor.x - vp_pos.x;
                let in_y = ui_cursor.y - vp_pos.y;
                let is_inside =
                    in_x >= 0.0 && in_x <= logical_size.x && in_y >= 0.0 && in_y <= logical_size.y;
                let sy = (computed.size().y / win_scale) / (WORLD_MAP_HEIGHT as f32);
                let safe_sy = if sy.is_finite() && sy > 0.01 { sy } else { 1.0 };
                (is_inside, safe_sy)
            } else {
                (curr.y >= 80.0, 1.0)
            };

            if in_vp || map_state.is_dragging {
                let screen_delta = curr - prev;
                let current_zoom = if map_state.zoom.is_finite() && map_state.zoom > 0.05 {
                    map_state.zoom
                } else {
                    1.0
                };
                let denom = vp_scale_y * current_zoom;
                if denom.abs() > 0.001 {
                    let world_delta = screen_delta / denom;
                    if world_delta.x.is_finite() && world_delta.y.is_finite() {
                        map_state.center -= world_delta;
                    }
                }
                map_state.is_dragging = true;
                *cursor_mode = CursorMode::Grabbing;
            }
        }
        if !map_state.center.x.is_finite() || !map_state.center.y.is_finite() {
            map_state.center = Vec2::ZERO;
        }
        map_state.last_cursor_screen_pos = cursor_pos;
    } else {
        map_state.is_dragging = false;
        map_state.last_cursor_screen_pos = cursor_pos;
    }
}

fn sync_world_map_terrain(
    map_cache: Res<MapCache>,
    layers: Res<WorldMapLayers>,
    env_state: Option<Res<EnvironmentState>>,
    mut map_state: ResMut<WorldMapState>,
    mut images: ResMut<Assets<Image>>,
    viewport_query: Query<&ComputedNode, With<WorldMapViewport>>,
) {
    let tex_aspect = (WORLD_MAP_WIDTH as f32) / (WORLD_MAP_HEIGHT as f32);
    let vp_aspect = viewport_query
        .single()
        .ok()
        .map(|c| c.size())
        .filter(|s| s.x > 1.0 && s.y > 1.0)
        .map_or(tex_aspect, |s| s.x / s.y);

    let time_of_day = env_state.as_ref().map_or(0.5, |e| e.time_of_day);
    let time_bucket = if layers.follow_day_night {
        (time_of_day * 120.0) as i32
    } else {
        -1
    };

    let center_dist = (map_state.center - map_state.last_drawn_center).length();
    let zoom_diff = (map_state.zoom - map_state.last_drawn_zoom).abs();
    let aspect_diff = (vp_aspect - map_state.last_drawn_aspect).abs();
    let cache_changed = map_cache.version != map_state.last_cache_version;
    let layers_changed = *layers != map_state.last_drawn_layers;
    let time_changed = time_bucket != map_state.last_drawn_time_bucket;

    if center_dist < 0.25
        && zoom_diff < 0.005
        && aspect_diff < 0.005
        && !cache_changed
        && !layers_changed
        && !time_changed
    {
        return;
    }

    map_state.last_drawn_center = map_state.center;
    map_state.last_drawn_zoom = map_state.zoom;
    map_state.last_drawn_aspect = vp_aspect;
    map_state.last_cache_version = map_cache.version;
    map_state.last_drawn_layers = *layers;
    map_state.last_drawn_time_bucket = time_bucket;

    let Some(mut image) = images.get_mut(&map_state.terrain_image) else {
        return;
    };
    let Some(ref mut data) = image.data else {
        return;
    };

    let half_w = (WORLD_MAP_WIDTH as f32) / 2.0;
    let half_h = (WORLD_MAP_HEIGHT as f32) / 2.0;
    let zoom = if map_state.zoom.is_finite() && map_state.zoom > 0.05 {
        map_state.zoom
    } else {
        1.0
    };
    let aspect_scale_x = (vp_aspect / tex_aspect).clamp(0.25, 4.0);
    let zoom_x = zoom / aspect_scale_x;
    let center = if map_state.center.is_finite() {
        map_state.center
    } else {
        Vec2::ZERO
    };
    let tint = if layers.follow_day_night {
        daylight_tint(time_of_day)
    } else {
        [1.0, 1.0, 1.0]
    };

    for py in 0..WORLD_MAP_HEIGHT {
        let raw_z = center.y + (py as f32 - half_h) / zoom;
        let wz = if raw_z.is_finite() {
            raw_z.clamp(-2_000_000.0, 2_000_000.0).floor() as i32
        } else {
            0
        };
        let row_offset = (py as usize) * (WORLD_MAP_WIDTH as usize) * 4;

        let col_z = wz >> 4;
        let lz = (wz & 15) as usize;

        let north_wz = wz.saturating_sub(1);
        let north_col_z = north_wz >> 4;
        let north_lz = (north_wz & 15) as usize;

        let mut last_col_x = i32::MIN;
        let mut current_chunk: Option<&crate::map::cache::MapChunk> = None;
        let mut north_chunk: Option<&crate::map::cache::MapChunk> = None;

        for px in 0..WORLD_MAP_WIDTH {
            let raw_x = center.x + (px as f32 - half_w) / zoom_x;
            let wx = if raw_x.is_finite() {
                raw_x.clamp(-2_000_000.0, 2_000_000.0).floor() as i32
            } else {
                0
            };
            let idx = row_offset + (px as usize) * 4;

            let col_x = wx >> 4;
            let lx = (wx & 15) as usize;

            if col_x != last_col_x {
                last_col_x = col_x;
                current_chunk = map_cache.get_chunk(IVec2::new(col_x, col_z));
                north_chunk = if north_col_z == col_z {
                    current_chunk
                } else {
                    map_cache.get_chunk(IVec2::new(col_x, north_col_z))
                };
            }

            let mut color = if let Some(chunk) = current_chunk {
                let pixel = chunk.get(lx, lz);
                if pixel.voxel != Voxel::Air {
                    let north_h = north_chunk.map(|nc| nc.get(lx, north_lz).height);
                    let mut c = apply_relief_shading(pixel.color, pixel.height, north_h);

                    // Subtle topographic elevation contour lines on 8-block height band transitions
                    if layers.show_elevation
                        && pixel.water_depth == 0
                        && north_h.is_some_and(|nh| (pixel.height >> 3) != (nh >> 3))
                    {
                        c[0] = (c[0] as f32 * 0.88) as u8;
                        c[1] = (c[1] as f32 * 0.88) as u8;
                        c[2] = (c[2] as f32 * 0.88) as u8;
                    }

                    // Day/Night dynamic lighting tint
                    if layers.follow_day_night {
                        c[0] = (c[0] as f32 * tint[0]).clamp(0.0, 255.0) as u8;
                        c[1] = (c[1] as f32 * tint[1]).clamp(0.0, 255.0) as u8;
                        c[2] = (c[2] as f32 * tint[2]).clamp(0.0, 255.0) as u8;
                    }

                    // Subtle 16x16 Chunk Grid overlay
                    if layers.show_chunk_grid && (lx == 0 || lz == 0) {
                        c[0] = c[0].saturating_add(10);
                        c[1] = c[1].saturating_add(10);
                        c[2] = c[2].saturating_add(14);
                    }

                    c
                } else {
                    unexplored_color(wx, wz)
                }
            } else {
                unexplored_color(wx, wz)
            };

            if layers.show_chunk_grid && current_chunk.is_none() && (lx == 0 || lz == 0) {
                color[0] = color[0].saturating_add(5);
                color[1] = color[1].saturating_add(6);
                color[2] = color[2].saturating_add(8);
            }

            data[idx] = color[0];
            data[idx + 1] = color[1];
            data[idx + 2] = color[2];
            data[idx + 3] = color[3];
        }
    }
}

fn update_world_map_ui(
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    map_state: Res<WorldMapState>,
    layers: Res<WorldMapLayers>,
    waypoints: Res<WorldWaypoints>,
    asset_server: Res<AssetServer>,
    viewport_query: Query<(&ComputedNode, &UiGlobalTransform), With<WorldMapViewport>>,
    loc: Option<Res<Localization>>,
    mut marker_query: Query<
        (&mut Node, &mut UiTransform, &mut Visibility),
        (
            With<WorldMapPlayerMarker>,
            Without<WorldMapWaypointPin>,
            Without<WorldMapPlacementGhost>,
        ),
    >,
    mut wp_pins_query: Query<
        (&WorldMapWaypointPin, &mut Node, &mut Visibility),
        (
            Without<WorldMapPlayerMarker>,
            Without<WorldMapPlacementGhost>,
        ),
    >,
    mut ghost_query: Query<
        (&mut Node, &mut ImageNode, &mut Visibility),
        (
            With<WorldMapPlacementGhost>,
            Without<WorldMapPlayerMarker>,
            Without<WorldMapWaypointPin>,
        ),
    >,
    mut player_text_query: Query<&mut Text, (With<WorldMapInfoText>, Without<WorldMapCursorText>)>,
    mut cursor_text_query: Query<&mut Text, (With<WorldMapCursorText>, Without<WorldMapInfoText>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let p = player_transform.translation;
    let px_block = (p.x / VOXEL_SIZE).floor() as i32;
    let py_block = (p.y / VOXEL_SIZE).floor() as i32;
    let pz_block = (p.z / VOXEL_SIZE).floor() as i32;

    for mut text in &mut player_text_query {
        if let Some(ref l) = loc {
            **text = l.t_fmt(
                "map.player_pos",
                &[
                    &px_block.to_string(),
                    &py_block.to_string(),
                    &pz_block.to_string(),
                ],
            );
        } else {
            **text = format!("Player: X: {} Y: {} Z: {}", px_block, py_block, pz_block);
        }
    }

    let half_w = (WORLD_MAP_WIDTH as f32) / 2.0;
    let half_h = (WORLD_MAP_HEIGHT as f32) / 2.0;
    let tex_aspect = (WORLD_MAP_WIDTH as f32) / (WORLD_MAP_HEIGHT as f32);
    let safe_zoom = if map_state.zoom.is_finite() && map_state.zoom > 0.05 {
        map_state.zoom
    } else {
        1.0
    };
    let center = if map_state.center.is_finite() {
        map_state.center
    } else {
        Vec2::ZERO
    };

    let (vp_w, vp_h) = if let Ok((vp_node, _)) = viewport_query.single() {
        let logical = vp_node.size() * vp_node.inverse_scale_factor();
        if logical.x > 1.0 && logical.y > 1.0 {
            (logical.x, logical.y)
        } else {
            (WORLD_MAP_VIEWPORT_WIDTH, WORLD_MAP_VIEWPORT_HEIGHT)
        }
    } else {
        (WORLD_MAP_VIEWPORT_WIDTH, WORLD_MAP_VIEWPORT_HEIGHT)
    };

    let vp_aspect = vp_w / vp_h;
    let aspect_scale_x = (vp_aspect / tex_aspect).clamp(0.25, 4.0);
    let zoom_x = safe_zoom / aspect_scale_x;

    // 1. Player Marker
    let player_map_x = half_w + (p.x / VOXEL_SIZE - center.x) * zoom_x;
    let player_map_y = half_h + (p.z / VOXEL_SIZE - center.y) * safe_zoom;
    let u_pct = ((player_map_x / (WORLD_MAP_WIDTH as f32)) * 100.0).clamp(-20.0, 120.0);
    let v_pct = ((player_map_y / (WORLD_MAP_HEIGHT as f32)) * 100.0).clamp(-20.0, 120.0);

    let marker_visible = layers.show_player_marker
        && player_map_x >= 0.0
        && player_map_x <= WORLD_MAP_WIDTH as f32
        && player_map_y >= 0.0
        && player_map_y <= WORLD_MAP_HEIGHT as f32;

    let yaw = if let Ok(camera_transform) = camera_query.single() {
        let (y, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        if y.is_finite() { y } else { 0.0 }
    } else {
        0.0
    };

    for (mut node, mut ui_transform, mut vis) in &mut marker_query {
        node.left = percent(u_pct);
        node.top = percent(v_pct);
        ui_transform.translation = Val2::ZERO;
        ui_transform.scale = Vec2::ONE;
        ui_transform.rotation = Rot2::radians(-yaw);
        *vis = if marker_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // 2. Waypoint Pins (percentage-locked at every zoom)
    for (pin, mut node, mut vis) in &mut wp_pins_query {
        let map_x = half_w + (pin.world_x as f32 - center.x) * zoom_x;
        let map_y = half_h + (pin.world_z as f32 - center.y) * safe_zoom;
        let visible = layers.show_waypoints
            && map_x >= 0.0
            && map_x <= WORLD_MAP_WIDTH as f32
            && map_y >= 0.0
            && map_y <= WORLD_MAP_HEIGHT as f32;
        if visible {
            node.left = percent((map_x / (WORLD_MAP_WIDTH as f32)) * 100.0);
            node.top = percent((map_y / (WORLD_MAP_HEIGHT as f32)) * 100.0);
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // 3. Cursor Coordinates & Waypoint Placement Ghost (UI-scale corrected)
    let mut ghost_shown = false;
    if let (Some(window), Ok((vp_node, vp_transform))) = (window, viewport_query.single()) {
        let inv_scale = vp_node.inverse_scale_factor();
        let vp_size = vp_node.size() * inv_scale;
        if vp_size.x > 0.0 && vp_size.y > 0.0 {
            let vp_pos = vp_transform.translation * inv_scale - vp_size / 2.0;
            if let Some(cursor) = window.cursor_position() {
                let ui_cursor = cursor * window.scale_factor() * inv_scale;
                let in_vp_x = ui_cursor.x - vp_pos.x;
                let in_vp_y = ui_cursor.y - vp_pos.y;

                if in_vp_x >= 0.0 && in_vp_x <= vp_size.x && in_vp_y >= 0.0 && in_vp_y <= vp_size.y
                {
                    if waypoints.placing_mode {
                        let icon = asset_server.load(waypoints.selected_kind.icon_path());
                        for (mut g_node, mut g_img, mut g_vis) in &mut ghost_query {
                            g_node.left = px(in_vp_x);
                            g_node.top = px(in_vp_y);
                            if g_img.image != icon {
                                g_img.image = icon.clone();
                            }
                            *g_vis = Visibility::Visible;
                        }
                        ghost_shown = true;
                    }

                    let scale_x = vp_size.x / (WORLD_MAP_WIDTH as f32);
                    let scale_y = vp_size.y / (WORLD_MAP_HEIGHT as f32);
                    let tex_x = in_vp_x / scale_x;
                    let tex_y = in_vp_y / scale_y;

                    let raw_cx = center.x + (tex_x - half_w) / zoom_x;
                    let raw_cz = center.y + (tex_y - half_h) / safe_zoom;
                    let cursor_world_x = if raw_cx.is_finite() {
                        raw_cx.clamp(-2_000_000.0, 2_000_000.0).floor() as i32
                    } else {
                        0
                    };
                    let cursor_world_z = if raw_cz.is_finite() {
                        raw_cz.clamp(-2_000_000.0, 2_000_000.0).floor() as i32
                    } else {
                        0
                    };

                    for mut text in &mut cursor_text_query {
                        if let Some(ref l) = loc {
                            **text = l.t_fmt(
                                "map.cursor_pos",
                                &[
                                    &cursor_world_x.to_string(),
                                    &cursor_world_z.to_string(),
                                    &format!("{:.2}", safe_zoom),
                                ],
                            );
                        } else {
                            **text = format!(
                                "Cursor: X: {} Z: {} | Zoom: {:.2}x",
                                cursor_world_x, cursor_world_z, safe_zoom
                            );
                        }
                    }
                    if !ghost_shown {
                        for (_, _, mut g_vis) in &mut ghost_query {
                            *g_vis = Visibility::Hidden;
                        }
                    }
                    return;
                }
            }
        }
    }

    if !ghost_shown {
        for (_, _, mut g_vis) in &mut ghost_query {
            *g_vis = Visibility::Hidden;
        }
    }

    for mut text in &mut cursor_text_query {
        if let Some(ref l) = loc {
            **text = l.t_fmt("map.zoom", &[&format!("{:.2}", safe_zoom)]);
        } else {
            **text = format!("Zoom: {:.2}x", safe_zoom);
        }
    }
}
