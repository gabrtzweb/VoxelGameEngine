use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::{IVec3, Mesh},
};

use super::{shapes::mesh_shaped_voxels, textures::VoxelTextureRegistry};
use crate::{
    simulation::fluid::water_surface_height_offset,
    world::{BlockShape, CHUNK_SIZE, CHUNK_VOLUME, Chunk, VOXEL_SIZE, Voxel, VoxelAccess},
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
    pub visibility_mask: u64,
}

pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub uv_bs: Vec<[f32; 2]>,
    pub colors: Vec<[u8; 4]>,
    pub indices: Vec<u32>,
}

/// Packs a 4-channel float color `[r, g, b, a]` (0.0..1.0) into 4 normalized bytes `[u8; 4]`.
/// Enables GPU hardware vertex fetch decompression from 4 bytes into `vec4<f32>` (Unorm8x4).
#[inline(always)]
pub fn pack_color(color: [f32; 4]) -> [u8; 4] {
    [
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

impl MeshBuffers {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(quads: usize) -> Self {
        let verts = quads * 4;
        let idxs = quads * 6;
        Self {
            positions: Vec::with_capacity(verts),
            normals: Vec::with_capacity(verts),
            uvs: Vec::with_capacity(verts),
            uv_bs: Vec::with_capacity(verts),
            colors: Vec::with_capacity(verts),
            indices: Vec::with_capacity(idxs),
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.positions.clear();
        self.normals.clear();
        self.uvs.clear();
        self.uv_bs.clear();
        self.colors.clear();
        self.indices.clear();
    }

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

        let mut color = key.tint_color;
        color[3] = if key.voxel.is_water() {
            0.5
        } else if key.voxel.is_fluid() {
            1.0
        } else if key.voxel.is_transparent() {
            0.0
        } else {
            1.0
        };
        let packed_color = pack_color(color);
        let layer = key.texture_layer as f32;
        let frame_count = if key.voxel.is_light() {
            -(key.frame_count.max(1) as f32)
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

            if key.voxel.is_fluid() {
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
            self.colors.push(packed_color);
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

        if key.voxel.is_fluid() && direction == FaceDirection::PositiveY {
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
                self.colors.push(packed_color);
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

        let indices = if self.positions.len() <= 65535 {
            Indices::U16(self.indices.iter().map(|&i| i as u16).collect())
        } else {
            Indices::U32(self.indices)
        };

        Some(
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, self.uv_bs)
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_COLOR,
                VertexAttributeValues::Unorm8x4(self.colors),
            )
            .with_inserted_indices(indices),
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

            if voxel.is_empty()
                || chunk.get_shape(lx, ly, lz).0 != BlockShape::Full
                || voxel.has_custom_mesh()
            {
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
                visibility_mask: (1u64 << 36) - 1,
            };
        };

        // Early-exit 1: A chunk with zero non-air voxels contains no geometry.
        if chunk.is_empty() {
            return ChunkMeshes {
                opaque: None,
                transparent: None,
                visibility_mask: (1u64 << 36) - 1,
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
                    visibility_mask: 0,
                };
            }
        }

        let chunk_voxel_origin = chunk_coordinate * CHUNK_SIZE as i32;

        let mut opaque_buffers = MeshBuffers::with_capacity(256);
        let mut transparent_buffers = MeshBuffers::with_capacity(64);

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

                        if voxel.is_empty()
                            || chunk.get_shape(lx, ly, lz).0 != BlockShape::Full
                            || voxel.has_custom_mesh()
                        {
                            continue;
                        }

                        let world_voxel = chunk_voxel_origin + local_voxel;
                        let neighbor_coordinate = world_voxel + direction.normal();
                        let neighbor = world.get_voxel(neighbor_coordinate).unwrap_or(Voxel::Air);
                        let (neighbor_shape, _) = world.get_shape(neighbor_coordinate);
                        let mut step_bottom_offset_cm = 0u8;

                        if voxel.is_fluid()
                            && direction != FaceDirection::PositiveY
                            && direction != FaceDirection::NegativeY
                        {
                            if neighbor == voxel {
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
                            } else if !should_render_face(voxel, neighbor) {
                                continue;
                            }
                        } else if neighbor_shape == BlockShape::Full
                            && !should_render_face(voxel, neighbor)
                        {
                            continue;
                        }

                        let (texture_layer, frame_count) =
                            textures.get_face_texture_info(voxel, world_voxel, direction);
                        let is_grass = matches!(
                            voxel,
                            Voxel::Soil_Grass | Voxel::Soil_Peat_Grass | Voxel::Soil_Silt_Grass
                        );
                        let mut tint_color = if is_grass {
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

                        tint_color[3] = if voxel.is_water() {
                            1.0
                        } else if voxel.is_fluid() {
                            2.0
                        } else if voxel.is_transparent() {
                            0.0
                        } else {
                            1.0
                        };

                        let surface_offset_cm = if voxel.is_fluid() {
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

        let visibility_mask = compute_chunk_visibility_mask(chunk);

        ChunkMeshes {
            opaque: opaque_buffers.into_mesh(),
            transparent: transparent_buffers.into_mesh(),
            visibility_mask,
        }
    }
}

pub fn should_render_face(voxel: Voxel, neighbor: Voxel) -> bool {
    if voxel.is_fluid() {
        return neighbor.is_empty()
            || neighbor == Voxel::Occupied
            || (neighbor.is_fluid() && neighbor != voxel)
            || (!neighbor.is_fluid() && (neighbor.is_transparent() || neighbor.is_leaves()));
    }

    if voxel.is_transparent() {
        return neighbor.is_empty()
            || neighbor == Voxel::Occupied
            || neighbor.is_fluid()
            || neighbor.is_leaves()
            || (neighbor.is_transparent() && neighbor != voxel);
    }

    if voxel.is_leaves() {
        // Leaves cull against solid opaque blocks (trunks, branches, dirt, stone)
        // but render against air, water, and adjacent leaves for dense volumetric foliage.
        return !neighbor.is_solid_opaque();
    }

    // Solid opaque blocks render against air, water, leaves, fluids, or occupied cells
    neighbor.is_empty()
        || neighbor.is_transparent()
        || neighbor.is_leaves()
        || neighbor.is_fluid()
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

            let target_buffers = if key.voxel.is_transparent() || key.voxel.is_fluid() {
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

pub fn face_uv_to_voxel(face: usize, u: usize, v: usize) -> (usize, usize, usize) {
    match face {
        0 => (0, v, u),              // NegativeX: x=0, y=v, z=u
        1 => (CHUNK_SIZE - 1, v, u), // PositiveX: x=15, y=v, z=u
        2 => (u, 0, v),              // NegativeY: x=u, y=0, z=v
        3 => (u, CHUNK_SIZE - 1, v), // PositiveY: x=u, y=15, z=v
        4 => (u, v, 0),              // NegativeZ: x=u, y=v, z=0
        _ => (u, v, CHUNK_SIZE - 1), // PositiveZ: x=u, y=v, z=15
    }
}

/// Evaluates face-to-face visibility through non-solid voxels inside a chunk.
///
/// Returns a 36-bit bitmask where bit `(in_face * 6 + out_face)` is set if sight/air
/// can traverse continuously from `in_face` to `out_face` without being blocked by solid opaque blocks.
pub fn compute_chunk_visibility_mask(chunk: &Chunk) -> u64 {
    if chunk.is_empty() {
        return (1u64 << 36) - 1;
    }

    if chunk.is_fully_solid_opaque() {
        return 0;
    }

    // 4096 bits = 64 u64 words
    let mut visited = [0u64; 64];

    // Mask solid opaque blocks as visited so flood fill cannot enter them
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let voxel = chunk.get(x, y, z);
                if voxel.is_solid_opaque() && chunk.get_shape(x, y, z).0 == BlockShape::Full {
                    let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                    visited[idx / 64] |= 1u64 << (idx % 64);
                }
            }
        }
    }

    let mut connectivity: u64 = 0;
    // Chunk volume is 4096 voxels. Each voxel index is visited at most once,
    // so a fixed stack buffer of 4096 u16 avoids all heap allocations in the mesher.
    let mut queue = [0u16; CHUNK_VOLUME];

    for start_face in 0..6 {
        for v in 0..CHUNK_SIZE {
            for u in 0..CHUNK_SIZE {
                let (x, y, z) = face_uv_to_voxel(start_face, u, v);
                let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;

                if (visited[idx / 64] & (1u64 << (idx % 64))) != 0 {
                    continue;
                }

                visited[idx / 64] |= 1u64 << (idx % 64);
                let mut head = 0usize;
                let mut tail = 0usize;
                queue[tail] = idx as u16;
                tail += 1;

                let mut touched_faces = 1u8 << start_face;

                while head < tail {
                    let cur = queue[head] as usize;
                    head += 1;

                    let cx = cur % CHUNK_SIZE;
                    let cz = (cur / CHUNK_SIZE) % CHUNK_SIZE;
                    let cy = cur / (CHUNK_SIZE * CHUNK_SIZE);

                    if cx == 0 {
                        touched_faces |= 1 << 0;
                    }
                    if cx == CHUNK_SIZE - 1 {
                        touched_faces |= 1 << 1;
                    }
                    if cy == 0 {
                        touched_faces |= 1 << 2;
                    }
                    if cy == CHUNK_SIZE - 1 {
                        touched_faces |= 1 << 3;
                    }
                    if cz == 0 {
                        touched_faces |= 1 << 4;
                    }
                    if cz == CHUNK_SIZE - 1 {
                        touched_faces |= 1 << 5;
                    }

                    if cx > 0 {
                        let n_idx = cur - 1;
                        if (visited[n_idx / 64] & (1u64 << (n_idx % 64))) == 0 {
                            visited[n_idx / 64] |= 1u64 << (n_idx % 64);
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                    if cx + 1 < CHUNK_SIZE {
                        let n_idx = cur + 1;
                        if (visited[n_idx / 64] & (1u64 << (n_idx % 64))) == 0 {
                            visited[n_idx / 64] |= 1u64 << (n_idx % 64);
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                    if cz > 0 {
                        let n_idx = cur - CHUNK_SIZE;
                        if (visited[n_idx / 64] & (1u64 << (n_idx % 64))) == 0 {
                            visited[n_idx / 64] |= 1u64 << (n_idx % 64);
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                    if cz + 1 < CHUNK_SIZE {
                        let n_idx = cur + CHUNK_SIZE;
                        if (visited[n_idx / 64] & (1u64 << (n_idx % 64))) == 0 {
                            visited[n_idx / 64] |= 1u64 << (n_idx % 64);
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                    if cy > 0 {
                        let n_idx = cur - CHUNK_SIZE * CHUNK_SIZE;
                        if (visited[n_idx / 64] & (1u64 << (n_idx % 64))) == 0 {
                            visited[n_idx / 64] |= 1u64 << (n_idx % 64);
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                    if cy + 1 < CHUNK_SIZE {
                        let n_idx = cur + CHUNK_SIZE * CHUNK_SIZE;
                        if (visited[n_idx / 64] & (1u64 << (n_idx % 64))) == 0 {
                            visited[n_idx / 64] |= 1u64 << (n_idx % 64);
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                }

                for f1 in 0..6 {
                    if (touched_faces & (1 << f1)) != 0 {
                        for f2 in 0..6 {
                            if (touched_faces & (1 << f2)) != 0 {
                                connectivity |= 1u64 << (f1 * 6 + f2);
                            }
                        }
                    }
                }
            }
        }
    }

    connectivity
}
