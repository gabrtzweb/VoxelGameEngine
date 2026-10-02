use std::collections::HashMap;

use bevy::prelude::*;

use crate::world::{CHUNK_SIZE, ChunkHomogeneity, VOXEL_SIZE, Voxel, VoxelWorld};

const LIGHT_INTENSITY: f32 = 450_000.0;
const LIGHT_RANGE: f32 = 16.0;
const MAX_TORCHES_PER_CHUNK: usize = 16;

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

    match chunk.homogeneity() {
        ChunkHomogeneity::Empty => return,
        ChunkHomogeneity::Solid(v)
            if !v.is_point_light_fixture()
                && v != Voxel::Liquid_Lava
                && v != Voxel::Liquid_Molten =>
        {
            return;
        }
        _ => {}
    }

    let chunk_origin = chunk_coordinate * CHUNK_SIZE as i32;
    let mut torch_coords = Vec::with_capacity(MAX_TORCHES_PER_CHUNK);
    let mut lava_pos_sum = Vec3::ZERO;
    let mut lava_count = 0usize;

    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let voxel = chunk.get(x, y, z);
                if voxel.is_point_light_fixture() {
                    let world_voxel = chunk_origin + IVec3::new(x as i32, y as i32, z as i32);
                    torch_coords.push(world_voxel);
                } else if voxel == Voxel::Liquid_Lava || voxel == Voxel::Liquid_Molten {
                    let world_pos = (chunk_origin + IVec3::new(x as i32, y as i32, z as i32))
                        .as_vec3()
                        + Vec3::splat(0.5);
                    lava_pos_sum += world_pos;
                    lava_count += 1;
                }
            }
        }
    }

    // 1. Point light fixtures (torches, lamps): capped at MAX_TORCHES_PER_CHUNK
    for block_coord in torch_coords.into_iter().take(MAX_TORCHES_PER_CHUNK) {
        sync_block_light(commands, world, block_coord, registry);
    }

    // 2. Lava pools: consolidate all lava blocks in this chunk into at most ONE single
    // representative ambient point light at the centroid.
    // Prevents thousands of overlapping GPU lights that blow up cluster buffers and crash drivers.
    if lava_count > 0 {
        let centroid = lava_pos_sum / lava_count as f32;
        let pool_key = chunk_origin + IVec3::splat(CHUNK_SIZE as i32 / 2);

        let entity = commands
            .spawn((
                PointLight {
                    color: Color::srgb(1.0, 0.45, 0.15),
                    intensity: 450_000.0,
                    range: 18.0,
                    radius: 2.0,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_translation(centroid),
            ))
            .id();

        registry.entries.insert(
            pool_key,
            LightState {
                entity,
                voxel: Voxel::Liquid_Lava,
                count: 1,
            },
        );
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
    if let Some(voxel) = world
        .get_voxel(block_coord)
        .filter(|v| v.is_point_light_fixture())
    {
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
