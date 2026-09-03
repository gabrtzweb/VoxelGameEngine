use bevy::prelude::*;

use super::{
    chunk::{CHUNK_SIZE, Voxel},
    modifications::WorldModificationStore,
    render::{ChunkMaterial, ChunkMeshRegistry, sync_chunk_render},
    targeting::{CurrentTarget, TargetingSet},
    world::VoxelWorld,
};

const HOLD_DELAY: f32 = 0.25;
const REPEAT_INTERVAL: f32 = 0.16;

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
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    current_target: Res<CurrentTarget>,
    material: Res<ChunkMaterial>,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut registry: ResMut<ChunkMeshRegistry>,
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
        if remove_voxel(&mut world, &mut modifications, target.hit_voxel) {
            Some(target.hit_voxel)
        } else {
            None
        }
    } else if place_action {
        if let Some(place_position) = target.place_voxel {
            if place_voxel(&mut world, &mut modifications, place_position) {
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

    for coordinate in dirty_chunks {
        if world.get_chunk(coordinate).is_none() {
            continue;
        }

        sync_chunk_render(
            &mut commands,
            &world,
            coordinate,
            &mut registry,
            &mut meshes,
            &material,
        );
    }
}

fn remove_voxel(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    position: IVec3,
) -> bool {
    let Some(voxel) = world.get_voxel(position) else {
        return false;
    };

    if voxel == Voxel::Air {
        return false;
    }

    let Some(chunk_coordinate) = world.set_voxel(position, Voxel::Air) else {
        return false;
    };

    modifications.record(position, Voxel::Air);

    info!(
        "Removed voxel {:?} from chunk {:?}",
        position, chunk_coordinate
    );

    true
}

fn place_voxel(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    position: IVec3,
) -> bool {
    let Some(voxel) = world.get_voxel(position) else {
        return false;
    };

    if voxel != Voxel::Air {
        return false;
    }

    let Some(chunk_coordinate) = world.set_voxel(position, Voxel::Solid) else {
        return false;
    };

    modifications.record(position, Voxel::Solid);

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
