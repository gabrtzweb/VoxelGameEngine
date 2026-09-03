mod dev_camera;
mod dev_stats;
mod voxel;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, window::PresentMode};

use dev_camera::{DevCamera, DevCameraPlugin};

use dev_stats::DevStatsPlugin;

use voxel::{ChunkManagerPlugin, TargetingPlugin, VoxelDebugPlugin, VoxelInteractionPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DevCameraPlugin)
        .add_plugins(ChunkManagerPlugin)
        .add_plugins(DevStatsPlugin)
        .add_plugins(TargetingPlugin)
        .add_plugins(VoxelInteractionPlugin)
        .add_plugins(VoxelDebugPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Main directional light.
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));

    // Development camera.
    let camera_transform =
        Transform::from_xyz(-10.0, 12.0, 14.0).looking_at(Vec3::new(4.0, 3.0, 4.0), Vec3::Y);

    let dev_camera = DevCamera::from_transform(&camera_transform);

    commands.spawn((Camera3d::default(), camera_transform, dev_camera));
}
