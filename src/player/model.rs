use std::f32::consts::PI;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use crate::menu::MenuState;

use super::{
    GameMode, PLAYER_HEIGHT, Player, PlayerEnvironmentStatus, PlayerStance,
    controller::{PlayerCamera, PlayerMotion},
};

const SKIN_RESOLUTION: f32 = 64.0;
const PIXEL_SCALE: f32 = PLAYER_HEIGHT / 32.0; // 0.05625m per pixel -> 32 pixels = 1.8m

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

pub struct PlayerModelPlugin;

impl Plugin for PlayerModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player_model).add_systems(
            Update,
            update_player_model.after(crate::player::PlayerSet::Movement),
        );
    }
}

pub fn load_player_skin_image() -> Image {
    let path = "assets/textures/mobs/player_skin.png";
    let raw = match image::open(path) {
        Ok(img) => img.into_rgba8(),
        Err(_) => {
            // Generate fallback 64x64 Steve skin if file is missing
            let mut img = image::RgbaImage::new(64, 64);
            for pixel in img.pixels_mut() {
                *pixel = image::Rgba([45, 90, 180, 255]);
            }
            img
        }
    };

    let (width, height) = raw.dimensions();
    let data = raw.into_raw();
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}

pub fn setup_player_model(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let skin_image = load_player_skin_image();
    let skin_handle = images.add(skin_image);

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(skin_handle),
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        ..default()
    });

    let head_mesh = meshes.add(build_head_mesh());
    let torso_mesh = meshes.add(build_torso_mesh());
    let left_arm_mesh = meshes.add(build_left_arm_mesh());
    let right_arm_mesh = meshes.add(build_right_arm_mesh());
    let left_leg_mesh = meshes.add(build_left_leg_mesh());
    let right_leg_mesh = meshes.add(build_right_leg_mesh());

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
            MeshMaterial3d(material_handle.clone()),
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
            MeshMaterial3d(material_handle.clone()),
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
            MeshMaterial3d(material_handle.clone()),
            Transform::from_translation(Vec3::new(6.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 0.0)),
            Visibility::Inherited,
        ))
        .id();

    // Right Arm (shoulder pivot at top right of torso: local x = -6 * PIXEL_SCALE, y = 12 * PIXEL_SCALE)
    let right_arm = commands
        .spawn((
            PlayerRightArm,
            PlayerLimb::RightArm,
            Mesh3d(right_arm_mesh),
            MeshMaterial3d(material_handle.clone()),
            Transform::from_translation(Vec3::new(-6.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 0.0)),
            Visibility::Inherited,
        ))
        .id();

    // Left Leg (hip pivot at y = 12 * PIXEL_SCALE, x = +2 * PIXEL_SCALE)
    let left_leg = commands
        .spawn((
            PlayerLeftLeg,
            PlayerLimb::LeftLeg,
            Mesh3d(left_leg_mesh),
            MeshMaterial3d(material_handle.clone()),
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
            MeshMaterial3d(material_handle),
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

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_player_model(
    time: Res<Time>,
    env_status: Res<PlayerEnvironmentStatus>,
    game_mode: Res<GameMode>,
    menu_state: Option<Res<State<MenuState>>>,
    player: Single<(&Transform, &PlayerMotion), With<Player>>,
    camera: Single<(&PlayerCamera, &Transform), (With<Camera3d>, Without<Player>)>,
    model_root: Single<
        (&mut Transform, &mut PlayerAnimState),
        (
            With<PlayerModelRoot>,
            Without<Player>,
            Without<Camera3d>,
            Without<PlayerLimb>,
        ),
    >,
    mut limbs_query: Query<
        (&PlayerLimb, &mut Transform, Option<&mut Visibility>),
        (Without<PlayerModelRoot>, Without<Player>, Without<Camera3d>),
    >,
) {
    let (player_transform, motion) = player.into_inner();
    let (camera_controller, camera_transform) = camera.into_inner();
    let (mut root_transform, mut anim_state) = model_root.into_inner();

    let is_paused = menu_state
        .as_ref()
        .is_some_and(|s| *s.get() == MenuState::Pause || *s.get() == MenuState::Settings);

    let delta_seconds = if is_paused { 0.0 } else { time.delta_secs() };

    // 1. First-Person vs Third-Person Visibility:
    // In First-Person: Head is HIDDEN to prevent camera interior skull clipping.
    // Torso, Arms, and Legs remain visible beneath the camera!
    // In Third-Person: Head is VISIBLE.
    let is_third_person = camera_controller.is_third_person();
    let should_show_head = *game_mode == GameMode::Spectator || is_third_person;

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

    // Position root at player feet and align with world heading (+ PI aligns model front face with forward)
    root_transform.translation = player_transform.translation;
    root_transform.rotation = Quat::from_rotation_y(anim_state.body_yaw + PI);

    let water_submersion = env_status.submersion;
    let in_deep_water = water_submersion > 0.45;
    let is_swimming = in_deep_water
        && (!motion.grounded || h_speed > 1.2 || motion.velocity.y.abs() > 0.3 || motion.flying);

    // 3. Stance Offsets & Base Postures:
    let (torso_trans, torso_base_rot, leg_y, leg_z, base_head_pitch_offset) = if is_swimming {
        // Prone swimming: torso waist shifted back so neck and head align at camera origin (z=0)
        let swim_y = motion.current_eye_height - 3.0 * PIXEL_SCALE;
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
                Vec3::new(0.0, 12.0 * PIXEL_SCALE, 0.0),
                Quat::IDENTITY,
                12.0 * PIXEL_SCALE,
                0.0,
                0.0,
            ),
            PlayerStance::Crouching => (
                Vec3::new(0.0, 7.0 * PIXEL_SCALE, -2.0 * PIXEL_SCALE),
                Quat::from_rotation_x(0.22),
                7.0 * PIXEL_SCALE,
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

    // Flying state parameters
    let v_y = motion.velocity.y;
    let is_flying_up = motion.flying && v_y > 0.5;
    let is_flying_down = motion.flying && v_y < -0.5;
    let is_flying_fast = motion.flying && h_speed > 0.5;
    let hover = (time.elapsed_secs() * 2.2).sin() * 0.08;

    // Torso rotation based on movement state
    let torso_rot = if is_swimming {
        Quat::from_rotation_x(1.50)
    } else if is_flying_up {
        Quat::from_rotation_x(-0.25)
    } else if is_flying_down {
        Quat::from_rotation_x(0.20)
    } else if is_flying_fast {
        let fly_tilt = (h_speed / 15.0).clamp(0.2, 0.8) * 1.3;
        Quat::from_rotation_x(fly_tilt)
    } else if motion.flying {
        Quat::from_rotation_x(hover * 0.4)
    } else {
        torso_base_rot
    };

    // Head orientation (-camera_pitch tracks look angle without inversion)
    let head_rot = if is_third_person {
        let head_yaw = (camera_yaw - anim_state.body_yaw).clamp(-1.2, 1.2);
        let stance_pitch_offset = if is_swimming || motion.stance == PlayerStance::Crawling {
            -1.35
        } else if is_flying_up {
            0.15
        } else if is_flying_down {
            -0.15
        } else if is_flying_fast {
            -((h_speed / 15.0).clamp(0.2, 0.8) * 1.3 * 0.8)
        } else {
            base_head_pitch_offset
        };
        Quat::from_euler(
            EulerRot::YXZ,
            head_yaw,
            -camera_pitch + stance_pitch_offset,
            0.0,
        )
    } else {
        Quat::IDENTITY
    };

    // Swimming cycles
    let swim_kick = (anim_state.swim_cycle * 1.5).sin() * 0.25;
    let swim_reach = (anim_state.swim_cycle).cos() * 0.25;
    let swim_stroke = (anim_state.swim_cycle).sin() * 0.35;

    // Crawling cycles
    let crawl_swing = (anim_state.walk_cycle).sin() * 0.5;

    // Walking / sprinting cycles
    let stride = (h_speed / 5.0).clamp(0.1, 1.2);
    let leg_angle = (anim_state.walk_cycle).sin() * 0.65 * stride;
    let arm_angle = (anim_state.walk_cycle).sin() * 0.55 * stride;
    let crouch_bend = if motion.stance == PlayerStance::Crouching {
        -0.30
    } else {
        0.0
    };

    // 5. Apply transforms to all limbs:
    for (limb, mut transform, opt_visibility) in &mut limbs_query {
        match limb {
            PlayerLimb::Head => {
                if let Some(mut vis) = opt_visibility {
                    *vis = if should_show_head {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                }
                transform.translation = Vec3::new(0.0, 12.0 * PIXEL_SCALE, 0.0);
                transform.rotation = head_rot;
            }
            PlayerLimb::Torso => {
                transform.translation = torso_trans;
                transform.rotation = torso_rot;
            }
            PlayerLimb::LeftArm => {
                transform.translation = Vec3::new(6.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 0.0);
                if is_swimming {
                    // Stretched forward swimming stroke
                    transform.rotation = Quat::from_rotation_x(PI + swim_reach * 0.25)
                        * Quat::from_rotation_z(0.12 + swim_stroke.max(0.0) * 0.35);
                } else if motion.stance == PlayerStance::Crawling {
                    // Straight forward crawl strokes along the ground
                    if h_speed > 0.05 {
                        transform.rotation = Quat::from_rotation_x(PI + crawl_swing * 0.35)
                            * Quat::from_rotation_z(0.08);
                    } else {
                        transform.rotation =
                            Quat::from_rotation_x(PI) * Quat::from_rotation_z(0.08);
                    }
                } else if motion.flying {
                    if is_flying_up {
                        // Ascending: reaching upward into the sky
                        transform.rotation =
                            Quat::from_rotation_x(2.8) * Quat::from_rotation_z(0.2);
                    } else if is_flying_down {
                        // Descending: arms outstretched for balance
                        transform.rotation =
                            Quat::from_rotation_z(0.65) * Quat::from_rotation_x(-0.15);
                    } else if is_flying_fast {
                        // Fast horizontal flight: streamlined arms
                        transform.rotation =
                            Quat::from_rotation_x(0.2) * Quat::from_rotation_z(0.15);
                    } else {
                        // Hovering: gentle floating levitation
                        transform.rotation =
                            Quat::from_rotation_z(0.25 + hover * 0.5) * Quat::from_rotation_x(-0.1);
                    }
                } else if !motion.grounded {
                    // Jumping / Airborne fall
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
                transform.translation = Vec3::new(-6.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 0.0);
                if is_swimming {
                    // Stretched forward swimming stroke
                    transform.rotation = Quat::from_rotation_x(PI - swim_reach * 0.25)
                        * Quat::from_rotation_z(-0.12 - swim_stroke.min(0.0).abs() * 0.35);
                } else if motion.stance == PlayerStance::Crawling {
                    // Straight forward crawl strokes along the ground
                    if h_speed > 0.05 {
                        transform.rotation = Quat::from_rotation_x(PI - crawl_swing * 0.35)
                            * Quat::from_rotation_z(-0.08);
                    } else {
                        transform.rotation =
                            Quat::from_rotation_x(PI) * Quat::from_rotation_z(-0.08);
                    }
                } else if motion.flying {
                    if is_flying_up {
                        // Ascending: reaching upward into the sky
                        transform.rotation =
                            Quat::from_rotation_x(2.8) * Quat::from_rotation_z(-0.2);
                    } else if is_flying_down {
                        // Descending: arms outstretched for balance
                        transform.rotation =
                            Quat::from_rotation_z(-0.65) * Quat::from_rotation_x(-0.15);
                    } else if is_flying_fast {
                        // Fast horizontal flight: streamlined arms
                        transform.rotation =
                            Quat::from_rotation_x(0.2) * Quat::from_rotation_z(-0.15);
                    } else {
                        // Hovering: gentle floating levitation
                        transform.rotation = Quat::from_rotation_z(-0.25 - hover * 0.5)
                            * Quat::from_rotation_x(-0.1);
                    }
                } else if !motion.grounded {
                    let target = Quat::from_rotation_z(-0.35) * Quat::from_rotation_x(-0.25);
                    let blend = 1.0 - (-12.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                } else if h_speed > 0.05 {
                    transform.rotation = Quat::from_rotation_x(arm_angle);
                } else {
                    let target = Quat::from_rotation_x(-breathe);
                    let blend = 1.0 - (-10.0 * delta_seconds).exp();
                    transform.rotation = transform.rotation.slerp(target, blend);
                }
            }
            PlayerLimb::LeftLeg => {
                transform.translation = Vec3::new(2.0 * PIXEL_SCALE, leg_y, leg_z);
                if is_swimming {
                    // Stretched straight back horizontally with flutter kick
                    transform.rotation = Quat::from_rotation_x(1.50 + swim_kick);
                } else if motion.stance == PlayerStance::Crawling {
                    // Stretched straight back horizontally
                    if h_speed > 0.05 {
                        transform.rotation = Quat::from_rotation_x(1.52 - crawl_swing * 0.25);
                    } else {
                        transform.rotation = Quat::from_rotation_x(1.52);
                    }
                } else if motion.flying {
                    if is_flying_up {
                        // Ascending: legs trailing down/back
                        transform.rotation = Quat::from_rotation_x(0.30);
                    } else if is_flying_down {
                        // Descending: legs slightly tucked
                        transform.rotation =
                            Quat::from_rotation_x(-0.25) * Quat::from_rotation_z(0.1);
                    } else if is_flying_fast {
                        // Fast horizontal flight: legs trailing straight back
                        let fly_tilt = (h_speed / 15.0).clamp(0.2, 0.8) * 1.3;
                        transform.rotation = Quat::from_rotation_x(fly_tilt);
                    } else {
                        // Hovering: trailing gracefully
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
                transform.translation = Vec3::new(-2.0 * PIXEL_SCALE, leg_y, leg_z);
                if is_swimming {
                    // Stretched straight back horizontally with flutter kick
                    transform.rotation = Quat::from_rotation_x(1.50 - swim_kick);
                } else if motion.stance == PlayerStance::Crawling {
                    // Stretched straight back horizontally
                    if h_speed > 0.05 {
                        transform.rotation = Quat::from_rotation_x(1.52 + crawl_swing * 0.25);
                    } else {
                        transform.rotation = Quat::from_rotation_x(1.52);
                    }
                } else if motion.flying {
                    if is_flying_up {
                        // Ascending: legs trailing down/back
                        transform.rotation = Quat::from_rotation_x(0.30);
                    } else if is_flying_down {
                        // Descending: legs slightly tucked
                        transform.rotation =
                            Quat::from_rotation_x(-0.25) * Quat::from_rotation_z(-0.1);
                    } else if is_flying_fast {
                        // Fast horizontal flight: legs trailing straight back
                        let fly_tilt = (h_speed / 15.0).clamp(0.2, 0.8) * 1.3;
                        transform.rotation = Quat::from_rotation_x(fly_tilt);
                    } else {
                        // Hovering: trailing gracefully
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
// Procedural Minecraft Skin Mesh Construction (64x64 format with 3D overlays)
// ---------------------------------------------------------------------------

fn build_head_mesh() -> Mesh {
    let mut builder = SkinBoxBuilder::default();
    // Base head: 8x8x8 px, pivot at neck base (y = 0.0 to 8*scale)
    builder.add_box(
        Vec3::new(8.0 * PIXEL_SCALE, 8.0 * PIXEL_SCALE, 8.0 * PIXEL_SCALE),
        Vec3::new(0.0, 4.0 * PIXEL_SCALE, 0.0), // center
        (0, 0),
        (8, 8, 8),
        0.0,
    );
    // Head overlay (hat / hair layer): expanded by 1.5cm
    builder.add_box(
        Vec3::new(8.0 * PIXEL_SCALE, 8.0 * PIXEL_SCALE, 8.0 * PIXEL_SCALE),
        Vec3::new(0.0, 4.0 * PIXEL_SCALE, 0.0),
        (32, 0),
        (8, 8, 8),
        0.015,
    );
    builder.build()
}

fn build_torso_mesh() -> Mesh {
    let mut builder = SkinBoxBuilder::default();
    // Base torso: 8x12x4 px, pivot at waist (y = 0.0 to 12*scale)
    builder.add_box(
        Vec3::new(8.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, 6.0 * PIXEL_SCALE, 0.0),
        (16, 16),
        (8, 12, 4),
        0.0,
    );
    // Torso overlay (jacket): expanded by 1.5cm
    builder.add_box(
        Vec3::new(8.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, 6.0 * PIXEL_SCALE, 0.0),
        (16, 32),
        (8, 12, 4),
        0.015,
    );
    builder.build()
}

fn build_left_arm_mesh() -> Mesh {
    let mut builder = SkinBoxBuilder::default();
    // Base left arm: 4x12x4 px, pivot at shoulder joint (y = 0 down to -12*scale)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
        (32, 48),
        (4, 12, 4),
        0.0,
    );
    // Left arm overlay (sleeve)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
        (48, 48),
        (4, 12, 4),
        0.015,
    );
    builder.build()
}

fn build_right_arm_mesh() -> Mesh {
    let mut builder = SkinBoxBuilder::default();
    // Base right arm: 4x12x4 px, pivot at shoulder joint (y = 0 down to -12*scale)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
        (40, 16),
        (4, 12, 4),
        0.0,
    );
    // Right arm overlay (sleeve)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
        (40, 32),
        (4, 12, 4),
        0.015,
    );
    builder.build()
}

fn build_left_leg_mesh() -> Mesh {
    let mut builder = SkinBoxBuilder::default();
    // Base left leg: 4x12x4 px, pivot at hip joint (y = 0 down to -12*scale)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
        (16, 48),
        (4, 12, 4),
        0.0,
    );
    // Left leg overlay (pants)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
        (0, 48),
        (4, 12, 4),
        0.015,
    );
    builder.build()
}

fn build_right_leg_mesh() -> Mesh {
    let mut builder = SkinBoxBuilder::default();
    // Base right leg: 4x12x4 px, pivot at hip joint (y = 0 down to -12*scale)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
        (0, 16),
        (4, 12, 4),
        0.0,
    );
    // Right leg overlay (pants)
    builder.add_box(
        Vec3::new(4.0 * PIXEL_SCALE, 12.0 * PIXEL_SCALE, 4.0 * PIXEL_SCALE),
        Vec3::new(0.0, -6.0 * PIXEL_SCALE, 0.0),
        (0, 32),
        (4, 12, 4),
        0.015,
    );
    builder.build()
}

#[derive(Default)]
struct SkinBoxBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl SkinBoxBuilder {
    fn add_box(
        &mut self,
        size: Vec3,
        center: Vec3,
        tex_origin: (u32, u32),
        tex_dims: (u32, u32, u32),
        expand: f32,
    ) {
        let (ox, oy) = (tex_origin.0 as f32, tex_origin.1 as f32);
        let (w, h, d) = (tex_dims.0 as f32, tex_dims.1 as f32, tex_dims.2 as f32);

        let hx = size.x * 0.5 + expand;
        let hy = size.y * 0.5 + expand;
        let hz = size.z * 0.5 + expand;

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
            (ox + d, oy + d),
            (w, h),
        );

        // Back face (-Z)
        self.push_quad(
            [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]],
            [0.0, 0.0, -1.0],
            (ox + 2.0 * d + w, oy + d),
            (w, h),
        );

        // Right face (-X, wearer's right)
        self.push_quad(
            [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]],
            [-1.0, 0.0, 0.0],
            (ox, oy + d),
            (d, h),
        );

        // Left face (+X, wearer's left)
        self.push_quad(
            [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]],
            [1.0, 0.0, 0.0],
            (ox + d + w, oy + d),
            (d, h),
        );

        // Top face (+Y)
        self.push_quad(
            [[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]],
            [0.0, 1.0, 0.0],
            (ox + d, oy),
            (w, d),
        );

        // Bottom face (-Y)
        self.push_quad(
            [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
            [0.0, -1.0, 0.0],
            (ox + d + w, oy),
            (w, d),
        );
    }

    fn push_quad(
        &mut self,
        vertices: [[f32; 3]; 4],
        normal: [f32; 3],
        uv_origin: (f32, f32),
        uv_size: (f32, f32),
    ) {
        let base = self.positions.len() as u32;

        let u0 = uv_origin.0 / SKIN_RESOLUTION;
        let u1 = (uv_origin.0 + uv_size.0) / SKIN_RESOLUTION;
        let v0 = uv_origin.1 / SKIN_RESOLUTION;
        let v1 = (uv_origin.1 + uv_size.1) / SKIN_RESOLUTION;

        for v in &vertices {
            self.positions.push(*v);
            self.normals.push(normal);
        }

        // UV mapping: [BL, BR, TR, TL]
        self.uvs.push([u0, v1]);
        self.uvs.push([u1, v1]);
        self.uvs.push([u1, v0]);
        self.uvs.push([u0, v0]);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_skin_box_builds_valid_cuboid_mesh() {
        let head = build_head_mesh();
        assert!(head.count_vertices() > 0);
        // Base box (24 verts) + overlay box (24 verts) = 48 vertices
        assert_eq!(head.count_vertices(), 48);

        let positions = head.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        if let bevy::render::mesh::VertexAttributeValues::Float32x3(pos) = positions {
            assert_eq!(pos.len(), 48);
        } else {
            panic!("Expected Float32x3 vertex positions");
        }
    }

    #[test]
    fn player_model_height_proportions_match_1_8m() {
        let leg_height = 12.0 * PIXEL_SCALE;
        let torso_height = 12.0 * PIXEL_SCALE;
        let head_height = 8.0 * PIXEL_SCALE;
        let total_height = leg_height + torso_height + head_height;
        assert!((total_height - PLAYER_HEIGHT).abs() < 1e-4);
    }
}
