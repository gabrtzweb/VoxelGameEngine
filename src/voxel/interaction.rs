use std::collections::HashMap;

use bevy::prelude::*;

use super::{
    chunk::{CHUNK_SIZE, Voxel},
    mesher::ChunkMesher,
    targeting::{CurrentTarget, TargetingSet},
    world::VoxelWorld,
};

const HOLD_DELAY: f32 = 0.25;
const REPEAT_INTERVAL: f32 = 0.16;

#[derive(Resource, Default)]
pub struct ChunkMeshRegistry {
    handles: HashMap<IVec3, Handle<Mesh>>,
}

impl ChunkMeshRegistry {
    pub fn insert(&mut self, coordinate: IVec3, mesh_handle: Handle<Mesh>) {
        self.handles.insert(coordinate, mesh_handle);
    }

    pub fn get(&self, coordinate: IVec3) -> Option<&Handle<Mesh>> {
        self.handles.get(&coordinate)
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }
}

#[derive(Default)]
struct HoldActionState {
    hold_time: f32,
    repeat_time: f32,
}

impl HoldActionState {
    fn update(&mut self, pressed: bool, just_pressed: bool, delta_seconds: f32) -> bool {
        if just_pressed {
            self.hold_time = 0.0;
            self.repeat_time = 0.0;
            return true;
        }

        if !pressed {
            self.hold_time = 0.0;
            self.repeat_time = 0.0;
            return false;
        }

        self.hold_time += delta_seconds;

        if self.hold_time < HOLD_DELAY {
            return false;
        }

        self.repeat_time += delta_seconds;

        if self.repeat_time >= REPEAT_INTERVAL {
            self.repeat_time -= REPEAT_INTERVAL;
            return true;
        }

        false
    }
}

#[derive(Default)]
struct InteractionState {
    break_action: HoldActionState,
    place_action: HoldActionState,
}

pub struct VoxelInteractionPlugin;

impl Plugin for VoxelInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, edit_voxels.after(TargetingSet::UpdateTarget));
    }
}

fn edit_voxels(
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    current_target: Res<CurrentTarget>,
    mut world: ResMut<VoxelWorld>,
    chunk_meshes: Res<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut interaction_state: Local<InteractionState>,
) {
    let delta_seconds = time.delta_secs();

    let break_action = interaction_state.break_action.update(
        mouse.pressed(MouseButton::Left),
        mouse.just_pressed(MouseButton::Left),
        delta_seconds,
    );

    let place_action = interaction_state.place_action.update(
        mouse.pressed(MouseButton::Right),
        mouse.just_pressed(MouseButton::Right),
        delta_seconds,
    );

    if !break_action && !place_action {
        return;
    }

    let Some(target) = current_target.hit else {
        return;
    };

    let edited_voxel = if break_action {
        if remove_voxel(&mut world, target.hit_voxel) {
            Some(target.hit_voxel)
        } else {
            None
        }
    } else if place_action {
        if let Some(place_position) = target.place_voxel {
            if place_voxel(&mut world, place_position) {
                Some(place_position)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let Some(edited_voxel) = edited_voxel else {
        return;
    };

    let dirty_chunks = affected_chunks(edited_voxel);

    for chunk_coordinate in dirty_chunks {
        if world.get_chunk(chunk_coordinate).is_none() {
            continue;
        }

        let Some(mesh_handle) = chunk_meshes.get(chunk_coordinate) else {
            continue;
        };

        let rebuilt_mesh = ChunkMesher::build_mesh(&world, chunk_coordinate);

        if let Some(mut mesh) = meshes.get_mut(mesh_handle) {
            *mesh = rebuilt_mesh;
        }
    }
}

fn remove_voxel(world: &mut VoxelWorld, position: IVec3) -> bool {
    let Some(voxel) = world.get_voxel(position) else {
        return false;
    };

    if voxel == Voxel::Air {
        return false;
    }

    let Some(chunk_coordinate) = world.set_voxel(position, Voxel::Air) else {
        return false;
    };

    info!(
        "Removed voxel {:?} from chunk {:?}",
        position, chunk_coordinate
    );

    true
}

fn place_voxel(world: &mut VoxelWorld, position: IVec3) -> bool {
    let Some(voxel) = world.get_voxel(position) else {
        return false;
    };

    if voxel != Voxel::Air {
        return false;
    }

    let Some(chunk_coordinate) = world.set_voxel(position, Voxel::Solid) else {
        return false;
    };

    info!(
        "Placed voxel {:?} in chunk {:?}",
        position, chunk_coordinate
    );

    true
}

fn affected_chunks(world_voxel: IVec3) -> Vec<IVec3> {
    let (chunk_coordinate, local_coordinate) = VoxelWorld::world_voxel_to_chunk(world_voxel);

    let mut chunks = Vec::with_capacity(4);

    chunks.push(chunk_coordinate);

    let max_local = (CHUNK_SIZE - 1) as u32;

    if local_coordinate.x == 0 {
        chunks.push(chunk_coordinate + IVec3::new(-1, 0, 0));
    } else if local_coordinate.x == max_local {
        chunks.push(chunk_coordinate + IVec3::new(1, 0, 0));
    }

    if local_coordinate.y == 0 {
        chunks.push(chunk_coordinate + IVec3::new(0, -1, 0));
    } else if local_coordinate.y == max_local {
        chunks.push(chunk_coordinate + IVec3::new(0, 1, 0));
    }

    if local_coordinate.z == 0 {
        chunks.push(chunk_coordinate + IVec3::new(0, 0, -1));
    } else if local_coordinate.z == max_local {
        chunks.push(chunk_coordinate + IVec3::new(0, 0, 1));
    }

    chunks
}
