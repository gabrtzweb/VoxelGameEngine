use std::collections::{HashMap, HashSet};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};

use crate::{
    generation::{BiomeType, CaveGenerator, ClimateGenerator, StrataGenerator, TerrainGenerator},
    meshing::{
        ChunkLod, ChunkMaterial, ChunkMeshRegistry, ChunkMeshingTask, LodMeshRegistry,
        apply_lod_mesh, build_lod_mesh, remove_chunk_render, remove_lod_render,
    },
    player::{Player, PlayerSet},
    simulation::lighting::{VoxelLightRegistry, remove_chunk_lights, sync_chunk_lights},
    world::{
        VOXEL_SIZE, chunk::Chunk, modifications::WorldModificationStore, storage::VoxelWorld,
        streaming::queues::ChunkStreamingQueues,
    },
};

const DEFAULT_RENDER_DISTANCE: i32 = 8;
const DEFAULT_LOD_RENDER_DISTANCE: i32 = 8;
pub const DEFAULT_SIMULATION_DISTANCE: i32 = 8;

pub const WORLD_MIN_CHUNK_Y: i32 = -16;
pub const WORLD_MAX_CHUNK_Y: i32 = 16;

const MAX_GENERATION_TASKS_IN_FLIGHT: usize = 64;
const MAX_GENERATION_TASKS_STARTED_PER_FRAME: usize = 32;

const MAX_LOD_TASKS_IN_FLIGHT: usize = 32;
const MAX_LOD_TASKS_STARTED_PER_FRAME: usize = 16;

const MAX_CHUNK_UNLOADS_PER_FRAME: usize = 128;
const MAX_LOD_UNLOADS_PER_FRAME: usize = 64;

pub const NEIGHBOR_DIRECTIONS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

#[derive(Resource, Clone, Reflect)]
pub struct ChunkStreamingSettings {
    pub render_distance: i32,
    pub lod_render_distance: i32,
    pub simulation_distance: i32,
}

impl Default for ChunkStreamingSettings {
    fn default() -> Self {
        Self {
            render_distance: DEFAULT_RENDER_DISTANCE,
            lod_render_distance: DEFAULT_LOD_RENDER_DISTANCE,
            simulation_distance: DEFAULT_SIMULATION_DISTANCE,
        }
    }
}

#[derive(Resource, Default)]
pub struct ChunkStreamingState {
    pub last_player_chunk: Option<IVec3>,
    pub last_camera_fwd: Option<Vec2>,
    pub desired_chunks: HashSet<IVec3>,
    pub desired_lod_columns: HashMap<IVec2, ChunkLod>,
}

#[derive(Component)]
pub struct ChunkGenerationTask {
    pub coordinate: IVec3,
    pub task: Task<GeneratedChunk>,
}

pub struct GeneratedChunk {
    pub coordinate: IVec3,
    pub chunk: Chunk,
}

#[derive(Component)]
pub struct LodMeshingTask {
    pub coordinate: IVec2,
    pub lod: ChunkLod,
    pub task: Task<CompletedLodMesh>,
}

pub struct CompletedLodMesh {
    pub coordinate: IVec2,
    pub lod: ChunkLod,
    pub meshes: crate::meshing::greedy::ChunkMeshes,
    pub map_chunks: Vec<(IVec2, crate::map::MapChunk)>,
}

pub struct ChunkStreamingPlugin;

impl Plugin for ChunkStreamingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .init_resource::<ChunkStreamingSettings>()
            .init_resource::<ChunkStreamingState>()
            .init_resource::<ChunkStreamingQueues>()
            .init_resource::<WorldModificationStore>()
            .init_resource::<VoxelLightRegistry>()
            .insert_resource(TerrainGenerator::default())
            .register_type::<ChunkStreamingSettings>()
            .register_type::<TerrainGenerator>()
            .register_type::<ClimateGenerator>()
            .register_type::<CaveGenerator>()
            .register_type::<StrataGenerator>()
            .register_type::<BiomeType>()
            .add_systems(
                Update,
                (
                    handle_terrain_generator_reload,
                    plan_chunk_streaming,
                    process_chunk_unloads,
                    process_lod_unloads,
                    start_generation_tasks,
                    collect_generation_tasks,
                    start_lod_meshing_tasks,
                    collect_lod_meshing_tasks,
                )
                    .chain()
                    .after(PlayerSet::Movement),
            );
    }
}

pub fn handle_terrain_generator_reload(
    generator: Res<TerrainGenerator>,
    mut last_version: Local<u32>,
    mut state: ResMut<ChunkStreamingState>,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut mesh_registry: ResMut<ChunkMeshRegistry>,
    mut lod_registry: ResMut<LodMeshRegistry>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    generation_tasks: Query<(Entity, &ChunkGenerationTask)>,
    meshing_tasks: Query<(Entity, &ChunkMeshingTask)>,
    lod_tasks: Query<(Entity, &LodMeshingTask)>,
    mut commands: Commands,
    mut queues: ResMut<ChunkStreamingQueues>,
    mut map_cache: Option<ResMut<crate::map::MapCache>>,
) {
    if generator.version == *last_version {
        return;
    }

    *last_version = generator.version;

    // 1. Despawn all in-flight generation tasks
    for (entity, _) in &generation_tasks {
        if let Ok(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.try_despawn();
        }
    }

    // 2. Despawn all in-flight meshing tasks
    for (entity, _) in &meshing_tasks {
        if let Ok(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.try_despawn();
        }
    }

    // 2b. Despawn all in-flight LOD meshing tasks
    for (entity, _) in &lod_tasks {
        if let Ok(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.try_despawn();
        }
    }

    // 3. Despawn and remove all existing chunk meshes
    let mesh_coords: Vec<IVec3> = mesh_registry.iter_coordinates().copied().collect();
    for coordinate in mesh_coords {
        remove_chunk_render(&mut commands, coordinate, &mut mesh_registry, &mut meshes);
    }

    // 3b. Despawn and remove all existing LOD meshes
    let lod_coords: Vec<IVec2> = lod_registry.iter_coordinates().copied().collect();
    for coordinate in lod_coords {
        remove_lod_render(&mut commands, coordinate, &mut lod_registry, &mut meshes);
    }

    // 4. Remove all chunk point lights
    let light_coords: Vec<IVec3> = world.iter_chunks().map(|(&c, _)| c).collect();
    for coordinate in light_coords {
        remove_chunk_lights(&mut commands, coordinate, &mut light_registry);
    }

    // 5. Purge world storage and recorded modifications
    world.clear();
    modifications.clear();
    if let Some(ref mut cache) = map_cache {
        cache.clear();
    }

    // 6. Clear streaming queues
    queues.load.clear();
    queues.unload.clear();
    queues.remesh.clear();
    queues.remesh_set.clear();
    queues.lod_load.clear();
    queues.lod_unload.clear();

    // 7. Reset player tracking so plan_chunk_streaming immediately queues full reload around player
    state.last_player_chunk = None;
    state.last_camera_fwd = None;
    state.desired_chunks.clear();
    state.desired_lod_columns.clear();
}

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

    let desired_chunks =
        desired_chunk_coordinates(player_chunk, settings.render_distance, &terrain_generator);

    state.desired_chunks = desired_chunks.clone();

    let generating_chunks: HashSet<IVec3> = generation_tasks
        .iter()
        .map(|task| task.coordinate)
        .collect();

    let mut chunks_to_load: Vec<IVec3> = desired_chunks
        .iter()
        .filter(|coordinate| {
            !world.contains_chunk(**coordinate) && !generating_chunks.contains(coordinate)
        })
        .copied()
        .collect();

    let mut chunks_to_unload: Vec<IVec3> = world
        .iter_chunks()
        .map(|(&coordinate, _)| coordinate)
        .filter(|coord| !desired_chunks.contains(coord))
        .collect();

    // View-cone forward weighting: chunks in player's forward view cone get up to 2.5x priority
    chunks_to_load.sort_by_key(|coordinate| {
        let delta = *coordinate - player_chunk;
        let dist = ((delta.x * delta.x + delta.y * delta.y + delta.z * delta.z) as f32).sqrt();
        let weight = if dist <= 1.5 || current_fwd == Vec2::ZERO {
            1.0
        } else {
            let dir = Vec2::new(delta.x as f32, delta.z as f32).normalize_or_zero();
            let dot = dir.dot(current_fwd);
            (1.0 - 0.6 * dot).max(0.2)
        };
        (dist * weight * 1000.0) as i64
    });

    chunks_to_unload.sort_by_key(|coordinate| {
        std::cmp::Reverse(chunk_distance_squared(*coordinate, player_chunk))
    });

    queues.load.clear();
    queues.unload.clear();

    queues.load.extend(chunks_to_load);

    queues.unload.extend(chunks_to_unload);

    // LOD Column Streaming Planning
    let center_2d = IVec2::new(player_chunk.x, player_chunk.z);
    let mut desired_lod = HashMap::new();

    if settings.lod_render_distance > 0 {
        let total_r = settings.render_distance + settings.lod_render_distance;
        let r_real = settings.render_distance as f32;
        let r_mid = r_real + settings.lod_render_distance as f32 * 0.5;

        // Step 1: For near and mid distances (up to r_mid), use Lod1 (1x1 chunk columns)
        // Inside real render distance: provide an instant Lod1 far-mesh placeholder
        // if real voxel chunk meshes haven't arrived yet, completely eliminating the void gap!
        let mid_r = r_mid.ceil() as i32;
        for dz in -mid_r..=mid_r {
            for dx in -mid_r..=mid_r {
                let dist = ((dx * dx + dz * dz) as f32).sqrt();
                if dist > r_mid {
                    continue;
                }

                let col = center_2d + IVec2::new(dx, dz);
                if dist <= r_real {
                    if !mesh_registry.has_column_mesh(col) {
                        desired_lod.insert(col, ChunkLod::Lod1);
                    }
                } else {
                    desired_lod.insert(col, ChunkLod::Lod1);
                }
            }
        }

        // Step 2: For far distance (beyond r_mid up to total_r), use Lod2 (2x2 super-chunks)
        // Batch distant columns into 2x2 super-chunks to slash draw calls by ~75%.
        let min_super_x = (center_2d.x - total_r).div_euclid(2) * 2;
        let max_super_x = (center_2d.x + total_r).div_euclid(2) * 2;
        let min_super_z = (center_2d.y - total_r).div_euclid(2) * 2;
        let max_super_z = (center_2d.y + total_r).div_euclid(2) * 2;

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
                        let dist = ((delta.x * delta.x + delta.y * delta.y) as f32).sqrt();

                        if desired_lod.contains_key(&sub_col) || dist <= r_mid {
                            overlaps_near = true;
                        }
                        if dist <= total_r as f32 {
                            in_range_count += 1;
                        }
                    }
                }

                if !overlaps_near && in_range_count > 0 {
                    // All 4 sub-columns are safely in the far zone!
                    // Batch them into a single 2x2 Lod2 super-chunk
                    desired_lod.insert(super_coord, ChunkLod::Lod2);
                } else if overlaps_near {
                    // If any sub-column was in near/mid zone, any OTHER sub-column that is beyond r_mid
                    // but within total_r should still be covered by Lod1 so there's no void gap!
                    for ox in 0..2 {
                        for oz in 0..2 {
                            let sub_col = super_coord + IVec2::new(ox, oz);
                            let delta = sub_col - center_2d;
                            let dist = ((delta.x * delta.x + delta.y * delta.y) as f32).sqrt();
                            if dist > r_mid
                                && dist <= total_r as f32
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

    state.desired_lod_columns = desired_lod.clone();

    // LOD columns to unload:
    // Only unload if no longer desired (e.g. out of range, or real meshes arrived, or tier changed)
    let mut lod_to_unload: Vec<IVec2> = lod_registry
        .iter_coordinates()
        .filter(|coord| !desired_lod.contains_key(coord))
        .copied()
        .collect();

    lod_to_unload.sort_by_key(|coord| {
        let delta = *coord - center_2d;
        std::cmp::Reverse(delta.x * delta.x + delta.y * delta.y)
    });

    queues.lod_unload.clear();
    queues.lod_unload.extend(lod_to_unload);

    // LOD columns to load
    let active_lod: HashMap<IVec2, ChunkLod> = lod_tasks
        .iter()
        .map(|task| (task.coordinate, task.lod))
        .collect();

    let mut lod_to_load: Vec<(IVec2, ChunkLod)> = desired_lod
        .into_iter()
        .filter(|(coord, lod)| {
            lod_registry.get_lod(coord) != Some(*lod) && active_lod.get(coord) != Some(lod)
        })
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
}

pub fn process_chunk_unloads(
    mut commands: Commands,
    mut queues: ResMut<ChunkStreamingQueues>,
    mut world: ResMut<VoxelWorld>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut registry: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for _ in 0..MAX_CHUNK_UNLOADS_PER_FRAME {
        let Some(coordinate) = queues.unload.pop_front() else {
            break;
        };

        if world.get_chunk(coordinate).is_none() {
            continue;
        }

        remove_chunk_lights(&mut commands, coordinate, &mut light_registry);

        if world.remove_chunk(coordinate).is_none() {
            continue;
        }

        remove_chunk_render(&mut commands, coordinate, &mut registry, &mut meshes);
        queues.remesh_set.remove(&coordinate);

        for neighbor in neighbors(coordinate) {
            if world.get_chunk(neighbor).is_some() {
                queues.enqueue_remesh(neighbor);
            }
        }
    }
}

pub fn process_lod_unloads(
    mut commands: Commands,
    mut queues: ResMut<ChunkStreamingQueues>,
    mut lod_registry: ResMut<LodMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for _ in 0..MAX_LOD_UNLOADS_PER_FRAME {
        let Some(coordinate) = queues.lod_unload.pop_front() else {
            break;
        };

        remove_lod_render(&mut commands, coordinate, &mut lod_registry, &mut meshes);
    }
}

pub fn start_generation_tasks(
    mut commands: Commands,
    terrain_generator: Res<TerrainGenerator>,
    active_tasks: Query<&ChunkGenerationTask>,
    mut queues: ResMut<ChunkStreamingQueues>,
) {
    let active_count = active_tasks.iter().count();

    if active_count >= MAX_GENERATION_TASKS_IN_FLIGHT {
        return;
    }

    let available_slots = MAX_GENERATION_TASKS_IN_FLIGHT - active_count;

    let tasks_to_start = available_slots.min(MAX_GENERATION_TASKS_STARTED_PER_FRAME);

    let pool = AsyncComputeTaskPool::get();

    for _ in 0..tasks_to_start {
        let Some(coordinate) = queues.load.pop_front() else {
            break;
        };

        let generator = terrain_generator.clone();

        let task = pool.spawn(async move {
            let chunk = generator.generate_chunk(coordinate);

            GeneratedChunk { coordinate, chunk }
        });

        commands.spawn(ChunkGenerationTask { coordinate, task });
    }
}

pub fn collect_generation_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkGenerationTask)>,
    state: Res<ChunkStreamingState>,
    modifications: Res<WorldModificationStore>,
    mut world: ResMut<VoxelWorld>,
    mut queues: ResMut<ChunkStreamingQueues>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut map_cache: Option<ResMut<crate::map::MapCache>>,
) {
    for (entity, mut generation_task) in &mut tasks {
        let Some(mut generated) = check_ready(&mut generation_task.task) else {
            continue;
        };

        commands.entity(entity).despawn();

        if !state.desired_chunks.contains(&generated.coordinate) {
            continue;
        }

        modifications.apply_to_chunk(generated.coordinate, &mut generated.chunk);

        world.insert_chunk(generated.coordinate, generated.chunk);

        if let Some(ref mut cache) = map_cache
            && generated.coordinate.y >= -8
            && generated.coordinate.y <= 14
        {
            cache.mark_dirty(IVec2::new(generated.coordinate.x, generated.coordinate.z));
        }

        sync_chunk_lights(
            &mut commands,
            &world,
            generated.coordinate,
            &mut light_registry,
        );

        queues.enqueue_remesh(generated.coordinate);

        for neighbor in neighbors(generated.coordinate) {
            if world.get_chunk(neighbor).is_some() {
                queues.enqueue_remesh(neighbor);
            }
        }
    }
}

pub fn start_lod_meshing_tasks(
    mut commands: Commands,
    terrain_generator: Res<TerrainGenerator>,
    material: Option<Res<ChunkMaterial>>,
    active_tasks: Query<&LodMeshingTask>,
    mut queues: ResMut<ChunkStreamingQueues>,
) {
    let Some(material) = material else {
        return;
    };

    let active_count = active_tasks.iter().count();
    if active_count >= MAX_LOD_TASKS_IN_FLIGHT {
        return;
    }

    let available_slots = MAX_LOD_TASKS_IN_FLIGHT - active_count;
    let tasks_to_start = available_slots.min(MAX_LOD_TASKS_STARTED_PER_FRAME);
    let pool = AsyncComputeTaskPool::get();

    for _ in 0..tasks_to_start {
        let Some((coordinate, lod)) = queues.lod_load.pop_front() else {
            break;
        };

        let generator = terrain_generator.clone();
        let textures = material.texture_registry.clone();

        let task = pool.spawn(async move {
            let meshes = build_lod_mesh(coordinate, lod, &generator, &textures);
            let mut map_chunks = Vec::new();
            for dx in 0..lod.chunk_extent() {
                for dz in 0..lod.chunk_extent() {
                    let sub_coord = coordinate + IVec2::new(dx, dz);
                    let map_chunk =
                        crate::map::cache::generate_lod_map_chunk(sub_coord, &generator);
                    map_chunks.push((sub_coord, map_chunk));
                }
            }
            CompletedLodMesh {
                coordinate,
                lod,
                meshes,
                map_chunks,
            }
        });

        commands.spawn(LodMeshingTask {
            coordinate,
            lod,
            task,
        });
    }
}

const MAX_LOD_MESH_UPLOADS_PER_FRAME: usize = 16;

pub fn collect_lod_meshing_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut LodMeshingTask)>,
    state: Res<ChunkStreamingState>,
    material: Option<Res<ChunkMaterial>>,
    mut lod_registry: ResMut<LodMeshRegistry>,
    mut map_cache: Option<ResMut<crate::map::MapCache>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(material) = material else {
        return;
    };

    let mut uploaded = 0;
    for (entity, mut lod_task) in &mut tasks {
        if uploaded >= MAX_LOD_MESH_UPLOADS_PER_FRAME {
            break;
        }

        let Some(completed) = check_ready(&mut lod_task.task) else {
            continue;
        };

        uploaded += 1;
        commands.entity(entity).despawn();

        // Ensure this LOD column is still desired at this tier
        if state.desired_lod_columns.get(&completed.coordinate) != Some(&completed.lod) {
            continue;
        }

        apply_lod_mesh(
            &mut commands,
            completed.coordinate,
            completed.lod,
            completed.meshes,
            &mut lod_registry,
            &mut meshes,
            &material,
        );

        // Update world map and minimap with newly generated LOD surface terrain & trees
        if let Some(ref mut cache) = map_cache {
            for (sub_coord, map_chunk) in completed.map_chunks {
                if cache.get_chunk(sub_coord).is_none() {
                    cache.insert_lod_chunk(sub_coord, map_chunk);
                }
            }
        }
    }
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

    let mut chunks = HashSet::new();

    // Use cylindrical horizontal distance so vertical mountain peaks are not sliced off
    let min_y = (center.y - render_distance).max(WORLD_MIN_CHUNK_Y);
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
            let est_max_chunk_y = est_max_height.div_euclid(crate::world::CHUNK_SIZE as i32);

            // Safe column upper bound:
            // 1. At least 1 chunk buffer above treetops/mountain peaks
            // 2. But keep chunks around the player loaded if player is flying high (center.y + 2)
            let col_max_y = (est_max_chunk_y + 1).max(center.y + 2).min(max_y);

            for y in min_y..=col_max_y {
                chunks.insert(IVec3::new(col_chunk_x, y, col_chunk_z));
            }
        }
    }

    chunks
}

pub fn neighbors(coordinate: IVec3) -> impl Iterator<Item = IVec3> {
    NEIGHBOR_DIRECTIONS
        .into_iter()
        .map(move |direction| coordinate + direction)
}

pub fn chunk_distance_squared(a: IVec3, b: IVec3) -> i32 {
    let delta = a - b;

    delta.x * delta.x + delta.y * delta.y + delta.z * delta.z
}
