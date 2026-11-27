pub mod atmosphere;
pub mod celestial;
pub mod clouds;
pub mod foliage;
pub mod particles;
pub mod post_process;
pub mod sky;
pub mod stars;
pub mod time;
pub mod weather;

use bevy::prelude::*;

use atmosphere::{
    DAY_SUN_ILLUMINANCE, NIGHT_MOON_ILLUMINANCE, sample_ambient_brightness, sample_ambient_color,
    sample_sky_color, sync_fog_distance, update_atmosphere,
};
use time::{advance_environment_clock, handle_environment_input};

pub use foliage::{FoliagePlugin, GrassConfig, GrassStyle};
pub use particles::{WorldParticleBurst, WorldParticleBurstQueue};
pub use time::{DayPhase, EnvironmentState};
pub use weather::{WeatherManager, WeatherPlugin, WeatherType, WindState};

use crate::player::PlayerCamera;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        let is_paused = app
            .world()
            .get_resource::<crate::core::config::LocalConfig>()
            .is_some_and(|c| c.is_time_paused);
        let default_state = EnvironmentState {
            is_time_paused: is_paused,
            ..Default::default()
        };
        let initial_clear = sample_sky_color(default_state.time_of_day);
        let initial_ambient = sample_ambient_color(default_state.time_of_day);
        let initial_brightness = sample_ambient_brightness(default_state.time_of_day);

        app.insert_resource(default_state)
            .init_resource::<particles::WorldParticleBurstQueue>()
            .add_plugins(weather::WeatherPlugin)
            .add_plugins(foliage::FoliagePlugin)
            .add_plugins(post_process::PostProcessPlugin)
            .add_plugins(sky::SkyDomePlugin)
            .insert_resource(GlobalAmbientLight {
                color: initial_ambient,
                brightness: initial_brightness,
                ..default()
            })
            .insert_resource(ClearColor(initial_clear))
            .add_systems(
                Startup,
                (
                    celestial::setup_celestial,
                    stars::setup_starfield,
                    clouds::setup_clouds,
                    particles::setup_world_particles,
                    particles::setup_wind_streaks,
                    particles::setup_water_splashes_3d,
                    particles::setup_fluid_bubbles,
                    particles::setup_shooting_stars,
                    particles::setup_water_godrays_3d,
                    configure_gpu_cluster_capacity,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_environment_input,
                    advance_environment_clock,
                    update_atmosphere,
                    sync_fog_distance,
                    sync_celestial_system,
                    celestial::sync_celestial_shadow_quality,
                    sync_starfield_system,
                    sync_cloud_system,
                    particles::update_world_particles,
                    particles::update_wind_streaks,
                    particles::update_water_splashes_3d,
                    particles::update_fluid_bubbles,
                    particles::update_shooting_stars,
                    particles::update_water_godrays_3d,
                )
                    .chain()
                    .after(crate::player::PlayerSet::Movement),
            );
    }
}

fn sync_celestial_system(
    state: Res<EnvironmentState>,
    camera: Single<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    sun_visual: Single<
        (&mut Transform, &mut Visibility),
        (
            With<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
            Without<Camera3d>,
        ),
    >,
    moon_visual: Single<
        (&mut Transform, &mut Visibility),
        (
            With<celestial::MoonVisual>,
            Without<celestial::SunVisual>,
            Without<Camera3d>,
        ),
    >,
    sun_light: Single<
        (&mut DirectionalLight, &mut Transform, &mut Visibility),
        (
            With<celestial::SunLight>,
            Without<celestial::MoonLight>,
            Without<Camera3d>,
            Without<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
        ),
    >,
    moon_light: Single<
        (&mut DirectionalLight, &mut Transform, &mut Visibility),
        (
            With<celestial::MoonLight>,
            Without<celestial::SunLight>,
            Without<Camera3d>,
            Without<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
        ),
    >,
    celestial_materials: Res<celestial::CelestialMaterials>,
    moon_textures: Res<celestial::MoonPhaseTextures>,
    materials: ResMut<Assets<StandardMaterial>>,
    (local_config, weather, env_status, world, sky_palette): (
        Option<Res<crate::core::config::LocalConfig>>,
        Option<Res<weather::WeatherManager>>,
        Option<Res<crate::player::PlayerEnvironmentStatus>>,
        Option<Res<crate::world::VoxelWorld>>,
        Option<Res<sky::SkyPaletteState>>,
    ),
) {
    let shadows_enabled = local_config
        .as_ref()
        .map(|c| c.shadow_quality != crate::core::ShadowQuality::Off)
        .unwrap_or(true);

    let weather_dim = weather.as_ref().map_or(1.0, |w| {
        (1.0 - w.rain_intensity * 0.70 - w.thunder_intensity * 0.25 - w.dust_intensity * 0.60)
            .clamp(0.05, 1.0)
    });

    let underground_factor = env_status.as_ref().map_or(0.0, |s| s.underground_factor);
    // Keep directional sunlight shining at 100% through cave entrances and near-surface openings.
    // Shadows cast by terrain geometry block the sun naturally; fade out completely in deep caverns
    // beyond shadow cascade distance so sunlight never leaks through deep roofs.
    let sun_underground_dim = (1.0 - (underground_factor - 0.35).max(0.0) * 1.54).clamp(0.0, 1.0);
    // Attenuate directional sunlight smoothly with water depth (bright reefs -> dark abyssal trenches)
    let sun_underwater_dim = if env_status.as_ref().is_some_and(|s| s.is_camera_in_water) {
        let depth_factor = ((12.0 - camera.translation.y) / 42.0).clamp(0.0, 1.0);
        (1.0 - depth_factor * 0.94).powf(1.6) * 0.80
    } else {
        1.0
    };

    let sun_dir = celestial::calculate_sun_direction(state.time_of_day);
    let moon_dir = -sun_dir;
    // Smoothly fade sun/moon discs with ocean depth (1.0 near surface -> 0.0 in deep trenches, no sudden pop)
    let underwater_celestial_fade = if env_status.as_ref().is_some_and(|s| s.is_camera_in_water) {
        let depth_t = ((10.0 - camera.translation.y) / 38.0).clamp(0.0, 1.0);
        (1.0 - depth_t).powf(1.5)
    } else {
        1.0
    };
    let (sun_visibility, moon_visibility) = if let Some(ref w) = world {
        (
            post_process::calculate_celestial_visibility(w, camera.translation, sun_dir, 96.0),
            post_process::calculate_celestial_visibility(w, camera.translation, moon_dir, 96.0),
        )
    } else {
        (1.0, 1.0)
    };

    celestial::sync_celestial_transforms(
        camera,
        sun_visual,
        moon_visual,
        sun_light,
        moon_light,
        celestial_materials,
        moon_textures,
        materials,
        state.time_of_day,
        state.moon_texture_index(),
        state.moon_phase().illuminance_factor(),
        DAY_SUN_ILLUMINANCE * weather_dim * sun_underground_dim * sun_underwater_dim,
        NIGHT_MOON_ILLUMINANCE * weather_dim * sun_underground_dim * sun_underwater_dim,
        shadows_enabled,
        underground_factor,
        sun_visibility,
        moon_visibility,
        sky_palette.as_ref().map_or(0.0, |sp| sp.climate_temp),
        underwater_celestial_fade,
    );
}

/// Pre-allocates GPU view clustering Z slice and index capacities to 8192
/// to completely prevent terminal resize warnings during deep cavern exploration.
fn configure_gpu_cluster_capacity(
    mut cluster_settings: Option<ResMut<bevy::light::cluster::GlobalClusterSettings>>,
) {
    if let Some(ref mut settings) = cluster_settings
        && let Some(ref mut gpu) = settings.gpu_clustering {
            gpu.initial_z_slice_list_capacity = 8192;
            gpu.initial_index_list_capacity = 8192;
        }
}

fn sync_starfield_system(
    time: Res<Time>,
    state: Res<EnvironmentState>,
    camera: Single<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    starfield_root: stars::StarfieldRootQuery,
    material_handles: Res<stars::StarfieldMaterialHandles>,
    materials: ResMut<Assets<StandardMaterial>>,
    weather: Option<Res<weather::WeatherManager>>,
) {
    let rain_factor = weather.as_ref().map_or(0.0, |w| {
        (w.rain_intensity * 0.8 + w.thunder_intensity * 0.4 + w.snow_intensity * 0.7 + w.dust_intensity * 0.8)
            .clamp(0.0, 1.0)
    });
    stars::sync_starfield(
        camera,
        starfield_root,
        material_handles,
        materials,
        state.time_of_day,
        time.elapsed_secs(),
        rain_factor,
    );
}

fn sync_cloud_system(
    state: Res<EnvironmentState>,
    camera: clouds::CameraTransformQuery,
    cloud_layers: clouds::CloudLayerQuery,
    material_handle: Res<clouds::CloudMaterialHandle>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    cloud_map: Res<clouds::CloudTextureMap>,
    mut layers_res: ResMut<clouds::CloudLayersResource>,
    (env_status, local_config, weather, lightning): (
        Option<Res<crate::player::PlayerEnvironmentStatus>>,
        Option<Res<crate::core::config::LocalConfig>>,
        Option<Res<weather::WeatherManager>>,
        Option<Res<weather::LightningState>>,
    ),
    mut cloud_drift_secs: Local<f64>,
    mut last_in_game_secs: Local<Option<f64>>,
) {
    let is_underwater = env_status.as_ref().is_some_and(|s| s.is_camera_in_water);
    let cloud_quality = local_config
        .as_ref()
        .map(|c| c.cloud_quality)
        .unwrap_or(crate::core::CloudQuality::Fancy3D);
    let cloud_render_distance = local_config
        .as_ref()
        .map(|c| c.cloud_render_distance)
        .unwrap_or(64);

    let (storm_darkening, snow_i, dust_i, wind_speed_mult) = weather.as_ref().map_or(
        (0.0, 0.0, 0.0, 1.0_f64),
        |w| {
            let storm = (w.rain_intensity * 0.85 + w.thunder_intensity * 0.45).min(1.0);
            let mult = 1.0
                + (w.rain_intensity * 0.85
                    + w.thunder_intensity * 1.65
                    + w.snow_intensity * 0.90
                    + w.dust_intensity * 1.90) as f64;
            (storm, w.snow_intensity, w.dust_intensity, mult)
        },
    );
    let lightning_flash = lightning.as_ref().map_or(0.0, |l| l.flash_intensity);

    let now_in_game = state.total_in_game_seconds();
    if let Some(prev) = *last_in_game_secs {
        let delta = now_in_game - prev;
        if delta.abs() > 0.5 {
            // F6 time scrub / phase jump: advance clouds directly in lockstep with time
            *cloud_drift_secs += delta;
        } else if delta > 0.0 {
            *cloud_drift_secs += delta * wind_speed_mult;
        }
    } else {
        *cloud_drift_secs = now_in_game;
    }
    *last_in_game_secs = Some(now_in_game);

    clouds::sync_clouds(
        *cloud_drift_secs,
        camera,
        cloud_layers,
        material_handle,
        materials,
        &mut meshes,
        &cloud_map,
        &mut layers_res,
        state.time_of_day,
        is_underwater,
        cloud_quality,
        cloud_render_distance,
        storm_darkening,
        snow_i,
        dust_i,
        lightning_flash,
    );
}
