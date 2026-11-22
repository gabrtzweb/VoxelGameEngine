use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    input::mouse::AccumulatedMouseScroll,
    math::Rot2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    text::FontSize,
    ui::UiTransform,
    window::PrimaryWindow,
};

use crate::{
    menu::{CursorMode, MenuState},
    player::{Player, PlayerCamera},
    world::{VOXEL_SIZE, Voxel},
};

use super::{
    cache::MapCache,
    color::{apply_relief_shading, unexplored_color},
};

pub const WORLD_MAP_WIDTH: u32 = 900;
pub const WORLD_MAP_HEIGHT: u32 = 440;
pub const WORLD_MAP_FRAME_WIDTH: f32 = 1920.0;
pub const WORLD_MAP_FRAME_HEIGHT: f32 = 1080.0;
pub const WORLD_MAP_VIEWPORT_WIDTH: f32 = 1800.0;
pub const WORLD_MAP_VIEWPORT_HEIGHT: f32 = 880.0;
pub const WORLD_MAP_VIEWPORT_LEFT: f32 = 60.0;
pub const WORLD_MAP_VIEWPORT_TOP: f32 = 140.0;
pub const WORLD_MARKER_SIZE: f32 = 28.0;

#[derive(Component)]
pub struct WorldMapRoot;

#[derive(Component)]
pub struct WorldMapFrame;

#[derive(Component)]
pub struct WorldMapViewport;

#[derive(Component)]
pub struct WorldMapPlayerMarker;

#[derive(Component)]
pub struct WorldMapInfoText;

#[derive(Component)]
pub struct WorldMapCursorText;

#[derive(Resource)]
pub struct WorldMapState {
    pub center: Vec2,
    pub zoom: f32,
    pub is_dragging: bool,
    pub last_cursor_screen_pos: Option<Vec2>,
    pub terrain_image: Handle<Image>,
    pub marker_image: Handle<Image>,
    pub last_drawn_center: Vec2,
    pub last_drawn_zoom: f32,
    pub last_drawn_aspect: f32,
    pub last_cache_version: u64,
}

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world_map_resources)
            .add_systems(OnEnter(MenuState::WorldMap), spawn_world_map_ui)
            .add_systems(OnExit(MenuState::WorldMap), despawn_world_map_ui)
            .add_systems(
                Update,
                (
                    handle_world_map_input,
                    handle_world_map_sidebar_buttons,
                    sync_world_map_terrain,
                    update_world_map_ui,
                )
                    .run_if(in_state(MenuState::WorldMap)),
            );
    }
}

fn setup_world_map_resources(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let terrain_pixels =
        [18u8, 19u8, 24u8, 255u8].repeat((WORLD_MAP_WIDTH * WORLD_MAP_HEIGHT) as usize);

    let mut terrain_img = Image::new_fill(
        Extent3d {
            width: WORLD_MAP_WIDTH,
            height: WORLD_MAP_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &terrain_pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    terrain_img.sampler = ImageSampler::nearest();
    let terrain_handle = images.add(terrain_img);

    let marker_handle: Handle<Image> =
        asset_server.load("textures/interfaces/atlases/decorations/marker_red.png");

    commands.insert_resource(WorldMapState {
        center: Vec2::ZERO,
        zoom: 1.0,
        is_dragging: false,
        last_cursor_screen_pos: None,
        terrain_image: terrain_handle,
        marker_image: marker_handle,
        last_drawn_center: Vec2::splat(999_999.0),
        last_drawn_zoom: -1.0,
        last_drawn_aspect: -1.0,
        last_cache_version: u64::MAX,
    });
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum WorldMapAction {
    CenterPlayer,
    CreateWaypoint,
}

fn spawn_world_map_ui(
    mut commands: Commands,
    mut map_state: ResMut<WorldMapState>,
    player_query: Query<&Transform, With<Player>>,
    app_font: Option<Res<crate::core::AppFont>>,
    loc: Option<Res<crate::core::Localization>>,
    _game_settings: Option<Res<crate::menu::GameSettings>>,
) {
    use bevy::{picking::hover::Hovered, ui_widgets::Button};
    use crate::menu::{ThemedButton, menu_bar::MENU_BAR_HEIGHT, theme::palette};

    let font_handle = app_font.as_ref().map(|f| f.source()).unwrap_or_default();

    let make_font = |size: f32| TextFont {
        font: font_handle.clone(),
        font_size: FontSize::Px(size),
        ..default()
    };

    // When opening the world map, center immediately on the player's position
    if let Ok(player_transform) = player_query.single() {
        let p = player_transform.translation;
        map_state.center = Vec2::new(p.x / VOXEL_SIZE, p.z / VOXEL_SIZE);
    }
    map_state.last_drawn_center = Vec2::splat(999_999.0); // Force redraw

    commands
        .spawn((
            WorldMapRoot,
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
            BackgroundColor(palette::PANEL_INNER_BG),
            ZIndex(250),
        ))
        .with_children(|root| {
            // Left / Main Interactive Full-Height World Map Viewport
            root.spawn((
                WorldMapFrame,
                WorldMapViewport,
                Node {
                    flex_grow: 1.0,
                    height: percent(100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ZIndex(5),
            ))
            .with_children(|viewport| {
                // Terrain Render Texture (fills viewport 100%)
                viewport.spawn((
                    ImageNode {
                        image: map_state.terrain_image.clone(),
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
                    ZIndex(1),
                ));

                // Player Directional Red Marker (percentage-positioned so it stays 100% locked at every zoom)
                let half_m = WORLD_MARKER_SIZE / 2.0;
                viewport.spawn((
                    WorldMapPlayerMarker,
                    ImageNode {
                        image: map_state.marker_image.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(WORLD_MARKER_SIZE),
                        height: px(WORLD_MARKER_SIZE),
                        left: percent(50.0),
                        top: percent(50.0),
                        margin: UiRect {
                            left: px(-half_m),
                            top: px(-half_m),
                            ..default()
                        },
                        ..default()
                    },
                    UiTransform::IDENTITY,
                    Visibility::Visible,
                    ZIndex(30),
                ));

                // Subtle bottom-left controls hint
                let hint_text = loc.as_ref().map_or(
                    "LMB Drag: Pan   |   Scroll: Zoom   |   Space: Center   |   [M / ESC] Close",
                    |l| l.t("map.controls_hint"),
                );
                viewport
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(18.0),
                            bottom: px(14.0),
                            padding: UiRect::axes(px(12.0), px(6.0)),
                            border_radius: BorderRadius::all(px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.78)),
                        ZIndex(40),
                    ))
                    .with_children(|pill| {
                        pill.spawn((
                            Text::new(hint_text),
                            make_font(13.0),
                            TextColor(palette::TEXT_MUTED),
                            crate::core::text_shadow_default(),
                        ));
                    });
            });

            // Right-Docked Full-Height Map Sidebar (standardized 310px width)
            root.spawn((
                Node {
                    width: px(palette::DRAWER_COL1_WIDTH),
                    min_width: px(palette::DRAWER_COL1_WIDTH),
                    flex_shrink: 0.0,
                    height: percent(100.0),
                    border: UiRect::left(px(1.5)),
                    padding: UiRect::all(px(22.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(18.0),
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL1_BG),
                BorderColor::all(palette::DRAWER_DIVIDER),
                ZIndex(10),
            ))
            .with_children(|sidebar| {
                // Quick Actions
                sidebar
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8.0),
                        ..default()
                    })
                    .with_children(|actions| {
                        actions.spawn((
                            Text::new("MAP TOOLS"),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        let btn_theme = ThemedButton::subtle();

                        // + New Waypoint button
                        actions
                            .spawn((
                                Button,
                                Hovered::default(),
                                WorldMapAction::CreateWaypoint,
                                btn_theme,
                                BackgroundColor(btn_theme.normal_bg),
                                BorderColor::all(palette::ACCENT_LINE),
                                Node {
                                    width: percent(100.0),
                                    height: px(38.0),
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::all(px(4.0)),
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("+ New Waypoint"),
                                    make_font(14.0),
                                    TextColor(palette::TEXT_ACCENT),
                                    crate::core::text_shadow_default(),
                                ));
                            });

                        // Center on Player button
                        actions
                            .spawn((
                                Button,
                                Hovered::default(),
                                WorldMapAction::CenterPlayer,
                                btn_theme,
                                BackgroundColor(btn_theme.normal_bg),
                                BorderColor::all(btn_theme.normal_border),
                                Node {
                                    width: percent(100.0),
                                    height: px(36.0),
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::all(px(4.0)),
                                    display: Display::Flex,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Center on Player"),
                                    make_font(14.0),
                                    TextColor(palette::TEXT_PRIMARY),
                                    crate::core::text_shadow_default(),
                                ));
                            });
                    });

                // Waypoints List (placeholder)
                sidebar
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8.0),
                        ..default()
                    })
                    .with_children(|wps| {
                        wps.spawn((
                            Text::new("WAYPOINTS"),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        wps.spawn((
                            Node {
                                width: percent(100.0),
                                padding: UiRect::all(px(12.0)),
                                border: UiRect::all(px(1.0)),
                                border_radius: BorderRadius::all(px(5.0)),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(palette::PANEL_INNER_BG),
                            BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                        ))
                        .with_children(|wp| {
                            wp.spawn((
                                Text::new("Spawn Camp"),
                                make_font(13.0),
                                TextColor(palette::TEXT_PRIMARY),
                            ));
                            wp.spawn((
                                Text::new("X: -10  Z: 14"),
                                make_font(12.0),
                                TextColor(palette::TEXT_MUTED),
                            ));
                        });
                    });

                // Map Layers / Filters (placeholder)
                sidebar
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|layers| {
                        layers.spawn((
                            Text::new("LAYERS"),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        for (name, state) in [
                            ("Player Marker", "ON"),
                            ("Waypoints", "ON"),
                            ("Biome Grid", "ON"),
                            ("Caves / Underground", "OFF"),
                        ] {
                            layers
                                .spawn(Node {
                                    width: percent(100.0),
                                    height: px(28.0),
                                    padding: UiRect::horizontal(px(10.0)),
                                    border_radius: BorderRadius::all(px(4.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .insert(BackgroundColor(palette::PANEL_INNER_BG))
                                .with_children(|row| {
                                    row.spawn((
                                        Text::new(name),
                                        make_font(13.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));
                                    row.spawn((
                                        Text::new(state),
                                        make_font(12.0),
                                        TextColor(if state == "ON" {
                                            palette::TEXT_ACCENT
                                        } else {
                                            palette::TEXT_MUTED
                                        }),
                                    ));
                                });
                        }
                    });

                // Push live coordinate readouts to the bottom of the sidebar
                sidebar.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                sidebar
                    .spawn((
                        Node {
                            width: percent(100.0),
                            padding: UiRect::all(px(12.0)),
                            border: UiRect::all(px(1.0)),
                            border_radius: BorderRadius::all(px(5.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6.0),
                            ..default()
                        },
                        BackgroundColor(palette::PANEL_INNER_BG),
                        BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                    ))
                    .with_children(|coords| {
                        coords.spawn((
                            WorldMapInfoText,
                            Text::new("Player: X: 0 Y: 0 Z: 0"),
                            make_font(13.0),
                            TextColor(palette::TEXT_PRIMARY),
                            crate::core::text_shadow_default(),
                        ));
                        coords.spawn((
                            WorldMapCursorText,
                            Text::new("Cursor: X: 0 Z: 0 | Zoom: 1.00x"),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                            crate::core::text_shadow_default(),
                        ));
                    });
            });
        });
}

fn handle_world_map_sidebar_buttons(
    mut buttons_query: Query<
        (
            &bevy::picking::hover::Hovered,
            Has<bevy::ui::Pressed>,
            &WorldMapAction,
            &crate::menu::ThemedButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<bevy::ui_widgets::Button>,
    >,
    mouse: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, With<Player>>,
    mut map_state: ResMut<WorldMapState>,
) {
    let clicked = mouse.just_pressed(MouseButton::Left);
    for (hovered, is_pressed, action, theme, mut bg, mut border) in &mut buttons_query {
        let (next_bg, next_border) = theme.colors(hovered.get(), is_pressed);
        bg.0 = next_bg;
        if *action != WorldMapAction::CreateWaypoint {
            *border = BorderColor::all(next_border);
        }
        if hovered.get() && clicked {
            match action {
                WorldMapAction::CenterPlayer => {
                    if let Ok(t) = player_query.single() {
                        let p = t.translation;
                        map_state.center = Vec2::new(p.x / VOXEL_SIZE, p.z / VOXEL_SIZE);
                    }
                }
                WorldMapAction::CreateWaypoint => {
                    info!("Create Waypoint placeholder triggered.");
                }
            }
        }
    }
}

fn despawn_world_map_ui(mut commands: Commands, query: Query<Entity, With<WorldMapRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_world_map_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    player_query: Query<&Transform, With<Player>>,
    mut map_state: ResMut<WorldMapState>,
    mut    cursor_mode: ResMut<CursorMode>,
    viewport_query: Query<(&UiGlobalTransform, &ComputedNode), With<WorldMapViewport>>,
) {
    let Some(window) = window else {
        return;
    };

    let cursor_pos = window.cursor_position();

    // 1. Center on player on Spacebar
    if keyboard.just_pressed(KeyCode::Space)
        && let Ok(player_transform) = player_query.single()
    {
        let p = player_transform.translation;
        map_state.center = Vec2::new(p.x / VOXEL_SIZE, p.z / VOXEL_SIZE);
    }

    // 2. Pan with Arrow keys or WASD
    let mut pan_dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        pan_dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        pan_dir.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        pan_dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        pan_dir.y += 1.0;
    }
    let zoom = map_state.zoom;
    if pan_dir != Vec2::ZERO {
        map_state.center += pan_dir * (4.0 / zoom);
    }

    // 3. Mouse Zoom
    let scroll_y = scroll.delta.y;
    if scroll_y.abs() > 0.001 {
        let zoom_factor = if scroll_y > 0.0 { 1.15 } else { 1.0 / 1.15 };
        let new_zoom = (map_state.zoom * zoom_factor).clamp(0.20, 4.0);
        map_state.zoom = new_zoom;
    }

    // 4. Mouse Left Click Drag Pan
    if mouse_button.pressed(MouseButton::Left) {
        if let (Some(curr), Some(prev)) = (cursor_pos, map_state.last_cursor_screen_pos) {
            let (in_vp, vp_scale_y) = if let Ok((vp_transform, computed)) = viewport_query.single() {
                let logical_size = computed.size() * computed.inverse_scale_factor();
                let vp_center = vp_transform.translation * computed.inverse_scale_factor();
                let vp_pos = vp_center - logical_size * 0.5;
                let in_x = curr.x - vp_pos.x;
                let in_y = curr.y - vp_pos.y;
                let is_inside = in_x >= 0.0
                    && in_x <= logical_size.x
                    && in_y >= 0.0
                    && in_y <= logical_size.y;
                let sy = logical_size.y / (WORLD_MAP_HEIGHT as f32);
                let safe_sy = if sy.is_finite() && sy > 0.01 { sy } else { 1.0 };
                (is_inside, safe_sy)
            } else {
                (curr.y >= 80.0, 1.0)
            };

            if in_vp || map_state.is_dragging {
                let screen_delta = curr - prev;
                let current_zoom = if map_state.zoom.is_finite() && map_state.zoom > 0.05 {
                    map_state.zoom
                } else {
                    1.0
                };
                let denom = vp_scale_y * current_zoom;
                if denom.abs() > 0.001 {
                    let world_delta = screen_delta / denom;
                    if world_delta.x.is_finite() && world_delta.y.is_finite() {
                        map_state.center -= world_delta;
                    }
                }
                map_state.is_dragging = true;
                *cursor_mode = CursorMode::Grabbing;
            }
        }
        if !map_state.center.x.is_finite() || !map_state.center.y.is_finite() {
            map_state.center = Vec2::ZERO;
        }
        map_state.last_cursor_screen_pos = cursor_pos;
    } else {
        map_state.is_dragging = false;
        map_state.last_cursor_screen_pos = cursor_pos;
    }
}

fn sync_world_map_terrain(
    map_cache: Res<MapCache>,
    mut map_state: ResMut<WorldMapState>,
    mut images: ResMut<Assets<Image>>,
    viewport_query: Query<&ComputedNode, With<WorldMapViewport>>,
) {
    let tex_aspect = (WORLD_MAP_WIDTH as f32) / (WORLD_MAP_HEIGHT as f32);
    let vp_aspect = viewport_query
        .single()
        .ok()
        .map(|c| c.size())
        .filter(|s| s.x > 1.0 && s.y > 1.0)
        .map_or(tex_aspect, |s| s.x / s.y);

    let center_dist = (map_state.center - map_state.last_drawn_center).length();
    let zoom_diff = (map_state.zoom - map_state.last_drawn_zoom).abs();
    let aspect_diff = (vp_aspect - map_state.last_drawn_aspect).abs();
    let cache_changed = map_cache.version != map_state.last_cache_version;

    // Redraw when moved, zoomed, viewport resized, or cache updated
    if center_dist < 0.25 && zoom_diff < 0.005 && aspect_diff < 0.005 && !cache_changed {
        return;
    }

    map_state.last_drawn_center = map_state.center;
    map_state.last_drawn_zoom = map_state.zoom;
    map_state.last_drawn_aspect = vp_aspect;
    map_state.last_cache_version = map_cache.version;

    let Some(mut image) = images.get_mut(&map_state.terrain_image) else {
        return;
    };
    let Some(ref mut data) = image.data else {
        return;
    };

    let half_w = (WORLD_MAP_WIDTH as f32) / 2.0;
    let half_h = (WORLD_MAP_HEIGHT as f32) / 2.0;
    let zoom = if map_state.zoom.is_finite() && map_state.zoom > 0.05 {
        map_state.zoom
    } else {
        1.0
    };
    let aspect_scale_x = (vp_aspect / tex_aspect).clamp(0.25, 4.0);
    let zoom_x = zoom / aspect_scale_x;
    let center = if map_state.center.is_finite() {
        map_state.center
    } else {
        Vec2::ZERO
    };

    for py in 0..WORLD_MAP_HEIGHT {
        let raw_z = center.y + (py as f32 - half_h) / zoom;
        let wz = if raw_z.is_finite() {
            raw_z.clamp(-2_000_000.0, 2_000_000.0).floor() as i32
        } else {
            0
        };
        let row_offset = (py as usize) * (WORLD_MAP_WIDTH as usize) * 4;

        let col_z = wz >> 4;
        let lz = (wz & 15) as usize;

        let north_wz = wz.saturating_sub(1);
        let north_col_z = north_wz >> 4;
        let north_lz = (north_wz & 15) as usize;

        let mut last_col_x = i32::MIN;
        let mut current_chunk: Option<&crate::map::cache::MapChunk> = None;
        let mut north_chunk: Option<&crate::map::cache::MapChunk> = None;

        for px in 0..WORLD_MAP_WIDTH {
            let raw_x = center.x + (px as f32 - half_w) / zoom_x;
            let wx = if raw_x.is_finite() {
                raw_x.clamp(-2_000_000.0, 2_000_000.0).floor() as i32
            } else {
                0
            };
            let idx = row_offset + (px as usize) * 4;

            let col_x = wx >> 4;
            let lx = (wx & 15) as usize;

            if col_x != last_col_x {
                last_col_x = col_x;
                current_chunk = map_cache.get_chunk(IVec2::new(col_x, col_z));
                north_chunk = if north_col_z == col_z {
                    current_chunk
                } else {
                    map_cache.get_chunk(IVec2::new(col_x, north_col_z))
                };
            }

            let color = if let Some(chunk) = current_chunk {
                let pixel = chunk.get(lx, lz);
                if pixel.voxel != Voxel::Air {
                    let north_h = north_chunk.map(|nc| nc.get(lx, north_lz).height);
                    apply_relief_shading(pixel.color, pixel.height, north_h)
                } else {
                    unexplored_color(wx, wz)
                }
            } else {
                unexplored_color(wx, wz)
            };

            data[idx] = color[0];
            data[idx + 1] = color[1];
            data[idx + 2] = color[2];
            data[idx + 3] = color[3];
        }
    }
}

fn update_world_map_ui(
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    map_state: Res<WorldMapState>,
    viewport_query: Query<(&ComputedNode, &UiGlobalTransform), With<WorldMapViewport>>,
    loc: Option<Res<crate::core::Localization>>,
    mut marker_query: Query<
        (&mut Node, &mut UiTransform, &mut Visibility),
        With<WorldMapPlayerMarker>,
    >,
    mut player_text_query: Query<&mut Text, (With<WorldMapInfoText>, Without<WorldMapCursorText>)>,
    mut cursor_text_query: Query<&mut Text, (With<WorldMapCursorText>, Without<WorldMapInfoText>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let p = player_transform.translation;
    let px_block = (p.x / VOXEL_SIZE).floor() as i32;
    let py_block = (p.y / VOXEL_SIZE).floor() as i32;
    let pz_block = (p.z / VOXEL_SIZE).floor() as i32;

    // 1. Update Player position readout
    for mut text in &mut player_text_query {
        if let Some(ref l) = loc {
            **text = l.t_fmt(
                "map.player_pos",
                &[&px_block.to_string(), &py_block.to_string(), &pz_block.to_string()],
            );
        } else {
            **text = format!("Player: X: {} Y: {} Z: {}", px_block, py_block, pz_block);
        }
    }

    // 2. Position the Player Marker relative to the interactive viewport (in logical pixels)
    let half_w = (WORLD_MAP_WIDTH as f32) / 2.0;
    let half_h = (WORLD_MAP_HEIGHT as f32) / 2.0;
    let tex_aspect = (WORLD_MAP_WIDTH as f32) / (WORLD_MAP_HEIGHT as f32);
    let safe_zoom = if map_state.zoom.is_finite() && map_state.zoom > 0.05 {
        map_state.zoom
    } else {
        1.0
    };
    let center = if map_state.center.is_finite() {
        map_state.center
    } else {
        Vec2::ZERO
    };

    let (vp_w, vp_h) = if let Ok((vp_node, _)) = viewport_query.single() {
        let logical = vp_node.size() * vp_node.inverse_scale_factor();
        if logical.x > 1.0 && logical.y > 1.0 {
            (logical.x, logical.y)
        } else {
            (WORLD_MAP_VIEWPORT_WIDTH, WORLD_MAP_VIEWPORT_HEIGHT)
        }
    } else {
        (WORLD_MAP_VIEWPORT_WIDTH, WORLD_MAP_VIEWPORT_HEIGHT)
    };

    let vp_aspect = vp_w / vp_h;
    let aspect_scale_x = (vp_aspect / tex_aspect).clamp(0.25, 4.0);
    let zoom_x = safe_zoom / aspect_scale_x;

    let player_map_x = half_w + (p.x / VOXEL_SIZE - center.x) * zoom_x;
    let player_map_y = half_h + (p.z / VOXEL_SIZE - center.y) * safe_zoom;

    let u_pct = ((player_map_x / (WORLD_MAP_WIDTH as f32)) * 100.0).clamp(-20.0, 120.0);
    let v_pct = ((player_map_y / (WORLD_MAP_HEIGHT as f32)) * 100.0).clamp(-20.0, 120.0);

    let marker_visible = player_map_x >= 0.0
        && player_map_x <= WORLD_MAP_WIDTH as f32
        && player_map_y >= 0.0
        && player_map_y <= WORLD_MAP_HEIGHT as f32;

    let yaw = if let Ok(camera_transform) = camera_query.single() {
        let (y, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        if y.is_finite() { y } else { 0.0 }
    } else {
        0.0
    };

    for (mut node, mut ui_transform, mut vis) in &mut marker_query {
        node.left = percent(u_pct);
        node.top = percent(v_pct);
        ui_transform.translation = Val2::ZERO;
        ui_transform.scale = Vec2::ONE;
        ui_transform.rotation = Rot2::radians(-yaw);
        *vis = if marker_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // 3. Update Cursor coordinates under mouse pointer (in logical pixels)
    if let (Some(window), Ok((vp_node, vp_transform))) = (window, viewport_query.single()) {
        let inv_scale = vp_node.inverse_scale_factor();
        let vp_size = vp_node.size() * inv_scale;
        if vp_size.x > 0.0 && vp_size.y > 0.0 {
            let vp_pos = vp_transform.translation * inv_scale - vp_size / 2.0;
            if let Some(cursor) = window.cursor_position() {
                let in_vp_x = cursor.x - vp_pos.x;
                let in_vp_y = cursor.y - vp_pos.y;

                if in_vp_x >= 0.0 && in_vp_x <= vp_size.x && in_vp_y >= 0.0 && in_vp_y <= vp_size.y
                {
                    let scale_x = vp_size.x / (WORLD_MAP_WIDTH as f32);
                    let scale_y = vp_size.y / (WORLD_MAP_HEIGHT as f32);
                    let tex_x = in_vp_x / scale_x;
                    let tex_y = in_vp_y / scale_y;

                    let raw_cx = center.x + (tex_x - half_w) / zoom_x;
                    let raw_cz = center.y + (tex_y - half_h) / safe_zoom;
                    let cursor_world_x = if raw_cx.is_finite() {
                        raw_cx.clamp(-2_000_000.0, 2_000_000.0).floor() as i32
                    } else {
                        0
                    };
                    let cursor_world_z = if raw_cz.is_finite() {
                        raw_cz.clamp(-2_000_000.0, 2_000_000.0).floor() as i32
                    } else {
                        0
                    };

                    for mut text in &mut cursor_text_query {
                        if let Some(ref l) = loc {
                            **text = l.t_fmt(
                                "map.cursor_pos",
                                &[
                                    &cursor_world_x.to_string(),
                                    &cursor_world_z.to_string(),
                                    &format!("{:.2}", safe_zoom),
                                ],
                            );
                        } else {
                            **text = format!(
                                "Cursor: X: {} Z: {} | Zoom: {:.2}x",
                                cursor_world_x, cursor_world_z, safe_zoom
                            );
                        }
                    }
                    return;
                }
            }
        }
    }

    // Default cursor text when outside viewport
    for mut text in &mut cursor_text_query {
        if let Some(ref l) = loc {
            **text = l.t_fmt("map.zoom", &[&format!("{:.2}", safe_zoom)]);
        } else {
            **text = format!("Zoom: {:.2}x", safe_zoom);
        }
    }
}

