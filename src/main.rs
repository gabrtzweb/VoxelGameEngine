mod core;
mod environment;
mod gameplay;
mod generation;
mod menu;
mod meshing;
mod player;
mod simulation;
mod world;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    window::{PresentMode, PrimaryWindow},
    winit::WinitWindows,
};

use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use core::DevStatsPlugin;
use environment::EnvironmentPlugin;
use gameplay::GameplayPlugin;
use menu::MenuPlugin;
use meshing::MeshingPlugin;
use player::PlayerPlugin;
use simulation::SimulationPlugin;
use world::WorldPlugin;

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
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxel Game".to_string(),
                        resolution: (1280, 720).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(
            WorldInspectorPlugin::default()
                .run_if(|inspector: Res<player::InspectorInteraction>| inspector.active),
        )
        .add_plugins(EnvironmentPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(MeshingPlugin)
        .add_plugins(SimulationPlugin)
        .add_plugins(GameplayPlugin)
        .add_plugins(DevStatsPlugin)
        .add_plugins(MenuPlugin)
        .add_systems(Update, set_window_icons)
        .run();
}
