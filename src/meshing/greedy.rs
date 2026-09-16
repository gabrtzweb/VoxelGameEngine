use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::{IVec3, Mesh},
};

use super::{
    shapes::{is_chunk_local_centered_layer, is_chunk_local_isolated_voxel, mesh_centered_voxels},
    textures::VoxelTextureRegistry,
};
use crate::{
    gameplay::shaping::is_centered_layer,
    simulation::fluid::water_surface_height_offset,
    world::{CHUNK_SIZE, Chunk, VOXEL_SIZE, Voxel, VoxelAccess},
};

const MASK_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceDirection {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl FaceDirection {
    pub fn normal(self) -> IVec3 {
        match self {
            Self::PositiveX => IVec3::new(1, 0, 0),
            Self::NegativeX => IVec3::new(-1, 0, 0),
            Self::PositiveY => IVec3::new(0, 1, 0),
            Self::NegativeY => IVec3::new(0, -1, 0),
            Self::PositiveZ => IVec3::new(0, 0, 1),
            Self::NegativeZ => IVec3::new(0, 0, -1),
        }
    }

    pub fn normal_f32(self) -> [f32; 3] {
        match self {
            Self::PositiveX => [1.0, 0.0, 0.0],
            Self::NegativeX => [-1.0, 0.0, 0.0],
            Self::PositiveY => [0.0, 1.0, 0.0],
            Self::NegativeY => [0.0, -1.0, 0.0],
            Self::PositiveZ => [0.0, 0.0, 1.0],
            Self::NegativeZ => [0.0, 0.0, -1.0],
        }
    }
}

pub const FACE_DIRECTIONS: [FaceDirection; 6] = [
    FaceDirection::PositiveX,
    FaceDirection::NegativeX,
    FaceDirection::PositiveY,
    FaceDirection::NegativeY,
    FaceDirection::PositiveZ,
    FaceDirection::NegativeZ,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceKey {
    pub voxel: Voxel,
    pub texture_layer: u16,
    pub frame_count: u16,
    pub tint_color: [f32; 4],
    pub surface_offset_cm: u8,
    pub step_bottom_offset_cm: u8,
    pub is_isolated_voxel: bool,
}

impl FaceKey {
    pub fn matches(self, other: Self) -> bool {
        self.voxel == other.voxel
            && self.texture_layer == other.texture_layer
            && self.frame_count == other.frame_count
            && self.tint_color == other.tint_color
            && self.surface_offset_cm == other.surface_offset_cm
            && self.step_bottom_offset_cm == other.step_bottom_offset_cm
            && self.is_isolated_voxel == other.is_isolated_voxel
    }
}

pub struct ChunkMeshes {
    pub opaque: Option<Mesh>,
    pub transparent: Option<Mesh>,
}

pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub uv_bs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl MeshBuffers {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            uv_bs: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn push_quad(
        &mut self,
        direction: FaceDirection,
        key: FaceKey,
        slice: usize,
        u: usize,
        v: usize,
        width: usize,
        height: usize,
    ) {
        let base_index = self.positions.len() as u32;

        let vertices = quad_vertices(direction, slice, u, v, width, height);

        let color = key.tint_color;
        let layer = key.texture_layer as f32;
        let frame_count = if key.voxel.is_light() {
            -4.0
        } else {
            key.frame_count as f32
        };

        let surface_offset = key.surface_offset_cm as f32 / 100.0;
        let step_bottom_offset = key.step_bottom_offset_cm as f32 / 100.0;

        for (i, vertex) in vertices.iter().enumerate() {
            let mut pos = [
                vertex[0] * VOXEL_SIZE,
                vertex[1] * VOXEL_SIZE,
                vertex[2] * VOXEL_SIZE,
            ];

            if key.voxel.is_water() {
                match direction {
                    FaceDirection::PositiveY => {
                        pos[1] -= surface_offset;
                    }
                    FaceDirection::PositiveX
                    | FaceDirection::NegativeX
                    | FaceDirection::PositiveZ
                    | FaceDirection::NegativeZ => {
                        if i == 1 || i == 2 {
                            pos[1] -= surface_offset;
                        } else if key.step_bottom_offset_cm > 0 {
                            pos[1] += VOXEL_SIZE - step_bottom_offset;
                        }
                    }
                    FaceDirection::NegativeY => {}
                }
            }

            self.positions.push(pos);
            self.normals.push(direction.normal_f32());
            self.colors.push(color);
            self.uv_bs.push([layer, frame_count]);
        }

        let uvs_to_push = if key.is_isolated_voxel {
            [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
        } else {
            let scale = if key.voxel == Voxel::WaterFlowing {
                4.0
            } else {
                2.0
            };

            let u_min = (u as f32) / scale;
            let u_max = ((u + width) as f32) / scale;
            let v_min = (v as f32) / scale;
            let v_max = ((v + height) as f32) / scale;

            [
                [u_min, v_min],
                [u_min, v_max],
                [u_max, v_max],
                [u_max, v_min],
            ]
        };

        self.uvs.extend_from_slice(&uvs_to_push);

        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);

        if key.voxel.is_water() && direction == FaceDirection::PositiveY {
            let under_base_index = self.positions.len() as u32;

            for vertex in vertices.iter() {
                let mut pos = [
                    vertex[0] * VOXEL_SIZE,
                    vertex[1] * VOXEL_SIZE,
                    vertex[2] * VOXEL_SIZE,
                ];
                pos[1] -= surface_offset;

                self.positions.push(pos);
                self.normals.push(FaceDirection::NegativeY.normal_f32());
                self.colors.push(color);
                self.uv_bs.push([layer, frame_count]);
            }

            self.uvs.extend_from_slice(&uvs_to_push);

            self.indices.extend_from_slice(&[
                under_base_index,
                under_base_index + 2,
                under_base_index + 1,
                under_base_index,
                under_base_index + 3,
                under_base_index + 2,
            ]);
        }
    }

    pub fn into_mesh(self) -> Option<Mesh> {
        if self.positions.is_empty() {
            return None;
        }

        Some(
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, self.uv_bs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
            .with_inserted_indices(Indices::U32(self.indices)),
        )
    }
}

pub struct ChunkMesher;

impl ChunkMesher {
    pub fn build_meshes(
        world: &impl VoxelAccess,
        chunk_coordinate: IVec3,
        textures: &VoxelTextureRegistry,
    ) -> ChunkMeshes {
        let Some(chunk) = world.get_chunk(chunk_coordinate) else {
            return ChunkMeshes {
                opaque: None,
                transparent: None,
            };
        };

        // Early-exit 1: A chunk with zero non-air voxels contains no geometry.
        if chunk.is_empty() {
            return ChunkMeshes {
                opaque: None,
                transparent: None,
            };
        }

        // Early-exit 2: A chunk that is 100% solid opaque and surrounded on all 6 faces by
        // other 100% solid opaque chunks has zero exposed boundary or internal faces.
        let is_fully_solid = chunk.is_fully_solid_opaque();
        if is_fully_solid {
            let all_neighbors_solid = FACE_DIRECTIONS.iter().all(|&direction| {
                let neighbor_chunk_coord = chunk_coordinate + direction.normal();
                world
                    .get_chunk(neighbor_chunk_coord)
                    .is_some_and(Chunk::is_fully_solid_opaque)
            });

            if all_neighbors_solid {
                return ChunkMeshes {
                    opaque: None,
                    transparent: None,
                };
            }
        }

        let chunk_voxel_origin = chunk_coordinate * CHUNK_SIZE as i32;

        let mut opaque_buffers = MeshBuffers::new();
        let mut transparent_buffers = MeshBuffers::new();

        for direction in FACE_DIRECTIONS {
            // Optimization for solid chunks: if the chunk is solid opaque and the neighbor
            // in `direction` is also solid opaque, the boundary slice towards it has 0 faces.
            if is_fully_solid {
                let neighbor_chunk_coord = chunk_coordinate + direction.normal();
                if world
                    .get_chunk(neighbor_chunk_coord)
                    .is_some_and(Chunk::is_fully_solid_opaque)
                {
                    continue;
                }
            }

            let slice_range = if is_fully_solid {
                match direction {
                    FaceDirection::PositiveX
                    | FaceDirection::PositiveY
                    | FaceDirection::PositiveZ => (CHUNK_SIZE - 1)..CHUNK_SIZE,
                    FaceDirection::NegativeX
                    | FaceDirection::NegativeY
                    | FaceDirection::NegativeZ => 0..1,
                }
            } else {
                0..CHUNK_SIZE
            };

            for slice in slice_range {
                let mut mask: [Option<FaceKey>; MASK_SIZE] = [None; MASK_SIZE];

                for v in 0..CHUNK_SIZE {
                    for u in 0..CHUNK_SIZE {
                        let local_voxel = mask_to_voxel(direction, slice, u, v);

                        let voxel = chunk.get(
                            local_voxel.x as usize,
                            local_voxel.y as usize,
                            local_voxel.z as usize,
                        );

                        if voxel.is_empty()
                            || voxel == Voxel::Occupied
                            || is_chunk_local_centered_layer(chunk, local_voxel)
                        {
                            continue;
                        }

                        let world_voxel = chunk_voxel_origin + local_voxel;
                        let neighbor_coordinate = world_voxel + direction.normal();
                        let neighbor = world.get_voxel(neighbor_coordinate).unwrap_or(Voxel::Air);

                        let neighbor_is_centered = is_centered_layer(world, neighbor_coordinate);
                        let mut step_bottom_offset_cm = 0u8;

                        if !neighbor_is_centered {
                            if voxel.is_water()
                                && direction != FaceDirection::PositiveY
                                && direction != FaceDirection::NegativeY
                            {
                                if neighbor.is_water() {
                                    let v_offset =
                                        (water_surface_height_offset(world, world_voxel) * 100.0)
                                            .round() as u8;
                                    let n_offset =
                                        (water_surface_height_offset(world, neighbor_coordinate)
                                            * 100.0)
                                            .round() as u8;
                                    if v_offset < n_offset {
                                        step_bottom_offset_cm = n_offset;
                                    } else {
                                        continue;
                                    }
                                } else if !neighbor.is_empty() && neighbor != Voxel::Occupied {
                                    continue;
                                }
                            } else if !should_render_face(voxel, neighbor) {
                                continue;
                            }
                        }

                        let (texture_layer, frame_count) =
                            textures.get_texture_info(voxel, world_voxel);
                        let tint_color = voxel.tint_color_at(world_voxel);

                        let surface_offset_cm = if voxel.is_water() {
                            (water_surface_height_offset(world, world_voxel) * 100.0).round() as u8
                        } else {
                            0
                        };

                        let is_isolated_voxel = if voxel.is_water() {
                            false
                        } else {
                            is_chunk_local_isolated_voxel(chunk, local_voxel)
                        };

                        mask[mask_index(u, v)] = Some(FaceKey {
                            voxel,
                            texture_layer,
                            frame_count,
                            tint_color,
                            surface_offset_cm,
                            step_bottom_offset_cm,
                            is_isolated_voxel,
                        });
                    }
                }

                greedy_merge_mask(
                    &mut mask,
                    direction,
                    slice,
                    &mut opaque_buffers,
                    &mut transparent_buffers,
                );
            }
        }

        if !is_fully_solid {
            mesh_centered_voxels(
                world,
                chunk,
                chunk_coordinate,
                textures,
                &mut opaque_buffers,
                &mut transparent_buffers,
            );
        }

        ChunkMeshes {
            opaque: opaque_buffers.into_mesh(),
            transparent: transparent_buffers.into_mesh(),
        }
    }
}

pub fn should_render_face(voxel: Voxel, neighbor: Voxel) -> bool {
    if voxel.is_transparent() {
        return neighbor.is_empty() || neighbor == Voxel::Occupied;
    }

    neighbor.is_empty() || neighbor.is_transparent() || neighbor == Voxel::Occupied
}

pub fn greedy_merge_mask(
    mask: &mut [Option<FaceKey>; MASK_SIZE],
    direction: FaceDirection,
    slice: usize,
    opaque_buffers: &mut MeshBuffers,
    transparent_buffers: &mut MeshBuffers,
) {
    for v in 0..CHUNK_SIZE {
        let mut u = 0;

        while u < CHUNK_SIZE {
            let index = mask_index(u, v);

            let Some(key) = mask[index] else {
                u += 1;
                continue;
            };

            let (width, height) = if key.is_isolated_voxel {
                (1, 1)
            } else {
                let mut width = 1;

                while u + width < CHUNK_SIZE {
                    let candidate = mask[mask_index(u + width, v)];

                    if candidate.is_some_and(|c| c.matches(key)) {
                        width += 1;
                    } else {
                        break;
                    }
                }

                let mut height = 1;

                'rows: while v + height < CHUNK_SIZE {
                    for du in 0..width {
                        let candidate = mask[mask_index(u + du, v + height)];

                        if !candidate.is_some_and(|c| c.matches(key)) {
                            break 'rows;
                        }
                    }

                    height += 1;
                }

                (width, height)
            };

            for dv in 0..height {
                for du in 0..width {
                    mask[mask_index(u + du, v + dv)] = None;
                }
            }

            let target_buffers = if key.voxel.is_transparent() {
                &mut *transparent_buffers
            } else {
                &mut *opaque_buffers
            };

            target_buffers.push_quad(direction, key, slice, u, v, width, height);

            u += width;
        }
    }
}

pub fn mask_index(u: usize, v: usize) -> usize {
    u + v * CHUNK_SIZE
}

pub fn mask_to_voxel(direction: FaceDirection, slice: usize, u: usize, v: usize) -> IVec3 {
    match direction {
        FaceDirection::PositiveX | FaceDirection::NegativeX => {
            IVec3::new(slice as i32, v as i32, u as i32)
        }
        FaceDirection::PositiveY | FaceDirection::NegativeY => {
            IVec3::new(u as i32, slice as i32, v as i32)
        }
        FaceDirection::PositiveZ | FaceDirection::NegativeZ => {
            IVec3::new(u as i32, v as i32, slice as i32)
        }
    }
}

pub fn quad_vertices(
    direction: FaceDirection,
    slice: usize,
    u: usize,
    v: usize,
    width: usize,
    height: usize,
) -> [[f32; 3]; 4] {
    let slice = slice as f32;
    let u0 = u as f32;
    let u1 = (u + width) as f32;
    let v0 = v as f32;
    let v1 = (v + height) as f32;

    match direction {
        FaceDirection::PositiveX => {
            let x = slice + 1.0;
            [[x, v0, u0], [x, v1, u0], [x, v1, u1], [x, v0, u1]]
        }
        FaceDirection::NegativeX => {
            let x = slice;
            [[x, v0, u1], [x, v1, u1], [x, v1, u0], [x, v0, u0]]
        }
        FaceDirection::PositiveY => {
            let y = slice + 1.0;
            [[u0, y, v0], [u0, y, v1], [u1, y, v1], [u1, y, v0]]
        }
        FaceDirection::NegativeY => {
            let y = slice;
            [[u0, y, v1], [u0, y, v0], [u1, y, v0], [u1, y, v1]]
        }
        FaceDirection::PositiveZ => {
            let z = slice + 1.0;
            [[u1, v0, z], [u1, v1, z], [u0, v1, z], [u0, v0, z]]
        }
        FaceDirection::NegativeZ => {
            let z = slice;
            [[u0, v0, z], [u0, v1, z], [u1, v1, z], [u1, v0, z]]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        meshing::textures::build_voxel_texture_array,
        world::{Chunk, Voxel, VoxelWorld},
    };
    use bevy::render::mesh::VertexAttributeValues;

    #[test]
    fn mesher_applies_tint_color_to_vertices() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, Voxel::Grass);
        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let opaque = meshes.opaque.expect("Opaque mesh should exist");

        let colors = opaque
            .attribute(Mesh::ATTRIBUTE_COLOR)
            .expect("Mesh should have vertex colors");

        if let VertexAttributeValues::Float32x4(color_data) = colors {
            assert!(!color_data.is_empty());
            let grass_tint = Voxel::Grass.tint_color();
            for vertex_color in color_data {
                assert_eq!(*vertex_color, grass_tint);
            }
        } else {
            panic!("Expected Float32x4 vertex colors");
        }
    }

    #[test]
    fn mesher_applies_animation_frame_count_metadata() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, Voxel::Water);
        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let transparent = meshes.transparent.expect("Transparent mesh should exist");

        let uv_bs = transparent
            .attribute(Mesh::ATTRIBUTE_UV_1)
            .expect("Mesh should have UV_1");

        if let VertexAttributeValues::Float32x2(uv_b_data) = uv_bs {
            assert!(!uv_b_data.is_empty());
            for entry in uv_b_data {
                assert_eq!(entry[1], 36.0, "Water frame count should be 36.0");
            }
        } else {
            panic!("Expected Float32x2 UV_1");
        }

        let positions = transparent
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Mesh should have positions");

        if let VertexAttributeValues::Float32x3(pos_data) = positions {
            assert!(!pos_data.is_empty());
            let max_y = pos_data
                .iter()
                .map(|p| p[1])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(
                (max_y - 0.45).abs() < 1e-4,
                "Surface water max Y should be 0.45, got {}",
                max_y
            );
        } else {
            panic!("Expected Float32x3 positions");
        }

        let normals = transparent
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .expect("Mesh should have normals");
        if let VertexAttributeValues::Float32x3(norm_data) = normals {
            let has_down_normal = norm_data.iter().any(|n| n[1] < -0.9);
            assert!(
                has_down_normal,
                "Transparent mesh should have downward-facing normals for underwater view"
            );
        } else {
            panic!("Expected Float32x3 normals");
        }
    }

    #[test]
    fn mesher_renders_centered_voxels_correctly() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, Voxel::Stone);
        chunk.set(1, 0, 0, Voxel::Occupied);
        chunk.set(0, 0, 1, Voxel::Occupied);
        chunk.set(1, 0, 1, Voxel::Occupied);
        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let opaque = meshes
            .opaque
            .expect("Opaque mesh should exist for centered voxel");

        let positions = opaque
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Mesh should have positions");

        if let VertexAttributeValues::Float32x3(pos_data) = positions {
            assert!(!pos_data.is_empty());
            let min_x = pos_data.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
            let max_x = pos_data
                .iter()
                .map(|p| p[0])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(
                (min_x - 0.25).abs() < 1e-4,
                "min_x should be 0.25, got {}",
                min_x
            );
            assert!(
                (max_x - 0.75).abs() < 1e-4,
                "max_x should be 0.75, got {}",
                max_x
            );
        } else {
            panic!("Expected Float32x3 positions");
        }
    }

    #[test]
    fn ground_below_centered_voxel_renders_top_face() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, Voxel::Sand);
        chunk.set(1, 0, 0, Voxel::Sand);
        chunk.set(0, 0, 1, Voxel::Sand);
        chunk.set(1, 0, 1, Voxel::Sand);

        chunk.set(0, 1, 0, Voxel::Stone);
        chunk.set(1, 1, 0, Voxel::Occupied);
        chunk.set(0, 1, 1, Voxel::Occupied);
        chunk.set(1, 1, 1, Voxel::Occupied);

        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let opaque = meshes.opaque.expect("Opaque mesh should exist");

        let normals = opaque
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .expect("Mesh should have normals");

        if let VertexAttributeValues::Float32x3(norm_data) = normals {
            let positions = opaque
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .expect("Mesh should have positions");
            if let VertexAttributeValues::Float32x3(pos_data) = positions {
                let mut up_faces_at_ground = 0;
                for (norm, pos) in norm_data.iter().zip(pos_data.iter()) {
                    if norm[1] > 0.9 && (pos[1] - 0.50).abs() < 1e-4 {
                        up_faces_at_ground += 1;
                    }
                }
                assert!(
                    up_faces_at_ground >= 4,
                    "Ground under centered column must render its top face (at least 4 vertices), got {}",
                    up_faces_at_ground
                );
            }
        }
    }

    #[test]
    fn waterlogged_centered_voxel_renders_both_opaque_and_transparent_meshes() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, Voxel::Stone);
        chunk.set(1, 0, 0, Voxel::WaterOccupied);
        chunk.set(0, 0, 1, Voxel::WaterOccupied);
        chunk.set(1, 0, 1, Voxel::WaterOccupied);

        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        assert!(
            meshes.opaque.is_some(),
            "Opaque mesh should exist for column post"
        );
        assert!(
            meshes.transparent.is_some(),
            "Transparent mesh should exist for waterlogging"
        );
    }

    #[test]
    fn water_step_renders_vertical_face_between_different_heights() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(1, 0, 1, Voxel::Water);
        chunk.set(2, 0, 1, Voxel::WaterFlowing);
        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let transparent = meshes.transparent.expect("Transparent mesh should exist");

        let normals = transparent
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .expect("Mesh should have normals");
        let positions = transparent
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Mesh should have positions");

        if let (
            VertexAttributeValues::Float32x3(norm_data),
            VertexAttributeValues::Float32x3(pos_data),
        ) = (normals, positions)
        {
            let mut found_step_quad = false;
            for (norm, pos) in norm_data.iter().zip(pos_data.iter()) {
                if norm[0] > 0.9 && pos[1] >= 0.39 && pos[1] <= 0.46 {
                    found_step_quad = true;
                    break;
                }
            }
            assert!(
                found_step_quad,
                "Expected vertical step quad between water levels facing +X"
            );
        } else {
            panic!("Expected Float32x3 attributes");
        }
    }

    #[test]
    fn mesher_full_block_face_maps_single_texture_uv() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        for dy in 0..2 {
            for dz in 0..2 {
                for dx in 0..2 {
                    chunk.set(dx, dy, dz, Voxel::Stone);
                }
            }
        }
        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let opaque = meshes.opaque.expect("Opaque mesh should exist");

        let uvs = opaque
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .expect("Mesh should have UV_0");

        if let VertexAttributeValues::Float32x2(uv_data) = uvs {
            assert!(!uv_data.is_empty());
            for quad_uvs in uv_data.chunks_exact(4) {
                let min_u = quad_uvs
                    .iter()
                    .map(|uv| uv[0])
                    .fold(f32::INFINITY, f32::min);
                let max_u = quad_uvs
                    .iter()
                    .map(|uv| uv[0])
                    .fold(f32::NEG_INFINITY, f32::max);
                let min_v = quad_uvs
                    .iter()
                    .map(|uv| uv[1])
                    .fold(f32::INFINITY, f32::min);
                let max_v = quad_uvs
                    .iter()
                    .map(|uv| uv[1])
                    .fold(f32::NEG_INFINITY, f32::max);

                assert!(
                    (max_u - min_u - 1.0).abs() < 1e-4,
                    "Each full block face U span must be 1.0 (single texture), got {}",
                    max_u - min_u
                );
                assert!(
                    (max_v - min_v - 1.0).abs() < 1e-4,
                    "Each full block face V span must be 1.0 (single texture), got {}",
                    max_v - min_v
                );
            }
        } else {
            panic!("Expected Float32x2 UV_0 attributes");
        }
    }

    #[test]
    fn mesher_isolated_single_voxel_maps_full_texture_uv() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, Voxel::Stone);
        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let opaque = meshes.opaque.expect("Opaque mesh should exist");

        let uvs = opaque
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .expect("Mesh should have UV_0");

        if let VertexAttributeValues::Float32x2(uv_data) = uvs {
            assert!(!uv_data.is_empty());
            for quad_uvs in uv_data.chunks_exact(4) {
                let min_u = quad_uvs
                    .iter()
                    .map(|uv| uv[0])
                    .fold(f32::INFINITY, f32::min);
                let max_u = quad_uvs
                    .iter()
                    .map(|uv| uv[0])
                    .fold(f32::NEG_INFINITY, f32::max);
                let min_v = quad_uvs
                    .iter()
                    .map(|uv| uv[1])
                    .fold(f32::INFINITY, f32::min);
                let max_v = quad_uvs
                    .iter()
                    .map(|uv| uv[1])
                    .fold(f32::NEG_INFINITY, f32::max);

                assert_eq!(min_u, 0.0, "Isolated single voxel min U must be 0.0");
                assert_eq!(max_u, 1.0, "Isolated single voxel max U must be 1.0");
                assert_eq!(min_v, 0.0, "Isolated single voxel min V must be 0.0");
                assert_eq!(max_v, 1.0, "Isolated single voxel max V must be 1.0");
            }
        } else {
            panic!("Expected Float32x2 UV_0 attributes");
        }
    }

    #[test]
    fn mesher_two_adjacent_subvoxels_combine_uv() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, Voxel::Stone);
        chunk.set(1, 0, 0, Voxel::Stone);
        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let opaque = meshes.opaque.expect("Opaque mesh should exist");

        let normals = opaque
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .expect("Mesh should have normals");
        let uvs = opaque
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .expect("Mesh should have UV_0");

        if let (
            VertexAttributeValues::Float32x3(norm_data),
            VertexAttributeValues::Float32x2(uv_data),
        ) = (normals, uvs)
        {
            let mut found_top = false;
            for (quad_norms, quad_uvs) in norm_data.chunks_exact(4).zip(uv_data.chunks_exact(4)) {
                if quad_norms[0][1] > 0.9 {
                    found_top = true;
                    let min_u = quad_uvs
                        .iter()
                        .map(|uv| uv[0])
                        .fold(f32::INFINITY, f32::min);
                    let max_u = quad_uvs
                        .iter()
                        .map(|uv| uv[0])
                        .fold(f32::NEG_INFINITY, f32::max);
                    assert!(
                        (max_u - min_u - 1.0).abs() < 1e-4,
                        "Merged top face must combine U across the two voxels to span 1.0, got {}",
                        max_u - min_u
                    );
                }
            }
            assert!(found_top, "Top face quad should exist");
        } else {
            panic!("Expected Float32x3 normals and Float32x2 UV_0 attributes");
        }
    }

    #[test]
    fn mesher_slab_sides_map_half_texture_uv() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, Voxel::Stone);
        chunk.set(1, 0, 0, Voxel::Stone);
        chunk.set(0, 0, 1, Voxel::Stone);
        chunk.set(1, 0, 1, Voxel::Stone);
        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let opaque = meshes.opaque.expect("Opaque mesh should exist");

        let normals = opaque
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .expect("Mesh should have normals");
        let uvs = opaque
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .expect("Mesh should have UV_0");

        if let (
            VertexAttributeValues::Float32x3(norm_data),
            VertexAttributeValues::Float32x2(uv_data),
        ) = (normals, uvs)
        {
            for (quad_norms, quad_uvs) in norm_data.chunks_exact(4).zip(uv_data.chunks_exact(4)) {
                if quad_norms[0][1].abs() < 0.1 {
                    let min_v = quad_uvs
                        .iter()
                        .map(|uv| uv[1])
                        .fold(f32::INFINITY, f32::min);
                    let max_v = quad_uvs
                        .iter()
                        .map(|uv| uv[1])
                        .fold(f32::NEG_INFINITY, f32::max);
                    assert!(
                        (min_v - 0.0).abs() < 1e-4,
                        "Bottom slab side min V must be 0.0, got {}",
                        min_v
                    );
                    assert!(
                        (max_v - 0.5).abs() < 1e-4,
                        "Bottom slab side max V must be 0.5, got {}",
                        max_v
                    );
                }
            }
        } else {
            panic!("Expected Float32x3 normals and Float32x2 UV_0 attributes");
        }
    }

    #[test]
    fn mesher_emissive_light_blocks_include_faces_and_emissive_metadata() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        for dy in 0..2 {
            for dz in 0..2 {
                for dx in 0..2 {
                    chunk.set(dx, dy, dz, Voxel::LightWarm);
                }
            }
        }
        world.insert_chunk(IVec3::ZERO, chunk);

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let opaque = meshes
            .opaque
            .expect("Opaque mesh should exist for light block");

        let uv_bs = opaque
            .attribute(Mesh::ATTRIBUTE_UV_1)
            .expect("Mesh should have UV_1");

        if let VertexAttributeValues::Float32x2(uv_b_data) = uv_bs {
            assert!(!uv_b_data.is_empty());
            for entry in uv_b_data {
                assert_eq!(entry[1], -4.0, "Light block emission metadata must be -4.0");
            }
        } else {
            panic!("Expected Float32x2 UV_1");
        }
    }

    #[test]
    fn mesher_early_exits_on_empty_air_chunk() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        assert!(meshes.opaque.is_none());
        assert!(meshes.transparent.is_none());
    }

    #[test]
    fn mesher_early_exits_on_fully_solid_chunk_surrounded_by_solid() {
        let mut world = VoxelWorld::default();
        // Insert central solid chunk
        world.insert_chunk(IVec3::ZERO, Chunk::filled(Voxel::Stone));

        // Insert all 6 neighbors as solid chunks
        for direction in FACE_DIRECTIONS {
            world.insert_chunk(direction.normal(), Chunk::filled(Voxel::Stone));
        }

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        assert!(meshes.opaque.is_none());
        assert!(meshes.transparent.is_none());
    }

    #[test]
    fn mesher_solid_chunk_adjacent_to_air_renders_only_exposed_boundary() {
        let mut world = VoxelWorld::default();
        // Insert central solid chunk
        world.insert_chunk(IVec3::ZERO, Chunk::filled(Voxel::Stone));

        // Insert 5 neighbors as solid, leave +Y (top) as air
        world.insert_chunk(IVec3::X, Chunk::filled(Voxel::Stone));
        world.insert_chunk(IVec3::NEG_X, Chunk::filled(Voxel::Stone));
        world.insert_chunk(IVec3::NEG_Y, Chunk::filled(Voxel::Stone));
        world.insert_chunk(IVec3::Z, Chunk::filled(Voxel::Stone));
        world.insert_chunk(IVec3::NEG_Z, Chunk::filled(Voxel::Stone));
        world.insert_chunk(IVec3::Y, Chunk::new()); // Empty air chunk on top

        let (_, registry) = build_voxel_texture_array();
        let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &registry);
        let opaque = meshes.opaque.expect("Should render top face against air");
        assert!(meshes.transparent.is_none());

        // Every vertex must point strictly upwards (Positive Y) because all other 5 sides are occluded!
        let normals = opaque.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap();
        if let VertexAttributeValues::Float32x3(norm_data) = normals {
            assert!(!norm_data.is_empty());
            for norm in norm_data {
                assert_eq!(
                    *norm,
                    [0.0, 1.0, 0.0],
                    "Only Positive Y faces should be rendered"
                );
            }
        } else {
            panic!("Expected Float32x3 normals");
        }
    }
}
