use bevy::prelude::*;

use super::{
    greedy::{FaceDirection, MeshBuffers},
    textures::VoxelTextureRegistry,
};
use crate::world::{BlockShape, CHUNK_SIZE, Chunk, VOXEL_SIZE, Voxel, VoxelAccess};

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
            && shape != BlockShape::CrossBillboard;

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
        } else if is_transparent_pass {
            0.0
        } else if voxel.is_painted() {
            0.5
        } else {
            1.0
        };
        let top_tint = tint;
        let is_grass = voxel.is_grass_soil();
        let bottom_tint = if is_grass {
            [1.0, 1.0, 1.0, tint[3]]
        } else {
            tint
        };
        let side_tint = if is_grass && !textures.full_grass {
            [1.0, 1.0, 1.0, tint[3]]
        } else {
            tint
        };

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
            continue;
        }

        let (box_a, maybe_box_b) = shape.local_boxes(orientation);

        match shape {
            BlockShape::Torch
            | BlockShape::Basket
            | BlockShape::Pebble
            | BlockShape::PebbleCluster
            | BlockShape::Bamboo
            | BlockShape::CrossBillboard => unreachable!(),
            BlockShape::Full | BlockShape::Column | BlockShape::Cocoon => {
                push_shape_box(
                    buffers,
                    fx,
                    fy,
                    fz,
                    box_a[0],
                    box_a[1],
                    [false; 6],
                    top_layer,
                    bottom_layer,
                    side_layer,
                    frame_count_f32,
                    top_tint,
                    bottom_tint,
                    side_tint,
                    &is_neighbor_solid,
                );
            }
            BlockShape::Slab => {
                let extra = chunk.get_extra_slab(x, y, z);
                let mut base_cull = [false; 6];
                if let Some((_extra_voxel, extra_orient)) = extra {
                    if orientation == 0 && extra_orient == 1 {
                        base_cull[2] = true; // +Y face touches upper slab
                    } else if orientation == 1 && extra_orient == 0 {
                        base_cull[3] = true; // -Y face touches lower slab
                    }
                }

                push_shape_box(
                    buffers,
                    fx,
                    fy,
                    fz,
                    box_a[0],
                    box_a[1],
                    base_cull,
                    top_layer,
                    bottom_layer,
                    side_layer,
                    frame_count_f32,
                    top_tint,
                    bottom_tint,
                    side_tint,
                    &is_neighbor_solid,
                );

                if let Some((extra_voxel, extra_orient)) = extra {
                    let (extra_side_layer, extra_frame_count) = textures.get_face_texture_info(
                        extra_voxel,
                        world_voxel,
                        FaceDirection::PositiveX,
                    );
                    let (extra_top_layer, _) = textures.get_face_texture_info(
                        extra_voxel,
                        world_voxel,
                        FaceDirection::PositiveY,
                    );
                    let (extra_bottom_layer, _) = textures.get_face_texture_info(
                        extra_voxel,
                        world_voxel,
                        FaceDirection::NegativeY,
                    );

                    let extra_frame_count_f32 = if extra_voxel.is_light() {
                        -(extra_frame_count.max(1) as f32)
                    } else {
                        extra_frame_count as f32
                    };

                    let mut extra_tint = extra_voxel.tint_color_at(world_voxel);
                    extra_tint[3] = if extra_voxel.is_water() {
                        1.0
                    } else if extra_voxel.is_fluid() {
                        2.0
                    } else if extra_voxel.is_transparent() {
                        0.0
                    } else if extra_voxel.is_painted() {
                        0.5
                    } else {
                        1.0
                    };
                    let extra_top_tint = extra_tint;
                    let is_extra_grass = extra_voxel.is_grass_soil();
                    let extra_bottom_tint = if is_extra_grass {
                        [1.0, 1.0, 1.0, extra_tint[3]]
                    } else {
                        extra_tint
                    };
                    let extra_side_tint = if is_extra_grass && !textures.full_grass {
                        [1.0, 1.0, 1.0, extra_tint[3]]
                    } else {
                        extra_tint
                    };

                    let mut extra_cull = [false; 6];
                    if orientation == 0 && extra_orient == 1 {
                        extra_cull[3] = true; // -Y face touches lower slab
                    } else if orientation == 1 && extra_orient == 0 {
                        extra_cull[2] = true; // +Y face touches upper slab
                    }

                    let (extra_box_a, _) = BlockShape::Slab.local_boxes(extra_orient);
                    let extra_buffers = if extra_voxel.is_transparent() || extra_voxel.is_fluid() {
                        &mut *transparent_buffers
                    } else {
                        &mut *opaque_buffers
                    };

                    push_shape_box(
                        extra_buffers,
                        fx,
                        fy,
                        fz,
                        extra_box_a[0],
                        extra_box_a[1],
                        extra_cull,
                        extra_top_layer,
                        extra_bottom_layer,
                        extra_side_layer,
                        extra_frame_count_f32,
                        extra_top_tint,
                        extra_bottom_tint,
                        extra_side_tint,
                        &is_neighbor_solid,
                    );
                }
            }
            BlockShape::Stair => {
                let is_inverted = orientation >= 4;
                let step_internal_cull = if !is_inverted { 3 } else { 2 };
                let mut step_cull = [false; 6];
                step_cull[step_internal_cull] = true;

                push_shape_box(
                    buffers,
                    fx,
                    fy,
                    fz,
                    box_a[0],
                    box_a[1],
                    [false; 6],
                    top_layer,
                    bottom_layer,
                    side_layer,
                    frame_count_f32,
                    top_tint,
                    bottom_tint,
                    side_tint,
                    &is_neighbor_solid,
                );
                if let Some(box_b) = maybe_box_b {
                    push_shape_box(
                        buffers,
                        fx,
                        fy,
                        fz,
                        box_b[0],
                        box_b[1],
                        step_cull,
                        top_layer,
                        bottom_layer,
                        side_layer,
                        frame_count_f32,
                        top_tint,
                        bottom_tint,
                        side_tint,
                        &is_neighbor_solid,
                    );
                }
            }
        }
    }
}

/// Pushes an axis-aligned shape sub-box with unified boundary neighbor culling.
/// cull order: [+X, -X, +Y, -Y, +Z, -Z]
fn push_shape_box(
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

    let v_top = (1.0 - local_max.y).clamp(0.0, 1.0);
    let v_bot = (1.0 - local_min.y).clamp(0.0, 1.0);
    let side_uvs = [[0.0, v_bot], [0.0, v_top], [1.0, v_top], [1.0, v_bot]];

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

fn push_quad_face(
    buffers: &mut MeshBuffers,
    vertices: [[f32; 3]; 4],
    normal: [f32; 3],
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    let uvs = if normal[1].abs() > 0.5 {
        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
    } else {
        [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
    };
    push_quad_face_uvs(buffers, vertices, normal, texture_layer, frame_count, tint_color, uvs);
}

fn push_quad_face_uvs(
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

fn push_quad_face_with_uvs(
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

fn mesh_basket(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    top_layer: u16,
    bottom_layer: u16,
    side_layer: u16,
    frame_count: f32,
    top_tint: [f32; 4],
    bottom_tint: [f32; 4],
    side_tint: [f32; 4],
    is_neighbor_solid: &impl Fn(IVec3) -> bool,
) {
    // 1. Exterior 6 faces (culls against adjacent solid blocks)
    push_shape_box(
        buffers,
        fx,
        fy,
        fz,
        Vec3::ZERO,
        Vec3::ONE,
        [false; 6],
        top_layer,
        bottom_layer,
        side_layer,
        frame_count,
        top_tint,
        bottom_tint,
        side_tint,
        is_neighbor_solid,
    );

    // 2. Interior cavity (rendered if the top is open, so looking inside reveals the walls and floor)
    if !is_neighbor_solid(IVec3::Y) {
        let x0 = fx + 2.0 / 16.0;
        let x1 = fx + 14.0 / 16.0;
        let z0 = fz + 2.0 / 16.0;
        let z1 = fz + 14.0 / 16.0;
        let y_floor = fy + 2.0 / 16.0;
        let y_top = fy + 1.0;

        let uv_min = 2.0 / 16.0;
        let uv_max = 14.0 / 16.0;

        // Interior floor (+Y, facing up inside cavity)
        push_quad_face_with_uvs(
            buffers,
            [
                [x0, y_floor, z0],
                [x0, y_floor, z1],
                [x1, y_floor, z1],
                [x1, y_floor, z0],
            ],
            [0.0, 1.0, 0.0],
            bottom_layer,
            frame_count,
            bottom_tint,
            [
                [uv_min, uv_min],
                [uv_min, uv_max],
                [uv_max, uv_max],
                [uv_max, uv_min],
            ],
        );

        // Interior North wall (at z = z0, facing +Z inside cavity)
        push_quad_face_with_uvs(
            buffers,
            [
                [x1, y_floor, z0],
                [x1, y_top, z0],
                [x0, y_top, z0],
                [x0, y_floor, z0],
            ],
            [0.0, 0.0, 1.0],
            side_layer,
            frame_count,
            side_tint,
            [[uv_max, 1.0], [uv_max, 0.0], [uv_min, 0.0], [uv_min, 1.0]],
        );

        // Interior South wall (at z = z1, facing -Z inside cavity)
        push_quad_face_with_uvs(
            buffers,
            [
                [x0, y_floor, z1],
                [x0, y_top, z1],
                [x1, y_top, z1],
                [x1, y_floor, z1],
            ],
            [0.0, 0.0, -1.0],
            side_layer,
            frame_count,
            side_tint,
            [[uv_min, 1.0], [uv_min, 0.0], [uv_max, 0.0], [uv_max, 1.0]],
        );

        // Interior West wall (at x = x0, facing +X inside cavity)
        push_quad_face_with_uvs(
            buffers,
            [
                [x0, y_floor, z0],
                [x0, y_top, z0],
                [x0, y_top, z1],
                [x0, y_floor, z1],
            ],
            [1.0, 0.0, 0.0],
            side_layer,
            frame_count,
            side_tint,
            [[uv_min, 1.0], [uv_min, 0.0], [uv_max, 0.0], [uv_max, 1.0]],
        );

        // Interior East wall (at x = x1, facing -X inside cavity)
        push_quad_face_with_uvs(
            buffers,
            [
                [x1, y_floor, z1],
                [x1, y_top, z1],
                [x1, y_top, z0],
                [x1, y_floor, z0],
            ],
            [-1.0, 0.0, 0.0],
            side_layer,
            frame_count,
            side_tint,
            [[uv_max, 1.0], [uv_max, 0.0], [uv_min, 0.0], [uv_min, 1.0]],
        );
    }
}

fn push_torch_stick_box(
    buffers: &mut MeshBuffers,
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    let u_min = 0.0;
    let u_max = 2.0 / 16.0;
    let v_min = 0.0;
    let v_max = 14.0 / 16.0;

    // +X face
    push_quad_face_with_uvs(
        buffers,
        [
            [max_x, min_y, min_z],
            [max_x, max_y, min_z],
            [max_x, max_y, max_z],
            [max_x, min_y, max_z],
        ],
        [1.0, 0.0, 0.0],
        texture_layer,
        frame_count,
        tint_color,
        [
            [u_min, v_max],
            [u_min, v_min],
            [u_max, v_min],
            [u_max, v_max],
        ],
    );

    // -X face
    push_quad_face_with_uvs(
        buffers,
        [
            [min_x, min_y, max_z],
            [min_x, max_y, max_z],
            [min_x, max_y, min_z],
            [min_x, min_y, min_z],
        ],
        [-1.0, 0.0, 0.0],
        texture_layer,
        frame_count,
        tint_color,
        [
            [u_min, v_max],
            [u_min, v_min],
            [u_max, v_min],
            [u_max, v_max],
        ],
    );

    // +Z face
    push_quad_face_with_uvs(
        buffers,
        [
            [max_x, min_y, max_z],
            [max_x, max_y, max_z],
            [min_x, max_y, max_z],
            [min_x, min_y, max_z],
        ],
        [0.0, 0.0, 1.0],
        texture_layer,
        frame_count,
        tint_color,
        [
            [u_min, v_max],
            [u_min, v_min],
            [u_max, v_min],
            [u_max, v_max],
        ],
    );

    // -Z face
    push_quad_face_with_uvs(
        buffers,
        [
            [min_x, min_y, min_z],
            [min_x, max_y, min_z],
            [max_x, max_y, min_z],
            [max_x, min_y, min_z],
        ],
        [0.0, 0.0, -1.0],
        texture_layer,
        frame_count,
        tint_color,
        [
            [u_min, v_max],
            [u_min, v_min],
            [u_max, v_min],
            [u_max, v_max],
        ],
    );

    // +Y face (top flame cap)
    push_quad_face_with_uvs(
        buffers,
        [
            [min_x, max_y, min_z],
            [min_x, max_y, max_z],
            [max_x, max_y, max_z],
            [max_x, max_y, min_z],
        ],
        [0.0, 1.0, 0.0],
        texture_layer,
        frame_count,
        tint_color,
        [
            [u_min, u_min],
            [u_min, u_max],
            [u_max, u_max],
            [u_max, u_min],
        ],
    );

    // -Y face (bottom base)
    push_quad_face_with_uvs(
        buffers,
        [
            [min_x, min_y, max_z],
            [min_x, min_y, min_z],
            [max_x, min_y, min_z],
            [max_x, min_y, max_z],
        ],
        [0.0, -1.0, 0.0],
        texture_layer,
        frame_count,
        tint_color,
        [
            [u_min, v_max],
            [u_min, v_max - u_max],
            [u_max, v_max - u_max],
            [u_max, v_max],
        ],
    );
}

fn mesh_torch(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    orientation: u8,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    let stick_w = 2.0 / 16.0;
    let half_w = stick_w * 0.5;
    let b_u_min = 2.0 / 16.0;
    let b_u_max = 6.0 / 16.0;
    let b_v_min = 3.0 / 16.0;
    let b_v_max = 11.0 / 16.0;

    match orientation % 5 {
        0 => {
            // Floor torch (standing centered on floor, compact 10-pixel height)
            let x0 = fx + 0.5 - half_w;
            let x1 = fx + 0.5 + half_w;
            let z0 = fz + 0.5 - half_w;
            let z1 = fz + 0.5 + half_w;
            let y0 = fy;
            let y1 = fy + 10.0 / 16.0;

            push_torch_stick_box(
                buffers,
                x0,
                y0,
                z0,
                x1,
                y1,
                z1,
                texture_layer,
                frame_count,
                tint_color,
            );
        }
        1 => {
            // Wall North (-Z): wall is at z = fz + 0.0
            let x0 = fx + 0.5 - half_w;
            let x1 = fx + 0.5 + half_w;
            let z0 = fz + 4.0 / 16.0;
            let z1 = fz + 6.0 / 16.0;
            let y0 = fy + 2.0 / 16.0;
            let y1 = fy + 12.0 / 16.0;

            push_torch_stick_box(
                buffers,
                x0,
                y0,
                z0,
                x1,
                y1,
                z1,
                texture_layer,
                frame_count,
                tint_color,
            );

            // Wall bracket connecting wall (z = fz) to torch (z = z0)
            let y_b0 = fy + 3.0 / 16.0;
            let y_b1 = fy + 11.0 / 16.0;
            let bx = fx + 0.5;

            // +X side of bracket
            push_quad_face_with_uvs(
                buffers,
                [
                    [bx, y_b0, fz],
                    [bx, y_b1, fz],
                    [bx, y_b1, z0],
                    [bx, y_b0, z0],
                ],
                [1.0, 0.0, 0.0],
                texture_layer,
                frame_count,
                tint_color,
                [
                    [b_u_max, b_v_max],
                    [b_u_max, b_v_min],
                    [b_u_min, b_v_min],
                    [b_u_min, b_v_max],
                ],
            );
            // -X side of bracket
            push_quad_face_with_uvs(
                buffers,
                [
                    [bx, y_b0, z0],
                    [bx, y_b1, z0],
                    [bx, y_b1, fz],
                    [bx, y_b0, fz],
                ],
                [-1.0, 0.0, 0.0],
                texture_layer,
                frame_count,
                tint_color,
                [
                    [b_u_min, b_v_max],
                    [b_u_min, b_v_min],
                    [b_u_max, b_v_min],
                    [b_u_max, b_v_max],
                ],
            );
        }
        2 => {
            // Wall South (+Z): wall is at z = fz + 1.0
            let x0 = fx + 0.5 - half_w;
            let x1 = fx + 0.5 + half_w;
            let z0 = fz + 10.0 / 16.0;
            let z1 = fz + 12.0 / 16.0;
            let y0 = fy + 2.0 / 16.0;
            let y1 = fy + 12.0 / 16.0;

            push_torch_stick_box(
                buffers,
                x0,
                y0,
                z0,
                x1,
                y1,
                z1,
                texture_layer,
                frame_count,
                tint_color,
            );

            // Wall bracket connecting torch (z = z1) to wall (z = fz + 1.0)
            let y_b0 = fy + 3.0 / 16.0;
            let y_b1 = fy + 11.0 / 16.0;
            let bx = fx + 0.5;
            let wall_z = fz + 1.0;

            // +X side of bracket
            push_quad_face_with_uvs(
                buffers,
                [
                    [bx, y_b0, z1],
                    [bx, y_b1, z1],
                    [bx, y_b1, wall_z],
                    [bx, y_b0, wall_z],
                ],
                [1.0, 0.0, 0.0],
                texture_layer,
                frame_count,
                tint_color,
                [
                    [b_u_min, b_v_max],
                    [b_u_min, b_v_min],
                    [b_u_max, b_v_min],
                    [b_u_max, b_v_max],
                ],
            );
            // -X side of bracket
            push_quad_face_with_uvs(
                buffers,
                [
                    [bx, y_b0, wall_z],
                    [bx, y_b1, wall_z],
                    [bx, y_b1, z1],
                    [bx, y_b0, z1],
                ],
                [-1.0, 0.0, 0.0],
                texture_layer,
                frame_count,
                tint_color,
                [
                    [b_u_max, b_v_max],
                    [b_u_max, b_v_min],
                    [b_u_min, b_v_min],
                    [b_u_min, b_v_max],
                ],
            );
        }
        3 => {
            // Wall West (-X): wall is at x = fx + 0.0
            let x0 = fx + 4.0 / 16.0;
            let x1 = fx + 6.0 / 16.0;
            let z0 = fz + 0.5 - half_w;
            let z1 = fz + 0.5 + half_w;
            let y0 = fy + 2.0 / 16.0;
            let y1 = fy + 12.0 / 16.0;

            push_torch_stick_box(
                buffers,
                x0,
                y0,
                z0,
                x1,
                y1,
                z1,
                texture_layer,
                frame_count,
                tint_color,
            );

            // Wall bracket connecting wall (x = fx) to torch (x = x0)
            let y_b0 = fy + 3.0 / 16.0;
            let y_b1 = fy + 11.0 / 16.0;
            let bz = fz + 0.5;

            // +Z side of bracket
            push_quad_face_with_uvs(
                buffers,
                [
                    [x0, y_b0, bz],
                    [x0, y_b1, bz],
                    [fx, y_b1, bz],
                    [fx, y_b0, bz],
                ],
                [0.0, 0.0, 1.0],
                texture_layer,
                frame_count,
                tint_color,
                [
                    [b_u_min, b_v_max],
                    [b_u_min, b_v_min],
                    [b_u_max, b_v_min],
                    [b_u_max, b_v_max],
                ],
            );
            // -Z side of bracket
            push_quad_face_with_uvs(
                buffers,
                [
                    [fx, y_b0, bz],
                    [fx, y_b1, bz],
                    [x0, y_b1, bz],
                    [x0, y_b0, bz],
                ],
                [0.0, 0.0, -1.0],
                texture_layer,
                frame_count,
                tint_color,
                [
                    [b_u_max, b_v_max],
                    [b_u_max, b_v_min],
                    [b_u_min, b_v_min],
                    [b_u_min, b_v_max],
                ],
            );
        }
        _ => {
            // Wall East (+X): wall is at x = fx + 1.0
            let x0 = fx + 10.0 / 16.0;
            let x1 = fx + 12.0 / 16.0;
            let z0 = fz + 0.5 - half_w;
            let z1 = fz + 0.5 + half_w;
            let y0 = fy + 2.0 / 16.0;
            let y1 = fy + 12.0 / 16.0;

            push_torch_stick_box(
                buffers,
                x0,
                y0,
                z0,
                x1,
                y1,
                z1,
                texture_layer,
                frame_count,
                tint_color,
            );

            // Wall bracket connecting torch (x = x1) to wall (x = fx + 1.0)
            let y_b0 = fy + 3.0 / 16.0;
            let y_b1 = fy + 11.0 / 16.0;
            let bz = fz + 0.5;
            let wall_x = fx + 1.0;

            // +Z side of bracket
            push_quad_face_with_uvs(
                buffers,
                [
                    [wall_x, y_b0, bz],
                    [wall_x, y_b1, bz],
                    [x1, y_b1, bz],
                    [x1, y_b0, bz],
                ],
                [0.0, 0.0, 1.0],
                texture_layer,
                frame_count,
                tint_color,
                [
                    [b_u_max, b_v_max],
                    [b_u_max, b_v_min],
                    [b_u_min, b_v_min],
                    [b_u_min, b_v_max],
                ],
            );
            // -Z side of bracket
            push_quad_face_with_uvs(
                buffers,
                [
                    [x1, y_b0, bz],
                    [x1, y_b1, bz],
                    [wall_x, y_b1, bz],
                    [wall_x, y_b0, bz],
                ],
                [0.0, 0.0, -1.0],
                texture_layer,
                frame_count,
                tint_color,
                [
                    [b_u_min, b_v_max],
                    [b_u_min, b_v_min],
                    [b_u_max, b_v_min],
                    [b_u_max, b_v_max],
                ],
            );
        }
    }
}

fn mesh_pebble(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    shape: BlockShape,
    orientation: u8,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    match shape {
        BlockShape::Pebble => {
            push_pebble_element(
                buffers,
                fx,
                fy,
                fz,
                orientation,
                [7.0, 0.0, 5.0],
                [11.0, 2.0, 10.0],
                [0.0, 7.0, 4.0, 9.0],  // north
                [0.0, 5.0, 5.0, 7.0],  // east
                [4.0, 7.0, 8.0, 9.0],  // south
                [5.0, 5.0, 10.0, 7.0], // west
                [4.0, 5.0, 0.0, 0.0],  // up
                [8.0, 0.0, 4.0, 5.0],  // down
                texture_layer,
                frame_count,
                tint_color,
            );
        }
        BlockShape::PebbleCluster => {
            // Main pebble (size 5 x 2 x 5)
            push_pebble_element(
                buffers,
                fx,
                fy,
                fz,
                orientation,
                [7.0, 0.0, 8.0],
                [12.0, 2.0, 13.0],
                [8.0, 0.0, 13.0, 2.0],  // north
                [8.0, 2.0, 13.0, 4.0],  // east
                [8.0, 4.0, 13.0, 6.0],  // south
                [5.0, 8.0, 10.0, 10.0], // west
                [5.0, 5.0, 0.0, 0.0],   // up
                [5.0, 5.0, 0.0, 10.0],  // down
                texture_layer,
                frame_count,
                tint_color,
            );
            // Secondary pebble (size 3 x 1 x 4)
            push_pebble_element(
                buffers,
                fx,
                fy,
                fz,
                orientation,
                [4.0, 0.0, 5.0],
                [7.0, 1.0, 9.0],
                [0.0, 10.0, 3.0, 11.0], // north
                [8.0, 6.0, 12.0, 7.0],  // east
                [3.0, 10.0, 6.0, 11.0], // south
                [8.0, 7.0, 12.0, 8.0],  // west
                [8.0, 4.0, 5.0, 0.0],   // up
                [8.0, 4.0, 5.0, 8.0],   // down
                texture_layer,
                frame_count,
                tint_color,
            );
            // Third pebble (size 3 x 1 x 3)
            push_pebble_element(
                buffers,
                fx,
                fy,
                fz,
                orientation,
                [9.0, 0.0, 3.0],
                [12.0, 1.0, 6.0],
                [0.0, 10.0, 3.0, 11.0], // north
                [8.0, 6.0, 11.0, 7.0],  // east
                [3.0, 10.0, 6.0, 11.0], // south
                [8.0, 7.0, 11.0, 8.0],  // west
                [8.0, 3.0, 5.0, 0.0],   // up
                [8.0, 3.0, 5.0, 6.0],   // down
                texture_layer,
                frame_count,
                tint_color,
            );
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn push_pebble_element(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    orientation: u8,
    from: [f32; 3],
    to: [f32; 3],
    north_uv: [f32; 4],
    east_uv: [f32; 4],
    south_uv: [f32; 4],
    west_uv: [f32; 4],
    up_uv: [f32; 4],
    down_uv: [f32; 4],
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    let x0 = from[0] / 16.0;
    let x1 = to[0] / 16.0;
    let y0 = from[1] / 16.0;
    let y1 = to[1] / 16.0;
    let z0 = from[2] / 16.0;
    let z1 = to[2] / 16.0;

    let rot_vert = |p: [f32; 3]| -> [f32; 3] {
        let (rx, rz) = match orientation % 4 {
            0 => (p[0], p[2]),
            1 => (1.0 - p[2], p[0]),
            2 => (1.0 - p[0], 1.0 - p[2]),
            _ => (p[2], 1.0 - p[0]),
        };
        [fx + rx, fy + p[1], fz + rz]
    };

    let rot_norm = |n: [f32; 3]| -> [f32; 3] {
        match orientation % 4 {
            0 => n,
            1 => [-n[2], n[1], n[0]],
            2 => [-n[0], n[1], -n[2]],
            _ => [n[2], n[1], -n[0]],
        }
    };

    // North face (-Z)
    {
        let u1 = north_uv[0] / 16.0;
        let v1 = north_uv[1] / 16.0;
        let u2 = north_uv[2] / 16.0;
        let v2 = north_uv[3] / 16.0;
        push_quad_face_with_uvs(
            buffers,
            [
                rot_vert([x0, y0, z0]),
                rot_vert([x0, y1, z0]),
                rot_vert([x1, y1, z0]),
                rot_vert([x1, y0, z0]),
            ],
            rot_norm([0.0, 0.0, -1.0]),
            texture_layer,
            frame_count,
            tint_color,
            [[u1, v2], [u1, v1], [u2, v1], [u2, v2]],
        );
    }

    // South face (+Z)
    {
        let u1 = south_uv[0] / 16.0;
        let v1 = south_uv[1] / 16.0;
        let u2 = south_uv[2] / 16.0;
        let v2 = south_uv[3] / 16.0;
        push_quad_face_with_uvs(
            buffers,
            [
                rot_vert([x1, y0, z1]),
                rot_vert([x1, y1, z1]),
                rot_vert([x0, y1, z1]),
                rot_vert([x0, y0, z1]),
            ],
            rot_norm([0.0, 0.0, 1.0]),
            texture_layer,
            frame_count,
            tint_color,
            [[u1, v2], [u1, v1], [u2, v1], [u2, v2]],
        );
    }

    // East face (+X)
    {
        let u1 = east_uv[0] / 16.0;
        let v1 = east_uv[1] / 16.0;
        let u2 = east_uv[2] / 16.0;
        let v2 = east_uv[3] / 16.0;
        push_quad_face_with_uvs(
            buffers,
            [
                rot_vert([x1, y0, z0]),
                rot_vert([x1, y1, z0]),
                rot_vert([x1, y1, z1]),
                rot_vert([x1, y0, z1]),
            ],
            rot_norm([1.0, 0.0, 0.0]),
            texture_layer,
            frame_count,
            tint_color,
            [[u2, v2], [u2, v1], [u1, v1], [u1, v2]],
        );
    }

    // West face (-X)
    {
        let u1 = west_uv[0] / 16.0;
        let v1 = west_uv[1] / 16.0;
        let u2 = west_uv[2] / 16.0;
        let v2 = west_uv[3] / 16.0;
        push_quad_face_with_uvs(
            buffers,
            [
                rot_vert([x0, y0, z1]),
                rot_vert([x0, y1, z1]),
                rot_vert([x0, y1, z0]),
                rot_vert([x0, y0, z0]),
            ],
            rot_norm([-1.0, 0.0, 0.0]),
            texture_layer,
            frame_count,
            tint_color,
            [[u2, v2], [u2, v1], [u1, v1], [u1, v2]],
        );
    }

    // Up face (+Y)
    {
        let u1 = up_uv[0] / 16.0;
        let v1 = up_uv[1] / 16.0;
        let u2 = up_uv[2] / 16.0;
        let v2 = up_uv[3] / 16.0;
        push_quad_face_with_uvs(
            buffers,
            [
                rot_vert([x0, y1, z0]),
                rot_vert([x0, y1, z1]),
                rot_vert([x1, y1, z1]),
                rot_vert([x1, y1, z0]),
            ],
            rot_norm([0.0, 1.0, 0.0]),
            texture_layer,
            frame_count,
            tint_color,
            [[u1, v1], [u1, v2], [u2, v2], [u2, v1]],
        );
    }

    // Down face (-Y)
    {
        let u1 = down_uv[0] / 16.0;
        let v1 = down_uv[1] / 16.0;
        let u2 = down_uv[2] / 16.0;
        let v2 = down_uv[3] / 16.0;
        push_quad_face_with_uvs(
            buffers,
            [
                rot_vert([x0, y0, z1]),
                rot_vert([x0, y0, z0]),
                rot_vert([x1, y0, z0]),
                rot_vert([x1, y0, z1]),
            ],
            rot_norm([0.0, -1.0, 0.0]),
            texture_layer,
            frame_count,
            tint_color,
            [[u1, v1], [u1, v2], [u2, v2], [u2, v1]],
        );
    }
}

fn mesh_cross_billboard(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    let p00 = [fx, fy, fz];
    let p01 = [fx, fy + 1.0, fz];
    let p11 = [fx + 1.0, fy + 1.0, fz + 1.0];
    let p10 = [fx + 1.0, fy, fz + 1.0];

    let q00 = [fx, fy, fz + 1.0];
    let q01 = [fx, fy + 1.0, fz + 1.0];
    let q11 = [fx + 1.0, fy + 1.0, fz];
    let q10 = [fx + 1.0, fy, fz];

    let uv_fwd = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];

    // Plane 1: from (0,0) to (1,1) diagonal (both front and back faces)
    let n1 = [0.70710677, 0.0, -0.70710677];
    let n1_rev = [-0.70710677, 0.0, 0.70710677];
    push_quad_face_with_uvs(
        buffers,
        [p00, p01, p11, p10],
        n1,
        texture_layer,
        frame_count,
        tint_color,
        uv_fwd,
    );
    push_quad_face_with_uvs(
        buffers,
        [p10, p11, p01, p00],
        n1_rev,
        texture_layer,
        frame_count,
        tint_color,
        uv_fwd,
    );

    // Plane 2: from (0,1) to (1,0) diagonal (both front and back faces)
    let n2 = [-0.70710677, 0.0, -0.70710677];
    let n2_rev = [0.70710677, 0.0, 0.70710677];
    push_quad_face_with_uvs(
        buffers,
        [q00, q01, q11, q10],
        n2,
        texture_layer,
        frame_count,
        tint_color,
        uv_fwd,
    );
    push_quad_face_with_uvs(
        buffers,
        [q10, q11, q01, q00],
        n2_rev,
        texture_layer,
        frame_count,
        tint_color,
        uv_fwd,
    );
}

#[allow(clippy::too_many_arguments)]
fn mesh_bamboo(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    side_layer: u16,
    top_layer: u16,
    bottom_layer: u16,
    frame_count: f32,
    side_tint: [f32; 4],
    top_tint: [f32; 4],
    bottom_tint: [f32; 4],
    cull_top: bool,
    cull_bottom: bool,
) {
    let min_x = fx + 6.5 / 16.0;
    let max_x = fx + 9.5 / 16.0;
    let min_y = fy + 0.0;
    let max_y = fy + 1.0;
    let min_z = fz + 6.5 / 16.0;
    let max_z = fz + 9.5 / 16.0;

    let u_side_min = 6.0 / 16.0;
    let u_side_max = 9.0 / 16.0;

    // +X (East)
    push_quad_face_with_uvs(
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
        [
            [u_side_min, 1.0],
            [u_side_min, 0.0],
            [u_side_max, 0.0],
            [u_side_max, 1.0],
        ],
    );

    // -X (West)
    push_quad_face_with_uvs(
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
        [
            [u_side_min, 1.0],
            [u_side_min, 0.0],
            [u_side_max, 0.0],
            [u_side_max, 1.0],
        ],
    );

    // +Z (South)
    push_quad_face_with_uvs(
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
        [
            [u_side_min, 1.0],
            [u_side_min, 0.0],
            [u_side_max, 0.0],
            [u_side_max, 1.0],
        ],
    );

    // -Z (North)
    push_quad_face_with_uvs(
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
        [
            [u_side_min, 1.0],
            [u_side_min, 0.0],
            [u_side_max, 0.0],
            [u_side_max, 1.0],
        ],
    );

    let u_cap_min = 6.0 / 16.0;
    let u_cap_max = 9.0 / 16.0;
    let v_cap_min = 6.0 / 16.0;
    let v_cap_max = 9.0 / 16.0;

    // +Y (Top)
    if !cull_top {
        push_quad_face_with_uvs(
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
            [
                [u_cap_min, v_cap_min],
                [u_cap_min, v_cap_max],
                [u_cap_max, v_cap_max],
                [u_cap_max, v_cap_min],
            ],
        );
    }

    // -Y (Bottom)
    if !cull_bottom {
        push_quad_face_with_uvs(
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
            [
                [u_cap_min, v_cap_min],
                [u_cap_min, v_cap_max],
                [u_cap_max, v_cap_max],
                [u_cap_max, v_cap_min],
            ],
        );
    }
}

