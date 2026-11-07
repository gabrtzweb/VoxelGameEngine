use std::f32::consts::PI;

use bevy::{
    asset::RenderAssetUsages,
    light::NotShadowCaster,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::{
    gameplay::{
        SelectedVoxel, VoxelItemMeshCache,
        feedback::{BlockBreakEvent, BlockPlaceEvent},
    },
    menu::MenuState,
    meshing::{ChunkMaterial, OpaqueChunkMaterial},
    world::Voxel,
};

use super::{
    CharacterRace, GameMode, InspectorInteraction, PLAYER_HEIGHT, Player, PlayerEnvironmentStatus,
    PlayerStance, SelectedRace,
    controller::{PlayerCamera, PlayerMotion},
    hotbar::Hotbar,
};

pub(crate) const PIXEL_SCALE: f32 = PLAYER_HEIGHT / 32.0; // 0.05625m per pixel -> 32 pixels = 1.8m

#[derive(Component)]
pub struct PlayerModelRoot;

#[derive(Component)]
pub struct PlayerHead;

#[derive(Component)]
pub struct PlayerTorso;

#[derive(Component)]
pub struct PlayerLeftArm;

#[derive(Component)]
pub struct PlayerRightArm;

#[derive(Component)]
pub struct PlayerLeftLeg;

#[derive(Component)]
pub struct PlayerRightLeg;

#[derive(Component)]
pub struct PlayerThirdPersonHeldItem;

#[derive(Component)]
pub struct PlayerFirstPersonRig;

#[derive(Component)]
pub struct PlayerFirstPersonArm;

#[derive(Component)]
pub struct PlayerFirstPersonHeldItem;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerLimb {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

#[derive(Component, Default)]
pub struct PlayerAnimState {
    pub walk_cycle: f32,
    pub swim_cycle: f32,
    pub body_yaw: f32,
}

#[derive(Resource)]
pub struct PlayerHandAnimState {
    /// Mining / attack swing progress [0.0 .. 1.0], 0.0 = idle
    pub swing_timer: f32,
    pub is_swinging: bool,
    /// Placement dip/recoil progress [0.0 .. 1.0]
    pub place_recoil: f32,
    /// Equip raise progress [0.0 .. 1.0], 1.0 = fully raised
    pub equip_progress: f32,
    pub last_voxel: Option<Voxel>,
    pub last_slot: usize,
    pub displayed_voxel: Option<Voxel>,
    pub mesh_initialized: bool,
}

impl Default for PlayerHandAnimState {
    fn default() -> Self {
        Self {
            swing_timer: 0.0,
            is_swinging: false,
            place_recoil: 0.0,
            equip_progress: 1.0,
            last_voxel: None,
            last_slot: 0,
            displayed_voxel: None,
            mesh_initialized: false,
        }
    }
}

#[derive(Resource)]
pub struct PlayerRaceMaterials {
    pub skin: Handle<StandardMaterial>,
    pub torso: Handle<StandardMaterial>,
    pub arms: Handle<StandardMaterial>,
    pub legs: Handle<StandardMaterial>,
}

pub struct PlayerModelPlugin;

impl Plugin for PlayerModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerHandAnimState>()
            .add_observer(on_block_break_hand_swing)
            .add_observer(on_block_place_hand_recoil)
            .add_systems(Startup, setup_player_model)
            .add_systems(
                Update,
                (
                    sync_player_race_materials,
                    trigger_hand_swing_on_click,
                    sync_held_item_meshes,
                    update_player_model.after(crate::player::PlayerSet::Movement),
                )
                    .chain(),
            );
    }
}

fn on_block_break_hand_swing(
    _event: On<BlockBreakEvent>,
    mut hand_anim: ResMut<PlayerHandAnimState>,
) {
    hand_anim.is_swinging = true;
    hand_anim.swing_timer = 0.0;
}

fn on_block_place_hand_recoil(
    _event: On<BlockPlaceEvent>,
    mut hand_anim: ResMut<PlayerHandAnimState>,
) {
    hand_anim.place_recoil = 1.0;
    if !hand_anim.is_swinging {
        hand_anim.is_swinging = true;
        hand_anim.swing_timer = 0.0;
    }
}

fn trigger_hand_swing_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    control_settings: Option<Res<crate::core::ControlSettings>>,
    menu_state: Option<Res<State<MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    mut hand_anim: ResMut<PlayerHandAnimState>,
) {
    if menu_state.is_some_and(|s| *s.get() != MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }

    let primary_just_pressed = control_settings.as_ref().map_or(
        mouse.just_pressed(MouseButton::Left),
        |c| c.is_action_just_pressed(crate::core::InputAction::PrimaryAction, &keyboard, &mouse),
    );
    let primary_held = control_settings.as_ref().map_or(
        mouse.pressed(MouseButton::Left),
        |c| c.is_action_pressed(crate::core::InputAction::PrimaryAction, &keyboard, &mouse),
    );

    if primary_just_pressed || (primary_held && !hand_anim.is_swinging) {
        hand_anim.is_swinging = true;
        hand_anim.swing_timer = 0.0;
    }
}

pub fn setup_player_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected_race: Option<Res<SelectedRace>>,
) {
    let race = selected_race.map_or(CharacterRace::Human, |r| r.0);

    let skin_mat = materials.add(StandardMaterial {
        base_color: race.skin_color(),
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        cull_mode: None,
        ..default()
    });
    let torso_mat = materials.add(StandardMaterial {
        base_color: race.torso_color(),
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        cull_mode: None,
        ..default()
    });
    let arms_mat = materials.add(StandardMaterial {
        base_color: race.arms_color(),
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        cull_mode: None,
        ..default()
    });
    let legs_mat = materials.add(StandardMaterial {
        base_color: race.legs_color(),
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        cull_mode: None,
        ..default()
    });

    commands.insert_resource(PlayerRaceMaterials {
        skin: skin_mat.clone(),
        torso: torso_mat.clone(),
        arms: arms_mat.clone(),
        legs: legs_mat.clone(),
    });

    let head_mesh = meshes.add(build_head_mesh(race));
    let torso_mesh = meshes.add(build_torso_mesh(race));
    let left_arm_mesh = meshes.add(build_left_arm_mesh());
    let right_arm_mesh = meshes.add(build_right_arm_mesh());
    let left_leg_mesh = meshes.add(build_left_leg_mesh());
    let right_leg_mesh = meshes.add(build_right_leg_mesh());
    let dummy_item_mesh = meshes.add(build_left_arm_mesh());

    // Root entity at player origin (feet at y=0)
    let root = commands
        .spawn((
            PlayerModelRoot,
            PlayerAnimState::default(),
            Transform::default(),
            Visibility::Inherited,
        ))
        .id();

    // Torso (waist pivot at y = 0.675)
    let torso = commands
        .spawn((
            PlayerTorso,
            PlayerLimb::Torso,
            Mesh3d(torso_mesh),
            MeshMaterial3d(torso_mat),
            Transform::from_translation(Vec3::new(0.0, 12.0 * PIXEL_SCALE, 0.0)),
            Visibility::Inherited,
        ))
        .id();

    // Head (neck pivot at top of torso: local y = 12 * PIXEL_SCALE)
    let head = commands
        .spawn((
            PlayerHead,
            PlayerLimb::Head,
            Mesh3d(head_mesh),
            MeshMaterial3d(skin_mat),
            Transform::from_translation(Vec3::new(0.0, 12.0 * PIXEL_SCALE, 0.0)),
            Visibility::Hidden, // Hidden in 1st person, visible in 3rd person
        ))
        .id();

    // Left Arm (shoulder pivot at top left of torso: local x = +6 * PIXEL_SCALE, y = 12 * PIXEL_SCALE)
    let left_arm = commands
        .spawn((
            PlayerLeftArm,
            PlayerLimb::LeftArm,
            Mesh3d(left_arm_mesh),
            MeshMaterial3d(arms_mat.clone()),
            Transform::from_translation(Vec3::new(6.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 0.0)),
            Visibility::Inherited,
        ))
        .id();

    // Right Arm (shoulder pivot at top right of torso: local x = -6 * PIXEL_SCALE, y = 12 * PIXEL_SCALE)
    let right_arm = commands
        .spawn((
            PlayerRightArm,
            PlayerLimb::RightArm,
            NotShadowCaster,
            Mesh3d(right_arm_mesh),
            MeshMaterial3d(arms_mat),
            Transform::from_translation(Vec3::new(-6.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 0.0)),
            Visibility::Inherited,
        ))
        .id();

    // Held item attached directly to the tip of the player's right arm (used in both 1st and 3rd person)
    let tp_held_item = commands
        .spawn((
            PlayerThirdPersonHeldItem,
            NotShadowCaster,
            Mesh3d(dummy_item_mesh),
            Transform::from_translation(Vec3::new(0.0, -10.5 * PIXEL_SCALE, 2.2 * PIXEL_SCALE))
                .with_rotation(Quat::from_euler(EulerRot::YXZ, 0.35, 0.75, 0.0))
                .with_scale(Vec3::splat(0.72)),
            Visibility::Hidden,
        ))
        .id();
    commands.entity(right_arm).add_child(tp_held_item);

    // Left Leg (hip pivot at y = 12 * PIXEL_SCALE, x = +2 * PIXEL_SCALE)
    let left_leg = commands
        .spawn((
            PlayerLeftLeg,
            PlayerLimb::LeftLeg,
            Mesh3d(left_leg_mesh),
            MeshMaterial3d(legs_mat.clone()),
            Transform::from_translation(Vec3::new(2.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 0.0)),
            Visibility::Inherited,
        ))
        .id();

    // Right Leg (hip pivot at y = 12 * PIXEL_SCALE, x = -2 * PIXEL_SCALE)
    let right_leg = commands
        .spawn((
            PlayerRightLeg,
            PlayerLimb::RightLeg,
            Mesh3d(right_leg_mesh),
            MeshMaterial3d(legs_mat),
            Transform::from_translation(Vec3::new(-2.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 0.0)),
            Visibility::Inherited,
        ))
        .id();

    // Build hierarchy:
    // root -> [torso, left_leg, right_leg]
    // torso -> [head, left_arm, right_arm]
    commands
        .entity(torso)
        .add_children(&[head, left_arm, right_arm]);
    commands
        .entity(root)
        .add_children(&[torso, left_leg, right_leg]);
}

pub fn sync_player_race_materials(
    selected_race: Option<Res<SelectedRace>>,
    race_mats: Option<Res<PlayerRaceMaterials>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut head_query: Query<&mut Mesh3d, (With<PlayerHead>, Without<PlayerTorso>)>,
    mut torso_query: Query<&mut Mesh3d, (With<PlayerTorso>, Without<PlayerHead>)>,
) {
    let Some(selected_race) = selected_race else {
        return;
    };
    if !selected_race.is_changed() {
        return;
    }
    let Some(mats) = race_mats else {
        return;
    };
    let race = selected_race.0;

    if let Some(mut mat) = materials.get_mut(&mats.skin) {
        mat.base_color = race.skin_color();
    }
    if let Some(mut mat) = materials.get_mut(&mats.torso) {
        mat.base_color = race.torso_color();
    }
    if let Some(mut mat) = materials.get_mut(&mats.arms) {
        mat.base_color = race.arms_color();
    }
    if let Some(mut mat) = materials.get_mut(&mats.legs) {
        mat.base_color = race.legs_color();
    }

    // Rebuild race-specific silhouette meshes (ears, horns, snout, beard, tail, pauldrons)
    let new_head = meshes.add(build_head_mesh(race));
    for mut mesh3d in &mut head_query {
        mesh3d.0 = new_head.clone();
    }

    let new_torso = meshes.add(build_torso_mesh(race));
    for mut mesh3d in &mut torso_query {
        mesh3d.0 = new_torso.clone();
    }
}

fn sync_held_item_meshes(
    mut commands: Commands,
    selected_voxel: Option<Res<SelectedVoxel>>,
    hotbar: Option<Res<Hotbar>>,
    chunk_material: Option<Res<ChunkMaterial>>,
    mut item_mesh_cache: Option<ResMut<VoxelItemMeshCache>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut hand_anim: ResMut<PlayerHandAnimState>,
    mut fp_item_query: Query<
        (
            Entity,
            &mut Mesh3d,
            Option<&MeshMaterial3d<OpaqueChunkMaterial>>,
        ),
        (
            With<PlayerFirstPersonHeldItem>,
            Without<PlayerThirdPersonHeldItem>,
        ),
    >,
    mut tp_item_query: Query<
        (
            Entity,
            &mut Mesh3d,
            Option<&MeshMaterial3d<OpaqueChunkMaterial>>,
        ),
        (
            With<PlayerThirdPersonHeldItem>,
            Without<PlayerFirstPersonHeldItem>,
        ),
    >,
) {
    let current_voxel = selected_voxel.and_then(|s| s.0).filter(|v| !v.is_empty());
    let current_slot = hotbar.as_ref().map_or(0, |h| h.active_slot);

    // Detect slot or item change to trigger smooth equip raise animation
    if current_voxel != hand_anim.last_voxel || current_slot != hand_anim.last_slot {
        hand_anim.last_voxel = current_voxel;
        hand_anim.last_slot = current_slot;
        hand_anim.equip_progress = 0.0;
    }

    let Some(chunk_material) = chunk_material else {
        return;
    };
    let Some(ref mut cache) = item_mesh_cache else {
        return;
    };

    if hand_anim.displayed_voxel != current_voxel || !hand_anim.mesh_initialized {
        hand_anim.displayed_voxel = current_voxel;
        hand_anim.mesh_initialized = true;

        if let Some(voxel) = current_voxel {
            let mesh_handle = cache.get_or_create_held_mesh(voxel, &mut meshes, &chunk_material);
            let mat_handle = chunk_material.opaque.clone();

            for (entity, mut mesh3d, has_mat) in &mut fp_item_query {
                mesh3d.0 = mesh_handle.clone();
                if has_mat.is_none() {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial3d(mat_handle.clone()));
                }
            }

            for (entity, mut mesh3d, has_mat) in &mut tp_item_query {
                mesh3d.0 = mesh_handle.clone();
                if has_mat.is_none() {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial3d(mat_handle.clone()));
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_player_model(
    time: Res<Time>,
    env_status: Res<PlayerEnvironmentStatus>,
    game_mode: Res<GameMode>,
    selected_race: Option<Res<SelectedRace>>,
    menu_state: Option<Res<State<MenuState>>>,
    mut hand_anim: ResMut<PlayerHandAnimState>,
    player: Single<(&Transform, &PlayerMotion), With<Player>>,
    camera: Single<(&PlayerCamera, &Transform), (With<Camera3d>, Without<Player>)>,
    model_root: Single<
        (&mut Transform, &mut PlayerAnimState),
        (
            With<PlayerModelRoot>,
            Without<Player>,
            Without<Camera3d>,
            Without<PlayerLimb>,
            Without<PlayerFirstPersonRig>,
        ),
    >,
    mut limbs_query: Query<
        (&PlayerLimb, &mut Transform, Option<&mut Visibility>),
        (
            Without<PlayerModelRoot>,
            Without<Player>,
            Without<Camera3d>,
            Without<PlayerFirstPersonRig>,
            Without<PlayerFirstPersonArm>,
            Without<PlayerFirstPersonHeldItem>,
            Without<PlayerThirdPersonHeldItem>,
        ),
    >,
    mut fp_rig_query: Query<
        (&mut Transform, &mut Visibility),
        (
            With<PlayerFirstPersonRig>,
            Without<PlayerModelRoot>,
            Without<Player>,
            Without<Camera3d>,
            Without<PlayerLimb>,
        ),
    >,
    mut fp_arm_query: Query<
        (&mut Transform, &mut Visibility),
        (
            With<PlayerFirstPersonArm>,
            Without<PlayerFirstPersonRig>,
            Without<PlayerFirstPersonHeldItem>,
            Without<PlayerThirdPersonHeldItem>,
            Without<PlayerLimb>,
            Without<PlayerModelRoot>,
            Without<Player>,
            Without<Camera3d>,
        ),
    >,
    mut fp_item_query: Query<
        (&mut Transform, &mut Visibility),
        (
            With<PlayerFirstPersonHeldItem>,
            Without<PlayerFirstPersonRig>,
            Without<PlayerFirstPersonArm>,
            Without<PlayerThirdPersonHeldItem>,
            Without<PlayerLimb>,
            Without<PlayerModelRoot>,
            Without<Player>,
            Without<Camera3d>,
        ),
    >,
    mut tp_item_query: Query<
        (&mut Transform, &mut Visibility),
        (
            With<PlayerThirdPersonHeldItem>,
            Without<PlayerFirstPersonRig>,
            Without<PlayerFirstPersonArm>,
            Without<PlayerFirstPersonHeldItem>,
            Without<PlayerLimb>,
            Without<PlayerModelRoot>,
            Without<Player>,
            Without<Camera3d>,
        ),
    >,
) {
    let (player_transform, motion) = player.into_inner();
    let (camera_controller, camera_transform) = camera.into_inner();
    let (mut root_transform, mut anim_state) = model_root.into_inner();

    let race = selected_race.map_or(CharacterRace::Human, |r| r.0);
    let props = race.proportions();

    let current_menu = menu_state.as_ref().map(|s| *s.get());
    let is_paused = matches!(
        current_menu,
        Some(
            MenuState::Pause
                | MenuState::Settings
                | MenuState::Controls
                | MenuState::MainMenu
                | MenuState::RaceSelection
                | MenuState::Loading
        )
    );
    let in_pregame_menu = matches!(
        current_menu,
        Some(MenuState::MainMenu | MenuState::RaceSelection | MenuState::Loading)
    );

    let delta_seconds = if is_paused { 0.0 } else { time.delta_secs() };

    // Advance hand swing, placement recoil, and equip raise animations
    if hand_anim.is_swinging {
        hand_anim.swing_timer += delta_seconds * 4.2;
        if hand_anim.swing_timer >= 1.0 {
            hand_anim.swing_timer = 0.0;
            hand_anim.is_swinging = false;
        }
    }
    if hand_anim.place_recoil > 0.0 {
        hand_anim.place_recoil = (hand_anim.place_recoil - delta_seconds * 5.5).max(0.0);
    }
    if hand_anim.equip_progress < 1.0 {
        hand_anim.equip_progress = (hand_anim.equip_progress + delta_seconds * 6.0).min(1.0);
    }

    let swing_phase = if hand_anim.is_swinging {
        (hand_anim.swing_timer * PI).sin()
    } else {
        0.0
    };

    // 1. First-Person vs Third-Person Visibility:
    let is_third_person = camera_controller.is_third_person();
    let is_spectator = *game_mode == GameMode::Spectator;
    let should_show_head = is_spectator || is_third_person;
    let has_held_item = hand_anim.displayed_voxel.is_some();

    // Unused legacy first-person rig queries remain hidden
    for (_, mut fp_vis) in &mut fp_rig_query {
        *fp_vis = Visibility::Hidden;
    }
    for (_, mut arm_vis) in &mut fp_arm_query {
        *arm_vis = Visibility::Hidden;
    }
    for (_, mut item_vis) in &mut fp_item_query {
        *item_vis = Visibility::Hidden;
    }

    // Update the held item attached to the player's real right arm (visible in both 1st and 3rd person!)
    for (mut tp_tf, mut tp_vis) in &mut tp_item_query {
        let target = if !is_spectator && !in_pregame_menu && has_held_item {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *tp_vis != target {
            *tp_vis = target;
        }
        if !is_spectator && !in_pregame_menu && has_held_item {
            tp_tf.translation = Vec3::new(0.0, -10.2 * PIXEL_SCALE, 2.6 * PIXEL_SCALE);
            tp_tf.rotation = Quat::from_euler(EulerRot::YXZ, 0.25, 0.55, 0.0);
            tp_tf.scale = Vec3::splat(0.82);
        }
    }

    // 2. Body Yaw Alignment:
    let (_, camera_pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
    let camera_yaw = camera_controller.yaw;

    let h_vel = Vec2::new(motion.velocity.x, motion.velocity.z);
    let h_speed = h_vel.length();

    if !is_third_person {
        // In first person, body faces directly along camera yaw so looking down shows body centered
        anim_state.body_yaw = camera_yaw;
    } else if h_speed > 0.15 {
        // In third person, body turns smoothly toward movement heading or camera yaw
        let target_yaw = (-motion.velocity.x).atan2(-motion.velocity.z);
        let diff = (target_yaw - anim_state.body_yaw).rem_euclid(2.0 * PI);
        let shortest = if diff > PI { diff - 2.0 * PI } else { diff };
        anim_state.body_yaw += shortest * (1.0 - (-10.0 * delta_seconds).exp());
    } else {
        // When stationary, keep body aligned within 65 degrees of camera yaw
        let diff = (camera_yaw - anim_state.body_yaw).rem_euclid(2.0 * PI);
        let shortest = if diff > PI { diff - 2.0 * PI } else { diff };
        if shortest.abs() > 1.15 {
            let adjust = shortest - 1.15 * shortest.signum();
            anim_state.body_yaw += adjust * (1.0 - (-8.0 * delta_seconds).exp());
        }
    }

    // Position root at player feet, apply race silhouette scale, and align with world heading
    root_transform.translation = player_transform.translation;
    root_transform.rotation = Quat::from_rotation_y(anim_state.body_yaw + PI);
    root_transform.scale = props.root_scale;

    let water_submersion = env_status.submersion;
    let in_deep_water = water_submersion > 0.45;
    let is_swimming = in_deep_water
        && (!motion.grounded || h_speed > 1.2 || motion.velocity.y.abs() > 0.3 || motion.flying);

    // 3. Stance Offsets & Base Postures:
    let leg_height = 12.0 * PIXEL_SCALE * props.leg_scale.y;
    let (torso_trans, torso_base_rot, leg_y, leg_z, base_head_pitch_offset) = if is_swimming {
        let swim_y = (motion.current_eye_height - 3.0 * PIXEL_SCALE) / props.root_scale.y;
        (
            Vec3::new(0.0, swim_y, -12.0 * PIXEL_SCALE),
            Quat::from_rotation_x(1.50),
            swim_y,
            -12.0 * PIXEL_SCALE,
            -1.35,
        )
    } else {
        match motion.stance {
            PlayerStance::Standing => (
                Vec3::new(0.0, leg_height, 0.0),
                Quat::IDENTITY,
                leg_height,
                0.0,
                0.0,
            ),
            PlayerStance::Crouching => (
                Vec3::new(0.0, 7.0 * PIXEL_SCALE * props.leg_scale.y, -2.0 * PIXEL_SCALE),
                Quat::from_rotation_x(0.22),
                7.0 * PIXEL_SCALE * props.leg_scale.y,
                -2.0 * PIXEL_SCALE,
                -0.22,
            ),
            PlayerStance::Crawling => {
                root_transform.translation += Vec3::Y * 0.05;
                (
                    Vec3::new(0.0, 3.5 * PIXEL_SCALE, -12.0 * PIXEL_SCALE),
                    Quat::from_rotation_x(1.52),
                    3.5 * PIXEL_SCALE,
                    -12.0 * PIXEL_SCALE,
                    -1.35,
                )
            }
        }
    };

    // 4. Locomotion / Procedural Cycles:
    let breathe = (time.elapsed_secs() * 2.0).sin() * 0.02;

    if is_swimming {
        anim_state.swim_cycle += delta_seconds * 4.0;
    } else if motion.stance == PlayerStance::Crawling && h_speed > 0.05 {
        anim_state.walk_cycle += h_speed * delta_seconds * 2.2;
    } else if h_speed > 0.05 {
        anim_state.walk_cycle += h_speed * delta_seconds * 1.7;
    }

    let leg_angle = (anim_state.walk_cycle * 2.0).sin() * 0.65;
    let arm_angle = -(anim_state.walk_cycle * 2.0).sin() * 0.65;
    let swim_stroke = (anim_state.swim_cycle).sin() * 0.7;
    let swim_kick = (anim_state.swim_cycle * 2.0).sin() * 0.45;
    let crawl_swing = (anim_state.walk_cycle * 1.8).sin() * 0.55;

    let crouch_bend = if motion.stance == PlayerStance::Crouching {
        -0.45
    } else {
        0.0
    };

    // Flight animation variables
    let is_flying_fast = motion.flying && h_speed > 5.0;
    let is_flying_up = motion.flying && motion.velocity.y > 1.0;
    let is_flying_down = motion.flying && motion.velocity.y < -1.0;
    let hover = (time.elapsed_secs() * 3.0).sin() * 0.08;

    let shoulder_x = 6.0 * PIXEL_SCALE * props.shoulder_spread;
    let hip_x = 2.0 * PIXEL_SCALE * props.hip_spread;

    // 5. Update Each Limb Entity:
    for (limb, mut transform, mut vis) in &mut limbs_query {
        match limb {
            PlayerLimb::Torso => {
                transform.translation = torso_trans;
                transform.scale = props.torso_scale;
                if motion.flying {
                    if is_flying_fast {
                        let pitch_tilt = (h_speed / 18.0).clamp(0.2, 0.8) * 1.4;
                        transform.rotation = Quat::from_rotation_x(pitch_tilt);
                    } else if is_flying_up {
                        transform.rotation = Quat::from_rotation_x(-0.25);
                    } else if is_flying_down {
                        transform.rotation = Quat::from_rotation_x(0.35);
                    } else {
                        transform.rotation = Quat::from_rotation_x(hover * 0.5);
                    }
                } else {
                    transform.rotation = torso_base_rot * Quat::from_rotation_x(breathe * 0.5);
                }
            }
            PlayerLimb::Head => {
                if let Some(ref mut vis) = vis {
                    let target_vis = if should_show_head {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                    if **vis != target_vis {
                        **vis = target_vis;
                    }
                }
                // Counter-scale torso so head retains its intended racial proportions
                transform.scale = props.head_scale / props.torso_scale;
                transform.translation = Vec3::new(0.0, 12.0 * PIXEL_SCALE, 0.0);
                if motion.flying && is_flying_fast {
                    transform.rotation = Quat::from_rotation_x(-0.8 + camera_pitch.clamp(-0.4, 0.4));
                } else {
                    let look_pitch = (-camera_pitch + base_head_pitch_offset).clamp(-1.45, 1.45);
                    transform.rotation = Quat::from_rotation_x(look_pitch);
                }
            }
            PlayerLimb::LeftArm => {
                transform.scale = props.arm_scale / props.torso_scale;
                transform.translation = Vec3::new(shoulder_x / props.torso_scale.x, 12.0 * PIXEL_SCALE, 0.0);
                if is_swimming {
                    transform.rotation = Quat::from_rotation_x(swim_stroke - 0.3)
                        * Quat::from_rotation_z(0.35 + swim_stroke.abs() * 0.3);
                } else if motion.stance == PlayerStance::Crawling {
                    if h_speed > 0.05 {
                        transform.rotation = Quat::from_rotation_x(1.52 + crawl_swing)
                            * Quat::from_rotation_z(0.2);
                    } else {
                        transform.rotation = Quat::from_rotation_x(1.4) * Quat::from_rotation_z(0.2);
                    }
                } else if motion.flying {
                    if is_flying_fast {
                        transform.rotation =
                            Quat::from_rotation_x(-2.6) * Quat::from_rotation_z(0.15);
                    } else if is_flying_up {
                        transform.rotation =
                            Quat::from_rotation_x(-1.8) * Quat::from_rotation_z(0.25);
                    } else if is_flying_down {
                        transform.rotation =
                            Quat::from_rotation_z(0.65) * Quat::from_rotation_x(-0.15);
                    } else {
                        transform.rotation = Quat::from_rotation_z(0.25 + hover * 0.5)
                            * Quat::from_rotation_x(-0.1);
                    }
                } else if !motion.grounded {
                    let target = Quat::from_rotation_z(0.35) * Quat::from_rotation_x(-0.25);
                    let blend = 1.0 - (-12.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                } else if h_speed > 0.05 {
                    transform.rotation = Quat::from_rotation_x(-arm_angle);
                } else {
                    let target = Quat::from_rotation_x(breathe);
                    let blend = 1.0 - (-10.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                }
            }
            PlayerLimb::RightArm => {
                // Keep the player's real right arm visible in both 1st and 3rd person
                if let Some(ref mut vis) = vis {
                    let target_vis = if in_pregame_menu || is_spectator {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    };
                    if **vis != target_vis {
                        **vis = target_vis;
                    }
                }

                transform.scale = props.arm_scale / props.torso_scale;
                transform.translation = Vec3::new(
                    -shoulder_x / props.torso_scale.x,
                    12.0 * PIXEL_SCALE,
                    0.0,
                );

                // Natural resting posture (downward at the side, slightly angled forward when carrying an item)
                let equip_dip = (1.0 - hand_anim.equip_progress) * 0.25;
                let hold_offset = if has_held_item { -0.35 + equip_dip } else { -0.08 };
                let action_swing = -swing_phase * 1.25 - hand_anim.place_recoil * 0.45;

                if is_swimming {
                    transform.rotation = Quat::from_rotation_x(swim_stroke - 0.3 + action_swing)
                        * Quat::from_rotation_z(-0.35 - swim_stroke.abs() * 0.3);
                } else if motion.stance == PlayerStance::Crawling {
                    if h_speed > 0.05 {
                        transform.rotation =
                            Quat::from_rotation_x(1.52 - crawl_swing + action_swing)
                                * Quat::from_rotation_z(-0.2);
                    } else {
                        transform.rotation = Quat::from_rotation_x(1.4 + action_swing)
                            * Quat::from_rotation_z(-0.2);
                    }
                } else if motion.flying {
                    if is_flying_fast {
                        transform.rotation = Quat::from_rotation_x(-2.6 + action_swing)
                            * Quat::from_rotation_z(-0.15);
                    } else if is_flying_up {
                        transform.rotation = Quat::from_rotation_x(-1.8 + action_swing)
                            * Quat::from_rotation_z(-0.25);
                    } else if is_flying_down {
                        transform.rotation = Quat::from_rotation_z(-0.65)
                            * Quat::from_rotation_x(-0.15 + action_swing);
                    } else {
                        transform.rotation = Quat::from_rotation_z(-0.25 - hover * 0.5)
                            * Quat::from_rotation_x(-0.1 + hold_offset + action_swing);
                    }
                } else if !motion.grounded {
                    let target = Quat::from_rotation_z(-0.35)
                        * Quat::from_rotation_x(-0.25 + hold_offset + action_swing);
                    let blend = 1.0 - (-12.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                } else if h_speed > 0.05 {
                    transform.rotation =
                        Quat::from_rotation_x(arm_angle * 0.75 + hold_offset + action_swing);
                } else {
                    let target = Quat::from_rotation_x(-breathe + hold_offset + action_swing);
                    let blend = 1.0 - (-12.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                }
            }
            PlayerLimb::LeftLeg => {
                transform.scale = props.leg_scale;
                transform.translation = Vec3::new(hip_x, leg_y, leg_z);
                if is_swimming {
                    transform.rotation = Quat::from_rotation_x(1.50 + swim_kick);
                } else if motion.stance == PlayerStance::Crawling {
                    if h_speed > 0.05 {
                        transform.rotation = Quat::from_rotation_x(1.52 - crawl_swing * 0.25);
                    } else {
                        transform.rotation = Quat::from_rotation_x(1.52);
                    }
                } else if motion.flying {
                    if is_flying_up {
                        transform.rotation = Quat::from_rotation_x(0.30);
                    } else if is_flying_down {
                        transform.rotation =
                            Quat::from_rotation_x(-0.25) * Quat::from_rotation_z(0.1);
                    } else if is_flying_fast {
                        let fly_tilt = (h_speed / 15.0).clamp(0.2, 0.8) * 1.3;
                        transform.rotation = Quat::from_rotation_x(fly_tilt);
                    } else {
                        transform.rotation = Quat::from_rotation_x(0.15 + hover);
                    }
                } else if !motion.grounded {
                    let target = Quat::from_rotation_x(-0.3);
                    let blend = 1.0 - (-12.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                } else if h_speed > 0.05 {
                    transform.rotation = Quat::from_rotation_x(leg_angle + crouch_bend);
                } else {
                    let target = Quat::from_rotation_x(crouch_bend);
                    let blend = 1.0 - (-10.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                }
            }
            PlayerLimb::RightLeg => {
                transform.scale = props.leg_scale;
                transform.translation = Vec3::new(-hip_x, leg_y, leg_z);
                if is_swimming {
                    transform.rotation = Quat::from_rotation_x(1.50 - swim_kick);
                } else if motion.stance == PlayerStance::Crawling {
                    if h_speed > 0.05 {
                        transform.rotation = Quat::from_rotation_x(1.52 + crawl_swing * 0.25);
                    } else {
                        transform.rotation = Quat::from_rotation_x(1.52);
                    }
                } else if motion.flying {
                    if is_flying_up {
                        transform.rotation = Quat::from_rotation_x(0.30);
                    } else if is_flying_down {
                        transform.rotation =
                            Quat::from_rotation_x(-0.25) * Quat::from_rotation_z(-0.1);
                    } else if is_flying_fast {
                        let fly_tilt = (h_speed / 15.0).clamp(0.2, 0.8) * 1.3;
                        transform.rotation = Quat::from_rotation_x(fly_tilt);
                    } else {
                        transform.rotation = Quat::from_rotation_x(0.25 - hover);
                    }
                } else if !motion.grounded {
                    let target = Quat::from_rotation_x(-0.15);
                    let blend = 1.0 - (-12.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                } else if h_speed > 0.05 {
                    transform.rotation = Quat::from_rotation_x(-leg_angle + crouch_bend);
                } else {
                    let target = Quat::from_rotation_x(crouch_bend);
                    let blend = 1.0 - (-10.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Procedural Voxel Humanoid & Race Silhouette Mesh Construction
// ---------------------------------------------------------------------------

pub(crate) fn build_head_mesh(race: CharacterRace) -> Mesh {
    let mut builder = VoxelBoxBuilder::default();
    let s = PIXEL_SCALE;

    // Base head: 8x8x8 px, pivot at neck base (y = 0.0 to 8*scale)
    builder.add_box(Vec3::new(8.0 * s, 8.0 * s, 8.0 * s), Vec3::new(0.0, 4.0 * s, 0.0));

    match race {
        CharacterRace::Human => {
            // Subtle nose bridge & brow contour
            builder.add_box(Vec3::new(1.5 * s, 2.0 * s, 1.0 * s), Vec3::new(0.0, 3.8 * s, 4.3 * s));
        }
        CharacterRace::Elf | CharacterRace::Drow => {
            // Long swept-back pointed elven ears on left (+X) and right (-X)
            for sign in [-1.0, 1.0] {
                builder.add_box(
                    Vec3::new(1.2 * s, 1.8 * s, 2.2 * s),
                    Vec3::new(sign * 4.5 * s, 4.4 * s, -0.5 * s),
                );
                builder.add_box(
                    Vec3::new(1.0 * s, 1.4 * s, 2.8 * s),
                    Vec3::new(sign * 5.2 * s, 5.1 * s, -2.2 * s),
                );
                builder.add_box(
                    Vec3::new(0.8 * s, 1.0 * s, 1.8 * s),
                    Vec3::new(sign * 5.7 * s, 5.7 * s, -3.8 * s),
                );
            }
        }
        CharacterRace::Dwarf => {
            // Heavy brow ridge
            builder.add_box(Vec3::new(8.4 * s, 1.5 * s, 1.4 * s), Vec3::new(0.0, 5.6 * s, 4.2 * s));
            // Prominent nose
            builder.add_box(Vec3::new(2.2 * s, 2.2 * s, 1.5 * s), Vec3::new(0.0, 4.0 * s, 4.5 * s));
            // Thick volumetric mustache & tiered dwarf beard cascading down over chest
            builder.add_box(Vec3::new(7.6 * s, 2.0 * s, 2.0 * s), Vec3::new(0.0, 2.4 * s, 4.4 * s));
            builder.add_box(Vec3::new(8.4 * s, 3.5 * s, 2.8 * s), Vec3::new(0.0, 0.5 * s, 3.8 * s));
            builder.add_box(Vec3::new(6.8 * s, 3.5 * s, 2.4 * s), Vec3::new(0.0, -2.2 * s, 3.8 * s));
            builder.add_box(Vec3::new(4.6 * s, 2.5 * s, 2.0 * s), Vec3::new(0.0, -4.5 * s, 3.8 * s));
        }
        CharacterRace::Halfling => {
            // Short pointed ears & locks
            for sign in [-1.0, 1.0] {
                builder.add_box(
                    Vec3::new(1.2 * s, 1.6 * s, 1.8 * s),
                    Vec3::new(sign * 4.4 * s, 4.2 * s, -0.4 * s),
                );
            }
            // Top hair tuft
            builder.add_box(Vec3::new(6.5 * s, 1.2 * s, 6.5 * s), Vec3::new(0.0, 8.4 * s, 0.2 * s));
        }
        CharacterRace::Orc => {
            // Heavy jutting brow ridge
            builder.add_box(Vec3::new(8.6 * s, 1.6 * s, 1.6 * s), Vec3::new(0.0, 5.8 * s, 4.2 * s));
            // Broad jutting lower jaw
            builder.add_box(Vec3::new(8.6 * s, 2.8 * s, 2.2 * s), Vec3::new(0.0, 1.4 * s, 4.2 * s));
            // Two upward-protruding tusks
            for sign in [-1.0, 1.0] {
                builder.add_box(
                    Vec3::new(1.1 * s, 2.2 * s, 1.1 * s),
                    Vec3::new(sign * 2.5 * s, 2.9 * s, 5.1 * s),
                );
                // Pointed orc ears
                builder.add_box(
                    Vec3::new(1.4 * s, 1.6 * s, 2.0 * s),
                    Vec3::new(sign * 4.5 * s, 4.5 * s, -0.6 * s),
                );
            }
        }
        CharacterRace::Dragonborn => {
            // Elongated draconic snout / muzzle projecting forward (+Z)
            builder.add_box(Vec3::new(6.2 * s, 4.2 * s, 3.6 * s), Vec3::new(0.0, 3.2 * s, 5.4 * s));
            builder.add_box(Vec3::new(5.0 * s, 3.2 * s, 2.2 * s), Vec3::new(0.0, 2.9 * s, 7.6 * s));
            // Swept-back draconic horns & frills (-Z)
            for sign in [-1.0, 1.0] {
                builder.add_box(
                    Vec3::new(1.6 * s, 1.8 * s, 3.5 * s),
                    Vec3::new(sign * 2.8 * s, 7.6 * s, -2.5 * s),
                );
                builder.add_box(
                    Vec3::new(1.3 * s, 1.5 * s, 3.2 * s),
                    Vec3::new(sign * 3.2 * s, 8.8 * s, -5.0 * s),
                );
            }
            // Central skull crest
            builder.add_box(Vec3::new(1.4 * s, 2.0 * s, 6.0 * s), Vec3::new(0.0, 8.4 * s, -1.5 * s));
        }
        CharacterRace::Tiefling => {
            // Pointed ears
            for sign in [-1.0, 1.0] {
                builder.add_box(
                    Vec3::new(1.1 * s, 1.5 * s, 2.2 * s),
                    Vec3::new(sign * 4.4 * s, 4.4 * s, -0.8 * s),
                );
                // Curved infernal horns rising from forehead and sweeping back/up
                builder.add_box(
                    Vec3::new(1.8 * s, 2.2 * s, 2.0 * s),
                    Vec3::new(sign * 2.6 * s, 7.8 * s, 2.2 * s),
                );
                builder.add_box(
                    Vec3::new(1.5 * s, 2.4 * s, 2.6 * s),
                    Vec3::new(sign * 3.2 * s, 9.6 * s, 0.2 * s),
                );
                builder.add_box(
                    Vec3::new(1.2 * s, 2.2 * s, 2.4 * s),
                    Vec3::new(sign * 3.5 * s, 11.2 * s, -1.8 * s),
                );
            }
        }
    }

    builder.build()
}

pub(crate) fn build_torso_mesh(race: CharacterRace) -> Mesh {
    let mut builder = VoxelBoxBuilder::default();
    let s = PIXEL_SCALE;

    // Base torso: 8x12x4 px, pivot at waist (y = 0.0 to 12*scale)
    builder.add_box(
        Vec3::new(8.0 * s, 12.0 * s, 4.0 * s),
        Vec3::new(0.0, 6.0 * s, 0.0),
    );

    match race {
        CharacterRace::Dwarf | CharacterRace::Orc => {
            // Broad shoulder pauldrons & chest plate relief
            builder.add_box(
                Vec3::new(9.2 * s, 5.5 * s, 4.8 * s),
                Vec3::new(0.0, 8.8 * s, 0.0),
            );
        }
        CharacterRace::Dragonborn => {
            // Dorsal back spines & thick segmented draconic tail projecting backward (-Z)
            builder.add_box(
                Vec3::new(1.2 * s, 8.0 * s, 1.4 * s),
                Vec3::new(0.0, 6.5 * s, -2.5 * s),
            );
            builder.add_box(
                Vec3::new(3.2 * s, 3.0 * s, 4.5 * s),
                Vec3::new(0.0, 1.8 * s, -3.8 * s),
            );
            builder.add_box(
                Vec3::new(2.4 * s, 2.2 * s, 4.8 * s),
                Vec3::new(0.0, 0.4 * s, -7.2 * s),
            );
            builder.add_box(
                Vec3::new(1.6 * s, 1.6 * s, 4.2 * s),
                Vec3::new(0.0, -0.8 * s, -10.6 * s),
            );
        }
        CharacterRace::Tiefling => {
            // Slender devilish tail with spade tip extending from lower back (-Z)
            builder.add_box(
                Vec3::new(1.4 * s, 1.4 * s, 4.2 * s),
                Vec3::new(0.0, 1.6 * s, -3.6 * s),
            );
            builder.add_box(
                Vec3::new(1.1 * s, 1.2 * s, 4.5 * s),
                Vec3::new(0.0, 0.6 * s, -7.2 * s),
            );
            builder.add_box(
                Vec3::new(1.0 * s, 1.0 * s, 3.6 * s),
                Vec3::new(0.0, 1.2 * s, -10.4 * s),
            );
            // Spade tail tip
            builder.add_box(
                Vec3::new(2.2 * s, 0.8 * s, 2.2 * s),
                Vec3::new(0.0, 1.6 * s, -12.4 * s),
            );
        }
        CharacterRace::Drow => {
            // High underdark mantle collar around neck
            builder.add_box(
                Vec3::new(8.6 * s, 2.0 * s, 4.6 * s),
                Vec3::new(0.0, 11.4 * s, -0.4 * s),
            );
        }
        _ => {}
    }

    builder.build()
}

pub(crate) fn build_left_arm_mesh() -> Mesh {
    let mut builder = VoxelBoxBuilder::default();
    // Base left arm: 4x12x4 px, pivot at shoulder joint (y = 0 down to -12*scale)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
    );
    builder.build()
}

pub(crate) fn build_right_arm_mesh() -> Mesh {
    let mut builder = VoxelBoxBuilder::default();
    // Base right arm: 4x12x4 px, pivot at shoulder joint (y = 0 down to -12*scale)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
    );
    builder.build()
}

pub(crate) fn build_left_leg_mesh() -> Mesh {
    let mut builder = VoxelBoxBuilder::default();
    // Base left leg: 4x12x4 px, pivot at hip joint (y = 0 down to -12*scale)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
    );
    builder.build()
}

pub(crate) fn build_right_leg_mesh() -> Mesh {
    let mut builder = VoxelBoxBuilder::default();
    // Base right leg: 4x12x4 px, pivot at hip joint (y = 0 down to -12*scale)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
    );
    builder.build()
}

#[derive(Default)]
struct VoxelBoxBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl VoxelBoxBuilder {
    fn add_box(&mut self, size: Vec3, center: Vec3) {
        let hx = size.x * 0.5;
        let hy = size.y * 0.5;
        let hz = size.z * 0.5;

        let cx = center.x;
        let cy = center.y;
        let cz = center.z;

        let x0 = cx - hx;
        let x1 = cx + hx;
        let y0 = cy - hy;
        let y1 = cy + hy;
        let z0 = cz - hz;
        let z1 = cz + hz;

        // Front face (+Z)
        self.push_quad(
            [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]],
            [0.0, 0.0, 1.0],
        );

        // Back face (-Z)
        self.push_quad(
            [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]],
            [0.0, 0.0, -1.0],
        );

        // Right face (-X, wearer's right)
        self.push_quad(
            [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]],
            [-1.0, 0.0, 0.0],
        );

        // Left face (+X, wearer's left)
        self.push_quad(
            [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]],
            [1.0, 0.0, 0.0],
        );

        // Top face (+Y)
        self.push_quad(
            [[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]],
            [0.0, 1.0, 0.0],
        );

        // Bottom face (-Y)
        self.push_quad(
            [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
            [0.0, -1.0, 0.0],
        );
    }

    fn push_quad(&mut self, vertices: [[f32; 3]; 4], normal: [f32; 3]) {
        let base = self.positions.len() as u32;

        for v in &vertices {
            self.positions.push(*v);
            self.normals.push(normal);
        }

        self.uvs.push([0.0, 1.0]);
        self.uvs.push([1.0, 1.0]);
        self.uvs.push([1.0, 0.0]);
        self.uvs.push([0.0, 0.0]);

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}
