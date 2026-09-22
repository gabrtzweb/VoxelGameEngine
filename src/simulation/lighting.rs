use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::world::{CHUNK_SIZE, VOXEL_SIZE, Voxel, VoxelWorld};

const LIGHT_INTENSITY: f32 = 750_000.0;
const LIGHT_RANGE: f32 = 26.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LightState {
    pub entity: Entity,
    pub voxel: Voxel,
    pub count: u8,
}

#[derive(Resource, Default)]
pub struct VoxelLightRegistry {
    pub entries: HashMap<IVec3, LightState>,
}

pub fn sync_voxel_light(
    commands: &mut Commands,
    world: &VoxelWorld,
    world_voxel: IVec3,
    registry: &mut VoxelLightRegistry,
) {
    sync_block_light(commands, world, world_voxel, registry);
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
    let mut block_coords = HashSet::new();

    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let voxel = chunk.get(x, y, z);
                if voxel.is_light() {
                    let world_voxel = chunk_origin + IVec3::new(x as i32, y as i32, z as i32);
                    block_coords.insert(world_voxel);
                }
            }
        }
    }

    for block_coord in block_coords {
        sync_block_light(commands, world, block_coord, registry);
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
        .filter_map(|(&block_coord, &state)| {
            let (light_chunk, _) = VoxelWorld::world_voxel_to_chunk(block_coord);

            if light_chunk == chunk_coordinate {
                Some((block_coord, state.entity))
            } else {
                None
            }
        })
        .collect();

    for (block_coord, entity) in lights_to_remove {
        if let Ok(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.despawn();
        }
        registry.entries.remove(&block_coord);
    }
}

fn sync_block_light(
    commands: &mut Commands,
    world: &VoxelWorld,
    block_coord: IVec3,
    registry: &mut VoxelLightRegistry,
) {
    if let Some(voxel) = world.get_voxel(block_coord).filter(|v| v.is_light()) {
        if let Some(&existing) = registry.entries.get(&block_coord) {
            if existing.voxel == voxel {
                return;
            }
            commands.entity(existing.entity).despawn();
        }

        let pos = block_coord.as_vec3() * VOXEL_SIZE + Vec3::splat(VOXEL_SIZE * 0.5);

        let entity = commands
            .spawn((
                PointLight {
                    color: voxel.light_color(),
                    intensity: LIGHT_INTENSITY,
                    range: LIGHT_RANGE,
                    radius: 0.0,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_translation(pos),
            ))
            .id();

        registry.entries.insert(
            block_coord,
            LightState {
                entity,
                voxel,
                count: 1,
            },
        );
    } else if let Some(existing) = registry.entries.remove(&block_coord) {
        commands.entity(existing.entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Chunk;

    #[test]
    fn block_light_registers_point_light_for_light_block() {
        let mut app = App::new();
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, Voxel::LightWarm);
        world.insert_chunk(IVec3::ZERO, chunk);

        let mut registry = VoxelLightRegistry::default();
        let mut commands = app.world_mut().commands();

        sync_voxel_light(&mut commands, &world, IVec3::ZERO, &mut registry);

        assert_eq!(
            registry.entries.len(),
            1,
            "Exactly 1 light entity should be registered for the 1m light block"
        );
        let block_coord = IVec3::ZERO;
        let state = registry
            .entries
            .get(&block_coord)
            .expect("Light state should exist");
        assert_eq!(state.count, 1);
        assert_eq!(state.voxel, Voxel::LightWarm);
    }
}
