pub mod celestial;
pub mod clouds;
pub mod stars;

use bevy::{camera::Exposure, prelude::*};

use crate::voxel::{VOXEL_SIZE, chunk::CHUNK_SIZE, chunk_manager::ChunkStreamingSettings};

pub const DAY_SUN_ILLUMINANCE: f32 = 6_500.0;
pub const DAY_FILL_ILLUMINANCE: f32 = 2_200.0;
pub const DAY_AMBIENT_BRIGHTNESS: f32 = 450.0;
pub const DAY_EXPOSURE_EV100: f32 = 11.0;

pub const NIGHT_MOON_ILLUMINANCE: f32 = 450.0;
pub const NIGHT_AMBIENT_BRIGHTNESS: f32 = 28.0;
pub const NIGHT_EXPOSURE_EV100: f32 = 9.2;

const FOG_START_FACTOR: f32 = 0.50;
const FOG_END_FACTOR: f32 = 0.90;
const F6_HOLD_THRESHOLD: f32 = 0.25;
const F6_SCRUB_SPEED: f32 = 0.22;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DayPhase {
    Morning,
    #[default]
    Noon,
    Evening,
    Night,
}

impl DayPhase {
    pub fn target_time(self) -> f32 {
        match self {
            DayPhase::Morning => 0.00,
            DayPhase::Noon => 0.25,
            DayPhase::Evening => 0.50,
            DayPhase::Night => 0.75,
        }
    }

    pub fn next(self) -> Self {
        match self {
            DayPhase::Morning => DayPhase::Noon,
            DayPhase::Noon => DayPhase::Evening,
            DayPhase::Evening => DayPhase::Night,
            DayPhase::Night => DayPhase::Morning,
        }
    }

    pub fn from_time(time_of_day: f32) -> Self {
        let t = time_of_day.rem_euclid(1.0);
        if !(0.125..0.875).contains(&t) {
            DayPhase::Morning
        } else if t < 0.375 {
            DayPhase::Noon
        } else if t < 0.625 {
            DayPhase::Evening
        } else {
            DayPhase::Night
        }
    }
}

pub const MOON_PHASE_NAMES: [&str; 8] = [
    "Full Moon",
    "Waning Gibbous",
    "Third Quarter",
    "Waning Crescent",
    "New Moon",
    "Waxing Crescent",
    "First Quarter",
    "Waxing Gibbous",
];

#[derive(Resource)]
pub struct EnvironmentState {
    pub time_of_day: f32,
    pub day_length_seconds: f32,
    pub day_count: u32,
    pub phase: DayPhase,
    pub is_time_paused: bool,
    pub f6_hold_duration: f32,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            time_of_day: 0.20,
            day_length_seconds: 600.0,
            day_count: 0,
            phase: DayPhase::Noon,
            is_time_paused: false,
            f6_hold_duration: 0.0,
        }
    }
}

impl EnvironmentState {
    pub fn moon_phase(&self) -> usize {
        (self.day_count as usize) % 8
    }

    pub fn moon_phase_name(&self) -> &'static str {
        MOON_PHASE_NAMES[self.moon_phase()]
    }
}

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        let default_state = EnvironmentState::default();
        let initial_clear = sample_sky_color(default_state.time_of_day);
        let initial_ambient = sample_ambient_color(default_state.time_of_day);
        let initial_brightness = sample_ambient_brightness(default_state.time_of_day);

        app.insert_resource(default_state)
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
                    sync_starfield_system,
                    sync_cloud_system,
                )
                    .chain(),
            );
    }
}

fn handle_environment_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EnvironmentState>,
) {
    let delta = time.delta_secs();

    if keyboard.pressed(KeyCode::F6) {
        state.f6_hold_duration += delta;

        // If held past threshold, scrub time forward continuously.
        if state.f6_hold_duration > F6_HOLD_THRESHOLD {
            let prev = state.time_of_day;
            let next = (prev + delta * F6_SCRUB_SPEED).rem_euclid(1.0);
            if next < prev {
                state.day_count = state.day_count.wrapping_add(1);
            }
            state.time_of_day = next;
            state.phase = DayPhase::from_time(next);
        }
    }

    if keyboard.just_released(KeyCode::F6) {
        // If release happens quickly, treat it as a single-click phase snap.
        if state.f6_hold_duration <= F6_HOLD_THRESHOLD {
            let next_phase = state.phase.next();
            let target_time = next_phase.target_time();
            if target_time <= state.time_of_day {
                state.day_count = state.day_count.wrapping_add(1);
            }
            state.time_of_day = target_time;
            state.phase = next_phase;
            info!(
                "Phase stepped to {:?} (Day {}, Moon: {})",
                state.phase,
                state.day_count,
                state.moon_phase_name()
            );
        }
        state.f6_hold_duration = 0.0;
    }
}

fn advance_environment_clock(time: Res<Time>, mut state: ResMut<EnvironmentState>) {
    if state.is_time_paused || state.day_length_seconds <= 0.0 {
        return;
    }

    let delta = time.delta_secs();
    let prev = state.time_of_day;
    let advance = delta / state.day_length_seconds;
    let next = prev + advance;

    if next >= 1.0 {
        state.day_count = state.day_count.wrapping_add(next.floor() as u32);
    }

    let wrapped = next.rem_euclid(1.0);
    state.time_of_day = wrapped;
    state.phase = DayPhase::from_time(wrapped);
}

fn update_atmosphere(
    state: Res<EnvironmentState>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    camera: Single<(&mut DistanceFog, &mut Exposure), With<Camera3d>>,
) {
    let t = state.time_of_day;

    clear_color.0 = sample_sky_color(t);
    ambient.color = sample_ambient_color(t);
    ambient.brightness = sample_ambient_brightness(t);

    let (mut fog, mut exposure) = camera.into_inner();
    fog.color = sample_fog_color(t);
    fog.directional_light_color = sample_fog_light_color(t);
    fog.directional_light_exponent = sample_fog_light_exponent(t);
    exposure.ev100 = sample_exposure(t);
}

fn sync_fog_distance(
    settings: Res<ChunkStreamingSettings>,
    fog: Single<&mut DistanceFog, With<Camera3d>>,
) {
    let mut fog = fog.into_inner();
    let chunk_world_size = CHUNK_SIZE as f32 * VOXEL_SIZE;
    let render_radius = settings.render_distance.max(1) as f32 * chunk_world_size;
    fog.falloff = FogFalloff::Linear {
        start: render_radius * FOG_START_FACTOR,
        end: render_radius * FOG_END_FACTOR,
    };
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_celestial_system(
    state: Res<EnvironmentState>,
    camera: Single<&Transform, With<Camera3d>>,
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
) {
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
        state.moon_phase(),
        DAY_SUN_ILLUMINANCE,
        DAY_FILL_ILLUMINANCE,
        NIGHT_MOON_ILLUMINANCE,
    );
}

fn sync_starfield_system(
    state: Res<EnvironmentState>,
    camera: Single<&Transform, With<Camera3d>>,
    star_query: stars::StarQuery,
    material_handle: Res<stars::StarfieldMaterialHandle>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    stars::sync_starfield(
        camera,
        star_query,
        material_handle,
        materials,
        state.time_of_day,
    );
}

fn sync_cloud_system(
    time: Res<Time>,
    state: Res<EnvironmentState>,
    camera: Single<&Transform, With<Camera3d>>,
    cloud: Single<&mut Transform, (With<clouds::CloudVisual>, Without<Camera3d>)>,
    material_handle: Res<clouds::CloudMaterialHandle>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    clouds::sync_clouds(
        time,
        camera,
        cloud,
        material_handle,
        materials,
        state.time_of_day,
    );
}

// -------------------------------------------------------------------------
// Smooth 4-Phase Palette Interpolation
// -------------------------------------------------------------------------

fn sample_4stop<T: Copy>(
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

fn lerp_linear_rgba(a: LinearRgba, b: LinearRgba, factor: f32) -> LinearRgba {
    LinearRgba::new(
        a.red + (b.red - a.red) * factor,
        a.green + (b.green - a.green) * factor,
        a.blue + (b.blue - a.blue) * factor,
        a.alpha + (b.alpha - a.alpha) * factor,
    )
}

fn lerp_f32(a: f32, b: f32, factor: f32) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_phase_progression_loops_cleanly() {
        assert_eq!(DayPhase::Morning.next(), DayPhase::Noon);
        assert_eq!(DayPhase::Noon.next(), DayPhase::Evening);
        assert_eq!(DayPhase::Evening.next(), DayPhase::Night);
        assert_eq!(DayPhase::Night.next(), DayPhase::Morning);
    }

    #[test]
    fn moon_phase_rotates_through_all_eight_variants() {
        let mut state = EnvironmentState::default();
        for day in 0..8 {
            state.day_count = day;
            assert_eq!(state.moon_phase(), day as usize);
            assert_eq!(state.moon_phase_name(), MOON_PHASE_NAMES[day as usize]);
        }
        state.day_count = 8;
        assert_eq!(state.moon_phase(), 0);
    }

    #[test]
    fn atmosphere_sampler_continuous_at_boundaries() {
        let eps = 0.0001;
        let c_before = sample_sky_color(0.25 - eps);
        let c_at = sample_sky_color(0.25);
        let c_after = sample_sky_color(0.25 + eps);

        if let (Color::LinearRgba(a), Color::LinearRgba(b), Color::LinearRgba(c)) =
            (c_before, c_at, c_after)
        {
            assert!((a.red - b.red).abs() < 0.01);
            assert!((b.red - c.red).abs() < 0.01);
        }
    }
}
