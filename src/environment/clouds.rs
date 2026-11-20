use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::Face,
    },
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};

use crate::player::PlayerCamera;

/// Active cloud layer count. Set to 1, 2, or 3 to easily adjust the number of visible layers.
pub const ACTIVE_LAYER_COUNT: usize = 2;

#[derive(Clone, Debug)]
pub struct CloudLayerConfig {
    pub altitude: f32,
    pub thickness: f32,
    pub cell_size: f32,
    #[allow(dead_code)]
    pub grid_radius: i32,
    pub base_alpha: f32,
    pub wind_speed: Vec2,
    pub texture_offset: IVec2,
}

pub const CLOUD_LAYER_CONFIGS: &[CloudLayerConfig] = &[
    // Layer 0: Lower cumulus layer (majestic, voluminous voxel cumulus)
    CloudLayerConfig {
        altitude: 168.0,
        thickness: 6.0,
        cell_size: 12.0,
        grid_radius: 24,
        base_alpha: 0.85,
        wind_speed: Vec2::new(3.2, 1.0),
        texture_offset: IVec2::new(0, 0),
    },
    // Layer 1: Mid-altitude altocumulus layer (thicker billowy cloud blanket)
    CloudLayerConfig {
        altitude: 215.0,
        thickness: 8.0,
        cell_size: 16.0,
        grid_radius: 22,
        base_alpha: 0.70,
        wind_speed: Vec2::new(2.1, 0.6),
        texture_offset: IVec2::new(128, 64),
    },
    // Layer 2: High cirrus layer (very high, deep atmospheric layer)
    CloudLayerConfig {
        altitude: 275.0,
        thickness: 7.0,
        cell_size: 20.0,
        grid_radius: 20,
        base_alpha: 0.50,
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

#[derive(Resource, Clone)]
pub struct CloudTextureMap {
    pub width: u32,
    pub height: u32,
    pub data: Arc<[bool]>,
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
    pub active_task: Option<Task<(IVec2, Mesh)>>,
    pub active_quality: crate::core::CloudQuality,
    pub active_distance: i32,
}

#[derive(Resource)]
pub struct CloudLayersResource {
    pub layers: Vec<SingleLayerState>,
}

pub fn setup_clouds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_config: Option<Res<crate::core::config::LocalConfig>>,
) {
    let cloud_quality = local_config
        .as_ref()
        .map(|c| c.cloud_quality)
        .unwrap_or(crate::core::CloudQuality::Fancy3D);
    let cloud_distance = local_config
        .as_ref()
        .map(|c| c.cloud_render_distance)
        .unwrap_or(64)
        .clamp(16, 128);
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
        let initial_mesh =
            generate_cloud_mesh(&cloud_map, config, IVec2::ZERO, cloud_quality, cloud_distance);
        let mesh_handle = meshes.add(initial_mesh);

        let initial_visibility = if cloud_quality == crate::core::CloudQuality::Off {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };

        commands.spawn((
            CloudLayerVisual { layer_index: index },
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material_handle.clone()),
            Transform::from_xyz(0.0, config.altitude, 0.0),
            initial_visibility,
            NotShadowCaster,
            NotShadowReceiver,
            bevy::camera::visibility::NoFrustumCulling,
        ));

        layer_states.push(SingleLayerState {
            current_cell_center: IVec2::ZERO,
            mesh_handle,
            active_task: None,
            active_quality: cloud_quality,
            active_distance: cloud_distance,
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
    cloud_quality: crate::core::CloudQuality,
    cloud_render_distance: i32,
    storm_darkening: f32,
) {
    let camera_translation = camera.0.translation;
    let cloud_distance = cloud_render_distance.clamp(16, 128);

    for (layer_visual, mut cloud_transform, mut visibility) in &mut cloud_layers {
        if is_underwater || cloud_quality == crate::core::CloudQuality::Off {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }
        if *visibility != Visibility::Visible {
            *visibility = Visibility::Visible;
        }

        let index = layer_visual.layer_index;
        if index >= CLOUD_LAYER_CONFIGS.len() || index >= layers_res.layers.len() {
            continue;
        }
        let config = &CLOUD_LAYER_CONFIGS[index];
        let layer_state = &mut layers_res.layers[index];

        if layer_state.active_quality != cloud_quality || layer_state.active_distance != cloud_distance {
            layer_state.active_quality = cloud_quality;
            layer_state.active_distance = cloud_distance;
            layer_state.current_cell_center = IVec2::splat(i32::MIN);
        }

        // 1. Poll in-flight async cloud meshing task
        if let Some(mut task) = layer_state.active_task.take() {
            if let Some((completed_center, new_mesh)) = check_ready(&mut task) {
                layer_state.current_cell_center = completed_center;
                if let Some(mut mesh) = meshes.get_mut(&layer_state.mesh_handle) {
                    *mesh = new_mesh;
                }
            } else {
                layer_state.active_task = Some(task);
            }
        }

        let period_x = (cloud_map.width as f64) * (config.cell_size as f64);
        let period_z = (cloud_map.height as f64) * (config.cell_size as f64);

        let wind_x =
            (in_game_time_seconds * config.wind_speed.x as f64).rem_euclid(period_x) as f32;
        let wind_z =
            (in_game_time_seconds * config.wind_speed.y as f64).rem_euclid(period_z) as f32;

        let world_center_x = camera_translation.x - wind_x;
        let world_center_z = camera_translation.z - wind_z;

        let cell_x = (world_center_x / config.cell_size).floor() as i32;
        let cell_z = (world_center_z / config.cell_size).floor() as i32;

        let target_cell_center = IVec2::new(cell_x, cell_z);

        // 2. Spawn async task if target cell center has shifted and no task is in flight
        if target_cell_center != layer_state.current_cell_center
            && layer_state.active_task.is_none()
        {
            let pool = AsyncComputeTaskPool::get();
            let map_clone = cloud_map.clone();
            let config_clone = config.clone();
            layer_state.active_task = Some(pool.spawn(async move {
                let mesh = generate_cloud_mesh(
                    &map_clone,
                    &config_clone,
                    target_cell_center,
                    cloud_quality,
                    cloud_distance,
                );
                (target_cell_center, mesh)
            }));
        }

        let subcell_x =
            world_center_x - (layer_state.current_cell_center.x as f32 * config.cell_size);
        let subcell_z =
            world_center_z - (layer_state.current_cell_center.y as f32 * config.cell_size);

        cloud_transform.translation = Vec3::new(
            camera_translation.x - subcell_x,
            config.altitude,
            camera_translation.z - subcell_z,
        );
    }

    // Dynamic atmospheric cloud tinting across the day-night cycle.
    let mut target_color = sample_cloud_color(time_of_day);
    if storm_darkening > 0.0 {
        let lin = target_color.to_linear();
        let dark_factor = (1.0 - storm_darkening * 0.65).clamp(0.20, 1.0);
        let storm_rgb = LinearRgba::new(
            lin.red * dark_factor * 0.90,
            lin.green * dark_factor * 0.92,
            lin.blue * dark_factor,
            (lin.alpha + storm_darkening * 0.15).min(1.0),
        );
        target_color = Color::LinearRgba(storm_rgb);
    }

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
            LinearRgba::new(1.0, 0.92, 0.85, 0.95), // Dawn warm peach
            LinearRgba::new(1.0, 1.0, 1.0, 0.98),   // Midday bright white
            f,
        )
    } else if t < 0.50 {
        let f = (t - 0.25) / 0.25;
        (
            LinearRgba::new(1.0, 1.0, 1.0, 0.98),   // Midday bright white
            LinearRgba::new(1.0, 0.72, 0.60, 0.95), // Sunset vibrant amber/pink
            f,
        )
    } else if t < 0.75 {
        let f = (t - 0.50) / 0.25;
        (
            LinearRgba::new(1.0, 0.72, 0.60, 0.95), // Sunset vibrant amber/pink
            LinearRgba::new(0.30, 0.35, 0.48, 0.85), // Midnight deep navy
            f,
        )
    } else {
        let f = (t - 0.75) / 0.25;
        (
            LinearRgba::new(0.30, 0.35, 0.48, 0.85), // Midnight deep navy
            LinearRgba::new(1.0, 0.92, 0.85, 0.95),  // Dawn warm peach
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

pub fn generate_cloud_mesh(
    cloud_map: &CloudTextureMap,
    config: &CloudLayerConfig,
    center: IVec2,
    quality: crate::core::CloudQuality,
    cloud_render_distance: i32,
) -> Mesh {
    match quality {
        crate::core::CloudQuality::Off => {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 4]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 4]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 4]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0, 0.0]; 4]);
            mesh.insert_indices(bevy::render::mesh::Indices::U32(vec![0, 1, 2, 0, 2, 3]));
            mesh
        }
        crate::core::CloudQuality::Fast2D => {
            generate_cloud_mesh_internal(cloud_map, config, center, cloud_render_distance, false)
        }
        crate::core::CloudQuality::Fancy3D => {
            generate_cloud_mesh_internal(cloud_map, config, center, cloud_render_distance, true)
        }
    }
}

/// High-performance 2D-greedy cloud mesh builder supporting ultra-distant cloud horizons (up to 128 chunks / 2048m).
///
/// - Inside the solid core (`d <= 0.72 * R`): full 2D rectangle greedy merging along X and Z collapses
///   large cloud sheets into minimal quads.
/// - Inside the outer horizon fade ring (`0.72 * R .. 0.98 * R`): bounded horizontal runs (`<= 4` cells)
///   keep smooth radial alpha dissolve into the sky.
/// - In `Fancy3D` mode: North/South/East/West vertical side walls are greedy-merged into runs and culled
///   beyond `0.68 * R` where distant clouds are viewed edge-on/underneath.
fn generate_cloud_mesh_internal(
    cloud_map: &CloudTextureMap,
    config: &CloudLayerConfig,
    center: IVec2,
    cloud_render_distance: i32,
    is_3d: bool,
) -> Mesh {
    let cell_size = config.cell_size;
    let world_radius = (cloud_render_distance.clamp(16, 128) as f32) * 16.0;
    let grid_radius = (world_radius / cell_size).ceil() as i32;
    let dim = (grid_radius * 2 + 1) as usize;

    let fade_inner = world_radius * 0.72;
    let fade_outer = world_radius * 0.98;
    let fade_inner_sq = fade_inner * fade_inner;
    let fade_outer_sq = fade_outer * fade_outer;
    let fade_range = (fade_outer - fade_inner).max(1.0);
    let wall_max_r = world_radius * 0.68;
    let wall_max_sq = wall_max_r * wall_max_r;

    let y0 = 0.0;
    let y1 = if is_3d { config.thickness } else { 0.05 };
    let base_alpha = config.base_alpha;

    // 0 = empty/outside, 1 = solid core cloud cell, 2 = fade-ring cloud cell
    let mut grid = vec![0u8; dim * dim];
    for gz in 0..dim {
        let cz = gz as i32 - grid_radius;
        let wz = (cz as f32 + 0.5) * cell_size;
        let sample_z = center.y + cz + config.texture_offset.y;
        let row_off = gz * dim;
        for gx in 0..dim {
            let cx = gx as i32 - grid_radius;
            let wx = (cx as f32 + 0.5) * cell_size;
            let dist_sq = wx * wx + wz * wz;
            if dist_sq > fade_outer_sq {
                continue;
            }
            let sample_x = center.x + cx + config.texture_offset.x;
            if cloud_map.is_cloud(sample_x, sample_z) {
                grid[row_off + gx] = if dist_sq <= fade_inner_sq { 1 } else { 2 };
            }
        }
    }

    let est_quads = (dim * dim / 4).max(256);
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(est_quads * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(est_quads * 4);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(est_quads * 4);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(est_quads * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(est_quads * 6);

    let calc_alpha = |x: f32, z: f32| -> f32 {
        let d_sq = x * x + z * z;
        if d_sq <= fade_inner_sq {
            base_alpha
        } else if d_sq >= fade_outer_sq {
            0.0
        } else {
            let d = d_sq.sqrt();
            let t = (d - fade_inner) / fade_range;
            let smooth = crate::core::math::smoothstep(t);
            (1.0 - smooth) * base_alpha
        }
    };

    let add_quad = |p: [[f32; 3]; 4],
                    norm: [f32; 3],
                    shade: [f32; 3],
                    const_alpha: Option<f32>,
                    pos_out: &mut Vec<[f32; 3]>,
                    norm_out: &mut Vec<[f32; 3]>,
                    uv_out: &mut Vec<[f32; 2]>,
                    col_out: &mut Vec<[f32; 4]>,
                    idx_out: &mut Vec<u32>| {
        let start_idx = pos_out.len() as u32;
        let uv_coords = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        for (i, pt) in p.iter().enumerate() {
            let a = const_alpha.unwrap_or_else(|| calc_alpha(pt[0], pt[2]));
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

    let bottom_shade = if is_3d {
        [0.91, 0.92, 0.94]
    } else {
        [0.93, 0.94, 0.95]
    };
    let top_shade = [1.00, 1.00, 1.00];
    let west_east_shade = [0.96, 0.97, 0.98];
    let north_south_shade = [0.93, 0.94, 0.95];

    // Pass 1: 2D greedy rectangle merge for solid core (1) + bounded runs for fade ring (2)
    let mut visited = vec![false; dim * dim];
    for gz in 0..dim {
        let row_off = gz * dim;
        for gx in 0..dim {
            let idx = row_off + gx;
            let cell_kind = grid[idx];
            if cell_kind == 0 || visited[idx] {
                continue;
            }

            let max_w = if cell_kind == 1 { dim - gx } else { (dim - gx).min(4) };
            let mut w = 1;
            while w < max_w && grid[row_off + gx + w] == cell_kind && !visited[row_off + gx + w] {
                w += 1;
            }

            let max_h = if cell_kind == 1 { dim - gz } else { (dim - gz).min(4) };
            let mut h = 1;
            'rows: while h < max_h {
                let next_row = (gz + h) * dim;
                for dx in 0..w {
                    let ni = next_row + gx + dx;
                    if grid[ni] != cell_kind || visited[ni] {
                        break 'rows;
                    }
                }
                h += 1;
            }

            for dz in 0..h {
                let r = (gz + dz) * dim;
                for dx in 0..w {
                    visited[r + gx + dx] = true;
                }
            }

            let x0 = (gx as i32 - grid_radius) as f32 * cell_size;
            let x1 = ((gx + w) as i32 - grid_radius) as f32 * cell_size;
            let z0 = (gz as i32 - grid_radius) as f32 * cell_size;
            let z1 = ((gz + h) as i32 - grid_radius) as f32 * cell_size;
            let const_a = if cell_kind == 1 {
                Some(base_alpha)
            } else {
                None
            };

            // Bottom face (-Y)
            add_quad(
                [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
                [0.0, -1.0, 0.0],
                bottom_shade,
                const_a,
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut colors,
                &mut indices,
            );
            // Top face (+Y)
            add_quad(
                [[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]],
                [0.0, 1.0, 0.0],
                top_shade,
                const_a,
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut colors,
                &mut indices,
            );
        }
    }

    // Pass 2 (Fancy3D only): Greedy-merged vertical side walls within 68% horizon radius
    if is_3d {
        let has_cloud = |gx: i32, gz: i32| -> bool {
            let cx = gx - grid_radius;
            let cz = gz - grid_radius;
            cloud_map.is_cloud(
                center.x + cx + config.texture_offset.x,
                center.y + cz + config.texture_offset.y,
            )
        };

        // North (-Z) and South (+Z) walls merged along X
        for gz in 0..dim {
            let cz = gz as i32 - grid_radius;
            let wz_c = (cz as f32 + 0.5) * cell_size;
            if wz_c * wz_c > wall_max_sq {
                continue;
            }
            let z0 = cz as f32 * cell_size;
            let z1 = z0 + cell_size;
            let row_off = gz * dim;

            // North (-Z)
            let mut gx = 0;
            while gx < dim {
                let cx = gx as i32 - grid_radius;
                let wx_c = (cx as f32 + 0.5) * cell_size;
                if grid[row_off + gx] == 0
                    || wx_c * wx_c + wz_c * wz_c > wall_max_sq
                    || has_cloud(gx as i32, gz as i32 - 1)
                {
                    gx += 1;
                    continue;
                }
                let start = gx;
                gx += 1;
                while gx < dim && gx - start < 16 {
                    let ncx = gx as i32 - grid_radius;
                    let nwx = (ncx as f32 + 0.5) * cell_size;
                    if grid[row_off + gx] == 0
                        || nwx * nwx + wz_c * wz_c > wall_max_sq
                        || has_cloud(gx as i32, gz as i32 - 1)
                    {
                        break;
                    }
                    gx += 1;
                }
                let x0 = (start as i32 - grid_radius) as f32 * cell_size;
                let x1 = (gx as i32 - grid_radius) as f32 * cell_size;
                add_quad(
                    [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]],
                    [0.0, 0.0, -1.0],
                    north_south_shade,
                    Some(base_alpha),
                    &mut positions,
                    &mut normals,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                );
            }

            // South (+Z)
            let mut gx = 0;
            while gx < dim {
                let cx = gx as i32 - grid_radius;
                let wx_c = (cx as f32 + 0.5) * cell_size;
                if grid[row_off + gx] == 0
                    || wx_c * wx_c + wz_c * wz_c > wall_max_sq
                    || has_cloud(gx as i32, gz as i32 + 1)
                {
                    gx += 1;
                    continue;
                }
                let start = gx;
                gx += 1;
                while gx < dim && gx - start < 16 {
                    let ncx = gx as i32 - grid_radius;
                    let nwx = (ncx as f32 + 0.5) * cell_size;
                    if grid[row_off + gx] == 0
                        || nwx * nwx + wz_c * wz_c > wall_max_sq
                        || has_cloud(gx as i32, gz as i32 + 1)
                    {
                        break;
                    }
                    gx += 1;
                }
                let x0 = (start as i32 - grid_radius) as f32 * cell_size;
                let x1 = (gx as i32 - grid_radius) as f32 * cell_size;
                add_quad(
                    [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]],
                    [0.0, 0.0, 1.0],
                    north_south_shade,
                    Some(base_alpha),
                    &mut positions,
                    &mut normals,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                );
            }
        }

        // West (-X) and East (+X) walls merged along Z
        for gx in 0..dim {
            let cx = gx as i32 - grid_radius;
            let wx_c = (cx as f32 + 0.5) * cell_size;
            if wx_c * wx_c > wall_max_sq {
                continue;
            }
            let x0 = cx as f32 * cell_size;
            let x1 = x0 + cell_size;

            // West (-X)
            let mut gz = 0;
            while gz < dim {
                let cz = gz as i32 - grid_radius;
                let wz_c = (cz as f32 + 0.5) * cell_size;
                if grid[gz * dim + gx] == 0
                    || wx_c * wx_c + wz_c * wz_c > wall_max_sq
                    || has_cloud(gx as i32 - 1, gz as i32)
                {
                    gz += 1;
                    continue;
                }
                let start = gz;
                gz += 1;
                while gz < dim && gz - start < 16 {
                    let ncz = gz as i32 - grid_radius;
                    let nwz = (ncz as f32 + 0.5) * cell_size;
                    if grid[gz * dim + gx] == 0
                        || wx_c * wx_c + nwz * nwz > wall_max_sq
                        || has_cloud(gx as i32 - 1, gz as i32)
                    {
                        break;
                    }
                    gz += 1;
                }
                let z0 = (start as i32 - grid_radius) as f32 * cell_size;
                let z1 = (gz as i32 - grid_radius) as f32 * cell_size;
                add_quad(
                    [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]],
                    [-1.0, 0.0, 0.0],
                    west_east_shade,
                    Some(base_alpha),
                    &mut positions,
                    &mut normals,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                );
            }

            // East (+X)
            let mut gz = 0;
            while gz < dim {
                let cz = gz as i32 - grid_radius;
                let wz_c = (cz as f32 + 0.5) * cell_size;
                if grid[gz * dim + gx] == 0
                    || wx_c * wx_c + wz_c * wz_c > wall_max_sq
                    || has_cloud(gx as i32 + 1, gz as i32)
                {
                    gz += 1;
                    continue;
                }
                let start = gz;
                gz += 1;
                while gz < dim && gz - start < 16 {
                    let ncz = gz as i32 - grid_radius;
                    let nwz = (ncz as f32 + 0.5) * cell_size;
                    if grid[gz * dim + gx] == 0
                        || wx_c * wx_c + nwz * nwz > wall_max_sq
                        || has_cloud(gx as i32 + 1, gz as i32)
                    {
                        break;
                    }
                    gz += 1;
                }
                let z0 = (start as i32 - grid_radius) as f32 * cell_size;
                let z1 = (gz as i32 - grid_radius) as f32 * cell_size;
                add_quad(
                    [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]],
                    [1.0, 0.0, 0.0],
                    west_east_shade,
                    Some(base_alpha),
                    &mut positions,
                    &mut normals,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                );
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
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
            data: clean_data.into(),
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
        data: data.into(),
    }
}
