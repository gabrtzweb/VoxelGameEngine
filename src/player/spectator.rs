use bevy::prelude::*;

use super::{FastFlightState, FreeCamState, InspectorInteraction, PlayerCamera};

pub(super) fn spectator_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    control_settings: Option<Res<crate::core::ControlSettings>>,
    time: Res<Time>,
    free_cam: Res<FreeCamState>,
    fast_flight: Option<Res<FastFlightState>>,
    inspector: Option<Res<InspectorInteraction>>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    camera: Single<(&mut Transform, &PlayerCamera), With<Camera3d>>,
) {
    if menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }

    if !free_cam.0 {
        return;
    }

    let (mut transform, camera) = camera.into_inner();

    let forward = *transform.forward();
    let right = *transform.right();

    let mut direction = Vec3::ZERO;

    let fwd_pressed = control_settings
        .as_ref()
        .map_or(keyboard.pressed(KeyCode::KeyW), |c| {
            c.is_action_pressed(crate::core::InputAction::MoveForward, &keyboard, &mouse)
        });
    let back_pressed = control_settings
        .as_ref()
        .map_or(keyboard.pressed(KeyCode::KeyS), |c| {
            c.is_action_pressed(crate::core::InputAction::MoveBackward, &keyboard, &mouse)
        });
    let right_pressed = control_settings
        .as_ref()
        .map_or(keyboard.pressed(KeyCode::KeyD), |c| {
            c.is_action_pressed(crate::core::InputAction::MoveRight, &keyboard, &mouse)
        });
    let left_pressed = control_settings
        .as_ref()
        .map_or(keyboard.pressed(KeyCode::KeyA), |c| {
            c.is_action_pressed(crate::core::InputAction::MoveLeft, &keyboard, &mouse)
        });
    let up_pressed = control_settings
        .as_ref()
        .map_or(keyboard.pressed(KeyCode::Space), |c| {
            c.is_action_pressed(crate::core::InputAction::Jump, &keyboard, &mouse)
        });
    let down_pressed = control_settings.as_ref().map_or(
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight),
        |c| c.is_action_pressed(crate::core::InputAction::Crouch, &keyboard, &mouse),
    );

    if fwd_pressed {
        direction += forward;
    }
    if back_pressed {
        direction -= forward;
    }
    if right_pressed {
        direction += right;
    }
    if left_pressed {
        direction -= right;
    }
    if up_pressed {
        direction += Vec3::Y;
    }
    if down_pressed {
        direction -= Vec3::Y;
    }

    if direction == Vec3::ZERO {
        return;
    }

    let is_fast = control_settings.as_ref().map_or(
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
        |c| c.is_action_pressed(crate::core::InputAction::Sprint, &keyboard, &mouse),
    );

    let boost_mult = if fast_flight.as_ref().is_some_and(|f| f.0) {
        5.0
    } else {
        1.0
    };
    let speed = if is_fast {
        camera.spectator_fast_speed
    } else {
        camera.spectator_speed
    } * boost_mult;

    transform.translation += direction.normalize() * speed * time.delta_secs();
}
