use std::f32::consts::FRAC_PI_2;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

#[derive(Component)]
pub struct DevCamera {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub sensitivity: f32,
    yaw: f32,
    pitch: f32,
}

impl DevCamera {
    pub fn from_transform(transform: &Transform) -> Self {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

        Self {
            walk_speed: 5.0,
            run_speed: 15.0,
            sensitivity: 0.002,
            yaw,
            pitch,
        }
    }
}

pub struct DevCameraPlugin;

impl Plugin for DevCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_crosshair)
            .add_systems(PostStartup, lock_cursor)
            .add_systems(Update, (camera_look, camera_movement));
    }
}

fn lock_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Locked;
}

fn camera_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera_query: Single<(&mut Transform, &mut DevCamera), With<Camera3d>>,
) {
    let delta = mouse_motion.delta;

    if delta == Vec2::ZERO {
        return;
    }

    let (mut transform, mut camera) = camera_query.into_inner();

    camera.yaw -= delta.x * camera.sensitivity;
    camera.pitch -= delta.y * camera.sensitivity;

    camera.pitch = camera.pitch.clamp(
        -FRAC_PI_2 + 0.001,
        FRAC_PI_2 - 0.001,
    );

    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        camera.yaw,
        camera.pitch,
        0.0,
    );
}

fn camera_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Single<(&mut Transform, &DevCamera), With<Camera3d>>,
) {
    let (mut transform, camera) = camera_query.into_inner();

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

    if keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight)
    {
        direction -= Vec3::Y;
    }

    if direction == Vec3::ZERO {
        return;
    }

    let is_running =
        keyboard.pressed(KeyCode::ShiftLeft)
            || keyboard.pressed(KeyCode::ShiftRight);

    let speed = if is_running {
        camera.run_speed
    } else {
        camera.walk_speed
    };

    transform.translation +=
        direction.normalize() * speed * time.delta_secs();
}

fn spawn_crosshair(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ZIndex(100),
        children![
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px(2.0),
                    height: px(14.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    width: px(14.0),
                    height: px(2.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            ),
        ],
    ));
}