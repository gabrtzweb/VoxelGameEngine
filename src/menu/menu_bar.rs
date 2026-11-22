use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::Button,
};

use crate::{
    core::{AppFont, text_shadow_default},
    menu::creative_inventory::CreativeSearchQuery,
};

use super::{MenuReturnTarget, MenuState, theme::palette};

pub const MENU_BAR_HEIGHT: f32 = 80.0; // 20 px @ 4x
pub const MENU_SCALE: f32 = 4.0;

#[derive(Component)]
pub struct MenuBarRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarAction {
    PrevTab,      // [Q]
    TabInventory, // 01
    TabSkills,    // 02
    TabJournal,   // 03
    TabQuests,    // 04
    TabWorldMap,  // 05
    TabSettings,  // 06
    NextTab,      // [E]
    CloseMenu,    // [ESC]
}

#[derive(Component)]
pub struct MenuBarClockText;

#[derive(Component)]
pub struct MenuBarClockIcon;

#[derive(Component)]
pub struct MenuBarTabLabel {
    pub action: MenuBarAction,
}

#[derive(Component)]
pub struct MenuBarTabUnderline {
    pub action: MenuBarAction,
}

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_menu_bar_visibility,
                handle_menu_bar_buttons,
                handle_menu_bar_shortcuts,
                update_menu_bar_tab_highlights,
                update_menu_bar_clock,
            ),
        );
    }
}

fn sync_menu_bar_visibility(
    mut commands: Commands,
    menu_state: Res<State<MenuState>>,
    menu_return_target: Res<MenuReturnTarget>,
    world_session: Option<Res<crate::world::WorldSession>>,
    menu_bar_query: Query<Entity, With<MenuBarRoot>>,
    asset_server: Res<AssetServer>,
    app_font: Option<Res<AppFont>>,
    loc: Option<Res<crate::core::Localization>>,
) {
    let in_active_session = world_session.as_ref().map_or_else(
        || menu_return_target.0 != MenuState::MainMenu,
        |s| s.is_active,
    );
    let is_in_menu = in_active_session
        && *menu_state.get() != MenuState::None
        && *menu_state.get() != MenuState::MainMenu
        && *menu_state.get() != MenuState::RaceSelection
        && *menu_state.get() != MenuState::Loading;
    let existing = menu_bar_query.iter().next();

    if is_in_menu && existing.is_none() {
        spawn_menu_bar(&mut commands, &asset_server, app_font.as_ref(), loc.as_deref());
    } else if !is_in_menu && let Some(entity) = existing {
        commands.entity(entity).despawn();
    }
}

fn spawn_menu_bar(
    commands: &mut Commands,
    asset_server: &AssetServer,
    app_font: Option<&Res<AppFont>>,
    loc: Option<&crate::core::Localization>,
) {
    let font_handle = app_font.map(|f| f.source());

    let make_tab_font = |size: f32| {
        let mut tf = TextFont {
            font_size: FontSize::Px(size),
            ..default()
        };
        if let Some(ref font) = font_handle {
            tf.font = font.clone();
        }
        tf
    };

    commands
        .spawn((
            MenuBarRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                right: px(0.0),
                top: px(0.0),
                height: px(MENU_BAR_HEIGHT),
                border: UiRect::bottom(px(1.5)),
                padding: UiRect::horizontal(px(28.0)),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(palette::BAR_BG),
            BorderColor::all(palette::BAR_BORDER),
            ZIndex(400), // Render above containers (300), below cursor (1000)
        ))
        .with_children(|bar| {
            // Left group: [Q]  (INVENTORY   SKILLS   JOURNAL & QUESTS   WORLD MAP   SYSTEM)  [E]
            bar.spawn(Node {
                height: percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(24.0),
                padding: UiRect::left(px(28.0)),
                ..default()
            })
            .with_children(|nav| {
                // [Q] Keycap Navigation Button
                nav.spawn((
                    Button,
                    Hovered::default(),
                    MenuBarAction::PrevTab,
                    Node {
                        width: px(30.0),
                        height: px(30.0),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(4.0)),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(palette::SLOT_EMPTY_BG),
                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                ))
                .with_children(|q_btn| {
                    q_btn.spawn((
                        Text::new("Q"),
                        make_tab_font(12.0),
                        TextColor(palette::TEXT_MUTED),
                        text_shadow_default(),
                    ));
                });

                let tab_inv = loc.map_or("INVENTORY", |l| l.t("menu.bar.inventory"));
                let tab_skills = loc.map_or("SKILLS", |l| l.t("menu.bar.skills"));
                let tab_quests = loc.map_or("QUESTS", |l| l.t("menu.bar.quests"));
                let tab_journal = loc.map_or("JOURNAL", |l| l.t("menu.bar.journal"));
                let tab_map = loc.map_or("WORLD MAP", |l| l.t("menu.bar.world_map"));
                let tab_sys = loc.map_or("SYSTEM", |l| l.t("menu.bar.system"));

                let tabs = [
                    (
                        MenuBarAction::TabInventory,
                        tab_inv,
                        "textures/interfaces/icons/menu_inventory.png",
                    ),
                    (
                        MenuBarAction::TabSkills,
                        tab_skills,
                        "textures/interfaces/icons/menu_skills.png",
                    ),
                    (
                        MenuBarAction::TabJournal,
                        tab_journal,
                        "textures/interfaces/icons/menu_journal.png",
                    ),
                    (
                        MenuBarAction::TabQuests,
                        tab_quests,
                        "textures/interfaces/icons/menu_quests.png",
                    ),
                    (
                        MenuBarAction::TabWorldMap,
                        tab_map,
                        "textures/interfaces/icons/menu_map.png",
                    ),
                    (
                        MenuBarAction::TabSettings,
                        tab_sys,
                        "textures/interfaces/icons/menu_system.png",
                    ),
                ];

                // Inner row holding the 6 tabs with icon + label
                nav.spawn(Node {
                    height: percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(48.0),
                    ..default()
                })
                .with_children(|tabs_row| {
                    for (action, label, icon_path) in tabs {
                        let icon_handle = asset_server.load(icon_path);
                        tabs_row
                            .spawn((
                                Button,
                                Hovered::default(),
                                action,
                                Node {
                                    height: percent(100.0),
                                    padding: UiRect::horizontal(px(8.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    column_gap: px(8.0),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|tab_btn| {
                                tab_btn.spawn((
                                    ImageNode {
                                        image: icon_handle,
                                        ..default()
                                    },
                                    Node {
                                        width: px(18.0),
                                        height: px(18.0),
                                        ..default()
                                    },
                                ));

                                tab_btn.spawn((
                                    MenuBarTabLabel { action },
                                    Text::new(label),
                                    make_tab_font(15.0),
                                    TextColor(palette::TEXT_MUTED),
                                    text_shadow_default(),
                                ));

                                tab_btn.spawn((
                                    MenuBarTabUnderline { action },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(6.0),
                                        right: px(6.0),
                                        top: px(53.0),
                                        height: px(2.5),
                                        border_radius: BorderRadius::all(px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(palette::ACCENT_LINE),
                                    Visibility::Hidden,
                                ));
                            });
                    }
                });

                // [E] Keycap Navigation Button
                nav.spawn((
                    Button,
                    Hovered::default(),
                    MenuBarAction::NextTab,
                    Node {
                        width: px(30.0),
                        height: px(30.0),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(4.0)),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(palette::SLOT_EMPTY_BG),
                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                ))
                .with_children(|e_btn| {
                    e_btn.spawn((
                        Text::new("E"),
                        make_tab_font(12.0),
                        TextColor(palette::TEXT_MUTED),
                        text_shadow_default(),
                    ));
                });
            });

            // Right group: [Level]   [Live Clock]   [Coins]   [ESC]
            bar.spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(22.0),
                padding: UiRect::right(px(8.0)),
                ..default()
            })
            .with_children(|status| {
                // Level badge
                status
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|lvl| {
                        lvl.spawn((
                            ImageNode {
                                image: asset_server.load("textures/interfaces/icons/ui_level.png"),
                                ..default()
                            },
                            Node {
                                width: px(18.0),
                                height: px(18.0),
                                ..default()
                            },
                        ));
                        lvl.spawn((
                            Text::new("Lv. 1"),
                            make_tab_font(14.0),
                            TextColor(palette::TEXT_PRIMARY),
                            text_shadow_default(),
                        ));
                    });

                // Live Clock + Time-of-Day Icon
                status
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|clk| {
                        clk.spawn((
                            MenuBarClockIcon,
                            ImageNode {
                                image: asset_server
                                    .load("textures/interfaces/icons/time_noon.png"),
                                ..default()
                            },
                            Node {
                                width: px(18.0),
                                height: px(18.0),
                                ..default()
                            },
                        ));
                        clk.spawn((
                            MenuBarClockText,
                            Text::new("12:00"),
                            make_tab_font(14.0),
                            TextColor(palette::TEXT_PRIMARY),
                            text_shadow_default(),
                        ));
                    });

                // Gold Coins badge
                status
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|coins| {
                        coins.spawn((
                            ImageNode {
                                image: asset_server.load("textures/interfaces/icons/ui_coin.png"),
                                ..default()
                            },
                            Node {
                                width: px(18.0),
                                height: px(18.0),
                                ..default()
                            },
                        ));
                        coins.spawn((
                            Text::new("0"),
                            make_tab_font(14.0),
                            TextColor(palette::TEXT_ACCENT),
                            text_shadow_default(),
                        ));
                    });

                // [ESC] Close Menu Keycap Button (matches [Q] / [E] style)
                status
                    .spawn((
                        Button,
                        Hovered::default(),
                        MenuBarAction::CloseMenu,
                        Node {
                            height: px(30.0),
                            padding: UiRect::horizontal(px(10.0)),
                            border: UiRect::all(px(1.0)),
                            border_radius: BorderRadius::all(px(4.0)),
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(palette::SLOT_EMPTY_BG),
                        BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                    ))
                    .with_children(|esc_btn| {
                        esc_btn.spawn((
                            Text::new("ESC"),
                            make_tab_font(11.0),
                            TextColor(palette::TEXT_MUTED),
                            text_shadow_default(),
                        ));
                    });
            });
        });
}

fn handle_menu_bar_buttons(
    buttons_query: Query<(&Hovered, Has<Pressed>, &MenuBarAction), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    menu_state: Res<State<MenuState>>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    let left_clicked = mouse.just_pressed(MouseButton::Left);

    for (hovered, is_pressed, action) in &buttons_query {
        if hovered.get() && (left_clicked || is_pressed) {
            match action {
                MenuBarAction::PrevTab => {
                    if left_clicked {
                        cycle_tab(&menu_state, &mut next_state, false);
                    }
                }
                MenuBarAction::NextTab => {
                    if left_clicked {
                        cycle_tab(&menu_state, &mut next_state, true);
                    }
                }
                MenuBarAction::TabInventory => {
                    if left_clicked {
                        next_state.set(MenuState::Inventory);
                    }
                }
                MenuBarAction::TabSkills => {
                    if left_clicked {
                        next_state.set(MenuState::Skills);
                    }
                }
                MenuBarAction::TabQuests => {
                    if left_clicked {
                        next_state.set(MenuState::Quests);
                    }
                }
                MenuBarAction::TabJournal => {
                    if left_clicked {
                        next_state.set(MenuState::Journal);
                    }
                }
                MenuBarAction::TabWorldMap => {
                    if left_clicked {
                        next_state.set(MenuState::WorldMap);
                    }
                }
                MenuBarAction::TabSettings => {
                    if left_clicked {
                        next_state.set(MenuState::Pause);
                    }
                }
                MenuBarAction::CloseMenu => {
                    if left_clicked {
                        next_state.set(MenuState::None);
                    }
                }
            }
        }
    }
}

fn handle_menu_bar_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    control_settings: Option<Res<crate::core::ControlSettings>>,
    menu_state: Res<State<MenuState>>,
    mut next_state: ResMut<NextState<MenuState>>,
    search_query: Option<Res<CreativeSearchQuery>>,
) {
    if *menu_state.get() == MenuState::None || *menu_state.get() == MenuState::MainMenu {
        return;
    }

    // Do not trigger tab cycling while typing in the creative search box!
    if search_query.as_ref().is_some_and(|s| s.is_focused) {
        return;
    }

    let prev_pressed = control_settings
        .as_ref()
        .map_or(keyboard.just_pressed(KeyCode::KeyQ), |c| {
            c.is_action_just_pressed(crate::core::InputAction::MenuTabPrev, &keyboard, &mouse)
        });

    let next_pressed = control_settings
        .as_ref()
        .map_or(keyboard.just_pressed(KeyCode::KeyE), |c| {
            c.is_action_just_pressed(crate::core::InputAction::MenuTabNext, &keyboard, &mouse)
        });

    if prev_pressed {
        cycle_tab(&menu_state, &mut next_state, false);
    } else if next_pressed {
        cycle_tab(&menu_state, &mut next_state, true);
    }
}

fn cycle_tab(
    menu_state: &Res<State<MenuState>>,
    next_state: &mut ResMut<NextState<MenuState>>,
    forward: bool,
) {
    let tabs = [
        MenuState::Inventory,
        MenuState::Skills,
        MenuState::Journal,
        MenuState::Quests,
        MenuState::WorldMap,
        MenuState::Pause,
    ];

    let current = *menu_state.get();
    let cur_idx = match current {
        MenuState::Inventory => 0,
        MenuState::Skills => 1,
        MenuState::Journal => 2,
        MenuState::Quests => 3,
        MenuState::WorldMap => 4,
        MenuState::Pause | MenuState::Settings | MenuState::Controls => 5,
        _ => 0,
    };

    let next_idx = if forward {
        (cur_idx + 1) % tabs.len()
    } else {
        (cur_idx + tabs.len() - 1) % tabs.len()
    };

    next_state.set(tabs[next_idx]);
}

fn update_menu_bar_tab_highlights(
    menu_state: Res<State<MenuState>>,
    loc: Option<Res<crate::core::Localization>>,
    tab_buttons_query: Query<(&MenuBarAction, &Hovered, &Children), With<Button>>,
    mut labels_query: Query<(&MenuBarTabLabel, &mut TextColor, &mut Text)>,
    mut underlines_query: Query<(&MenuBarTabUnderline, &mut Visibility)>,
) {
    let current = *menu_state.get();

    for (action, hovered, children) in &tab_buttons_query {
        let is_active = match action {
            MenuBarAction::TabInventory => current == MenuState::Inventory,
            MenuBarAction::TabSkills => current == MenuState::Skills,
            MenuBarAction::TabQuests => current == MenuState::Quests,
            MenuBarAction::TabJournal => current == MenuState::Journal,
            MenuBarAction::TabWorldMap => current == MenuState::WorldMap,
            MenuBarAction::TabSettings => {
                current == MenuState::Pause
                    || current == MenuState::Settings
                    || current == MenuState::Controls
            }
            _ => continue,
        };

        for child in children.iter() {
            if let Ok((label, mut color, mut text)) = labels_query.get_mut(child) {
                if let Some(ref l) = loc {
                    let localized_str = match label.action {
                        MenuBarAction::TabInventory => l.t("menu.bar.inventory"),
                        MenuBarAction::TabSkills => l.t("menu.bar.skills"),
                        MenuBarAction::TabQuests => l.t("menu.bar.quests"),
                        MenuBarAction::TabJournal => l.t("menu.bar.journal"),
                        MenuBarAction::TabWorldMap => l.t("menu.bar.world_map"),
                        MenuBarAction::TabSettings => l.t("menu.bar.system"),
                        _ => "",
                    };
                    if !localized_str.is_empty() && **text != localized_str {
                        **text = localized_str.to_string();
                    }
                }

                color.0 = if is_active {
                    palette::TEXT_ACCENT
                } else if hovered.get() {
                    palette::TEXT_PRIMARY
                } else {
                    palette::TEXT_MUTED
                };
            }

            if let Ok((_, mut vis)) = underlines_query.get_mut(child) {
                *vis = if is_active {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

fn update_menu_bar_clock(
    env_state: Option<Res<crate::environment::EnvironmentState>>,
    asset_server: Res<AssetServer>,
    mut clock_text_q: Query<&mut Text, With<MenuBarClockText>>,
    mut clock_icon_q: Query<&mut ImageNode, With<MenuBarClockIcon>>,
) {
    let Some(env) = env_state else {
        return;
    };
    let total_mins = (env.time_of_day * 24.0 * 60.0) as u32 % (24 * 60);
    let hour = total_mins / 60;
    let time_str = format!("{:02}:{:02}", hour, total_mins % 60);

    for mut t in &mut clock_text_q {
        if t.0 != time_str {
            t.0 = time_str.clone();
        }
    }

    let icon_path = match hour {
        5..=10 => "textures/interfaces/icons/time_morning.png",
        11..=16 => "textures/interfaces/icons/time_noon.png",
        17..=20 => "textures/interfaces/icons/time_evening.png",
        _ => "textures/interfaces/icons/time_night.png",
    };
    let handle = asset_server.load(icon_path);
    for mut img in &mut clock_icon_q {
        if img.image != handle {
            img.image = handle.clone();
        }
    }
}

