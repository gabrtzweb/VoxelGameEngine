use bevy::{camera::Exposure, prelude::*};

use super::time::EnvironmentState;
use crate::{
    player::PlayerCamera,
    world::{CHUNK_SIZE, VOXEL_SIZE, streaming::ChunkStreamingSettings},
};

pub const DAY_SUN_ILLUMINANCE: f32 = 7_000.0;
pub const DAY_FILL_ILLUMINANCE: f32 = 2_800.0;
pub const DAY_AMBIENT_BRIGHTNESS: f32 = 2_600.0;
pub const DAY_EXPOSURE_EV100: f32 = 11.2;

pub const NIGHT_MOON_ILLUMINANCE: f32 = 800.0;
pub const NIGHT_AMBIENT_BRIGHTNESS: f32 = 120.0;
pub const NIGHT_EXPOSURE_EV100: f32 = 9.0;

/// Subterranean ambient lighting constants:
/// Independent of whether the sun is up or down in the sky.
/// Provides rich, visible cavern ambiance (~1,450 lx) with an atmospheric
/// cool mineral slate tint that preserves block and ore character.
pub const SUBTERRANEAN_AMBIENT_BRIGHTNESS: f32 = 1_450.0;
pub const SUBTERRANEAN_AMBIENT_COLOR: Color = Color::srgb(0.68, 0.74, 0.86);
pub const SUBTERRANEAN_CLEAR_COLOR: Color = Color::srgb(0.14, 0.16, 0.22);
pub const SUBTERRANEAN_FOG_COLOR: Color = Color::srgb(0.18, 0.22, 0.28);

pub const FOG_START_FACTOR: f32 = 0.50;
pub const FOG_END_FACTOR: f32 = 0.90;

type CameraAtmosphereQuery<'w, 's> = Single<
    'w,
    's,
    (
        &'static mut DistanceFog,
        &'static mut Exposure,
        &'static GlobalTransform,
    ),
    (With<Camera3d>, With<PlayerCamera>),
>;

pub fn update_atmosphere(
    state: Res<EnvironmentState>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    camera: CameraAtmosphereQuery,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    weather: Option<Res<super::weather::WeatherManager>>,
    lightning: Option<Res<super::weather::LightningState>>,
) {
    let t = state.time_of_day;
    let (mut fog, mut exposure, camera_transform) = camera.into_inner();

    let is_underwater = env_status.as_ref().is_some_and(|s| s.is_camera_in_water);
    let underground_factor = env_status.as_ref().map_or(0.0, |s| s.underground_factor);
    let cam_y = camera_transform.translation().y;
    let deep_subterranean_factor = ((-cam_y) / 48.0).clamp(0.0, 1.0);

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
        let mut computed_sky = sample_sky_color(t);
        let mut computed_ambient = sample_ambient_color(t);
        let mut computed_fog = sample_fog_color(t);
        let mut computed_brightness = sample_ambient_brightness(t);
        let mut computed_exposure = sample_exposure(t);

        // Apply weather modulations if active
        if let Some(w) = weather {
            let rain = w.rain_intensity;
            let thunder = w.thunder_intensity;
            let snow = w.snow_intensity;
            let dust = w.dust_intensity;

            // 1. Modulate ambient light and exposure
            let dim_factor = (1.0 - rain * 0.42 - thunder * 0.38 - dust * 0.30).clamp(0.20, 1.0);
            computed_brightness *= dim_factor;
            computed_exposure -= rain * 0.65 + thunder * 0.85 + dust * 0.50;

            // 2. Weather sky and fog color targets
            let storm_sky = LinearRgba::new(0.24, 0.28, 0.35, 1.0);
            let snow_sky = LinearRgba::new(0.60, 0.64, 0.70, 1.0);
            let dust_sky = LinearRgba::new(0.68, 0.52, 0.32, 1.0);

            let storm_weight = (rain * 0.75 + thunder * 0.25).clamp(0.0, 1.0);
            let snow_weight = snow.clamp(0.0, 1.0);
            let dust_weight = dust.clamp(0.0, 1.0);

            let lerp_rgba = |a: LinearRgba, b: LinearRgba, factor: f32| -> LinearRgba {
                LinearRgba::new(
                    a.red + (b.red - a.red) * factor,
                    a.green + (b.green - a.green) * factor,
                    a.blue + (b.blue - a.blue) * factor,
                    a.alpha + (b.alpha - a.alpha) * factor,
                )
            };

            let sky_lin = computed_sky.to_linear();
            let mut final_sky = sky_lin;
            if storm_weight > 0.0 {
                final_sky = lerp_rgba(final_sky, storm_sky, storm_weight * 0.85);
            }
            if snow_weight > 0.0 {
                final_sky = lerp_rgba(final_sky, snow_sky, snow_weight * 0.75);
            }
            if dust_weight > 0.0 {
                final_sky = lerp_rgba(final_sky, dust_sky, dust_weight * 0.90);
            }
            computed_sky = Color::LinearRgba(final_sky);

            let fog_lin = computed_fog.to_linear();
            let mut final_fog = fog_lin;
            if storm_weight > 0.0 {
                final_fog = lerp_rgba(final_fog, storm_sky, storm_weight * 0.80);
            }
            if snow_weight > 0.0 {
                final_fog = lerp_rgba(final_fog, snow_sky, snow_weight * 0.70);
            }
            if dust_weight > 0.0 {
                final_fog = lerp_rgba(final_fog, dust_sky, dust_weight * 0.85);
            }
            computed_fog = Color::LinearRgba(final_fog);

            // Slightly shift ambient toward sky hue
            computed_ambient =
                Color::LinearRgba(lerp_rgba(computed_ambient.to_linear(), final_sky, 0.20));
        }

        // Full-scene instantaneous flash during lightning strikes
        if let Some(ref l) = lightning {
            if l.flash_intensity > 0.01 {
                let flash_col = Color::srgb(0.92, 0.95, 1.0);
                computed_sky = flash_col;
                computed_ambient = flash_col;
                computed_brightness += l.flash_intensity * 8000.0;
                computed_exposure += l.flash_intensity * 3.0;
            }
        }

        // Subterranean Lighting & Atmosphere:
        // Decouple caves from surface sun/moon cycle. Deep underground, caves maintain
        // a soft, luminous, neutral subterranean ambient level (~1,450 lx)
        // so stone walls, ores, and tunnels are clearly visible with vibrant textures.
        if underground_factor > 0.001 {
            let sub_ambient_target = lerp_f32(
                SUBTERRANEAN_AMBIENT_BRIGHTNESS,
                1_400.0,
                deep_subterranean_factor,
            );
            computed_brightness =
                lerp_f32(computed_brightness, sub_ambient_target, underground_factor);

            let lerp_rgba = |a: LinearRgba, b: LinearRgba, factor: f32| -> LinearRgba {
                LinearRgba::new(
                    a.red + (b.red - a.red) * factor,
                    a.green + (b.green - a.green) * factor,
                    a.blue + (b.blue - a.blue) * factor,
                    a.alpha + (b.alpha - a.alpha) * factor,
                )
            };

            computed_ambient = Color::LinearRgba(lerp_rgba(
                computed_ambient.to_linear(),
                SUBTERRANEAN_AMBIENT_COLOR.to_linear(),
                underground_factor,
            ));

            computed_sky = Color::LinearRgba(lerp_rgba(
                computed_sky.to_linear(),
                SUBTERRANEAN_CLEAR_COLOR.to_linear(),
                underground_factor,
            ));

            computed_fog = Color::LinearRgba(lerp_rgba(
                computed_fog.to_linear(),
                SUBTERRANEAN_FOG_COLOR.to_linear(),
                underground_factor,
            ));

            computed_exposure = lerp_f32(computed_exposure, 11.0, underground_factor);
        }

        clear_color.0 = computed_sky;
        ambient.color = computed_ambient;
        ambient.brightness = computed_brightness;

        fog.color = computed_fog;
        // Directional light forward-scattering in distance fog must be completely faded out underground,
        // preventing artificial sun glow spots or disc blurs from appearing on cave ceilings.
        let fog_sun_col = sample_fog_light_color(t);
        fog.directional_light_color = Color::LinearRgba(lerp_linear_rgba(
            fog_sun_col.to_linear(),
            LinearRgba::NONE,
            underground_factor,
        ));
        fog.directional_light_exponent = sample_fog_light_exponent(t);
        exposure.ev100 = computed_exposure;
    }
}

pub fn sync_fog_distance(
    settings: Res<ChunkStreamingSettings>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
    camera: Single<&mut DistanceFog, (With<Camera3d>, With<PlayerCamera>)>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    weather: Option<Res<super::weather::WeatherManager>>,
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
    let underground_factor = env_status.as_ref().map_or(0.0, |s| s.underground_factor);

    if is_underwater {
        fog.falloff = FogFalloff::Linear {
            start: 1.0,
            end: 22.0,
        };
    } else {
        let weather_mult = weather.as_ref().map_or(1.0, |w| {
            let intensity = (w.rain_intensity * 0.35
                + w.thunder_intensity * 0.25
                + w.snow_intensity * 0.45
                + w.dust_intensity * 0.55)
                .min(0.68);
            1.0 - intensity
        });

        let chunk_world_size = CHUNK_SIZE as f32 * VOXEL_SIZE;
        let total_chunks = (settings.render_distance + settings.lod_render_distance.max(0)).max(1);
        let render_radius = total_chunks as f32 * chunk_world_size;

        let surface_start = render_radius * FOG_START_FACTOR * weather_mult;
        let surface_end = render_radius * FOG_END_FACTOR * weather_mult;

        // In caverns, maintain standard render distance fog with gentle falloff
        // so players can see the extent of large caverns and chambers clearly without blinding mist.
        let cave_start = surface_start * 0.80;
        let cave_end = surface_end * 0.95;

        fog.falloff = FogFalloff::Linear {
            start: lerp_f32(surface_start, cave_start, underground_factor),
            end: lerp_f32(surface_end, cave_end, underground_factor),
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

#[inline(always)]
pub fn lerp_f32(a: f32, b: f32, factor: f32) -> f32 {
    crate::core::math::lerp(a, b, factor)
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
    let morning = LinearRgba::new(0.95, 0.82, 0.72, 1.0);
    let noon = LinearRgba::new(0.88, 0.94, 1.08, 1.0);
    let evening = LinearRgba::new(0.95, 0.72, 0.55, 1.0);
    let night = LinearRgba::new(0.32, 0.38, 0.58, 1.0);

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
        2_000.0,
        DAY_AMBIENT_BRIGHTNESS,
        2_000.0,
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


