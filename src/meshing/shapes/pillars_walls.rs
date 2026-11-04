use bevy::prelude::*;

use crate::meshing::greedy::MeshBuffers;
use super::push_shape_box;

pub fn mesh_column_cocoon(
    buffers: &mut MeshBuffers,
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
