use bevy::prelude::*;
use bevy::{picking::hover::Hovered, ui::Pressed, ui_widgets::Button};

use crate::{
    core::{AppFont, Localization, text_shadow_default},
    environment::EnvironmentState,
    gameplay::SelectedVoxel,
    generation::TerrainGenerator,
    meshing::{ChunkMaterial, ChunkMeshRegistry, sync_chunk_render},
    player::{PLAYER_EYE_HEIGHT, Player, PlayerCamera, PlayerMotion, hotbar::Hotbar},
    simulation::{VoxelLightRegistry, sync_chunk_lights},
    world::{VoxelWorld, WorldModificationStore},
};

use super::{MenuReturnTarget, MenuState, menu_bar::MENU_BAR_HEIGHT, theme::palette};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PauseSubmenu {
    #[default]
    None,
    WorldSession,
}

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct WorldSessionPanel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum PauseMenuAction {
    Resume,
    SaveWorld,
    Settings,
    Controls,
    WorldSession,
    Restart,
    Quit,
    ExitGame,
}

#[derive(Component)]
struct PauseButtonLabel;

#[derive(Component)]
struct PauseButtonAccent;

#[derive(Component)]
struct ToggleUndiscoveredCompendiumButton;

#[derive(Component)]
struct ToggleUndiscoveredCompendiumBadge;

#[derive(Component)]
struct TogglePreloadWorldMapButton;

#[derive(Component)]
struct TogglePreloadWorldMapBadge;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PauseSubmenu>().add_systems(
            Update,
            (
                sync_pause_sidebar_visibility,
                handle_pause_menu_buttons,
                update_pause_button_hover,
                sync_world_session_panel,
                handle_session_compendium_toggle,
            ),
        );
    }
}

fn sync_pause_sidebar_visibility(
    mut commands: Commands,
    menu_state: Res<State<MenuState>>,
    menu_return_target: Res<MenuReturnTarget>,
    world_session: Option<Res<crate::world::WorldSession>>,
    mut pause_submenu: ResMut<PauseSubmenu>,
    root_query: Query<Entity, With<PauseMenuRoot>>,
    loc: Option<Res<Localization>>,
    app_font: Option<Res<AppFont>>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
) {
    let in_active_session = world_session.as_ref().map_or_else(
        || menu_return_target.0 != MenuState::MainMenu,
        |s| s.is_active,
    );
    let current = *menu_state.get();
    let should_show = in_active_session
        && matches!(
            current,
            MenuState::Pause | MenuState::Settings | MenuState::Controls
        );
    let existing = root_query.iter().next();

    if current != MenuState::Pause && *pause_submenu != PauseSubmenu::None {
        *pause_submenu = PauseSubmenu::None;
    }

    if should_show && existing.is_none() {
        *pause_submenu = PauseSubmenu::None;
        spawn_pause_sidebar(
            &mut commands,
            loc.as_deref(),
            app_font.as_ref(),
            game_settings.as_deref(),
        );
    } else if !should_show && let Some(entity) = existing {
        *pause_submenu = PauseSubmenu::None;
        commands.entity(entity).despawn();
    }
}

fn spawn_pause_sidebar(
    commands: &mut Commands,
    loc: Option<&Localization>,
    app_font: Option<&Res<AppFont>>,
    game_settings: Option<&crate::menu::GameSettings>,
) {
    let font_handle = app_font.map(|f| f.source());

    let make_font = |size: f32| {
        let mut tf = TextFont {
            font_size: FontSize::Px(size),
            ..default()
        };
        if let Some(ref font) = font_handle {
            tf.font = font.clone();
        }
        tf
    };

    let dimming_alpha = game_settings.map_or(0.65, |s| s.background_dimming.alpha());

    commands
        .spawn((
            PauseMenuRoot,
            crate::menu::MenuBackdrop,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(MENU_BAR_HEIGHT),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, dimming_alpha)),
            ZIndex(290),
        ))
        .with_children(|backdrop| {
            // Column 1: Left-Docked Full-Height System Drawer
            backdrop
                .spawn((
                    Node {
                        width: px(palette::DRAWER_COL1_WIDTH),
                        min_width: px(palette::DRAWER_COL1_WIDTH),
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
                    BackgroundColor(palette::DRAWER_COL1_BG),
                    BorderColor::all(palette::DRAWER_DIVIDER),
                ))
                .with_children(|sidebar| {
                    let resume_lbl = loc.map_or("Resume Game", |l| l.t("menu.pause.resume"));
                    let save_lbl = loc.map_or("Save World", |l| l.t("menu.pause.save_world"));
                    let settings_lbl =
                        loc.map_or("Settings & Options", |l| l.t("menu.pause.settings"));
                    let controls_lbl =
                        loc.map_or("Controls & Keybinds", |l| l.t("menu.pause.controls"));
                    let session_lbl =
                        loc.map_or("World & Session", |l| l.t("menu.pause.world_session"));
                    let restart_lbl = loc.map_or("Restart World", |l| l.t("menu.pause.restart"));
                    let quit_lbl = loc.map_or("Return to Main Menu", |l| l.t("menu.pause.quit"));
                    let exit_lbl = loc.map_or("Exit Game", |l| l.t("menu.pause.exit_game"));

                    // Block 1: Session actions
                    for (action, label, has_submenu) in [
                        (PauseMenuAction::Resume, resume_lbl, false),
                        (PauseMenuAction::SaveWorld, save_lbl, false),
                    ] {
                        spawn_sidebar_item(sidebar, action, label, has_submenu, &make_font);
                    }

                    sidebar.spawn((
                        Node {
                            height: px(1.0),
                            margin: UiRect::new(px(24.0), px(24.0), px(12.0), px(12.0)),
                            ..default()
                        },
                        BackgroundColor(palette::PANEL_BORDER_SUBTLE),
                    ));

                    // Block 2: Configuration & Submenus
                    for (action, label, has_submenu) in [
                        (PauseMenuAction::Settings, settings_lbl, true),
                        (PauseMenuAction::Controls, controls_lbl, true),
                        (PauseMenuAction::WorldSession, session_lbl, true),
                    ] {
                        spawn_sidebar_item(sidebar, action, label, has_submenu, &make_font);
                    }

                    // Push Block 3 to the bottom of the full-height sidebar
                    sidebar.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });

                    sidebar.spawn((
                        Node {
                            height: px(1.0),
                            margin: UiRect::new(px(24.0), px(24.0), px(10.0), px(10.0)),
                            ..default()
                        },
                        BackgroundColor(palette::PANEL_BORDER_SUBTLE),
                    ));

                    // Block 3: Destructive / Exit actions anchored at footer
                    for (action, label, has_submenu) in [
                        (PauseMenuAction::Restart, restart_lbl, false),
                        (PauseMenuAction::Quit, quit_lbl, false),
                        (PauseMenuAction::ExitGame, exit_lbl, false),
                    ] {
                        spawn_sidebar_item(sidebar, action, label, has_submenu, &make_font);
                    }

                    // Subtle version footer
                    sidebar
                        .spawn(Node {
                            padding: UiRect::new(px(26.0), px(24.0), px(12.0), px(0.0)),
                            ..default()
                        })
                        .with_children(|foot| {
                            foot.spawn((
                                Text::new("v0.3.0"),
                                make_font(12.0),
                                TextColor(Color::srgba(0.55, 0.59, 0.67, 0.60)),
                            ));
                        });
                });

            // Column 2: World & Session Submenu Drawer (placeholder multiplayer/session options)
            backdrop
                .spawn((
                    WorldSessionPanel,
                    Node {
                        width: px(palette::DRAWER_COL2_WIDTH),
                        min_width: px(palette::DRAWER_COL2_WIDTH),
                        flex_shrink: 0.0,
                        height: percent(100.0),
                        border: UiRect::right(px(1.5)),
                        padding: UiRect::vertical(px(28.0)),
                        display: Display::None,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Stretch,
                        row_gap: px(4.0),
                        ..default()
                    },
                    BackgroundColor(palette::DRAWER_COL2_BG),
                    BorderColor::all(palette::DRAWER_DIVIDER),
                ))
                .with_children(|panel| {
                    let show_undiscovered = game_settings
                        .is_some_and(|s| s.show_undiscovered_compendium);
                    let compendium_toggle_lbl = loc.map_or("Show Undiscovered Compendium", |l| {
                        l.t("menu.session.show_undiscovered_compendium")
                    });
                    let badge_str = if show_undiscovered {
                        loc.map_or("ON", |l| l.t("map.state.on"))
                    } else {
                        loc.map_or("OFF", |l| l.t("map.state.off"))
                    };

                    // Functional top button: Show Undiscovered Compendium
                    panel
                        .spawn((
                            Button,
                            Hovered::default(),
                            ToggleUndiscoveredCompendiumButton,
                            Node {
                                width: percent(100.0),
                                height: px(46.0),
                                padding: UiRect::new(px(24.0), px(20.0), px(0.0), px(0.0)),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                                column_gap: px(12.0),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(compendium_toggle_lbl),
                                make_font(14.0),
                                TextColor(palette::TEXT_PRIMARY),
                                text_shadow_default(),
                            ));
                            row.spawn((
                                ToggleUndiscoveredCompendiumBadge,
                                Text::new(badge_str),
                                make_font(12.0),
                                TextColor(if show_undiscovered {
                                    palette::TEXT_ACCENT
                                } else {
                                    palette::TEXT_MUTED
                                }),
                                text_shadow_default(),
                            ));
                        });

                    // Functional top button 2: Preload Full World Map
                    let preload_map = game_settings.is_some_and(|s| s.preload_full_world_map);
                    let preload_toggle_lbl = loc.map_or("Preload Full World Map", |l| {
                        l.t("menu.session.preload_full_world_map")
                    });
                    let preload_badge_str = if preload_map {
                        loc.map_or("ON", |l| l.t("map.state.on"))
                    } else {
                        loc.map_or("OFF", |l| l.t("map.state.off"))
                    };

                    panel
                        .spawn((
                            Button,
                            Hovered::default(),
                            TogglePreloadWorldMapButton,
                            Node {
                                width: percent(100.0),
                                height: px(46.0),
                                padding: UiRect::new(px(24.0), px(20.0), px(0.0), px(0.0)),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                                column_gap: px(12.0),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(preload_toggle_lbl),
                                make_font(14.0),
                                TextColor(palette::TEXT_PRIMARY),
                                text_shadow_default(),
                            ));
                            row.spawn((
                                TogglePreloadWorldMapBadge,
                                Text::new(preload_badge_str),
                                make_font(12.0),
                                TextColor(if preload_map {
                                    palette::TEXT_ACCENT
                                } else {
                                    palette::TEXT_MUTED
                                }),
                                text_shadow_default(),
                            ));
                        });

                    // Subtle divider below active session toggles
                    panel.spawn((
                        Node {
                            height: px(1.0),
                            margin: UiRect::new(px(20.0), px(20.0), px(6.0), px(6.0)),
                            ..default()
                        },
                        BackgroundColor(palette::DRAWER_DIVIDER),
                    ));

                    let session_items = [
                        (
                            loc.map_or("Open to LAN", |l| l.t("menu.session.open_lan")),
                            "Soon",
                        ),
                        (
                            loc.map_or("Host Online Session", |l| l.t("menu.session.host_online")),
                            "Soon",
                        ),
                        (
                            loc.map_or("Invite Players", |l| l.t("menu.session.invite_players")),
                            "Soon",
                        ),
                        (
                            loc.map_or("Session Privacy", |l| l.t("menu.session.privacy")),
                            "Solo",
                        ),
                        (
                            loc.map_or("Game Mode", |l| l.t("menu.session.game_mode")),
                            "Creative",
                        ),
                        (
                            loc.map_or("Difficulty", |l| l.t("menu.session.difficulty")),
                            "Normal",
                        ),
                    ];

                    for (label, badge) in session_items {
                        panel
                            .spawn((
                                Node {
                                    width: percent(100.0),
                                    height: px(46.0),
                                    padding: UiRect::new(px(24.0), px(20.0), px(0.0), px(0.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|row| {
                                row.spawn((
                                    Text::new(label),
                                    make_font(15.0),
                                    TextColor(palette::TEXT_MUTED),
                                    text_shadow_default(),
                                ));
                                row.spawn((
                                    Text::new(badge),
                                    make_font(12.0),
                                    TextColor(Color::srgba(0.62, 0.66, 0.74, 0.65)),
                                ));
                            });
                    }
                });
        });
}

fn spawn_sidebar_item(
    parent: &mut ChildSpawnerCommands,
    action: PauseMenuAction,
    label: &str,
    has_submenu: bool,
    make_font: &impl Fn(f32) -> TextFont,
) {
    parent
        .spawn((
            Button,
            Hovered::default(),
            action,
            Node {
                width: percent(100.0),
                height: px(48.0),
                padding: UiRect::new(px(26.0), px(22.0), px(0.0), px(0.0)),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|btn| {
            // Left gold accent bar (visible on active / hovered)
            btn.spawn((
                PauseButtonAccent,
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
                Visibility::Hidden,
            ));

            btn.spawn((
                PauseButtonLabel,
                Text::new(label),
                make_font(16.0),
                TextColor(palette::TEXT_PRIMARY),
                text_shadow_default(),
            ));

            if has_submenu {
                btn.spawn((
                    Text::new(">"),
                    make_font(15.0),
                    TextColor(palette::TEXT_MUTED),
                ));
            }
        });
}

fn sync_world_session_panel(
    pause_submenu: Res<PauseSubmenu>,
    menu_state: Res<State<MenuState>>,
    mut panel_query: Query<&mut Node, With<WorldSessionPanel>>,
) {
    if !pause_submenu.is_changed() && !menu_state.is_changed() {
        return;
    }
    let show =
        *menu_state.get() == MenuState::Pause && *pause_submenu == PauseSubmenu::WorldSession;
    for mut node in &mut panel_query {
        node.display = if show { Display::Flex } else { Display::None };
    }
}

fn handle_session_compendium_toggle(
    mouse: Res<ButtonInput<MouseButton>>,
    loc: Option<Res<Localization>>,
    mut game_settings: Option<ResMut<crate::menu::GameSettings>>,
    mut local_config: Option<ResMut<crate::core::LocalConfig>>,
    mut compendium_btn_q: Query<
        (&Hovered, Has<Pressed>, &mut BackgroundColor),
        (
            With<Button>,
            With<ToggleUndiscoveredCompendiumButton>,
            Without<TogglePreloadWorldMapButton>,
        ),
    >,
    mut preload_btn_q: Query<
        (&Hovered, Has<Pressed>, &mut BackgroundColor),
        (
            With<Button>,
            With<TogglePreloadWorldMapButton>,
            Without<ToggleUndiscoveredCompendiumButton>,
        ),
    >,
    mut compendium_badge_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<ToggleUndiscoveredCompendiumBadge>,
            Without<TogglePreloadWorldMapBadge>,
        ),
    >,
    mut preload_badge_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<TogglePreloadWorldMapBadge>,
            Without<ToggleUndiscoveredCompendiumBadge>,
        ),
    >,
) {
    let Some(ref mut settings) = game_settings else {
        return;
    };

    let clicked = mouse.just_pressed(MouseButton::Left);
    let mut toggled = false;

    for (hovered, is_pressed, mut bg) in &mut compendium_btn_q {
        let is_hovered = hovered.get() || is_pressed;
        bg.0 = if is_hovered {
            palette::NAV_ITEM_HOVER_BG
        } else {
            Color::NONE
        };
        if clicked && hovered.get() {
            settings.show_undiscovered_compendium = !settings.show_undiscovered_compendium;
            toggled = true;
        }
    }

    for (hovered, is_pressed, mut bg) in &mut preload_btn_q {
        let is_hovered = hovered.get() || is_pressed;
        bg.0 = if is_hovered {
            palette::NAV_ITEM_HOVER_BG
        } else {
            Color::NONE
        };
        if clicked && hovered.get() {
            settings.preload_full_world_map = !settings.preload_full_world_map;
            toggled = true;
        }
    }

    if toggled && let Some(ref mut cfg) = local_config {
        cfg.show_undiscovered_compendium = settings.show_undiscovered_compendium;
        cfg.preload_full_world_map = settings.preload_full_world_map;
        let _ = cfg.save();
    }

    if toggled || settings.is_changed() {
        let on_str = loc.as_deref().map_or("ON", |l| l.t("map.state.on"));
        let off_str = loc.as_deref().map_or("OFF", |l| l.t("map.state.off"));

        let comp_enabled = settings.show_undiscovered_compendium;
        for (mut text, mut color) in &mut compendium_badge_q {
            text.0 = if comp_enabled { on_str } else { off_str }.to_string();
            color.0 = if comp_enabled {
                palette::TEXT_ACCENT
            } else {
                palette::TEXT_MUTED
            };
        }

        let preload_enabled = settings.preload_full_world_map;
        for (mut text, mut color) in &mut preload_badge_q {
            text.0 = if preload_enabled { on_str } else { off_str }.to_string();
            color.0 = if preload_enabled {
                palette::TEXT_ACCENT
            } else {
                palette::TEXT_MUTED
            };
        }
    }
}

fn update_pause_button_hover(
    menu_state: Res<State<MenuState>>,
    pause_submenu: Res<PauseSubmenu>,
    mut buttons_query: Query<
        (
            &Hovered,
            Has<Pressed>,
            &PauseMenuAction,
            &mut BackgroundColor,
            &Children,
        ),
        With<Button>,
    >,
    mut labels_query: Query<&mut TextColor, With<PauseButtonLabel>>,
    mut accents_query: Query<&mut Visibility, With<PauseButtonAccent>>,
) {
    let current = *menu_state.get();
    for (hovered, is_pressed, action, mut bg, children) in &mut buttons_query {
        let is_selected_submenu = matches!(
            (action, current),
            (PauseMenuAction::Settings, MenuState::Settings)
                | (PauseMenuAction::Controls, MenuState::Controls)
        ) || (*action == PauseMenuAction::WorldSession
            && current == MenuState::Pause
            && *pause_submenu == PauseSubmenu::WorldSession);
        let is_hovered = hovered.get() || is_pressed;

        bg.0 = if is_selected_submenu {
            palette::NAV_ITEM_ACTIVE_BG
        } else if is_hovered {
            palette::NAV_ITEM_HOVER_BG
        } else {
            Color::NONE
        };

        for child in children.iter() {
            if let Ok(mut text_color) = labels_query.get_mut(child) {
                text_color.0 = if is_selected_submenu || is_hovered {
                    palette::TEXT_ACCENT
                } else {
                    palette::TEXT_PRIMARY
                };
            }
            if let Ok(mut vis) = accents_query.get_mut(child) {
                *vis = if is_selected_submenu || is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

type PauseRestartParams<'w, 's> = (
    Option<ResMut<'w, VoxelWorld>>,
    Option<ResMut<'w, WorldModificationStore>>,
    Option<Res<'w, TerrainGenerator>>,
    Option<ResMut<'w, VoxelLightRegistry>>,
    Option<ResMut<'w, ChunkMeshRegistry>>,
    Option<ResMut<'w, Assets<Mesh>>>,
    Option<Res<'w, ChunkMaterial>>,
    Option<ResMut<'w, Hotbar>>,
    Option<ResMut<'w, SelectedVoxel>>,
    Option<ResMut<'w, crate::map::cache::MapCache>>,
);

type PauseWeatherRestartParams<'w> = (
    Option<ResMut<'w, crate::environment::weather::WeatherManager>>,
    Option<ResMut<'w, crate::environment::weather::LightningState>>,
    Option<ResMut<'w, crate::environment::weather::WindState>>,
    Option<ResMut<'w, crate::environment::weather::WeatherParticlePool>>,
    Option<ResMut<'w, crate::audio::weather_audio::PendingThunderQueue>>,
    Option<ResMut<'w, crate::player::PlayerInventory>>,
    Option<Res<'w, crate::player::WorldSpawnPoint>>,
);

type PauseSessionParams<'w> = (
    Option<ResMut<'w, crate::world::WorldSession>>,
    Option<ResMut<'w, crate::menu::main_menu::MainMenuPhase>>,
    Option<ResMut<'w, crate::meshing::LodMeshRegistry>>,
    Option<ResMut<'w, crate::world::streaming::ChunkStreamingState>>,
    Option<ResMut<'w, crate::world::streaming::ChunkStreamingQueues>>,
);

fn handle_pause_menu_buttons(
    buttons_query: Query<(&Hovered, &PauseMenuAction), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    menu_state: Res<State<MenuState>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut menu_return_target: ResMut<MenuReturnTarget>,
    mut env_state: Option<ResMut<EnvironmentState>>,
    mut player_query: Query<(&mut Transform, &mut PlayerMotion), With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), (With<Camera3d>, Without<Player>)>,
    (
        mut world_query,
        mut modifications_query,
        terrain_generator_query,
        mut light_registry_query,
        mut mesh_registry_query,
        mut meshes_query,
        chunk_material_query,
        mut hotbar_query,
        mut selected_voxel_query,
        mut map_cache_query,
    ): PauseRestartParams,
    (
        mut weather_query,
        mut lightning_query,
        mut wind_query,
        mut particle_pool_query,
        mut pending_thunder_query,
        mut player_inv_query,
        world_spawn_query,
    ): PauseWeatherRestartParams,
    (
        mut world_session,
        mut main_menu_phase,
        mut lod_registry_query,
        mut streaming_state_query,
        mut streaming_queues_query,
    ): PauseSessionParams,
    lightning_entities_query: Query<
        Entity,
        Or<(
            With<crate::environment::weather::lightning::LightningBoltEntity>,
            With<crate::environment::weather::lightning::LightningImpactLight>,
        )>,
    >,
    mut commands: Commands,
    mut pause_submenu: ResMut<PauseSubmenu>,
    mut app_exit: MessageWriter<bevy::app::AppExit>,
) {
    let left_clicked = mouse.just_pressed(MouseButton::Left);
    if !left_clicked {
        return;
    }

    let current = *menu_state.get();
    for (hovered, action) in &buttons_query {
        if hovered.get() {
            match action {
                PauseMenuAction::Resume => {
                    *pause_submenu = PauseSubmenu::None;
                    next_state.set(MenuState::None);
                }
                PauseMenuAction::SaveWorld => {
                    info!("Save World placeholder triggered.");
                }
                PauseMenuAction::Settings => {
                    *pause_submenu = PauseSubmenu::None;
                    menu_return_target.0 = MenuState::Pause;
                    if current == MenuState::Settings {
                        next_state.set(MenuState::Pause);
                    } else {
                        next_state.set(MenuState::Settings);
                    }
                }
                PauseMenuAction::Controls => {
                    *pause_submenu = PauseSubmenu::None;
                    menu_return_target.0 = MenuState::Pause;
                    if current == MenuState::Controls {
                        next_state.set(MenuState::Pause);
                    } else {
                        next_state.set(MenuState::Controls);
                    }
                }
                PauseMenuAction::WorldSession => {
                    if current != MenuState::Pause {
                        next_state.set(MenuState::Pause);
                        *pause_submenu = PauseSubmenu::WorldSession;
                    } else if *pause_submenu == PauseSubmenu::WorldSession {
                        *pause_submenu = PauseSubmenu::None;
                    } else {
                        *pause_submenu = PauseSubmenu::WorldSession;
                    }
                }
                PauseMenuAction::Restart => {
                    // 1. Reset Environment to default Day 1 Noon
                    if let Some(ref mut env) = env_state {
                        **env = EnvironmentState::default();
                    }

                    // 2. Reset Weather, Lightning, Wind, and clear active particles / pending thunder
                    if let Some(ref mut weather) = weather_query {
                        **weather = crate::environment::weather::WeatherManager::default();
                    }
                    if let Some(ref mut lightning) = lightning_query {
                        **lightning = crate::environment::weather::LightningState::default();
                    }
                    if let Some(ref mut wind) = wind_query {
                        **wind = crate::environment::weather::WindState::default();
                    }
                    if let Some(ref mut particle_pool) = particle_pool_query {
                        particle_pool.clear();
                    }
                    if let Some(ref mut pending_thunder) = pending_thunder_query {
                        pending_thunder.0.clear();
                    }
                    for entity in &lightning_entities_query {
                        commands.entity(entity).despawn();
                    }

                    // 3. Teleport player and camera to active world spawn (respects Adventure Mode chosen spawn!)
                    let spawn_pos = world_spawn_query
                        .as_deref()
                        .map_or(Vec3::new(-10.0, 10.38, 14.0), |ws| ws.0);
                    for (mut transform, mut motion) in &mut player_query {
                        transform.translation = spawn_pos;
                        motion.velocity = Vec3::ZERO;
                        motion.flying = false;
                    }

                    for (mut transform, mut camera) in &mut camera_query {
                        let cam_pos = spawn_pos + Vec3::Y * PLAYER_EYE_HEIGHT;
                        let look_target = cam_pos + Vec3::new(14.0, -2.0, -10.0);
                        *transform = Transform::from_translation(cam_pos)
                            .looking_at(look_target, Vec3::Y);
                        let new_pc = PlayerCamera::from_transform(&transform);
                        *camera = new_pc;
                    }

                    // 4. Reverse world modifications
                    if let (
                        Some(ref mut world),
                        Some(ref mut modifications),
                        Some(terrain_gen),
                        Some(ref mut light_reg),
                        Some(ref mut mesh_reg),
                        Some(ref mut meshes),
                        Some(chunk_mat),
                    ) = (
                        world_query.as_deref_mut(),
                        modifications_query.as_deref_mut(),
                        terrain_generator_query.as_deref(),
                        light_registry_query.as_deref_mut(),
                        mesh_registry_query.as_deref_mut(),
                        meshes_query.as_deref_mut(),
                        chunk_material_query.as_deref(),
                    ) {
                        let modified_coords = modifications.modified_chunks();
                        modifications.clear();

                        for coord in modified_coords {
                            let fresh_chunk = terrain_gen.generate_chunk(coord);
                            world.insert_chunk(coord, fresh_chunk);
                            sync_chunk_lights(&mut commands, world, coord, light_reg);
                            sync_chunk_render(
                                &mut commands,
                                world,
                                coord,
                                mesh_reg,
                                meshes,
                                chunk_mat,
                            );
                        }
                    }

                    // 5. Reset hotbar, player inventory, and selected voxel
                    if let Some(ref mut hotbar) = hotbar_query {
                        **hotbar = Hotbar::default();
                    }
                    if let Some(ref mut player_inv) = player_inv_query {
                        **player_inv = crate::player::PlayerInventory::default();
                    }
                    if let Some(ref mut selected) = selected_voxel_query {
                        selected.0 = None;
                    }
                    if let Some(ref mut cache) = map_cache_query {
                        cache.clear();
                    }

                    info!(
                        "Game restarted: player teleported to spawn, world modifications reversed, day reset to 1, weather reset to Clear Sky"
                    );
                    next_state.set(MenuState::None);
                }
                PauseMenuAction::Quit => {
                    // 1. Deactivate world session
                    if let Some(ref mut session) = world_session {
                        session.is_active = false;
                    }

                    // 2. Clear voxel world
                    if let Some(ref mut world) = world_query {
                        world.clear();
                    }

                    // 3. Clear modifications and map cache
                    if let Some(ref mut modifications) = modifications_query {
                        modifications.clear();
                    }
                    if let Some(ref mut cache) = map_cache_query {
                        cache.clear();
                    }

                    // 4. Despawn chunk meshes & LOD meshes
                    if let Some(ref mut mesh_reg) = mesh_registry_query {
                        mesh_reg.clear(&mut commands);
                    }
                    if let Some(ref mut lod_reg) = lod_registry_query {
                        lod_reg.clear(&mut commands);
                    }

                    // 5. Reset streaming queues and state
                    if let Some(ref mut st) = streaming_state_query {
                        **st = crate::world::streaming::ChunkStreamingState::default();
                    }
                    if let Some(ref mut q) = streaming_queues_query {
                        **q = crate::world::streaming::ChunkStreamingQueues::default();
                    }

                    // 6. Reset player position and camera to spawn
                    let spawn_pos = Vec3::new(-10.0, 10.38, 14.0);
                    for (mut transform, mut motion) in &mut player_query {
                        transform.translation = spawn_pos;
                        motion.velocity = Vec3::ZERO;
                        motion.flying = false;
                    }
                    for (mut transform, mut camera) in &mut camera_query {
                        let cam_pos = spawn_pos + Vec3::Y * PLAYER_EYE_HEIGHT;
                        *transform = Transform::from_translation(cam_pos)
                            .looking_at(Vec3::new(4.0, 3.0, 4.0), Vec3::Y);
                        let new_pc = PlayerCamera::from_transform(&transform);
                        *camera = new_pc;
                    }

                    // 7. Reset environment & weather
                    if let Some(ref mut env) = env_state {
                        **env = EnvironmentState::default();
                    }
                    if let Some(ref mut weather) = weather_query {
                        **weather = crate::environment::weather::WeatherManager::default();
                    }
                    if let Some(ref mut lightning) = lightning_query {
                        **lightning = crate::environment::weather::LightningState::default();
                    }
                    if let Some(ref mut wind) = wind_query {
                        **wind = crate::environment::weather::WindState::default();
                    }
                    if let Some(ref mut particle_pool) = particle_pool_query {
                        particle_pool.clear();
                    }
                    if let Some(ref mut pending_thunder) = pending_thunder_query {
                        pending_thunder.0.clear();
                    }
                    for entity in &lightning_entities_query {
                        commands.entity(entity).despawn();
                    }

                    // 8. Reset hotbar and inventory
                    if let Some(ref mut hotbar) = hotbar_query {
                        **hotbar = Hotbar::default();
                    }
                    if let Some(ref mut player_inv) = player_inv_query {
                        **player_inv = crate::player::PlayerInventory::default();
                    }
                    if let Some(ref mut selected) = selected_voxel_query {
                        selected.0 = None;
                    }

                    // 9. Return to Main Menu with action buttons
                    if let Some(ref mut phase) = main_menu_phase {
                        **phase = crate::menu::main_menu::MainMenuPhase::ActionMenu;
                    }
                    menu_return_target.0 = MenuState::MainMenu;
                    next_state.set(MenuState::MainMenu);
                    info!("Exited session to Main Menu: world cleared and session ended");
                }
                PauseMenuAction::ExitGame => {
                    app_exit.write(bevy::app::AppExit::Success);
                }
            }
        }
    }
}
