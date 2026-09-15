use bevy::prelude::*;

use crate::{environment::EnvironmentState, voxel::chunk_manager::ChunkStreamingSettings};

use super::{GameSettings, MenuState};

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
enum SettingsAction {
    DecRenderDistance,
    IncRenderDistance,
    DecFov,
    IncFov,
    ToggleFog,
    ToggleTimePause,
    Back,
}

#[derive(Component)]
struct RenderDistanceLabel;

#[derive(Component)]
struct FovLabel;

#[derive(Component)]
struct FogLabel;

#[derive(Component)]
struct TimePauseLabel;

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Settings), spawn_settings_menu)
            .add_systems(OnExit(MenuState::Settings), despawn_settings_menu)
            .add_systems(
                Update,
                (handle_settings_buttons, update_settings_labels)
                    .run_if(in_state(MenuState::Settings)),
            );
    }
}

fn spawn_settings_menu(
    mut commands: Commands,
    chunk_settings: Res<ChunkStreamingSettings>,
    game_settings: Res<GameSettings>,
    env_state: Option<Res<EnvironmentState>>,
) {
    let render_dist = chunk_settings.render_distance;
    let fov = game_settings.fov_degrees as i32;
    let fog_enabled = game_settings.fog_enabled;
    let time_paused = env_state.as_ref().is_some_and(|e| e.is_time_paused);

    commands
        .spawn((
            SettingsMenuRoot,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(12.0),
                        width: px(340.0),
                        padding: UiRect::axes(px(24.0), px(24.0)),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.07, 0.10, 0.95)),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.42, 0.80)),
                ))
                .with_children(|card| {
                    // Header Title
                    card.spawn((
                        Text::new("SETTINGS"),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.95, 0.98)),
                        Node {
                            margin: UiRect::bottom(px(6.0)),
                            ..default()
                        },
                    ));

                    // 1. Render Distance Stepper
                    spawn_stepper_row(
                        card,
                        "Render Distance",
                        format!("{render_dist} Chunks"),
                        SettingsAction::DecRenderDistance,
                        SettingsAction::IncRenderDistance,
                        RenderDistanceLabel,
                    );

                    // 2. Field of View Stepper
                    spawn_stepper_row(
                        card,
                        "Field of View",
                        format!("{fov}°"),
                        SettingsAction::DecFov,
                        SettingsAction::IncFov,
                        FovLabel,
                    );

                    // 3. Fog Toggle Button
                    spawn_toggle_button(
                        card,
                        SettingsAction::ToggleFog,
                        format!("Fog: {}", if fog_enabled { "Enabled" } else { "Disabled" }),
                        FogLabel,
                    );

                    // 4. Time Pause Toggle Button
                    spawn_toggle_button(
                        card,
                        SettingsAction::ToggleTimePause,
                        format!(
                            "Time Flow: {}",
                            if time_paused { "Paused" } else { "Running" }
                        ),
                        TimePauseLabel,
                    );

                    // 5. Back Button
                    card.spawn((
                        Button,
                        SettingsAction::Back,
                        Node {
                            width: px(280.0),
                            height: px(36.0),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(px(8.0)),
                            border: UiRect::all(px(1.5)),
                            border_radius: BorderRadius::all(px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.16, 0.16, 0.20, 0.90)),
                        BorderColor::all(Color::srgba(0.32, 0.32, 0.38, 0.70)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Back"),
                            TextFont {
                                font_size: FontSize::Px(14.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.90, 0.90, 0.92)),
                        ));
                    });
                });
        });
}

fn spawn_stepper_row<T: Component>(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    initial_val: String,
    dec_action: SettingsAction,
    inc_action: SettingsAction,
    marker: T,
) {
    parent
        .spawn(Node {
            width: px(280.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(8.0), px(4.0)),
            ..default()
        })
        .with_children(|row| {
            // Label on left
            row.spawn((
                Text::new(title),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.88)),
            ));

            // Controls on right: [-] Value [+]
            row.spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(6.0),
                ..default()
            })
            .with_children(|controls| {
                // Decrement button
                controls
                    .spawn((
                        Button,
                        dec_action,
                        Node {
                            width: px(28.0),
                            height: px(28.0),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(px(1.0)),
                            border_radius: BorderRadius::all(px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.18, 0.18, 0.22, 0.90)),
                        BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 0.70)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("-"),
                            TextFont {
                                font_size: FontSize::Px(14.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                // Value Text
                controls.spawn((
                    marker,
                    Text::new(initial_val),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.88, 0.45)),
                    Node {
                        width: px(90.0),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ));

                // Increment button
                controls
                    .spawn((
                        Button,
                        inc_action,
                        Node {
                            width: px(28.0),
                            height: px(28.0),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(px(1.0)),
                            border_radius: BorderRadius::all(px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.18, 0.18, 0.22, 0.90)),
                        BorderColor::all(Color::srgba(0.35, 0.35, 0.40, 0.70)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("+"),
                            TextFont {
                                font_size: FontSize::Px(14.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
        });
}

fn spawn_toggle_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    action: SettingsAction,
    label: String,
    marker: T,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: px(280.0),
                height: px(34.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1.5)),
                border_radius: BorderRadius::all(px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.16, 0.16, 0.20, 0.90)),
            BorderColor::all(Color::srgba(0.32, 0.32, 0.38, 0.70)),
        ))
        .with_children(|btn| {
            btn.spawn((
                marker,
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.90, 0.90, 0.92)),
            ));
        });
}

fn despawn_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity)]
fn handle_settings_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &SettingsAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut chunk_settings: ResMut<ChunkStreamingSettings>,
    mut game_settings: ResMut<GameSettings>,
    mut env_state: Option<ResMut<EnvironmentState>>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, action, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.35, 0.35, 0.44, 1.0));
                *border_color = BorderColor::all(Color::srgb(1.0, 0.90, 0.40));

                match action {
                    SettingsAction::DecRenderDistance => {
                        chunk_settings.render_distance =
                            (chunk_settings.render_distance - 1).max(2);
                    }
                    SettingsAction::IncRenderDistance => {
                        chunk_settings.render_distance =
                            (chunk_settings.render_distance + 1).min(16);
                    }
                    SettingsAction::DecFov => {
                        game_settings.fov_degrees = (game_settings.fov_degrees - 5.0).max(60.0);
                    }
                    SettingsAction::IncFov => {
                        game_settings.fov_degrees = (game_settings.fov_degrees + 5.0).min(110.0);
                    }
                    SettingsAction::ToggleFog => {
                        game_settings.fog_enabled = !game_settings.fog_enabled;
                    }
                    SettingsAction::ToggleTimePause => {
                        if let Some(ref mut env) = env_state {
                            env.is_time_paused = !env.is_time_paused;
                        }
                    }
                    SettingsAction::Back => {
                        next_state.set(MenuState::Pause);
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.24, 0.24, 0.30, 0.95));
                *border_color = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgba(0.16, 0.16, 0.20, 0.90));
                *border_color = BorderColor::all(Color::srgba(0.32, 0.32, 0.38, 0.70));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_settings_labels(
    chunk_settings: Res<ChunkStreamingSettings>,
    game_settings: Res<GameSettings>,
    env_state: Option<Res<EnvironmentState>>,
    mut render_dist_query: Query<
        &mut Text,
        (
            With<RenderDistanceLabel>,
            Without<FovLabel>,
            Without<FogLabel>,
            Without<TimePauseLabel>,
        ),
    >,
    mut fov_query: Query<
        &mut Text,
        (
            With<FovLabel>,
            Without<RenderDistanceLabel>,
            Without<FogLabel>,
            Without<TimePauseLabel>,
        ),
    >,
    mut fog_query: Query<
        &mut Text,
        (
            With<FogLabel>,
            Without<RenderDistanceLabel>,
            Without<FovLabel>,
            Without<TimePauseLabel>,
        ),
    >,
    mut time_pause_query: Query<
        &mut Text,
        (
            With<TimePauseLabel>,
            Without<RenderDistanceLabel>,
            Without<FovLabel>,
            Without<FogLabel>,
        ),
    >,
) {
    if chunk_settings.is_changed() {
        for mut text in &mut render_dist_query {
            text.0 = format!("{} Chunks", chunk_settings.render_distance);
        }
    }

    if game_settings.is_changed() {
        for mut text in &mut fov_query {
            text.0 = format!("{}°", game_settings.fov_degrees as i32);
        }
        for mut text in &mut fog_query {
            text.0 = format!(
                "Fog: {}",
                if game_settings.fog_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );
        }
    }

    if let Some(ref env) = env_state
        && env.is_changed()
    {
        for mut text in &mut time_pause_query {
            text.0 = format!(
                "Time Flow: {}",
                if env.is_time_paused {
                    "Paused"
                } else {
                    "Running"
                }
            );
        }
    }
}
