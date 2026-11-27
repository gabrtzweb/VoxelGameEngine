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

use crate::{
    environment::{DayPhase, EnvironmentState},
    generation::BiomeType,
    player::{PlayerCamera, PlayerEnvironmentStatus},
    world::{Voxel, storage::VoxelWorld},
};

const MAX_AMBIENT_FISH: usize = 84;
const FISH_SHEET_FILES: [&str; 6] = [
    "assets/textures/particles/ambience_shallow_fish.png",
    "assets/textures/particles/ambience_shallow_fish1.png",
    "assets/textures/particles/ambience_shallow_fish2.png",
    "assets/textures/particles/ambience_deep_fish.png",
    "assets/textures/particles/ambience_deep_fish1.png",
    "assets/textures/particles/ambience_deep_fish2.png",
];
const FISH_ATLAS_W: u32 = 96; // 6 sheets * 16px
const FISH_ATLAS_H: u32 = 32; // 4 rows * 8px

#[derive(Component)]
pub struct AmbientFishBatchMesh;

#[derive(Clone, Copy, Default)]
struct AmbientFish {
    position: Vec3,
    heading: f32,
    speed: f32,
    vertical_vel: f32,
    size: f32,
    sheet_idx: usize, // 0..3 shallow, 3..6 deep
    cell_idx: usize,  // 0..8 (0,1,3,4 = side-view; 2,5,6,7 = top-down)
    phase: f32,
    turn_rate: f32,
    age: f32,
    max_age: f32,
    alive: bool,
}

#[derive(Resource)]
pub struct AmbientFishPool {
    fish: Vec<AmbientFish>,
    mesh_handle: Handle<Mesh>,
    rng: u64,
}

impl AmbientFishPool {
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

    /// Returns the first dead slot so living fish are never abruptly overwritten mid-swim.
    fn alloc_slot(&self) -> Option<usize> {
        self.fish.iter().position(|f| !f.alive)
    }
}

/// Cells (0,0), (1,0), (1,1), (0,2) -> indices 0, 1, 3, 4 are side-profile fish;
/// indices 2, 5, 6, 7 are top-down minnows.
#[inline]
fn is_top_down_cell(cell_idx: usize) -> bool {
    matches!(cell_idx, 2 | 5 | 6 | 7)
}

pub fn setup_ambient_fish(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut atlas = vec![0u8; (FISH_ATLAS_W * FISH_ATLAS_H * 4) as usize];
    for (sheet_idx, path) in FISH_SHEET_FILES.iter().enumerate() {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (iw, ih) = rgba.dimensions();
            let copy_w = iw.min(16);
            let copy_h = ih.min(32);
            let ox = sheet_idx as u32 * 16;
            for py in 0..copy_h {
                for px in 0..copy_w {
                    let p = rgba.get_pixel(px, py);
                    let dst = (((py * FISH_ATLAS_W) + (ox + px)) * 4) as usize;
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
            width: FISH_ATLAS_W,
            height: FISH_ATLAS_H,
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

    // AlphaMode::Mask(0.5) renders fish in the OPAQUE pass BEFORE the translucent water surface,
    // so fish ALWAYS appear physically submerged underneath waves, reflections, and underwater fog!
    let mat_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(tex_handle),
        unlit: true,
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    commands.insert_resource(AmbientFishPool {
        fish: vec![AmbientFish::default(); MAX_AMBIENT_FISH],
        mesh_handle: mesh_handle.clone(),
        rng: 0xF158_A7B3_91C4_2026,
    });

    commands.spawn((
        AmbientFishBatchMesh,
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
pub fn update_ambient_fish(
    time: Res<Time>,
    mut pool: ResMut<AmbientFishPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_world: Option<Res<VoxelWorld>>,
    env_state: Option<Res<EnvironmentState>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut batch_query: Query<&mut Visibility, With<AmbientFishBatchMesh>>,
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

    // Throttle school spawn scans to ~8 Hz; spawn in a mid-to-far donut ring (8m .. 28m)
    // so schools populate the distance ahead instead of crowding right in front of the player.
    *scan_accum += dt;
    if *scan_accum >= 0.125 {
        *scan_accum = 0.0;
        let active_count = pool.fish.iter().filter(|f| f.alive).count();
        if active_count < MAX_AMBIENT_FISH - 4 {
            let is_deep_biome = matches!(
                status.current_biome,
                BiomeType::OceanTrench | BiomeType::ColdOcean
            );
            let attempts = if status.is_camera_in_water { 10 } else { 5 };
            let base_y = cam_pos.y.floor() as i32;

            for _ in 0..attempts {
                let angle = pool.next_f32() * TAU;
                // Bias radius outward (sqrt distribution in [8.0 .. 28.0]) for richer mid/far water life
                let r_t = pool.next_f32().sqrt();
                let dist = 8.0 + r_t * 20.0;
                let wx = (cam_pos.x + angle.cos() * dist).floor() as i32;
                let wz = (cam_pos.z + angle.sin() * dist).floor() as i32;
                let wy = base_y + (pool.next_u32() % 15) as i32 - 9;
                let iv = IVec3::new(wx, wy, wz);
                if !world.get_voxel(iv).is_some_and(Voxel::is_water) {
                    continue;
                }
                // Ensure open water room (not trapped in a 1-block puddle)
                if !world.get_voxel(iv + IVec3::X).is_some_and(Voxel::is_water)
                    && !world.get_voxel(iv - IVec3::X).is_some_and(Voxel::is_water)
                {
                    continue;
                }

                let use_deep = wy <= 2 || (is_deep_biome && wy <= 8);
                let sheet_idx = if use_deep {
                    3 + (pool.next_u32() as usize % 3)
                } else {
                    pool.next_u32() as usize % 3
                };
                let cell_idx = pool.next_u32() as usize % 8;
                let school_heading = pool.next_f32() * TAU;
                let school_size = 3 + (pool.next_u32() as usize % 4); // 3..6 fish per school
                let base_speed = 0.70 + pool.next_f32() * 0.65;

                for _ in 0..school_size {
                    let Some(slot) = pool.alloc_slot() else {
                        break;
                    };
                    let ox = (pool.next_f32() - 0.5) * 1.8;
                    let oy = (pool.next_f32() - 0.5) * 0.8;
                    let oz = (pool.next_f32() - 0.5) * 1.8;
                    let spawn_pos = Vec3::new(
                        wx as f32 + 0.5 + ox,
                        wy as f32 + 0.45 + oy,
                        wz as f32 + 0.5 + oz,
                    );
                    let spawn_iv = spawn_pos.floor().as_ivec3();
                    if !world.get_voxel(spawn_iv).is_some_and(Voxel::is_water) {
                        continue;
                    }
                    pool.fish[slot] = AmbientFish {
                        position: spawn_pos,
                        heading: school_heading + (pool.next_f32() - 0.5) * 0.35,
                        speed: base_speed * (0.88 + pool.next_f32() * 0.24),
                        vertical_vel: (pool.next_f32() - 0.5) * 0.12,
                        size: 0.28 + pool.next_f32() * 0.16,
                        sheet_idx,
                        cell_idx,
                        phase: pool.next_f32() * TAU,
                        turn_rate: (pool.next_f32() - 0.5) * 0.38,
                        age: 0.0,
                        max_age: 22.0 + pool.next_f32() * 16.0,
                        alive: true,
                    };
                }
                break;
            }
        }
    }

    if !pool.fish.iter().any(|f| f.alive) {
        if *batch_vis != Visibility::Hidden {
            *batch_vis = Visibility::Hidden;
        }
        return;
    }

    let day_light = env_state
        .as_ref()
        .map_or(1.0, |e| match DayPhase::from_time(e.time_of_day) {
            DayPhase::Night => 0.30,
            DayPhase::Evening | DayPhase::Morning => 0.78,
            DayPhase::Noon => 1.0,
        });

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut v_cursor = 0u32;

    for f in pool.fish.iter_mut() {
        if !f.alive {
            continue;
        }
        f.age += dt;
        if f.age >= f.max_age {
            f.alive = false;
            continue;
        }
        let to_fish = f.position - cam_pos;
        let dist_sq = to_fish.length_squared();
        if dist_sq > 34.0 * 34.0 {
            f.alive = false;
            continue;
        }
        let dist = dist_sq.sqrt();

        // Shy behavior: when player swims close (< 6.0m), fish gently dart away into the distance!
        let mut speed_mult = 1.0;
        let mut dive_bias = 0.0;
        if dist < 6.0 && dist > 0.05 {
            let away_angle = to_fish.z.atan2(to_fish.x);
            let mut diff = away_angle - f.heading;
            while diff > std::f32::consts::PI {
                diff -= TAU;
            }
            while diff < -std::f32::consts::PI {
                diff += TAU;
            }
            f.heading += diff * (4.5 * dt).min(1.0);
            speed_mult = 1.85;
        }

        // Natural end-of-life exit: in the final 4 seconds, gently dive toward seabed/aquatic flora
        // or cruise away into the distance, scaling down smoothly so fish never blink out abruptly.
        let time_left = f.max_age - f.age;
        if time_left < 4.0 {
            dive_bias = -0.35;
            speed_mult *= 1.25;
        }

        f.heading += (f.turn_rate + (f.age * 0.85 + f.phase).sin() * 0.28) * dt;
        let swim_dir = Vec3::new(f.heading.cos(), 0.0, f.heading.sin());
        let bob_y = (f.age * 2.2 + f.phase).sin() * 0.07 + f.vertical_vel + dive_bias;
        let next_pos =
            f.position + Vec3::new(swim_dir.x * f.speed * speed_mult, bob_y, swim_dir.z * f.speed * speed_mult) * dt;

        let next_iv = next_pos.floor().as_ivec3();
        let next_vox = world.get_voxel(next_iv);
        let is_aquatic_cover = next_vox.is_some_and(|v| v.is_marine_flora() || v.is_coral_fan());
        let next_in_water = (next_vox.is_some_and(Voxel::is_water) || is_aquatic_cover)
            && !(next_pos.y - next_iv.y as f32 > 0.72
                && !world
                    .get_voxel(next_iv + IVec3::Y)
                    .is_some_and(Voxel::is_water));

        if next_in_water {
            f.position = next_pos;
            // If diving into aquatic flora near end-of-life, retire smoothly inside the plants
            if time_left < 3.5 && is_aquatic_cover {
                f.age = f.max_age;
                continue;
            }
        } else {
            f.heading += 2.15;
            f.vertical_vel = -f.vertical_vel.signum() * 0.12;
        }

        // Smooth scale-in (first 1.2s) and scale-out (last 2.5s or far edge > 28m)
        // Works cleanly with AlphaMode::Mask(0.5) so fish never pop in/out!
        let enter_scale = (f.age / 1.2).clamp(0.0, 1.0);
        let exit_scale = (time_left / 2.5).clamp(0.0, 1.0);
        let far_scale = (1.0 - (dist - 28.0).max(0.0) / 6.0).clamp(0.0, 1.0);
        let vis_scale = enter_scale * exit_scale * far_scale;
        if vis_scale <= 0.04 {
            continue;
        }

        // Physical underwater color absorption: tint fish into the surrounding water palette
        // (muted cyan-teal in shallows -> deeper blue-slate at depth) so they look genuinely submerged, never "dry"!
        let depth_t = ((12.0 - f.position.y) / 34.0).clamp(0.0, 1.0);
        let light = day_light * (0.76 - depth_t * 0.44).max(0.24);
        let r_absorb = light * (0.58 - depth_t * 0.28);
        let g_absorb = light * (0.84 - depth_t * 0.16);
        let b_absorb = light * (0.94 - depth_t * 0.06);
        let col = [r_absorb, g_absorb, b_absorb, 1.0];

        // Orient each sprite authentically: top-down minnows lie flat in the water plane,
        // while side-profile fish swim upright along their heading with gentle tail-wag!
        let wag = (f.age * 11.0 + f.phase).sin() * 0.09;
        let side_perp = Vec3::new(-swim_dir.z, 0.0, swim_dir.x);
        let forward = (swim_dir + side_perp * wag).normalize_or(Vec3::X);
        let cur_size = f.size * vis_scale;
        let half_w = forward * (cur_size * 0.5);
        let half_h = if is_top_down_cell(f.cell_idx) {
            // Flat horizontal plane with a tiny tilt so visible from both above and underwater
            (side_perp * 0.92 + Vec3::Y * 0.38).normalize_or(side_perp) * (cur_size * 0.5)
        } else {
            Vec3::Y * (cur_size * 0.5)
        };

        let p0 = f.position - half_w - half_h;
        let p1 = f.position + half_w - half_h;
        let p2 = f.position + half_w + half_h;
        let p3 = f.position - half_w + half_h;

        let col_idx = (f.cell_idx % 2) as f32;
        let row_idx = (f.cell_idx / 2) as f32;
        let u0 = (f.sheet_idx as f32 * 16.0 + col_idx * 8.0) / FISH_ATLAS_W as f32;
        let u1 = u0 + 8.0 / FISH_ATLAS_W as f32;
        let v0 = (row_idx * 8.0) / FISH_ATLAS_H as f32;
        let v1 = v0 + 8.0 / FISH_ATLAS_H as f32;

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
