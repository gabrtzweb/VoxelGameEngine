use std::collections::HashMap;

use bevy::prelude::*;

use super::{
    chunk::Voxel,
    mesher::ChunkMesher,
    targeting::{CurrentTarget, TargetingSet},
    world::VoxelWorld,
};

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
}

pub struct VoxelInteractionPlugin;

impl Plugin for VoxelInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, edit_voxels.after(TargetingSet::UpdateTarget));
    }
}

fn edit_voxels(
    mouse: Res<ButtonInput<MouseButton>>,
    current_target: Res<CurrentTarget>,
    mut world: ResMut<VoxelWorld>,
    chunk_meshes: Res<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let break_pressed = mouse.just_pressed(MouseButton::Left);
    let place_pressed = mouse.just_pressed(MouseButton::Right);

    if !break_pressed && !place_pressed {
        return;
    }

    let Some(target) = current_target.hit else {
        return;
    };

    let changed_chunk = if break_pressed {
        remove_voxel(&mut world, target.hit_voxel)
    } else if let Some(place_position) = target.place_voxel {
        place_voxel(&mut world, place_position)
    } else {
        None
    };

    let Some(chunk_coordinate) = changed_chunk else {
        return;
    };

    let Some(chunk) = world.get_chunk(chunk_coordinate) else {
        return;
    };

    let rebuilt_mesh = ChunkMesher::build_mesh(chunk);

    let Some(mesh_handle) = chunk_meshes.get(chunk_coordinate) else {
        return;
    };

    if let Some(mut mesh) = meshes.get_mut(mesh_handle) {
        *mesh = rebuilt_mesh;
    }
}

fn remove_voxel(world: &mut VoxelWorld, position: IVec3) -> Option<IVec3> {
    let voxel = world.get_voxel(position)?;

    if voxel == Voxel::Air {
        return None;
    }

    let chunk_coordinate = world.set_voxel(position, Voxel::Air)?;

    info!(
        "Removed voxel {:?} from chunk {:?}",
        position, chunk_coordinate
    );

    Some(chunk_coordinate)
}

fn place_voxel(world: &mut VoxelWorld, position: IVec3) -> Option<IVec3> {
    let voxel = world.get_voxel(position)?;

    if voxel != Voxel::Air {
        return None;
    }

    let chunk_coordinate = world.set_voxel(position, Voxel::Solid)?;

    info!(
        "Placed voxel {:?} in chunk {:?}",
        position, chunk_coordinate
    );

    Some(chunk_coordinate)
}
