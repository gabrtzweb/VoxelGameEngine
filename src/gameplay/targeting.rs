use bevy::prelude::*;

use crate::world::{ChunkHomogeneity, VOXEL_SIZE, Voxel, VoxelWorld};

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

    draw_voxel_outline(
        &mut gizmos,
        target.hit_voxel,
        Color::srgba(1.0, 1.0, 1.0, 0.95),
    );
}

fn draw_voxel_outline(gizmos: &mut Gizmos, voxel: IVec3, color: Color) {
    let center = (voxel.as_vec3() + Vec3::splat(0.5)) * VOXEL_SIZE;

    gizmos.cube(
        Transform::from_translation(center).with_scale(Vec3::splat(VOXEL_SIZE)),
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
                    return Some(RaycastHit { voxel, face_normal });
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
