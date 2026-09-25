use bevy::{camera::Exposure, prelude::*};

use super::time::EnvironmentState;
use crate::{
    player::PlayerCamera,
    world::{CHUNK_SIZE, VOXEL_SIZE, streaming::ChunkStreamingSettings},
};

pub const DAY_SUN_ILLUMINANCE: f32 = 6_500.0;
pub const DAY_FILL_ILLUMINANCE: f32 = 2_200.0;
pub const DAY_AMBIENT_BRIGHTNESS: f32 = 450.0;
pub const DAY_EXPOSURE_EV100: f32 = 11.0;

pub const NIGHT_MOON_ILLUMINANCE: f32 = 450.0;
pub const NIGHT_AMBIENT_BRIGHTNESS: f32 = 28.0;
pub const NIGHT_EXPOSURE_EV100: f32 = 9.2;

pub const FOG_START_FACTOR: f32 = 0.50;
pub const FOG_END_FACTOR: f32 = 0.90;

type CameraAtmosphereQuery<'w, 's> = Single<
    'w,
    's,
    (&'static mut DistanceFog, &'static mut Exposure),
    (With<Camera3d>, With<PlayerCamera>),
>;

pub fn update_atmosphere(
    state: Res<EnvironmentState>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    camera: CameraAtmosphereQuery,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
) {
    let t = state.time_of_day;
    let (mut fog, mut exposure) = camera.into_inner();

    let is_underwater = env_status.as_ref().is_some_and(|s| s.is_camera_in_water);

    if is_underwater {
        let water_fog_color = Color::srgb(0.04, 0.20, 0.35);
        clear_color.0 = Color::srgb(0.02, 0.12, 0.24);
        ambient.color = Color::srgb(0.15, 0.40, 0.60);
        ambient.brightness = sample_ambient_brightness(t).max(250.0);

        fog.color = water_fog_color;
        fog.directional_light_color = water_fog_color;
        fog.directional_light_exponent = 4.0;
        exposure.ev100 = sample_exposure(t);
    } else {
        clear_color.0 = sample_sky_color(t);
        ambient.color = sample_ambient_color(t);
        ambient.brightness = sample_ambient_brightness(t);

        fog.color = sample_fog_color(t);
        fog.directional_light_color = sample_fog_light_color(t);
        fog.directional_light_exponent = sample_fog_light_exponent(t);
        exposure.ev100 = sample_exposure(t);
    }
}

pub fn sync_fog_distance(
    settings: Res<ChunkStreamingSettings>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
    camera: Single<&mut DistanceFog, (With<Camera3d>, With<PlayerCamera>)>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
) {
    let mut fog = camera.into_inner();

    if let Some(ref gs) = game_settings
        && !gs.fog_enabled
    {
        fog.falloff = FogFalloff::Linear {
            start: 99999.0,
            end: 100000.0,
        };
        return;
    }

    let is_underwater = env_status.as_ref().is_some_and(|s| s.is_camera_in_water);

    if is_underwater {
        fog.falloff = FogFalloff::Linear {
            start: 1.0,
            end: 22.0,
        };
    } else {
        let chunk_world_size = CHUNK_SIZE as f32 * VOXEL_SIZE;
        let render_radius = settings.render_distance.max(1) as f32 * chunk_world_size;
        fog.falloff = FogFalloff::Linear {
            start: render_radius * FOG_START_FACTOR,
            end: render_radius * FOG_END_FACTOR,
        };
    }
}

pub fn sample_4stop<T: Copy>(
    time_of_day: f32,
    morning: T,
    noon: T,
    evening: T,
    night: T,
    lerp_fn: impl Fn(T, T, f32) -> T,
) -> T {
    let t = time_of_day.rem_euclid(1.0);
    if t < 0.25 {
        lerp_fn(morning, noon, t / 0.25)
    } else if t < 0.50 {
        lerp_fn(noon, evening, (t - 0.25) / 0.25)
    } else if t < 0.75 {
        lerp_fn(evening, night, (t - 0.50) / 0.25)
    } else {
        lerp_fn(night, morning, (t - 0.75) / 0.25)
    }
}

pub fn lerp_linear_rgba(a: LinearRgba, b: LinearRgba, factor: f32) -> LinearRgba {
    LinearRgba::new(
        a.red + (b.red - a.red) * factor,
        a.green + (b.green - a.green) * factor,
        a.blue + (b.blue - a.blue) * factor,
        a.alpha + (b.alpha - a.alpha) * factor,
    )
}

pub fn lerp_f32(a: f32, b: f32, factor: f32) -> f32 {
    a + (b - a) * factor
}

pub fn sample_sky_color(time_of_day: f32) -> Color {
    let morning = LinearRgba::new(0.68, 0.48, 0.52, 1.0); // Dawn rosy peach
    let noon = LinearRgba::new(0.38, 0.62, 0.92, 1.0); // Midday azure blue
    let evening = LinearRgba::new(0.72, 0.36, 0.24, 1.0); // Sunset amber/crimson
    let night = LinearRgba::new(0.020, 0.035, 0.080, 1.0); // Midnight dark indigo

    Color::LinearRgba(sample_4stop(
        time_of_day,
        morning,
        noon,
        evening,
        night,
        lerp_linear_rgba,
    ))
}

pub fn sample_ambient_color(time_of_day: f32) -> Color {
    let morning = LinearRgba::new(0.92, 0.78, 0.70, 1.0);
    let noon = LinearRgba::new(0.85, 0.88, 0.95, 1.0);
    let evening = LinearRgba::new(0.90, 0.65, 0.50, 1.0);
    let night = LinearRgba::new(0.24, 0.28, 0.44, 1.0);

    Color::LinearRgba(sample_4stop(
        time_of_day,
        morning,
        noon,
        evening,
        night,
        lerp_linear_rgba,
    ))
}

pub fn sample_ambient_brightness(time_of_day: f32) -> f32 {
    sample_4stop(
        time_of_day,
        220.0,
        DAY_AMBIENT_BRIGHTNESS,
        200.0,
        NIGHT_AMBIENT_BRIGHTNESS,
        lerp_f32,
    )
}

pub fn sample_fog_color(time_of_day: f32) -> Color {
    let morning = LinearRgba::new(0.75, 0.62, 0.62, 1.0);
    let noon = LinearRgba::new(0.60, 0.72, 0.84, 1.0);
    let evening = LinearRgba::new(0.76, 0.48, 0.38, 1.0);
    let night = LinearRgba::new(0.045, 0.065, 0.12, 1.0);

    Color::LinearRgba(sample_4stop(
        time_of_day,
        morning,
        noon,
        evening,
        night,
        lerp_linear_rgba,
    ))
}

pub fn sample_fog_light_color(time_of_day: f32) -> Color {
    let morning = LinearRgba::new(1.0, 0.85, 0.65, 0.18);
    let noon = LinearRgba::new(1.0, 0.92, 0.78, 0.12);
    let evening = LinearRgba::new(1.0, 0.62, 0.38, 0.22);
    let night = LinearRgba::new(0.40, 0.48, 0.75, 0.06);

    Color::LinearRgba(sample_4stop(
        time_of_day,
        morning,
        noon,
        evening,
        night,
        lerp_linear_rgba,
    ))
}

pub fn sample_fog_light_exponent(time_of_day: f32) -> f32 {
    sample_4stop(time_of_day, 18.0, 20.0, 16.0, 14.0, lerp_f32)
}

pub fn sample_exposure(time_of_day: f32) -> f32 {
    sample_4stop(
        time_of_day,
        10.4,
        DAY_EXPOSURE_EV100,
        10.2,
        NIGHT_EXPOSURE_EV100,
        lerp_f32,
    )
}
