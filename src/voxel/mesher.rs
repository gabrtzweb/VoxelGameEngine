use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::{IVec3, Mesh},
};

use super::{
    chunk::{CHUNK_SIZE, VOXEL_SIZE, Voxel},
    world::VoxelWorld,
};

const MASK_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaceDirection {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl FaceDirection {
    fn normal(self) -> IVec3 {
        match self {
            Self::PositiveX => IVec3::new(1, 0, 0),
            Self::NegativeX => IVec3::new(-1, 0, 0),
            Self::PositiveY => IVec3::new(0, 1, 0),
            Self::NegativeY => IVec3::new(0, -1, 0),
            Self::PositiveZ => IVec3::new(0, 0, 1),
            Self::NegativeZ => IVec3::new(0, 0, -1),
        }
    }

    fn normal_f32(self) -> [f32; 3] {
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

const FACE_DIRECTIONS: [FaceDirection; 6] = [
    FaceDirection::PositiveX,
    FaceDirection::NegativeX,
    FaceDirection::PositiveY,
    FaceDirection::NegativeY,
    FaceDirection::PositiveZ,
    FaceDirection::NegativeZ,
];

#[derive(Clone, Copy, Debug)]
struct FaceKey {
    voxel: Voxel,
    texture_layer: u16,
}

impl FaceKey {
    fn matches(self, other: Self) -> bool {
        self.voxel == other.voxel && self.texture_layer == other.texture_layer
    }
}

struct MeshBuffers {
    positions: Vec<[f32; 3]>,

    normals: Vec<[f32; 3]>,

    uvs: Vec<[f32; 2]>,

    colors: Vec<[f32; 4]>,

    indices: Vec<u32>,
}

impl MeshBuffers {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push_quad(
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

        let color = key.voxel.display_color();

        for vertex in vertices {
            self.positions.push([
                vertex[0] * VOXEL_SIZE,
                vertex[1] * VOXEL_SIZE,
                vertex[2] * VOXEL_SIZE,
            ]);

            self.normals.push(direction.normal_f32());

            self.colors.push(color);
        }

        let width = width as f32;

        let height = height as f32;

        self.uvs
            .extend_from_slice(&[[0.0, 0.0], [0.0, height], [width, height], [width, 0.0]]);

        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    fn into_mesh(self) -> Option<Mesh> {
        if self.indices.is_empty() {
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
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
            .with_inserted_indices(Indices::U32(self.indices)),
        )
    }
}

pub struct ChunkMesher;

impl ChunkMesher {
    pub fn build_mesh(world: &VoxelWorld, chunk_coordinate: IVec3) -> Option<Mesh> {
        let chunk = world.get_chunk(chunk_coordinate)?;

        let chunk_voxel_origin = chunk_coordinate * CHUNK_SIZE as i32;

        let mut buffers = MeshBuffers::new();

        for direction in FACE_DIRECTIONS {
            for slice in 0..CHUNK_SIZE {
                let mut mask: [Option<FaceKey>; MASK_SIZE] = [None; MASK_SIZE];

                for v in 0..CHUNK_SIZE {
                    for u in 0..CHUNK_SIZE {
                        let local_voxel = mask_to_voxel(direction, slice, u, v);

                        let voxel = chunk.get(
                            local_voxel.x as usize,
                            local_voxel.y as usize,
                            local_voxel.z as usize,
                        );

                        if voxel.is_empty() {
                            continue;
                        }

                        let world_voxel = chunk_voxel_origin + local_voxel;

                        let neighbor = world_voxel + direction.normal();

                        if world.get_voxel(neighbor).is_some_and(Voxel::occludes_faces) {
                            continue;
                        }

                        let texture_layer = texture_layer_for(voxel, direction);

                        mask[mask_index(u, v)] = Some(FaceKey {
                            voxel,
                            texture_layer,
                        });
                    }
                }

                greedy_merge_mask(&mut mask, direction, slice, &mut buffers);
            }
        }

        buffers.into_mesh()
    }
}

fn greedy_merge_mask(
    mask: &mut [Option<FaceKey>; MASK_SIZE],
    direction: FaceDirection,
    slice: usize,
    buffers: &mut MeshBuffers,
) {
    for v in 0..CHUNK_SIZE {
        let mut u = 0;

        while u < CHUNK_SIZE {
            let index = mask_index(u, v);

            let Some(key) = mask[index] else {
                u += 1;
                continue;
            };

            let mut width = 1;

            while u + width < CHUNK_SIZE {
                let candidate = mask[mask_index(u + width, v)];

                let Some(candidate) = candidate else {
                    break;
                };

                if !candidate.matches(key) {
                    break;
                }

                width += 1;
            }

            let mut height = 1;

            'height_search: while v + height < CHUNK_SIZE {
                for offset in 0..width {
                    let candidate = mask[mask_index(u + offset, v + height)];

                    let Some(candidate) = candidate else {
                        break 'height_search;
                    };

                    if !candidate.matches(key) {
                        break 'height_search;
                    }
                }

                height += 1;
            }

            buffers.push_quad(direction, key, slice, u, v, width, height);

            for clear_v in v..v + height {
                for clear_u in u..u + width {
                    mask[mask_index(clear_u, clear_v)] = None;
                }
            }

            u += width;
        }
    }
}

fn mask_index(u: usize, v: usize) -> usize {
    u + v * CHUNK_SIZE
}

fn mask_to_voxel(direction: FaceDirection, slice: usize, u: usize, v: usize) -> IVec3 {
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

fn quad_vertices(
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

fn texture_layer_for(voxel: Voxel, direction: FaceDirection) -> u16 {
    match voxel {
        Voxel::Air => 0,

        Voxel::Grass => match direction {
            FaceDirection::PositiveY => 0,
            FaceDirection::NegativeY => 2,
            _ => 1,
        },

        Voxel::Dirt => 2,
        Voxel::Stone => 3,
        Voxel::Sand => 4,
        Voxel::Water => 5,
        Voxel::Light => 6,
    }
}
