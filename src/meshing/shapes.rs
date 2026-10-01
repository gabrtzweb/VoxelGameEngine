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
        if voxel.is_empty() {
            continue;
        }

        let buffers = if voxel.is_transparent() {
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

        let frame_count_f32 = if voxel.is_light() {
            -(frame_count.max(1) as f32)
        } else {
            frame_count as f32
        };

        let tint = voxel.tint_color_at(world_voxel);
        let top_tint = tint;
        let is_grass = matches!(voxel, Voxel::Soil_Grass | Voxel::Soil_Peat_Grass | Voxel::Soil_Silt_Grass);
        let bottom_tint = if is_grass {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            tint
        };
        let side_tint = if is_grass && !textures.full_grass {
            [1.0, 1.0, 1.0, 1.0]
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

        let (box_a, maybe_box_b) = shape.local_boxes(orientation);

        match shape {
            BlockShape::Full | BlockShape::Column => {
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
                        -4.0
                    } else {
                        extra_frame_count as f32
                    };

                    let extra_tint = extra_voxel.tint_color_at(world_voxel);
                    let extra_top_tint = extra_tint;
                    let is_extra_grass = matches!(extra_voxel, Voxel::Soil_Grass | Voxel::Soil_Peat_Grass | Voxel::Soil_Silt_Grass);
                    let extra_bottom_tint = if is_extra_grass {
                        [1.0, 1.0, 1.0, 1.0]
                    } else {
                        extra_tint
                    };
                    let extra_side_tint = if is_extra_grass && !textures.full_grass {
                        [1.0, 1.0, 1.0, 1.0]
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
                    let extra_buffers = if extra_voxel.is_transparent() {
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
#[allow(clippy::too_many_arguments)]
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

    // +X (East, idx 0)
    if !cull[0] {
        push_quad_face(
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
        );
    }

    // -X (West, idx 1)
    if !cull[1] {
        push_quad_face(
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
        push_quad_face(
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
        );
    }

    // -Z (North, idx 5)
    if !cull[5] {
        push_quad_face(
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
    let base_index = buffers.positions.len() as u32;

    for v in &vertices {
        buffers
            .positions
            .push([v[0] * VOXEL_SIZE, v[1] * VOXEL_SIZE, v[2] * VOXEL_SIZE]);
        buffers.normals.push(normal);
        buffers.colors.push(tint_color);
        buffers.uv_bs.push([texture_layer as f32, frame_count]);
    }

    let uvs = if normal[1].abs() > 0.5 {
        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
    } else {
        [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
    };

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
