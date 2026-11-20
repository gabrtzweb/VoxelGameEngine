use bevy::prelude::*;

use crate::meshing::greedy::MeshBuffers;
use crate::world::{BlockShape, Voxel, VoxelAccess};
use super::{push_quad_face_uvs, push_shape_box};

#[allow(clippy::too_many_arguments)]
pub fn mesh_column_cocoon(
    world: &impl VoxelAccess,
    world_voxel: IVec3,
    buffers: &mut MeshBuffers,
    voxel: Voxel,
    shape: BlockShape,
    orientation: u8,
    fx: f32,
    fy: f32,
    fz: f32,
    mut box_a: [Vec3; 2],
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
    let col_orient = orientation % 7;
    let mut internal_cull = [false; 6]; // [+X, -X, +Y, -Y, +Z, -Z]

    // Seamless perpendicular Column-to-Column connection:
    // Extend centered columns by 0.25m to meet the flush face (0.25 / 0.75) of an adjacent
    // perpendicular centered Column (e.g. horizontal branch meeting a vertical slender trunk,
    // perpendicular horizontal beams meeting at a corner/T-junction, or vertical pillar meeting a beam).
    if shape == BlockShape::Column {
        let is_perp_col = |offset: IVec3, matches_orient: fn(u8) -> bool| -> bool {
            let npos = world_voxel + offset;
            if !world
                .get_voxel(npos)
                .is_some_and(|v| !v.is_empty() && !v.is_fluid())
            {
                return false;
            }
            let (ns, no) = world.get_shape(npos);
            ns == BlockShape::Column && matches_orient(no % 7)
        };

        match col_orient {
            0 => {
                // Vertical centered column: connect vertically (+Y/-Y) into horizontal beams (5 or 6)
                if is_perp_col(IVec3::Y, |o| o == 5 || o == 6) {
                    box_a[1].y = 1.25;
                    internal_cull[2] = true;
                }
                if is_perp_col(IVec3::NEG_Y, |o| o == 5 || o == 6) {
                    box_a[0].y = -0.25;
                    internal_cull[3] = true;
                }
            }
            5 => {
                // Horizontal X-axis column: connect along +X/-X into vertical (0) or Z-horizontal (6) column
                if is_perp_col(IVec3::X, |o| o == 0 || o == 6) {
                    box_a[1].x = 1.25;
                    internal_cull[0] = true;
                }
                if is_perp_col(IVec3::NEG_X, |o| o == 0 || o == 6) {
                    box_a[0].x = -0.25;
                    internal_cull[1] = true;
                }
            }
            6 => {
                // Horizontal Z-axis column: connect along +Z/-Z into vertical (0) or X-horizontal (5) column
                if is_perp_col(IVec3::Z, |o| o == 0 || o == 5) {
                    box_a[1].z = 1.25;
                    internal_cull[4] = true;
                }
                if is_perp_col(IVec3::NEG_Z, |o| o == 0 || o == 5) {
                    box_a[0].z = -0.25;
                    internal_cull[5] = true;
                }
            }
            _ => {}
        }
    }

    // Horizontal axis-aligned pillars (logs, bones, basalt, etc.) in Column shape:
    // orient % 7 == 5 -> X-axis horizontal column (end caps on +X/-X, bark grain along X)
    // orient % 7 == 6 -> Z-axis horizontal column (end caps on +Z/-Z, bark grain along Z)
    if shape == BlockShape::Column && voxel.is_axis_aligned() && (col_orient == 5 || col_orient == 6) {
        let min_x = fx + box_a[0].x;
        let max_x = fx + box_a[1].x;
        let min_y = fy + box_a[0].y;
        let max_y = fy + box_a[1].y;
        let min_z = fz + box_a[0].z;
        let max_z = fz + box_a[1].z;

        let cap_uvs = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];

        if col_orient == 5 {
            let v0 = box_a[0].x;
            let v1 = box_a[1].x;
            // Horizontal along X: +X/-X are end caps, +Y/-Y/+Z/-Z are bark barrel along X
            if !internal_cull[0] && !is_neighbor_solid(IVec3::X) {
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
            if !internal_cull[1] && !is_neighbor_solid(IVec3::NEG_X) {
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
                [[0.25, 1.0 - v0], [0.75, 1.0 - v0], [0.75, 1.0 - v1], [0.25, 1.0 - v1]],
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
                [[0.75, 1.0 - v0], [0.25, 1.0 - v0], [0.25, 1.0 - v1], [0.75, 1.0 - v1]],
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
                [[0.25, 1.0 - v1], [0.75, 1.0 - v1], [0.75, 1.0 - v0], [0.25, 1.0 - v0]],
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
                [[0.25, 1.0 - v0], [0.75, 1.0 - v0], [0.75, 1.0 - v1], [0.25, 1.0 - v1]],
            );
        } else {
            let v0 = box_a[0].z;
            let v1 = box_a[1].z;
            // Horizontal along Z: +Z/-Z are end caps, +Y/-Y/+X/-X are bark barrel along Z
            if !internal_cull[4] && !is_neighbor_solid(IVec3::Z) {
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
            if !internal_cull[5] && !is_neighbor_solid(IVec3::NEG_Z) {
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
                [[0.25, v0], [0.25, v1], [0.75, v1], [0.75, v0]],
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
                [[0.25, v1], [0.25, v0], [0.75, v0], [0.75, v1]],
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
                [[0.25, v0], [0.75, v0], [0.75, v1], [0.25, v1]],
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
                [[0.25, v1], [0.75, v1], [0.75, v0], [0.25, v0]],
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
        internal_cull,
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
