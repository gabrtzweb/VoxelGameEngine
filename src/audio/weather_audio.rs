use bevy::{
    audio::{AudioPlayer, AudioSink, PlaybackSettings, Volume},
    prelude::*,
};

use super::settings::AudioSettings;
use crate::environment::weather::{TriggerThunderEvent, WeatherManager, WindState};

#[derive(Component)]
pub struct RainAudioLoop;

#[derive(Component)]
pub struct WindAudioLoop;

#[derive(Component)]
pub struct SnowAudioLoop;

#[derive(Component)]
pub struct SandstormAudioLoop;

/// Acoustic thunderclap waiting for speed-of-sound physical arrival.
#[derive(Debug, Clone)]
pub struct PendingThunder {
    pub delay: f32,
    pub distance: f32,
    pub strike_pos: Vec3,
}

#[derive(Resource, Default)]
pub struct PendingThunderQueue(pub Vec<PendingThunder>);

/// Audio handles library for environmental phenomena.
#[derive(Resource)]
pub struct WeatherSoundAssets {
    pub rain_loop: Handle<AudioSource>,
    pub wind_loop: Handle<AudioSource>,
    pub snow_loop: Handle<AudioSource>,
    pub sandstorm_loop: Handle<AudioSource>,

    pub lightning_impacts: [Handle<AudioSource>; 3],
    pub thunders_near: [Handle<AudioSource>; 3],
    pub thunders_mid: [Handle<AudioSource>; 3],
    pub thunders_far: [Handle<AudioSource>; 3],
}

pub struct WeatherAudioPlugin;

impl Plugin for WeatherAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingThunderQueue>()
            .add_systems(Startup, setup_weather_audio)
            .add_observer(queue_lightning_thunder)
            .add_systems(Update, (update_weather_audio_loops, tick_thunder_queue));
    }
}

fn setup_weather_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let rain_loop = asset_server.load("sounds/weather/rain1.ogg");
    let wind_loop = asset_server.load("sounds/weather/wary_wind1.ogg");
    let snow_loop = asset_server.load("sounds/weather/snow-01.ogg");
    let sandstorm_loop = asset_server.load("sounds/weather/sandstorm-01.ogg");

    let lightning_impacts = [
        asset_server.load("sounds/weather/lightning_impact1.ogg"),
        asset_server.load("sounds/weather/lightning_impact2.ogg"),
        asset_server.load("sounds/weather/lightning_impact3.ogg"),
    ];

    let thunders_near = [
        asset_server.load("sounds/weather/thunder1.ogg"),
        asset_server.load("sounds/weather/thunder2.ogg"),
        asset_server.load("sounds/weather/thunder3.ogg"),
    ];

    let thunders_mid = [
        asset_server.load("sounds/weather/thunder1_long.ogg"),
        asset_server.load("sounds/weather/thunder2_long.ogg"),
        asset_server.load("sounds/weather/thunder3_long.ogg"),
    ];

    let thunders_far = [
        asset_server.load("sounds/weather/thunder1_distance.ogg"),
        asset_server.load("sounds/weather/thunder2_distance.ogg"),
        asset_server.load("sounds/weather/thunder3_distance.ogg"),
    ];

    commands.insert_resource(WeatherSoundAssets {
        rain_loop: rain_loop.clone(),
        wind_loop: wind_loop.clone(),
        snow_loop: snow_loop.clone(),
        sandstorm_loop: sandstorm_loop.clone(),
        lightning_impacts,
        thunders_near,
        thunders_mid,
        thunders_far,
    });

    // Spawn 4 persistent ambient loops with initial 0.0 volume
    commands.spawn((
        RainAudioLoop,
        AudioPlayer(rain_loop),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
    ));

    commands.spawn((
        WindAudioLoop,
        AudioPlayer(wind_loop),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
    ));

    commands.spawn((
        SnowAudioLoop,
        AudioPlayer(snow_loop),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
    ));

    commands.spawn((
        SandstormAudioLoop,
        AudioPlayer(sandstorm_loop),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
    ));
}

/// Dynamically modulates loop audio levels based on live weather intensities and wind speeds.
fn update_weather_audio_loops(
    settings: Res<AudioSettings>,
    weather: Option<Res<WeatherManager>>,
    wind: Option<Res<WindState>>,
    mut rain_query: Query<&mut AudioSink, (With<RainAudioLoop>, Without<WindAudioLoop>, Without<SnowAudioLoop>, Without<SandstormAudioLoop>)>,
    mut wind_query: Query<&mut AudioSink, (With<WindAudioLoop>, Without<RainAudioLoop>, Without<SnowAudioLoop>, Without<SandstormAudioLoop>)>,
    mut snow_query: Query<&mut AudioSink, (With<SnowAudioLoop>, Without<RainAudioLoop>, Without<WindAudioLoop>, Without<SandstormAudioLoop>)>,
    mut sand_query: Query<&mut AudioSink, (With<SandstormAudioLoop>, Without<RainAudioLoop>, Without<WindAudioLoop>, Without<SnowAudioLoop>)>,
) {
    let master = settings.master_volume * settings.weather_volume;

    let rain_target = weather
        .as_ref()
        .map_or(0.0, |w| w.rain_intensity * 0.90 * master);

    let wind_speed = wind.as_ref().map_or(0.0, |w| w.current_wind.length());
    let wind_target = ((wind_speed / 9.0).clamp(0.0, 1.0) * 0.70 * master).max(
        weather
            .as_ref()
            .map_or(0.0, |w| if w.current.is_storm() { 0.45 * master } else { 0.0 }),
    );

    let snow_target = weather
        .as_ref()
        .map_or(0.0, |w| w.snow_intensity * 0.65 * master);

    let sand_target = weather
        .as_ref()
        .map_or(0.0, |w| w.dust_intensity * 0.85 * master);

    for mut sink in &mut rain_query {
        sink.set_volume(Volume::Linear(rain_target));
    }

    for mut sink in &mut wind_query {
        sink.set_volume(Volume::Linear(wind_target));
    }

    for mut sink in &mut snow_query {
        sink.set_volume(Volume::Linear(snow_target));
    }

    for mut sink in &mut sand_query {
        sink.set_volume(Volume::Linear(sand_target));
    }
}

/// Receives strike events from the lightning system and schedules speed-of-sound acoustic arrival.
fn queue_lightning_thunder(
    trigger: On<TriggerThunderEvent>,
    mut queue: ResMut<PendingThunderQueue>,
) {
    let ev = trigger.event();
    // Physical speed of sound: ~343 m/s. Min delay 0.08s, capped at 4.5s.
    let delay = (ev.distance_to_camera / 343.0).clamp(0.08, 4.5);
    queue.0.push(PendingThunder {
        delay,
        distance: ev.distance_to_camera,
        strike_pos: ev.strike_pos,
    });
}

/// Advances acoustic delay timers and plays distance-attenuated thunder claps upon soundwave arrival.
fn tick_thunder_queue(
    time: Res<Time>,
    assets: Option<Res<WeatherSoundAssets>>,
    settings: Res<AudioSettings>,
    mut queue: ResMut<PendingThunderQueue>,
    mut commands: Commands,
) {
    let Some(ref sound_assets) = assets else {
        return;
    };

    let dt = time.delta_secs();
    let master = settings.master_volume * settings.weather_volume;

    queue.0.retain_mut(|thunder| {
        thunder.delay -= dt;
        if thunder.delay <= 0.0 {
            let dist = thunder.distance;

            // Pick clip and volume according to physical distance to lightning strike
            if dist < 42.0 {
                // Near Strike (<42m): instantaneous sharp lightning crack + heavy thunder
                let impact_clip = sound_assets.lightning_impacts[0].clone();
                let thunder_clip = sound_assets.thunders_near[0].clone();

                let vol_crack = (1.0 * master).clamp(0.0, 1.0);
                let vol_thunder = (0.95 * master).clamp(0.0, 1.0);

                commands.spawn((
                    AudioPlayer(impact_clip),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(vol_crack)),
                ));
                commands.spawn((
                    AudioPlayer(thunder_clip),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(vol_thunder)),
                ));
            } else if dist < 110.0 {
                // Medium Distance (42m - 110m): deep rolling prolonged thunder
                let thunder_clip = sound_assets.thunders_mid[1].clone();
                let dist_atten = (1.0 - (dist - 42.0) / 120.0).clamp(0.35, 0.90);
                let vol = (dist_atten * master).clamp(0.0, 1.0);

                commands.spawn((
                    AudioPlayer(thunder_clip),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(vol)),
                ));
            } else {
                // Distant Strike (>110m): low muffled acoustic rumble echoing across mountains
                let thunder_clip = sound_assets.thunders_far[0].clone();
                let dist_atten = (1.0 - (dist / 280.0)).clamp(0.15, 0.55);
                let vol = (dist_atten * master).clamp(0.0, 1.0);

                commands.spawn((
                    AudioPlayer(thunder_clip),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(vol)),
                ));
            }

            false // Remove from queue
        } else {
            true // Keep waiting
        }
    });
}
