use bevy::{
    audio::{AudioPlayer, AudioSink, PlaybackSettings, Volume},
    prelude::*,
};

use super::settings::AudioSettings;
use crate::{
    environment::{DayPhase, EnvironmentState},
    generation::BiomeType,
    player::PlayerEnvironmentStatus,
};

#[derive(Component)]
pub struct UnderwaterAudioLoop;

#[derive(Resource)]
pub struct AmbienceSoundAssets {
    pub underwater_loop: Handle<AudioSource>,
    pub underwater_bubbles: Vec<Handle<AudioSource>>,
    pub cave_sounds: Vec<Handle<AudioSource>>,
    pub birds: Vec<Handle<AudioSource>>,
    pub insects: Vec<Handle<AudioSource>>,
    pub waves: Vec<Handle<AudioSource>>,
    pub desert_wind: Vec<Handle<AudioSource>>,
    pub cold_wind: Vec<Handle<AudioSource>>,
    pub swamp_wind: Vec<Handle<AudioSource>>,
    pub storm_wind: Vec<Handle<AudioSource>>,
    pub surface_breeze: Vec<Handle<AudioSource>>,
}

#[derive(Resource)]
pub struct AmbienceState {
    pub cave_timer: f32,
    pub surface_timer: f32,
    pub bubble_timer: f32,
    pub current_underwater_vol: f32,
    rng_state: u64,
}

impl Default for AmbienceState {
    fn default() -> Self {
        Self {
            cave_timer: 20.0,
            surface_timer: 12.0,
            bubble_timer: 6.0,
            current_underwater_vol: 0.0,
            rng_state: 0x6A09_E667_BB67_AE85,
        }
    }
}

impl AmbienceState {
    fn next_float(&mut self) -> f32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.rng_state >> 32) as u32 as f32) / 4294967295.0
    }
}

pub struct AmbientAudioPlugin;

impl Plugin for AmbientAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AmbienceState>()
            .add_systems(Startup, setup_ambient_audio)
            .add_systems(Update, (update_ambient_audio, update_underwater_loop));
    }
}

fn setup_ambient_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let underwater_loop = asset_server.load("sounds/ambience/underwater/underwater_ambience.ogg");

    let underwater_bubbles = vec![
        asset_server.load("sounds/ambience/underwater/bubbles1.ogg"),
        asset_server.load("sounds/ambience/underwater/bubbles2.ogg"),
        asset_server.load("sounds/ambience/underwater/bubbles3.ogg"),
        asset_server.load("sounds/ambience/underwater/bubbles4.ogg"),
        asset_server.load("sounds/ambience/underwater/bubbles5.ogg"),
        asset_server.load("sounds/ambience/underwater/bubbles6.ogg"),
        asset_server.load("sounds/ambience/underwater/crackles1.ogg"),
        asset_server.load("sounds/ambience/underwater/crackles2.ogg"),
    ];

    let cave_sounds = vec![
        asset_server.load("sounds/ambience/cave/cave1.ogg"),
        asset_server.load("sounds/ambience/cave/cave2.ogg"),
        asset_server.load("sounds/ambience/cave/rocks1.ogg"),
        asset_server.load("sounds/ambience/cave/rocks2.ogg"),
        asset_server.load("sounds/ambience/cave/rocks3.ogg"),
        asset_server.load("sounds/ambience/cave/unknown1.ogg"),
        asset_server.load("sounds/ambience/cave/unknown2.ogg"),
        asset_server.load("sounds/ambience/cave/unknown3.ogg"),
        asset_server.load("sounds/ambience/cave/unknown4.ogg"),
    ];

    let birds = vec![
        asset_server.load("sounds/ambience/additions/birds/bird1.ogg"),
        asset_server.load("sounds/ambience/additions/birds/bird2.ogg"),
        asset_server.load("sounds/ambience/additions/birds/bird_flapping1.ogg"),
        asset_server.load("sounds/ambience/additions/birds/bird_flapping2l.ogg"),
        asset_server.load("sounds/ambience/additions/birds/bird_flapping2r.ogg"),
    ];

    let insects = vec![
        asset_server.load("sounds/ambience/additions/insects/insect1.ogg"),
        asset_server.load("sounds/ambience/additions/insects/insect2.ogg"),
        asset_server.load("sounds/ambience/additions/insects/insect3.ogg"),
        asset_server.load("sounds/ambience/additions/insects/insect_long1.ogg"),
    ];

    let waves = vec![
        asset_server.load("sounds/ambience/beach/wave1.ogg"),
        asset_server.load("sounds/ambience/beach/wave2.ogg"),
        asset_server.load("sounds/ambience/beach/water.ogg"),
    ];

    let desert_wind = vec![
        asset_server.load("sounds/ambience/desert/desert_wind1.ogg"),
        asset_server.load("sounds/ambience/desert/desert_wind2.ogg"),
    ];

    let cold_wind = vec![
        asset_server.load("sounds/ambience/snowy_plains/distant_wind.ogg"),
        asset_server.load("sounds/ambience/snowy_plains/somber_wind.ogg"),
    ];

    let swamp_wind = vec![
        asset_server.load("sounds/ambience/forest/normal_wind_swamp.ogg"),
        asset_server.load("sounds/ambience/additions/insects/insect_long1.ogg"),
    ];

    let storm_wind = vec![
        asset_server.load("sounds/ambience/rainy_weather/indoor_wind.ogg"),
        asset_server.load("sounds/ambience/rainy_weather/rough_wind.ogg"),
    ];

    let surface_breeze = vec![
        asset_server.load("sounds/ambience/forest/normal_wind.ogg"),
        asset_server.load("sounds/ambience/forest/tree_wind.ogg"),
        asset_server.load("sounds/ambience/environment/outside1.ogg"),
        asset_server.load("sounds/ambience/environment/outside2.ogg"),
        asset_server.load("sounds/ambience/environment/outside_distant.ogg"),
    ];

    commands.insert_resource(AmbienceSoundAssets {
        underwater_loop: underwater_loop.clone(),
        underwater_bubbles,
        cave_sounds,
        birds,
        insects,
        waves,
        desert_wind,
        cold_wind,
        swamp_wind,
        storm_wind,
        surface_breeze,
    });

    commands.spawn((
        UnderwaterAudioLoop,
        AudioPlayer(underwater_loop),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
    ));
}

fn update_underwater_loop(
    time: Res<Time>,
    settings: Res<AudioSettings>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    mut state: ResMut<AmbienceState>,
    mut loop_query: Query<&mut AudioSink, With<UnderwaterAudioLoop>>,
) {
    let is_camera_in_water = env_status.as_ref().is_some_and(|s| s.is_camera_in_water);
    let master = settings.master_volume * settings.ambient_volume * 0.50;

    let target_vol = if is_camera_in_water {
        0.80 * master
    } else {
        0.0
    };

    let dt = time.delta_secs();
    // Smooth volume fade (fade in quickly, fade out smoothly)
    let speed = if target_vol > state.current_underwater_vol {
        6.0
    } else {
        3.0
    };
    state.current_underwater_vol +=
        (target_vol - state.current_underwater_vol) * (speed * dt).min(1.0);

    for mut sink in &mut loop_query {
        sink.set_volume(Volume::Linear(state.current_underwater_vol));
    }
}

fn update_ambient_audio(
    time: Res<Time>,
    settings: Res<AudioSettings>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    env_state: Option<Res<EnvironmentState>>,
    assets: Res<AmbienceSoundAssets>,
    mut state: ResMut<AmbienceState>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let master = settings.master_volume * settings.ambient_volume * 0.50;
    if master <= 0.001 {
        return;
    }

    let is_underground = env_status.as_ref().is_some_and(|s| s.is_underground);
    let is_camera_water = env_status.as_ref().is_some_and(|s| s.is_camera_in_water);
    let biome = env_status
        .as_ref()
        .map_or(BiomeType::default(), |s| s.current_biome);
    let day_phase = env_state
        .as_ref()
        .map_or(DayPhase::Noon, |s| DayPhase::from_time(s.time_of_day));

    // 1. Underwater periodic bubbles
    if is_camera_water {
        state.bubble_timer -= dt;
        if state.bubble_timer <= 0.0 {
            state.bubble_timer = 4.0 + state.next_float() * 6.0;
            if !assets.underwater_bubbles.is_empty() {
                let idx = (state.next_float() * assets.underwater_bubbles.len() as f32) as usize
                    % assets.underwater_bubbles.len();
                let sound = assets.underwater_bubbles[idx].clone();
                let pitch = 0.92 + state.next_float() * 0.16;
                let final_vol = 0.65 * master;
                commands.spawn((
                    AudioPlayer(sound),
                    PlaybackSettings::DESPAWN
                        .with_volume(Volume::Linear(final_vol))
                        .with_speed(pitch),
                ));
            }
        }
        return; // Suppress surface & cave sounds while submerged underwater
    }

    // 2. Subterranean / Cave periodic drones and rock rattles
    if is_underground {
        state.cave_timer -= dt;
        if state.cave_timer <= 0.0 {
            state.cave_timer = 18.0 + state.next_float() * 24.0;
            if !assets.cave_sounds.is_empty() {
                let idx = (state.next_float() * assets.cave_sounds.len() as f32) as usize
                    % assets.cave_sounds.len();
                let sound = assets.cave_sounds[idx].clone();
                let pitch = 0.90 + state.next_float() * 0.20;
                let final_vol = 0.60 * master;
                commands.spawn((
                    AudioPlayer(sound),
                    PlaybackSettings::DESPAWN
                        .with_volume(Volume::Linear(final_vol))
                        .with_speed(pitch),
                ));
            }
        }
        return; // Suppress surface nature sounds while underground
    }

    // 3. Surface Biome Ambience
    state.surface_timer -= dt;
    if state.surface_timer <= 0.0 {
        state.surface_timer = 12.0 + state.next_float() * 16.0;

        let sound_pool = match biome {
            BiomeType::SandyShores
            | BiomeType::GravelBeach
            | BiomeType::DuneBeach
            | BiomeType::CraggyBeach
            | BiomeType::TidePools
            | BiomeType::CoastalCliffs
            | BiomeType::TropicalIsland
            | BiomeType::TemperateIsland
            | BiomeType::CoralIsland
            | BiomeType::RockyIsland
            | BiomeType::WarmOcean
            | BiomeType::ColdOcean
            | BiomeType::CoralReefs
            | BiomeType::SeagrassMeadows
            | BiomeType::KelpForests
            | BiomeType::OceanTrench
            | BiomeType::Caldera => &assets.waves,

            BiomeType::DesertSpires
            | BiomeType::AncientSands
            | BiomeType::DesertOasis
            | BiomeType::AcaciaPlains
            | BiomeType::FracturedSavanna
            | BiomeType::SavannaBadlands
            | BiomeType::WitheredWoods
            | BiomeType::AshenWastes
            | BiomeType::ObsidianMire
            | BiomeType::AcidPools
            | BiomeType::SulfurVents
            | BiomeType::ScorchedFlats
            | BiomeType::DuneSea
            | BiomeType::RedrockCanyon
            | BiomeType::PaintedMesas => &assets.desert_wind,

            BiomeType::AlpinePeaks
            | BiomeType::JaggedPeaks
            | BiomeType::MonolithCliffs
            | BiomeType::WindyPass
            | BiomeType::GlacialChasm
            | BiomeType::IceSheet
            | BiomeType::BleakTaiga
            | BiomeType::FrozenCliffs
            | BiomeType::FrozenIsland
            | BiomeType::TundraIsland
            | BiomeType::WintryForest
            | BiomeType::FrostPinewoods
            | BiomeType::IceboundRidge
            | BiomeType::PineHighlands
            | BiomeType::SnowyPlains
            | BiomeType::ColdShrubland
            | BiomeType::PermafrostSteppe => &assets.cold_wind,

            BiomeType::WillowMarsh
            | BiomeType::MangroveSwamp
            | BiomeType::OrchidSwamp
            | BiomeType::SwampyShore
            | BiomeType::Mudflats => &assets.swamp_wind,

            _ => {
                let roll = state.next_float();
                if roll < 0.25 {
                    &assets.surface_breeze
                } else if roll < 0.35 {
                    &assets.storm_wind
                } else {
                    match day_phase {
                        DayPhase::Morning | DayPhase::Noon => &assets.birds,
                        DayPhase::Evening | DayPhase::Night => &assets.insects,
                    }
                }
            }
        };

        let chosen_sound = if !sound_pool.is_empty() {
            let idx = (state.next_float() * sound_pool.len() as f32) as usize % sound_pool.len();
            Some(sound_pool[idx].clone())
        } else if !assets.surface_breeze.is_empty() {
            let idx = (state.next_float() * assets.surface_breeze.len() as f32) as usize
                % assets.surface_breeze.len();
            Some(assets.surface_breeze[idx].clone())
        } else {
            None
        };

        if let Some(sound) = chosen_sound {
            let pitch = 0.94 + state.next_float() * 0.12;
            let final_vol = 0.50 * master;
            commands.spawn((
                AudioPlayer(sound),
                PlaybackSettings::DESPAWN
                    .with_volume(Volume::Linear(final_vol))
                    .with_speed(pitch),
            ));
        }
    }
}
