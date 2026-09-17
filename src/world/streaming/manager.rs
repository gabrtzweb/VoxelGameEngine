use std::collections::HashSet;

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};

use crate::{
    generation::{BiomeType, CaveGenerator, ClimateGenerator, StrataGenerator, TerrainGenerator},
    meshing::{ChunkMeshRegistry, remove_chunk_render},
    player::{Player, PlayerSet},
    simulation::lighting::{VoxelLightRegistry, remove_chunk_lights, sync_chunk_lights},
    world::{
        VOXEL_SIZE, chunk::Chunk, modifications::WorldModificationStore, storage::VoxelWorld,
        streaming::queues::ChunkStreamingQueues,
    },
};

const DEFAULT_RENDER_DISTANCE: i32 = 12;
pub const DEFAULT_SIMULATION_DISTANCE: i32 = 4;

pub const WORLD_MIN_CHUNK_Y: i32 = -10;
pub const WORLD_MAX_CHUNK_Y: i32 = 10;

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

pub fn handle_terrain_generator_reload(
    generator: Res<TerrainGenerator>,
    mut state: ResMut<ChunkStreamingState>,
    world: Res<VoxelWorld>,
    generation_tasks: Query<(Entity, &ChunkGenerationTask)>,
    mut commands: Commands,
    mut queues: ResMut<ChunkStreamingQueues>,
) {
    if !generator.is_changed() || generator.is_added() {
        return;
    }

    state.last_player_chunk = None;

    for (entity, _) in &generation_tasks {
        commands.entity(entity).despawn();
    }

    queues.remesh.clear();
    queues.remesh_set.clear();

    let loaded: Vec<IVec3> = world.iter_chunks().map(|(&c, _)| c).collect();
    for coordinate in loaded {
        if !queues.unload.contains(&coordinate) {
            queues.unload.push_back(coordinate);
        }
    }
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

    let loaded_chunks: HashSet<IVec3> = world
        .iter_chunks()
        .map(|(&coordinate, _)| coordinate)
        .collect();

    let generating_chunks: HashSet<IVec3> = generation_tasks
        .iter()
        .map(|task| task.coordinate)
        .collect();

    let mut chunks_to_load: Vec<IVec3> = desired_chunks
        .iter()
        .filter(|coordinate| {
            !loaded_chunks.contains(coordinate) && !generating_chunks.contains(coordinate)
        })
        .copied()
        .collect();

    let mut chunks_to_unload: Vec<IVec3> =
        loaded_chunks.difference(&desired_chunks).copied().collect();

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
        queues.remesh.retain(|&c| c != coordinate);

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

pub fn collect_generation_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkGenerationTask)>,
    state: Res<ChunkStreamingState>,
    modifications: Res<WorldModificationStore>,
    mut world: ResMut<VoxelWorld>,
    mut queues: ResMut<ChunkStreamingQueues>,
    mut light_registry: ResMut<VoxelLightRegistry>,
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

    for y in -render_distance..=render_distance {
        for z in -render_distance..=render_distance {
            for x in -render_distance..=render_distance {
                let distance_squared = x * x + y * y + z * z;

                if distance_squared > radius_squared {
                    continue;
                }

                let coordinate = center + IVec3::new(x, y, z);

                if coordinate.y < WORLD_MIN_CHUNK_Y || coordinate.y > WORLD_MAX_CHUNK_Y {
                    continue;
                }

                chunks.insert(coordinate);
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
