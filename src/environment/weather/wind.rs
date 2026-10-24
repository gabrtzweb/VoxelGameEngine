use bevy::prelude::*;

use super::state::WeatherManager;
use crate::{environment::time::EnvironmentState, player::PlayerCamera};

/// 3D Continuous harmonic wind simulation resource.
/// Implements Particle Rain's non-repeating dual harmonic wave formula.
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct WindState {
    /// Active 3D wind velocity vector at the player camera position (m/s).
    pub current_wind: Vec3,
    /// Wind scalar speed (m/s).
    pub current_speed: f32,
    /// Wind compass direction in radians (horizontal plane).
    pub current_heading: f32,

    /// Base cardinal orientation vector of the prevailing wind.
    pub base_direction: Vec2,
    /// Rate at which wind gusts evolve over time.
    pub modulation_speed: f32,
    /// Spatial frequency of localized wind gusts.
    pub gust_frequency: f32,
    /// Baseline breeze strength coefficient (m/s).
    pub base_strength: f32,
}

impl Default for WindState {
    fn default() -> Self {
        Self {
            current_wind: Vec3::new(2.5, 0.0, 1.2),
            current_speed: 2.77,
            current_heading: 0.44,
            base_direction: Vec2::new(0.85, 0.52).normalize(),
            modulation_speed: 0.025,
            gust_frequency: 0.045,
            base_strength: 3.2,
        }
    }
}

impl WindState {
    /// Harmonic dual-sine wave generator from Particle Rain:
    /// `[sin(coord * freq + time) * sin(0.5 * coord * freq + 0.5 * time)] * strength + strength`
    #[inline]
    pub fn one_wind_please(coord: f32, freq: f32, time: f32, strength: f32) -> f32 {
        let phase = coord * freq + time;
        let s1 = (phase).sin();
        let s2 = (0.5 * phase).sin();
        (s1 * s2) * strength + strength
    }

    /// Elevation lapse scaling factor from Particle Rain:
    /// Clamps smoothly between Y = 50.0 (calm valley) and Y = 90.0 (high mountain gusts).
    #[inline]
    pub fn y_level_multiplier(y: f32) -> f32 {
        ((y - 50.0) / 40.0).clamp(0.15, 1.0)
    }

    /// Evaluates the 3D wind vector at any arbitrary world coordinate and game time.
    pub fn sample_wind(
        &self,
        world_pos: Vec3,
        game_time_seconds: f64,
        storm_factor: f32,
    ) -> Vec3 {
        let t = (game_time_seconds as f32) * self.modulation_speed;
        let y_mult = Self::y_level_multiplier(world_pos.y);

        // Effective strength amplifies during heavy storms and blizzards
        let strength = self.base_strength * (1.0 + storm_factor * 2.2);

        let wind_x = Self::one_wind_please(world_pos.x, self.gust_frequency, t, strength)
            * self.base_direction.x;
        let wind_z = Self::one_wind_please(world_pos.z, self.gust_frequency, t, strength)
            * self.base_direction.y;

        Vec3::new(wind_x * y_mult, 0.0, wind_z * y_mult)
    }
}

/// System sampling and refreshing local wind parameters around the player camera.
pub fn update_wind_field(
    time: Res<Time>,
    env_state: Option<Res<EnvironmentState>>,
    weather: Option<Res<WeatherManager>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut wind: ResMut<WindState>,
) {
    let cam_pos = camera_query
        .iter()
        .next()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    let game_time = env_state
        .as_ref()
        .map(|e| e.total_in_game_seconds())
        .unwrap_or_else(|| time.elapsed_secs_f64());

    let storm_factor = weather.as_ref().map_or(0.0, |w| {
        w.thunder_intensity
            .max(w.snow_intensity * 0.8)
            .max(w.dust_intensity)
    });

    let sampled = wind.sample_wind(cam_pos, game_time, storm_factor);
    wind.current_wind = sampled;
    wind.current_speed = (sampled.x * sampled.x + sampled.z * sampled.z).sqrt();
    wind.current_heading = sampled.z.atan2(sampled.x);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_y_level_multiplier_clamping() {
        assert_eq!(WindState::y_level_multiplier(40.0), 0.15);
        assert_eq!(WindState::y_level_multiplier(50.0), 0.15);
        assert!((WindState::y_level_multiplier(70.0) - 0.50).abs() < 1e-5);
        assert_eq!(WindState::y_level_multiplier(90.0), 1.0);
        assert_eq!(WindState::y_level_multiplier(150.0), 1.0);
    }

    #[test]
    fn test_wind_vector_sampling() {
        let wind = WindState::default();
        let sample1 = wind.sample_wind(Vec3::new(0.0, 70.0, 0.0), 0.0, 0.0);
        assert_eq!(sample1.y, 0.0);
        assert!(sample1.x > 0.0);
        assert!(sample1.z > 0.0);

        let sample_storm = wind.sample_wind(Vec3::new(0.0, 70.0, 0.0), 0.0, 1.0);
        assert!(sample_storm.length() > sample1.length());
    }
}
