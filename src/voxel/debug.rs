use bevy::{
    gizmos::config::{DefaultGizmoConfigGroup, GizmoConfigStore},
    prelude::*,
};

use super::{
    render::ChunkMeshRegistry,
    world::{CHUNK_WORLD_SIZE, VoxelWorld},
};

const DEBUG_RENDER_DISTANCE: f32 = 96.0;

#[derive(Resource, Default)]
pub struct VoxelDebugSettings {
    pub enabled: bool,
}

pub struct VoxelDebugPlugin;

impl Plugin for VoxelDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelDebugSettings>()
            .add_systems(Startup, configure_gizmos)
            .add_systems(Update, (toggle_debug, draw_chunk_outlines));
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
            "Chunk debug: {}",
            if settings.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}

fn draw_chunk_outlines(
    settings: Res<VoxelDebugSettings>,
    registry: Res<ChunkMeshRegistry>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    mut gizmos: Gizmos,
) {
    if !settings.enabled {
        return;
    }

    let camera_position = camera.translation();

    let chunk_color = Color::srgba(0.1, 0.75, 1.0, 0.9);

    for &coordinate in registry.iter_coordinates() {
        let chunk_origin = VoxelWorld::chunk_translation(coordinate);

        let chunk_center = chunk_origin + Vec3::splat(CHUNK_WORLD_SIZE * 0.5);

        if camera_position.distance_squared(chunk_center)
            > DEBUG_RENDER_DISTANCE * DEBUG_RENDER_DISTANCE
        {
            continue;
        }

        gizmos.cube(
            Transform::from_translation(chunk_center).with_scale(Vec3::splat(CHUNK_WORLD_SIZE)),
            chunk_color,
        );
    }
}
