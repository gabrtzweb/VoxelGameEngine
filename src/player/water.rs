use bevy::prelude::*;

use crate::world::{VOXEL_SIZE, Voxel, VoxelWorld};

use super::{PLAYER_HEIGHT, PLAYER_WIDTH};

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
        BackgroundColor(Color::srgba(0.02, 0.20, 0.34, 0.08)),
        // Keep HUD and crosshair above the tint.
        ZIndex(-10),
        Visibility::Hidden,
    ));
}

pub(super) fn update_underwater_effect(
    env_status: Res<super::PlayerEnvironmentStatus>,
    overlay: Single<(&mut Visibility, &mut BackgroundColor), With<UnderwaterOverlay>>,
) {
    let (mut visibility, mut bg) = overlay.into_inner();
    if let Some(fluid) = env_status.camera_fluid {
        *visibility = Visibility::Visible;
        bg.0 = match fluid {
            Voxel::LiquidLava | Voxel::LiquidCoreMagma | Voxel::LiquidMolten => {
                Color::srgba(0.88, 0.28, 0.03, 0.42)
            }
            Voxel::LiquidAcid => Color::srgba(0.18, 0.68, 0.08, 0.28),
            Voxel::LiquidTar => Color::srgba(0.05, 0.01, 0.09, 0.58),
            Voxel::LiquidBlood => Color::srgba(0.42, 0.03, 0.04, 0.38),
            Voxel::LiquidSludge => Color::srgba(0.20, 0.14, 0.08, 0.42),
            Voxel::LiquidOoze => Color::srgba(0.24, 0.29, 0.06, 0.38),
            _ => Color::srgba(0.02, 0.20, 0.34, 0.08),
        };
    } else {
        *visibility = Visibility::Hidden;
    }
}

pub(super) fn player_submersion(world: &VoxelWorld, position: Vec3) -> (f32, Option<Voxel>) {
    let (body_min, body_max) = player_body_bounds(position);

    let min_voxel = world_position_to_voxel(body_min);

    let max_voxel = world_position_to_voxel(body_max - Vec3::splat(WATER_BOUNDS_EPSILON));

    let mut fluid_volume = 0.0;
    let mut best_fluid: Option<Voxel> = None;
    let mut best_vol = 0.0f32;

    for y in min_voxel.y..=max_voxel.y {
        for z in min_voxel.z..=max_voxel.z {
            for x in min_voxel.x..=max_voxel.x {
                let coordinate = IVec3::new(x, y, z);

                let Some(voxel) = world.get_voxel(coordinate) else {
                    continue;
                };
                let is_submerged_fluid =
                    voxel.is_fluid() || voxel.is_marine_flora() || voxel.is_coral_fan();
                if !is_submerged_fluid {
                    continue;
                }

                let vol = overlap_volume(body_min, body_max, coordinate);
                fluid_volume += vol;
                let normalized = if voxel.is_marine_flora() || voxel.is_coral_fan() {
                    Voxel::LiquidWater
                } else {
                    voxel
                };
                if vol > best_vol {
                    best_vol = vol;
                    best_fluid = Some(normalized);
                }
            }
        }
    }

    let player_volume = PLAYER_WIDTH * PLAYER_WIDTH * PLAYER_HEIGHT;

    if player_volume <= 0.0 || fluid_volume <= 0.0 {
        return (0.0, None);
    }

    ((fluid_volume / player_volume).clamp(0.0, 1.0), best_fluid)
}

pub(crate) fn fluid_at_point(world: &VoxelWorld, position: Vec3) -> Option<Voxel> {
    let coordinate = world_position_to_voxel(position);

    let voxel = world.get_voxel(coordinate)?;
    let is_fluid_or_flora = |v: Voxel| v.is_fluid() || v.is_marine_flora() || v.is_coral_fan();
    if !is_fluid_or_flora(voxel) {
        return None;
    }

    let above_coordinate = coordinate + IVec3::Y;
    if !world
        .get_voxel(above_coordinate)
        .is_some_and(is_fluid_or_flora)
    {
        let surface_y = coordinate.y as f32 * VOXEL_SIZE + 0.90;
        if position.y >= surface_y {
            return None;
        }
    }

    if voxel.is_marine_flora() || voxel.is_coral_fan() {
        Some(Voxel::LiquidWater)
    } else {
        Some(voxel)
    }
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
