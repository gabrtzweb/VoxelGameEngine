use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    text::FontSize,
};

use crate::voxel::{
    CHUNK_VOLUME, ChunkMeshRegistry, VOXEL_SIZE, VoxelWorld, targeting::CurrentTarget,
};

const STATS_UPDATE_INTERVAL: f32 = 0.25;

#[derive(Component)]
struct DevStatsText;

pub struct DevStatsPlugin;

impl Plugin for DevStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_dev_stats)
            .add_systems(Update, update_dev_stats);
    }
}

fn spawn_dev_stats(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            padding: UiRect::all(px(8)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ZIndex(200),
        DevStatsText,
    ));
}

fn update_dev_stats(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    world: Res<VoxelWorld>,
    chunk_meshes: Res<ChunkMeshRegistry>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    current_target: Res<CurrentTarget>,
    mut text: Single<&mut Text, With<DevStatsText>>,
    mut update_timer: Local<f32>,
) {
    *update_timer += time.delta_secs();

    if *update_timer < STATS_UPDATE_INTERVAL {
        return;
    }

    *update_timer = 0.0;

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diagnostic| diagnostic.smoothed())
        .unwrap_or(0.0);

    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diagnostic| diagnostic.smoothed())
        .unwrap_or(0.0);

    let loaded_chunks = world.iter_chunks().count();

    let meshed_chunks = chunk_meshes.len();

    let camera_position = camera.translation();

    let camera_voxel = IVec3::new(
        (camera_position.x / VOXEL_SIZE).floor() as i32,
        (camera_position.y / VOXEL_SIZE).floor() as i32,
        (camera_position.z / VOXEL_SIZE).floor() as i32,
    );

    let (camera_chunk, _) = VoxelWorld::world_voxel_to_chunk(camera_voxel);

    let target_text = if let Some(target) = current_target.hit {
        format!(
            "{}, {}, {}",
            target.hit_voxel.x, target.hit_voxel.y, target.hit_voxel.z,
        )
    } else {
        "None".to_string()
    };

    text.0 = format!(
        "FPS: {fps:.1}\n\
         Frame: {frame_time:.2} ms\n\
         Loaded chunks: {loaded_chunks}\n\
         Meshed chunks: {meshed_chunks}\n\
         Voxel capacity: {}\n\
         Camera: {:.1}, {:.1}, {:.1}\n\
         Camera chunk: {}, {}, {}\n\
         Target voxel: {}",
        loaded_chunks * CHUNK_VOLUME,
        camera_position.x,
        camera_position.y,
        camera_position.z,
        camera_chunk.x,
        camera_chunk.y,
        camera_chunk.z,
        target_text,
    );
}
