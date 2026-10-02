use std::collections::{HashSet, VecDeque};
use bevy::prelude::*;

use crate::{
    meshing::pipeline::ChunkMeshRegistry,
    player::PlayerCamera,
    world::{block::BlockShape, VoxelWorld, CHUNK_SIZE, VOXEL_SIZE},
};

/// Component attached to chunk mesh entities storing their grid coordinate.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoordinate(pub IVec3);

/// Marker component attached to chunk mesh entities that are completely subterranean (buried under solid surface terrain).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubterraneanChunkMesh;

/// State tracker for cave occlusion culling and debug telemetry.
#[derive(Resource, Debug, Clone)]
pub struct CaveCullingState {
    #[allow(dead_code)]
    pub is_player_underground: bool,
    pub last_player_chunk: IVec3,
    pub last_player_block: IVec3,
    pub rendered_chunks: usize,
    pub culled_chunks: usize,
    #[allow(dead_code)]
    pub rendered_vertices: usize,
    #[allow(dead_code)]
    pub culled_vertices: usize,
}

impl Default for CaveCullingState {
    fn default() -> Self {
        Self {
            is_player_underground: false,
            last_player_chunk: IVec3::splat(i32::MIN),
            last_player_block: IVec3::splat(i32::MIN),
            rendered_chunks: 0,
            culled_chunks: 0,
            rendered_vertices: 0,
            culled_vertices: 0,
        }
    }
}

pub const OPPOSITE_FACES: [usize; 6] = [1, 0, 3, 2, 5, 4];
pub const FACE_OFFSETS: [IVec3; 6] = [
    IVec3::new(-1, 0, 0), // NegativeX (0)
    IVec3::new(1, 0, 0),  // PositiveX (1)
    IVec3::new(0, -1, 0), // NegativeY (2)
    IVec3::new(0, 1, 0),  // PositiveY (3)
    IVec3::new(0, 0, -1), // NegativeZ (4)
    IVec3::new(0, 0, 1),  // PositiveZ (5)
];

const MAX_HORIZONTAL_RADIUS: i32 = 14;
const MAX_VERTICAL_RADIUS: i32 = 12;

/// Determines which boundary faces of the camera chunk are reachable from the camera's voxel.
pub fn get_camera_reachable_faces(
    cam_pos: Vec3,
    cam_chunk: IVec3,
    chunk_mask: u64,
    world: Option<&VoxelWorld>,
) -> u8 {
    let Some(w) = world else {
        // If world is None (e.g. testing), allow all faces with self-connectivity bits in mask
        let mut faces = 0u8;
        for f in 0..6 {
            if (chunk_mask & (1u64 << (f * 6 + f))) != 0 {
                faces |= 1 << f;
            }
        }
        return faces;
    };

    let Some(chunk) = w.get_chunk(cam_chunk) else {
        return 0x3F;
    };

    if chunk.is_empty() {
        return 0x3F;
    }

    if chunk.is_fully_solid_opaque() {
        return 0;
    }

    let chunk_origin = cam_chunk * CHUNK_SIZE as i32;
    let lx = ((cam_pos.x - chunk_origin.x as f32) / VOXEL_SIZE).floor() as i32;
    let ly = ((cam_pos.y - chunk_origin.y as f32) / VOXEL_SIZE).floor() as i32;
    let lz = ((cam_pos.z - chunk_origin.z as f32) / VOXEL_SIZE).floor() as i32;

    let cx = lx.clamp(0, CHUNK_SIZE as i32 - 1) as usize;
    let cy = ly.clamp(0, CHUNK_SIZE as i32 - 1) as usize;
    let cz = lz.clamp(0, CHUNK_SIZE as i32 - 1) as usize;

    let start_voxel = chunk.get(cx, cy, cz);
    if start_voxel.is_solid_opaque() && chunk.get_shape(cx, cy, cz).0 == BlockShape::Full {
        return 0;
    }

    // Flood fill inside camera chunk from (cx, cy, cz)
    let mut visited = [0u64; 64];

    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let v = chunk.get(x, y, z);
                if v.is_solid_opaque() && chunk.get_shape(x, y, z).0 == BlockShape::Full {
                    let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                    visited[idx / 64] |= 1u64 << (idx % 64);
                }
            }
        }
    }

    let start_idx = cx + cz * CHUNK_SIZE + cy * CHUNK_SIZE * CHUNK_SIZE;
    visited[start_idx / 64] |= 1u64 << (start_idx % 64);

    let mut queue = Vec::with_capacity(256);
    queue.push(start_idx);

    let mut reachable = 0u8;
    let mut head = 0;

    while head < queue.len() {
        let cur = queue[head];
        head += 1;

        let vx = cur % CHUNK_SIZE;
        let vz = (cur / CHUNK_SIZE) % CHUNK_SIZE;
        let vy = cur / (CHUNK_SIZE * CHUNK_SIZE);

        if vx == 0 { reachable |= 1 << 0; }
        if vx == CHUNK_SIZE - 1 { reachable |= 1 << 1; }
        if vy == 0 { reachable |= 1 << 2; }
        if vy == CHUNK_SIZE - 1 { reachable |= 1 << 3; }
        if vz == 0 { reachable |= 1 << 4; }
        if vz == CHUNK_SIZE - 1 { reachable |= 1 << 5; }

        let push_n = |n_idx: usize, visited_bits: &mut [u64; 64], q: &mut Vec<usize>| {
            if (visited_bits[n_idx / 64] & (1u64 << (n_idx % 64))) == 0 {
                visited_bits[n_idx / 64] |= 1u64 << (n_idx % 64);
                q.push(n_idx);
            }
        };

        if vx > 0 { push_n(cur - 1, &mut visited, &mut queue); }
        if vx + 1 < CHUNK_SIZE { push_n(cur + 1, &mut visited, &mut queue); }
        if vz > 0 { push_n(cur - CHUNK_SIZE, &mut visited, &mut queue); }
        if vz + 1 < CHUNK_SIZE { push_n(cur + CHUNK_SIZE, &mut visited, &mut queue); }
        if vy > 0 { push_n(cur - CHUNK_SIZE * CHUNK_SIZE, &mut visited, &mut queue); }
        if vy + 1 < CHUNK_SIZE { push_n(cur + CHUNK_SIZE * CHUNK_SIZE, &mut visited, &mut queue); }
    }

    reachable
}

/// Returns the 36-bit face-permeability mask for a chunk coordinate.
pub fn get_chunk_permeability(
    coord: IVec3,
    registry: &ChunkMeshRegistry,
    world: Option<&VoxelWorld>,
) -> u64 {
    if let Some(mask) = registry.get_visibility_mask_opt(&coord) {
        return mask;
    }

    if let Some(w) = world {
        if let Some(chunk) = w.get_chunk(coord) {
            if chunk.is_empty() {
                return (1u64 << 36) - 1;
            }
            if chunk.is_fully_solid_opaque() {
                return 0;
            }
            if chunk.is_subterranean() {
                return 0;
            }
        } else {
            // Unloaded chunk
            if coord.y >= 3 {
                return (1u64 << 36) - 1;
            } else {
                return 0;
            }
        }
    } else if coord.y <= 0 {
        return 0;
    } else {
        return (1u64 << 36) - 1;
    }

    0
}

/// High-performance chunk graph occlusion culling system (Sodium architecture).
pub fn update_cave_culling_system(
    camera_query: Query<&Transform, With<PlayerCamera>>,
    world: Option<Res<VoxelWorld>>,
    registry: Res<ChunkMeshRegistry>,
    mut culling_state: ResMut<CaveCullingState>,
    mut chunk_meshes: Query<(&ChunkCoordinate, &mut Visibility)>,
) {
    let Some(cam_tf) = camera_query.iter().next() else {
        return;
    };

    let cam_pos = cam_tf.translation;
    let cam_chunk = IVec3::new(
        (cam_pos.x / CHUNK_SIZE as f32).floor() as i32,
        (cam_pos.y / CHUNK_SIZE as f32).floor() as i32,
        (cam_pos.z / CHUNK_SIZE as f32).floor() as i32,
    );

    let cam_block = IVec3::new(
        (cam_pos.x / VOXEL_SIZE).floor() as i32,
        (cam_pos.y / VOXEL_SIZE).floor() as i32,
        (cam_pos.z / VOXEL_SIZE).floor() as i32,
    );

    if cam_block == culling_state.last_player_block && !registry.is_changed() {
        return;
    }
    culling_state.last_player_block = cam_block;
    culling_state.last_player_chunk = cam_chunk;

    let is_in_solid_rock = world
        .as_ref()
        .and_then(|w| w.get_voxel(cam_block))
        .is_some_and(|v| v.is_solid_opaque());

    let mut visible_chunks = HashSet::with_capacity(512);

    if is_in_solid_rock {
        // In spectator mode clipping inside solid rock, strictly keep immediate 3x3x3 chunks visible.
        // All distant caves across the world are culled!
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    visible_chunks.insert(cam_chunk + IVec3::new(dx, dy, dz));
                }
            }
        }
    } else {
        let cam_mask = get_chunk_permeability(cam_chunk, &registry, world.as_deref());
        let reachable_faces = get_camera_reachable_faces(cam_pos, cam_chunk, cam_mask, world.as_deref());

        visible_chunks.insert(cam_chunk);
        let mut queue = VecDeque::with_capacity(256);

        // Queue only neighbor chunks in directions the camera can actually see through open air
        for out_face in 0..6 {
            if (reachable_faces & (1 << out_face)) != 0 {
                let neighbor = cam_chunk + FACE_OFFSETS[out_face];
                let in_face = OPPOSITE_FACES[out_face];
                if visible_chunks.insert(neighbor) {
                    queue.push_back((neighbor, in_face));
                }
            }
        }

        while let Some((curr, in_face)) = queue.pop_front() {
            let mask = get_chunk_permeability(curr, &registry, world.as_deref());
            if mask == 0 {
                continue;
            }

            for out_face in 0..6 {
                if (mask & (1u64 << (in_face * 6 + out_face))) == 0 {
                    continue;
                }

                let neighbor = curr + FACE_OFFSETS[out_face];
                if (neighbor.x - cam_chunk.x).abs() > MAX_HORIZONTAL_RADIUS
                    || (neighbor.z - cam_chunk.z).abs() > MAX_HORIZONTAL_RADIUS
                    || (neighbor.y - cam_chunk.y).abs() > MAX_VERTICAL_RADIUS
                {
                    continue;
                }

                let in_face_of_neighbor = OPPOSITE_FACES[out_face];
                if visible_chunks.insert(neighbor) {
                    queue.push_back((neighbor, in_face_of_neighbor));
                }
            }
        }
    }

    let mut rendered_count = 0;
    let mut culled_count = 0;

    for (coord, mut visibility) in &mut chunk_meshes {
        let target = if visible_chunks.contains(&coord.0) {
            rendered_count += 1;
            Visibility::Inherited
        } else {
            culled_count += 1;
            Visibility::Hidden
        };

        if *visibility != target {
            *visibility = target;
        }
    }

    culling_state.rendered_chunks = rendered_count;
    culling_state.culled_chunks = culled_count;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::TerrainGenerator;

    #[test]
    fn test_chunk_subterranean_flag() {
        let generator = TerrainGenerator::default();
        let deep_chunk = generator.generate_chunk(IVec3::new(0, -4, 0));
        assert!(deep_chunk.is_subterranean(), "Chunk at Y=-4 should be subterranean");

        let bedrock = generator.generate_chunk(IVec3::new(0, -17, 0));
        assert!(bedrock.is_subterranean(), "Bedrock chunk should be subterranean");
    }

    #[test]
    fn test_cave_culling_system_visibility_switching() {
        let mut app = App::new();
        app.init_resource::<CaveCullingState>();
        app.init_resource::<ChunkMeshRegistry>();

        let cam_transform = Transform::from_xyz(8.0, 25.0, 8.0);
        app.world_mut().spawn((
            PlayerCamera::from_transform(&cam_transform),
            cam_transform,
        ));

        // Spawn a surface chunk at (0, 1, 0)
        let surface_chunk = app
            .world_mut()
            .spawn((
                ChunkCoordinate(IVec3::new(0, 1, 0)),
                Visibility::Inherited,
            ))
            .id();

        // Spawn a buried cave chunk at (0, -3, 0)
        let deep_cave = app
            .world_mut()
            .spawn((
                ChunkCoordinate(IVec3::new(0, -3, 0)),
                SubterraneanChunkMesh,
                Visibility::Inherited,
            ))
            .id();

        app.add_systems(Update, update_cave_culling_system);

        // Run traversal: with default empty registry, solid ground blocks -Y
        app.update();

        assert_eq!(
            *app.world().get::<Visibility>(surface_chunk).unwrap(),
            Visibility::Inherited,
            "Surface chunk containing camera must be visible"
        );
        assert_eq!(
            *app.world().get::<Visibility>(deep_cave).unwrap(),
            Visibility::Hidden,
            "Deep cave chunk must be culled"
        );
    }
}
