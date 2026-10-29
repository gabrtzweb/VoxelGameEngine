use bevy::{
    audio::{AudioPlayer, AudioSink, PlaybackSettings, Volume},
    prelude::*,
};

use super::settings::AudioSettings;

pub const MENU_MUSIC_TRACKS: [&str; 4] = [
    "sounds/music/menu/beginning_2.ogg",
    "sounds/music/menu/floating_trees.ogg",
    "sounds/music/menu/moog_city_2.ogg",
    "sounds/music/menu/mutation.ogg",
];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MusicContext {
    #[default]
    Menu,
    Game,
}

#[derive(Resource)]
pub struct MusicState {
    pub context: MusicContext,
    pub menu_playlist: Vec<usize>,
    pub menu_idx: usize,
    pub game_playlist: Vec<usize>,
    pub game_idx: usize,
    pub is_playing: bool,
    pub play_timer: f32,
    pub track_estimated_duration: f32,
    pub delay_timer: f32,
    pub next_delay: f32,
    pub fade_progress: f32,
    pub active_entity: Option<Entity>,
    pub sink_attached: bool,
    rng_state: u64,
}

impl MusicState {
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xA5A5_5A5A_1234_5678);

        let mut state = Self {
            context: MusicContext::Menu,
            menu_playlist: (0..MENU_MUSIC_TRACKS.len()).collect(),
            menu_idx: 0,
            game_playlist: (0..GAME_MUSIC_TRACKS.len()).collect(),
            game_idx: 0,
            is_playing: false,
            play_timer: 0.0,
            track_estimated_duration: 180.0,
            // Initial 0.8s peaceful breather before menu music starts fading in
            delay_timer: 0.0,
            next_delay: 0.8,
            fade_progress: 0.0,
            active_entity: None,
            sink_attached: false,
            rng_state: seed ^ 0x6A09_E667_BB67_AE85,
        };

        state.shuffle_menu();
        state.shuffle_game();
        state
    }

    fn next_u64(&mut self) -> u64 {
        self.rng_state = self.rng_state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.rng_state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max <= 1 {
            0
        } else {
            (self.next_u64() % max as u64) as usize
        }
    }

    pub fn shuffle_menu(&mut self) {
        let n = self.menu_playlist.len();
        for i in (1..n).rev() {
            let j = self.next_usize(i + 1);
            self.menu_playlist.swap(i, j);
        }
        self.menu_idx = 0;
    }

    pub fn shuffle_game(&mut self) {
        let n = self.game_playlist.len();
        for i in (1..n).rev() {
            let j = self.next_usize(i + 1);
            self.game_playlist.swap(i, j);
        }
        self.game_idx = 0;
    }

    pub fn next_menu_track(&mut self) -> &'static str {
        if self.menu_playlist.is_empty() {
            return MENU_MUSIC_TRACKS[0];
        }
        if self.menu_idx >= self.menu_playlist.len() {
            let last = self.menu_playlist[self.menu_playlist.len() - 1];
            self.shuffle_menu();
            // Prevent immediate repetition across shuffle cycles
            if self.menu_playlist.len() > 1 && self.menu_playlist[0] == last {
                self.menu_playlist.swap(0, 1);
            }
        }
        let track_idx = self.menu_playlist[self.menu_idx];
        self.menu_idx += 1;
        MENU_MUSIC_TRACKS[track_idx]
    }

    pub fn next_game_track(&mut self) -> &'static str {
        if self.game_playlist.is_empty() {
            return GAME_MUSIC_TRACKS[0];
        }
        if self.game_idx >= self.game_playlist.len() {
            let last = self.game_playlist[self.game_playlist.len() - 1];
            self.shuffle_game();
            // Prevent immediate repetition across shuffle cycles
            if self.game_playlist.len() > 1 && self.game_playlist[0] == last {
                self.game_playlist.swap(0, 1);
            }
        }
        let track_idx = self.game_playlist[self.game_idx];
        self.game_idx += 1;
        GAME_MUSIC_TRACKS[track_idx]
    }
}

impl Default for MusicState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GameplayMusicPlugin;

impl Plugin for GameplayMusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>()
            .add_systems(Update, update_gameplay_music);
    }
}

/// Manages background music score: context-aware switching between menu and gameplay soundtracks,
/// non-repeating shuffle, smooth 4-second cross-fades, and natural inter-track intervals.
fn update_gameplay_music(
    time: Res<Time<Real>>,
    settings: Res<AudioSettings>,
    mut music_state: ResMut<MusicState>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    world_session: Option<Res<crate::world::WorldSession>>,
    asset_server: Res<AssetServer>,
    mut sinks: Query<(Entity, &mut AudioSink), With<ActiveMusicTrack>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let target_volume = settings.master_volume * settings.music_volume * 0.50;

    let in_session = world_session.as_ref().map_or(false, |s| s.is_active);
    let current_menu = menu_state
        .as_ref()
        .map(|s| *s.get())
        .unwrap_or(crate::menu::MenuState::MainMenu);

    // Menu context is active before entering gameplay or when returning to main menu
    let target_context = if !in_session
        || current_menu == crate::menu::MenuState::MainMenu
        || current_menu == crate::menu::MenuState::Loading
    {
        MusicContext::Menu
    } else {
        MusicContext::Game
    };

    // Context switch: if switching between Menu and Gameplay, immediately stop & clean up current music
    if music_state.context != target_context {
        if let Some(entity) = music_state.active_entity.take() {
            if let Ok(mut entity_cmds) = commands.get_entity(entity) {
                entity_cmds.despawn();
            }
        }
        music_state.is_playing = false;
        music_state.sink_attached = false;
        music_state.play_timer = 0.0;
        music_state.fade_progress = 0.0;
        music_state.context = target_context;

        match target_context {
            MusicContext::Menu => {
                // When returning to main menu, quickly start menu music with smooth fade-in
                music_state.delay_timer = 0.0;
                music_state.next_delay = 0.5;
            }
            MusicContext::Game => {
                // When entering gameplay from loading, provide a peaceful 8s initial pause
                music_state.delay_timer = 0.0;
                music_state.next_delay = 8.0;
            }
        }
        return;
    }

    if music_state.is_playing {
        music_state.play_timer += dt;

        let mut is_finished = false;

        if let Some(entity) = music_state.active_entity {
            if let Ok((_, mut sink)) = sinks.get_mut(entity) {
                // The AudioSink has been attached by Bevy's audio system
                music_state.sink_attached = true;

                if sink.empty() {
                    is_finished = true;
                } else {
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

                    sink.set_volume(Volume::Linear(target_volume * music_state.fade_progress));
                }
            } else if music_state.sink_attached {
                // The sink was attached previously, but the entity was despawned by PlaybackSettings::DESPAWN
                is_finished = true;
            } else if music_state.play_timer > 15.0 {
                // Timeout fallback if the asset failed to load after 15 seconds
                is_finished = true;
            }
        } else {
            is_finished = true;
        }

        // Track finished: clean up and enter organic inter-track pause
        if is_finished || music_state.play_timer >= music_state.track_estimated_duration {
            if let Some(entity) = music_state.active_entity.take() {
                if let Ok(mut entity_cmds) = commands.get_entity(entity) {
                    entity_cmds.despawn();
                }
            }
            music_state.is_playing = false;
            music_state.sink_attached = false;
            music_state.play_timer = 0.0;
            music_state.delay_timer = 0.0;

            match music_state.context {
                MusicContext::Menu => {
                    // Peaceful pause between menu tracks (4 to 10 seconds)
                    let pseudo_rand = (time.elapsed_secs() * 37.0).fract();
                    music_state.next_delay = 4.0 + pseudo_rand * 6.0;
                }
                MusicContext::Game => {
                    // Organic Minecraft-style pause between game tracks (60 to 120 seconds)
                    let pseudo_rand = (time.elapsed_secs() * 37.0).fract();
                    music_state.next_delay = 60.0 + pseudo_rand * 60.0;
                }
            }
        }
    } else {
        music_state.delay_timer += dt;

        if music_state.delay_timer >= music_state.next_delay && target_volume > 0.005 {
            let (path, est_duration) = match music_state.context {
                MusicContext::Menu => {
                    let p = music_state.next_menu_track();
                    (p, 180.0)
                }
                MusicContext::Game => {
                    let p = music_state.next_game_track();
                    (p, 190.0)
                }
            };

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
            music_state.sink_attached = false;
            music_state.play_timer = 0.0;
            music_state.fade_progress = 0.0;
            music_state.track_estimated_duration = est_duration;
        }
    }
}
