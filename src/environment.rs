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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MoonPhase {
    #[default]
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl MoonPhase {
    pub fn from_day_of_month(day_of_month: u32) -> Self {
        match day_of_month {
            1..=3 => MoonPhase::NewMoon,
            4..=7 => MoonPhase::WaxingCrescent,
            8..=10 => MoonPhase::FirstQuarter,
            11..=14 => MoonPhase::WaxingGibbous,
            15..=17 => MoonPhase::FullMoon,
            18..=21 => MoonPhase::WaningGibbous,
            22..=24 => MoonPhase::LastQuarter,
            25..=28 => MoonPhase::WaningCrescent,
            _ => MoonPhase::NewMoon,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            MoonPhase::NewMoon => "New Moon",
            MoonPhase::WaxingCrescent => "Waxing Crescent",
            MoonPhase::FirstQuarter => "First Quarter",
            MoonPhase::WaxingGibbous => "Waxing Gibbous",
            MoonPhase::FullMoon => "Full Moon",
            MoonPhase::WaningGibbous => "Waning Gibbous",
            MoonPhase::LastQuarter => "Last Quarter",
            MoonPhase::WaningCrescent => "Waning Crescent",
        }
    }

    pub fn texture_index(self) -> usize {
        match self {
            MoonPhase::FullMoon => 0,
            MoonPhase::WaningGibbous => 1,
            MoonPhase::LastQuarter => 2,
            MoonPhase::WaningCrescent => 3,
            MoonPhase::NewMoon => 4,
            MoonPhase::WaxingCrescent => 5,
            MoonPhase::FirstQuarter => 6,
            MoonPhase::WaxingGibbous => 7,
        }
    }

    pub fn illuminance_factor(self) -> f32 {
        match self {
            MoonPhase::FullMoon => 1.0,
            MoonPhase::WaxingGibbous | MoonPhase::WaningGibbous => 0.75,
            MoonPhase::FirstQuarter | MoonPhase::LastQuarter => 0.50,
            MoonPhase::WaxingCrescent | MoonPhase::WaningCrescent => 0.25,
            MoonPhase::NewMoon => 0.05,
        }
    }
}

#[allow(dead_code)]
pub const MOON_PHASE_NAMES: [&str; 8] = [
    "Full Moon",
    "Waning Gibbous",
    "Last Quarter",
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
            day_length_seconds: 1440.0,
            day_count: 1,
            phase: DayPhase::Noon,
            is_time_paused: false,
            f6_hold_duration: 0.0,
        }
    }
}

impl EnvironmentState {
    pub fn day_of_month(&self) -> u32 {
        (self.day_count.saturating_sub(1) % 28) + 1
    }

    pub fn month(&self) -> u32 {
        (self.day_count.saturating_sub(1) / 28) + 1
    }

    pub fn month_of_year(&self) -> u32 {
        ((self.month() - 1) % 12) + 1
    }

    #[allow(dead_code)]
    pub fn year(&self) -> u32 {
        (self.day_count.saturating_sub(1) / 336) + 1
    }

    pub fn day_of_year(&self) -> u32 {
        (self.day_count.saturating_sub(1) % 336) + 1
    }

    pub fn day_of_season(&self) -> u32 {
        (self.day_of_year() - 1) % 84 + 1
    }

    pub fn season(&self) -> Season {
        match (self.month_of_year() - 1) / 3 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            3 => Season::Winter,
            _ => Season::Spring,
        }
    }

    pub fn moon_phase(&self) -> MoonPhase {
        MoonPhase::from_day_of_month(self.day_of_month())
    }

    pub fn moon_texture_index(&self) -> usize {
        self.moon_phase().texture_index()
    }

    pub fn moon_phase_name(&self) -> &'static str {
        self.moon_phase().name()
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
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    mut state: ResMut<EnvironmentState>,
) {
    if menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None) {
        return;
    }

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
                "Phase stepped to {:?} (Day {}/Month {}, Season: {:?}, Moon: {})",
                state.phase,
                state.day_of_month(),
                state.month(),
                state.season(),
                state.moon_phase_name()
            );
        }
        state.f6_hold_duration = 0.0;
    }
}

fn advance_environment_clock(
    time: Res<Time>,
    mut state: ResMut<EnvironmentState>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
) {
    if state.is_time_paused || state.day_length_seconds <= 0.0 {
        return;
    }

    if let Some(ref menu) = menu_state
        && (*menu.get() == crate::menu::MenuState::Pause
            || *menu.get() == crate::menu::MenuState::Settings)
    {
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

fn sync_fog_distance(
    settings: Res<ChunkStreamingSettings>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
    camera: Single<&mut DistanceFog, With<Camera3d>>,
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
        state.moon_texture_index(),
        state.moon_phase().illuminance_factor(),
        DAY_SUN_ILLUMINANCE,
        DAY_FILL_ILLUMINANCE,
        NIGHT_MOON_ILLUMINANCE,
    );
}

fn sync_starfield_system(
    state: Res<EnvironmentState>,
    camera: Single<&Transform, With<Camera3d>>,
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
    time: Res<Time>,
    state: Res<EnvironmentState>,
    camera: Single<(&Transform, &GlobalTransform), With<Camera3d>>,
    cloud: clouds::CloudQuery,
    material_handle: Res<clouds::CloudMaterialHandle>,
    materials: ResMut<Assets<StandardMaterial>>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
) {
    let is_underwater = env_status.as_ref().is_some_and(|s| s.is_camera_in_water);

    clouds::sync_clouds(
        time,
        camera,
        cloud,
        material_handle,
        materials,
        state.time_of_day,
        is_underwater,
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
    fn moon_phases_match_28_day_month_specification() {
        for day in 1..=3 {
            assert_eq!(MoonPhase::from_day_of_month(day), MoonPhase::NewMoon);
        }
        for day in 4..=7 {
            assert_eq!(MoonPhase::from_day_of_month(day), MoonPhase::WaxingCrescent);
        }
        for day in 8..=10 {
            assert_eq!(MoonPhase::from_day_of_month(day), MoonPhase::FirstQuarter);
        }
        for day in 11..=14 {
            assert_eq!(MoonPhase::from_day_of_month(day), MoonPhase::WaxingGibbous);
        }
        for day in 15..=17 {
            assert_eq!(MoonPhase::from_day_of_month(day), MoonPhase::FullMoon);
        }
        for day in 18..=21 {
            assert_eq!(MoonPhase::from_day_of_month(day), MoonPhase::WaningGibbous);
        }
        for day in 22..=24 {
            assert_eq!(MoonPhase::from_day_of_month(day), MoonPhase::LastQuarter);
        }
        for day in 25..=28 {
            assert_eq!(MoonPhase::from_day_of_month(day), MoonPhase::WaningCrescent);
        }
    }

    #[test]
    fn calendar_progression_and_seasons() {
        let mut state = EnvironmentState::default();
        assert_eq!(state.day_count, 1);
        assert_eq!(state.day_of_month(), 1);
        assert_eq!(state.month(), 1);
        assert_eq!(state.season(), Season::Spring);
        assert_eq!(state.day_of_season(), 1);
        assert_eq!(state.moon_phase(), MoonPhase::NewMoon);
        assert_eq!(state.day_length_seconds, 1440.0);

        // Day 28: end of month 1 (Spring)
        state.day_count = 28;
        assert_eq!(state.day_of_month(), 28);
        assert_eq!(state.month(), 1);
        assert_eq!(state.season(), Season::Spring);
        assert_eq!(state.day_of_season(), 28);
        assert_eq!(state.moon_phase(), MoonPhase::WaningCrescent);

        // Day 29: start of month 2 (Spring)
        state.day_count = 29;
        assert_eq!(state.day_of_month(), 1);
        assert_eq!(state.month(), 2);
        assert_eq!(state.season(), Season::Spring);
        assert_eq!(state.day_of_season(), 29);
        assert_eq!(state.moon_phase(), MoonPhase::NewMoon);

        // Day 84: end of month 3 / end of Spring
        state.day_count = 84;
        assert_eq!(state.day_of_month(), 28);
        assert_eq!(state.month(), 3);
        assert_eq!(state.season(), Season::Spring);
        assert_eq!(state.day_of_season(), 84);

        // Day 85: start of month 4 / start of Summer
        state.day_count = 85;
        assert_eq!(state.day_of_month(), 1);
        assert_eq!(state.month(), 4);
        assert_eq!(state.season(), Season::Summer);
        assert_eq!(state.day_of_season(), 1);

        // Day 169: start of month 7 / start of Autumn
        state.day_count = 169;
        assert_eq!(state.day_of_month(), 1);
        assert_eq!(state.month(), 7);
        assert_eq!(state.season(), Season::Autumn);
        assert_eq!(state.day_of_season(), 1);

        // Day 253: start of month 10 / start of Winter
        state.day_count = 253;
        assert_eq!(state.day_of_month(), 1);
        assert_eq!(state.month(), 10);
        assert_eq!(state.season(), Season::Winter);
        assert_eq!(state.day_of_season(), 1);

        // Day 336: end of month 12 / end of Year 1
        state.day_count = 336;
        assert_eq!(state.day_of_month(), 28);
        assert_eq!(state.month(), 12);
        assert_eq!(state.season(), Season::Winter);
        assert_eq!(state.day_of_season(), 84);
        assert_eq!(state.year(), 1);

        // Day 337: Year 2, Month 13 (month_of_year 1), Spring Day 1
        state.day_count = 337;
        assert_eq!(state.day_of_month(), 1);
        assert_eq!(state.month(), 13);
        assert_eq!(state.month_of_year(), 1);
        assert_eq!(state.season(), Season::Spring);
        assert_eq!(state.day_of_season(), 1);
        assert_eq!(state.year(), 2);
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
