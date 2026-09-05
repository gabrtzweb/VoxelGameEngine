use bevy::prelude::*;

use super::{GameMode, PlayerCamera};

pub(super) fn spectator_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    camera: Single<(&mut Transform, &PlayerCamera), With<PlayerCamera>>,
) {
    if *game_mode != GameMode::Spectator {
        return;
    }

    let (mut transform, camera) = camera.into_inner();

    let forward = *transform.forward();

    let right = *transform.right();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += forward;
    }

    if keyboard.pressed(KeyCode::KeyS) {
        direction -= forward;
    }

    if keyboard.pressed(KeyCode::KeyD) {
        direction += right;
    }

    if keyboard.pressed(KeyCode::KeyA) {
        direction -= right;
    }

    if keyboard.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }

    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        direction -= Vec3::Y;
    }

    if direction == Vec3::ZERO {
        return;
    }

    let is_fast = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let speed = if is_fast {
        camera.spectator_fast_speed
    } else {
        camera.spectator_speed
    };

    transform.translation += direction.normalize() * speed * time.delta_secs();
}
