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
#[allow(clippy::too_many_arguments)]
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

    let is_soil = matches!(
        surface_voxel,
        Voxel::Soil_Grass
            | Voxel::Soil_Peat_Grass
            | Voxel::Soil_Silt_Grass
            | Voxel::Soil_Snowy_Grass
            | Voxel::Soil_Snowy_Peat
            | Voxel::Soil_Snowy_Silt
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
        let sub_layer = textures
            .get_face_texture_info(Voxel::Rock_Granite, world_pos - IVec3::Y, direction)
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
            heights[k] = col.terrain_height;
            water_levels[k] = col.water_level;
            materials[k] = generator.surface_material_at(world_x, col.terrain_height, world_z, col);
        }
    }

    let mut opaque_buffers = MeshBuffers::new();
    let mut transparent_buffers = MeshBuffers::new();

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

            // --- B. Vertical Side Walls (Stepped Cuboid Faces) ---
            // West neighbor (i - 1, j)
            let h_west = heights[idx(i - 1, j)];
            let bot_west = if i == 0 {
                ((h_west + 1) as f32).min(top_y - 2.0)
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
                ((h_east + 1) as f32).min(top_y - 2.0)
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
                ((h_north + 1) as f32).min(top_y - 2.0)
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
                ((h_south + 1) as f32).min(top_y - 2.0)
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
                        Voxel::Liquid_Water,
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

    // 3. Procedural Trees in LOD chunks
    if generator.tree_density > 0.0 {
        let min_cell_x = (chunk_origin_x - 3).div_euclid(5);
        let max_cell_x = (chunk_origin_x + region_size + 2).div_euclid(5);
        let min_cell_z = (chunk_origin_z - 3).div_euclid(5);
        let max_cell_z = (chunk_origin_z + region_size + 2).div_euclid(5);

        for cell_z in min_cell_z..=max_cell_z {
            for cell_x in min_cell_x..=max_cell_x {
                let hash = crate::generation::trees::hash_tree_cell(cell_x, cell_z, generator.seed);
                let offset_x = (hash % 3) as i32 + 1;
                let offset_z = ((hash >> 2) % 3) as i32 + 1;
                let tx = cell_x * 5 + offset_x;
                let tz = cell_z * 5 + offset_z;

                // Ownership check: this LOD mesh owns and generates trees whose base trunk is within its bounds
                if tx < chunk_origin_x
                    || tx >= chunk_origin_x + region_size
                    || tz < chunk_origin_z
                    || tz >= chunk_origin_z + region_size
                {
                    continue;
                }

                let col_data = generator.sample_column(tx, tz);
                let Some((species, base_prob)) =
                    crate::generation::trees::biome_tree_profile(col_data.biome)
                else {
                    continue;
                };

                let roll = ((hash >> 4) & 0xFFFF) as f32 / 65535.0;
                if roll >= base_prob * generator.tree_density {
                    continue;
                }
                if col_data.is_cliff || col_data.surface_shape == crate::world::BlockShape::Stair {
                    continue;
                }

                let is_swamp_tree = species == crate::generation::trees::TreeSpecies::Mangrove
                    || (species == crate::generation::trees::TreeSpecies::Pine
                        && col_data.biome == crate::generation::BiomeType::CypressSwamp);

                let ty = if let Some(water_level) = col_data.water_level {
                    if !is_swamp_tree || water_level - col_data.terrain_height > 2 {
                        continue;
                    }
                    col_data.terrain_height
                } else {
                    col_data.terrain_height
                };

                let surface_mat = generator.surface_material_at(tx, ty, tz, col_data);
                if !crate::generation::trees::is_soil_valid_for_species(species, surface_mat) {
                    continue;
                }

                push_lod_tree(
                    &mut opaque_buffers,
                    tx,
                    ty,
                    tz,
                    species,
                    col_data.biome,
                    hash,
                    chunk_origin_x,
                    chunk_origin_z,
                    textures,
                );
            }
        }
    }

    ChunkMeshes {
        opaque: opaque_buffers.into_mesh(),
        transparent: transparent_buffers.into_mesh(),
        visibility_mask: (1u64 << 36) - 1,
    }
}

/// Emits a cuboid voxel tree (trunk + canopy) into the LOD buffers.
#[allow(clippy::too_many_arguments)]
fn push_lod_tree(
    buffers: &mut MeshBuffers,
    tx: i32,
    ty: i32,
    tz: i32,
    species: crate::generation::trees::TreeSpecies,
    col_biome: crate::generation::BiomeType,
    hash: u32,
    chunk_origin_x: i32,
    chunk_origin_z: i32,
    textures: &VoxelTextureRegistry,
) {
    use crate::generation::trees::TreeSpecies;

    let local_tx = (tx - chunk_origin_x) as f32;
    let local_tz = (tz - chunk_origin_z) as f32;
    let ground_y = (ty + 1) as f32;

    if species == TreeSpecies::Cactus {
        let height = 2 + ((hash >> 12) % 3) as i32;
        let top_y = ground_y + height as f32;
        let x0 = local_tx;
        let x1 = local_tx + 1.0;
        let z0 = local_tz;
        let z1 = local_tz + 1.0;

        let cactus_layer = textures
            .get_face_texture_info(
                Voxel::Tree_Cactus,
                IVec3::new(tx, ty + 1, tz),
                FaceDirection::PositiveX,
            )
            .0 as f32;

        push_lod_quad(
            buffers,
            FaceDirection::PositiveY,
            [x0, top_y, z0],
            [x0, top_y, z1],
            [x1, top_y, z1],
            [x1, top_y, z0],
            1.0,
            1.0,
            cactus_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::NegativeX,
            [x0, ground_y, z1],
            [x0, top_y, z1],
            [x0, top_y, z0],
            [x0, ground_y, z0],
            1.0,
            height as f32,
            cactus_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::PositiveX,
            [x1, ground_y, z0],
            [x1, top_y, z0],
            [x1, top_y, z1],
            [x1, ground_y, z1],
            1.0,
            height as f32,
            cactus_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::NegativeZ,
            [x0, ground_y, z0],
            [x0, top_y, z0],
            [x1, top_y, z0],
            [x1, ground_y, z0],
            1.0,
            height as f32,
            cactus_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::PositiveZ,
            [x1, ground_y, z1],
            [x1, top_y, z1],
            [x0, top_y, z1],
            [x0, ground_y, z1],
            1.0,
            height as f32,
            cactus_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        return;
    }

    if species == TreeSpecies::Dead || species == TreeSpecies::Charred {
        let height = 4 + ((hash >> 12) % 3) as i32;
        let top_y = ground_y + height as f32;
        let x0 = local_tx;
        let x1 = local_tx + 1.0;
        let z0 = local_tz;
        let z1 = local_tz + 1.0;

        let log_voxel = species.log_voxel();
        let log_side_layer = textures
            .get_face_texture_info(
                log_voxel,
                IVec3::new(tx, ty + 1, tz),
                FaceDirection::PositiveX,
            )
            .0 as f32;
        let log_top_layer = textures
            .get_face_texture_info(
                log_voxel,
                IVec3::new(tx, ty + height, tz),
                FaceDirection::PositiveY,
            )
            .0 as f32;

        push_lod_quad(
            buffers,
            FaceDirection::PositiveY,
            [x0, top_y, z0],
            [x0, top_y, z1],
            [x1, top_y, z1],
            [x1, top_y, z0],
            1.0,
            1.0,
            log_top_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::NegativeX,
            [x0, ground_y, z1],
            [x0, top_y, z1],
            [x0, top_y, z0],
            [x0, ground_y, z0],
            1.0,
            height as f32,
            log_side_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::PositiveX,
            [x1, ground_y, z0],
            [x1, top_y, z0],
            [x1, top_y, z1],
            [x1, ground_y, z1],
            1.0,
            height as f32,
            log_side_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::NegativeZ,
            [x0, ground_y, z0],
            [x0, top_y, z0],
            [x1, top_y, z0],
            [x1, ground_y, z0],
            1.0,
            height as f32,
            log_side_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::PositiveZ,
            [x1, ground_y, z1],
            [x1, top_y, z1],
            [x0, top_y, z1],
            [x0, ground_y, z1],
            1.0,
            height as f32,
            log_side_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        return;
    }

    // Standard Trees: Trunk + Stepped Canopy
    let height = 6 + ((hash >> 12) % 3) as i32;
    let canopy_base_y = ground_y + (height - 2) as f32;
    let canopy_top_y = ground_y + (height + 1) as f32;
    let dome_top_y = canopy_top_y + 1.0;

    let log_voxel = species.log_voxel();
    let leaf_voxel = crate::generation::trees::leaves_voxel_variant(species, hash, col_biome)
        .unwrap_or(Voxel::Tree_Oak_Leaves);

    // 1. Trunk (from ground to canopy bottom)
    let trunk_h = canopy_base_y - ground_y;
    if trunk_h > 0.0 {
        let x0 = local_tx;
        let x1 = local_tx + 1.0;
        let z0 = local_tz;
        let z1 = local_tz + 1.0;
        let log_layer = textures
            .get_face_texture_info(
                log_voxel,
                IVec3::new(tx, ty + 1, tz),
                FaceDirection::PositiveX,
            )
            .0 as f32;

        push_lod_quad(
            buffers,
            FaceDirection::NegativeX,
            [x0, ground_y, z1],
            [x0, canopy_base_y, z1],
            [x0, canopy_base_y, z0],
            [x0, ground_y, z0],
            1.0,
            trunk_h,
            log_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::PositiveX,
            [x1, ground_y, z0],
            [x1, canopy_base_y, z0],
            [x1, canopy_base_y, z1],
            [x1, ground_y, z1],
            1.0,
            trunk_h,
            log_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::NegativeZ,
            [x0, ground_y, z0],
            [x0, canopy_base_y, z0],
            [x1, canopy_base_y, z0],
            [x1, ground_y, z0],
            1.0,
            trunk_h,
            log_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
        push_lod_quad(
            buffers,
            FaceDirection::PositiveZ,
            [x1, ground_y, z1],
            [x1, canopy_base_y, z1],
            [x0, canopy_base_y, z1],
            [x0, ground_y, z1],
            1.0,
            trunk_h,
            log_layer,
            [1.0, 1.0, 1.0, 1.0],
        );
    }

    // 2. Main Canopy (3x3 blocks, 3m tall)
    let cx0 = local_tx - 1.0;
    let cx1 = local_tx + 2.0;
    let cz0 = local_tz - 1.0;
    let cz1 = local_tz + 2.0;
    let canopy_h = canopy_top_y - canopy_base_y;

    let leaf_layer = textures
        .get_face_texture_info(
            leaf_voxel,
            IVec3::new(tx, ty + height, tz),
            FaceDirection::PositiveY,
        )
        .0 as f32;
    let mut tint = leaf_voxel.tint_color_at(IVec3::new(tx, ty + height, tz));
    tint[3] = 1.0;

    // Canopy bottom face
    push_lod_quad(
        buffers,
        FaceDirection::NegativeY,
        [cx0, canopy_base_y, cz1],
        [cx0, canopy_base_y, cz0],
        [cx1, canopy_base_y, cz0],
        [cx1, canopy_base_y, cz1],
        3.0,
        3.0,
        leaf_layer,
        tint,
    );
    // Canopy top face
    push_lod_quad(
        buffers,
        FaceDirection::PositiveY,
        [cx0, canopy_top_y, cz0],
        [cx0, canopy_top_y, cz1],
        [cx1, canopy_top_y, cz1],
        [cx1, canopy_top_y, cz0],
        3.0,
        3.0,
        leaf_layer,
        tint,
    );
    // Canopy 4 sides
    push_lod_quad(
        buffers,
        FaceDirection::NegativeX,
        [cx0, canopy_base_y, cz1],
        [cx0, canopy_top_y, cz1],
        [cx0, canopy_top_y, cz0],
        [cx0, canopy_base_y, cz0],
        3.0,
        canopy_h,
        leaf_layer,
        tint,
    );
    push_lod_quad(
        buffers,
        FaceDirection::PositiveX,
        [cx1, canopy_base_y, cz0],
        [cx1, canopy_top_y, cz0],
        [cx1, canopy_top_y, cz1],
        [cx1, canopy_base_y, cz1],
        3.0,
        canopy_h,
        leaf_layer,
        tint,
    );
    push_lod_quad(
        buffers,
        FaceDirection::NegativeZ,
        [cx0, canopy_base_y, cz0],
        [cx0, canopy_top_y, cz0],
        [cx1, canopy_top_y, cz0],
        [cx1, canopy_base_y, cz0],
        3.0,
        canopy_h,
        leaf_layer,
        tint,
    );
    push_lod_quad(
        buffers,
        FaceDirection::PositiveZ,
        [cx1, canopy_base_y, cz1],
        [cx1, canopy_top_y, cz1],
        [cx0, canopy_top_y, cz1],
        [cx0, canopy_base_y, cz1],
        3.0,
        canopy_h,
        leaf_layer,
        tint,
    );

    // 3. Canopy Dome Cap (1x1 block, 1m tall on top of canopy)
    let dx0 = local_tx;
    let dx1 = local_tx + 1.0;
    let dz0 = local_tz;
    let dz1 = local_tz + 1.0;

    push_lod_quad(
        buffers,
        FaceDirection::PositiveY,
        [dx0, dome_top_y, dz0],
        [dx0, dome_top_y, dz1],
        [dx1, dome_top_y, dz1],
        [dx1, dome_top_y, dz0],
        1.0,
        1.0,
        leaf_layer,
        tint,
    );
    push_lod_quad(
        buffers,
        FaceDirection::NegativeX,
        [dx0, canopy_top_y, dz1],
        [dx0, dome_top_y, dz1],
        [dx0, dome_top_y, dz0],
        [dx0, canopy_top_y, dz0],
        1.0,
        1.0,
        leaf_layer,
        tint,
    );
    push_lod_quad(
        buffers,
        FaceDirection::PositiveX,
        [dx1, canopy_top_y, dz0],
        [dx1, dome_top_y, dz0],
        [dx1, dome_top_y, dz1],
        [dx1, canopy_top_y, dz1],
        1.0,
        1.0,
        leaf_layer,
        tint,
    );
    push_lod_quad(
        buffers,
        FaceDirection::NegativeZ,
        [dx0, canopy_top_y, dz0],
        [dx0, dome_top_y, dz0],
        [dx1, dome_top_y, dz0],
        [dx1, canopy_top_y, dz0],
        1.0,
        1.0,
        leaf_layer,
        tint,
    );
    push_lod_quad(
        buffers,
        FaceDirection::PositiveZ,
        [dx1, canopy_top_y, dz1],
        [dx1, dome_top_y, dz1],
        [dx0, dome_top_y, dz1],
        [dx0, canopy_top_y, dz1],
        1.0,
        1.0,
        leaf_layer,
        tint,
    );
}
