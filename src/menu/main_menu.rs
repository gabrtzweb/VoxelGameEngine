use std::sync::Arc;

use bevy::{
    ecs::system::SystemParam,
    picking::{Pickable, hover::Hovered},
    prelude::*,
    ui::Pressed,
    ui_widgets::Button,
};

use crate::{
    audio::UiSound,
    core::{AppFont, Localization, text_shadow_default},
    generation::{AdventureSpawnChoice, AdventureWorldMap, AdventureWorldScale, TerrainGenerator},
    player::{
        CharacterReady, CharacterSubrace, PLAYER_EYE_HEIGHT, Player, PlayerCamera, PlayerMotion,
        SelectedRace, SelectedSubrace, WorldSpawnPoint,
    },
    world::{VOXEL_SIZE, WorldSession},
};

use super::{GameSettings, MenuReturnTarget, MenuState, ThemedButton, theme::palette};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainMenuPhase {
    #[default]
    Splash,
    ActionMenu,
    NewGameModeSelect,
    AdventureSetup,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectedWorldMode {
    Adventure,
    #[default]
    Sandbox,
    Laboratory,
}

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct MainMenuSplashContainer;

#[derive(Component)]
pub struct MainMenuActionContainer;

#[derive(Component)]
pub struct MainMenuNewGameContainer;

#[derive(Component)]
pub struct MainMenuAdventureSetupContainer;

#[derive(Component)]
pub struct MainMenuPromptText;

#[derive(Component)]
struct AdventureSpawnOptionVisual(AdventureSpawnChoice);

#[derive(Component)]
struct AdventureScaleOptionVisual(AdventureWorldScale);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuButtonAction {
    NewGame,
    StartAdventure,
    StartSandbox,
    StartLaboratory,
    SelectAdventureSpawn(AdventureSpawnChoice),
    SelectAdventureScale(AdventureWorldScale),
    ConfirmAdventure,
    BackToNewGameSelect,
    BackToActionMenu,
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
            .init_resource::<SelectedWorldMode>()
            .init_resource::<AdventureSpawnChoice>()
            .init_resource::<AdventureWorldScale>()
            .add_systems(OnEnter(MenuState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(MenuState::MainMenu), despawn_main_menu)
            .add_systems(
                Update,
                (
                    handle_main_menu_splash_inputs,
                    handle_main_menu_action_buttons,
                    update_main_menu_button_hover,
                    sync_adventure_setup_selection,
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

    let title_str = loc
        .as_ref()
        .map_or("VOXEL ENGINE", |l| l.t("menu.main.title"));
    let version_str = loc
        .as_ref()
        .map_or("Alpha 0.3.0", |l| l.t("menu.main.version"));
    let press_any_str = loc
        .as_ref()
        .map_or("Press any key to start", |l| l.t("menu.main.press_any_key"));

    let new_game_str = loc
        .as_ref()
        .map_or("New Game", |l| l.t("menu.main.new_game"));
    let new_game_title_str = loc
        .as_ref()
        .map_or("SELECT WORLD MODE", |l| l.t("menu.main.new_game.title"));
    let new_game_subtitle_str = loc
        .as_ref()
        .map_or("Choose how you want to begin your journey", |l| {
            l.t("menu.main.new_game.subtitle")
        });
    let adv_title_str = loc
        .as_ref()
        .map_or("Adventure Mode", |l| l.t("menu.main.mode.adventure.title"));
    let adv_tag_str = loc
        .as_ref()
        .map_or("PRE-SELECTED MAP", |l| l.t("menu.main.mode.adventure.tag"));
    let adv_desc_str = loc.as_ref().map_or(
        "A bounded, handcrafted world on a pre-selected map filled with curated details and landmarks.",
        |l| l.t("menu.main.mode.adventure.desc"),
    );
    let adv_cta_str = loc
        .as_ref()
        .map_or("Start Adventure  >", |l| l.t("menu.main.mode.adventure.cta"));
    let sbx_title_str = loc
        .as_ref()
        .map_or("Sandbox World", |l| l.t("menu.main.mode.sandbox.title"));
    let sbx_tag_str = loc
        .as_ref()
        .map_or("INFINITE PROCEDURAL", |l| l.t("menu.main.mode.sandbox.tag"));
    let sbx_desc_str = loc.as_ref().map_or(
        "An infinite procedurally generated world with diverse biomes and total freedom to explore and build.",
        |l| l.t("menu.main.mode.sandbox.desc"),
    );
    let sbx_cta_str = loc
        .as_ref()
        .map_or("Start Sandbox  >", |l| l.t("menu.main.mode.sandbox.cta"));
    let lab_title_str = loc
        .as_ref()
        .map_or("Laboratory", |l| l.t("menu.main.mode.laboratory.title"));
    let lab_tag_str = loc
        .as_ref()
        .map_or("BIOME ATLAS", |l| l.t("menu.main.mode.laboratory.tag"));
    let lab_desc_str = loc.as_ref().map_or(
        "A compact circular world showcasing all 78 biomes side-by-side in radial slices for rapid inspection.",
        |l| l.t("menu.main.mode.laboratory.desc"),
    );
    let lab_cta_str = loc
        .as_ref()
        .map_or("Start Laboratory  >", |l| l.t("menu.main.mode.laboratory.cta"));
    let back_str = loc.as_ref().map_or("<  Back", |l| l.t("menu.main.back"));
    let adv_setup_title = loc
        .as_ref()
        .map_or("ADVENTURE WORLD SETUP", |l| l.t("menu.main.adventure.title"));
    let adv_setup_subtitle = loc.as_ref().map_or(
        "Choose your starting region on the Western Continent and world scale",
        |l| l.t("menu.main.adventure.subtitle"),
    );
    let adv_spawn_header = loc.as_ref().map_or(
        "STARTING REGION (WESTERN CONTINENT)",
        |l| l.t("menu.main.adventure.spawn_header"),
    );
    let adv_scale_header = loc
        .as_ref()
        .map_or("WORLD SCALE", |l| l.t("menu.main.adventure.scale_header"));
    let adv_scale_hint = loc.as_ref().map_or(
        "Mini fits inside max LOD view (64 chunks) for fast full-world testing.",
        |l| l.t("menu.main.adventure.scale_hint"),
    );
    let adv_confirm_str = loc
        .as_ref()
        .map_or("Start Adventure  >", |l| l.t("menu.main.adventure.confirm"));
    let adv_map_preview: Handle<Image> =
        asset_server.load("maps/adventure/map_preview.png");
    let settings_str = loc
        .as_ref()
        .map_or("Settings", |l| l.t("menu.main.settings"));
    let controls_str = loc
        .as_ref()
        .map_or("Controls", |l| l.t("menu.main.controls"));
    let quit_str = loc
        .as_ref()
        .map_or("Quit to Desktop", |l| l.t("menu.main.quit"));
    let create_hint_str = loc
        .as_ref()
        .map_or("Create a character first to play", |l| {
            l.t("menu.main.create_character_hint")
        });
    let no_char_str = loc
        .as_ref()
        .map_or("No Character", |l| l.t("menu.main.no_character"));
    let no_char_sub_str = loc
        .as_ref()
        .map_or("Choose your race and subrace to begin", |l| {
            l.t("menu.main.no_character_sub")
        });
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

            // Action Menu Container (2-side layout: Left = Title & Buttons, Right = Character Panel)
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

                        // Primary NEW GAME Button (locked until character is created/selected)
                        if is_char_ready {
                            left_col
                                .spawn((
                                    Button,
                                    Hovered::default(),
                                    MainMenuButtonAction::NewGame,
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
                                        Text::new(new_game_str.to_uppercase()),
                                        make_font(22.0),
                                        TextColor(palette::TEXT_ACCENT),
                                        text_shadow_default(),
                                    ));
                                });
                        } else {
                            // Locked New Game button + hint
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
                                                Text::new(new_game_str.to_uppercase()),
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

                // RIGHT SIDE: Character Showcase Panel
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
                                                (attrs.move_speed_multiplier * 100.0).round()
                                                    as i32
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
                                        .with_children(
                                            |pill| {
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
                                            },
                                        );
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

            // New Game Mode Selection Container (Adventure Mode vs Sandbox World)
            let new_game_display = if current_phase == MainMenuPhase::NewGameModeSelect {
                Display::Flex
            } else {
                Display::None
            };
            let new_game_visibility = if current_phase == MainMenuPhase::NewGameModeSelect {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            root.spawn((
                MainMenuNewGameContainer,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    right: px(0.0),
                    bottom: px(0.0),
                    display: new_game_display,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(28.0),
                    padding: UiRect::axes(percent(6.0), percent(6.0)),
                    ..default()
                },
                new_game_visibility,
                ZIndex(10),
            ))
            .with_children(|ng| {
                // Header
                ng.spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(6.0),
                    ..default()
                })
                .with_children(|hdr| {
                    hdr.spawn((
                        Pickable::IGNORE,
                        Text::new(new_game_title_str),
                        make_font(34.0),
                        TextColor(Color::srgb(1.0, 0.88, 0.42)),
                        TextShadow {
                            offset: Vec2::new(2.5, 2.5),
                            color: Color::srgba(0.0, 0.0, 0.0, 0.90),
                        },
                    ));
                    hdr.spawn((
                        Pickable::IGNORE,
                        Text::new(new_game_subtitle_str),
                        make_font(15.0),
                        TextColor(palette::TEXT_MUTED),
                        text_shadow_default(),
                    ));
                });

                // Three World Mode Cards Side-by-Side
                ng.spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Stretch,
                    justify_content: JustifyContent::Center,
                    column_gap: px(20.0),
                    ..default()
                })
                .with_children(|cards| {
                    let modes = [
                        (
                            MainMenuButtonAction::StartLaboratory,
                            lab_tag_str,
                            lab_title_str,
                            lab_desc_str,
                            lab_cta_str,
                            Color::srgb(0.44, 0.92, 0.64),
                        ),
                        (
                            MainMenuButtonAction::StartSandbox,
                            sbx_tag_str,
                            sbx_title_str,
                            sbx_desc_str,
                            sbx_cta_str,
                            Color::srgb(0.42, 0.82, 0.95),
                        ),
                        (
                            MainMenuButtonAction::StartAdventure,
                            adv_tag_str,
                            adv_title_str,
                            adv_desc_str,
                            adv_cta_str,
                            Color::srgb(0.95, 0.78, 0.36),
                        ),
                    ];

                    for (action, tag, title, desc, cta, tag_color) in modes {
                        cards
                            .spawn((
                                Button,
                                Hovered::default(),
                                action,
                                ThemedButton::default(),
                                Node {
                                    width: px(310.0),
                                    min_height: px(224.0),
                                    padding: UiRect::all(px(24.0)),
                                    border: UiRect::all(px(2.0)),
                                    border_radius: BorderRadius::all(px(8.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::FlexStart,
                                    row_gap: px(14.0),
                                    ..default()
                                },
                                BackgroundColor(palette::BTN_NORMAL_BG),
                                BorderColor::all(palette::BTN_NORMAL_BORDER),
                            ))
                            .with_children(|card| {
                                // Tag pill
                                card.spawn((
                                    Pickable::IGNORE,
                                    Node {
                                        padding: UiRect::axes(px(10.0), px(4.0)),
                                        border_radius: BorderRadius::all(px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.78)),
                                ))
                                .with_children(|pill| {
                                    pill.spawn((
                                        Pickable::IGNORE,
                                        Text::new(tag),
                                        make_font(11.0),
                                        TextColor(tag_color),
                                    ));
                                });

                                // Mode Title (highlighted on hover via MainMenuButtonLabel)
                                card.spawn((
                                    Pickable::IGNORE,
                                    MainMenuButtonLabel,
                                    Text::new(title),
                                    make_font(24.0),
                                    TextColor(palette::TEXT_PRIMARY),
                                    text_shadow_default(),
                                ));

                                // Short Description
                                card.spawn((
                                    Pickable::IGNORE,
                                    Text::new(desc),
                                    make_font(14.0),
                                    TextColor(palette::TEXT_MUTED),
                                    Node {
                                        flex_grow: 1.0,
                                        ..default()
                                    },
                                ));

                                // Bottom CTA label
                                card.spawn((
                                    Pickable::IGNORE,
                                    Text::new(cta),
                                    make_font(15.0),
                                    TextColor(palette::TEXT_ACCENT),
                                    text_shadow_default(),
                                ));
                            });
                    }
                });

                // Back Button
                ng.spawn((
                    Button,
                    Hovered::default(),
                    MainMenuButtonAction::BackToActionMenu,
                    ThemedButton::default(),
                    Node {
                        width: px(220.0),
                        height: px(44.0),
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
                        Text::new(back_str),
                        make_font(16.0),
                        TextColor(palette::TEXT_PRIMARY),
                        text_shadow_default(),
                    ));
                });
            });

            // Adventure Setup Container (Map Preview + 3 Starting Spawns + 3 World Scales)
            let adv_display = if current_phase == MainMenuPhase::AdventureSetup {
                Display::Flex
            } else {
                Display::None
            };
            let adv_visibility = if current_phase == MainMenuPhase::AdventureSetup {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            root.spawn((
                MainMenuAdventureSetupContainer,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    right: px(0.0),
                    bottom: px(0.0),
                    display: adv_display,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(20.0),
                    padding: UiRect::axes(percent(4.0), percent(4.0)),
                    ..default()
                },
                adv_visibility,
                ZIndex(10),
            ))
            .with_children(|setup| {
                // Header
                setup
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(4.0),
                        ..default()
                    })
                    .with_children(|hdr| {
                        hdr.spawn((
                            Pickable::IGNORE,
                            Text::new(adv_setup_title),
                            make_font(30.0),
                            TextColor(Color::srgb(1.0, 0.88, 0.42)),
                            TextShadow {
                                offset: Vec2::new(2.5, 2.5),
                                color: Color::srgba(0.0, 0.0, 0.0, 0.90),
                            },
                        ));
                        hdr.spawn((
                            Pickable::IGNORE,
                            Text::new(adv_setup_subtitle),
                            make_font(14.0),
                            TextColor(palette::TEXT_MUTED),
                            text_shadow_default(),
                        ));
                    });

                // Main 2-Column Panel
                setup
                    .spawn((
                        Node {
                            padding: UiRect::all(px(20.0)),
                            border: UiRect::all(px(1.5)),
                            border_radius: BorderRadius::all(px(8.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::FlexStart,
                            column_gap: px(22.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.90)),
                        BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                    ))
                    .with_children(|panel| {
                        // LEFT COLUMN: Map Preview with 3 Interactive Pins + Scale Selector
                        panel
                            .spawn(Node {
                                width: px(540.0),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(14.0),
                                ..default()
                            })
                            .with_children(|left| {
                                // Framed World Map Preview (540 x 301 = exact 1024:571 ratio)
                                let map_w = 540.0;
                                let map_h = 301.0;
                                left.spawn((
                                    Node {
                                        width: px(map_w),
                                        height: px(map_h),
                                        border: UiRect::all(px(2.0)),
                                        border_radius: BorderRadius::all(px(6.0)),
                                        overflow: Overflow::clip(),
                                        position_type: PositionType::Relative,
                                        ..default()
                                    },
                                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                                ))
                                .with_children(|map_box| {
                                    map_box.spawn((
                                        Pickable::IGNORE,
                                        ImageNode {
                                            image: adv_map_preview,
                                            image_mode: NodeImageMode::Stretch,
                                            ..default()
                                        },
                                        Node {
                                            width: percent(100.0),
                                            height: percent(100.0),
                                            ..default()
                                        },
                                    ));

                                    // 3 Interactive Spawn Pins on the Western Continent
                                    for (idx, choice) in
                                        AdventureSpawnChoice::ALL.into_iter().enumerate()
                                    {
                                        let (u, v) = choice.preview_uv();
                                        let pin_size = 26.0;
                                        let left_px = u * map_w - pin_size * 0.5;
                                        let top_px = v * map_h - pin_size * 0.5;
                                        map_box
                                            .spawn((
                                                Button,
                                                Hovered::default(),
                                                MainMenuButtonAction::SelectAdventureSpawn(choice),
                                                AdventureSpawnOptionVisual(choice),
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: px(left_px),
                                                    top: px(top_px),
                                                    width: px(pin_size),
                                                    height: px(pin_size),
                                                    border: UiRect::all(px(2.0)),
                                                    border_radius: BorderRadius::all(px(
                                                        pin_size * 0.5,
                                                    )),
                                                    display: Display::Flex,
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgba(
                                                    0.07, 0.08, 0.12, 0.92,
                                                )),
                                                BorderColor::all(palette::BTN_NORMAL_BORDER),
                                                ZIndex(5),
                                            ))
                                            .with_children(|pin| {
                                                pin.spawn((
                                                    Pickable::IGNORE,
                                                    Text::new(format!("{}", idx + 1)),
                                                    make_font(13.0),
                                                    TextColor(palette::TEXT_PRIMARY),
                                                    text_shadow_default(),
                                                ));
                                            });
                                    }
                                });

                                // World Scale Selector
                                left.spawn((
                                    Pickable::IGNORE,
                                    Text::new(adv_scale_header),
                                    make_font(12.0),
                                    TextColor(palette::TEXT_ACCENT),
                                ));

                                left.spawn(Node {
                                    width: percent(100.0),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(8.0),
                                    ..default()
                                })
                                .with_children(|row| {
                                    for sc in AdventureWorldScale::ALL {
                                        let label = loc.as_ref().map_or(sc.fallback_label(), |l| {
                                            l.t(sc.loc_key())
                                        });
                                        row.spawn((
                                            Button,
                                            Hovered::default(),
                                            MainMenuButtonAction::SelectAdventureScale(sc),
                                            AdventureScaleOptionVisual(sc),
                                            Node {
                                                flex_grow: 1.0,
                                                height: px(38.0),
                                                padding: UiRect::horizontal(px(8.0)),
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
                                                Text::new(label),
                                                make_font(12.5),
                                                TextColor(palette::TEXT_PRIMARY),
                                                text_shadow_default(),
                                            ));
                                        });
                                    }
                                });

                                left.spawn((
                                    Pickable::IGNORE,
                                    Text::new(adv_scale_hint),
                                    make_font(11.5),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                            });

                        // RIGHT COLUMN: 3 Starting Spawn Cards
                        panel
                            .spawn(Node {
                                width: px(360.0),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(10.0),
                                ..default()
                            })
                            .with_children(|right| {
                                right.spawn((
                                    Pickable::IGNORE,
                                    Text::new(adv_spawn_header),
                                    make_font(12.0),
                                    TextColor(palette::TEXT_ACCENT),
                                    Node {
                                        margin: UiRect::bottom(px(2.0)),
                                        ..default()
                                    },
                                ));

                                for choice in AdventureSpawnChoice::ALL {
                                    let title = loc
                                        .as_ref()
                                        .map_or(choice.fallback_title(), |l| l.t(choice.title_key()));
                                    let desc = loc
                                        .as_ref()
                                        .map_or(choice.fallback_desc(), |l| l.t(choice.desc_key()));

                                    right
                                        .spawn((
                                            Button,
                                            Hovered::default(),
                                            MainMenuButtonAction::SelectAdventureSpawn(choice),
                                            AdventureSpawnOptionVisual(choice),
                                            Node {
                                                width: percent(100.0),
                                                padding: UiRect::all(px(14.0)),
                                                border: UiRect::all(px(1.5)),
                                                border_radius: BorderRadius::all(px(6.0)),
                                                display: Display::Flex,
                                                flex_direction: FlexDirection::Column,
                                                row_gap: px(5.0),
                                                ..default()
                                            },
                                            BackgroundColor(palette::BTN_NORMAL_BG),
                                            BorderColor::all(palette::BTN_NORMAL_BORDER),
                                        ))
                                        .with_children(|card| {
                                            card.spawn((
                                                Pickable::IGNORE,
                                                Text::new(title),
                                                make_font(16.0),
                                                TextColor(palette::TEXT_PRIMARY),
                                                text_shadow_default(),
                                            ));
                                            card.spawn((
                                                Pickable::IGNORE,
                                                Text::new(desc),
                                                make_font(12.5),
                                                TextColor(palette::TEXT_MUTED),
                                            ));
                                        });
                                }
                            });
                    });

                // Bottom Action Row (Back + Start Adventure)
                setup
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(16.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Button,
                            Hovered::default(),
                            MainMenuButtonAction::BackToNewGameSelect,
                            ThemedButton::default(),
                            Node {
                                width: px(200.0),
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
                                Text::new(back_str),
                                make_font(16.0),
                                TextColor(palette::TEXT_PRIMARY),
                                text_shadow_default(),
                            ));
                        });

                        row.spawn((
                            Button,
                            Hovered::default(),
                            MainMenuButtonAction::ConfirmAdventure,
                            ThemedButton::default(),
                            Node {
                                width: px(260.0),
                                height: px(46.0),
                                border: UiRect::all(px(2.0)),
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
                                Text::new(adv_confirm_str),
                                make_font(18.0),
                                TextColor(palette::TEXT_ACCENT),
                                text_shadow_default(),
                            ));
                        });
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

#[derive(SystemParam)]
struct WorldLaunchParams<'w, 's> {
    selected_world_mode: ResMut<'w, SelectedWorldMode>,
    selected_spawn: ResMut<'w, AdventureSpawnChoice>,
    selected_scale: ResMut<'w, AdventureWorldScale>,
    terrain_generator: Option<ResMut<'w, TerrainGenerator>>,
    world_spawn: Option<ResMut<'w, WorldSpawnPoint>>,
    player_q: Query<'w, 's, (&'static mut Transform, &'static mut PlayerMotion), With<Player>>,
    camera_q: Query<'w, 's, &'static mut Transform, (With<PlayerCamera>, Without<Player>)>,
    world_session: Option<ResMut<'w, WorldSession>>,
    game_mode: Option<ResMut<'w, crate::player::GameMode>>,
}

#[allow(clippy::too_many_arguments)]
fn handle_main_menu_action_buttons(
    buttons_query: Query<(Entity, &Hovered, &MainMenuButtonAction), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut phase: ResMut<MainMenuPhase>,
    mut launch: WorldLaunchParams,
    mut next_state: ResMut<NextState<MenuState>>,
    mut menu_return_target: ResMut<MenuReturnTarget>,
    selected_subrace: Option<Res<SelectedSubrace>>,
    mut selected_race: Option<ResMut<SelectedRace>>,
    character_ready: Option<Res<CharacterReady>>,
    mut commands: Commands,
) {
    if *phase == MainMenuPhase::Splash {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        *phase = match *phase {
            MainMenuPhase::AdventureSetup => MainMenuPhase::NewGameModeSelect,
            MainMenuPhase::NewGameModeSelect => MainMenuPhase::ActionMenu,
            MainMenuPhase::ActionMenu => MainMenuPhase::Splash,
            MainMenuPhase::Splash => MainMenuPhase::Splash,
        };
        for (entity, _, _) in &buttons_query {
            commands.entity(entity).remove::<Pressed>();
        }
        commands.trigger(UiSound::Click);
        return;
    }

    let left_clicked = mouse.just_pressed(MouseButton::Left);
    if !left_clicked {
        return;
    }

    for (clicked_entity, hovered, action) in &buttons_query {
        if hovered.get() {
            commands.entity(clicked_entity).remove::<Pressed>();
            match *action {
                MainMenuButtonAction::NewGame => {
                    if *phase != MainMenuPhase::ActionMenu
                        || !character_ready.as_ref().is_some_and(|r| r.0)
                    {
                        return;
                    }
                    *phase = MainMenuPhase::NewGameModeSelect;
                    commands.trigger(UiSound::Click);
                }
                MainMenuButtonAction::StartAdventure => {
                    if *phase != MainMenuPhase::NewGameModeSelect
                        || !character_ready.as_ref().is_some_and(|r| r.0)
                    {
                        return;
                    }
                    *launch.selected_world_mode = SelectedWorldMode::Adventure;
                    *phase = MainMenuPhase::AdventureSetup;
                    commands.trigger(UiSound::Click);
                }
                MainMenuButtonAction::SelectAdventureSpawn(choice) => {
                    if *phase != MainMenuPhase::AdventureSetup {
                        return;
                    }
                    if *launch.selected_spawn != choice {
                        *launch.selected_spawn = choice;
                        commands.trigger(UiSound::Click);
                    }
                }
                MainMenuButtonAction::SelectAdventureScale(scale) => {
                    if *phase != MainMenuPhase::AdventureSetup {
                        return;
                    }
                    if *launch.selected_scale != scale {
                        *launch.selected_scale = scale;
                        commands.trigger(UiSound::Click);
                    }
                }
                MainMenuButtonAction::ConfirmAdventure => {
                    if *phase != MainMenuPhase::AdventureSetup
                        || !character_ready.as_ref().is_some_and(|r| r.0)
                    {
                        return;
                    }
                    *launch.selected_world_mode = SelectedWorldMode::Adventure;

                    // Load handcrafted Adventure World Map and teleport player to chosen starting region
                    if let Some(ref mut terrain_gen) = launch.terrain_generator {
                        match AdventureWorldMap::load_with_scale(*launch.selected_scale) {
                            Ok(map) => {
                                let (sx, sz) = map.spawn_world_xz(*launch.selected_spawn);
                                terrain_gen.adventure_map = Some(Arc::new(map));
                                terrain_gen.laboratory_map = None;
                                terrain_gen.version = terrain_gen.version.wrapping_add(1);

                                let col = terrain_gen.sample_column(sx, sz);
                                let sy = (col.terrain_height.max(terrain_gen.sea_level) + 2) as f32
                                    * VOXEL_SIZE;
                                let spawn_pos =
                                    Vec3::new(sx as f32 * VOXEL_SIZE, sy, sz as f32 * VOXEL_SIZE);
                                if let Some(ref mut ws) = launch.world_spawn {
                                    ws.0 = spawn_pos;
                                }
                                for (mut tf, mut motion) in &mut launch.player_q {
                                    tf.translation = spawn_pos;
                                    motion.velocity = Vec3::ZERO;
                                }
                                for mut cam_tf in &mut launch.camera_q {
                                    cam_tf.translation = spawn_pos + Vec3::Y * PLAYER_EYE_HEIGHT;
                                }
                            }
                            Err(err) => {
                                error!("Could not load Adventure World map: {err}");
                            }
                        }
                    }

                    if let (Some(sub), Some(race)) = (&selected_subrace, selected_race.as_mut()) {
                        race.0 = sub.0.rig_race();
                    }
                    if let Some(ref mut session) = launch.world_session {
                        session.is_active = true;
                    }
                    *phase = MainMenuPhase::ActionMenu;
                    menu_return_target.0 = MenuState::Pause;
                    next_state.set(MenuState::Loading);
                }
                MainMenuButtonAction::StartSandbox => {
                    if *phase != MainMenuPhase::NewGameModeSelect
                        || !character_ready.as_ref().is_some_and(|r| r.0)
                    {
                        return;
                    }
                    *launch.selected_world_mode = SelectedWorldMode::Sandbox;

                    if let Some(ref mut terrain_gen) = launch.terrain_generator {
                        terrain_gen.adventure_map = None;
                        terrain_gen.laboratory_map = None;
                        // Fresh random seed every time a new Sandbox world is created
                        let nanos = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_or(0u64, |d| d.as_nanos() as u64);
                        let mut z = nanos.wrapping_add(0x9E37_79B9_7F4A_7C15);
                        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                        terrain_gen.seed = ((z ^ (z >> 31)) as u32).max(1);
                        terrain_gen.version = terrain_gen.version.wrapping_add(1);
                        let (sx, sz) = terrain_gen.find_safe_land_spawn(-10, 14);
                        let col = terrain_gen.sample_column(sx, sz);
                        let sy = (col.terrain_height.max(terrain_gen.sea_level) + 2) as f32
                            * VOXEL_SIZE;
                        let spawn_pos =
                            Vec3::new(sx as f32 * VOXEL_SIZE, sy, sz as f32 * VOXEL_SIZE);
                        if let Some(ref mut ws) = launch.world_spawn {
                            ws.0 = spawn_pos;
                        }
                        for (mut tf, mut motion) in &mut launch.player_q {
                            tf.translation = spawn_pos;
                            motion.velocity = Vec3::ZERO;
                        }
                        for mut cam_tf in &mut launch.camera_q {
                            cam_tf.translation = spawn_pos + Vec3::Y * PLAYER_EYE_HEIGHT;
                        }
                    }

                    if let (Some(sub), Some(race)) = (&selected_subrace, selected_race.as_mut()) {
                        race.0 = sub.0.rig_race();
                    }
                    if let Some(ref mut session) = launch.world_session {
                        session.is_active = true;
                    }
                    *phase = MainMenuPhase::ActionMenu;
                    menu_return_target.0 = MenuState::Pause;
                    next_state.set(MenuState::Loading);
                }
                MainMenuButtonAction::StartLaboratory => {
                    if *phase != MainMenuPhase::NewGameModeSelect
                        || !character_ready.as_ref().is_some_and(|r| r.0)
                    {
                        return;
                    }
                    *launch.selected_world_mode = SelectedWorldMode::Laboratory;
                    if let Some(ref mut gm) = launch.game_mode {
                        **gm = crate::player::GameMode::Creative;
                    }

                    if let Some(ref mut terrain_gen) = launch.terrain_generator {
                        let lab = crate::generation::LaboratoryWorldMap::new();
                        let (sx, sz) = lab.spawn_world_xz();
                        terrain_gen.adventure_map = None;
                        terrain_gen.laboratory_map = Some(Arc::new(lab));
                        terrain_gen.version = terrain_gen.version.wrapping_add(1);

                        let col = terrain_gen.sample_column(sx, sz);
                        let sy = (col.terrain_height.max(terrain_gen.sea_level) + 2) as f32
                            * VOXEL_SIZE;
                        let spawn_pos =
                            Vec3::new(sx as f32 * VOXEL_SIZE, sy, sz as f32 * VOXEL_SIZE);
                        if let Some(ref mut ws) = launch.world_spawn {
                            ws.0 = spawn_pos;
                        }
                        for (mut tf, mut motion) in &mut launch.player_q {
                            tf.translation = spawn_pos;
                            motion.velocity = Vec3::ZERO;
                        }
                        for mut cam_tf in &mut launch.camera_q {
                            cam_tf.translation = spawn_pos + Vec3::Y * PLAYER_EYE_HEIGHT;
                        }
                    }

                    if let (Some(sub), Some(race)) = (&selected_subrace, selected_race.as_mut()) {
                        race.0 = sub.0.rig_race();
                    }
                    if let Some(ref mut session) = launch.world_session {
                        session.is_active = true;
                    }
                    *phase = MainMenuPhase::ActionMenu;
                    menu_return_target.0 = MenuState::Pause;
                    next_state.set(MenuState::Loading);
                }
                MainMenuButtonAction::BackToNewGameSelect => {
                    if *phase != MainMenuPhase::AdventureSetup {
                        return;
                    }
                    *phase = MainMenuPhase::NewGameModeSelect;
                    commands.trigger(UiSound::Click);
                }
                MainMenuButtonAction::BackToActionMenu => {
                    if *phase != MainMenuPhase::NewGameModeSelect {
                        return;
                    }
                    *phase = MainMenuPhase::ActionMenu;
                    commands.trigger(UiSound::Click);
                }
                MainMenuButtonAction::SelectCharacter => {
                    if *phase != MainMenuPhase::ActionMenu {
                        return;
                    }
                    menu_return_target.0 = MenuState::MainMenu;
                    next_state.set(MenuState::RaceSelection);
                }
                MainMenuButtonAction::Settings => {
                    if *phase != MainMenuPhase::ActionMenu {
                        return;
                    }
                    menu_return_target.0 = MenuState::MainMenu;
                    next_state.set(MenuState::Settings);
                }
                MainMenuButtonAction::Controls => {
                    if *phase != MainMenuPhase::ActionMenu {
                        return;
                    }
                    menu_return_target.0 = MenuState::MainMenu;
                    next_state.set(MenuState::Controls);
                }
                MainMenuButtonAction::Quit => {
                    if *phase != MainMenuPhase::ActionMenu {
                        return;
                    }
                    commands.write_message(bevy::app::AppExit::Success);
                }
            }
            return;
        }
    }
}

fn sync_adventure_setup_selection(
    selected_spawn: Res<AdventureSpawnChoice>,
    selected_scale: Res<AdventureWorldScale>,
    mut spawn_q: Query<
        (
            &AdventureSpawnOptionVisual,
            &Hovered,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Without<AdventureScaleOptionVisual>,
    >,
    mut scale_q: Query<
        (
            &AdventureScaleOptionVisual,
            &Hovered,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Without<AdventureSpawnOptionVisual>,
    >,
) {
    for (opt, hovered, mut bg, mut border) in &mut spawn_q {
        let is_selected = opt.0 == *selected_spawn;
        if is_selected {
            bg.0 = Color::srgba(0.14, 0.16, 0.22, 0.96);
            *border = BorderColor::all(palette::ACCENT_LINE);
        } else if hovered.get() {
            bg.0 = palette::BTN_HOVER_BG;
            *border = BorderColor::all(palette::BTN_HOVER_BORDER);
        } else {
            bg.0 = palette::BTN_NORMAL_BG;
            *border = BorderColor::all(palette::BTN_NORMAL_BORDER);
        }
    }

    for (opt, hovered, mut bg, mut border) in &mut scale_q {
        let is_selected = opt.0 == *selected_scale;
        if is_selected {
            bg.0 = Color::srgba(0.14, 0.16, 0.22, 0.96);
            *border = BorderColor::all(palette::ACCENT_LINE);
        } else if hovered.get() {
            bg.0 = palette::BTN_HOVER_BG;
            *border = BorderColor::all(palette::BTN_HOVER_BORDER);
        } else {
            bg.0 = palette::BTN_NORMAL_BG;
            *border = BorderColor::all(palette::BTN_NORMAL_BORDER);
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

#[allow(clippy::type_complexity)]
fn sync_main_menu_phase(
    phase: Res<MainMenuPhase>,
    mut splash_query: Query<
        (&mut Node, &mut Visibility),
        (
            With<MainMenuSplashContainer>,
            Without<MainMenuActionContainer>,
            Without<MainMenuNewGameContainer>,
            Without<MainMenuAdventureSetupContainer>,
        ),
    >,
    mut action_query: Query<
        (&mut Node, &mut Visibility),
        (
            With<MainMenuActionContainer>,
            Without<MainMenuSplashContainer>,
            Without<MainMenuNewGameContainer>,
            Without<MainMenuAdventureSetupContainer>,
        ),
    >,
    mut new_game_query: Query<
        (&mut Node, &mut Visibility),
        (
            With<MainMenuNewGameContainer>,
            Without<MainMenuSplashContainer>,
            Without<MainMenuActionContainer>,
            Without<MainMenuAdventureSetupContainer>,
        ),
    >,
    mut adv_setup_query: Query<
        (&mut Node, &mut Visibility),
        (
            With<MainMenuAdventureSetupContainer>,
            Without<MainMenuSplashContainer>,
            Without<MainMenuActionContainer>,
            Without<MainMenuNewGameContainer>,
        ),
    >,
) {
    if !phase.is_changed() {
        return;
    }

    let is_splash = *phase == MainMenuPhase::Splash;
    let is_action = *phase == MainMenuPhase::ActionMenu;
    let is_new_game = *phase == MainMenuPhase::NewGameModeSelect;
    let is_adv_setup = *phase == MainMenuPhase::AdventureSetup;

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
        node.display = if is_action {
            Display::Flex
        } else {
            Display::None
        };
        *vis = if is_action {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (mut node, mut vis) in &mut new_game_query {
        node.display = if is_new_game {
            Display::Flex
        } else {
            Display::None
        };
        *vis = if is_new_game {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (mut node, mut vis) in &mut adv_setup_query {
        node.display = if is_adv_setup {
            Display::Flex
        } else {
            Display::None
        };
        *vis = if is_adv_setup {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
