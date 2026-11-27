use std::f32::consts::TAU;

use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
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
    generation::BiomeType,
    player::{PlayerCamera, PlayerEnvironmentStatus},
    world::{Voxel, storage::VoxelWorld},
};

const MAX_DRAGONFLIES: usize = 36;
const DRAGONFLY_FILES: [&str; 3] = [
    "assets/textures/particles/ambience_dragonfly.png",
    "assets/textures/particles/ambience_dragonfly1.png",
    "assets/textures/particles/ambience_dragonfly2.png",
];
const DRAGONFLY_ATLAS_W: u32 = 48; // 3 variants * 16px
const DRAGONFLY_ATLAS_H: u32 = 48; // 3 flap frames * 16px

#[derive(Component)]
pub struct DragonflyBatchMesh;

#[derive(Clone, Copy, Default)]
struct Dragonfly {
    position: Vec3,
    heading: f32,
    speed: f32,
    hover_timer: f32,
    dart_timer: f32,
    is_hovering: bool,
    size: f32,
    variant: usize,
    phase: f32,
    age: f32,
    max_age: f32,
    alive: bool,
}

#[derive(Resource)]
pub struct DragonflyPool {
    flies: Vec<Dragonfly>,
    cursor: usize,
    mesh_handle: Handle<Mesh>,
    rng: u64,
}

impl DragonflyPool {
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

pub fn setup_dragonflies(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut atlas = vec![0u8; (DRAGONFLY_ATLAS_W * DRAGONFLY_ATLAS_H * 4) as usize];
    for (v_idx, path) in DRAGONFLY_FILES.iter().enumerate() {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (iw, ih) = rgba.dimensions();
            let copy_w = iw.min(16);
            let copy_h = ih.min(48);
            let ox = v_idx as u32 * 16;
            for py in 0..copy_h {
                for px in 0..copy_w {
                    let p = rgba.get_pixel(px, py);
                    let dst = (((py * DRAGONFLY_ATLAS_W) + (ox + px)) * 4) as usize;
                    atlas[dst] = p[0];
                    atlas[dst + 1] = p[1];
                    atlas[dst + 2] = p[2];
                    atlas[dst + 3] = p[3];
                }
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: DRAGONFLY_ATLAS_W,
            height: DRAGONFLY_ATLAS_H,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas,
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
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    commands.insert_resource(DragonflyPool {
        flies: vec![Dragonfly::default(); MAX_DRAGONFLIES],
        cursor: 0,
        mesh_handle: mesh_handle.clone(),
        rng: 0xD8A6_0F17_5531_9921,
    });

    commands.spawn((
        DragonflyBatchMesh,
        Mesh3d(mesh_handle),
        MeshMaterial3d(mat_handle),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
        NoFrustumCulling,
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn update_dragonflies(
    time: Res<Time>,
    mut pool: ResMut<DragonflyPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_world: Option<Res<VoxelWorld>>,
    env_state: Option<Res<EnvironmentState>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    weather: Option<Res<WeatherManager>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut batch_query: Query<&mut Visibility, With<DragonflyBatchMesh>>,
    mut scan_accum: Local<f32>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok(cam_tf) = camera_query.single() else {
        return;
    };
    let Ok(mut batch_vis) = batch_query.single_mut() else {
        return;
    };
    let Some(ref world) = voxel_world else {
        return;
    };

    let cam_pos = cam_tf.translation;
    let status = env_status.as_deref().copied().unwrap_or_default();
    let is_day = env_state.as_ref().is_none_or(|e| {
        !matches!(DayPhase::from_time(e.time_of_day), DayPhase::Night)
    });
    let is_calm = weather.as_ref().is_none_or(|w| {
        w.rain_intensity < 0.25 && w.snow_intensity < 0.15 && w.dust_intensity < 0.20
    });

    // Spawn dragonflies primarily in swamps, wetlands, jungles, lakes/rivers, and lush forests
    let biome_weight: f32 = match status.current_biome {
        BiomeType::WillowMarsh
        | BiomeType::MangroveSwamp
        | BiomeType::OrchidSwamp
        | BiomeType::SwampyShore
        | BiomeType::Mudflats
        | BiomeType::JungleWetlands => 1.0,
        BiomeType::DenseRainforest
        | BiomeType::BambooGrove
        | BiomeType::VolcanicJungle
        | BiomeType::CalmLake
        | BiomeType::Rapids
        | BiomeType::Estuary
        | BiomeType::ReedyShore => 0.85,
        BiomeType::MistyWoods
        | BiomeType::VerdantWoods
        | BiomeType::AutumnForest
        | BiomeType::ShieldForest
        | BiomeType::FertileValley
        | BiomeType::FlowerMeadow
        | BiomeType::SakuraGrove
        | BiomeType::Grasslands => 0.45,
        _ => 0.0,
    };

    *scan_accum += dt;
    if *scan_accum >= 0.16 && biome_weight > 0.0 && is_day && is_calm && !status.is_underground && !status.is_camera_in_water {
        *scan_accum = 0.0;
        let active_count = pool.flies.iter().filter(|f| f.alive).count();
        let target_max = ((MAX_DRAGONFLIES as f32) * biome_weight).round() as usize;
        if active_count < target_max {
            let base_y = cam_pos.y.floor() as i32;
            for _ in 0..5 {
                let wx = (cam_pos.x + (pool.next_f32() - 0.5) * 32.0).floor() as i32;
                let wz = (cam_pos.z + (pool.next_f32() - 0.5) * 32.0).floor() as i32;
                let mut found_y = None;
                for wy in (base_y - 5..=base_y + 6).rev() {
                    let here = IVec3::new(wx, wy, wz);
                    let below = IVec3::new(wx, wy - 1, wz);
                    if world.get_voxel(here) == Some(Voxel::Air)
                        && world.get_voxel(below).is_some_and(|v| v.is_water() || v.is_collidable())
                    {
                        // Bonus preference for hovering right over water or marshy ground
                        found_y = Some(wy);
                        break;
                    }
                }
                let Some(gy) = found_y else {
                    continue;
                };
                let spawn_pos = Vec3::new(
                    wx as f32 + 0.5,
                    gy as f32 + 0.65 + pool.next_f32() * 1.85,
                    wz as f32 + 0.5,
                );
                let slot = pool.cursor;
                pool.cursor = (pool.cursor + 1) % MAX_DRAGONFLIES;
                pool.flies[slot] = Dragonfly {
                    position: spawn_pos,
                    heading: pool.next_f32() * TAU,
                    speed: 2.2 + pool.next_f32() * 1.4,
                    hover_timer: 0.4 + pool.next_f32() * 0.9,
                    dart_timer: 0.5 + pool.next_f32() * 0.8,
                    is_hovering: pool.next_f32() < 0.5,
                    size: 0.30 + pool.next_f32() * 0.12,
                    variant: pool.next_u32() as usize % 3,
                    phase: pool.next_f32() * TAU,
                    age: 0.0,
                    max_age: 9.0 + pool.next_f32() * 8.0,
                    alive: true,
                };
                break;
            }
        }
    }

    if !pool.flies.iter().any(|f| f.alive) {
        if *batch_vis != Visibility::Hidden {
            *batch_vis = Visibility::Hidden;
        }
        return;
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

    for f in pool.flies.iter_mut() {
        if !f.alive {
            continue;
        }
        f.age += dt;
        if f.age >= f.max_age {
            f.alive = false;
            continue;
        }
        let dist_sq = (f.position - cam_pos).length_squared();
        if dist_sq > 26.0 * 26.0 {
            f.alive = false;
            continue;
        }

        // Authentic dragonfly dart-and-hover motion
        if f.is_hovering {
            f.hover_timer -= dt;
            f.position.y += (f.age * 16.0 + f.phase).sin() * 0.08 * dt;
            if f.hover_timer <= 0.0 {
                f.is_hovering = false;
                f.dart_timer = 0.45 + ((f.phase * 7.1 + f.age).sin().abs()) * 0.75;
                f.heading += (f.age * 3.7 + f.phase).cos() * 1.4;
            }
        } else {
            f.dart_timer -= dt;
            let dir = Vec3::new(f.heading.cos(), 0.0, f.heading.sin());
            let bob_y = (f.age * 6.5 + f.phase).sin() * 0.35;
            let next_pos = f.position + Vec3::new(dir.x * f.speed, bob_y, dir.z * f.speed) * dt;
            let next_iv = next_pos.floor().as_ivec3();
            if world.get_voxel(next_iv) == Some(Voxel::Air) {
                f.position = next_pos;
            } else {
                f.heading += 2.35;
            }
            if f.dart_timer <= 0.0 {
                f.is_hovering = true;
                f.hover_timer = 0.55 + ((f.phase * 13.3 + f.age).sin().abs()) * 0.95;
            }
        }

        let life_alpha = smooth_in_out_alpha(f.age, f.max_age, 0.7, 1.1);
        let far_fade = (1.0 - (dist_sq.sqrt() - 16.0).max(0.0) / 10.0).clamp(0.0, 1.0);
        let alpha = life_alpha * far_fade * 0.96;
        if alpha <= 0.01 {
            continue;
        }

        // Fast 3-frame wing buzz (dragonfly wings beat rapidly ~24 Hz)
        let frame = ((f.age * 24.0 + f.phase * 3.0) as usize) % 3;
        let u0 = (f.variant as f32 * 16.0) / DRAGONFLY_ATLAS_W as f32;
        let u1 = u0 + 16.0 / DRAGONFLY_ATLAS_W as f32;
        let v0 = (frame as f32 * 16.0) / DRAGONFLY_ATLAS_H as f32;
        let v1 = v0 + 16.0 / DRAGONFLY_ATLAS_H as f32;

        // Horizontal-tilted quad oriented along flight heading (dragonfly sprite faces +Y in texture)
        let fwd = Vec3::new(f.heading.cos(), 0.0, f.heading.sin());
        let right = Vec3::new(-fwd.z, 0.0, fwd.x);
        let tilted_fwd = (fwd * 0.92 + Vec3::Y * 0.38).normalize_or(fwd);

        let half_w = right * (f.size * 0.5);
        let half_h = tilted_fwd * (f.size * 0.5);

        let p0 = f.position - half_w - half_h;
        let p1 = f.position + half_w - half_h;
        let p2 = f.position + half_w + half_h;
        let p3 = f.position - half_w + half_h;

        let col = [day_light, day_light, day_light, alpha];
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
