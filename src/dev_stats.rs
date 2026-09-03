use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    text::FontSize,
};

use crate::{
    player::{GameMode, Player, PlayerMotion},
    voxel::{CHUNK_VOLUME, ChunkMeshRegistry, VOXEL_SIZE, VoxelWorld, targeting::CurrentTarget},
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
    game_mode: Res<GameMode>,
    player: Single<(&Transform, &PlayerMotion), With<Player>>,
    camera: Single<&Transform, (With<Camera3d>, Without<Player>)>,
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

    let (player_transform, player_motion) = player.into_inner();

    let player_position = player_transform.translation;

    // Logical 1 x 1 x 1 meter block coordinates.
    let player_block = IVec3::new(
        player_position.x.floor() as i32,
        player_position.y.floor() as i32,
        player_position.z.floor() as i32,
    );

    // Internal 0.5 meter voxel coordinates.
    let player_voxel = IVec3::new(
        (player_position.x / VOXEL_SIZE).floor() as i32,
        (player_position.y / VOXEL_SIZE).floor() as i32,
        (player_position.z / VOXEL_SIZE).floor() as i32,
    );

    let (player_chunk, _) = VoxelWorld::world_voxel_to_chunk(player_voxel);

    let camera_position = camera.translation;

    let flight_text = match *game_mode {
        GameMode::Creative => {
            if player_motion.flying {
                "On"
            } else {
                "Off"
            }
        }
        GameMode::Spectator => "N/A",
    };

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
        Mode: {}\n\
        Flight: {}\n\
        Position: {}, {}, {}\n\
        Player chunk: {}, {}, {}\n\
        Camera: {:.1}, {:.1}, {:.1}\n\
        Loaded chunks: {}\n\
        Meshed chunks: {}\n\
        Voxel capacity: {}\n\
        Target voxel: {}",
        game_mode.label(),
        flight_text,
        player_block.x,
        player_block.y,
        player_block.z,
        player_chunk.x,
        player_chunk.y,
        player_chunk.z,
        camera_position.x,
        camera_position.y,
        camera_position.z,
        loaded_chunks,
        meshed_chunks,
        loaded_chunks * CHUNK_VOLUME,
        target_text,
    );
}
