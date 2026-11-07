use bevy::prelude::*;

use crate::{
    meshing::{greedy::MeshBuffers, textures::FaceTextureInfo},
    world::{BlockShape, Voxel, VoxelAccess},
};
use super::{push_quad_face_with_uvs, push_shape_box};

pub fn mesh_basket(
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
    false,
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
pub fn mesh_torch(
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
pub fn mesh_pebble(
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
pub fn mesh_flat_pad(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    voxel: Voxel,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
    stem_info: Option<FaceTextureInfo>,
) {
    let is_petals = voxel.is_petal_carpet();
    let y = if is_petals { fy + 0.05 } else { fy + 0.02 };

    // If petal carpet, render supporting vertical stem underneath connecting ground to petals
    if is_petals {
        if let Some(stem) = stem_info {
            let stem_tint = [1.0, 1.0, 1.0, 1.0];
            let stem_layer = stem.start_layer;
            let stem_frame_count = stem.frame_count as f32;
            let uv_cross = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
            // Diagonal quad A (front-left to back-right)
            let a0 = [fx, fy, fz];
            let a1 = [fx, fy + 1.0, fz];
            let a2 = [fx + 1.0, fy + 1.0, fz + 1.0];
            let a3 = [fx + 1.0, fy, fz + 1.0];
            push_quad_face_with_uvs(buffers, [a0, a1, a2, a3], [0.707, 0.0, -0.707], stem_layer, stem_frame_count, stem_tint, uv_cross);
            push_quad_face_with_uvs(buffers, [a3, a2, a1, a0], [-0.707, 0.0, 0.707], stem_layer, stem_frame_count, stem_tint, uv_cross);
            // Diagonal quad B (front-right to back-left)
            let b0 = [fx + 1.0, fy, fz];
            let b1 = [fx + 1.0, fy + 1.0, fz];
            let b2 = [fx, fy + 1.0, fz + 1.0];
            let b3 = [fx, fy, fz + 1.0];
            push_quad_face_with_uvs(buffers, [b0, b1, b2, b3], [0.707, 0.0, 0.707], stem_layer, stem_frame_count, stem_tint, uv_cross);
            push_quad_face_with_uvs(buffers, [b3, b2, b1, b0], [-0.707, 0.0, -0.707], stem_layer, stem_frame_count, stem_tint, uv_cross);
        }
    }

    // Top face
    let p0 = [fx, y, fz + 1.0];
    let p1 = [fx + 1.0, y, fz + 1.0];
    let p2 = [fx + 1.0, y, fz];
    let p3 = [fx, y, fz];
    let uv_top = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    push_quad_face_with_uvs(buffers, [p0, p1, p2, p3], [0.0, 1.0, 0.0], texture_layer, frame_count, tint_color, uv_top);
    // Bottom face
    let b0 = [fx, y, fz];
    let b1 = [fx + 1.0, y, fz];
    let b2 = [fx + 1.0, y, fz + 1.0];
    let b3 = [fx, y, fz + 1.0];
    let uv_bot = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    push_quad_face_with_uvs(buffers, [b0, b1, b2, b3], [0.0, -1.0, 0.0], texture_layer, frame_count, tint_color, uv_bot);
}

pub fn mesh_wall_decal(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    orientation: u8,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    let d = 0.015;
    let uv_std = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
    match orientation % 6 {
        0 => {
            // Floor flat pad
            let y = fy + d;
            let uv_top = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
            let uv_bot = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
            push_quad_face_with_uvs(buffers, [[fx, y, fz + 1.0], [fx + 1.0, y, fz + 1.0], [fx + 1.0, y, fz], [fx, y, fz]], [0.0, 1.0, 0.0], texture_layer, frame_count, tint_color, uv_top);
            push_quad_face_with_uvs(buffers, [[fx, y, fz], [fx + 1.0, y, fz], [fx + 1.0, y, fz + 1.0], [fx, y, fz + 1.0]], [0.0, -1.0, 0.0], texture_layer, frame_count, tint_color, uv_bot);
        }
        1 => {
            // Ceiling flat pad
            let y = fy + 1.0 - d;
            let uv_down = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
            let uv_up = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
            push_quad_face_with_uvs(buffers, [[fx, y, fz], [fx + 1.0, y, fz], [fx + 1.0, y, fz + 1.0], [fx, y, fz + 1.0]], [0.0, -1.0, 0.0], texture_layer, frame_count, tint_color, uv_down);
            push_quad_face_with_uvs(buffers, [[fx, y, fz + 1.0], [fx + 1.0, y, fz + 1.0], [fx + 1.0, y, fz], [fx, y, fz]], [0.0, 1.0, 0.0], texture_layer, frame_count, tint_color, uv_up);
        }
        2 => {
            // Attached to North face (+Z normal)
            let z = fz + d;
            push_quad_face_with_uvs(buffers, [[fx, fy, z], [fx, fy + 1.0, z], [fx + 1.0, fy + 1.0, z], [fx + 1.0, fy, z]], [0.0, 0.0, 1.0], texture_layer, frame_count, tint_color, uv_std);
            push_quad_face_with_uvs(buffers, [[fx + 1.0, fy, z], [fx + 1.0, fy + 1.0, z], [fx, fy + 1.0, z], [fx, fy, z]], [0.0, 0.0, -1.0], texture_layer, frame_count, tint_color, uv_std);
        }
        3 => {
            // Attached to South face (-Z normal)
            let z = fz + 1.0 - d;
            push_quad_face_with_uvs(buffers, [[fx + 1.0, fy, z], [fx + 1.0, fy + 1.0, z], [fx, fy + 1.0, z], [fx, fy, z]], [0.0, 0.0, -1.0], texture_layer, frame_count, tint_color, uv_std);
            push_quad_face_with_uvs(buffers, [[fx, fy, z], [fx, fy + 1.0, z], [fx + 1.0, fy + 1.0, z], [fx + 1.0, fy, z]], [0.0, 0.0, 1.0], texture_layer, frame_count, tint_color, uv_std);
        }
        4 => {
            // Attached to West face (+X normal)
            let x = fx + d;
            push_quad_face_with_uvs(buffers, [[x, fy, fz + 1.0], [x, fy + 1.0, fz + 1.0], [x, fy + 1.0, fz], [x, fy, fz]], [1.0, 0.0, 0.0], texture_layer, frame_count, tint_color, uv_std);
            push_quad_face_with_uvs(buffers, [[x, fy, fz], [x, fy + 1.0, fz], [x, fy + 1.0, fz + 1.0], [x, fy, fz + 1.0]], [-1.0, 0.0, 0.0], texture_layer, frame_count, tint_color, uv_std);
        }
        _ => {
            // Attached to East face (-X normal)
            let x = fx + 1.0 - d;
            push_quad_face_with_uvs(buffers, [[x, fy, fz], [x, fy + 1.0, fz], [x, fy + 1.0, fz + 1.0], [x, fy, fz + 1.0]], [-1.0, 0.0, 0.0], texture_layer, frame_count, tint_color, uv_std);
            push_quad_face_with_uvs(buffers, [[x, fy, fz + 1.0], [x, fy + 1.0, fz + 1.0], [x, fy + 1.0, fz], [x, fy, fz]], [1.0, 0.0, 0.0], texture_layer, frame_count, tint_color, uv_std);
        }
    }
}

pub fn mesh_cactus(
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
    let min_x = fx + 1.0 / 16.0;
    let max_x = fx + 15.0 / 16.0;
    let min_y = fy;
    let max_y = fy + 1.0;
    let min_z = fz + 1.0 / 16.0;
    let max_z = fz + 15.0 / 16.0;
    // Top face
    if !is_neighbor_solid(IVec3::Y) {
        let top_uvs = [
            [1.0 / 16.0, 1.0 / 16.0],
            [1.0 / 16.0, 15.0 / 16.0],
            [15.0 / 16.0, 15.0 / 16.0],
            [15.0 / 16.0, 1.0 / 16.0],
        ];
        push_quad_face_with_uvs(
            buffers,
            [
                [min_x, max_y, max_z],
                [max_x, max_y, max_z],
                [max_x, max_y, min_z],
                [min_x, max_y, min_z],
            ],
            [0.0, 1.0, 0.0],
            top_layer,
            frame_count,
            top_tint,
            top_uvs,
        );
    }
    // Bottom face
    if !is_neighbor_solid(IVec3::NEG_Y) {
        let bot_uvs = [
            [1.0 / 16.0, 1.0 / 16.0],
            [1.0 / 16.0, 15.0 / 16.0],
            [15.0 / 16.0, 15.0 / 16.0],
            [15.0 / 16.0, 1.0 / 16.0],
        ];
        push_quad_face_with_uvs(
            buffers,
            [
                [min_x, min_y, min_z],
                [max_x, min_y, min_z],
                [max_x, min_y, max_z],
                [min_x, min_y, max_z],
            ],
            [0.0, -1.0, 0.0],
            bottom_layer,
            frame_count,
            bottom_tint,
            bot_uvs,
        );
    }
    let side_uvs = [
        [1.0 / 16.0, 1.0],
        [1.0 / 16.0, 0.0],
        [15.0 / 16.0, 0.0],
        [15.0 / 16.0, 1.0],
    ];
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
        side_uvs,
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
        side_uvs,
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
        side_uvs,
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
        side_uvs,
    );
}
pub fn mesh_cross_billboard(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    mesh_cross_billboard_oriented(
        buffers,
        fx,
        fy,
        fz,
        0,
        texture_layer,
        frame_count,
        tint_color,
    );
}

pub fn mesh_cross_billboard_oriented(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    orientation: u8,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    // Orientations (matching WallDecal / 6-face attachment):
    // 0: Floor (+Y growth, base at y = fy, tip at y = fy + 0.998)
    // 1: Ceiling (-Y growth, base at y = fy + 1.0, tip at y = fy + 0.002)
    // 2: North Wall (-Z support, +Z growth, base at z = fz, tip at z = fz + 0.998)
    // 3: South Wall (+Z support, -Z growth, base at z = fz + 1.0, tip at z = fz + 0.002)
    // 4: West Wall (-X support, +X growth, base at x = fx, tip at x = fx + 0.998)
    // 5: East Wall (+X support, -X growth, base at x = fx + 1.0, tip at x = fx + 0.002)
    let (p00, p01, p11, p10, q00, q01, q11, q10) = match orientation % 6 {
        0 => {
            let base_y = fy;
            let tip_y = fy + 0.998;
            (
                [fx, base_y, fz],
                [fx, tip_y, fz],
                [fx + 1.0, tip_y, fz + 1.0],
                [fx + 1.0, base_y, fz + 1.0],
                [fx, base_y, fz + 1.0],
                [fx, tip_y, fz + 1.0],
                [fx + 1.0, tip_y, fz],
                [fx + 1.0, base_y, fz],
            )
        }
        1 => {
            let base_y = fy + 1.0;
            let tip_y = fy + 0.002;
            (
                [fx, base_y, fz + 1.0],
                [fx, tip_y, fz + 1.0],
                [fx + 1.0, tip_y, fz],
                [fx + 1.0, base_y, fz],
                [fx, base_y, fz],
                [fx, tip_y, fz],
                [fx + 1.0, tip_y, fz + 1.0],
                [fx + 1.0, base_y, fz + 1.0],
            )
        }
        2 => {
            let base_z = fz;
            let tip_z = fz + 0.998;
            (
                [fx, fy, base_z],
                [fx, fy, tip_z],
                [fx + 1.0, fy + 1.0, tip_z],
                [fx + 1.0, fy + 1.0, base_z],
                [fx, fy + 1.0, base_z],
                [fx, fy + 1.0, tip_z],
                [fx + 1.0, fy, tip_z],
                [fx + 1.0, fy, base_z],
            )
        }
        3 => {
            let base_z = fz + 1.0;
            let tip_z = fz + 0.002;
            (
                [fx + 1.0, fy, base_z],
                [fx + 1.0, fy, tip_z],
                [fx, fy + 1.0, tip_z],
                [fx, fy + 1.0, base_z],
                [fx + 1.0, fy + 1.0, base_z],
                [fx + 1.0, fy + 1.0, tip_z],
                [fx, fy, tip_z],
                [fx, fy, base_z],
            )
        }
        4 => {
            let base_x = fx;
            let tip_x = fx + 0.998;
            (
                [base_x, fy, fz],
                [tip_x, fy, fz],
                [tip_x, fy + 1.0, fz + 1.0],
                [base_x, fy + 1.0, fz + 1.0],
                [base_x, fy + 1.0, fz],
                [tip_x, fy + 1.0, fz],
                [tip_x, fy, fz + 1.0],
                [base_x, fy, fz + 1.0],
            )
        }
        _ => {
            let base_x = fx + 1.0;
            let tip_x = fx + 0.002;
            (
                [base_x, fy, fz + 1.0],
                [tip_x, fy, fz + 1.0],
                [tip_x, fy + 1.0, fz],
                [base_x, fy + 1.0, fz],
                [base_x, fy + 1.0, fz + 1.0],
                [tip_x, fy + 1.0, fz + 1.0],
                [tip_x, fy, fz],
                [base_x, fy, fz],
            )
        }
    };

    let eps = 0.005;
    let uv_fwd = [[eps, 1.0 - eps], [eps, eps], [1.0 - eps, eps], [1.0 - eps, 1.0 - eps]];
    // Back faces must mirror U coordinates so both sides share identical world-space texture orientation
    // (like a physical card in 3D space, identical to Minecraft flora), preventing flora billboards from
    // unnaturally tracking camera rotation.
    let uv_rev = [[1.0 - eps, 1.0 - eps], [1.0 - eps, eps], [eps, eps], [eps, 1.0 - eps]];

    // Upward-facing normal for cross-billboards ensures uniform sky & sunlight illumination
    // on both sides, identical to Minecraft's flora lighting model.
    let n_up = [0.0, 1.0, 0.0];
    push_quad_face_with_uvs(
        buffers,
        [p00, p01, p11, p10],
        n_up,
        texture_layer,
        frame_count,
        tint_color,
        uv_fwd,
    );
    push_quad_face_with_uvs(
        buffers,
        [p10, p11, p01, p00],
        n_up,
        texture_layer,
        frame_count,
        tint_color,
        uv_rev,
    );

    push_quad_face_with_uvs(
        buffers,
        [q00, q01, q11, q10],
        n_up,
        texture_layer,
        frame_count,
        tint_color,
        uv_fwd,
    );
    push_quad_face_with_uvs(
        buffers,
        [q10, q11, q01, q00],
        n_up,
        texture_layer,
        frame_count,
        tint_color,
        uv_rev,
    );
}

pub fn mesh_bamboo(
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

pub fn mesh_rubble(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    orientation: u8,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    // 1. Main slab-like central boulder (size 9 x 6 x 8, from [3.0, 0.0, 3.0] to [12.0, 6.0, 11.0])
    push_pebble_element(
        buffers,
        fx,
        fy,
        fz,
        orientation,
        [3.0, 0.0, 3.0],
        [12.0, 6.0, 11.0],
        [0.0, 0.0, 9.0, 6.0],   // north (-Z)
        [0.0, 0.0, 8.0, 6.0],   // east (+X)
        [0.0, 0.0, 9.0, 6.0],   // south (+Z)
        [0.0, 0.0, 8.0, 6.0],   // west (-X)
        [0.0, 0.0, 9.0, 8.0],   // up (+Y)
        [0.0, 0.0, 9.0, 8.0],   // down (-Y)
        texture_layer,
        frame_count,
        tint_color,
    );

    // 2. Secondary side chunk (size 5 x 4 x 6, from [11.0, 0.0, 5.0] to [16.0, 4.0, 11.0])
    push_pebble_element(
        buffers,
        fx,
        fy,
        fz,
        orientation,
        [11.0, 0.0, 5.0],
        [16.0, 4.0, 11.0],
        [4.0, 2.0, 9.0, 6.0],
        [2.0, 2.0, 8.0, 6.0],
        [4.0, 2.0, 9.0, 6.0],
        [2.0, 2.0, 8.0, 6.0],
        [4.0, 2.0, 9.0, 8.0],
        [4.0, 2.0, 9.0, 8.0],
        texture_layer,
        frame_count,
        tint_color,
    );

    // 3. Front smaller rock (size 6 x 3 x 5, from [4.0, 0.0, 11.0] to [10.0, 3.0, 16.0])
    push_pebble_element(
        buffers,
        fx,
        fy,
        fz,
        orientation,
        [4.0, 0.0, 11.0],
        [10.0, 3.0, 16.0],
        [6.0, 4.0, 12.0, 7.0],
        [5.0, 4.0, 10.0, 7.0],
        [6.0, 4.0, 12.0, 7.0],
        [5.0, 4.0, 10.0, 7.0],
        [6.0, 4.0, 12.0, 9.0],
        [6.0, 4.0, 12.0, 9.0],
        texture_layer,
        frame_count,
        tint_color,
    );

    // 4. Corner splinter rock (size 4 x 2 x 4, from [0.0, 0.0, 2.0] to [4.0, 2.0, 6.0])
    push_pebble_element(
        buffers,
        fx,
        fy,
        fz,
        orientation,
        [0.0, 0.0, 2.0],
        [4.0, 2.0, 6.0],
        [8.0, 8.0, 12.0, 10.0],
        [8.0, 8.0, 12.0, 10.0],
        [8.0, 8.0, 12.0, 10.0],
        [8.0, 8.0, 12.0, 10.0],
        [8.0, 8.0, 12.0, 12.0],
        [8.0, 8.0, 12.0, 12.0],
        texture_layer,
        frame_count,
        tint_color,
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeleothemDirection {
    Down,
    Up,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeleothemPiece {
    Base,
    Frustum,
    Middle,
    Tip,
    TipMerge,
}

pub fn determine_speleothem_part(
    world: &impl VoxelAccess,
    world_voxel: IVec3,
    voxel: Voxel,
) -> (SpeleothemDirection, SpeleothemPiece) {
    let mut count_above = 0;
    let mut curr = world_voxel + IVec3::Y;
    while count_above < 24 {
        if let Some(v) = world.get_voxel(curr) {
            if v == voxel {
                count_above += 1;
                curr += IVec3::Y;
                continue;
            }
        }
        break;
    }
    let ceil_is_solid = world.get_voxel(curr).is_some_and(|v| v.is_solid_opaque());

    let mut count_below = 0;
    let mut curr = world_voxel - IVec3::Y;
    while count_below < 24 {
        if let Some(v) = world.get_voxel(curr) {
            if v == voxel {
                count_below += 1;
                curr -= IVec3::Y;
                continue;
            }
        }
        break;
    }
    let floor_is_solid = world.get_voxel(curr).is_some_and(|v| v.is_solid_opaque());

    let total_length = count_above + 1 + count_below;
    let dist_from_ceil = count_above;
    let dist_from_floor = count_below;

    if ceil_is_solid && !floor_is_solid {
        // Stalactite hanging from ceiling
        let dist_from_tip = count_below;
        let piece = match dist_from_tip {
            0 => SpeleothemPiece::Tip,
            1 => SpeleothemPiece::Frustum,
            _ => {
                if dist_from_ceil == 0 {
                    SpeleothemPiece::Base
                } else {
                    SpeleothemPiece::Middle
                }
            }
        };
        (SpeleothemDirection::Down, piece)
    } else if !ceil_is_solid && floor_is_solid {
        // Stalagmite standing on floor
        let dist_from_tip = count_above;
        let piece = match dist_from_tip {
            0 => SpeleothemPiece::Tip,
            1 => SpeleothemPiece::Frustum,
            _ => {
                if dist_from_floor == 0 {
                    SpeleothemPiece::Base
                } else {
                    SpeleothemPiece::Middle
                }
            }
        };
        (SpeleothemDirection::Up, piece)
    } else if ceil_is_solid && floor_is_solid {
        // Merged column between ceiling and floor
        if total_length == 1 {
            (SpeleothemDirection::Down, SpeleothemPiece::TipMerge)
        } else {
            let split = total_length / 2;
            if dist_from_ceil < split {
                let piece = if dist_from_ceil == split - 1 {
                    SpeleothemPiece::TipMerge
                } else if dist_from_ceil == 0 {
                    SpeleothemPiece::Base
                } else {
                    SpeleothemPiece::Middle
                };
                (SpeleothemDirection::Down, piece)
            } else {
                let piece = if dist_from_floor == total_length - split - 1 {
                    SpeleothemPiece::TipMerge
                } else if dist_from_floor == 0 {
                    SpeleothemPiece::Base
                } else {
                    SpeleothemPiece::Middle
                };
                (SpeleothemDirection::Up, piece)
            }
        }
    } else {
        // Floating / unsupported fallback (hangs downwards)
        let dist_from_tip = count_below;
        let piece = match dist_from_tip {
            0 => SpeleothemPiece::Tip,
            1 => SpeleothemPiece::Frustum,
            _ => SpeleothemPiece::Middle,
        };
        (SpeleothemDirection::Down, piece)
    }
}

pub fn mesh_speleothem(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    texture_layer: u16,
    frame_count: f32,
    tint_color: [f32; 4],
) {
    mesh_cross_billboard(
        buffers,
        fx,
        fy,
        fz,
        texture_layer,
        frame_count,
        tint_color,
    );
}
