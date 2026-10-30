pub mod atmosphere;
pub mod celestial;
pub mod clouds;
pub mod foliage;
pub mod post_process;
pub mod stars;
pub mod time;
pub mod weather;

use bevy::prelude::*;

use atmosphere::{
    DAY_FILL_ILLUMINANCE, DAY_SUN_ILLUMINANCE, NIGHT_MOON_ILLUMINANCE,
    sample_ambient_brightness, sample_ambient_color, sample_sky_color, sync_fog_distance,
    update_atmosphere,
};
use time::{advance_environment_clock, handle_environment_input};

pub use foliage::{FoliagePlugin, GrassConfig, GrassStyle};
pub use time::{DayPhase, EnvironmentState};
pub use weather::{WeatherManager, WeatherPlugin, WeatherType, WindState};

use crate::player::PlayerCamera;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        let is_paused = app
            .world()
            .get_resource::<crate::core::config::LocalConfig>()
            .map_or(false, |c| c.is_time_paused);
        let mut default_state = EnvironmentState::default();
        default_state.is_time_paused = is_paused;
        let initial_clear = sample_sky_color(default_state.time_of_day);
        let initial_ambient = sample_ambient_color(default_state.time_of_day);
        let initial_brightness = sample_ambient_brightness(default_state.time_of_day);

        app.insert_resource(default_state)
            .add_plugins(weather::WeatherPlugin)
            .add_plugins(foliage::FoliagePlugin)
            .add_plugins(post_process::PostProcessPlugin)
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
        (&mut DirectionalLight, &mut Transform),
        (
            With<celestial::SunLight>,
            Without<celestial::MoonLight>,
            Without<celestial::SkyFillLight>,
            Without<Camera3d>,
            Without<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
        ),
    >,
    moon_light: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<celestial::MoonLight>,
            Without<celestial::SunLight>,
            Without<celestial::SkyFillLight>,
            Without<Camera3d>,
            Without<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
        ),
    >,
    sky_fill: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<celestial::SkyFillLight>,
            Without<celestial::SunLight>,
            Without<celestial::MoonLight>,
            Without<Camera3d>,
            Without<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
        ),
    >,
    celestial_materials: Res<celestial::CelestialMaterials>,
    moon_textures: Res<celestial::MoonPhaseTextures>,
    materials: ResMut<Assets<StandardMaterial>>,
    local_config: Option<Res<crate::core::config::LocalConfig>>,
    weather: Option<Res<weather::WeatherManager>>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    world: Option<Res<crate::world::VoxelWorld>>,
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
    // Sky fill light has no shadow mapping, so it gently dims only as player goes deep underground
    let fill_underground_dim = (1.0 - (underground_factor - 0.40).max(0.0) * 1.5).clamp(0.10, 1.0);
    // Keep directional sunlight shining at 100% through cave entrances and near-surface openings.
    // Shadows cast by terrain geometry block the sun naturally; dimming only occurs in deepest depths.
    let sun_underground_dim = (1.0 - (underground_factor - 0.70).max(0.0) * 1.5).clamp(0.40, 1.0);

    let sun_dir = celestial::calculate_sun_direction(state.time_of_day);
    let moon_dir = -sun_dir;
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
        sky_fill,
        celestial_materials,
        moon_textures,
        materials,
        state.time_of_day,
        state.moon_texture_index(),
        state.moon_phase().illuminance_factor(),
        DAY_SUN_ILLUMINANCE * weather_dim * sun_underground_dim,
        DAY_FILL_ILLUMINANCE * weather_dim * fill_underground_dim,
        NIGHT_MOON_ILLUMINANCE * weather_dim * sun_underground_dim,
        shadows_enabled,
        underground_factor,
        sun_visibility,
        moon_visibility,
    );
}

/// Pre-allocates GPU view clustering Z slice and index capacities to 8192
/// to completely prevent terminal resize warnings during deep cavern exploration.
fn configure_gpu_cluster_capacity(
    mut cluster_settings: Option<ResMut<bevy::light::cluster::GlobalClusterSettings>>,
) {
    if let Some(ref mut settings) = cluster_settings {
        if let Some(ref mut gpu) = settings.gpu_clustering {
            gpu.initial_z_slice_list_capacity = 8192;
            gpu.initial_index_list_capacity = 8192;
        }
    }
}

fn sync_starfield_system(
    state: Res<EnvironmentState>,
    camera: Single<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    starfield_root: stars::StarfieldRootQuery,
    material_handle: Res<stars::StarfieldMaterialHandle>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    stars::sync_starfield(
        camera,
        starfield_root,
        material_handle,
        materials,
        state.time_of_day,
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
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    local_config: Option<Res<crate::core::config::LocalConfig>>,
    weather: Option<Res<weather::WeatherManager>>,
) {
    let is_underwater = env_status.as_ref().is_some_and(|s| s.is_camera_in_water);
    let cloud_quality = local_config
        .as_ref()
        .map(|c| c.cloud_quality)
        .unwrap_or(crate::core::CloudQuality::Fancy3D);

    let storm_darkening = weather.as_ref().map_or(0.0, |w| {
        (w.rain_intensity * 0.70 + w.thunder_intensity * 0.30 + w.dust_intensity * 0.50).min(1.0)
    });

    clouds::sync_clouds(
        state.total_in_game_seconds(),
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
        storm_darkening,
    );
}
