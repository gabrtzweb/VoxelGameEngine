use bevy::{camera::Exposure, prelude::*};

use super::time::EnvironmentState;
use crate::{
    player::PlayerCamera,
    world::{CHUNK_SIZE, VOXEL_SIZE, streaming::ChunkStreamingSettings},
};

pub const DAY_SUN_ILLUMINANCE: f32 = 16_500.0;
pub const DAY_AMBIENT_BRIGHTNESS: f32 = 1_420.0;
pub const DAY_EXPOSURE_EV100: f32 = 11.76;

pub const NIGHT_MOON_ILLUMINANCE: f32 = 1_100.0;
pub const NIGHT_AMBIENT_BRIGHTNESS: f32 = 160.0;
pub const NIGHT_EXPOSURE_EV100: f32 = 9.1;

/// Subterranean ambient lighting constants:
/// Independent of whether the sun is up or down in the sky.
/// Provides rich, visible cavern ambiance (~1,450 lx) with an atmospheric
/// cool mineral slate tint that preserves block and ore character.
pub const SUBTERRANEAN_AMBIENT_BRIGHTNESS: f32 = 1_450.0;
pub const SUBTERRANEAN_AMBIENT_COLOR: Color = Color::srgb(0.68, 0.74, 0.86);
pub const SUBTERRANEAN_CLEAR_COLOR: Color = Color::srgb(0.14, 0.16, 0.22);
pub const SUBTERRANEAN_FOG_COLOR: Color = Color::srgb(0.18, 0.22, 0.28);

pub const FOG_START_FACTOR: f32 = 0.72;
pub const FOG_END_FACTOR: f32 = 0.96;

#[derive(Debug, Clone, Copy)]
pub struct BiomeFogBlend {
    pub color_tint: LinearRgba,
    pub tint_strength: f32,
    pub density_mult: f32,
    pub initialized: bool,
}

impl Default for BiomeFogBlend {
    fn default() -> Self {
        Self {
            color_tint: LinearRgba::new(0.52, 0.74, 0.96, 1.0),
            tint_strength: 0.0,
            density_mult: 1.0,
            initialized: false,
        }
    }
}

/// Returns `(target_fog_tint, tint_strength, density_multiplier)` for atmospheric biome fog.
pub fn biome_atmosphere_fog_profile(biome: crate::generation::BiomeType) -> (LinearRgba, f32, f32) {
    use crate::generation::BiomeType as B;
    match biome {
        // Misty swamps, marshes & humid wetlands
        B::WillowMarsh | B::MangroveSwamp | B::OrchidSwamp | B::SwampyShore | B::Mudflats => {
            (LinearRgba::new(0.44, 0.64, 0.56, 1.0), 0.45, 1.55)
        }
        B::MistyWoods | B::MistyCrags | B::BambooGrove => {
            (LinearRgba::new(0.54, 0.72, 0.80, 1.0), 0.40, 1.45)
        }
        B::DenseRainforest | B::JungleWetlands | B::VolcanicJungle => {
            (LinearRgba::new(0.46, 0.70, 0.64, 1.0), 0.34, 1.32)
        }
        // Arid deserts, dunes, canyons & savannas
        B::DuneSea | B::AncientSands | B::DesertOasis | B::DesertSpires => {
            (LinearRgba::new(0.90, 0.76, 0.54, 1.0), 0.38, 1.22)
        }
        B::RedrockCanyon
        | B::PaintedMesas
        | B::SavannaBadlands
        | B::FracturedSavanna
        | B::AcaciaPlains => (LinearRgba::new(0.86, 0.68, 0.48, 1.0), 0.34, 1.15),
        // Frozen, glacial & polar biomes
        B::SnowyPlains
        | B::ColdShrubland
        | B::PermafrostSteppe
        | B::GlacialChasm
        | B::IceSheet
        | B::FrozenCliffs
        | B::WintryForest
        | B::FrostPinewoods
        | B::IceboundRidge
        | B::BleakTaiga
        | B::FrozenIsland
        | B::TundraIsland => (LinearRgba::new(0.72, 0.86, 1.00, 1.0), 0.38, 1.20),
        // Volcanic, ash & dark fantasy lands
        B::Caldera
        | B::VolcanicPeaks
        | B::AshenWastes
        | B::ScorchedFlats
        | B::ObsidianMire => (LinearRgba::new(0.46, 0.35, 0.30, 1.0), 0.52, 1.60),
        B::AcidPools | B::SulfurVents => (LinearRgba::new(0.58, 0.64, 0.34, 1.0), 0.46, 1.50),
        B::WitheredWoods => (LinearRgba::new(0.36, 0.34, 0.46, 1.0), 0.44, 1.40),
        // Warm floral valleys, amethyst & sakura
        B::SakuraGrove | B::LavenderValley | B::FlowerMeadow | B::AmethystCanyon => {
            (LinearRgba::new(0.76, 0.74, 0.92, 1.0), 0.22, 0.95)
        }
        // Coastal & open oceans
        B::WarmOcean
        | B::ColdOcean
        | B::OceanTrench
        | B::CoralReefs
        | B::SeagrassMeadows
        | B::KelpForests
        | B::TropicalIsland
        | B::CoralIsland => (LinearRgba::new(0.46, 0.74, 0.96, 1.0), 0.22, 0.88),
        // Default crisp aerial perspective
        _ => (LinearRgba::new(0.54, 0.76, 0.96, 1.0), 0.08, 0.95),
    }
}

type CameraAtmosphereQuery<'w, 's> = Single<
    'w,
    's,
    (
        &'static mut DistanceFog,
        &'static mut Exposure,
        &'static GlobalTransform,
    ),
    (With<Camera3d>, With<PlayerCamera>),
>;

pub fn update_atmosphere(
    time: Res<Time>,
    state: Res<EnvironmentState>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    camera: CameraAtmosphereQuery,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    selected_subrace: Option<Res<crate::player::SelectedSubrace>>,
    weather: Option<Res<super::weather::WeatherManager>>,
    lightning: Option<Res<super::weather::LightningState>>,
    (game_settings, sky_palette): (
        Option<Res<crate::menu::GameSettings>>,
        Option<Res<super::sky::SkyPaletteState>>,
    ),
    mut biome_fog: Local<BiomeFogBlend>,
) {
    let t = state.time_of_day;
    let (mut fog, mut exposure, camera_transform) = camera.into_inner();

    let camera_fluid = env_status.as_ref().and_then(|s| s.camera_fluid);
    let is_underwater = camera_fluid.is_some_and(crate::world::Voxel::is_water);
    let underground_factor = env_status.as_ref().map_or(0.0, |s| s.underground_factor);
    let cam_y = camera_transform.translation().y;
    let deep_subterranean_factor = ((-cam_y) / 48.0).clamp(0.0, 1.0);

    if let Some(fluid) = camera_fluid && !is_underwater {
        // Bespoke submersion visuals for non-water liquids (Lava, Magma, Molten, Acid, Tar, Blood, Sludge, Ooze)
        let (fog_rgb, amb_rgb, amb_bright) = match fluid {
            crate::world::Voxel::LiquidLava
            | crate::world::Voxel::LiquidCoreMagma
            | crate::world::Voxel::LiquidMolten => {
                ((0.88, 0.24, 0.02), (1.0, 0.45, 0.12), 1_400.0)
            }
            crate::world::Voxel::LiquidAcid => ((0.18, 0.68, 0.08), (0.35, 0.95, 0.22), 720.0),
            crate::world::Voxel::LiquidTar => ((0.03, 0.008, 0.06), (0.14, 0.06, 0.22), 65.0),
            crate::world::Voxel::LiquidBlood => ((0.36, 0.02, 0.03), (0.55, 0.08, 0.08), 160.0),
            crate::world::Voxel::LiquidSludge => ((0.18, 0.12, 0.07), (0.30, 0.22, 0.14), 130.0),
            crate::world::Voxel::LiquidOoze => ((0.20, 0.25, 0.05), (0.36, 0.42, 0.10), 175.0),
            _ => ((0.04, 0.20, 0.35), (0.15, 0.40, 0.60), 220.0),
        };
        let fluid_fog_color = Color::srgb(fog_rgb.0, fog_rgb.1, fog_rgb.2);
        clear_color.0 = Color::srgb(fog_rgb.0 * 0.65, fog_rgb.1 * 0.65, fog_rgb.2 * 0.65);
        ambient.color = Color::srgb(amb_rgb.0, amb_rgb.1, amb_rgb.2);
        ambient.brightness = amb_bright;
        fog.color = fluid_fog_color;
        fog.directional_light_color = fluid_fog_color;
        fog.directional_light_exponent = 2.0;
        exposure.ev100 = sample_exposure(t);
    } else if is_underwater {
        // Time-of-day & strong depth-aware underwater absorption fog (shallow turquoise -> dark abyssal trench)
        let sun_dir = crate::environment::celestial::calculate_sun_direction(t);
        let day_factor = ((sun_dir.y + 0.08) / 0.40).clamp(0.0, 1.0);
        // Sea level is ~Y=12..16; depth_factor reaches 1.0 around Y=-32 (deep ocean trench)
        let depth_factor = ((12.0 - cam_y) / 42.0).clamp(0.0, 1.0);
        let abyss_curve = depth_factor * depth_factor;

        let shallow_day = LinearRgba::new(0.045, 0.28, 0.46, 1.0);
        let deep_day = LinearRgba::new(0.004, 0.022, 0.058, 1.0);
        let night_water = LinearRgba::new(0.003, 0.012, 0.032, 1.0);
        let day_blend = LinearRgba::new(
            shallow_day.red + (deep_day.red - shallow_day.red) * depth_factor,
            shallow_day.green + (deep_day.green - shallow_day.green) * depth_factor,
            shallow_day.blue + (deep_day.blue - shallow_day.blue) * depth_factor,
            1.0,
        );
        let uw_fog_lin = LinearRgba::new(
            night_water.red + (day_blend.red - night_water.red) * day_factor,
            night_water.green + (day_blend.green - night_water.green) * day_factor,
            night_water.blue + (day_blend.blue - night_water.blue) * day_factor,
            1.0,
        );
        let water_fog_color = Color::LinearRgba(uw_fog_lin);

        clear_color.0 = Color::LinearRgba(LinearRgba::new(
            uw_fog_lin.red * 0.55,
            uw_fog_lin.green * 0.55,
            uw_fog_lin.blue * 0.65,
            1.0,
        ));
        let light_scale = (1.0 - depth_factor * 0.85) * (0.15 + 0.85 * day_factor);
        ambient.color = Color::srgb(
            0.04 + 0.16 * light_scale,
            0.12 + 0.36 * light_scale,
            0.22 + 0.42 * light_scale,
        );
        // Deep trenches drop ambient light dramatically so abyssal dives feel genuinely dark
        let shallow_ambient = sample_ambient_brightness(t) * 0.78;
        ambient.brightness = (shallow_ambient * (1.0 - abyss_curve * 0.90)).max(38.0);

        fog.color = water_fog_color;
        fog.directional_light_color = Color::srgb(
            0.02 + 0.20 * light_scale,
            0.10 + 0.48 * light_scale,
            0.18 + 0.56 * light_scale,
        );
        fog.directional_light_exponent = 6.0;
        exposure.ev100 = sample_exposure(t) + abyss_curve * 0.45;
    } else {
        let mut computed_sky = sample_sky_color(t);
        let mut computed_ambient = sample_ambient_color(t);
        let mut computed_fog = sample_fog_color(t);
        let mut computed_brightness = sample_ambient_brightness(t);
        let mut computed_exposure = sample_exposure(t);

        // Apply weather modulations if active
        if let Some(w) = weather {
            let rain = w.rain_intensity;
            let thunder = w.thunder_intensity;
            let snow = w.snow_intensity;
            let dust = w.dust_intensity;

            // 1. Modulate ambient light and exposure
            let dim_factor = (1.0 - rain * 0.42 - thunder * 0.38 - dust * 0.30).clamp(0.20, 1.0);
            computed_brightness *= dim_factor;
            computed_exposure -= rain * 0.65 + thunder * 0.85 + dust * 0.50;

            // 2. Weather sky and fog color targets
            let storm_sky = LinearRgba::new(0.24, 0.28, 0.35, 1.0);
            let snow_sky = LinearRgba::new(0.60, 0.64, 0.70, 1.0);
            let dust_sky = LinearRgba::new(0.68, 0.52, 0.32, 1.0);

            let storm_weight = (rain * 0.75 + thunder * 0.25).clamp(0.0, 1.0);
            let snow_weight = snow.clamp(0.0, 1.0);
            let dust_weight = dust.clamp(0.0, 1.0);

            let lerp_rgba = |a: LinearRgba, b: LinearRgba, factor: f32| -> LinearRgba {
                LinearRgba::new(
                    a.red + (b.red - a.red) * factor,
                    a.green + (b.green - a.green) * factor,
                    a.blue + (b.blue - a.blue) * factor,
                    a.alpha + (b.alpha - a.alpha) * factor,
                )
            };

            let sky_lin = computed_sky.to_linear();
            let mut final_sky = sky_lin;
            if storm_weight > 0.0 {
                final_sky = lerp_rgba(final_sky, storm_sky, storm_weight * 0.85);
            }
            if snow_weight > 0.0 {
                final_sky = lerp_rgba(final_sky, snow_sky, snow_weight * 0.75);
            }
            if dust_weight > 0.0 {
                final_sky = lerp_rgba(final_sky, dust_sky, dust_weight * 0.90);
            }
            computed_sky = Color::LinearRgba(final_sky);

            let fog_lin = computed_fog.to_linear();
            let mut final_fog = fog_lin;
            if storm_weight > 0.0 {
                final_fog = lerp_rgba(final_fog, storm_sky, storm_weight * 0.80);
            }
            if snow_weight > 0.0 {
                final_fog = lerp_rgba(final_fog, snow_sky, snow_weight * 0.70);
            }
            if dust_weight > 0.0 {
                final_fog = lerp_rgba(final_fog, dust_sky, dust_weight * 0.85);
            }
            computed_fog = Color::LinearRgba(final_fog);

            // Slightly shift ambient toward sky hue
            computed_ambient =
                Color::LinearRgba(lerp_rgba(computed_ambient.to_linear(), final_sky, 0.20));
        }

        // Full-scene instantaneous flash during lightning strikes
        if let Some(ref l) = lightning
            && l.flash_intensity > 0.01 {
                let flash_col = Color::srgb(0.92, 0.95, 1.0);
                computed_sky = flash_col;
                computed_ambient = flash_col;
                computed_brightness += l.flash_intensity * 8000.0;
                computed_exposure += l.flash_intensity * 3.0;
            }

        // Subterranean Lighting & Atmosphere:
        // Decouple caves from surface sun/moon cycle. Deep underground, caves maintain
        // a soft, luminous, neutral subterranean ambient level (~1,450 lx)
        // so stone walls, ores, and tunnels are clearly visible with vibrant textures.
        if underground_factor > 0.001 {
            let sub_ambient_target = lerp_f32(
                SUBTERRANEAN_AMBIENT_BRIGHTNESS,
                1_400.0,
                deep_subterranean_factor,
            );
            computed_brightness =
                lerp_f32(computed_brightness, sub_ambient_target, underground_factor);

            let lerp_rgba = |a: LinearRgba, b: LinearRgba, factor: f32| -> LinearRgba {
                LinearRgba::new(
                    a.red + (b.red - a.red) * factor,
                    a.green + (b.green - a.green) * factor,
                    a.blue + (b.blue - a.blue) * factor,
                    a.alpha + (b.alpha - a.alpha) * factor,
                )
            };

            computed_ambient = Color::LinearRgba(lerp_rgba(
                computed_ambient.to_linear(),
                SUBTERRANEAN_AMBIENT_COLOR.to_linear(),
                underground_factor,
            ));

            computed_sky = Color::LinearRgba(lerp_rgba(
                computed_sky.to_linear(),
                SUBTERRANEAN_CLEAR_COLOR.to_linear(),
                underground_factor,
            ));

            computed_fog = Color::LinearRgba(lerp_rgba(
                computed_fog.to_linear(),
                SUBTERRANEAN_FOG_COLOR.to_linear(),
                underground_factor,
            ));

            computed_exposure = lerp_f32(computed_exposure, 11.0, underground_factor);
        }

        // Racial Cave Vision & Night Clarity (Dark Elf +85%, Deep Dwarf +65%, Mountain Dwarf +45%, etc.)
        let cave_vision = selected_subrace
            .as_ref()
            .map_or(0.0, |s| s.0.attributes().cave_vision_bonus);
        if cave_vision > 0.01 {
            // Night darkness factor (0.0 at noon, 1.0 at midnight)
            let night_factor = ((1.0 - (computed_brightness / 900.0)).clamp(0.0, 1.0)) * 0.45;
            let vision_strength = underground_factor.max(night_factor) * cave_vision;
            computed_brightness *= 1.0 + vision_strength * 1.15;
            computed_exposure -= vision_strength * 0.70;
        }

        // Match horizon fog cleanly to the Sky Dome middle horizon color, then tint smoothly by biome when Atmospheric Fog is active
        let atm_fog_enabled = game_settings
            .as_ref()
            .is_none_or(|gs| gs.atmospheric_fog_enabled);
        let mut fog_lin = if let Some(ref sp) = sky_palette {
            let sky_horizon = lerp_linear_rgba(
                sp.middle_color,
                sp.sunset_color,
                (sp.sunset_strength * 0.42 * (1.0 - sp.rain_factor * 0.7)).clamp(0.0, 1.0),
            );
            lerp_linear_rgba(computed_fog.to_linear(), sky_horizon, 0.72)
        } else {
            computed_fog.to_linear()
        };

        if atm_fog_enabled && underground_factor < 0.85 {
            let cur_biome = env_status
                .as_ref()
                .map_or(crate::generation::BiomeType::Grasslands, |s| s.current_biome);
            let (target_tint, target_strength, target_density) =
                biome_atmosphere_fog_profile(cur_biome);
            let blend_t = if !biome_fog.initialized {
                biome_fog.initialized = true;
                1.0
            } else {
                // Slow, silky glide across biome borders (~3-4s) so color & mist never snap
                1.0 - (-0.55 * time.delta_secs()).exp()
            };
            biome_fog.color_tint = lerp_linear_rgba(biome_fog.color_tint, target_tint, blend_t);
            biome_fog.tint_strength =
                lerp_f32(biome_fog.tint_strength, target_strength, blend_t);
            biome_fog.density_mult = lerp_f32(biome_fog.density_mult, target_density, blend_t);

            let daylight_weight = sky_palette.as_ref().map_or(1.0, |sp| 0.25 + 0.75 * sp.sun_factor);
            let effective_tint =
                biome_fog.tint_strength * daylight_weight * (1.0 - underground_factor);
            fog_lin = lerp_linear_rgba(fog_lin, biome_fog.color_tint, effective_tint.clamp(0.0, 0.65));
        }
        computed_fog = Color::LinearRgba(fog_lin);

        // Subtle 3-zone climate ambient tint (Cold -> crisp glacial blue, Warm -> gentle golden warmth)
        if let Some(ref sp) = sky_palette {
            let ct = sp.climate_temp;
            if ct < -0.01 {
                let cold_amb = LinearRgba::new(0.56, 0.82, 1.28, 1.0);
                computed_ambient = Color::LinearRgba(lerp_linear_rgba(
                    computed_ambient.to_linear(),
                    cold_amb,
                    (-ct) * 0.42 * sp.sun_factor,
                ));
            } else if ct > 0.01 {
                let warm_amb = LinearRgba::new(0.96, 0.84, 0.72, 1.0);
                computed_ambient = Color::LinearRgba(lerp_linear_rgba(
                    computed_ambient.to_linear(),
                    warm_amb,
                    ct * 0.18 * sp.sun_factor,
                ));
            }
        }

        clear_color.0 = computed_sky;
        ambient.color = computed_ambient;
        ambient.brightness = computed_brightness;

        fog.color = computed_fog;
        // Directional light forward-scattering in distance fog must be completely faded out underground,
        // preventing artificial sun glow spots or disc blurs from appearing on cave ceilings.
        let fog_sun_col = sample_fog_light_color(t);
        fog.directional_light_color = Color::LinearRgba(lerp_linear_rgba(
            fog_sun_col.to_linear(),
            LinearRgba::NONE,
            underground_factor,
        ));
        fog.directional_light_exponent = sample_fog_light_exponent(t);
        exposure.ev100 = computed_exposure;
    }
}

pub fn sync_fog_distance(
    time: Res<Time>,
    settings: Res<ChunkStreamingSettings>,
    game_settings: Option<Res<crate::menu::GameSettings>>,
    camera: Single<(&mut DistanceFog, &GlobalTransform), (With<Camera3d>, With<PlayerCamera>)>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    weather: Option<Res<super::weather::WeatherManager>>,
    mut smoothed_density_mult: Local<f32>,
) {
    let (mut fog, cam_tf) = camera.into_inner();

    let atm_fog_enabled = game_settings
        .as_ref()
        .is_none_or(|gs| gs.atmospheric_fog_enabled);

    let camera_fluid = env_status.as_ref().and_then(|s| s.camera_fluid);
    let is_underwater = camera_fluid.is_some_and(crate::world::Voxel::is_water);
    let underground_factor = env_status.as_ref().map_or(0.0, |s| s.underground_factor);

    if let Some(fluid) = camera_fluid && !is_underwater {
        let (start, end) = match fluid {
            crate::world::Voxel::LiquidLava
            | crate::world::Voxel::LiquidCoreMagma
            | crate::world::Voxel::LiquidMolten => (0.4, 3.2),
            crate::world::Voxel::LiquidTar => (0.25, 2.2),
            crate::world::Voxel::LiquidAcid => (0.8, 7.5),
            crate::world::Voxel::LiquidBlood => (0.6, 5.5),
            crate::world::Voxel::LiquidSludge => (0.6, 4.8),
            crate::world::Voxel::LiquidOoze => (0.6, 5.0),
            _ => (1.0, 12.0),
        };
        fog.falloff = FogFalloff::Linear { start, end };
        return;
    }

    if is_underwater {
        let cam_y = cam_tf.translation().y;
        let depth_factor = ((12.0 - cam_y) / 42.0).clamp(0.0, 1.0);
        // Clear 40m visibility in sunlit shallow coral reefs -> moody 22m abyssal visibility in deep trenches
        let uw_end = 40.0 - depth_factor * 18.0;
        let uw_start = 3.0 - depth_factor * 0.5;
        fog.falloff = FogFalloff::Linear {
            start: uw_start,
            end: uw_end,
        };
        return;
    }

    // Chunk Border Fog is handled independently in the voxel/water shaders (outer horizon rim only),
    // so Bevy's DistanceFog is dedicated purely to Atmospheric Fog when above water.
    if !atm_fog_enabled {
        fog.falloff = FogFalloff::Linear {
            start: 99999.0,
            end: 100000.0,
        };
        return;
    }

    let weather_mult = weather.as_ref().map_or(1.0, |w| {
        let intensity = (w.rain_intensity * 0.35
            + w.thunder_intensity * 0.25
            + w.snow_intensity * 0.45
            + w.dust_intensity * 0.55)
            .min(0.68);
        1.0 - intensity
    });

    let chunk_world_size = CHUNK_SIZE as f32 * VOXEL_SIZE;
    let total_chunks = (settings.render_distance + settings.lod_render_distance.max(0)).max(1);
    let render_radius = total_chunks as f32 * chunk_world_size;

    // Smooth per-biome atmospheric fog density multiplier (~3-4s glide across biome borders)
    let target_biome_density = env_status.as_ref().map_or(0.95, |s| {
        biome_atmosphere_fog_profile(s.current_biome).2
    });
    if *smoothed_density_mult <= 0.001 {
        *smoothed_density_mult = target_biome_density;
    } else {
        let k = 1.0 - (-0.55 * time.delta_secs()).exp();
        *smoothed_density_mult = lerp_f32(*smoothed_density_mult, target_biome_density, k);
    }

    // Altitude attenuation: valleys collect soft aerial mist while mountain peaks stay crisp
    let cam_y = cam_tf.translation().y;
    let altitude_mult = 1.0 - ((cam_y - 48.0) / 110.0).clamp(0.0, 0.52);

    let weather_boost = 1.0 + (1.0 - weather_mult) * 1.4;
    let atm_density = (0.00225 / (render_radius / 384.0).clamp(0.65, 2.2))
        * *smoothed_density_mult
        * altitude_mult
        * weather_boost;

    let cave_density = atm_density * 1.25;
    fog.falloff = FogFalloff::Exponential {
        density: lerp_f32(atm_density, cave_density, underground_factor),
    };
}

/// Classifies biomes into 3 atmospheric temperature zones:
/// `-1.0` = Cold (polar/glacial/tundra/alpine), `0.0` = Temperate, `+1.0` = Warm (desert/savanna/volcanic).
pub fn biome_climate_zone(biome: crate::generation::BiomeType) -> f32 {
    use crate::generation::BiomeType as B;
    match biome {
        B::SnowyPlains
        | B::ColdShrubland
        | B::PermafrostSteppe
        | B::GlacialChasm
        | B::IceSheet
        | B::FrozenCliffs
        | B::WintryForest
        | B::FrostPinewoods
        | B::IceboundRidge
        | B::BleakTaiga
        | B::FrozenIsland
        | B::TundraIsland
        | B::ColdOcean
        | B::AlpinePeaks
        | B::JaggedPeaks => -1.0,
        B::DuneSea
        | B::AncientSands
        | B::DesertOasis
        | B::DesertSpires
        | B::RedrockCanyon
        | B::PaintedMesas
        | B::SavannaBadlands
        | B::FracturedSavanna
        | B::AcaciaPlains
        | B::Caldera
        | B::VolcanicPeaks
        | B::AshenWastes
        | B::ScorchedFlats
        | B::SulfurVents => 1.0,
        _ => 0.0,
    }
}

pub fn sample_4stop<T: Copy>(
    time_of_day: f32,
    morning: T,
    noon: T,
    evening: T,
    night: T,
    lerp_fn: impl Fn(T, T, f32) -> T,
) -> T {
    let t = time_of_day.rem_euclid(1.0);
    let smooth = |x: f32| {
        let c = x.clamp(0.0, 1.0);
        c * c * (3.0 - 2.0 * c)
    };
    if t < 0.25 {
        // 06:00 -> 12:00 (Morning to Noon)
        lerp_fn(morning, noon, smooth(t / 0.25))
    } else if t < 0.50 {
        // 12:00 -> 18:00 (Noon to Evening, staying bright midday until ~15:00)
        let u = ((t - 0.34) / 0.16).clamp(0.0, 1.0);
        lerp_fn(noon, evening, smooth(u))
    } else if t < 0.75 {
        // 18:00 -> 00:00 (Evening to Night, reaching full dark night by ~21:00)
        let u = ((t - 0.50) / 0.13).clamp(0.0, 1.0);
        lerp_fn(evening, night, smooth(u))
    } else {
        // 00:00 -> 06:00 (Stay genuinely dark night through 00:00..04:15, then dawn rises 04:15..06:00)
        let u = ((t - 0.925) / 0.075).clamp(0.0, 1.0);
        lerp_fn(night, morning, smooth(u))
    }
}

pub fn lerp_linear_rgba(a: LinearRgba, b: LinearRgba, factor: f32) -> LinearRgba {
    LinearRgba::new(
        a.red + (b.red - a.red) * factor,
        a.green + (b.green - a.green) * factor,
        a.blue + (b.blue - a.blue) * factor,
        a.alpha + (b.alpha - a.alpha) * factor,
    )
}

#[inline(always)]
pub fn lerp_f32(a: f32, b: f32, factor: f32) -> f32 {
    crate::core::math::lerp(a, b, factor)
}

pub fn sample_sky_color(time_of_day: f32) -> Color {
    let morning = LinearRgba::new(0.76, 0.46, 0.42, 1.0); // Dawn warm rose-gold
    let noon = LinearRgba::new(0.24, 0.56, 0.96, 1.0); // Vibrant midday azure
    let evening = LinearRgba::new(0.82, 0.34, 0.18, 1.0); // Sunset rich amber-crimson
    let night = LinearRgba::new(0.016, 0.032, 0.082, 1.0); // Deep midnight indigo

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
    // Cool sky-tinted shadows contrasting against warm direct sunlight
    let morning = LinearRgba::new(0.76, 0.72, 0.88, 1.0);
    let noon = LinearRgba::new(0.58, 0.78, 1.18, 1.0);
    let evening = LinearRgba::new(0.82, 0.62, 0.68, 1.0);
    let night = LinearRgba::new(0.26, 0.36, 0.62, 1.0);

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
        1_150.0,
        DAY_AMBIENT_BRIGHTNESS,
        1_100.0,
        NIGHT_AMBIENT_BRIGHTNESS,
        lerp_f32,
    )
}

pub fn sample_fog_color(time_of_day: f32) -> Color {
    let morning = LinearRgba::new(0.84, 0.58, 0.50, 1.0);
    let noon = LinearRgba::new(0.52, 0.74, 0.96, 1.0);
    let evening = LinearRgba::new(0.88, 0.44, 0.28, 1.0);
    let night = LinearRgba::new(0.036, 0.060, 0.125, 1.0);

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
    let morning = LinearRgba::new(1.0, 0.78, 0.48, 0.28);
    let noon = LinearRgba::new(1.0, 0.90, 0.70, 0.18);
    let evening = LinearRgba::new(1.0, 0.52, 0.26, 0.36);
    let night = LinearRgba::new(0.38, 0.50, 0.82, 0.08);

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
    sample_4stop(time_of_day, 12.0, 14.0, 10.0, 14.0, lerp_f32)
}

pub fn sample_exposure(time_of_day: f32) -> f32 {
    sample_4stop(
        time_of_day,
        10.9,
        DAY_EXPOSURE_EV100,
        10.7,
        NIGHT_EXPOSURE_EV100,
        lerp_f32,
    )
}
