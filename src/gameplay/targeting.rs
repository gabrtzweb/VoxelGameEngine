use bevy::prelude::*;

use crate::{
    player::PlayerCamera,
    world::{BlockShape, ChunkHomogeneity, VOXEL_SIZE, Voxel, VoxelAccess, VoxelWorld},
};

const MAX_TARGET_DISTANCE: f32 = 10.0;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetingSet {
    UpdateTarget,
}

#[derive(Clone, Copy, Debug)]
pub struct VoxelTarget {
    pub hit_voxel: IVec3,
    pub place_voxel: Option<IVec3>,
    #[allow(dead_code)]
    pub face_normal: IVec3,
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
    camera: Single<&GlobalTransform, (With<Camera3d>, With<PlayerCamera>)>,
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

    if world
        .get_voxel(camera_voxel)
        .is_some_and(Voxel::is_collidable)
    {
        current_target.hit = None;
        return;
    }

    let is_submerged = world.get_voxel(camera_voxel).is_some_and(Voxel::is_water);

    current_target.hit = raycast_world(
        &world,
        ray_origin,
        ray_direction,
        MAX_TARGET_DISTANCE,
        is_submerged,
    )
    .map(|hit| {
        let place_voxel = if hit.face_normal == IVec3::ZERO {
            None
        } else {
            Some(hit.voxel + hit.face_normal)
        };

        VoxelTarget {
            hit_voxel: hit.voxel,
            place_voxel,
            face_normal: hit.face_normal,
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

    let Some(voxel) = world.get_voxel(target.hit_voxel) else {
        return;
    };

    if voxel.is_empty() {
        return;
    }

    let (shape, orientation) = world.get_shape(target.block_origin);
    let origin = target.block_origin.as_vec3() * VOXEL_SIZE;
    let color = Color::srgba(1.0, 1.0, 1.0, 0.95);

    let (box_a, maybe_box_b) = shape.local_boxes(orientation);
    draw_box_outline(&mut gizmos, origin, box_a[0], box_a[1], color);
    if let Some(box_b) = maybe_box_b {
        draw_box_outline(&mut gizmos, origin, box_b[0], box_b[1], color);
    }
}

fn draw_box_outline(
    gizmos: &mut Gizmos,
    block_origin: Vec3,
    local_min: Vec3,
    local_max: Vec3,
    color: Color,
) {
    let world_min = block_origin + local_min * VOXEL_SIZE;
    let world_max = block_origin + local_max * VOXEL_SIZE;
    let center = (world_min + world_max) * 0.5;
    let size = world_max - world_min;

    gizmos.cube(
        Transform::from_translation(center).with_scale(size),
        color,
    );
}

pub fn block_origin_from_voxel(voxel: IVec3) -> IVec3 {
    voxel
}

#[allow(dead_code)]
pub fn adjacent_block_origin(_block_origin: IVec3, hit_voxel: IVec3, face_normal: IVec3) -> IVec3 {
    hit_voxel + face_normal
}

fn raycast_world(
    world: &VoxelWorld,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    ignore_water: bool,
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

    let mut cached_chunk_coord = IVec3::new(i32::MIN, i32::MIN, i32::MIN);
    let mut cached_chunk_homogeneity = ChunkHomogeneity::Mixed;

    while traveled_distance <= max_grid_distance {
        let (chunk_coord, _) = VoxelWorld::world_voxel_to_chunk(voxel);
        if chunk_coord != cached_chunk_coord {
            cached_chunk_coord = chunk_coord;
            cached_chunk_homogeneity = world
                .get_chunk(chunk_coord)
                .map_or(ChunkHomogeneity::Empty, |c| c.homogeneity());
        }

        match cached_chunk_homogeneity {
            ChunkHomogeneity::Empty => {
                // Chunk is 100% air; skip individual voxel lookups entirely
            }
            ChunkHomogeneity::Solid(solid_mat) => {
                if !ignore_water || !solid_mat.is_water() {
                    return Some(RaycastHit { voxel, face_normal });
                }
            }
            ChunkHomogeneity::Mixed => {
                if let Some(current_voxel) = world.get_voxel(voxel)
                    && current_voxel != Voxel::Air
                    && (!ignore_water || !current_voxel.is_water())
                {
                    let (shape, orientation) = world.get_shape(voxel);
                    if shape == BlockShape::Full {
                        return Some(RaycastHit { voxel, face_normal });
                    }

                    let (box_a, maybe_box_b) = shape.local_boxes(orientation);
                    let mut best =
                        ray_hit_local_box(grid_origin, direction, voxel, box_a[0], box_a[1]);
                    if let Some(box_b) = maybe_box_b {
                        if let Some(hit_b) =
                            ray_hit_local_box(grid_origin, direction, voxel, box_b[0], box_b[1])
                        {
                            best = match best {
                                Some(hit_a) if hit_a.0 <= hit_b.0 => Some(hit_a),
                                _ => Some(hit_b),
                            };
                        }
                    }

                    if let Some((_, hit_normal)) = best {
                        return Some(RaycastHit {
                            voxel,
                            face_normal: hit_normal,
                        });
                    }
                }
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

fn ray_hit_local_box(
    origin: Vec3,
    direction: Vec3,
    voxel: IVec3,
    local_min: Vec3,
    local_max: Vec3,
) -> Option<(f32, IVec3)> {
    let box_min = voxel.as_vec3() + local_min;
    let box_max = voxel.as_vec3() + local_max;

    let inv_x = if direction.x.abs() > 1e-6 {
        1.0 / direction.x
    } else {
        f32::INFINITY
    };
    let inv_y = if direction.y.abs() > 1e-6 {
        1.0 / direction.y
    } else {
        f32::INFINITY
    };
    let inv_z = if direction.z.abs() > 1e-6 {
        1.0 / direction.z
    } else {
        f32::INFINITY
    };

    let mut t1_x = (box_min.x - origin.x) * inv_x;
    let mut t2_x = (box_max.x - origin.x) * inv_x;
    let mut norm_x = if direction.x < 0.0 {
        IVec3::X
    } else {
        IVec3::NEG_X
    };
    if t1_x > t2_x {
        std::mem::swap(&mut t1_x, &mut t2_x);
        norm_x = -norm_x;
    }

    let mut t1_y = (box_min.y - origin.y) * inv_y;
    let mut t2_y = (box_max.y - origin.y) * inv_y;
    let mut norm_y = if direction.y < 0.0 {
        IVec3::Y
    } else {
        IVec3::NEG_Y
    };
    if t1_y > t2_y {
        std::mem::swap(&mut t1_y, &mut t2_y);
        norm_y = -norm_y;
    }

    if (t1_x > t2_y) || (t1_y > t2_x) {
        return None;
    }

    let mut t_enter = t1_x.max(t1_y);
    let mut hit_normal = if t1_y > t1_x { norm_y } else { norm_x };

    let mut t1_z = (box_min.z - origin.z) * inv_z;
    let mut t2_z = (box_max.z - origin.z) * inv_z;
    let mut norm_z = if direction.z < 0.0 {
        IVec3::Z
    } else {
        IVec3::NEG_Z
    };
    if t1_z > t2_z {
        std::mem::swap(&mut t1_z, &mut t2_z);
        norm_z = -norm_z;
    }

    if (t_enter > t2_z) || (t1_z > t2_x.min(t2_y)) {
        return None;
    }

    if t1_z > t_enter {
        t_enter = t1_z;
        hit_normal = norm_z;
    }

    let t_exit = t2_x.min(t2_y).min(t2_z);

    if t_enter <= t_exit && t_exit >= 0.0 {
        Some((t_enter.max(0.0), hit_normal))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::IVec3;

    #[test]
    fn block_origin_maps_directly_to_voxel() {
        assert_eq!(
            block_origin_from_voxel(IVec3::new(3, 2, 1)),
            IVec3::new(3, 2, 1)
        );
        assert_eq!(
            block_origin_from_voxel(IVec3::new(-1, -2, -3)),
            IVec3::new(-1, -2, -3)
        );
    }

    #[test]
    fn adjacent_block_origin_moves_one_block_along_the_hit_face() {
        let origin = IVec3::new(-2, 4, 6);

        assert_eq!(
            adjacent_block_origin(origin, IVec3::new(-2, 4, 6), IVec3::X),
            IVec3::new(-1, 4, 6)
        );
        assert_eq!(
            adjacent_block_origin(origin, IVec3::new(-2, 4, 6), -IVec3::Z),
            IVec3::new(-2, 4, 5)
        );
        assert_eq!(
            adjacent_block_origin(origin, IVec3::new(-2, 4, 6), IVec3::Y),
            IVec3::new(-2, 5, 6)
        );
    }
}
