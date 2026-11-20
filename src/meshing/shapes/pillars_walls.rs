use bevy::prelude::*;

use crate::meshing::greedy::MeshBuffers;
use crate::world::{BlockShape, Voxel};
use super::{push_quad_face_uvs, push_shape_box};

#[allow(clippy::too_many_arguments)]
pub fn mesh_column_cocoon(
    buffers: &mut MeshBuffers,
    voxel: Voxel,
    shape: BlockShape,
    orientation: u8,
    fx: f32,
    fy: f32,
    fz: f32,
    box_a: [Vec3; 2],
    top_layer: u16,
    bottom_layer: u16,
    side_layer: u16,
    frame_count_f32: f32,
    top_tint: [f32; 4],
    bottom_tint: [f32; 4],
    side_tint: [f32; 4],
    is_neighbor_solid: &impl Fn(IVec3) -> bool,
    is_grass: bool,
) {
    // Horizontal axis-aligned pillars (logs, bones, basalt, etc.) in Column shape:
    // orient % 7 == 5 -> X-axis horizontal column (end caps on +X/-X, bark grain along X)
    // orient % 7 == 6 -> Z-axis horizontal column (end caps on +Z/-Z, bark grain along Z)
    let col_orient = orientation % 7;
    if shape == BlockShape::Column && voxel.is_axis_aligned() && (col_orient == 5 || col_orient == 6) {
        let min_x = fx + box_a[0].x;
        let max_x = fx + box_a[1].x;
        let min_y = fy + box_a[0].y;
        let max_y = fy + box_a[1].y;
        let min_z = fz + box_a[0].z;
        let max_z = fz + box_a[1].z;

        let cap_uvs = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];

        if col_orient == 5 {
            // Horizontal along X: +X/-X are end caps, +Y/-Y/+Z/-Z are bark barrel along X
            if !is_neighbor_solid(IVec3::X) {
                push_quad_face_uvs(
                    buffers,
                    [
                        [max_x, min_y, min_z],
                        [max_x, max_y, min_z],
                        [max_x, max_y, max_z],
                        [max_x, min_y, max_z],
                    ],
                    [1.0, 0.0, 0.0],
                    top_layer,
                    frame_count_f32,
                    top_tint,
                    cap_uvs,
                );
            }
            if !is_neighbor_solid(IVec3::NEG_X) {
                push_quad_face_uvs(
                    buffers,
                    [
                        [min_x, min_y, max_z],
                        [min_x, max_y, max_z],
                        [min_x, max_y, min_z],
                        [min_x, min_y, min_z],
                    ],
                    [-1.0, 0.0, 0.0],
                    bottom_layer,
                    frame_count_f32,
                    bottom_tint,
                    cap_uvs,
                );
            }
            // +Y barrel (grain runs along X: min_x -> max_x)
            push_quad_face_uvs(
                buffers,
                [
                    [min_x, max_y, min_z],
                    [min_x, max_y, max_z],
                    [max_x, max_y, max_z],
                    [max_x, max_y, min_z],
                ],
                [0.0, 1.0, 0.0],
                side_layer,
                frame_count_f32,
                side_tint,
                [[0.25, 1.0], [0.75, 1.0], [0.75, 0.0], [0.25, 0.0]],
            );
            // -Y barrel
            push_quad_face_uvs(
                buffers,
                [
                    [min_x, min_y, max_z],
                    [min_x, min_y, min_z],
                    [max_x, min_y, min_z],
                    [max_x, min_y, max_z],
                ],
                [0.0, -1.0, 0.0],
                side_layer,
                frame_count_f32,
                side_tint,
                [[0.75, 1.0], [0.25, 1.0], [0.25, 0.0], [0.75, 0.0]],
            );
            // +Z barrel
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
                frame_count_f32,
                side_tint,
                [[0.25, 0.0], [0.75, 0.0], [0.75, 1.0], [0.25, 1.0]],
            );
            // -Z barrel
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
                frame_count_f32,
                side_tint,
                [[0.25, 1.0], [0.75, 1.0], [0.75, 0.0], [0.25, 0.0]],
            );
        } else {
            // Horizontal along Z: +Z/-Z are end caps, +Y/-Y/+X/-X are bark barrel along Z
            if !is_neighbor_solid(IVec3::Z) {
                push_quad_face_uvs(
                    buffers,
                    [
                        [max_x, min_y, max_z],
                        [max_x, max_y, max_z],
                        [min_x, max_y, max_z],
                        [min_x, min_y, max_z],
                    ],
                    [0.0, 0.0, 1.0],
                    top_layer,
                    frame_count_f32,
                    top_tint,
                    cap_uvs,
                );
            }
            if !is_neighbor_solid(IVec3::NEG_Z) {
                push_quad_face_uvs(
                    buffers,
                    [
                        [min_x, min_y, min_z],
                        [min_x, max_y, min_z],
                        [max_x, max_y, min_z],
                        [max_x, min_y, min_z],
                    ],
                    [0.0, 0.0, -1.0],
                    bottom_layer,
                    frame_count_f32,
                    bottom_tint,
                    cap_uvs,
                );
            }
            // +Y barrel (grain runs along Z: min_z -> max_z)
            push_quad_face_uvs(
                buffers,
                [
                    [min_x, max_y, min_z],
                    [min_x, max_y, max_z],
                    [max_x, max_y, max_z],
                    [max_x, max_y, min_z],
                ],
                [0.0, 1.0, 0.0],
                side_layer,
                frame_count_f32,
                side_tint,
                [[0.25, 0.0], [0.25, 1.0], [0.75, 1.0], [0.75, 0.0]],
            );
            // -Y barrel
            push_quad_face_uvs(
                buffers,
                [
                    [min_x, min_y, max_z],
                    [min_x, min_y, min_z],
                    [max_x, min_y, min_z],
                    [max_x, min_y, max_z],
                ],
                [0.0, -1.0, 0.0],
                side_layer,
                frame_count_f32,
                side_tint,
                [[0.25, 1.0], [0.25, 0.0], [0.75, 0.0], [0.75, 1.0]],
            );
            // +X barrel
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
                frame_count_f32,
                side_tint,
                [[0.25, 0.0], [0.75, 0.0], [0.75, 1.0], [0.25, 1.0]],
            );
            // -X barrel
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
                frame_count_f32,
                side_tint,
                [[0.25, 1.0], [0.75, 1.0], [0.75, 0.0], [0.25, 0.0]],
            );
        }
        return;
    }

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
        is_neighbor_solid,
        is_grass,
    );
}
