use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    math::Rot2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    text::FontSize,
    ui::UiTransform,
};

use crate::{
    generation::TerrainGenerator,
    menu::MenuState,
    player::{Player, PlayerCamera},
    world::{VOXEL_SIZE, Voxel},
};

use super::{
    cache::{MapCache, MapChunk},
    color::{apply_relief_shading, unexplored_color},
};

pub const MINIMAP_SIZE: u32 = 192;
pub const MINIMAP_FRAME_SIZE: f32 = 216.0;
pub const MARKER_SIZE: u32 = 24;

#[derive(Component)]
pub struct MinimapRoot;

#[derive(Component)]
pub struct MinimapCoordsText;

#[derive(Component)]
pub struct MinimapBiomeText;

#[derive(Component)]
pub struct MinimapDateText;

#[derive(Component)]
pub struct MinimapDisplayImage;

#[derive(Component)]
pub struct MinimapPlayerMarker;

#[derive(Resource)]
pub struct MinimapState {
    pub terrain_image: Handle<Image>,
    pub last_player_block: IVec2,
    pub last_cache_version: u64,
    pub last_marker_yaw: f32,
    pub last_update_time: f32,
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

fn setup_minimap(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    app_font: Option<Res<crate::core::AppFont>>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source()).unwrap_or_default();
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

    // 2. Load textures from assets/textures/interfaces/atlases/
    let bg_handle: Handle<Image> = asset_server.load("textures/interfaces/atlases/map_background.png");
    let marker_handle: Handle<Image> = asset_server.load("textures/interfaces/atlases/decorations/marker_red.png");

    commands.insert_resource(MinimapState {
        terrain_image: terrain_handle.clone(),
        last_player_block: IVec2::new(i32::MAX, i32::MAX),
        last_cache_version: u64::MAX,
        last_marker_yaw: f32::MAX,
        last_update_time: 0.0,
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
            // Container with map_background texture extending outward
            root.spawn(Node {
                position_type: PositionType::Relative,
                width: px(MINIMAP_FRAME_SIZE),
                height: px(MINIMAP_FRAME_SIZE),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|frame| {
                // Background parchment image filling the frame
                frame.spawn((
                    ImageNode {
                        image: bg_handle,
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

                // Minimap Terrain Viewport Image (192x192) centered on top of background
                frame.spawn((
                    MinimapDisplayImage,
                    ImageNode {
                        image: terrain_handle,
                        ..default()
                    },
                    Node {
                        width: px(MINIMAP_SIZE as f32),
                        height: px(MINIMAP_SIZE as f32),
                        ..default()
                    },
                    ZIndex(5),
                ));

                // North Cardinal Indicator
                frame.spawn((
                    Text::new("N"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.98)),
                    crate::core::text_shadow_default(),
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(16.0),
                        ..default()
                    },
                    ZIndex(15),
                ));

                // South Cardinal Indicator
                frame.spawn((
                    Text::new("S"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.98)),
                    crate::core::text_shadow_default(),
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: px(16.0),
                        ..default()
                    },
                    ZIndex(15),
                ));

                // West Cardinal Indicator
                frame.spawn((
                    Text::new("W"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.98)),
                    crate::core::text_shadow_default(),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(16.0),
                        ..default()
                    },
                    ZIndex(15),
                ));

                // East Cardinal Indicator
                frame.spawn((
                    Text::new("E"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.98)),
                    crate::core::text_shadow_default(),
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(16.0),
                        ..default()
                    },
                    ZIndex(15),
                ));

                // Center Directional Player Red Marker
                let marker_offset = (MINIMAP_FRAME_SIZE - MARKER_SIZE as f32) / 2.0;
                frame.spawn((
                    MinimapPlayerMarker,
                    ImageNode {
                        image: marker_handle,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(marker_offset),
                        top: px(marker_offset),
                        width: px(MARKER_SIZE as f32),
                        height: px(MARKER_SIZE as f32),
                        ..default()
                    },
                    UiTransform::IDENTITY,
                    ZIndex(25),
                ));
            });

            // Information Readout Footer (Transparent, cleanly spaced)
            root.spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(2.0),
                padding: UiRect::axes(px(8.0), px(2.0)),
                ..default()
            })
            .with_children(|pill| {
                pill.spawn((
                    MinimapCoordsText,
                    Text::new("Coordinates: XYZ: 0, 0, 0"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.98)),
                    crate::core::text_shadow_default(),
                ));
                pill.spawn((
                    MinimapBiomeText,
                    Text::new("Biome: Plains"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: FontSize::Px(10.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.98)),
                    crate::core::text_shadow_default(),
                ));
                pill.spawn((
                    MinimapDateText,
                    Text::new("Day: 1, 06:00, spring"),
                    TextFont {
                        font: font_handle,
                        font_size: FontSize::Px(10.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.98)),
                    crate::core::text_shadow_default(),
                ));
            });
        });
}

fn sync_minimap_terrain(
    player_query: Query<&Transform, With<Player>>,
    map_cache: Res<MapCache>,
    time: Res<Time>,
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

    let cache_changed = map_cache.version != minimap_state.last_cache_version;
    let moved = player_block != minimap_state.last_player_block;

    if !moved && !cache_changed {
        return;
    }

    let now = time.elapsed_secs();
    // Throttle minimap texture generation to at most 20 Hz (every 0.05s) to avoid redundant GPU re-uploads
    if (now - minimap_state.last_update_time) < 0.05 {
        return;
    }

    minimap_state.last_player_block = player_block;
    minimap_state.last_cache_version = map_cache.version;
    minimap_state.last_update_time = now;

    let Some(mut image) = images.get_mut(&minimap_state.terrain_image) else {
        return;
    };
    let Some(ref mut data) = image.data else {
        return;
    };

    let half = (MINIMAP_SIZE / 2) as i32;

    for py in 0..MINIMAP_SIZE as i32 {
        let wz = player_block.y + (py - half);
        let chunk_z = wz.div_euclid(16);
        let lz = wz.rem_euclid(16) as usize;

        let north_wz = wz - 1;
        let north_chunk_z = north_wz.div_euclid(16);
        let north_lz = north_wz.rem_euclid(16) as usize;

        let row_offset = (py as usize) * (MINIMAP_SIZE as usize) * 4;

        let mut cached_chunk_x = i32::MIN;
        let mut cached_chunk: Option<&MapChunk> = None;
        let mut cached_north_chunk: Option<&MapChunk> = None;

        for px in 0..MINIMAP_SIZE as i32 {
            let wx = player_block.x + (px - half);
            let chunk_x = wx.div_euclid(16);
            let lx = wx.rem_euclid(16) as usize;

            if chunk_x != cached_chunk_x {
                cached_chunk_x = chunk_x;
                cached_chunk = map_cache.get_chunk(IVec2::new(chunk_x, chunk_z));
                cached_north_chunk = if chunk_z == north_chunk_z {
                    cached_chunk
                } else {
                    map_cache.get_chunk(IVec2::new(chunk_x, north_chunk_z))
                };
            }

            let idx = row_offset + (px as usize) * 4;

            let color = if let Some(chunk) = cached_chunk {
                let pixel = chunk.get(lx, lz);
                if pixel.voxel != Voxel::Air {
                    let north_h = cached_north_chunk.map(|nc| nc.get(lx, north_lz).height);
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

fn sync_minimap_marker(
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut marker_query: Query<&mut UiTransform, With<MinimapPlayerMarker>>,
    mut minimap_state: ResMut<MinimapState>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let (yaw, _, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    if (yaw - minimap_state.last_marker_yaw).abs() < 0.015 {
        return;
    }

    minimap_state.last_marker_yaw = yaw;

    for mut ui_transform in &mut marker_query {
        ui_transform.rotation = Rot2::radians(-yaw);
    }
}

fn update_minimap_ui(
    player_query: Query<&Transform, With<Player>>,
    mut coords_query: Query<
        &mut Text,
        (
            With<MinimapCoordsText>,
            Without<MinimapBiomeText>,
            Without<MinimapDateText>,
        ),
    >,
    mut biome_query: Query<
        &mut Text,
        (
            With<MinimapBiomeText>,
            Without<MinimapCoordsText>,
            Without<MinimapDateText>,
        ),
    >,
    mut date_query: Query<
        &mut Text,
        (
            With<MinimapDateText>,
            Without<MinimapCoordsText>,
            Without<MinimapBiomeText>,
        ),
    >,
    terrain_generator: Option<Res<TerrainGenerator>>,
    environment: Option<Res<crate::environment::EnvironmentState>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let p = player_transform.translation;
    let vx = (p.x / VOXEL_SIZE).floor() as i32;
    let vy = (p.y / VOXEL_SIZE).floor() as i32;
    let vz = (p.z / VOXEL_SIZE).floor() as i32;

    for mut text in &mut coords_query {
        **text = format!("Coordinates: XYZ: {}, {}, {}", vx, vy, vz);
    }

    let biome_name = if let Some(ref generator) = terrain_generator {
        generator.sample_column(vx, vz).biome.name()
    } else {
        "Plains"
    };

    for mut text in &mut biome_query {
        **text = format!("Biome: {}", biome_name);
    }

    let (day, time_str, season_name) = if let Some(ref env) = environment {
        let hours = (env.time_of_day * 24.0 + 6.0).rem_euclid(24.0);
        let h = hours.floor() as u32;
        let m = ((hours - hours.floor()) * 60.0).floor() as u32;
        (
            env.day_of_season(),
            format!("{h:02}:{m:02}"),
            env.season().name().to_lowercase(),
        )
    } else {
        (1, "06:00".to_string(), "spring".to_string())
    };

    for mut text in &mut date_query {
        **text = format!("Day: {day}, {time_str}, {season_name}");
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
