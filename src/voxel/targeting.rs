use bevy::prelude::*;

use super::{
    chunk::{VOXEL_SIZE, Voxel},
    interaction_mode::InteractionMode,
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
                face_normal: hit.face_normal,
                block_origin: block_origin_from_voxel(hit.voxel),
            }
        });
}

fn draw_current_target_highlight(
    interaction_mode: Res<InteractionMode>,
    world: Res<VoxelWorld>,
    current_target: Res<CurrentTarget>,
    mut gizmos: Gizmos,
) {
    let Some(target) = current_target.hit else {
        return;
    };

    match *interaction_mode {
        InteractionMode::Block => {
            let voxels = crate::voxel::shaping::get_block_voxels(&world, target.block_origin);
            let layer0_centered = crate::voxel::shaping::is_layer_centered(&voxels[0..4]);
            let layer1_centered = crate::voxel::shaping::is_layer_centered(&voxels[4..8]);

            if layer0_centered && layer1_centered {
                let center =
                    (target.block_origin.as_vec3() + Vec3::new(1.0, 1.0, 1.0)) * VOXEL_SIZE;
                gizmos.cube(
                    Transform::from_translation(center).with_scale(Vec3::new(
                        VOXEL_SIZE,
                        2.0 * VOXEL_SIZE,
                        VOXEL_SIZE,
                    )),
                    Color::srgba(1.0, 1.0, 1.0, 0.95),
                );
            } else {
                if layer0_centered {
                    let center =
                        (target.block_origin.as_vec3() + Vec3::new(1.0, 0.5, 1.0)) * VOXEL_SIZE;
                    gizmos.cube(
                        Transform::from_translation(center).with_scale(Vec3::splat(VOXEL_SIZE)),
                        Color::srgba(1.0, 1.0, 1.0, 0.95),
                    );
                }
                if layer1_centered {
                    let center =
                        (target.block_origin.as_vec3() + Vec3::new(1.0, 1.5, 1.0)) * VOXEL_SIZE;
                    gizmos.cube(
                        Transform::from_translation(center).with_scale(Vec3::splat(VOXEL_SIZE)),
                        Color::srgba(1.0, 1.0, 1.0, 0.95),
                    );
                }
                draw_block_shape_outline(
                    &world,
                    &mut gizmos,
                    target.block_origin,
                    layer0_centered,
                    layer1_centered,
                    Color::srgba(1.0, 1.0, 1.0, 0.95),
                );
            }
        }

        InteractionMode::Voxel => {
            let Some(voxel) = world.get_voxel(target.hit_voxel) else {
                return;
            };

            if voxel.is_empty() {
                return;
            }

            if crate::voxel::shaping::is_centered_layer(&world, target.hit_voxel) {
                let bx = target.hit_voxel.x.div_euclid(2) * 2;
                let bz = target.hit_voxel.z.div_euclid(2) * 2;
                let center = Vec3::new(
                    bx as f32 + 1.0,
                    target.hit_voxel.y as f32 + 0.5,
                    bz as f32 + 1.0,
                ) * VOXEL_SIZE;
                gizmos.cube(
                    Transform::from_translation(center).with_scale(Vec3::splat(VOXEL_SIZE)),
                    Color::srgba(1.0, 1.0, 1.0, 0.95),
                );
            } else {
                draw_voxel_outline(
                    &mut gizmos,
                    target.hit_voxel,
                    Color::srgba(1.0, 1.0, 1.0, 0.95),
                );
            }
        }
    }
}

fn draw_voxel_outline(gizmos: &mut Gizmos, voxel: IVec3, color: Color) {
    let center = (voxel.as_vec3() + Vec3::splat(0.5)) * VOXEL_SIZE;

    gizmos.cube(
        Transform::from_translation(center).with_scale(Vec3::splat(VOXEL_SIZE)),
        color,
    );
}

fn draw_block_shape_outline(
    world: &VoxelWorld,
    gizmos: &mut Gizmos,
    block_origin: IVec3,
    layer0_centered: bool,
    layer1_centered: bool,
    color: Color,
) {
    draw_x_edges(
        world,
        gizmos,
        block_origin,
        layer0_centered,
        layer1_centered,
        color,
    );
    draw_y_edges(
        world,
        gizmos,
        block_origin,
        layer0_centered,
        layer1_centered,
        color,
    );
    draw_z_edges(
        world,
        gizmos,
        block_origin,
        layer0_centered,
        layer1_centered,
        color,
    );
}

fn draw_x_edges(
    world: &VoxelWorld,
    gizmos: &mut Gizmos,
    block_origin: IVec3,
    layer0_centered: bool,
    layer1_centered: bool,
    color: Color,
) {
    for x in 0..VOXELS_PER_BLOCK {
        for y in 0..=VOXELS_PER_BLOCK {
            for z in 0..=VOXELS_PER_BLOCK {
                let quadrants = [
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x, y - 1, z - 1),
                        layer0_centered,
                        layer1_centered,
                    ),
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x, y, z - 1),
                        layer0_centered,
                        layer1_centered,
                    ),
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x, y - 1, z),
                        layer0_centered,
                        layer1_centered,
                    ),
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x, y, z),
                        layer0_centered,
                        layer1_centered,
                    ),
                ];

                if should_draw_edge(quadrants) {
                    let start = voxel_grid_point(block_origin, x, y, z);
                    let end = voxel_grid_point(block_origin, x + 1, y, z);

                    gizmos.line(start, end, color);
                }
            }
        }
    }
}

fn draw_y_edges(
    world: &VoxelWorld,
    gizmos: &mut Gizmos,
    block_origin: IVec3,
    layer0_centered: bool,
    layer1_centered: bool,
    color: Color,
) {
    for y in 0..VOXELS_PER_BLOCK {
        for x in 0..=VOXELS_PER_BLOCK {
            for z in 0..=VOXELS_PER_BLOCK {
                let quadrants = [
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x - 1, y, z - 1),
                        layer0_centered,
                        layer1_centered,
                    ),
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x, y, z - 1),
                        layer0_centered,
                        layer1_centered,
                    ),
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x - 1, y, z),
                        layer0_centered,
                        layer1_centered,
                    ),
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x, y, z),
                        layer0_centered,
                        layer1_centered,
                    ),
                ];

                if should_draw_edge(quadrants) {
                    let start = voxel_grid_point(block_origin, x, y, z);
                    let end = voxel_grid_point(block_origin, x, y + 1, z);

                    gizmos.line(start, end, color);
                }
            }
        }
    }
}

fn draw_z_edges(
    world: &VoxelWorld,
    gizmos: &mut Gizmos,
    block_origin: IVec3,
    layer0_centered: bool,
    layer1_centered: bool,
    color: Color,
) {
    for z in 0..VOXELS_PER_BLOCK {
        for x in 0..=VOXELS_PER_BLOCK {
            for y in 0..=VOXELS_PER_BLOCK {
                let quadrants = [
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x - 1, y - 1, z),
                        layer0_centered,
                        layer1_centered,
                    ),
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x, y - 1, z),
                        layer0_centered,
                        layer1_centered,
                    ),
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x - 1, y, z),
                        layer0_centered,
                        layer1_centered,
                    ),
                    is_solid_local(
                        world,
                        block_origin,
                        IVec3::new(x, y, z),
                        layer0_centered,
                        layer1_centered,
                    ),
                ];

                if should_draw_edge(quadrants) {
                    let start = voxel_grid_point(block_origin, x, y, z);
                    let end = voxel_grid_point(block_origin, x, y, z + 1);

                    gizmos.line(start, end, color);
                }
            }
        }
    }
}

fn should_draw_edge(quadrants: [bool; 4]) -> bool {
    match quadrants.iter().filter(|&&solid| solid).count() {
        0 | 4 => false,
        1 | 3 => true,
        2 => (quadrants[0] && quadrants[3]) || (quadrants[1] && quadrants[2]),
        _ => unreachable!(),
    }
}

fn is_solid_local(
    world: &VoxelWorld,
    block_origin: IVec3,
    local_position: IVec3,
    layer0_centered: bool,
    layer1_centered: bool,
) -> bool {
    if local_position.x < 0
        || local_position.x >= VOXELS_PER_BLOCK
        || local_position.y < 0
        || local_position.y >= VOXELS_PER_BLOCK
        || local_position.z < 0
        || local_position.z >= VOXELS_PER_BLOCK
    {
        return false;
    }

    if (local_position.y == 0 && layer0_centered) || (local_position.y == 1 && layer1_centered) {
        return false;
    }

    world
        .get_voxel(block_origin + local_position)
        .is_some_and(|voxel| !voxel.is_empty() && voxel != Voxel::Occupied)
}

fn voxel_grid_point(block_origin: IVec3, x: i32, y: i32, z: i32) -> Vec3 {
    (block_origin.as_vec3() + Vec3::new(x as f32, y as f32, z as f32)) * VOXEL_SIZE
}

pub fn block_origin_from_voxel(voxel: IVec3) -> IVec3 {
    IVec3::new(
        voxel.x.div_euclid(VOXELS_PER_BLOCK) * VOXELS_PER_BLOCK,
        voxel.y.div_euclid(VOXELS_PER_BLOCK) * VOXELS_PER_BLOCK,
        voxel.z.div_euclid(VOXELS_PER_BLOCK) * VOXELS_PER_BLOCK,
    )
}

pub fn adjacent_block_origin(block_origin: IVec3, hit_voxel: IVec3, face_normal: IVec3) -> IVec3 {
    let mut origin = block_origin;
    if face_normal.x > 0 {
        origin.x = hit_voxel.x + 1;
    } else if face_normal.x < 0 {
        origin.x = hit_voxel.x - VOXELS_PER_BLOCK;
    } else if face_normal.y > 0 {
        origin.y = hit_voxel.y + 1;
    } else if face_normal.y < 0 {
        origin.y = hit_voxel.y - VOXELS_PER_BLOCK;
    } else if face_normal.z > 0 {
        origin.z = hit_voxel.z + 1;
    } else if face_normal.z < 0 {
        origin.z = hit_voxel.z - VOXELS_PER_BLOCK;
    }
    origin
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
        if let Some(current_voxel) = world.get_voxel(voxel)
            && current_voxel != Voxel::Air
        {
            return Some(RaycastHit { voxel, face_normal });
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
    use super::{adjacent_block_origin, block_origin_from_voxel};
    use bevy::prelude::IVec3;

    #[test]
    fn block_origin_aligns_positive_and_negative_voxels_to_the_two_voxel_grid() {
        assert_eq!(
            block_origin_from_voxel(IVec3::new(3, 2, 1)),
            IVec3::new(2, 2, 0)
        );
        assert_eq!(
            block_origin_from_voxel(IVec3::new(-1, -2, -3)),
            IVec3::new(-2, -2, -4)
        );
    }

    #[test]
    fn adjacent_block_origin_moves_one_logical_block_along_the_hit_face() {
        let origin = IVec3::new(-2, 4, 6);

        // Full block hit on +X face at x = -1
        assert_eq!(
            adjacent_block_origin(origin, IVec3::new(-1, 4, 6), IVec3::X),
            IVec3::new(0, 4, 6)
        );
        // Full block hit on -Z face at z = 6
        assert_eq!(
            adjacent_block_origin(origin, IVec3::new(-2, 4, 6), -IVec3::Z),
            IVec3::new(-2, 4, 4)
        );
        // Bottom slab hit on top (+Y) face at y = 4 (origin is y = 4) -> connects directly at y = 5
        assert_eq!(
            adjacent_block_origin(origin, IVec3::new(-2, 4, 6), IVec3::Y),
            IVec3::new(-2, 5, 6)
        );
    }

    #[test]
    fn is_solid_local_ignores_centered_layer_for_standard_outline() {
        use super::{Voxel, is_solid_local};
        use crate::voxel::{chunk::Chunk, world::VoxelWorld};

        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());
        let origin = IVec3::new(0, 0, 0);

        // Layer 0 is centered
        world.set_voxel(origin + IVec3::new(0, 0, 0), Voxel::Sand);
        world.set_voxel(origin + IVec3::new(1, 0, 0), Voxel::Occupied);
        world.set_voxel(origin + IVec3::new(0, 0, 1), Voxel::Occupied);
        world.set_voxel(origin + IVec3::new(1, 0, 1), Voxel::Occupied);

        // Layer 1 is a slab (4 sand voxels)
        world.set_voxel(origin + IVec3::new(0, 1, 0), Voxel::Sand);
        world.set_voxel(origin + IVec3::new(1, 1, 0), Voxel::Sand);
        world.set_voxel(origin + IVec3::new(0, 1, 1), Voxel::Sand);
        world.set_voxel(origin + IVec3::new(1, 1, 1), Voxel::Sand);

        // Layer 0 is treated as non-solid for standard grid outline since layer0_centered = true
        assert!(!is_solid_local(
            &world,
            origin,
            IVec3::new(0, 0, 0),
            true,
            false
        ));
        assert!(!is_solid_local(
            &world,
            origin,
            IVec3::new(1, 0, 0),
            true,
            false
        ));

        // Layer 1 is recognized as solid for standard grid outline
        assert!(is_solid_local(
            &world,
            origin,
            IVec3::new(0, 1, 0),
            true,
            false
        ));
        assert!(is_solid_local(
            &world,
            origin,
            IVec3::new(1, 1, 1),
            true,
            false
        ));
    }
}
