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

use super::{MenuReturnTarget, MenuState};

pub const MENU_BAR_HEIGHT: f32 = 80.0; // 20 px @ 4x
pub const MENU_SCALE: f32 = 4.0;

#[derive(Component)]
pub struct MenuBarRoot;

#[derive(Component)]
pub struct MenuBarJournalOverlay;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarAction {
    PrevTab,         // [Q] (01)
    TabInventory,    // (02)
    TabJournal,      // (03)
    TabWorldMap,     // (04)
    TabSettings,     // (05)
    NextTab,         // [E] (06)
}

#[derive(Component)]
pub struct MenuBarTabLabel {
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
                handle_journal_overlay_visibility,
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
    let bar_texture = asset_server.load("textures/interfaces/containers/menu_bar.png");

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
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.09, 0.10)),
            ZIndex(400), // Render above containers (300), below cursor (1000)
        ))
        .with_children(|root| {
            root.spawn((
                ImageNode {
                    image: bar_texture,
                    rect: Some(Rect::new(0.0, 0.0, 480.0, 20.0)),
                    ..default()
                },
                Node {
                    width: px(1920.0),
                    height: px(MENU_BAR_HEIGHT),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
            ))
            .with_children(|bar| {
            // 01: [Q] Navigation Button (11, 6, 8, 8) -> 4x: (44, 24, 32, 32)
            bar.spawn((
                Button,
                Hovered::default(),
                MenuBarAction::PrevTab,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(11.0 * MENU_SCALE),
                    top: px(6.0 * MENU_SCALE),
                    width: px(8.0 * MENU_SCALE),
                    height: px(8.0 * MENU_SCALE),
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));

            // Tabs Configuration: (action, label, x, y, w, h)
            let tab_inv = loc.map_or("INVENTORY", |l| l.t("menu.bar.inventory"));
            let tab_journal = loc.map_or("JOURNAL", |l| l.t("menu.bar.journal"));
            let tab_map = loc.map_or("WORLD MAP", |l| l.t("menu.bar.world_map"));
            let tab_sys = loc.map_or("SYSTEM", |l| l.t("menu.bar.system"));

            let tabs = [
                (MenuBarAction::TabInventory, tab_inv, 27.0, 5.0, 48.0, 10.0),
                (MenuBarAction::TabJournal, tab_journal, 87.0, 5.0, 48.0, 10.0),
                (MenuBarAction::TabWorldMap, tab_map, 147.0, 5.0, 48.0, 10.0),
                (MenuBarAction::TabSettings, tab_sys, 207.0, 5.0, 48.0, 10.0),
            ];

            for (action, label, x, y, w, h) in tabs {
                bar.spawn((
                    Button,
                    Hovered::default(),
                    action,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(x * MENU_SCALE),
                        top: px(y * MENU_SCALE),
                        width: px(w * MENU_SCALE),
                        height: px(h * MENU_SCALE),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|tab_btn| {
                    tab_btn.spawn((
                        MenuBarTabLabel { action },
                        Text::new(label),
                        make_tab_font(13.0),
                        TextColor(Color::srgb(0.85, 0.86, 0.90)),
                        text_shadow_default(),
                    ));
                });
            }

            // 06: [E] Navigation Button (263, 6, 8, 8) -> 4x: (1052, 24, 32, 32)
            bar.spawn((
                Button,
                Hovered::default(),
                MenuBarAction::NextTab,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(263.0 * MENU_SCALE),
                    top: px(6.0 * MENU_SCALE),
                    width: px(8.0 * MENU_SCALE),
                    height: px(8.0 * MENU_SCALE),
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));

            // Currency 1 Text Readout (411, 5, 20, 10) -> 4x: (1644, 20, 80, 40)
            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(411.0 * MENU_SCALE),
                    top: px(5.0 * MENU_SCALE),
                    width: px(20.0 * MENU_SCALE),
                    height: px(10.0 * MENU_SCALE),
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|well| {
                well.spawn((
                    Text::new("0"),
                    make_tab_font(12.0),
                    TextColor(Color::srgb(0.95, 0.85, 0.30)),
                    text_shadow_default(),
                ));
            });

            // Currency 2 Text Readout (449, 5, 20, 10) -> 4x: (1796, 20, 80, 40)
            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(449.0 * MENU_SCALE),
                    top: px(5.0 * MENU_SCALE),
                    width: px(20.0 * MENU_SCALE),
                    height: px(10.0 * MENU_SCALE),
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|well| {
                well.spawn((
                    Text::new("0"),
                    make_tab_font(12.0),
                    TextColor(Color::srgb(0.30, 0.85, 0.95)),
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
        MenuState::Journal,
        MenuState::WorldMap,
        MenuState::Pause,
    ];

    let current = *menu_state.get();
    let cur_idx = match current {
        MenuState::Inventory => 0,
        MenuState::Journal => 1,
        MenuState::WorldMap => 2,
        MenuState::Pause | MenuState::Settings | MenuState::Controls => 3,
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
    mut labels_query: Query<(&MenuBarTabLabel, &mut TextColor, &mut Text)>,
) {
    let loc_changed = loc.as_ref().is_some_and(|l| l.is_changed());
    if !menu_state.is_changed() && !loc_changed {
        return;
    }

    let current = *menu_state.get();
    for (label, mut color, mut text) in &mut labels_query {
        let is_active = match label.action {
            MenuBarAction::TabInventory => current == MenuState::Inventory,
            MenuBarAction::TabJournal => current == MenuState::Journal,
            MenuBarAction::TabWorldMap => current == MenuState::WorldMap,
            MenuBarAction::TabSettings => {
                current == MenuState::Pause || current == MenuState::Settings || current == MenuState::Controls
            }
            _ => false,
        };

        if let Some(ref l) = loc {
            let localized_str = match label.action {
                MenuBarAction::TabInventory => l.t("menu.bar.inventory"),
                MenuBarAction::TabJournal => l.t("menu.bar.journal"),
                MenuBarAction::TabWorldMap => l.t("menu.bar.world_map"),
                MenuBarAction::TabSettings => l.t("menu.bar.system"),
                _ => "",
            };
            if !localized_str.is_empty() && **text != localized_str {
                **text = localized_str.to_string();
            }
        }

        *color = if is_active {
            TextColor(Color::srgb(1.0, 0.90, 0.40)) // Golden active highlight
        } else {
            TextColor(Color::srgb(0.85, 0.86, 0.90)) // Soft slate idle
        };
    }
}

fn handle_journal_overlay_visibility(
    mut commands: Commands,
    menu_state: Res<State<MenuState>>,
    overlay_query: Query<Entity, With<MenuBarJournalOverlay>>,
    app_font: Option<Res<AppFont>>,
    loc: Option<Res<crate::core::Localization>>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
) {
    let is_journal = *menu_state.get() == MenuState::Journal;
    let existing = overlay_query.iter().next();

    if is_journal && existing.is_none() {
        let gui_scale = game_settings.as_ref().map_or(4, |s| s.gui_scale).clamp(2, 4);
        let scale_factor = (gui_scale as f32 / 4.0).clamp(0.5, 1.0);
        let font_handle = app_font.as_ref().map(|f| f.source());
        let mut title_font = TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        };
        let mut subtitle_font = TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        };
        if let Some(font) = font_handle {
            title_font.font = font.clone();
            subtitle_font.font = font;
        }

        let title_text = loc.as_ref().map_or("EXPEDITION JOURNAL", |l| l.t("menu.journal.title"));
        let empty_text = loc.as_ref().map_or(
            "No entries recorded yet. Quests and bestiary system coming soon.",
            |l| l.t("menu.journal.empty"),
        );

        let dimming_alpha = game_settings
            .as_ref()
            .map_or(0.75, |s| s.background_dimming.alpha());

        commands
            .spawn((
                MenuBarJournalOverlay,
                crate::menu::MenuBackdrop,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(MENU_BAR_HEIGHT),
                    right: px(0.0),
                    bottom: px(0.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(12.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, dimming_alpha)),
                ZIndex(300),
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        UiTransform {
                            scale: Vec2::splat(scale_factor),
                            ..UiTransform::IDENTITY
                        },
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: px(12.0),
                            ..default()
                        },
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(title_text),
                            title_font,
                            TextColor(Color::srgb(0.95, 0.92, 0.80)),
                            text_shadow_default(),
                        ));
                        card.spawn((
                            Text::new(empty_text),
                            subtitle_font,
                            TextColor(Color::srgb(0.65, 0.68, 0.72)),
                            text_shadow_default(),
                        ));
                    });
            });
    } else if !is_journal && let Some(entity) = existing {
        commands.entity(entity).despawn();
    }
}
