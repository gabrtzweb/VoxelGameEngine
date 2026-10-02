use std::f32::consts::FRAC_PI_2;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

use crate::{
    gameplay::RadialMenuState,
    menu::GameSettings,
    world::{VOXEL_SIZE, Voxel, VoxelWorld},
};

use super::GameMode;

pub const THIRD_PERSON_DISTANCE: f32 = 3.5;

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

    pub third_person: bool,

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

pub fn toggle_inspector_interaction(
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

pub fn toggle_camera_view(
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

pub fn camera_look(
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

pub fn resolve_third_person_camera_distance(
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
