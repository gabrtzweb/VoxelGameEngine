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
    let block_coord = IVec3::new(
        world_voxel.x.div_euclid(2),
        world_voxel.y.div_euclid(2),
        world_voxel.z.div_euclid(2),
    );
    sync_block_light(commands, world, block_coord, registry);
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
                    block_coords.insert(IVec3::new(
                        world_voxel.x.div_euclid(2),
                        world_voxel.y.div_euclid(2),
                        world_voxel.z.div_euclid(2),
                    ));
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
            let (light_chunk, _) = VoxelWorld::world_voxel_to_chunk(block_coord * 2);

            if light_chunk == chunk_coordinate {
                Some((block_coord, state.entity))
            } else {
                None
            }
        })
        .collect();

    for (block_coord, entity) in lights_to_remove {
        commands.entity(entity).despawn();
        registry.entries.remove(&block_coord);
    }
}

fn sync_block_light(
    commands: &mut Commands,
    world: &VoxelWorld,
    block_coord: IVec3,
    registry: &mut VoxelLightRegistry,
) {
    let block_origin = block_coord * 2;
    let mut light_positions = Vec::new();
    let mut dominant_voxel = None;

    for dy in 0..2 {
        for dz in 0..2 {
            for dx in 0..2 {
                let sub_pos = block_origin + IVec3::new(dx, dy, dz);
                if let Some(v) = world.get_voxel(sub_pos)
                    && v.is_light()
                {
                    light_positions.push(sub_pos);
                    if dominant_voxel.is_none() {
                        dominant_voxel = Some(v);
                    }
                }
            }
        }
    }

    let count = light_positions.len() as u8;

    if let Some(voxel) = dominant_voxel {
        if let Some(&existing) = registry.entries.get(&block_coord) {
            if existing.count == count && existing.voxel == voxel {
                return;
            }
            commands.entity(existing.entity).despawn();
        }

        let avg_pos = light_positions
            .iter()
            .map(|p| p.as_vec3() * VOXEL_SIZE + Vec3::splat(VOXEL_SIZE * 0.5))
            .sum::<Vec3>()
            / (count as f32);

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
                Transform::from_translation(avg_pos),
            ))
            .id();

        registry.entries.insert(
            block_coord,
            LightState {
                entity,
                voxel,
                count,
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
    fn block_light_consolidates_eight_subvoxels_into_single_point_light() {
        let mut app = App::new();
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::default();
        for dy in 0..2 {
            for dz in 0..2 {
                for dx in 0..2 {
                    chunk.set(dx, dy, dz, Voxel::LightWarm);
                }
            }
        }
        world.insert_chunk(IVec3::ZERO, chunk);

        let mut registry = VoxelLightRegistry::default();
        let mut commands = app.world_mut().commands();

        for dy in 0..2 {
            for dz in 0..2 {
                for dx in 0..2 {
                    sync_voxel_light(&mut commands, &world, IVec3::new(dx, dy, dz), &mut registry);
                }
            }
        }

        assert_eq!(
            registry.entries.len(),
            1,
            "Exactly 1 light entity should be registered for the full 1m block"
        );
        let block_coord = IVec3::ZERO;
        let state = registry
            .entries
            .get(&block_coord)
            .expect("Light state should exist");
        assert_eq!(state.count, 8);
        assert_eq!(state.voxel, Voxel::LightWarm);
    }
}
