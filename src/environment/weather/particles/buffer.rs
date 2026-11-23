use bevy::prelude::*;

pub const MAX_WEATHER_PARTICLES: usize = 12288;
const FOG_SLOT_COUNT: usize = 512;
const FAST_SLOT_COUNT: usize = MAX_WEATHER_PARTICLES - FOG_SLOT_COUNT;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum ParticleRotationMode {
    /// Quads align and stretch along instantaneous velocity vector (e.g. rain drops).
    #[default]
    RelativeVelocity,
    /// View-aligned billboards facing the camera (e.g. snowflakes, dust haze).
    CopyCamera,
    /// Flat horizontal quads parallel to the XZ plane (e.g. water ripples, low mist).
    Horizontal,
    /// Vertical billboards aligned along world Y axis.
    Vertical,
    /// Vertical plane aligned with YZ axis (facing ±X surface of blocks).
    VerticalX,
    /// Vertical plane aligned with XY axis (facing ±Z surface of blocks).
    VerticalZ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum ParticleFadeMode {
    #[default]
    Out,
    In,
    InOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum WeatherParticleKind {
    #[default]
    RainDrop,
    HeavyRainDrop,
    SnowflakeA,
    SnowflakeB,
    WaterRipple,
    GroundSplatter,
    GroundFog,
    DustHaze,
    WindowStreak,
    ThermalSmoke,
    SkitteringDust,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct WeatherParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub size: Vec2,
    pub color: [f32; 4],
    pub uv_min_max: [f32; 4], // [u_min, v_min, u_max, v_max]
    pub rotation_mode: ParticleRotationMode,
    pub fade_mode: ParticleFadeMode,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    /// Absolute world Y elevation below which this particle collides with a roof or ground.
    pub kill_y: f32,
    pub alive: bool,
    pub kind: WeatherParticleKind,
}

impl Default for WeatherParticle {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            size: Vec2::splat(1.0),
            color: [1.0, 1.0, 1.0, 1.0],
            uv_min_max: [0.0, 0.0, 1.0, 1.0],
            rotation_mode: ParticleRotationMode::RelativeVelocity,
            fade_mode: ParticleFadeMode::Out,
            rotation: 0.0,
            angular_velocity: 0.0,
            lifetime: 0.0,
            max_lifetime: 1.0,
            kill_y: -9999.0,
            alive: false,
            kind: WeatherParticleKind::RainDrop,
        }
    }
}

impl WeatherParticle {
    #[inline]
    pub fn current_alpha(&self) -> f32 {
        if self.max_lifetime <= 0.0 {
            return 0.0;
        }
        let progress = (self.lifetime / self.max_lifetime).clamp(0.0, 1.0);
        let base_alpha = self.color[3];

        match self.fade_mode {
            ParticleFadeMode::Out => base_alpha * (1.0 - progress),
            ParticleFadeMode::In => base_alpha * progress,
            ParticleFadeMode::InOut => {
                if progress < 0.30 {
                    let t = progress / 0.30;
                    base_alpha * (t * t * (3.0 - 2.0 * t))
                } else if progress > 0.65 {
                    let t = (1.0 - progress) / 0.35;
                    base_alpha * (t * t * (3.0 - 2.0 * t))
                } else {
                    base_alpha
                }
            }
        }
    }
}

/// Pre-allocated ring buffer resource for high-speed DOD particle management.
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct WeatherParticlePool {
    #[reflect(ignore)]
    pub particles: Vec<WeatherParticle>,
    pub cursor: usize,
    pub fog_cursor: usize,
    pub last_spawned_idx: usize,
    pub active_count: usize,
}

impl Default for WeatherParticlePool {
    fn default() -> Self {
        let mut particles = Vec::with_capacity(MAX_WEATHER_PARTICLES);
        particles.resize_with(MAX_WEATHER_PARTICLES, WeatherParticle::default);
        Self {
            particles,
            cursor: 0,
            fog_cursor: 0,
            last_spawned_idx: 0,
            active_count: 0,
        }
    }
}

impl WeatherParticlePool {
    /// Clears and disables all active particles in the pool immediately.
    pub fn clear(&mut self) {
        for p in self.particles.iter_mut() {
            p.alive = false;
        }
        self.cursor = 0;
        self.fog_cursor = 0;
        self.last_spawned_idx = 0;
        self.active_count = 0;
    }

    /// Sets initial rotation angle (radians) and angular velocity (rad/s) on the most recently spawned particle.
    #[inline]
    pub fn set_last_rotation(&mut self, rotation: f32, angular_velocity: f32) {
        if let Some(p) = self.particles.get_mut(self.last_spawned_idx) {
            p.rotation = rotation;
            p.angular_velocity = angular_velocity;
        }
    }

    /// Spawns a new weather particle into the ring buffer. Long-lived `GroundFog` particles use a
    /// dedicated ring slice so high-rate rain/snow spawning can never overwrite active fog banks.
    #[inline]
    pub fn spawn(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        size: Vec2,
        color: [f32; 4],
        uv_min_max: [f32; 4],
        rotation_mode: ParticleRotationMode,
        fade_mode: ParticleFadeMode,
        max_lifetime: f32,
        kill_y: f32,
        kind: WeatherParticleKind,
    ) {
        let idx = if kind == WeatherParticleKind::GroundFog {
            let slot = FAST_SLOT_COUNT + self.fog_cursor;
            self.fog_cursor = (self.fog_cursor + 1) % FOG_SLOT_COUNT;
            slot
        } else {
            let slot = self.cursor;
            self.cursor = (self.cursor + 1) % FAST_SLOT_COUNT;
            slot
        };
        self.last_spawned_idx = idx;

        self.particles[idx] = WeatherParticle {
            position,
            velocity,
            size,
            color,
            uv_min_max,
            rotation_mode,
            fade_mode,
            rotation: 0.0,
            angular_velocity: 0.0,
            lifetime: 0.0,
            max_lifetime,
            kill_y,
            alive: true,
            kind,
        };
    }

    /// Ticks physical movement and lifetime for all active particles.
    pub fn tick(&mut self, dt: f32, wind: Vec3) {
        let mut active = 0;

        for p in self.particles.iter_mut() {
            if !p.alive {
                continue;
            }

            p.lifetime += dt;
            if p.lifetime >= p.max_lifetime {
                p.alive = false;
                continue;
            }

            // Surface / roof impact check: kill particles that hit ceilings, roofs, or terrain
            if p.position.y <= p.kill_y {
                p.alive = false;
                continue;
            }

            // Physics integration per particle kind
            match p.kind {
                WeatherParticleKind::RainDrop | WeatherParticleKind::HeavyRainDrop => {
                    // Rain falls fast downwards with minor wind sway
                    p.velocity.x = p.velocity.x.lerp(wind.x * 0.85, dt * 1.5);
                    p.velocity.z = p.velocity.z.lerp(wind.z * 0.85, dt * 1.5);
                    p.velocity.y -= 12.0 * dt; // gravity
                    p.position += p.velocity * dt;
                }
                WeatherParticleKind::SnowflakeA | WeatherParticleKind::SnowflakeB => {
                    // Snow drifts significantly with wind gusts
                    p.velocity.x = p.velocity.x.lerp(wind.x * 0.95, dt * 2.0);
                    p.velocity.z = p.velocity.z.lerp(wind.z * 0.95, dt * 2.0);
                    p.velocity.y = (p.velocity.y - 1.2 * dt).max(-3.5);
                    p.position += p.velocity * dt;
                }
                WeatherParticleKind::DustHaze => {
                    // Swirling dust haze blown across ground with gradual expansion
                    p.velocity.x = p.velocity.x.lerp(wind.x * 1.3, dt * 3.0);
                    p.velocity.z = p.velocity.z.lerp(wind.z * 1.3, dt * 3.0);
                    p.position += p.velocity * dt;
                    p.rotation += p.angular_velocity * dt;
                    p.size += Vec2::splat(dt * 0.45);
                }
                WeatherParticleKind::GroundFog => {
                    // Low-lying mist hovers gently over terrain
                    p.velocity.y = 0.03;
                    p.velocity.x = wind.x * 0.06;
                    p.velocity.z = wind.z * 0.06;
                    p.position += p.velocity * dt;
                    p.rotation += p.angular_velocity * dt;
                }
                WeatherParticleKind::GroundSplatter => {
                    p.velocity.y -= 9.8 * dt;
                    p.position += p.velocity * dt;
                }
                WeatherParticleKind::WindowStreak => {
                    p.velocity.y = -1.2;
                    p.position += p.velocity * dt;
                }
                WeatherParticleKind::ThermalSmoke => {
                    // Rising buoyant steam plume with minor drag and wind drift
                    p.velocity.y = (p.velocity.y - 0.4 * dt).max(0.6);
                    p.velocity.x = p.velocity.x.lerp(wind.x * 0.4, dt * 1.5);
                    p.velocity.z = p.velocity.z.lerp(wind.z * 0.4, dt * 1.5);
                    p.position += p.velocity * dt;
                    // Plume expands over time
                    p.size += Vec2::splat(dt * 0.5);
                }
                WeatherParticleKind::SkitteringDust => {
                    // Skitters swiftly across ground surface with wind
                    p.velocity.x = p.velocity.x.lerp(wind.x * 1.6, dt * 3.5);
                    p.velocity.z = p.velocity.z.lerp(wind.z * 1.6, dt * 3.5);
                    p.position += p.velocity * dt;
                }
                WeatherParticleKind::WaterRipple => {
                    // Stationary expanding quad on water surface
                }
            }

            active += 1;
        }

        self.active_count = active;
    }
}
