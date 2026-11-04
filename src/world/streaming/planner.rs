use std::collections::{HashMap, HashSet};
use bevy::prelude::*;

use crate::{
    generation::TerrainGenerator,
    meshing::{ChunkLod, ChunkMeshRegistry, LodMeshRegistry},
    player::Player,
    world::{
        CHUNK_SIZE, VOXEL_SIZE, VoxelWorld,
        streaming::{
            ChunkStreamingQueues,
            manager::{
                ChunkGenerationTask, ChunkStreamingSettings, ChunkStreamingState, LodMeshingTask,
                WORLD_MAX_CHUNK_Y, WORLD_MIN_CHUNK_Y,
            },
        },
    },
};

pub fn plan_chunk_streaming(
    player: Single<&Transform, With<Player>>,
    camera: Option<Single<&Transform, (With<Camera3d>, With<crate::player::PlayerCamera>)>>,
    settings: Res<ChunkStreamingSettings>,
    terrain_generator: Res<TerrainGenerator>,
    mut state: ResMut<ChunkStreamingState>,
    world: Res<VoxelWorld>,
    mesh_registry: Res<ChunkMeshRegistry>,
    generation_tasks: Query<&ChunkGenerationTask>,
    lod_tasks: Query<&LodMeshingTask>,
    lod_registry: Res<LodMeshRegistry>,
    mut queues: ResMut<ChunkStreamingQueues>,
) {
    let player_chunk = player_chunk_coordinate(player.translation);
    let current_fwd = camera
        .as_ref()
        .map(|c| {
            let f = c.forward();
            Vec2::new(f.x, f.z).normalize_or_zero()
        })
        .unwrap_or(Vec2::ZERO);

    let moved_chunk = state.last_player_chunk != Some(player_chunk);
    let camera_turned = state.last_camera_fwd.is_some_and(|last| {
        current_fwd != Vec2::ZERO && current_fwd.dot(last) < 0.95 // ~18 degrees turn
    });

    if !settings.is_changed()
        && !moved_chunk
        && !mesh_registry.is_changed()
        && (!camera_turned || (queues.load.is_empty() && queues.lod_load.is_empty()))
    {
        return;
    }

    state.last_player_chunk = Some(player_chunk);
    state.last_camera_fwd = Some(current_fwd);

    state.desired_chunks =
        desired_chunk_coordinates(player_chunk, settings.render_distance, &terrain_generator);

    let generating_chunks: HashSet<IVec3> = generation_tasks
        .iter()
        .map(|task| task.coordinate)
        .collect();

    let mut chunks_to_load: Vec<IVec3> = state
        .desired_chunks
        .iter()
        .filter(|coordinate| {
            !world.contains_chunk(**coordinate) && !generating_chunks.contains(coordinate)
        })
        .copied()
        .collect();

    let mut chunks_to_unload: Vec<IVec3> = world
        .iter_chunks()
        .map(|(&coordinate, _)| coordinate)
        .filter(|coord| !state.desired_chunks.contains(coord))
        .collect();

    // View-cone forward weighting: chunks in player's forward view cone get up to 2.5x priority
    chunks_to_load.sort_by_key(|coordinate| {
        let delta = *coordinate - player_chunk;
        let dist_sq = (delta.x * delta.x + delta.y * delta.y + delta.z * delta.z) as f32;
        let weight = if dist_sq <= 2.25 || current_fwd == Vec2::ZERO {
            1.0
        } else {
            let dir = Vec2::new(delta.x as f32, delta.z as f32).normalize_or_zero();
            let dot = dir.dot(current_fwd);
            (1.0 - 0.6 * dot).max(0.2)
        };
        (dist_sq * weight * 1000.0) as i64
    });

    chunks_to_unload.sort_by_key(|coordinate| {
        std::cmp::Reverse(chunk_distance_squared(*coordinate, player_chunk))
    });

    queues.load.clear();
    queues.load.extend(chunks_to_load);

    queues.unload.clear();
    queues.unload.extend(chunks_to_unload);

    // Dynamic Horizon LOD Column Planning
    let mut desired_lod: HashMap<IVec2, ChunkLod> = HashMap::default();
    let center_2d = IVec2::new(player_chunk.x, player_chunk.z);

    if settings.lod_render_distance > 0 {
        let total_r = settings.render_distance + settings.lod_render_distance;
        let r_real = settings.render_distance as f32;
        let r_mid = r_real + settings.lod_render_distance as f32 * 0.5;

        let r_mid_sq = r_mid * r_mid;
        let r_real_sq = r_real * r_real;
        let total_r_sq = (total_r * total_r) as f32;

        let mid_r = r_mid.ceil() as i32;
        for dz in -mid_r..=mid_r {
            for dx in -mid_r..=mid_r {
                let dist_sq = (dx * dx + dz * dz) as f32;
                if dist_sq > r_mid_sq {
                    continue;
                }

                let col = center_2d + IVec2::new(dx, dz);
                if dist_sq <= r_real_sq {
                    if !mesh_registry.has_column_surface_mesh(col) {
                        desired_lod.insert(col, ChunkLod::Lod1);
                    }
                } else {
                    desired_lod.insert(col, ChunkLod::Lod1);
                }
            }
        }

        let min_super_x = (center_2d.x - total_r) & !1;
        let max_super_x = (center_2d.x + total_r) & !1;
        let min_super_z = (center_2d.y - total_r) & !1;
        let max_super_z = (center_2d.y + total_r) & !1;

        let mut sz = min_super_z;
        while sz <= max_super_z {
            let mut sx = min_super_x;
            while sx <= max_super_x {
                let super_coord = IVec2::new(sx, sz);

                let mut overlaps_near = false;
                let mut in_range_count = 0;

                for ox in 0..2 {
                    for oz in 0..2 {
                        let sub_col = super_coord + IVec2::new(ox, oz);
                        let delta = sub_col - center_2d;
                        let dist_sq = (delta.x * delta.x + delta.y * delta.y) as f32;

                        if desired_lod.contains_key(&sub_col) || dist_sq <= r_mid_sq {
                            overlaps_near = true;
                        }
                        if dist_sq <= total_r_sq {
                            in_range_count += 1;
                        }
                    }
                }

                if !overlaps_near && in_range_count > 0 {
                    desired_lod.insert(super_coord, ChunkLod::Lod2);
                } else if overlaps_near {
                    for ox in 0..2 {
                        for oz in 0..2 {
                            let sub_col = super_coord + IVec2::new(ox, oz);
                            let delta = sub_col - center_2d;
                            let dist_sq = (delta.x * delta.x + delta.y * delta.y) as f32;
                            if dist_sq > r_mid_sq
                                && dist_sq <= total_r_sq
                                && !desired_lod.contains_key(&sub_col)
                            {
                                desired_lod.insert(sub_col, ChunkLod::Lod1);
                            }
                        }
                    }
                }

                sx += 2;
            }
            sz += 2;
        }
    }

    // LOD columns to unload
    let mut lod_to_unload: Vec<IVec2> = Vec::new();
    for coord in lod_registry.iter_coordinates() {
        let current_lod = lod_registry.get_lod(coord);
        let desired = desired_lod.get(coord).copied();
        if desired != current_lod {
            lod_to_unload.push(*coord);
        }
    }

    queues.lod_unload.clear();
    queues.lod_unload.extend(lod_to_unload);

    // LOD columns to load
    let active_lod: HashMap<IVec2, ChunkLod> = lod_tasks
        .iter()
        .map(|task| (task.coordinate, task.lod))
        .collect();

    let mut lod_to_load: Vec<(IVec2, ChunkLod)> = desired_lod
        .iter()
        .filter(|(coord, lod)| {
            lod_registry.get_lod(coord) != Some(**lod) && active_lod.get(*coord) != Some(lod)
        })
        .map(|(&coord, &lod)| (coord, lod))
        .collect();

    lod_to_load.sort_by_key(|(coord, lod)| {
        let offset = if *lod == ChunkLod::Lod2 {
            Vec2::new(0.5, 0.5)
        } else {
            Vec2::ZERO
        };
        let col_f = Vec2::new(coord.x as f32 + offset.x, coord.y as f32 + offset.y);
        let center_f = Vec2::new(center_2d.x as f32, center_2d.y as f32);
        let delta = col_f - center_f;
        let dist = delta.length();
        let weight = if dist <= 1.5 || current_fwd == Vec2::ZERO {
            1.0
        } else {
            let dir = delta.normalize_or_zero();
            let dot = dir.dot(current_fwd);
            (1.0 - 0.6 * dot).max(0.2)
        };
        (dist * weight * 1000.0) as i64
    });

    queues.lod_load.clear();
    queues.lod_load.extend(lod_to_load);

    state.desired_lod_columns = desired_lod;
}

pub fn player_chunk_coordinate(player_position: Vec3) -> IVec3 {
    let player_voxel = IVec3::new(
        (player_position.x / VOXEL_SIZE).floor() as i32,
        (player_position.y / VOXEL_SIZE).floor() as i32,
        (player_position.z / VOXEL_SIZE).floor() as i32,
    );

    let (chunk_coordinate, _) = VoxelWorld::world_voxel_to_chunk(player_voxel);
    chunk_coordinate
}

pub fn desired_chunk_coordinates(
    center: IVec3,
    render_distance: i32,
    generator: &TerrainGenerator,
) -> HashSet<IVec3> {
    let radius_squared = render_distance * render_distance;

    let diameter = (render_distance * 2 + 1) as usize;
    let est_cols = diameter * diameter;
    let est_chunks = (est_cols * 6).max(32);
    let mut chunks = HashSet::with_capacity(est_chunks);

    let max_y = (center.y + render_distance).min(WORLD_MAX_CHUNK_Y);

    for z in -render_distance..=render_distance {
        for x in -render_distance..=render_distance {
            let horizontal_dist_sq = x * x + z * z;
            if horizontal_dist_sq > radius_squared {
                continue;
            }

            let col_chunk_x = center.x + x;
            let col_chunk_z = center.z + z;
            let est_max_height = generator.estimate_chunk_max_height(col_chunk_x, col_chunk_z);
            let est_max_chunk_y = est_max_height.div_euclid(CHUNK_SIZE as i32);

            let est_min_height =
                generator.estimate_chunk_surface_min_height(col_chunk_x, col_chunk_z);
            let est_min_chunk_y = est_min_height.div_euclid(CHUNK_SIZE as i32);

            let (col_min_y, col_max_y) = if horizontal_dist_sq <= 16 {
                // Near player: load deeper for caves & ravines the player can enter
                let min_y = (center.y - 4).min(est_min_chunk_y - 1).max(WORLD_MIN_CHUNK_Y);
                let max_y_col = (est_max_chunk_y + 1).max(center.y + 2).min(max_y);
                (min_y, max_y_col)
            } else {
                // Distant columns: only load the visible surface terrain envelope
                let min_y = (est_min_chunk_y - 1).max(WORLD_MIN_CHUNK_Y);
                let max_y_col = (est_max_chunk_y + 1).min(max_y);
                (min_y, max_y_col)
            };

            for y in col_min_y..=col_max_y {
                chunks.insert(IVec3::new(col_chunk_x, y, col_chunk_z));
            }
        }
    }

    chunks
}

pub fn chunk_distance_squared(a: IVec3, b: IVec3) -> i32 {
    let delta = a - b;
    delta.x * delta.x + delta.y * delta.y + delta.z * delta.z
}
