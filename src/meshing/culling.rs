use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

use crate::{
    meshing::pipeline::ChunkMeshRegistry,
    player::PlayerCamera,
    world::{CHUNK_SIZE, Chunk, VOXEL_SIZE, VoxelWorld, block::BlockShape},
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
    pub is_player_underground: bool,
    pub last_player_chunk: IVec3,
    pub last_player_block: IVec3,
    pub rendered_chunks: usize,
    pub culled_chunks: usize,
    pub rendered_vertices: usize,
    pub culled_vertices: usize,
    pub visible_chunks: HashSet<IVec3>,
    pub previously_visible_chunks: HashSet<IVec3>,
    pub traversal_queue: VecDeque<(IVec3, usize, u8)>,
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
            visible_chunks: HashSet::with_capacity(512),
            previously_visible_chunks: HashSet::with_capacity(512),
            traversal_queue: VecDeque::with_capacity(256),
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

const MAX_HORIZONTAL_RADIUS: i32 = 32;
const MAX_VERTICAL_RADIUS: i32 = 16;

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

        if vx == 0 {
            reachable |= 1 << 0;
        }
        if vx == CHUNK_SIZE - 1 {
            reachable |= 1 << 1;
        }
        if vy == 0 {
            reachable |= 1 << 2;
        }
        if vy == CHUNK_SIZE - 1 {
            reachable |= 1 << 3;
        }
        if vz == 0 {
            reachable |= 1 << 4;
        }
        if vz == CHUNK_SIZE - 1 {
            reachable |= 1 << 5;
        }

        let push_n = |n_idx: usize, visited_bits: &mut [u64; 64], q: &mut Vec<usize>| {
            if (visited_bits[n_idx / 64] & (1u64 << (n_idx % 64))) == 0 {
                visited_bits[n_idx / 64] |= 1u64 << (n_idx % 64);
                q.push(n_idx);
            }
        };

        if vx > 0 {
            push_n(cur - 1, &mut visited, &mut queue);
        }
        if vx + 1 < CHUNK_SIZE {
            push_n(cur + 1, &mut visited, &mut queue);
        }
        if vz > 0 {
            push_n(cur - CHUNK_SIZE, &mut visited, &mut queue);
        }
        if vz + 1 < CHUNK_SIZE {
            push_n(cur + CHUNK_SIZE, &mut visited, &mut queue);
        }
        if vy > 0 {
            push_n(cur - CHUNK_SIZE * CHUNK_SIZE, &mut visited, &mut queue);
        }
        if vy + 1 < CHUNK_SIZE {
            push_n(cur + CHUNK_SIZE * CHUNK_SIZE, &mut visited, &mut queue);
        }
    }

    reachable
}

/// Determines if a 3D world position is underground (beneath solid terrain or inside a subterranean chunk).
pub fn is_position_underground(cam_pos: Vec3, world: Option<&VoxelWorld>) -> bool {
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

    let Some(w) = world else {
        return cam_chunk.y <= 0;
    };

    if let Some(chunk) = w.get_chunk(cam_chunk) {
        if chunk.is_subterranean() {
            return true;
        }
    }

    // Cast a vertical ray upwards through the column to detect solid roof/ceiling
    // Check up to 96 blocks above the camera for solid opaque ground
    let mut solid_count = 0;
    for dy in 1..=96 {
        let check_coord = cam_block + IVec3::new(0, dy, 0);
        if let Some(voxel) = w.get_voxel(check_coord) {
            if voxel.is_solid_opaque() {
                solid_count += 1;
                if solid_count >= 2 {
                    return true;
                }
            }
        }
    }

    false
}

/// Returns the 36-bit face-permeability mask for a chunk coordinate.
pub fn get_chunk_permeability(
    coord: IVec3,
    registry: &ChunkMeshRegistry,
    world: Option<&VoxelWorld>,
    is_player_underground: bool,
) -> u64 {
    let is_subterranean = registry.is_subterranean(&coord)
        || world.is_some_and(|w| w.get_chunk(coord).is_some_and(Chunk::is_subterranean));

    if !is_player_underground && is_subterranean && !registry.contains(&coord) {
        return 0;
    }

    if let Some(mask) = registry.get_visibility_mask_opt(&coord) {
        return mask;
    }

    if let Some(w) = world {
        if let Some(chunk) = w.get_chunk(coord) {
            if !is_player_underground && chunk.is_subterranean() {
                return 0;
            }
            if chunk.is_empty() {
                return (1u64 << 36) - 1;
            }
            if chunk.is_fully_solid_opaque() {
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
    } else if coord.y <= 0 || (!is_player_underground && is_subterranean) {
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
    streaming_settings: Option<Res<crate::world::streaming::ChunkStreamingSettings>>,
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

    let is_player_underground = is_position_underground(cam_pos, world.as_deref());
    culling_state.is_player_underground = is_player_underground;

    let is_in_solid_rock = world
        .as_ref()
        .and_then(|w| w.get_voxel(cam_block))
        .is_some_and(|v| v.is_solid_opaque());

    culling_state.visible_chunks.clear();

    if is_in_solid_rock {
        // In spectator mode clipping inside solid rock, strictly keep immediate 3x3x3 chunks visible.
        // All distant caves across the world are culled!
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    culling_state
                        .visible_chunks
                        .insert(cam_chunk + IVec3::new(dx, dy, dz));
                }
            }
        }
    } else {
        let max_h_radius = streaming_settings
            .as_ref()
            .map(|s| (s.render_distance + 1).min(MAX_HORIZONTAL_RADIUS))
            .unwrap_or(MAX_HORIZONTAL_RADIUS);
        let max_v_radius = streaming_settings
            .as_ref()
            .map(|s| (s.render_distance + 1).min(MAX_VERTICAL_RADIUS))
            .unwrap_or(MAX_VERTICAL_RADIUS);

        let cam_mask =
            get_chunk_permeability(cam_chunk, &registry, world.as_deref(), is_player_underground);
        let reachable_faces =
            get_camera_reachable_faces(cam_pos, cam_chunk, cam_mask, world.as_deref());

        culling_state.visible_chunks.insert(cam_chunk);
        culling_state.traversal_queue.clear();

        for out_face in 0..6 {
            if (reachable_faces & (1 << out_face)) != 0 {
                let neighbor = cam_chunk + FACE_OFFSETS[out_face];
                let in_face = OPPOSITE_FACES[out_face];

                if culling_state.visible_chunks.insert(neighbor) {
                    culling_state
                        .traversal_queue
                        .push_back((neighbor, in_face, 0));
                }
            }
        }

        while let Some((curr, in_face, _)) = culling_state.traversal_queue.pop_front() {
            let mask =
                get_chunk_permeability(curr, &registry, world.as_deref(), is_player_underground);
            if mask == 0 {
                continue;
            }

            for out_face in 0..6 {
                if (mask & (1u64 << (in_face * 6 + out_face))) == 0 {
                    continue;
                }

                let neighbor = curr + FACE_OFFSETS[out_face];
                if (neighbor.x - cam_chunk.x).abs() > max_h_radius
                    || (neighbor.z - cam_chunk.z).abs() > max_h_radius
                    || (neighbor.y - cam_chunk.y).abs() > max_v_radius
                {
                    continue;
                }

                let in_face_of_neighbor = OPPOSITE_FACES[out_face];

                if culling_state.visible_chunks.insert(neighbor) {
                    culling_state
                        .traversal_queue
                        .push_back((neighbor, in_face_of_neighbor, 0));
                }
            }
        }
    }

    // Direct indexed entity visibility updates (avoiding full iteration across all entities)
    let chunks_to_hide: Vec<IVec3> = culling_state
        .previously_visible_chunks
        .difference(&culling_state.visible_chunks)
        .copied()
        .collect();

    let chunks_to_show: Vec<IVec3> = culling_state
        .visible_chunks
        .difference(&culling_state.previously_visible_chunks)
        .copied()
        .collect();

    let apply_chunk_visibility = |coord: &IVec3,
                                  is_visible_in_culling: bool,
                                  reg: &ChunkMeshRegistry,
                                  meshes_query: &mut Query<(&ChunkCoordinate, &mut Visibility)>| {
        if let Some(render_data) = reg.get_render_data(coord) {
            let target = if render_data.is_subterranean {
                if is_visible_in_culling {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                }
            } else {
                // Surface terrain is always visible to open air unless deep underground with no opening
                if !is_player_underground || is_visible_in_culling {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                }
            };

            // Procedural grass foliage distance culling: only render expensive blade quads within 5 chunks (80m)
            let is_near_grass = (coord.x - cam_chunk.x).abs() <= 5 && (coord.z - cam_chunk.z).abs() <= 5;
            let grass_target = if target == Visibility::Inherited && is_near_grass {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };

            let mut apply = |part_opt: &Option<crate::meshing::pipeline::ChunkRenderPart>, vis: Visibility| {
                if let Some(part) = part_opt {
                    if let Ok((_, mut cur_vis)) = meshes_query.get_mut(part.entity) {
                        if *cur_vis != vis {
                            *cur_vis = vis;
                        }
                    }
                }
            };
            apply(&render_data.opaque, target);
            apply(&render_data.transparent, target);
            apply(&render_data.grass, grass_target);
        }
    };

    let moved_chunk = cam_chunk != culling_state.last_player_chunk;

    if culling_state.previously_visible_chunks.is_empty() {
        for coord in &culling_state.visible_chunks {
            apply_chunk_visibility(coord, true, &registry, &mut chunk_meshes);
        }
    } else {
        for coord in &chunks_to_hide {
            apply_chunk_visibility(coord, false, &registry, &mut chunk_meshes);
        }
        for coord in &chunks_to_show {
            apply_chunk_visibility(coord, true, &registry, &mut chunk_meshes);
        }
        if moved_chunk || registry.is_changed() {
            for coord in &culling_state.visible_chunks {
                apply_chunk_visibility(coord, true, &registry, &mut chunk_meshes);
            }
        }
    }

    culling_state.previously_visible_chunks = culling_state.visible_chunks.clone();

    // Fast O(visible_chunks) telemetry metrics calculation
    let mut rendered_count = 0;
    let mut rendered_verts = 0;
    for coord in &culling_state.visible_chunks {
        if let Some(render_data) = registry.get_render_data(coord) {
            rendered_count += 1;
            let is_near_grass = (coord.x - cam_chunk.x).abs() <= 5 && (coord.z - cam_chunk.z).abs() <= 5;
            let mut v = 0;
            if let Some(ref p) = render_data.opaque { v += p.vertex_count; }
            if let Some(ref p) = render_data.transparent { v += p.vertex_count; }
            if is_near_grass && let Some(ref p) = render_data.grass { v += p.vertex_count; }
            rendered_verts += v;
        }
    }

    culling_state.rendered_chunks = rendered_count;
    culling_state.culled_chunks = registry.len().saturating_sub(rendered_count);
    culling_state.rendered_vertices = rendered_verts;
    culling_state.culled_vertices = registry.total_vertices().saturating_sub(rendered_verts);
}
