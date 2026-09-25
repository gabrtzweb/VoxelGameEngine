use bevy::platform::collections::HashMap;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::{
    meshing::{ChunkMaterial, greedy::FaceDirection},
    world::{Voxel, VoxelWorld},
};

#[derive(Event, Debug, Clone, Copy)]
pub struct BlockBreakEvent {
    pub position: IVec3,
    pub voxel: Voxel,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct BlockPlaceEvent {
    pub position: IVec3,
    pub voxel: Voxel,
}

#[derive(Component)]
pub struct DebrisParticle {
    pub velocity: Vec3,
    pub rot_vel: Vec3,
    pub lifetime: f32,
    pub bounce_count: u8,
}

#[derive(Component)]
pub struct BlockPlacementPop {
    pub elapsed: f32,
    pub duration: f32,
}

#[derive(Resource, Default)]
pub struct FeedbackMeshCache {
    debris_meshes: HashMap<Voxel, Handle<Mesh>>,
    pop_meshes: HashMap<Voxel, Handle<Mesh>>,
}

type CubeFaceDef = (FaceDirection, [[f32; 3]; 4], [f32; 3], [[f32; 2]; 4]);

const CUBE_FACES: [CubeFaceDef; 6] = [
    (
        FaceDirection::PositiveY,
        [
            [-1.0, 1.0, -1.0],
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, -1.0],
        ],
        [0.0, 1.0, 0.0],
        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
    ),
    (
        FaceDirection::NegativeY,
        [
            [-1.0, -1.0, 1.0],
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, -1.0, 1.0],
        ],
        [0.0, -1.0, 0.0],
        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
    ),
    (
        FaceDirection::PositiveX,
        [
            [1.0, -1.0, 1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [1.0, 1.0, 1.0],
        ],
        [1.0, 0.0, 0.0],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    ),
    (
        FaceDirection::NegativeX,
        [
            [-1.0, -1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, 1.0],
            [-1.0, 1.0, -1.0],
        ],
        [-1.0, 0.0, 0.0],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    ),
    (
        FaceDirection::PositiveZ,
        [
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ],
        [0.0, 0.0, 1.0],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    ),
    (
        FaceDirection::NegativeZ,
        [
            [1.0, -1.0, -1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0],
        ],
        [0.0, 0.0, -1.0],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    ),
];

fn create_cube_mesh(
    half_extent: f32,
    voxel: Voxel,
    chunk_material: &ChunkMaterial,
    is_pop: bool,
) -> Mesh {
    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut uv_bs = Vec::with_capacity(24);
    let mut colors = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (face_dir, quad_positions, normal, face_uvs) in &CUBE_FACES {
        let base_index = positions.len() as u32;

        let (layer, frames) =
            chunk_material
                .texture_registry
                .get_face_texture_info(voxel, IVec3::ZERO, *face_dir);
        let frame_count = if voxel.is_light() {
            -4.0
        } else {
            frames as f32
        };

        let color = if is_pop {
            if voxel == Voxel::Grass {
                if *face_dir == FaceDirection::PositiveY {
                    voxel.tint_color()
                } else {
                    [1.0, 1.0, 1.0, 1.0]
                }
            } else if voxel.is_leaves() {
                voxel.tint_color()
            } else {
                [1.0, 1.0, 1.0, 1.0]
            }
        } else {
            voxel.tint_color()
        };

        for (i, &pos) in quad_positions.iter().enumerate() {
            positions.push([
                pos[0] * half_extent,
                pos[1] * half_extent,
                pos[2] * half_extent,
            ]);
            normals.push(*normal);
            uv_bs.push([layer as f32, frame_count]);
            colors.push(color);

            let uv = face_uvs[i];
            if is_pop {
                uvs.push(uv);
            } else {
                // Crop center sub-patch of texture for sub-voxel debris
                uvs.push([uv[0] * 0.25 + 0.35, uv[1] * 0.25 + 0.35]);
            }
        }

        indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, uv_bs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

fn handle_block_break_feedback(
    event: On<BlockBreakEvent>,
    mut commands: Commands,
    chunk_material: Option<Res<ChunkMaterial>>,
    mut mesh_cache: ResMut<FeedbackMeshCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    active_pops: Query<(Entity, &Transform), With<BlockPlacementPop>>,
    mut seed: Local<u64>,
) {
    let Some(chunk_material) = chunk_material else {
        return;
    };

    if *seed == 0 {
        *seed = 0x9E37_79B9_7F4A_7C15;
    }

    let center = event.position.as_vec3() + Vec3::splat(0.5);

    // Despawn any placement pop at this block location
    for (pop_entity, pop_transform) in &active_pops {
        if (pop_transform.translation - center).length_squared() < 0.05 {
            commands.entity(pop_entity).despawn();
        }
    }

    if event.voxel.is_empty() || event.voxel.is_water() {
        return;
    }

    let mesh_handle = mesh_cache
        .debris_meshes
        .entry(event.voxel)
        .or_insert_with(|| meshes.add(create_cube_mesh(0.040, event.voxel, &chunk_material, false)))
        .clone();

    let material_handle = if event.voxel.is_transparent() {
        chunk_material.transparent.clone()
    } else {
        chunk_material.opaque.clone()
    };

    // Spawn 8 debris particles in a subtle, contained scatter
    for _ in 0..8 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let rx = ((*seed >> 32) as u32 as f32) / 4294967295.0;
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let ry = ((*seed >> 32) as u32 as f32) / 4294967295.0;
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let rz = ((*seed >> 32) as u32 as f32) / 4294967295.0;

        let offset = Vec3::new(rx - 0.5, ry - 0.5, rz - 0.5) * 0.35;
        let spawn_pos = center + offset;
        let outward = offset.normalize_or_zero();

        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let speed_rand = ((*seed >> 32) as u32 as f32) / 4294967295.0;
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let up_rand = ((*seed >> 32) as u32 as f32) / 4294967295.0;
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let life_rand = ((*seed >> 32) as u32 as f32) / 4294967295.0;

        let velocity =
            outward * (0.8 + speed_rand * 1.2) + Vec3::new(0.0, 0.9 + up_rand * 1.0, 0.0);
        let rot_vel = Vec3::new(rx - 0.5, ry - 0.5, rz - 0.5) * 8.0;
        let lifetime = 0.38 + life_rand * 0.22;

        commands.spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material_handle.clone()),
            Transform::from_translation(spawn_pos),
            DebrisParticle {
                velocity,
                rot_vel,
                lifetime,
                bounce_count: 0,
            },
        ));
    }
}

fn handle_block_place_feedback(
    event: On<BlockPlaceEvent>,
    mut commands: Commands,
    chunk_material: Option<Res<ChunkMaterial>>,
    mut mesh_cache: ResMut<FeedbackMeshCache>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(chunk_material) = chunk_material else {
        return;
    };

    if event.voxel.is_empty() || event.voxel.is_water() {
        return;
    }

    let mesh_handle = mesh_cache
        .pop_meshes
        .entry(event.voxel)
        .or_insert_with(|| meshes.add(create_cube_mesh(0.5, event.voxel, &chunk_material, true)))
        .clone();

    let material_handle = if event.voxel.is_transparent() {
        chunk_material.transparent.clone()
    } else {
        chunk_material.opaque.clone()
    };

    let center = event.position.as_vec3() + Vec3::splat(0.5);

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_translation(center).with_scale(Vec3::splat(1.15)),
        BlockPlacementPop {
            elapsed: 0.0,
            duration: 0.18,
        },
    ));
}

fn update_block_placement_pops(
    mut commands: Commands,
    time: Res<Time>,
    mut pops: Query<(Entity, &mut Transform, &mut BlockPlacementPop)>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, mut pop) in &mut pops {
        pop.elapsed += delta;
        if pop.elapsed >= pop.duration {
            commands.entity(entity).despawn();
            continue;
        }

        let p = (pop.elapsed / pop.duration).clamp(0.0, 1.0);
        let bounce = 0.15 * (1.0 - p) * (p * std::f32::consts::PI * 1.5).cos();
        transform.scale = Vec3::splat(1.0 + bounce);
    }
}

fn update_debris_particles(
    mut commands: Commands,
    time: Res<Time>,
    world: Res<VoxelWorld>,
    mut particles: Query<(Entity, &mut Transform, &mut DebrisParticle)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle) in &mut particles {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Shrink particle near end of lifetime
        if particle.lifetime < 0.20 {
            let factor = (particle.lifetime / 0.20).clamp(0.0, 1.0);
            transform.scale = Vec3::splat(factor);
        }

        // Gravity & air drag
        particle.velocity.y -= 16.0 * dt;
        particle.velocity.x *= (1.0 - 1.5 * dt).max(0.0);
        particle.velocity.z *= (1.0 - 1.5 * dt).max(0.0);

        // Rotation
        transform.rotate_local_x(particle.rot_vel.x * dt);
        transform.rotate_local_y(particle.rot_vel.y * dt);
        transform.rotate_local_z(particle.rot_vel.z * dt);

        let curr_pos = transform.translation;
        let next_pos = curr_pos + particle.velocity * dt;

        if particle.bounce_count < 3 {
            let next_iv = next_pos.floor().as_ivec3();
            let is_solid = world.get_voxel(next_iv).is_some_and(|v| v.is_collidable());

            if is_solid {
                let curr_iv = curr_pos.floor().as_ivec3();
                if particle.velocity.y < 0.0 && curr_iv.y != next_iv.y {
                    // Hit floor
                    particle.velocity.y = -particle.velocity.y * 0.38;
                    particle.velocity.x *= 0.65;
                    particle.velocity.z *= 0.65;
                    particle.rot_vel *= 0.5;
                    particle.bounce_count += 1;
                    transform.translation.y = next_iv.y as f32 + 1.0 + 0.040;
                    transform.translation.x = next_pos.x;
                    transform.translation.z = next_pos.z;
                } else if curr_iv.x != next_iv.x {
                    // Hit X wall
                    particle.velocity.x = -particle.velocity.x * 0.38;
                    particle.bounce_count += 1;
                    transform.translation.y = next_pos.y;
                    transform.translation.z = next_pos.z;
                } else if curr_iv.z != next_iv.z {
                    // Hit Z wall
                    particle.velocity.z = -particle.velocity.z * 0.38;
                    particle.bounce_count += 1;
                    transform.translation.x = next_pos.x;
                    transform.translation.y = next_pos.y;
                } else {
                    particle.velocity = Vec3::ZERO;
                    particle.rot_vel = Vec3::ZERO;
                    particle.bounce_count = 3;
                }
            } else {
                transform.translation = next_pos;
            }
        } else {
            transform.translation = next_pos;
        }
    }
}

pub struct InteractionFeedbackPlugin;

impl Plugin for InteractionFeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FeedbackMeshCache>()
            .add_observer(handle_block_break_feedback)
            .add_observer(handle_block_place_feedback)
            .add_systems(
                Update,
                (update_debris_particles, update_block_placement_pops),
            );
    }
}
