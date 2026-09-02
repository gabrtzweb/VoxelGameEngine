use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::Mesh,
};

use super::chunk::{Chunk, Voxel, CHUNK_SIZE, VOXEL_SIZE};

struct Face {
    neighbor: (isize, isize, isize),
    normal: [f32; 3],
    vertices: [[f32; 3]; 4],
}

const FACES: [Face; 6] = [
    // Right (+X)
    Face {
        neighbor: (1, 0, 0),
        normal: [1.0, 0.0, 0.0],
        vertices: [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
    },
    // Left (-X)
    Face {
        neighbor: (-1, 0, 0),
        normal: [-1.0, 0.0, 0.0],
        vertices: [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
    },
    // Top (+Y)
    Face {
        neighbor: (0, 1, 0),
        normal: [0.0, 1.0, 0.0],
        vertices: [
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
    },
    // Bottom (-Y)
    Face {
        neighbor: (0, -1, 0),
        normal: [0.0, -1.0, 0.0],
        vertices: [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
        ],
    },
    // Front (+Z)
    Face {
        neighbor: (0, 0, 1),
        normal: [0.0, 0.0, 1.0],
        vertices: [
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    },
    // Back (-Z)
    Face {
        neighbor: (0, 0, -1),
        normal: [0.0, 0.0, -1.0],
        vertices: [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
    },
];

const FACE_UVS: [[f32; 2]; 4] = [
    [0.0, 0.0],
    [0.0, 1.0],
    [1.0, 1.0],
    [1.0, 0.0],
];

pub struct ChunkMesher;

impl ChunkMesher {
    pub fn build_mesh(chunk: &Chunk) -> Mesh {
        let face_count = Self::exposed_face_count(chunk);

        let mut positions = Vec::with_capacity(face_count * 4);
        let mut normals = Vec::with_capacity(face_count * 4);
        let mut uvs = Vec::with_capacity(face_count * 4);
        let mut indices = Vec::with_capacity(face_count * 6);

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if chunk.get(x, y, z) == Voxel::Air {
                        continue;
                    }

                    for face in &FACES {
                        let neighbor_x = x as isize + face.neighbor.0;
                        let neighbor_y = y as isize + face.neighbor.1;
                        let neighbor_z = z as isize + face.neighbor.2;

                        if !Self::is_face_exposed(
                            chunk,
                            neighbor_x,
                            neighbor_y,
                            neighbor_z,
                        ) {
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

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
    }

    pub fn exposed_face_count(chunk: &Chunk) -> usize {
        let mut face_count = 0;

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if chunk.get(x, y, z) == Voxel::Air {
                        continue;
                    }

                    for face in &FACES {
                        let neighbor_x = x as isize + face.neighbor.0;
                        let neighbor_y = y as isize + face.neighbor.1;
                        let neighbor_z = z as isize + face.neighbor.2;

                        if Self::is_face_exposed(
                            chunk,
                            neighbor_x,
                            neighbor_y,
                            neighbor_z,
                        ) {
                            face_count += 1;
                        }
                    }
                }
            }
        }

        face_count
    }

    fn is_face_exposed(
        chunk: &Chunk,
        x: isize,
        y: isize,
        z: isize,
    ) -> bool {
        if !Chunk::is_inside(x, y, z) {
            return true;
        }

        chunk.get(x as usize, y as usize, z as usize) == Voxel::Air
    }
}