use bevy::prelude::*;

use crate::{
    core::{text_shadow_default, AppFont, DynamicFpsSettings},
    environment::EnvironmentState,
    meshing::{ChunkMaterial, VoxelTextureRegistry},
    world::{ChunkStreamingQueues, ChunkStreamingSettings, VoxelWorld},
};

use super::{GameSettings, MenuState};

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
enum SettingsAction {
    DecScreenMode,
    IncScreenMode,
    DecRenderDistance,
    IncRenderDistance,
    DecSimulationDistance,
    IncSimulationDistance,
    DecFov,
    IncFov,
    ToggleFog,
    ToggleViewBobbing,
    ToggleFullGrass,
    ToggleVsync,
    ToggleDynamicFps,
    ToggleTimePause,
    Back,
}

#[derive(Component)]
struct ScreenModeLabel;

#[derive(Component)]
struct RenderDistanceLabel;

#[derive(Component)]
struct SimulationDistanceLabel;

#[derive(Component)]
struct FovLabel;

#[derive(Component)]
struct FogLabel;

#[derive(Component)]
struct ViewBobbingLabel;

#[derive(Component)]
struct FullGrassLabel;

#[derive(Component)]
struct VsyncLabel;

#[derive(Component)]
struct DynamicFpsLabel;

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
    app_font: Option<Res<AppFont>>,
) {
    let render_dist = chunk_settings.render_distance;
    let sim_dist = chunk_settings.simulation_distance;
    let fov = game_settings.fov_degrees as i32;
    let fog_enabled = game_settings.fog_enabled;
    let time_paused = env_state.as_ref().is_some_and(|e| e.is_time_paused);
    let font_handle = app_font.as_ref().map(|f| f.source());

    let mut header_font = TextFont {
        font_size: FontSize::Px(22.0),
        ..default()
    };
    if let Some(ref font) = font_handle {
        header_font.font = font.clone();
    }

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
                        padding: UiRect::axes(px(24.0), px(16.0)),
                        row_gap: px(8.0),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.10, 0.10, 0.13, 0.95)),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.42, 0.80)),
                ))
                .with_children(|card| {
                    // Header Title
                    card.spawn((
                        Text::new("SETTINGS"),
                        header_font,
                        TextColor(Color::srgb(0.95, 0.95, 0.98)),
                        text_shadow_default(),
                        Node {
                            margin: UiRect::bottom(px(4.0)),
                            ..default()
                        },
                    ));

                    // 0. Screen Mode Stepper
                    spawn_stepper_row(
                        card,
                        "Screen Mode",
                        game_settings.screen_mode.label().to_string(),
                        SettingsAction::DecScreenMode,
                        SettingsAction::IncScreenMode,
                        ScreenModeLabel,
                        font_handle.as_ref(),
                    );

                    // 1. Render Distance Stepper
                    spawn_stepper_row(
                        card,
                        "Render Distance",
                        format!("{render_dist} Chunks"),
                        SettingsAction::DecRenderDistance,
                        SettingsAction::IncRenderDistance,
                        RenderDistanceLabel,
                        font_handle.as_ref(),
                    );

                    // 1b. Simulation Distance Stepper
                    spawn_stepper_row(
                        card,
                        "Simulation Distance",
                        format!("{sim_dist} Chunks"),
                        SettingsAction::DecSimulationDistance,
                        SettingsAction::IncSimulationDistance,
                        SimulationDistanceLabel,
                        font_handle.as_ref(),
                    );

                    // 2. Field of View Stepper
                    spawn_stepper_row(
                        card,
                        "Field of View",
                        format!("{fov}°"),
                        SettingsAction::DecFov,
                        SettingsAction::IncFov,
                        FovLabel,
                        font_handle.as_ref(),
                    );

                    // 3. Fog Toggle Button
                    spawn_toggle_button(
                        card,
                        SettingsAction::ToggleFog,
                        format!("Fog: {}", if fog_enabled { "Enabled" } else { "Disabled" }),
                        FogLabel,
                        font_handle.as_ref(),
                    );

                    // 4. View Bobbing Toggle Button
                    spawn_toggle_button(
                        card,
                        SettingsAction::ToggleViewBobbing,
                        format!(
                            "View Bobbing: {}",
                            if game_settings.view_bobbing {
                                "Enabled"
                            } else {
                                "Disabled"
                            }
                        ),
                        ViewBobbingLabel,
                        font_handle.as_ref(),
                    );

                    // 4b. Grass Sides Toggle Button
                    spawn_toggle_button(
                        card,
                        SettingsAction::ToggleFullGrass,
                        format!(
                            "Grass Sides: {}",
                            if game_settings.full_grass {
                                "Full Grass"
                            } else {
                                "Side Textures"
                            }
                        ),
                        FullGrassLabel,
                        font_handle.as_ref(),
                    );

                    // 5. VSync Toggle Button
                    spawn_toggle_button(
                        card,
                        SettingsAction::ToggleVsync,
                        format!(
                            "VSync: {}",
                            if game_settings.vsync_enabled {
                                "Enabled"
                            } else {
                                "Disabled"
                            }
                        ),
                        VsyncLabel,
                        font_handle.as_ref(),
                    );

                    // 6. Dynamic FPS Toggle Button
                    spawn_toggle_button(
                        card,
                        SettingsAction::ToggleDynamicFps,
                        format!(
                            "Dynamic FPS: {}",
                            if game_settings.dynamic_fps_enabled {
                                "Enabled"
                            } else {
                                "Disabled"
                            }
                        ),
                        DynamicFpsLabel,
                        font_handle.as_ref(),
                    );

                    // 7. Time Pause Toggle Button
                    spawn_toggle_button(
                        card,
                        SettingsAction::ToggleTimePause,
                        format!(
                            "Time Flow: {}",
                            if time_paused { "Paused" } else { "Running" }
                        ),
                        TimePauseLabel,
                        font_handle.as_ref(),
                    );

                    // 8. Back Button
                    let mut back_font = TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    };
                    if let Some(ref font) = font_handle {
                        back_font.font = font.clone();
                    }

                    card.spawn((
                        Button,
                        SettingsAction::Back,
                        Node {
                            width: px(280.0),
                            height: px(34.0),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(px(6.0)),
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
                            back_font,
                            TextColor(Color::srgb(0.90, 0.90, 0.92)),
                            text_shadow_default(),
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
    font_handle: Option<&crate::core::FontSource>,
) {
    let mut title_font = TextFont {
        font_size: FontSize::Px(13.0),
        ..default()
    };
    let mut btn_font = TextFont {
        font_size: FontSize::Px(14.0),
        ..default()
    };
    let mut val_font = TextFont {
        font_size: FontSize::Px(13.0),
        ..default()
    };
    if let Some(font) = font_handle {
        title_font.font = font.clone();
        btn_font.font = font.clone();
        val_font.font = font.clone();
    }

    parent
        .spawn(Node {
            width: px(280.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(8.0), px(3.0)),
            ..default()
        })
        .with_children(|row| {
            // Label on left
            row.spawn((
                Text::new(title),
                title_font,
                TextColor(Color::srgb(0.85, 0.85, 0.88)),
                text_shadow_default(),
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
                            btn_font.clone(),
                            TextColor(Color::WHITE),
                            text_shadow_default(),
                        ));
                    });

                // Value Text
                controls.spawn((
                    marker,
                    Text::new(initial_val),
                    val_font,
                    TextColor(Color::srgb(1.0, 0.88, 0.45)),
                    text_shadow_default(),
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
                            btn_font,
                            TextColor(Color::WHITE),
                            text_shadow_default(),
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
    font_handle: Option<&crate::core::FontSource>,
) {
    let mut btn_font = TextFont {
        font_size: FontSize::Px(13.0),
        ..default()
    };
    if let Some(font) = font_handle {
        btn_font.font = font.clone();
    }

    parent
        .spawn((
            Button,
            action,
            Node {
                width: px(280.0),
                height: px(32.0),
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
                btn_font,
                TextColor(Color::srgb(0.90, 0.90, 0.92)),
                text_shadow_default(),
            ));
        });
}

fn despawn_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
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
    mut dynamic_fps_settings: Option<ResMut<DynamicFpsSettings>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut chunk_material: Option<ResMut<ChunkMaterial>>,
    mut texture_registry: Option<ResMut<VoxelTextureRegistry>>,
    mut streaming_queues: Option<ResMut<ChunkStreamingQueues>>,
    world: Option<Res<VoxelWorld>>,
    mut window_query: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    for (interaction, action, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.35, 0.35, 0.44, 1.0));
                *border_color = BorderColor::all(Color::srgb(1.0, 0.90, 0.40));

                match action {
                    SettingsAction::DecScreenMode => {
                        game_settings.screen_mode = game_settings.screen_mode.prev();
                        if let Ok(mut window) = window_query.single_mut() {
                            window.mode = game_settings.screen_mode.to_window_mode();
                        }
                    }
                    SettingsAction::IncScreenMode => {
                        game_settings.screen_mode = game_settings.screen_mode.next();
                        if let Ok(mut window) = window_query.single_mut() {
                            window.mode = game_settings.screen_mode.to_window_mode();
                        }
                    }
                    SettingsAction::DecRenderDistance => {
                        chunk_settings.render_distance =
                            (chunk_settings.render_distance - 1).max(2);
                    }
                    SettingsAction::IncRenderDistance => {
                        chunk_settings.render_distance =
                            (chunk_settings.render_distance + 1).min(16);
                    }
                    SettingsAction::DecSimulationDistance => {
                        chunk_settings.simulation_distance =
                            (chunk_settings.simulation_distance - 1).max(2);
                    }
                    SettingsAction::IncSimulationDistance => {
                        chunk_settings.simulation_distance =
                            (chunk_settings.simulation_distance + 1).min(8);
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
                    SettingsAction::ToggleViewBobbing => {
                        game_settings.view_bobbing = !game_settings.view_bobbing;
                    }
                    SettingsAction::ToggleFullGrass => {
                        game_settings.full_grass = !game_settings.full_grass;
                        if let Some(ref mut reg) = texture_registry {
                            reg.full_grass = game_settings.full_grass;
                        }
                        if let Some(ref mut mat) = chunk_material {
                            mat.texture_registry.full_grass = game_settings.full_grass;
                        }
                        if let Some(ref world) = world
                            && let Some(ref mut queues) = streaming_queues
                        {
                            for (&coord, _) in world.iter_chunks() {
                                queues.enqueue_remesh(coord);
                            }
                        }
                    }
                    SettingsAction::ToggleVsync => {
                        game_settings.vsync_enabled = !game_settings.vsync_enabled;
                        if let Ok(mut window) = window_query.single_mut() {
                            window.present_mode = if game_settings.vsync_enabled {
                                bevy::window::PresentMode::AutoVsync
                            } else {
                                bevy::window::PresentMode::AutoNoVsync
                            };
                        }
                    }
                    SettingsAction::ToggleDynamicFps => {
                        game_settings.dynamic_fps_enabled = !game_settings.dynamic_fps_enabled;
                        if let Some(ref mut dyn_fps) = dynamic_fps_settings {
                            dyn_fps.enabled = game_settings.dynamic_fps_enabled;
                        }
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
    mut labels_query: Query<(
        &mut Text,
        Option<&ScreenModeLabel>,
        Option<&RenderDistanceLabel>,
        Option<&SimulationDistanceLabel>,
        Option<&FovLabel>,
        Option<&FogLabel>,
        Option<&ViewBobbingLabel>,
        Option<&FullGrassLabel>,
        Option<&VsyncLabel>,
        Option<&DynamicFpsLabel>,
        Option<&TimePauseLabel>,
    )>,
) {
    let chunk_changed = chunk_settings.is_changed();
    let game_changed = game_settings.is_changed();
    let env_changed = env_state.as_ref().is_some_and(|e| e.is_changed());

    if !chunk_changed && !game_changed && !env_changed {
        return;
    }

    for (
        mut text,
        screen_mode,
        render_dist,
        sim_dist,
        fov,
        fog,
        bobbing,
        grass,
        vsync,
        dyn_fps,
        time_pause,
    ) in &mut labels_query
    {
        if game_changed {
            if screen_mode.is_some() {
                text.0 = game_settings.screen_mode.label().to_string();
            } else if fov.is_some() {
                text.0 = format!("{}°", game_settings.fov_degrees as i32);
            } else if fog.is_some() {
                text.0 = format!(
                    "Fog: {}",
                    if game_settings.fog_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            } else if bobbing.is_some() {
                text.0 = format!(
                    "View Bobbing: {}",
                    if game_settings.view_bobbing {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            } else if grass.is_some() {
                text.0 = format!(
                    "Grass Sides: {}",
                    if game_settings.full_grass {
                        "Full Grass"
                    } else {
                        "Side Textures"
                    }
                );
            } else if vsync.is_some() {
                text.0 = format!(
                    "VSync: {}",
                    if game_settings.vsync_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            } else if dyn_fps.is_some() {
                text.0 = format!(
                    "Dynamic FPS: {}",
                    if game_settings.dynamic_fps_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            }
        }
        if chunk_changed {
            if render_dist.is_some() {
                text.0 = format!("{} Chunks", chunk_settings.render_distance);
            } else if sim_dist.is_some() {
                text.0 = format!("{} Chunks", chunk_settings.simulation_distance);
            }
        }
        if env_changed && time_pause.is_some() {
            let paused = env_state.as_ref().is_some_and(|e| e.is_time_paused);
            text.0 = format!(
                "Time Flow: {}",
                if paused { "Paused" } else { "Running" }
            );
        }
    }
}
