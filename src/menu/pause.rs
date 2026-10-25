use bevy::prelude::*;
use bevy::{picking::hover::Hovered, ui::Pressed, ui_widgets::Button};

use crate::{
    core::{AppFont, Localization, text_shadow_default},
    environment::EnvironmentState,
    gameplay::SelectedVoxel,
    generation::TerrainGenerator,
    meshing::{ChunkMaterial, ChunkMeshRegistry, sync_chunk_render},
    player::{PLAYER_EYE_HEIGHT, Player, PlayerCamera, PlayerMotion, hotbar::Hotbar},
    simulation::{VoxelLightRegistry, sync_chunk_lights},
    world::{Voxel, VoxelWorld, WorldModificationStore},
};

use super::MenuState;

pub const PAUSE_SCALE: f32 = 4.0;

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum PauseMenuAction {
    Resume,    // 01
    Settings,  // 02
    Controls,  // 03 (WIP)
    Stats,     // 04 (WIP)
    Restart,   // 05
    Quit,      // 06
}

#[derive(Component)]
struct PauseButtonLabel;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Pause), spawn_pause_menu)
            .add_systems(OnExit(MenuState::Pause), despawn_pause_menu)
            .add_systems(
                Update,
                (handle_pause_menu_buttons, update_pause_button_hover)
                    .run_if(in_state(MenuState::Pause)),
            );
    }
}

fn spawn_pause_menu(
    mut commands: Commands,
    loc: Option<Res<Localization>>,
    app_font: Option<Res<AppFont>>,
    asset_server: Res<AssetServer>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
) {
    let gui_scale = game_settings.as_ref().map_or(4, |s| s.gui_scale).clamp(2, 4);
    let scale_factor = (gui_scale as f32 / 4.0).clamp(0.5, 1.0);
    let font_handle = app_font.as_ref().map(|f| f.source());
    let pause_texture =
        asset_server.load("textures/interfaces/containers/pause.png");

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

    let dimming_alpha = game_settings
        .as_ref()
        .map_or(0.75, |s| s.background_dimming.alpha());

    commands
        .spawn((
            PauseMenuRoot,
            crate::menu::MenuBackdrop,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, dimming_alpha)),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            // Fullscreen container image (480x270 @ 4x = 1920x1080, modulated by gui_scale)
            backdrop
                .spawn((
                    ImageNode {
                        image: pause_texture,
                        ..default()
                    },
                    UiTransform {
                        scale: Vec2::splat(scale_factor),
                        ..UiTransform::IDENTITY
                    },
                    Node {
                        width: px(1920.0),
                        height: px(1080.0),
                        ..default()
                    },
                ))
                .with_children(|container| {
                    let resume_lbl = loc.as_ref().map_or("Resume Game", |l| l.t("menu.pause.resume"));
                    let settings_lbl = loc.as_ref().map_or("Settings & Options", |l| l.t("menu.pause.settings"));
                    let controls_lbl = loc.as_ref().map_or("Controls & Keybinds", |l| l.t("menu.pause.controls"));
                    let stats_lbl = loc.as_ref().map_or("Statistics & Feats", |l| l.t("menu.pause.stats"));
                    let restart_lbl = loc.as_ref().map_or("Restart Game", |l| l.t("menu.pause.restart"));
                    let quit_lbl = loc.as_ref().map_or("Quit to Desktop", |l| l.t("menu.pause.quit"));

                    // 6 Buttons in column at x = 19 (76 px), w = 128 (512 px), h = 14 (56 px)
                    let buttons = [
                        (PauseMenuAction::Resume, resume_lbl, 40.0),
                        (PauseMenuAction::Settings, settings_lbl, 58.0),
                        (PauseMenuAction::Controls, controls_lbl, 76.0),
                        (PauseMenuAction::Stats, stats_lbl, 94.0),
                        (PauseMenuAction::Restart, restart_lbl, 112.0),
                        (PauseMenuAction::Quit, quit_lbl, 130.0),
                    ];

                    for (action, label, y) in buttons {
                        container
                            .spawn((
                                Button,
                                Hovered::default(),
                                action,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(19.0 * PAUSE_SCALE),
                                    top: px(y * PAUSE_SCALE),
                                    width: px(128.0 * PAUSE_SCALE),
                                    height: px(14.0 * PAUSE_SCALE),
                                    display: Display::Flex,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    PauseButtonLabel,
                                    Text::new(label),
                                    make_font(18.0),
                                    TextColor(Color::srgb(0.90, 0.92, 0.96)),
                                    text_shadow_default(),
                                ));
                            });
                    }
                });
        });
}

fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn update_pause_button_hover(
    buttons_query: Query<(&Hovered, Has<Pressed>, &Children), With<Button>>,
    mut labels_query: Query<&mut TextColor, With<PauseButtonLabel>>,
) {
    for (hovered, is_pressed, children) in &buttons_query {
        let is_active = hovered.get() || is_pressed;
        for child in children.iter() {
            if let Ok(mut text_color) = labels_query.get_mut(child) {
                *text_color = if is_active {
                    TextColor(Color::srgb(1.0, 0.90, 0.40)) // Golden glow on hover
                } else {
                    TextColor(Color::srgb(0.90, 0.92, 0.96))
                };
            }
        }
    }
}

type PauseRestartParams<'w, 's> = (
    Option<ResMut<'w, VoxelWorld>>,
    Option<ResMut<'w, WorldModificationStore>>,
    Option<Res<'w, TerrainGenerator>>,
    Option<ResMut<'w, VoxelLightRegistry>>,
    Option<ResMut<'w, ChunkMeshRegistry>>,
    Option<ResMut<'w, Assets<Mesh>>>,
    Option<Res<'w, ChunkMaterial>>,
    Option<ResMut<'w, Hotbar>>,
    Option<ResMut<'w, SelectedVoxel>>,
    Option<ResMut<'w, crate::map::cache::MapCache>>,
);

type PauseWeatherRestartParams<'w> = (
    Option<ResMut<'w, crate::environment::weather::WeatherManager>>,
    Option<ResMut<'w, crate::environment::weather::LightningState>>,
    Option<ResMut<'w, crate::environment::weather::WindState>>,
    Option<ResMut<'w, crate::environment::weather::WeatherParticlePool>>,
    Option<ResMut<'w, crate::audio::weather_audio::PendingThunderQueue>>,
    Option<ResMut<'w, crate::player::PlayerInventory>>,
);

fn handle_pause_menu_buttons(
    buttons_query: Query<(&Hovered, Has<Pressed>, &PauseMenuAction), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut env_state: Option<ResMut<EnvironmentState>>,
    mut player_query: Query<(&mut Transform, &mut PlayerMotion), With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), (With<Camera3d>, Without<Player>)>,
    (
        mut world_query,
        mut modifications_query,
        terrain_generator_query,
        mut light_registry_query,
        mut mesh_registry_query,
        mut meshes_query,
        chunk_material_query,
        mut hotbar_query,
        mut selected_voxel_query,
        mut map_cache_query,
    ): PauseRestartParams,
    (
        mut weather_query,
        mut lightning_query,
        mut wind_query,
        mut particle_pool_query,
        mut pending_thunder_query,
        mut player_inv_query,
    ): PauseWeatherRestartParams,
    lightning_entities_query: Query<
        Entity,
        Or<(
            With<crate::environment::weather::lightning::LightningBoltEntity>,
            With<crate::environment::weather::lightning::LightningImpactLight>,
        )>,
    >,
    mut commands: Commands,
) {
    let left_clicked = mouse.just_pressed(MouseButton::Left);

    for (hovered, is_pressed, action) in &buttons_query {
        if hovered.get() && (left_clicked || is_pressed) {
            match action {
                PauseMenuAction::Resume => {
                    next_state.set(MenuState::None);
                }
                PauseMenuAction::Settings => {
                    next_state.set(MenuState::Settings);
                }
                PauseMenuAction::Controls => {
                    next_state.set(MenuState::Controls);
                }
                PauseMenuAction::Stats => {
                    info!("Statistics & Feats tracking coming soon in Phase 2!");
                }
                PauseMenuAction::Restart => {
                    // 1. Reset Environment to default Day 1 Noon
                    if let Some(ref mut env) = env_state {
                        **env = EnvironmentState::default();
                    }

                    // 2. Reset Weather, Lightning, Wind, and clear active particles / pending thunder
                    if let Some(ref mut weather) = weather_query {
                        **weather = crate::environment::weather::WeatherManager::default();
                    }
                    if let Some(ref mut lightning) = lightning_query {
                        **lightning = crate::environment::weather::LightningState::default();
                    }
                    if let Some(ref mut wind) = wind_query {
                        **wind = crate::environment::weather::WindState::default();
                    }
                    if let Some(ref mut particle_pool) = particle_pool_query {
                        particle_pool.clear();
                    }
                    if let Some(ref mut pending_thunder) = pending_thunder_query {
                        pending_thunder.0.clear();
                    }
                    for entity in &lightning_entities_query {
                        commands.entity(entity).despawn();
                    }

                    // 3. Teleport player and camera to spawn
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

                    // 4. Reverse world modifications
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

                    // 5. Reset hotbar, player inventory, and selected voxel
                    if let Some(ref mut hotbar) = hotbar_query {
                        **hotbar = Hotbar::default();
                    }
                    if let Some(ref mut player_inv) = player_inv_query {
                        **player_inv = crate::player::PlayerInventory::default();
                    }
                    if let Some(ref mut selected) = selected_voxel_query {
                        selected.0 = Some(Voxel::LoamGrass);
                    }
                    if let Some(ref mut cache) = map_cache_query {
                        cache.clear();
                    }

                    info!(
                        "Game restarted: player teleported to spawn, world modifications reversed, day reset to 1, weather reset to Clear Sky"
                    );
                    next_state.set(MenuState::None);
                }
                PauseMenuAction::Quit => {
                    commands.write_message(bevy::app::AppExit::Success);
                }
            }
        }
    }
}
