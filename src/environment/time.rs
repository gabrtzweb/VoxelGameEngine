use bevy::prelude::*;

pub const F6_HOLD_THRESHOLD: f32 = 0.25;
pub const F6_SCRUB_SPEED: f32 = 0.22;

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

pub fn handle_environment_input(
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

pub fn advance_environment_clock(
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
