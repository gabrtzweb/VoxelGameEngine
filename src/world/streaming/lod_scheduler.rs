use bevy::{prelude::*, tasks::{AsyncComputeTaskPool, futures::check_ready}};

use crate::{
    generation::TerrainGenerator,
    meshing::{ChunkMaterial, LodMeshRegistry, apply_lod_mesh, build_lod_mesh},
    world::streaming::{
        ChunkStreamingQueues,
        manager::{
            ChunkStreamingState, CompletedLodMesh, LodMeshingTask, MAX_LOD_MESH_UPLOADS_PER_FRAME,
            MAX_LOD_TASKS_IN_FLIGHT, MAX_LOD_TASKS_STARTED_PER_FRAME,
        },
    },
};

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
