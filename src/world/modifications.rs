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
    extra_slabs: HashMap<IVec3, HashMap<UVec3, Option<(Voxel, u8)>>>,
}

impl WorldModificationStore {
    pub fn record(&mut self, world_voxel: IVec3, voxel: Voxel) {
        let (chunk_coordinate, local_coordinate) = VoxelWorld::world_voxel_to_chunk(world_voxel);

        self.chunks
            .entry(chunk_coordinate)
            .or_default()
            .insert(local_coordinate, voxel);

        if voxel.is_empty() {
            if let Some(shape_map) = self.shapes.get_mut(&chunk_coordinate) {
                shape_map.remove(&local_coordinate);
            }
            if let Some(extra_map) = self.extra_slabs.get_mut(&chunk_coordinate) {
                extra_map.remove(&local_coordinate);
            }
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

    pub fn record_extra_slab(&mut self, world_voxel: IVec3, extra_slab: Option<(Voxel, u8)>) {
        let (chunk_coordinate, local_coordinate) = VoxelWorld::world_voxel_to_chunk(world_voxel);

        self.extra_slabs
            .entry(chunk_coordinate)
            .or_default()
            .insert(local_coordinate, extra_slab);
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

        if let Some(extra_mods) = self.extra_slabs.get(&chunk_coordinate) {
            for (&local_coordinate, &extra_slab) in extra_mods {
                chunk.set_extra_slab(
                    local_coordinate.x as usize,
                    local_coordinate.y as usize,
                    local_coordinate.z as usize,
                    extra_slab,
                );
            }
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.shapes.clear();
        self.extra_slabs.clear();
    }

    pub fn modified_chunks(&self) -> Vec<IVec3> {
        self.chunks.keys().copied().collect()
    }
}
