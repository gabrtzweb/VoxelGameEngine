use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    ui::Pressed,
    ui_widgets::Button,
};

use crate::{
    audio::UiSound,
    core::{AppFont, Localization, text_shadow_default},
    player::{CharacterReady, CharacterSubrace, SelectedRace, SelectedSubrace},
    world::WorldSession,
};

use super::{GameSettings, MenuReturnTarget, MenuState, ThemedButton, theme::palette};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainMenuPhase {
    #[default]
    Splash,
    ActionMenu,
}

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct MainMenuSplashContainer;

#[derive(Component)]
pub struct MainMenuActionContainer;

#[derive(Component)]
pub struct MainMenuPromptText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuButtonAction {
    Play,
    SelectCharacter,
    Settings,
    Controls,
    Quit,
}

#[derive(Component)]
pub struct MainMenuButtonLabel;

#[derive(Component)]
struct MainMenuDisabledButton;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MainMenuPhase>()
            .add_systems(OnEnter(MenuState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(MenuState::MainMenu), despawn_main_menu)
            .add_systems(
                Update,
                (
                    handle_main_menu_splash_inputs,
                    handle_main_menu_action_buttons,
                    update_main_menu_button_hover,
                    animate_main_menu_splash,
                    sync_main_menu_phase,
                )
                    .run_if(in_state(MenuState::MainMenu)),
            );
    }
}

fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_font: Option<Res<AppFont>>,
    loc: Option<Res<Localization>>,
    _game_settings: Option<Res<GameSettings>>,
    phase: Res<MainMenuPhase>,
    selected_subrace: Option<Res<SelectedSubrace>>,
    character_ready: Option<Res<CharacterReady>>,
) {
    let font_handle = app_font.map(|f| f.source());
    let bg_texture = asset_server.load("textures/interfaces/main_menu_bg.png");

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

    let title_str = loc.as_ref().map_or("VOXEL ENGINE", |l| l.t("menu.main.title"));
    let version_str = loc.as_ref().map_or("Alpha 0.3.0", |l| l.t("menu.main.version"));
    let press_any_str = loc
        .as_ref()
        .map_or("Press any key to start", |l| l.t("menu.main.press_any_key"));

    let play_str = loc.as_ref().map_or("Play", |l| l.t("menu.main.play"));
    let settings_str = loc.as_ref().map_or("Settings", |l| l.t("menu.main.settings"));
    let controls_str = loc.as_ref().map_or("Controls", |l| l.t("menu.main.controls"));
    let quit_str = loc.as_ref().map_or("Quit to Desktop", |l| l.t("menu.main.quit"));
    let create_hint_str = loc.as_ref().map_or(
        "Create a character first to play",
        |l| l.t("menu.main.create_character_hint"),
    );
    let no_char_str = loc
        .as_ref()
        .map_or("No Character", |l| l.t("menu.main.no_character"));
    let no_char_sub_str = loc.as_ref().map_or(
        "Choose your race and subrace to begin",
        |l| l.t("menu.main.no_character_sub"),
    );
    let create_char_btn_str = loc
        .as_ref()
        .map_or("Create Character  >", |l| l.t("menu.main.create_character"));
    let change_char_btn_str = loc
        .as_ref()
        .map_or("Change Character  >", |l| l.t("menu.main.change_character"));

    let is_char_ready = character_ready.is_some_and(|r| r.0);
    let subrace = selected_subrace.map_or(CharacterSubrace::default(), |s| s.0);
    let player_name = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "Adventurer".to_string());

    let current_phase = *phase;

    commands
        .spawn((
            MainMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                right: px(0.0),
                bottom: px(0.0),
                width: percent(100.0),
                height: percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            ZIndex(300),
        ))
        .with_children(|root| {
            // Fullscreen Background Image
            root.spawn((
                Pickable::IGNORE,
                ImageNode {
                    image: bg_texture,
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
                ZIndex(0),
            ));

            // Atmospheric Vignette & Contrast Overlay
            root.spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.30)),
                ZIndex(1),
            ));

            // Splash Screen Container (Centered Title + Press Any Key)
            let splash_display = if current_phase == MainMenuPhase::Splash {
                Display::Flex
            } else {
                Display::None
            };
            let splash_visibility = if current_phase == MainMenuPhase::Splash {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            root.spawn((
                MainMenuSplashContainer,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    right: px(0.0),
                    bottom: px(0.0),
                    display: splash_display,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::vertical(percent(14.0)),
                    ..default()
                },
                splash_visibility,
                ZIndex(10),
            ))
            .with_children(|splash| {
                splash
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|hdr| {
                        hdr.spawn((
                            Text::new(title_str),
                            make_font(68.0),
                            TextColor(Color::srgb(1.0, 0.88, 0.42)),
                            TextShadow {
                                offset: Vec2::new(3.0, 3.0),
                                color: Color::srgba(0.0, 0.0, 0.0, 0.90),
                            },
                        ));
                        hdr.spawn((
                            Text::new(version_str),
                            make_font(18.0),
                            TextColor(Color::srgba(0.85, 0.89, 0.96, 0.80)),
                            text_shadow_default(),
                        ));
                    });

                splash.spawn((
                    MainMenuPromptText,
                    Text::new(press_any_str),
                    make_font(26.0),
                    TextColor(Color::srgb(0.96, 0.97, 1.0)),
                    TextShadow {
                        offset: Vec2::new(2.5, 2.5),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.95),
                    },
                ));
            });

            // Action Menu Container (Enshrouded-inspired 2-side layout: Left = Title & Buttons, Right = Character Panel)
            let action_display = if current_phase == MainMenuPhase::ActionMenu {
                Display::Flex
            } else {
                Display::None
            };
            let action_visibility = if current_phase == MainMenuPhase::ActionMenu {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            root.spawn((
                MainMenuActionContainer,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    right: px(0.0),
                    bottom: px(0.0),
                    display: action_display,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(percent(8.0), percent(8.0)),
                    ..default()
                },
                action_visibility,
                ZIndex(10),
            ))
            .with_children(|action_root| {
                // LEFT SIDE: Game Title + Navigation Buttons
                action_root
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        justify_content: JustifyContent::Center,
                        row_gap: px(14.0),
                        ..default()
                    })
                    .with_children(|left_col| {
                        // Title & Version
                        left_col
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::FlexStart,
                                row_gap: px(4.0),
                                margin: UiRect::bottom(px(22.0)),
                                ..default()
                            })
                            .with_children(|hdr| {
                                hdr.spawn((
                                    Pickable::IGNORE,
                                    Text::new(title_str),
                                    make_font(54.0),
                                    TextColor(Color::srgb(1.0, 0.88, 0.42)),
                                    TextShadow {
                                        offset: Vec2::new(3.0, 3.0),
                                        color: Color::srgba(0.0, 0.0, 0.0, 0.90),
                                    },
                                ));
                                hdr.spawn((
                                    Pickable::IGNORE,
                                    Text::new(version_str),
                                    make_font(15.0),
                                    TextColor(palette::TEXT_MUTED),
                                    text_shadow_default(),
                                ));
                            });

                        // Primary PLAY Button (locked until character is created/selected)
                        if is_char_ready {
                            left_col
                                .spawn((
                                    Button,
                                    Hovered::default(),
                                    MainMenuButtonAction::Play,
                                    ThemedButton::default(),
                                    Node {
                                        width: px(320.0),
                                        height: px(56.0),
                                        border: UiRect::all(px(2.0)),
                                        border_radius: BorderRadius::all(px(5.0)),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(palette::BTN_NORMAL_BG),
                                    BorderColor::all(palette::ACCENT_LINE),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Pickable::IGNORE,
                                        MainMenuButtonLabel,
                                        Text::new(play_str.to_uppercase()),
                                        make_font(22.0),
                                        TextColor(palette::TEXT_ACCENT),
                                        text_shadow_default(),
                                    ));
                                });
                        } else {
                            // Locked Play button + hint
                            left_col
                                .spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::FlexStart,
                                    row_gap: px(6.0),
                                    margin: UiRect::bottom(px(4.0)),
                                    ..default()
                                })
                                .with_children(|locked| {
                                    locked
                                        .spawn((
                                            MainMenuDisabledButton,
                                            Node {
                                                width: px(320.0),
                                                height: px(56.0),
                                                border: UiRect::all(px(1.5)),
                                                border_radius: BorderRadius::all(px(5.0)),
                                                display: Display::Flex,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.07, 0.08, 0.11, 0.72)),
                                            BorderColor::all(Color::srgba(0.24, 0.27, 0.34, 0.55)),
                                        ))
                                        .with_children(|btn| {
                                            btn.spawn((
                                                Pickable::IGNORE,
                                                Text::new(play_str.to_uppercase()),
                                                make_font(22.0),
                                                TextColor(Color::srgba(0.50, 0.54, 0.62, 0.65)),
                                            ));
                                        });

                                    locked.spawn((
                                        Pickable::IGNORE,
                                        Text::new(create_hint_str),
                                        make_font(13.0),
                                        TextColor(palette::TEXT_ACCENT),
                                        text_shadow_default(),
                                    ));
                                });
                        }

                        // Secondary Buttons
                        let secondary = [
                            (MainMenuButtonAction::Settings, settings_str),
                            (MainMenuButtonAction::Controls, controls_str),
                            (MainMenuButtonAction::Quit, quit_str),
                        ];

                        for (action, label) in secondary {
                            left_col
                                .spawn((
                                    Button,
                                    Hovered::default(),
                                    action,
                                    ThemedButton::default(),
                                    Node {
                                        width: px(320.0),
                                        height: px(46.0),
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(5.0)),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(palette::BTN_NORMAL_BG),
                                    BorderColor::all(palette::BTN_NORMAL_BORDER),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Pickable::IGNORE,
                                        MainMenuButtonLabel,
                                        Text::new(label),
                                        make_font(18.0),
                                        TextColor(palette::TEXT_PRIMARY),
                                        text_shadow_default(),
                                    ));
                                });
                        }
                    });

                // RIGHT SIDE: Character Showcase Panel (Enshrouded-style)
                action_root
                    .spawn((
                        Node {
                            width: px(320.0),
                            padding: UiRect::all(px(22.0)),
                            border: UiRect::all(px(1.5)),
                            border_radius: BorderRadius::all(px(8.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(16.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.86)),
                        BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                    ))
                    .with_children(|char_card| {
                        if is_char_ready {
                            let sub_icon = asset_server.load(subrace.icon_asset_path());
                            let sub_name = subrace.display_name(loc.as_deref());
                            let attrs = subrace.attributes();

                            // Portrait Frame
                            char_card
                                .spawn((
                                    Pickable::IGNORE,
                                    Node {
                                        width: px(84.0),
                                        height: px(84.0),
                                        padding: UiRect::all(px(4.0)),
                                        border: UiRect::all(px(2.0)),
                                        border_radius: BorderRadius::all(px(6.0)),
                                        display: Display::Flex,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(palette::PANEL_INNER_BG),
                                    BorderColor::all(palette::ACCENT_LINE),
                                ))
                                .with_children(|frame| {
                                    frame.spawn((
                                        Pickable::IGNORE,
                                        ImageNode {
                                            image: sub_icon,
                                            image_mode: NodeImageMode::Stretch,
                                            ..default()
                                        },
                                        Node {
                                            width: px(72.0),
                                            height: px(72.0),
                                            ..default()
                                        },
                                    ));
                                });

                            // Character Identity
                            char_card
                                .spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    row_gap: px(3.0),
                                    ..default()
                                })
                                .with_children(|id| {
                                    id.spawn((
                                        Pickable::IGNORE,
                                        Text::new(player_name),
                                        make_font(20.0),
                                        TextColor(palette::TEXT_PRIMARY),
                                        text_shadow_default(),
                                    ));
                                    id.spawn((
                                        Pickable::IGNORE,
                                        Text::new(sub_name),
                                        make_font(13.0),
                                        TextColor(palette::TEXT_ACCENT),
                                    ));
                                    id.spawn((
                                        Pickable::IGNORE,
                                        Text::new("Lv. 1"),
                                        make_font(12.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));
                                });

                            // Compact Attribute Summary Row
                            char_card
                                .spawn(Node {
                                    width: percent(100.0),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    column_gap: px(8.0),
                                    ..default()
                                })
                                .with_children(|row| {
                                    let pills = [
                                        (
                                            loc.as_ref().map_or("HP", |l| l.t("attr.health")),
                                            format!("{}", attrs.max_health),
                                        ),
                                        (
                                            loc.as_ref().map_or("Def", |l| l.t("attr.defense")),
                                            format!("{}", attrs.defense),
                                        ),
                                        (
                                            loc.as_ref().map_or("Spd", |l| l.t("attr.move_speed")),
                                            format!(
                                                "{}%",
                                                (attrs.move_speed_multiplier * 100.0).round() as i32
                                            ),
                                        ),
                                    ];

                                    for (k, v) in pills {
                                        row.spawn((
                                            Pickable::IGNORE,
                                            Node {
                                                flex_grow: 1.0,
                                                padding: UiRect::axes(px(8.0), px(6.0)),
                                                border_radius: BorderRadius::all(px(4.0)),
                                                display: Display::Flex,
                                                flex_direction: FlexDirection::Column,
                                                align_items: AlignItems::Center,
                                                row_gap: px(2.0),
                                                ..default()
                                            },
                                            BackgroundColor(palette::PANEL_INNER_BG),
                                        ))
                                        .with_children(|pill| {
                                            pill.spawn((
                                                Pickable::IGNORE,
                                                Text::new(k),
                                                make_font(10.0),
                                                TextColor(palette::TEXT_MUTED),
                                            ));
                                            pill.spawn((
                                                Pickable::IGNORE,
                                                Text::new(v),
                                                make_font(13.0),
                                                TextColor(palette::TEXT_PRIMARY),
                                            ));
                                        });
                                    }
                                });

                            // Change Character Button
                            char_card
                                .spawn((
                                    Button,
                                    Hovered::default(),
                                    MainMenuButtonAction::SelectCharacter,
                                    ThemedButton::default(),
                                    Node {
                                        width: percent(100.0),
                                        height: px(42.0),
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(5.0)),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(palette::BTN_NORMAL_BG),
                                    BorderColor::all(palette::BTN_NORMAL_BORDER),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Pickable::IGNORE,
                                        MainMenuButtonLabel,
                                        Text::new(change_char_btn_str),
                                        make_font(15.0),
                                        TextColor(palette::TEXT_PRIMARY),
                                        text_shadow_default(),
                                    ));
                                });
                        } else {
                            // Empty Portrait Slot
                            char_card
                                .spawn((
                                    Pickable::IGNORE,
                                    Node {
                                        width: px(84.0),
                                        height: px(84.0),
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(6.0)),
                                        display: Display::Flex,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(palette::PANEL_INNER_BG),
                                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                                ))
                                .with_children(|empty| {
                                    empty.spawn((
                                        Pickable::IGNORE,
                                        Text::new("?"),
                                        make_font(34.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));
                                });

                            char_card
                                .spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    row_gap: px(4.0),
                                    ..default()
                                })
                                .with_children(|txt| {
                                    txt.spawn((
                                        Pickable::IGNORE,
                                        Text::new(no_char_str),
                                        make_font(18.0),
                                        TextColor(palette::TEXT_PRIMARY),
                                        text_shadow_default(),
                                    ));
                                    txt.spawn((
                                        Pickable::IGNORE,
                                        Text::new(no_char_sub_str),
                                        make_font(12.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));
                                });

                            // Prominent Create Character Button
                            char_card
                                .spawn((
                                    Button,
                                    Hovered::default(),
                                    MainMenuButtonAction::SelectCharacter,
                                    ThemedButton::default(),
                                    Node {
                                        width: percent(100.0),
                                        height: px(46.0),
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(5.0)),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(palette::BTN_HOVER_BG),
                                    BorderColor::all(palette::ACCENT_LINE),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Pickable::IGNORE,
                                        MainMenuButtonLabel,
                                        Text::new(create_char_btn_str),
                                        make_font(16.0),
                                        TextColor(palette::TEXT_ACCENT),
                                        text_shadow_default(),
                                    ));
                                });
                        }
                    });
            });
        });
}

fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_main_menu_splash_inputs(
    mut phase: ResMut<MainMenuPhase>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if *phase != MainMenuPhase::Splash {
        return;
    }

    let key_pressed = keyboard.get_just_pressed().next().is_some();
    let mouse_pressed = mouse.get_just_pressed().next().is_some();

    if key_pressed || mouse_pressed {
        *phase = MainMenuPhase::ActionMenu;
        commands.trigger(UiSound::Click);
    }
}

fn handle_main_menu_action_buttons(
    buttons_query: Query<(&Hovered, Has<Pressed>, &MainMenuButtonAction), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut phase: ResMut<MainMenuPhase>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut menu_return_target: ResMut<MenuReturnTarget>,
    mut world_session: Option<ResMut<WorldSession>>,
    selected_subrace: Option<Res<SelectedSubrace>>,
    mut selected_race: Option<ResMut<SelectedRace>>,
    character_ready: Option<Res<CharacterReady>>,
    mut commands: Commands,
) {
    if *phase != MainMenuPhase::ActionMenu {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        *phase = MainMenuPhase::Splash;
        commands.trigger(UiSound::Click);
        return;
    }

    let left_clicked = mouse.just_pressed(MouseButton::Left);

    for (hovered, is_pressed, action) in &buttons_query {
        if hovered.get() && (left_clicked || is_pressed) {
            match action {
                MainMenuButtonAction::Play => {
                    if !character_ready.as_ref().is_some_and(|r| r.0) {
                        return;
                    }
                    if let (Some(sub), Some(race)) = (&selected_subrace, selected_race.as_mut()) {
                        race.0 = sub.0.rig_race();
                    }
                    if let Some(ref mut session) = world_session {
                        session.is_active = true;
                    }
                    menu_return_target.0 = MenuState::Pause;
                    next_state.set(MenuState::Loading);
                }
                MainMenuButtonAction::SelectCharacter => {
                    menu_return_target.0 = MenuState::MainMenu;
                    next_state.set(MenuState::RaceSelection);
                }
                MainMenuButtonAction::Settings => {
                    menu_return_target.0 = MenuState::MainMenu;
                    next_state.set(MenuState::Settings);
                }
                MainMenuButtonAction::Controls => {
                    menu_return_target.0 = MenuState::MainMenu;
                    next_state.set(MenuState::Controls);
                }
                MainMenuButtonAction::Quit => {
                    commands.write_message(bevy::app::AppExit::Success);
                }
            }
            return;
        }
    }
}

fn update_main_menu_button_hover(
    mut buttons_query: Query<
        (
            &Hovered,
            Has<Pressed>,
            &ThemedButton,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        With<Button>,
    >,
    mut labels_query: Query<&mut TextColor, With<MainMenuButtonLabel>>,
) {
    for (hovered, is_pressed, style, mut bg, mut border, children) in &mut buttons_query {
        let is_active = hovered.get() || is_pressed;
        let (bg_col, border_col) = style.colors(hovered.get(), is_pressed);
        bg.0 = bg_col;
        *border = BorderColor::all(border_col);

        for child in children.iter() {
            if let Ok(mut text_color) = labels_query.get_mut(child) {
                if is_active {
                    text_color.0 = palette::TEXT_ACCENT;
                } else {
                    text_color.0 = palette::TEXT_PRIMARY;
                }
            }
        }
    }
}

fn animate_main_menu_splash(
    time: Res<Time>,
    phase: Res<MainMenuPhase>,
    mut prompt_query: Query<&mut TextColor, With<MainMenuPromptText>>,
) {
    if *phase != MainMenuPhase::Splash {
        return;
    }

    let t = time.elapsed_secs() * 3.5;
    let alpha = 0.35 + 0.65 * t.sin().abs();

    for mut color in &mut prompt_query {
        color.0 = Color::srgba(0.96, 0.97, 1.0, alpha);
    }
}

fn sync_main_menu_phase(
    phase: Res<MainMenuPhase>,
    mut splash_query: Query<
        (&mut Node, &mut Visibility),
        (
            With<MainMenuSplashContainer>,
            Without<MainMenuActionContainer>,
        ),
    >,
    mut action_query: Query<
        (&mut Node, &mut Visibility),
        (
            With<MainMenuActionContainer>,
            Without<MainMenuSplashContainer>,
        ),
    >,
) {
    if !phase.is_changed() {
        return;
    }

    let is_splash = *phase == MainMenuPhase::Splash;

    for (mut node, mut vis) in &mut splash_query {
        node.display = if is_splash {
            Display::Flex
        } else {
            Display::None
        };
        *vis = if is_splash {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (mut node, mut vis) in &mut action_query {
        node.display = if is_splash {
            Display::None
        } else {
            Display::Flex
        };
        *vis = if is_splash {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}
