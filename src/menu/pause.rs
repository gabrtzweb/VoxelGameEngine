use bevy::prelude::*;

use crate::{
    environment::{DayPhase, EnvironmentState},
    gameplay::SelectedVoxel,
    generation::TerrainGenerator,
    meshing::{ChunkMaterial, ChunkMeshRegistry, sync_chunk_render},
    player::{PLAYER_EYE_HEIGHT, Player, PlayerCamera, PlayerMotion, hotbar::Hotbar},
    simulation::{VoxelLightRegistry, sync_chunk_lights},
    world::{Voxel, VoxelWorld, WorldModificationStore},
};

use super::MenuState;

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
enum PauseMenuAction {
    Resume,
    Settings,
    Restart,
    Quit,
}

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Pause), spawn_pause_menu)
            .add_systems(OnExit(MenuState::Pause), despawn_pause_menu)
            .add_systems(
                Update,
                handle_pause_menu_buttons.run_if(in_state(MenuState::Pause)),
            );
    }
}

fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            PauseMenuRoot,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.60)),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(14.0),
                        width: px(280.0),
                        padding: UiRect::axes(px(24.0), px(28.0)),
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
                        Text::new("GAME PAUSED"),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.95, 0.98)),
                        Node {
                            margin: UiRect::bottom(px(8.0)),
                            ..default()
                        },
                    ));

                    // Buttons
                    spawn_menu_button(card, "Resume Game", PauseMenuAction::Resume);
                    spawn_menu_button(card, "Settings", PauseMenuAction::Settings);
                    spawn_menu_button(card, "Restart Game", PauseMenuAction::Restart);
                    spawn_menu_button(card, "Quit to Desktop", PauseMenuAction::Quit);
                });
        });
}

fn spawn_menu_button(parent: &mut ChildSpawnerCommands, label: &str, action: PauseMenuAction) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: px(220.0),
                height: px(40.0),
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
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.90, 0.90, 0.92)),
            ));
        });
}

fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_pause_menu_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &PauseMenuAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<MenuState>>,
    mut env_state: Option<ResMut<EnvironmentState>>,
    mut player_query: Query<(&mut Transform, &mut PlayerMotion), With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), (With<Camera3d>, Without<Player>)>,
    mut world_query: Option<ResMut<VoxelWorld>>,
    mut modifications_query: Option<ResMut<WorldModificationStore>>,
    terrain_generator_query: Option<Res<TerrainGenerator>>,
    mut light_registry_query: Option<ResMut<VoxelLightRegistry>>,
    mut mesh_registry_query: Option<ResMut<ChunkMeshRegistry>>,
    mut meshes_query: Option<ResMut<Assets<Mesh>>>,
    chunk_material_query: Option<Res<ChunkMaterial>>,
    mut hotbar_query: Option<ResMut<Hotbar>>,
    mut selected_voxel_query: Option<ResMut<SelectedVoxel>>,
    mut map_cache_query: Option<ResMut<crate::map::MapCache>>,
    mut commands: Commands,
) {
    for (interaction, action, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgba(0.35, 0.35, 0.44, 1.0));
                *border_color = BorderColor::all(Color::srgb(1.0, 0.90, 0.40));

                match action {
                    PauseMenuAction::Resume => {
                        next_state.set(MenuState::None);
                    }
                    PauseMenuAction::Settings => {
                        next_state.set(MenuState::Settings);
                    }
                    PauseMenuAction::Restart => {
                        // 1. Reset Environment to Day 1 Noon
                        if let Some(ref mut env) = env_state {
                            env.day_count = 1;
                            env.time_of_day = 0.20;
                            env.phase = DayPhase::Noon;
                            env.is_time_paused = false;
                            env.f6_hold_duration = 0.0;
                        }

                        // 2. Teleport player and camera to spawn
                        let spawn_pos = Vec3::new(-10.0, 10.38, 14.0);
                        for (mut transform, mut motion) in &mut player_query {
                            transform.translation = spawn_pos;
                            motion.velocity = Vec3::ZERO;
                            motion.flying = false;
                        }

                        for (mut transform, mut camera) in &mut camera_query {
                            let cam_pos = spawn_pos + Vec3::Y * PLAYER_EYE_HEIGHT;
                            *transform = Transform::from_translation(cam_pos)
                                .looking_at(Vec3::new(4.0, 3.0, 4.0), Vec3::Y);
                            let new_pc = PlayerCamera::from_transform(&transform);
                            *camera = new_pc;
                        }

                        // 3. Reverse world modifications
                        if let (
                            Some(ref mut world),
                            Some(ref mut modifications),
                            Some(terrain_gen),
                            Some(ref mut light_reg),
                            Some(ref mut mesh_reg),
                            Some(ref mut meshes),
                            Some(chunk_mat),
                        ) = (
                            world_query.as_deref_mut(),
                            modifications_query.as_deref_mut(),
                            terrain_generator_query.as_deref(),
                            light_registry_query.as_deref_mut(),
                            mesh_registry_query.as_deref_mut(),
                            meshes_query.as_deref_mut(),
                            chunk_material_query.as_deref(),
                        ) {
                            let modified_coords = modifications.modified_chunks();
                            modifications.clear();

                            for coord in modified_coords {
                                let fresh_chunk = terrain_gen.generate_chunk(coord);
                                world.insert_chunk(coord, fresh_chunk);
                                sync_chunk_lights(&mut commands, world, coord, light_reg);
                                sync_chunk_render(
                                    &mut commands,
                                    world,
                                    coord,
                                    mesh_reg,
                                    meshes,
                                    chunk_mat,
                                );
                            }
                        }

                        // 4. Reset hotbar and selected voxel
                        if let Some(ref mut hotbar) = hotbar_query {
                            **hotbar = Hotbar::default();
                        }
                        if let Some(ref mut selected) = selected_voxel_query {
                            selected.0 = Some(Voxel::Grass);
                        }
                        if let Some(ref mut cache) = map_cache_query {
                            cache.clear();
                        }

                        info!(
                            "Game restarted: player teleported to spawn, world modifications reversed, day reset to 1"
                        );
                        next_state.set(MenuState::None);
                    }
                    PauseMenuAction::Quit => {
                        commands.write_message(bevy::app::AppExit::Success);
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
