use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::{IVec3, Mesh},
};

use super::{shapes::mesh_shaped_voxels, textures::VoxelTextureRegistry};
use crate::{
    simulation::fluid::water_surface_height_offset,
    world::{BlockShape, CHUNK_SIZE, Chunk, VOXEL_SIZE, Voxel, VoxelAccess},
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

pub const FACE_NORMALS_IVEC3: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

pub const FACE_NORMALS_F32: [[f32; 3]; 6] = [
    [1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, -1.0],
];

impl FaceDirection {
    #[inline(always)]
    pub fn normal(self) -> IVec3 {
        FACE_NORMALS_IVEC3[self as usize]
    }

    #[inline(always)]
    pub fn normal_f32(self) -> [f32; 3] {
        FACE_NORMALS_F32[self as usize]
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

        let u_min = u as f32;
        let u_max = (u + width) as f32;
        let v_min = v as f32;
        let v_max = (v + height) as f32;

        let uvs_to_push = match direction {
            FaceDirection::PositiveY | FaceDirection::NegativeY => [
                [u_min, v_min],
                [u_min, v_max],
                [u_max, v_max],
                [u_max, v_min],
            ],
            _ => [
                [u_min, v_max],
                [u_min, v_min],
                [u_max, v_min],
                [u_max, v_max],
            ],
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SliceBitmask {
    pub opaque: [u16; CHUNK_SIZE],
    pub special: [u16; CHUNK_SIZE],
}

pub fn extract_slice_bitmask(
    chunk: &Chunk,
    direction: FaceDirection,
    slice: usize,
) -> SliceBitmask {
    if chunk.is_empty() {
        return SliceBitmask::default();
    }

    if chunk.is_fully_solid_opaque() {
        return SliceBitmask {
            opaque: [0xFFFF; CHUNK_SIZE],
            special: [0; CHUNK_SIZE],
        };
    }

    let mut mask = SliceBitmask::default();

    for v in 0..CHUNK_SIZE {
        let mut row_opaque = 0u16;
        let mut row_special = 0u16;

        for u in 0..CHUNK_SIZE {
            let local_voxel = mask_to_voxel(direction, slice, u, v);
            let lx = local_voxel.x as usize;
            let ly = local_voxel.y as usize;
            let lz = local_voxel.z as usize;
            let voxel = chunk.get(lx, ly, lz);

            if voxel.is_empty() || chunk.get_shape(lx, ly, lz).0 != BlockShape::Full {
                continue;
            }

            let bit = 1u16 << u;
            if voxel.is_solid_opaque() {
                row_opaque |= bit;
            } else {
                row_special |= bit;
            }
        }

        mask.opaque[v] = row_opaque;
        mask.special[v] = row_special;
    }

    mask
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

            // Pre-extract slice bitmasks for the current chunk along this direction
            let chunk_slices: [SliceBitmask; CHUNK_SIZE] =
                std::array::from_fn(|s| extract_slice_bitmask(chunk, direction, s));

            // Boundary neighbor slice bitmask (from adjacent chunk)
            let neighbor_boundary_slice = {
                let neighbor_chunk_coord = chunk_coordinate + direction.normal();
                if let Some(neighbor_chunk) = world.get_chunk(neighbor_chunk_coord) {
                    let boundary_s = match direction {
                        FaceDirection::PositiveX
                        | FaceDirection::PositiveY
                        | FaceDirection::PositiveZ => 0,
                        FaceDirection::NegativeX
                        | FaceDirection::NegativeY
                        | FaceDirection::NegativeZ => CHUNK_SIZE - 1,
                    };
                    extract_slice_bitmask(neighbor_chunk, direction, boundary_s)
                } else {
                    SliceBitmask::default()
                }
            };

            for slice in slice_range {
                let current_slice = chunk_slices[slice];
                let neighbor_slice = match direction {
                    FaceDirection::PositiveX
                    | FaceDirection::PositiveY
                    | FaceDirection::PositiveZ => {
                        if slice + 1 < CHUNK_SIZE {
                            chunk_slices[slice + 1]
                        } else {
                            neighbor_boundary_slice
                        }
                    }
                    FaceDirection::NegativeX
                    | FaceDirection::NegativeY
                    | FaceDirection::NegativeZ => {
                        if slice > 0 {
                            chunk_slices[slice - 1]
                        } else {
                            neighbor_boundary_slice
                        }
                    }
                };

                let mut mask: [Option<FaceKey>; MASK_SIZE] = [None; MASK_SIZE];

                for v in 0..CHUNK_SIZE {
                    let current_opaque = current_slice.opaque[v];
                    let neighbor_opaque = neighbor_slice.opaque[v];

                    // Bitwise Face Culling across all 16 voxels in the row:
                    // An opaque face is visible ONLY if neighbor is NOT opaque.
                    let visible_opaque = current_opaque & !neighbor_opaque;
                    let special_voxels = current_slice.special[v];

                    let mut candidate_bits = visible_opaque | special_voxels;
                    if candidate_bits == 0 {
                        // Entire 16-voxel row has 0 exposed faces: skip all world queries
                        continue;
                    }

                    // Extract candidate positions using CPU intrinsic trailing_zeros
                    while candidate_bits != 0 {
                        let u = candidate_bits.trailing_zeros() as usize;
                        candidate_bits &= candidate_bits - 1; // Clear lowest set bit

                        let local_voxel = mask_to_voxel(direction, slice, u, v);
                        let lx = local_voxel.x as usize;
                        let ly = local_voxel.y as usize;
                        let lz = local_voxel.z as usize;
                        let voxel = chunk.get(lx, ly, lz);

                        if voxel.is_empty() || chunk.get_shape(lx, ly, lz).0 != BlockShape::Full {
                            continue;
                        }

                        let world_voxel = chunk_voxel_origin + local_voxel;
                        let neighbor_coordinate = world_voxel + direction.normal();
                        let neighbor = world.get_voxel(neighbor_coordinate).unwrap_or(Voxel::Air);
                        let (neighbor_shape, _) = world.get_shape(neighbor_coordinate);
                        let mut step_bottom_offset_cm = 0u8;

                        if voxel.is_water()
                            && direction != FaceDirection::PositiveY
                            && direction != FaceDirection::NegativeY
                        {
                            if neighbor.is_water() {
                                let v_offset = (water_surface_height_offset(world, world_voxel)
                                    * 100.0)
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
                            } else if !neighbor.is_empty() {
                                continue;
                            }
                        } else if neighbor_shape == BlockShape::Full
                            && !should_render_face(voxel, neighbor)
                        {
                            continue;
                        }

                        let (texture_layer, frame_count) =
                            textures.get_face_texture_info(voxel, world_voxel, direction);
                        let tint_color = if voxel == Voxel::Grass {
                            match direction {
                                FaceDirection::PositiveY => voxel.tint_color_at(world_voxel),
                                FaceDirection::NegativeY => [1.0, 1.0, 1.0, 1.0],
                                _ => {
                                    if textures.full_grass {
                                        voxel.tint_color_at(world_voxel)
                                    } else {
                                        [1.0, 1.0, 1.0, 1.0]
                                    }
                                }
                            }
                        } else {
                            voxel.tint_color_at(world_voxel)
                        };

                        let surface_offset_cm = if voxel.is_water() {
                            (water_surface_height_offset(world, world_voxel) * 100.0).round() as u8
                        } else {
                            0
                        };

                        let is_isolated_voxel = false;

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

        if chunk.has_shapes() {
            mesh_shaped_voxels(
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

    if voxel.is_leaves() {
        // Leaves cull against solid opaque blocks (trunks, branches, dirt, stone)
        // but render against air, water, and adjacent leaves for dense volumetric foliage.
        return !neighbor.is_solid_opaque();
    }

    // Solid opaque blocks render against air, water, leaves, or occupied cells
    neighbor.is_empty()
        || neighbor.is_transparent()
        || neighbor.is_leaves()
        || neighbor == Voxel::Occupied
}

pub fn greedy_merge_mask(
    mask: &mut [Option<FaceKey>; MASK_SIZE],
    direction: FaceDirection,
    slice: usize,
    opaque_buffers: &mut MeshBuffers,
    transparent_buffers: &mut MeshBuffers,
) {
    let mut populated = [0u16; CHUNK_SIZE];
    for v in 0..CHUNK_SIZE {
        let mut row_bits = 0u16;
        for u in 0..CHUNK_SIZE {
            if mask[mask_index(u, v)].is_some() {
                row_bits |= 1 << u;
            }
        }
        populated[v] = row_bits;
    }

    for v in 0..CHUNK_SIZE {
        while populated[v] != 0 {
            let u = populated[v].trailing_zeros() as usize;

            let index = mask_index(u, v);
            let Some(key) = mask[index] else {
                populated[v] &= !(1 << u);
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

            let width_mask = if width >= 16 {
                0xFFFFu16
            } else {
                ((1u16 << width) - 1) << u
            };
            let clear_mask = !width_mask;

            for dv in 0..height {
                populated[v + dv] &= clear_mask;
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
