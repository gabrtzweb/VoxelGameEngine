use bevy::platform::collections::HashMap;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::{
    audio::SoundMaterial,
    generation::TerrainGenerator,
    menu::MenuState,
    meshing::{ChunkMaterial, greedy::FaceDirection},
    player::{GameMode, Player, PlayerInventory, hotbar::Hotbar},
    world::{BlockShape, Voxel, VoxelWorld},
};

use super::feedback::BlockBreakEvent;

const DROP_HALF_EXTENT: f32 = 0.135;
const HELD_HALF_EXTENT: f32 = 0.155;
const DROP_MAX_LIFETIME: f32 = 300.0;
const MAGNET_RADIUS: f32 = 2.65;
const PICKUP_RADIUS: f32 = 0.70;

#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnItemDropEvent {
    pub position: Vec3,
    pub velocity: Vec3,
    pub voxel: Voxel,
    pub pickup_delay: f32,
}

#[derive(Component, Debug)]
pub struct ItemDrop {
    pub voxel: Voxel,
    pub velocity: Vec3,
    pub age: f32,
    pub pickup_delay: f32,
    pub bob_phase: f32,
    pub base_y: f32,
    pub grounded: bool,
    pub in_fluid: bool,
}

#[derive(Resource, Default)]
pub struct VoxelItemMeshCache {
    drop_meshes: HashMap<Voxel, Handle<Mesh>>,
    held_meshes: HashMap<Voxel, Handle<Mesh>>,
}

impl VoxelItemMeshCache {
    pub fn get_or_create_drop_mesh(
        &mut self,
        voxel: Voxel,
        meshes: &mut Assets<Mesh>,
        chunk_material: &ChunkMaterial,
    ) -> Handle<Mesh> {
        self.drop_meshes
            .entry(voxel)
            .or_insert_with(|| {
                meshes.add(create_voxel_item_mesh(
                    DROP_HALF_EXTENT,
                    voxel,
                    chunk_material,
                ))
            })
            .clone()
    }

    pub fn get_or_create_held_mesh(
        &mut self,
        voxel: Voxel,
        meshes: &mut Assets<Mesh>,
        chunk_material: &ChunkMaterial,
    ) -> Handle<Mesh> {
        self.held_meshes
            .entry(voxel)
            .or_insert_with(|| {
                meshes.add(create_voxel_item_mesh(
                    HELD_HALF_EXTENT,
                    voxel,
                    chunk_material,
                ))
            })
            .clone()
    }
}

type CubeFaceDef = (FaceDirection, [[f32; 3]; 4], [f32; 3], [[f32; 2]; 4]);

const ITEM_CUBE_FACES: [CubeFaceDef; 6] = [
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

pub fn create_voxel_item_mesh(
    half_extent: f32,
    voxel: Voxel,
    chunk_material: &ChunkMaterial,
) -> Mesh {
    let shape = voxel.default_shape();
    if voxel.is_flora_billboard() || shape == BlockShape::CrossBillboard {
        return create_cross_flora_item_mesh(half_extent * 1.15, voxel, chunk_material);
    }

    let (scale_x, scale_y, scale_z) = match shape {
        BlockShape::Slab => (1.0, 0.5, 1.0),
        BlockShape::Torch | BlockShape::Speleothem => (0.28, 0.85, 0.28),
        BlockShape::Column | BlockShape::Bamboo => (0.45, 1.0, 0.45),
        BlockShape::FlatPad | BlockShape::WallDecal => (1.0, 0.20, 1.0),
        BlockShape::Pebble | BlockShape::PebbleCluster | BlockShape::Rubble => (0.65, 0.45, 0.65),
        BlockShape::Cocoon => (0.75, 0.88, 0.75),
        _ => (1.0, 1.0, 1.0),
    };

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut uv_bs = Vec::with_capacity(24);
    let mut colors = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (face_dir, quad_positions, normal, face_uvs) in &ITEM_CUBE_FACES {
        let base_index = positions.len() as u32;

        let (layer, frames) =
            chunk_material
                .texture_registry
                .get_face_texture_info(voxel, IVec3::ZERO, *face_dir);
        let frame_count = if voxel.is_light() {
            -(frames.max(1) as f32)
        } else {
            frames as f32
        };

        let mut color = if voxel.is_grass_soil() {
            if *face_dir == FaceDirection::PositiveY {
                voxel.tint_color()
            } else {
                [1.0, 1.0, 1.0, 1.0]
            }
        } else {
            voxel.tint_color()
        };
        // Tag vertex color.a > 6.5 so voxel_opaque.wesl treats this mesh as a dynamic rotated item/held block
        // (preserving tint on all rotated faces and disabling world-space coated noise/texel-snap artifacts).
        color[3] = if voxel.is_painted() { 7.6 } else { 7.0 };

        for (i, &pos) in quad_positions.iter().enumerate() {
            positions.push([
                pos[0] * half_extent * scale_x,
                pos[1] * half_extent * scale_y,
                pos[2] * half_extent * scale_z,
            ]);
            normals.push(*normal);
            uv_bs.push([layer as f32, frame_count]);
            colors.push(color);
            uvs.push(face_uvs[i]);
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

fn create_cross_flora_item_mesh(
    half_extent: f32,
    voxel: Voxel,
    chunk_material: &ChunkMaterial,
) -> Mesh {
    let mut positions = Vec::with_capacity(16);
    let mut normals = Vec::with_capacity(16);
    let mut uvs = Vec::with_capacity(16);
    let mut uv_bs = Vec::with_capacity(16);
    let mut colors = Vec::with_capacity(16);
    let mut indices = Vec::with_capacity(24);

    let (layer, frames) = chunk_material.texture_registry.get_face_texture_info(
        voxel,
        IVec3::ZERO,
        FaceDirection::PositiveX,
    );
    let frame_count = if voxel.is_light() {
        -(frames.max(1) as f32)
    } else {
        frames as f32
    };
    let mut color = voxel.tint_color();
    color[3] = if voxel.is_painted() { 7.6 } else { 7.0 };

    let h = half_extent;
    let planes: [([[f32; 3]; 4], [f32; 3]); 4] = [
        (
            [[-h, -h, -h], [h, -h, h], [h, h, h], [-h, h, -h]],
            [0.707, 0.0, -0.707],
        ),
        (
            [[h, -h, h], [-h, -h, -h], [-h, h, -h], [h, h, h]],
            [-0.707, 0.0, 0.707],
        ),
        (
            [[-h, -h, h], [h, -h, -h], [h, h, -h], [-h, h, h]],
            [0.707, 0.0, 0.707],
        ),
        (
            [[h, -h, -h], [-h, -h, h], [-h, h, h], [h, h, -h]],
            [-0.707, 0.0, -0.707],
        ),
    ];
    let quad_uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

    for (verts, norm) in &planes {
        let base = positions.len() as u32;
        for (i, &v) in verts.iter().enumerate() {
            positions.push(v);
            normals.push(*norm);
            uvs.push(quad_uvs[i]);
            uv_bs.push([layer as f32, frame_count]);
            colors.push(color);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
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

/// Determines whether a dropped voxel item floats upward in water/fluids or sinks.
pub fn is_voxel_buoyant(voxel: Voxel) -> bool {
    if voxel.is_leaves_const()
        || voxel.is_flora_billboard()
        || voxel.default_shape() == BlockShape::CrossBillboard
    {
        return true;
    }
    matches!(
        SoundMaterial::from_voxel(voxel),
        SoundMaterial::Wood | SoundMaterial::Leaves | SoundMaterial::Snow
    )
}

fn handle_spawn_item_drop(
    event: On<SpawnItemDropEvent>,
    mut commands: Commands,
    chunk_material: Option<Res<ChunkMaterial>>,
    mut mesh_cache: ResMut<VoxelItemMeshCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rng_seed: Local<u64>,
) {
    let Some(chunk_material) = chunk_material else {
        return;
    };

    if event.voxel.is_empty() || event.voxel.is_fluid() {
        return;
    }

    if *rng_seed == 0 {
        *rng_seed = 0x517C_C1B7_2722_0A95;
    }
    *rng_seed = rng_seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let bob_phase = ((*rng_seed >> 32) as u32 as f32 / 4294967295.0) * std::f32::consts::TAU;

    let mesh_handle = mesh_cache.get_or_create_drop_mesh(event.voxel, &mut meshes, &chunk_material);
    let material_handle = chunk_material.opaque.clone();

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_translation(event.position),
        ItemDrop {
            voxel: event.voxel,
            velocity: event.velocity,
            age: 0.0,
            pickup_delay: event.pickup_delay,
            bob_phase,
            base_y: event.position.y,
            grounded: false,
            in_fluid: false,
        },
    ));
}

fn handle_block_break_spawn_drop(
    event: On<BlockBreakEvent>,
    mut commands: Commands,
    mut rng_seed: Local<u64>,
) {
    if event.voxel.is_empty() || event.voxel.is_fluid() {
        return;
    }

    let drop_voxel = event
        .voxel
        .double_tall_bottom_partner()
        .unwrap_or(event.voxel);

    if *rng_seed == 0 {
        *rng_seed = 0x85EB_CA6B_C2B2_AE35;
    }
    *rng_seed = rng_seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let rx = ((*rng_seed >> 32) as u32 as f32 / 4294967295.0) - 0.5;
    *rng_seed = rng_seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let rz = ((*rng_seed >> 32) as u32 as f32 / 4294967295.0) - 0.5;

    let position = event.position.as_vec3() + Vec3::new(0.5, 0.45, 0.5);
    let velocity = Vec3::new(rx * 1.8, 2.8, rz * 1.8);

    commands.trigger(SpawnItemDropEvent {
        position,
        velocity,
        voxel: drop_voxel,
        pickup_delay: 0.38,
    });
}

fn update_item_drops(
    mut commands: Commands,
    time: Res<Time>,
    menu_state: Option<Res<State<MenuState>>>,
    game_mode: Option<Res<GameMode>>,
    world: Option<Res<VoxelWorld>>,
    terrain_generator: Option<Res<TerrainGenerator>>,
    player_query: Query<&Transform, (With<Player>, Without<ItemDrop>)>,
    mut hotbar: Option<ResMut<Hotbar>>,
    mut player_inv: Option<ResMut<PlayerInventory>>,
    mut drops_query: Query<(Entity, &mut Transform, &mut ItemDrop)>,
) {
    let is_paused = menu_state.as_ref().is_some_and(|s| {
        matches!(
            *s.get(),
            MenuState::MainMenu
                | MenuState::RaceSelection
                | MenuState::Loading
                | MenuState::Pause
                | MenuState::Settings
                | MenuState::Controls
        )
    });
    if is_paused {
        return;
    }

    let dt = time.delta_secs().min(0.05);
    if dt <= 0.0 {
        return;
    }

    let Some(world) = world else {
        return;
    };

    let player_pos = player_query.iter().next().map(|t| t.translation);
    let is_spectator = game_mode.is_some_and(|gm| *gm == GameMode::Spectator);

    for (entity, mut transform, mut drop) in &mut drops_query {
        drop.age += dt;

        if drop.age >= DROP_MAX_LIFETIME || transform.translation.y < -64.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Continuous 3D Y-axis rotation
        transform.rotate_y(1.85 * dt);

        // 1. Check Player Magnetism / Suction & Pickup
        let mut magnetized = false;
        if drop.age >= drop.pickup_delay
            && !is_spectator
            && let Some(p_feet) = player_pos
        {
            let target_center = p_feet + Vec3::Y * 0.90;
            let to_player = target_center - transform.translation;
            let dist = to_player.length();

            if dist <= PICKUP_RADIUS {
                // Collect into Hotbar or PlayerInventory without triggering hotbar change detection unless a slot changes
                let mut collected = false;
                if let Some(ref mut hb) = hotbar {
                    if hb.as_ref().slots.contains(&Some(drop.voxel)) {
                        collected = true;
                    } else if let Some(empty_idx) = hb.as_ref().slots.iter().position(|s| s.is_none()) {
                        hb.slots[empty_idx] = Some(drop.voxel);
                        collected = true;
                    }
                }
                if !collected && let Some(ref mut inv) = player_inv {
                    if inv.as_ref().slots.contains(&Some(drop.voxel)) || inv.add_item(drop.voxel) {
                        collected = true;
                    } else {
                        collected = true;
                    }
                }

                if collected {
                    commands.entity(entity).despawn();
                    continue;
                }
            } else if dist <= MAGNET_RADIUS {
                magnetized = true;
                drop.grounded = false;
                let pull_factor = (1.0 - dist / MAGNET_RADIUS).clamp(0.15, 1.0);
                let desired_vel = to_player.normalize_or_zero() * (4.5 + pull_factor * 12.5);
                let blend = (14.0 * dt).clamp(0.0, 1.0);
                drop.velocity = drop.velocity.lerp(desired_vel, blend);
                transform.translation += drop.velocity * dt;
                drop.base_y = transform.translation.y;
            }
        }

        if magnetized {
            continue;
        }

        // 2. Check Water / Fluid Submersion & Buoyancy
        let sample_pos = Vec3::new(
            transform.translation.x,
            drop.base_y,
            transform.translation.z,
        );
        let iv = sample_pos.floor().as_ivec3();
        let in_fluid = world.get_voxel(iv).is_some_and(|v| v.is_fluid());
        drop.in_fluid = in_fluid;

        if in_fluid {
            drop.grounded = false;
            let buoyant = is_voxel_buoyant(drop.voxel);

            // Check if voxel directly above is air/non-fluid (surface equilibrium)
            let above_is_fluid = world
                .get_voxel(iv + IVec3::Y)
                .is_some_and(|v| v.is_fluid());
            let surface_y = iv.y as f32 + 0.88;

            if buoyant {
                if !above_is_fluid && drop.base_y >= surface_y - 0.12 {
                    // Float gently at water surface
                    let diff = surface_y - drop.base_y;
                    drop.velocity.y =
                        drop.velocity.y * (1.0 - 6.0 * dt).max(0.0) + diff * 8.0 * dt;
                } else {
                    // Rise upward through water
                    drop.velocity.y = (drop.velocity.y + 12.0 * dt).min(2.6);
                }
            } else {
                // Dense block items sink slowly with water drag
                drop.velocity.y = (drop.velocity.y - 5.5 * dt).max(-1.75);
            }

            // Apply downstream river flow current
            if let Some(ref terrain_gen) = terrain_generator {
                let flow = terrain_gen.river_flow_velocity(sample_pos.x, sample_pos.z);
                if flow.length_squared() > 0.001 {
                    let target_xz = Vec2::new(flow.x, flow.z) * 0.65;
                    drop.velocity.x += (target_xz.x - drop.velocity.x) * (3.0 * dt).min(1.0);
                    drop.velocity.z += (target_xz.y - drop.velocity.z) * (3.0 * dt).min(1.0);
                } else {
                    drop.velocity.x *= (1.0 - 3.5 * dt).max(0.0);
                    drop.velocity.z *= (1.0 - 3.5 * dt).max(0.0);
                }
            } else {
                drop.velocity.x *= (1.0 - 3.5 * dt).max(0.0);
                drop.velocity.z *= (1.0 - 3.5 * dt).max(0.0);
            }
        } else if !drop.grounded {
            // Air gravity & drag
            drop.velocity.y = (drop.velocity.y - 18.0 * dt).max(-28.0);
            drop.velocity.x *= (1.0 - 1.8 * dt).max(0.0);
            drop.velocity.z *= (1.0 - 1.8 * dt).max(0.0);
        } else {
            // Grounded: verify support block still exists underneath
            let below_iv = Vec3::new(
                sample_pos.x,
                drop.base_y - DROP_HALF_EXTENT - 0.10,
                sample_pos.z,
            )
            .floor()
            .as_ivec3();
            let supported = world.get_voxel(below_iv).is_some_and(|v| v.is_collidable());
            if !supported {
                drop.grounded = false;
            } else {
                drop.velocity = Vec3::ZERO;
            }
        }

        // 3. Integrate Velocity & Voxel Collisions
        if !drop.grounded {
            let next_x = sample_pos.x + drop.velocity.x * dt;
            let check_x_iv = Vec3::new(
                next_x + DROP_HALF_EXTENT * drop.velocity.x.signum(),
                drop.base_y,
                sample_pos.z,
            )
            .floor()
            .as_ivec3();
            if world
                .get_voxel(check_x_iv)
                .is_some_and(|v| v.is_collidable())
            {
                drop.velocity.x = -drop.velocity.x * 0.35;
            } else {
                transform.translation.x = next_x;
            }

            let next_z = sample_pos.z + drop.velocity.z * dt;
            let check_z_iv = Vec3::new(
                transform.translation.x,
                drop.base_y,
                next_z + DROP_HALF_EXTENT * drop.velocity.z.signum(),
            )
            .floor()
            .as_ivec3();
            if world
                .get_voxel(check_z_iv)
                .is_some_and(|v| v.is_collidable())
            {
                drop.velocity.z = -drop.velocity.z * 0.35;
            } else {
                transform.translation.z = next_z;
            }

            let next_y = drop.base_y + drop.velocity.y * dt;
            let foot_y = next_y - DROP_HALF_EXTENT;
            let check_y_iv = Vec3::new(
                transform.translation.x,
                foot_y,
                transform.translation.z,
            )
            .floor()
            .as_ivec3();

            if drop.velocity.y <= 0.0
                && world
                    .get_voxel(check_y_iv)
                    .is_some_and(|v| v.is_collidable())
            {
                let floor_top = check_y_iv.y as f32 + 1.0;
                drop.base_y = floor_top + DROP_HALF_EXTENT + 0.06;
                if drop.velocity.y < -2.2 && !in_fluid {
                    drop.velocity.y = -drop.velocity.y * 0.28;
                    drop.velocity.x *= 0.55;
                    drop.velocity.z *= 0.55;
                } else {
                    drop.velocity = Vec3::ZERO;
                    if !in_fluid {
                        drop.grounded = true;
                    }
                }
            } else {
                drop.base_y = next_y;
            }
        }

        // 4. Apply Smooth Vertical Bobbing Offset
        let bob_offset = if drop.grounded || drop.in_fluid {
            (drop.age * 2.8 + drop.bob_phase).sin() * 0.055
        } else {
            0.0
        };
        transform.translation.y = drop.base_y + bob_offset;
    }
}

fn clear_item_drops_on_main_menu(
    mut commands: Commands,
    drops_query: Query<Entity, With<ItemDrop>>,
) {
    for entity in &drops_query {
        commands.entity(entity).despawn();
    }
}

pub struct ItemDropPlugin;

impl Plugin for ItemDropPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelItemMeshCache>()
            .add_observer(handle_spawn_item_drop)
            .add_observer(handle_block_break_spawn_drop)
            .add_systems(OnEnter(MenuState::MainMenu), clear_item_drops_on_main_menu)
            .add_systems(Update, update_item_drops);
    }
}
