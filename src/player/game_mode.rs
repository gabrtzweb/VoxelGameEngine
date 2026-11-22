use bevy::prelude::*;

use super::{
    PLAYER_EYE_HEIGHT, Player,
    controller::{PlayerCamera, PlayerMotion},
};

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Survival,
    Creative,
    Spectator,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Survival => "Survival",
            Self::Creative => "Creative",
            Self::Spectator => "Spectator",
        }
    }

    pub fn loc_key(self) -> &'static str {
        match self {
            Self::Survival => "hud.gamemode.survival",
            Self::Creative => "hud.gamemode.creative",
            Self::Spectator => "hud.gamemode.spectator",
        }
    }
}

pub(super) fn toggle_game_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_mode: ResMut<GameMode>,
    player: Single<(&Transform, &mut PlayerMotion), With<Player>>,
    camera: Single<(&mut Transform, &mut PlayerCamera), Without<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::F4) {
        return;
    }

    let (player_transform, mut motion) = player.into_inner();
    let (mut cam_transform, mut cam_ctrl) = camera.into_inner();

    motion.velocity = Vec3::ZERO;

    *game_mode = match *game_mode {
        GameMode::Survival => GameMode::Creative,
        GameMode::Creative => {
            let eye_pos = player_transform.translation + Vec3::Y * PLAYER_EYE_HEIGHT;
            let horiz_forward = Quat::from_rotation_y(cam_ctrl.yaw) * Vec3::NEG_Z;
            cam_transform.translation = eye_pos - horiz_forward * 3.8 + Vec3::Y * 1.1;
            cam_ctrl.pitch = -0.22;
            cam_transform.rotation =
                Quat::from_euler(EulerRot::YXZ, cam_ctrl.yaw, cam_ctrl.pitch, 0.0);
            GameMode::Spectator
        }
        GameMode::Spectator => {
            cam_transform.translation = player_transform.translation + Vec3::Y * PLAYER_EYE_HEIGHT;
            motion.flying = false;
            GameMode::Survival
        }
    };

    if *game_mode == GameMode::Survival {
        motion.flying = false;
    }

    info!("Game mode: {}", game_mode.label());
}
