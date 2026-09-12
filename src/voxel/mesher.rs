use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::{IVec3, Mesh},
};

use super::{
    chunk::{CHUNK_SIZE, Chunk, VOXEL_SIZE, Voxel},
    shaping::{centered_layer_coordinates, is_centered_layer},
    texture::VoxelTextureRegistry,
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct FaceKey {
    voxel: Voxel,
    texture_layer: u16,
    frame_count: u16,
    tint_color: [f32; 4],
}

impl FaceKey {
    fn matches(self, other: Self) -> bool {
        self.voxel == other.voxel
            && self.texture_layer == other.texture_layer
            && self.frame_count == other.frame_count
            && self.tint_color == other.tint_color
    }
}

pub struct ChunkMeshes {
    pub opaque: Option<Mesh>,
    pub transparent: Option<Mesh>,
}

struct MeshBuffers {
    positions: Vec<[f32; 3]>,

    normals: Vec<[f32; 3]>,

    uvs: Vec<[f32; 2]>,

    uv_bs: Vec<[f32; 2]>,

    colors: Vec<[f32; 4]>,

    indices: Vec<u32>,
}

impl MeshBuffers {
    fn new() -> Self {
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

        let color = key.tint_color;
        let layer = key.texture_layer as f32;
        let frame_count = key.frame_count as f32;

        const WATER_SURFACE_OFFSET: f32 = 0.10;

        for (i, vertex) in vertices.iter().enumerate() {
            let mut pos = [
                vertex[0] * VOXEL_SIZE,
                vertex[1] * VOXEL_SIZE,
                vertex[2] * VOXEL_SIZE,
            ];

            if key.voxel == Voxel::Water {
                match direction {
                    FaceDirection::PositiveY => {
                        pos[1] -= WATER_SURFACE_OFFSET;
                    }
                    FaceDirection::PositiveX
                    | FaceDirection::NegativeX
                    | FaceDirection::PositiveZ
                    | FaceDirection::NegativeZ => {
                        if i == 1 || i == 2 {
                            pos[1] -= WATER_SURFACE_OFFSET;
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

        if key.voxel == Voxel::Water && direction == FaceDirection::PositiveY {
            let under_base_index = self.positions.len() as u32;

            for vertex in &vertices {
                let pos = [
                    vertex[0] * VOXEL_SIZE,
                    vertex[1] * VOXEL_SIZE - WATER_SURFACE_OFFSET,
                    vertex[2] * VOXEL_SIZE,
                ];
                self.positions.push(pos);
                self.normals.push([0.0, -1.0, 0.0]);
                self.colors.push(color);
                self.uv_bs.push([layer, frame_count]);
            }

            self.uvs
                .extend_from_slice(&[[0.0, 0.0], [0.0, height], [width, height], [width, 0.0]]);

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
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, self.uv_bs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
            .with_inserted_indices(Indices::U32(self.indices)),
        )
    }
}

pub struct ChunkMesher;

impl ChunkMesher {
    pub fn build_meshes(
        world: &VoxelWorld,
        chunk_coordinate: IVec3,
        textures: &VoxelTextureRegistry,
    ) -> ChunkMeshes {
        let Some(chunk) = world.get_chunk(chunk_coordinate) else {
            return ChunkMeshes {
                opaque: None,
                transparent: None,
            };
        };

        let chunk_voxel_origin = chunk_coordinate * CHUNK_SIZE as i32;

        let mut opaque_buffers = MeshBuffers::new();

        let mut transparent_buffers = MeshBuffers::new();

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

                        // Light blocks have their own
                        // emissive render entities.
                        if voxel.is_empty()
                            || voxel == Voxel::Light
                            || voxel == Voxel::Occupied
                            || is_chunk_local_centered_layer(chunk, local_voxel)
                        {
                            continue;
                        }

                        let world_voxel = chunk_voxel_origin + local_voxel;

                        let neighbor_coordinate = world_voxel + direction.normal();

                        let neighbor = world.get_voxel(neighbor_coordinate).unwrap_or(Voxel::Air);

                        let neighbor_is_centered = is_centered_layer(world, neighbor_coordinate);
                        if !neighbor_is_centered && !should_render_face(voxel, neighbor) {
                            continue;
                        }

                        let (texture_layer, frame_count) =
                            textures.get_texture_info(voxel, world_voxel);
                        let tint_color = voxel.tint_color_at(world_voxel);

                        mask[mask_index(u, v)] = Some(FaceKey {
                            voxel,
                            texture_layer,
                            frame_count,
                            tint_color,
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

        mesh_centered_voxels(
            world,
            chunk,
            chunk_coordinate,
            textures,
            &mut opaque_buffers,
        );

        ChunkMeshes {
            opaque: opaque_buffers.into_mesh(),

            transparent: transparent_buffers.into_mesh(),
        }
    }
}

fn should_render_face(voxel: Voxel, neighbor: Voxel) -> bool {
    if voxel.is_transparent() {
        // Water against Water does not generate
        // internal geometry.
        //
        // Water against solid terrain also does not
        // need a face because the solid face will be
        // visible through the transparent material.
        return neighbor.is_empty();
    }

    // Opaque terrain next to Water or Occupied must keep its face
    // because it needs to remain visible through Water or across the 0.25m gap.
    neighbor.is_empty() || neighbor.is_transparent() || neighbor == Voxel::Occupied
}

fn greedy_merge_mask(
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

            let buffers = if key.voxel.is_transparent() {
                &mut *transparent_buffers
            } else {
                &mut *opaque_buffers
            };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::{chunk::Chunk, texture::build_voxel_texture_array};
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
            // Top face (+Y) has max Y at 0.40m rather than 0.50m
            let max_y = pos_data
                .iter()
                .map(|p| p[1])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(
                (max_y - 0.40).abs() < 1e-4,
                "Surface water max Y should be 0.40, got {}",
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
        // Set up a centered voxel at layer y=0
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
            // Centered voxel is 0.5m wide, centered in 1.0m block -> X and Z in [0.25, 0.75]
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
        // Ground at y=0 is solid Sand across all 4 quadrants
        chunk.set(0, 0, 0, Voxel::Sand);
        chunk.set(1, 0, 0, Voxel::Sand);
        chunk.set(0, 0, 1, Voxel::Sand);
        chunk.set(1, 0, 1, Voxel::Sand);

        // Centered voxel directly above at y=1
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
                assert_eq!(
                    up_faces_at_ground, 16,
                    "All 4 ground quads under centered column must render their top face (16 vertices), got {}",
                    up_faces_at_ground
                );
            }
        }
    }
}

fn is_chunk_local_centered_layer(chunk: &Chunk, local_voxel: IVec3) -> bool {
    let bx = (local_voxel.x as usize / 2) * 2;
    let bz = (local_voxel.z as usize / 2) * 2;
    let y = local_voxel.y as usize;

    chunk.get(bx, y, bz) == Voxel::Occupied
        || chunk.get(bx + 1, y, bz) == Voxel::Occupied
        || chunk.get(bx, y, bz + 1) == Voxel::Occupied
        || chunk.get(bx + 1, y, bz + 1) == Voxel::Occupied
}

fn get_chunk_local_centered_material(chunk: &Chunk, local_voxel: IVec3) -> Option<Voxel> {
    let bx = (local_voxel.x as usize / 2) * 2;
    let bz = (local_voxel.z as usize / 2) * 2;
    let y = local_voxel.y as usize;

    for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        let v = chunk.get(bx + dx, y, bz + dz);
        if !v.is_empty() && v != Voxel::Occupied {
            return Some(v);
        }
    }
    None
}

fn mesh_centered_voxels(
    world: &VoxelWorld,
    chunk: &Chunk,
    chunk_coordinate: IVec3,
    textures: &VoxelTextureRegistry,
    opaque_buffers: &mut MeshBuffers,
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
                let (texture_layer, frame_count) = textures.get_texture_info(material, world_voxel);
                let tint_color = material.tint_color_at(world_voxel);

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
                    texture_layer,
                    frame_count,
                    tint_color,
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
                    texture_layer,
                    frame_count,
                    tint_color,
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
                    texture_layer,
                    frame_count,
                    tint_color,
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
                    texture_layer,
                    frame_count,
                    tint_color,
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
                        texture_layer,
                        frame_count,
                        tint_color,
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
                        texture_layer,
                        frame_count,
                        tint_color,
                    );
                }
            }
        }
    }
}

fn push_centered_quad(
    buffers: &mut MeshBuffers,
    vertices: [[f32; 3]; 4],
    normal: [f32; 3],
    texture_layer: u16,
    frame_count: u16,
    tint_color: [f32; 4],
) {
    let base_index = buffers.positions.len() as u32;

    for v in &vertices {
        buffers
            .positions
            .push([v[0] * VOXEL_SIZE, v[1] * VOXEL_SIZE, v[2] * VOXEL_SIZE]);
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
