pub mod collision;
pub mod controller;
pub mod game_mode;
pub mod spectator;
pub mod water;

use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_sky_gradient::prelude::*;

pub use controller::{PlayerCamera, PlayerMotion};

pub use game_mode::GameMode;

pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;

const CAMERA_FOV_DEGREES: f32 = 90.0;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct PlayerBody;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerSet {
    Movement,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameMode>()
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
                    game_mode::toggle_game_mode,
                    controller::toggle_camera_view,
                    controller::camera_look,
                    controller::creative_movement,
                    spectator::spectator_movement,
                    update_player_body,
                    water::update_underwater_effect,
                )
                    .chain()
                    .in_set(PlayerSet::Movement),
            );
    }
}

fn spawn_player_and_camera(
    mut commands: Commands,

    mut meshes: ResMut<Assets<Mesh>>,

    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_position = Vec3::new(-10.0, 10.38, 14.0);

    commands.spawn((
        Player,
        PlayerMotion::default(),
        Transform::from_translation(player_position),
    ));

    let body_mesh = meshes.add(Cuboid::new(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH));

    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.45, 0.9),

        perceptual_roughness: 0.8,

        ..default()
    });

    commands.spawn((
        PlayerBody,
        Mesh3d(body_mesh),
        MeshMaterial3d(body_material),
        Transform::from_translation(player_position + Vec3::Y * (PLAYER_HEIGHT * 0.5)),
        Visibility::Hidden,
    ));

    let camera_position = player_position + Vec3::Y * PLAYER_EYE_HEIGHT;

    let camera_transform =
        Transform::from_translation(camera_position).looking_at(Vec3::new(4.0, 3.0, 4.0), Vec3::Y);

    let player_camera = PlayerCamera::from_transform(&camera_transform);

    commands.spawn((
        Camera3d::default(),
        SkyboxMagnetTag,
        Exposure { ev100: 11.0 },
        Tonemapping::AcesFitted,
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV_DEGREES.to_radians(),

            ..default()
        }),
        camera_transform,
        player_camera,
    ));
}

#[allow(clippy::type_complexity)]
fn update_player_body(
    game_mode: Res<GameMode>,

    player: Single<(&Transform, &PlayerMotion), With<Player>>,

    camera: Single<&PlayerCamera, With<Camera3d>>,

    body: Single<(&mut Transform, &mut Visibility), (With<PlayerBody>, Without<Player>)>,
) {
    let (player_transform, player_motion) = player.into_inner();

    let (mut body_transform, mut visibility) = body.into_inner();

    body_transform.translation = player_transform.translation + Vec3::Y * (PLAYER_HEIGHT * 0.5);

    body_transform.rotation = Quat::from_rotation_y(player_motion.facing_yaw);

    let should_be_visible = *game_mode == GameMode::Spectator || camera.is_third_person();

    *visibility = if should_be_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
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
