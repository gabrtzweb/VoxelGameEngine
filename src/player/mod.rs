pub mod camera;
pub mod collision;
pub mod controller;
pub mod dynamic_light;
pub mod game_mode;
pub mod hotbar;
pub mod inventory;
pub mod model;
pub mod movement;
pub mod race;
pub mod spectator;
pub mod state;
pub mod survival_hud;
pub mod water;

use bevy::{
    camera::Exposure,
    core_pipeline::prepass::DepthPrepass,
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Tonemapping,
    window::{CursorGrabMode, CursorOptions},
};

pub use controller::{InspectorInteraction, PlayerCamera, PlayerMotion, PlayerStance};

pub use game_mode::{FreeCamState, GameMode};
pub use inventory::PlayerInventory;
pub use race::{
    CharacterRace, CharacterReady, CharacterSubrace, RaceAttributes, RaceProportions, SelectedRace,
    SelectedSubrace,
};
pub use state::PlayerEnvironmentStatus;

pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;

const CAMERA_FOV_DEGREES: f32 = 90.0;

#[derive(Component)]
pub struct Player;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerSet {
    Movement,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameMode>()
            .init_resource::<SelectedRace>()
            .init_resource::<SelectedSubrace>()
            .init_resource::<CharacterReady>()
            .init_resource::<InspectorInteraction>()
            .init_resource::<PlayerEnvironmentStatus>()
            .init_resource::<PlayerInventory>()
            .init_resource::<FastFlightState>()
            .init_resource::<FreeCamState>()
            .add_plugins(hotbar::HotbarPlugin)
            .add_plugins(model::PlayerModelPlugin)
            .add_plugins(survival_hud::SurvivalHudPlugin)
            .configure_sets(Update, PlayerSet::Movement)
            .add_systems(
                Startup,
                (
                    spawn_player_and_camera,
                    spawn_crosshair,
                    water::spawn_underwater_overlay,
                ),
            )
            .add_systems(PostStartup, lock_cursor)
            .add_systems(
                Update,
                (
                    state::update_player_environment_status,
                    controller::toggle_inspector_interaction,
                    game_mode::toggle_game_mode,
                    game_mode::toggle_free_cam,
                    toggle_fast_flight,
                    controller::toggle_camera_view,
                    controller::camera_look,
                    controller::creative_movement,
                    spectator::spectator_movement,
                    enforce_adventure_world_bounds,
                    water::update_underwater_effect,
                    update_crosshair_visibility,
                    sync_camera_transmission,
                    sync_camera_anti_aliasing,
                    sync_camera_bloom,
                    dynamic_light::update_player_dynamic_light,
                )
                    .chain()
                    .in_set(PlayerSet::Movement),
            );
    }
}

pub use survival_hud::{GameModeToastState, PlayerVitals};

#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldSpawnPoint(pub Vec3);

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FastFlightState(pub bool);

/// Toggles 5x Fast Flight with the `F` key in Creative, Spectator, and Free Cam modes.
fn toggle_fast_flight(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_mode: Res<GameMode>,
    free_cam: Res<FreeCamState>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    loc: Option<Res<crate::core::Localization>>,
    mut fast_flight: ResMut<FastFlightState>,
    mut toast: Option<ResMut<GameModeToastState>>,
) {
    if *game_mode == GameMode::Survival && !free_cam.0 {
        if fast_flight.0 {
            fast_flight.0 = false;
        }
        return;
    }

    if menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyF) {
        fast_flight.0 = !fast_flight.0;
        if let Some(ref mut toast_state) = toast {
            let msg = if fast_flight.0 {
                loc.as_ref()
                    .map_or("Fast Flight: Enabled (5x)", |l| l.t("hud.fast_flight.on"))
            } else {
                loc.as_ref()
                    .map_or("Fast Flight: Disabled", |l| l.t("hud.fast_flight.off"))
            };
            toast_state.show_message(msg);
        }
    }
}

/// Enforces invisible boundary walls along the edges of the bounded Adventure Mode world,
/// preventing players (and Spectator cameras) from wandering infinitely into the outer ocean.
fn enforce_adventure_world_bounds(
    time: Res<Time>,
    terrain_generator: Option<Res<crate::generation::TerrainGenerator>>,
    loc: Option<Res<crate::core::Localization>>,
    mut toast: Option<ResMut<GameModeToastState>>,
    mut warn_cooldown: Local<f32>,
    mut player_q: Query<(&mut Transform, &mut PlayerMotion), With<Player>>,
    mut camera_q: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let Some(ref terrain_gen) = terrain_generator else {
        return;
    };
    let Some((half_w, half_h)) = terrain_gen
        .adventure_map
        .as_ref()
        .map(|m| m.half_extents())
        .or_else(|| {
            terrain_gen
                .laboratory_map
                .as_ref()
                .map(|l| l.half_extents())
        })
    else {
        return;
    };

    if *warn_cooldown > 0.0 {
        *warn_cooldown = (*warn_cooldown - time.delta_secs()).max(0.0);
    }

    let voxel_size = crate::world::VOXEL_SIZE;
    let margin = 4.0 * voxel_size;
    let max_x = (half_w * voxel_size - margin).max(16.0);
    let max_z = (half_h * voxel_size - margin).max(16.0);

    let mut hit_border = false;

    for (mut tf, mut motion) in &mut player_q {
        if tf.translation.x < -max_x {
            tf.translation.x = -max_x;
            motion.velocity.x = motion.velocity.x.max(0.0);
            hit_border = true;
        } else if tf.translation.x > max_x {
            tf.translation.x = max_x;
            motion.velocity.x = motion.velocity.x.min(0.0);
            hit_border = true;
        }

        if tf.translation.z < -max_z {
            tf.translation.z = -max_z;
            motion.velocity.z = motion.velocity.z.max(0.0);
            hit_border = true;
        } else if tf.translation.z > max_z {
            tf.translation.z = max_z;
            motion.velocity.z = motion.velocity.z.min(0.0);
            hit_border = true;
        }
    }

    for mut cam_tf in &mut camera_q {
        if cam_tf.translation.x < -max_x {
            cam_tf.translation.x = -max_x;
            hit_border = true;
        } else if cam_tf.translation.x > max_x {
            cam_tf.translation.x = max_x;
            hit_border = true;
        }
        if cam_tf.translation.z < -max_z {
            cam_tf.translation.z = -max_z;
            hit_border = true;
        } else if cam_tf.translation.z > max_z {
            cam_tf.translation.z = max_z;
            hit_border = true;
        }
    }

    if hit_border && *warn_cooldown <= 0.0 {
        *warn_cooldown = 2.6;
        if let Some(ref mut toast_state) = toast {
            let msg = loc.as_ref().map_or(
                "You have reached the edge of the known world",
                |l| l.t("hud.world_border.reached"),
            );
            toast_state.show_message(msg);
        }
    }
}

fn spawn_player_and_camera(
    mut commands: Commands,
    terrain_generator: Option<Res<crate::generation::TerrainGenerator>>,
    local_config: Option<Res<crate::core::config::LocalConfig>>,
) {
    let spawn_x = -10.0;
    let spawn_z = 14.0;
    let player_y = if let Some(ref generator) = terrain_generator {
        let col = generator.sample_column(spawn_x as i32, spawn_z as i32);
        ((col.terrain_height + 2) as f32 * crate::world::VOXEL_SIZE).max(10.38)
    } else {
        10.38
    };

    let player_position = Vec3::new(spawn_x, player_y, spawn_z);
    commands.insert_resource(WorldSpawnPoint(player_position));

    commands.spawn((
        Player,
        PlayerMotion::default(),
        Transform::from_translation(player_position),
    ));

    let camera_position = player_position + Vec3::Y * PLAYER_EYE_HEIGHT;

    let camera_transform =
        Transform::from_translation(camera_position).looking_at(Vec3::new(4.0, 3.0, 4.0), Vec3::Y);

    let player_camera = PlayerCamera::from_transform(&camera_transform);

    let mut camera_builder = commands.spawn((
        Camera3d::default(),
        bevy::camera::Hdr,
        DepthPrepass,
        DistanceFog {
            color: Color::srgb(0.67, 0.75, 0.82),

            directional_light_color: Color::srgba(1.0, 0.95, 0.88, 0.10),

            directional_light_exponent: 24.0,

            falloff: FogFalloff::Linear {
                start: 99999.0,
                end: 100000.0,
            },
        },
        Exposure { ev100: 11.0 },
        Tonemapping::AcesFitted,
        crate::environment::post_process::PostProcessSettings::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV_DEGREES.to_radians(),
            near: 0.05,
            far: 4000.0,
            ..default()
        }),
        camera_transform,
        player_camera,
    ));

    let transmission_active = local_config
        .as_ref()
        .map(|c| c.transmission_enabled)
        .unwrap_or(false);

    if transmission_active {
        camera_builder.insert(bevy::pbr::ScreenSpaceTransmission::default());
    }

    let hdr_bloom_active = local_config.as_ref().map(|c| c.hdr_bloom).unwrap_or(true);

    if hdr_bloom_active {
        camera_builder.insert(default_bloom_settings());
    }

    let aa_mode = local_config
        .as_ref()
        .map(|c| c.anti_aliasing)
        .unwrap_or(crate::core::AntiAliasingMode::Fxaa);

    match aa_mode {
        crate::core::AntiAliasingMode::Off => {
            camera_builder.insert(bevy::render::view::Msaa::Off);
        }
        crate::core::AntiAliasingMode::Fxaa => {
            camera_builder.insert((
                bevy::render::view::Msaa::Off,
                bevy::anti_alias::fxaa::Fxaa::default(),
            ));
        }
        crate::core::AntiAliasingMode::Msaa2x => {
            camera_builder.insert(bevy::render::view::Msaa::Sample2);
        }
        crate::core::AntiAliasingMode::Msaa4x => {
            camera_builder.insert(bevy::render::view::Msaa::Sample4);
        }
    }
}

fn sync_camera_transmission(
    local_config: Option<Res<crate::core::config::LocalConfig>>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    transmission_query: Query<&bevy::pbr::ScreenSpaceTransmission, With<PlayerCamera>>,
    mut commands: Commands,
) {
    let Some(local_config) = local_config else {
        return;
    };
    if !local_config.is_changed() {
        return;
    }
    if let Ok(camera_entity) = camera_query.single() {
        let has_transmission = transmission_query.get(camera_entity).is_ok();
        if local_config.transmission_enabled && !has_transmission {
            commands
                .entity(camera_entity)
                .insert(bevy::pbr::ScreenSpaceTransmission::default());
        } else if !local_config.transmission_enabled && has_transmission {
            commands
                .entity(camera_entity)
                .remove::<bevy::pbr::ScreenSpaceTransmission>();
        }
    }
}

pub fn default_bloom_settings() -> Bloom {
    let mut bloom = Bloom::NATURAL;
    bloom.intensity = 0.20;
    bloom.prefilter.threshold = 1.0;
    bloom.prefilter.threshold_softness = 0.5;
    bloom
}

fn sync_camera_bloom(
    local_config: Option<Res<crate::core::config::LocalConfig>>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    bloom_query: Query<&Bloom, With<PlayerCamera>>,
    mut commands: Commands,
) {
    let Some(local_config) = local_config else {
        return;
    };
    if !local_config.is_changed() {
        return;
    }
    if let Ok(camera_entity) = camera_query.single() {
        let has_bloom = bloom_query.get(camera_entity).is_ok();
        if local_config.hdr_bloom && !has_bloom {
            commands
                .entity(camera_entity)
                .insert(default_bloom_settings());
        } else if !local_config.hdr_bloom && has_bloom {
            commands.entity(camera_entity).remove::<Bloom>();
        }
    }
}

fn sync_camera_anti_aliasing(
    local_config: Option<Res<crate::core::config::LocalConfig>>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    mut commands: Commands,
) {
    let Some(local_config) = local_config else {
        return;
    };
    if !local_config.is_changed() {
        return;
    }
    if let Ok(camera_entity) = camera_query.single() {
        match local_config.anti_aliasing {
            crate::core::AntiAliasingMode::Off => {
                commands
                    .entity(camera_entity)
                    .insert(bevy::render::view::Msaa::Off)
                    .remove::<bevy::anti_alias::fxaa::Fxaa>();
            }
            crate::core::AntiAliasingMode::Fxaa => {
                commands.entity(camera_entity).insert((
                    bevy::render::view::Msaa::Off,
                    bevy::anti_alias::fxaa::Fxaa::default(),
                ));
            }
            crate::core::AntiAliasingMode::Msaa2x => {
                commands
                    .entity(camera_entity)
                    .insert(bevy::render::view::Msaa::Sample2)
                    .remove::<bevy::anti_alias::fxaa::Fxaa>();
            }
            crate::core::AntiAliasingMode::Msaa4x => {
                commands
                    .entity(camera_entity)
                    .insert(bevy::render::view::Msaa::Sample4)
                    .remove::<bevy::anti_alias::fxaa::Fxaa>();
            }
        }
    }
}

fn lock_cursor(
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    cursor_options.visible = false;
    if menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None) {
        cursor_options.grab_mode = CursorGrabMode::Confined;
    } else {
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

#[derive(Component)]
pub struct CrosshairUi;

#[derive(Component, Clone, Copy)]
struct CrosshairBar {
    vertical: bool,
}

fn spawn_crosshair(mut commands: Commands) {
    commands.spawn((
        CrosshairUi,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ZIndex(100),
        children![
            (
                CrosshairBar { vertical: true },
                Node {
                    position_type: PositionType::Absolute,
                    width: px(2.0),
                    height: px(14.0),
                    border: UiRect::all(px(0.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                BorderColor::all(Color::NONE),
            ),
            (
                CrosshairBar { vertical: false },
                Node {
                    position_type: PositionType::Absolute,
                    width: px(14.0),
                    height: px(2.0),
                    border: UiRect::all(px(0.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                BorderColor::all(Color::NONE),
            ),
        ],
    ));
}

fn update_crosshair_visibility(
    time: Res<Time>,
    game_mode: Res<GameMode>,
    free_cam: Res<FreeCamState>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
    current_target: Option<Res<crate::gameplay::CurrentTarget>>,
    mut fade_alpha: Local<f32>,
    mut crosshair_query: Query<&mut Visibility, With<CrosshairUi>>,
    mut bars_query: Query<(
        &CrosshairBar,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    let hide = menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active)
        || *game_mode == GameMode::Spectator
        || free_cam.0;

    let high_contrast = game_settings
        .as_ref()
        .is_some_and(|s| s.high_contrast_crosshair);
    let dynamic = game_settings.as_ref().is_some_and(|s| s.dynamic_crosshair);
    let has_target = current_target.as_ref().is_some_and(|t| t.hit.is_some());

    let target_alpha = if hide || (dynamic && !has_target) {
        0.0
    } else {
        1.0
    };

    let dt = time.delta_secs();
    if *fade_alpha == 0.0 && target_alpha == 1.0 && !dynamic {
        *fade_alpha = 1.0;
    } else {
        let smooth = 1.0 - (-14.0 * dt).exp();
        *fade_alpha += (target_alpha - *fade_alpha) * smooth;
        if (*fade_alpha - target_alpha).abs() < 0.01 {
            *fade_alpha = target_alpha;
        }
    }

    let show_vis = !hide && *fade_alpha > 0.01;
    for mut vis in crosshair_query.iter_mut() {
        let target_vis = if show_vis {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *vis != target_vis {
            *vis = target_vis;
        }
    }

    let (thickness, length, fill, border_px, border_col) = if high_contrast {
        (
            4.0,
            20.0,
            Color::srgba(1.0, 0.92, 0.25, *fade_alpha),
            1.0,
            Color::srgba(0.0, 0.0, 0.0, *fade_alpha),
        )
    } else {
        (
            2.0,
            14.0,
            Color::srgba(1.0, 1.0, 1.0, 0.90 * *fade_alpha),
            0.0,
            Color::NONE,
        )
    };

    for (bar, mut node, mut bg, mut border) in &mut bars_query {
        if bar.vertical {
            node.width = px(thickness);
            node.height = px(length);
        } else {
            node.width = px(length);
            node.height = px(thickness);
        }
        node.border = UiRect::all(px(border_px));
        bg.0 = fill;
        *border = BorderColor::all(border_col);
    }
}
