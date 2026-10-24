use bevy::{
    audio::{AudioPlayer, AudioSink, PlaybackSettings, Volume},
    prelude::*,
};

use super::settings::AudioSettings;

pub const GAME_MUSIC_TRACKS: [&str; 39] = [
    "sounds/music/game/a_familiar_room.ogg",
    "sounds/music/game/an_ordinary_day.ogg",
    "sounds/music/game/ancestry.ogg",
    "sounds/music/game/below_and_above.ogg",
    "sounds/music/game/broken_clocks.ogg",
    "sounds/music/game/bromeliad.ogg",
    "sounds/music/game/clark.ogg",
    "sounds/music/game/comforting_memories.ogg",
    "sounds/music/game/crescent_dunes.ogg",
    "sounds/music/game/danny.ogg",
    "sounds/music/game/deeper.ogg",
    "sounds/music/game/dry_hands.ogg",
    "sounds/music/game/echo_in_the_wind.ogg",
    "sounds/music/game/eld_unknown.ogg",
    "sounds/music/game/endless.ogg",
    "sounds/music/game/featherfall.ogg",
    "sounds/music/game/fireflies.ogg",
    "sounds/music/game/floating_dream.ogg",
    "sounds/music/game/haggstrom.ogg",
    "sounds/music/game/infinite_amethyst.ogg",
    "sounds/music/game/key.ogg",
    "sounds/music/game/komorebi.ogg",
    "sounds/music/game/left_to_bloom.ogg",
    "sounds/music/game/lilypad.ogg",
    "sounds/music/game/living_mice.ogg",
    "sounds/music/game/mice_on_venus.ogg",
    "sounds/music/game/minecraft.ogg",
    "sounds/music/game/one_more_day.ogg",
    "sounds/music/game/os_piano.ogg",
    "sounds/music/game/oxygene.ogg",
    "sounds/music/game/pokopoko.ogg",
    "sounds/music/game/puzzlebox.ogg",
    "sounds/music/game/stand_tall.ogg",
    "sounds/music/game/subwoofer_lullaby.ogg",
    "sounds/music/game/sweden.ogg",
    "sounds/music/game/watcher.ogg",
    "sounds/music/game/wending.ogg",
    "sounds/music/game/wet_hands.ogg",
    "sounds/music/game/yakusoku.ogg",
];

#[derive(Component)]
pub struct ActiveMusicTrack;

#[derive(Resource)]
pub struct MusicState {
    pub current_track_idx: usize,
    pub is_playing: bool,
    pub play_timer: f32,
    pub track_estimated_duration: f32,
    pub delay_timer: f32,
    pub next_delay: f32,
    pub fade_progress: f32,
    pub active_entity: Option<Entity>,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            current_track_idx: 0,
            is_playing: false,
            play_timer: 0.0,
            track_estimated_duration: 180.0,
            // Initial 6-second peaceful breather before starting the first track
            delay_timer: 0.0,
            next_delay: 6.0,
            fade_progress: 0.0,
            active_entity: None,
        }
    }
}

pub struct GameplayMusicPlugin;

impl Plugin for GameplayMusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>()
            .add_systems(Update, update_gameplay_music);
    }
}

/// Manages background gameplay soundtrack progression, smooth cross-fades, and natural intervals.
fn update_gameplay_music(
    time: Res<Time>,
    settings: Res<AudioSettings>,
    mut music_state: ResMut<MusicState>,
    asset_server: Res<AssetServer>,
    mut sinks: Query<&mut AudioSink, With<ActiveMusicTrack>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let target_volume = settings.master_volume * settings.music_volume;

    if music_state.is_playing {
        music_state.play_timer += dt;

        // Smooth 4-second fade in
        if music_state.play_timer < 4.0 {
            music_state.fade_progress = (music_state.play_timer / 4.0).clamp(0.0, 1.0);
        } else if music_state.play_timer > (music_state.track_estimated_duration - 4.0) {
            // Smooth 4-second fade out
            let remain = music_state.track_estimated_duration - music_state.play_timer;
            music_state.fade_progress = (remain / 4.0).clamp(0.0, 1.0);
        } else {
            music_state.fade_progress = 1.0;
        }

        // Apply faded volume to audio sink
        for mut sink in &mut sinks {
            sink.set_volume(Volume::Linear(target_volume * music_state.fade_progress));
        }

        // Track finished: clean up and enter organic inter-track pause (60 to 150 seconds)
        if music_state.play_timer >= music_state.track_estimated_duration {
            if let Some(entity) = music_state.active_entity.take() {
                commands.entity(entity).despawn();
            }
            music_state.is_playing = false;
            music_state.play_timer = 0.0;
            music_state.delay_timer = 0.0;

            // Organic Minecraft-style pause between tracks
            let pseudo_rand = (time.elapsed_secs() * 37.0).fract();
            music_state.next_delay = 60.0 + pseudo_rand * 90.0;
        }
    } else {
        music_state.delay_timer += dt;

        if music_state.delay_timer >= music_state.next_delay && target_volume > 0.01 {
            // Pick next random track different from the previous one
            let track_count = GAME_MUSIC_TRACKS.len();
            let rand_step = 1 + ((time.elapsed_secs() * 13.0) as usize % (track_count - 1));
            let next_idx = (music_state.current_track_idx + rand_step) % track_count;
            music_state.current_track_idx = next_idx;

            let path = GAME_MUSIC_TRACKS[next_idx];
            let handle: Handle<AudioSource> = asset_server.load(path);

            let entity = commands
                .spawn((
                    ActiveMusicTrack,
                    AudioPlayer(handle),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.0)),
                ))
                .id();

            music_state.active_entity = Some(entity);
            music_state.is_playing = true;
            music_state.play_timer = 0.0;
            music_state.fade_progress = 0.0;
            // Estimated typical track length ~190 seconds
            music_state.track_estimated_duration = 190.0;
        }
    }
}
