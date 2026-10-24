use bevy::{
    audio::{AudioPlayer, PlaybackSettings, Volume},
    prelude::*,
};

use super::settings::AudioSettings;
use crate::player::PlayerCamera;

/// Event to trigger a 2D global sound effect.
#[derive(Event, Debug, Clone)]
pub struct PlaySfxEvent {
    pub sound: Handle<AudioSource>,
    pub volume: f32,
}

/// Event to trigger a 3D spatially attenuated sound effect.
#[derive(Event, Debug, Clone)]
pub struct PlaySpatialSfxEvent {
    pub sound: Handle<AudioSource>,
    pub position: Vec3,
    pub volume: f32,
    pub max_distance: f32,
}

pub struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_play_sfx)
            .add_observer(handle_play_spatial_sfx);
    }
}

fn handle_play_sfx(
    trigger: On<PlaySfxEvent>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    let final_vol = (ev.volume * settings.master_volume * settings.sfx_volume).clamp(0.0, 1.0);
    if final_vol <= 0.001 {
        return;
    }

    commands.spawn((
        AudioPlayer(ev.sound.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(final_vol)),
    ));
}

fn handle_play_spatial_sfx(
    trigger: On<PlaySpatialSfxEvent>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    let Ok(cam_transform) = camera_query.single() else {
        return;
    };
    let listener_pos = cam_transform.translation;
    let ev = trigger.event();

    let dist = listener_pos.distance(ev.position);
    if dist > ev.max_distance {
        return;
    }

    let dist_factor = (1.0 - (dist / ev.max_distance)).clamp(0.0, 1.0);
    let final_vol = (ev.volume * dist_factor * settings.master_volume * settings.sfx_volume)
        .clamp(0.0, 1.0);

    if final_vol <= 0.001 {
        return;
    }

    commands.spawn((
        AudioPlayer(ev.sound.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(final_vol)),
    ));
}
