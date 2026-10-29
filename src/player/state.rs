use bevy::prelude::*;

use super::{
    Player, PlayerCamera,
    water::{is_point_in_water, player_submersion},
};
use crate::{
    generation::{BiomeType, TerrainGenerator},
    world::VoxelWorld,
};

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PlayerEnvironmentStatus {
    pub is_camera_in_water: bool,
    pub submersion: f32,
    pub current_biome: BiomeType,
    pub is_underground: bool,
    pub underground_factor: f32,
    pub depth_below_surface: f32,
    pub has_roof_overhead: bool,
}

type CameraTransformQuery<'w, 's> =
    Option<Single<'w, 's, &'static GlobalTransform, (With<Camera3d>, With<PlayerCamera>)>>;

pub fn update_player_environment_status(
    time: Option<Res<Time>>,
    world: Option<Res<VoxelWorld>>,
    generator: Option<Res<TerrainGenerator>>,
    player_query: Option<Single<&Transform, With<Player>>>,
    camera_query: CameraTransformQuery,
    mut status: ResMut<PlayerEnvironmentStatus>,
) {
    let Some(world) = world else {
        *status = PlayerEnvironmentStatus::default();
        return;
    };

    let is_camera_in_water = camera_query
        .as_ref()
        .map(|cam| is_point_in_water(&world, cam.translation()))
        .unwrap_or(false);

    let submersion = player_query
        .as_ref()
        .map(|player| player_submersion(&world, player.translation))
        .unwrap_or(0.0);

    let (current_biome, is_underground, depth_below_surface, has_roof, target_underground_factor) = if let (
        Some(terrain_gen),
        Some(pos),
    ) = (
        generator.as_ref(),
        camera_query
            .map(|c| c.translation())
            .or_else(|| player_query.as_ref().map(|p| p.translation)),
    ) {
        let block_x = (pos.x / crate::world::VOXEL_SIZE).floor() as i32;
        let block_y = (pos.y / crate::world::VOXEL_SIZE).floor() as i32;
        let block_z = (pos.z / crate::world::VOXEL_SIZE).floor() as i32;
        let col = terrain_gen.sample_column(block_x, block_z);
        let depth = (col.terrain_height as f32 - pos.y).max(0.0);

        // Detect solid roof overhead (cave ceiling, tunnel, or mine roof)
        let block_pos = IVec3::new(block_x, block_y, block_z);
        let mut roof = false;
        for dy in 1..=14 {
            let check_pos = block_pos + IVec3::new(0, dy, 0);
            if let Some(vox) = world.get_voxel(check_pos) {
                if vox.is_solid_opaque() || vox.is_leaves() {
                    roof = true;
                    break;
                }
            }
        }

        let underground = pos.y < (col.terrain_height as f32 - 0.5) || (roof && depth > 0.0);
        let target_factor = if roof {
            (0.70 + (depth / 10.0) * 0.30).clamp(0.70, 1.0)
        } else if pos.y < col.terrain_height as f32 {
            (depth / 6.0).clamp(0.0, 1.0)
        } else {
            0.0
        };

        (col.biome, underground, depth, roof, target_factor)
    } else {
        (BiomeType::Plains, false, 0.0, false, 0.0)
    };

    let dt = time.as_ref().map(|t| t.delta_secs()).unwrap_or(0.016);
    // Smooth eye adaptation transition into and out of caverns
    status.underground_factor += (target_underground_factor - status.underground_factor) * (3.5 * dt).min(1.0);
    status.is_camera_in_water = is_camera_in_water;
    status.submersion = submersion;
    status.current_biome = current_biome;
    status.is_underground = is_underground;
    status.depth_below_surface = depth_below_surface;
    status.has_roof_overhead = has_roof;
}
