use bevy::prelude::*;
use bevy::{picking::hover::Hovered, ui::Pressed, ui_widgets::Button};

use crate::{
    audio::AudioSettings,
    core::{
        AppFont, DynamicFpsSettings, LocalConfig, Localization, PerformancePreset,
        config::{
            AntiAliasingMode, CloudQuality, CoatedTextureMode, ShadowQuality, TargetFpsLimit,
        },
        text_shadow_default,
    },
    environment::EnvironmentState,
    meshing::{ChunkMaterial, VoxelTextureRegistry},
    world::{ChunkStreamingQueues, ChunkStreamingSettings, VoxelWorld},
};

use super::{
    BackgroundDimming, GameSettings, MenuBlurIntensity, MenuPauseMode, MenuReturnTarget, MenuState,
    ScreenMode,
};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    World,
    Graphics,
    Shaders,
    Camera,
    Audio,
    Accessibility,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsDetailOpen(pub bool);

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
pub struct SettingsMenuCard;

#[derive(Component)]
struct SettingsDetailPanel;

#[derive(Component)]
struct TabButtonLabel;

#[derive(Component)]
struct TabButtonAccent;

#[derive(Component, Clone, Copy)]
struct SettingsSection(SettingsTab);

#[derive(Component)]
struct TabButton(SettingsTab);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum SettingsAction {
    SwitchTab(SettingsTab),
    DecLanguage,
    IncLanguage,
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
    DecCloudDistance,
    IncCloudDistance,
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
    ToggleProceduralGrass,
    CycleGrassStyle,
    DecGrassDensity,
    IncGrassDensity,
    ToggleVsync,
    ToggleDynamicFps,
    ToggleTimePause,
    DecGuiScale,
    IncGuiScale,
    DecMenuBlur,
    IncMenuBlur,
    DecBackgroundDimming,
    IncBackgroundDimming,
    DecCoatedTextures,
    IncCoatedTextures,
    ToggleWavingFoliage,
    ToggleLeafCulling,
    ToggleConnectedTextures,
    DecMenuPause,
    IncMenuPause,
    ToggleMouseLockBorderless,
    DecAutoSave,
    IncAutoSave,
    DecCameraSmoothing,
    IncCameraSmoothing,
    DecThirdPersonDist,
    IncThirdPersonDist,
    ToggleThirdPersonShoulder,
    ToggleThirdPersonFreeOrbit,
    DecUiVol,
    IncUiVol,
    ToggleMuteUnfocus,
    DecFpsHud,
    IncFpsHud,
    ToggleHighContrastCrosshair,
    ToggleDynamicCrosshair,
    ToggleMainHand,
    ToggleSubtitles,
    TogglePixelatedShadows,
    ToggleHdrBloom,
    ToggleDynamicLighting,
    Back,
}

#[derive(Component)]
struct DynamicLightingLabel;

#[derive(Component)]
struct CoatedTexturesLabel;

#[derive(Component)]
struct WavingFoliageLabel;

#[derive(Component)]
struct LeafCullingLabel;

#[derive(Component)]
struct ConnectedTexturesLabel;

#[derive(Component)]
struct MenuPauseLabel;

#[derive(Component)]
struct BorderlessMouseLockRow;

#[derive(Component)]
struct MouseLockBorderlessLabel;

#[derive(Component)]
struct AutoSaveLabel;

#[derive(Component)]
struct CameraSmoothingLabel;

#[derive(Component)]
struct ThirdPersonDistLabel;

#[derive(Component)]
struct ThirdPersonShoulderLabel;

#[derive(Component)]
struct ThirdPersonFreeOrbitLabel;

#[derive(Component)]
struct UiVolLabel;

#[derive(Component)]
struct MuteUnfocusLabel;

#[derive(Component)]
struct FpsHudLabel;

#[derive(Component)]
struct HighContrastCrosshairLabel;

#[derive(Component)]
struct DynamicCrosshairLabel;

#[derive(Component)]
struct MainHandLabel;

#[derive(Component)]
struct SubtitlesLabel;

#[derive(Component)]
struct PixelatedShadowsLabel;

#[derive(Component)]
struct HdrBloomLabel;

#[derive(Component)]
struct ProceduralGrassLabel;

#[derive(Component)]
struct GrassStyleLabel;

#[derive(Component)]
struct GrassDensityLabel;

#[derive(Component)]
struct GuiScaleLabel;

#[derive(Component)]
struct MenuBlurLabel;

#[derive(Component)]
struct BackgroundDimmingLabel;

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
struct CloudDistanceLabel;

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

#[derive(Component)]
struct LanguageLabel;

fn format_coated_textures(mode: CoatedTextureMode, loc: &Localization) -> String {
    match mode {
        CoatedTextureMode::Off => loc.t("coated_textures.off").to_string(),
        CoatedTextureMode::Low => loc.t("coated_textures.low").to_string(),
        CoatedTextureMode::Medium => loc.t("coated_textures.medium").to_string(),
        CoatedTextureMode::High => loc.t("coated_textures.high").to_string(),
        CoatedTextureMode::Ultra => loc.t("coated_textures.ultra").to_string(),
    }
}

fn format_screen_mode(mode: ScreenMode, loc: &Localization) -> String {
    match mode {
        ScreenMode::Windowed => loc.t("screen_mode.windowed").to_string(),
        ScreenMode::ExclusiveFullscreen => loc.t("screen_mode.exclusive").to_string(),
        ScreenMode::BorderlessFullscreen => loc.t("screen_mode.borderless").to_string(),
    }
}

fn format_target_fps(fps: TargetFpsLimit, loc: &Localization) -> String {
    match fps {
        TargetFpsLimit::Unlocked => loc.t("common.unlocked").to_string(),
        _ => fps.label().to_string(),
    }
}

fn format_preset(preset: Option<PerformancePreset>, loc: &Localization) -> String {
    match preset {
        Some(PerformancePreset::Potato) => loc.t("preset.potato").to_string(),
        Some(PerformancePreset::Low) => loc.t("preset.low").to_string(),
        Some(PerformancePreset::Medium) => loc.t("preset.medium").to_string(),
        Some(PerformancePreset::High) => loc.t("preset.high").to_string(),
        Some(PerformancePreset::Ultra) => loc.t("preset.ultra").to_string(),
        None | Some(PerformancePreset::Custom) => loc.t("preset.custom").to_string(),
    }
}

fn format_clouds(clouds: CloudQuality, loc: &Localization) -> String {
    match clouds {
        CloudQuality::Off => loc.t("clouds.off").to_string(),
        CloudQuality::Fast2D => loc.t("clouds.fast").to_string(),
        CloudQuality::Fancy3D => loc.t("clouds.fancy").to_string(),
    }
}

fn format_shadows(shadows: ShadowQuality, loc: &Localization) -> String {
    match shadows {
        ShadowQuality::Off => loc.t("shadows.off").to_string(),
        ShadowQuality::Low => loc.t("shadows.low").to_string(),
        ShadowQuality::Medium => loc.t("shadows.medium").to_string(),
        ShadowQuality::High => loc.t("shadows.high").to_string(),
        ShadowQuality::Ultra => loc.t("shadows.ultra").to_string(),
    }
}

fn format_anti_aliasing(aa: AntiAliasingMode, loc: &Localization) -> String {
    match aa {
        AntiAliasingMode::Off => loc.t("antialiasing.off").to_string(),
        AntiAliasingMode::Fxaa => "FXAA".to_string(),
        AntiAliasingMode::Msaa2x => loc.t("antialiasing.msaa2x").to_string(),
        AntiAliasingMode::Msaa4x => loc.t("antialiasing.msaa4x").to_string(),
    }
}

fn format_blur(blur: MenuBlurIntensity, loc: &Localization) -> String {
    match blur {
        MenuBlurIntensity::Off => loc.t("blur.off").to_string(),
        MenuBlurIntensity::Low => loc.t("blur.low").to_string(),
        MenuBlurIntensity::Medium => loc.t("blur.medium").to_string(),
        MenuBlurIntensity::High => loc.t("blur.high").to_string(),
        MenuBlurIntensity::Ultra => loc.t("blur.ultra").to_string(),
    }
}

fn format_dimming(dimming: BackgroundDimming, loc: &Localization) -> String {
    match dimming {
        BackgroundDimming::Off => loc.t("dimming.off").to_string(),
        BackgroundDimming::Light => loc.t("dimming.light").to_string(),
        BackgroundDimming::Medium => loc.t("dimming.medium").to_string(),
        BackgroundDimming::Dark => loc.t("dimming.dark").to_string(),
        BackgroundDimming::Heavy => loc.t("dimming.heavy").to_string(),
    }
}

fn format_grass_density(density: u32, loc: &Localization) -> String {
    match density {
        0..=16 => loc.t("grass.density_low").to_string(),
        17..=32 => loc.t("grass.density_medium").to_string(),
        33..=48 => loc.t("grass.density_high").to_string(),
        _ => loc.t("grass.density_ultra").to_string(),
    }
}

fn format_grass_style(style: &str, loc: &Localization) -> String {
    match style {
        "segmented" | "Segmented" => loc.t("grass.style_segmented").to_string(),
        _ => loc.t("grass.style_tapered").to_string(),
    }
}

fn format_menu_pause(mode: MenuPauseMode, loc: &Localization) -> String {
    match mode {
        MenuPauseMode::SystemOnly => loc.t("menu_pause.system_only").to_string(),
        MenuPauseMode::AllMenus => loc.t("menu_pause.all_menus").to_string(),
        MenuPauseMode::Never => loc.t("menu_pause.never").to_string(),
    }
}

fn format_fps_hud(mode: crate::core::DebugHudMode, loc: &Localization) -> String {
    match mode {
        crate::core::DebugHudMode::Hidden => loc.t("fps_hud.off").to_string(),
        crate::core::DebugHudMode::FpsOnly => loc.t("fps_hud.fps_only").to_string(),
        crate::core::DebugHudMode::Minimal => loc.t("fps_hud.compact").to_string(),
        crate::core::DebugHudMode::Extended => loc.t("fps_hud.full").to_string(),
    }
}

fn format_camera_smoothing(level: u32, loc: &Localization) -> String {
    match level {
        0 => loc.t("smoothing.off").to_string(),
        1 => loc.t("smoothing.low").to_string(),
        2 => loc.t("smoothing.medium").to_string(),
        _ => loc.t("smoothing.high").to_string(),
    }
}

fn format_auto_save(minutes: u32, loc: &Localization) -> String {
    if minutes == 0 {
        loc.t("common.disabled").to_string()
    } else {
        format!("{minutes} min")
    }
}

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsTab>()
            .init_resource::<SettingsDetailOpen>()
            .add_systems(OnEnter(MenuState::Settings), spawn_settings_menu)
            .add_systems(OnExit(MenuState::Settings), despawn_settings_menu)
            .add_systems(
                Update,
                (
                    handle_settings_buttons,
                    handle_language_changed,
                    update_settings_labels,
                    update_extended_settings_labels,
                    update_audio_settings_labels,
                    update_language_label,
                    update_settings_tab_view,
                    update_settings_card_scale,
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
    detail_open: Res<SettingsDetailOpen>,
    loc: Res<Localization>,
    env_state: Option<Res<EnvironmentState>>,
    app_font: Option<Res<AppFont>>,
    asset_server: Res<AssetServer>,
    menu_return_target: Res<MenuReturnTarget>,
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

    use crate::menu::{menu_bar::MENU_BAR_HEIGHT, theme::palette};

    let mut header_font = TextFont {
        font_size: FontSize::Px(13.0),
        ..default()
    };
    if let Some(ref font) = font_handle {
        header_font.font = font.clone();
    }

    let is_from_main_menu = menu_return_target.0 == MenuState::MainMenu;
    let top_offset = if is_from_main_menu {
        0.0
    } else {
        MENU_BAR_HEIGHT
    };
    let left_offset = if is_from_main_menu {
        0.0
    } else {
        palette::DRAWER_COL1_WIDTH
    };

    commands
        .spawn((
            SettingsMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(left_offset),
                top: px(top_offset),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                ..default()
            },
            BackgroundColor(Color::NONE),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            if is_from_main_menu {
                let bg_texture = asset_server.load("textures/interfaces/main_menu_bg.png");
                backdrop.spawn((
                    ImageNode {
                        image: bg_texture,
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    ZIndex(0),
                ));
                backdrop.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                    ZIndex(1),
                ));
            }

            backdrop
                .spawn((
                    SettingsMenuCard,
                    ZIndex(10),
                    Node {
                        height: percent(100.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                ))
                .with_children(|drawers| {
                    // Column 2: Settings Categories Drawer
                    drawers
                        .spawn((
                            Node {
                                width: px(palette::DRAWER_COL2_WIDTH),
                                min_width: px(palette::DRAWER_COL2_WIDTH),
                                flex_shrink: 0.0,
                                height: percent(100.0),
                                border: UiRect::right(px(1.5)),
                                padding: UiRect::vertical(px(28.0)),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Stretch,
                                row_gap: px(4.0),
                                ..default()
                            },
                            BackgroundColor(palette::DRAWER_COL2_BG),
                            BorderColor::all(palette::DRAWER_DIVIDER),
                        ))
                        .with_children(|cat_col| {
                            if is_from_main_menu {
                                let mut back_font = TextFont {
                                    font_size: FontSize::Px(15.0),
                                    ..default()
                                };
                                if let Some(ref font) = font_handle {
                                    back_font.font = font.clone();
                                }

                                let back_theme = crate::menu::ThemedButton {
                                    normal_bg: Color::NONE,
                                    normal_border: Color::NONE,
                                    hover_bg: palette::NAV_ITEM_HOVER_BG,
                                    hover_border: Color::NONE,
                                    pressed_bg: palette::NAV_ITEM_ACTIVE_BG,
                                };
                                cat_col
                                    .spawn((
                                        Button,
                                        Hovered::default(),
                                        SettingsAction::Back,
                                        back_theme,
                                        BackgroundColor(Color::NONE),
                                        BorderColor::all(Color::NONE),
                                        Node {
                                            width: percent(100.0),
                                            height: px(44.0),
                                            padding: UiRect::new(px(24.0), px(20.0), px(0.0), px(0.0)),
                                            margin: UiRect::bottom(px(8.0)),
                                            border: UiRect::bottom(px(1.0)),
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            column_gap: px(10.0),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new(format!("<  {}", loc.t("menu.settings.back"))),
                                            back_font,
                                            TextColor(palette::TEXT_MUTED),
                                            text_shadow_default(),
                                        ));
                                    });
                            }

                            spawn_tab_button(
                                cat_col,
                                loc.t("menu.settings.tab_world"),
                                SettingsTab::World,
                                false,
                                font_handle.as_ref(),
                            );
                            spawn_tab_button(
                                cat_col,
                                loc.t("menu.settings.tab_graphics"),
                                SettingsTab::Graphics,
                                false,
                                font_handle.as_ref(),
                            );
                            spawn_tab_button(
                                cat_col,
                                loc.t("menu.settings.tab_shaders"),
                                SettingsTab::Shaders,
                                false,
                                font_handle.as_ref(),
                            );
                            spawn_tab_button(
                                cat_col,
                                loc.t("menu.settings.tab_camera"),
                                SettingsTab::Camera,
                                false,
                                font_handle.as_ref(),
                            );
                            spawn_tab_button(
                                cat_col,
                                loc.t("menu.settings.tab_audio"),
                                SettingsTab::Audio,
                                false,
                                font_handle.as_ref(),
                            );
                            spawn_tab_button(
                                cat_col,
                                loc.t("menu.settings.tab_accessibility"),
                                SettingsTab::Accessibility,
                                false,
                                font_handle.as_ref(),
                            );
                        });

                    // Column 3: Settings Detail Panel (opens when a category in Column 2 is selected)
                    drawers
                        .spawn((
                            SettingsDetailPanel,
                            Node {
                                width: px(palette::DRAWER_COL3_WIDTH),
                                height: percent(100.0),
                                border: UiRect::right(px(1.5)),
                                padding: UiRect::all(px(24.0)),
                                display: if detail_open.0 {
                                    Display::Flex
                                } else {
                                    Display::None
                                },
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Stretch,
                                row_gap: px(12.0),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            BackgroundColor(palette::DRAWER_COL3_BG),
                            BorderColor::all(palette::DRAWER_DIVIDER),
                        ))
                        .with_children(|card| {
                            // Accessibility & UI Settings Container
                            card.spawn((
                                SettingsSection(SettingsTab::Accessibility),
                                Node {
                                    width: percent(100.0),
                                    display: if *settings_tab == SettingsTab::Accessibility {
                                        Display::Flex
                                    } else {
                                        Display::None
                                    },
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(10.0),
                                    align_items: AlignItems::Stretch,
                                    ..default()
                                },
                            ))
                            .with_children(|col_acc| {
                                spawn_stepper_row(
                                    col_acc,
                                    loc.t("menu.settings.language"),
                                    loc.active_locale.display_name().to_string(),
                                    SettingsAction::DecLanguage,
                                    SettingsAction::IncLanguage,
                                    LanguageLabel,
                                    font_handle.as_ref(),
                                );

                                spawn_stepper_row(
                                    col_acc,
                                    loc.t("menu.settings.gui_scale"),
                                    format!("{}x", game_settings.gui_scale),
                                    SettingsAction::DecGuiScale,
                                    SettingsAction::IncGuiScale,
                                    GuiScaleLabel,
                                    font_handle.as_ref(),
                                );

                                spawn_stepper_row(
                                    col_acc,
                                    loc.t("menu.settings.menu_blur"),
                                    format_blur(game_settings.menu_blur_intensity, &loc),
                                    SettingsAction::DecMenuBlur,
                                    SettingsAction::IncMenuBlur,
                                    MenuBlurLabel,
                                    font_handle.as_ref(),
                                );

                                spawn_stepper_row(
                                    col_acc,
                                    loc.t("menu.settings.background_dimming"),
                                    format_dimming(game_settings.background_dimming, &loc),
                                    SettingsAction::DecBackgroundDimming,
                                    SettingsAction::IncBackgroundDimming,
                                    BackgroundDimmingLabel,
                                    font_handle.as_ref(),
                                );

                                spawn_stepper_row(
                                    col_acc,
                                    loc.t("menu.settings.fps_hud"),
                                    format_fps_hud(local_config.fps_hud_mode, &loc),
                                    SettingsAction::DecFpsHud,
                                    SettingsAction::IncFpsHud,
                                    FpsHudLabel,
                                    font_handle.as_ref(),
                                );

                                spawn_toggle_button(
                                    col_acc,
                                    SettingsAction::ToggleHighContrastCrosshair,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.high_contrast_crosshair"),
                                        if game_settings.high_contrast_crosshair {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    HighContrastCrosshairLabel,
                                    font_handle.as_ref(),
                                );

                                spawn_toggle_button(
                                    col_acc,
                                    SettingsAction::ToggleDynamicCrosshair,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.dynamic_crosshair"),
                                        if game_settings.dynamic_crosshair {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    DynamicCrosshairLabel,
                                    font_handle.as_ref(),
                                );

                                spawn_toggle_button(
                                    col_acc,
                                    SettingsAction::ToggleMainHand,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.main_hand"),
                                        if game_settings.left_handed {
                                            loc.t("menu.settings.hand_left")
                                        } else {
                                            loc.t("menu.settings.hand_right")
                                        }
                                    ),
                                    MainHandLabel,
                                    font_handle.as_ref(),
                                );

                                spawn_toggle_button(
                                    col_acc,
                                    SettingsAction::ToggleSubtitles,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.subtitles"),
                                        if game_settings.subtitles_enabled {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    SubtitlesLabel,
                                    font_handle.as_ref(),
                                );
                            });

                            // Graphics & Display Settings Container (Display options at top)
                            card.spawn((
                                SettingsSection(SettingsTab::Graphics),
                                Node {
                                    width: percent(100.0),
                                    display: if *settings_tab == SettingsTab::Graphics {
                                        Display::Flex
                                    } else {
                                        Display::None
                                    },
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(16.0),
                                    align_items: AlignItems::Stretch,
                                    ..default()
                                },
                            ))
                            .with_children(|columns| {
                                // Top Block: Display Settings
                                columns
                                    .spawn(Node {
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(10.0),
                                        width: percent(100.0),
                                        ..default()
                                    })
                                    .with_children(|col_disp| {
                                        spawn_stepper_row(
                                            col_disp,
                                            loc.t("menu.settings.screen_mode"),
                                            format_screen_mode(game_settings.screen_mode, &loc),
                                            SettingsAction::DecScreenMode,
                                            SettingsAction::IncScreenMode,
                                            ScreenModeLabel,
                                            font_handle.as_ref(),
                                        );

                                        // Conditional Borderless Mouse Lock (shown only in Borderless mode)
                                        col_disp
                                            .spawn((
                                                BorderlessMouseLockRow,
                                                Node {
                                                    width: percent(100.0),
                                                    display: if game_settings.screen_mode
                                                        == ScreenMode::BorderlessFullscreen
                                                    {
                                                        Display::Flex
                                                    } else {
                                                        Display::None
                                                    },
                                                    ..default()
                                                },
                                            ))
                                            .with_children(|wrap| {
                                                spawn_toggle_button(
                                                    wrap,
                                                    SettingsAction::ToggleMouseLockBorderless,
                                                    format!(
                                                        "{}: {}",
                                                        loc.t(
                                                            "menu.settings.mouse_lock_borderless"
                                                        ),
                                                        if game_settings.mouse_lock_borderless {
                                                            loc.t("common.enabled")
                                                        } else {
                                                            loc.t("common.disabled")
                                                        }
                                                    ),
                                                    MouseLockBorderlessLabel,
                                                    font_handle.as_ref(),
                                                );
                                            });

                                        spawn_stepper_row(
                                            col_disp,
                                            loc.t("menu.settings.target_fps"),
                                            format_target_fps(local_config.target_fps, &loc),
                                            SettingsAction::DecTargetFps,
                                            SettingsAction::IncTargetFps,
                                            TargetFpsLabel,
                                            font_handle.as_ref(),
                                        );

                                        spawn_toggle_button(
                                            col_disp,
                                            SettingsAction::ToggleVsync,
                                            format!(
                                                "{}: {}",
                                                loc.t("menu.settings.vsync"),
                                                if game_settings.vsync_enabled {
                                                    loc.t("common.enabled")
                                                } else {
                                                    loc.t("common.disabled")
                                                }
                                            ),
                                            VsyncLabel,
                                            font_handle.as_ref(),
                                        );

                                        spawn_toggle_button(
                                            col_disp,
                                            SettingsAction::ToggleDynamicFps,
                                            format!(
                                                "{}: {}",
                                                loc.t("menu.settings.dynamic_fps"),
                                                if game_settings.dynamic_fps_enabled {
                                                    loc.t("common.enabled")
                                                } else {
                                                    loc.t("common.disabled")
                                                }
                                            ),
                                            DynamicFpsLabel,
                                            font_handle.as_ref(),
                                        );
                                    });

                                // Graphics Quality & Fidelity
                                columns
                                    .spawn(Node {
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(10.0),
                                        width: percent(100.0),
                                        ..default()
                                    })
                                    .with_children(|col_left| {
                                // Render Scale Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.render_scale"),
                                    crate::core::config::format_render_scale(
                                        local_config.render_scale,
                                    ),
                                    SettingsAction::DecRenderScale,
                                    SettingsAction::IncRenderScale,
                                    RenderScaleLabel,
                                    font_handle.as_ref(),
                                );

                                // Upscaling Filter Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.upscaling_filter"),
                                    local_config.upscaling_filter.label().to_string(),
                                    SettingsAction::DecUpscalingFilter,
                                    SettingsAction::IncUpscalingFilter,
                                    UpscalingFilterLabel,
                                    font_handle.as_ref(),
                                );

                                // Anti-Aliasing Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.anti_aliasing"),
                                    format_anti_aliasing(local_config.anti_aliasing, &loc),
                                    SettingsAction::DecAntiAliasing,
                                    SettingsAction::IncAntiAliasing,
                                    AntiAliasingLabel,
                                    font_handle.as_ref(),
                                );

                                // Shadow Quality Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.shadows"),
                                    format_shadows(local_config.shadow_quality, &loc),
                                    SettingsAction::DecShadowQuality,
                                    SettingsAction::IncShadowQuality,
                                    ShadowQualityLabel,
                                    font_handle.as_ref(),
                                );

                                // Cloud Quality Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.clouds"),
                                    format_clouds(local_config.cloud_quality, &loc),
                                    SettingsAction::DecCloudQuality,
                                    SettingsAction::IncCloudQuality,
                                    CloudQualityLabel,
                                    font_handle.as_ref(),
                                );

                                // Cloud Distance Stepper (16..=128 chunks)
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.cloud_distance"),
                                    loc.t_args(
                                        "common.chunks",
                                        &[&local_config.cloud_render_distance.to_string()],
                                    ),
                                    SettingsAction::DecCloudDistance,
                                    SettingsAction::IncCloudDistance,
                                    CloudDistanceLabel,
                                    font_handle.as_ref(),
                                );

                                // Full Surface Sides Toggle Button
                                spawn_toggle_button(
                                    col_left,
                                    SettingsAction::ToggleFullGrass,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.full_grass"),
                                        if game_settings.full_grass {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    FullGrassLabel,
                                    font_handle.as_ref(),
                                );
                            });

                        // Right Column: Atmosphere & Foliage Shaders
                        columns
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(10.0),
                                width: percent(100.0),
                                ..default()
                            })
                            .with_children(|col_right| {
                                // God Rays Toggle Button
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleGodRays,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.god_rays"),
                                        if local_config.god_rays_enabled {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    GodRaysLabel,
                                    font_handle.as_ref(),
                                );

                                // Fog Toggle Button
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleFog,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.fog"),
                                        if fog_enabled {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    FogLabel,
                                    font_handle.as_ref(),
                                );

                                // Transmission Toggle Button
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleTransmission,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.transmission"),
                                        if local_config.transmission_enabled {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    TransmissionLabel,
                                    font_handle.as_ref(),
                                );

                                // Connected Textures Toggle
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleConnectedTextures,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.connected_textures"),
                                        if local_config.connected_textures {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    ConnectedTexturesLabel,
                                    font_handle.as_ref(),
                                );
                            });
                    });

                    // World & Simulation Settings Container
                    card.spawn((
                        SettingsSection(SettingsTab::World),
                        Node {
                            width: percent(100.0),
                            display: if *settings_tab == SettingsTab::World {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            flex_direction: FlexDirection::Column,
                            row_gap: px(16.0),
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                    ))
                    .with_children(|columns| {
                        // Left Column: Streaming & Chunks
                        columns
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(10.0),
                                width: percent(100.0),
                                ..default()
                            })
                            .with_children(|col_left| {
                                // Render Distance Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.render_distance"),
                                    loc.t_args("common.chunks", &[&render_dist.to_string()]),
                                    SettingsAction::DecRenderDistance,
                                    SettingsAction::IncRenderDistance,
                                    RenderDistanceLabel,
                                    font_handle.as_ref(),
                                );

                                // LOD Distance Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.lod_distance"),
                                    if lod_dist == 0 {
                                        loc.t("common.disabled").to_string()
                                    } else {
                                        loc.t_args("common.chunks", &[&lod_dist.to_string()])
                                    },
                                    SettingsAction::DecLodDistance,
                                    SettingsAction::IncLodDistance,
                                    LodDistanceLabel,
                                    font_handle.as_ref(),
                                );

                                // Simulation Distance Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.sim_distance"),
                                    loc.t_args("common.chunks", &[&sim_dist.to_string()]),
                                    SettingsAction::DecSimulationDistance,
                                    SettingsAction::IncSimulationDistance,
                                    SimulationDistanceLabel,
                                    font_handle.as_ref(),
                                );

                                // Mesh Upload Budget Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.mesh_uploads"),
                                    loc.t_args("common.chunks", &[&local_config.mesh_upload_budget.to_string()]),
                                    SettingsAction::DecMeshUploadBudget,
                                    SettingsAction::IncMeshUploadBudget,
                                    MeshUploadBudgetLabel,
                                    font_handle.as_ref(),
                                );
                            });

                        // Right Column: Camera & Environment Flow
                        // World Time & Pause Behavior
                        columns
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(10.0),
                                width: percent(100.0),
                                ..default()
                            })
                            .with_children(|col_world_extra| {
                                // Time Pause Toggle Button
                                spawn_toggle_button(
                                    col_world_extra,
                                    SettingsAction::ToggleTimePause,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.time_flow"),
                                        if time_paused {
                                            loc.t("common.paused")
                                        } else {
                                            loc.t("common.running")
                                        }
                                    ),
                                    TimePauseLabel,
                                    font_handle.as_ref(),
                                );

                                // Menu Pause Behavior Stepper
                                spawn_stepper_row(
                                    col_world_extra,
                                    loc.t("menu.settings.menu_pause"),
                                    format_menu_pause(game_settings.menu_pause_mode, &loc),
                                    SettingsAction::DecMenuPause,
                                    SettingsAction::IncMenuPause,
                                    MenuPauseLabel,
                                    font_handle.as_ref(),
                                );

                                // Auto-Save Interval Stepper (placeholder)
                                spawn_stepper_row(
                                    col_world_extra,
                                    loc.t("menu.settings.auto_save"),
                                    format_auto_save(game_settings.auto_save_minutes, &loc),
                                    SettingsAction::DecAutoSave,
                                    SettingsAction::IncAutoSave,
                                    AutoSaveLabel,
                                    font_handle.as_ref(),
                                );
                            });
                    });

                    // Camera Settings Container
                    card.spawn((
                        SettingsSection(SettingsTab::Camera),
                        Node {
                            width: percent(100.0),
                            display: if *settings_tab == SettingsTab::Camera {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            flex_direction: FlexDirection::Column,
                            row_gap: px(10.0),
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                    ))
                    .with_children(|col_cam| {
                        // Field of View Stepper
                        spawn_stepper_row(
                            col_cam,
                            loc.t("menu.settings.fov"),
                            format!("{fov}°"),
                            SettingsAction::DecFov,
                            SettingsAction::IncFov,
                            FovLabel,
                            font_handle.as_ref(),
                        );

                        // Dynamic FOV / Reduced Motion Toggle Button
                        spawn_toggle_button(
                            col_cam,
                            SettingsAction::ToggleDynamicFov,
                            format!(
                                "{}: {}",
                                loc.t("menu.settings.fov_effects"),
                                if game_settings.dynamic_fov_enabled {
                                    loc.t("common.normal")
                                } else {
                                    loc.t("common.reduced")
                                }
                            ),
                            DynamicFovLabel,
                            font_handle.as_ref(),
                        );

                        // View Bobbing Toggle Button
                        spawn_toggle_button(
                            col_cam,
                            SettingsAction::ToggleViewBobbing,
                            format!(
                                "{}: {}",
                                loc.t("menu.settings.view_bobbing"),
                                if game_settings.view_bobbing {
                                    loc.t("common.enabled")
                                } else {
                                    loc.t("common.disabled")
                                }
                            ),
                            ViewBobbingLabel,
                            font_handle.as_ref(),
                        );

                        // Camera Smoothing Stepper
                        spawn_stepper_row(
                            col_cam,
                            loc.t("menu.settings.camera_smoothing"),
                            format_camera_smoothing(game_settings.camera_smoothing, &loc),
                            SettingsAction::DecCameraSmoothing,
                            SettingsAction::IncCameraSmoothing,
                            CameraSmoothingLabel,
                            font_handle.as_ref(),
                        );

                        // Third-Person Distance Stepper
                        spawn_stepper_row(
                            col_cam,
                            loc.t("menu.settings.third_person_dist"),
                            format!("{:.1}m", game_settings.third_person_distance),
                            SettingsAction::DecThirdPersonDist,
                            SettingsAction::IncThirdPersonDist,
                            ThirdPersonDistLabel,
                            font_handle.as_ref(),
                        );

                        // Third-Person Style Toggle (Centered vs Over-the-Shoulder)
                        spawn_toggle_button(
                            col_cam,
                            SettingsAction::ToggleThirdPersonShoulder,
                            format!(
                                "{}: {}",
                                loc.t("menu.settings.third_person_style"),
                                if game_settings.third_person_over_shoulder {
                                    loc.t("menu.settings.third_person_shoulder")
                                } else {
                                    loc.t("menu.settings.third_person_centered")
                                }
                            ),
                            ThirdPersonShoulderLabel,
                            font_handle.as_ref(),
                        );

                        // Third-Person Free 360° Orbit Toggle
                        spawn_toggle_button(
                            col_cam,
                            SettingsAction::ToggleThirdPersonFreeOrbit,
                            format!(
                                "{}: {}",
                                loc.t("menu.settings.third_person_free_orbit"),
                                if game_settings.third_person_free_orbit {
                                    loc.t("common.enabled")
                                } else {
                                    loc.t("common.disabled")
                                }
                            ),
                            ThirdPersonFreeOrbitLabel,
                            font_handle.as_ref(),
                        );
                    });

                    // Shaders & Visual FX Settings Container
                    card.spawn((
                        SettingsSection(SettingsTab::Shaders),
                        Node {
                            width: percent(100.0),
                            display: if *settings_tab == SettingsTab::Shaders {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            flex_direction: FlexDirection::Column,
                            row_gap: px(16.0),
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                    ))
                    .with_children(|columns| {
                        // Left Column: Materials & Lighting
                        columns
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(10.0),
                                width: percent(100.0),
                                ..default()
                            })
                            .with_children(|col_left| {
                                // Coated Textures Stepper
                                spawn_stepper_row(
                                    col_left,
                                    loc.t("menu.settings.coated_textures"),
                                    format_coated_textures(local_config.coated_textures, &loc),
                                    SettingsAction::DecCoatedTextures,
                                    SettingsAction::IncCoatedTextures,
                                    CoatedTexturesLabel,
                                    font_handle.as_ref(),
                                );

                                // Pixelated Shadows Toggle
                                spawn_toggle_button(
                                    col_left,
                                    SettingsAction::TogglePixelatedShadows,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.pixelated_shadows"),
                                        if local_config.pixelated_shadows {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    PixelatedShadowsLabel,
                                    font_handle.as_ref(),
                                );

                                // HDR Bloom & Glow Toggle
                                spawn_toggle_button(
                                    col_left,
                                    SettingsAction::ToggleHdrBloom,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.hdr_bloom"),
                                        if local_config.hdr_bloom {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    HdrBloomLabel,
                                    font_handle.as_ref(),
                                );
                            });

                        // Right Column: Dynamic Motion & Foliage
                        columns
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(10.0),
                                width: percent(100.0),
                                ..default()
                            })
                            .with_children(|col_right| {
                                // Waving Foliage & Leaves Toggle
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleWavingFoliage,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.waving_foliage"),
                                        if local_config.waving_foliage {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    WavingFoliageLabel,
                                    font_handle.as_ref(),
                                );

                                // Internal Leaf Culling Toggle (Optional Optimization)
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleLeafCulling,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.leaf_culling"),
                                        if local_config.leaf_culling {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    LeafCullingLabel,
                                    font_handle.as_ref(),
                                );

                                // Dynamic Handheld Lighting Toggle
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleDynamicLighting,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.dynamic_lighting"),
                                        if local_config.dynamic_lighting {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    DynamicLightingLabel,
                                    font_handle.as_ref(),
                                );

                                // Procedural Grass Toggle Button (moved from Graphics to Shaders & FX)
                                spawn_toggle_button(
                                    col_right,
                                    SettingsAction::ToggleProceduralGrass,
                                    format!(
                                        "{}: {}",
                                        loc.t("menu.settings.procedural_grass"),
                                        if local_config.grass_enabled {
                                            loc.t("common.enabled")
                                        } else {
                                            loc.t("common.disabled")
                                        }
                                    ),
                                    ProceduralGrassLabel,
                                    font_handle.as_ref(),
                                );

                                // Grass Density Stepper (moved from Graphics to Shaders & FX)
                                spawn_stepper_row(
                                    col_right,
                                    loc.t("menu.settings.grass_density"),
                                    format_grass_density(local_config.grass_density, &loc),
                                    SettingsAction::DecGrassDensity,
                                    SettingsAction::IncGrassDensity,
                                    GrassDensityLabel,
                                    font_handle.as_ref(),
                                );
                            });
                    });

                    // Audio Settings Container (full width so steppers align cleanly to right edge)
                    card.spawn((
                        SettingsSection(SettingsTab::Audio),
                        Node {
                            width: percent(100.0),
                            display: if *settings_tab == SettingsTab::Audio {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            row_gap: px(10.0),
                            ..default()
                        },
                    ))
                    .with_children(|col_audio| {
                        spawn_stepper_row(
                            col_audio,
                            loc.t("menu.settings.master_vol"),
                            crate::core::config::format_volume(audio_settings.master_volume),
                            SettingsAction::DecMasterVol,
                            SettingsAction::IncMasterVol,
                            MasterVolLabel,
                            font_handle.as_ref(),
                        );

                        spawn_stepper_row(
                            col_audio,
                            loc.t("menu.settings.music_vol"),
                            crate::core::config::format_volume(audio_settings.music_volume),
                            SettingsAction::DecMusicVol,
                            SettingsAction::IncMusicVol,
                            MusicVolLabel,
                            font_handle.as_ref(),
                        );

                        spawn_stepper_row(
                            col_audio,
                            loc.t("menu.settings.weather_vol"),
                            crate::core::config::format_volume(audio_settings.weather_volume),
                            SettingsAction::DecWeatherVol,
                            SettingsAction::IncWeatherVol,
                            WeatherVolLabel,
                            font_handle.as_ref(),
                        );

                        spawn_stepper_row(
                            col_audio,
                            loc.t("menu.settings.sfx_vol"),
                            crate::core::config::format_volume(audio_settings.sfx_volume),
                            SettingsAction::DecSfxVol,
                            SettingsAction::IncSfxVol,
                            SfxVolLabel,
                            font_handle.as_ref(),
                        );

                            spawn_stepper_row(
                                col_audio,
                                loc.t("menu.settings.ambient_vol"),
                                crate::core::config::format_volume(audio_settings.ambient_volume),
                                SettingsAction::DecAmbientVol,
                                SettingsAction::IncAmbientVol,
                                AmbientVolLabel,
                                font_handle.as_ref(),
                            );

                            spawn_stepper_row(
                                col_audio,
                                loc.t("menu.settings.ui_vol"),
                                crate::core::config::format_volume(game_settings.ui_volume),
                                SettingsAction::DecUiVol,
                                SettingsAction::IncUiVol,
                                UiVolLabel,
                                font_handle.as_ref(),
                            );

                            spawn_toggle_button(
                                col_audio,
                                SettingsAction::ToggleMuteUnfocus,
                                format!(
                                    "{}: {}",
                                    loc.t("menu.settings.mute_on_unfocus"),
                                    if game_settings.mute_on_unfocus {
                                        loc.t("common.enabled")
                                    } else {
                                        loc.t("common.disabled")
                                    }
                                ),
                                MuteUnfocusLabel,
                                font_handle.as_ref(),
                            );
                        });
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

    let step_theme = crate::menu::ThemedButton::subtle();

    parent
        .spawn(Node {
            width: percent(100.0),
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
                        step_theme,
                        BackgroundColor(step_theme.normal_bg),
                        BorderColor::all(step_theme.normal_border),
                        Node {
                            width: px(28.0),
                            height: px(28.0),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(px(1.5)),
                            border_radius: BorderRadius::all(px(4.0)),
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
                        step_theme,
                        BackgroundColor(step_theme.normal_bg),
                        BorderColor::all(step_theme.normal_border),
                        Node {
                            width: px(28.0),
                            height: px(28.0),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(px(1.5)),
                            border_radius: BorderRadius::all(px(4.0)),
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
) {
    let mut btn_font = TextFont {
        font_size: FontSize::Px(14.5),
        ..default()
    };
    if let Some(font) = font_handle {
        btn_font.font = font.clone();
    }

    let toggle_theme = crate::menu::ThemedButton::subtle();

    parent
        .spawn((
            Button,
            Hovered::default(),
            action,
            toggle_theme,
            BackgroundColor(toggle_theme.normal_bg),
            BorderColor::all(toggle_theme.normal_border),
            Node {
                width: percent(100.0),
                height: px(36.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1.5)),
                border_radius: BorderRadius::all(px(4.0)),
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
) {
    use crate::menu::theme::palette;

    let mut btn_font = TextFont {
        font_size: FontSize::Px(15.0),
        ..default()
    };
    if let Some(font) = font_handle {
        btn_font.font = font.clone();
    }

    let text_color = if is_active {
        palette::TEXT_ACCENT
    } else {
        palette::TEXT_PRIMARY
    };

    let tab_theme = crate::menu::ThemedButton {
        normal_bg: Color::NONE,
        normal_border: Color::NONE,
        hover_bg: palette::NAV_ITEM_HOVER_BG,
        hover_border: Color::NONE,
        pressed_bg: palette::NAV_ITEM_ACTIVE_BG,
    };

    parent
        .spawn((
            Button,
            Hovered::default(),
            SettingsAction::SwitchTab(tab),
            TabButton(tab),
            tab_theme,
            BackgroundColor(if is_active {
                palette::NAV_ITEM_ACTIVE_BG
            } else {
                Color::NONE
            }),
            BorderColor::all(Color::NONE),
            Node {
                width: percent(100.0),
                height: px(46.0),
                padding: UiRect::new(px(24.0), px(20.0), px(0.0), px(0.0)),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|btn| {
            btn.spawn((
                TabButtonAccent,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(8.0),
                    bottom: px(8.0),
                    width: px(3.0),
                    border_radius: BorderRadius::right(px(2.0)),
                    ..default()
                },
                BackgroundColor(palette::ACCENT_LINE),
                if is_active {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ));

            btn.spawn((
                TabButtonLabel,
                Text::new(title),
                btn_font.clone(),
                TextColor(text_color),
                text_shadow_default(),
            ));

            btn.spawn((
                Text::new(">"),
                btn_font,
                TextColor(palette::TEXT_MUTED),
            ));
        });
}

fn despawn_settings_menu(
    mut commands: Commands,
    mut detail_open: ResMut<SettingsDetailOpen>,
    query: Query<Entity, With<SettingsMenuRoot>>,
) {
    detail_open.0 = false;
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
            &crate::menu::ThemedButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mouse: Res<ButtonInput<MouseButton>>,
    mut chunk_settings: ResMut<ChunkStreamingSettings>,
    mut game_settings: ResMut<GameSettings>,
    mut local_config: ResMut<LocalConfig>,
    mut audio_settings: ResMut<AudioSettings>,
    mut settings_tab: ResMut<SettingsTab>,
    mut detail_open: ResMut<SettingsDetailOpen>,
    mut loc: ResMut<Localization>,
    mut env_state: Option<ResMut<EnvironmentState>>,
    mut dynamic_fps_settings: Option<ResMut<DynamicFpsSettings>>,
    mut debug_hud_settings: Option<ResMut<crate::core::DebugHudSettings>>,
    menu_return_target: Res<MenuReturnTarget>,
    mut next_state: ResMut<NextState<MenuState>>,
    (mut chunk_material, mut texture_registry, mut streaming_queues, world): (
        Option<ResMut<ChunkMaterial>>,
        Option<ResMut<VoxelTextureRegistry>>,
        Option<ResMut<ChunkStreamingQueues>>,
        Option<Res<VoxelWorld>>,
    ),
    mut window_query: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    let left_clicked = mouse.just_pressed(MouseButton::Left);

    for (hovered, is_pressed, action, theme, mut bg, mut border) in &mut interaction_query {
        let is_hovered = hovered.get();
        let (next_bg, next_border) = if let SettingsAction::SwitchTab(tab) = action {
            if detail_open.0 && *tab == *settings_tab {
                (crate::menu::theme::palette::NAV_ITEM_ACTIVE_BG, Color::NONE)
            } else {
                theme.colors(is_hovered, is_pressed)
            }
        } else {
            theme.colors(is_hovered, is_pressed)
        };
        bg.0 = next_bg;
        *border = BorderColor::all(next_border);

        if is_hovered && left_clicked {
            let mut sync_config = true;
            match action {
                SettingsAction::SwitchTab(tab) => {
                    if detail_open.0 && *settings_tab == *tab {
                        detail_open.0 = false;
                    } else {
                        *settings_tab = *tab;
                        detail_open.0 = true;
                    }
                    sync_config = false;
                }
                SettingsAction::DecLanguage => {
                    let next = loc.active_locale.prev();
                    loc.set_locale(next);
                    local_config.language = next.code().to_string();
                }
                SettingsAction::IncLanguage => {
                    let next = loc.active_locale.next();
                    loc.set_locale(next);
                    local_config.language = next.code().to_string();
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
                    game_settings.leaf_culling = local_config.leaf_culling;
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
                        reg.leaf_culling = game_settings.leaf_culling;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.full_grass = game_settings.full_grass;
                        mat.texture_registry.leaf_culling = game_settings.leaf_culling;
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
                    game_settings.leaf_culling = local_config.leaf_culling;
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
                        reg.leaf_culling = game_settings.leaf_culling;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.full_grass = game_settings.full_grass;
                        mat.texture_registry.leaf_culling = game_settings.leaf_culling;
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
                SettingsAction::DecCloudDistance => {
                    local_config.cloud_render_distance =
                        (local_config.cloud_render_distance - 8).max(16);
                }
                SettingsAction::IncCloudDistance => {
                    local_config.cloud_render_distance =
                        (local_config.cloud_render_distance + 8).min(128);
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
                SettingsAction::DecGuiScale => {
                    game_settings.gui_scale = (game_settings.gui_scale.saturating_sub(1)).max(2);
                    local_config.gui_scale = game_settings.gui_scale;
                }
                SettingsAction::IncGuiScale => {
                    game_settings.gui_scale = (game_settings.gui_scale + 1).min(4);
                    local_config.gui_scale = game_settings.gui_scale;
                }
                SettingsAction::DecMenuBlur => {
                    game_settings.menu_blur_intensity = game_settings.menu_blur_intensity.prev();
                    local_config.menu_blur_intensity = game_settings.menu_blur_intensity;
                }
                SettingsAction::IncMenuBlur => {
                    game_settings.menu_blur_intensity = game_settings.menu_blur_intensity.next();
                    local_config.menu_blur_intensity = game_settings.menu_blur_intensity;
                }
                SettingsAction::DecBackgroundDimming => {
                    game_settings.background_dimming = game_settings.background_dimming.prev();
                    local_config.background_dimming = game_settings.background_dimming;
                }
                SettingsAction::IncBackgroundDimming => {
                    game_settings.background_dimming = game_settings.background_dimming.next();
                    local_config.background_dimming = game_settings.background_dimming;
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
                        (chunk_settings.lod_render_distance + 2).min(64);
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
                SettingsAction::ToggleProceduralGrass => {
                    local_config.grass_enabled = !local_config.grass_enabled;
                    if let Some(ref mut reg) = texture_registry {
                        reg.grass_enabled = local_config.grass_enabled;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.grass_enabled = local_config.grass_enabled;
                    }
                    if let Some(ref world) = world
                        && let Some(ref mut queues) = streaming_queues
                    {
                        for (&coord, _) in world.iter_chunks() {
                            queues.enqueue_remesh(coord);
                        }
                    }
                }
                SettingsAction::CycleGrassStyle => {
                    local_config.grass_style =
                        crate::core::config::cycle_grass_style(&local_config.grass_style);
                }
                SettingsAction::DecGrassDensity => {
                    local_config.grass_density =
                        crate::core::config::prev_grass_density(local_config.grass_density);
                    if let Some(ref mut reg) = texture_registry {
                        reg.grass_density = local_config.grass_density;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.grass_density = local_config.grass_density;
                    }
                    if let Some(ref world) = world
                        && let Some(ref mut queues) = streaming_queues
                    {
                        for (&coord, _) in world.iter_chunks() {
                            queues.enqueue_remesh(coord);
                        }
                    }
                }
                SettingsAction::IncGrassDensity => {
                    local_config.grass_density =
                        crate::core::config::next_grass_density(local_config.grass_density);
                    if let Some(ref mut reg) = texture_registry {
                        reg.grass_density = local_config.grass_density;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.grass_density = local_config.grass_density;
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
                SettingsAction::DecCoatedTextures => {
                    local_config.coated_textures = local_config.coated_textures.prev();
                }
                SettingsAction::IncCoatedTextures => {
                    local_config.coated_textures = local_config.coated_textures.next();
                }
                SettingsAction::ToggleWavingFoliage => {
                    local_config.waving_foliage = !local_config.waving_foliage;
                }
                SettingsAction::ToggleLeafCulling => {
                    game_settings.leaf_culling = !game_settings.leaf_culling;
                    local_config.leaf_culling = game_settings.leaf_culling;
                    if let Some(ref mut reg) = texture_registry {
                        reg.leaf_culling = game_settings.leaf_culling;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.leaf_culling = game_settings.leaf_culling;
                    }
                    if let Some(ref world) = world
                        && let Some(ref mut queues) = streaming_queues
                    {
                        for (&coord, _) in world.iter_chunks() {
                            queues.enqueue_remesh(coord);
                        }
                    }
                }
                SettingsAction::ToggleConnectedTextures => {
                    game_settings.connected_textures = !game_settings.connected_textures;
                    local_config.connected_textures = game_settings.connected_textures;
                    if let Some(ref mut reg) = texture_registry {
                        reg.connected_textures = game_settings.connected_textures;
                    }
                    if let Some(ref mut mat) = chunk_material {
                        mat.texture_registry.connected_textures = game_settings.connected_textures;
                    }
                    if let Some(ref world) = world
                        && let Some(ref mut queues) = streaming_queues
                    {
                        for (&coord, _) in world.iter_chunks() {
                            queues.enqueue_remesh(coord);
                        }
                    }
                }
                SettingsAction::DecMenuPause => {
                    game_settings.menu_pause_mode = game_settings.menu_pause_mode.prev();
                    local_config.menu_pause_mode = game_settings.menu_pause_mode;
                }
                SettingsAction::IncMenuPause => {
                    game_settings.menu_pause_mode = game_settings.menu_pause_mode.next();
                    local_config.menu_pause_mode = game_settings.menu_pause_mode;
                }
                SettingsAction::ToggleMouseLockBorderless => {
                    game_settings.mouse_lock_borderless = !game_settings.mouse_lock_borderless;
                    local_config.mouse_lock_borderless = game_settings.mouse_lock_borderless;
                }
                SettingsAction::DecAutoSave => {
                    let steps = [0u32, 5, 10, 15];
                    let idx = steps
                        .iter()
                        .position(|&m| m == game_settings.auto_save_minutes)
                        .unwrap_or(2);
                    game_settings.auto_save_minutes = steps[(idx + steps.len() - 1) % steps.len()];
                    local_config.auto_save_minutes = game_settings.auto_save_minutes;
                }
                SettingsAction::IncAutoSave => {
                    let steps = [0u32, 5, 10, 15];
                    let idx = steps
                        .iter()
                        .position(|&m| m == game_settings.auto_save_minutes)
                        .unwrap_or(2);
                    game_settings.auto_save_minutes = steps[(idx + 1) % steps.len()];
                    local_config.auto_save_minutes = game_settings.auto_save_minutes;
                }
                SettingsAction::DecCameraSmoothing => {
                    game_settings.camera_smoothing = game_settings.camera_smoothing.saturating_sub(1);
                    local_config.camera_smoothing = game_settings.camera_smoothing;
                }
                SettingsAction::IncCameraSmoothing => {
                    game_settings.camera_smoothing = (game_settings.camera_smoothing + 1).min(3);
                    local_config.camera_smoothing = game_settings.camera_smoothing;
                }
                SettingsAction::DecThirdPersonDist => {
                    game_settings.third_person_distance =
                        (game_settings.third_person_distance - 0.5).clamp(2.5, 8.0);
                    local_config.third_person_distance = game_settings.third_person_distance;
                }
                SettingsAction::IncThirdPersonDist => {
                    game_settings.third_person_distance =
                        (game_settings.third_person_distance + 0.5).clamp(2.5, 8.0);
                    local_config.third_person_distance = game_settings.third_person_distance;
                }
                SettingsAction::ToggleThirdPersonShoulder => {
                    game_settings.third_person_over_shoulder =
                        !game_settings.third_person_over_shoulder;
                    local_config.third_person_over_shoulder =
                        game_settings.third_person_over_shoulder;
                }
                SettingsAction::ToggleThirdPersonFreeOrbit => {
                    game_settings.third_person_free_orbit = !game_settings.third_person_free_orbit;
                    local_config.third_person_free_orbit = game_settings.third_person_free_orbit;
                }
                SettingsAction::DecUiVol => {
                    game_settings.ui_volume = (game_settings.ui_volume - 0.05).clamp(0.0, 1.0);
                    local_config.ui_volume = game_settings.ui_volume;
                }
                SettingsAction::IncUiVol => {
                    game_settings.ui_volume = (game_settings.ui_volume + 0.05).clamp(0.0, 1.0);
                    local_config.ui_volume = game_settings.ui_volume;
                }
                SettingsAction::ToggleMuteUnfocus => {
                    game_settings.mute_on_unfocus = !game_settings.mute_on_unfocus;
                    local_config.mute_on_unfocus = game_settings.mute_on_unfocus;
                }
                SettingsAction::DecFpsHud => {
                    local_config.fps_hud_mode = local_config.fps_hud_mode.prev();
                    if let Some(ref mut hud) = debug_hud_settings {
                        hud.mode = local_config.fps_hud_mode;
                    }
                }
                SettingsAction::IncFpsHud => {
                    local_config.fps_hud_mode = local_config.fps_hud_mode.next();
                    if let Some(ref mut hud) = debug_hud_settings {
                        hud.mode = local_config.fps_hud_mode;
                    }
                }
                SettingsAction::ToggleHighContrastCrosshair => {
                    game_settings.high_contrast_crosshair = !game_settings.high_contrast_crosshair;
                    local_config.high_contrast_crosshair = game_settings.high_contrast_crosshair;
                }
                SettingsAction::ToggleDynamicCrosshair => {
                    game_settings.dynamic_crosshair = !game_settings.dynamic_crosshair;
                    local_config.dynamic_crosshair = game_settings.dynamic_crosshair;
                }
                SettingsAction::ToggleMainHand => {
                    game_settings.left_handed = !game_settings.left_handed;
                    local_config.left_handed = game_settings.left_handed;
                }
                SettingsAction::ToggleSubtitles => {
                    game_settings.subtitles_enabled = !game_settings.subtitles_enabled;
                    local_config.subtitles_enabled = game_settings.subtitles_enabled;
                }
                SettingsAction::TogglePixelatedShadows => {
                    local_config.pixelated_shadows = !local_config.pixelated_shadows;
                }
                SettingsAction::ToggleHdrBloom => {
                    local_config.hdr_bloom = !local_config.hdr_bloom;
                }
                SettingsAction::ToggleDynamicLighting => {
                    local_config.dynamic_lighting = !local_config.dynamic_lighting;
                }
                SettingsAction::Back => {
                    next_state.set(menu_return_target.0);
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
                local_config.leaf_culling = game_settings.leaf_culling;
                local_config.vsync_enabled = game_settings.vsync_enabled;
                local_config.dynamic_fps_enabled = game_settings.dynamic_fps_enabled;
                local_config.dynamic_fov_enabled = game_settings.dynamic_fov_enabled;
                local_config.gui_scale = game_settings.gui_scale;
                local_config.menu_blur_intensity = game_settings.menu_blur_intensity;
                local_config.background_dimming = game_settings.background_dimming;
                local_config.screen_mode = game_settings.screen_mode.label().to_lowercase();
                let _ = local_config.save();
            }
        }
    }
}

fn handle_language_changed(
    loc: Res<Localization>,
    mut commands: Commands,
    query: Query<Entity, With<SettingsMenuRoot>>,
    chunk_settings: Res<ChunkStreamingSettings>,
    game_settings: Res<GameSettings>,
    local_config: Res<LocalConfig>,
    audio_settings: Res<AudioSettings>,
    settings_tab: Res<SettingsTab>,
    detail_open: Res<SettingsDetailOpen>,
    env_state: Option<Res<EnvironmentState>>,
    app_font: Option<Res<AppFont>>,
    asset_server: Res<AssetServer>,
    menu_return_target: Res<MenuReturnTarget>,
) {
    if !loc.is_changed() || loc.is_added() {
        return;
    }
    for entity in &query {
        commands.entity(entity).despawn();
    }
    spawn_settings_menu(
        commands,
        chunk_settings,
        game_settings,
        local_config,
        audio_settings,
        settings_tab,
        detail_open,
        loc,
        env_state,
        app_font,
        asset_server,
        menu_return_target,
    );
}

fn update_settings_labels(
    chunk_settings: Res<ChunkStreamingSettings>,
    game_settings: Res<GameSettings>,
    local_config: Res<LocalConfig>,
    loc: Res<Localization>,
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
            Option<&CloudDistanceLabel>,
            Option<&GodRaysLabel>,
            Option<&FogLabel>,
            Option<&TransmissionLabel>,
            Option<&FullGrassLabel>,
            Option<&TimePauseLabel>,
        ),
        (
            Option<&GuiScaleLabel>,
            Option<&MenuBlurLabel>,
            Option<&BackgroundDimmingLabel>,
            Option<&ProceduralGrassLabel>,
            Option<&GrassStyleLabel>,
            Option<&GrassDensityLabel>,
        ),
        (
            Option<&CoatedTexturesLabel>,
            Option<&WavingFoliageLabel>,
            Option<&LeafCullingLabel>,
            Option<&ConnectedTexturesLabel>,
            Option<&MenuPauseLabel>,
            Option<&PixelatedShadowsLabel>,
            Option<&HdrBloomLabel>,
            Option<&DynamicLightingLabel>,
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
            cloud_distance,
            god_rays,
            fog,
            transmission,
            grass,
            time_pause,
        ),
        (
            gui_scale,
            menu_blur,
            background_dimming,
            proc_grass,
            grass_style,
            grass_density,
        ),
        (
            coated_textures,
            waving_foliage,
            leaf_culling,
            connected_textures,
            menu_pause,
            pixelated_shadows,
            hdr_bloom,
            dynamic_lighting,
        ),
    ) in &mut labels_query
    {
        if config_changed {
            if preset.is_some() {
                text.0 = format_preset(local_config.preset, &loc);
            } else if render_scale.is_some() {
                text.0 = crate::core::config::format_render_scale(local_config.render_scale);
            } else if upscaling_filter.is_some() {
                text.0 = local_config.upscaling_filter.label().to_string();
            } else if target_fps.is_some() {
                text.0 = format_target_fps(local_config.target_fps, &loc);
            } else if anti_aliasing.is_some() {
                text.0 = format_anti_aliasing(local_config.anti_aliasing, &loc);
            } else if shadow_quality.is_some() {
                text.0 = format_shadows(local_config.shadow_quality, &loc);
            } else if cloud_quality.is_some() {
                text.0 = format_clouds(local_config.cloud_quality, &loc);
            } else if cloud_distance.is_some() {
                text.0 = loc.t_args(
                    "common.chunks",
                    &[&local_config.cloud_render_distance.to_string()],
                );
            } else if god_rays.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.god_rays"),
                    if local_config.god_rays_enabled {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if transmission.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.transmission"),
                    if local_config.transmission_enabled {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if mesh_uploads.is_some() {
                text.0 = loc.t_args("common.chunks", &[&local_config.mesh_upload_budget.to_string()]);
            } else if proc_grass.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.procedural_grass"),
                    if local_config.grass_enabled {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if grass_style.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.grass_style"),
                    format_grass_style(&local_config.grass_style, &loc)
                );
            } else if grass_density.is_some() {
                text.0 = format_grass_density(local_config.grass_density, &loc);
            } else if coated_textures.is_some() {
                text.0 = format_coated_textures(local_config.coated_textures, &loc);
            } else if waving_foliage.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.waving_foliage"),
                    if local_config.waving_foliage {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if leaf_culling.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.leaf_culling"),
                    if local_config.leaf_culling {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if connected_textures.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.connected_textures"),
                    if local_config.connected_textures {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if menu_pause.is_some() {
                text.0 = format_menu_pause(game_settings.menu_pause_mode, &loc);
            } else if pixelated_shadows.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.pixelated_shadows"),
                    if local_config.pixelated_shadows {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if hdr_bloom.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.hdr_bloom"),
                    if local_config.hdr_bloom {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if dynamic_lighting.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.dynamic_lighting"),
                    if local_config.dynamic_lighting {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            }
        }
        if game_changed {
            if gui_scale.is_some() {
                text.0 = format!("{}x", game_settings.gui_scale);
            } else if menu_blur.is_some() {
                text.0 = format_blur(game_settings.menu_blur_intensity, &loc);
            } else if background_dimming.is_some() {
                text.0 = format_dimming(game_settings.background_dimming, &loc);
            } else if screen_mode.is_some() {
                text.0 = format_screen_mode(game_settings.screen_mode, &loc);
            } else if fov.is_some() {
                text.0 = format!("{}°", game_settings.fov_degrees as i32);
            } else if dynamic_fov.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.fov_effects"),
                    if game_settings.dynamic_fov_enabled {
                        loc.t("common.normal")
                    } else {
                        loc.t("common.reduced")
                    }
                );
            } else if fog.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.fog"),
                    if game_settings.fog_enabled {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if bobbing.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.view_bobbing"),
                    if game_settings.view_bobbing {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if grass.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.full_grass"),
                    if game_settings.full_grass {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if vsync.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.vsync"),
                    if game_settings.vsync_enabled {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            } else if dyn_fps.is_some() {
                text.0 = format!(
                    "{}: {}",
                    loc.t("menu.settings.dynamic_fps"),
                    if game_settings.dynamic_fps_enabled {
                        loc.t("common.enabled")
                    } else {
                        loc.t("common.disabled")
                    }
                );
            }
        }
        if chunk_changed {
            if render_dist.is_some() {
                text.0 = loc.t_args("common.chunks", &[&chunk_settings.render_distance.to_string()]);
            } else if lod_dist.is_some() {
                text.0 = if chunk_settings.lod_render_distance == 0 {
                    loc.t("common.disabled").to_string()
                } else {
                    loc.t_args("common.chunks", &[&chunk_settings.lod_render_distance.to_string()])
                };
            } else if sim_dist.is_some() {
                text.0 = loc.t_args("common.chunks", &[&chunk_settings.simulation_distance.to_string()]);
            } else if mesh_uploads.is_some() {
                text.0 = loc.t_args("common.chunks", &[&chunk_settings.mesh_upload_budget.to_string()]);
            }
        }
        if (env_changed || time_pause.is_some()) && time_pause.is_some() {
            let paused = env_state
                .as_ref()
                .map_or(local_config.is_time_paused, |e| e.is_time_paused);
            text.0 = format!(
                "{}: {}",
                loc.t("menu.settings.time_flow"),
                if paused {
                    loc.t("common.paused")
                } else {
                    loc.t("common.running")
                }
            );
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

fn update_language_label(
    loc: Res<Localization>,
    mut query: Query<&mut Text, With<LanguageLabel>>,
) {
    if !loc.is_changed() {
        return;
    }
    for mut text in &mut query {
        text.0 = loc.active_locale.display_name().to_string();
    }
}

fn update_settings_tab_view(
    settings_tab: Res<SettingsTab>,
    detail_open: Res<SettingsDetailOpen>,
    tab_buttons_query: Query<(&TabButton, &Hovered, &Children)>,
    mut text_color_query: Query<&mut TextColor, With<TabButtonLabel>>,
    mut accent_query: Query<&mut Visibility, With<TabButtonAccent>>,
    mut detail_panel_query: Query<&mut Node, (With<SettingsDetailPanel>, Without<SettingsSection>)>,
    mut sections_query: Query<(&SettingsSection, &mut Node), Without<SettingsDetailPanel>>,
) {
    use crate::menu::theme::palette;

    for (tab_btn, hovered, children) in &tab_buttons_query {
        let is_active = detail_open.0 && tab_btn.0 == *settings_tab;
        let is_highlighted = is_active || hovered.get();
        let color = if is_highlighted {
            palette::TEXT_ACCENT
        } else {
            palette::TEXT_PRIMARY
        };
        for child in children.iter() {
            if let Ok(mut text_color) = text_color_query.get_mut(child) {
                text_color.0 = color;
            }
            if let Ok(mut vis) = accent_query.get_mut(child) {
                *vis = if is_highlighted {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }

    if !settings_tab.is_changed() && !detail_open.is_changed() {
        return;
    }

    for mut node in &mut detail_panel_query {
        node.display = if detail_open.0 {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (sec, mut node) in &mut sections_query {
        node.display = if sec.0 == *settings_tab {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn update_settings_card_scale(
    _game_settings: Res<GameSettings>,
    _card_query: Query<&mut UiTransform, With<SettingsMenuCard>>,
) {
}

fn update_extended_settings_labels(
    game_settings: Res<GameSettings>,
    local_config: Res<LocalConfig>,
    loc: Res<Localization>,
    mut borderless_row_q: Query<&mut Node, With<BorderlessMouseLockRow>>,
    mut labels_q: Query<(
        &mut Text,
        (
            Option<&MouseLockBorderlessLabel>,
            Option<&AutoSaveLabel>,
            Option<&CameraSmoothingLabel>,
            Option<&ThirdPersonDistLabel>,
            Option<&ThirdPersonShoulderLabel>,
            Option<&ThirdPersonFreeOrbitLabel>,
            Option<&UiVolLabel>,
            Option<&MuteUnfocusLabel>,
            Option<&FpsHudLabel>,
            Option<&HighContrastCrosshairLabel>,
            Option<&DynamicCrosshairLabel>,
            Option<&MainHandLabel>,
            Option<&SubtitlesLabel>,
        ),
    )>,
) {
    if !game_settings.is_changed() && !local_config.is_changed() && !loc.is_changed() {
        return;
    }

    let is_borderless = game_settings.screen_mode == ScreenMode::BorderlessFullscreen;
    for mut node in &mut borderless_row_q {
        node.display = if is_borderless {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (
        mut text,
        (
            mouse_lock,
            auto_save,
            cam_smooth,
            tp_dist,
            tp_shoulder,
            tp_free_orbit,
            ui_vol,
            mute_unfocus,
            fps_hud,
            crosshair,
            dyn_crosshair,
            main_hand,
            subtitles,
        ),
    ) in &mut labels_q
    {
        if mouse_lock.is_some() {
            text.0 = format!(
                "{}: {}",
                loc.t("menu.settings.mouse_lock_borderless"),
                if game_settings.mouse_lock_borderless {
                    loc.t("common.enabled")
                } else {
                    loc.t("common.disabled")
                }
            );
        } else if auto_save.is_some() {
            text.0 = format_auto_save(game_settings.auto_save_minutes, &loc);
        } else if cam_smooth.is_some() {
            text.0 = format_camera_smoothing(game_settings.camera_smoothing, &loc);
        } else if tp_dist.is_some() {
            text.0 = format!("{:.1}m", game_settings.third_person_distance);
        } else if tp_shoulder.is_some() {
            text.0 = format!(
                "{}: {}",
                loc.t("menu.settings.third_person_style"),
                if game_settings.third_person_over_shoulder {
                    loc.t("menu.settings.third_person_shoulder")
                } else {
                    loc.t("menu.settings.third_person_centered")
                }
            );
        } else if tp_free_orbit.is_some() {
            text.0 = format!(
                "{}: {}",
                loc.t("menu.settings.third_person_free_orbit"),
                if game_settings.third_person_free_orbit {
                    loc.t("common.enabled")
                } else {
                    loc.t("common.disabled")
                }
            );
        } else if ui_vol.is_some() {
            text.0 = crate::core::config::format_volume(game_settings.ui_volume);
        } else if mute_unfocus.is_some() {
            text.0 = format!(
                "{}: {}",
                loc.t("menu.settings.mute_on_unfocus"),
                if game_settings.mute_on_unfocus {
                    loc.t("common.enabled")
                } else {
                    loc.t("common.disabled")
                }
            );
        } else if fps_hud.is_some() {
            text.0 = format_fps_hud(local_config.fps_hud_mode, &loc);
        } else if crosshair.is_some() {
            text.0 = format!(
                "{}: {}",
                loc.t("menu.settings.high_contrast_crosshair"),
                if game_settings.high_contrast_crosshair {
                    loc.t("common.enabled")
                } else {
                    loc.t("common.disabled")
                }
            );
        } else if dyn_crosshair.is_some() {
            text.0 = format!(
                "{}: {}",
                loc.t("menu.settings.dynamic_crosshair"),
                if game_settings.dynamic_crosshair {
                    loc.t("common.enabled")
                } else {
                    loc.t("common.disabled")
                }
            );
        } else if main_hand.is_some() {
            text.0 = format!(
                "{}: {}",
                loc.t("menu.settings.main_hand"),
                if game_settings.left_handed {
                    loc.t("menu.settings.hand_left")
                } else {
                    loc.t("menu.settings.hand_right")
                }
            );
        } else if subtitles.is_some() {
            text.0 = format!(
                "{}: {}",
                loc.t("menu.settings.subtitles"),
                if game_settings.subtitles_enabled {
                    loc.t("common.enabled")
                } else {
                    loc.t("common.disabled")
                }
            );
        }
    }
}

