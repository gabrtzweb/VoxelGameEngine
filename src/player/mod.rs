pub mod collision;
pub mod controller;
pub mod game_mode;
pub mod spectator;

use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

pub use controller::{PlayerCamera, PlayerMotion};
pub use game_mode::GameMode;

pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;

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
            .configure_sets(Update, PlayerSet::Movement)
            .add_systems(Startup, (spawn_player_and_camera, spawn_crosshair))
            .add_systems(PostStartup, lock_cursor)
            .add_systems(
                Update,
                (
                    game_mode::toggle_game_mode,
                    controller::camera_look,
                    controller::creative_movement,
                    spectator::spectator_movement,
                )
                    .chain()
                    .in_set(PlayerSet::Movement),
            );
    }
}

fn spawn_player_and_camera(mut commands: Commands) {
    // Keep the camera roughly at the old
    // development-camera starting position.
    let player_position = Vec3::new(-10.0, 10.38, 14.0);

    commands.spawn((
        Player,
        PlayerMotion::default(),
        Transform::from_translation(player_position),
    ));

    let camera_position = player_position + Vec3::Y * PLAYER_EYE_HEIGHT;

    let camera_transform =
        Transform::from_translation(camera_position).looking_at(Vec3::new(4.0, 3.0, 4.0), Vec3::Y);

    let player_camera = PlayerCamera::from_transform(&camera_transform);

    commands.spawn((Camera3d::default(), camera_transform, player_camera));
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
