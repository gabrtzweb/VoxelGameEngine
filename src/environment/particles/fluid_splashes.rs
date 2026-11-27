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
    world::Voxel,
};

// =============================================================================
// 3D FLUID SPLASHES (1-draw-call batched hollow 3D prisms)
// =============================================================================

const SPLASH_3D_FRAMES: usize = 9;
const SPLASH_3D_ATLAS_W: usize = 144; // 9 frames x 16px
const SPLASH_3D_ATLAS_H: usize = 80; // Row 0 (0..32): curtain, Row 1 (32..64): foam, Row 2 (64..80): ring
const MAX_SPLASH_3D_WAVES: usize = 48;

const SPLASH_3D_CURTAIN_FILES: [&str; SPLASH_3D_FRAMES] = [
    "assets/textures/particles/fluid_splash.png",
    "assets/textures/particles/fluid_splash1.png",
    "assets/textures/particles/fluid_splash2.png",
    "assets/textures/particles/fluid_splash3.png",
    "assets/textures/particles/fluid_splash4.png",
    "assets/textures/particles/fluid_splash5.png",
    "assets/textures/particles/fluid_splash6.png",
    "assets/textures/particles/fluid_splash7.png",
    "assets/textures/particles/fluid_splash8.png",
];

const SPLASH_3D_FOAM_FILES: [&str; SPLASH_3D_FRAMES] = [
    "assets/textures/particles/fluid_splash_foam.png",
    "assets/textures/particles/fluid_splash_foam1.png",
    "assets/textures/particles/fluid_splash_foam2.png",
    "assets/textures/particles/fluid_splash_foam3.png",
    "assets/textures/particles/fluid_splash_foam4.png",
    "assets/textures/particles/fluid_splash_foam5.png",
    "assets/textures/particles/fluid_splash_foam6.png",
    "assets/textures/particles/fluid_splash_foam7.png",
    "assets/textures/particles/fluid_splash_foam8.png",
];

const SPLASH_3D_RING_FILES: [&str; SPLASH_3D_FRAMES] = [
    "assets/textures/particles/fluid_splash_ring.png",
    "assets/textures/particles/fluid_splash_ring1.png",
    "assets/textures/particles/fluid_splash_ring2.png",
    "assets/textures/particles/fluid_splash_ring3.png",
    "assets/textures/particles/fluid_splash_ring4.png",
    "assets/textures/particles/fluid_splash_ring5.png",
    "assets/textures/particles/fluid_splash_ring6.png",
    "assets/textures/particles/fluid_splash_ring7.png",
    "assets/textures/particles/fluid_splash_ring8.png",
];

/// Returns `(curtain_rgb, foam_rgb, ring_rgb, ring_alpha, is_emissive)` tailored to each liquid's visual palette.
pub(super) fn fluid_splash_palette(
    fluid: Voxel,
    biome_water_tint: [f32; 3],
) -> ([f32; 3], [f32; 3], [f32; 3], f32, bool) {
    match fluid {
        Voxel::LiquidLava | Voxel::LiquidCoreMagma | Voxel::LiquidMolten => (
            [1.65, 0.44, 0.06],
            [2.05, 1.38, 0.28],
            [1.35, 0.34, 0.05],
            0.82,
            true,
        ),
        Voxel::LiquidAcid => (
            [0.22, 0.92, 0.14],
            [0.68, 1.25, 0.36],
            [0.24, 0.82, 0.16],
            0.72,
            true,
        ),
        Voxel::LiquidTar => (
            [0.14, 0.04, 0.22],
            [0.36, 0.16, 0.48],
            [0.18, 0.06, 0.28],
            0.80,
            false,
        ),
        Voxel::LiquidBlood => (
            [0.44, 0.04, 0.05],
            [0.76, 0.16, 0.16],
            [0.42, 0.05, 0.06],
            0.76,
            false,
        ),
        Voxel::LiquidSludge => (
            // Muddy dark brown matching liquid_sludge.png
            [0.24, 0.16, 0.10],
            [0.40, 0.29, 0.19],
            [0.22, 0.15, 0.09],
            0.78,
            false,
        ),
        Voxel::LiquidOoze => (
            // Swamp olive-green matching liquid_ooze.png
            [0.28, 0.34, 0.07],
            [0.46, 0.52, 0.12],
            [0.28, 0.33, 0.08],
            0.76,
            false,
        ),
        _ => (
            // Deep, natural water tone matching our lake/ocean shader rather than washed-out cyan
            [
                biome_water_tint[0] * 0.56,
                biome_water_tint[1] * 0.70,
                biome_water_tint[2] * 0.84,
            ],
            [0.86, 0.93, 0.98],
            [
                biome_water_tint[0] * 0.48 + 0.30,
                biome_water_tint[1] * 0.52 + 0.32,
                biome_water_tint[2] * 0.56 + 0.34,
            ],
            0.52,
            false,
        ),
    }
}

#[derive(Component)]
pub struct WaterSplash3DBatchMesh;

#[derive(Clone, Copy, Default)]
struct WaterSplash3DWave {
    position: Vec3,
    width: f32,
    height: f32,
    curtain_tint: [f32; 3],
    foam_tint: [f32; 3],
    ring_tint: [f32; 3],
    ring_alpha: f32,
    is_emissive: bool,
    age: f32,
    delay: f32,
    max_age: f32,
    alive: bool,
}

#[derive(Resource)]
pub struct WaterSplash3DPool {
    waves: Vec<WaterSplash3DWave>,
    cursor: usize,
    mesh_handle: Handle<Mesh>,
}

impl WaterSplash3DPool {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn spawn_wave(
        &mut self,
        position: Vec3,
        width: f32,
        height: f32,
        curtain_tint: [f32; 3],
        foam_tint: [f32; 3],
        ring_tint: [f32; 3],
        ring_alpha: f32,
        is_emissive: bool,
        delay: f32,
    ) {
        let slot = self.cursor;
        self.cursor = (self.cursor + 1) % MAX_SPLASH_3D_WAVES;
        self.waves[slot] = WaterSplash3DWave {
            position,
            width,
            height,
            curtain_tint,
            foam_tint,
            ring_tint,
            ring_alpha,
            is_emissive,
            age: 0.0,
            delay,
            max_age: 0.85,
            alive: true,
        };
    }
}

pub fn setup_water_splashes_3d(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut atlas_bytes = vec![0u8; SPLASH_3D_ATLAS_W * SPLASH_3D_ATLAS_H * 4];

    let blit_row = |atlas: &mut [u8], files: &[&str; SPLASH_3D_FRAMES], row_y: usize, max_h: usize| {
        for (i, path) in files.iter().enumerate() {
            if let Ok(img) = image::open(path) {
                let rgba = img.to_rgba8();
                let (iw, ih) = rgba.dimensions();
                let cw = (iw as usize).min(16);
                let ch = (ih as usize).min(max_h);
                let ox = i * 16;
                for py in 0..ch {
                    for px in 0..cw {
                        let src = rgba.get_pixel(px as u32, py as u32).0;
                        let dst = ((row_y + py) * SPLASH_3D_ATLAS_W + (ox + px)) * 4;
                        atlas[dst..dst + 4].copy_from_slice(&src);
                    }
                }
            }
        }
    };

    blit_row(&mut atlas_bytes, &SPLASH_3D_CURTAIN_FILES, 0, 32);
    blit_row(&mut atlas_bytes, &SPLASH_3D_FOAM_FILES, 32, 32);
    blit_row(&mut atlas_bytes, &SPLASH_3D_RING_FILES, 64, 16);

    let mut image = Image::new(
        Extent3d {
            width: SPLASH_3D_ATLAS_W as u32,
            height: SPLASH_3D_ATLAS_H as u32,
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
        WaterSplash3DBatchMesh,
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(mat_handle),
        Transform::IDENTITY,
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
        bevy::camera::visibility::NoFrustumCulling,
    ));

    commands.insert_resource(WaterSplash3DPool {
        waves: vec![WaterSplash3DWave::default(); MAX_SPLASH_3D_WAVES],
        cursor: 0,
        mesh_handle,
    });
}

#[allow(clippy::too_many_arguments)]
pub fn update_water_splashes_3d(
    time: Res<Time>,
    mut pool: ResMut<WaterSplash3DPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    env_state: Option<Res<EnvironmentState>>,
    mut batch_query: Query<&mut Visibility, With<WaterSplash3DBatchMesh>>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok(mut batch_vis) = batch_query.single_mut() else {
        return;
    };
    if !pool.waves.iter().any(|w| w.alive) {
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

    let push_quad = |positions: &mut Vec<[f32; 3]>,
                     normals: &mut Vec<[f32; 3]>,
                     uvs: &mut Vec<[f32; 2]>,
                     colors: &mut Vec<[f32; 4]>,
                     indices: &mut Vec<u32>,
                     v_cursor: &mut u32,
                     pts: [[f32; 3]; 4],
                     uv_rect: [f32; 4],
                     col: [f32; 4]| {
        positions.extend_from_slice(&pts);
        normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
        uvs.extend_from_slice(&[
            [uv_rect[0], uv_rect[3]],
            [uv_rect[2], uv_rect[3]],
            [uv_rect[2], uv_rect[1]],
            [uv_rect[0], uv_rect[1]],
        ]);
        colors.extend_from_slice(&[col; 4]);
        indices.extend_from_slice(&[
            *v_cursor,
            *v_cursor + 1,
            *v_cursor + 2,
            *v_cursor,
            *v_cursor + 2,
            *v_cursor + 3,
        ]);
        *v_cursor += 4;
    };

    for w in pool.waves.iter_mut() {
        if !w.alive {
            continue;
        }
        w.age += dt;
        if w.age < w.delay {
            continue;
        }
        let local_age = w.age - w.delay;
        if local_age >= w.max_age {
            w.alive = false;
            continue;
        }

        let progress = (local_age / w.max_age).clamp(0.0, 0.999);
        let frame = ((progress * SPLASH_3D_FRAMES as f32).floor() as usize).min(SPLASH_3D_FRAMES - 1);
        let u0 = (frame * 16) as f32 / SPLASH_3D_ATLAS_W as f32;
        let u1 = ((frame + 1) * 16) as f32 / SPLASH_3D_ATLAS_W as f32;

        let curtain_uv = [u0, 0.0, u1, 32.0 / SPLASH_3D_ATLAS_H as f32];
        let foam_uv = [u0, 32.0 / SPLASH_3D_ATLAS_H as f32, u1, 64.0 / SPLASH_3D_ATLAS_H as f32];
        let ring_uv = [u0, 64.0 / SPLASH_3D_ATLAS_H as f32, u1, 1.0];

        let alpha_fade = if progress > 0.78 {
            1.0 - (progress - 0.78) / 0.22
        } else {
            1.0
        };

        let cx = w.position.x;
        let cy = w.position.y;
        let cz = w.position.z;
        let hw = w.width * 0.5;
        let top_y = cy + w.height;

        let light_scale = if w.is_emissive { 1.0 } else { day_light };
        let water_col = [
            w.curtain_tint[0] * light_scale,
            w.curtain_tint[1] * light_scale,
            w.curtain_tint[2] * light_scale,
            0.84 * alpha_fade,
        ];
        let foam_col = [
            w.foam_tint[0] * light_scale,
            w.foam_tint[1] * light_scale,
            w.foam_tint[2] * light_scale,
            0.88 * alpha_fade,
        ];
        let ring_col = [
            w.ring_tint[0] * light_scale,
            w.ring_tint[1] * light_scale,
            w.ring_tint[2] * light_scale,
            w.ring_alpha * alpha_fade,
        ];

        // 1. 4-sided 3D hollow prism (water-colored splash curtain)
        let emit_prism = |positions: &mut Vec<[f32; 3]>,
                          normals: &mut Vec<[f32; 3]>,
                          uvs: &mut Vec<[f32; 2]>,
                          colors: &mut Vec<[f32; 4]>,
                          indices: &mut Vec<u32>,
                          v_cursor: &mut u32,
                          r: f32,
                          uv_rect: [f32; 4],
                          col: [f32; 4]| {
            // North (-Z)
            push_quad(positions, normals, uvs, colors, indices, v_cursor,
                [[cx - r, cy, cz - r], [cx + r, cy, cz - r], [cx + r, top_y, cz - r], [cx - r, top_y, cz - r]],
                uv_rect, col);
            // South (+Z)
            push_quad(positions, normals, uvs, colors, indices, v_cursor,
                [[cx + r, cy, cz + r], [cx - r, cy, cz + r], [cx - r, top_y, cz + r], [cx + r, top_y, cz + r]],
                uv_rect, col);
            // East (+X)
            push_quad(positions, normals, uvs, colors, indices, v_cursor,
                [[cx + r, cy, cz - r], [cx + r, cy, cz + r], [cx + r, top_y, cz + r], [cx + r, top_y, cz - r]],
                uv_rect, col);
            // West (-X)
            push_quad(positions, normals, uvs, colors, indices, v_cursor,
                [[cx - r, cy, cz + r], [cx - r, cy, cz - r], [cx - r, top_y, cz - r], [cx - r, top_y, cz + r]],
                uv_rect, col);
        };

        emit_prism(&mut positions, &mut normals, &mut uvs, &mut colors, &mut indices, &mut v_cursor, hw, curtain_uv, water_col);
        // 2. Pure white foam overlay prism (offset outward slightly to prevent z-fighting)
        emit_prism(&mut positions, &mut normals, &mut uvs, &mut colors, &mut indices, &mut v_cursor, hw + 0.005, foam_uv, foam_col);

        // 3. Flat horizontal splash ring on the water surface
        let rw = w.width * 0.85;
        let ry = cy + 0.03;
        push_quad(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut colors,
            &mut indices,
            &mut v_cursor,
            [
                [cx - rw, ry, cz - rw],
                [cx + rw, ry, cz - rw],
                [cx + rw, ry, cz + rw],
                [cx - rw, ry, cz + rw],
            ],
            ring_uv,
            ring_col,
        );
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

