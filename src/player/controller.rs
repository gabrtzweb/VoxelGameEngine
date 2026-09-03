use std::f32::consts::FRAC_PI_2;

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

use crate::voxel::VoxelWorld;

use super::{
    GameMode, PLAYER_EYE_HEIGHT, Player,
    collision::{is_grounded, move_with_collisions},
};

const WALK_SPEED: f32 = 5.0;
const SPRINT_SPEED: f32 = 8.0;

const FLY_SPEED: f32 = 7.0;
const FAST_FLY_SPEED: f32 = 15.0;

const GRAVITY: f32 = 24.0;
const TERMINAL_VELOCITY: f32 = 50.0;
const JUMP_SPEED: f32 = 8.0;

const DOUBLE_JUMP_WINDOW: f32 = 0.30;

const THIRD_PERSON_DISTANCE: f32 = 3.5;

const STEP_CAMERA_RECOVERY_SPEED: f32 = 14.0;

#[derive(Component)]
pub struct PlayerCamera {
    pub sensitivity: f32,
    pub spectator_speed: f32,
    pub spectator_fast_speed: f32,

    yaw: f32,
    pitch: f32,

    third_person: bool,
}

impl PlayerCamera {
    pub fn from_transform(transform: &Transform) -> Self {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

        Self {
            sensitivity: 0.002,
            spectator_speed: 5.0,
            spectator_fast_speed: 15.0,
            yaw,
            pitch,
            third_person: false,
        }
    }

    pub fn is_third_person(&self) -> bool {
        self.third_person
    }
}

#[derive(Component)]
pub struct PlayerMotion {
    pub velocity: Vec3,
    pub grounded: bool,
    pub flying: bool,

    pub(super) facing_yaw: f32,

    camera_step_offset: f32,
}

impl Default for PlayerMotion {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            grounded: false,
            flying: false,
            facing_yaw: 0.0,
            camera_step_offset: 0.0,
        }
    }
}

#[derive(Default)]
pub(super) struct JumpTapState {
    since_last_press: Option<f32>,
}

pub(super) fn toggle_camera_view(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_mode: Res<GameMode>,
    camera: Single<&mut PlayerCamera, With<Camera3d>>,
) {
    if *game_mode != GameMode::Creative {
        return;
    }

    if !keyboard.just_pressed(KeyCode::F5) {
        return;
    }

    let mut camera = camera.into_inner();

    camera.third_person = !camera.third_person;

    info!(
        "Camera view: {}",
        if camera.third_person {
            "third person"
        } else {
            "first person"
        }
    );
}

pub(super) fn camera_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    camera: Single<(&mut Transform, &mut PlayerCamera), With<Camera3d>>,
) {
    let delta = mouse_motion.delta;

    if delta == Vec2::ZERO {
        return;
    }

    let (mut transform, mut camera) = camera.into_inner();

    camera.yaw -= delta.x * camera.sensitivity;

    camera.pitch -= delta.y * camera.sensitivity;

    camera.pitch = camera.pitch.clamp(-FRAC_PI_2 + 0.001, FRAC_PI_2 - 0.001);

    transform.rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);
}

pub(super) fn creative_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    world: Res<VoxelWorld>,
    player: Single<(&mut Transform, &mut PlayerMotion), With<Player>>,
    camera: Single<(&mut Transform, &PlayerCamera), (With<Camera3d>, Without<Player>)>,
    mut jump_tap: Local<JumpTapState>,
) {
    if *game_mode != GameMode::Creative {
        jump_tap.since_last_press = None;

        return;
    }

    let delta_seconds = time.delta_secs();

    if let Some(elapsed) = jump_tap.since_last_press.as_mut() {
        *elapsed += delta_seconds;
    }

    let (mut player_transform, mut motion) = player.into_inner();

    let (mut camera_transform, camera_controller) = camera.into_inner();

    if !motion.flying {
        motion.grounded = is_grounded(&world, player_transform.translation);
    }

    if keyboard.just_pressed(KeyCode::Space) {
        let double_tap = jump_tap
            .since_last_press
            .is_some_and(|elapsed| elapsed <= DOUBLE_JUMP_WINDOW);

        if double_tap {
            motion.flying = !motion.flying;

            motion.velocity = Vec3::ZERO;

            motion.grounded = false;

            motion.camera_step_offset = 0.0;

            jump_tap.since_last_press = None;

            info!(
                "Creative flight: {}",
                if motion.flying { "enabled" } else { "disabled" }
            );
        } else {
            jump_tap.since_last_press = Some(0.0);

            if motion.grounded && !motion.flying {
                motion.velocity.y = JUMP_SPEED;

                motion.grounded = false;
            }
        }
    }

    let camera_forward = *camera_transform.forward();

    let camera_right = *camera_transform.right();

    let (yaw, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    motion.facing_yaw = yaw;

    let is_fast = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if motion.flying {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            direction += camera_forward;
        }

        if keyboard.pressed(KeyCode::KeyS) {
            direction -= camera_forward;
        }

        if keyboard.pressed(KeyCode::KeyD) {
            direction += camera_right;
        }

        if keyboard.pressed(KeyCode::KeyA) {
            direction -= camera_right;
        }

        if keyboard.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }

        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            direction -= Vec3::Y;
        }

        let speed = if is_fast { FAST_FLY_SPEED } else { FLY_SPEED };

        let movement = if direction == Vec3::ZERO {
            Vec3::ZERO
        } else {
            direction.normalize() * speed * delta_seconds
        };

        let (new_position, collision) =
            move_with_collisions(&world, player_transform.translation, movement, false);

        player_transform.translation = new_position;

        motion.velocity = Vec3::ZERO;

        motion.grounded = collision.grounded;

        motion.camera_step_offset = 0.0;
    } else {
        let mut forward = Vec3::new(camera_forward.x, 0.0, camera_forward.z);

        let mut right = Vec3::new(camera_right.x, 0.0, camera_right.z);

        if forward.length_squared() > 0.0 {
            forward = forward.normalize();
        }

        if right.length_squared() > 0.0 {
            right = right.normalize();
        }

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

        if direction != Vec3::ZERO {
            direction = direction.normalize();
        }

        let speed = if is_fast { SPRINT_SPEED } else { WALK_SPEED };

        motion.velocity.x = direction.x * speed;

        motion.velocity.z = direction.z * speed;

        motion.velocity.y -= GRAVITY * delta_seconds;

        motion.velocity.y = motion.velocity.y.max(-TERMINAL_VELOCITY);

        let movement = motion.velocity * delta_seconds;

        let allow_step = motion.grounded && motion.velocity.y <= 0.0;

        let (new_position, collision) =
            move_with_collisions(&world, player_transform.translation, movement, allow_step);

        player_transform.translation = new_position;

        if collision.blocked_x {
            motion.velocity.x = 0.0;
        }

        if collision.blocked_z {
            motion.velocity.z = 0.0;
        }

        if collision.blocked_y {
            motion.velocity.y = 0.0;
        }

        motion.grounded = collision.grounded;

        if collision.step_height > 0.0 {
            // The physical body moves instantly,
            // while the camera catches up smoothly.
            motion.camera_step_offset -= collision.step_height;
        }

        let smoothing = 1.0 - (-STEP_CAMERA_RECOVERY_SPEED * delta_seconds).exp();

        motion.camera_step_offset += (0.0 - motion.camera_step_offset) * smoothing;

        if motion.camera_step_offset.abs() < 0.001 {
            motion.camera_step_offset = 0.0;
        }
    }

    let eye_position =
        player_transform.translation + Vec3::Y * (PLAYER_EYE_HEIGHT + motion.camera_step_offset);

    if camera_controller.is_third_person() {
        camera_transform.translation = eye_position - camera_forward * THIRD_PERSON_DISTANCE;
    } else {
        camera_transform.translation = eye_position;
    }
}
