use bevy::prelude::*;

use super::{
    greedy::{ChunkMeshes, FaceDirection, MeshBuffers},
    textures::VoxelTextureRegistry,
};
use crate::{
    generation::TerrainGenerator,
    world::{CHUNK_SIZE, Voxel},
};

/// Level-of-Detail tier for terrain rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
pub enum ChunkLod {
    /// Full 1m³ greedy-meshed voxels with custom shapes, caves, and full physics.
    Lod0,
    /// Near horizon 1m×1m block resolution (16×16 cells per chunk column) with 2D greedy merging.
    Lod1,
    /// Mid-distance 2m×2m cuboid block grid (16×16 cells per 32×32m 2×2 super-chunk).
    Lod2,
    /// Far-distance 4m×4m cuboid block grid (16×16 cells per 64×64m 4×4 super-chunk).
    Lod3,
}

impl ChunkLod {
    /// Horizontal sampling stride in meters (voxels).
    #[inline]
    pub fn stride(self) -> i32 {
        match self {
            Self::Lod0 | Self::Lod1 => 1,
            Self::Lod2 => 2,
            Self::Lod3 => 4,
        }
    }

    /// Extent of the LOD mesh along X and Z in chunk columns (1 for Lod0/Lod1, 2 for Lod2, 4 for Lod3).
    #[inline]
    pub fn chunk_extent(self) -> i32 {
        match self {
            Self::Lod0 | Self::Lod1 => 1,
            Self::Lod2 => 2,
            Self::Lod3 => 4,
        }
    }

    /// Size of the LOD mesh in meters (voxels) along X and Z.
    #[inline]
    pub fn world_size(self) -> f32 {
        (self.chunk_extent() * CHUNK_SIZE as i32) as f32
    }
}

/// Pushes an arbitrary quad with explicit UVs into `MeshBuffers`.
fn push_lod_textured_quad(
    buffers: &mut MeshBuffers,
    verts: [[f32; 3]; 4],
    normal: [f32; 3],
    uvs: [[f32; 2]; 4],
    layer: f32,
    color: [f32; 4],
) {
    let base = buffers.positions.len() as u32;
    buffers.positions.extend_from_slice(&verts);
    buffers.normals.extend_from_slice(&[normal; 4]);
    buffers.colors.extend_from_slice(&[color; 4]);
    buffers.uv_bs.extend_from_slice(&[[layer, 0.0]; 4]);
    buffers.uvs.extend_from_slice(&uvs);
    buffers
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Helper to push an axis-aligned rectangular quad into `MeshBuffers`.
fn push_lod_quad(
    buffers: &mut MeshBuffers,
    direction: FaceDirection,
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    p3: [f32; 3],
    u_span: f32,
    v_span: f32,
    layer: f32,
    color: [f32; 4],
) {
    let uvs = match direction {
        FaceDirection::PositiveY | FaceDirection::NegativeY => {
            [[0.0, 0.0], [0.0, v_span], [u_span, v_span], [u_span, 0.0]]
        }
        _ => [[0.0, v_span], [0.0, 0.0], [u_span, 0.0], [u_span, v_span]],
    };
    push_lod_textured_quad(
        buffers,
        [p0, p1, p2, p3],
        direction.normal_f32(),
        uvs,
        layer,
        color,
    );
}

/// Computes substrate-aware foliage tint matching LOD0 (`tint[3] = 6.0` for rigid alpha-cutout flora).
#[inline]
fn lod_flora_tint(voxel: Voxel, ground: Voxel, world_pos: IVec3) -> [f32; 4] {
    let mut tint = voxel.tint_color_at(world_pos);
    if voxel.is_tinted_flora() {
        if ground.is_sandy_soil() {
            let dry_olive = [0.74, 0.78, 0.44];
            let t = 0.45;
            tint[0] = tint[0] * (1.0 - t) + dry_olive[0] * t;
            tint[1] = tint[1] * (1.0 - t) + dry_olive[1] * t;
            tint[2] = tint[2] * (1.0 - t) + dry_olive[2] * t;
        } else if !ground.is_grass_soil() && ground.is_fertile_soil() {
            let earthy_green = [0.56, 0.76, 0.40];
            let t = 0.18;
            tint[0] = tint[0] * (1.0 - t) + earthy_green[0] * t;
            tint[1] = tint[1] * (1.0 - t) + earthy_green[1] * t;
            tint[2] = tint[2] * (1.0 - t) + earthy_green[2] * t;
        }
    }
    tint[3] = 6.0;
    tint
}

/// Emits 2 intersecting double-sided diagonal alpha-cutout quads for LOD1 plants/bushes/flowers.
fn push_lod_cross_billboard(
    buffers: &mut MeshBuffers,
    cx: f32,
    base_y: f32,
    cz: f32,
    height: f32,
    layer: f32,
    color: [f32; 4],
) {
    let fx = cx - 0.5;
    let fz = cz - 0.5;
    let tip_y = base_y + height - 0.002;
    let eps = 0.005;
    let uv_fwd = [[eps, 1.0 - eps], [eps, eps], [1.0 - eps, eps], [1.0 - eps, 1.0 - eps]];
    let uv_rev = [[1.0 - eps, 1.0 - eps], [1.0 - eps, eps], [eps, eps], [eps, 1.0 - eps]];
    let n_up = [0.0, 1.0, 0.0];

    let a00 = [fx, base_y, fz];
    let a01 = [fx, tip_y, fz];
    let a11 = [fx + 1.0, tip_y, fz + 1.0];
    let a10 = [fx + 1.0, base_y, fz + 1.0];
    push_lod_textured_quad(buffers, [a00, a01, a11, a10], n_up, uv_fwd, layer, color);
    push_lod_textured_quad(buffers, [a10, a11, a01, a00], n_up, uv_rev, layer, color);

    let b00 = [fx, base_y, fz + 1.0];
    let b01 = [fx, tip_y, fz + 1.0];
    let b11 = [fx + 1.0, tip_y, fz];
    let b10 = [fx + 1.0, base_y, fz];
    push_lod_textured_quad(buffers, [b00, b01, b11, b10], n_up, uv_fwd, layer, color);
    push_lod_textured_quad(buffers, [b10, b11, b01, b00], n_up, uv_rev, layer, color);
}

/// Emits a lightweight top-facing alpha-cutout carpet quad (e.g., fallen cherry petals).
fn push_lod_flat_carpet(
    buffers: &mut MeshBuffers,
    x0: f32,
    y: f32,
    z0: f32,
    layer: f32,
    color: [f32; 4],
) {
    let py = y + 0.04;
    let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    push_lod_textured_quad(
        buffers,
        [
            [x0, py, z0 + 1.0],
            [x0 + 1.0, py, z0 + 1.0],
            [x0 + 1.0, py, z0],
            [x0, py, z0],
        ],
        [0.0, 1.0, 0.0],
        uvs,
        layer,
        color,
    );
}

/// Pushes a vertical side wall between `bot_y` and `top_y` with block textures.
fn push_lod_wall(
    buffers: &mut MeshBuffers,
    direction: FaceDirection,
    p_bot_left: [f32; 3],
    p_top_left: [f32; 3],
    p_top_right: [f32; 3],
    p_bot_right: [f32; 3],
    u_span: f32,
    top_y: f32,
    bot_y: f32,
    surface_voxel: Voxel,
    textures: &VoxelTextureRegistry,
    world_pos: IVec3,
) {
    let height = top_y - bot_y;
    if height <= 0.0 {
        return;
    }

    let top_side_layer = textures
        .get_face_texture_info(surface_voxel, world_pos, direction)
        .0 as f32;

    let is_soil = surface_voxel.is_grass_soil()
        || matches!(
            surface_voxel,
            Voxel::LoamSnow | Voxel::PeatSnow | Voxel::SiltSnow
        );

    if height <= 1.0 || !is_soil {
        push_lod_quad(
            buffers,
            direction,
            p_bot_left,
            p_top_left,
            p_top_right,
            p_bot_right,
            u_span,
            height,
            top_side_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
    } else {
        // Top 1 block: surface voxel side (e.g. grass side)
        let mid_y = top_y - 1.0;
        let mut p_mid_left = p_top_left;
        p_mid_left[1] = mid_y;
        let mut p_mid_right = p_top_right;
        p_mid_right[1] = mid_y;

        push_lod_quad(
            buffers,
            direction,
            p_mid_left,
            p_top_left,
            p_top_right,
            p_mid_right,
            u_span,
            1.0,
            top_side_layer,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Subsurface remaining drop: stone strata
        // Samples realistic geological stone strata based on depth
        let sub_voxel = if world_pos.y > 60 {
            Voxel::Dolomite
        } else if world_pos.y > 20 {
            Voxel::Andesite
        } else if world_pos.y > -20 {
            Voxel::Limestone
        } else {
            Voxel::Slate
        };

        let sub_layer = textures
            .get_face_texture_info(sub_voxel, world_pos - IVec3::Y, direction)
            .0 as f32;

        push_lod_quad(
            buffers,
            direction,
            p_bot_left,
            p_mid_left,
            p_mid_right,
            p_bot_right,
            u_span,
            mid_y - bot_y,
            sub_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
    }
}

/// Builds 2D greedy-merged cuboid voxel LOD meshes for a column or super-chunk.
///
/// `Lod1` keeps crisp 1x1m block resolution matching near chunks; `Lod2` (2x2m) and
/// `Lod3` (4x4m) cover mid/far horizons in `16x16` cell super-chunks with full 2D greedy merging.
pub fn build_lod_mesh(
    chunk_coordinate: IVec2,
    lod: ChunkLod,
    generator: &TerrainGenerator,
    textures: &VoxelTextureRegistry,
) -> ChunkMeshes {
    let stride = lod.stride();
    let region_size = lod.chunk_extent() * CHUNK_SIZE as i32;
    let num_cells = region_size / stride; // 16x16 cells for Lod1, Lod2, and Lod3
    let grid_dim = num_cells + 2;

    let chunk_origin_x = chunk_coordinate.x * CHUNK_SIZE as i32;
    let chunk_origin_z = chunk_coordinate.y * CHUNK_SIZE as i32;

    let total_samples = (grid_dim * grid_dim) as usize;
    let mut cols = vec![crate::generation::generator::TerrainColumn::default(); total_samples];
    let mut heights = vec![0i32; total_samples];
    let mut materials = vec![Voxel::Air; total_samples];
    let mut water_levels = vec![None; total_samples];

    let idx = |i: i32, j: i32| -> usize { ((i + 1) * grid_dim + (j + 1)) as usize };
    let sample_offset = if stride > 1 { stride / 2 } else { 0 };

    // 1. Sample column heights and materials across the grid + 1-cell border
    for i in -1..=num_cells {
        let world_x = chunk_origin_x + i * stride + sample_offset;
        for j in -1..=num_cells {
            let world_z = chunk_origin_z + j * stride + sample_offset;
            let col = if stride == 1 {
                generator.sample_column_cached(world_x, world_z)
            } else {
                generator.sample_column(world_x, world_z)
            };
            let k = idx(i, j);
            let (eff_height, eff_mat) =
                generator.lod_surface_height_and_material(world_x, world_z, col);
            cols[k] = col;
            heights[k] = eff_height;
            water_levels[k] = col.water_level;
            materials[k] = eff_mat;
        }
    }

    let mut opaque_buffers = MeshBuffers::with_capacity(if lod == ChunkLod::Lod1 {
        (num_cells * num_cells * 2) as usize
    } else {
        (num_cells * num_cells) as usize
    });
    let mut transparent_buffers = MeshBuffers::with_capacity(64);
    let cell_w = stride as f32;

    let n = num_cells as usize;
    let mut top_layers = vec![0u16; n * n];
    let mut top_tints = vec![[1.0f32; 4]; n * n];
    let mut top_tint_q = vec![[0u8; 3]; n * n];

    for i in 0..num_cells {
        let world_x = chunk_origin_x + i * stride + sample_offset;
        for j in 0..num_cells {
            let world_z = chunk_origin_z + j * stride + sample_offset;
            let k = idx(i, j);
            let h_curr = heights[k];
            let voxel = materials[k];
            let world_pos = IVec3::new(world_x, h_curr, world_z);

            let layer = textures
                .get_face_texture_info(voxel, world_pos, FaceDirection::PositiveY)
                .0;
            let mut tint = voxel.tint_color_at(world_pos);
            if h_curr >= 110 {
                let snow_blend = ((h_curr - 110) as f32 / 16.0).clamp(0.0, 1.0);
                tint[0] = tint[0] * (1.0 - snow_blend) + 0.95 * snow_blend;
                tint[1] = tint[1] * (1.0 - snow_blend) + 0.98 * snow_blend;
                tint[2] = tint[2] * (1.0 - snow_blend) + 1.00 * snow_blend;
            }
            tint[3] = 1.0;

            let cell_i = (i as usize) * n + (j as usize);
            top_layers[cell_i] = layer;
            top_tints[cell_i] = tint;
            top_tint_q[cell_i] = [
                (tint[0] * 64.0).round().clamp(0.0, 255.0) as u8,
                (tint[1] * 64.0).round().clamp(0.0, 255.0) as u8,
                (tint[2] * 64.0).round().clamp(0.0, 255.0) as u8,
            ];
        }
    }

    // 2A. 2D Greedy Merge for Top Faces (+Y)
    let mut visited = vec![false; n * n];
    for j in 0..num_cells {
        for i in 0..num_cells {
            let start_ci = (i as usize) * n + (j as usize);
            if visited[start_ci] {
                continue;
            }
            let h0 = heights[idx(i, j)];
            let layer0 = top_layers[start_ci];
            let tq0 = top_tint_q[start_ci];
            let tint0 = top_tints[start_ci];

            let mut w = 1i32;
            while i + w < num_cells {
                let ci = ((i + w) as usize) * n + (j as usize);
                if visited[ci]
                    || heights[idx(i + w, j)] != h0
                    || top_layers[ci] != layer0
                    || top_tint_q[ci] != tq0
                {
                    break;
                }
                w += 1;
            }

            let mut d = 1i32;
            'rows: while j + d < num_cells {
                for di in 0..w {
                    let ci = ((i + di) as usize) * n + ((j + d) as usize);
                    if visited[ci]
                        || heights[idx(i + di, j + d)] != h0
                        || top_layers[ci] != layer0
                        || top_tint_q[ci] != tq0
                    {
                        break 'rows;
                    }
                }
                d += 1;
            }

            for di in 0..w {
                for dj in 0..d {
                    visited[((i + di) as usize) * n + ((j + dj) as usize)] = true;
                }
            }

            let local_x0 = (i * stride) as f32;
            let local_x1 = ((i + w) * stride) as f32;
            let local_z0 = (j * stride) as f32;
            let local_z1 = ((j + d) * stride) as f32;
            let top_y = (h0 + 1) as f32;

            push_lod_quad(
                &mut opaque_buffers,
                FaceDirection::PositiveY,
                [local_x0, top_y, local_z0],
                [local_x0, top_y, local_z1],
                [local_x1, top_y, local_z1],
                [local_x1, top_y, local_z0],
                (w * stride) as f32,
                (d * stride) as f32,
                layer0 as f32,
                tint0,
            );
        }
    }

    // 2B. Stepped Vertical Side Walls (horizon skirt is shallow on 1x1 Lod1, deeper on outer tiers)
    let skirt_depth = match lod {
        ChunkLod::Lod0 | ChunkLod::Lod1 => 2.0,
        ChunkLod::Lod2 => 5.0,
        ChunkLod::Lod3 => 10.0,
    };

    for i in 0..num_cells {
        let local_x0 = (i * stride) as f32;
        let local_x1 = ((i + 1) * stride) as f32;
        let world_x = chunk_origin_x + i * stride + sample_offset;

        for j in 0..num_cells {
            let local_z0 = (j * stride) as f32;
            let local_z1 = ((j + 1) * stride) as f32;
            let world_z = chunk_origin_z + j * stride + sample_offset;

            let h_curr = heights[idx(i, j)];
            let voxel = materials[idx(i, j)];
            let world_pos = IVec3::new(world_x, h_curr, world_z);
            let top_y = (h_curr + 1) as f32;

            let h_west = heights[idx(i - 1, j)];
            let bot_west = if i == 0 {
                ((h_west + 1) as f32).min(top_y - skirt_depth)
            } else {
                (h_west + 1) as f32
            };
            if top_y > bot_west {
                push_lod_wall(
                    &mut opaque_buffers,
                    FaceDirection::NegativeX,
                    [local_x0, bot_west, local_z1],
                    [local_x0, top_y, local_z1],
                    [local_x0, top_y, local_z0],
                    [local_x0, bot_west, local_z0],
                    cell_w,
                    top_y,
                    bot_west,
                    voxel,
                    textures,
                    world_pos,
                );
            }

            let h_east = heights[idx(i + 1, j)];
            let bot_east = if i == num_cells - 1 {
                ((h_east + 1) as f32).min(top_y - skirt_depth)
            } else {
                (h_east + 1) as f32
            };
            if top_y > bot_east {
                push_lod_wall(
                    &mut opaque_buffers,
                    FaceDirection::PositiveX,
                    [local_x1, bot_east, local_z0],
                    [local_x1, top_y, local_z0],
                    [local_x1, top_y, local_z1],
                    [local_x1, bot_east, local_z1],
                    cell_w,
                    top_y,
                    bot_east,
                    voxel,
                    textures,
                    world_pos,
                );
            }

            let h_north = heights[idx(i, j - 1)];
            let bot_north = if j == 0 {
                ((h_north + 1) as f32).min(top_y - skirt_depth)
            } else {
                (h_north + 1) as f32
            };
            if top_y > bot_north {
                push_lod_wall(
                    &mut opaque_buffers,
                    FaceDirection::NegativeZ,
                    [local_x0, bot_north, local_z0],
                    [local_x0, top_y, local_z0],
                    [local_x1, top_y, local_z0],
                    [local_x1, bot_north, local_z0],
                    cell_w,
                    top_y,
                    bot_north,
                    voxel,
                    textures,
                    world_pos,
                );
            }

            let h_south = heights[idx(i, j + 1)];
            let bot_south = if j == num_cells - 1 {
                ((h_south + 1) as f32).min(top_y - skirt_depth)
            } else {
                (h_south + 1) as f32
            };
            if top_y > bot_south {
                push_lod_wall(
                    &mut opaque_buffers,
                    FaceDirection::PositiveZ,
                    [local_x1, bot_south, local_z1],
                    [local_x1, top_y, local_z1],
                    [local_x0, top_y, local_z1],
                    [local_x0, bot_south, local_z1],
                    cell_w,
                    top_y,
                    bot_south,
                    voxel,
                    textures,
                    world_pos,
                );
            }
        }
    }

    // 2C. 2D Greedy Merge for Water Surface Quads
    visited.fill(false);
    for j in 0..num_cells {
        for i in 0..num_cells {
            let start_ci = (i as usize) * n + (j as usize);
            if visited[start_ci] {
                continue;
            }
            let k = idx(i, j);
            let Some(wl) = water_levels[k] else {
                visited[start_ci] = true;
                continue;
            };
            if heights[k] >= wl {
                visited[start_ci] = true;
                continue;
            }

            let has_same_water = |ci: i32, cj: i32| -> bool {
                let ck = idx(ci, cj);
                water_levels[ck] == Some(wl) && heights[ck] < wl
            };

            let mut w = 1i32;
            while i + w < num_cells {
                let ci = ((i + w) as usize) * n + (j as usize);
                if visited[ci] || !has_same_water(i + w, j) {
                    break;
                }
                w += 1;
            }

            let mut d = 1i32;
            'w_rows: while j + d < num_cells {
                for di in 0..w {
                    let ci = ((i + di) as usize) * n + ((j + d) as usize);
                    if visited[ci] || !has_same_water(i + di, j + d) {
                        break 'w_rows;
                    }
                }
                d += 1;
            }

            for di in 0..w {
                for dj in 0..d {
                    visited[((i + di) as usize) * n + ((j + dj) as usize)] = true;
                }
            }

            let world_x = chunk_origin_x + i * stride + sample_offset;
            let world_z = chunk_origin_z + j * stride + sample_offset;
            let water_y = (wl + 1) as f32 - 0.05;
            let water_layer = textures
                .get_face_texture_info(
                    Voxel::LiquidWater,
                    IVec3::new(world_x, wl, world_z),
                    FaceDirection::PositiveY,
                )
                .0 as f32;

            let local_x0 = (i * stride) as f32;
            let local_x1 = ((i + w) * stride) as f32;
            let local_z0 = (j * stride) as f32;
            let local_z1 = ((j + d) * stride) as f32;

            push_lod_quad(
                &mut transparent_buffers,
                FaceDirection::PositiveY,
                [local_x0, water_y, local_z0],
                [local_x0, water_y, local_z1],
                [local_x1, water_y, local_z1],
                [local_x1, water_y, local_z0],
                (w * stride) as f32,
                (d * stride) as f32,
                water_layer,
                [0.35, 0.65, 0.92, 1.0],
            );
        }
    }

    // --- D. Distant LOD Trees (zero-noise O(1) lookup from sampled grid + opaque Early-Z proxies) ---
    let min_wx = chunk_origin_x;
    let min_wz = chunk_origin_z;
    let max_wx = chunk_origin_x + region_size - 1;
    let max_wz = chunk_origin_z + region_size - 1;

    let sample_grid_idx = |wx: i32, wz: i32| -> usize {
        let gi = ((wx - chunk_origin_x) / stride).clamp(0, num_cells - 1);
        let gj = ((wz - chunk_origin_z) / stride).clamp(0, num_cells - 1);
        idx(gi, gj)
    };

    let lod_trees = crate::generation::trees::sample_lod_trees_in_region(
        min_wx,
        min_wz,
        max_wx,
        max_wz,
        generator.seed,
        |wx, wz| cols[sample_grid_idx(wx, wz)],
        |wx, _wy, wz, _col| {
            let g = materials[sample_grid_idx(wx, wz)];
            if g.is_solid_opaque() && !g.is_fluid() && !g.is_empty() {
                Some(g)
            } else {
                None
            }
        },
    );

    let is_lod1 = lod == ChunkLod::Lod1;
    let is_lod3 = lod == ChunkLod::Lod3;

    // --- D1. Lightweight LOD Surface Flora & Rubble (Lod1 full detail, Lod2 macro cacti/bamboo) ---
    if is_lod1 || lod == ChunkLod::Lod2 {
        for i in 0..num_cells {
            let world_x = chunk_origin_x + i * stride + sample_offset;
            let lx_center = (i * stride) as f32 + (stride as f32 * 0.5);
            for j in 0..num_cells {
                let k = idx(i, j);
                let col = cols[k];
                let h_curr = heights[k];
                // Skip columns covered by 3D rock boulders/overhangs (no surface scatter inside overhang solids)
                if h_curr > col.terrain_height {
                    continue;
                }
                let ground = materials[k];
                let world_z = chunk_origin_z + j * stride + sample_offset;

                let Some(prop) = crate::generation::flora::sample_lod_surface_prop(
                    world_x,
                    world_z,
                    ground,
                    col,
                    generator.seed,
                    is_lod1,
                ) else {
                    continue;
                };

                let lz_center = (j * stride) as f32 + (stride as f32 * 0.5);
                let base_y = (h_curr + 1) as f32;
                let world_pos = IVec3::new(world_x, h_curr + 1, world_z);

                match prop {
                    crate::generation::flora::LodFloraKind::RubbleBox(voxel) => {
                        let layer = textures
                            .get_face_texture_info(voxel, world_pos, FaceDirection::PositiveY)
                            .0 as f32;
                        push_lod_box(
                            &mut opaque_buffers,
                            lx_center - 0.28,
                            base_y,
                            lz_center - 0.28,
                            lx_center + 0.28,
                            base_y + 0.26,
                            lz_center + 0.28,
                            layer,
                            [1.0, 1.0, 1.0, 1.0],
                            true,
                        );
                    }
                    crate::generation::flora::LodFloraKind::TallBox {
                        voxel,
                        height,
                        half_width,
                    } => {
                        let layer = textures
                            .get_face_texture_info(voxel, world_pos, FaceDirection::PositiveX)
                            .0 as f32;
                        let mut tint = voxel.tint_color_at(world_pos);
                        tint[3] = if voxel == Voxel::BambooLeaves { 2.0 } else { 1.0 };
                        push_lod_box(
                            &mut opaque_buffers,
                            lx_center - half_width,
                            base_y,
                            lz_center - half_width,
                            lx_center + half_width,
                            base_y + height,
                            lz_center + half_width,
                            layer,
                            tint,
                            true,
                        );
                    }
                    crate::generation::flora::LodFloraKind::CrossPlant { bot, top } => {
                        let layer = textures
                            .get_face_texture_info(bot, world_pos, FaceDirection::PositiveX)
                            .0 as f32;
                        let tint = lod_flora_tint(bot, ground, world_pos);
                        push_lod_cross_billboard(
                            &mut opaque_buffers,
                            lx_center,
                            base_y,
                            lz_center,
                            1.0,
                            layer,
                            tint,
                        );
                        if let Some(top_vox) = top {
                            let top_pos = world_pos + IVec3::Y;
                            let top_layer = textures
                                .get_face_texture_info(top_vox, top_pos, FaceDirection::PositiveX)
                                .0 as f32;
                            let top_tint = lod_flora_tint(top_vox, ground, top_pos);
                            push_lod_cross_billboard(
                                &mut opaque_buffers,
                                lx_center,
                                base_y + 1.0,
                                lz_center,
                                1.0,
                                top_layer,
                                top_tint,
                            );
                        }
                    }
                    crate::generation::flora::LodFloraKind::FlatCarpet(voxel) => {
                        let layer = textures
                            .get_face_texture_info(voxel, world_pos, FaceDirection::PositiveY)
                            .0 as f32;
                        let mut tint = voxel.tint_color_at(world_pos);
                        tint[3] = 6.0;
                        push_lod_flat_carpet(
                            &mut opaque_buffers,
                            lx_center - 0.5,
                            base_y,
                            lz_center - 0.5,
                            layer,
                            tint,
                        );
                    }
                }
            }
        }
    }

    for tree in lod_trees {
        // In far Lod3 super-chunks (4x4), thin to 50% of trees with wider canopies
        // to preserve dense horizon silhouettes at half the triangle count.
        if is_lod3 && ((tree.root_x.div_euclid(7) ^ tree.root_z.div_euclid(7)) & 1) != 0 {
            continue;
        }

        let lx = (tree.root_x - chunk_origin_x) as f32 + 0.5;
        let lz = (tree.root_z - chunk_origin_z) as f32 + 0.5;
        let base_y = (tree.root_y + 1) as f32;
        let world_pos = IVec3::new(tree.root_x, tree.root_y + 1, tree.root_z);

        // Trunk cuboid (omit top face when a canopy covers it)
        let trunk_half = if is_lod3 { 0.75 } else { 0.5 };
        let trunk_layer = textures
            .get_face_texture_info(tree.log_voxel, world_pos, FaceDirection::PositiveX)
            .0 as f32;
        push_lod_box(
            &mut opaque_buffers,
            lx - trunk_half,
            base_y,
            lz - trunk_half,
            lx + trunk_half,
            base_y + tree.trunk_height,
            lz + trunk_half,
            trunk_layer,
            [1.0, 1.0, 1.0, 1.0],
            tree.leaf_voxel.is_none(),
        );

        // Canopy cuboid(s) with solid opaque LOD leaf tag (alpha = 2.0: no discard, no wind sway, full 6-face tint)
        if let Some(leaf_voxel) = tree.leaf_voxel {
            let leaf_layer = textures
                .get_face_texture_info(leaf_voxel, world_pos, FaceDirection::PositiveY)
                .0 as f32;
            let mut leaf_tint = leaf_voxel.tint_color_at(world_pos);
            leaf_tint[3] = 2.0;

            let r_scale = if is_lod3 { 1.3 } else { 1.0 };

            if tree.is_conifer && !is_lod3 {
                // Two-tiered conical silhouette for Lod1/Lod2 pines/redwoods
                let r1 = tree.canopy_radius;
                let y0 = base_y + (tree.trunk_height - tree.canopy_height).max(2.5);
                let y_mid = y0 + tree.canopy_height * 0.55;
                let y_top = base_y + tree.trunk_height + 1.5;
                push_lod_box(
                    &mut opaque_buffers,
                    lx - r1,
                    y0,
                    lz - r1,
                    lx + r1,
                    y_mid,
                    lz + r1,
                    leaf_layer,
                    leaf_tint,
                    true,
                );
                let r2 = (r1 * 0.58).max(1.2);
                push_lod_box(
                    &mut opaque_buffers,
                    lx - r2,
                    y_mid,
                    lz - r2,
                    lx + r2,
                    y_top,
                    lz + r2,
                    leaf_layer,
                    leaf_tint,
                    true,
                );
            } else if is_lod1 {
                // Near 1x1 Lod1 broadleaf/palm trees: crisp 2-layer rounded crown matching LOD0 silhouette
                let r1 = tree.canopy_radius;
                let r2 = (r1 * 0.65).max(1.2);
                let y0 = base_y + tree.trunk_height - tree.canopy_height * 0.45;
                let y_mid = y0 + tree.canopy_height * 0.72;
                let y1 = y0 + tree.canopy_height + 0.6;
                push_lod_box(
                    &mut opaque_buffers,
                    lx - r1,
                    y0,
                    lz - r1,
                    lx + r1,
                    y_mid,
                    lz + r1,
                    leaf_layer,
                    leaf_tint,
                    true,
                );
                push_lod_box(
                    &mut opaque_buffers,
                    lx - r2,
                    y_mid,
                    lz - r2,
                    lx + r2,
                    y1,
                    lz + r2,
                    leaf_layer,
                    leaf_tint,
                    true,
                );
            } else {
                let r = tree.canopy_radius * r_scale;
                let y0 = if tree.is_conifer {
                    base_y + (tree.trunk_height - tree.canopy_height).max(2.5)
                } else {
                    base_y + tree.trunk_height - tree.canopy_height * 0.45
                };
                let y1 = y0 + tree.canopy_height;
                push_lod_box(
                    &mut opaque_buffers,
                    lx - r,
                    y0,
                    lz - r,
                    lx + r,
                    y1,
                    lz + r,
                    leaf_layer,
                    leaf_tint,
                    true,
                );
            }
        }
    }

    ChunkMeshes {
        opaque: opaque_buffers.into_mesh(),
        transparent: transparent_buffers.into_mesh(),
        grass: None,
        visibility_mask: (1u64 << 36) - 1,
    }
}

/// Pushes a 4-face or 5-face axis-aligned box into `MeshBuffers` for distant LOD tree proxies.
#[allow(clippy::too_many_arguments)]
fn push_lod_box(
    buffers: &mut MeshBuffers,
    x0: f32,
    y0: f32,
    z0: f32,
    x1: f32,
    y1: f32,
    z1: f32,
    layer: f32,
    color: [f32; 4],
    include_top: bool,
) {
    let w = (x1 - x0).max(0.5);
    let h = (y1 - y0).max(0.5);
    let d = (z1 - z0).max(0.5);

    if include_top {
        // Top (+Y)
        push_lod_quad(
            buffers,
            FaceDirection::PositiveY,
            [x0, y1, z0],
            [x0, y1, z1],
            [x1, y1, z1],
            [x1, y1, z0],
            w,
            d,
            layer,
            color,
        );
    }
    // North (-Z)
    push_lod_quad(
        buffers,
        FaceDirection::NegativeZ,
        [x0, y0, z0],
        [x0, y1, z0],
        [x1, y1, z0],
        [x1, y0, z0],
        w,
        h,
        layer,
        color,
    );
    // South (+Z)
    push_lod_quad(
        buffers,
        FaceDirection::PositiveZ,
        [x1, y0, z1],
        [x1, y1, z1],
        [x0, y1, z1],
        [x0, y0, z1],
        w,
        h,
        layer,
        color,
    );
    // West (-X)
    push_lod_quad(
        buffers,
        FaceDirection::NegativeX,
        [x0, y0, z1],
        [x0, y1, z1],
        [x0, y1, z0],
        [x0, y0, z0],
        d,
        h,
        layer,
        color,
    );
    // East (+X)
    push_lod_quad(
        buffers,
        FaceDirection::PositiveX,
        [x1, y0, z0],
        [x1, y1, z0],
        [x1, y1, z1],
        [x1, y0, z1],
        d,
        h,
        layer,
        color,
    );
}
