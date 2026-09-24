use std::collections::HashSet;

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};

use super::{
    greedy::{ChunkMesher, ChunkMeshes},
    pipeline::{
        ChunkMaterial, ChunkMeshRegistry, apply_chunk_mesh, remove_chunk_render,
        setup_chunk_material,
    },
};
use crate::world::{
    Chunk, ChunkHomogeneity, ChunkNeighborhood, NEIGHBOR_DIRECTIONS, VoxelWorld,
    streaming::{ChunkStreamingQueues, ChunkStreamingState},
};

const MAX_MESHING_TASKS_IN_FLIGHT: usize = 32;
const MAX_MESHING_TASKS_STARTED_PER_FRAME: usize = 8;

#[derive(Component)]
pub struct ChunkMeshingTask {
    pub coordinate: IVec3,
    pub task: Task<CompletedChunkMesh>,
}

pub struct CompletedChunkMesh {
    pub coordinate: IVec3,
    pub meshes: ChunkMeshes,
}

pub struct AsyncMesherPlugin;

impl Plugin for AsyncMesherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkMeshRegistry>()
            .add_systems(Startup, setup_chunk_material)
            .add_systems(Update, collect_meshing_tasks)
            .add_systems(
                PostUpdate,
                (collect_meshing_tasks, start_meshing_tasks).chain(),
            );
    }
}

pub fn start_meshing_tasks(
    mut commands: Commands,
    active_tasks: Query<&ChunkMeshingTask>,
    mut queues: ResMut<ChunkStreamingQueues>,
    world: Res<VoxelWorld>,
    material: Res<ChunkMaterial>,
    mut registry: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    streaming_state: Option<Res<ChunkStreamingState>>,
) {
    let active_count = active_tasks.iter().count();
    if active_count >= MAX_MESHING_TASKS_IN_FLIGHT {
        return;
    }

    let in_flight: HashSet<IVec3> = active_tasks.iter().map(|t| t.coordinate).collect();
    let available_slots =
        (MAX_MESHING_TASKS_IN_FLIGHT - active_count).min(MAX_MESHING_TASKS_STARTED_PER_FRAME);

    let pool = AsyncComputeTaskPool::get();
    let mut started = 0;
    let mut deferred = Vec::new();

    while started < available_slots {
        let Some(coordinate) = queues.remesh.pop_front() else {
            break;
        };

        if in_flight.contains(&coordinate) {
            deferred.push(coordinate);
            continue;
        }

        if !queues.remesh_set.remove(&coordinate) {
            continue;
        }

        if let Some(ref state) = streaming_state {
            if !state.desired_chunks.is_empty() && !state.desired_chunks.contains(&coordinate) {
                continue;
            }
        }

        let Some(chunk) = world.get_chunk(coordinate) else {
            continue;
        };

        // Early skip 1: completely empty chunk (100% Air) has no geometry.
        if chunk.homogeneity() == ChunkHomogeneity::Empty {
            if registry.contains(&coordinate) {
                remove_chunk_render(&mut commands, coordinate, &mut registry, &mut meshes);
            }
            continue;
        }

        // Early skip 2: 100% solid chunk surrounded on all 6 faces by solid opaque chunks has zero exposed faces.
        if chunk.is_fully_solid_opaque() {
            let all_neighbors_solid = NEIGHBOR_DIRECTIONS.iter().all(|&dir| {
                world
                    .get_chunk(coordinate + dir)
                    .is_some_and(Chunk::is_fully_solid_opaque)
            });
            if all_neighbors_solid {
                if registry.contains(&coordinate) {
                    remove_chunk_render(&mut commands, coordinate, &mut registry, &mut meshes);
                }
                continue;
            }
        }

        let neighborhood = ChunkNeighborhood::new(&world, coordinate);
        let textures = material.texture_registry.clone();

        let task = pool.spawn(async move {
            let meshes = ChunkMesher::build_meshes(&neighborhood, coordinate, &textures);
            CompletedChunkMesh { coordinate, meshes }
        });

        commands.spawn(ChunkMeshingTask { coordinate, task });
        started += 1;
    }

    for coord in deferred.into_iter().rev() {
        queues.remesh.push_front(coord);
    }
}

pub fn collect_meshing_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkMeshingTask)>,
    world: Res<VoxelWorld>,
    material: Res<ChunkMaterial>,
    mut registry: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut meshing_task) in &mut tasks {
        let Some(completed) = check_ready(&mut meshing_task.task) else {
            continue;
        };

        commands.entity(entity).despawn();

        if world.get_chunk(completed.coordinate).is_none() {
            remove_chunk_render(
                &mut commands,
                completed.coordinate,
                &mut registry,
                &mut meshes,
            );
            continue;
        }

        apply_chunk_mesh(
            &mut commands,
            completed.coordinate,
            completed.meshes,
            &mut registry,
            &mut meshes,
            &material,
        );
    }
}
