use bevy::prelude::*;

use crate::{
    meshing::{greedy::{FaceDirection, MeshBuffers}, textures::VoxelTextureRegistry},
    world::{BlockShape, Chunk},
};
use super::push_shape_box;

pub fn mesh_slab(
    is_transparent_pass: bool,
    transparent_buffers: &mut MeshBuffers,
    opaque_buffers: &mut MeshBuffers,
    chunk: &Chunk,
    x: usize,
    y: usize,
    z: usize,
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
    textures: &VoxelTextureRegistry,
    world_voxel: IVec3,
) {
    let extra = chunk.get_extra_slab(x, y, z);
    let mut base_cull = [false; 6];
    if let Some((_extra_voxel, extra_orient)) = extra {
        if orientation == 0 && extra_orient == 1 {
            base_cull[2] = true; // +Y face touches upper slab
        } else if orientation == 1 && extra_orient == 0 {
            base_cull[3] = true; // -Y face touches lower slab
        }
    }
    let base_buffers = if is_transparent_pass {
        &mut *transparent_buffers
    } else {
        &mut *opaque_buffers
    };
    push_shape_box(
        base_buffers,
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
        is_neighbor_solid,
        is_grass,
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
            0.6
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
            is_neighbor_solid,
            is_extra_grass,
        );
    }
}

pub fn mesh_stair(
    buffers: &mut MeshBuffers,
    fx: f32,
    fy: f32,
    fz: f32,
    orientation: u8,
    box_a: [Vec3; 2],
    maybe_box_b: Option<[Vec3; 2]>,
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
        is_neighbor_solid,
        is_grass,
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
            is_neighbor_solid,
            is_grass,
        );
    }
}
