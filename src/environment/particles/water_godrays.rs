use std::f32::consts::TAU;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use super::smooth_in_out_alpha;
use crate::{
    environment::EnvironmentState,
    player::{PlayerCamera, PlayerEnvironmentStatus},
    world::{Voxel, storage::VoxelWorld},
};

// =============================================================================
// 3D WORLD-ANCHORED UNDERWATER GOD RAYS (`water_godray.png`)
// =============================================================================

const MAX_UW_GODRAYS: usize = 28;

#[derive(Component)]
pub struct WaterGodRay3DBatchMesh;

#[derive(Clone, Copy, Default)]
struct WaterGodRay3D {
    /// Top anchor on the water surface in world space
    surface_pos: Vec3,
    width: f32,
    height: f32,
    alpha: f32,
    age: f32,
    max_age: f32,
    alive: bool,
}

#[derive(Resource)]
pub struct WaterGodRay3DPool {
    rays: Vec<WaterGodRay3D>,
    cursor: usize,
    mesh_handle: Handle<Mesh>,
    rng: u64,
}

impl WaterGodRay3DPool {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.rng = self
            .rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.rng >> 32) as u32
    }

    #[inline]
    fn next_f32(&mut self) -> f32 {
        (self.next_u32() & 0x00FF_FFFF) as f32 / 16_777_216.0
    }
}

pub fn setup_water_godrays_3d(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (w, h, bytes) = if let Ok(img) = image::open("assets/textures/particles/water_godray.png") {
        let rgba = img.to_rgba8();
        let (iw, ih) = rgba.dimensions();
        (iw, ih, rgba.into_raw())
    } else {
        (64, 64, vec![255u8; 64 * 64 * 4])
    };

    let mut image = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    let tex_handle = images.add(image);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32, 0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32, 0.0, 0.0, 0.0]; 3]);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    let mesh_handle = meshes.add(mesh);

    let mat_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(tex_handle),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    commands.spawn((
        WaterGodRay3DBatchMesh,
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(mat_handle),
        Transform::IDENTITY,
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
        bevy::camera::visibility::NoFrustumCulling,
    ));

    commands.insert_resource(WaterGodRay3DPool {
        rays: vec![WaterGodRay3D::default(); MAX_UW_GODRAYS],
        cursor: 0,
        mesh_handle,
        rng: 0x60D4_A715_3388_1234,
    });
}

#[allow(clippy::too_many_arguments)]
pub fn update_water_godrays_3d(
    time: Res<Time>,
    mut pool: ResMut<WaterGodRay3DPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_world: Option<Res<VoxelWorld>>,
    env_state: Option<Res<EnvironmentState>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    local_config: Option<Res<crate::core::config::LocalConfig>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut batch_query: Query<&mut Visibility, With<WaterGodRay3DBatchMesh>>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok(cam_tf) = camera_query.single() else {
        return;
    };
    let Ok(mut batch_vis) = batch_query.single_mut() else {
        return;
    };
    let status = env_status.as_deref().copied().unwrap_or_default();
    let god_rays_enabled = local_config.as_ref().is_none_or(|c| c.god_rays_enabled);
    let sun_dir = env_state.as_ref().map_or(Vec3::new(0.4, 0.85, 0.2), |e| {
        crate::environment::celestial::calculate_sun_direction(e.time_of_day)
    });
    let sun_elev = ((sun_dir.y - 0.05) / 0.35).clamp(0.0, 1.0);

    if !status.is_camera_in_water || !god_rays_enabled || sun_elev <= 0.01 {
        if *batch_vis != Visibility::Hidden {
            *batch_vis = Visibility::Hidden;
        }
        return;
    }

    let cam_pos = cam_tf.translation;

    // Spawn 3D world-anchored volumetric godray quads hanging from the water surface around the player
    if let Some(ref world) = voxel_world {
        let active_count = pool.rays.iter().filter(|r| r.alive).count();
        if active_count < MAX_UW_GODRAYS && pool.next_f32() < (7.5 * dt).min(1.0) {
            let angle = pool.next_f32() * TAU;
            let dist = 4.5 + pool.next_f32() * 19.0;
            let wx = (cam_pos.x + angle.cos() * dist).floor() as i32;
            let wz = (cam_pos.z + angle.sin() * dist).floor() as i32;
            let start_y = (cam_pos.y.floor() as i32).max(-10);
            // Scan upward to find the water surface exposed to Air
            for wy in start_y..=(start_y + 24) {
                let p = IVec3::new(wx, wy, wz);
                if world.get_voxel(p).is_some_and(Voxel::is_water)
                    && world.get_voxel(p + IVec3::Y) == Some(Voxel::Air)
                {
                    let width = 2.6 + pool.next_f32() * 2.2;
                    let height = 9.5 + pool.next_f32() * 7.0;
                    let alpha = 0.20 + pool.next_f32() * 0.14;
                    let max_age = 4.5 + pool.next_f32() * 3.5;
                    let slot = pool.cursor;
                    pool.cursor = (pool.cursor + 1) % MAX_UW_GODRAYS;
                    pool.rays[slot] = WaterGodRay3D {
                        surface_pos: Vec3::new(wx as f32 + 0.5, wy as f32 + 0.88, wz as f32 + 0.5),
                        width,
                        height,
                        alpha,
                        age: 0.0,
                        max_age,
                        alive: true,
                    };
                    break;
                }
            }
        }
    }

    // Slant rays along the sun's incoming vector (clamped to a pleasing natural underwater angle)
    let slant_dir = Vec3::new(-sun_dir.x * 0.45, -1.0, -sun_dir.z * 0.45).normalize();

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut v_cursor = 0u32;

    for r in pool.rays.iter_mut() {
        if !r.alive {
            continue;
        }
        r.age += dt;
        if r.age >= r.max_age {
            r.alive = false;
            continue;
        }
        let dist_sq = (r.surface_pos - cam_pos).length_squared();
        if dist_sq > 34.0 * 34.0 {
            r.alive = false;
            continue;
        }

        let top_c = r.surface_pos;
        let bot_c = top_c + slant_dir * r.height;
        let mid_c = (top_c + bot_c) * 0.5;

        // Cylindrical Y-axis billboard: always faces the camera horizontally while staying 100% locked in 3D world space!
        let to_cam = Vec3::new(cam_pos.x - mid_c.x, 0.0, cam_pos.z - mid_c.z).normalize_or(Vec3::Z);
        let right = to_cam.cross(Vec3::Y).normalize_or(Vec3::X);
        let half_w = right * (r.width * 0.5);

        let p0 = bot_c - half_w;
        let p1 = bot_c + half_w;
        let p2 = top_c + half_w;
        let p3 = top_c - half_w;

        let dist = dist_sq.sqrt();
        let near_fade = ((dist - 2.0) / 3.0).clamp(0.0, 1.0);
        let far_fade = (1.0 - (dist - 16.0).max(0.0) / 14.0).clamp(0.0, 1.0);
        let life_fade = smooth_in_out_alpha(r.age, r.max_age, 1.2, 1.6);
        let a = r.alpha * sun_elev * near_fade * far_fade * life_fade;
        if a <= 0.005 {
            continue;
        }
        let col = [0.65 * a, 0.94 * a, 1.0 * a, a];

        positions.extend_from_slice(&[p0.to_array(), p1.to_array(), p2.to_array(), p3.to_array()]);
        normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
        uvs.extend_from_slice(&[[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
        colors.extend_from_slice(&[col; 4]);
        indices.extend_from_slice(&[
            v_cursor,
            v_cursor + 1,
            v_cursor + 2,
            v_cursor,
            v_cursor + 2,
            v_cursor + 3,
        ]);
        v_cursor += 4;
    }

    if positions.is_empty() {
        if *batch_vis != Visibility::Hidden {
            *batch_vis = Visibility::Hidden;
        }
        return;
    }

    if *batch_vis != Visibility::Visible {
        *batch_vis = Visibility::Visible;
    }

    if let Some(mut mesh) = meshes.get_mut(&pool.mesh_handle) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
    }
}

