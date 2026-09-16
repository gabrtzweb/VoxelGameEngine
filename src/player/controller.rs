use std::f32::consts::FRAC_PI_2;

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

use crate::{
    menu::GameSettings,
    voxel::{VOXEL_SIZE, Voxel, VoxelWorld, shaping::RadialMenuState},
};

use super::{
    GameMode, PLAYER_EYE_HEIGHT, PLAYER_HEIGHT, Player, PlayerEnvironmentStatus,
    collision::{has_headroom, is_grounded, move_with_collisions},
};

const WALK_SPEED: f32 = 5.0;
const SPRINT_SPEED: f32 = 8.0;

pub const CROUCH_HEIGHT: f32 = 1.30;
pub const CRAWL_HEIGHT: f32 = 0.45;

pub const CROUCH_EYE_HEIGHT: f32 = 1.20;
pub const CRAWL_EYE_HEIGHT: f32 = 0.40;

const CROUCH_SPEED: f32 = WALK_SPEED * 0.55;
const CRAWL_SPEED: f32 = WALK_SPEED * 0.35;

const FLY_SPEED: f32 = 7.0;
const FAST_FLY_SPEED: f32 = 15.0;

const GRAVITY: f32 = 24.0;
const TERMINAL_VELOCITY: f32 = 50.0;
const JUMP_SPEED: f32 = 8.0;

const WATER_MOVE_SPEED: f32 = 3.0;
const WATER_FAST_MOVE_SPEED: f32 = 4.5;

const WATER_GRAVITY: f32 = 4.5;
const WATER_BUOYANCY: f32 = 6.5;

const WATER_VERTICAL_DRAG: f32 = 2.0;

const WATER_SWIM_ACCELERATION: f32 = 12.0;

const WATER_MAX_ASCEND_SPEED: f32 = 4.0;
const WATER_MAX_DESCEND_SPEED: f32 = 4.0;

const WATER_SUBMERSION_THRESHOLD: f32 = 0.05;

const DOUBLE_JUMP_WINDOW: f32 = 0.40;

const THIRD_PERSON_DISTANCE: f32 = 3.5;

const STEP_CAMERA_RECOVERY_SPEED: f32 = 14.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerStance {
    #[default]
    Standing,
    Crouching,
    Crawling,
}

#[derive(Resource, Default)]
pub struct InspectorInteraction {
    pub active: bool,
}

#[derive(Component)]
pub struct PlayerCamera {
    pub sensitivity: f32,

    pub spectator_speed: f32,
    pub spectator_fast_speed: f32,

    pub yaw: f32,
    pub pitch: f32,

    third_person: bool,

    pub current_fov_degrees: f32,
    pub zoom_factor: f32,
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

            current_fov_degrees: 90.0,
            zoom_factor: 0.35,
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

    pub stance: PlayerStance,
    pub current_eye_height: f32,

    pub bob_timer: f32,
    pub bob_offset: Vec3,
    pub bob_roll: f32,
}

impl Default for PlayerMotion {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,

            grounded: false,

            flying: false,

            facing_yaw: 0.0,

            camera_step_offset: 0.0,

            stance: PlayerStance::Standing,
            current_eye_height: PLAYER_EYE_HEIGHT,

            bob_timer: 0.0,
            bob_offset: Vec3::ZERO,
            bob_roll: 0.0,
        }
    }
}

impl PlayerMotion {
    pub fn stance_height(&self) -> f32 {
        match self.stance {
            PlayerStance::Standing => PLAYER_HEIGHT,
            PlayerStance::Crouching => CROUCH_HEIGHT,
            PlayerStance::Crawling => CRAWL_HEIGHT,
        }
    }

    pub fn target_eye_height(&self) -> f32 {
        match self.stance {
            PlayerStance::Standing => PLAYER_EYE_HEIGHT,
            PlayerStance::Crouching => CROUCH_EYE_HEIGHT,
            PlayerStance::Crawling => CRAWL_EYE_HEIGHT,
        }
    }
}

#[derive(Default)]
pub(super) struct JumpTapState {
    since_last_press: Option<f32>,
}

pub(super) fn toggle_inspector_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inspector_interaction: ResMut<InspectorInteraction>,
    cursor_options: Single<&mut CursorOptions>,
) {
    if !keyboard.just_pressed(KeyCode::F1) {
        return;
    }

    inspector_interaction.active = !inspector_interaction.active;

    let mut cursor_options = cursor_options.into_inner();

    if inspector_interaction.active {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;

        info!("Inspector interaction: enabled");
    } else {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;

        info!("Inspector interaction: disabled");
    }
}

pub(super) fn toggle_camera_view(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_mode: Res<GameMode>,
    inspector_interaction: Res<InspectorInteraction>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    camera: Single<&mut PlayerCamera, With<Camera3d>>,
) {
    if inspector_interaction.active
        || menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
    {
        return;
    }

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
    inspector_interaction: Res<InspectorInteraction>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    radial_menu_state: Option<Res<RadialMenuState>>,
    game_settings: Res<GameSettings>,
    camera: Single<(&mut Transform, &mut PlayerCamera), With<Camera3d>>,
) {
    if inspector_interaction.active
        || menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || radial_menu_state.is_some_and(|r| r.is_open)
    {
        return;
    }

    let delta = mouse_motion.delta;

    if delta == Vec2::ZERO {
        return;
    }

    let (mut transform, mut camera) = camera.into_inner();

    let zoom_factor = (camera.current_fov_degrees / game_settings.fov_degrees).clamp(0.05, 1.2);
    let effective_sensitivity = camera.sensitivity * zoom_factor;

    camera.yaw -= delta.x * effective_sensitivity;

    camera.pitch -= delta.y * effective_sensitivity;

    camera.pitch = camera.pitch.clamp(-FRAC_PI_2 + 0.001, FRAC_PI_2 - 0.001);

    let (_, _, current_roll) = transform.rotation.to_euler(EulerRot::YXZ);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, current_roll);
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn creative_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    inspector_interaction: Res<InspectorInteraction>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    game_settings: Res<GameSettings>,
    world: Res<VoxelWorld>,
    player: Single<(&mut Transform, &mut PlayerMotion), With<Player>>,
    camera: Single<
        (&mut Transform, &mut PlayerCamera, &mut Projection),
        (With<Camera3d>, Without<Player>),
    >,
    env_status: Res<PlayerEnvironmentStatus>,
    mut jump_tap: Local<JumpTapState>,
) {
    let is_paused = menu_state.as_ref().is_some_and(|s| {
        *s.get() == crate::menu::MenuState::Pause || *s.get() == crate::menu::MenuState::Settings
    });

    if inspector_interaction.active || is_paused {
        jump_tap.since_last_press = None;

        return;
    }

    if *game_mode != GameMode::Creative {
        jump_tap.since_last_press = None;

        return;
    }

    let is_in_inventory = menu_state
        .as_ref()
        .is_some_and(|s| *s.get() == crate::menu::MenuState::Inventory);

    let delta_seconds = time.delta_secs();

    if let Some(elapsed) = jump_tap.since_last_press.as_mut() {
        *elapsed += delta_seconds;
    }

    let (mut player_transform, mut motion) = player.into_inner();

    let (mut camera_transform, mut camera_controller, camera_projection) = camera.into_inner();

    if !motion.flying {
        motion.grounded = is_grounded(&world, player_transform.translation, motion.stance_height());
    }

    let water_submersion = env_status.submersion;

    let in_water = water_submersion > WATER_SUBMERSION_THRESHOLD;

    // Stance resolution with headroom safety (disabled while in inventory UI)
    if motion.flying {
        motion.stance = PlayerStance::Standing;
    } else if !is_in_inventory {
        let requested_stance = if keyboard.pressed(KeyCode::KeyC) {
            PlayerStance::Crawling
        } else if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight)
        {
            PlayerStance::Crouching
        } else {
            PlayerStance::Standing
        };

        match (motion.stance, requested_stance) {
            (PlayerStance::Crawling, PlayerStance::Standing) => {
                if has_headroom(&world, player_transform.translation, PLAYER_HEIGHT) {
                    motion.stance = PlayerStance::Standing;
                } else if has_headroom(&world, player_transform.translation, CROUCH_HEIGHT) {
                    motion.stance = PlayerStance::Crouching;
                }
            }
            (PlayerStance::Crawling, PlayerStance::Crouching) => {
                if has_headroom(&world, player_transform.translation, CROUCH_HEIGHT) {
                    motion.stance = PlayerStance::Crouching;
                }
            }
            (PlayerStance::Crouching, PlayerStance::Standing) => {
                if has_headroom(&world, player_transform.translation, PLAYER_HEIGHT) {
                    motion.stance = PlayerStance::Standing;
                }
            }
            _ => {
                motion.stance = requested_stance;
            }
        }
    }

    // Smooth eye height interpolation
    let target_eye_height = motion.target_eye_height();
    let eye_smoothing = 1.0 - (-14.0 * delta_seconds).exp();
    motion.current_eye_height += (target_eye_height - motion.current_eye_height) * eye_smoothing;

    if !is_in_inventory && keyboard.just_pressed(KeyCode::Space) {
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

            if motion.grounded && !motion.flying && motion.stance != PlayerStance::Crawling {
                motion.velocity.y = JUMP_SPEED;
                motion.grounded = false;
            }
        }
    }

    let camera_forward = *camera_transform.forward();

    let camera_right = *camera_transform.right();

    let (yaw, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    motion.facing_yaw = yaw;

    let is_fast = !is_in_inventory
        && (keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight));

    let movement_keyboard = if is_in_inventory {
        None
    } else {
        Some(&*keyboard)
    };

    if motion.flying {
        fly_movement(
            movement_keyboard,
            delta_seconds,
            &world,
            &mut player_transform,
            &mut motion,
            camera_forward,
            camera_right,
            is_fast,
        );
    } else {
        grounded_or_water_movement(
            movement_keyboard,
            delta_seconds,
            &world,
            &mut player_transform,
            &mut motion,
            camera_forward,
            camera_right,
            is_fast,
            water_submersion,
            in_water,
        );
    }

    update_camera_step_offset(&mut motion, delta_seconds);

    // View bobbing calculation
    let horiz_speed = motion.velocity.x.hypot(motion.velocity.z);
    let is_moving = horiz_speed > 0.1;

    if motion.grounded && !motion.flying && is_moving && game_settings.view_bobbing {
        let bob_frequency = if is_fast { 14.0 } else { 10.0 };
        motion.bob_timer += delta_seconds * bob_frequency;

        let intensity = (horiz_speed / SPRINT_SPEED).clamp(0.0, 1.0);
        let bob_x = (motion.bob_timer * 0.5).sin() * 0.035 * intensity;
        let bob_y = motion.bob_timer.sin().abs() * 0.025 * intensity;
        let target_bob = Vec3::new(bob_x, -bob_y, 0.0);
        let target_roll = (motion.bob_timer * 0.5).sin() * 0.008 * intensity;

        let bob_smooth = 1.0 - (-12.0 * delta_seconds).exp();
        let current_bob = motion.bob_offset;
        let current_roll = motion.bob_roll;
        motion.bob_offset += (target_bob - current_bob) * bob_smooth;
        motion.bob_roll += (target_roll - current_roll) * bob_smooth;
    } else {
        let bob_smooth = 1.0 - (-10.0 * delta_seconds).exp();
        let current_bob = motion.bob_offset;
        let current_roll = motion.bob_roll;
        motion.bob_offset += (Vec3::ZERO - current_bob) * bob_smooth;
        motion.bob_roll += (0.0 - current_roll) * bob_smooth;
    }

    // Camera zoom (Z key) & Dynamic FOV Kick
    let is_zooming = !is_in_inventory
        && keyboard.pressed(KeyCode::KeyZ)
        && menu_state
            .as_ref()
            .is_none_or(|s| *s.get() == crate::menu::MenuState::None);

    let is_sprinting = is_fast && (is_moving || motion.flying);

    let mut target_fov = game_settings.fov_degrees;
    if is_zooming {
        if keyboard.just_pressed(KeyCode::KeyZ) {
            camera_controller.zoom_factor = 0.35;
        }

        let scroll = mouse_scroll.delta.y;
        if scroll.abs() > 0.001 {
            // Scroll UP (positive) zooms in closer (decreases zoom factor / FOV)
            // Scroll DOWN (negative) zooms out (increases zoom factor / FOV)
            camera_controller.zoom_factor =
                (camera_controller.zoom_factor - scroll * 0.05).clamp(0.05, 0.85);
        }

        target_fov = (game_settings.fov_degrees * camera_controller.zoom_factor).clamp(4.0, 85.0);
    } else if is_sprinting {
        target_fov += 8.0;
    }

    let fov_smooth = 1.0 - (-14.0 * delta_seconds).exp();
    camera_controller.current_fov_degrees +=
        (target_fov - camera_controller.current_fov_degrees) * fov_smooth;

    if let Projection::Perspective(ref mut perspective) = *camera_projection.into_inner() {
        perspective.fov = camera_controller.current_fov_degrees.to_radians();
    }

    let eye_position = player_transform.translation
        + Vec3::Y * (motion.current_eye_height + motion.camera_step_offset)
        + motion.bob_offset;

    if camera_controller.is_third_person() {
        let actual_distance = resolve_third_person_camera_distance(
            &world,
            eye_position,
            camera_forward,
            THIRD_PERSON_DISTANCE,
        );
        camera_transform.translation = eye_position - camera_forward * actual_distance;
    } else {
        let forward_horiz = Vec3::new(
            -camera_controller.yaw.sin(),
            0.0,
            -camera_controller.yaw.cos(),
        )
        .normalize_or_zero();
        let eye_forward_offset = match motion.stance {
            PlayerStance::Standing => 0.16,
            PlayerStance::Crouching => 0.10,
            PlayerStance::Crawling => 0.18,
        };
        camera_transform.translation = eye_position + forward_horiz * eye_forward_offset;
    }

    camera_transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        camera_controller.yaw,
        camera_controller.pitch,
        motion.bob_roll,
    );
}

fn resolve_third_person_camera_distance(
    world: &VoxelWorld,
    eye_position: Vec3,
    camera_forward: Vec3,
    max_distance: f32,
) -> f32 {
    let camera_dir = -camera_forward;
    let camera_margin = 0.25; // 25 cm cushion from any solid block
    let step = 0.05; // 5 cm ray step
    let mut dist = 0.0;

    while dist < max_distance {
        dist += step;
        let test_pos = eye_position + camera_dir * dist;

        // Check if test_pos or near box overlaps any solid collidable voxel
        let min_p = test_pos - Vec3::splat(camera_margin);
        let max_p = test_pos + Vec3::splat(camera_margin);

        let min_vx = (min_p.x / VOXEL_SIZE).floor() as i32;
        let max_vx = (max_p.x / VOXEL_SIZE).floor() as i32;
        let min_vy = (min_p.y / VOXEL_SIZE).floor() as i32;
        let max_vy = (max_p.y / VOXEL_SIZE).floor() as i32;
        let min_vz = (min_p.z / VOXEL_SIZE).floor() as i32;
        let max_vz = (max_p.z / VOXEL_SIZE).floor() as i32;

        let mut hit = false;
        for y in min_vy..=max_vy {
            for z in min_vz..=max_vz {
                for x in min_vx..=max_vx {
                    if world
                        .get_voxel(IVec3::new(x, y, z))
                        .is_some_and(Voxel::is_collidable)
                    {
                        hit = true;
                        break;
                    }
                }
                if hit {
                    break;
                }
            }
            if hit {
                break;
            }
        }

        if hit {
            return (dist - step - 0.05).max(0.20);
        }
    }

    max_distance
}

#[allow(clippy::too_many_arguments)]
fn fly_movement(
    keyboard: Option<&ButtonInput<KeyCode>>,
    delta_seconds: f32,
    world: &VoxelWorld,
    player_transform: &mut Transform,
    motion: &mut PlayerMotion,
    camera_forward: Vec3,
    camera_right: Vec3,
    is_fast: bool,
) {
    let mut direction = Vec3::ZERO;

    if let Some(keyboard) = keyboard {
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
    }

    let speed = if is_fast { FAST_FLY_SPEED } else { FLY_SPEED };

    let movement = if direction == Vec3::ZERO {
        Vec3::ZERO
    } else {
        direction.normalize() * speed * delta_seconds
    };

    let moving_vertically = keyboard.is_some_and(|k| {
        k.pressed(KeyCode::Space)
            || k.pressed(KeyCode::ControlLeft)
            || k.pressed(KeyCode::ControlRight)
    });

    let allow_step = !moving_vertically;

    let (new_position, collision) = move_with_collisions(
        world,
        player_transform.translation,
        movement,
        allow_step,
        motion.stance_height(),
    );

    player_transform.translation = new_position;

    motion.velocity = Vec3::ZERO;

    motion.grounded = collision.grounded;

    if collision.step_height > 0.0 {
        motion.camera_step_offset -= collision.step_height;
    }
}

#[allow(clippy::too_many_arguments)]
fn grounded_or_water_movement(
    keyboard: Option<&ButtonInput<KeyCode>>,
    delta_seconds: f32,
    world: &VoxelWorld,
    player_transform: &mut Transform,
    motion: &mut PlayerMotion,
    camera_forward: Vec3,
    camera_right: Vec3,
    is_fast: bool,
    water_submersion: f32,
    in_water: bool,
) {
    let mut forward = Vec3::new(camera_forward.x, 0.0, camera_forward.z);

    let mut right = Vec3::new(camera_right.x, 0.0, camera_right.z);

    if forward.length_squared() > 0.0 {
        forward = forward.normalize();
    }

    if right.length_squared() > 0.0 {
        right = right.normalize();
    }

    let mut direction = Vec3::ZERO;

    if let Some(keyboard) = keyboard {
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
    }

    if direction != Vec3::ZERO {
        direction = direction.normalize();
    }

    let speed = if in_water {
        if is_fast {
            WATER_FAST_MOVE_SPEED
        } else {
            WATER_MOVE_SPEED
        }
    } else {
        match motion.stance {
            PlayerStance::Standing => {
                if is_fast {
                    SPRINT_SPEED
                } else {
                    WALK_SPEED
                }
            }
            PlayerStance::Crouching => {
                if is_fast {
                    CROUCH_SPEED * 1.3
                } else {
                    CROUCH_SPEED
                }
            }
            PlayerStance::Crawling => CRAWL_SPEED,
        }
    };

    motion.velocity.x = direction.x * speed;

    motion.velocity.z = direction.z * speed;

    if in_water {
        update_water_vertical_velocity(keyboard, delta_seconds, motion, water_submersion);
    } else {
        motion.velocity.y -= GRAVITY * delta_seconds;

        motion.velocity.y = motion.velocity.y.max(-TERMINAL_VELOCITY);
    }

    // Ledge clamping when crouching on ground
    if motion.stance == PlayerStance::Crouching && motion.grounded && !in_water {
        let mut move_x = motion.velocity.x * delta_seconds;
        let mut move_z = motion.velocity.z * delta_seconds;
        let height = motion.stance_height();
        let pos = player_transform.translation;

        if move_x != 0.0 {
            let cand_x = pos + Vec3::new(move_x, 0.0, 0.0);
            if !is_grounded(world, cand_x, height) {
                move_x = 0.0;
            }
        }
        if move_z != 0.0 {
            let cand_z = pos + Vec3::new(0.0, 0.0, move_z);
            if !is_grounded(world, cand_z, height) {
                move_z = 0.0;
            }
        }
        if move_x != 0.0 && move_z != 0.0 {
            let cand_both = pos + Vec3::new(move_x, 0.0, move_z);
            if !is_grounded(world, cand_both, height) {
                move_x = 0.0;
                move_z = 0.0;
            }
        }
        motion.velocity.x = if delta_seconds > 0.0 {
            move_x / delta_seconds
        } else {
            0.0
        };
        motion.velocity.z = if delta_seconds > 0.0 {
            move_z / delta_seconds
        } else {
            0.0
        };
    }

    let movement = motion.velocity * delta_seconds;

    let allow_step = motion.grounded && motion.velocity.y <= 0.0;

    let (new_position, collision) = move_with_collisions(
        world,
        player_transform.translation,
        movement,
        allow_step,
        motion.stance_height(),
    );

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
        motion.camera_step_offset -= collision.step_height;
    }
}

fn update_water_vertical_velocity(
    keyboard: Option<&ButtonInput<KeyCode>>,
    delta_seconds: f32,
    motion: &mut PlayerMotion,
    water_submersion: f32,
) {
    motion.velocity.y -= WATER_GRAVITY * delta_seconds;

    motion.velocity.y += WATER_BUOYANCY * water_submersion * delta_seconds;

    let drag = (-WATER_VERTICAL_DRAG * delta_seconds).exp();

    motion.velocity.y *= drag;

    if let Some(keyboard) = keyboard {
        if keyboard.pressed(KeyCode::Space) {
            motion.velocity.y += WATER_SWIM_ACCELERATION * delta_seconds;
        }

        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            motion.velocity.y -= WATER_SWIM_ACCELERATION * delta_seconds;
        }
    }

    motion.velocity.y = motion
        .velocity
        .y
        .clamp(-WATER_MAX_DESCEND_SPEED, WATER_MAX_ASCEND_SPEED);
}

fn update_camera_step_offset(motion: &mut PlayerMotion, delta_seconds: f32) {
    let smoothing = 1.0 - (-STEP_CAMERA_RECOVERY_SPEED * delta_seconds).exp();

    motion.camera_step_offset += (0.0 - motion.camera_step_offset) * smoothing;

    if motion.camera_step_offset.abs() < 0.001 {
        motion.camera_step_offset = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_stance_heights_and_eye_heights_are_consistent() {
        let mut motion = PlayerMotion::default();
        assert_eq!(motion.stance, PlayerStance::Standing);
        assert_eq!(motion.stance_height(), PLAYER_HEIGHT);
        assert_eq!(motion.target_eye_height(), PLAYER_EYE_HEIGHT);

        motion.stance = PlayerStance::Crouching;
        assert_eq!(motion.stance_height(), CROUCH_HEIGHT);
        assert_eq!(motion.target_eye_height(), CROUCH_EYE_HEIGHT);

        motion.stance = PlayerStance::Crawling;
        assert_eq!(motion.stance_height(), CRAWL_HEIGHT);
        assert_eq!(motion.target_eye_height(), CRAWL_EYE_HEIGHT);
    }

    #[test]
    fn crawl_height_fits_in_single_voxel() {
        // Single voxel is 0.5m tall. Crawl height must be strictly less than 0.50m.
        assert!(CRAWL_HEIGHT < 0.50);
        assert!(CRAWL_HEIGHT > 0.30);
    }

    #[test]
    fn player_camera_defaults_and_zoom_factor() {
        let transform = Transform::IDENTITY;
        let cam = PlayerCamera::from_transform(&transform);
        assert_eq!(cam.current_fov_degrees, 90.0);
        assert!((cam.zoom_factor - 0.35).abs() < 1e-5);

        // Zoom in step (scroll up)
        let zoomed_in = (cam.zoom_factor - 1.0 * 0.05).clamp(0.05, 0.85);
        assert!((zoomed_in - 0.30).abs() < 1e-5);

        // Zoom out step (scroll down)
        let zoomed_out = (cam.zoom_factor - (-1.0) * 0.05).clamp(0.05, 0.85);
        assert!((zoomed_out - 0.40).abs() < 1e-5);
    }
}
