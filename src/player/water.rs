use bevy::prelude::*;

use crate::voxel::{VOXEL_SIZE, VoxelWorld, chunk::Voxel};

use super::{PLAYER_HEIGHT, PLAYER_WIDTH, PlayerCamera};

const WATER_BOUNDS_EPSILON: f32 = 0.001;

#[derive(Component)]
pub(super) struct UnderwaterOverlay;

pub(super) fn spawn_underwater_overlay(mut commands: Commands) {
    commands.spawn((
        UnderwaterOverlay,
        Node {
            position_type: PositionType::Absolute,

            left: px(0.0),

            top: px(0.0),

            width: percent(100.0),

            height: percent(100.0),

            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.20, 0.34, 0.16)),
        // Keep HUD and crosshair above the tint.
        ZIndex(-10),
        Visibility::Hidden,
    ));
}

pub(super) fn update_underwater_effect(
    world: Res<VoxelWorld>,

    camera: Single<&GlobalTransform, With<PlayerCamera>>,

    overlay: Single<&mut Visibility, With<UnderwaterOverlay>>,
) {
    let underwater = is_point_in_water(&world, camera.translation());

    let mut visibility = overlay.into_inner();

    *visibility = if underwater {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub(super) fn player_submersion(world: &VoxelWorld, position: Vec3) -> f32 {
    let (body_min, body_max) = player_body_bounds(position);

    let min_voxel = world_position_to_voxel(body_min);

    let max_voxel = world_position_to_voxel(body_max - Vec3::splat(WATER_BOUNDS_EPSILON));

    let mut water_volume = 0.0;

    for y in min_voxel.y..=max_voxel.y {
        for z in min_voxel.z..=max_voxel.z {
            for x in min_voxel.x..=max_voxel.x {
                let coordinate = IVec3::new(x, y, z);

                if world.get_voxel(coordinate) != Some(Voxel::Water) {
                    continue;
                }

                water_volume += overlap_volume(body_min, body_max, coordinate);
            }
        }
    }

    let player_volume = PLAYER_WIDTH * PLAYER_WIDTH * PLAYER_HEIGHT;

    if player_volume <= 0.0 {
        return 0.0;
    }

    (water_volume / player_volume).clamp(0.0, 1.0)
}

fn is_point_in_water(world: &VoxelWorld, position: Vec3) -> bool {
    let coordinate = world_position_to_voxel(position);

    world.get_voxel(coordinate) == Some(Voxel::Water)
}

fn player_body_bounds(position: Vec3) -> (Vec3, Vec3) {
    let half_width = PLAYER_WIDTH * 0.5;

    let minimum = Vec3::new(position.x - half_width, position.y, position.z - half_width);

    let maximum = Vec3::new(
        position.x + half_width,
        position.y + PLAYER_HEIGHT,
        position.z + half_width,
    );

    (minimum, maximum)
}

fn world_position_to_voxel(position: Vec3) -> IVec3 {
    IVec3::new(
        (position.x / VOXEL_SIZE).floor() as i32,
        (position.y / VOXEL_SIZE).floor() as i32,
        (position.z / VOXEL_SIZE).floor() as i32,
    )
}

fn overlap_volume(body_min: Vec3, body_max: Vec3, voxel_coordinate: IVec3) -> f32 {
    let voxel_min = voxel_coordinate.as_vec3() * VOXEL_SIZE;

    let voxel_max = voxel_min + Vec3::splat(VOXEL_SIZE);

    let overlap_min = body_min.max(voxel_min);

    let overlap_max = body_max.min(voxel_max);

    let overlap = overlap_max - overlap_min;

    if overlap.x <= 0.0 || overlap.y <= 0.0 || overlap.z <= 0.0 {
        return 0.0;
    }

    overlap.x * overlap.y * overlap.z
}
