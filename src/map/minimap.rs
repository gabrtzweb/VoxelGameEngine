use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    text::FontSize,
};

use crate::{
    generation::TerrainGenerator,
    menu::MenuState,
    player::Player,
    world::{VOXEL_SIZE, Voxel},
};

use super::{
    cache::MapCache,
    color::{apply_relief_shading, unexplored_color, voxel_map_color_at},
};

pub const MINIMAP_SIZE: u32 = 192;
pub const MARKER_SIZE: u32 = 24;

#[derive(Component)]
pub struct MinimapRoot;

#[derive(Component)]
pub struct MinimapCoordsText;

#[derive(Component)]
pub struct MinimapBiomeText;

#[derive(Component)]
pub struct MinimapDisplayImage;

#[derive(Component)]
pub struct MinimapPlayerMarker;

#[derive(Resource)]
pub struct MinimapState {
    pub terrain_image: Handle<Image>,
    pub marker_image: Handle<Image>,
    pub last_player_block: IVec2,
    pub last_cache_version: u64,
    pub last_marker_yaw: f32,
}

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_minimap).add_systems(
            Update,
            (
                sync_minimap_terrain,
                sync_minimap_marker,
                update_minimap_ui,
                manage_minimap_visibility,
            ),
        );
    }
}

fn setup_minimap(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // 1. Create dynamic 192x192 terrain map texture
    let terrain_pixels = [18u8, 19u8, 24u8, 255u8].repeat((MINIMAP_SIZE * MINIMAP_SIZE) as usize);
    let mut terrain_img = Image::new_fill(
        Extent3d {
            width: MINIMAP_SIZE,
            height: MINIMAP_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &terrain_pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    terrain_img.sampler = ImageSampler::nearest();
    let terrain_handle = images.add(terrain_img);

    // 2. Create dynamic 24x24 player marker arrow texture
    let mut marker_pixels = vec![0u8; (MARKER_SIZE * MARKER_SIZE * 4) as usize];
    draw_player_arrow(&mut marker_pixels, 0.0);
    let mut marker_img = Image::new_fill(
        Extent3d {
            width: MARKER_SIZE,
            height: MARKER_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &marker_pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    marker_img.sampler = ImageSampler::nearest();
    let marker_handle = images.add(marker_img);

    commands.insert_resource(MinimapState {
        terrain_image: terrain_handle.clone(),
        marker_image: marker_handle.clone(),
        last_player_block: IVec2::new(i32::MAX, i32::MAX),
        last_cache_version: u64::MAX,
        last_marker_yaw: f32::MAX,
    });

    // 3. Spawn Minimap Square HUD in top-right corner
    commands
        .spawn((
            MinimapRoot,
            Node {
                position_type: PositionType::Absolute,
                top: px(14.0),
                right: px(14.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6.0),
                ..default()
            },
            ZIndex(150),
        ))
        .with_children(|root| {
            // Square Frame container
            root.spawn((
                Node {
                    position_type: PositionType::Relative,
                    width: px(MINIMAP_SIZE as f32 + 8.0),
                    height: px(MINIMAP_SIZE as f32 + 8.0),
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(3.0)),
                    border_radius: BorderRadius::all(px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.07, 0.09, 0.95)),
                BorderColor::all(Color::srgba(0.32, 0.35, 0.44, 0.95)),
            ))
            .with_children(|frame| {
                // North Cardinal Indicator
                frame.spawn((
                    Text::new("N"),
                    TextFont {
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.40, 0.40)), // Classic red North marker
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(2.0),
                        ..default()
                    },
                    ZIndex(10),
                ));

                // South Cardinal Indicator
                frame.spawn((
                    Text::new("S"),
                    TextFont {
                        font_size: FontSize::Px(10.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.78, 0.85)),
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: px(2.0),
                        ..default()
                    },
                    ZIndex(10),
                ));

                // West Cardinal Indicator
                frame.spawn((
                    Text::new("W"),
                    TextFont {
                        font_size: FontSize::Px(10.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.78, 0.85)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(3.0),
                        ..default()
                    },
                    ZIndex(10),
                ));

                // East Cardinal Indicator
                frame.spawn((
                    Text::new("E"),
                    TextFont {
                        font_size: FontSize::Px(10.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.78, 0.85)),
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(3.0),
                        ..default()
                    },
                    ZIndex(10),
                ));

                // Minimap Terrain Viewport Image
                frame.spawn((
                    MinimapDisplayImage,
                    ImageNode {
                        image: terrain_handle,
                        ..default()
                    },
                    Node {
                        width: px(MINIMAP_SIZE as f32),
                        height: px(MINIMAP_SIZE as f32),
                        border_radius: BorderRadius::all(px(3.0)),
                        ..default()
                    },
                ));

                // Center Directional Player Marker Arrow
                let marker_offset = (MINIMAP_SIZE as f32 - MARKER_SIZE as f32) / 2.0;
                frame.spawn((
                    MinimapPlayerMarker,
                    ImageNode {
                        image: marker_handle,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(marker_offset + 4.0),
                        top: px(marker_offset + 4.0),
                        width: px(MARKER_SIZE as f32),
                        height: px(MARKER_SIZE as f32),
                        ..default()
                    },
                    ZIndex(20),
                ));
            });

            // Information Readout Footer Pill (Coordinates & Biome)
            root.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: px(2.0),
                    padding: UiRect::axes(px(12.0), px(4.0)),
                    border: UiRect::all(px(1.0)),
                    border_radius: BorderRadius::all(px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.07, 0.09, 0.88)),
                BorderColor::all(Color::srgba(0.28, 0.30, 0.36, 0.75)),
            ))
            .with_children(|pill| {
                pill.spawn((
                    MinimapCoordsText,
                    Text::new("X: 0  Y: 0  Z: 0"),
                    TextFont {
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.88, 0.90, 0.94)),
                ));
                pill.spawn((
                    MinimapBiomeText,
                    Text::new("Plains"),
                    TextFont {
                        font_size: FontSize::Px(10.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.70, 0.84, 0.65)),
                ));
            });
        });
}

fn sync_minimap_terrain(
    player_query: Query<&Transform, With<Player>>,
    map_cache: Res<MapCache>,
    mut minimap_state: ResMut<MinimapState>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let p_pos = player_transform.translation;
    let player_block = IVec2::new(
        (p_pos.x / VOXEL_SIZE).floor() as i32,
        (p_pos.z / VOXEL_SIZE).floor() as i32,
    );

    let needs_update = player_block != minimap_state.last_player_block
        || map_cache.version != minimap_state.last_cache_version;

    if !needs_update {
        return;
    }

    minimap_state.last_player_block = player_block;
    minimap_state.last_cache_version = map_cache.version;

    let Some(mut image) = images.get_mut(&minimap_state.terrain_image) else {
        return;
    };
    let Some(ref mut data) = image.data else {
        return;
    };

    let half = (MINIMAP_SIZE / 2) as i32;

    for py in 0..MINIMAP_SIZE as i32 {
        let wz = player_block.y + (py - half);
        let row_offset = (py as usize) * (MINIMAP_SIZE as usize) * 4;

        for px in 0..MINIMAP_SIZE as i32 {
            let wx = player_block.x + (px - half);
            let idx = row_offset + (px as usize) * 4;

            let color = if let Some(pixel) = map_cache.get_pixel(wx, wz) {
                if pixel.voxel != Voxel::Air {
                    let base = voxel_map_color_at(pixel.voxel, pixel.water_depth, wx, wz);
                    let north_h = map_cache.get_pixel(wx, wz - 1).map(|p| p.height);
                    apply_relief_shading(base, pixel.height, north_h)
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

fn sync_minimap_marker(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut minimap_state: ResMut<MinimapState>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let (yaw, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    if (yaw - minimap_state.last_marker_yaw).abs() < 0.015 {
        return;
    }

    minimap_state.last_marker_yaw = yaw;

    let Some(mut marker_img) = images.get_mut(&minimap_state.marker_image) else {
        return;
    };

    if let Some(ref mut data) = marker_img.data {
        draw_player_arrow(data, yaw);
    }
}

fn update_minimap_ui(
    player_query: Query<&Transform, With<Player>>,
    mut coords_query: Query<&mut Text, (With<MinimapCoordsText>, Without<MinimapBiomeText>)>,
    mut biome_query: Query<&mut Text, (With<MinimapBiomeText>, Without<MinimapCoordsText>)>,
    terrain_generator: Option<Res<TerrainGenerator>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let p = player_transform.translation;
    let vx = (p.x / VOXEL_SIZE).floor() as i32;
    let vy = (p.y / VOXEL_SIZE).floor() as i32;
    let vz = (p.z / VOXEL_SIZE).floor() as i32;

    for mut text in &mut coords_query {
        **text = format!("X: {:>4}  Y: {:>3}  Z: {:>4}", vx, vy, vz);
    }

    let biome_name = if let Some(ref generator) = terrain_generator {
        generator.sample_column(vx, vz).biome.name()
    } else {
        "Plains"
    };

    for mut text in &mut biome_query {
        **text = biome_name.to_string();
    }
}

fn manage_minimap_visibility(
    menu_state: Option<Res<State<MenuState>>>,
    mut root_query: Query<&mut Visibility, With<MinimapRoot>>,
) {
    let Some(menu_state) = menu_state else {
        return;
    };

    let hide = matches!(
        *menu_state.get(),
        MenuState::WorldMap | MenuState::Pause | MenuState::Settings
    );

    let target_vis = if hide {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };

    for mut vis in &mut root_query {
        if *vis != target_vis {
            *vis = target_vis;
        }
    }
}

/// Renders a sharp, anti-aliased red directional chevron arrow into a 24x24 RGBA buffer.
pub fn draw_player_arrow(canvas: &mut [u8], yaw: f32) {
    canvas.fill(0);

    let size = MARKER_SIZE as f32;
    let cx = size / 2.0;
    let cy = size / 2.0;

    // Camera forward vector in 2D top-down map:
    // yaw = 0 faces -Z (North), pointing UP on map (dy < 0)
    let ux = -yaw.sin();
    let uy = -yaw.cos();
    let vx = -uy;
    let vy = ux;

    // Geometry of directional chevron
    let tip = Vec2::new(cx + 8.5 * ux, cy + 8.5 * uy);
    let left_wing = Vec2::new(cx - 6.5 * ux - 5.5 * vx, cy - 6.5 * uy - 5.5 * vy);
    let right_wing = Vec2::new(cx - 6.5 * ux + 5.5 * vx, cy - 6.5 * uy + 5.5 * vy);
    let notch = Vec2::new(cx - 2.5 * ux, cy - 2.5 * uy);

    for y in 0..MARKER_SIZE {
        for x in 0..MARKER_SIZE {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let in_left = point_in_triangle(p, tip, left_wing, notch);
            let in_right = point_in_triangle(p, tip, notch, right_wing);

            let idx = ((y * MARKER_SIZE + x) * 4) as usize;

            if in_left || in_right {
                // Vibrant red core
                canvas[idx] = 235;
                canvas[idx + 1] = 30;
                canvas[idx + 2] = 30;
                canvas[idx + 3] = 255;
            } else {
                // 1px Dark border outline
                let d_left =
                    dist_to_segment(p, tip, left_wing).min(dist_to_segment(p, left_wing, notch));
                let d_right =
                    dist_to_segment(p, tip, right_wing).min(dist_to_segment(p, right_wing, notch));
                let d_min = d_left.min(d_right);

                if d_min <= 1.25 {
                    canvas[idx] = 18;
                    canvas[idx + 1] = 18;
                    canvas[idx + 2] = 22;
                    canvas[idx + 3] = 255;
                }
            }
        }
    }
}

#[inline]
fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

#[inline]
fn sign(p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

#[inline]
fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let len_sq = ab.length_squared();
    if len_sq == 0.0 {
        return ap.length();
    }
    let t = (ap.dot(ab) / len_sq).clamp(0.0, 1.0);
    let projection = a + ab * t;
    (p - projection).length()
}
