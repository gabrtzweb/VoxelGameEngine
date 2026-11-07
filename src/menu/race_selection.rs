use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    ui::Pressed,
    ui_widgets::Button,
};

use crate::{
    audio::UiSound,
    core::{AppFont, Localization, text_shadow_default},
    player::{CharacterRace, SelectedRace},
    world::WorldSession,
};

use super::{GameSettings, MenuBackdrop, MenuButtonTexture, MenuReturnTarget, MenuState};

#[derive(Component)]
pub struct RaceSelectionRoot;

#[derive(Component)]
pub struct RaceSelectionCard;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct RaceCardButton(pub CharacterRace);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum RaceMenuAction {
    Back,
    Confirm,
}

#[derive(Component)]
pub struct RaceCardLabel;

#[derive(Component)]
pub struct RaceActionButtonLabel;

pub struct RaceSelectionMenuPlugin;

impl Plugin for RaceSelectionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::RaceSelection), spawn_race_selection_menu)
            .add_systems(OnExit(MenuState::RaceSelection), despawn_race_selection_menu)
            .add_systems(
                Update,
                (
                    handle_race_selection_buttons,
                    update_race_card_hover,
                    update_race_action_button_hover,
                    update_race_selection_card_scale,
                )
                    .run_if(in_state(MenuState::RaceSelection)),
            );
    }
}

fn spawn_race_selection_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loc: Option<Res<Localization>>,
    app_font: Option<Res<AppFont>>,
    game_settings: Option<Res<GameSettings>>,
    selected_race: Option<Res<SelectedRace>>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source());
    let bg_texture = asset_server.load("textures/interfaces/main_menu_bg.png");
    let btn_normal = asset_server.load("textures/interfaces/containers/buttons/button_medium.png");
    let btn_hover =
        asset_server.load("textures/interfaces/containers/buttons/button_medium_hover.png");

    let current_race = selected_race.map_or(CharacterRace::Human, |r| r.0);
    let scale_factor = game_settings
        .as_ref()
        .map_or(1.0, |s| (s.gui_scale as f32 / 4.0).clamp(0.5, 1.0));

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
        .map_or("ESCOLHA SUA RAÇA", |l| l.t("menu.race_selection.title"));
    let subtitle_str = loc.as_ref().map_or(
        "Selecione a raça do seu personagem para começar a jornada",
        |l| l.t("menu.race_selection.subtitle"),
    );
    let back_str = loc
        .as_ref()
        .map_or("Voltar", |l| l.t("menu.race_selection.back"));
    let confirm_str = loc
        .as_ref()
        .map_or("Confirmar", |l| l.t("menu.race_selection.confirm"));

    commands
        .spawn((
            RaceSelectionRoot,
            MenuBackdrop,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                right: px(0.0),
                bottom: px(0.0),
                width: percent(100.0),
                height: percent(100.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            // Background wallpaper
            backdrop.spawn((
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

            // Dim overlay
            backdrop.spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.50)),
                ZIndex(1),
            ));

            // Main Modal Window Card
            backdrop
                .spawn((
                    RaceSelectionCard,
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::axes(px(32.0 * scale_factor), px(24.0 * scale_factor)),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(8.0)),
                        row_gap: px(16.0 * scale_factor),
                        max_width: percent(92.0),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.35, 0.42, 0.55, 0.75)),
                    BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.92)),
                    ZIndex(10),
                ))
                .with_children(|card| {
                    // Header container
                    card.spawn((
                        Pickable::IGNORE,
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(4.0 * scale_factor),
                            margin: UiRect::bottom(px(8.0 * scale_factor)),
                            ..default()
                        },
                    ))
                    .with_children(|header| {
                        // Title
                        header.spawn((
                            Pickable::IGNORE,
                            Text::new(title_str),
                            make_font(28.0 * scale_factor),
                            TextColor(Color::srgb(0.96, 0.97, 1.0)),
                            text_shadow_default(),
                        ));

                        // Subtitle
                        header.spawn((
                            Pickable::IGNORE,
                            Text::new(subtitle_str),
                            make_font(14.0 * scale_factor),
                            TextColor(Color::srgb(0.72, 0.76, 0.84)),
                            text_shadow_default(),
                        ));
                    });

                    // 4x2 Race Selection Grid Container
                    card.spawn((
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![
                                GridTrack::px(180.0 * scale_factor),
                                GridTrack::px(180.0 * scale_factor),
                                GridTrack::px(180.0 * scale_factor),
                                GridTrack::px(180.0 * scale_factor),
                            ],
                            grid_template_rows: vec![
                                GridTrack::px(120.0 * scale_factor),
                                GridTrack::px(120.0 * scale_factor),
                            ],
                            column_gap: px(16.0 * scale_factor),
                            row_gap: px(16.0 * scale_factor),
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            margin: UiRect::vertical(px(8.0 * scale_factor)),
                            ..default()
                        },
                    ))
                    .with_children(|grid| {
                        for race in CharacterRace::ALL {
                            let icon_handle = asset_server.load(race.icon_asset_path());
                            let race_name = race.display_name(loc.as_deref());
                            let is_current = race == current_race;

                            let base_border = if is_current {
                                Color::srgb(0.95, 0.80, 0.25)
                            } else {
                                Color::srgba(0.28, 0.35, 0.46, 0.80)
                            };

                            let base_bg = if is_current {
                                Color::srgba(0.16, 0.20, 0.28, 0.96)
                            } else {
                                Color::srgba(0.10, 0.12, 0.18, 0.88)
                            };

                            grid.spawn((
                                Button,
                                Hovered::default(),
                                RaceCardButton(race),
                                Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::all(px(10.0 * scale_factor)),
                                    border: UiRect::all(px(2.0)),
                                    border_radius: BorderRadius::all(px(6.0)),
                                    row_gap: px(8.0 * scale_factor),
                                    ..default()
                                },
                                BorderColor::all(base_border),
                                BackgroundColor(base_bg),
                            ))
                            .with_children(|race_card| {
                                // Race 16x16 icon scaled up crisply (48x48)
                                race_card.spawn((
                                    Pickable::IGNORE,
                                    ImageNode {
                                        image: icon_handle,
                                        image_mode: NodeImageMode::Stretch,
                                        ..default()
                                    },
                                    Node {
                                        width: px(48.0 * scale_factor),
                                        height: px(48.0 * scale_factor),
                                        ..default()
                                    },
                                ));

                                // Race Name Label
                                race_card.spawn((
                                    Pickable::IGNORE,
                                    RaceCardLabel,
                                    Text::new(race_name),
                                    make_font(16.0 * scale_factor),
                                    TextColor(if is_current {
                                        Color::srgb(1.0, 0.92, 0.45)
                                    } else {
                                        Color::srgb(0.90, 0.92, 0.96)
                                    }),
                                    text_shadow_default(),
                                ));
                            });
                        }
                    });

                    // Bottom Bar: Back and Confirm buttons
                    card.spawn((
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: px(20.0 * scale_factor),
                            margin: UiRect::top(px(12.0 * scale_factor)),
                            ..default()
                        },
                    ))
                    .with_children(|bottom| {
                        let actions = [
                            (RaceMenuAction::Back, back_str),
                            (RaceMenuAction::Confirm, confirm_str),
                        ];

                        for (action, label) in actions {
                            bottom
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
                                        width: px(220.0 * scale_factor),
                                        height: px(48.0 * scale_factor),
                                        display: Display::Flex,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Pickable::IGNORE,
                                        RaceActionButtonLabel,
                                        Text::new(label),
                                        make_font(18.0 * scale_factor),
                                        TextColor(Color::srgb(0.92, 0.94, 0.98)),
                                        text_shadow_default(),
                                    ));
                                });
                        }
                    });
                });
        });
}

fn despawn_race_selection_menu(
    mut commands: Commands,
    query: Query<Entity, With<RaceSelectionRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_race_selection_buttons(
    race_buttons: Query<(&Hovered, &RaceCardButton), With<Button>>,
    action_buttons: Query<(&Hovered, &RaceMenuAction), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected_race: ResMut<SelectedRace>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut menu_return_target: ResMut<MenuReturnTarget>,
    mut world_session: Option<ResMut<WorldSession>>,
    mut commands: Commands,
) {
    let left_clicked = mouse.just_pressed(MouseButton::Left);
    let escape_pressed = keyboard.just_pressed(KeyCode::Escape);
    let enter_pressed =
        keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter);

    // Escape returns back to Main Menu
    if escape_pressed {
        commands.trigger(UiSound::Click);
        next_state.set(MenuState::MainMenu);
        return;
    }

    // Enter confirms the currently selected race and starts loading
    if enter_pressed {
        commands.trigger(UiSound::Click);
        if let Some(ref mut session) = world_session {
            session.is_active = true;
        }
        menu_return_target.0 = MenuState::Pause;
        next_state.set(MenuState::Loading);
        return;
    }

    if !left_clicked {
        return;
    }

    // Clicking a race card selects that race (sound is played once by global detect_button_click_sound)
    for (hovered, race_button) in &race_buttons {
        if hovered.get() {
            if selected_race.0 != race_button.0 {
                selected_race.0 = race_button.0;
            }
            return;
        }
    }

    // Handle Back & Confirm buttons (sound is played once by global detect_button_click_sound)
    for (hovered, action) in &action_buttons {
        if hovered.get() {
            match action {
                RaceMenuAction::Back => {
                    next_state.set(MenuState::MainMenu);
                }
                RaceMenuAction::Confirm => {
                    if let Some(ref mut session) = world_session {
                        session.is_active = true;
                    }
                    menu_return_target.0 = MenuState::Pause;
                    next_state.set(MenuState::Loading);
                }
            }
            return;
        }
    }
}

fn update_race_card_hover(
    selected_race: Res<SelectedRace>,
    mut race_cards: Query<
        (
            &Hovered,
            Has<Pressed>,
            &RaceCardButton,
            &mut BorderColor,
            &mut BackgroundColor,
            &Children,
        ),
        With<Button>,
    >,
    mut labels: Query<&mut TextColor, With<RaceCardLabel>>,
) {
    for (hovered, is_pressed, race_button, mut border, mut bg, children) in &mut race_cards {
        let is_current = race_button.0 == selected_race.0;
        let is_active = hovered.get() || is_pressed;

        if is_active {
            *border = BorderColor::all(Color::srgb(1.0, 0.88, 0.35));
            *bg = BackgroundColor(Color::srgba(0.20, 0.25, 0.36, 0.98));
        } else if is_current {
            *border = BorderColor::all(Color::srgb(0.95, 0.80, 0.25));
            *bg = BackgroundColor(Color::srgba(0.16, 0.20, 0.28, 0.96));
        } else {
            *border = BorderColor::all(Color::srgba(0.28, 0.35, 0.46, 0.80));
            *bg = BackgroundColor(Color::srgba(0.10, 0.12, 0.18, 0.88));
        }

        for child in children.iter() {
            if let Ok(mut text_color) = labels.get_mut(child) {
                if is_active || is_current {
                    text_color.0 = Color::srgb(1.0, 0.92, 0.45);
                } else {
                    text_color.0 = Color::srgb(0.90, 0.92, 0.96);
                }
            }
        }
    }
}

fn update_race_action_button_hover(
    mut buttons_query: Query<
        (
            &Hovered,
            Has<Pressed>,
            &MenuButtonTexture,
            &mut ImageNode,
            &Children,
        ),
        (With<Button>, With<RaceMenuAction>),
    >,
    mut labels_query: Query<&mut TextColor, With<RaceActionButtonLabel>>,
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

fn update_race_selection_card_scale(
    game_settings: Res<GameSettings>,
    mut card_query: Query<&mut Node, With<RaceSelectionCard>>,
) {
    if !game_settings.is_changed() {
        return;
    }
    let scale_factor = (game_settings.gui_scale as f32 / 4.0).clamp(0.5, 1.0);
    for mut node in &mut card_query {
        node.padding = UiRect::axes(px(32.0 * scale_factor), px(24.0 * scale_factor));
        node.row_gap = px(16.0 * scale_factor);
    }
}
