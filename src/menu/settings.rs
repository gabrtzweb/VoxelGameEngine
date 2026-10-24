use bevy::prelude::*;
use bevy::{picking::hover::Hovered, ui::Pressed, ui_widgets::Button};

use crate::{
    audio::AudioSettings,
    core::{AppFont, DynamicFpsSettings, LocalConfig, PerformancePreset, text_shadow_default},
    environment::EnvironmentState,
    meshing::{ChunkMaterial, VoxelTextureRegistry},
    world::{ChunkStreamingQueues, ChunkStreamingSettings, VoxelWorld},
};

use super::{GameSettings, MenuState};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Video,
    Audio,
}

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
struct VideoSettingsSection;

#[derive(Component)]
struct AudioSettingsSection;

#[derive(Component)]
struct TabButton(SettingsTab);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsAction {
    SwitchTab(SettingsTab),
    DecMasterVol,
    IncMasterVol,
    DecMusicVol,
    IncMusicVol,
    DecWeatherVol,
    IncWeatherVol,
    DecSfxVol,
    IncSfxVol,
    DecAmbientVol,
    IncAmbientVol,
    DecPreset,
    IncPreset,
    DecRenderScale,
    IncRenderScale,
    DecUpscalingFilter,
    IncUpscalingFilter,
    DecTargetFps,
    IncTargetFps,
    DecMeshUploadBudget,
    IncMeshUploadBudget,
    ToggleGodRays,
    ToggleTransmission,
    DecAntiAliasing,
    IncAntiAliasing,
    DecShadowQuality,
    IncShadowQuality,
    DecCloudQuality,
    IncCloudQuality,
    DecScreenMode,
    IncScreenMode,
    DecRenderDistance,
    IncRenderDistance,
    DecLodDistance,
    IncLodDistance,
    DecSimulationDistance,
    IncSimulationDistance,
    DecFov,
    IncFov,
    ToggleDynamicFov,
    ToggleFog,
    ToggleViewBobbing,
    ToggleFullGrass,
    ToggleVsync,
    ToggleDynamicFps,
    ToggleTimePause,
    Back,
}

#[derive(Component)]
struct PresetLabel;

#[derive(Component)]
struct RenderScaleLabel;

#[derive(Component)]
struct UpscalingFilterLabel;

#[derive(Component)]
struct TargetFpsLabel;

#[derive(Component)]
struct MeshUploadBudgetLabel;

#[derive(Component)]
struct GodRaysLabel;

#[derive(Component)]
struct TransmissionLabel;

#[derive(Component)]
struct AntiAliasingLabel;

#[derive(Component)]
struct ShadowQualityLabel;

#[derive(Component)]
struct CloudQualityLabel;

#[derive(Component)]
struct ScreenModeLabel;

#[derive(Component)]
struct RenderDistanceLabel;

#[derive(Component)]
struct LodDistanceLabel;

#[derive(Component)]
struct SimulationDistanceLabel;

#[derive(Component)]
struct FovLabel;

#[derive(Component)]
struct DynamicFovLabel;

#[derive(Component)]
struct FogLabel;

#[derive(Component)]
struct ViewBobbingLabel;

#[derive(Component)]
struct FullGrassLabel;

#[derive(Component)]
struct VsyncLabel;

#[derive(Component)]
struct DynamicFpsLabel;

#[derive(Component)]
struct TimePauseLabel;

#[derive(Component)]
struct MasterVolLabel;

#[derive(Component)]
struct MusicVolLabel;

#[derive(Component)]
struct WeatherVolLabel;

#[derive(Component)]
struct SfxVolLabel;

#[derive(Component)]
struct AmbientVolLabel;

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsTab>()
            .add_systems(OnEnter(MenuState::Settings), spawn_settings_menu)
            .add_systems(OnExit(MenuState::Settings), despawn_settings_menu)
            .add_systems(
                Update,
                (
                    handle_settings_buttons,
                    update_settings_labels,
                    update_audio_settings_labels,
                    update_settings_tab_view,
                )
                    .run_if(in_state(MenuState::Settings)),
            );
    }
}

fn spawn_settings_menu(
    mut commands: Commands,
    chunk_settings: Res<ChunkStreamingSettings>,
    game_settings: Res<GameSettings>,
    local_config: Res<LocalConfig>,
    audio_settings: Res<AudioSettings>,
    settings_tab: Res<SettingsTab>,
    env_state: Option<Res<EnvironmentState>>,
    app_font: Option<Res<AppFont>>,
    asset_server: Res<AssetServer>,
) {
    let render_dist = chunk_settings.render_distance;
    let lod_dist = chunk_settings.lod_render_distance;
    let sim_dist = chunk_settings.simulation_distance;
    let fov = game_settings.fov_degrees as i32;
    let fog_enabled = game_settings.fog_enabled;
    let time_paused = env_state
        .as_ref()
        .map_or(local_config.is_time_paused, |e| e.is_time_paused);
    let font_handle = app_font.as_ref().map(|f| f.source());

    let btn_normal = asset_server.load("textures/interfaces/containers/buttons/button_medium.png");
    let btn_hover = asset_server.load("textures/interfaces/containers/buttons/button_medium_hover.png");
    let small_normal = asset_server.load("textures/interfaces/containers/buttons/button_small.png");
    let small_hover = asset_server.load("textures/interfaces/containers/buttons/button_small_hover.png");

    let mut header_font = TextFont {
        font_size: FontSize::Px(32.0),
        ..default()
    };
    if let Some(ref font) = font_handle {
        header_font.font = font.clone();
    }

    commands
        .spawn((
            SettingsMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(80.0),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(16.0),
                    ..default()
                })
                .with_children(|card| {
                    // Header Title
                    card.spawn((
                        Text::new("SETTINGS"),
                        header_font,
                        TextColor(Color::srgb(0.95, 0.95, 0.98)),
                        text_shadow_default(),
                        Node {
                            margin: UiRect::bottom(px(4.0)),
                            ..default()
                        },
                    ));

                    // Tab Navigation Bar
                    card.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        column_gap: px(12.0),
                        margin: UiRect::bottom(px(6.0)),
                        ..default()
                    })
                    .with_children(|tab_bar| {
                        spawn_tab_button(
                            tab_bar,
                            "Video & Graphics",
                            SettingsTab::Video,
                            *settings_tab == SettingsTab::Video,
                            font_handle.as_ref(),
                            &btn_normal,
                            &btn_hover,
                        );
                        spawn_tab_button(
                            tab_bar,
                            "Audio & Sounds",
                            SettingsTab::Audio,
                            *settings_tab == SettingsTab::Audio,
                            font_handle.as_ref(),
                            &btn_normal,
                            &btn_hover,
                        );
                    });

                    // 2-Column Video Settings Container
                    card.spawn((
                        VideoSettingsSection,
                        Node {
                            display: if *settings_tab == SettingsTab::Video {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            flex_direction: FlexDirection::Row,
                            column_gap: px(24.0),
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                    ))
                    .with_children(|columns| {
                        // Left Column: Display & World Settings
                        columns
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(10.0),
                                width: px(330.0),
                                ..default()
                            })
                            .with_children(|col_left| {
                                // 0. Screen Mode Stepper
                                spawn_stepper_row(
                                    col_left,
                                    "Screen Mode",
                                    game_settings.screen_mode.label().to_string(),
                                    SettingsAction::DecScreenMode,
                                    SettingsAction::IncScreenMode,
                                    ScreenModeLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 1. Target FPS Stepper
                                spawn_stepper_row(
                                    col_left,
                                    "Target FPS",
                                    local_config.target_fps.label().to_string(),
                                    SettingsAction::DecTargetFps,
                                    SettingsAction::IncTargetFps,
                                    TargetFpsLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 2. VSync Toggle Button
                                spawn_toggle_button(
                                    col_left,
                                    SettingsAction::ToggleVsync,
                                    format!(
                                        "VSync: {}",
                                        if game_settings.vsync_enabled {
                                            "Enabled"
                                        } else {
                                            "Disabled"
                                        }
                                    ),
                                    VsyncLabel,
                                    font_handle.as_ref(),
                                    &btn_normal,
                                    &btn_hover,
                                );

                                // 3. Dynamic FPS Toggle Button
                                spawn_toggle_button(
                                    col_left,
                                    SettingsAction::ToggleDynamicFps,
                                    format!(
                                        "Dynamic FPS: {}",
                                        if game_settings.dynamic_fps_enabled {
                                            "Enabled"
                                        } else {
                                            "Disabled"
                                        }
                                    ),
                                    DynamicFpsLabel,
                                    font_handle.as_ref(),
                                    &btn_normal,
                                    &btn_hover,
                                );

                                // 4. Render Distance Stepper
                                spawn_stepper_row(
                                    col_left,
                                    "Render Distance",
                                    format!("{render_dist} Chunks"),
                                    SettingsAction::DecRenderDistance,
                                    SettingsAction::IncRenderDistance,
                                    RenderDistanceLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 5. LOD Distance Stepper
                                spawn_stepper_row(
                                    col_left,
                                    "LOD Distance",
                                    if lod_dist == 0 {
                                        "Disabled".to_string()
                                    } else {
                                        format!("{lod_dist} Chunks")
                                    },
                                    SettingsAction::DecLodDistance,
                                    SettingsAction::IncLodDistance,
                                    LodDistanceLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 6. Simulation Distance Stepper
                                spawn_stepper_row(
                                    col_left,
                                    "Simulation Distance",
                                    format!("{sim_dist} Chunks"),
                                    SettingsAction::DecSimulationDistance,
                                    SettingsAction::IncSimulationDistance,
                                    SimulationDistanceLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 7. Mesh Upload Budget Stepper
                                spawn_stepper_row(
                                    col_left,
                                    "Mesh Uploads",
                                    format!("{} Chunks", local_config.mesh_upload_budget),
                                    SettingsAction::DecMeshUploadBudget,
                                    SettingsAction::IncMeshUploadBudget,
                                    MeshUploadBudgetLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 8. Field of View Stepper
                                spawn_stepper_row(
                                    col_left,
                                    "Field of View",
                                    format!("{fov}°"),
                                    SettingsAction::DecFov,
                                    SettingsAction::IncFov,
                                    FovLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 9. Dynamic FOV / Reduced Motion Toggle Button
                                spawn_toggle_button(
                                    col_left,
                                    SettingsAction::ToggleDynamicFov,
                                    format!(
                                        "FOV Effects: {}",
                                        if game_settings.dynamic_fov_enabled {
                                            "Normal"
                                        } else {
                                            "Reduced"
                                        }
                                    ),
                                    DynamicFovLabel,
                                    font_handle.as_ref(),
                                    &btn_normal,
                                    &btn_hover,
                                );

                                // 10. View Bobbing Toggle Button
                                spawn_toggle_button(
                                    col_left,
                                    SettingsAction::ToggleViewBobbing,
                                    format!(
                                        "View Bobbing: {}",
                                        if game_settings.view_bobbing {
                                            "Enabled"
                                        } else {
                                            "Disabled"
                                        }
                                    ),
                                    ViewBobbingLabel,
                                    font_handle.as_ref(),
                                    &btn_normal,
                                    &btn_hover,
                                );
                            });

                        // Right Column: Performance & Graphics Tweaks
                        columns
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(10.0),
                                width: px(330.0),
                                ..default()
                            })
                            .with_children(|col_right| {
                                // 0. Performance Preset Stepper
                                let preset_label_str = local_config
                                    .preset
                                    .map_or("Custom".to_string(), |p| p.short_name().to_string());
                                spawn_stepper_row(
                                    col_right,
                                    "Preset",
                                    preset_label_str,
                                    SettingsAction::DecPreset,
                                    SettingsAction::IncPreset,
                                    PresetLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 1. Render Scale Stepper
                                spawn_stepper_row(
                                    col_right,
                                    "Render Scale",
                                    crate::core::config::format_render_scale(
                                        local_config.render_scale,
                                    ),
                                    SettingsAction::DecRenderScale,
                                    SettingsAction::IncRenderScale,
                                    RenderScaleLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 2. Upscaling Filter Stepper
                                spawn_stepper_row(
                                    col_right,
                                    "Upscaling Filter",
                                    local_config.upscaling_filter.label().to_string(),
                                    SettingsAction::DecUpscalingFilter,
                                    SettingsAction::IncUpscalingFilter,
                                    UpscalingFilterLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 3. Anti-Aliasing Stepper
                                spawn_stepper_row(
                                    col_right,
                                    "Anti-Aliasing",
                                    local_config.anti_aliasing.label().to_string(),
                                    SettingsAction::DecAntiAliasing,
                                    SettingsAction::IncAntiAliasing,
                                    AntiAliasingLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 4. Shadow Quality Stepper
                                spawn_stepper_row(
                                    col_right,
                                    "Shadows",
                                    local_config.shadow_quality.label().to_string(),
                                    SettingsAction::DecShadowQuality,
                                    SettingsAction::IncShadowQuality,
                                    ShadowQualityLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 5. Cloud Quality Stepper
                                spawn_stepper_row(
                                    col_right,
                                    "Clouds",
                                    local_config.cloud_quality.label().to_string(),
                                    SettingsAction::DecCloudQuality,
                                    SettingsAction::IncCloudQuality,
                                    CloudQualityLabel,
                                    font_handle.as_ref(),
                                    &small_normal,
                                    &small_hover,
                                );

                                // 6. God Rays Toggle Button
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleGodRays,
                                    format!(
                                        "God Rays: {}",
                                        if local_config.god_rays_enabled {
                                            "Enabled"
                                        } else {
                                            "Disabled"
                                        }
                                    ),
                                    GodRaysLabel,
                                    font_handle.as_ref(),
                                    &btn_normal,
                                    &btn_hover,
                                );

                                // 7. Fog Toggle Button
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleFog,
                                    format!(
                                        "Fog: {}",
                                        if fog_enabled { "Enabled" } else { "Disabled" }
                                    ),
                                    FogLabel,
                                    font_handle.as_ref(),
                                    &btn_normal,
                                    &btn_hover,
                                );

                                // 8. Transmission Toggle Button
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleTransmission,
                                    format!(
                                        "Transmission: {}",
                                        if local_config.transmission_enabled {
                                            "Enabled"
                                        } else {
                                            "Disabled"
                                        }
                                    ),
                                    TransmissionLabel,
                                    font_handle.as_ref(),
                                    &btn_normal,
                                    &btn_hover,
                                );

                                // 9. Full Surface Sides Toggle Button
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleFullGrass,
                                    format!(
                                        "Full Surface Sides: {}",
                                        if game_settings.full_grass {
                                            "Enabled"
                                        } else {
                                            "Disabled"
                                        }
                                    ),
                                    FullGrassLabel,
                                    font_handle.as_ref(),
                                    &btn_normal,
                                    &btn_hover,
                                );

                                // 10. Time Pause Toggle Button
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleTimePause,
                                    format!(
                                        "Time Flow: {}",
                                        if time_paused { "Paused" } else { "Running" }
                                    ),
                                    TimePauseLabel,
                                    font_handle.as_ref(),
                                    &btn_normal,
                                    &btn_hover,
                                );
                            });
                    });

                    // Audio Settings Container
                    card.spawn((
                        AudioSettingsSection,
                        Node {
                            display: if *settings_tab == SettingsTab::Audio {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(12.0),
                            width: px(380.0),
                            margin: UiRect::vertical(px(20.0)),
                            ..default()
                        },
                    ))
                    .with_children(|col_audio| {
                        spawn_stepper_row(
                            col_audio,
                            "Master Volume",
                            crate::core::config::format_volume(audio_settings.master_volume),
                            SettingsAction::DecMasterVol,
                            SettingsAction::IncMasterVol,
                            MasterVolLabel,
                            font_handle.as_ref(),
                            &small_normal,
                            &small_hover,
                        );

                        spawn_stepper_row(
                            col_audio,
                            "Music Volume",
                            crate::core::config::format_volume(audio_settings.music_volume),
                            SettingsAction::DecMusicVol,
                            SettingsAction::IncMusicVol,
                            MusicVolLabel,
                            font_handle.as_ref(),
                            &small_normal,
                            &small_hover,
                        );

                        spawn_stepper_row(
                            col_audio,
                            "Weather Volume",
                            crate::core::config::format_volume(audio_settings.weather_volume),
                            SettingsAction::DecWeatherVol,
                            SettingsAction::IncWeatherVol,
                            WeatherVolLabel,
                            font_handle.as_ref(),
                            &small_normal,
                            &small_hover,
                        );

                        spawn_stepper_row(
                            col_audio,
                            "SFX Volume",
                            crate::core::config::format_volume(audio_settings.sfx_volume),
                            SettingsAction::DecSfxVol,
                            SettingsAction::IncSfxVol,
                            SfxVolLabel,
                            font_handle.as_ref(),
                            &small_normal,
                            &small_hover,
                        );

                        spawn_stepper_row(
                            col_audio,
                            "Ambient Volume",
                            crate::core::config::format_volume(audio_settings.ambient_volume),
                            SettingsAction::DecAmbientVol,
                            SettingsAction::IncAmbientVol,
                            AmbientVolLabel,
                            font_handle.as_ref(),
                            &small_normal,
                            &small_hover,
                        );
                    });

                    // Centered Back Button
                    let mut back_font = TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    };
                    if let Some(ref font) = font_handle {
                        back_font.font = font.clone();
                    }

                    card.spawn((
                        Button,
                        Hovered::default(),
                        SettingsAction::Back,
                        crate::menu::MenuButtonTexture {
                            normal: btn_normal.clone(),
                            hover: btn_hover.clone(),
                        },
                        ImageNode {
                            image: btn_normal.clone(),
                            image_mode: NodeImageMode::Sliced(TextureSlicer {
                                border: BorderRect::all(4.0),
                                ..default()
                            }),
                            ..default()
                        },
                        Node {
                            width: px(330.0),
                            height: px(38.0),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(px(10.0)),
                            ..default()
                        },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Back"),
                            back_font,
                            TextColor(Color::srgb(0.95, 0.95, 0.98)),
                            text_shadow_default(),
                        ));
                    });
                });
        });
}

fn spawn_stepper_row<T: Component>(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    initial_val: String,
    dec_action: SettingsAction,
    inc_action: SettingsAction,
    marker: T,
    font_handle: Option<&crate::core::FontSource>,
    small_normal: &Handle<Image>,
    small_hover: &Handle<Image>,
) {
    let mut title_font = TextFont {
        font_size: FontSize::Px(15.0),
        ..default()
    };
    let mut btn_font = TextFont {
        font_size: FontSize::Px(15.0),
        ..default()
    };
    let mut val_font = TextFont {
        font_size: FontSize::Px(15.0),
        ..default()
    };
    if let Some(font) = font_handle {
        title_font.font = font.clone();
        btn_font.font = font.clone();
        val_font.font = font.clone();
    }

    parent
        .spawn(Node {
            width: px(330.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(6.0), px(3.0)),
            ..default()
        })
        .with_children(|row| {
            // Label on left
            row.spawn((
                Text::new(title),
                title_font,
                TextColor(Color::srgb(0.85, 0.85, 0.88)),
                text_shadow_default(),
            ));

            // Controls on right: [-] Value [+]
            row.spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(6.0),
                ..default()
            })
            .with_children(|controls| {
                // Decrement button
                controls
                    .spawn((
                        Button,
                        Hovered::default(),
                        dec_action,
                        crate::menu::MenuButtonTexture {
                            normal: small_normal.clone(),
                            hover: small_hover.clone(),
                        },
                        ImageNode {
                            image: small_normal.clone(),
                            ..default()
                        },
                        Node {
                            width: px(28.0),
                            height: px(28.0),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("-"),
                            btn_font.clone(),
                            TextColor(Color::WHITE),
                            text_shadow_default(),
                        ));
                    });

                // Value Text
                controls.spawn((
                    marker,
                    Text::new(initial_val),
                    val_font,
                    TextColor(Color::srgb(1.0, 0.88, 0.45)),
                    text_shadow_default(),
                    Node {
                        width: px(125.0),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ));

                // Increment button
                controls
                    .spawn((
                        Button,
                        Hovered::default(),
                        inc_action,
                        crate::menu::MenuButtonTexture {
                            normal: small_normal.clone(),
                            hover: small_hover.clone(),
                        },
                        ImageNode {
                            image: small_normal.clone(),
                            ..default()
                        },
                        Node {
                            width: px(28.0),
                            height: px(28.0),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("+"),
                            btn_font,
                            TextColor(Color::WHITE),
                            text_shadow_default(),
                        ));
                    });
            });
        });
}

fn spawn_toggle_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    action: SettingsAction,
    label: String,
    marker: T,
    font_handle: Option<&crate::core::FontSource>,
    btn_normal: &Handle<Image>,
    btn_hover: &Handle<Image>,
) {
    let mut btn_font = TextFont {
        font_size: FontSize::Px(14.5),
        ..default()
    };
    if let Some(font) = font_handle {
        btn_font.font = font.clone();
    }

    parent
        .spawn((
            Button,
            Hovered::default(),
            action,
            crate::menu::MenuButtonTexture {
                normal: btn_normal.clone(),
                hover: btn_hover.clone(),
            },
            ImageNode {
                image: btn_normal.clone(),
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(4.0),
                    ..default()
                }),
                ..default()
            },
            Node {
                width: px(330.0),
                height: px(36.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|btn| {
            btn.spawn((
                marker,
                Text::new(label),
                btn_font,
                TextColor(Color::srgb(0.95, 0.95, 0.98)),
                text_shadow_default(),
            ));
        });
}

fn spawn_tab_button(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    tab: SettingsTab,
    is_active: bool,
    font_handle: Option<&crate::core::FontSource>,
    btn_normal: &Handle<Image>,
    btn_hover: &Handle<Image>,
) {
    let mut btn_font = TextFont {
        font_size: FontSize::Px(15.0),
        ..default()
    };
    if let Some(font) = font_handle {
        btn_font.font = font.clone();
    }

    let text_color = if is_active {
        Color::srgb(1.0, 0.88, 0.45)
    } else {
        Color::srgb(0.78, 0.78, 0.82)
    };

    parent
        .spawn((
            Button,
            Hovered::default(),
            SettingsAction::SwitchTab(tab),
            TabButton(tab),
            crate::menu::MenuButtonTexture {
                normal: btn_normal.clone(),
                hover: btn_hover.clone(),
            },
            ImageNode {
                image: btn_normal.clone(),
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(4.0),
                    ..default()
                }),
                ..default()
            },
            Node {
                width: px(170.0),
                height: px(34.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(title),
                btn_font,
                TextColor(text_color),
                text_shadow_default(),
            ));
        });
}

fn despawn_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_settings_buttons(
    mut interaction_query: Query<
        (
            &Hovered,
            Has<Pressed>,
            &SettingsAction,
            &crate::menu::MenuButtonTexture,
            &mut ImageNode,
        ),
        With<Button>,
    >,
    mouse: Res<ButtonInput<MouseButton>>,
    mut chunk_settings: ResMut<ChunkStreamingSettings>,
    mut game_settings: ResMut<GameSettings>,
    mut local_config: ResMut<LocalConfig>,
    mut audio_settings: ResMut<AudioSettings>,
    mut settings_tab: ResMut<SettingsTab>,
    mut env_state: Option<ResMut<EnvironmentState>>,
    mut dynamic_fps_settings: Option<ResMut<DynamicFpsSettings>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut chunk_material: Option<ResMut<ChunkMaterial>>,
    mut texture_registry: Option<ResMut<VoxelTextureRegistry>>,
    mut streaming_queues: Option<ResMut<ChunkStreamingQueues>>,
    world: Option<Res<VoxelWorld>>,
    mut window_query: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    let left_clicked = mouse.just_pressed(MouseButton::Left);

    for (hovered, is_pressed, action, btn_tex, mut img) in &mut interaction_query {
        let is_hovered = hovered.get();
        if is_hovered || is_pressed {
            img.image = btn_tex.hover.clone();
        } else {
            img.image = btn_tex.normal.clone();
        }

        if is_hovered && left_clicked {
            let mut sync_config = true;
            match action {
                SettingsAction::SwitchTab(tab) => {
                    *settings_tab = *tab;
                    sync_config = false;
                }
                SettingsAction::DecMasterVol => {
                    local_config.master_volume =
                        crate::core::config::step_volume_down(local_config.master_volume);
                    audio_settings.master_volume = local_config.master_volume;
                }
                SettingsAction::IncMasterVol => {
                    local_config.master_volume =
                        crate::core::config::step_volume_up(local_config.master_volume);
                    audio_settings.master_volume = local_config.master_volume;
                }
                SettingsAction::DecMusicVol => {
                    local_config.music_volume =
                        crate::core::config::step_volume_down(local_config.music_volume);
                    audio_settings.music_volume = local_config.music_volume;
                }
                SettingsAction::IncMusicVol => {
                    local_config.music_volume =
                        crate::core::config::step_volume_up(local_config.music_volume);
                    audio_settings.music_volume = local_config.music_volume;
                }
                SettingsAction::DecWeatherVol => {
                    local_config.weather_volume =
                        crate::core::config::step_volume_down(local_config.weather_volume);
                    audio_settings.weather_volume = local_config.weather_volume;
                }
                SettingsAction::IncWeatherVol => {
                    local_config.weather_volume =
                        crate::core::config::step_volume_up(local_config.weather_volume);
                    audio_settings.weather_volume = local_config.weather_volume;
                }
                SettingsAction::DecSfxVol => {
                    local_config.sfx_volume =
                        crate::core::config::step_volume_down(local_config.sfx_volume);
                    audio_settings.sfx_volume = local_config.sfx_volume;
                }
                SettingsAction::IncSfxVol => {
                    local_config.sfx_volume =
                        crate::core::config::step_volume_up(local_config.sfx_volume);
                    audio_settings.sfx_volume = local_config.sfx_volume;
                }
                SettingsAction::DecAmbientVol => {
                    local_config.ambient_volume =
                        crate::core::config::step_volume_down(local_config.ambient_volume);
                    audio_settings.ambient_volume = local_config.ambient_volume;
                }
                SettingsAction::IncAmbientVol => {
                    local_config.ambient_volume =
                        crate::core::config::step_volume_up(local_config.ambient_volume);
                    audio_settings.ambient_volume = local_config.ambient_volume;
                }
                SettingsAction::DecPreset => {
                    let current = local_config.preset.unwrap_or(PerformancePreset::Custom);
                    let prev = current.prev();
                    local_config.apply_preset(prev);
                    chunk_settings.render_distance = local_config.render_distance;
                    chunk_settings.lod_render_distance = local_config.lod_render_distance;
                    chunk_settings.simulation_distance = local_config.simulation_distance;
                    chunk_settings.mesh_upload_budget = local_config.mesh_upload_budget;
                    game_settings.fog_enabled = local_config.fog_enabled;
                    game_settings.view_bobbing = local_config.view_bobbing;
                    game_settings.full_grass = local_config.full_grass;
                    game_settings.vsync_enabled = local_config.vsync_enabled;
                    game_settings.dynamic_fps_enabled = local_config.dynamic_fps_enabled;
                    game_settings.dynamic_fov_enabled = local_config.dynamic_fov_enabled;

                    if let Ok(mut window) = window_query.single_mut() {
                        window.present_mode = if game_settings.vsync_enabled {
                            bevy::window::PresentMode::AutoVsync
                        } else {
                            bevy::window::PresentMode::AutoNoVsync
                        };
                    }
                    if let Some(ref mut dyn_fps) = dynamic_fps_settings {
                        dyn_fps.enabled = game_settings.dynamic_fps_enabled;
                        dyn_fps.active_target_fps = local_config.target_fps.to_fps();
                    }
                    if let Some(ref mut reg) = texture_registry {
                        reg.full_grass = game_settings.full_grass;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.full_grass = game_settings.full_grass;
                    }
                    if let Some(ref world) = world
                        && let Some(ref mut queues) = streaming_queues
                    {
                        for (&coord, _) in world.iter_chunks() {
                            queues.enqueue_remesh(coord);
                        }
                    }
                    let _ = local_config.save();
                    sync_config = false;
                }
                SettingsAction::IncPreset => {
                    let current = local_config.preset.unwrap_or(PerformancePreset::Custom);
                    let next = current.next();
                    local_config.apply_preset(next);
                    chunk_settings.render_distance = local_config.render_distance;
                    chunk_settings.lod_render_distance = local_config.lod_render_distance;
                    chunk_settings.simulation_distance = local_config.simulation_distance;
                    chunk_settings.mesh_upload_budget = local_config.mesh_upload_budget;
                    game_settings.fog_enabled = local_config.fog_enabled;
                    game_settings.view_bobbing = local_config.view_bobbing;
                    game_settings.full_grass = local_config.full_grass;
                    game_settings.vsync_enabled = local_config.vsync_enabled;
                    game_settings.dynamic_fps_enabled = local_config.dynamic_fps_enabled;
                    game_settings.dynamic_fov_enabled = local_config.dynamic_fov_enabled;

                    if let Ok(mut window) = window_query.single_mut() {
                        window.present_mode = if game_settings.vsync_enabled {
                            bevy::window::PresentMode::AutoVsync
                        } else {
                            bevy::window::PresentMode::AutoNoVsync
                        };
                    }
                    if let Some(ref mut dyn_fps) = dynamic_fps_settings {
                        dyn_fps.enabled = game_settings.dynamic_fps_enabled;
                        dyn_fps.active_target_fps = local_config.target_fps.to_fps();
                    }
                    if let Some(ref mut reg) = texture_registry {
                        reg.full_grass = game_settings.full_grass;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.full_grass = game_settings.full_grass;
                    }
                    if let Some(ref world) = world
                        && let Some(ref mut queues) = streaming_queues
                    {
                        for (&coord, _) in world.iter_chunks() {
                            queues.enqueue_remesh(coord);
                        }
                    }
                    let _ = local_config.save();
                    sync_config = false;
                }
                SettingsAction::DecRenderScale => {
                    local_config.render_scale =
                        crate::core::config::prev_render_scale(local_config.render_scale);
                }
                SettingsAction::IncRenderScale => {
                    local_config.render_scale =
                        crate::core::config::next_render_scale(local_config.render_scale);
                }
                SettingsAction::DecUpscalingFilter => {
                    local_config.upscaling_filter = local_config.upscaling_filter.prev();
                }
                SettingsAction::IncUpscalingFilter => {
                    local_config.upscaling_filter = local_config.upscaling_filter.next();
                }
                SettingsAction::DecTargetFps => {
                    local_config.target_fps = local_config.target_fps.prev();
                    if let Some(ref mut dyn_fps) = dynamic_fps_settings {
                        dyn_fps.active_target_fps = local_config.target_fps.to_fps();
                    }
                }
                SettingsAction::IncTargetFps => {
                    local_config.target_fps = local_config.target_fps.next();
                    if let Some(ref mut dyn_fps) = dynamic_fps_settings {
                        dyn_fps.active_target_fps = local_config.target_fps.to_fps();
                    }
                }
                SettingsAction::DecMeshUploadBudget => {
                    local_config.mesh_upload_budget =
                        crate::core::config::prev_mesh_upload_budget(local_config.mesh_upload_budget);
                    chunk_settings.mesh_upload_budget = local_config.mesh_upload_budget;
                }
                SettingsAction::IncMeshUploadBudget => {
                    local_config.mesh_upload_budget =
                        crate::core::config::next_mesh_upload_budget(local_config.mesh_upload_budget);
                    chunk_settings.mesh_upload_budget = local_config.mesh_upload_budget;
                }
                SettingsAction::ToggleGodRays => {
                    local_config.god_rays_enabled = !local_config.god_rays_enabled;
                }
                SettingsAction::ToggleTransmission => {
                    local_config.transmission_enabled = !local_config.transmission_enabled;
                }
                SettingsAction::DecAntiAliasing => {
                    local_config.anti_aliasing = local_config.anti_aliasing.prev();
                }
                SettingsAction::IncAntiAliasing => {
                    local_config.anti_aliasing = local_config.anti_aliasing.next();
                }
                SettingsAction::DecShadowQuality => {
                    local_config.shadow_quality = local_config.shadow_quality.prev();
                    local_config.shadows_enabled =
                        local_config.shadow_quality != crate::core::ShadowQuality::Off;
                }
                SettingsAction::IncShadowQuality => {
                    local_config.shadow_quality = local_config.shadow_quality.next();
                    local_config.shadows_enabled =
                        local_config.shadow_quality != crate::core::ShadowQuality::Off;
                }
                SettingsAction::DecCloudQuality => {
                    local_config.cloud_quality = local_config.cloud_quality.prev();
                }
                SettingsAction::IncCloudQuality => {
                    local_config.cloud_quality = local_config.cloud_quality.next();
                }
                SettingsAction::ToggleDynamicFov => {
                    game_settings.dynamic_fov_enabled = !game_settings.dynamic_fov_enabled;
                }
                SettingsAction::DecScreenMode => {
                    game_settings.screen_mode = game_settings.screen_mode.prev();
                    if let Ok(mut window) = window_query.single_mut() {
                        window.mode = game_settings.screen_mode.to_window_mode();
                    }
                }
                SettingsAction::IncScreenMode => {
                    game_settings.screen_mode = game_settings.screen_mode.next();
                    if let Ok(mut window) = window_query.single_mut() {
                        window.mode = game_settings.screen_mode.to_window_mode();
                    }
                }
                SettingsAction::DecRenderDistance => {
                    chunk_settings.render_distance = (chunk_settings.render_distance - 1).max(2);
                }
                SettingsAction::IncRenderDistance => {
                    chunk_settings.render_distance = (chunk_settings.render_distance + 1).min(16);
                }
                SettingsAction::DecLodDistance => {
                    chunk_settings.lod_render_distance =
                        (chunk_settings.lod_render_distance - 2).max(0);
                }
                SettingsAction::IncLodDistance => {
                    chunk_settings.lod_render_distance =
                        (chunk_settings.lod_render_distance + 2).min(32);
                }
                SettingsAction::DecSimulationDistance => {
                    chunk_settings.simulation_distance =
                        (chunk_settings.simulation_distance - 1).max(2);
                }
                SettingsAction::IncSimulationDistance => {
                    chunk_settings.simulation_distance =
                        (chunk_settings.simulation_distance + 1).min(8);
                }
                SettingsAction::DecFov => {
                    game_settings.fov_degrees = (game_settings.fov_degrees - 5.0).max(60.0);
                }
                SettingsAction::IncFov => {
                    game_settings.fov_degrees = (game_settings.fov_degrees + 5.0).min(110.0);
                }
                SettingsAction::ToggleFog => {
                    game_settings.fog_enabled = !game_settings.fog_enabled;
                }
                SettingsAction::ToggleViewBobbing => {
                    game_settings.view_bobbing = !game_settings.view_bobbing;
                }
                SettingsAction::ToggleFullGrass => {
                    game_settings.full_grass = !game_settings.full_grass;
                    if let Some(ref mut reg) = texture_registry {
                        reg.full_grass = game_settings.full_grass;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.full_grass = game_settings.full_grass;
                    }
                    if let Some(ref world) = world
                        && let Some(ref mut queues) = streaming_queues
                    {
                        for (&coord, _) in world.iter_chunks() {
                            queues.enqueue_remesh(coord);
                        }
                    }
                }
                SettingsAction::ToggleVsync => {
                    game_settings.vsync_enabled = !game_settings.vsync_enabled;
                    if let Ok(mut window) = window_query.single_mut() {
                        window.present_mode = if game_settings.vsync_enabled {
                            bevy::window::PresentMode::AutoVsync
                        } else {
                            bevy::window::PresentMode::AutoNoVsync
                        };
                    }
                }
                SettingsAction::ToggleDynamicFps => {
                    game_settings.dynamic_fps_enabled = !game_settings.dynamic_fps_enabled;
                    if let Some(ref mut dyn_fps) = dynamic_fps_settings {
                        dyn_fps.enabled = game_settings.dynamic_fps_enabled;
                    }
                }
                SettingsAction::ToggleTimePause => {
                    if let Some(ref mut env) = env_state {
                        env.is_time_paused = !env.is_time_paused;
                        local_config.is_time_paused = env.is_time_paused;
                    }
                    let _ = local_config.save();
                    sync_config = false;
                }
                SettingsAction::Back => {
                    next_state.set(MenuState::Pause);
                    sync_config = false;
                }
            }

            if sync_config {
                local_config.preset = Some(PerformancePreset::Custom);
                local_config.render_distance = chunk_settings.render_distance;
                local_config.lod_render_distance = chunk_settings.lod_render_distance;
                local_config.simulation_distance = chunk_settings.simulation_distance;
                local_config.mesh_upload_budget = chunk_settings.mesh_upload_budget;
                local_config.fov_degrees = game_settings.fov_degrees;
                local_config.fog_enabled = game_settings.fog_enabled;
                local_config.view_bobbing = game_settings.view_bobbing;
                local_config.full_grass = game_settings.full_grass;
                local_config.vsync_enabled = game_settings.vsync_enabled;
                local_config.dynamic_fps_enabled = game_settings.dynamic_fps_enabled;
                local_config.dynamic_fov_enabled = game_settings.dynamic_fov_enabled;
                local_config.screen_mode = game_settings.screen_mode.label().to_lowercase();
                let _ = local_config.save();
            }
        }
    }
}

fn update_settings_labels(
    chunk_settings: Res<ChunkStreamingSettings>,
    game_settings: Res<GameSettings>,
    local_config: Res<LocalConfig>,
    env_state: Option<Res<EnvironmentState>>,
    mut labels_query: Query<(
        &mut Text,
        (
            Option<&ScreenModeLabel>,
            Option<&TargetFpsLabel>,
            Option<&VsyncLabel>,
            Option<&DynamicFpsLabel>,
            Option<&RenderDistanceLabel>,
            Option<&LodDistanceLabel>,
            Option<&SimulationDistanceLabel>,
            Option<&MeshUploadBudgetLabel>,
            Option<&FovLabel>,
            Option<&DynamicFovLabel>,
            Option<&ViewBobbingLabel>,
        ),
        (
            Option<&PresetLabel>,
            Option<&RenderScaleLabel>,
            Option<&UpscalingFilterLabel>,
            Option<&AntiAliasingLabel>,
            Option<&ShadowQualityLabel>,
            Option<&CloudQualityLabel>,
            Option<&GodRaysLabel>,
            Option<&FogLabel>,
            Option<&TransmissionLabel>,
            Option<&FullGrassLabel>,
            Option<&TimePauseLabel>,
        ),
    )>,
) {
    let chunk_changed = chunk_settings.is_changed();
    let game_changed = game_settings.is_changed();
    let config_changed = local_config.is_changed();
    let env_changed = env_state.as_ref().is_some_and(|e| e.is_changed());

    if !chunk_changed && !game_changed && !config_changed && !env_changed {
        return;
    }

    for (
        mut text,
        (
            screen_mode,
            target_fps,
            vsync,
            dyn_fps,
            render_dist,
            lod_dist,
            sim_dist,
            mesh_uploads,
            fov,
            dynamic_fov,
            bobbing,
        ),
        (
            preset,
            render_scale,
            upscaling_filter,
            anti_aliasing,
            shadow_quality,
            cloud_quality,
            god_rays,
            fog,
            transmission,
            grass,
            time_pause,
        ),
    ) in &mut labels_query
    {
        if config_changed {
            if preset.is_some() {
                text.0 = local_config
                    .preset
                    .map_or("Custom".to_string(), |p| p.short_name().to_string());
            } else if render_scale.is_some() {
                text.0 = crate::core::config::format_render_scale(local_config.render_scale);
            } else if upscaling_filter.is_some() {
                text.0 = local_config.upscaling_filter.label().to_string();
            } else if target_fps.is_some() {
                text.0 = local_config.target_fps.label().to_string();
            } else if anti_aliasing.is_some() {
                text.0 = local_config.anti_aliasing.label().to_string();
            } else if shadow_quality.is_some() {
                text.0 = local_config.shadow_quality.label().to_string();
            } else if cloud_quality.is_some() {
                text.0 = local_config.cloud_quality.label().to_string();
            } else if god_rays.is_some() {
                text.0 = format!(
                    "God Rays: {}",
                    if local_config.god_rays_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            } else if transmission.is_some() {
                text.0 = format!(
                    "Transmission: {}",
                    if local_config.transmission_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            } else if mesh_uploads.is_some() {
                text.0 = format!("{} Chunks", local_config.mesh_upload_budget);
            }
        }
        if game_changed {
            if screen_mode.is_some() {
                text.0 = game_settings.screen_mode.label().to_string();
            } else if fov.is_some() {
                text.0 = format!("{}°", game_settings.fov_degrees as i32);
            } else if dynamic_fov.is_some() {
                text.0 = format!(
                    "FOV Effects: {}",
                    if game_settings.dynamic_fov_enabled {
                        "Normal"
                    } else {
                        "Reduced"
                    }
                );
            } else if fog.is_some() {
                text.0 = format!(
                    "Fog: {}",
                    if game_settings.fog_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            } else if bobbing.is_some() {
                text.0 = format!(
                    "View Bobbing: {}",
                    if game_settings.view_bobbing {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            } else if grass.is_some() {
                text.0 = format!(
                    "Full Surface Sides: {}",
                    if game_settings.full_grass {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            } else if vsync.is_some() {
                text.0 = format!(
                    "VSync: {}",
                    if game_settings.vsync_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            } else if dyn_fps.is_some() {
                text.0 = format!(
                    "Dynamic FPS: {}",
                    if game_settings.dynamic_fps_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
            }
        }
        if chunk_changed {
            if render_dist.is_some() {
                text.0 = format!("{} Chunks", chunk_settings.render_distance);
            } else if lod_dist.is_some() {
                text.0 = if chunk_settings.lod_render_distance == 0 {
                    "Disabled".to_string()
                } else {
                    format!("{} Chunks", chunk_settings.lod_render_distance)
                };
            } else if sim_dist.is_some() {
                text.0 = format!("{} Chunks", chunk_settings.simulation_distance);
            } else if mesh_uploads.is_some() {
                text.0 = format!("{} Chunks", chunk_settings.mesh_upload_budget);
            }
        }
        if (env_changed || time_pause.is_some()) && time_pause.is_some() {
            let paused = env_state
                .as_ref()
                .map_or(local_config.is_time_paused, |e| e.is_time_paused);
            text.0 = format!("Time Flow: {}", if paused { "Paused" } else { "Running" });
        }
    }
}

fn update_audio_settings_labels(
    audio_settings: Res<AudioSettings>,
    local_config: Res<LocalConfig>,
    mut volume_query: Query<(
        &mut Text,
        Option<&MasterVolLabel>,
        Option<&MusicVolLabel>,
        Option<&WeatherVolLabel>,
        Option<&SfxVolLabel>,
        Option<&AmbientVolLabel>,
    )>,
) {
    if !audio_settings.is_changed() && !local_config.is_changed() {
        return;
    }

    for (mut text, master, music, weather, sfx, ambient) in &mut volume_query {
        if master.is_some() {
            text.0 = crate::core::config::format_volume(audio_settings.master_volume);
        } else if music.is_some() {
            text.0 = crate::core::config::format_volume(audio_settings.music_volume);
        } else if weather.is_some() {
            text.0 = crate::core::config::format_volume(audio_settings.weather_volume);
        } else if sfx.is_some() {
            text.0 = crate::core::config::format_volume(audio_settings.sfx_volume);
        } else if ambient.is_some() {
            text.0 = crate::core::config::format_volume(audio_settings.ambient_volume);
        }
    }
}

fn update_settings_tab_view(
    settings_tab: Res<SettingsTab>,
    tab_buttons_query: Query<(&TabButton, &Children)>,
    mut text_color_query: Query<&mut TextColor>,
    mut video_section_query: Query<
        &mut Node,
        (With<VideoSettingsSection>, Without<AudioSettingsSection>),
    >,
    mut audio_section_query: Query<
        &mut Node,
        (With<AudioSettingsSection>, Without<VideoSettingsSection>),
    >,
) {
    if !settings_tab.is_changed() {
        return;
    }

    for mut node in &mut video_section_query {
        node.display = if *settings_tab == SettingsTab::Video {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in &mut audio_section_query {
        node.display = if *settings_tab == SettingsTab::Audio {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (tab_btn, children) in &tab_buttons_query {
        let is_active = tab_btn.0 == *settings_tab;
        let color = if is_active {
            Color::srgb(1.0, 0.88, 0.45)
        } else {
            Color::srgb(0.78, 0.78, 0.82)
        };
        for child in children.iter() {
            if let Ok(mut text_color) = text_color_query.get_mut(child) {
                text_color.0 = color;
            }
        }
    }
}
