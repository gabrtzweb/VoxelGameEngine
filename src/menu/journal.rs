use std::collections::HashSet;

use bevy::{
    input::mouse::AccumulatedMouseScroll,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::Button,
};

use crate::{
    core::{AppFont, Localization, text_shadow_default},
    environment::{EnvironmentState, weather::WeatherManager},
    gameplay::{
        BlockIcons,
        feedback::{BlockBreakEvent, BlockPlaceEvent},
        item_drops::SpawnItemDropEvent,
    },
    generation::BiomeType,
    player::{Player, PlayerEnvironmentStatus, PlayerInventory, PlayerMotion, hotbar::Hotbar},
    world::{Voxel, WorldSession},
};

use super::{
    GameSettings, MenuState, creative_inventory::CreativeCategory, menu_bar::MENU_BAR_HEIGHT,
    theme::palette,
};

#[derive(Resource, Default)]
pub struct JournalProgress {
    pub time_played_secs: f32,
    pub blocks_placed: u64,
    pub blocks_broken: u64,
    pub ores_mined: u64,
    pub timber_chopped: u64,
    pub crops_harvested: u64,
    pub items_dropped: u64,
    pub distance_traveled: f32,
    pub distance_swum: f32,
    pub jumps_performed: u64,
    pub max_altitude: Option<i32>,
    pub min_altitude: Option<i32>,
    was_grounded: bool,
    last_player_pos: Option<Vec3>,
    pub discovered_biomes: HashSet<BiomeType>,
    pub discovered_voxels: HashSet<Voxel>,
}

impl JournalProgress {
    fn record_voxel(&mut self, voxel: Voxel) {
        if voxel != Voxel::Air {
            self.discovered_voxels.insert(voxel);
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JournalTab {
    #[default]
    Biomes,
    Bestiary,
    Flora,
    Blocks,
    Items,
    Statistics,
}

impl JournalTab {
    const COMPENDIUM_TABS: [Self; 5] = [
        Self::Biomes,
        Self::Bestiary,
        Self::Flora,
        Self::Blocks,
        Self::Items,
    ];

    fn loc_key(self) -> &'static str {
        match self {
            Self::Biomes => "journal.tab.biomes",
            Self::Bestiary => "journal.tab.bestiary",
            Self::Flora => "journal.tab.flora",
            Self::Blocks => "journal.tab.blocks",
            Self::Items => "journal.tab.items",
            Self::Statistics => "journal.tab.statistics",
        }
    }

    fn fallback_label(self) -> &'static str {
        match self {
            Self::Biomes => "Biomes & Regions",
            Self::Bestiary => "Bestiary",
            Self::Flora => "Flora & Agriculture",
            Self::Blocks => "Blocks & Minerals",
            Self::Items => "Items & Equipment",
            Self::Statistics => "Statistics",
        }
    }
}

const BIOME_CATEGORY_ORDER: [&str; 7] = [
    "journal.biome_cat.peaks",
    "journal.biome_cat.canyons",
    "journal.biome_cat.arid",
    "journal.biome_cat.forests",
    "journal.biome_cat.glacial",
    "journal.biome_cat.coastal",
    "journal.biome_cat.caves",
];

#[derive(Component)]
struct JournalMenuRoot;

#[derive(Component, Clone, Copy)]
struct JournalNavButton(JournalTab);

#[derive(Component)]
struct JournalNavLabel;

#[derive(Component)]
struct JournalNavAccent;

#[derive(Component, Clone, Copy)]
struct JournalSection(JournalTab);

#[derive(Component)]
struct JournalSeasonText;

#[derive(Component)]
struct JournalDayText;

#[derive(Component)]
struct JournalTimeText;

#[derive(Component)]
struct JournalTimeIcon;

#[derive(Component)]
struct JournalWeatherText;

#[derive(Component)]
struct JournalWeatherIcon;

#[derive(Component)]
struct JournalScrollArea;

#[derive(Component)]
struct JournalScrollbarTrack;

#[derive(Component)]
struct JournalScrollbarThumb;

#[derive(Resource, Default)]
struct JournalScrollDrag {
    dragging: bool,
}

pub struct JournalMenuPlugin;

impl Plugin for JournalMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JournalTab>()
            .init_resource::<JournalProgress>()
            .init_resource::<JournalScrollDrag>()
            .add_observer(on_journal_block_break)
            .add_observer(on_journal_block_place)
            .add_observer(on_journal_item_drop)
            .add_systems(OnEnter(MenuState::Journal), spawn_journal_menu)
            .add_systems(OnExit(MenuState::Journal), despawn_journal_menu)
            .add_systems(Update, track_journal_progress)
            .add_systems(
                Update,
                (
                    handle_journal_nav_clicks,
                    update_journal_tab_view,
                    handle_journal_scroll_and_scrollbar,
                    update_journal_almanac,
                )
                    .run_if(in_state(MenuState::Journal)),
            );
    }
}

fn on_journal_block_break(
    trigger: On<BlockBreakEvent>,
    mut progress: ResMut<JournalProgress>,
) {
    let voxel = trigger.event().voxel;
    if voxel == Voxel::Air {
        return;
    }
    progress.blocks_broken += 1;
    progress.record_voxel(voxel);
    if CreativeCategory::Ores.matches(voxel) {
        progress.ores_mined += 1;
    }
    if CreativeCategory::Trees.matches(voxel) {
        progress.timber_chopped += 1;
    }
    if CreativeCategory::Vegetation.matches(voxel) {
        progress.crops_harvested += 1;
    }
}

fn on_journal_block_place(
    trigger: On<BlockPlaceEvent>,
    mut progress: ResMut<JournalProgress>,
) {
    let voxel = trigger.event().voxel;
    if voxel == Voxel::Air {
        return;
    }
    progress.blocks_placed += 1;
    progress.record_voxel(voxel);
}

fn on_journal_item_drop(
    trigger: On<SpawnItemDropEvent>,
    mut progress: ResMut<JournalProgress>,
) {
    let voxel = trigger.event().voxel;
    if voxel == Voxel::Air || voxel.is_fluid() {
        return;
    }
    progress.items_dropped += 1;
    progress.record_voxel(voxel);
}

fn track_journal_progress(
    time: Res<Time>,
    world_session: Option<Res<WorldSession>>,
    player_q: Query<(&Transform, &PlayerMotion), With<Player>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    hotbar: Option<Res<Hotbar>>,
    inventory: Option<Res<PlayerInventory>>,
    mut progress: ResMut<JournalProgress>,
) {
    if !world_session.as_ref().is_some_and(|s| s.is_active) {
        progress.last_player_pos = None;
        return;
    }

    progress.time_played_secs += time.delta_secs();

    let in_water = env_status.as_ref().is_some_and(|e| e.submersion > 0.25);

    if let Ok((tf, motion)) = player_q.single() {
        let pos = tf.translation;
        let y_int = pos.y.round() as i32;
        progress.max_altitude = Some(progress.max_altitude.map_or(y_int, |m| m.max(y_int)));
        progress.min_altitude = Some(progress.min_altitude.map_or(y_int, |m| m.min(y_int)));

        if progress.was_grounded && !motion.grounded && !motion.flying && motion.velocity.y > 2.0 {
            progress.jumps_performed += 1;
        }
        progress.was_grounded = motion.grounded;

        if let Some(prev) = progress.last_player_pos {
            let delta = Vec2::new(pos.x - prev.x, pos.z - prev.z).length();
            if delta > 0.001 && delta < 10.0 {
                if in_water {
                    progress.distance_swum += delta;
                } else {
                    progress.distance_traveled += delta;
                }
            }
        }
        progress.last_player_pos = Some(pos);
    }

    if let Some(env) = env_status {
        progress.discovered_biomes.insert(env.current_biome);
    }

    if let Some(hb) = hotbar
        && (hb.is_changed() || progress.discovered_voxels.is_empty())
    {
        for voxel in hb.slots.into_iter().flatten() {
            progress.record_voxel(voxel);
        }
    }

    if let Some(inv) = inventory
        && inv.is_changed()
    {
        for voxel in inv.slots.into_iter().flatten() {
            progress.record_voxel(voxel);
        }
        if let Some(v) = inv.offhand_slot {
            progress.record_voxel(v);
        }
        if let Some(v) = inv.utility_slot {
            progress.record_voxel(v);
        }
    }
}

fn format_duration(secs: f32) -> String {
    let total = secs as u64;
    let hours = total / 3600;
    let mins = (total % 3600) / 60;
    let rem_secs = total % 60;
    if hours > 0 {
        format!("{hours:02}h {mins:02}m")
    } else if mins > 0 {
        format!("{mins}m {rem_secs:02}s")
    } else {
        format!("{rem_secs}s")
    }
}

fn voxel_matches_journal_tab(tab: JournalTab, voxel: Voxel) -> bool {
    voxel_journal_subcategory(tab, voxel).is_some()
}

fn voxel_journal_subcategory(tab: JournalTab, voxel: Voxel) -> Option<&'static str> {
    let id = voxel.id();
    let tex = voxel.texture_name().unwrap_or("");
    match tab {
        JournalTab::Flora => {
            if CreativeCategory::Organics.matches(voxel) {
                Some("journal.flora_cat.organics")
            } else if CreativeCategory::Vegetation.matches(voxel) {
                if tex.starts_with("fungi_") || id.contains("fungi") || id.contains("mushroom") {
                    Some("journal.flora_cat.mushrooms")
                } else if tex.starts_with("aqua_")
                    || id.contains("coral")
                    || id.contains("kelp")
                    || id.contains("seagrass")
                    || id.contains("algae")
                {
                    Some("journal.flora_cat.aquatic")
                } else {
                    Some("journal.flora_cat.plants")
                }
            } else {
                None
            }
        }
        JournalTab::Blocks => {
            if CreativeCategory::Ores.matches(voxel) {
                Some("journal.blocks_cat.ores")
            } else if CreativeCategory::Trees.matches(voxel) {
                Some("journal.blocks_cat.timber")
            } else if CreativeCategory::Soils.matches(voxel) {
                Some("journal.blocks_cat.soils")
            } else if CreativeCategory::Rocks.matches(voxel) {
                Some("journal.blocks_cat.rocks")
            } else {
                None
            }
        }
        JournalTab::Items => {
            if CreativeCategory::Lighting.matches(voxel) {
                Some("journal.items_cat.lighting")
            } else if CreativeCategory::Decorations.matches(voxel) {
                Some("journal.items_cat.decorations")
            } else {
                None
            }
        }
        _ => None,
    }
}

fn journal_subcategories_for(tab: JournalTab) -> &'static [(&'static str, &'static str)] {
    match tab {
        JournalTab::Flora => &[
            ("journal.flora_cat.plants", "Plants & Crops"),
            ("journal.flora_cat.mushrooms", "Fungi & Mushrooms"),
            ("journal.flora_cat.aquatic", "Aquatic Flora & Corals"),
            ("journal.flora_cat.organics", "Organics & Nests"),
        ],
        JournalTab::Blocks => &[
            ("journal.blocks_cat.rocks", "Rocks & Stone"),
            ("journal.blocks_cat.soils", "Soils & Sands"),
            ("journal.blocks_cat.ores", "Ores, Gems & Crystals"),
            ("journal.blocks_cat.timber", "Logs & Timber"),
        ],
        JournalTab::Items => &[
            ("journal.items_cat.lighting", "Lighting & Fire"),
            ("journal.items_cat.decorations", "Decorations & Containers"),
        ],
        _ => &[],
    }
}

fn format_distance(meters: f32) -> String {
    if meters >= 1000.0 {
        format!("{:.2} km", meters / 1000.0)
    } else {
        format!("{} m", meters.round() as u64)
    }
}

fn season_loc_key(day_num: u32) -> &'static str {
    match (day_num.saturating_sub(1) / 30) % 4 {
        0 => "journal.season.spring",
        1 => "journal.season.summer",
        2 => "journal.season.autumn",
        _ => "journal.season.winter",
    }
}

fn weather_loc_key(weather_debug: &str) -> &'static str {
    match weather_debug {
        "Rain" => "journal.weather.rain",
        "HeavyRain" => "journal.weather.heavy_rain",
        "Thunderstorm" => "journal.weather.thunderstorm",
        "Snow" => "journal.weather.snow",
        "Blizzard" => "journal.weather.blizzard",
        "DustStorm" => "journal.weather.dust_storm",
        _ => "journal.weather.clear",
    }
}

fn weather_icon_for(weather_name: &str) -> &'static str {
    match weather_name {
        "Rain" => "textures/interfaces/icons/weather_raining.png",
        "HeavyRain" => "textures/interfaces/icons/weather_heavy_rain.png",
        "Thunderstorm" => "textures/interfaces/icons/weather_thunder.png",
        "Snow" => "textures/interfaces/icons/weather_snow.png",
        "Blizzard" => "textures/interfaces/icons/weather_blizzard.png",
        "DustStorm" => "textures/interfaces/icons/weather_dust_storm.png",
        _ => "textures/interfaces/icons/weather_clear.png",
    }
}

fn time_icon_for(hour: u32) -> &'static str {
    match hour {
        5..=10 => "textures/interfaces/icons/time_morning.png",
        11..=16 => "textures/interfaces/icons/time_noon.png",
        17..=20 => "textures/interfaces/icons/time_evening.png",
        _ => "textures/interfaces/icons/time_night.png",
    }
}

fn spawn_nav_item(
    parent: &mut ChildSpawnerCommands,
    tab: JournalTab,
    is_active: bool,
    label: &str,
    font: TextFont,
    arrow_font: TextFont,
) {
    parent
        .spawn((
            Button,
            Hovered::default(),
            JournalNavButton(tab),
            Node {
                width: percent(100.0),
                height: px(44.0),
                padding: UiRect::new(px(26.0), px(22.0), px(0.0), px(0.0)),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(if is_active {
                palette::NAV_ITEM_ACTIVE_BG
            } else {
                Color::NONE
            }),
        ))
        .with_children(|btn| {
            btn.spawn((
                JournalNavAccent,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(7.0),
                    bottom: px(7.0),
                    width: px(3.0),
                    border_radius: BorderRadius::right(px(2.0)),
                    ..default()
                },
                BackgroundColor(palette::ACCENT_LINE),
                if is_active {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ));

            btn.spawn((
                JournalNavLabel,
                Text::new(label),
                font,
                TextColor(if is_active {
                    palette::TEXT_ACCENT
                } else {
                    palette::TEXT_PRIMARY
                }),
                text_shadow_default(),
            ));

            btn.spawn((
                Text::new(">"),
                arrow_font,
                TextColor(palette::TEXT_MUTED),
            ));
        });
}

fn spawn_journal_menu(
    mut commands: Commands,
    active_tab: Res<JournalTab>,
    progress: Res<JournalProgress>,
    env_state: Option<Res<EnvironmentState>>,
    weather: Option<Res<WeatherManager>>,
    loc: Option<Res<Localization>>,
    block_icons: Option<Res<BlockIcons>>,
    asset_server: Res<AssetServer>,
    app_font: Option<Res<AppFont>>,
    game_settings: Option<Res<GameSettings>>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source());
    let make_font = |size: f32| {
        let mut tf = TextFont {
            font_size: FontSize::Px(size),
            ..default()
        };
        if let Some(ref font) = font_handle {
            tf.font = font.clone();
        }
        tf
    };

    let t = |key: &str, fallback: &str| -> String {
        loc.as_ref()
            .map_or_else(|| fallback.to_string(), |l| l.t(key).to_string())
    };

    let dimming_alpha = game_settings.map_or(0.65, |s| s.background_dimming.alpha());

    let day_num = env_state.as_ref().map_or(1, |e| e.day_count);
    let season_str = t(season_loc_key(day_num), "Spring").to_uppercase();
    let day_lbl = t("journal.day", "Day");
    let time_of_day = env_state.as_ref().map_or(0.5, |e| e.time_of_day);
    let total_mins = (time_of_day * 24.0 * 60.0) as u32 % (24 * 60);
    let hour = total_mins / 60;
    let time_str = format!("{:02}:{:02}", hour, total_mins % 60);
    let time_icon = asset_server.load(time_icon_for(hour));
    let weather_debug = weather
        .as_ref()
        .map_or("Clear".to_string(), |w| format!("{:?}", w.current));
    let weather_str = t(weather_loc_key(&weather_debug), &weather_debug);
    let weather_icon = asset_server.load(weather_icon_for(&weather_debug));

    let compendium_header = t("journal.compendium", "COMPENDIUM");
    let entry_lbl = t("journal.entry", "Entry");
    let undiscovered_lbl = t("journal.undiscovered", "Undiscovered");
    let discovered_lbl = t("journal.discovered", "Discovered");
    let empty_catalog_lbl = t(
        "journal.empty_catalog",
        "No entries discovered yet. Collect or place items in your inventory to record them here.",
    );

    commands
        .spawn((
            JournalMenuRoot,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, dimming_alpha)),
            ZIndex(300),
        ))
        .with_children(|root| {
            // Left Column: Almanac + Compendium Header + Sub-tabs + Statistics
            root.spawn((
                Node {
                    width: px(palette::DRAWER_COL1_WIDTH),
                    min_width: px(palette::DRAWER_COL1_WIDTH),
                    flex_shrink: 0.0,
                    height: percent(100.0),
                    border: UiRect::right(px(1.5)),
                    padding: UiRect::vertical(px(20.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2.0),
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL1_BG),
                BorderColor::all(palette::DRAWER_DIVIDER),
            ))
            .with_children(|nav| {
                // Compact World Almanac Card at top
                nav.spawn((
                    Node {
                        margin: UiRect::new(px(18.0), px(18.0), px(0.0), px(16.0)),
                        padding: UiRect::all(px(12.0)),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(6.0)),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8.0),
                        ..default()
                    },
                    BackgroundColor(palette::PANEL_INNER_BG),
                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                ))
                .with_children(|almanac| {
                    almanac
                        .spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                JournalSeasonText,
                                Text::new(season_str),
                                make_font(13.0),
                                TextColor(palette::TEXT_ACCENT),
                                text_shadow_default(),
                            ));
                            row.spawn((
                                JournalDayText,
                                Text::new(format!("{day_lbl} {day_num}")),
                                make_font(13.0),
                                TextColor(palette::TEXT_MUTED),
                            ));
                        });

                    almanac
                        .spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(6.0),
                                ..default()
                            })
                            .with_children(|time_row| {
                                time_row.spawn((
                                    JournalTimeIcon,
                                    ImageNode {
                                        image: time_icon,
                                        ..default()
                                    },
                                    Node {
                                        width: px(18.0),
                                        height: px(18.0),
                                        ..default()
                                    },
                                ));
                                time_row.spawn((
                                    JournalTimeText,
                                    Text::new(time_str),
                                    make_font(13.0),
                                    TextColor(palette::TEXT_PRIMARY),
                                ));
                            });

                            row.spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(6.0),
                                ..default()
                            })
                            .with_children(|weather_row| {
                                weather_row.spawn((
                                    JournalWeatherIcon,
                                    ImageNode {
                                        image: weather_icon,
                                        ..default()
                                    },
                                    Node {
                                        width: px(18.0),
                                        height: px(18.0),
                                        ..default()
                                    },
                                ));
                                weather_row.spawn((
                                    JournalWeatherText,
                                    Text::new(weather_str),
                                    make_font(13.0),
                                    TextColor(palette::TEXT_PRIMARY),
                                ));
                            });
                        });
                });

                // COMPENDIUM section header
                nav.spawn((
                    Text::new(compendium_header.to_uppercase()),
                    make_font(11.0),
                    TextColor(palette::TEXT_MUTED),
                    Node {
                        margin: UiRect::new(px(26.0), px(22.0), px(4.0), px(6.0)),
                        ..default()
                    },
                ));

                for tab in JournalTab::COMPENDIUM_TABS {
                    let lbl = t(tab.loc_key(), tab.fallback_label());
                    spawn_nav_item(
                        nav,
                        tab,
                        tab == *active_tab,
                        &lbl,
                        make_font(15.0),
                        make_font(14.0),
                    );
                }

                // Divider before Statistics
                nav.spawn((
                    Node {
                        height: px(1.0),
                        margin: UiRect::new(px(20.0), px(20.0), px(12.0), px(10.0)),
                        ..default()
                    },
                    BackgroundColor(palette::DRAWER_DIVIDER),
                ));

                let stats_lbl = t(
                    JournalTab::Statistics.loc_key(),
                    JournalTab::Statistics.fallback_label(),
                );
                spawn_nav_item(
                    nav,
                    JournalTab::Statistics,
                    *active_tab == JournalTab::Statistics,
                    &stats_lbl,
                    make_font(15.0),
                    make_font(14.0),
                );
            });

            // Right Main Panel Wrapper (Scrollable content + Right-edge Scrollbar)
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    height: percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL2_BG),
            ))
            .with_children(|right_wrap| {
                // Scrollable Journal Content Area
                right_wrap
                    .spawn((
                        JournalScrollArea,
                        ScrollPosition::default(),
                        Node {
                            flex_grow: 1.0,
                            height: percent(100.0),
                            padding: UiRect::new(px(32.0), px(20.0), px(32.0), px(32.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                    ))
                    .with_children(|main| {
                        // 1. Biomes & Regions (44 biomes grouped by climate, visited biomes highlighted)
                        main.spawn((
                            JournalSection(JournalTab::Biomes),
                            Node {
                                width: percent(100.0),
                                display: if *active_tab == JournalTab::Biomes {
                                    Display::Flex
                                } else {
                                    Display::None
                                },
                                flex_direction: FlexDirection::Column,
                                row_gap: px(20.0),
                                padding: UiRect::bottom(px(32.0)),
                                ..default()
                            },
                        ))
                        .with_children(|sec| {
                            sec.spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(4.0),
                                ..default()
                            })
                            .with_children(|hdr| {
                                hdr.spawn(Node {
                                    width: percent(100.0),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .with_children(|top| {
                                    top.spawn((
                                        Text::new(
                                            t("journal.tab.biomes", "Biomes & Regions")
                                                .to_uppercase(),
                                        ),
                                        make_font(18.0),
                                        TextColor(palette::TEXT_ACCENT),
                                        text_shadow_default(),
                                    ));
                                    top.spawn((
                                        Text::new(format!(
                                            "{discovered_lbl}: {} / {}",
                                            progress.discovered_biomes.len(),
                                            BiomeType::ALL.len()
                                        )),
                                        make_font(13.0),
                                        TextColor(palette::TEXT_ACCENT),
                                    ));
                                });
                                hdr.spawn((
                                    Text::new(t(
                                        "journal.biomes.subtitle",
                                        "44 distinct surface, aquatic, and subterranean climates across the world.",
                                    )),
                                    make_font(13.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                            });

                            let mut any_biome = false;
                            for cat_key in BIOME_CATEGORY_ORDER {
                                let cat_biomes: Vec<BiomeType> = BiomeType::ALL
                                    .iter()
                                    .copied()
                                    .filter(|b| {
                                        b.category_key() == cat_key
                                            && progress.discovered_biomes.contains(b)
                                    })
                                    .collect();
                                if cat_biomes.is_empty() {
                                    continue;
                                }
                                any_biome = true;
                                let cat_title = t(cat_key, cat_key);
                                sec.spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(10.0),
                                    ..default()
                                })
                                .with_children(|group| {
                                    group.spawn((
                                        Text::new(cat_title.to_uppercase()),
                                        make_font(12.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));

                                    group
                                        .spawn(Node {
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Row,
                                            flex_wrap: FlexWrap::Wrap,
                                            column_gap: px(12.0),
                                            row_gap: px(12.0),
                                            ..default()
                                        })
                                        .with_children(|grid| {
                                            for biome in cat_biomes {
                                                let biome_name =
                                                    t(biome.translation_key(), biome.name());
                                                let icon_handle =
                                                    asset_server.load(biome.icon_asset_path());

                                                grid.spawn((
                                                    Node {
                                                        width: px(235.0),
                                                        padding: UiRect::axes(px(12.0), px(10.0)),
                                                        border: UiRect::all(px(1.5)),
                                                        border_radius: BorderRadius::all(px(6.0)),
                                                        display: Display::Flex,
                                                        flex_direction: FlexDirection::Row,
                                                        align_items: AlignItems::Center,
                                                        column_gap: px(12.0),
                                                        ..default()
                                                    },
                                                    BackgroundColor(palette::PANEL_INNER_BG),
                                                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                                                ))
                                                .with_children(|card| {
                                                    card.spawn((
                                                        ImageNode {
                                                            image: icon_handle,
                                                            image_mode: NodeImageMode::Stretch,
                                                            ..default()
                                                        },
                                                        Node {
                                                            width: px(36.0),
                                                            height: px(36.0),
                                                            flex_shrink: 0.0,
                                                            ..default()
                                                        },
                                                    ));

                                                    card.spawn((
                                                        Text::new(biome_name),
                                                        make_font(13.0),
                                                        TextColor(palette::TEXT_PRIMARY),
                                                        text_shadow_default(),
                                                    ));
                                                });
                                            }
                                        });
                                });
                            }

                            if !any_biome {
                                sec.spawn((
                                    Node {
                                        width: percent(100.0),
                                        padding: UiRect::all(px(24.0)),
                                        border: UiRect::all(px(1.0)),
                                        border_radius: BorderRadius::all(px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(palette::PANEL_INNER_BG),
                                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                                ))
                                .with_children(|empty| {
                                    empty.spawn((
                                        Text::new(empty_catalog_lbl.clone()),
                                        make_font(13.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));
                                });
                            }
                        });

                        // 2. Bestiary (localized subtitle + placeholder cards)
                        main.spawn((
                            JournalSection(JournalTab::Bestiary),
                            Node {
                                width: percent(100.0),
                                display: if *active_tab == JournalTab::Bestiary {
                                    Display::Flex
                                } else {
                                    Display::None
                                },
                                flex_direction: FlexDirection::Column,
                                row_gap: px(16.0),
                                ..default()
                            },
                        ))
                        .with_children(|sec| {
                            sec.spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(4.0),
                                ..default()
                            })
                            .with_children(|hdr| {
                                hdr.spawn((
                                    Text::new(
                                        t("journal.tab.bestiary", "Bestiary").to_uppercase(),
                                    ),
                                    make_font(18.0),
                                    TextColor(palette::TEXT_ACCENT),
                                    text_shadow_default(),
                                ));
                                hdr.spawn((
                                    Text::new(t(
                                        "journal.bestiary.subtitle",
                                        "Creatures tamed or defeated across the world will be unlocked and recorded here.",
                                    )),
                                    make_font(13.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                            });

                            sec.spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: px(14.0),
                                row_gap: px(14.0),
                                ..default()
                            })
                            .with_children(|grid| {
                                for idx in 1..=8 {
                                    grid.spawn((
                                        Node {
                                            width: px(170.0),
                                            height: px(110.0),
                                            padding: UiRect::all(px(14.0)),
                                            border: UiRect::all(px(1.5)),
                                            border_radius: BorderRadius::all(px(6.0)),
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::SpaceBetween,
                                            ..default()
                                        },
                                        BackgroundColor(palette::PANEL_INNER_BG),
                                        BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                                    ))
                                    .with_children(|card| {
                                        card.spawn((
                                            Text::new(format!("{entry_lbl} #{idx:02}")),
                                            make_font(11.0),
                                            TextColor(palette::TEXT_MUTED),
                                        ));
                                        card.spawn((
                                            Text::new("???"),
                                            make_font(18.0),
                                            TextColor(Color::srgba(0.62, 0.66, 0.74, 0.55)),
                                        ));
                                        card.spawn((
                                            Text::new(undiscovered_lbl.clone()),
                                            make_font(11.0),
                                            TextColor(palette::TEXT_MUTED),
                                        ));
                                    });
                                }
                            });
                        });

                        // 3. Live Discovered Catalogs: Flora, Blocks & Minerals, Items & Equipment
                        let catalog_tabs = [
                            (
                                JournalTab::Flora,
                                "journal.flora.subtitle",
                                "Plants, crops, mushrooms, and wild flora collected or placed in your inventory.",
                            ),
                            (
                                JournalTab::Blocks,
                                "journal.blocks.subtitle",
                                "Rocks, soils, ores, gems, and timber collected or placed in your inventory.",
                            ),
                            (
                                JournalTab::Items,
                                "journal.items.subtitle",
                                "Torches, lanterns, campfires, tools, and crafted decorations discovered so far.",
                            ),
                        ];

                        for (tab, sub_key, sub_fallback) in catalog_tabs {
                            let tab_title = t(tab.loc_key(), tab.fallback_label());
                            let mut discovered: Vec<Voxel> = progress
                                .discovered_voxels
                                .iter()
                                .copied()
                                .filter(|&v| voxel_matches_journal_tab(tab, v))
                                .collect();
                            discovered.sort_by_key(|v| *v as u16);

                            let total_in_tab = Voxel::ALL
                                .iter()
                                .copied()
                                .filter(|&v| v != Voxel::Air && voxel_matches_journal_tab(tab, v))
                                .count();

                            main.spawn((
                                JournalSection(tab),
                                Node {
                                    width: percent(100.0),
                                    display: if *active_tab == tab {
                                        Display::Flex
                                    } else {
                                        Display::None
                                    },
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(16.0),
                                    padding: UiRect::bottom(px(32.0)),
                                    ..default()
                                },
                            ))
                            .with_children(|sec| {
                                sec.spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(4.0),
                                    ..default()
                                })
                                .with_children(|hdr| {
                                    hdr.spawn(Node {
                                        width: percent(100.0),
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    })
                                    .with_children(|top| {
                                        top.spawn((
                                            Text::new(tab_title.to_uppercase()),
                                            make_font(18.0),
                                            TextColor(palette::TEXT_ACCENT),
                                            text_shadow_default(),
                                        ));
                                        top.spawn((
                                            Text::new(format!(
                                                "{discovered_lbl}: {} / {total_in_tab}",
                                                discovered.len()
                                            )),
                                            make_font(13.0),
                                            TextColor(palette::TEXT_ACCENT),
                                        ));
                                    });
                                    hdr.spawn((
                                        Text::new(t(sub_key, sub_fallback)),
                                        make_font(13.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));
                                });

                                if discovered.is_empty() {
                                    sec.spawn((
                                        Node {
                                            width: percent(100.0),
                                            padding: UiRect::all(px(24.0)),
                                            border: UiRect::all(px(1.0)),
                                            border_radius: BorderRadius::all(px(6.0)),
                                            ..default()
                                        },
                                        BackgroundColor(palette::PANEL_INNER_BG),
                                        BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                                    ))
                                    .with_children(|empty| {
                                        empty.spawn((
                                            Text::new(empty_catalog_lbl.clone()),
                                            make_font(13.0),
                                            TextColor(palette::TEXT_MUTED),
                                        ));
                                    });
                                } else {
                                    for &(cat_key, cat_fallback) in journal_subcategories_for(tab) {
                                        let cat_voxels: Vec<Voxel> = discovered
                                            .iter()
                                            .copied()
                                            .filter(|&v| {
                                                voxel_journal_subcategory(tab, v) == Some(cat_key)
                                            })
                                            .collect();
                                        if cat_voxels.is_empty() {
                                            continue;
                                        }

                                        sec.spawn(Node {
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Column,
                                            row_gap: px(10.0),
                                            ..default()
                                        })
                                        .with_children(|group| {
                                            group.spawn((
                                                Text::new(t(cat_key, cat_fallback).to_uppercase()),
                                                make_font(12.0),
                                                TextColor(palette::TEXT_MUTED),
                                            ));

                                            group
                                                .spawn(Node {
                                                    display: Display::Flex,
                                                    flex_direction: FlexDirection::Row,
                                                    flex_wrap: FlexWrap::Wrap,
                                                    column_gap: px(12.0),
                                                    row_gap: px(12.0),
                                                    ..default()
                                                })
                                                .with_children(|grid| {
                                                    for voxel in cat_voxels {
                                                        let name = loc.as_ref().map_or_else(
                                                            || voxel.label().to_string(),
                                                            |l| l.t_block(voxel).to_string(),
                                                        );
                                                        let icon_img = block_icons
                                                            .as_ref()
                                                            .map(|b| b.get(voxel));

                                                        grid.spawn((
                                                            Node {
                                                                width: px(235.0),
                                                                padding: UiRect::axes(
                                                                    px(12.0),
                                                                    px(10.0),
                                                                ),
                                                                border: UiRect::all(px(1.5)),
                                                                border_radius: BorderRadius::all(
                                                                    px(6.0),
                                                                ),
                                                                display: Display::Flex,
                                                                flex_direction: FlexDirection::Row,
                                                                align_items: AlignItems::Center,
                                                                column_gap: px(12.0),
                                                                ..default()
                                                            },
                                                            BackgroundColor(
                                                                palette::PANEL_INNER_BG,
                                                            ),
                                                            BorderColor::all(
                                                                palette::PANEL_BORDER_SUBTLE,
                                                            ),
                                                        ))
                                                        .with_children(|card| {
                                                            if let Some(handle) = icon_img {
                                                                card.spawn((
                                                                    ImageNode {
                                                                        image: handle,
                                                                        image_mode:
                                                                            NodeImageMode::Stretch,
                                                                        ..default()
                                                                    },
                                                                    Node {
                                                                        width: px(36.0),
                                                                        height: px(36.0),
                                                                        flex_shrink: 0.0,
                                                                        ..default()
                                                                    },
                                                                ));
                                                            }
                                                            card.spawn((
                                                                Text::new(name),
                                                                make_font(13.0),
                                                                TextColor(palette::TEXT_PRIMARY),
                                                                text_shadow_default(),
                                                            ));
                                                        });
                                                    }
                                                });
                                        });
                                    }
                                }
                            });
                        }

                        // 4. Live Statistics (4 categorized groups)
                        main.spawn((
                            JournalSection(JournalTab::Statistics),
                            Node {
                                width: percent(100.0),
                                display: if *active_tab == JournalTab::Statistics {
                                    Display::Flex
                                } else {
                                    Display::None
                                },
                                flex_direction: FlexDirection::Column,
                                row_gap: px(20.0),
                                padding: UiRect::bottom(px(32.0)),
                                ..default()
                            },
                        ))
                        .with_children(|sec| {
                            sec.spawn((
                                Text::new(
                                    t("journal.tab.statistics", "Statistics").to_uppercase(),
                                ),
                                make_font(18.0),
                                TextColor(palette::TEXT_ACCENT),
                                text_shadow_default(),
                            ));

                            let stat_groups: [(&str, &str, Vec<(&str, String, String)>); 4] = [
                                (
                                    "journal.stats.cat.exploration",
                                    "Exploration & World",
                                    vec![
                                        (
                                            "time_played",
                                            t("journal.stats.time_played", "Time Played"),
                                            format_duration(progress.time_played_secs),
                                        ),
                                        (
                                            "days_survived",
                                            t("journal.stats.days_survived", "Days Survived"),
                                            format!("{day_num}"),
                                        ),
                                        (
                                            "biomes_discovered",
                                            t("journal.stats.biomes_discovered", "Biomes Discovered"),
                                            format!(
                                                "{} / {}",
                                                progress.discovered_biomes.len(),
                                                BiomeType::ALL.len()
                                            ),
                                        ),
                                        (
                                            "distance_traveled",
                                            t("journal.stats.distance_traveled", "Distance Walked"),
                                            format_distance(progress.distance_traveled),
                                        ),
                                        (
                                            "distance_swum",
                                            t("journal.stats.distance_swum", "Distance Swum"),
                                            format_distance(progress.distance_swum),
                                        ),
                                        (
                                            "jumps_performed",
                                            t("journal.stats.jumps_performed", "Jumps Performed"),
                                            format!("{}", progress.jumps_performed),
                                        ),
                                        (
                                            "highest_altitude",
                                            t("journal.stats.highest_altitude", "Highest Altitude"),
                                            progress
                                                .max_altitude
                                                .map_or("--".to_string(), |y| format!("{y} m")),
                                        ),
                                        (
                                            "deepest_depth",
                                            t("journal.stats.deepest_depth", "Deepest Depth"),
                                            progress
                                                .min_altitude
                                                .map_or("--".to_string(), |y| format!("{y} m")),
                                        ),
                                    ],
                                ),
                                (
                                    "journal.stats.cat.building",
                                    "Building & Gathering",
                                    vec![
                                        (
                                            "blocks_placed",
                                            t("journal.stats.blocks_placed", "Blocks Placed"),
                                            format!("{}", progress.blocks_placed),
                                        ),
                                        (
                                            "blocks_broken",
                                            t("journal.stats.blocks_broken", "Blocks Broken"),
                                            format!("{}", progress.blocks_broken),
                                        ),
                                        (
                                            "ores_mined",
                                            t("journal.stats.ores_mined", "Ores & Gems Mined"),
                                            format!("{}", progress.ores_mined),
                                        ),
                                        (
                                            "timber_chopped",
                                            t("journal.stats.timber_chopped", "Timber Chopped"),
                                            format!("{}", progress.timber_chopped),
                                        ),
                                        (
                                            "crops_harvested",
                                            t("journal.stats.crops_harvested", "Crops Harvested"),
                                            format!("{}", progress.crops_harvested),
                                        ),
                                        (
                                            "items_dropped",
                                            t("journal.stats.items_dropped", "Items Dropped"),
                                            format!("{}", progress.items_dropped),
                                        ),
                                    ],
                                ),
                                (
                                    "journal.stats.cat.combat",
                                    "Combat & Survival",
                                    vec![
                                        (
                                            "creatures_defeated",
                                            t("journal.stats.creatures_defeated", "Creatures Defeated"),
                                            "0".to_string(),
                                        ),
                                        (
                                            "creatures_tamed",
                                            t("journal.stats.creatures_tamed", "Creatures Tamed"),
                                            "0".to_string(),
                                        ),
                                        (
                                            "bosses_defeated",
                                            t("journal.stats.bosses_defeated", "Bosses Defeated"),
                                            "0".to_string(),
                                        ),
                                        (
                                            "deaths",
                                            t("journal.stats.deaths", "Deaths"),
                                            "0".to_string(),
                                        ),
                                        (
                                            "damage_dealt",
                                            t("journal.stats.damage_dealt", "Damage Dealt"),
                                            "0".to_string(),
                                        ),
                                        (
                                            "damage_taken",
                                            t("journal.stats.damage_taken", "Damage Taken"),
                                            "0".to_string(),
                                        ),
                                    ],
                                ),
                                (
                                    "journal.stats.cat.progression",
                                    "Progression & Crafting",
                                    vec![
                                        (
                                            "quests_completed",
                                            t("journal.stats.quests_completed", "Quests Completed"),
                                            "0".to_string(),
                                        ),
                                        (
                                            "items_crafted",
                                            t("journal.stats.items_crafted", "Items Crafted"),
                                            "0".to_string(),
                                        ),
                                        (
                                            "consumables_used",
                                            t("journal.stats.consumables_used", "Consumables Used"),
                                            "0".to_string(),
                                        ),
                                        (
                                            "gold_earned",
                                            t("journal.stats.gold_earned", "Gold Earned"),
                                            "0".to_string(),
                                        ),
                                        (
                                            "skill_points_spent",
                                            t(
                                                "journal.stats.skill_points_spent",
                                                "Skill Points Spent",
                                            ),
                                            "0".to_string(),
                                        ),
                                    ],
                                ),
                            ];

                            for (cat_key, cat_fallback, stats) in stat_groups {
                                sec.spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(10.0),
                                    ..default()
                                })
                                .with_children(|group| {
                                    group.spawn((
                                        Text::new(t(cat_key, cat_fallback).to_uppercase()),
                                        make_font(12.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));

                                    group
                                        .spawn(Node {
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Row,
                                            flex_wrap: FlexWrap::Wrap,
                                            column_gap: px(12.0),
                                            row_gap: px(12.0),
                                            ..default()
                                        })
                                        .with_children(|grid| {
                                            for (icon_id, label, val) in stats {
                                                let icon_handle = asset_server.load(format!(
                                                    "textures/interfaces/icons/stats/stat_{icon_id}.png"
                                                ));
                                                grid.spawn((
                                                    Node {
                                                        width: px(235.0),
                                                        padding: UiRect::axes(px(12.0), px(10.0)),
                                                        border: UiRect::all(px(1.5)),
                                                        border_radius: BorderRadius::all(px(6.0)),
                                                        display: Display::Flex,
                                                        flex_direction: FlexDirection::Row,
                                                        align_items: AlignItems::Center,
                                                        column_gap: px(12.0),
                                                        ..default()
                                                    },
                                                    BackgroundColor(palette::PANEL_INNER_BG),
                                                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                                                ))
                                                .with_children(|card| {
                                                    card.spawn((
                                                        ImageNode {
                                                            image: icon_handle,
                                                            image_mode: NodeImageMode::Stretch,
                                                            ..default()
                                                        },
                                                        Node {
                                                            width: px(34.0),
                                                            height: px(34.0),
                                                            flex_shrink: 0.0,
                                                            ..default()
                                                        },
                                                    ));

                                                    card.spawn(Node {
                                                        display: Display::Flex,
                                                        flex_direction: FlexDirection::Column,
                                                        row_gap: px(3.0),
                                                        ..default()
                                                    })
                                                    .with_children(|txt| {
                                                        txt.spawn((
                                                            Text::new(label),
                                                            make_font(11.0),
                                                            TextColor(palette::TEXT_MUTED),
                                                        ));
                                                        txt.spawn((
                                                            Text::new(val),
                                                            make_font(17.0),
                                                            TextColor(palette::TEXT_PRIMARY),
                                                            text_shadow_default(),
                                                        ));
                                                    });
                                                });
                                            }
                                        });
                                });
                            }
                        });
                    });

                // Right-Edge Scrollbar Track & Draggable Thumb
                right_wrap
                    .spawn((
                        JournalScrollbarTrack,
                        Hovered::default(),
                        Node {
                            width: px(16.0),
                            min_width: px(16.0),
                            flex_shrink: 0.0,
                            height: percent(100.0),
                            padding: UiRect::axes(px(4.0), px(12.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.55)),
                        Visibility::Hidden,
                    ))
                    .with_children(|track| {
                        track.spawn((
                            JournalScrollbarThumb,
                            Hovered::default(),
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(4.0),
                                right: px(4.0),
                                top: px(12.0),
                                height: px(60.0),
                                border_radius: BorderRadius::all(px(4.0)),
                                ..default()
                            },
                            BackgroundColor(palette::ACCENT_LINE),
                        ));
                    });
            });
        });
}

fn despawn_journal_menu(mut commands: Commands, query: Query<Entity, With<JournalMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_journal_nav_clicks(
    query: Query<(&Hovered, &JournalNavButton), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut active_tab: ResMut<JournalTab>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    for (hovered, nav_btn) in &query {
        if hovered.get() {
            *active_tab = nav_btn.0;
        }
    }
}

fn update_journal_tab_view(
    active_tab: Res<JournalTab>,
    mut nav_query: Query<(&JournalNavButton, &Hovered, &mut BackgroundColor, &Children)>,
    mut labels_query: Query<&mut TextColor, With<JournalNavLabel>>,
    mut accents_query: Query<&mut Visibility, With<JournalNavAccent>>,
    mut sections_query: Query<(&JournalSection, &mut Node)>,
) {
    for (nav_btn, hovered, mut bg, children) in &mut nav_query {
        let is_active = nav_btn.0 == *active_tab;
        let is_hovered = hovered.get();
        bg.0 = if is_active {
            palette::NAV_ITEM_ACTIVE_BG
        } else if is_hovered {
            palette::NAV_ITEM_HOVER_BG
        } else {
            Color::NONE
        };
        for child in children.iter() {
            if let Ok(mut color) = labels_query.get_mut(child) {
                color.0 = if is_active || is_hovered {
                    palette::TEXT_ACCENT
                } else {
                    palette::TEXT_PRIMARY
                };
            }
            if let Ok(mut vis) = accents_query.get_mut(child) {
                *vis = if is_active || is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }

    if !active_tab.is_changed() {
        return;
    }

    for (sec, mut node) in &mut sections_query {
        node.display = if sec.0 == *active_tab {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn handle_journal_scroll_and_scrollbar(
    active_tab: Res<JournalTab>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut drag: ResMut<JournalScrollDrag>,
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<JournalScrollArea>>,
    mut track_q: Query<
        (&Hovered, &ComputedNode, &UiGlobalTransform, &mut Visibility),
        (With<JournalScrollbarTrack>, Without<JournalScrollbarThumb>),
    >,
    mut thumb_q: Query<
        (&Hovered, &mut Node, &mut BackgroundColor),
        (With<JournalScrollbarThumb>, Without<JournalScrollbarTrack>),
    >,
) {
    let Ok((mut scroll_pos, scroll_node)) = scroll_q.single_mut() else {
        return;
    };

    if active_tab.is_changed() {
        scroll_pos.y = 0.0;
        drag.dragging = false;
    }

    let inv_scale = scroll_node.inverse_scale_factor();
    let viewport_h = scroll_node.size().y * inv_scale;
    let content_h = scroll_node.content_size().y * inv_scale;
    let max_scroll = (content_h - viewport_h).max(0.0);

    if max_scroll > 1.0 && mouse_scroll.delta.y.abs() > 0.001 {
        let step = match mouse_scroll.unit {
            bevy::input::mouse::MouseScrollUnit::Line => 48.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => 1.0,
        };
        scroll_pos.y = (scroll_pos.y - mouse_scroll.delta.y * step).clamp(0.0, max_scroll);
    } else if max_scroll <= 1.0 {
        scroll_pos.y = 0.0;
    }

    if let Ok((track_hovered, track_node, track_tf, mut track_vis)) = track_q.single_mut() {
        if max_scroll <= 1.0 {
            *track_vis = Visibility::Hidden;
            drag.dragging = false;
            return;
        }
        *track_vis = Visibility::Visible;

        let track_inv = track_node.inverse_scale_factor();
        let pad_v = 12.0;
        let track_h = (track_node.size().y * track_inv - pad_v * 2.0).max(20.0);
        let thumb_h = ((viewport_h / content_h.max(1.0)) * track_h).clamp(32.0, track_h);
        let usable_h = (track_h - thumb_h).max(1.0);

        let thumb_hovered = thumb_q.single().is_ok_and(|(h, _, _)| h.get());

        if mouse.just_pressed(MouseButton::Left) && (track_hovered.get() || thumb_hovered) {
            drag.dragging = true;
        }
        if !mouse.pressed(MouseButton::Left) {
            drag.dragging = false;
        }

        if drag.dragging
            && let Ok(win) = windows.single()
            && let Some(cursor_pos) = win.cursor_position()
        {
            let track_center_y = track_tf.translation.y * track_inv;
            let track_top = track_center_y - (track_node.size().y * track_inv) * 0.5 + pad_v;
            let ratio = ((cursor_pos.y - track_top - thumb_h * 0.5) / usable_h).clamp(0.0, 1.0);
            scroll_pos.y = ratio * max_scroll;
        }

        scroll_pos.y = scroll_pos.y.clamp(0.0, max_scroll);
        let ratio = (scroll_pos.y / max_scroll).clamp(0.0, 1.0);
        let thumb_top = pad_v + ratio * usable_h;

        if let Ok((hovered, mut thumb_node, mut thumb_bg)) = thumb_q.single_mut() {
            thumb_node.height = px(thumb_h);
            thumb_node.top = px(thumb_top);
            thumb_bg.0 = if drag.dragging || hovered.get() {
                palette::TEXT_ACCENT
            } else {
                Color::srgba(0.55, 0.60, 0.70, 0.70)
            };
        }
    }
}

fn update_journal_almanac(
    env_state: Option<Res<EnvironmentState>>,
    weather: Option<Res<WeatherManager>>,
    loc: Option<Res<Localization>>,
    asset_server: Res<AssetServer>,
    mut season_q: Query<
        &mut Text,
        (
            With<JournalSeasonText>,
            Without<JournalDayText>,
            Without<JournalTimeText>,
            Without<JournalWeatherText>,
        ),
    >,
    mut day_q: Query<
        &mut Text,
        (
            With<JournalDayText>,
            Without<JournalSeasonText>,
            Without<JournalTimeText>,
            Without<JournalWeatherText>,
        ),
    >,
    mut time_q: Query<
        &mut Text,
        (
            With<JournalTimeText>,
            Without<JournalSeasonText>,
            Without<JournalDayText>,
            Without<JournalWeatherText>,
        ),
    >,
    mut weather_q: Query<
        &mut Text,
        (
            With<JournalWeatherText>,
            Without<JournalSeasonText>,
            Without<JournalDayText>,
            Without<JournalTimeText>,
        ),
    >,
    mut time_icon_q: Query<&mut ImageNode, (With<JournalTimeIcon>, Without<JournalWeatherIcon>)>,
    mut weather_icon_q: Query<&mut ImageNode, (With<JournalWeatherIcon>, Without<JournalTimeIcon>)>,
) {
    if let Some(ref env) = env_state {
        let day_num = env.day_count;
        let season_name = loc.as_ref().map_or("SPRING".to_string(), |l| {
            l.t(season_loc_key(day_num)).to_uppercase()
        });
        let day_lbl = loc.as_ref().map_or("Day", |l| l.t("journal.day"));
        let total_mins = (env.time_of_day * 24.0 * 60.0) as u32 % (24 * 60);
        let hour = total_mins / 60;
        let time_str = format!("{:02}:{:02}", hour, total_mins % 60);
        let day_str = format!("{day_lbl} {day_num}");

        for mut t in &mut season_q {
            if t.0 != season_name {
                t.0 = season_name.clone();
            }
        }
        for mut t in &mut day_q {
            if t.0 != day_str {
                t.0 = day_str.clone();
            }
        }
        for mut t in &mut time_q {
            if t.0 != time_str {
                t.0 = time_str.clone();
            }
        }
        let icon_handle = asset_server.load(time_icon_for(hour));
        for mut img in &mut time_icon_q {
            if img.image != icon_handle {
                img.image = icon_handle.clone();
            }
        }
    }

    if let Some(ref w) = weather {
        let weather_debug = format!("{:?}", w.current);
        let weather_str = loc.as_ref().map_or_else(
            || weather_debug.clone(),
            |l| l.t(weather_loc_key(&weather_debug)).to_string(),
        );
        for mut t in &mut weather_q {
            if t.0 != weather_str {
                t.0 = weather_str.clone();
            }
        }
        let icon_handle = asset_server.load(weather_icon_for(&weather_debug));
        for mut img in &mut weather_icon_q {
            if img.image != icon_handle {
                img.image = icon_handle.clone();
            }
        }
    }
}
