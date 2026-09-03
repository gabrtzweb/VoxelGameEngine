use std::collections::HashMap;

use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use super::{
    chunk::{CHUNK_SIZE, VOXEL_SIZE, Voxel},
    world::VoxelWorld,
};

const LIGHT_INTENSITY: f32 = 65_000.0;
const LIGHT_RANGE: f32 = 16.0;
const LIGHT_RADIUS: f32 = 0.25;

const LIGHT_COLOR: Color = Color::srgb(1.0, 0.82, 0.42);

#[derive(Clone, Copy)]
struct VoxelLightEntities {
    visual: Entity,
    light: Entity,
}

#[derive(Resource)]
pub struct VoxelLightRegistry {
    entries: HashMap<IVec3, VoxelLightEntities>,

    mesh: Handle<Mesh>,

    material: Handle<StandardMaterial>,
}

impl FromWorld for VoxelLightRegistry {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();

            meshes.add(Cuboid::new(VOXEL_SIZE, VOXEL_SIZE, VOXEL_SIZE))
        };

        let material = {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();

            materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.78, 0.28),

                emissive: LinearRgba::rgb(8.0, 5.0, 1.2),

                // Keeps every face of the block
                // visually bright.
                unlit: true,

                perceptual_roughness: 0.7,

                ..default()
            })
        };

        Self {
            entries: HashMap::new(),

            mesh,
            material,
        }
    }
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

        (false, Some(entities)) => {
            despawn_voxel_light(commands, world_voxel, entities, registry);
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
    let lights_to_remove: Vec<(IVec3, VoxelLightEntities)> = registry
        .entries
        .iter()
        .filter_map(|(&world_voxel, &entities)| {
            let (light_chunk, _) = VoxelWorld::world_voxel_to_chunk(world_voxel);

            if light_chunk == chunk_coordinate {
                Some((world_voxel, entities))
            } else {
                None
            }
        })
        .collect();

    for (world_voxel, entities) in lights_to_remove {
        despawn_voxel_light(commands, world_voxel, entities, registry);
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

    // Visual entity.
    //
    // This can be frustum culled normally without
    // affecting the actual light source.
    let visual = commands
        .spawn((
            Mesh3d(registry.mesh.clone()),
            MeshMaterial3d(registry.material.clone()),
            Transform::from_translation(position),
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .id();

    // Lighting entity.
    //
    // Deliberately separate from the visible mesh so
    // lighting does not disappear when the block itself
    // leaves the camera frustum.
    let light = commands
        .spawn((
            PointLight {
                color: LIGHT_COLOR,

                intensity: LIGHT_INTENSITY,

                range: LIGHT_RANGE,

                radius: LIGHT_RADIUS,

                shadow_maps_enabled: false,

                ..default()
            },
            Transform::from_translation(position),
        ))
        .id();

    registry
        .entries
        .insert(world_voxel, VoxelLightEntities { visual, light });
}

fn despawn_voxel_light(
    commands: &mut Commands,
    world_voxel: IVec3,
    entities: VoxelLightEntities,
    registry: &mut VoxelLightRegistry,
) {
    commands.entity(entities.visual).despawn();

    commands.entity(entities.light).despawn();

    registry.entries.remove(&world_voxel);
}
