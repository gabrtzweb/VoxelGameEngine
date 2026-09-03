use std::collections::HashMap;

use bevy::prelude::*;

use super::chunk::{CHUNK_SIZE, Chunk, VOXEL_SIZE, Voxel};

pub const CHUNK_WORLD_SIZE: f32 = CHUNK_SIZE as f32 * VOXEL_SIZE;

#[derive(Resource, Default)]
pub struct VoxelWorld {
    chunks: HashMap<IVec3, Chunk>,
}

impl VoxelWorld {
    pub fn insert_chunk(&mut self, coordinate: IVec3, chunk: Chunk) {
        self.chunks.insert(coordinate, chunk);
    }

    pub fn get_chunk(&self, coordinate: IVec3) -> Option<&Chunk> {
        self.chunks.get(&coordinate)
    }

    pub fn get_chunk_mut(&mut self, coordinate: IVec3) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coordinate)
    }

    pub fn get_voxel(&self, world_voxel: IVec3) -> Option<Voxel> {
        let (chunk_coordinate, local_coordinate) = Self::world_voxel_to_chunk(world_voxel);

        let chunk = self.get_chunk(chunk_coordinate)?;

        Some(chunk.get(
            local_coordinate.x as usize,
            local_coordinate.y as usize,
            local_coordinate.z as usize,
        ))
    }

    pub fn set_voxel(&mut self, world_voxel: IVec3, voxel: Voxel) -> Option<IVec3> {
        let (chunk_coordinate, local_coordinate) = Self::world_voxel_to_chunk(world_voxel);

        let chunk = self.get_chunk_mut(chunk_coordinate)?;

        chunk.set(
            local_coordinate.x as usize,
            local_coordinate.y as usize,
            local_coordinate.z as usize,
            voxel,
        );

        Some(chunk_coordinate)
    }

    pub fn world_voxel_to_chunk(world_voxel: IVec3) -> (IVec3, UVec3) {
        let chunk_size = CHUNK_SIZE as i32;

        let chunk_coordinate = IVec3::new(
            world_voxel.x.div_euclid(chunk_size),
            world_voxel.y.div_euclid(chunk_size),
            world_voxel.z.div_euclid(chunk_size),
        );

        let local_coordinate = UVec3::new(
            world_voxel.x.rem_euclid(chunk_size) as u32,
            world_voxel.y.rem_euclid(chunk_size) as u32,
            world_voxel.z.rem_euclid(chunk_size) as u32,
        );

        (chunk_coordinate, local_coordinate)
    }

    pub fn chunk_translation(coordinate: IVec3) -> Vec3 {
        Vec3::new(
            coordinate.x as f32,
            coordinate.y as f32,
            coordinate.z as f32,
        ) * CHUNK_WORLD_SIZE
    }

    pub fn iter_chunks(&self) -> impl Iterator<Item = (&IVec3, &Chunk)> {
        self.chunks.iter()
    }

    pub fn remove_chunk(&mut self, coordinate: IVec3) -> Option<Chunk> {
        self.chunks.remove(&coordinate)
    }
}
