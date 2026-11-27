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
    environment::{
        DayPhase, EnvironmentState,
        weather::{WeatherManager, WindState},
    },
    generation::BiomeType,
    player::{PlayerCamera, PlayerEnvironmentStatus},
    world::{Voxel, storage::VoxelWorld},
};

// =============================================================================
// Animated Wind Swirl Ribbons (horizontal flipbook streaks)
// =============================================================================

const MAX_WIND_STREAKS: usize = 64;
const WIND_ATLAS_W: usize = 320; // 64 (wind.png) + 128 (wind1.png) + 128 (wind2.png)
const WIND_ATLAS_H: usize = 576; // max strip height (wind2.png = 18 * 32)

#[derive(Component)]
pub struct WindStreakBatchMesh;

#[derive(Clone, Copy)]
struct WindStreak {
    position: Vec3,
    velocity: Vec3,
    width: f32,
    height: f32,
    variant: usize, // 0 = wind.png (12f), 1 = wind1.png (12f), 2 = wind2.png (18f)
    alpha: f32,
    age: f32,
    max_age: f32,
    alive: bool,
}

impl Default for WindStreak {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::X,
            width: 2.2,
            height: 0.78,
            variant: 0,
            alpha: 0.55,
            age: 0.0,
            max_age: 1.6,
            alive: false,
        }
    }
}

#[derive(Resource)]
pub struct WindStreakPool {
    streaks: Vec<WindStreak>,
    cursor: usize,
    mesh_handle: Handle<Mesh>,
    rng: u64,
}

impl WindStreakPool {
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

pub fn setup_wind_streaks(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut atlas_bytes = vec![0u8; WIND_ATLAS_W * WIND_ATLAS_H * 4];
    let sources: [(&str, usize, usize, usize); 3] = [
        ("assets/textures/particles/wind.png", 0, 64, 384),
        ("assets/textures/particles/wind1.png", 64, 128, 384),
        ("assets/textures/particles/wind2.png", 192, 128, 576),
    ];

    for &(path, ox, max_w, max_h) in &sources {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (iw, ih) = rgba.dimensions();
            let cw = (iw as usize).min(max_w);
            let ch = (ih as usize).min(max_h);
            for py in 0..ch {
                for px in 0..cw {
                    let src = rgba.get_pixel(px as u32, py as u32).0;
                    let dst = (py * WIND_ATLAS_W + (ox + px)) * 4;
                    atlas_bytes[dst..dst + 4].copy_from_slice(&src);
                }
            }
        }
    }

    let mut atlas_image = Image::new(
        Extent3d {
            width: WIND_ATLAS_W as u32,
            height: WIND_ATLAS_H as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    atlas_image.sampler = ImageSampler::nearest();
    let atlas_handle = images.add(atlas_image);

    let material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(atlas_handle),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 0.0]; 3]);
    mesh.insert_indices(Indices::U32(vec![0, 0, 0]));
    let mesh_handle = meshes.add(mesh);

    commands.spawn((
        WindStreakBatchMesh,
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material_handle),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
        bevy::camera::visibility::NoFrustumCulling,
    ));

    commands.insert_resource(WindStreakPool {
        streaks: vec![WindStreak::default(); MAX_WIND_STREAKS],
        cursor: 0,
        mesh_handle,
        rng: 0x571D_BEEF_1337_CAFE,
    });
}

#[allow(clippy::too_many_arguments)]
pub fn update_wind_streaks(
    time: Res<Time>,
    mut pool: ResMut<WindStreakPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_world: Option<Res<VoxelWorld>>,
    env_state: Option<Res<EnvironmentState>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    weather: Option<Res<WeatherManager>>,
    wind: Option<Res<WindState>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut batch_query: Query<&mut Visibility, With<WindStreakBatchMesh>>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok(cam_tf) = camera_query.single() else {
        return;
    };
    let Ok(mut batch_vis) = batch_query.single_mut() else {
        return;
    };
    let Some(ref voxel_world) = voxel_world else {
        return;
    };

    let status = env_status.as_deref().copied().unwrap_or_default();
    let cam_pos = cam_tf.translation;

    let wind_vec = wind.as_ref().map_or(Vec3::new(1.0, 0.0, 0.4), |w| w.current_wind);
    let wind_horiz = Vec3::new(wind_vec.x, 0.0, wind_vec.z);
    let wind_dir = if wind_horiz.length_squared() > 0.001 {
        wind_horiz.normalize()
    } else {
        Vec3::new(0.866, 0.0, 0.5)
    };

    let storm_boost = weather.as_ref().map_or(0.0, |w| {
        w.rain_intensity * 1.4 + w.thunder_intensity * 2.0 + w.snow_intensity * 1.2 + w.dust_intensity * 2.2
    });

    // Spawn near terrain in windy biomes (plains, steppes, mountains, coasts, and during storms)
    if !status.is_underground && !status.is_camera_in_water {
        let biome_rate = match status.current_biome {
            BiomeType::AlpinePeaks
            | BiomeType::JaggedPeaks
            | BiomeType::WindyPass
            | BiomeType::MonolithCliffs
            | BiomeType::RollingSteppe
            | BiomeType::CoastalCliffs
            | BiomeType::DuneSea
            | BiomeType::IceboundRidge
            | BiomeType::SnowyPlains => 2.2,
            BiomeType::Grasslands
            | BiomeType::FlowerMeadow
            | BiomeType::SandyShores
            | BiomeType::DuneBeach
            | BiomeType::AcaciaPlains
            | BiomeType::PermafrostSteppe
            | BiomeType::EmeraldFoothills => 1.15,
            _ => 0.45,
        };
        let total_rate = biome_rate + storm_boost * 1.6;
        if pool.next_f32() < (total_rate * dt).min(1.0) {
            let wx = (cam_pos.x + (pool.next_f32() - 0.5) * 42.0).floor() as i32;
            let wz = (cam_pos.z + (pool.next_f32() - 0.5) * 42.0).floor() as i32;
            let base_y = cam_pos.y.floor() as i32;
            for wy in (base_y - 5..=base_y + 10).rev() {
                let here = IVec3::new(wx, wy, wz);
                let below = IVec3::new(wx, wy - 1, wz);
                if voxel_world.get_voxel(here) == Some(Voxel::Air)
                    && voxel_world.get_voxel(below).is_some_and(Voxel::is_collidable)
                {
                    let height_above = 1.2 + pool.next_f32() * 5.2;
                    let spawn_pos = Vec3::new(
                        wx as f32 + 0.5,
                        wy as f32 + height_above,
                        wz as f32 + 0.5,
                    );
                    let speed = (2.8 + pool.next_f32() * 2.2) * (1.0 + storm_boost * 0.35);
                    let variant = pool.next_u32() as usize % 3;
                    let (w, h) = match variant {
                        0 => (2.1 + pool.next_f32() * 0.7, 0.78 + pool.next_f32() * 0.25),
                        1 => (3.0 + pool.next_f32() * 0.9, 0.82 + pool.next_f32() * 0.25),
                        _ => (3.2 + pool.next_f32() * 0.9, 0.85 + pool.next_f32() * 0.25),
                    };
                    let max_age = 1.45 + pool.next_f32() * 0.65;
                    let alpha = (0.34 + pool.next_f32() * 0.16 + storm_boost * 0.08).min(0.68);
                    let slot = pool.cursor;
                    pool.cursor = (pool.cursor + 1) % MAX_WIND_STREAKS;
                    pool.streaks[slot] = WindStreak {
                        position: spawn_pos,
                        velocity: wind_dir * speed,
                        width: w,
                        height: h,
                        variant,
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

    let day_light = env_state
        .as_ref()
        .map_or(1.0, |e| match DayPhase::from_time(e.time_of_day) {
            DayPhase::Night => 0.35,
            DayPhase::Evening | DayPhase::Morning => 0.85,
            DayPhase::Noon => 1.0,
        });

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut v_cursor = 0u32;

    for s in pool.streaks.iter_mut() {
        if !s.alive {
            continue;
        }
        s.age += dt;
        if s.age >= s.max_age {
            s.alive = false;
            continue;
        }
        s.position += s.velocity * dt;

        let progress = (s.age / s.max_age).clamp(0.0, 0.999);
        let (ox, fw, frame_count) = match s.variant {
            0 => (0.0_f32, 64.0_f32, 12usize),
            1 => (64.0_f32, 128.0_f32, 12usize),
            _ => (192.0_f32, 128.0_f32, 18usize),
        };
        let frame = ((progress * frame_count as f32).floor() as usize).min(frame_count - 1);
        let u0 = ox / WIND_ATLAS_W as f32;
        let u1 = (ox + fw) / WIND_ATLAS_W as f32;
        let v0 = (frame as f32 * 32.0) / WIND_ATLAS_H as f32;
        let v1 = ((frame + 1) as f32 * 32.0) / WIND_ATLAS_H as f32;

        // Orient ribbon along its velocity vector and upright so it reads cleanly from any angle
        let fwd = s.velocity.normalize_or_zero();
        let half_w = fwd * (s.width * 0.5);
        let half_h = Vec3::Y * (s.height * 0.5);

        let p0 = s.position - half_w - half_h;
        let p1 = s.position + half_w - half_h;
        let p2 = s.position + half_w + half_h;
        let p3 = s.position - half_w + half_h;

        let fade = smooth_in_out_alpha(s.age, s.max_age, 0.25, 0.35);
        let col = [day_light, day_light, day_light * 1.02, s.alpha * fade];

        positions.extend_from_slice(&[p0.to_array(), p1.to_array(), p2.to_array(), p3.to_array()]);
        normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
        uvs.extend_from_slice(&[[u0, v1], [u1, v1], [u1, v0], [u0, v0]]);
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

