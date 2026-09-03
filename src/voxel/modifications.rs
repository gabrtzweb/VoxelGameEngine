use std::collections::HashMap;

use bevy::prelude::*;

use super::{
    chunk::{Chunk, Voxel},
    world::VoxelWorld,
};

#[derive(Resource, Default)]
pub struct WorldModificationStore {
    chunks: HashMap<IVec3, HashMap<UVec3, Voxel>>,
}

impl WorldModificationStore {
    pub fn record(&mut self, world_voxel: IVec3, voxel: Voxel) {
        let (chunk_coordinate, local_coordinate) = VoxelWorld::world_voxel_to_chunk(world_voxel);

        self.chunks
            .entry(chunk_coordinate)
            .or_default()
            .insert(local_coordinate, voxel);
    }

    pub fn apply_to_chunk(&self, chunk_coordinate: IVec3, chunk: &mut Chunk) {
        let Some(modifications) = self.chunks.get(&chunk_coordinate) else {
            return;
        };

        for (&local_coordinate, &voxel) in modifications {
            chunk.set(
                local_coordinate.x as usize,
                local_coordinate.y as usize,
                local_coordinate.z as usize,
                voxel,
            );
        }
    }
}
