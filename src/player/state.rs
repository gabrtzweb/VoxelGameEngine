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
}

type CameraTransformQuery<'w, 's> =
    Option<Single<'w, 's, &'static GlobalTransform, (With<Camera3d>, With<PlayerCamera>)>>;

pub fn update_player_environment_status(
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
        .map(|cam| is_point_in_water(&world, cam.translation()))
        .unwrap_or(false);

    let submersion = player_query
        .as_ref()
        .map(|player| player_submersion(&world, player.translation))
        .unwrap_or(0.0);

    let current_biome = if let (Some(terrain_gen), Some(player)) = (generator.as_ref(), player_query.as_ref()) {
        let block_x = (player.translation.x / crate::world::VOXEL_SIZE).floor() as i32;
        let block_z = (player.translation.z / crate::world::VOXEL_SIZE).floor() as i32;
        terrain_gen.sample_column(block_x, block_z).biome
    } else {
        BiomeType::Steppe
    };

    status.is_camera_in_water = is_camera_in_water;
    status.submersion = submersion;
    status.current_biome = current_biome;
}
