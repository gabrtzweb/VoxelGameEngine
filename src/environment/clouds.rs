use bevy::{
    asset::RenderAssetUsages,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::Face,
    },
};

use crate::player::PlayerCamera;

/// Active cloud layer count. Set to 1, 2, or 3 to easily adjust the number of visible layers.
pub const ACTIVE_LAYER_COUNT: usize = 2;

#[derive(Clone, Debug)]
pub struct CloudLayerConfig {
    pub altitude: f32,
    pub thickness: f32,
    pub cell_size: f32,
    pub grid_radius: i32,
    pub base_alpha: f32,
    pub wind_speed: Vec2,
    pub texture_offset: IVec2,
}

pub const CLOUD_LAYER_CONFIGS: &[CloudLayerConfig] = &[
    // Layer 0: Lower cumulus layer (slightly faster, closer to terrain)
    CloudLayerConfig {
        altitude: 108.0,
        thickness: 2.0,
        cell_size: 12.0,
        grid_radius: 24,
        base_alpha: 0.58,
        wind_speed: Vec2::new(3.2, 1.0),
        texture_offset: IVec2::new(0, 0),
    },
    // Layer 1: Mid-altitude altocumulus layer (slower drift, larger cell scale, higher up)
    CloudLayerConfig {
        altitude: 156.0,
        thickness: 2.0,
        cell_size: 16.0,
        grid_radius: 22,
        base_alpha: 0.44,
        wind_speed: Vec2::new(2.1, 0.6),
        texture_offset: IVec2::new(128, 64),
    },
    // Layer 2: High cirrus layer (very high, faint, gentle drift)
    CloudLayerConfig {
        altitude: 204.0,
        thickness: 2.5,
        cell_size: 20.0,
        grid_radius: 20,
        base_alpha: 0.32,
        wind_speed: Vec2::new(1.4, 0.4),
        texture_offset: IVec2::new(64, 192),
    },
];

#[derive(Component)]
pub struct CloudLayerVisual {
    pub layer_index: usize,
}

#[derive(Resource)]
pub struct CloudMaterialHandle(pub Handle<StandardMaterial>);

#[derive(Resource)]
pub struct CloudTextureMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<bool>,
}

impl CloudTextureMap {
    #[inline]
    pub fn is_cloud(&self, x: i32, z: i32) -> bool {
        let u = x.rem_euclid(self.width as i32) as usize;
        let v = z.rem_euclid(self.height as i32) as usize;
        self.data[v * self.width as usize + u]
    }
}

pub struct SingleLayerState {
    pub current_cell_center: IVec2,
    pub mesh_handle: Handle<Mesh>,
}

#[derive(Resource)]
pub struct CloudLayersResource {
    pub layers: Vec<SingleLayerState>,
}

pub fn setup_clouds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cloud_map = load_cloud_texture_map();

    let material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: Some(Face::Back),
        double_sided: false,
        fog_enabled: false,
        ..default()
    });

    let mut layer_states = Vec::new();
    let active_count = ACTIVE_LAYER_COUNT.clamp(1, CLOUD_LAYER_CONFIGS.len());
    let active_configs = &CLOUD_LAYER_CONFIGS[..active_count];

    for (index, config) in active_configs.iter().enumerate() {
        let initial_mesh = generate_3d_cloud_mesh(&cloud_map, config, IVec2::ZERO);
        let mesh_handle = meshes.add(initial_mesh);

        commands.spawn((
            CloudLayerVisual { layer_index: index },
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material_handle.clone()),
            Transform::from_xyz(0.0, config.altitude, 0.0),
            Visibility::Visible,
            NotShadowCaster,
            NotShadowReceiver,
        ));

        layer_states.push(SingleLayerState {
            current_cell_center: IVec2::ZERO,
            mesh_handle,
        });
    }

    commands.insert_resource(CloudMaterialHandle(material_handle));
    commands.insert_resource(CloudLayersResource {
        layers: layer_states,
    });
    commands.insert_resource(cloud_map);
}

pub type CloudLayerQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static CloudLayerVisual,
        &'static mut Transform,
        &'static mut Visibility,
    ),
    Without<Camera3d>,
>;

pub type CameraTransformQuery<'w, 's> = Single<
    'w,
    's,
    (&'static Transform, &'static GlobalTransform),
    (With<Camera3d>, With<PlayerCamera>),
>;

#[allow(clippy::too_many_arguments)]
pub fn sync_clouds(
    in_game_time_seconds: f64,
    camera: CameraTransformQuery,
    mut cloud_layers: CloudLayerQuery,
    material_handle: Res<CloudMaterialHandle>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    meshes: &mut Assets<Mesh>,
    cloud_map: &CloudTextureMap,
    layers_res: &mut CloudLayersResource,
    time_of_day: f32,
    is_underwater: bool,
) {
    let camera_translation = camera.0.translation;

    for (layer_visual, mut cloud_transform, mut visibility) in &mut cloud_layers {
        if is_underwater {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;

        let index = layer_visual.layer_index;
        if index >= CLOUD_LAYER_CONFIGS.len() || index >= layers_res.layers.len() {
            continue;
        }
        let config = &CLOUD_LAYER_CONFIGS[index];
        let layer_state = &mut layers_res.layers[index];

        let period_x = (cloud_map.width as f64) * (config.cell_size as f64);
        let period_z = (cloud_map.height as f64) * (config.cell_size as f64);

        let wind_x = (in_game_time_seconds * config.wind_speed.x as f64).rem_euclid(period_x) as f32;
        let wind_z = (in_game_time_seconds * config.wind_speed.y as f64).rem_euclid(period_z) as f32;

        let world_center_x = camera_translation.x - wind_x;
        let world_center_z = camera_translation.z - wind_z;

        let cell_x = (world_center_x / config.cell_size).floor() as i32;
        let cell_z = (world_center_z / config.cell_size).floor() as i32;

        let target_cell_center = IVec2::new(cell_x, cell_z);

        if target_cell_center != layer_state.current_cell_center {
            layer_state.current_cell_center = target_cell_center;
            if let Some(mut mesh) = meshes.get_mut(&layer_state.mesh_handle) {
                *mesh = generate_3d_cloud_mesh(cloud_map, config, target_cell_center);
            }
        }

        let subcell_x = world_center_x - (cell_x as f32 * config.cell_size);
        let subcell_z = world_center_z - (cell_z as f32 * config.cell_size);

        cloud_transform.translation = Vec3::new(
            camera_translation.x - subcell_x,
            config.altitude,
            camera_translation.z - subcell_z,
        );
    }

    // Dynamic atmospheric cloud tinting across the day-night cycle.
    let target_color = sample_cloud_color(time_of_day);
    if let Some(mat) = materials.get(&material_handle.0)
        && mat.base_color != target_color
        && let Some(mut mat_mut) = materials.get_mut(&material_handle.0)
    {
        mat_mut.base_color = target_color;
    }
}

pub fn sample_cloud_color(time_of_day: f32) -> Color {
    // 0.00: Morning / Dawn
    // 0.25: Noon / Midday
    // 0.50: Evening / Sunset
    // 0.75: Night / Midnight
    let t = time_of_day.rem_euclid(1.0);

    let (c1, c2, factor) = if t < 0.25 {
        let f = t / 0.25;
        (
            LinearRgba::new(1.0, 0.88, 0.78, 0.90), // Dawn warm peach
            LinearRgba::new(1.0, 1.0, 1.0, 0.95),   // Midday bright white
            f,
        )
    } else if t < 0.50 {
        let f = (t - 0.25) / 0.25;
        (
            LinearRgba::new(1.0, 1.0, 1.0, 0.95),   // Midday bright white
            LinearRgba::new(1.0, 0.65, 0.52, 0.90), // Sunset vibrant amber/pink
            f,
        )
    } else if t < 0.75 {
        let f = (t - 0.50) / 0.25;
        (
            LinearRgba::new(1.0, 0.65, 0.52, 0.90), // Sunset vibrant amber/pink
            LinearRgba::new(0.20, 0.24, 0.36, 0.75), // Midnight deep navy
            f,
        )
    } else {
        let f = (t - 0.75) / 0.25;
        (
            LinearRgba::new(0.20, 0.24, 0.36, 0.75), // Midnight deep navy
            LinearRgba::new(1.0, 0.88, 0.78, 0.90),  // Dawn warm peach
            f,
        )
    };

    let lerped = LinearRgba::new(
        c1.red + (c2.red - c1.red) * factor,
        c1.green + (c2.green - c1.green) * factor,
        c1.blue + (c2.blue - c1.blue) * factor,
        c1.alpha + (c2.alpha - c1.alpha) * factor,
    );

    Color::LinearRgba(lerped)
}

pub fn generate_3d_cloud_mesh(
    cloud_map: &CloudTextureMap,
    config: &CloudLayerConfig,
    center: IVec2,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let cell_size = config.cell_size;
    let grid_radius = config.grid_radius;
    let fade_inner = cell_size * (grid_radius as f32 * 0.58);
    let fade_outer = cell_size * (grid_radius as f32 * 0.95);
    let fade_outer_sq = fade_outer * fade_outer;
    let fade_range = fade_outer - fade_inner;
    let y0 = 0.0;
    let y1 = config.thickness;
    let base_alpha = config.base_alpha;

    let calc_alpha = |x: f32, z: f32| -> f32 {
        let d = (x * x + z * z).sqrt();
        if d <= fade_inner {
            base_alpha
        } else if d >= fade_outer {
            0.0
        } else {
            let t = (d - fade_inner) / fade_range;
            let smooth = t * t * (3.0 - 2.0 * t);
            (1.0 - smooth) * base_alpha
        }
    };

    let add_quad = |
        p: [[f32; 3]; 4],
        norm: [f32; 3],
        shade: [f32; 3],
        pos_out: &mut Vec<[f32; 3]>,
        norm_out: &mut Vec<[f32; 3]>,
        uv_out: &mut Vec<[f32; 2]>,
        col_out: &mut Vec<[f32; 4]>,
        idx_out: &mut Vec<u32>,
    | {
        let start_idx = pos_out.len() as u32;
        let uv_coords = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        for (i, pt) in p.iter().enumerate() {
            let a = calc_alpha(pt[0], pt[2]);
            pos_out.push(*pt);
            norm_out.push(norm);
            uv_out.push(uv_coords[i]);
            col_out.push([shade[0], shade[1], shade[2], a]);
        }
        idx_out.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    };

    // Shading factors tuned for crisp, bright white clouds
    let bottom_shade = [0.82, 0.83, 0.86]; // Clean bright underside
    let top_shade = [1.00, 1.00, 1.00];    // Full sunlight top
    let west_east_shade = [0.90, 0.91, 0.93]; // Bright vertical walls
    let north_south_shade = [0.87, 0.88, 0.90]; // Crisp side shading

    for cz in -grid_radius..=grid_radius {
        let wz = cz as f32 * cell_size;
        let z0 = wz;
        let z1 = wz + cell_size;

        let mut cx = -grid_radius;
        while cx <= grid_radius {
            let sample_x = center.x + cx + config.texture_offset.x;
            let sample_z = center.y + cz + config.texture_offset.y;
            let wx = cx as f32 * cell_size;
            let dist_sq = wx * wx + wz * wz;

            if dist_sq > fade_outer_sq || !cloud_map.is_cloud(sample_x, sample_z) {
                cx += 1;
                continue;
            }

            // Merge contiguous horizontal runs of cloud cells along X
            let run_start_cx = cx;
            let mut run_end_cx = cx;

            while run_end_cx + 1 <= grid_radius {
                let next_cx = run_end_cx + 1;
                let next_wx = next_cx as f32 * cell_size;
                let next_dist_sq = next_wx * next_wx + wz * wz;
                if next_dist_sq > fade_outer_sq
                    || !cloud_map.is_cloud(
                        center.x + next_cx + config.texture_offset.x,
                        center.y + cz + config.texture_offset.y,
                    )
                {
                    break;
                }
                run_end_cx = next_cx;
            }

            let x0 = run_start_cx as f32 * cell_size;
            let x1 = (run_end_cx + 1) as f32 * cell_size;

            // Merged Bottom face (normal [0, -1, 0]) - CCW facing down
            add_quad(
                [
                    [x0, y0, z0],
                    [x1, y0, z0],
                    [x1, y0, z1],
                    [x0, y0, z1],
                ],
                [0.0, -1.0, 0.0],
                bottom_shade,
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut colors,
                &mut indices,
            );

            // Merged Top face (normal [0, 1, 0]) - CCW facing up
            add_quad(
                [
                    [x0, y1, z1],
                    [x1, y1, z1],
                    [x1, y1, z0],
                    [x0, y1, z0],
                ],
                [0.0, 1.0, 0.0],
                top_shade,
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut colors,
                &mut indices,
            );

            // West wall on run start if exposed (-X) - CCW facing -X
            let has_west = cloud_map.is_cloud(
                center.x + run_start_cx - 1 + config.texture_offset.x,
                sample_z,
            );
            if !has_west {
                add_quad(
                    [
                        [x0, y0, z0],
                        [x0, y0, z1],
                        [x0, y1, z1],
                        [x0, y1, z0],
                    ],
                    [-1.0, 0.0, 0.0],
                    west_east_shade,
                    &mut positions,
                    &mut normals,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                );
            }

            // East wall on run end if exposed (+X) - CCW facing +X
            let has_east = cloud_map.is_cloud(
                center.x + run_end_cx + 1 + config.texture_offset.x,
                sample_z,
            );
            if !has_east {
                add_quad(
                    [
                        [x1, y0, z1],
                        [x1, y0, z0],
                        [x1, y1, z0],
                        [x1, y1, z1],
                    ],
                    [1.0, 0.0, 0.0],
                    west_east_shade,
                    &mut positions,
                    &mut normals,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                );
            }

            // North and South walls for each individual cell in this run
            for cell_cx in run_start_cx..=run_end_cx {
                let cell_x0 = cell_cx as f32 * cell_size;
                let cell_x1 = cell_x0 + cell_size;
                let cell_sample_x = center.x + cell_cx + config.texture_offset.x;

                // North wall (-Z) - CCW facing -Z
                let has_north = cloud_map.is_cloud(cell_sample_x, sample_z - 1);
                if !has_north {
                    add_quad(
                        [
                            [cell_x1, y0, z0],
                            [cell_x0, y0, z0],
                            [cell_x0, y1, z0],
                            [cell_x1, y1, z0],
                        ],
                        [0.0, 0.0, -1.0],
                        north_south_shade,
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                    );
                }

                // South wall (+Z) - CCW facing +Z
                let has_south = cloud_map.is_cloud(cell_sample_x, sample_z + 1);
                if !has_south {
                    add_quad(
                        [
                            [cell_x0, y0, z1],
                            [cell_x1, y0, z1],
                            [cell_x1, y1, z1],
                            [cell_x0, y1, z1],
                        ],
                        [0.0, 0.0, 1.0],
                        north_south_shade,
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                    );
                }
            }

            cx = run_end_cx + 1;
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn load_cloud_texture_map() -> CloudTextureMap {
    let path = "assets/textures/environments/clouds.png";
    if let Ok(opened) = image::open(path) {
        let rgba = opened.into_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        let mut raw_data = Vec::with_capacity((width * height) as usize);
        for pixel in rgba.pixels() {
            raw_data.push(pixel[3] > 64);
        }

        // Clean up isolated 1-pixel noise specks
        let mut clean_data = raw_data.clone();
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let idx = (y * width as i32 + x) as usize;
                if raw_data[idx] {
                    let mut neighbors = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = (x + dx).rem_euclid(width as i32) as usize;
                            let ny = (y + dy).rem_euclid(height as i32) as usize;
                            if raw_data[ny * width as usize + nx] {
                                neighbors += 1;
                            }
                        }
                    }
                    if neighbors == 0 {
                        clean_data[idx] = false;
                    }
                }
            }
        }

        return CloudTextureMap {
            width,
            height,
            data: clean_data,
        };
    }

    warn!(
        "Failed to load cloud image at {}, using procedural fallback",
        path
    );
    procedural_cloud_fallback()
}

fn procedural_cloud_fallback() -> CloudTextureMap {
    let size = 64;
    let mut data = vec![false; size * size];
    for y in 0..size {
        for x in 0..size {
            let cx = (x as f32 - 32.0) / 10.0;
            let cy = (y as f32 - 32.0) / 10.0;
            if (cx * cx + cy * cy) < 2.5 {
                data[y * size + x] = true;
            }
        }
    }
    CloudTextureMap {
        width: size as u32,
        height: size as u32,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::mesh::VertexAttributeValues;

    #[test]
    fn test_cloud_texture_map_loads() {
        let map = load_cloud_texture_map();
        assert!(map.width > 0);
        assert!(map.height > 0);
        let cloud_count = map.data.iter().filter(|&&c| c).count();
        assert!(cloud_count > 0, "Cloud map should contain solid cloud cells");
    }

    #[test]
    fn test_3d_cloud_mesh_attributes_and_shading() {
        let map = load_cloud_texture_map();
        let config = &CLOUD_LAYER_CONFIGS[0];
        let mesh = generate_3d_cloud_mesh(&map, config, IVec2::ZERO);

        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).expect("Missing positions");
        let normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).expect("Missing normals");
        let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR).expect("Missing vertex colors");
        let indices = mesh.indices().expect("Missing indices");

        assert!(!indices.is_empty(), "Cloud mesh must contain indices");

        let fade_inner = config.cell_size * (config.grid_radius as f32 * 0.58);
        let fade_outer = config.cell_size * (config.grid_radius as f32 * 0.95);

        if let (
            VertexAttributeValues::Float32x3(pos_data),
            VertexAttributeValues::Float32x3(norm_data),
            VertexAttributeValues::Float32x4(color_data),
        ) = (positions, normals, colors) {
            assert_eq!(pos_data.len(), norm_data.len());
            assert_eq!(pos_data.len(), color_data.len());

            let mut has_bottom = false;
            let mut has_top = false;
            let mut has_side = false;

            for i in 0..norm_data.len() {
                let norm = norm_data[i];
                let col = color_data[i];
                let pos = pos_data[i];

                let dist = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
                if dist <= fade_inner {
                    assert!(
                        (col[3] - config.base_alpha).abs() < 0.01,
                        "Inner radius vertices must have full base alpha, got {}",
                        col[3]
                    );
                } else if dist >= fade_outer {
                    assert!(
                        col[3] < 0.05,
                        "Outer radius vertices must have near-zero alpha, got {}",
                        col[3]
                    );
                }

                // Shading verification (whiter thresholds)
                if norm[1] < -0.9 {
                    has_bottom = true;
                    assert!(
                        (col[0] - 0.82).abs() < 0.01,
                        "Bottom face must have 0.82 bright underside shadow"
                    );
                } else if norm[1] > 0.9 {
                    has_top = true;
                    assert!(
                        (col[0] - 1.00).abs() < 0.01,
                        "Top face must have 1.00 full sunlight"
                    );
                } else {
                    has_side = true;
                    assert!(
                        col[0] >= 0.85 && col[0] <= 0.94,
                        "Side walls must have 0.87-0.90 directional shading, got {}",
                        col[0]
                    );
                }
            }

            assert!(has_bottom, "Must have bottom faces");
            assert!(has_top, "Must have top faces");
            assert!(has_side, "Must have 3D vertical side faces");
        } else {
            panic!("Unexpected vertex attribute format");
        }
    }

    #[test]
    fn test_sample_cloud_color_day_night() {
        let noon = sample_cloud_color(0.25);
        if let Color::LinearRgba(c) = noon {
            assert!(c.red > 0.95 && c.green > 0.95 && c.blue > 0.95);
        } else {
            panic!("Expected LinearRgba");
        }

        let sunset = sample_cloud_color(0.50);
        if let Color::LinearRgba(c) = sunset {
            assert!(c.red > c.blue, "Sunset should be warmer than noon/night");
        } else {
            panic!("Expected LinearRgba");
        }
    }
}
