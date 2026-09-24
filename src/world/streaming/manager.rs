use std::collections::HashSet;

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};

use crate::{
    generation::{BiomeType, CaveGenerator, ClimateGenerator, StrataGenerator, TerrainGenerator},
    meshing::{ChunkMeshRegistry, ChunkMeshingTask, remove_chunk_render},
    player::{Player, PlayerSet},
    simulation::lighting::{VoxelLightRegistry, remove_chunk_lights, sync_chunk_lights},
    world::{
        VOXEL_SIZE, chunk::Chunk, modifications::WorldModificationStore, storage::VoxelWorld,
        streaming::queues::ChunkStreamingQueues,
    },
};

const DEFAULT_RENDER_DISTANCE: i32 = 12;
pub const DEFAULT_SIMULATION_DISTANCE: i32 = 4;

pub const WORLD_MIN_CHUNK_Y: i32 = -16;
pub const WORLD_MAX_CHUNK_Y: i32 = 16;

const MAX_GENERATION_TASKS_IN_FLIGHT: usize = 24;
const MAX_GENERATION_TASKS_STARTED_PER_FRAME: usize = 8;

const MAX_CHUNK_UNLOADS_PER_FRAME: usize = 64;

pub const NEIGHBOR_DIRECTIONS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];
#[allow(dead_code)]
pub const NEIGHBOR_CHUNK_OFFSETS: [IVec3; 6] = NEIGHBOR_DIRECTIONS;

#[derive(Resource)]
pub struct ChunkStreamingSettings {
    pub render_distance: i32,
    pub simulation_distance: i32,
}

impl Default for ChunkStreamingSettings {
    fn default() -> Self {
        Self {
            render_distance: DEFAULT_RENDER_DISTANCE,
            simulation_distance: DEFAULT_SIMULATION_DISTANCE,
        }
    }
}

#[derive(Resource, Default)]
pub struct ChunkStreamingState {
    pub last_player_chunk: Option<IVec3>,
    pub desired_chunks: HashSet<IVec3>,
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
                    start_generation_tasks,
                    collect_generation_tasks,
                )
                    .chain()
                    .after(PlayerSet::Movement),
            );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_terrain_generator_reload(
    generator: Res<TerrainGenerator>,
    mut last_version: Local<u32>,
    mut state: ResMut<ChunkStreamingState>,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut mesh_registry: ResMut<ChunkMeshRegistry>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    generation_tasks: Query<(Entity, &ChunkGenerationTask)>,
    meshing_tasks: Query<(Entity, &ChunkMeshingTask)>,
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
            entity_cmds.despawn();
        }
    }

    // 2. Despawn all in-flight meshing tasks
    for (entity, _) in &meshing_tasks {
        if let Ok(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.despawn();
        }
    }

    // 3. Despawn and remove all existing chunk meshes
    let mesh_coords: Vec<IVec3> = mesh_registry.iter_coordinates().copied().collect();
    for coordinate in mesh_coords {
        remove_chunk_render(&mut commands, coordinate, &mut mesh_registry, &mut meshes);
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

    // 7. Reset player tracking so plan_chunk_streaming immediately queues full reload around player
    state.last_player_chunk = None;
    state.desired_chunks.clear();
}

pub fn plan_chunk_streaming(
    player: Single<&Transform, With<Player>>,
    settings: Res<ChunkStreamingSettings>,
    mut state: ResMut<ChunkStreamingState>,
    world: Res<VoxelWorld>,
    generation_tasks: Query<&ChunkGenerationTask>,
    mut queues: ResMut<ChunkStreamingQueues>,
) {
    let player_chunk = player_chunk_coordinate(player.translation);

    if !settings.is_changed() && state.last_player_chunk == Some(player_chunk) {
        return;
    }

    state.last_player_chunk = Some(player_chunk);

    let desired_chunks = desired_chunk_coordinates(player_chunk, settings.render_distance);

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

    chunks_to_load.sort_by_key(|coordinate| chunk_distance_squared(*coordinate, player_chunk));

    chunks_to_unload.sort_by_key(|coordinate| {
        std::cmp::Reverse(chunk_distance_squared(*coordinate, player_chunk))
    });

    queues.load.clear();
    queues.unload.clear();

    queues.load.extend(chunks_to_load);

    queues.unload.extend(chunks_to_unload);
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

#[allow(clippy::too_many_arguments)]
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

        if let Some(ref mut cache) = map_cache {
            if generated.coordinate.y >= -8 && generated.coordinate.y <= 14 {
                cache.mark_dirty(IVec2::new(generated.coordinate.x, generated.coordinate.z));
            }
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

pub fn player_chunk_coordinate(player_position: Vec3) -> IVec3 {
    let player_voxel = IVec3::new(
        (player_position.x / VOXEL_SIZE).floor() as i32,
        (player_position.y / VOXEL_SIZE).floor() as i32,
        (player_position.z / VOXEL_SIZE).floor() as i32,
    );

    let (chunk_coordinate, _) = VoxelWorld::world_voxel_to_chunk(player_voxel);

    chunk_coordinate
}

pub fn desired_chunk_coordinates(center: IVec3, render_distance: i32) -> HashSet<IVec3> {
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

            for y in min_y..=max_y {
                chunks.insert(IVec3::new(center.x + x, y, center.z + z));
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
