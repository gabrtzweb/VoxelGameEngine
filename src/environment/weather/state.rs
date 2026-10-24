use bevy::prelude::*;

use crate::environment::time::EnvironmentState;

pub const F7_HOLD_THRESHOLD: f32 = 0.25;

/// High-level weather classifications supported by the simulation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum WeatherType {
    #[default]
    Clear,
    Rain,
    HeavyRain,
    Thunderstorm,
    Snow,
    Blizzard,
    DustStorm,
}

impl WeatherType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Clear => "Clear Sky",
            Self::Rain => "Rain",
            Self::HeavyRain => "Heavy Downpour",
            Self::Thunderstorm => "Thunderstorm",
            Self::Snow => "Gentle Snow",
            Self::Blizzard => "Blizzard",
            Self::DustStorm => "Desert Dust Storm",
        }
    }

    pub fn is_rain(self) -> bool {
        matches!(self, Self::Rain | Self::HeavyRain | Self::Thunderstorm)
    }

    pub fn is_snow(self) -> bool {
        matches!(self, Self::Snow | Self::Blizzard)
    }

    pub fn is_dust(self) -> bool {
        matches!(self, Self::DustStorm)
    }

    pub fn is_storm(self) -> bool {
        matches!(self, Self::HeavyRain | Self::Thunderstorm | Self::Blizzard | Self::DustStorm)
    }

    pub fn target_rain_intensity(self) -> f32 {
        match self {
            Self::Clear | Self::Snow | Self::Blizzard | Self::DustStorm => 0.0,
            Self::Rain => 0.55,
            Self::HeavyRain => 0.85,
            Self::Thunderstorm => 1.0,
        }
    }

    pub fn target_thunder_intensity(self) -> f32 {
        match self {
            Self::Thunderstorm => 1.0,
            Self::HeavyRain => 0.35,
            _ => 0.0,
        }
    }

    pub fn target_snow_intensity(self) -> f32 {
        match self {
            Self::Snow => 0.55,
            Self::Blizzard => 1.0,
            _ => 0.0,
        }
    }

    pub fn target_dust_intensity(self) -> f32 {
        match self {
            Self::DustStorm => 1.0,
            _ => 0.0,
        }
    }

    pub fn cycle_next(self) -> Self {
        match self {
            Self::Clear => Self::Rain,
            Self::Rain => Self::HeavyRain,
            Self::HeavyRain => Self::Thunderstorm,
            Self::Thunderstorm => Self::Snow,
            Self::Snow => Self::Blizzard,
            Self::Blizzard => Self::DustStorm,
            Self::DustStorm => Self::Clear,
        }
    }
}

/// Central state manager resource for the dynamic weather system.
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct WeatherManager {
    /// Active baseline weather type.
    pub current: WeatherType,
    /// Destination weather type during cross-fade transitions.
    pub target: WeatherType,
    /// Transition progress [0.0..=1.0]. 1.0 indicates full convergence on `target`.
    pub transition: f32,
    /// Rate of transition progress per second (0.33 => ~3 seconds crossfade).
    pub transition_speed: f32,

    /// Smooth continuous precipitation and storm intensity channels [0.0..=1.0].
    pub rain_intensity: f32,
    pub thunder_intensity: f32,
    pub snow_intensity: f32,
    pub dust_intensity: f32,

    /// Whether the user has manually forced weather via F7 override.
    pub is_manual_override: bool,

    /// Automatic weather cycle scheduler timer (in seconds).
    pub cycle_timer: f32,
    pub next_cycle_duration: f32,

    /// Timer for post-precipitation ground fog/mist.
    pub after_weather_timer: f32,

    /// Keypress tracking for F7 hold vs tap.
    pub f7_hold_duration: f32,

    /// Transient notification message for developer feedback.
    pub hud_toast: Option<(String, f32)>,
}

impl Default for WeatherManager {
    fn default() -> Self {
        Self {
            current: WeatherType::Clear,
            target: WeatherType::Clear,
            transition: 1.0,
            transition_speed: 0.35,
            rain_intensity: 0.0,
            thunder_intensity: 0.0,
            snow_intensity: 0.0,
            dust_intensity: 0.0,
            is_manual_override: false,
            cycle_timer: 0.0,
            next_cycle_duration: 600.0, // 10 minutes default clear period
            after_weather_timer: 0.0,
            f7_hold_duration: 0.0,
            hud_toast: None,
        }
    }
}

impl WeatherManager {
    /// Forces immediate or smooth transition to a target weather type.
    pub fn set_target_weather(&mut self, target: WeatherType) {
        if self.target != target {
            self.current = if self.transition >= 0.95 {
                self.target
            } else {
                self.current
            };
            self.target = target;
            self.transition = 0.0;
        }
    }

    /// Cycles manually to the next weather variant in the test catalog.
    pub fn cycle_manual_weather(&mut self) {
        let next = self.target.cycle_next();
        self.is_manual_override = true;
        self.set_target_weather(next);
        self.hud_toast = Some((
            format!("Weather set to: {} [Manual Override]", next.name()),
            3.0,
        ));
        info!(
            "Weather manual cycle: {} -> {} (Manual override: true)",
            self.current.name(),
            next.name()
        );
    }

    /// Restores natural procedural/automatic weather cycles.
    pub fn restore_automatic_weather(&mut self) {
        self.is_manual_override = false;
        self.cycle_timer = 0.0;
        self.next_cycle_duration = 300.0;
        self.hud_toast = Some((
            "Weather: Automatic / Dynamic Simulation Restored".to_string(),
            3.0,
        ));
        info!("Weather manual override cleared: dynamic weather resumed");
    }

    /// Overall precipitation factor across all forms (rain, snow, dust).
    pub fn any_precipitation_intensity(&self) -> f32 {
        self.rain_intensity
            .max(self.snow_intensity)
            .max(self.dust_intensity)
    }
}

/// System handling F7 hotkey inputs to manually cycle or restore weather states.
pub fn handle_weather_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut weather: ResMut<WeatherManager>,
) {
    let delta = time.delta_secs();

    if keyboard.pressed(KeyCode::F7) {
        weather.f7_hold_duration += delta;
    }

    if keyboard.just_released(KeyCode::F7) {
        let shift_held =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        if shift_held {
            // Shift + F7 toggles back to automatic / dynamic weather
            weather.restore_automatic_weather();
        } else if weather.f7_hold_duration <= F7_HOLD_THRESHOLD {
            // Tap F7: cycle to next weather type
            weather.cycle_manual_weather();
        } else {
            // Long hold (> 0.25s): restore automatic simulation
            weather.restore_automatic_weather();
        }
        weather.f7_hold_duration = 0.0;
    }

    // Tick HUD toast notification
    if let Some((_, ref mut timer)) = weather.hud_toast {
        *timer -= delta;
        if *timer <= 0.0 {
            weather.hud_toast = None;
        }
    }
}

/// Core simulation system advancing weather timers and interpolating intensities.
pub fn advance_weather_simulation(
    time: Res<Time>,
    env_state: Option<Res<EnvironmentState>>,
    mut weather: ResMut<WeatherManager>,
) {
    let delta = time.delta_secs();

    // 1. Advance automatic cycle scheduler if not locked by manual override
    if !weather.is_manual_override {
        weather.cycle_timer += delta;
        if weather.cycle_timer >= weather.next_cycle_duration {
            weather.cycle_timer = 0.0;

            // Pick next weather based on pseudo-random cycle
            let in_game_time = env_state
                .as_ref()
                .map(|e| e.total_in_game_seconds())
                .unwrap_or(0.0);
            let seed = (in_game_time as u64).wrapping_mul(1103515245) % 100;

            let next_target = if seed < 60 {
                // 60% Clear
                weather.next_cycle_duration = 480.0 + (seed as f32) * 5.0; // 8 - 16 min
                WeatherType::Clear
            } else if seed < 80 {
                // 20% Rain
                weather.next_cycle_duration = 240.0 + (seed as f32) * 3.0; // 4 - 8 min
                WeatherType::Rain
            } else if seed < 90 {
                // 10% Heavy Rain
                weather.next_cycle_duration = 180.0 + (seed as f32) * 2.0;
                WeatherType::HeavyRain
            } else {
                // 10% Thunderstorm
                weather.next_cycle_duration = 120.0 + (seed as f32) * 2.0;
                WeatherType::Thunderstorm
            };

            weather.set_target_weather(next_target);
            info!(
                "Dynamic Weather transition scheduled: {} (duration: {:.0}s)",
                next_target.name(),
                weather.next_cycle_duration
            );
        }
    }

    // 2. Interpolate transition progress towards target
    if weather.transition < 1.0 {
        weather.transition = (weather.transition + delta * weather.transition_speed).min(1.0);
        if weather.transition >= 1.0 {
            let previous = weather.current;
            weather.current = weather.target;

            // If transitioning away from rain/storm, start post-weather ground mist timer
            if previous.is_rain() && !weather.current.is_rain() {
                weather.after_weather_timer = 45.0; // 45 seconds of ground mist
            }
        }
    }

    // 3. Smoothly converge intensity channels towards current target values
    let target_rain = weather.target.target_rain_intensity();
    let target_thunder = weather.target.target_thunder_intensity();
    let target_snow = weather.target.target_snow_intensity();
    let target_dust = weather.target.target_dust_intensity();

    let lerp_speed = 0.40 * delta;
    weather.rain_intensity = weather.rain_intensity.lerp(target_rain, lerp_speed);
    weather.thunder_intensity = weather.thunder_intensity.lerp(target_thunder, lerp_speed);
    weather.snow_intensity = weather.snow_intensity.lerp(target_snow, lerp_speed);
    weather.dust_intensity = weather.dust_intensity.lerp(target_dust, lerp_speed);

    // 4. Update post-weather ground mist timer
    if weather.after_weather_timer > 0.0 {
        weather.after_weather_timer = (weather.after_weather_timer - delta).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_type_cycling() {
        let mut w = WeatherType::Clear;
        w = w.cycle_next();
        assert_eq!(w, WeatherType::Rain);
        w = w.cycle_next();
        assert_eq!(w, WeatherType::HeavyRain);
        w = w.cycle_next();
        assert_eq!(w, WeatherType::Thunderstorm);
        w = w.cycle_next();
        assert_eq!(w, WeatherType::Snow);
        w = w.cycle_next();
        assert_eq!(w, WeatherType::Blizzard);
        w = w.cycle_next();
        assert_eq!(w, WeatherType::DustStorm);
        w = w.cycle_next();
        assert_eq!(w, WeatherType::Clear);
    }

    #[test]
    fn test_weather_manager_manual_override() {
        let mut manager = WeatherManager::default();
        assert!(!manager.is_manual_override);
        assert_eq!(manager.target, WeatherType::Clear);

        manager.cycle_manual_weather();
        assert!(manager.is_manual_override);
        assert_eq!(manager.target, WeatherType::Rain);
        assert_eq!(manager.transition, 0.0);

        manager.restore_automatic_weather();
        assert!(!manager.is_manual_override);
    }

    #[test]
    fn test_weather_intensities() {
        assert_eq!(WeatherType::Clear.target_rain_intensity(), 0.0);
        assert!(WeatherType::Rain.target_rain_intensity() > 0.0);
        assert_eq!(WeatherType::Thunderstorm.target_thunder_intensity(), 1.0);
        assert_eq!(WeatherType::Blizzard.target_snow_intensity(), 1.0);
        assert_eq!(WeatherType::DustStorm.target_dust_intensity(), 1.0);
    }
}
