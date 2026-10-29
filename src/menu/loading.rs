use bevy::prelude::*;

use crate::{
    core::{AppFont, Localization, text_shadow_default},
    meshing::ChunkMeshingTask,
    world::{
        storage::VoxelWorld,
        streaming::{
            manager::ChunkGenerationTask,
            ChunkStreamingQueues, ChunkStreamingState,
        },
    },
};

use super::MenuState;

#[derive(Resource, Debug, Clone)]
pub struct LoadingState {
    pub display_progress: f32,
    pub target_progress: f32,
    pub total_chunks: usize,
    pub loaded_chunks: usize,
    pub elapsed_time: f32,
    pub min_duration: f32,
    pub max_duration: f32,
    pub is_complete: bool,
}

impl Default for LoadingState {
    fn default() -> Self {
        Self {
            display_progress: 0.0,
            target_progress: 0.0,
            total_chunks: 0,
            loaded_chunks: 0,
            elapsed_time: 0.0,
            min_duration: 0.5,
            max_duration: 8.0,
            is_complete: false,
        }
    }
}

#[derive(Component)]
pub struct LoadingScreenRoot;

#[derive(Component)]
pub struct LoadingBarFill;

#[derive(Component)]
pub struct LoadingStatusText;

#[derive(Component)]
pub struct LoadingPercentText;

#[derive(Component)]
pub struct LoadingCounterText;

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingState>()
            .add_systems(OnEnter(MenuState::Loading), spawn_loading_screen)
            .add_systems(OnExit(MenuState::Loading), despawn_loading_screen)
            .add_systems(
                Update,
                (
                    update_loading_progress,
                    animate_loading_ui,
                    check_loading_transition,
                )
                    .chain()
                    .run_if(in_state(MenuState::Loading)),
            );
    }
}

fn spawn_loading_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_font: Option<Res<AppFont>>,
    loc: Option<Res<Localization>>,
    mut loading_state: ResMut<LoadingState>,
) {
    *loading_state = LoadingState::default();

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
    let version_str = loc.as_ref().map_or("Alpha 0.1.0", |l| l.t("menu.main.version"));
    let loading_header_str = loc
        .as_ref()
        .map_or("LOADING WORLD...", |l| l.t("menu.loading.title"));
    let initial_status_str = loc
        .as_ref()
        .map_or("Generating terrain...", |l| l.t("menu.loading.generating"));
    let tip_str = loc.as_ref().map_or(
        "Tip: Press Tab to open the system menu bar and I for inventory.",
        |l| l.t("menu.loading.tip"),
    );

    commands
        .spawn((
            LoadingScreenRoot,
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
            ZIndex(350),
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
                BackgroundColor(Color::srgba(0.01, 0.02, 0.03, 0.58)),
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
                header.spawn((
                    Text::new(title_str),
                    make_font(60.0),
                    TextColor(Color::srgb(1.0, 0.88, 0.42)),
                    TextShadow {
                        offset: Vec2::new(3.0, 3.0),
                        color: Color::srgba(0.0, 0.0, 0.0, 0.90),
                    },
                ));

                header.spawn((
                    Text::new(version_str),
                    make_font(17.0),
                    TextColor(Color::srgba(0.85, 0.89, 0.96, 0.80)),
                    text_shadow_default(),
                ));
            });

            // Center Loading Box Card
            root.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(14.0),
                    padding: UiRect::axes(px(40.0), px(28.0)),
                    border: UiRect::all(px(1.5)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    width: px(520.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.07, 0.11, 0.92)),
                BorderColor::all(Color::srgba(0.40, 0.46, 0.58, 0.45)),
                ZIndex(10),
            ))
            .with_children(|card| {
                // Loading Section Title
                card.spawn((
                    Text::new(loading_header_str),
                    make_font(20.0),
                    TextColor(Color::srgb(0.96, 0.97, 1.0)),
                    text_shadow_default(),
                ));

                // Status row: Status message on left, Percentage on right
                card.spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        width: percent(100.0),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    row.spawn((
                        LoadingStatusText,
                        Text::new(initial_status_str),
                        make_font(15.0),
                        TextColor(Color::srgb(0.85, 0.90, 0.98)),
                        text_shadow_default(),
                    ));

                    row.spawn((
                        LoadingPercentText,
                        Text::new("0%"),
                        make_font(16.0),
                        TextColor(Color::srgb(1.0, 0.88, 0.42)),
                        text_shadow_default(),
                    ));
                });

                // Progress Bar Track
                card.spawn((
                    Node {
                        width: percent(100.0),
                        height: px(18.0),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        overflow: Overflow::clip(),
                        display: Display::Flex,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.95)),
                    BorderColor::all(Color::srgba(0.38, 0.43, 0.54, 0.65)),
                ))
                .with_children(|track| {
                    track.spawn((
                        LoadingBarFill,
                        Node {
                            width: percent(0.0),
                            height: percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.98, 0.76, 0.22)),
                    ));
                });

                // Chunk Counter
                card.spawn((
                    LoadingCounterText,
                    Text::new("0 / 0 chunks"),
                    make_font(13.0),
                    TextColor(Color::srgba(0.72, 0.78, 0.88, 0.75)),
                    text_shadow_default(),
                ));
            });

            // Bottom Tip Container
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: percent(12.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(px(24.0), px(10.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                ZIndex(10),
            ))
            .with_children(|tip_box| {
                tip_box.spawn((
                    Text::new(tip_str),
                    make_font(14.0),
                    TextColor(Color::srgba(0.85, 0.89, 0.95, 0.85)),
                    text_shadow_default(),
                ));
            });
        });
}

fn despawn_loading_screen(mut commands: Commands, query: Query<Entity, With<LoadingScreenRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn update_loading_progress(
    time: Res<Time<Real>>,
    mut loading_state: ResMut<LoadingState>,
    streaming_state: Res<ChunkStreamingState>,
    world: Res<VoxelWorld>,
    queues: Res<ChunkStreamingQueues>,
    meshing_tasks: Query<&ChunkMeshingTask>,
    generation_tasks: Query<&ChunkGenerationTask>,
) {
    let dt = time.delta_secs();
    loading_state.elapsed_time += dt;

    let total_desired = streaming_state.desired_chunks.len();
    if total_desired == 0 {
        loading_state.target_progress = 0.05;
        let smooth = 1.0 - (-4.0 * dt).exp();
        loading_state.display_progress +=
            (loading_state.target_progress - loading_state.display_progress) * smooth;
        return;
    }

    loading_state.total_chunks = total_desired;

    let generated_count = streaming_state
        .desired_chunks
        .iter()
        .filter(|coord| world.contains_chunk(**coord))
        .count();

    let pending_remesh = queues.remesh.len() + meshing_tasks.iter().count();
    let pending_generation = queues.load.len() + generation_tasks.iter().count();

    let meshed_count = (generated_count as isize - pending_remesh as isize).max(0) as usize;
    loading_state.loaded_chunks = meshed_count.min(total_desired);

    let gen_ratio = (generated_count as f32 / total_desired as f32).clamp(0.0, 1.0);
    let mesh_ratio = (meshed_count as f32 / total_desired as f32).clamp(0.0, 1.0);
    let raw_progress = (gen_ratio * 0.5 + mesh_ratio * 0.5).clamp(0.0, 1.0);

    let fully_loaded =
        generated_count >= total_desired && pending_remesh == 0 && pending_generation == 0;
    let timed_out = loading_state.elapsed_time >= loading_state.max_duration;

    if fully_loaded || timed_out {
        loading_state.target_progress = 1.0;
        loading_state.is_complete = true;
    } else {
        loading_state.target_progress = raw_progress.clamp(0.05, 0.99);
    }

    let speed = if loading_state.is_complete { 8.0 } else { 4.0 };
    let smooth = 1.0 - (-speed * dt).exp();
    loading_state.display_progress +=
        (loading_state.target_progress - loading_state.display_progress) * smooth;
}

fn animate_loading_ui(
    time: Res<Time<Real>>,
    loading_state: Res<LoadingState>,
    loc: Option<Res<Localization>>,
    mut fill_query: Query<&mut Node, With<LoadingBarFill>>,
    mut status_query: Query<(&mut Text, &mut TextColor), With<LoadingStatusText>>,
    mut percent_query: Query<
        &mut Text,
        (
            With<LoadingPercentText>,
            Without<LoadingStatusText>,
            Without<LoadingCounterText>,
        ),
    >,
    mut counter_query: Query<
        &mut Text,
        (
            With<LoadingCounterText>,
            Without<LoadingStatusText>,
            Without<LoadingPercentText>,
        ),
    >,
) {
    let p = loading_state.display_progress.clamp(0.0, 1.0);

    for mut node in &mut fill_query {
        node.width = percent(p * 100.0);
    }

    let percent_int = (p * 100.0).floor() as u32;
    for mut text in &mut percent_query {
        **text = format!("{percent_int}%");
    }

    for mut text in &mut counter_query {
        **text = format!(
            "{} / {} chunks",
            loading_state.loaded_chunks, loading_state.total_chunks
        );
    }

    let status_str = if loading_state.is_complete || p >= 0.98 {
        loc.as_ref()
            .map_or("Mundo pronto!", |l| l.t("menu.loading.ready"))
    } else if p < 0.45 {
        loc.as_ref()
            .map_or("Gerando terreno...", |l| l.t("menu.loading.generating"))
    } else if p < 0.90 {
        loc.as_ref().map_or(
            "Construindo blocos e malhas...",
            |l| l.t("menu.loading.meshing"),
        )
    } else {
        loc.as_ref()
            .map_or("Finalizando mundo...", |l| l.t("menu.loading.finalizing"))
    };

    let pulse = 0.75 + 0.25 * (time.elapsed_secs() * 4.0).sin().abs();
    for (mut text, mut color) in &mut status_query {
        **text = status_str.to_string();
        color.0 = Color::srgba(0.85, 0.90, 0.98, pulse);
    }
}

fn check_loading_transition(
    mut next_state: ResMut<NextState<MenuState>>,
    loading_state: Res<LoadingState>,
) {
    if loading_state.is_complete
        && loading_state.elapsed_time >= loading_state.min_duration
        && loading_state.display_progress >= 0.98
    {
        next_state.set(MenuState::None);
    }
}
