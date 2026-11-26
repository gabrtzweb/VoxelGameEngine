use bevy::prelude::*;

use super::{
    GameModeToastState, InspectorInteraction, PLAYER_EYE_HEIGHT, Player,
    controller::{PlayerCamera, PlayerMotion},
};

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Survival,
    Creative,
    Spectator,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeCamState(pub bool);

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
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    mut game_mode: ResMut<GameMode>,
    mut free_cam: ResMut<FreeCamState>,
    player: Single<(&Transform, &mut PlayerMotion), With<Player>>,
    camera: Single<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }

    if !keyboard.just_pressed(KeyCode::F4) {
        return;
    }

    let (player_transform, mut motion) = player.into_inner();
    let mut cam_transform = camera.into_inner();

    if free_cam.0 {
        free_cam.0 = false;
        cam_transform.translation = player_transform.translation + Vec3::Y * PLAYER_EYE_HEIGHT;
    }

    motion.velocity = Vec3::ZERO;

    *game_mode = match *game_mode {
        GameMode::Survival => GameMode::Creative,
        GameMode::Creative => {
            motion.flying = true;
            motion.grounded = false;
            GameMode::Spectator
        }
        GameMode::Spectator => {
            motion.flying = false;
            GameMode::Survival
        }
    };

    if *game_mode == GameMode::Survival {
        motion.flying = false;
    }

    info!("Game mode: {}", game_mode.label());
}

pub(super) fn toggle_free_cam(
    keyboard: Res<ButtonInput<KeyCode>>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    loc: Option<Res<crate::core::Localization>>,
    mut free_cam: ResMut<FreeCamState>,
    mut toast: Option<ResMut<GameModeToastState>>,
    player: Single<(&Transform, &mut PlayerMotion), With<Player>>,
    camera: Single<(&mut Transform, &mut PlayerCamera), Without<Player>>,
) {
    if menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }

    if !keyboard.just_pressed(KeyCode::F8) {
        return;
    }

    let (player_transform, mut motion) = player.into_inner();
    let (mut cam_transform, mut cam_ctrl) = camera.into_inner();

    free_cam.0 = !free_cam.0;
    motion.velocity = Vec3::ZERO;

    if free_cam.0 {
        let eye_pos = player_transform.translation + Vec3::Y * PLAYER_EYE_HEIGHT;
        let horiz_forward = Quat::from_rotation_y(cam_ctrl.yaw) * Vec3::NEG_Z;
        cam_transform.translation = eye_pos - horiz_forward * 3.8 + Vec3::Y * 1.1;
        cam_ctrl.pitch = -0.22;
        cam_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, cam_ctrl.yaw, cam_ctrl.pitch, 0.0);
    } else {
        cam_transform.translation = player_transform.translation + Vec3::Y * PLAYER_EYE_HEIGHT;
    }

    if let Some(ref mut toast_state) = toast {
        let msg = if free_cam.0 {
            loc.as_ref()
                .map_or("Free Cam: Enabled", |l| l.t("hud.freecam.on"))
        } else {
            loc.as_ref()
                .map_or("Free Cam: Disabled", |l| l.t("hud.freecam.off"))
        };
        toast_state.show_message(msg);
    }

    info!("Free Cam: {}", if free_cam.0 { "enabled" } else { "disabled" });
}
