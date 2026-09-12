mod dev_stats;
mod environment;
mod player;
mod voxel;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    window::{PresentMode, PrimaryWindow},
    winit::WinitWindows,
};

use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use dev_stats::DevStatsPlugin;
use environment::EnvironmentPlugin;
use player::PlayerPlugin;
use voxel::{
    ChunkManagerPlugin, FluidSimulationPlugin, ShapingPlugin, TargetingPlugin, VoxelDebugPlugin,
    VoxelInteractionPlugin, VoxelMaterial,
};

use winit::{platform::windows::WindowExtWindows, window::Icon};

fn set_window_icons(
    primary_window: Single<Entity, With<PrimaryWindow>>,
    windows: Option<NonSend<WinitWindows>>,
    mut initialized: Local<bool>,
) {
    if *initialized {
        return;
    }

    let Some(windows) = windows else {
        return;
    };

    let entity = *primary_window;

    let Some(window) = windows.get_window(entity) else {
        return;
    };

    let image = image::open("assets/icon.ico")
        .expect("Failed to load application icon")
        .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    let icon = Icon::from_rgba(rgba, width, height).expect("Failed to create application icon");

    window.set_window_icon(Some(icon.clone()));
    window.set_taskbar_icon(Some(icon));

    *initialized = true;
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel Game".to_string(),
                resolution: (1280, 720).into(),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(MaterialPlugin::<VoxelMaterial>::default())
        .add_plugins(EnvironmentPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ChunkManagerPlugin)
        .add_plugins(DevStatsPlugin)
        .add_plugins(TargetingPlugin)
        .add_plugins(VoxelInteractionPlugin)
        .add_plugins(VoxelDebugPlugin)
        .add_plugins(ShapingPlugin)
        .add_plugins(FluidSimulationPlugin)
        .add_systems(Update, set_window_icons)
        .run();
}
