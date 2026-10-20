use voxel_game_engine::core::{LocalConfig, PerformancePreset};

#[test]
fn test_default_config() {
    let config = LocalConfig::default();
    assert_eq!(config.backend, "auto");
    assert_eq!(config.render_distance, 8);
    assert_eq!(config.lod_render_distance, 8);
    assert_eq!(config.simulation_distance, 8);
    assert!(config.fog_enabled);
    assert!(!config.full_grass);
    assert!(!config.transmission_enabled);
    assert!(config.shadows_enabled);
    assert!(config.god_rays_enabled);
    assert_eq!(config.render_scale, 1.0);
}

#[test]
fn test_presets() {
    let mut config = LocalConfig::default();
    config.apply_preset(PerformancePreset::Potato);
    assert_eq!(config.render_distance, 2);
    assert_eq!(config.lod_render_distance, 2);
    assert_eq!(config.simulation_distance, 2);
    assert!(!config.fog_enabled);
    assert!(config.full_grass);
    assert!(!config.transmission_enabled);
    assert!(!config.shadows_enabled);
    assert!(!config.god_rays_enabled);
    assert_eq!(config.render_scale, 0.50);

    config.apply_preset(PerformancePreset::Low);
    assert_eq!(config.render_distance, 4);
    assert_eq!(config.lod_render_distance, 4);
    assert_eq!(config.simulation_distance, 4);
    assert!(!config.fog_enabled);
    assert!(config.full_grass);
    assert_eq!(config.render_scale, 0.75);

    config.apply_preset(PerformancePreset::Medium);
    assert_eq!(config.render_distance, 8);
    assert_eq!(config.lod_render_distance, 8);
    assert_eq!(config.simulation_distance, 8);
    assert!(config.fog_enabled);
    assert!(!config.full_grass);
    assert_eq!(config.render_scale, 1.00);

    config.apply_preset(PerformancePreset::High);
    assert_eq!(config.render_distance, 8);
    assert_eq!(config.lod_render_distance, 16);
    assert_eq!(config.simulation_distance, 8);
    assert!(config.fog_enabled);
    assert!(!config.full_grass);
    assert_eq!(config.render_scale, 1.00);

    config.apply_preset(PerformancePreset::Ultra);
    assert_eq!(config.render_distance, 16);
    assert_eq!(config.lod_render_distance, 32);
    assert_eq!(config.simulation_distance, 16);
    assert!(config.fog_enabled);
    assert!(!config.full_grass);
    assert!(!config.vsync_enabled);
    assert!(config.dynamic_fps_enabled);
    assert!(config.transmission_enabled);
    assert!(config.shadows_enabled);
    assert!(config.god_rays_enabled);
    assert_eq!(config.render_scale, 1.00);
}

#[test]
fn test_json_roundtrip() {
    let mut config = LocalConfig::default();
    config.apply_preset(PerformancePreset::Potato);
    let json = serde_json::to_string(&config).expect("Serialize failed");
    let parsed: LocalConfig = serde_json::from_str(&json).expect("Deserialize failed");
    assert_eq!(parsed.render_distance, 2);
    assert_eq!(parsed.preset, Some(PerformancePreset::Potato));
    assert!(!parsed.transmission_enabled);
    assert!(!parsed.shadows_enabled);
    assert!(!parsed.god_rays_enabled);
    assert_eq!(parsed.render_scale, 0.50);
}

#[test]
fn test_render_scale_steps() {
    use voxel_game_engine::core::config::{format_render_scale, next_render_scale, prev_render_scale};

    assert_eq!(next_render_scale(0.50), 0.66);
    assert_eq!(next_render_scale(0.66), 0.75);
    assert_eq!(next_render_scale(0.75), 0.85);
    assert_eq!(next_render_scale(0.85), 1.00);
    assert_eq!(next_render_scale(1.00), 1.00);

    assert_eq!(prev_render_scale(1.00), 0.85);
    assert_eq!(prev_render_scale(0.85), 0.75);
    assert_eq!(prev_render_scale(0.75), 0.66);
    assert_eq!(prev_render_scale(0.66), 0.50);
    assert_eq!(prev_render_scale(0.50), 0.50);

    assert_eq!(format_render_scale(1.00), "100% (Native)");
    assert_eq!(format_render_scale(0.75), "75% (Quality)");
    assert_eq!(format_render_scale(0.50), "50% (Performance)");
}
