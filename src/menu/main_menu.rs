use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::Button,
};

use crate::{
    audio::UiSound,
    core::{AppFont, Localization, text_shadow_default},
};

use super::{GameSettings, MenuButtonTexture, MenuReturnTarget, MenuState};

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
    Settings,
    Controls,
    Quit,
}

#[derive(Component)]
pub struct MainMenuButtonLabel;

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
) {
    let font_handle = app_font.map(|f| f.source());
    let bg_texture = asset_server.load("textures/interfaces/main_menu_bg.png");
    let btn_normal = asset_server.load("textures/interfaces/containers/buttons/button_large.png");
    let btn_hover = asset_server.load("textures/interfaces/containers/buttons/button_large_hover.png");

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
    let version_str = loc.as_ref().map_or("Alpha 0.1.0", |l| l.t("menu.main.version"));
    let press_any_str = loc
        .as_ref()
        .map_or("Press any key to start", |l| l.t("menu.main.press_any_key"));

    let play_str = loc.as_ref().map_or("Play", |l| l.t("menu.main.play"));
    let settings_str = loc.as_ref().map_or("Settings", |l| l.t("menu.main.settings"));
    let controls_str = loc.as_ref().map_or("Controls", |l| l.t("menu.main.controls"));
    let quit_str = loc.as_ref().map_or("Quit to Desktop", |l| l.t("menu.main.quit"));

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
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.22)),
                ZIndex(1),
            ));

            // Header Container (Title & Subtitle)
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: percent(12.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(6.0),
                    ..default()
                },
                ZIndex(10),
            ))
            .with_children(|header| {
                // Game Title
                header.spawn((
                    Text::new(title_str),
                    make_font(68.0),
                    TextColor(Color::srgb(1.0, 0.88, 0.42)),
                    TextShadow {
                        offset: Vec2::new(3.0, 3.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.90),
                    },
                ));

                // Version Badge / Subtitle
                header.spawn((
                    Text::new(version_str),
                    make_font(18.0),
                    TextColor(Color::srgba(0.85, 0.89, 0.96, 0.80)),
                    text_shadow_default(),
                ));
            });

            // Splash Screen Prompt Container
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
                    bottom: percent(30.0),
                    display: splash_display,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                splash_visibility,
                ZIndex(10),
            ))
            .with_children(|splash| {
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

            // Action Menu Buttons Container
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
                    top: percent(38.0),
                    display: action_display,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(16.0),
                    ..default()
                },
                action_visibility,
                ZIndex(10),
            ))
            .with_children(|action_container| {
                let buttons = [
                    (MainMenuButtonAction::Play, play_str),
                    (MainMenuButtonAction::Settings, settings_str),
                    (MainMenuButtonAction::Controls, controls_str),
                    (MainMenuButtonAction::Quit, quit_str),
                ];

                for (action, label) in buttons {
                    action_container
                        .spawn((
                            Button,
                            Hovered::default(),
                            action,
                            MenuButtonTexture {
                                normal: btn_normal.clone(),
                                hover: btn_hover.clone(),
                            },
                            ImageNode {
                                image: btn_normal.clone(),
                                image_mode: NodeImageMode::Sliced(TextureSlicer {
                                    border: BorderRect::all(4.0),
                                    ..default()
                                }),
                                ..default()
                            },
                            Node {
                                width: px(384.0),
                                height: px(56.0),
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                MainMenuButtonLabel,
                                Text::new(label),
                                make_font(22.0),
                                TextColor(Color::srgb(0.92, 0.94, 0.98)),
                                text_shadow_default(),
                            ));
                        });
                }
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
    mut world_session: Option<ResMut<crate::world::WorldSession>>,
    mut commands: Commands,
) {
    if *phase != MainMenuPhase::ActionMenu {
        return;
    }

    // Escape in ActionMenu returns back to Splash
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
                    if let Some(ref mut session) = world_session {
                        session.is_active = true;
                    }
                    menu_return_target.0 = MenuState::Pause;
                    next_state.set(MenuState::Loading);
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
        }
    }
}

fn update_main_menu_button_hover(
    mut buttons_query: Query<
        (
            &Hovered,
            Has<Pressed>,
            &MenuButtonTexture,
            &mut ImageNode,
            &Children,
        ),
        With<Button>,
    >,
    mut labels_query: Query<&mut TextColor, With<MainMenuButtonLabel>>,
) {
    for (hovered, is_pressed, btn_tex, mut image_node, children) in &mut buttons_query {
        let is_active = hovered.get() || is_pressed;
        image_node.image = if is_active {
            btn_tex.hover.clone()
        } else {
            btn_tex.normal.clone()
        };

        for child in children.iter() {
            if let Ok(mut text_color) = labels_query.get_mut(child) {
                if is_active {
                    text_color.0 = Color::srgb(1.0, 0.92, 0.45);
                } else {
                    text_color.0 = Color::srgb(0.92, 0.94, 0.98);
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
