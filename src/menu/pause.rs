use bevy::prelude::*;

use crate::{
    core::{AppFont, text_shadow_default},
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

fn spawn_pause_menu(
    mut commands: Commands,
    app_font: Option<Res<AppFont>>,
    asset_server: Res<AssetServer>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source());
    let btn_normal = asset_server.load("textures/interfaces/containers/button.png");
    let btn_hover = asset_server.load("textures/interfaces/containers/button_hover.png");

    let mut title_font = TextFont {
        font_size: FontSize::Px(32.0),
        ..default()
    };
    if let Some(ref font) = font_handle {
        title_font.font = font.clone();
    }

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
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(20.0),
                    ..default()
                })
                .with_children(|card| {
                    // Header Title
                    card.spawn((
                        Text::new("GAME PAUSED"),
                        title_font,
                        TextColor(Color::srgb(0.95, 0.95, 0.98)),
                        text_shadow_default(),
                        Node {
                            margin: UiRect::bottom(px(8.0)),
                            ..default()
                        },
                    ));

                    // 2-Column Buttons Container
                    card.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(12.0),
                        ..default()
                    })
                    .with_children(|rows| {
                        // Row 1: Resume Game | Settings
                        rows.spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            column_gap: px(16.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_menu_button(
                                row,
                                "Resume Game",
                                PauseMenuAction::Resume,
                                font_handle.as_ref(),
                                &btn_normal,
                                &btn_hover,
                            );
                            spawn_menu_button(
                                row,
                                "Settings",
                                PauseMenuAction::Settings,
                                font_handle.as_ref(),
                                &btn_normal,
                                &btn_hover,
                            );
                        });

                        // Row 2: Restart Game | Quit to Desktop
                        rows.spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            column_gap: px(16.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_menu_button(
                                row,
                                "Restart Game",
                                PauseMenuAction::Restart,
                                font_handle.as_ref(),
                                &btn_normal,
                                &btn_hover,
                            );
                            spawn_menu_button(
                                row,
                                "Quit to Desktop",
                                PauseMenuAction::Quit,
                                font_handle.as_ref(),
                                &btn_normal,
                                &btn_hover,
                            );
                        });
                    });
                });
        });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: PauseMenuAction,
    font_handle: Option<&crate::core::FontSource>,
    btn_normal: &Handle<Image>,
    btn_hover: &Handle<Image>,
) {
    let mut btn_font = TextFont {
        font_size: FontSize::Px(16.0),
        ..default()
    };
    if let Some(font) = font_handle {
        btn_font.font = font.clone();
    }

    parent
        .spawn((
            Button,
            action,
            crate::menu::MenuButtonTexture {
                normal: btn_normal.clone(),
                hover: btn_hover.clone(),
            },
            ImageNode {
                image: btn_normal.clone(),
                ..default()
            },
            Node {
                width: px(220.0),
                height: px(40.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                btn_font,
                TextColor(Color::srgb(0.95, 0.95, 0.98)),
                text_shadow_default(),
            ));
        });
}

fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_pause_menu_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &PauseMenuAction,
            &crate::menu::MenuButtonTexture,
            &mut ImageNode,
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
    for (interaction, action, btn_tex, mut img) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                img.image = btn_tex.hover.clone();

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
                            selected.0 = Some(Voxel::Soil_Grass);
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
                img.image = btn_tex.hover.clone();
            }
            Interaction::None => {
                img.image = btn_tex.normal.clone();
            }
        }
    }
}
