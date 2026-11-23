use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, futures::check_ready},
};

use crate::{
    generation::TerrainGenerator,
    simulation::lighting::{VoxelLightRegistry, sync_chunk_lights},
    world::{
        VoxelWorld,
        modifications::WorldModificationStore,
        streaming::{
            ChunkStreamingQueues,
            manager::{
                ChunkGenerationTask, ChunkStreamingState, GeneratedChunk,
                MAX_GENERATION_TASKS_IN_FLIGHT, MAX_GENERATION_TASKS_STARTED_PER_FRAME, neighbors,
            },
        },
    },
};

pub fn start_generation_tasks(
    mut commands: Commands,
    terrain_generator: Res<TerrainGenerator>,
    active_tasks: Query<&ChunkGenerationTask>,
    mut queues: ResMut<ChunkStreamingQueues>,
    mut chunk_cache: Option<ResMut<crate::world::ChunkCache>>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
) {
    let is_loading = menu_state.is_some_and(|s| *s.get() == crate::menu::MenuState::Loading);
    let max_in_flight = if is_loading {
        48
    } else {
        MAX_GENERATION_TASKS_IN_FLIGHT
    };
    let max_started = if is_loading {
        24
    } else {
        MAX_GENERATION_TASKS_STARTED_PER_FRAME
    };

    let active_count = active_tasks.iter().count();

    if active_count >= max_in_flight {
        return;
    }

    let available_slots = max_in_flight - active_count;
    let tasks_to_start = available_slots.min(max_started);
    let pool = AsyncComputeTaskPool::get();

    for _ in 0..tasks_to_start {
        let Some(coordinate) = queues.load.pop_front() else {
            break;
        };

        if let Some(ref mut cache) = chunk_cache
            && let Some(cached_chunk_arc) = cache.take(&coordinate)
        {
            let task = pool.spawn(async move {
                GeneratedChunk {
                    coordinate,
                    chunk: cached_chunk_arc,
                }
            });
            commands.spawn(ChunkGenerationTask { coordinate, task });
            continue;
        }

        let generator = terrain_generator.clone();

        let task = pool.spawn(async move {
            let chunk = std::sync::Arc::new(generator.generate_chunk(coordinate));
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
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
) {
    let is_loading = menu_state.is_some_and(|s| *s.get() == crate::menu::MenuState::Loading);
    let max_collect = if is_loading { 48 } else { 6 };
    let mut collected = 0;

    for (entity, mut generation_task) in &mut tasks {
        if collected >= max_collect {
            break;
        }
        let Some(mut generated) = check_ready(&mut generation_task.task) else {
            continue;
        };

        collected += 1;
        commands.entity(entity).despawn();

        if !state.desired_chunks.contains(&generated.coordinate) {
            continue;
        }

        if modifications.has_modifications(generated.coordinate) {
            modifications.apply_to_chunk(
                generated.coordinate,
                std::sync::Arc::make_mut(&mut generated.chunk),
            );
        }
        let is_empty = generated.chunk.is_empty();
        world.insert_chunk_arc(generated.coordinate, generated.chunk);

        if is_empty {
            continue;
        }

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
            if let Some(neighbor_chunk) = world.get_chunk(neighbor)
                && !neighbor_chunk.is_empty()
            {
                queues.enqueue_remesh(neighbor);
            }
        }
    }
}
