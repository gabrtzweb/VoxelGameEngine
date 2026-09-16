use bevy::prelude::*;

use super::{
    Player,
    water::{is_point_in_water, player_submersion},
};
use crate::voxel::VoxelWorld;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PlayerEnvironmentStatus {
    pub is_camera_in_water: bool,
    pub submersion: f32,
}

pub fn update_player_environment_status(
    world: Option<Res<VoxelWorld>>,
    player_query: Option<Single<&Transform, With<Player>>>,
    camera_query: Option<Single<&GlobalTransform, With<Camera3d>>>,
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
        .map(|player| player_submersion(&world, player.translation))
        .unwrap_or(0.0);

    status.is_camera_in_water = is_camera_in_water;
    status.submersion = submersion;
}
