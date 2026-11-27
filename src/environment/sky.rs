use bevy::{
    camera::visibility::NoFrustumCulling,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

use super::{
    atmosphere::{lerp_f32, lerp_linear_rgba, sample_4stop},
    celestial::calculate_sun_direction,
    time::EnvironmentState,
};
use crate::player::PlayerCamera;

pub const SKY_DOME_RADIUS: f32 = 2400.0;

#[derive(Component)]
pub struct SkyDome;

#[derive(Resource)]
pub struct SkyDomeMaterialHandle(pub Handle<SkyDomeMaterial>);

/// Shared current sky palette for Sky Dome, Fog horizon matching, and Water Fresnel reflections.
#[derive(Resource, Debug, Clone, Copy)]
pub struct SkyPaletteState {
    pub up_color: LinearRgba,
    pub middle_color: LinearRgba,
    pub down_color: LinearRgba,
    pub sunset_color: LinearRgba,
    pub sun_dir: Vec3,
    pub sun_color: LinearRgba,
    pub sun_factor: f32,
    pub sunset_strength: f32,
    pub rain_factor: f32,
    pub climate_temp: f32,
}

impl Default for SkyPaletteState {
    fn default() -> Self {
        Self {
            up_color: LinearRgba::new(0.10, 0.34, 0.88, 1.0),
            middle_color: LinearRgba::new(0.42, 0.72, 1.02, 1.0),
            down_color: LinearRgba::new(0.32, 0.54, 0.78, 1.0),
            sunset_color: LinearRgba::new(1.35, 0.62, 0.28, 1.0),
            sun_dir: Vec3::Y,
            sun_color: LinearRgba::new(1.0, 0.88, 0.68, 1.0),
            sun_factor: 1.0,
            sunset_strength: 0.0,
            rain_factor: 0.0,
            climate_temp: 0.0,
        }
    }
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct SkyDomeUniform {
    pub up_color: Vec4,
    pub middle_color: Vec4,
    pub down_color: Vec4,
    pub sunset_color: Vec4,
    pub sun_dir_and_factor: Vec4,
    pub params: Vec4,
    pub glare_color: Vec4,
    pub rainbow_params: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SkyDomeMaterial {
    #[uniform(0)]
    pub data: SkyDomeUniform,
}

impl Material for SkyDomeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sky_dome.wesl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

pub struct SkyDomePlugin;

impl Plugin for SkyDomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkyPaletteState>()
            .add_plugins(MaterialPlugin::<SkyDomeMaterial>::default())
            .add_systems(Startup, setup_sky_dome)
            .add_systems(
                Update,
                sync_sky_dome
                    .after(crate::player::PlayerSet::Movement)
                    .after(super::atmosphere::update_atmosphere),
            );
    }
}

fn setup_sky_dome(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyDomeMaterial>>,
) {
    let mesh = meshes.add(Sphere::new(SKY_DOME_RADIUS).mesh().ico(2).unwrap());
    let material = materials.add(SkyDomeMaterial {
        data: SkyDomeUniform {
            up_color: Vec4::new(0.10, 0.34, 0.88, 1.0),
            middle_color: Vec4::new(0.42, 0.72, 1.02, 1.0),
            down_color: Vec4::new(0.32, 0.54, 0.78, 1.0),
            sunset_color: Vec4::new(1.35, 0.62, 0.28, 1.0),
            sun_dir_and_factor: Vec4::new(0.0, 1.0, 0.0, 1.0),
            params: Vec4::ZERO,
            glare_color: Vec4::new(1.2, 0.85, 0.5, 0.8),
            rainbow_params: Vec4::ZERO,
        },
    });

    commands.insert_resource(SkyDomeMaterialHandle(material.clone()));

    commands.spawn((
        SkyDome,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::default(),
        Visibility::Visible,
        NotShadowCaster,
        NotShadowReceiver,
        NoFrustumCulling,
    ));
}

fn rgba_to_vec4(c: LinearRgba) -> Vec4 {
    Vec4::new(c.red, c.green, c.blue, c.alpha)
}

fn sync_sky_dome(
    time: Res<Time>,
    state: Res<EnvironmentState>,
    camera: Single<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut dome_transform: Single<&mut Transform, (With<SkyDome>, Without<Camera3d>)>,
    material_handle: Res<SkyDomeMaterialHandle>,
    mut materials: ResMut<Assets<SkyDomeMaterial>>,
    mut sky_palette: ResMut<SkyPaletteState>,
    weather: Option<Res<super::weather::WeatherManager>>,
    lightning: Option<Res<super::weather::LightningState>>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    local_config: Option<Res<crate::core::LocalConfig>>,
    mut smoothed_aurora: Local<f32>,
) {
    dome_transform.translation = camera.translation;

    // Smoothly blend climate temperature zone (-1.0 Cold, 0.0 Temperate, +1.0 Warm)
    // Gentle ~3-4s glide across biome borders so skybox transitions never snap.
    let target_climate = env_status.as_ref().map_or(0.0, |s| {
        super::atmosphere::biome_climate_zone(s.current_biome)
    });
    let climate_blend = 1.0 - (-0.55 * time.delta_secs()).exp();
    sky_palette.climate_temp = lerp_f32(sky_palette.climate_temp, target_climate, climate_blend);
    let ct = sky_palette.climate_temp;

    let t = state.time_of_day;
    let sun_dir = calculate_sun_direction(t);
    let sun_elev = sun_dir.y;
    let sun_factor = ((sun_elev + 0.14) / 0.32).clamp(0.0, 1.0);
    let noon_factor = (sun_elev / 0.55).clamp(0.0, 1.0);
    let inv_noon = 1.0 - noon_factor;
    let sunset_strength = (inv_noon * inv_noon * sun_factor).clamp(0.0, 1.0);

    // 3-zone procedural base sky palette across 4 stops (Dawn / Noon / Sunset / Night)
    let mut up_col = sample_4stop(
        t,
        LinearRgba::new(0.18, 0.22, 0.54, 1.0), // Dawn zenith
        LinearRgba::new(0.08, 0.32, 0.88, 1.0), // Midday deep vibrant azure
        LinearRgba::new(0.20, 0.16, 0.42, 1.0), // Sunset twilight indigo-violet
        LinearRgba::new(0.008, 0.018, 0.052, 1.0), // Midnight deep navy
        lerp_linear_rgba,
    );

    let mut middle_col = sample_4stop(
        t,
        LinearRgba::new(0.72, 0.48, 0.46, 1.0), // Dawn rosy horizon
        LinearRgba::new(0.44, 0.74, 1.04, 1.0), // Midday luminous sky-blue horizon
        LinearRgba::new(0.86, 0.40, 0.24, 1.0), // Sunset warm horizon
        LinearRgba::new(0.024, 0.044, 0.098, 1.0), // Midnight subtle horizon
        lerp_linear_rgba,
    );

    let mut down_col = sample_4stop(
        t,
        LinearRgba::new(0.48, 0.36, 0.38, 1.0),
        LinearRgba::new(0.32, 0.56, 0.82, 1.0),
        LinearRgba::new(0.52, 0.26, 0.18, 1.0),
        LinearRgba::new(0.015, 0.028, 0.064, 1.0),
        lerp_linear_rgba,
    );

    let mut sunset_col = sample_4stop(
        t,
        LinearRgba::new(1.38, 0.72, 0.42, 1.0), // Warm peach-gold dawn ribbon
        LinearRgba::new(1.15, 0.82, 0.55, 1.0),
        LinearRgba::new(1.52, 0.54, 0.20, 1.0), // Fiery amber-crimson sunset ribbon
        LinearRgba::new(0.08, 0.10, 0.18, 1.0),
        lerp_linear_rgba,
    );

    // Subtle, perceptible 3-zone climate sky shift (Cold -> crisp glacial frost-blue, Warm -> gentle sun-warmed horizon)
    if ct < -0.01 {
        let cold_w = (-ct) * 0.42 * sun_factor;
        up_col = lerp_linear_rgba(up_col, LinearRgba::new(0.07, 0.34, 0.96, 1.0), cold_w);
        middle_col = lerp_linear_rgba(middle_col, LinearRgba::new(0.60, 0.84, 1.12, 1.0), cold_w);
    } else if ct > 0.01 {
        let warm_w = ct * 0.34 * sun_factor;
        up_col = lerp_linear_rgba(up_col, LinearRgba::new(0.13, 0.38, 0.86, 1.0), warm_w * 0.65);
        middle_col = lerp_linear_rgba(middle_col, LinearRgba::new(0.82, 0.78, 0.72, 1.0), warm_w);
    }

    // Weather modulation
    let mut rain_factor = 0.0_f32;
    if let Some(ref w) = weather {
        let storm = (w.rain_intensity * 0.75 + w.thunder_intensity * 0.25).clamp(0.0, 1.0);
        let snow = w.snow_intensity.clamp(0.0, 1.0);
        let dust = w.dust_intensity.clamp(0.0, 1.0);
        rain_factor = storm.max(snow * 0.75).max(dust * 0.85);

        let storm_up = LinearRgba::new(0.14, 0.18, 0.26, 1.0);
        let storm_mid = LinearRgba::new(0.24, 0.28, 0.36, 1.0);
        let snow_up = LinearRgba::new(0.44, 0.52, 0.64, 1.0);
        let snow_mid = LinearRgba::new(0.64, 0.68, 0.74, 1.0);
        let dust_up = LinearRgba::new(0.52, 0.40, 0.24, 1.0);
        let dust_mid = LinearRgba::new(0.74, 0.56, 0.34, 1.0);

        if storm > 0.001 {
            up_col = lerp_linear_rgba(up_col, storm_up, storm * 0.82);
            middle_col = lerp_linear_rgba(middle_col, storm_mid, storm * 0.82);
            down_col = lerp_linear_rgba(down_col, storm_mid, storm * 0.85);
        }
        if snow > 0.001 {
            up_col = lerp_linear_rgba(up_col, snow_up, snow * 0.72);
            middle_col = lerp_linear_rgba(middle_col, snow_mid, snow * 0.72);
            down_col = lerp_linear_rgba(down_col, snow_mid, snow * 0.75);
        }
        if dust > 0.001 {
            up_col = lerp_linear_rgba(up_col, dust_up, dust * 0.88);
            middle_col = lerp_linear_rgba(middle_col, dust_mid, dust * 0.88);
            down_col = lerp_linear_rgba(down_col, dust_mid, dust * 0.90);
        }
    }

    if let Some(ref l) = lightning
        && l.flash_intensity > 0.01
    {
        let flash = LinearRgba::new(0.95, 0.98, 1.15, 1.0);
        let f = (l.flash_intensity * 0.85).clamp(0.0, 1.0);
        up_col = lerp_linear_rgba(up_col, flash, f);
        middle_col = lerp_linear_rgba(middle_col, flash, f);
        down_col = lerp_linear_rgba(down_col, flash, f);
        sunset_col = lerp_linear_rgba(sunset_col, flash, f);
    }

    // Glare color (warm golden for Sun, cool silver-blue for Moon)
    let glare_rgb = if sun_factor > 0.25 {
        let dusk_glare = LinearRgba::new(1.55, 0.72, 0.30, 1.0);
        let noon_glare = LinearRgba::new(1.35, 1.12, 0.76, 1.0);
        lerp_linear_rgba(dusk_glare, noon_glare, noon_factor)
    } else {
        LinearRgba::new(0.32, 0.42, 0.68, 1.0)
    };
    let glare_strength = if sun_factor > 0.25 {
        lerp_f32(0.85, 0.55, noon_factor) * sun_factor
    } else {
        0.35 * (1.0 - sun_factor)
    };

    let sun_warmth = (sun_elev.max(0.0) / 0.42).clamp(0.0, 1.0);
    let sun_color = lerp_linear_rgba(
        LinearRgba::new(1.0, 0.48, 0.18, 1.0),
        LinearRgba::new(1.0, 0.88, 0.66, 1.0),
        sun_warmth.powf(0.68),
    );

    let underwater = if env_status.as_ref().is_some_and(|s| s.is_camera_in_water) {
        1.0
    } else {
        0.0
    };
    let underground = env_status.as_ref().map_or(0.0, |s| s.underground_factor);

    sky_palette.up_color = up_col;
    sky_palette.middle_color = middle_col;
    sky_palette.down_color = down_col;
    sky_palette.sunset_color = sunset_col;
    sky_palette.sun_dir = sun_dir;
    sky_palette.sun_color = sun_color;
    sky_palette.sun_factor = sun_factor;
    sky_palette.sunset_strength = sunset_strength;
    sky_palette.rain_factor = rain_factor;

    // Post-rain rainbow visibility: appears opposite the sun during daytime right after rain clears
    let rainbow_strength = if let Some(ref w) = weather {
        let post_rain = (w.after_weather_timer / 45.0).clamp(0.0, 1.0);
        let sun_window =
            ((sun_elev - 0.05) / 0.14).clamp(0.0, 1.0) * (1.0 - ((sun_elev - 0.55) / 0.30).clamp(0.0, 1.0));
        let clear_after = (1.0 - w.rain_intensity * 1.4).clamp(0.0, 1.0);
        (post_rain.sqrt() * sun_window * clear_after * 0.78).clamp(0.0, 0.85)
    } else {
        0.0
    };

    // Aurora Borealis visibility: fades in/out gradually when entering Cold/Polar biomes at night
    let aurora_enabled = local_config.as_ref().is_none_or(|c| c.aurora_borealis);
    let target_aurora = if aurora_enabled {
        let night_weight = ((0.22 - sun_factor) / 0.22).clamp(0.0, 1.0);
        let cold_weight = ((-ct - 0.15) / 0.55).clamp(0.0, 1.0);
        let clear_weight = (1.0 - rain_factor * 1.4).clamp(0.0, 1.0);
        (night_weight * night_weight * cold_weight * clear_weight).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let aurora_rate = if aurora_enabled { 0.28 } else { 2.5 };
    let aurora_k = 1.0 - (-aurora_rate * time.delta_secs()).exp();
    *smoothed_aurora = lerp_f32(*smoothed_aurora, target_aurora, aurora_k);
    let aurora_strength = *smoothed_aurora;

    if let Some(mut mat) = materials.get_mut(&material_handle.0) {
        mat.data = SkyDomeUniform {
            up_color: rgba_to_vec4(up_col),
            middle_color: rgba_to_vec4(middle_col),
            down_color: rgba_to_vec4(down_col),
            sunset_color: rgba_to_vec4(sunset_col),
            sun_dir_and_factor: Vec4::new(sun_dir.x, sun_dir.y, sun_dir.z, sun_factor),
            params: Vec4::new(sunset_strength, rain_factor, underwater, underground),
            glare_color: Vec4::new(glare_rgb.red, glare_rgb.green, glare_rgb.blue, glare_strength),
            rainbow_params: Vec4::new(
                rainbow_strength,
                aurora_strength,
                time.elapsed_secs(),
                0.0,
            ),
        };
    }
}
