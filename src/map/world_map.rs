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
    player::Player,
    world::{VOXEL_SIZE, Voxel},
};

use super::{
    cache::MapCache,
    color::{apply_relief_shading, unexplored_color},
};

pub const WORLD_MAP_WIDTH: u32 = 640;
pub const WORLD_MAP_HEIGHT: u32 = 360;
pub const WORLD_MARKER_SIZE: f32 = 24.0;

#[derive(Component)]
pub struct WorldMapRoot;

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
    pub last_cache_version: u64,
    #[allow(dead_code)]
    pub last_marker_yaw: f32,
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

    let marker_handle: Handle<Image> = asset_server.load("textures/gui/atlases/marker_red.png");

    commands.insert_resource(WorldMapState {
        center: Vec2::ZERO,
        zoom: 1.0,
        is_dragging: false,
        last_cursor_screen_pos: None,
        terrain_image: terrain_handle,
        marker_image: marker_handle,
        last_drawn_center: Vec2::splat(f32::MAX),
        last_drawn_zoom: -1.0,
        last_cache_version: u64::MAX,
        last_marker_yaw: f32::MAX,
    });
}

fn spawn_world_map_ui(
    mut commands: Commands,
    mut map_state: ResMut<WorldMapState>,
    player_query: Query<&Transform, With<Player>>,
    app_font: Option<Res<crate::core::AppFont>>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source()).unwrap_or_default();

    // When opening the world map, center immediately on the player's position
    if let Ok(player_transform) = player_query.single() {
        let p = player_transform.translation;
        map_state.center = Vec2::new(p.x / VOXEL_SIZE, p.z / VOXEL_SIZE);
    }
    map_state.last_drawn_center = Vec2::splat(f32::MAX); // Force redraw

    commands
        .spawn((
            WorldMapRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.96)),
            ZIndex(250),
        ))
        .with_children(|root| {
            // Header Bar
            root.spawn((
                Node {
                    width: percent(100.0),
                    height: px(44.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(px(20.0), px(8.0)),
                    border: UiRect::bottom(px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.09, 0.12, 0.95)),
                BorderColor::all(Color::srgba(0.25, 0.28, 0.35, 0.80)),
            ))
            .with_children(|header| {
                header.spawn((
                    Text::new("WORLD MAP"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.98)),
                    crate::core::text_shadow_default(),
                ));

                header.spawn((
                    Text::new("[M / ESC] Return to Game"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.65, 0.68, 0.75)),
                    crate::core::text_shadow_default(),
                ));
            });

            // Center Interactive Map Canvas Area
            root.spawn(Node {
                width: percent(100.0),
                height: percent(84.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|canvas_container| {
                canvas_container
                    .spawn((
                        WorldMapViewport,
                        Node {
                            position_type: PositionType::Relative,
                            height: percent(96.0),
                            aspect_ratio: Some(16.0 / 9.0),
                            border: UiRect::all(px(2.0)),
                            border_radius: BorderRadius::all(px(4.0)),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.35, 0.38, 0.48, 0.85)),
                    ))
                    .with_children(|viewport| {
                        // Terrain Render Texture (fills viewport 100%)
                        viewport.spawn((
                            ImageNode {
                                image: map_state.terrain_image.clone(),
                                ..default()
                            },
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(0.0),
                                top: px(0.0),
                                right: px(0.0),
                                bottom: px(0.0),
                                ..default()
                            },
                            ZIndex(1),
                        ));

                        // Player Directional Red Marker
                        let half_m = WORLD_MARKER_SIZE / 2.0;
                        viewport.spawn((
                            WorldMapPlayerMarker,
                            ImageNode {
                                image: map_state.marker_image.clone(),
                                ..default()
                            },
                            Node {
                                position_type: PositionType::Absolute,
                                width: px(WORLD_MARKER_SIZE),
                                height: px(WORLD_MARKER_SIZE),
                                left: percent(50.0),
                                top: percent(50.0),
                                ..default()
                            },
                            UiTransform::from_translation(Val2::new(px(-half_m), px(-half_m))),
                            Visibility::Visible,
                            ZIndex(30),
                        ));
                    });
            });

            // Footer HUD Bar
            root.spawn((
                Node {
                    width: percent(100.0),
                    height: px(40.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(px(20.0), px(6.0)),
                    border: UiRect::top(px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.09, 0.12, 0.95)),
                BorderColor::all(Color::srgba(0.25, 0.28, 0.35, 0.80)),
            ))
            .with_children(|footer| {
                // Controls hint
                footer.spawn((
                    Text::new("LMB Drag: Pan   |   Scroll: Zoom   |   Space: Center on Player"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.60, 0.64, 0.72)),
                    crate::core::text_shadow_default(),
                ));

                // Player coords readout
                footer.spawn((
                    WorldMapInfoText,
                    Text::new("Player: X: 0 Y: 0 Z: 0"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.90, 0.92, 0.96)),
                    crate::core::text_shadow_default(),
                ));

                // Cursor & Zoom readout
                footer.spawn((
                    WorldMapCursorText,
                    Text::new("Cursor: X: 0 Z: 0 | Zoom: 1.00x"),
                    TextFont {
                        font: font_handle,
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.82, 0.90)),
                    crate::core::text_shadow_default(),
                ));
            });
        });
}

fn despawn_world_map_ui(mut commands: Commands, query: Query<Entity, With<WorldMapRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_world_map_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    player_query: Query<&Transform, With<Player>>,
    mut map_state: ResMut<WorldMapState>,
    mut cursor_mode: ResMut<CursorMode>,
    viewport_query: Query<&ComputedNode, With<WorldMapViewport>>,
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
            let screen_delta = curr - prev;
            // Get viewport computed size to scale drag accurately
            let vp_scale = if let Ok(computed) = viewport_query.single() {
                computed.size().x / (WORLD_MAP_WIDTH as f32)
            } else {
                1.0
            };
            let current_zoom = map_state.zoom;
            let world_delta = screen_delta / (vp_scale * current_zoom);
            map_state.center -= world_delta;
        }
        map_state.is_dragging = true;
        map_state.last_cursor_screen_pos = cursor_pos;
        *cursor_mode = CursorMode::Grabbing;
    } else {
        map_state.is_dragging = false;
        map_state.last_cursor_screen_pos = cursor_pos;
    }
}

fn sync_world_map_terrain(
    map_cache: Res<MapCache>,
    mut map_state: ResMut<WorldMapState>,
    mut images: ResMut<Assets<Image>>,
) {
    let center_dist = (map_state.center - map_state.last_drawn_center).length();
    let zoom_diff = (map_state.zoom - map_state.last_drawn_zoom).abs();
    let cache_changed = map_cache.version != map_state.last_cache_version;

    // Redraw when moved by more than 0.25 blocks, or zoom changed, or cache updated
    if center_dist < 0.25 && zoom_diff < 0.005 && !cache_changed {
        return;
    }

    map_state.last_drawn_center = map_state.center;
    map_state.last_drawn_zoom = map_state.zoom;
    map_state.last_cache_version = map_cache.version;

    let Some(mut image) = images.get_mut(&map_state.terrain_image) else {
        return;
    };
    let Some(ref mut data) = image.data else {
        return;
    };

    let half_w = (WORLD_MAP_WIDTH as f32) / 2.0;
    let half_h = (WORLD_MAP_HEIGHT as f32) / 2.0;
    let zoom = map_state.zoom;
    let center = map_state.center;

    for py in 0..WORLD_MAP_HEIGHT {
        let wz = (center.y + (py as f32 - half_h) / zoom).floor() as i32;
        let row_offset = (py as usize) * (WORLD_MAP_WIDTH as usize) * 4;

        for px in 0..WORLD_MAP_WIDTH {
            let wx = (center.x + (px as f32 - half_w) / zoom).floor() as i32;
            let idx = row_offset + (px as usize) * 4;

            let color = if let Some(pixel) = map_cache.get_pixel(wx, wz) {
                if pixel.voxel != Voxel::Air {
                    let north_h = map_cache.get_pixel(wx, wz - 1).map(|p| p.height);
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

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_world_map_ui(
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    map_state: Res<WorldMapState>,
    viewport_query: Query<(&ComputedNode, &GlobalTransform), With<WorldMapViewport>>,
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
        **text = format!("Player: X: {} Y: {} Z: {}", px_block, py_block, pz_block);
    }

    // 2. Position the Player Marker relative to the interactive viewport
    let half_w = (WORLD_MAP_WIDTH as f32) / 2.0;
    let half_h = (WORLD_MAP_HEIGHT as f32) / 2.0;
    let player_map_x = half_w + (p.x / VOXEL_SIZE - map_state.center.x) * map_state.zoom;
    let player_map_y = half_h + (p.z / VOXEL_SIZE - map_state.center.y) * map_state.zoom;

    let pct_x = (player_map_x / WORLD_MAP_WIDTH as f32) * 100.0;
    let pct_y = (player_map_y / WORLD_MAP_HEIGHT as f32) * 100.0;

    let marker_visible = player_map_x >= 0.0
        && player_map_x <= WORLD_MAP_WIDTH as f32
        && player_map_y >= 0.0
        && player_map_y <= WORLD_MAP_HEIGHT as f32;

    let yaw = if let Ok(camera_transform) = camera_query.single() {
        let (y, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        y
    } else {
        0.0
    };

    let half_m = WORLD_MARKER_SIZE / 2.0;
    for (mut node, mut ui_transform, mut vis) in &mut marker_query {
        node.left = percent(pct_x);
        node.top = percent(pct_y);
        ui_transform.translation = Val2::new(px(-half_m), px(-half_m));
        ui_transform.rotation = Rot2::radians(-yaw);
        *vis = if marker_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // 3. Update Cursor coordinates under mouse pointer
    if let (Some(window), Ok((vp_node, vp_transform))) = (window, viewport_query.single()) {
        let vp_size = vp_node.size();
        if vp_size.x > 0.0 && vp_size.y > 0.0 {
            let vp_pos = vp_transform.translation().truncate() - vp_size / 2.0;
            if let Some(cursor) = window.cursor_position() {
                let in_vp_x = cursor.x - vp_pos.x;
                let in_vp_y = cursor.y - vp_pos.y;

                if in_vp_x >= 0.0 && in_vp_x <= vp_size.x && in_vp_y >= 0.0 && in_vp_y <= vp_size.y {
                    let scale_x = vp_size.x / (WORLD_MAP_WIDTH as f32);
                    let scale_y = vp_size.y / (WORLD_MAP_HEIGHT as f32);
                    let tex_x = in_vp_x / scale_x;
                    let tex_y = in_vp_y / scale_y;

                    let cursor_world_x =
                        (map_state.center.x + (tex_x - half_w) / map_state.zoom).floor() as i32;
                    let cursor_world_z =
                        (map_state.center.y + (tex_y - half_h) / map_state.zoom).floor() as i32;

                    for mut text in &mut cursor_text_query {
                        **text = format!(
                            "Cursor: X: {} Z: {} | Zoom: {:.2}x",
                            cursor_world_x, cursor_world_z, map_state.zoom
                        );
                    }
                    return;
                }
            }
        }
    }

    // Default cursor text when outside viewport
    for mut text in &mut cursor_text_query {
        **text = format!("Zoom: {:.2}x", map_state.zoom);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_map_marker_centered_percent() {
        let center = Vec2::new(100.0, -50.0);
        let player_world = Vec2::new(100.0, -50.0);
        let zoom = 1.0;

        let half_w = (WORLD_MAP_WIDTH as f32) / 2.0;
        let half_h = (WORLD_MAP_HEIGHT as f32) / 2.0;
        let player_map_x = half_w + (player_world.x - center.x) * zoom;
        let player_map_y = half_h + (player_world.y - center.y) * zoom;

        let pct_x = (player_map_x / WORLD_MAP_WIDTH as f32) * 100.0;
        let pct_y = (player_map_y / WORLD_MAP_HEIGHT as f32) * 100.0;

        assert!((pct_x - 50.0).abs() < 1e-4);
        assert!((pct_y - 50.0).abs() < 1e-4);
    }

    #[test]
    fn world_map_marker_aspect_ratio_16_9() {
        let ratio = (WORLD_MAP_WIDTH as f32) / (WORLD_MAP_HEIGHT as f32);
        assert!((ratio - (16.0 / 9.0)).abs() < 1e-4);
    }
}
