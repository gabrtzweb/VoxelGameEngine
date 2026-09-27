use bevy::prelude::*;

use crate::world::{BlockShape, ChunkHomogeneity, VOXEL_SIZE, Voxel, VoxelWorld};

use super::PLAYER_WIDTH;

const COLLISION_EPSILON: f32 = 0.001;

const MAX_MOVEMENT_STEP: f32 = 0.45;

const GROUND_PROBE_DISTANCE: f32 = 0.025;

pub(super) const AUTO_STEP_HEIGHT: f32 = 0.55;

#[derive(Default)]
pub struct CollisionResult {
    pub grounded: bool,
    pub blocked_x: bool,
    pub blocked_y: bool,
    pub blocked_z: bool,
    pub step_height: f32,
}

pub fn is_grounded(world: &VoxelWorld, position: Vec3, height: f32) -> bool {
    let probe_position = position - Vec3::Y * GROUND_PROBE_DISTANCE;

    collides_at(world, probe_position, height)
}

pub fn has_headroom(world: &VoxelWorld, position: Vec3, target_height: f32) -> bool {
    !collides_at(world, position, target_height)
}

pub fn move_with_collisions(
    world: &VoxelWorld,
    position: Vec3,
    movement: Vec3,
    allow_step: bool,
    height: f32,
) -> (Vec3, CollisionResult) {
    let largest_movement = movement.abs().max_element();

    let step_count = (largest_movement / MAX_MOVEMENT_STEP).ceil().max(1.0) as usize;

    let movement_step = movement / step_count as f32;

    let mut position = position;

    let mut result = CollisionResult::default();

    for _ in 0..step_count {
        let mut stepped_this_iteration = false;

        let (new_position, blocked, stepped) =
            resolve_x(world, position, movement_step.x, allow_step, height);

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
            height,
        );

        position = new_position;

        if blocked {
            result.blocked_z = true;
        }

        if stepped {
            result.step_height += AUTO_STEP_HEIGHT;
        }

        let (new_position, blocked, grounded) = resolve_y(world, position, movement_step.y, height);

        position = new_position;

        if blocked {
            result.blocked_y = true;
        }

        if grounded {
            result.grounded = true;
        }
    }

    if is_grounded(world, position, height) {
        result.grounded = true;
    }

    (position, result)
}

#[inline]
fn aabb_overlap(min_a: Vec3, max_a: Vec3, min_b: Vec3, max_b: Vec3) -> bool {
    min_a.x < max_b.x - COLLISION_EPSILON
        && max_a.x > min_b.x + COLLISION_EPSILON
        && min_a.y < max_b.y - COLLISION_EPSILON
        && max_a.y > min_b.y + COLLISION_EPSILON
        && min_a.z < max_b.z - COLLISION_EPSILON
        && max_a.z > min_b.z + COLLISION_EPSILON
}

#[inline]
fn for_each_voxel_box<F: FnMut(Vec3, Vec3)>(
    world: &VoxelWorld,
    voxel: IVec3,
    mut f: F,
) {
    let (shape, orientation) = world.get_shape(voxel);
    let v_pos = voxel.as_vec3() * VOXEL_SIZE;
    let (box_a, maybe_box_b) = shape.local_boxes(orientation);
    f(v_pos + box_a[0] * VOXEL_SIZE, v_pos + box_a[1] * VOXEL_SIZE);
    if let Some(box_b) = maybe_box_b {
        f(v_pos + box_b[0] * VOXEL_SIZE, v_pos + box_b[1] * VOXEL_SIZE);
    }
    if let Some((_extra_v, extra_orient)) = world.get_extra_slab(voxel) {
        let (extra_box, _) = BlockShape::Slab.local_boxes(extra_orient);
        f(v_pos + extra_box[0] * VOXEL_SIZE, v_pos + extra_box[1] * VOXEL_SIZE);
    }
}

fn resolve_x(
    world: &VoxelWorld,
    position: Vec3,
    movement: f32,
    allow_step: bool,
    height: f32,
) -> (Vec3, bool, bool) {
    if movement == 0.0 {
        return (position, false, false);
    }

    let mut candidate = position;
    candidate.x += movement;

    if !collides_at(world, candidate, height) {
        return (candidate, false, false);
    }

    if allow_step {
        let mut stepped_position = position;
        stepped_position.y += AUTO_STEP_HEIGHT;

        if !collides_at(world, stepped_position, height) {
            stepped_position.x += movement;

            if !collides_at(world, stepped_position, height) {
                return (stepped_position, false, true);
            }
        }
    }

    let voxels = overlapping_solid_voxels(world, candidate, height);
    let half_width = PLAYER_WIDTH * 0.5;
    let mut did_collide = false;

    for voxel in &voxels {
        for_each_voxel_box(world, *voxel, |box_min, box_max| {
            let p_min_y = candidate.y;
            let p_max_y = candidate.y + height;
            let p_min_z = candidate.z - half_width;
            let p_max_z = candidate.z + half_width;

            if p_min_y < box_max.y - COLLISION_EPSILON
                && p_max_y > box_min.y + COLLISION_EPSILON
                && p_min_z < box_max.z - COLLISION_EPSILON
                && p_max_z > box_min.z + COLLISION_EPSILON
            {
                let p_min_x = candidate.x - half_width;
                let p_max_x = candidate.x + half_width;

                if p_min_x < box_max.x - COLLISION_EPSILON && p_max_x > box_min.x + COLLISION_EPSILON {
                    did_collide = true;
                    if movement > 0.0 {
                        candidate.x = candidate.x.min(box_min.x - half_width - COLLISION_EPSILON);
                    } else {
                        candidate.x = candidate.x.max(box_max.x + half_width + COLLISION_EPSILON);
                    }
                }
            }
        });
    }

    (candidate, did_collide, false)
}

fn resolve_z(
    world: &VoxelWorld,
    position: Vec3,
    movement: f32,
    allow_step: bool,
    height: f32,
) -> (Vec3, bool, bool) {
    if movement == 0.0 {
        return (position, false, false);
    }

    let mut candidate = position;
    candidate.z += movement;

    if !collides_at(world, candidate, height) {
        return (candidate, false, false);
    }

    if allow_step {
        let mut stepped_position = position;
        stepped_position.y += AUTO_STEP_HEIGHT;

        if !collides_at(world, stepped_position, height) {
            stepped_position.z += movement;

            if !collides_at(world, stepped_position, height) {
                return (stepped_position, false, true);
            }
        }
    }

    let voxels = overlapping_solid_voxels(world, candidate, height);
    let half_width = PLAYER_WIDTH * 0.5;
    let mut did_collide = false;

    for voxel in &voxels {
        for_each_voxel_box(world, *voxel, |box_min, box_max| {
            let p_min_x = candidate.x - half_width;
            let p_max_x = candidate.x + half_width;
            let p_min_y = candidate.y;
            let p_max_y = candidate.y + height;

            if p_min_x < box_max.x - COLLISION_EPSILON
                && p_max_x > box_min.x + COLLISION_EPSILON
                && p_min_y < box_max.y - COLLISION_EPSILON
                && p_max_y > box_min.y + COLLISION_EPSILON
            {
                let p_min_z = candidate.z - half_width;
                let p_max_z = candidate.z + half_width;

                if p_min_z < box_max.z - COLLISION_EPSILON && p_max_z > box_min.z + COLLISION_EPSILON {
                    did_collide = true;
                    if movement > 0.0 {
                        candidate.z = candidate.z.min(box_min.z - half_width - COLLISION_EPSILON);
                    } else {
                        candidate.z = candidate.z.max(box_max.z + half_width + COLLISION_EPSILON);
                    }
                }
            }
        });
    }

    (candidate, did_collide, false)
}

fn resolve_y(world: &VoxelWorld, position: Vec3, movement: f32, height: f32) -> (Vec3, bool, bool) {
    if movement == 0.0 {
        return (position, false, false);
    }

    let mut candidate = position;
    candidate.y += movement;

    let voxels = overlapping_solid_voxels(world, candidate, height);
    if voxels.is_empty() {
        return (candidate, false, false);
    }

    let half_width = PLAYER_WIDTH * 0.5;
    let mut did_collide = false;

    for voxel in &voxels {
        for_each_voxel_box(world, *voxel, |box_min, box_max| {
            let p_min_x = candidate.x - half_width;
            let p_max_x = candidate.x + half_width;
            let p_min_z = candidate.z - half_width;
            let p_max_z = candidate.z + half_width;

            if p_min_x < box_max.x - COLLISION_EPSILON
                && p_max_x > box_min.x + COLLISION_EPSILON
                && p_min_z < box_max.z - COLLISION_EPSILON
                && p_max_z > box_min.z + COLLISION_EPSILON
            {
                let p_min_y = candidate.y;
                let p_max_y = candidate.y + height;

                if p_min_y < box_max.y - COLLISION_EPSILON && p_max_y > box_min.y + COLLISION_EPSILON {
                    did_collide = true;
                    if movement > 0.0 {
                        candidate.y = candidate.y.min(box_min.y - height - COLLISION_EPSILON);
                    } else {
                        candidate.y = candidate.y.max(box_max.y + COLLISION_EPSILON);
                    }
                }
            }
        });
    }

    (candidate, did_collide, did_collide && movement < 0.0)
}

pub fn collides_at(world: &VoxelWorld, position: Vec3, height: f32) -> bool {
    let (min_voxel, max_voxel) = body_voxel_bounds(position, height);
    let half_width = PLAYER_WIDTH * 0.5;
    let p_min = Vec3::new(position.x - half_width, position.y, position.z - half_width);
    let p_max = Vec3::new(position.x + half_width, position.y + height, position.z + half_width);

    for y in min_voxel.y..=max_voxel.y {
        for z in min_voxel.z..=max_voxel.z {
            for x in min_voxel.x..=max_voxel.x {
                let coord = IVec3::new(x, y, z);
                if world
                    .get_voxel(coord)
                    .is_some_and(Voxel::is_collidable)
                {
                    let mut collided = false;
                    for_each_voxel_box(world, coord, |box_min, box_max| {
                        if aabb_overlap(p_min, p_max, box_min, box_max) {
                            collided = true;
                        }
                    });
                    if collided {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub const MAX_OVERLAPPING_VOXELS: usize = 64;

#[derive(Clone, Copy)]
pub struct SolidVoxelBuffer {
    voxels: [IVec3; MAX_OVERLAPPING_VOXELS],
    len: usize,
}

impl Default for SolidVoxelBuffer {
    fn default() -> Self {
        Self {
            voxels: [IVec3::ZERO; MAX_OVERLAPPING_VOXELS],
            len: 0,
        }
    }
}

impl SolidVoxelBuffer {
    #[inline]
    pub fn push(&mut self, voxel: IVec3) {
        if self.len < MAX_OVERLAPPING_VOXELS {
            self.voxels[self.len] = voxel;
            self.len += 1;
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn as_slice(&self) -> &[IVec3] {
        &self.voxels[..self.len]
    }
}

impl<'a> IntoIterator for &'a SolidVoxelBuffer {
    type Item = &'a IVec3;
    type IntoIter = std::slice::Iter<'a, IVec3>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

fn overlapping_solid_voxels(world: &VoxelWorld, position: Vec3, height: f32) -> SolidVoxelBuffer {
    let (min_voxel, max_voxel) = body_voxel_bounds(position, height);

    let (min_chunk, _) = VoxelWorld::world_voxel_to_chunk(min_voxel);
    let (max_chunk, _) = VoxelWorld::world_voxel_to_chunk(max_voxel);

    // Fast O(1) path: If the bounding box lies entirely within an Empty chunk (e.g. open sky / airborne),
    // we can immediately return with zero collisions without scanning voxel coordinates.
    if min_chunk == max_chunk {
        if let Some(chunk) = world.get_chunk(min_chunk) {
            if chunk.homogeneity() == ChunkHomogeneity::Empty {
                return SolidVoxelBuffer::default();
            }
        } else {
            return SolidVoxelBuffer::default();
        }
    } else {
        let mut all_empty = true;
        for cy in min_chunk.y..=max_chunk.y {
            for cz in min_chunk.z..=max_chunk.z {
                for cx in min_chunk.x..=max_chunk.x {
                    if let Some(chunk) = world.get_chunk(IVec3::new(cx, cy, cz))
                        && chunk.homogeneity() != ChunkHomogeneity::Empty
                    {
                        all_empty = false;
                        break;
                    }
                }
                if !all_empty {
                    break;
                }
            }
            if !all_empty {
                break;
            }
        }

        if all_empty {
            return SolidVoxelBuffer::default();
        }
    }

    let mut voxels = SolidVoxelBuffer::default();

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

fn body_voxel_bounds(position: Vec3, height: f32) -> (IVec3, IVec3) {
    let half_width = PLAYER_WIDTH * 0.5;

    let min = Vec3::new(position.x - half_width, position.y, position.z - half_width);

    let max = Vec3::new(
        position.x + half_width,
        position.y + height,
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
