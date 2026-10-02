use bevy::{camera::Exposure, prelude::*};

use super::time::EnvironmentState;
use crate::{
    player::PlayerCamera,
    world::{CHUNK_SIZE, VOXEL_SIZE, streaming::ChunkStreamingSettings},
};

pub const DAY_SUN_ILLUMINANCE: f32 = 8_500.0;
pub const DAY_FILL_ILLUMINANCE: f32 = 1_200.0;
pub const DAY_AMBIENT_BRIGHTNESS: f32 = 1_000.0;
pub const DAY_EXPOSURE_EV100: f32 = 11.1;

pub const NIGHT_MOON_ILLUMINANCE: f32 = 600.0;
pub const NIGHT_AMBIENT_BRIGHTNESS: f32 = 50.0;
pub const NIGHT_EXPOSURE_EV100: f32 = 9.0;

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
    atmo_state: Option<Res<BiomeAtmosphereState>>,
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
        let biome_atmo = atmo_state.as_ref().map(|s| s.current).unwrap_or_default();

        let base_sky = sample_sky_color(t);
        let base_ambient = sample_ambient_color(t);
        let base_fog = sample_fog_color(t);

        let blend_linear = |base: Color, filter: LinearRgba| -> Color {
            let b = base.to_linear();
            Color::LinearRgba(LinearRgba::new(
                (b.red * filter.red).clamp(0.0, 2.0),
                (b.green * filter.green).clamp(0.0, 2.0),
                (b.blue * filter.blue).clamp(0.0, 2.0),
                1.0,
            ))
        };

        clear_color.0 = blend_linear(base_sky, biome_atmo.fog_color_filter);
        ambient.color = blend_linear(base_ambient, biome_atmo.ambient_color_filter);
        ambient.brightness = sample_ambient_brightness(t);

        fog.color = blend_linear(base_fog, biome_atmo.fog_color_filter);
        fog.directional_light_color = sample_fog_light_color(t);
        fog.directional_light_exponent = sample_fog_light_exponent(t);
        exposure.ev100 = sample_exposure(t) + biome_atmo.exposure_offset;
    }
}

pub fn sync_fog_distance(
    settings: Res<ChunkStreamingSettings>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
    camera: Single<&mut DistanceFog, (With<Camera3d>, With<PlayerCamera>)>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    atmo_state: Option<Res<BiomeAtmosphereState>>,
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
        let biome_mult = atmo_state
            .as_ref()
            .map(|s| s.current.fog_distance_multiplier)
            .unwrap_or(1.0);

        let chunk_world_size = CHUNK_SIZE as f32 * VOXEL_SIZE;
        let total_chunks = (settings.render_distance + settings.lod_render_distance.max(0)).max(1);
        let render_radius = total_chunks as f32 * chunk_world_size;
        fog.falloff = FogFalloff::Linear {
            start: render_radius * FOG_START_FACTOR * biome_mult,
            end: render_radius * FOG_END_FACTOR * biome_mult,
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
        800.0,
        DAY_AMBIENT_BRIGHTNESS,
        800.0,
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

#[derive(Clone, Copy, Debug, Reflect)]
pub struct BiomeAtmosphere {
    pub fog_color_filter: LinearRgba,
    pub fog_distance_multiplier: f32,
    pub ambient_color_filter: LinearRgba,
    pub exposure_offset: f32,
}

impl Default for BiomeAtmosphere {
    fn default() -> Self {
        Self {
            fog_color_filter: LinearRgba::new(1.0, 1.0, 1.0, 1.0),
            fog_distance_multiplier: 1.0,
            ambient_color_filter: LinearRgba::new(1.0, 1.0, 1.0, 1.0),
            exposure_offset: 0.0,
        }
    }
}

impl BiomeAtmosphere {
    pub fn lerp(&self, target: &BiomeAtmosphere, factor: f32) -> BiomeAtmosphere {
        let f = factor.clamp(0.0, 1.0);
        BiomeAtmosphere {
            fog_color_filter: LinearRgba::new(
                self.fog_color_filter.red
                    + (target.fog_color_filter.red - self.fog_color_filter.red) * f,
                self.fog_color_filter.green
                    + (target.fog_color_filter.green - self.fog_color_filter.green) * f,
                self.fog_color_filter.blue
                    + (target.fog_color_filter.blue - self.fog_color_filter.blue) * f,
                1.0,
            ),
            fog_distance_multiplier: self.fog_distance_multiplier
                + (target.fog_distance_multiplier - self.fog_distance_multiplier) * f,
            ambient_color_filter: LinearRgba::new(
                self.ambient_color_filter.red
                    + (target.ambient_color_filter.red - self.ambient_color_filter.red) * f,
                self.ambient_color_filter.green
                    + (target.ambient_color_filter.green - self.ambient_color_filter.green) * f,
                self.ambient_color_filter.blue
                    + (target.ambient_color_filter.blue - self.ambient_color_filter.blue) * f,
                1.0,
            ),
            exposure_offset: self.exposure_offset
                + (target.exposure_offset - self.exposure_offset) * f,
        }
    }
}

/// Runtime resource tracking smooth exponential blending between biome atmospheres.
#[derive(Resource, Debug, Clone, Reflect)]
pub struct BiomeAtmosphereState {
    pub current: BiomeAtmosphere,
    pub transition_speed: f32,
    pub initialized: bool,
}

impl Default for BiomeAtmosphereState {
    fn default() -> Self {
        Self {
            current: BiomeAtmosphere::default(),
            transition_speed: 1.5,
            initialized: false,
        }
    }
}

/// Progressively eases current atmospheric parameters toward the active biome's target profile.
pub fn update_biome_atmosphere_transition(
    time: Res<Time>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    mut atmo_state: ResMut<BiomeAtmosphereState>,
) {
    let target = env_status
        .as_ref()
        .map(|s| sample_biome_atmosphere(s.current_biome))
        .unwrap_or_default();

    if !atmo_state.initialized {
        atmo_state.current = target;
        atmo_state.initialized = true;
        return;
    }

    let dt = time.delta_secs();
    // Frame-rate independent exponential smoothing
    let factor = 1.0 - (-atmo_state.transition_speed * dt).exp();
    atmo_state.current = atmo_state.current.lerp(&target, factor);
}

pub fn sample_biome_atmosphere(biome: crate::generation::BiomeType) -> BiomeAtmosphere {
    use crate::generation::BiomeType;
    match biome {
        // Wetlands & Swamps: Low-altitude eerie mist and murky green/cyan haze
        BiomeType::CypressSwamp
        | BiomeType::FungalBog
        | BiomeType::MangroveSwamp
        | BiomeType::Marshland
        | BiomeType::PeatBog
        | BiomeType::SludgeWastes
        | BiomeType::TarPits
        | BiomeType::WeepingBayou => BiomeAtmosphere {
            fog_color_filter: LinearRgba::new(0.68, 0.85, 0.72, 1.0),
            fog_distance_multiplier: 0.68,
            ambient_color_filter: LinearRgba::new(0.82, 0.95, 0.86, 1.0),
            exposure_offset: -0.25,
        },

        // Volcanic Wastelands & Ash Barrens: Dark ash haze, warm ember atmosphere
        BiomeType::VolcanicFields
        | BiomeType::VolcanicPlains
        | BiomeType::DeadwoodThicket
        | BiomeType::ScorchedWastes => BiomeAtmosphere {
            fog_color_filter: LinearRgba::new(0.72, 0.62, 0.58, 1.0),
            fog_distance_multiplier: 0.72,
            ambient_color_filter: LinearRgba::new(1.05, 0.88, 0.78, 1.0),
            exposure_offset: -0.30,
        },

        // Alpine & Glacial: Crisp high-exposure distance fog, bright cyan-white
        BiomeType::GlacialPeaks
        | BiomeType::AlpineTundra
        | BiomeType::FrozenCaldera
        | BiomeType::KarstPeaks
        | BiomeType::JaggedCrags => BiomeAtmosphere {
            fog_color_filter: LinearRgba::new(0.94, 0.97, 1.06, 1.0),
            fog_distance_multiplier: 1.15,
            ambient_color_filter: LinearRgba::new(0.96, 0.98, 1.04, 1.0),
            exposure_offset: 0.35,
        },

        // Arid, Deserts & Canyons: Warm golden atmospheric haze
        BiomeType::DuneDesert
        | BiomeType::Badlands
        | BiomeType::WhiteDesert
        | BiomeType::PaintedDesert
        | BiomeType::WindsweptCanyons
        | BiomeType::OutbackScrubland => BiomeAtmosphere {
            fog_color_filter: LinearRgba::new(1.06, 0.96, 0.84, 1.0),
            fog_distance_multiplier: 0.90,
            ambient_color_filter: LinearRgba::new(1.06, 0.97, 0.86, 1.0),
            exposure_offset: 0.15,
        },

        // Ocean & Coastal: Deep blue horizon haze
        BiomeType::AbyssalTrench
        | BiomeType::DeepOcean
        | BiomeType::TemperateOcean
        | BiomeType::ChalkCliffs
        | BiomeType::CoastalCrags => BiomeAtmosphere {
            fog_color_filter: LinearRgba::new(0.88, 0.95, 1.04, 1.0),
            fog_distance_multiplier: 1.05,
            ambient_color_filter: LinearRgba::new(0.94, 0.97, 1.02, 1.0),
            exposure_offset: 0.05,
        },

        // Plains, Moors & Forests: Soft golden warmth, clear lush atmosphere
        _ => BiomeAtmosphere::default(),
    }
}
