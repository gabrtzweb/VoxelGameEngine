pub mod collision;
pub mod controller;
pub mod game_mode;
pub mod hotbar;
pub mod model;
pub mod spectator;
pub mod state;
pub mod water;

use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

#[allow(unused_imports)]
pub use controller::PlayerStance;
pub use controller::{InspectorInteraction, PlayerCamera, PlayerMotion};

pub use game_mode::GameMode;
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
                )
                    .chain()
                    .in_set(PlayerSet::Movement),
            );
    }
}

fn spawn_player_and_camera(
    mut commands: Commands,
    terrain_generator: Option<Res<crate::generation::TerrainGenerator>>,
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

    commands.spawn((
        Camera3d::default(),
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
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV_DEGREES.to_radians(),
            near: 0.05,
            ..default()
        }),
        camera_transform,
        player_camera,
    ));
}

fn lock_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.visible = false;

    cursor_options.grab_mode = CursorGrabMode::Locked;
}

fn spawn_crosshair(mut commands: Commands) {
    commands.spawn((
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
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9,),),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,

                    width: px(14.0),

                    height: px(2.0),

                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9,),),
            ),
        ],
    ));
}
