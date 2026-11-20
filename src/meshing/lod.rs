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
    /// Mid-distance 2m cuboid block grid (8x8 cells per chunk column).
    Lod1,
    /// Far-distance 4m cuboid block grid (4x4 cells per chunk column).
    Lod2,
}

impl ChunkLod {
    /// Horizontal sampling stride in meters (voxels).
    #[inline]
    pub fn stride(self) -> i32 {
        match self {
            Self::Lod0 => 1,
            Self::Lod1 => 2,
            Self::Lod2 => 4,
        }
    }

    /// Extent of the LOD mesh along X and Z in chunk columns (1 for Lod0/Lod1, 2 for Lod2 2x2 super-chunks).
    #[inline]
    pub fn chunk_extent(self) -> i32 {
        match self {
            Self::Lod0 | Self::Lod1 => 1,
            Self::Lod2 => 2,
        }
    }

    /// Size of the LOD mesh in meters (voxels) along X and Z.
    #[inline]
    pub fn world_size(self) -> f32 {
        (self.chunk_extent() * CHUNK_SIZE as i32) as f32
    }
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
    let base = buffers.positions.len() as u32;
    buffers.positions.extend_from_slice(&[p0, p1, p2, p3]);
    let norm = direction.normal_f32();
    buffers.normals.extend_from_slice(&[norm, norm, norm, norm]);
    buffers
        .colors
        .extend_from_slice(&[color, color, color, color]);
    buffers
        .uv_bs
        .extend_from_slice(&[[layer, 0.0], [layer, 0.0], [layer, 0.0], [layer, 0.0]]);

    let uvs = match direction {
        FaceDirection::PositiveY | FaceDirection::NegativeY => {
            [[0.0, 0.0], [0.0, v_span], [u_span, v_span], [u_span, 0.0]]
        }
        _ => [[0.0, v_span], [0.0, 0.0], [u_span, 0.0], [u_span, v_span]],
    };
    buffers.uvs.extend_from_slice(&uvs);

    buffers
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
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

/// Builds cuboid voxel block LOD meshes for a 16x16 chunk column.
///
/// Produces strictly axis-aligned cuboid blocks and vertical stepped walls,
/// matching the block aesthetic of the near world (identical to Distant Horizons / Voxy).
pub fn build_lod_mesh(
    chunk_coordinate: IVec2,
    lod: ChunkLod,
    generator: &TerrainGenerator,
    textures: &VoxelTextureRegistry,
) -> ChunkMeshes {
    let stride = lod.stride();
    let region_size = lod.chunk_extent() * CHUNK_SIZE as i32;
    let num_cells = region_size / stride; // 8 for Lod1 (16m / 2m), 8 for Lod2 (32m / 4m)
    let grid_dim = num_cells + 2;

    let chunk_origin_x = chunk_coordinate.x * CHUNK_SIZE as i32;
    let chunk_origin_z = chunk_coordinate.y * CHUNK_SIZE as i32;

    let total_samples = (grid_dim * grid_dim) as usize;
    let mut cols = vec![generator.sample_column(chunk_origin_x, chunk_origin_z); total_samples];
    let mut heights = vec![0i32; total_samples];
    let mut materials = vec![Voxel::Air; total_samples];
    let mut water_levels = vec![None; total_samples];

    let idx = |i: i32, j: i32| -> usize { ((i + 1) * grid_dim + (j + 1)) as usize };

    // 1. Sample column heights and materials across the grid + 1-cell border
    for i in -1..=num_cells {
        let world_x = chunk_origin_x + i * stride + stride / 2;
        for j in -1..=num_cells {
            let world_z = chunk_origin_z + j * stride + stride / 2;
            let col = generator.sample_column(world_x, world_z);
            let k = idx(i, j);
            cols[k] = col;
            heights[k] = col.terrain_height;
            water_levels[k] = col.water_level;
            materials[k] = generator.surface_material_at(world_x, col.terrain_height, world_z, col);
        }
    }

    let quad_estimate = (num_cells * num_cells * 2) as usize;
    let mut opaque_buffers = MeshBuffers::with_capacity(quad_estimate);
    let mut transparent_buffers = MeshBuffers::with_capacity(32);

    let cell_w = stride as f32;

    // 2. Build cuboid block tops and vertical stepped walls for each cell in the chunk
    for i in 0..num_cells {
        let local_x0 = (i * stride) as f32;
        let local_x1 = ((i + 1) * stride) as f32;

        for j in 0..num_cells {
            let local_z0 = (j * stride) as f32;
            let local_z1 = ((j + 1) * stride) as f32;

            let h_curr = heights[idx(i, j)];
            let voxel = materials[idx(i, j)];
            let wl = water_levels[idx(i, j)];

            let world_x = chunk_origin_x + i * stride + stride / 2;
            let world_z = chunk_origin_z + j * stride + stride / 2;
            let world_pos = IVec3::new(world_x, h_curr, world_z);

            let top_y = (h_curr + 1) as f32; // Top of the block

            // --- A. Top Face (PositiveY) ---
            let top_layer = textures
                .get_face_texture_info(voxel, world_pos, FaceDirection::PositiveY)
                .0 as f32;
            let mut tint = voxel.tint_color_at(world_pos);
            if h_curr >= 110 {
                let snow_blend = ((h_curr - 110) as f32 / 16.0).clamp(0.0, 1.0);
                tint[0] = tint[0] * (1.0 - snow_blend) + 0.95 * snow_blend;
                tint[1] = tint[1] * (1.0 - snow_blend) + 0.98 * snow_blend;
                tint[2] = tint[2] * (1.0 - snow_blend) + 1.00 * snow_blend;
            }
            tint[3] = 1.0;

            let p0 = [local_x0, top_y, local_z0];
            let p1 = [local_x0, top_y, local_z1];
            let p2 = [local_x1, top_y, local_z1];
            let p3 = [local_x1, top_y, local_z0];

            push_lod_quad(
                &mut opaque_buffers,
                FaceDirection::PositiveY,
                p0,
                p1,
                p2,
                p3,
                cell_w,
                cell_w,
                top_layer,
                tint,
            );

            // --- B. Vertical Side Walls (Stepped Cuboid Faces with Horizon Skirts) ---
            // West neighbor (i - 1, j)
            let h_west = heights[idx(i - 1, j)];
            let bot_west = if i == 0 {
                ((h_west + 1) as f32).min(top_y - 12.0)
            } else {
                (h_west + 1) as f32
            };
            if top_y > bot_west {
                let p_bot_left = [local_x0, bot_west, local_z1];
                let p_top_left = [local_x0, top_y, local_z1];
                let p_top_right = [local_x0, top_y, local_z0];
                let p_bot_right = [local_x0, bot_west, local_z0];
                push_lod_wall(
                    &mut opaque_buffers,
                    FaceDirection::NegativeX,
                    p_bot_left,
                    p_top_left,
                    p_top_right,
                    p_bot_right,
                    cell_w,
                    top_y,
                    bot_west,
                    voxel,
                    textures,
                    world_pos,
                );
            }

            // East neighbor (i + 1, j)
            let h_east = heights[idx(i + 1, j)];
            let bot_east = if i == num_cells - 1 {
                ((h_east + 1) as f32).min(top_y - 12.0)
            } else {
                (h_east + 1) as f32
            };
            if top_y > bot_east {
                let p_bot_left = [local_x1, bot_east, local_z0];
                let p_top_left = [local_x1, top_y, local_z0];
                let p_top_right = [local_x1, top_y, local_z1];
                let p_bot_right = [local_x1, bot_east, local_z1];
                push_lod_wall(
                    &mut opaque_buffers,
                    FaceDirection::PositiveX,
                    p_bot_left,
                    p_top_left,
                    p_top_right,
                    p_bot_right,
                    cell_w,
                    top_y,
                    bot_east,
                    voxel,
                    textures,
                    world_pos,
                );
            }

            // North neighbor (i, j - 1)
            let h_north = heights[idx(i, j - 1)];
            let bot_north = if j == 0 {
                ((h_north + 1) as f32).min(top_y - 12.0)
            } else {
                (h_north + 1) as f32
            };
            if top_y > bot_north {
                let p_bot_left = [local_x0, bot_north, local_z0];
                let p_top_left = [local_x0, top_y, local_z0];
                let p_top_right = [local_x1, top_y, local_z0];
                let p_bot_right = [local_x1, bot_north, local_z0];
                push_lod_wall(
                    &mut opaque_buffers,
                    FaceDirection::NegativeZ,
                    p_bot_left,
                    p_top_left,
                    p_top_right,
                    p_bot_right,
                    cell_w,
                    top_y,
                    bot_north,
                    voxel,
                    textures,
                    world_pos,
                );
            }

            // South neighbor (i, j + 1)
            let h_south = heights[idx(i, j + 1)];
            let bot_south = if j == num_cells - 1 {
                ((h_south + 1) as f32).min(top_y - 12.0)
            } else {
                (h_south + 1) as f32
            };
            if top_y > bot_south {
                let p_bot_left = [local_x1, bot_south, local_z1];
                let p_top_left = [local_x1, top_y, local_z1];
                let p_top_right = [local_x0, top_y, local_z1];
                let p_bot_right = [local_x0, bot_south, local_z1];
                push_lod_wall(
                    &mut opaque_buffers,
                    FaceDirection::PositiveZ,
                    p_bot_left,
                    p_top_left,
                    p_top_right,
                    p_bot_right,
                    cell_w,
                    top_y,
                    bot_south,
                    voxel,
                    textures,
                    world_pos,
                );
            }

            // --- C. Water Surface Quads ---
            if let Some(water_level) = wl
                && h_curr < water_level
            {
                let water_y = (water_level + 1) as f32 - 0.05;
                let water_layer = textures
                    .get_face_texture_info(
                        Voxel::LiquidWater,
                        IVec3::new(world_x, water_level, world_z),
                        FaceDirection::PositiveY,
                    )
                    .0 as f32;

                let wp0 = [local_x0, water_y, local_z0];
                let wp1 = [local_x0, water_y, local_z1];
                let wp2 = [local_x1, water_y, local_z1];
                let wp3 = [local_x1, water_y, local_z0];

                push_lod_quad(
                    &mut transparent_buffers,
                    FaceDirection::PositiveY,
                    wp0,
                    wp1,
                    wp2,
                    wp3,
                    cell_w,
                    cell_w,
                    water_layer,
                    [0.35, 0.65, 0.92, 1.0],
                );
            }
        }
    }

    // --- D. Distant LOD Trees (zero-noise O(1) lookup from sampled grid + opaque Early-Z proxies) ---
    let min_wx = chunk_origin_x;
    let min_wz = chunk_origin_z;
    let max_wx = chunk_origin_x + (num_cells as i32) * stride - 1;
    let max_wz = chunk_origin_z + (num_cells as i32) * stride - 1;

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

    let is_lod2 = lod == ChunkLod::Lod2;
    for tree in lod_trees {
        // In far Lod2 super-chunks, thin to 50% of trees and slightly widen canopies
        // to preserve dense horizon silhouettes at less than half the triangle count.
        if is_lod2 && ((tree.root_x.div_euclid(7) ^ tree.root_z.div_euclid(7)) & 1) != 0 {
            continue;
        }

        let lx = (tree.root_x - chunk_origin_x) as f32 + 0.5;
        let lz = (tree.root_z - chunk_origin_z) as f32 + 0.5;
        let base_y = (tree.root_y + 1) as f32;
        let world_pos = IVec3::new(tree.root_x, tree.root_y + 1, tree.root_z);

        // Trunk cuboid (omit top face when a canopy covers it)
        let trunk_half = if is_lod2 { 0.7 } else { 0.5 };
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

            let r_scale = if is_lod2 { 1.25 } else { 1.0 };

            if tree.is_conifer && !is_lod2 {
                // Two-tiered conical silhouette for mid-distance Lod1 pines/redwoods
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
