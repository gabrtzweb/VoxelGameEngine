mod dev_stats;
mod environment;
mod player;
mod voxel;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, window::PresentMode};

use dev_stats::DevStatsPlugin;
use environment::EnvironmentPlugin;
use player::PlayerPlugin;

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
        .add_plugins(EnvironmentPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ChunkManagerPlugin)
        .add_plugins(DevStatsPlugin)
        .add_plugins(TargetingPlugin)
        .add_plugins(VoxelInteractionPlugin)
        .add_plugins(VoxelDebugPlugin)
        .run();
}
