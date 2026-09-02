use bevy::prelude::*;

use super::{
    chunk::{Chunk, VOXEL_SIZE, Voxel},
    mesher::ChunkMesher,
};

const MAX_INTERACTION_DISTANCE: f32 = 10.0;

#[derive(Resource)]
pub struct ActiveChunk {
    pub chunk: Chunk,
    pub mesh_handle: Handle<Mesh>,
}

impl ActiveChunk {
    pub fn new(chunk: Chunk, mesh_handle: Handle<Mesh>) -> Self {
        Self { chunk, mesh_handle }
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
    mut active_chunk: ResMut<ActiveChunk>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let break_pressed = mouse.just_pressed(MouseButton::Left);
    let place_pressed = mouse.just_pressed(MouseButton::Right);

    if !break_pressed && !place_pressed {
        return;
    }

    let ray_origin = camera.translation();
    let ray_direction = *camera.forward();

    let Some(hit) = raycast_chunk(
        &active_chunk.chunk,
        ray_origin,
        ray_direction,
        MAX_INTERACTION_DISTANCE,
    ) else {
        return;
    };

    let changed = if break_pressed {
        remove_voxel(&mut active_chunk.chunk, hit.voxel)
    } else if let Some(target) = hit.previous {
        place_voxel(&mut active_chunk.chunk, target)
    } else {
        false
    };

    if !changed {
        return;
    }

    let rebuilt_mesh = ChunkMesher::build_mesh(&active_chunk.chunk);

    if let Some(mut mesh) = meshes.get_mut(&active_chunk.mesh_handle) {
        *mesh = rebuilt_mesh;
    }
}

fn remove_voxel(chunk: &mut Chunk, position: IVec3) -> bool {
    if !is_inside_chunk(position) {
        return false;
    }

    let x = position.x as usize;
    let y = position.y as usize;
    let z = position.z as usize;

    if chunk.get(x, y, z) == Voxel::Air {
        return false;
    }

    chunk.set(x, y, z, Voxel::Air);

    info!("Removed voxel at {:?}", position);

    true
}

fn place_voxel(chunk: &mut Chunk, position: IVec3) -> bool {
    if !is_inside_chunk(position) {
        return false;
    }

    let x = position.x as usize;
    let y = position.y as usize;
    let z = position.z as usize;

    if chunk.get(x, y, z) != Voxel::Air {
        return false;
    }

    chunk.set(x, y, z, Voxel::Solid);

    info!("Placed voxel at {:?}", position);

    true
}

fn raycast_chunk(
    chunk: &Chunk,
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
        if is_inside_chunk(voxel) {
            let current = chunk.get(voxel.x as usize, voxel.y as usize, voxel.z as usize);

            if current != Voxel::Air {
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

fn is_inside_chunk(position: IVec3) -> bool {
    Chunk::is_inside(
        position.x as isize,
        position.y as isize,
        position.z as isize,
    )
}
