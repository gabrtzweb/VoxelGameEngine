use std::collections::{HashSet, VecDeque};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ChunkStreamingQueues {
    pub load: VecDeque<IVec3>,
    pub unload: VecDeque<IVec3>,
    pub remesh: VecDeque<IVec3>,
    pub remesh_set: HashSet<IVec3>,
}

impl ChunkStreamingQueues {
    pub fn enqueue_remesh(&mut self, coordinate: IVec3) {
        if self.remesh_set.insert(coordinate) {
            self.remesh.push_back(coordinate);
        }
    }

    pub fn enqueue_priority_remesh(&mut self, coordinate: IVec3) {
        if self.remesh_set.insert(coordinate) {
            self.remesh.push_front(coordinate);
        } else if let Some(idx) = self.remesh.iter().position(|&c| c == coordinate) {
            self.remesh.remove(idx);
            self.remesh.push_front(coordinate);
        }
    }
}
