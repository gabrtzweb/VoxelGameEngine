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
    environment::{DayPhase, EnvironmentState, weather::WeatherManager},
    player::{PlayerCamera, PlayerEnvironmentStatus},
};

// =============================================================================
// NIGHT SKY SHOOTING STARS (1-draw-call, 3 bespoke variants shooting_star*.png)
// =============================================================================

const SHOOTING_STAR_VARIANTS: usize = 3;
const SHOOTING_STAR_ATLAS_W: usize = 64;
const SHOOTING_STAR_ATLAS_H: usize = 16 * SHOOTING_STAR_VARIANTS; // 48px
const MAX_SHOOTING_STARS: usize = 6;

const SHOOTING_STAR_FILES: [&str; SHOOTING_STAR_VARIANTS] = [
    "assets/textures/particles/shooting_star.png",
    "assets/textures/particles/shooting_star1.png",
    "assets/textures/particles/shooting_star2.png",
];

#[derive(Component)]
pub struct ShootingStarBatchMesh;

#[derive(Clone, Copy, Default)]
struct ShootingStar {
    /// Unit direction on the sky sphere relative to camera
    dir: Vec3,
    /// Angular velocity vector tangent to the sky sphere
    vel_dir: Vec3,
    length: f32,
    width: f32,
    variant: usize,
    age: f32,
    max_age: f32,
    alive: bool,
}

#[derive(Resource)]
pub struct ShootingStarPool {
    stars: Vec<ShootingStar>,
    cursor: usize,
    mesh_handle: Handle<Mesh>,
    spawn_cooldown: f32,
    rng: u64,
}

impl ShootingStarPool {
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

pub fn setup_shooting_stars(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut atlas_bytes = vec![0u8; SHOOTING_STAR_ATLAS_W * SHOOTING_STAR_ATLAS_H * 4];
    for (v_idx, path) in SHOOTING_STAR_FILES.iter().enumerate() {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (iw, ih) = rgba.dimensions();
            let cw = (iw as usize).min(64);
            let ch = (ih as usize).min(16);
            let oy = v_idx * 16;
            for py in 0..ch {
                for px in 0..cw {
                    let src = rgba.get_pixel(px as u32, py as u32).0;
                    let dst = ((oy + py) * SHOOTING_STAR_ATLAS_W + px) * 4;
                    atlas_bytes[dst..dst + 4].copy_from_slice(&src);
                }
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SHOOTING_STAR_ATLAS_W as u32,
            height: SHOOTING_STAR_ATLAS_H as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
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
        fog_enabled: false,
        ..default()
    });

    commands.spawn((
        ShootingStarBatchMesh,
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(mat_handle),
        Transform::IDENTITY,
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
        bevy::camera::visibility::NoFrustumCulling,
    ));

    commands.insert_resource(ShootingStarPool {
        stars: vec![ShootingStar::default(); MAX_SHOOTING_STARS],
        cursor: 0,
        mesh_handle,
        spawn_cooldown: 4.0,
        rng: 0x57A4_5400_7711_9922,
    });
}

#[allow(clippy::too_many_arguments)]
pub fn update_shooting_stars(
    time: Res<Time>,
    mut pool: ResMut<ShootingStarPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    env_state: Option<Res<EnvironmentState>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    weather: Option<Res<WeatherManager>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut batch_query: Query<&mut Visibility, With<ShootingStarBatchMesh>>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok(cam_tf) = camera_query.single() else {
        return;
    };
    let Ok(mut batch_vis) = batch_query.single_mut() else {
        return;
    };
    let status = env_status.as_deref().copied().unwrap_or_default();
    let is_night = env_state
        .as_ref()
        .is_some_and(|e| matches!(DayPhase::from_time(e.time_of_day), DayPhase::Night));
    let is_clear = weather
        .as_ref()
        .is_none_or(|w| w.rain_intensity < 0.25 && w.snow_intensity < 0.25 && w.thunder_intensity < 0.2);

    if is_night && is_clear && !status.is_underground && !status.is_camera_in_water {
        pool.spawn_cooldown -= dt;
        if pool.spawn_cooldown <= 0.0 {
            pool.spawn_cooldown = 4.5 + pool.next_f32() * 7.5;
            let az = pool.next_f32() * TAU;
            let elev = 0.30 + pool.next_f32() * 0.55;
            let dir = Vec3::new(az.cos() * elev.cos(), elev.sin(), az.sin() * elev.cos()).normalize();
            let tangent = dir.cross(Vec3::Y).normalize_or(Vec3::X);
            let down_tan = tangent.cross(dir).normalize_or(Vec3::NEG_Y);
            let angle = (pool.next_f32() - 0.5) * 1.4;
            let vel_dir = (tangent * angle.cos() + down_tan * (0.45 + angle.sin().abs() * 0.45)).normalize();
            let variant = pool.next_u32() as usize % SHOOTING_STAR_VARIANTS;
            let length = 14.0 + pool.next_f32() * 10.0;
            let width = 1.6 + pool.next_f32() * 0.8;
            let max_age = 0.75 + pool.next_f32() * 0.55;
            let slot = pool.cursor;
            pool.cursor = (pool.cursor + 1) % MAX_SHOOTING_STARS;
            pool.stars[slot] = ShootingStar {
                dir,
                vel_dir,
                length,
                width,
                variant,
                age: 0.0,
                max_age,
                alive: true,
            };
        }
    }

    if !pool.stars.iter().any(|s| s.alive) {
        if *batch_vis != Visibility::Hidden {
            *batch_vis = Visibility::Hidden;
        }
        return;
    }

    let cam_pos = cam_tf.translation;
    let sky_dist = 145.0;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut v_cursor = 0u32;

    for s in pool.stars.iter_mut() {
        if !s.alive {
            continue;
        }
        s.age += dt;
        if s.age >= s.max_age {
            s.alive = false;
            continue;
        }
        s.dir = (s.dir + s.vel_dir * (0.48 * dt)).normalize();
        let center = cam_pos + s.dir * sky_dist;
        let to_cam = (cam_pos - center).normalize_or(Vec3::NEG_Y);
        let fwd = s.vel_dir;
        let side = fwd.cross(to_cam).normalize_or(Vec3::X);

        let half_l = fwd * (s.length * 0.5);
        let half_w = side * (s.width * 0.5);
        let p0 = center - half_l - half_w;
        let p1 = center + half_l - half_w;
        let p2 = center + half_l + half_w;
        let p3 = center - half_l + half_w;

        let v0 = s.variant as f32 / SHOOTING_STAR_VARIANTS as f32;
        let v1 = (s.variant + 1) as f32 / SHOOTING_STAR_VARIANTS as f32;
        let fade = smooth_in_out_alpha(s.age, s.max_age, 0.14, 0.35);
        let col = [1.45 * fade, 1.42 * fade, 1.48 * fade, fade];

        positions.extend_from_slice(&[p0.to_array(), p1.to_array(), p2.to_array(), p3.to_array()]);
        normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
        uvs.extend_from_slice(&[[0.0, v1], [1.0, v1], [1.0, v0], [0.0, v0]]);
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

