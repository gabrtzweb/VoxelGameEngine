use bevy::{
    gizmos::config::{DefaultGizmoConfigGroup, GizmoConfigStore},
    prelude::*,
};

use super::{
    chunk::{CHUNK_SIZE, VOXEL_SIZE, Voxel},
    world::{CHUNK_WORLD_SIZE, VoxelWorld},
};

const VOXELS_PER_BLOCK: i32 = 2;
const BLOCK_SIZE: f32 = VOXEL_SIZE * VOXELS_PER_BLOCK as f32;
const DEBUG_RENDER_DISTANCE: f32 = 24.0;

const NEIGHBOR_DIRECTIONS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

#[derive(Resource)]
pub struct VoxelDebugSettings {
    pub enabled: bool,
}

impl Default for VoxelDebugSettings {
    fn default() -> Self {
        Self { enabled: false }
    }
}

pub struct VoxelDebugPlugin;

impl Plugin for VoxelDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelDebugSettings>()
            .add_systems(Startup, configure_gizmos)
            .add_systems(Update, (toggle_debug, draw_voxel_debug));
    }
}

fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();

    config.line.width = 1.5;
    config.depth_bias = -0.0001;
}

fn toggle_debug(keyboard: Res<ButtonInput<KeyCode>>, mut settings: ResMut<VoxelDebugSettings>) {
    if keyboard.just_pressed(KeyCode::F3) {
        settings.enabled = !settings.enabled;

        info!(
            "Voxel debug: {}",
            if settings.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}

fn draw_voxel_debug(
    world: Res<VoxelWorld>,
    settings: Res<VoxelDebugSettings>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    mut gizmos: Gizmos,
) {
    if !settings.enabled {
        return;
    }

    let camera_position = camera.translation();

    draw_chunk_outlines(&world, &mut gizmos, camera_position);

    draw_block_outlines(&world, &mut gizmos, camera_position);
}

fn draw_chunk_outlines(world: &VoxelWorld, gizmos: &mut Gizmos, camera_position: Vec3) {
    let chunk_color = Color::srgba(0.1, 0.75, 1.0, 0.9);

    for (&chunk_coordinate, _) in world.iter_chunks() {
        let chunk_origin = VoxelWorld::chunk_translation(chunk_coordinate);

        let chunk_center = chunk_origin + Vec3::splat(CHUNK_WORLD_SIZE * 0.5);

        if !is_within_debug_distance(camera_position, chunk_center) {
            continue;
        }

        gizmos.cube(
            Transform::from_translation(chunk_center).with_scale(Vec3::splat(CHUNK_WORLD_SIZE)),
            chunk_color,
        );
    }
}

fn draw_block_outlines(world: &VoxelWorld, gizmos: &mut Gizmos, camera_position: Vec3) {
    let block_color = Color::srgba(0.02, 0.02, 0.02, 0.8);

    let blocks_per_chunk = CHUNK_SIZE as i32 / VOXELS_PER_BLOCK;

    for (&chunk_coordinate, _) in world.iter_chunks() {
        let chunk_voxel_origin = chunk_coordinate * CHUNK_SIZE as i32;

        for block_y in 0..blocks_per_chunk {
            for block_z in 0..blocks_per_chunk {
                for block_x in 0..blocks_per_chunk {
                    let block_origin = chunk_voxel_origin
                        + IVec3::new(
                            block_x * VOXELS_PER_BLOCK,
                            block_y * VOXELS_PER_BLOCK,
                            block_z * VOXELS_PER_BLOCK,
                        );

                    let block_center = get_block_center(block_origin);

                    if !is_within_debug_distance(camera_position, block_center) {
                        continue;
                    }

                    if count_solid_voxels(world, block_origin) == 0 {
                        continue;
                    }

                    if block_is_exposed(world, block_origin) {
                        draw_block_outline(gizmos, block_origin, block_color);
                    }
                }
            }
        }
    }
}

fn draw_block_outline(gizmos: &mut Gizmos, block_origin: IVec3, color: Color) {
    let center = get_block_center(block_origin);

    gizmos.cube(
        Transform::from_translation(center).with_scale(Vec3::splat(BLOCK_SIZE)),
        color,
    );
}

fn count_solid_voxels(world: &VoxelWorld, block_origin: IVec3) -> usize {
    let mut count = 0;

    for y in 0..VOXELS_PER_BLOCK {
        for z in 0..VOXELS_PER_BLOCK {
            for x in 0..VOXELS_PER_BLOCK {
                let position = block_origin + IVec3::new(x, y, z);

                if world.get_voxel(position) == Some(Voxel::Solid) {
                    count += 1;
                }
            }
        }
    }

    count
}

fn block_is_exposed(world: &VoxelWorld, block_origin: IVec3) -> bool {
    for y in 0..VOXELS_PER_BLOCK {
        for z in 0..VOXELS_PER_BLOCK {
            for x in 0..VOXELS_PER_BLOCK {
                let position = block_origin + IVec3::new(x, y, z);

                if world.get_voxel(position) != Some(Voxel::Solid) {
                    continue;
                }

                if voxel_is_exposed(world, position) {
                    return true;
                }
            }
        }
    }

    false
}

fn voxel_is_exposed(world: &VoxelWorld, position: IVec3) -> bool {
    for direction in NEIGHBOR_DIRECTIONS {
        let neighbor = position + direction;

        if world.get_voxel(neighbor) == Some(Voxel::Air) {
            return true;
        }
    }

    false
}

fn get_block_center(block_origin: IVec3) -> Vec3 {
    (block_origin.as_vec3() + Vec3::splat(VOXELS_PER_BLOCK as f32 * 0.5)) * VOXEL_SIZE
}

fn is_within_debug_distance(camera_position: Vec3, position: Vec3) -> bool {
    camera_position.distance_squared(position) <= DEBUG_RENDER_DISTANCE * DEBUG_RENDER_DISTANCE
}
