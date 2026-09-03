use std::collections::{HashSet, VecDeque};

use crate::player::{Player, PlayerSet};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};

use super::{
    chunk::{Chunk, VOXEL_SIZE},
    light::{VoxelLightRegistry, remove_chunk_lights, sync_chunk_lights},
    modifications::WorldModificationStore,
    render::{
        ChunkMaterial, ChunkMeshRegistry, remove_chunk_render, setup_chunk_material,
        sync_chunk_render,
    },
    targeting::TargetingSet,
    terrain::TerrainGenerator,
    world::VoxelWorld,
};

const DEFAULT_RENDER_DISTANCE: i32 = 8;

const WORLD_MIN_CHUNK_Y: i32 = -8;
const WORLD_MAX_CHUNK_Y: i32 = 7;

const MAX_GENERATION_TASKS_IN_FLIGHT: usize = 24;
const MAX_GENERATION_TASKS_STARTED_PER_FRAME: usize = 8;

const MAX_CHUNK_UNLOADS_PER_FRAME: usize = 24;
const MAX_CHUNK_MESH_UPDATES_PER_FRAME: usize = 4;

const NEIGHBOR_DIRECTIONS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

#[derive(Resource)]
pub struct ChunkStreamingSettings {
    pub render_distance: i32,
}

impl Default for ChunkStreamingSettings {
    fn default() -> Self {
        Self {
            render_distance: DEFAULT_RENDER_DISTANCE,
        }
    }
}

#[derive(Resource, Default)]
struct ChunkStreamingState {
    last_player_chunk: Option<IVec3>,

    desired_chunks: HashSet<IVec3>,
}

#[derive(Resource, Default)]
struct ChunkStreamingQueues {
    load: VecDeque<IVec3>,

    unload: VecDeque<IVec3>,

    remesh: VecDeque<IVec3>,

    remesh_set: HashSet<IVec3>,
}

#[derive(Component)]
struct ChunkGenerationTask {
    coordinate: IVec3,
    task: Task<GeneratedChunk>,
}

struct GeneratedChunk {
    coordinate: IVec3,
    chunk: Chunk,
}

pub struct ChunkManagerPlugin;

impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .init_resource::<ChunkMeshRegistry>()
            .init_resource::<ChunkStreamingSettings>()
            .init_resource::<ChunkStreamingState>()
            .init_resource::<ChunkStreamingQueues>()
            .init_resource::<WorldModificationStore>()
            .init_resource::<VoxelLightRegistry>()
            .insert_resource(TerrainGenerator::default())
            .add_systems(Startup, setup_chunk_material)
            .add_systems(
                Update,
                (
                    plan_chunk_streaming,
                    process_chunk_unloads,
                    start_generation_tasks,
                    collect_generation_tasks,
                    process_chunk_meshing,
                )
                    .chain()
                    .after(PlayerSet::Movement)
                    .before(TargetingSet::UpdateTarget),
            );
    }
}

fn plan_chunk_streaming(
    player: Single<&Transform, With<Player>>,
    settings: Res<ChunkStreamingSettings>,
    mut state: ResMut<ChunkStreamingState>,
    world: Res<VoxelWorld>,
    generation_tasks: Query<&ChunkGenerationTask>,
    mut queues: ResMut<ChunkStreamingQueues>,
) {
    let player_chunk = player_chunk_coordinate(player.translation);

    if state.last_player_chunk == Some(player_chunk) {
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

fn process_chunk_unloads(
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

        // PointLight entities must disappear before
        // their voxel chunk is removed from memory.
        remove_chunk_lights(&mut commands, coordinate, &mut light_registry);

        if world.remove_chunk(coordinate).is_none() {
            continue;
        }

        remove_chunk_render(&mut commands, coordinate, &mut registry, &mut meshes);

        for neighbor in neighbors(coordinate) {
            if world.get_chunk(neighbor).is_some() {
                enqueue_remesh(&mut queues, neighbor);
            }
        }
    }
}

fn start_generation_tasks(
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

fn collect_generation_tasks(
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

        // The player may have moved while this chunk
        // was being generated asynchronously.
        if !state.desired_chunks.contains(&generated.coordinate) {
            continue;
        }

        // Restore runtime edits before inserting the
        // chunk into the active world.
        modifications.apply_to_chunk(generated.coordinate, &mut generated.chunk);

        world.insert_chunk(generated.coordinate, generated.chunk);

        // Any Light voxels contained in the generated
        // chunk now receive their PointLight entities.
        sync_chunk_lights(
            &mut commands,
            &world,
            generated.coordinate,
            &mut light_registry,
        );

        enqueue_remesh(&mut queues, generated.coordinate);

        for neighbor in neighbors(generated.coordinate) {
            if world.get_chunk(neighbor).is_some() {
                enqueue_remesh(&mut queues, neighbor);
            }
        }
    }
}

fn process_chunk_meshing(
    mut commands: Commands,
    mut queues: ResMut<ChunkStreamingQueues>,
    world: Res<VoxelWorld>,
    material: Res<ChunkMaterial>,
    mut registry: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for _ in 0..MAX_CHUNK_MESH_UPDATES_PER_FRAME {
        let Some(coordinate) = queues.remesh.pop_front() else {
            break;
        };

        queues.remesh_set.remove(&coordinate);

        if world.get_chunk(coordinate).is_none() {
            continue;
        }

        sync_chunk_render(
            &mut commands,
            &world,
            coordinate,
            &mut registry,
            &mut meshes,
            &material,
        );
    }
}

fn enqueue_remesh(queues: &mut ChunkStreamingQueues, coordinate: IVec3) {
    if queues.remesh_set.insert(coordinate) {
        queues.remesh.push_back(coordinate);
    }
}

fn player_chunk_coordinate(player_position: Vec3) -> IVec3 {
    let player_voxel = IVec3::new(
        (player_position.x / VOXEL_SIZE).floor() as i32,
        (player_position.y / VOXEL_SIZE).floor() as i32,
        (player_position.z / VOXEL_SIZE).floor() as i32,
    );

    let (chunk_coordinate, _) = VoxelWorld::world_voxel_to_chunk(player_voxel);

    chunk_coordinate
}

fn desired_chunk_coordinates(center: IVec3, render_distance: i32) -> HashSet<IVec3> {
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

fn neighbors(coordinate: IVec3) -> impl Iterator<Item = IVec3> {
    NEIGHBOR_DIRECTIONS
        .into_iter()
        .map(move |direction| coordinate + direction)
}

fn chunk_distance_squared(a: IVec3, b: IVec3) -> i32 {
    let delta = a - b;

    delta.x * delta.x + delta.y * delta.y + delta.z * delta.z
}
