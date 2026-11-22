use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::Button,
};

use crate::{
    core::{AppFont, text_shadow_default},
    environment::{EnvironmentState, weather::WeatherManager},
};

use super::{GameSettings, MenuState, menu_bar::MENU_BAR_HEIGHT, theme::palette};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JournalTab {
    #[default]
    Bestiary,
    Compendium,
    Statistics,
}

impl JournalTab {
    const ALL: [Self; 3] = [Self::Bestiary, Self::Compendium, Self::Statistics];

    fn label(self) -> &'static str {
        match self {
            Self::Bestiary => "Bestiary",
            Self::Compendium => "Compendium",
            Self::Statistics => "Statistics",
        }
    }
}

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

pub struct JournalMenuPlugin;

impl Plugin for JournalMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JournalTab>()
            .add_systems(OnEnter(MenuState::Journal), spawn_journal_menu)
            .add_systems(OnExit(MenuState::Journal), despawn_journal_menu)
            .add_systems(
                Update,
                (
                    handle_journal_nav_clicks,
                    update_journal_tab_view,
                    update_journal_almanac,
                )
                    .run_if(in_state(MenuState::Journal)),
            );
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

fn spawn_journal_menu(
    mut commands: Commands,
    active_tab: Res<JournalTab>,
    env_state: Option<Res<EnvironmentState>>,
    weather: Option<Res<WeatherManager>>,
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

    let dimming_alpha = game_settings.map_or(0.65, |s| s.background_dimming.alpha());

    let day_num = env_state.as_ref().map_or(1, |e| e.day_count);
    let season_name = match (day_num.saturating_sub(1) / 30) % 4 {
        0 => "Spring",
        1 => "Summer",
        2 => "Autumn",
        _ => "Winter",
    };
    let time_of_day = env_state.as_ref().map_or(0.5, |e| e.time_of_day);
    let total_mins = (time_of_day * 24.0 * 60.0) as u32 % (24 * 60);
    let hour = total_mins / 60;
    let time_str = format!("{:02}:{:02}", hour, total_mins % 60);
    let time_icon_path = time_icon_for(hour);
    let weather_str = weather
        .as_ref()
        .map_or("Clear".to_string(), |w| format!("{:?}", w.current));
    let weather_icon_path = weather_icon_for(&weather_str);
    let time_icon = asset_server.load(time_icon_path);
    let weather_icon = asset_server.load(weather_icon_path);

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
            // Left Column: Compact World Almanac Header + Journal Sections (standardized 310px)
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
                    row_gap: px(4.0),
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL1_BG),
                BorderColor::all(palette::DRAWER_DIVIDER),
            ))
            .with_children(|nav| {
                // Compact World Almanac Card at top of sidebar with real icons
                nav.spawn((
                    Node {
                        margin: UiRect::new(px(18.0), px(18.0), px(0.0), px(14.0)),
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
                    // Row 1: Season + Day
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
                                Text::new(season_name.to_uppercase()),
                                make_font(13.0),
                                TextColor(palette::TEXT_ACCENT),
                                text_shadow_default(),
                            ));
                            row.spawn((
                                JournalDayText,
                                Text::new(format!("Day {day_num}")),
                                make_font(13.0),
                                TextColor(palette::TEXT_MUTED),
                            ));
                        });

                    // Row 2: Time icon + clock  |  Weather icon + weather
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
                            .with_children(|t| {
                                t.spawn((
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
                                t.spawn((
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
                            .with_children(|w| {
                                w.spawn((
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
                                w.spawn((
                                    JournalWeatherText,
                                    Text::new(weather_str),
                                    make_font(13.0),
                                    TextColor(palette::TEXT_PRIMARY),
                                ));
                            });
                        });
                });

                for tab in JournalTab::ALL {
                    let is_active = tab == *active_tab;
                    nav.spawn((
                        Button,
                        Hovered::default(),
                        JournalNavButton(tab),
                        Node {
                            width: percent(100.0),
                            height: px(48.0),
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
                                top: px(8.0),
                                bottom: px(8.0),
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
                            Text::new(tab.label()),
                            make_font(16.0),
                            TextColor(if is_active {
                                palette::TEXT_ACCENT
                            } else {
                                palette::TEXT_PRIMARY
                            }),
                            text_shadow_default(),
                        ));

                        btn.spawn((
                            Text::new(">"),
                            make_font(15.0),
                            TextColor(palette::TEXT_MUTED),
                        ));
                    });
                }
            });

            // Right Main Panel: Selected Journal Section
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    height: percent(100.0),
                    padding: UiRect::all(px(32.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL2_BG),
            ))
            .with_children(|main| {
                // 1. Bestiary & Mobs
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
                    sec.spawn((
                        Text::new("BESTIARY"),
                        make_font(18.0),
                        TextColor(palette::TEXT_ACCENT),
                        text_shadow_default(),
                    ));

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
                                    Text::new(format!("Entry #{idx:02}")),
                                    make_font(11.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                                card.spawn((
                                    Text::new("???"),
                                    make_font(18.0),
                                    TextColor(Color::srgba(0.62, 0.66, 0.74, 0.55)),
                                ));
                                card.spawn((
                                    Text::new("Undiscovered"),
                                    make_font(11.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                            });
                        }
                    });
                });

                // 4. Compendium
                main.spawn((
                    JournalSection(JournalTab::Compendium),
                    Node {
                        width: percent(100.0),
                        display: if *active_tab == JournalTab::Compendium {
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
                    sec.spawn((
                        Text::new("COMPENDIUM"),
                        make_font(18.0),
                        TextColor(palette::TEXT_ACCENT),
                        text_shadow_default(),
                    ));

                    let categories = [
                        ("Blocks & Minerals", "Geological strata, ores & construction blocks"),
                        ("Flora & Agriculture", "Wild plants, trees, crops & culinary harvests"),
                        ("Biomes & Regions", "44 distinct surface & subterranean climates"),
                        ("Items & Equipment", "Tools, consumables & forged gear"),
                    ];
                    sec.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(16.0),
                        row_gap: px(16.0),
                        ..default()
                    })
                    .with_children(|grid| {
                        for (title, desc) in categories {
                            grid.spawn((
                                Node {
                                    width: px(280.0),
                                    padding: UiRect::all(px(16.0)),
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::all(px(6.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(6.0),
                                    ..default()
                                },
                                BackgroundColor(palette::PANEL_INNER_BG),
                                BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                            ))
                            .with_children(|card| {
                                card.spawn((
                                    Text::new(title),
                                    make_font(15.0),
                                    TextColor(palette::TEXT_PRIMARY),
                                    text_shadow_default(),
                                ));
                                card.spawn((
                                    Text::new(desc),
                                    make_font(12.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                            });
                        }
                    });
                });

                // 3. Statistics & Feats
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
                        row_gap: px(16.0),
                        ..default()
                    },
                ))
                .with_children(|sec| {
                    sec.spawn((
                        Text::new("STATISTICS"),
                        make_font(18.0),
                        TextColor(palette::TEXT_ACCENT),
                        text_shadow_default(),
                    ));

                    let stats = [
                        ("Days Survived", format!("{day_num}")),
                        ("Blocks Placed", "0".to_string()),
                        ("Blocks Broken", "0".to_string()),
                        ("Biomes Discovered", "1 / 44".to_string()),
                        ("Distance Traveled", "0 m".to_string()),
                        ("Crops Harvested", "0".to_string()),
                    ];
                    sec.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(14.0),
                        row_gap: px(14.0),
                        ..default()
                    })
                    .with_children(|grid| {
                        for (label, val) in stats {
                            grid.spawn((
                                Node {
                                    width: px(220.0),
                                    padding: UiRect::all(px(16.0)),
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::all(px(6.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(6.0),
                                    ..default()
                                },
                                BackgroundColor(palette::PANEL_INNER_BG),
                                BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                            ))
                            .with_children(|card| {
                                card.spawn((
                                    Text::new(label),
                                    make_font(12.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                                card.spawn((
                                    Text::new(val),
                                    make_font(18.0),
                                    TextColor(palette::TEXT_PRIMARY),
                                    text_shadow_default(),
                                ));
                            });
                        }
                    });
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

fn update_journal_almanac(
    env_state: Option<Res<EnvironmentState>>,
    weather: Option<Res<WeatherManager>>,
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
        let season_name = match (day_num.saturating_sub(1) / 30) % 4 {
            0 => "SPRING",
            1 => "SUMMER",
            2 => "AUTUMN",
            _ => "WINTER",
        };
        let total_mins = (env.time_of_day * 24.0 * 60.0) as u32 % (24 * 60);
        let hour = total_mins / 60;
        let time_str = format!("{:02}:{:02}", hour, total_mins % 60);
        let day_str = format!("Day {day_num}");

        for mut t in &mut season_q {
            if t.0 != season_name {
                t.0 = season_name.to_string();
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
        let weather_str = format!("{:?}", w.current);
        for mut t in &mut weather_q {
            if t.0 != weather_str {
                t.0 = weather_str.clone();
            }
        }
        let icon_handle = asset_server.load(weather_icon_for(&weather_str));
        for mut img in &mut weather_icon_q {
            if img.image != icon_handle {
                img.image = icon_handle.clone();
            }
        }
    }
}
