use std::collections::HashMap;

use bevy::prelude::*;

use super::{
    chunk::{CHUNK_SIZE, VOXEL_SIZE, Voxel},
    world::VoxelWorld,
};

const LIGHT_INTENSITY: f32 = 65_000.0;
const LIGHT_RANGE: f32 = 16.0;
const LIGHT_RADIUS: f32 = 0.25;

#[derive(Resource, Default)]
pub struct VoxelLightRegistry {
    entries: HashMap<IVec3, Entity>,
}

pub fn sync_voxel_light(
    commands: &mut Commands,
    world: &VoxelWorld,
    world_voxel: IVec3,
    registry: &mut VoxelLightRegistry,
) {
    let should_exist = world.get_voxel(world_voxel) == Some(Voxel::Light);

    let existing = registry.entries.get(&world_voxel).copied();

    match (should_exist, existing) {
        (true, None) => {
            spawn_voxel_light(commands, world_voxel, registry);
        }

        (false, Some(entity)) => {
            commands.entity(entity).despawn();

            registry.entries.remove(&world_voxel);
        }

        _ => {}
    }
}

pub fn sync_chunk_lights(
    commands: &mut Commands,
    world: &VoxelWorld,
    chunk_coordinate: IVec3,
    registry: &mut VoxelLightRegistry,
) {
    remove_chunk_lights(commands, chunk_coordinate, registry);

    let Some(chunk) = world.get_chunk(chunk_coordinate) else {
        return;
    };

    let chunk_origin = chunk_coordinate * CHUNK_SIZE as i32;

    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.get(x, y, z) != Voxel::Light {
                    continue;
                }

                let world_voxel = chunk_origin + IVec3::new(x as i32, y as i32, z as i32);

                spawn_voxel_light(commands, world_voxel, registry);
            }
        }
    }
}

pub fn remove_chunk_lights(
    commands: &mut Commands,
    chunk_coordinate: IVec3,
    registry: &mut VoxelLightRegistry,
) {
    let lights_to_remove: Vec<(IVec3, Entity)> = registry
        .entries
        .iter()
        .filter_map(|(&world_voxel, &entity)| {
            let (light_chunk, _) = VoxelWorld::world_voxel_to_chunk(world_voxel);

            if light_chunk == chunk_coordinate {
                Some((world_voxel, entity))
            } else {
                None
            }
        })
        .collect();

    for (world_voxel, entity) in lights_to_remove {
        commands.entity(entity).despawn();

        registry.entries.remove(&world_voxel);
    }
}

fn spawn_voxel_light(
    commands: &mut Commands,
    world_voxel: IVec3,
    registry: &mut VoxelLightRegistry,
) {
    if registry.entries.contains_key(&world_voxel) {
        return;
    }

    let position = world_voxel.as_vec3() * VOXEL_SIZE + Vec3::splat(VOXEL_SIZE * 0.5);

    let entity = commands
        .spawn((
            PointLight {
                color: Color::srgb(1.0, 0.72, 0.32),
                intensity: LIGHT_INTENSITY,
                range: LIGHT_RANGE,
                radius: LIGHT_RADIUS,

                // Intentionally disabled for now.
                // Point-light shadow maps are
                // considerably more expensive.
                shadow_maps_enabled: false,

                ..default()
            },
            Transform::from_translation(position),
        ))
        .id();

    registry.entries.insert(world_voxel, entity);
}
