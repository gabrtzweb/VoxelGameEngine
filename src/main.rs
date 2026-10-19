mod core;
mod environment;
mod gameplay;
mod generation;
mod map;
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

use bevy_egui::EguiPlugin;

use core::{
    AdaptiveUiPlugin, DevStatsPlugin, DynamicFpsPlugin, DynamicFpsSettings, FontPlugin, LocalConfig,
};
use environment::EnvironmentPlugin;
use gameplay::GameplayPlugin;
use generation::TerrainInspectorPlugin;
use map::MapPlugin;
use menu::{GameSettings, MenuPlugin, ScreenMode};
use meshing::MeshingPlugin;
use player::PlayerPlugin;
use simulation::SimulationPlugin;
use world::{ChunkStreamingSettings, WorldPlugin};

use winit::{platform::windows::WindowExtWindows, window::Icon};

fn set_window_icons(
    primary_window: Single<Entity, With<PrimaryWindow>>,
    windows: Option<NonSend<WinitWindows>>,
) {
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
}

fn main() {
    let local_config = LocalConfig::load_or_default();
    let backends = local_config.resolve_backends();

    let present_mode = if local_config.vsync_enabled {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };

    let screen_mode = match local_config.screen_mode.to_lowercase().as_str() {
        "borderless" | "borderlessfullscreen" => ScreenMode::BorderlessFullscreen,
        "fullscreen" | "exclusivefullscreen" => ScreenMode::ExclusiveFullscreen,
        _ => ScreenMode::Windowed,
    };
    let window_mode = screen_mode.to_window_mode();

    let chunk_settings = ChunkStreamingSettings {
        render_distance: local_config.render_distance,
        lod_render_distance: local_config.lod_render_distance,
        simulation_distance: local_config.simulation_distance,
    };

    let game_settings = GameSettings {
        fov_degrees: local_config.fov_degrees,
        fog_enabled: local_config.fog_enabled,
        view_bobbing: local_config.view_bobbing,
        full_grass: local_config.full_grass,
        screen_mode,
        vsync_enabled: local_config.vsync_enabled,
        dynamic_fps_enabled: local_config.dynamic_fps_enabled,
    };

    let dynamic_fps_settings = DynamicFpsSettings {
        enabled: local_config.dynamic_fps_enabled,
        ..default()
    };

    App::new()
        .insert_resource(local_config)
        .insert_resource(chunk_settings)
        .insert_resource(game_settings)
        .insert_resource(dynamic_fps_settings)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxel Game".to_string(),
                        resolution: (1280, 720).into(),
                        present_mode,
                        mode: window_mode,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(bevy::render::RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::Automatic(Box::new(
                        bevy::render::settings::WgpuSettings {
                            backends: Some(backends),
                            limits: bevy::render::settings::WgpuLimits {
                                max_texture_array_layers: 2048,
                                ..default()
                            },
                            ..default()
                        },
                    )),
                    ..default()
                }),
        )
        .add_plugins(FontPlugin)
        .add_plugins(AdaptiveUiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(TerrainInspectorPlugin)
        .add_plugins(EnvironmentPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(MeshingPlugin)
        .add_plugins(SimulationPlugin)
        .add_plugins(GameplayPlugin)
        .add_plugins(DevStatsPlugin)
        .add_plugins(DynamicFpsPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(MenuPlugin)
        .add_systems(Update, set_window_icons.run_if(run_once))
        .run();
}
