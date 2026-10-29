pub mod camera;
pub mod collision;
pub mod controller;
pub mod dynamic_light;
pub mod game_mode;
pub mod hotbar;
pub mod inventory;
pub mod model;
pub mod movement;
pub mod spectator;
pub mod state;
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

pub use game_mode::GameMode;
pub use inventory::PlayerInventory;
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
            .init_resource::<InspectorInteraction>()
            .init_resource::<PlayerEnvironmentStatus>()
            .init_resource::<PlayerInventory>()
            .add_plugins(hotbar::HotbarPlugin)
            .add_plugins(model::PlayerModelPlugin)
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
                    controller::toggle_camera_view,
                    controller::camera_look,
                    controller::creative_movement,
                    spectator::spectator_movement,
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
        bevy::render::view::ColorGrading::default(),
        crate::environment::post_process::PostProcessSettings::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV_DEGREES.to_radians(),
            near: 0.05,
            far: 2000.0,
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

    let hdr_bloom_active = local_config
        .as_ref()
        .map(|c| c.hdr_bloom)
        .unwrap_or(true);

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
            commands.entity(camera_entity).insert(default_bloom_settings());
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
                Node {
                    position_type: PositionType::Absolute,
                    width: px(2.0),
                    height: px(14.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px(14.0),
                    height: px(2.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            ),
        ],
    ));
}

fn update_crosshair_visibility(
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    mut crosshair_query: Query<&mut Visibility, With<CrosshairUi>>,
) {
    let hide = menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active);

    for mut vis in crosshair_query.iter_mut() {
        let target_vis = if hide {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if *vis != target_vis {
            *vis = target_vis;
        }
    }
}
