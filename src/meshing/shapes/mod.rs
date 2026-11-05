use bevy::prelude::*;

use super::{
    greedy::{FaceDirection, MeshBuffers},
    textures::VoxelTextureRegistry,
};
use crate::world::{BlockShape, CHUNK_SIZE, Chunk, VOXEL_SIZE, Voxel, VoxelAccess};

pub mod fixtures;
pub mod pillars_walls;
pub mod slabs_stairs;

pub use fixtures::*;
pub use pillars_walls::*;
pub use slabs_stairs::*;

pub fn mesh_shaped_voxels(
    world: &impl VoxelAccess,
    chunk: &Chunk,
    chunk_coordinate: IVec3,
    textures: &VoxelTextureRegistry,
    opaque_buffers: &mut MeshBuffers,
    transparent_buffers: &mut MeshBuffers,
) {
    let chunk_voxel_origin = chunk_coordinate * CHUNK_SIZE as i32;
    for (&index, &(shape, orientation)) in chunk.shapes() {
        let (x, y, z) = Chunk::index_to_xyz(index);
        let voxel = chunk.get(x, y, z);
        if voxel.is_empty() || shape == BlockShape::Full {
            continue;
        }
        let is_transparent_pass = (voxel.is_transparent() || voxel.is_fluid())
            && !voxel.is_cross_billboard()
            && shape != BlockShape::CrossBillboard
            && shape != BlockShape::FlatPad
            && shape != BlockShape::WallDecal
            && shape != BlockShape::Cactus
            && shape != BlockShape::Speleothem
            && !voxel.is_speleothem();
        let buffers = if is_transparent_pass {
            &mut *transparent_buffers
        } else {
            &mut *opaque_buffers
        };
        let world_voxel = chunk_voxel_origin + IVec3::new(x as i32, y as i32, z as i32);
        let (side_layer, frame_count) =
            textures.get_face_texture_info(voxel, world_voxel, FaceDirection::PositiveX);
        let (top_layer, _) =
            textures.get_face_texture_info(voxel, world_voxel, FaceDirection::PositiveY);
        let (bottom_layer, _) =
            textures.get_face_texture_info(voxel, world_voxel, FaceDirection::NegativeY);
        let rot_mode = if voxel.has_custom_mesh() || shape != BlockShape::Full {
            0
        } else {
            voxel.natural_rotation_mode() as u8
        };
        let frame_count_f32 = if voxel.is_light() {
            -(frame_count.max(1) as f32)
        } else if frame_count == 1 && rot_mode > 0 {
            1.0 + (rot_mode as f32) * 0.25
        } else {
            frame_count as f32
        };
        let mut tint = voxel.tint_color_at(world_voxel);
        tint[3] = if voxel.is_water() {
            1.0
        } else if voxel.is_fluid() {
            2.0
        } else if voxel.is_marine_flora() {
            4.0
        } else if voxel.is_flora_billboard() || voxel == Voxel::BambooLeaves {
            5.0
        } else if voxel.is_cross_billboard()
            || shape == BlockShape::CrossBillboard
            || voxel.is_speleothem()
            || shape == BlockShape::Speleothem
        {
            6.0
        } else if is_transparent_pass {
            0.0
        } else if voxel.is_painted() {
            0.6
        } else {
            1.0
        };
        let top_tint = tint;
        let is_grass = voxel.is_grass_soil();
        let is_turf = voxel.has_surface_turf();
        let bottom_tint = if is_grass {
            [1.0, 1.0, 1.0, tint[3]]
        } else {
            tint
        };
        let side_tint = tint;
        let is_neighbor_solid = |offset: IVec3| -> bool {
            let neighbor_pos = world_voxel + offset;
            let (n_shape, _) = world.get_shape(neighbor_pos);
            world
                .get_voxel(neighbor_pos)
                .is_some_and(|v| v.is_solid_opaque() && n_shape == BlockShape::Full)
        };
        let fx = x as f32;
        let fy = y as f32;
        let fz = z as f32;
        if voxel.is_basket() || shape == BlockShape::Basket {
            mesh_basket(
                buffers,
                fx,
                fy,
                fz,
                top_layer,
                bottom_layer,
                side_layer,
                frame_count_f32,
                top_tint,
                bottom_tint,
                side_tint,
                &is_neighbor_solid,
            );
            continue;
        }
        if voxel.is_torch() || shape == BlockShape::Torch {
            mesh_torch(
                buffers,
                fx,
                fy,
                fz,
                orientation,
                side_layer,
                frame_count_f32,
                side_tint,
            );
            continue;
        }
        if voxel.is_pebble()
            || matches!(
                shape,
                BlockShape::Pebble | BlockShape::PebbleCluster
            )
        {
            let pebble_shape = if matches!(
                shape,
                BlockShape::Pebble | BlockShape::PebbleCluster
            ) {
                shape
            } else {
                voxel.default_shape()
            };
            mesh_pebble(
                buffers,
                fx,
                fy,
                fz,
                pebble_shape,
                orientation,
                side_layer,
                frame_count_f32,
                side_tint,
            );
            continue;
        }
        if voxel.is_rubble() || shape == BlockShape::Rubble {
            mesh_rubble(
                buffers,
                fx,
                fy,
                fz,
                orientation,
                side_layer,
                frame_count_f32,
                side_tint,
            );
            continue;
        }
        if voxel.is_speleothem() || shape == BlockShape::Speleothem {
            let (dir, piece) = determine_speleothem_part(world, world_voxel, voxel);
            let layer = if let Some(layers) = textures.get_speleothem_layers(voxel) {
                match (dir, piece) {
                    (SpeleothemDirection::Down, SpeleothemPiece::Base) => layers.down_base,
                    (SpeleothemDirection::Down, SpeleothemPiece::Frustum) => layers.down_frustum,
                    (SpeleothemDirection::Down, SpeleothemPiece::Middle) => layers.down_middle,
                    (SpeleothemDirection::Down, SpeleothemPiece::Tip) => layers.down_tip,
                    (SpeleothemDirection::Down, SpeleothemPiece::TipMerge) => layers.down_tip_merge,
                    (SpeleothemDirection::Up, SpeleothemPiece::Base) => layers.up_base,
                    (SpeleothemDirection::Up, SpeleothemPiece::Frustum) => layers.up_frustum,
                    (SpeleothemDirection::Up, SpeleothemPiece::Middle) => layers.up_middle,
                    (SpeleothemDirection::Up, SpeleothemPiece::Tip) => layers.up_tip,
                    (SpeleothemDirection::Up, SpeleothemPiece::TipMerge) => layers.up_tip_merge,
                }
            } else {
                side_layer
            };
            mesh_speleothem(
                buffers,
                fx,
                fy,
                fz,
                layer,
                frame_count_f32,
                [1.0, 1.0, 1.0, 6.0],
            );
            continue;
        }
        if voxel.is_bamboo() || shape == BlockShape::Bamboo {
            let is_neighbor_bamboo = |offset: IVec3| -> bool {
                let neighbor_pos = world_voxel + offset;
                world
                    .get_voxel(neighbor_pos)
                    .is_some_and(|v| v.is_bamboo())
            };
            let cull_top = is_neighbor_bamboo(IVec3::Y) || is_neighbor_solid(IVec3::Y);
            let cull_bottom = is_neighbor_bamboo(IVec3::NEG_Y) || is_neighbor_solid(IVec3::NEG_Y);
            if voxel == Voxel::BambooLeaves {
                // Central 3px stalk inside the leaves block
                let (stalk_side_layer, _) = textures.get_face_texture_info(
                    Voxel::BambooStalk,
                    world_voxel,
                    FaceDirection::PositiveX,
                );
                let (stalk_top_layer, _) = textures.get_face_texture_info(
                    Voxel::BambooStalk,
                    world_voxel,
                    FaceDirection::PositiveY,
                );
                let (stalk_bottom_layer, _) = textures.get_face_texture_info(
                    Voxel::BambooStalk,
                    world_voxel,
                    FaceDirection::NegativeY,
                );
                mesh_bamboo(
                    buffers,
                    fx,
                    fy,
                    fz,
                    stalk_side_layer,
                    stalk_top_layer,
                    stalk_bottom_layer,
                    frame_count_f32,
                    side_tint,
                    top_tint,
                    bottom_tint,
                    cull_top,
                    cull_bottom,
                );
                // Foliage leaves cross mesh surrounding the central stalk
                mesh_cross_billboard(
                    buffers,
                    fx,
                    fy,
                    fz,
                    side_layer,
                    frame_count_f32,
                    side_tint,
                );
            } else {
                mesh_bamboo(
                    buffers,
                    fx,
                    fy,
                    fz,
                    side_layer,
                    top_layer,
                    bottom_layer,
                    frame_count_f32,
                    side_tint,
                    top_tint,
                    bottom_tint,
                    cull_top,
                    cull_bottom,
                );
            }
            continue;
        }
        if voxel.is_cross_billboard() || shape == BlockShape::CrossBillboard {
            mesh_cross_billboard(
                buffers,
                fx,
                fy,
                fz,
                side_layer,
                frame_count_f32,
                side_tint,
            );
            if voxel.is_marine_flora() || voxel.is_coral_fan() {
                mesh_waterlogged_envelope(
                    world,
                    transparent_buffers,
                    world_voxel,
                    fx,
                    fy,
                    fz,
                    textures,
                );
            }
            continue;
        }
        if shape == BlockShape::FlatPad {
            mesh_flat_pad(
                buffers,
                fx,
                fy,
                fz,
                voxel,
                side_layer,
                frame_count_f32,
                top_tint,
                textures.flower_carpet_stem,
            );
            continue;
        }
        if shape == BlockShape::WallDecal {
            mesh_wall_decal(
                buffers,
                fx,
                fy,
                fz,
                orientation,
                side_layer,
                frame_count_f32,
                top_tint,
            );
            continue;
        }
        if shape == BlockShape::Cactus {
            let is_neighbor_cactus_or_opaque = |offset: IVec3| -> bool {
                let neighbor_pos = world_voxel + offset;
                world
                    .get_voxel(neighbor_pos)
                    .is_some_and(|v| v == Voxel::VegeCactus || v.is_solid_opaque())
            };
            mesh_cactus(
                buffers,
                fx,
                fy,
                fz,
                top_layer,
                bottom_layer,
                side_layer,
                frame_count_f32,
                top_tint,
                bottom_tint,
                side_tint,
                &is_neighbor_cactus_or_opaque,
            );
            continue;
        }
        let (box_a, maybe_box_b) = shape.local_boxes(orientation);
        match shape {
            BlockShape::Torch
            | BlockShape::Basket
            | BlockShape::Pebble
            | BlockShape::PebbleCluster
            | BlockShape::Bamboo
            | BlockShape::CrossBillboard
            | BlockShape::FlatPad
            | BlockShape::WallDecal
            | BlockShape::Cactus
            | BlockShape::Rubble
            | BlockShape::Speleothem => unreachable!(),
            BlockShape::Full | BlockShape::Column | BlockShape::Cocoon => {
                mesh_column_cocoon(
                    buffers,
                    fx,
                    fy,
                    fz,
                    box_a,
                    top_layer,
                    bottom_layer,
                    side_layer,
                    frame_count_f32,
                    top_tint,
                    bottom_tint,
                    side_tint,
                    &is_neighbor_solid,
                    is_grass,
                );
            }
            BlockShape::Slab => {
                mesh_slab(
                    world,
                    voxel,
                    is_transparent_pass,
                    transparent_buffers,
                    opaque_buffers,
                    chunk,
                    x,
                    y,
                    z,
                    orientation,
                    fx,
                    fy,
                    fz,
                    box_a,
                    top_layer,
                    bottom_layer,
                    side_layer,
                    frame_count_f32,
                    top_tint,
                    bottom_tint,
                    side_tint,
                    &is_neighbor_solid,
                    is_turf,
                    textures,
                    world_voxel,
                );
            }
            BlockShape::Stair => {
                mesh_stair(
                    world,
                    world_voxel,
                    voxel,
                    textures,
                    buffers,
                    fx,
                    fy,
                    fz,
                    orientation,
                    box_a,
                    maybe_box_b,
                    top_layer,
                    bottom_layer,
                    side_layer,
                    frame_count_f32,
                    top_tint,
                    bottom_tint,
                    side_tint,
                    &is_neighbor_solid,
                    is_turf,
                );
            }
        }
    }
}

/// Pushes an axis-aligned shape sub-box with unified boundary neighbor culling.
/// cull order: [+X, -X, +Y, -Y, +Z, -Z]
pub fn push_shape_box(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    local_min: Vec3,
    local_max: Vec3,
    internal_cull: [bool; 6],
    top_layer: u16,
    bottom_layer: u16,
    side_layer: u16,
    frame_count: f32,
    top_tint: [f32; 4],
    bottom_tint: [f32; 4],
    side_tint: [f32; 4],
    is_neighbor_solid: &impl Fn(IVec3) -> bool,
    align_side_uv_to_top: bool,
) {
    push_shape_box_with_transition(
        buffers,
        fx,
        fy,
        fz,
        local_min,
        local_max,
        internal_cull,
        top_layer,
        bottom_layer,
        side_layer,
        frame_count,
        top_tint,
        bottom_tint,
        side_tint,
        is_neighbor_solid,
        align_side_uv_to_top,
        None,
        Voxel::Air,
    );
}

pub fn push_shape_box_with_transition(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    local_min: Vec3,
    local_max: Vec3,
    internal_cull: [bool; 6],
    top_layer: u16,
    bottom_layer: u16,
    side_layer: u16,
    frame_count: f32,
    top_tint: [f32; 4],
    bottom_tint: [f32; 4],
    side_tint: [f32; 4],
    is_neighbor_solid: &impl Fn(IVec3) -> bool,
    align_side_uv_to_top: bool,
    top_transition: Option<(u16, u8, u32)>,
    voxel: Voxel,
) {
    let min_x = fx + local_min.x;
    let min_y = fy + local_min.y;
    let min_z = fz + local_min.z;
    let max_x = fx + local_max.x;
    let max_y = fy + local_max.y;
    let max_z = fz + local_max.z;
    let mut cull = internal_cull;
    if local_max.x >= 0.999 && is_neighbor_solid(IVec3::X) {
        cull[0] = true;
    }
    if local_min.x <= 0.001 && is_neighbor_solid(IVec3::NEG_X) {
        cull[1] = true;
    }
    if local_max.y >= 0.999 && is_neighbor_solid(IVec3::Y) {
        cull[2] = true;
    }
    if local_min.y <= 0.001 && is_neighbor_solid(IVec3::NEG_Y) {
        cull[3] = true;
    }
    if local_max.z >= 0.999 && is_neighbor_solid(IVec3::Z) {
        cull[4] = true;
    }
    if local_min.z <= 0.001 && is_neighbor_solid(IVec3::NEG_Z) {
        cull[5] = true;
    }
    let eps = 0.0005;
    let (v_top, v_bot) = if align_side_uv_to_top && local_max.y <= 0.501 {
        (eps, ((local_max.y - local_min.y).clamp(0.0, 1.0) - eps).max(eps))
    } else {
        (
            ((1.0 - local_max.y).clamp(0.0, 1.0) + eps).min(1.0 - eps),
            ((1.0 - local_min.y).clamp(0.0, 1.0) - eps).max(eps),
        )
    };
    let side_uvs = [[eps, v_bot], [eps, v_top], [1.0 - eps, v_top], [1.0 - eps, v_bot]];
    // +X (East, idx 0)
    if !cull[0] {
        push_quad_face_uvs(
            buffers,
            [
                [max_x, min_y, min_z],
                [max_x, max_y, min_z],
                [max_x, max_y, max_z],
                [max_x, min_y, max_z],
            ],
            [1.0, 0.0, 0.0],
            side_layer,
            frame_count,
            side_tint,
            side_uvs,
        );
    }
    // -X (West, idx 1)
    if !cull[1] {
        push_quad_face_uvs(
            buffers,
            [
                [min_x, min_y, max_z],
                [min_x, max_y, max_z],
                [min_x, max_y, min_z],
                [min_x, min_y, min_z],
            ],
            [-1.0, 0.0, 0.0],
            side_layer,
            frame_count,
            side_tint,
            side_uvs,
        );
    }
    // +Y (Top, idx 2)
    if !cull[2] {
        if let Some((overlay_layer, mask_index, packed_tint)) = top_transition {
            if mask_index > 0 {
                push_quad_face_mask(
                    buffers,
                    [
                        [min_x, max_y, min_z],
                        [min_x, max_y, max_z],
                        [max_x, max_y, max_z],
                        [max_x, max_y, min_z],
                    ],
                    [0.0, 1.0, 0.0],
                    top_layer,
                    frame_count,
                    top_tint,
                    overlay_layer,
                    mask_index,
                    packed_tint,
                    voxel,
                );
            } else {
                push_quad_face(
                    buffers,
                    [
                        [min_x, max_y, min_z],
                        [min_x, max_y, max_z],
                        [max_x, max_y, max_z],
                        [max_x, max_y, min_z],
                    ],
                    [0.0, 1.0, 0.0],
                    top_layer,
                    frame_count,
                    top_tint,
                );
            }
        } else {
            push_quad_face(
                buffers,
                [
                    [min_x, max_y, min_z],
                    [min_x, max_y, max_z],
                    [max_x, max_y, max_z],
                    [max_x, max_y, min_z],
                ],
                [0.0, 1.0, 0.0],
                top_layer,
                frame_count,
                top_tint,
            );
        }
    }
    // -Y (Bottom, idx 3)
    if !cull[3] {
        push_quad_face(
            buffers,
            [
                [min_x, min_y, max_z],
                [min_x, min_y, min_z],
                [max_x, min_y, min_z],
                [max_x, min_y, max_z],
            ],
            [0.0, -1.0, 0.0],
            bottom_layer,
            frame_count,
            bottom_tint,
        );
    }
    // +Z (South, idx 4)
    if !cull[4] {
        push_quad_face_uvs(
            buffers,
            [
                [max_x, min_y, max_z],
                [max_x, max_y, max_z],
                [min_x, max_y, max_z],
                [min_x, min_y, max_z],
            ],
            [0.0, 0.0, 1.0],
            side_layer,
            frame_count,
            side_tint,
            side_uvs,
        );
    }
    // -Z (North, idx 5)
    if !cull[5] {
        push_quad_face_uvs(
            buffers,
            [
                [min_x, min_y, min_z],
                [min_x, max_y, min_z],
                [max_x, max_y, min_z],
                [max_x, min_y, min_z],
            ],
            [0.0, 0.0, -1.0],
            side_layer,
            frame_count,
            side_tint,
            side_uvs,
        );
    }
}

pub fn push_quad_face_mask(
    buffers: &mut MeshBuffers,
    vertices: [[f32; 3]; 4],
    normal: [f32; 3],
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
    overlay_layer: u16,
    mask_index: u8,
    packed_tint: u32,
    voxel: Voxel,
) {
    let eps = 0.0005;
    let uvs = if normal[1].abs() > 0.5 {
        [[eps, eps], [eps, 1.0 - eps], [1.0 - eps, 1.0 - eps], [1.0 - eps, eps]]
    } else {
        [[eps, 1.0 - eps], [eps, eps], [1.0 - eps, eps], [1.0 - eps, 1.0 - eps]]
    };

    let (tint, base_tinted) = if packed_tint != 0 {
        (packed_tint, 0u32)
    } else if voxel.is_grass_soil() {
        (crate::meshing::greedy::pack_tint7(tint_color), 1u32)
    } else {
        (0u32, 0u32)
    };
    let packed = (tint & 0x1F_FFFF) | (base_tinted << 21);
    let mask_alpha = -(1.0 + packed as f32);

    let base_index = buffers.positions.len() as u32;
    for v in &vertices {
        buffers
            .positions
            .push([v[0] * VOXEL_SIZE, v[1] * VOXEL_SIZE, v[2] * VOXEL_SIZE]);
        buffers.normals.push(normal);
        buffers.colors.push([1.0, 1.0, 1.0, mask_alpha]);
        let uv_b_x = texture_layer as f32 + (overlay_layer as f32) * 2048.0;
        let uv_b_y = if mask_index > 0 {
            (mask_index as f32) * 16.0 + frame_count
        } else {
            frame_count
        };
        buffers.uv_bs.push([uv_b_x, uv_b_y]);
    }
    buffers.uvs.extend_from_slice(&uvs);
    buffers.indices.extend_from_slice(&[
        base_index,
        base_index + 1,
        base_index + 2,
        base_index,
        base_index + 2,
        base_index + 3,
    ]);
}

pub fn push_quad_face(
    buffers: &mut MeshBuffers,
    vertices: [[f32; 3]; 4],
    normal: [f32; 3],
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    let eps = 0.0005;
    let uvs = if normal[1].abs() > 0.5 {
        [[eps, eps], [eps, 1.0 - eps], [1.0 - eps, 1.0 - eps], [1.0 - eps, eps]]
    } else {
        [[eps, 1.0 - eps], [eps, eps], [1.0 - eps, eps], [1.0 - eps, 1.0 - eps]]
    };
    push_quad_face_uvs(buffers, vertices, normal, texture_layer, frame_count, tint_color, uvs);
}

pub fn push_quad_face_uvs(
    buffers: &mut MeshBuffers,
    vertices: [[f32; 3]; 4],
    normal: [f32; 3],
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
    uvs: [[f32; 2]; 4],
) {
    let base_index = buffers.positions.len() as u32;
    for v in &vertices {
        buffers
            .positions
            .push([v[0] * VOXEL_SIZE, v[1] * VOXEL_SIZE, v[2] * VOXEL_SIZE]);
        buffers.normals.push(normal);
        buffers.colors.push(tint_color);
        buffers.uv_bs.push([texture_layer as f32, frame_count]);
    }
    buffers.uvs.extend_from_slice(&uvs);
    buffers.indices.extend_from_slice(&[
        base_index,
        base_index + 1,
        base_index + 2,
        base_index,
        base_index + 2,
        base_index + 3,
    ]);
}

pub fn push_quad_face_with_uvs(
    buffers: &mut MeshBuffers,
    vertices: [[f32; 3]; 4],
    normal: [f32; 3],
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
    uvs: [[f32; 2]; 4],
) {
    push_quad_face_uvs(buffers, vertices, normal, texture_layer, frame_count, tint_color, uvs);
}

fn mesh_waterlogged_envelope(
    world: &impl VoxelAccess,
    transparent_buffers: &mut MeshBuffers,
    world_voxel: IVec3,
    fx: f32,
    fy: f32,
    fz: f32,
    textures: &VoxelTextureRegistry,
) {
    let water_voxel = Voxel::LiquidWater;
    let (water_layer, water_frames) =
        textures.get_face_texture_info(water_voxel, world_voxel, FaceDirection::PositiveY);
    let mut water_tint = water_voxel.tint_color_at(world_voxel);
    water_tint[3] = 1.0;
    let frame_count_f32 = water_frames as f32;

    let is_submerged_boundary = |offset: IVec3| -> bool {
        let n_pos = world_voxel + offset;
        let n_voxel = world.get_voxel(n_pos).unwrap_or(Voxel::Air);
        n_voxel.is_water() || n_voxel.is_marine_flora() || n_voxel.is_coral_fan()
    };

    let is_solid_opaque = |offset: IVec3| -> bool {
        let n_pos = world_voxel + offset;
        let n_voxel = world.get_voxel(n_pos).unwrap_or(Voxel::Air);
        n_voxel.is_solid_opaque()
    };

    let is_at_surface = !is_submerged_boundary(IVec3::Y);
    let surface_offset = if is_at_surface { 0.10 } else { 0.0 };

    let min_x = fx;
    let max_x = fx + 1.0;
    let min_y = fy;
    let max_y = fy + 1.0 - surface_offset;
    let min_z = fz;
    let max_z = fz + 1.0;

    // +Y (Water surface) - rendered if air or non-submerged block is above
    if is_at_surface {
        push_quad_face_uvs(
            transparent_buffers,
            [
                [min_x, max_y, min_z],
                [min_x, max_y, max_z],
                [max_x, max_y, max_z],
                [max_x, max_y, min_z],
            ],
            [0.0, 1.0, 0.0],
            water_layer,
            frame_count_f32,
            water_tint,
            [
                [fx, fz],
                [fx, fz + 1.0],
                [fx + 1.0, fz + 1.0],
                [fx + 1.0, fz],
            ],
        );
    }

    // -Y (Bottom) - only if bordering air below
    if !is_submerged_boundary(IVec3::NEG_Y) && !is_solid_opaque(IVec3::NEG_Y) {
        push_quad_face(
            transparent_buffers,
            [
                [min_x, min_y, max_z],
                [min_x, min_y, min_z],
                [max_x, min_y, min_z],
                [max_x, min_y, max_z],
            ],
            [0.0, -1.0, 0.0],
            water_layer,
            frame_count_f32,
            water_tint,
        );
    }

    let is_exposed_lateral = |offset: IVec3| -> bool {
        !is_submerged_boundary(offset) && !is_solid_opaque(offset)
    };

    // +X (East)
    if is_exposed_lateral(IVec3::X) {
        push_quad_face(
            transparent_buffers,
            [
                [max_x, min_y, min_z],
                [max_x, max_y, min_z],
                [max_x, max_y, max_z],
                [max_x, min_y, max_z],
            ],
            [1.0, 0.0, 0.0],
            water_layer,
            frame_count_f32,
            water_tint,
        );
    }

    // -X (West)
    if is_exposed_lateral(IVec3::NEG_X) {
        push_quad_face(
            transparent_buffers,
            [
                [min_x, min_y, max_z],
                [min_x, max_y, max_z],
                [min_x, max_y, min_z],
                [min_x, min_y, min_z],
            ],
            [-1.0, 0.0, 0.0],
            water_layer,
            frame_count_f32,
            water_tint,
        );
    }

    // +Z (South)
    if is_exposed_lateral(IVec3::Z) {
        push_quad_face(
            transparent_buffers,
            [
                [max_x, min_y, max_z],
                [max_x, max_y, max_z],
                [min_x, max_y, max_z],
                [min_x, min_y, max_z],
            ],
            [0.0, 0.0, 1.0],
            water_layer,
            frame_count_f32,
            water_tint,
        );
    }

    // -Z (North)
    if is_exposed_lateral(IVec3::NEG_Z) {
        push_quad_face(
            transparent_buffers,
            [
                [min_x, min_y, min_z],
                [min_x, max_y, min_z],
                [max_x, max_y, min_z],
                [max_x, min_y, min_z],
            ],
            [0.0, 0.0, -1.0],
            water_layer,
            frame_count_f32,
            water_tint,
        );
    }
}
