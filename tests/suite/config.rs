use voxel_game_engine::core::{
    AntiAliasingMode, CloudQuality, LocalConfig, PerformancePreset, ShadowQuality, TargetFpsLimit,
    UpscalingFilter,
};

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
    assert_eq!(config.shadow_quality, ShadowQuality::Medium);
    assert_eq!(config.cloud_quality, CloudQuality::Fancy3D);
    assert_eq!(config.anti_aliasing, AntiAliasingMode::Fxaa);
    assert!(config.dynamic_fov_enabled);
    assert!(config.god_rays_enabled);
    assert_eq!(config.render_scale, 1.0);
    assert_eq!(config.target_fps, TargetFpsLimit::Fps60);
    assert_eq!(config.mesh_upload_budget, 12);
    assert_eq!(config.upscaling_filter, UpscalingFilter::Bilinear);
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
    assert_eq!(config.shadow_quality, ShadowQuality::Off);
    assert_eq!(config.cloud_quality, CloudQuality::Off);
    assert_eq!(config.anti_aliasing, AntiAliasingMode::Off);
    assert!(!config.god_rays_enabled);
    assert_eq!(config.render_scale, 0.50);
    assert_eq!(config.target_fps, TargetFpsLimit::Fps30);
    assert_eq!(config.mesh_upload_budget, 3);
    assert_eq!(config.upscaling_filter, UpscalingFilter::Bilinear);

    config.apply_preset(PerformancePreset::Low);
    assert_eq!(config.render_distance, 4);
    assert_eq!(config.lod_render_distance, 4);
    assert_eq!(config.simulation_distance, 4);
    assert!(!config.fog_enabled);
    assert!(config.full_grass);
    assert_eq!(config.shadow_quality, ShadowQuality::Off);
    assert!(!config.shadows_enabled);
    assert_eq!(config.cloud_quality, CloudQuality::Fast2D);
    assert_eq!(config.anti_aliasing, AntiAliasingMode::Off);
    assert_eq!(config.render_scale, 0.75);
    assert_eq!(config.target_fps, TargetFpsLimit::Fps60);
    assert_eq!(config.mesh_upload_budget, 6);
    assert_eq!(config.upscaling_filter, UpscalingFilter::Bilinear);

    config.apply_preset(PerformancePreset::Medium);
    assert_eq!(config.render_distance, 8);
    assert_eq!(config.lod_render_distance, 8);
    assert_eq!(config.simulation_distance, 8);
    assert!(config.fog_enabled);
    assert!(!config.full_grass);
    assert_eq!(config.shadow_quality, ShadowQuality::Medium);
    assert!(config.shadows_enabled);
    assert_eq!(config.cloud_quality, CloudQuality::Fancy3D);
    assert_eq!(config.anti_aliasing, AntiAliasingMode::Fxaa);
    assert_eq!(config.render_scale, 1.00);
    assert_eq!(config.target_fps, TargetFpsLimit::Fps60);
    assert_eq!(config.mesh_upload_budget, 12);
    assert_eq!(config.upscaling_filter, UpscalingFilter::Fsr1);

    config.apply_preset(PerformancePreset::High);
    assert_eq!(config.render_distance, 8);
    assert_eq!(config.lod_render_distance, 16);
    assert_eq!(config.simulation_distance, 8);
    assert!(config.fog_enabled);
    assert!(!config.full_grass);
    assert_eq!(config.shadow_quality, ShadowQuality::High);
    assert!(config.shadows_enabled);
    assert_eq!(config.cloud_quality, CloudQuality::Fancy3D);
    assert_eq!(config.anti_aliasing, AntiAliasingMode::Msaa2x);
    assert_eq!(config.render_scale, 1.00);
    assert_eq!(config.target_fps, TargetFpsLimit::Fps144);
    assert_eq!(config.mesh_upload_budget, 24);
    assert_eq!(config.upscaling_filter, UpscalingFilter::Fsr1);

    config.apply_preset(PerformancePreset::Ultra);
    assert_eq!(config.render_distance, 16);
    assert_eq!(config.lod_render_distance, 32);
    assert_eq!(config.simulation_distance, 16);
    assert!(config.fog_enabled);
    assert!(!config.full_grass);
    assert!(!config.vsync_enabled);
    assert!(config.dynamic_fps_enabled);
    assert!(config.transmission_enabled);
    assert_eq!(config.shadow_quality, ShadowQuality::Ultra);
    assert!(config.shadows_enabled);
    assert_eq!(config.cloud_quality, CloudQuality::Fancy3D);
    assert_eq!(config.anti_aliasing, AntiAliasingMode::Msaa4x);
    assert!(config.god_rays_enabled);
    assert_eq!(config.render_scale, 1.00);
    assert_eq!(config.target_fps, TargetFpsLimit::Unlocked);
    assert_eq!(config.mesh_upload_budget, 64);
    assert_eq!(config.upscaling_filter, UpscalingFilter::Fsr1);
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
    assert_eq!(parsed.shadow_quality, ShadowQuality::Off);
    assert_eq!(parsed.cloud_quality, CloudQuality::Off);
    assert_eq!(parsed.anti_aliasing, AntiAliasingMode::Off);
    assert!(parsed.dynamic_fov_enabled);
    assert!(!parsed.god_rays_enabled);
    assert_eq!(parsed.render_scale, 0.50);
    assert_eq!(parsed.target_fps, TargetFpsLimit::Fps30);
    assert_eq!(parsed.mesh_upload_budget, 3);
    assert_eq!(parsed.upscaling_filter, UpscalingFilter::Bilinear);
}

#[test]
fn test_graphics_enums_navigation() {
    assert_eq!(AntiAliasingMode::Off.next(), AntiAliasingMode::Fxaa);
    assert_eq!(AntiAliasingMode::Fxaa.next(), AntiAliasingMode::Msaa2x);
    assert_eq!(AntiAliasingMode::Msaa2x.next(), AntiAliasingMode::Msaa4x);
    assert_eq!(AntiAliasingMode::Msaa4x.next(), AntiAliasingMode::Off);
    assert_eq!(AntiAliasingMode::Off.prev(), AntiAliasingMode::Msaa4x);

    assert_eq!(ShadowQuality::Off.next(), ShadowQuality::Low);
    assert_eq!(ShadowQuality::Low.next(), ShadowQuality::Medium);
    assert_eq!(ShadowQuality::Medium.next(), ShadowQuality::High);
    assert_eq!(ShadowQuality::High.next(), ShadowQuality::Ultra);
    assert_eq!(ShadowQuality::Ultra.next(), ShadowQuality::Off);
    assert_eq!(ShadowQuality::Off.prev(), ShadowQuality::Ultra);

    assert_eq!(CloudQuality::Off.next(), CloudQuality::Fast2D);
    assert_eq!(CloudQuality::Fast2D.next(), CloudQuality::Fancy3D);
    assert_eq!(CloudQuality::Fancy3D.next(), CloudQuality::Off);
    assert_eq!(CloudQuality::Off.prev(), CloudQuality::Fancy3D);

    assert_eq!(TargetFpsLimit::Fps30.next(), TargetFpsLimit::Fps60);
    assert_eq!(TargetFpsLimit::Fps60.next(), TargetFpsLimit::Fps120);
    assert_eq!(TargetFpsLimit::Fps120.next(), TargetFpsLimit::Fps144);
    assert_eq!(TargetFpsLimit::Fps144.next(), TargetFpsLimit::Unlocked);
    assert_eq!(TargetFpsLimit::Unlocked.next(), TargetFpsLimit::Fps30);
    assert_eq!(TargetFpsLimit::Fps30.prev(), TargetFpsLimit::Unlocked);

    assert_eq!(TargetFpsLimit::Fps30.to_fps(), Some(30));
    assert_eq!(TargetFpsLimit::Fps60.to_fps(), Some(60));
    assert_eq!(TargetFpsLimit::Fps120.to_fps(), Some(120));
    assert_eq!(TargetFpsLimit::Fps144.to_fps(), Some(144));
    assert_eq!(TargetFpsLimit::Unlocked.to_fps(), None);

    assert_eq!(TargetFpsLimit::from_fps(30), TargetFpsLimit::Fps30);
    assert_eq!(TargetFpsLimit::from_fps(60), TargetFpsLimit::Fps60);
    assert_eq!(TargetFpsLimit::from_fps(120), TargetFpsLimit::Fps120);
    assert_eq!(TargetFpsLimit::from_fps(144), TargetFpsLimit::Fps144);
    assert_eq!(TargetFpsLimit::from_fps(999), TargetFpsLimit::Unlocked);

    assert_eq!(UpscalingFilter::Bilinear.next(), UpscalingFilter::Fsr1);
    assert_eq!(UpscalingFilter::Fsr1.next(), UpscalingFilter::Bilinear);
    assert_eq!(UpscalingFilter::Bilinear.prev(), UpscalingFilter::Fsr1);
    assert_eq!(UpscalingFilter::Fsr1.prev(), UpscalingFilter::Bilinear);
}

#[test]
fn test_render_scale_steps() {
    use voxel_game_engine::core::config::{
        format_render_scale, next_render_scale, prev_render_scale,
    };

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

#[test]
fn test_mesh_upload_budget_steps() {
    use voxel_game_engine::core::config::{next_mesh_upload_budget, prev_mesh_upload_budget};

    assert_eq!(next_mesh_upload_budget(3), 6);
    assert_eq!(next_mesh_upload_budget(6), 12);
    assert_eq!(next_mesh_upload_budget(12), 24);
    assert_eq!(next_mesh_upload_budget(24), 64);
    assert_eq!(next_mesh_upload_budget(64), 64);

    assert_eq!(prev_mesh_upload_budget(64), 24);
    assert_eq!(prev_mesh_upload_budget(24), 12);
    assert_eq!(prev_mesh_upload_budget(12), 6);
    assert_eq!(prev_mesh_upload_budget(6), 3);
    assert_eq!(prev_mesh_upload_budget(3), 3);
}

#[test]
fn test_aa_components_on_camera() {
    let mut app = bevy::app::App::new();
    let entity = app
        .world_mut()
        .spawn((
            bevy::camera::Camera3d::default(),
            bevy::render::view::Msaa::Sample4,
            bevy::anti_alias::fxaa::Fxaa::default(),
        ))
        .id();

    let msaa = app.world().get::<bevy::render::view::Msaa>(entity);
    assert_eq!(msaa, Some(&bevy::render::view::Msaa::Sample4));

    let fxaa = app.world().get::<bevy::anti_alias::fxaa::Fxaa>(entity);
    assert!(fxaa.is_some());
}
