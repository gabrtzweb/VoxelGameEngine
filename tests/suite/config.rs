use voxel_game_engine::core::{LocalConfig, PerformancePreset};

#[test]
fn test_default_config() {
    let config = LocalConfig::default();
    assert_eq!(config.backend, "auto");
    assert_eq!(config.render_distance, 8);
    assert_eq!(config.lod_render_distance, 8);
    assert_eq!(config.simulation_distance, 8);
}

#[test]
fn test_presets() {
    let mut config = LocalConfig::default();
    config.apply_preset(PerformancePreset::LowSpec);
    assert_eq!(config.render_distance, 2);
    assert_eq!(config.lod_render_distance, 6);
    assert_eq!(config.simulation_distance, 4);
    assert!(!config.fog_enabled);

    config.apply_preset(PerformancePreset::Ultra);
    assert_eq!(config.render_distance, 16);
    assert_eq!(config.lod_render_distance, 32);
    assert!(config.fog_enabled);
    assert!(config.vsync_enabled);
}

#[test]
fn test_json_roundtrip() {
    let mut config = LocalConfig::default();
    config.apply_preset(PerformancePreset::LowSpec);
    let json = serde_json::to_string(&config).expect("Serialize failed");
    let parsed: LocalConfig = serde_json::from_str(&json).expect("Deserialize failed");
    assert_eq!(parsed.render_distance, 2);
    assert_eq!(parsed.preset, Some(PerformancePreset::LowSpec));
}
