use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::{IVec3, Mesh},
};

use super::{
    chunk::{CHUNK_SIZE, VOXEL_SIZE, Voxel},
    world::VoxelWorld,
};

struct Face {
    neighbor: IVec3,
    normal: [f32; 3],
    vertices: [[f32; 3]; 4],
}

const FACES: [Face; 6] = [
    Face {
        neighbor: IVec3::new(1, 0, 0),
        normal: [1.0, 0.0, 0.0],
        vertices: [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
    },
    Face {
        neighbor: IVec3::new(-1, 0, 0),
        normal: [-1.0, 0.0, 0.0],
        vertices: [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
    },
    Face {
        neighbor: IVec3::new(0, 1, 0),
        normal: [0.0, 1.0, 0.0],
        vertices: [
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
    },
    Face {
        neighbor: IVec3::new(0, -1, 0),
        normal: [0.0, -1.0, 0.0],
        vertices: [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
        ],
    },
    Face {
        neighbor: IVec3::new(0, 0, 1),
        normal: [0.0, 0.0, 1.0],
        vertices: [
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    },
    Face {
        neighbor: IVec3::new(0, 0, -1),
        normal: [0.0, 0.0, -1.0],
        vertices: [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
    },
];

const FACE_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

pub struct ChunkMesher;

impl ChunkMesher {
    pub fn build_mesh(world: &VoxelWorld, chunk_coordinate: IVec3) -> Option<Mesh> {
        let chunk = world.get_chunk(chunk_coordinate)?;

        let mut positions = Vec::new();

        let mut normals = Vec::new();

        let mut uvs = Vec::new();

        let mut indices = Vec::new();

        let chunk_voxel_origin = chunk_coordinate * CHUNK_SIZE as i32;

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if chunk.get(x, y, z) == Voxel::Air {
                        continue;
                    }

                    let world_voxel = chunk_voxel_origin + IVec3::new(x as i32, y as i32, z as i32);

                    for face in &FACES {
                        let neighbor = world_voxel + face.neighbor;

                        if world.get_voxel(neighbor) == Some(Voxel::Solid) {
                            continue;
                        }

                        let base_index = positions.len() as u32;

                        let voxel_origin = [
                            x as f32 * VOXEL_SIZE,
                            y as f32 * VOXEL_SIZE,
                            z as f32 * VOXEL_SIZE,
                        ];

                        for vertex in face.vertices {
                            positions.push([
                                voxel_origin[0] + vertex[0] * VOXEL_SIZE,
                                voxel_origin[1] + vertex[1] * VOXEL_SIZE,
                                voxel_origin[2] + vertex[2] * VOXEL_SIZE,
                            ]);

                            normals.push(face.normal);
                        }

                        uvs.extend_from_slice(&FACE_UVS);

                        indices.extend_from_slice(&[
                            base_index,
                            base_index + 1,
                            base_index + 2,
                            base_index,
                            base_index + 2,
                            base_index + 3,
                        ]);
                    }
                }
            }
        }

        if indices.is_empty() {
            return None;
        }

        Some(
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(Indices::U32(indices)),
        )
    }
}
