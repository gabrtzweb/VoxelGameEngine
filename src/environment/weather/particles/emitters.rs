use bevy::prelude::*;

use super::buffer::{
    ParticleFadeMode, ParticleRotationMode, WeatherParticleKind, WeatherParticlePool,
};
use crate::{
    environment::weather::{state::WeatherManager, wind::WindState},
    generation::{TerrainGenerator, biome::BiomeType},
    player::{Player, PlayerCamera, PlayerMotion},
    world::{Voxel, storage::VoxelWorld},
};

// UV Coordinate rectangles matching `assets/textures/environments/weather_atlas.png` (256x256 px)
pub const UV_RAIN_DROP: [f32; 4] = [0.0, 0.0, 0.125, 0.125];
pub const UV_HEAVY_RAIN: [f32; 4] = [0.125, 0.0, 0.25, 0.125];
pub const UV_SNOW_A: [f32; 4] = [0.25, 0.0, 0.375, 0.125];
pub const UV_SNOW_B: [f32; 4] = [0.375, 0.0, 0.50, 0.125];
pub const UV_SNOW_C: [f32; 4] = [0.625, 0.0, 0.75, 0.125];
pub const UV_SNOW_D: [f32; 4] = [0.75, 0.0, 0.875, 0.125];
pub const UV_RIPPLE_0: [f32; 4] = [0.0, 0.125, 0.125, 0.25];
pub const UV_RIPPLE_1: [f32; 4] = [0.125, 0.125, 0.25, 0.25];
pub const UV_RIPPLE_2: [f32; 4] = [0.25, 0.125, 0.375, 0.25];
pub const UV_RIPPLE_3: [f32; 4] = [0.375, 0.125, 0.50, 0.25];
pub const UV_SPLASH: [f32; 4] = [0.50, 0.0, 0.625, 0.125];
pub const UV_FOG: [f32; 4] = [0.0, 0.25, 0.25, 0.50];
pub const UV_DUST: [f32; 4] = [0.25, 0.25, 0.50, 0.50];
pub const UV_STREAK: [f32; 4] = [0.50, 0.125, 0.625, 0.25];

/// Fast linear congruential PRNG state for per-frame particle scattering.
#[derive(Resource)]
pub struct WeatherRng(pub u64);

impl Default for WeatherRng {
    fn default() -> Self {
        Self(14050061177)
    }
}

impl WeatherRng {
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 32) as u32
    }

    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() & 0x00FFFFFF) as f32 / 16777216.0
    }

    /// Generates triangular distribution between -range and +range centered at 0.
    #[inline]
    pub fn next_triangle(&mut self, range: f32) -> f32 {
        (self.next_f32() - self.next_f32()) * range
    }
}

/// Scans top-down in a column to find the highest solid, roof, or ground block.
#[inline]
pub fn find_column_surface_y(
    voxel_world: &VoxelWorld,
    wx: i32,
    wz: i32,
    start_y: i32,
    min_y: i32,
) -> Option<i32> {
    for y in (min_y..=start_y).rev() {
        if let Some(voxel) = voxel_world.get_voxel(IVec3::new(wx, y, wz)) {
            if voxel != Voxel::Air && !voxel.is_empty() {
                return Some(y);
            }
        }
    }
    None
}

/// Emits atmospheric sky particles (rain, snow, dust) and surface phenomena (ripples, splatters, mist, streaks).
pub fn emit_weather_particles(
    time: Res<Time>,
    weather: Option<Res<WeatherManager>>,
    wind: Option<Res<WindState>>,
    voxel_world: Res<VoxelWorld>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    player_query: Query<&PlayerMotion, With<Player>>,
    terrain_generator: Option<Res<TerrainGenerator>>,
    mut pool: ResMut<WeatherParticlePool>,
    mut rng: ResMut<WeatherRng>,
) {
    let dt = time.delta_secs();
    let current_wind = wind.as_ref().map_or(Vec3::ZERO, |w| w.current_wind);

    // 1. Advance physics of all existing particles in DOD ring buffer
    pool.tick(dt, current_wind);

    let Some(ref w) = weather else {
        return;
    };

    let Ok(cam_transform) = camera_query.single() else {
        return;
    };
    let cam_pos = cam_transform.translation;

    // Underwater camera check: if the player's head is submerged, do not spawn sky weather particles
    let cam_block = IVec3::new(
        cam_pos.x.floor() as i32,
        cam_pos.y.floor() as i32,
        cam_pos.z.floor() as i32,
    );
    if voxel_world.get_voxel(cam_block).is_some_and(|v| v.is_water()) {
        return;
    }

    // Movement velocity lead so particles don't trail behind during sprinting/flying
    let player_vel = player_query.single().map_or(Vec3::ZERO, |m| m.velocity);
    let lead_pos = cam_pos + player_vel * 0.35;

    // Sample local biome climate for water tint & arid effects
    let (biome_type, water_color) = if let Some(ref generator) = terrain_generator {
        let sample = generator.sample_column(lead_pos.x.floor() as i32, lead_pos.z.floor() as i32);
        (sample.biome, sample.biome.water_color())
    } else {
        (BiomeType::Plains, [0.80, 0.90, 1.0, 0.65])
    };

    // User requirement: Weather is universal across biomes (Rain is rain, Snow is snow anywhere)
    let effective_rain = w.rain_intensity;
    let effective_snow = w.snow_intensity;

    let scan_start_y = (cam_pos.y + 36.0).floor() as i32;
    let scan_min_y = (cam_pos.y - 24.0).floor() as i32;

    // -------------------------------------------------------------
    // 2. Sky FX: Rain / Heavy Rain
    // -------------------------------------------------------------
    if effective_rain > 0.02 {
        let spawn_count = (effective_rain * 42.0).round() as usize;
        let is_heavy = w.thunder_intensity > 0.20 || effective_rain > 0.70;

        for _ in 0..spawn_count {
            let offset_x = rng.next_triangle(20.0);
            let offset_z = rng.next_triangle(20.0);
            let offset_y = rng.next_f32() * 18.0 - 2.0;

            let wx = (lead_pos.x + offset_x).floor() as i32;
            let wz = (lead_pos.z + offset_z).floor() as i32;
            let candidate_y = lead_pos.y + offset_y;

            // Roof / ceiling occlusion: find highest non-air block in this column
            let (spawn_y, kill_y) = if let Some(roof_block_y) =
                find_column_surface_y(&voxel_world, wx, wz, scan_start_y, scan_min_y)
            {
                let roof_top_y = roof_block_y as f32 + 1.0;
                // If candidate is inside or under the roof / shelter, DO NOT SPAWN!
                if candidate_y <= roof_top_y + 0.1 {
                    continue;
                }
                (candidate_y, roof_top_y)
            } else {
                (candidate_y, cam_pos.y - 30.0)
            };

            let pos = Vec3::new(lead_pos.x + offset_x, spawn_y, lead_pos.z + offset_z);
            let fall_speed = if is_heavy { -25.0 } else { -20.0 };
            let vel = Vec3::new(current_wind.x * 0.9, fall_speed, current_wind.z * 0.9);

            let uv = if is_heavy && rng.next_f32() < 0.40 {
                UV_HEAVY_RAIN
            } else {
                UV_RAIN_DROP
            };

            // User feedback: increased width/thickness for crisp visual readability
            let size = if is_heavy {
                Vec2::new(0.18, 0.95)
            } else {
                Vec2::new(0.14, 0.72)
            };

            let max_life = 1.0;
            pool.spawn(
                pos,
                vel,
                size,
                [0.90, 0.95, 1.0, 0.85],
                uv,
                ParticleRotationMode::RelativeVelocity,
                ParticleFadeMode::Out,
                max_life,
                kill_y,
                if is_heavy {
                    WeatherParticleKind::HeavyRainDrop
                } else {
                    WeatherParticleKind::RainDrop
                },
            );
        }
    }

    // -------------------------------------------------------------
    // 3. Sky FX: Snow / Blizzard
    // -------------------------------------------------------------
    if effective_snow > 0.02 {
        let spawn_count = (effective_snow * 36.0).round() as usize;

        for _ in 0..spawn_count {
            let offset_x = rng.next_triangle(18.0);
            let offset_z = rng.next_triangle(18.0);
            let offset_y = rng.next_f32() * 16.0 - 1.0;

            let wx = (lead_pos.x + offset_x).floor() as i32;
            let wz = (lead_pos.z + offset_z).floor() as i32;
            let candidate_y = lead_pos.y + offset_y;

            // Roof / ceiling occlusion for snowflakes
            let (spawn_y, kill_y) = if let Some(roof_block_y) =
                find_column_surface_y(&voxel_world, wx, wz, scan_start_y, scan_min_y)
            {
                let roof_top_y = roof_block_y as f32 + 1.0;
                if candidate_y <= roof_top_y + 0.1 {
                    continue;
                }
                (candidate_y, roof_top_y)
            } else {
                (candidate_y, cam_pos.y - 30.0)
            };

            let pos = Vec3::new(lead_pos.x + offset_x, spawn_y, lead_pos.z + offset_z);
            let vel = Vec3::new(
                current_wind.x * 0.95 + rng.next_triangle(0.6),
                -1.8 - rng.next_f32() * 1.2,
                current_wind.z * 0.95 + rng.next_triangle(0.6),
            );

            let uv = match rng.next_u32() & 3 {
                0 => UV_SNOW_A,
                1 => UV_SNOW_B,
                2 => UV_SNOW_C,
                _ => UV_SNOW_D,
            };

            let s = 0.24 + rng.next_f32() * 0.14;
            let size = Vec2::splat(s);

            pool.spawn(
                pos,
                vel,
                size,
                [1.0, 1.0, 1.0, 0.88],
                uv,
                ParticleRotationMode::CopyCamera,
                ParticleFadeMode::InOut,
                3.5,
                kill_y,
                WeatherParticleKind::SnowflakeA,
            );
        }
    }

    // -------------------------------------------------------------
    // 4. Sky FX: Dust Storm / Arid Haze
    // -------------------------------------------------------------
    let is_arid = matches!(biome_type, BiomeType::Desert | BiomeType::Savanna);
    let effective_dust = if is_arid && current_wind.length() > 5.0 {
        w.dust_intensity.max(0.40)
    } else {
        w.dust_intensity
    };

    if effective_dust > 0.02 {
        let spawn_count = (effective_dust * 26.0).round() as usize;

        for _ in 0..spawn_count {
            let offset_x = rng.next_triangle(22.0);
            let offset_z = rng.next_triangle(22.0);
            let offset_y = rng.next_f32() * 12.0 - 2.0;

            let wx = (lead_pos.x + offset_x).floor() as i32;
            let wz = (lead_pos.z + offset_z).floor() as i32;
            let candidate_y = lead_pos.y + offset_y;

            if let Some(roof_block_y) =
                find_column_surface_y(&voxel_world, wx, wz, scan_start_y, scan_min_y)
            {
                if candidate_y <= roof_block_y as f32 + 1.0 {
                    continue;
                }
            }

            let pos = Vec3::new(lead_pos.x + offset_x, candidate_y, lead_pos.z + offset_z);
            let vel = Vec3::new(
                current_wind.x * 1.4,
                -0.4 + rng.next_triangle(0.3),
                current_wind.z * 1.4,
            );

            let s = 0.40 + rng.next_f32() * 0.35;
            pool.spawn(
                pos,
                vel,
                Vec2::splat(s),
                [0.85, 0.72, 0.45, 0.45],
                UV_DUST,
                ParticleRotationMode::CopyCamera,
                ParticleFadeMode::InOut,
                2.2,
                -9999.0,
                WeatherParticleKind::DustHaze,
            );
        }
    }

    // -------------------------------------------------------------
    // 5. Surface FX: Water Ripples, Ground Splatters & Thermal Steam
    // -------------------------------------------------------------
    if effective_rain > 0.05 {
        let sample_attempts = (effective_rain * 15.0).round() as usize;

        for _ in 0..sample_attempts {
            let rx = rng.next_triangle(16.0);
            let rz = rng.next_triangle(16.0);
            let wx = (lead_pos.x + rx).floor() as i32;
            let wz = (lead_pos.z + rz).floor() as i32;

            let start_y = (lead_pos.y + 6.0).floor() as i32;
            let min_y = (lead_pos.y - 18.0).floor() as i32;

            for y in (min_y..=start_y).rev() {
                let check_pos = IVec3::new(wx, y, wz);
                if let Some(voxel) = voxel_world.get_voxel(check_pos) {
                    if voxel != Voxel::Air {
                        let is_water = matches!(voxel, Voxel::LiquidWater | Voxel::WaterOccupied)
                            || voxel.is_water();
                        let is_hot = matches!(
                            voxel,
                            Voxel::LiquidLava | Voxel::LiquidCoreMagma | Voxel::LiquidMolten
                        );

                        let surface_pos = Vec3::new(
                            wx as f32 + rng.next_f32(),
                            y as f32 + 1.01,
                            wz as f32 + rng.next_f32(),
                        );

                        if is_water {
                            // Spawn animated expanding water ripple quad tinted by biome water color
                            pool.spawn(
                                surface_pos,
                                Vec3::ZERO,
                                Vec2::splat(0.40 + rng.next_f32() * 0.20),
                                [water_color[0], water_color[1], water_color[2], 0.65],
                                UV_RIPPLE_0,
                                ParticleRotationMode::Horizontal,
                                ParticleFadeMode::Out,
                                0.38,
                                -9999.0,
                                WeatherParticleKind::WaterRipple,
                            );
                        } else if is_hot {
                            // Thermal steam / vaporization smoke plume rising from hot lava
                            pool.spawn(
                                surface_pos,
                                Vec3::new(
                                    rng.next_triangle(0.3),
                                    1.8 + rng.next_f32() * 0.8,
                                    rng.next_triangle(0.3),
                                ),
                                Vec2::splat(0.45 + rng.next_f32() * 0.3),
                                [0.92, 0.92, 0.96, 0.50],
                                UV_FOG,
                                ParticleRotationMode::CopyCamera,
                                ParticleFadeMode::InOut,
                                1.2,
                                -9999.0,
                                WeatherParticleKind::ThermalSmoke,
                            );
                        } else if rng.next_f32() < 0.35 {
                            // Spawn ground splatter bounce droplet
                            pool.spawn(
                                surface_pos,
                                Vec3::new(rng.next_triangle(0.8), 2.2, rng.next_triangle(0.8)),
                                Vec2::splat(0.12),
                                [0.90, 0.95, 1.0, 0.70],
                                UV_SPLASH,
                                ParticleRotationMode::CopyCamera,
                                ParticleFadeMode::Out,
                                0.18,
                                -9999.0,
                                WeatherParticleKind::GroundSplatter,
                            );
                        }
                        break;
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 6. Surface FX: Vertical Wall Streaks (`streak`) on Impermeable/Solid Faces
    // -------------------------------------------------------------
    if effective_rain > 0.15 {
        let streak_samples = (effective_rain * 7.0).round() as usize;

        for _ in 0..streak_samples {
            let rx = rng.next_triangle(12.0);
            let rz = rng.next_triangle(12.0);
            let wx = (lead_pos.x + rx).floor() as i32;
            let wz = (lead_pos.z + rz).floor() as i32;
            let wy = (lead_pos.y + rng.next_triangle(4.0)).floor() as i32;

            let check_pos = IVec3::new(wx, wy, wz);
            if let Some(voxel) = voxel_world.get_voxel(check_pos) {
                if voxel.is_collidable() {
                    let directions = [
                        (IVec3::new(1, 0, 0), ParticleRotationMode::VerticalX, Vec3::new(0.505, 0.5, 0.0)),
                        (IVec3::new(-1, 0, 0), ParticleRotationMode::VerticalX, Vec3::new(-0.505, 0.5, 0.0)),
                        (IVec3::new(0, 0, 1), ParticleRotationMode::VerticalZ, Vec3::new(0.0, 0.5, 0.505)),
                        (IVec3::new(0, 0, -1), ParticleRotationMode::VerticalZ, Vec3::new(0.0, 0.5, -0.505)),
                    ];

                    let dir_idx = (rng.next_u32() % 4) as usize;
                    let (offset_dir, rot_mode, face_offset) = directions[dir_idx];
                    let neighbor_pos = check_pos + offset_dir;

                    if let Some(Voxel::Air) = voxel_world.get_voxel(neighbor_pos) {
                        if let Some(Voxel::Air) = voxel_world.get_voxel(neighbor_pos + IVec3::Y) {
                            let streak_pos = Vec3::new(
                                wx as f32 + face_offset.x + (if face_offset.x == 0.0 { rng.next_triangle(0.35) } else { 0.0 }),
                                wy as f32 + face_offset.y,
                                wz as f32 + face_offset.z + (if face_offset.z == 0.0 { rng.next_triangle(0.35) } else { 0.0 }),
                            );

                            pool.spawn(
                                streak_pos,
                                Vec3::new(0.0, -1.2, 0.0),
                                Vec2::new(0.14, 0.42),
                                [0.85, 0.92, 1.0, 0.55],
                                UV_STREAK,
                                rot_mode,
                                ParticleFadeMode::InOut,
                                0.75,
                                -9999.0,
                                WeatherParticleKind::WindowStreak,
                            );
                        }
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 7. Surface FX: Arid Skittering Sand / Ground Dust
    // -------------------------------------------------------------
    if effective_dust > 0.08 || (is_arid && current_wind.length() > 4.0) {
        let dust_samples = (effective_dust.max(0.3) * 6.0).round() as usize;

        for _ in 0..dust_samples {
            let rx = rng.next_triangle(18.0);
            let rz = rng.next_triangle(18.0);
            let wx = (lead_pos.x + rx).floor() as i32;
            let wz = (lead_pos.z + rz).floor() as i32;

            let start_y = (lead_pos.y + 4.0).floor() as i32;
            let min_y = (lead_pos.y - 12.0).floor() as i32;

            for y in (min_y..=start_y).rev() {
                let check_pos = IVec3::new(wx, y, wz);
                if let Some(voxel) = voxel_world.get_voxel(check_pos) {
                    if voxel != Voxel::Air {
                        let dust_pos = Vec3::new(
                            wx as f32 + rng.next_f32(),
                            y as f32 + 1.06,
                            wz as f32 + rng.next_f32(),
                        );

                        pool.spawn(
                            dust_pos,
                            Vec3::new(current_wind.x * 1.5, 0.02, current_wind.z * 1.5),
                            Vec2::splat(0.25 + rng.next_f32() * 0.15),
                            [0.82, 0.70, 0.48, 0.40],
                            UV_DUST,
                            ParticleRotationMode::Horizontal,
                            ParticleFadeMode::InOut,
                            1.4,
                            -9999.0,
                            WeatherParticleKind::SkitteringDust,
                        );
                        break;
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 8. Surface FX: Post-Rain Rising Mist / Ground Fog
    // -------------------------------------------------------------
    if w.after_weather_timer > 0.0 || (w.current.is_rain() && w.rain_intensity < 0.40) {
        let fog_factor = (w.after_weather_timer / 45.0).clamp(0.0, 1.0);
        if rng.next_f32() < fog_factor * 0.25 {
            let rx = rng.next_triangle(22.0);
            let rz = rng.next_triangle(22.0);
            let wx = (lead_pos.x + rx).floor() as i32;
            let wz = (lead_pos.z + rz).floor() as i32;
            let start_y = (lead_pos.y + 4.0).floor() as i32;
            let min_y = (lead_pos.y - 14.0).floor() as i32;

            for y in (min_y..=start_y).rev() {
                let check_pos = IVec3::new(wx, y, wz);
                if let Some(voxel) = voxel_world.get_voxel(check_pos) {
                    if voxel != Voxel::Air {
                        let fog_pos = Vec3::new(wx as f32 + 0.5, y as f32 + 0.8, wz as f32 + 0.5);
                        pool.spawn(
                            fog_pos,
                            Vec3::new(current_wind.x * 0.2, 0.15, current_wind.z * 0.2),
                            Vec2::splat(2.5 + rng.next_f32() * 1.5),
                            [0.90, 0.95, 1.0, 0.35 * fog_factor],
                            UV_FOG,
                            ParticleRotationMode::Horizontal,
                            ParticleFadeMode::InOut,
                            8.0,
                            -9999.0,
                            WeatherParticleKind::GroundFog,
                        );
                        break;
                    }
                }
            }
        }
    }
}
