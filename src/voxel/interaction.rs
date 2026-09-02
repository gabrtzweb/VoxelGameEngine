use std::collections::HashMap;

use bevy::prelude::*;

use super::{
    chunk::{VOXEL_SIZE, Voxel},
    mesher::ChunkMesher,
    world::VoxelWorld,
};

const MAX_INTERACTION_DISTANCE: f32 = 10.0;

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
        app.add_systems(Update, edit_voxels);
    }
}

struct VoxelHit {
    voxel: IVec3,
    previous: Option<IVec3>,
}

fn edit_voxels(
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    mut world: ResMut<VoxelWorld>,
    chunk_meshes: Res<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let break_pressed = mouse.just_pressed(MouseButton::Left);
    let place_pressed = mouse.just_pressed(MouseButton::Right);

    if !break_pressed && !place_pressed {
        return;
    }

    let ray_origin = camera.translation();
    let ray_direction = *camera.forward();

    let Some(hit) = raycast_world(&world, ray_origin, ray_direction, MAX_INTERACTION_DISTANCE)
    else {
        return;
    };

    let changed_chunk = if break_pressed {
        remove_voxel(&mut world, hit.voxel)
    } else if let Some(target) = hit.previous {
        place_voxel(&mut world, target)
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

fn raycast_world(
    world: &VoxelWorld,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<VoxelHit> {
    let direction = direction.normalize();
    let grid_origin = origin / VOXEL_SIZE;

    let mut voxel = IVec3::new(
        grid_origin.x.floor() as i32,
        grid_origin.y.floor() as i32,
        grid_origin.z.floor() as i32,
    );

    let step = IVec3::new(
        direction.x.signum() as i32,
        direction.y.signum() as i32,
        direction.z.signum() as i32,
    );

    let delta_distance = Vec3::new(
        axis_delta(direction.x),
        axis_delta(direction.y),
        axis_delta(direction.z),
    );

    let mut side_distance = Vec3::new(
        initial_side_distance(grid_origin.x, voxel.x, step.x, delta_distance.x),
        initial_side_distance(grid_origin.y, voxel.y, step.y, delta_distance.y),
        initial_side_distance(grid_origin.z, voxel.z, step.z, delta_distance.z),
    );

    let max_grid_distance = max_distance / VOXEL_SIZE;
    let mut traveled_distance = 0.0;
    let mut previous = None;

    while traveled_distance <= max_grid_distance {
        if let Some(current_voxel) = world.get_voxel(voxel) {
            if current_voxel != Voxel::Air {
                return Some(VoxelHit { voxel, previous });
            }
        }

        previous = Some(voxel);

        if side_distance.x <= side_distance.y && side_distance.x <= side_distance.z {
            voxel.x += step.x;
            traveled_distance = side_distance.x;
            side_distance.x += delta_distance.x;
        } else if side_distance.y <= side_distance.z {
            voxel.y += step.y;
            traveled_distance = side_distance.y;
            side_distance.y += delta_distance.y;
        } else {
            voxel.z += step.z;
            traveled_distance = side_distance.z;
            side_distance.z += delta_distance.z;
        }
    }

    None
}

fn axis_delta(direction: f32) -> f32 {
    if direction.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        1.0 / direction.abs()
    }
}

fn initial_side_distance(origin: f32, voxel: i32, step: i32, delta_distance: f32) -> f32 {
    if step > 0 {
        (voxel as f32 + 1.0 - origin) * delta_distance
    } else if step < 0 {
        (origin - voxel as f32) * delta_distance
    } else {
        f32::INFINITY
    }
}
