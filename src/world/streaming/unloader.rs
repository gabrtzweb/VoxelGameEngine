use bevy::prelude::*;

use crate::{
    meshing::{ChunkMeshRegistry, LodMeshRegistry, remove_chunk_render, remove_lod_render},
    simulation::lighting::{VoxelLightRegistry, remove_chunk_lights},
    world::{
        VoxelWorld,
        streaming::{
            ChunkStreamingQueues,
            manager::{MAX_CHUNK_UNLOADS_PER_FRAME, MAX_LOD_UNLOADS_PER_FRAME, neighbors},
        },
    },
};

pub fn process_chunk_unloads(
    mut commands: Commands,
    mut queues: ResMut<ChunkStreamingQueues>,
    mut world: ResMut<VoxelWorld>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut registry: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_cache: Option<ResMut<crate::world::ChunkCache>>,
) {
    for _ in 0..MAX_CHUNK_UNLOADS_PER_FRAME {
        let Some(coordinate) = queues.unload.pop_front() else {
            break;
        };

        if world.get_chunk(coordinate).is_none() {
            continue;
        }

        remove_chunk_lights(&mut commands, coordinate, &mut light_registry);

        let Some(chunk_arc) = world.remove_chunk(coordinate) else {
            continue;
        };

        if let Some(ref mut cache) = chunk_cache {
            cache.insert(coordinate, chunk_arc);
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
