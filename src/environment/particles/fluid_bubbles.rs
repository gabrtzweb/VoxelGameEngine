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

use crate::{
    environment::{DayPhase, EnvironmentState},
    player::PlayerCamera,
    world::{Voxel, storage::VoxelWorld},
};

// =============================================================================
// SURFACE FLUID BUBBLES (1-draw-call 10-frame animated grayscale fluid_bubble.png)
// Active on Lava, CoreMagma, Molten, Tar, and Acid surfaces
// =============================================================================

const FLUID_BUBBLE_FRAMES: usize = 10;
const FLUID_BUBBLE_VARIANTS: usize = 3;
const FLUID_BUBBLE_ATLAS_W: usize = 16 * FLUID_BUBBLE_VARIANTS; // 48px
const FLUID_BUBBLE_ATLAS_H: usize = 16 * FLUID_BUBBLE_FRAMES; // 160px
const MAX_FLUID_BUBBLES: usize = 72;

const FLUID_BUBBLE_FILES: [&str; FLUID_BUBBLE_VARIANTS] = [
    "assets/textures/particles/fluid_bubble.png",
    "assets/textures/particles/fluid_bubble1.png",
    "assets/textures/particles/fluid_bubble2.png",
];

#[derive(Component)]
pub struct FluidBubbleBatchMesh;

#[derive(Clone, Copy, Default)]
struct FluidBubble {
    position: Vec3,
    size: f32,
    variant: usize,
    color: [f32; 3],
    is_emissive: bool,
    age: f32,
    max_age: f32,
    alive: bool,
}

#[derive(Resource)]
pub struct FluidBubblePool {
    bubbles: Vec<FluidBubble>,
    cursor: usize,
    mesh_handle: Handle<Mesh>,
    rng: u64,
}

impl FluidBubblePool {
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

pub fn setup_fluid_bubbles(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut atlas_bytes = vec![0u8; FLUID_BUBBLE_ATLAS_W * FLUID_BUBBLE_ATLAS_H * 4];
    for (v_idx, path) in FLUID_BUBBLE_FILES.iter().enumerate() {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (iw, ih) = rgba.dimensions();
            let cw = (iw as usize).min(16);
            let ch = (ih as usize).min(FLUID_BUBBLE_ATLAS_H);
            let ox = v_idx * 16;
            for py in 0..ch {
                for px in 0..cw {
                    let src = rgba.get_pixel(px as u32, py as u32).0;
                    let dst = (py * FLUID_BUBBLE_ATLAS_W + (ox + px)) * 4;
                    atlas_bytes[dst..dst + 4].copy_from_slice(&src);
                }
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: FLUID_BUBBLE_ATLAS_W as u32,
            height: FLUID_BUBBLE_ATLAS_H as u32,
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
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    commands.spawn((
        FluidBubbleBatchMesh,
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(mat_handle),
        Transform::IDENTITY,
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
        bevy::camera::visibility::NoFrustumCulling,
    ));

    commands.insert_resource(FluidBubblePool {
        bubbles: vec![FluidBubble::default(); MAX_FLUID_BUBBLES],
        cursor: 0,
        mesh_handle,
        rng: 0xB0BB_1E55_A1B2_C3D4,
    });
}

#[allow(clippy::too_many_arguments)]
pub fn update_fluid_bubbles(
    time: Res<Time>,
    mut pool: ResMut<FluidBubblePool>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_world: Option<Res<VoxelWorld>>,
    env_state: Option<Res<EnvironmentState>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut batch_query: Query<&mut Visibility, With<FluidBubbleBatchMesh>>,
    mut scan_accum: Local<f32>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok(cam_tf) = camera_query.single() else {
        return;
    };
    let Ok(mut batch_vis) = batch_query.single_mut() else {
        return;
    };

    // Throttle fluid surface voxel scan to ~10 Hz (avoids 18 voxel lookups every single frame at 120+ FPS)
    *scan_accum += dt;
    if *scan_accum >= 0.10 {
        *scan_accum = 0.0;
        if let Some(ref world) = voxel_world {
            let cam_pos = cam_tf.translation;
            let base_y = cam_pos.y.floor() as i32;
            for _ in 0..18 {
                let wx = (cam_pos.x + (pool.next_f32() - 0.5) * 34.0).floor() as i32;
                let wz = (cam_pos.z + (pool.next_f32() - 0.5) * 34.0).floor() as i32;
                let wy = base_y + (pool.next_u32() % 17) as i32 - 8;
                let pos = IVec3::new(wx, wy, wz);
                let Some(v) = world.get_voxel(pos) else {
                    continue;
                };
                let bubble_style = match v {
                    Voxel::LiquidLava | Voxel::LiquidCoreMagma | Voxel::LiquidMolten => {
                        Some(([1.65, 0.58, 0.12], true))
                    }
                    Voxel::LiquidTar => Some(([0.28, 0.10, 0.42], false)),
                    Voxel::LiquidAcid => Some(([0.34, 1.15, 0.20], true)),
                    _ => None,
                };
                let Some((color, is_emissive)) = bubble_style else {
                    continue;
                };
                // Only bubble on exposed liquid surfaces (Air directly above)
                if world.get_voxel(pos + IVec3::Y) != Some(Voxel::Air) {
                    continue;
                }
                let size = 0.26 + pool.next_f32() * 0.20;
                let max_age = 0.75 + pool.next_f32() * 0.45;
                let variant = pool.next_u32() as usize % FLUID_BUBBLE_VARIANTS;
                let ox = 0.15 + pool.next_f32() * 0.70;
                let oz = 0.15 + pool.next_f32() * 0.70;
                let slot = pool.cursor;
                pool.cursor = (pool.cursor + 1) % MAX_FLUID_BUBBLES;
                pool.bubbles[slot] = FluidBubble {
                    position: Vec3::new(wx as f32 + ox, wy as f32 + 0.91 + size * 0.45, wz as f32 + oz),
                    size,
                    variant,
                    color,
                    is_emissive,
                    age: 0.0,
                    max_age,
                    alive: true,
                };
            }
        }
    }

    if !pool.bubbles.iter().any(|b| b.alive) {
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

    let right = cam_tf.right().as_vec3();
    let up = Vec3::Y;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut v_cursor = 0u32;

    for b in pool.bubbles.iter_mut() {
        if !b.alive {
            continue;
        }
        b.age += dt;
        if b.age >= b.max_age {
            b.alive = false;
            continue;
        }
        let progress = (b.age / b.max_age).clamp(0.0, 0.999);
        let frame = ((progress * FLUID_BUBBLE_FRAMES as f32).floor() as usize)
            .min(FLUID_BUBBLE_FRAMES - 1);
        let u0 = b.variant as f32 / FLUID_BUBBLE_VARIANTS as f32;
        let u1 = (b.variant + 1) as f32 / FLUID_BUBBLE_VARIANTS as f32;
        let v0 = frame as f32 / FLUID_BUBBLE_FRAMES as f32;
        let v1 = (frame + 1) as f32 / FLUID_BUBBLE_FRAMES as f32;

        let half = b.size * 0.5;
        let r_vec = right * half;
        let u_vec = up * half;
        let p0 = b.position - r_vec - u_vec;
        let p1 = b.position + r_vec - u_vec;
        let p2 = b.position + r_vec + u_vec;
        let p3 = b.position - r_vec + u_vec;

        let ls = if b.is_emissive { 1.0 } else { day_light };
        let col = [b.color[0] * ls, b.color[1] * ls, b.color[2] * ls, 0.95];

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

