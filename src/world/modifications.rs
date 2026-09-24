use std::collections::HashMap;

use bevy::prelude::*;

use super::{
    block::{BlockShape, Voxel},
    chunk::Chunk,
    storage::VoxelWorld,
};

#[derive(Resource, Default)]
pub struct WorldModificationStore {
    chunks: HashMap<IVec3, HashMap<UVec3, Voxel>>,
    shapes: HashMap<IVec3, HashMap<UVec3, (BlockShape, u8)>>,
}

impl WorldModificationStore {
    pub fn record(&mut self, world_voxel: IVec3, voxel: Voxel) {
        let (chunk_coordinate, local_coordinate) = VoxelWorld::world_voxel_to_chunk(world_voxel);

        self.chunks
            .entry(chunk_coordinate)
            .or_default()
            .insert(local_coordinate, voxel);

        if voxel.is_empty()
            && let Some(shape_map) = self.shapes.get_mut(&chunk_coordinate)
        {
            shape_map.remove(&local_coordinate);
        }
    }

    pub fn record_shape(&mut self, world_voxel: IVec3, shape: BlockShape, orientation: u8) {
        let (chunk_coordinate, local_coordinate) = VoxelWorld::world_voxel_to_chunk(world_voxel);

        if shape == BlockShape::Full {
            if let Some(shape_map) = self.shapes.get_mut(&chunk_coordinate) {
                shape_map.remove(&local_coordinate);
            }
        } else {
            self.shapes
                .entry(chunk_coordinate)
                .or_default()
                .insert(local_coordinate, (shape, orientation));
        }
    }

    pub fn apply_to_chunk(&self, chunk_coordinate: IVec3, chunk: &mut Chunk) {
        if let Some(modifications) = self.chunks.get(&chunk_coordinate) {
            for (&local_coordinate, &voxel) in modifications {
                chunk.set(
                    local_coordinate.x as usize,
                    local_coordinate.y as usize,
                    local_coordinate.z as usize,
                    voxel,
                );
            }
        }

        if let Some(shape_mods) = self.shapes.get(&chunk_coordinate) {
            for (&local_coordinate, &(shape, orientation)) in shape_mods {
                chunk.set_shape(
                    local_coordinate.x as usize,
                    local_coordinate.y as usize,
                    local_coordinate.z as usize,
                    shape,
                    orientation,
                );
            }
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.shapes.clear();
    }

    pub fn modified_chunks(&self) -> Vec<IVec3> {
        self.chunks.keys().copied().collect()
    }
}
