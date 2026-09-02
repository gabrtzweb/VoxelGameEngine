use bevy::prelude::*;

use super::{
    chunk::{VOXEL_SIZE, Voxel},
    world::VoxelWorld,
};

const MAX_TARGET_DISTANCE: f32 = 10.0;
const VOXELS_PER_BLOCK: i32 = 2;

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
    face_normal: IVec3,
}

fn update_current_target(
    camera: Single<&GlobalTransform, With<Camera3d>>,
    world: Res<VoxelWorld>,
    mut current_target: ResMut<CurrentTarget>,
) {
    let ray_origin = camera.translation();
    let ray_direction = *camera.forward();

    let camera_voxel = IVec3::new(
        (ray_origin.x / VOXEL_SIZE).floor() as i32,
        (ray_origin.y / VOXEL_SIZE).floor() as i32,
        (ray_origin.z / VOXEL_SIZE).floor() as i32,
    );

    if world.get_voxel(camera_voxel) == Some(Voxel::Solid) {
        current_target.hit = None;
        return;
    }

    current_target.hit =
        raycast_world(&world, ray_origin, ray_direction, MAX_TARGET_DISTANCE).map(|hit| {
            let place_voxel = if hit.face_normal == IVec3::ZERO {
                None
            } else {
                Some(hit.voxel + hit.face_normal)
            };

            VoxelTarget {
                hit_voxel: hit.voxel,
                place_voxel,
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

    if count_solid_voxels(&world, target.block_origin) == 0 {
        return;
    }

    draw_block_shape_outline(
        &world,
        &mut gizmos,
        target.block_origin,
        Color::srgba(1.0, 1.0, 1.0, 0.95),
    );
}

fn draw_block_shape_outline(
    world: &VoxelWorld,
    gizmos: &mut Gizmos,
    block_origin: IVec3,
    color: Color,
) {
    draw_x_edges(world, gizmos, block_origin, color);

    draw_y_edges(world, gizmos, block_origin, color);

    draw_z_edges(world, gizmos, block_origin, color);
}

fn draw_x_edges(world: &VoxelWorld, gizmos: &mut Gizmos, block_origin: IVec3, color: Color) {
    for x in 0..VOXELS_PER_BLOCK {
        for y in 0..=VOXELS_PER_BLOCK {
            for z in 0..=VOXELS_PER_BLOCK {
                let quadrants = [
                    is_solid_local(world, block_origin, IVec3::new(x, y - 1, z - 1)),
                    is_solid_local(world, block_origin, IVec3::new(x, y, z - 1)),
                    is_solid_local(world, block_origin, IVec3::new(x, y - 1, z)),
                    is_solid_local(world, block_origin, IVec3::new(x, y, z)),
                ];

                if !should_draw_edge(quadrants) {
                    continue;
                }

                let start = voxel_grid_point(block_origin, x, y, z);

                let end = voxel_grid_point(block_origin, x + 1, y, z);

                gizmos.line(start, end, color);
            }
        }
    }
}

fn draw_y_edges(world: &VoxelWorld, gizmos: &mut Gizmos, block_origin: IVec3, color: Color) {
    for y in 0..VOXELS_PER_BLOCK {
        for x in 0..=VOXELS_PER_BLOCK {
            for z in 0..=VOXELS_PER_BLOCK {
                let quadrants = [
                    is_solid_local(world, block_origin, IVec3::new(x - 1, y, z - 1)),
                    is_solid_local(world, block_origin, IVec3::new(x, y, z - 1)),
                    is_solid_local(world, block_origin, IVec3::new(x - 1, y, z)),
                    is_solid_local(world, block_origin, IVec3::new(x, y, z)),
                ];

                if !should_draw_edge(quadrants) {
                    continue;
                }

                let start = voxel_grid_point(block_origin, x, y, z);

                let end = voxel_grid_point(block_origin, x, y + 1, z);

                gizmos.line(start, end, color);
            }
        }
    }
}

fn draw_z_edges(world: &VoxelWorld, gizmos: &mut Gizmos, block_origin: IVec3, color: Color) {
    for z in 0..VOXELS_PER_BLOCK {
        for x in 0..=VOXELS_PER_BLOCK {
            for y in 0..=VOXELS_PER_BLOCK {
                let quadrants = [
                    is_solid_local(world, block_origin, IVec3::new(x - 1, y - 1, z)),
                    is_solid_local(world, block_origin, IVec3::new(x, y - 1, z)),
                    is_solid_local(world, block_origin, IVec3::new(x - 1, y, z)),
                    is_solid_local(world, block_origin, IVec3::new(x, y, z)),
                ];

                if !should_draw_edge(quadrants) {
                    continue;
                }

                let start = voxel_grid_point(block_origin, x, y, z);

                let end = voxel_grid_point(block_origin, x, y, z + 1);

                gizmos.line(start, end, color);
            }
        }
    }
}

fn should_draw_edge(quadrants: [bool; 4]) -> bool {
    let solid_count = quadrants.iter().filter(|&&solid| solid).count();

    match solid_count {
        0 | 4 => false,

        1 | 3 => true,

        2 => {
            let diagonal_a = quadrants[0] && quadrants[3];

            let diagonal_b = quadrants[1] && quadrants[2];

            diagonal_a || diagonal_b
        }

        _ => false,
    }
}

fn is_solid_local(world: &VoxelWorld, block_origin: IVec3, local_position: IVec3) -> bool {
    if local_position.x < 0
        || local_position.x >= VOXELS_PER_BLOCK
        || local_position.y < 0
        || local_position.y >= VOXELS_PER_BLOCK
        || local_position.z < 0
        || local_position.z >= VOXELS_PER_BLOCK
    {
        return false;
    }

    world.get_voxel(block_origin + local_position) == Some(Voxel::Solid)
}

fn voxel_grid_point(block_origin: IVec3, x: i32, y: i32, z: i32) -> Vec3 {
    (block_origin.as_vec3() + Vec3::new(x as f32, y as f32, z as f32)) * VOXEL_SIZE
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
    let mut face_normal = IVec3::ZERO;

    while traveled_distance <= max_grid_distance {
        if let Some(current_voxel) = world.get_voxel(voxel) {
            if current_voxel != Voxel::Air {
                return Some(RaycastHit { voxel, face_normal });
            }
        }

        if side_distance.x <= side_distance.y && side_distance.x <= side_distance.z {
            voxel.x += step.x;

            traveled_distance = side_distance.x;

            side_distance.x += delta_distance.x;

            face_normal = IVec3::new(-step.x, 0, 0);
        } else if side_distance.y <= side_distance.z {
            voxel.y += step.y;

            traveled_distance = side_distance.y;

            side_distance.y += delta_distance.y;

            face_normal = IVec3::new(0, -step.y, 0);
        } else {
            voxel.z += step.z;

            traveled_distance = side_distance.z;

            side_distance.z += delta_distance.z;

            face_normal = IVec3::new(0, 0, -step.z);
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
