use bevy::prelude::*;

use super::{
    chunk::{VOXEL_SIZE, Voxel},
    world::VoxelWorld,
};

const MAX_TARGET_DISTANCE: f32 = 10.0;
const VOXELS_PER_BLOCK: i32 = 2;
const BLOCK_SIZE: f32 = VOXEL_SIZE * VOXELS_PER_BLOCK as f32;

const NEIGHBOR_DIRECTIONS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetingSet {
    UpdateTarget,
}

#[derive(Clone, Copy, Debug)]
pub struct VoxelTarget {
    pub hit_voxel: IVec3,
    pub place_voxel: Option<IVec3>,
    pub block_origin: IVec3,
}

#[derive(Resource, Default)]
pub struct CurrentTarget {
    pub hit: Option<VoxelTarget>,
}

pub struct TargetingPlugin;

impl Plugin for TargetingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentTarget>().add_systems(
            Update,
            (
                update_current_target.in_set(TargetingSet::UpdateTarget),
                draw_current_target_highlight.after(TargetingSet::UpdateTarget),
            ),
        );
    }
}

struct RaycastHit {
    voxel: IVec3,
    previous: Option<IVec3>,
}

fn update_current_target(
    camera: Single<&GlobalTransform, With<Camera3d>>,
    world: Res<VoxelWorld>,
    mut current_target: ResMut<CurrentTarget>,
) {
    let ray_origin = camera.translation();
    let ray_direction = *camera.forward();

    current_target.hit =
        raycast_world(&world, ray_origin, ray_direction, MAX_TARGET_DISTANCE).map(|hit| {
            VoxelTarget {
                hit_voxel: hit.voxel,
                place_voxel: hit.previous,
                block_origin: block_origin_from_voxel(hit.voxel),
            }
        });
}

fn draw_current_target_highlight(
    world: Res<VoxelWorld>,
    current_target: Res<CurrentTarget>,
    mut gizmos: Gizmos,
) {
    let Some(target) = current_target.hit else {
        return;
    };

    let solid_count = count_solid_voxels(&world, target.block_origin);

    if solid_count == 0 {
        return;
    }

    let total_voxels = (VOXELS_PER_BLOCK * VOXELS_PER_BLOCK * VOXELS_PER_BLOCK) as usize;

    if solid_count == total_voxels {
        draw_full_block_highlight(&mut gizmos, target.block_origin);
    } else {
        draw_partial_block_highlight(&world, &mut gizmos, target.block_origin);
    }
}

fn draw_full_block_highlight(gizmos: &mut Gizmos, block_origin: IVec3) {
    let center = (block_origin.as_vec3() + Vec3::splat(VOXELS_PER_BLOCK as f32 * 0.5)) * VOXEL_SIZE;

    gizmos.cube(
        Transform::from_translation(center).with_scale(Vec3::splat(BLOCK_SIZE)),
        Color::srgba(1.0, 1.0, 1.0, 0.95),
    );
}

fn draw_partial_block_highlight(world: &VoxelWorld, gizmos: &mut Gizmos, block_origin: IVec3) {
    for y in 0..VOXELS_PER_BLOCK {
        for z in 0..VOXELS_PER_BLOCK {
            for x in 0..VOXELS_PER_BLOCK {
                let voxel_position = block_origin + IVec3::new(x, y, z);

                if world.get_voxel(voxel_position) != Some(Voxel::Solid) {
                    continue;
                }

                if !voxel_is_exposed(world, voxel_position) {
                    continue;
                }

                let center = (voxel_position.as_vec3() + Vec3::splat(0.5)) * VOXEL_SIZE;

                gizmos.cube(
                    Transform::from_translation(center).with_scale(Vec3::splat(VOXEL_SIZE)),
                    Color::srgba(1.0, 1.0, 1.0, 0.95),
                );
            }
        }
    }
}

fn count_solid_voxels(world: &VoxelWorld, block_origin: IVec3) -> usize {
    let mut count = 0;

    for y in 0..VOXELS_PER_BLOCK {
        for z in 0..VOXELS_PER_BLOCK {
            for x in 0..VOXELS_PER_BLOCK {
                let voxel_position = block_origin + IVec3::new(x, y, z);

                if world.get_voxel(voxel_position) == Some(Voxel::Solid) {
                    count += 1;
                }
            }
        }
    }

    count
}

fn voxel_is_exposed(world: &VoxelWorld, position: IVec3) -> bool {
    for direction in NEIGHBOR_DIRECTIONS {
        let neighbor = position + direction;

        if world.get_voxel(neighbor) != Some(Voxel::Solid) {
            return true;
        }
    }

    false
}

fn block_origin_from_voxel(voxel: IVec3) -> IVec3 {
    IVec3::new(
        voxel.x.div_euclid(VOXELS_PER_BLOCK) * VOXELS_PER_BLOCK,
        voxel.y.div_euclid(VOXELS_PER_BLOCK) * VOXELS_PER_BLOCK,
        voxel.z.div_euclid(VOXELS_PER_BLOCK) * VOXELS_PER_BLOCK,
    )
}

fn raycast_world(
    world: &VoxelWorld,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<RaycastHit> {
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
                return Some(RaycastHit { voxel, previous });
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
