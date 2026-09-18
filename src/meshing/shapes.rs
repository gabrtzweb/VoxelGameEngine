use bevy::prelude::*;

use super::{
    greedy::{FaceDirection, MeshBuffers},
    textures::VoxelTextureRegistry,
};
use crate::{
    gameplay::shaping::{centered_layer_coordinates, is_centered_layer},
    simulation::fluid::water_surface_height_offset,
    world::{CHUNK_SIZE, Chunk, VOXEL_SIZE, Voxel, VoxelAccess},
};

pub fn is_chunk_local_isolated_voxel(chunk: &Chunk, local_voxel: IVec3) -> bool {
    let bx = (local_voxel.x as usize / 2) * 2;
    let by = (local_voxel.y as usize / 2) * 2;
    let bz = (local_voxel.z as usize / 2) * 2;

    let is_solid = |v: Voxel| {
        !v.is_empty() && !v.is_water() && v != Voxel::Occupied && v != Voxel::WaterOccupied
    };

    let mut solid_count = 0;
    for dy in 0..2 {
        for dz in 0..2 {
            for dx in 0..2 {
                if is_solid(chunk.get(bx + dx, by + dy, bz + dz)) {
                    solid_count += 1;
                    if solid_count > 1 {
                        return false;
                    }
                }
            }
        }
    }
    solid_count == 1
}

pub fn is_chunk_local_centered_layer(chunk: &Chunk, local_voxel: IVec3) -> bool {
    let bx = (local_voxel.x as usize / 2) * 2;
    let bz = (local_voxel.z as usize / 2) * 2;
    let y = local_voxel.y as usize;

    let is_occ = |v: Voxel| v == Voxel::Occupied || v == Voxel::WaterOccupied;
    is_occ(chunk.get(bx, y, bz))
        || is_occ(chunk.get(bx + 1, y, bz))
        || is_occ(chunk.get(bx, y, bz + 1))
        || is_occ(chunk.get(bx + 1, y, bz + 1))
}

pub fn get_chunk_local_centered_material(chunk: &Chunk, local_voxel: IVec3) -> Option<Voxel> {
    let bx = (local_voxel.x as usize / 2) * 2;
    let bz = (local_voxel.z as usize / 2) * 2;
    let y = local_voxel.y as usize;

    for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        let v = chunk.get(bx + dx, y, bz + dz);
        if !v.is_empty() && v != Voxel::Occupied && v != Voxel::WaterOccupied {
            return Some(v);
        }
    }
    None
}

pub fn mesh_centered_voxels(
    world: &impl VoxelAccess,
    chunk: &Chunk,
    chunk_coordinate: IVec3,
    textures: &VoxelTextureRegistry,
    opaque_buffers: &mut MeshBuffers,
    transparent_buffers: &mut MeshBuffers,
) {
    let chunk_voxel_origin = chunk_coordinate * CHUNK_SIZE as i32;

    for bz_idx in 0..(CHUNK_SIZE / 2) {
        let bz = bz_idx * 2;
        for y in 0..CHUNK_SIZE {
            for bx_idx in 0..(CHUNK_SIZE / 2) {
                let bx = bx_idx * 2;

                let local = IVec3::new(bx as i32, y as i32, bz as i32);
                if !is_chunk_local_centered_layer(chunk, local) {
                    continue;
                }

                let Some(material) = get_chunk_local_centered_material(chunk, local) else {
                    continue;
                };

                let world_voxel = chunk_voxel_origin + local;
                let (side_layer, frame_count) =
                    textures.get_face_texture_info(material, world_voxel, FaceDirection::PositiveX);
                let (top_layer, _) =
                    textures.get_face_texture_info(material, world_voxel, FaceDirection::PositiveY);
                let (bottom_layer, _) =
                    textures.get_face_texture_info(material, world_voxel, FaceDirection::NegativeY);

                let frame_count_f32 = if material.is_light() {
                    -4.0
                } else {
                    frame_count as f32
                };
                let top_tint = material.tint_color_at(world_voxel);
                let side_tint = if material == Voxel::Grass && !textures.full_grass {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    material.tint_color_at(world_voxel)
                };
                let bottom_tint = if material == Voxel::Grass {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    material.tint_color_at(world_voxel)
                };

                let min_x = bx as f32 + 0.5;
                let max_x = bx as f32 + 1.5;
                let min_y = y as f32;
                let max_y = (y + 1) as f32;
                let min_z = bz as f32 + 0.5;
                let max_z = bz as f32 + 1.5;

                // Check face culling for +Y (top):
                let top_world = world_voxel + IVec3::Y;
                let cull_top = if is_centered_layer(world, top_world) {
                    true
                } else {
                    let coords = centered_layer_coordinates(top_world);
                    coords.iter().all(|&c| {
                        world
                            .get_voxel(c)
                            .is_some_and(|v| !v.is_empty() && !v.is_transparent())
                    })
                };

                // Check face culling for -Y (bottom):
                let bottom_world = world_voxel - IVec3::Y;
                let cull_bottom = if is_centered_layer(world, bottom_world) {
                    true
                } else {
                    let coords = centered_layer_coordinates(bottom_world);
                    coords.iter().all(|&c| {
                        world
                            .get_voxel(c)
                            .is_some_and(|v| !v.is_empty() && !v.is_transparent())
                    })
                };

                // +X
                push_centered_quad(
                    opaque_buffers,
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
                );
                // -X
                push_centered_quad(
                    opaque_buffers,
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
                );
                // +Z
                push_centered_quad(
                    opaque_buffers,
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
                );
                // -Z
                push_centered_quad(
                    opaque_buffers,
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
                );

                if !cull_top {
                    // +Y
                    push_centered_quad(
                        opaque_buffers,
                        [
                            [min_x, max_y, min_z],
                            [min_x, max_y, max_z],
                            [max_x, max_y, max_z],
                            [max_x, max_y, min_z],
                        ],
                        [0.0, 1.0, 0.0],
                        top_layer,
                        frame_count_f32,
                        top_tint,
                    );
                }

                if !cull_bottom {
                    // -Y
                    push_centered_quad(
                        opaque_buffers,
                        [
                            [min_x, min_y, max_z],
                            [min_x, min_y, min_z],
                            [max_x, min_y, min_z],
                            [max_x, min_y, max_z],
                        ],
                        [0.0, -1.0, 0.0],
                        bottom_layer,
                        frame_count_f32,
                        bottom_tint,
                    );
                }

                // Render water if layer is waterlogged
                let is_waterlogged = chunk.get(bx, y, bz) == Voxel::WaterOccupied
                    || chunk.get(bx + 1, y, bz) == Voxel::WaterOccupied
                    || chunk.get(bx, y, bz + 1) == Voxel::WaterOccupied
                    || chunk.get(bx + 1, y, bz + 1) == Voxel::WaterOccupied;

                if is_waterlogged {
                    let (w_layer, w_frame_count) =
                        textures.get_texture_info(Voxel::WaterFlowing, world_voxel);
                    let w_tint = Voxel::Water.tint_color_at(world_voxel);

                    let b_min_x = bx as f32;
                    let b_max_x = (bx + 2) as f32;
                    let b_min_y = y as f32;
                    let b_max_y = (y + 1) as f32;
                    let b_min_z = bz as f32;
                    let b_max_z = (bz + 2) as f32;

                    let surface_offset = water_surface_height_offset(world, world_voxel);
                    let water_top_world = world_voxel + IVec3::Y;
                    let top_has_water = world
                        .get_voxel(water_top_world)
                        .is_some_and(Voxel::is_water);

                    let water_surface_y = if top_has_water {
                        b_max_y * VOXEL_SIZE
                    } else {
                        b_max_y * VOXEL_SIZE - surface_offset
                    };

                    if !top_has_water {
                        push_water_quad_both_sides(
                            transparent_buffers,
                            [
                                [b_min_x * VOXEL_SIZE, water_surface_y, b_min_z * VOXEL_SIZE],
                                [b_min_x * VOXEL_SIZE, water_surface_y, b_max_z * VOXEL_SIZE],
                                [b_max_x * VOXEL_SIZE, water_surface_y, b_max_z * VOXEL_SIZE],
                                [b_max_x * VOXEL_SIZE, water_surface_y, b_min_z * VOXEL_SIZE],
                            ],
                            w_layer,
                            w_frame_count,
                            w_tint,
                        );
                    }

                    let bottom_coords = centered_layer_coordinates(world_voxel - IVec3::Y);
                    let bottom_empty = bottom_coords
                        .iter()
                        .any(|&c| world.get_voxel(c).is_none_or(|v| v.is_empty()));
                    if bottom_empty {
                        push_centered_quad(
                            transparent_buffers,
                            [
                                [b_min_x, b_min_y, b_max_z],
                                [b_min_x, b_min_y, b_min_z],
                                [b_max_x, b_min_y, b_min_z],
                                [b_max_x, b_min_y, b_max_z],
                            ],
                            [0.0, -1.0, 0.0],
                            w_layer,
                            w_frame_count as f32,
                            w_tint,
                        );
                    }

                    let px_neighbor = world
                        .get_voxel(world_voxel + IVec3::new(2, 0, 0))
                        .unwrap_or(Voxel::Air);
                    if px_neighbor.is_empty() {
                        push_water_side_quad(
                            transparent_buffers,
                            [
                                [
                                    b_max_x * VOXEL_SIZE,
                                    b_min_y * VOXEL_SIZE,
                                    b_min_z * VOXEL_SIZE,
                                ],
                                [b_max_x * VOXEL_SIZE, water_surface_y, b_min_z * VOXEL_SIZE],
                                [b_max_x * VOXEL_SIZE, water_surface_y, b_max_z * VOXEL_SIZE],
                                [
                                    b_max_x * VOXEL_SIZE,
                                    b_min_y * VOXEL_SIZE,
                                    b_max_z * VOXEL_SIZE,
                                ],
                            ],
                            [1.0, 0.0, 0.0],
                            w_layer,
                            w_frame_count,
                            w_tint,
                        );
                    }

                    let nx_neighbor = world
                        .get_voxel(world_voxel - IVec3::new(1, 0, 0))
                        .unwrap_or(Voxel::Air);
                    if nx_neighbor.is_empty() {
                        push_water_side_quad(
                            transparent_buffers,
                            [
                                [
                                    b_min_x * VOXEL_SIZE,
                                    b_min_y * VOXEL_SIZE,
                                    b_max_z * VOXEL_SIZE,
                                ],
                                [b_min_x * VOXEL_SIZE, water_surface_y, b_max_z * VOXEL_SIZE],
                                [b_min_x * VOXEL_SIZE, water_surface_y, b_min_z * VOXEL_SIZE],
                                [
                                    b_min_x * VOXEL_SIZE,
                                    b_min_y * VOXEL_SIZE,
                                    b_min_z * VOXEL_SIZE,
                                ],
                            ],
                            [-1.0, 0.0, 0.0],
                            w_layer,
                            w_frame_count,
                            w_tint,
                        );
                    }

                    let pz_neighbor = world
                        .get_voxel(world_voxel + IVec3::new(0, 0, 2))
                        .unwrap_or(Voxel::Air);
                    if pz_neighbor.is_empty() {
                        push_water_side_quad(
                            transparent_buffers,
                            [
                                [
                                    b_max_x * VOXEL_SIZE,
                                    b_min_y * VOXEL_SIZE,
                                    b_max_z * VOXEL_SIZE,
                                ],
                                [b_max_x * VOXEL_SIZE, water_surface_y, b_max_z * VOXEL_SIZE],
                                [b_max_x * VOXEL_SIZE, water_surface_y, b_min_z * VOXEL_SIZE],
                                [
                                    b_max_x * VOXEL_SIZE,
                                    b_min_y * VOXEL_SIZE,
                                    b_min_z * VOXEL_SIZE,
                                ],
                            ],
                            [0.0, 0.0, 1.0],
                            w_layer,
                            w_frame_count,
                            w_tint,
                        );
                    }

                    let nz_neighbor = world
                        .get_voxel(world_voxel - IVec3::new(0, 0, 1))
                        .unwrap_or(Voxel::Air);
                    if nz_neighbor.is_empty() {
                        push_water_side_quad(
                            transparent_buffers,
                            [
                                [
                                    b_min_x * VOXEL_SIZE,
                                    b_min_y * VOXEL_SIZE,
                                    b_min_z * VOXEL_SIZE,
                                ],
                                [b_min_x * VOXEL_SIZE, water_surface_y, b_min_z * VOXEL_SIZE],
                                [b_max_x * VOXEL_SIZE, water_surface_y, b_min_z * VOXEL_SIZE],
                                [
                                    b_max_x * VOXEL_SIZE,
                                    b_min_y * VOXEL_SIZE,
                                    b_max_z * VOXEL_SIZE,
                                ],
                            ],
                            [0.0, 0.0, -1.0],
                            w_layer,
                            w_frame_count,
                            w_tint,
                        );
                    }
                }
            }
        }
    }
}

pub fn push_centered_quad(
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

pub fn push_water_quad_both_sides(
    buffers: &mut MeshBuffers,
    vertices: [[f32; 3]; 4],
    texture_layer: u16,
    frame_count: u16,
    tint_color: [f32; 4],
) {
    let base_index = buffers.positions.len() as u32;

    for v in &vertices {
        buffers.positions.push(*v);
        buffers.normals.push([0.0, 1.0, 0.0]);
        buffers.colors.push(tint_color);
        buffers
            .uv_bs
            .push([texture_layer as f32, frame_count as f32]);
    }

    buffers
        .uvs
        .extend_from_slice(&[[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]);

    buffers.indices.extend_from_slice(&[
        base_index,
        base_index + 1,
        base_index + 2,
        base_index,
        base_index + 2,
        base_index + 3,
    ]);

    let under_base_index = buffers.positions.len() as u32;

    for v in &vertices {
        buffers.positions.push(*v);
        buffers.normals.push([0.0, -1.0, 0.0]);
        buffers.colors.push(tint_color);
        buffers
            .uv_bs
            .push([texture_layer as f32, frame_count as f32]);
    }

    buffers
        .uvs
        .extend_from_slice(&[[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]);

    buffers.indices.extend_from_slice(&[
        under_base_index,
        under_base_index + 2,
        under_base_index + 1,
        under_base_index,
        under_base_index + 3,
        under_base_index + 2,
    ]);
}

pub fn push_water_side_quad(
    buffers: &mut MeshBuffers,
    vertices: [[f32; 3]; 4],
    normal: [f32; 3],
    texture_layer: u16,
    frame_count: u16,
    tint_color: [f32; 4],
) {
    let base_index = buffers.positions.len() as u32;

    for v in &vertices {
        buffers.positions.push(*v);
        buffers.normals.push(normal);
        buffers.colors.push(tint_color);
        buffers
            .uv_bs
            .push([texture_layer as f32, frame_count as f32]);
    }

    buffers
        .uvs
        .extend_from_slice(&[[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]);

    buffers.indices.extend_from_slice(&[
        base_index,
        base_index + 1,
        base_index + 2,
        base_index,
        base_index + 2,
        base_index + 3,
    ]);
}
