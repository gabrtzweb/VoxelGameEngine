use bevy::prelude::*;

use super::{
    PLAYER_EYE_HEIGHT, Player,
    controller::{PlayerCamera, PlayerMotion},
};

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Creative,
    Spectator,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Creative => "Creative",
            Self::Spectator => "Spectator",
        }
    }
}

pub(super) fn toggle_game_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_mode: ResMut<GameMode>,
    player: Single<(&Transform, &mut PlayerMotion), With<Player>>,
    mut camera: Single<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if !keyboard.just_pressed(KeyCode::F4) {
        return;
    }

    let (player_transform, mut motion) = player.into_inner();

    motion.velocity = Vec3::ZERO;

    *game_mode = match *game_mode {
        GameMode::Creative => GameMode::Spectator,

        GameMode::Spectator => {
            camera.translation = player_transform.translation + Vec3::Y * PLAYER_EYE_HEIGHT;

            GameMode::Creative
        }
    };

    info!("Game mode: {}", game_mode.label(),);
}
