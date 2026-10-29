use bevy::{prelude::*, render::render_resource::ShaderType};

use super::config::GrassConfig;
use crate::environment::weather::wind::WindState;

/// GPU uniform buffer matching std140 layout in `shaders/grass_blades.wesl`.
#[derive(ShaderType, Clone, Copy, Debug, Default, Reflect)]
pub struct GrassShaderUniformsGpu {
    pub wind_params: Vec4,
    pub wind_params2: Vec4,
    pub style_params: Vec4,
    pub gradient_params: Vec4,
    pub trail_params2: Vec4,
    pub entity_pushers: [Vec4; 8],
}

/// CPU-side state tracking animated phase accumulation.
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct FoliageAnimationState {
    pub wind_noise_scroll: Vec2,
    pub wind_flutter_phase: f32,
    pub plant_wave_phase: f32,
    pub dynamic_wind_time: f32,
    pub gpu_uniforms: GrassShaderUniformsGpu,
}

impl Default for FoliageAnimationState {
    fn default() -> Self {
        Self {
            wind_noise_scroll: Vec2::ZERO,
            wind_flutter_phase: 0.0,
            plant_wave_phase: 0.0,
            dynamic_wind_time: 0.0,
            gpu_uniforms: GrassShaderUniformsGpu::default(),
        }
    }
}

pub fn update_foliage_uniforms(
    time: Res<Time>,
    wind: Option<Res<WindState>>,
    config: Option<Res<GrassConfig>>,
    local_config: Option<Res<crate::core::LocalConfig>>,
    mut anim_state: ResMut<FoliageAnimationState>,
) {
    let dt = time.delta_secs();
    let default_cfg = GrassConfig::default();
    let cfg = config.as_ref().map(|c| &**c).unwrap_or(&default_cfg);

    // Wind direction and scalar speed (m/s)
    let (mut wind_dir, mut wind_speed) = if let Some(w) = wind.as_ref() {
        let dir = Vec2::new(w.current_wind.x, w.current_wind.z);
        let speed = if dir.length_squared() > 0.001 {
            dir.length()
        } else {
            1.0
        };
        (dir.normalize_or_zero(), speed)
    } else {
        (Vec2::new(1.0, 0.0), 1.0)
    };

    if cfg.dynamic_wind {
        anim_state.dynamic_wind_time += dt * 0.15;
        let drift_angle = (anim_state.dynamic_wind_time * 0.3).sin() * 0.45;
        let c = drift_angle.cos();
        let s = drift_angle.sin();
        let rotated_x = wind_dir.x * c - wind_dir.y * s;
        let rotated_z = wind_dir.x * s + wind_dir.y * c;
        wind_dir = Vec2::new(rotated_x, rotated_z).normalize_or_zero();

        let gust_mult = 0.7 + 0.3 * ((anim_state.dynamic_wind_time * 0.5).sin() * 0.5 + 0.5);
        wind_speed *= gust_mult * cfg.dynamic_wind_speed_limit;
    }

    // Scroll wind noise texture along wind direction
    let scroll_speed = 0.08 * (0.4 + 0.6 * (wind_speed / 5.0).clamp(0.0, 2.5));
    anim_state.wind_noise_scroll += wind_dir * (scroll_speed * dt);
    anim_state.wind_noise_scroll.x = anim_state.wind_noise_scroll.x.fract();
    anim_state.wind_noise_scroll.y = anim_state.wind_noise_scroll.y.fract();

    // High frequency flutter phase
    let flutter_rate = 6.0 + 8.0 * (wind_speed / 4.0).clamp(0.0, 2.0);
    anim_state.wind_flutter_phase = (anim_state.wind_flutter_phase + dt * flutter_rate) % 628.3185;

    // Plant bobbing wave phase
    let bob_rate = 2.2 + 2.5 * (wind_speed / 5.0).clamp(0.0, 2.0);
    anim_state.plant_wave_phase = (anim_state.plant_wave_phase + dt * bob_rate) % 628.3185;

    let style_flag = if cfg.grass_style == super::config::GrassStyle::Segmented {
        1.0
    } else {
        0.0
    };

    let waving_enabled = local_config.as_ref().map_or(true, |c| c.waving_foliage);
    let wind_intensity = if waving_enabled {
        (wind_speed / 4.0).clamp(0.0, 5.0)
    } else {
        0.0
    };

    let gpu = GrassShaderUniformsGpu {
        wind_params: Vec4::new(
            wind_dir.x,
            wind_dir.y,
            anim_state.wind_flutter_phase,
            anim_state.plant_wave_phase,
        ),
        wind_params2: Vec4::new(
            anim_state.wind_noise_scroll.x,
            anim_state.wind_noise_scroll.y,
            1.0, // noise scale
            cfg.blade_width,
        ),
        style_params: Vec4::new(
            style_flag,
            wind_intensity,
            cfg.blade_height * 1.45,
            cfg.grass_brightness,
        ),
        gradient_params: Vec4::new(
            cfg.blade_gradient_bottom,
            cfg.blade_gradient_top,
            cfg.blade_gradient_curve,
            cfg.height_variation,
        ),
        trail_params2: anim_state.gpu_uniforms.trail_params2,
        entity_pushers: anim_state.gpu_uniforms.entity_pushers,
    };

    anim_state.gpu_uniforms = gpu;
}
