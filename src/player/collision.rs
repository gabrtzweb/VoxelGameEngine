use bevy::prelude::*;

use crate::voxel::{VOXEL_SIZE, VoxelWorld, chunk::Voxel};

use super::{PLAYER_HEIGHT, PLAYER_WIDTH};

const COLLISION_EPSILON: f32 = 0.001;

const MAX_MOVEMENT_STEP: f32 = VOXEL_SIZE * 0.45;

const GROUND_PROBE_DISTANCE: f32 = 0.025;

pub(super) const AUTO_STEP_HEIGHT: f32 = VOXEL_SIZE;

#[derive(Default)]
pub struct CollisionResult {
    pub grounded: bool,
    pub blocked_x: bool,
    pub blocked_y: bool,
    pub blocked_z: bool,
    pub step_height: f32,
}

pub fn is_grounded(world: &VoxelWorld, position: Vec3) -> bool {
    let probe_position = position - Vec3::Y * GROUND_PROBE_DISTANCE;

    collides_at(world, probe_position)
}

pub fn move_with_collisions(
    world: &VoxelWorld,
    position: Vec3,
    movement: Vec3,
    allow_step: bool,
) -> (Vec3, CollisionResult) {
    let largest_movement = movement.abs().max_element();

    let step_count = (largest_movement / MAX_MOVEMENT_STEP).ceil().max(1.0) as usize;

    let movement_step = movement / step_count as f32;

    let mut position = position;

    let mut result = CollisionResult::default();

    for _ in 0..step_count {
        let mut stepped_this_iteration = false;

        let (new_position, blocked, stepped) =
            resolve_x(world, position, movement_step.x, allow_step);

        position = new_position;

        if blocked {
            result.blocked_x = true;
        }

        if stepped {
            stepped_this_iteration = true;
            result.step_height += AUTO_STEP_HEIGHT;
        }

        let (new_position, blocked, stepped) = resolve_z(
            world,
            position,
            movement_step.z,
            allow_step && !stepped_this_iteration,
        );

        position = new_position;

        if blocked {
            result.blocked_z = true;
        }

        if stepped {
            result.step_height += AUTO_STEP_HEIGHT;
        }

        let (new_position, blocked, grounded) = resolve_y(world, position, movement_step.y);

        position = new_position;

        if blocked {
            result.blocked_y = true;
        }

        if grounded {
            result.grounded = true;
        }
    }

    if is_grounded(world, position) {
        result.grounded = true;
    }

    (position, result)
}

fn resolve_x(
    world: &VoxelWorld,
    position: Vec3,
    movement: f32,
    allow_step: bool,
) -> (Vec3, bool, bool) {
    if movement == 0.0 {
        return (position, false, false);
    }

    let mut candidate = position;

    candidate.x += movement;

    let voxels = overlapping_solid_voxels(world, candidate);

    if voxels.is_empty() {
        return (candidate, false, false);
    }

    if allow_step {
        let mut stepped_position = position;

        stepped_position.y += AUTO_STEP_HEIGHT;

        if !collides_at(world, stepped_position) {
            stepped_position.x += movement;

            if !collides_at(world, stepped_position) {
                return (stepped_position, false, true);
            }
        }
    }

    let half_width = PLAYER_WIDTH * 0.5;

    for voxel in voxels {
        let voxel_min_x = voxel.x as f32 * VOXEL_SIZE;

        let voxel_max_x = voxel_min_x + VOXEL_SIZE;

        if movement > 0.0 {
            candidate.x = candidate
                .x
                .min(voxel_min_x - half_width - COLLISION_EPSILON);
        } else {
            candidate.x = candidate
                .x
                .max(voxel_max_x + half_width + COLLISION_EPSILON);
        }
    }

    (candidate, true, false)
}

fn resolve_z(
    world: &VoxelWorld,
    position: Vec3,
    movement: f32,
    allow_step: bool,
) -> (Vec3, bool, bool) {
    if movement == 0.0 {
        return (position, false, false);
    }

    let mut candidate = position;

    candidate.z += movement;

    let voxels = overlapping_solid_voxels(world, candidate);

    if voxels.is_empty() {
        return (candidate, false, false);
    }

    if allow_step {
        let mut stepped_position = position;

        stepped_position.y += AUTO_STEP_HEIGHT;

        if !collides_at(world, stepped_position) {
            stepped_position.z += movement;

            if !collides_at(world, stepped_position) {
                return (stepped_position, false, true);
            }
        }
    }

    let half_width = PLAYER_WIDTH * 0.5;

    for voxel in voxels {
        let voxel_min_z = voxel.z as f32 * VOXEL_SIZE;

        let voxel_max_z = voxel_min_z + VOXEL_SIZE;

        if movement > 0.0 {
            candidate.z = candidate
                .z
                .min(voxel_min_z - half_width - COLLISION_EPSILON);
        } else {
            candidate.z = candidate
                .z
                .max(voxel_max_z + half_width + COLLISION_EPSILON);
        }
    }

    (candidate, true, false)
}

fn resolve_y(world: &VoxelWorld, position: Vec3, movement: f32) -> (Vec3, bool, bool) {
    if movement == 0.0 {
        return (position, false, false);
    }

    let mut candidate = position;

    candidate.y += movement;

    let voxels = overlapping_solid_voxels(world, candidate);

    if voxels.is_empty() {
        return (candidate, false, false);
    }

    for voxel in voxels {
        let voxel_min_y = voxel.y as f32 * VOXEL_SIZE;

        let voxel_max_y = voxel_min_y + VOXEL_SIZE;

        if movement > 0.0 {
            candidate.y = candidate
                .y
                .min(voxel_min_y - PLAYER_HEIGHT - COLLISION_EPSILON);
        } else {
            candidate.y = candidate.y.max(voxel_max_y + COLLISION_EPSILON);
        }
    }

    (candidate, true, movement < 0.0)
}

fn collides_at(world: &VoxelWorld, position: Vec3) -> bool {
    let (min_voxel, max_voxel) = body_voxel_bounds(position);

    for y in min_voxel.y..=max_voxel.y {
        for z in min_voxel.z..=max_voxel.z {
            for x in min_voxel.x..=max_voxel.x {
                if world
                    .get_voxel(IVec3::new(x, y, z))
                    .is_some_and(Voxel::is_collidable)
                {
                    return true;
                }
            }
        }
    }

    false
}

fn overlapping_solid_voxels(world: &VoxelWorld, position: Vec3) -> Vec<IVec3> {
    let (min_voxel, max_voxel) = body_voxel_bounds(position);

    let mut voxels = Vec::new();

    for y in min_voxel.y..=max_voxel.y {
        for z in min_voxel.z..=max_voxel.z {
            for x in min_voxel.x..=max_voxel.x {
                let coordinate = IVec3::new(x, y, z);

                if world
                    .get_voxel(coordinate)
                    .is_some_and(Voxel::is_collidable)
                {
                    voxels.push(coordinate);
                }
            }
        }
    }

    voxels
}

fn body_voxel_bounds(position: Vec3) -> (IVec3, IVec3) {
    let half_width = PLAYER_WIDTH * 0.5;

    let min = Vec3::new(position.x - half_width, position.y, position.z - half_width);

    let max = Vec3::new(
        position.x + half_width,
        position.y + PLAYER_HEIGHT,
        position.z + half_width,
    );

    let min_voxel = IVec3::new(
        (min.x / VOXEL_SIZE).floor() as i32,
        (min.y / VOXEL_SIZE).floor() as i32,
        (min.z / VOXEL_SIZE).floor() as i32,
    );

    let max_voxel = IVec3::new(
        ((max.x - COLLISION_EPSILON) / VOXEL_SIZE).floor() as i32,
        ((max.y - COLLISION_EPSILON) / VOXEL_SIZE).floor() as i32,
        ((max.z - COLLISION_EPSILON) / VOXEL_SIZE).floor() as i32,
    );

    (min_voxel, max_voxel)
}
