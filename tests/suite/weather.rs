use bevy::prelude::Vec3;
use voxel_game_engine::environment::weather::{WeatherManager, WeatherType, WindState};

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
