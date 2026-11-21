use std::f32::consts::{PI, TAU};

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

use super::config::GrassConfig;
use crate::{
    environment::{EnvironmentState, weather::wind::WindState},
    player::PlayerCamera,
    world::{Voxel, storage::VoxelWorld},
};

#[derive(Component)]
pub struct WindBladeParticle {
    pub velocity: Vec3,
    pub lifetime_timer: Timer,
    pub roll_speed: f32,
    pub initial_scale: f32,
}

#[derive(Resource, Default)]
pub struct WindBladeParticleTimer {
    pub timer: Timer,
}

pub fn spawn_wind_blade_particles(
    mut commands: Commands,
    time: Res<Time>,
    wind: Option<Res<WindState>>,
    config: Option<Res<GrassConfig>>,
    mut timer: Local<f32>,
    asset_server: Res<AssetServer>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
) {
    let default_cfg = GrassConfig::default();
    let cfg = config.as_ref().map(|c| &**c).unwrap_or(&default_cfg);

    if !cfg.enabled || !cfg.blade_particles {
        return;
    }

    let Some(w) = wind.as_ref() else {
        return;
    };

    // Only shed grass blades if wind is sufficiently strong (>= 4.5 m/s)
    let speed = w.current_speed;
    if speed < 4.5 {
        return;
    }

    *timer += time.delta_secs();
    let spawn_interval = (1.5 / (speed / 4.0)).clamp(0.15, 1.2);
    if *timer < spawn_interval {
        return;
    }
    *timer = 0.0;

    let Some(cam_transform) = camera_query.iter().next() else {
        return;
    };

    let cam_pos = cam_transform.translation;
    let wind_norm = w.current_wind.normalize_or_zero();

    // Spawn 1-3 particles in a semi-circle upwind of the camera
    let spawn_count = ((speed / 6.0).ceil() as usize).clamp(1, 3);
    let texture = asset_server.load("textures/foliage/wind_blade.png");

    for i in 0..spawn_count {
        let angle_jitter = ((i as f32) - 1.0) * 0.4 + (time.elapsed_secs() * 3.7 + i as f32).sin() * 0.3;
        let c = angle_jitter.cos();
        let s = angle_jitter.sin();
        let offset_dir = Vec3::new(
            -wind_norm.x * c - -wind_norm.z * s,
            0.0,
            -wind_norm.x * s + -wind_norm.z * c,
        );

        let dist = 8.0 + (i as f32 * 4.0);
        let spawn_pos = cam_pos + offset_dir * dist + Vec3::new(0.0, 1.5 + (i as f32 * 0.8), 0.0);

        let particle_vel = w.current_wind * 0.85 + Vec3::new(0.0, -0.4, 0.0);

        commands.spawn((
            Sprite {
                image: texture.clone(),
                custom_size: Some(Vec2::splat(0.25)),
                color: Color::srgba(0.85, 0.95, 0.8, 0.9),
                ..default()
            },
            Transform::from_translation(spawn_pos).with_rotation(Quat::from_rotation_z(raw_angle(i))),
            WindBladeParticle {
                velocity: particle_vel,
                lifetime_timer: Timer::from_seconds(2.5, TimerMode::Once),
                roll_speed: 2.5 + (i as f32 * 1.2),
                initial_scale: 0.25,
            },
        ));
    }
}

fn raw_angle(idx: usize) -> f32 {
    (idx as f32 * 1.5) % (2.0 * PI)
}

pub fn update_wind_blade_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut WindBladeParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime_timer.tick(time.delta());

        if particle.lifetime_timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += particle.velocity * dt;
        transform.rotate_z(particle.roll_speed * dt);

        let progress = particle.lifetime_timer.fraction();
        let alpha = if progress < 0.2 {
            progress / 0.2
        } else if progress > 0.7 {
            1.0 - (progress - 0.7) / 0.3
        } else {
            1.0
        };

        sprite.color.set_alpha(alpha * 0.9);
    }
}

// =========================================================================
// Minecraft-Style Falling Canopy Leaf Particles (Batched Single Draw Call)
// =========================================================================

const MAX_FALLING_LEAVES: usize = 768;
const LEAF_ATLAS_W: usize = 64;
const LEAF_ATLAS_H: usize = 32;
const LEAF_CELL: usize = 8;

const LEAF_PARTICLE_FILES: [&str; 21] = [
    // 0..3: Cherry (colored pink)
    "assets/textures/particles/cherry_leaf.png",
    "assets/textures/particles/cherry_leaf1.png",
    "assets/textures/particles/cherry_leaf2.png",
    // 3..9: Common deciduous (grayscale -> biome foliage tinted)
    "assets/textures/particles/common_leaf.png",
    "assets/textures/particles/common_leaf1.png",
    "assets/textures/particles/common_leaf2.png",
    "assets/textures/particles/common_leaf3.png",
    "assets/textures/particles/common_leaf4.png",
    "assets/textures/particles/common_leaf5.png",
    // 9..12: Conifer (colored evergreen)
    "assets/textures/particles/conifer_leaf.png",
    "assets/textures/particles/conifer_leaf1.png",
    "assets/textures/particles/conifer_leaf2.png",
    // 12..15: Maple Orange (colored)
    "assets/textures/particles/maple_leaf_orange.png",
    "assets/textures/particles/maple_leaf_orange1.png",
    "assets/textures/particles/maple_leaf_orange2.png",
    // 15..18: Maple Red (colored)
    "assets/textures/particles/maple_leaf_red.png",
    "assets/textures/particles/maple_leaf_red1.png",
    "assets/textures/particles/maple_leaf_red2.png",
    // 18..21: Maple Yellow (colored)
    "assets/textures/particles/maple_leaf_yellow.png",
    "assets/textures/particles/maple_leaf_yellow1.png",
    "assets/textures/particles/maple_leaf_yellow2.png",
];

#[derive(Component)]
pub struct FallingLeafBatchMesh;

#[derive(Clone, Copy)]
struct FallingLeaf {
    position: Vec3,
    fall_speed: f32,
    flutter_phase: f32,
    flutter_freq: f32,
    flutter_amp: f32,
    rotation: f32,
    spin: f32,
    size: f32,
    color: [f32; 3],
    uv_rect: [f32; 4],
    age: f32,
    max_age: f32,
    landed: bool,
    landed_age: f32,
    alive: bool,
}

impl Default for FallingLeaf {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            fall_speed: 0.7,
            flutter_phase: 0.0,
            flutter_freq: 2.5,
            flutter_amp: 0.45,
            rotation: 0.0,
            spin: 1.5,
            size: 0.14,
            color: [1.0, 1.0, 1.0],
            uv_rect: [0.0, 0.0, 0.1, 0.1],
            age: 0.0,
            max_age: 10.0,
            landed: false,
            landed_age: 0.0,
            alive: false,
        }
    }
}

#[derive(Resource)]
pub struct FallingLeafPool {
    leaves: Vec<FallingLeaf>,
    cursor: usize,
    uv_rects: [[f32; 4]; 21],
    mesh_handle: Handle<Mesh>,
    rng: u64,
}

impl FallingLeafPool {
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

/// Builds a crisp nearest-neighbor atlas from all 21 leaf particle PNGs and spawns
/// the single batched mesh entity used to render falling canopy leaves.
pub fn setup_falling_leaf_particles(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut atlas_bytes = vec![0u8; LEAF_ATLAS_W * LEAF_ATLAS_H * 4];
    let mut uv_rects = [[0.0f32; 4]; 21];

    for (idx, path) in LEAF_PARTICLE_FILES.iter().enumerate() {
        let col = idx % (LEAF_ATLAS_W / LEAF_CELL);
        let row = idx / (LEAF_ATLAS_W / LEAF_CELL);
        let ox = col * LEAF_CELL;
        let oy = row * LEAF_CELL;

        let (w, h) = if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (iw, ih) = rgba.dimensions();
            let cw = (iw as usize).min(LEAF_CELL);
            let ch = (ih as usize).min(LEAF_CELL);
            for py in 0..ch {
                for px in 0..cw {
                    let src = rgba.get_pixel(px as u32, py as u32).0;
                    let dst_idx = ((oy + py) * LEAF_ATLAS_W + (ox + px)) * 4;
                    atlas_bytes[dst_idx..dst_idx + 4].copy_from_slice(&src);
                }
            }
            (cw, ch)
        } else {
            (5, 5)
        };

        uv_rects[idx] = [
            ox as f32 / LEAF_ATLAS_W as f32,
            oy as f32 / LEAF_ATLAS_H as f32,
            (ox + w) as f32 / LEAF_ATLAS_W as f32,
            (oy + h) as f32 / LEAF_ATLAS_H as f32,
        ];
    }

    let mut atlas_image = Image::new(
        Extent3d {
            width: LEAF_ATLAS_W as u32,
            height: LEAF_ATLAS_H as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    atlas_image.sampler = ImageSampler::nearest();
    let atlas_handle = images.add(atlas_image);

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

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_handle),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    commands.spawn((
        FallingLeafBatchMesh,
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material_handle),
        NotShadowCaster,
        NotShadowReceiver,
        Transform::default(),
        Visibility::Hidden,
    ));

    commands.insert_resource(FallingLeafPool {
        leaves: vec![FallingLeaf::default(); MAX_FALLING_LEAVES],
        cursor: 0,
        uv_rects,
        mesh_handle,
        rng: 0x517C_C1B7_2722_0A95,
    });
}

/// Maps a canopy leaf voxel to its `(atlas_index_start, count, needs_biome_tint)` tuple.
#[inline]
fn leaf_particle_spec(voxel: Voxel) -> (usize, usize, bool) {
    match voxel {
        Voxel::CherryLeaves => (0, 3, false),
        Voxel::PineLeaves | Voxel::YewLeaves | Voxel::RedwoodLeaves => (9, 3, true),
        Voxel::MapleLeavesOrange => (12, 3, false),
        Voxel::MapleLeavesRed => (15, 3, false),
        Voxel::MapleLeavesYellow => (18, 3, false),
        _ => (3, 6, true), // Grayscale common_leaf{,1..5} -> biome foliage tinted
    }
}

/// Samples nearby exposed canopy leaf blocks and simulates gently fluttering falling leaves.
pub fn update_falling_leaf_particles(
    time: Res<Time>,
    voxel_world: Res<VoxelWorld>,
    wind: Option<Res<WindState>>,
    env_state: Option<Res<EnvironmentState>>,
    mut pool: Option<ResMut<FallingLeafPool>>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut batch_query: Query<&mut Visibility, With<FallingLeafBatchMesh>>,
) {
    let Some(ref mut pool) = pool else {
        return;
    };
    let Ok(cam_transform) = camera_query.single() else {
        return;
    };
    let Ok(mut batch_vis) = batch_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs().min(0.1);
    let cam_pos = cam_transform.translation;
    let wind_vec = wind.as_ref().map_or(Vec3::ZERO, |w| w.current_wind);

    // Subtle day/night brightness modulation so falling leaves blend naturally at dusk/night
    let day_light = env_state.as_ref().map_or(1.0, |env| {
        let t = env.time_of_day;
        let sun = ((t - 0.25) * TAU).sin().clamp(-1.0, 1.0);
        0.28 + 0.72 * ((sun + 0.25) / 1.25).clamp(0.0, 1.0)
    });

    // 1. Sample random blocks in a wide radius around the player (64x64x24, ±32m horizontal)
    let probes = ((dt * 950.0).round() as usize).clamp(3, 32);
    for _ in 0..probes {
        let ox = ((pool.next_f32() - 0.5) * 64.0).round() as i32;
        let oz = ((pool.next_f32() - 0.5) * 64.0).round() as i32;
        let oy = (-4.0 + pool.next_f32() * 24.0).round() as i32;
        let pos = IVec3::new(
            cam_pos.x.floor() as i32 + ox,
            cam_pos.y.floor() as i32 + oy,
            cam_pos.z.floor() as i32 + oz,
        );

        let Some(voxel) = voxel_world.get_voxel(pos) else {
            continue;
        };
        if !voxel.is_leaves() {
            continue;
        }
        // Only shed from exposed undersides (Air directly below)
        if voxel_world.get_voxel(pos - IVec3::Y) != Some(Voxel::Air) {
            continue;
        }

        // Calm, natural forest leaf shedding (Cherry & Maple slightly more expressive)
        let drop_chance = match voxel {
            Voxel::CherryLeaves
            | Voxel::MapleLeavesOrange
            | Voxel::MapleLeavesRed
            | Voxel::MapleLeavesYellow => 0.36,
            _ => 0.22,
        };
        if pool.next_f32() >= drop_chance {
            continue;
        }

        let (start_idx, count, needs_tint) = leaf_particle_spec(voxel);
        let variant = start_idx + (pool.next_u32() as usize % count);
        let uv_rect = pool.uv_rects[variant];

        let color = if needs_tint {
            if voxel == Voxel::RedwoodLeaves {
                [0.35, 0.72, 0.42]
            } else {
                let t = voxel.tint_color_at(pos);
                [t[0], t[1], t[2]]
            }
        } else {
            [1.0, 1.0, 1.0]
        };

        let spawn_pos = Vec3::new(
            pos.x as f32 + 0.1 + pool.next_f32() * 0.8,
            pos.y as f32 - 0.04,
            pos.z as f32 + 0.1 + pool.next_f32() * 0.8,
        );

        let fall_speed = 0.65 + pool.next_f32() * 0.40;
        let flutter_phase = pool.next_f32() * TAU;
        let flutter_freq = 1.8 + pool.next_f32() * 1.8;
        let flutter_amp = 0.35 + pool.next_f32() * 0.45;
        let rotation = pool.next_f32() * TAU;
        let spin = (pool.next_f32() - 0.5) * 3.8;
        let size = 0.24 + pool.next_f32() * 0.10;
        let max_age = 24.0 + pool.next_f32() * 6.0;

        let slot = pool.cursor;
        pool.cursor = (pool.cursor + 1) % MAX_FALLING_LEAVES;
        pool.leaves[slot] = FallingLeaf {
            position: spawn_pos,
            fall_speed,
            flutter_phase,
            flutter_freq,
            flutter_amp,
            rotation,
            spin,
            size,
            color,
            uv_rect,
            age: 0.0,
            max_age,
            landed: false,
            landed_age: 0.0,
            alive: true,
        };
    }

    // 2. Simulate all active falling leaves and build batched camera-facing quads
    let cam_right = *cam_transform.right();
    let cam_up = *cam_transform.up();

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut cursor_v = 0u32;

    for leaf in pool.leaves.iter_mut() {
        if !leaf.alive {
            continue;
        }

        leaf.age += dt;
        if leaf.age >= leaf.max_age {
            leaf.alive = false;
            continue;
        }

        if leaf.landed {
            leaf.landed_age += dt;
            if leaf.landed_age >= 4.0 {
                leaf.alive = false;
                continue;
            }
        } else {
            let flutter = (leaf.age * leaf.flutter_freq + leaf.flutter_phase).sin() * leaf.flutter_amp;
            let flutter_z =
                (leaf.age * (leaf.flutter_freq * 0.83) + leaf.flutter_phase + 1.3).cos()
                    * (leaf.flutter_amp * 0.7);
            let next_pos = leaf.position
                + Vec3::new(
                    (flutter + wind_vec.x * 0.25) * dt,
                    -leaf.fall_speed * dt,
                    (flutter_z + wind_vec.z * 0.25) * dt,
                );

            let check_voxel = IVec3::new(
                next_pos.x.floor() as i32,
                (next_pos.y - 0.04).floor() as i32,
                next_pos.z.floor() as i32,
            );
            if let Some(v) = voxel_world.get_voxel(check_voxel)
                && (v.is_collidable() || v.is_water())
            {
                leaf.landed = true;
                leaf.landed_age = 0.0;
                leaf.position = Vec3::new(next_pos.x, check_voxel.y as f32 + 1.03, next_pos.z);
            } else {
                leaf.position = next_pos;
                leaf.rotation += leaf.spin * dt;
            }
        }

        let alpha = if leaf.landed {
            // Stay clearly visible on the ground for 2.5s, then fade out smoothly over 1.5s
            if leaf.landed_age < 2.5 {
                1.0
            } else {
                let t = (1.0 - (leaf.landed_age - 2.5) / 1.5).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            }
        } else if leaf.age < 0.35 {
            leaf.age / 0.35
        } else if leaf.age > leaf.max_age - 1.5 {
            ((leaf.max_age - leaf.age) / 1.5).clamp(0.0, 1.0)
        } else {
            1.0
        };

        if alpha <= 0.01 {
            continue;
        }

        let half = leaf.size * 0.5;
        let (sin_r, cos_r) = leaf.rotation.sin_cos();
        let (c0, c1, c2, c3) = if leaf.landed {
            // Settle flat on the ground when landed
            let rx = Vec3::new(cos_r, 0.0, sin_r) * half;
            let rz = Vec3::new(-sin_r, 0.0, cos_r) * half;
            (
                leaf.position - rx - rz,
                leaf.position + rx - rz,
                leaf.position + rx + rz,
                leaf.position - rx + rz,
            )
        } else {
            let rot_r = (cam_right * cos_r + cam_up * sin_r) * half;
            let rot_u = (-cam_right * sin_r + cam_up * cos_r) * half;
            (
                leaf.position - rot_r - rot_u,
                leaf.position + rot_r - rot_u,
                leaf.position + rot_r + rot_u,
                leaf.position - rot_r + rot_u,
            )
        };

        let col = [
            leaf.color[0] * day_light,
            leaf.color[1] * day_light,
            leaf.color[2] * day_light,
            alpha,
        ];
        let uv = leaf.uv_rect;

        positions.extend_from_slice(&[c0.to_array(), c1.to_array(), c2.to_array(), c3.to_array()]);
        normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
        uvs.extend_from_slice(&[
            [uv[0], uv[3]],
            [uv[2], uv[3]],
            [uv[2], uv[1]],
            [uv[0], uv[1]],
        ]);
        colors.extend_from_slice(&[col; 4]);
        indices.extend_from_slice(&[
            cursor_v,
            cursor_v + 1,
            cursor_v + 2,
            cursor_v,
            cursor_v + 2,
            cursor_v + 3,
        ]);
        cursor_v += 4;
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
