use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    text::FontSize,
};

use crate::{
    environment::EnvironmentState,
    player::{GameMode, Player, PlayerMotion},
    voxel::{
        CHUNK_VOLUME, ChunkMeshRegistry, InteractionMode, VOXEL_SIZE, VoxelWorld,
        targeting::CurrentTarget,
    },
};

const STATS_UPDATE_INTERVAL: f32 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugHudMode {
    #[default]
    Minimal,
    Extended,
    Hidden,
}

#[derive(Resource, Debug, Clone)]
pub struct DebugHudSettings {
    pub mode: DebugHudMode,
    pub prev_mode: DebugHudMode,
}

impl Default for DebugHudSettings {
    fn default() -> Self {
        Self {
            mode: DebugHudMode::Minimal,
            prev_mode: DebugHudMode::Minimal,
        }
    }
}

impl DebugHudSettings {
    pub fn toggle_mode(&mut self) {
        match self.mode {
            DebugHudMode::Minimal => {
                self.mode = DebugHudMode::Extended;
                self.prev_mode = DebugHudMode::Extended;
            }
            DebugHudMode::Extended => {
                self.mode = DebugHudMode::Minimal;
                self.prev_mode = DebugHudMode::Minimal;
            }
            DebugHudMode::Hidden => {
                self.mode = if self.prev_mode != DebugHudMode::Hidden {
                    self.prev_mode
                } else {
                    DebugHudMode::Minimal
                };
            }
        }
    }

    pub fn toggle_hidden(&mut self) {
        if self.mode == DebugHudMode::Hidden {
            self.mode = if self.prev_mode != DebugHudMode::Hidden {
                self.prev_mode
            } else {
                DebugHudMode::Minimal
            };
        } else {
            self.prev_mode = self.mode;
            self.mode = DebugHudMode::Hidden;
        }
    }
}

#[derive(Component)]
struct DevStatsText;

pub struct DevStatsPlugin;

impl Plugin for DevStatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugHudSettings>()
            .add_systems(Startup, spawn_dev_stats)
            .add_systems(Update, (handle_debug_hud_input, update_dev_stats).chain());
    }
}

fn spawn_dev_stats(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(13.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            padding: UiRect::axes(px(8), px(6)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.12)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ZIndex(200),
        Visibility::Visible,
        DevStatsText,
    ));
}

fn handle_debug_hud_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<DebugHudSettings>,
) {
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if keyboard.just_pressed(KeyCode::F3) {
        if shift_held {
            settings.toggle_hidden();
        } else {
            settings.toggle_mode();
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_dev_stats(
    hud_settings: Res<DebugHudSettings>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time<Real>>,
    world: Res<VoxelWorld>,
    chunk_meshes: Res<ChunkMeshRegistry>,
    game_mode: Res<GameMode>,
    interaction_mode: Res<InteractionMode>,
    environment: Option<Res<EnvironmentState>>,
    player: Single<(&Transform, &PlayerMotion), With<Player>>,
    camera: Single<&Transform, (With<Camera3d>, Without<Player>)>,
    current_target: Res<CurrentTarget>,
    text_query: Single<
        (
            &mut Text,
            &mut Visibility,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<DevStatsText>,
    >,
    mut update_timer: Local<f32>,
    mut last_mode: Local<Option<DebugHudMode>>,
) {
    let (mut text, mut visibility, mut bg_color, mut border_color) = text_query.into_inner();

    let mode = hud_settings.mode;
    let mode_changed = *last_mode != Some(mode);
    *last_mode = Some(mode);

    if mode == DebugHudMode::Hidden {
        *visibility = Visibility::Hidden;
        if !text.0.is_empty() {
            text.0.clear();
        }
        return;
    }

    *visibility = Visibility::Visible;

    *update_timer += time.delta_secs();
    if !mode_changed && *update_timer < STATS_UPDATE_INTERVAL {
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

    let (player_transform, player_motion) = player.into_inner();
    let player_position = player_transform.translation;

    let target_text = if let Some(target) = current_target.hit {
        format!(
            "{}, {}, {}",
            target.hit_voxel.x, target.hit_voxel.y, target.hit_voxel.z,
        )
    } else {
        "None".to_string()
    };

    match mode {
        DebugHudMode::Minimal => {
            *bg_color = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55));
            *border_color = BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.10));

            let (h, m, day_of_month, month, season_str) = if let Some(ref env) = environment {
                let hours = (env.time_of_day * 24.0 + 6.0).rem_euclid(24.0);
                let h = hours.floor() as u32;
                let m = ((hours - hours.floor()) * 60.0).floor() as u32;
                (h, m, env.day_of_month(), env.month(), env.season().name())
            } else {
                (0, 0, 1, 1, "Spring")
            };

            text.0 = format!(
                "FPS: {fps:.0} ({frame_time:.2} ms)\n\
                Time: {h:02}:{m:02} | Day {day_of_month} (M{month}, {season_str})\n\
                Pos: {:.1} / {:.1} / {:.1}\n\
                Edit Mode: {}\n\
                Target: {target_text}",
                player_position.x,
                player_position.y,
                player_position.z,
                interaction_mode.label(),
            );
        }
        DebugHudMode::Extended => {
            *bg_color = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72));
            *border_color = BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18));

            let loaded_chunks = world.iter_chunks().count();
            let meshed_chunks = chunk_meshes.len();
            let mesh_vertices = chunk_meshes.total_vertices();
            let mesh_triangles = chunk_meshes.total_triangles();

            let player_block = IVec3::new(
                player_position.x.floor() as i32,
                player_position.y.floor() as i32,
                player_position.z.floor() as i32,
            );

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

            let env_text = if let Some(ref env) = environment {
                let hours = (env.time_of_day * 24.0 + 6.0).rem_euclid(24.0);
                let h = hours.floor() as u32;
                let m = ((hours - hours.floor()) * 60.0).floor() as u32;
                format!(
                    "{:?} ({h:02}:{m:02}), Day {} (M{}, Y{}, {}, Day {}/84), Moon: {}",
                    env.phase,
                    env.day_of_month(),
                    env.month(),
                    env.year(),
                    env.season().name(),
                    env.day_of_season(),
                    env.moon_phase_name()
                )
            } else {
                "N/A".to_string()
            };

            text.0 = format!(
                "FPS: {fps:.1}\n\
                Frame: {frame_time:.2} ms\n\
                Mode: {}\n\
                Flight: {}\n\
                Time: {env_text}\n\
                Position: {}, {}, {}\n\
                Player chunk: {}, {}, {}\n\
                Camera: {:.1}, {:.1}, {:.1}\n\
                Loaded chunks: {}\n\
                Meshed chunks: {}\n\
                Mesh vertices: {}\n\
                Mesh triangles: {}\n\
                Voxel capacity: {}\n\
                Target voxel: {}\n\
                Edit Mode: {} [F3: Minimal | Shift+F3: Hide]",
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
                mesh_vertices,
                mesh_triangles,
                loaded_chunks * CHUNK_VOLUME,
                target_text,
                interaction_mode.label(),
            );
        }
        DebugHudMode::Hidden => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_hud_settings_toggles_cleanly() {
        let mut settings = DebugHudSettings::default();
        assert_eq!(settings.mode, DebugHudMode::Minimal);

        settings.toggle_mode();
        assert_eq!(settings.mode, DebugHudMode::Extended);

        settings.toggle_mode();
        assert_eq!(settings.mode, DebugHudMode::Minimal);

        // Hide HUD
        settings.toggle_hidden();
        assert_eq!(settings.mode, DebugHudMode::Hidden);

        // Unhide restores previous mode
        settings.toggle_hidden();
        assert_eq!(settings.mode, DebugHudMode::Minimal);

        // Switch to Extended then hide
        settings.toggle_mode();
        assert_eq!(settings.mode, DebugHudMode::Extended);
        settings.toggle_hidden();
        assert_eq!(settings.mode, DebugHudMode::Hidden);

        // Pressing F3 (toggle_mode) while hidden restores mode
        settings.toggle_mode();
        assert_eq!(settings.mode, DebugHudMode::Extended);
    }
}
