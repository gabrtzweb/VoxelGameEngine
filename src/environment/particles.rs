use std::f32::consts::TAU;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use crate::{
    environment::{
        DayPhase, EnvironmentState,
        weather::{WeatherManager, WindState},
    },
    generation::BiomeType,
    player::{PlayerCamera, PlayerEnvironmentStatus},
    simulation::VoxelLightRegistry,
    world::{Voxel, storage::VoxelWorld},
};

const MAX_WORLD_PARTICLES: usize = 1024;
const ATLAS_W: usize = 128;
const ATLAS_H: usize = 96;
const ATLAS_CELL: usize = 16;

const SPRITE_BUBBLE_START: usize = 0;
const SPRITE_BUBBLE_POP_START: usize = 3;
const SPRITE_DUST_START: usize = 6;
const SPRITE_FIREFLY_START: usize = 12;
const SPRITE_POLLEN_START: usize = 15;
const SPRITE_SMOKE_START: usize = 18;
const SPRITE_SPLASH_START: usize = 24;
const SPRITE_DUST_CLOUD_START: usize = 27;
const SPRITE_SMOKE_SMALL_START: usize = 33;
const SPRITE_STEAM_START: usize = 39;

const WORLD_PARTICLE_FILES: [&str; 42] = [
    // 0..3: Bubble (colored, 8x8)
    "assets/textures/particles/water_bubble.png",
    "assets/textures/particles/water_bubble1.png",
    "assets/textures/particles/water_bubble2.png",
    // 3..6: Bubble pop animation (colored, 16x16)
    "assets/textures/particles/water_bubble_pop.png",
    "assets/textures/particles/water_bubble_pop1.png",
    "assets/textures/particles/water_bubble_pop2.png",
    // 6..12: Footstep & landing dust gust animation (grayscale, 8x8)
    "assets/textures/particles/dust_gust.png",
    "assets/textures/particles/dust_gust1.png",
    "assets/textures/particles/dust_gust2.png",
    "assets/textures/particles/dust_gust3.png",
    "assets/textures/particles/dust_gust4.png",
    "assets/textures/particles/dust_gust5.png",
    // 12..15: Firefly glow (blinking emissive tint, 8x8)
    "assets/textures/particles/ambience_firefly.png",
    "assets/textures/particles/ambience_firefly1.png",
    "assets/textures/particles/ambience_firefly2.png",
    // 15..18: Ambient pollen specks (colored, 8x8)
    "assets/textures/particles/ambience_pollen.png",
    "assets/textures/particles/ambience_pollen1.png",
    "assets/textures/particles/ambience_pollen2.png",
    // 18..24: Campfire large smoke puffs (grayscale, 16x16)
    "assets/textures/particles/smoke_large.png",
    "assets/textures/particles/smoke_large1.png",
    "assets/textures/particles/smoke_large2.png",
    "assets/textures/particles/smoke_large3.png",
    "assets/textures/particles/smoke_large4.png",
    "assets/textures/particles/smoke_large5.png",
    // 24..27: Water entry & wading splash droplets (colored, 8x8)
    "assets/textures/particles/water_splash.png",
    "assets/textures/particles/water_splash1.png",
    "assets/textures/particles/water_splash2.png",
    // 27..33: Footstep & landing dust cloud animation (grayscale, 16x16)
    "assets/textures/particles/dust_cloud.png",
    "assets/textures/particles/dust_cloud1.png",
    "assets/textures/particles/dust_cloud2.png",
    "assets/textures/particles/dust_cloud3.png",
    "assets/textures/particles/dust_cloud4.png",
    "assets/textures/particles/dust_cloud5.png",
    // 33..39: Campfire small smoke wisps (grayscale, 8x8)
    "assets/textures/particles/smoke_small.png",
    "assets/textures/particles/smoke_small1.png",
    "assets/textures/particles/smoke_small2.png",
    "assets/textures/particles/smoke_small3.png",
    "assets/textures/particles/smoke_small4.png",
    "assets/textures/particles/smoke_small5.png",
    // 39..42: Underwater hydrothermal / magma steam wisps (8x8)
    "assets/textures/particles/smoke_steam.png",
    "assets/textures/particles/smoke_steam1.png",
    "assets/textures/particles/smoke_steam2.png",
];

/// Event-like burst requests pushed by gameplay/audio systems and drained each frame.
#[derive(Clone, Copy, Debug)]
pub enum WorldParticleBurst {
    FootstepDust {
        position: Vec3,
        count: u8,
        color: [f32; 3],
        spread: f32,
    },
    Splash {
        position: Vec3,
        intensity: f32,
        water_tint: [f32; 3],
    },
}

/// Queue of one-shot world particle bursts emitted by player footsteps, landings, and water splashes.
#[derive(Resource, Default)]
pub struct WorldParticleBurstQueue {
    pub bursts: Vec<WorldParticleBurst>,
}

impl WorldParticleBurstQueue {
    #[inline]
    pub fn push_dust(&mut self, position: Vec3, count: u8, color: [f32; 3], spread: f32) {
        self.bursts.push(WorldParticleBurst::FootstepDust {
            position,
            count,
            color,
            spread,
        });
    }

    #[inline]
    pub fn push_splash(&mut self, position: Vec3, intensity: f32, water_tint: [f32; 3]) {
        self.bursts.push(WorldParticleBurst::Splash {
            position,
            intensity,
            water_tint,
        });
    }
}

#[derive(Component)]
pub struct WorldParticleBatchMesh;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorldParticleKind {
    Bubble,
    BubblePop,
    FootstepDust,
    Firefly,
    Pollen,
    CampfireSmoke,
    WaterSplash,
}

#[derive(Clone, Copy)]
struct WorldParticle {
    kind: WorldParticleKind,
    position: Vec3,
    velocity: Vec3,
    size: f32,
    grow_rate: f32,
    rotation: f32,
    spin: f32,
    color: [f32; 3],
    base_alpha: f32,
    phase: f32,
    freq: f32,
    base_sprite_idx: usize,
    age: f32,
    max_age: f32,
    alive: bool,
}

impl Default for WorldParticle {
    fn default() -> Self {
        Self {
            kind: WorldParticleKind::CampfireSmoke,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            size: 0.4,
            grow_rate: 0.0,
            rotation: 0.0,
            spin: 0.0,
            color: [1.0, 1.0, 1.0],
            base_alpha: 1.0,
            phase: 0.0,
            freq: 1.0,
            base_sprite_idx: 0,
            age: 0.0,
            max_age: 1.0,
            alive: false,
        }
    }
}

#[derive(Resource)]
pub struct WorldParticlePool {
    particles: Vec<WorldParticle>,
    cursor: usize,
    uv_rects: [[f32; 4]; 42],
    mesh_handle: Handle<Mesh>,
    last_cam_pos: Vec3,
    has_last_cam: bool,
    rng: u64,
}

impl WorldParticlePool {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.rng = self
            .rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.rng >> 32) as u32
    }

    #[inline]
    fn next_f32(&mut self) -> f32 {
        (self.next_u32() & 0x00FF_FFFF) as f32 / 16_777_216.0
    }

    #[inline]
    fn spawn(&mut self, p: WorldParticle) {
        let slot = self.cursor;
        self.cursor = (self.cursor + 1) % MAX_WORLD_PARTICLES;
        self.particles[slot] = p;
    }

    fn clear(&mut self) {
        for p in self.particles.iter_mut() {
            p.alive = false;
        }
        self.cursor = 0;
    }
}

/// Builds a crisp nearest-neighbor texture atlas from all 27 world particle PNGs and spawns
/// the single batched mesh entity used to render bubbles, dust, fireflies, pollen, splashes, and smoke.
pub fn setup_world_particles(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut atlas_bytes = vec![0u8; ATLAS_W * ATLAS_H * 4];
    let mut uv_rects = [[0.0f32; 4]; 42];
    let cols = ATLAS_W / ATLAS_CELL;

    for (idx, path) in WORLD_PARTICLE_FILES.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        let ox = col * ATLAS_CELL;
        let oy = row * ATLAS_CELL;

        let (w, h) = if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (iw, ih) = rgba.dimensions();
            let cw = (iw as usize).min(ATLAS_CELL);
            let ch = (ih as usize).min(ATLAS_CELL);
            for py in 0..ch {
                for px in 0..cw {
                    let src = rgba.get_pixel(px as u32, py as u32).0;
                    let dst_idx = ((oy + py) * ATLAS_W + (ox + px)) * 4;
                    atlas_bytes[dst_idx..dst_idx + 4].copy_from_slice(&src);
                }
            }
            (cw, ch)
        } else {
            (8, 8)
        };

        uv_rects[idx] = [
            ox as f32 / ATLAS_W as f32,
            oy as f32 / ATLAS_H as f32,
            (ox + w) as f32 / ATLAS_W as f32,
            (oy + h) as f32 / ATLAS_H as f32,
        ];
    }

    let mut atlas_image = Image::new(
        Extent3d {
            width: ATLAS_W as u32,
            height: ATLAS_H as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    atlas_image.sampler = ImageSampler::nearest();
    let atlas_handle = images.add(atlas_image);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 0.0]; 3]);
    mesh.insert_indices(Indices::U32(vec![0, 0, 0]));

    let mesh_handle = meshes.add(mesh);

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_handle),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    commands.spawn((
        WorldParticleBatchMesh,
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material_handle),
        NotShadowCaster,
        NotShadowReceiver,
        Transform::default(),
        Visibility::Hidden,
    ));

    commands.insert_resource(WorldParticlePool {
        particles: vec![WorldParticle::default(); MAX_WORLD_PARTICLES],
        cursor: 0,
        uv_rects,
        mesh_handle,
        last_cam_pos: Vec3::ZERO,
        has_last_cam: false,
        rng: 0xA24B_AED4_963E_E407,
    });
}

#[inline]
fn spawn_bubble_at(pool: &mut WorldParticlePool, pos: IVec3, speed_mult: f32) {
    let variant = SPRITE_BUBBLE_START + (pool.next_u32() as usize % 3);
    let spawn_pos = Vec3::new(
        pos.x as f32 + 0.18 + pool.next_f32() * 0.64,
        pos.y as f32 + 1.03,
        pos.z as f32 + 0.18 + pool.next_f32() * 0.64,
    );
    let vy = (1.5 + pool.next_f32() * 0.85) * speed_mult;
    let wobble_phase = pool.next_f32() * TAU;
    let wobble_freq = 2.6 + pool.next_f32() * 1.6;
    let size = 0.22 + pool.next_f32() * 0.13;
    let max_age = 8.0 + pool.next_f32() * 4.0;

    pool.spawn(WorldParticle {
        kind: WorldParticleKind::Bubble,
        position: spawn_pos,
        velocity: Vec3::new(0.0, vy, 0.0),
        size,
        grow_rate: 0.0,
        rotation: 0.0,
        spin: 0.0,
        color: [1.0, 1.0, 1.0],
        base_alpha: 0.92,
        phase: wobble_phase,
        freq: wobble_freq,
        base_sprite_idx: variant,
        age: 0.0,
        max_age,
        alive: true,
    });
}

#[inline]
fn spawn_steam_at(pool: &mut WorldParticlePool, pos: IVec3) {
    let variant = SPRITE_STEAM_START + (pool.next_u32() as usize % 3);
    let spawn_pos = Vec3::new(
        pos.x as f32 + 0.16 + pool.next_f32() * 0.68,
        pos.y as f32 + 1.06,
        pos.z as f32 + 0.16 + pool.next_f32() * 0.68,
    );
    let vx = (pool.next_f32() - 0.5) * 0.18;
    let vz = (pool.next_f32() - 0.5) * 0.18;
    let vy = 0.72 + pool.next_f32() * 0.45;
    let size = 0.36 + pool.next_f32() * 0.22;
    let max_age = 2.6 + pool.next_f32() * 1.8;
    let rot = pool.next_f32() * TAU;
    let spin = (pool.next_f32() - 0.5) * 0.7;
    let phase = pool.next_f32() * TAU;
    let freq = 1.6 + pool.next_f32() * 0.9;

    pool.spawn(WorldParticle {
        kind: WorldParticleKind::CampfireSmoke,
        position: spawn_pos,
        velocity: Vec3::new(vx, vy, vz),
        size,
        grow_rate: 0.18,
        rotation: rot,
        spin,
        color: [0.88, 0.93, 0.96],
        base_alpha: 0.58,
        phase,
        freq,
        base_sprite_idx: variant,
        age: 0.0,
        max_age,
        alive: true,
    });
}

#[inline]
fn smooth_in_out_alpha(age: f32, max_age: f32, fade_in: f32, fade_out: f32) -> f32 {
    if age < fade_in && fade_in > 0.0 {
        let t = (age / fade_in).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    } else if age > max_age - fade_out && fade_out > 0.0 {
        let t = ((max_age - age) / fade_out).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    } else {
        1.0
    }
}

/// Emits, simulates, and renders all batched world particles (bubbles, footstep dust,
/// blinking fireflies, pollen, water splashes, and campfire smoke) in a single draw call.
#[allow(clippy::too_many_arguments)]
pub fn update_world_particles(
    time: Res<Time>,
    voxel_world: Res<VoxelWorld>,
    wind: Option<Res<WindState>>,
    weather: Option<Res<WeatherManager>>,
    env_state: Option<Res<EnvironmentState>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    light_registry: Option<Res<VoxelLightRegistry>>,
    mut burst_queue: ResMut<WorldParticleBurstQueue>,
    mut pool: Option<ResMut<WorldParticlePool>>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut batch_query: Query<&mut Visibility, With<WorldParticleBatchMesh>>,
) {
    let Some(ref mut pool) = pool else {
        burst_queue.bursts.clear();
        return;
    };
    let Ok(cam_transform) = camera_query.single() else {
        burst_queue.bursts.clear();
        return;
    };
    let Ok(mut batch_vis) = batch_query.single_mut() else {
        burst_queue.bursts.clear();
        return;
    };

    let dt = time.delta_secs().min(0.1);
    if dt <= 0.0 {
        return;
    }

    let cam_pos = cam_transform.translation;
    if pool.has_last_cam && (cam_pos - pool.last_cam_pos).length_squared() > 4096.0 {
        pool.clear();
    }
    pool.last_cam_pos = cam_pos;
    pool.has_last_cam = true;

    let wind_vec = wind.as_ref().map_or(Vec3::ZERO, |w| w.current_wind);
    let time_of_day = env_state.as_ref().map_or(0.5, |e| e.time_of_day);
    let day_phase = DayPhase::from_time(time_of_day);
    let day_light = {
        let sun = ((time_of_day - 0.25) * TAU).sin().clamp(-1.0, 1.0);
        0.28 + 0.72 * ((sun + 0.25) / 1.25).clamp(0.0, 1.0)
    };

    let status = env_status.as_deref().copied().unwrap_or_default();
    let is_stormy = weather.as_ref().is_some_and(|w| {
        w.rain_intensity > 0.25 || w.snow_intensity > 0.20 || w.dust_intensity > 0.20
    });

    // -------------------------------------------------------------
    // 1. Drain Footstep Dust & Water Entry / Item Splash Bursts
    // -------------------------------------------------------------
    for burst in burst_queue.bursts.drain(..) {
        match burst {
            WorldParticleBurst::FootstepDust {
                position,
                count,
                color,
                spread,
            } => {
                let is_landing = count >= 5 || spread >= 0.28;
                let cloud_prob = if is_landing { 0.75 } else { 0.45 };
                for _ in 0..count {
                    let use_cloud = pool.next_f32() < cloud_prob;
                    let jx = (pool.next_f32() - 0.5) * spread * 2.0;
                    let jz = (pool.next_f32() - 0.5) * spread * 2.0;
                    let jy = 0.05 + pool.next_f32() * 0.08;
                    let vx = jx * 1.4 + (pool.next_f32() - 0.5) * 0.45 + wind_vec.x * 0.08;
                    let vz = jz * 1.4 + (pool.next_f32() - 0.5) * 0.45 + wind_vec.z * 0.08;
                    let vy = if use_cloud {
                        0.28 + pool.next_f32() * 0.42
                    } else {
                        0.35 + pool.next_f32() * 0.55
                    };
                    let max_age = if use_cloud {
                        0.55 + pool.next_f32() * 0.28
                    } else {
                        0.45 + pool.next_f32() * 0.22
                    };
                    let size = if use_cloud {
                        0.56 + pool.next_f32() * 0.26
                    } else {
                        0.42 + pool.next_f32() * 0.20
                    };
                    let rot = pool.next_f32() * TAU;
                    let spin = (pool.next_f32() - 0.5) * 2.5;
                    let shade = 0.90 + pool.next_f32() * 0.16;

                    pool.spawn(WorldParticle {
                        kind: WorldParticleKind::FootstepDust,
                        position: position + Vec3::new(jx, jy, jz),
                        velocity: Vec3::new(vx, vy, vz),
                        size,
                        grow_rate: if use_cloud { 0.48 } else { 0.35 },
                        rotation: rot,
                        spin,
                        color: [color[0] * shade, color[1] * shade, color[2] * shade],
                        base_alpha: if use_cloud { 0.74 } else { 0.78 },
                        phase: 0.0,
                        freq: 0.0,
                        base_sprite_idx: if use_cloud {
                            SPRITE_DUST_CLOUD_START
                        } else {
                            SPRITE_DUST_START
                        },
                        age: 0.0,
                        max_age,
                        alive: true,
                    });
                }
            }
            WorldParticleBurst::Splash {
                position,
                intensity,
                water_tint,
            } => {
                let clamped = intensity.clamp(0.35, 1.8);
                let count = ((6.0 + clamped * 5.0).round() as usize).clamp(4, 18);
                for _ in 0..count {
                    let angle = pool.next_f32() * TAU;
                    let radial = (0.7 + pool.next_f32() * 1.5) * clamped.sqrt();
                    let vy = (1.6 + pool.next_f32() * 2.3) * clamped.clamp(0.55, 1.35);
                    let ox = angle.cos() * (0.08 + pool.next_f32() * 0.22);
                    let oz = angle.sin() * (0.08 + pool.next_f32() * 0.22);
                    let variant = SPRITE_SPLASH_START + (pool.next_u32() as usize % 3);
                    let size = 0.38 + pool.next_f32() * 0.20;
                    let max_age = 0.45 + pool.next_f32() * 0.40;
                    let tint = [
                        water_tint[0] * 0.45 + 0.55,
                        water_tint[1] * 0.45 + 0.55,
                        water_tint[2] * 0.35 + 0.65,
                    ];

                    let rot = pool.next_f32() * TAU;
                    let spin = (pool.next_f32() - 0.5) * 4.0;
                    let particle = WorldParticle {
                        kind: WorldParticleKind::WaterSplash,
                        position: position + Vec3::new(ox, 0.05, oz),
                        velocity: Vec3::new(angle.cos() * radial, vy, angle.sin() * radial),
                        size,
                        grow_rate: 0.0,
                        rotation: rot,
                        spin,
                        color: tint,
                        base_alpha: 0.92,
                        phase: 0.0,
                        freq: 0.0,
                        base_sprite_idx: variant,
                        age: 0.0,
                        max_age,
                        alive: true,
                    };
                    pool.spawn(particle);
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 2. Active Block Emitters via VoxelLightRegistry:
    //    - Lit Campfires -> Rising Smoke (NEVER torches)
    //    - Submerged GeothermalVents & Magma -> Rising Bubbles
    // -------------------------------------------------------------
    if let Some(ref registry) = light_registry {
        let max_dist_sq = 48.0 * 48.0;
        for (&pos, state) in &registry.entries {
            let center = pos.as_vec3() + Vec3::splat(0.5);
            if (center - cam_pos).length_squared() > max_dist_sq {
                continue;
            }

            if state.voxel == Voxel::Campfire {
                let spawn_prob = (4.2 * dt).min(1.0);
                if pool.next_f32() < spawn_prob {
                    let use_small = pool.next_f32() < 0.45;
                    let ox = (pool.next_f32() - 0.5) * 0.34;
                    let oz = (pool.next_f32() - 0.5) * 0.34;
                    let vx = wind_vec.x * 0.14 + (pool.next_f32() - 0.5) * 0.14;
                    let vz = wind_vec.z * 0.14 + (pool.next_f32() - 0.5) * 0.14;
                    let vy = if use_small {
                        0.92 + pool.next_f32() * 0.40
                    } else {
                        0.78 + pool.next_f32() * 0.36
                    };
                    let size = if use_small {
                        0.32 + pool.next_f32() * 0.16
                    } else {
                        0.52 + pool.next_f32() * 0.22
                    };
                    let max_age = if use_small {
                        3.4 + pool.next_f32() * 1.6
                    } else {
                        5.8 + pool.next_f32() * 2.6
                    };
                    let shade = 0.88 + pool.next_f32() * 0.12;
                    let variant_offset = pool.next_u32() as usize % 2;
                    let group_start = if use_small {
                        SPRITE_SMOKE_SMALL_START
                    } else {
                        SPRITE_SMOKE_START
                    };

                    let rot = pool.next_f32() * TAU;
                    let spin = (pool.next_f32() - 0.5) * 0.45;
                    let phase = pool.next_f32() * TAU;
                    let freq = 1.2 + pool.next_f32() * 0.6;
                    let particle = WorldParticle {
                        kind: WorldParticleKind::CampfireSmoke,
                        position: Vec3::new(
                            pos.x as f32 + 0.5 + ox,
                            pos.y as f32 + 0.72,
                            pos.z as f32 + 0.5 + oz,
                        ),
                        velocity: Vec3::new(vx, vy, vz),
                        size,
                        grow_rate: if use_small { 0.10 } else { 0.14 },
                        rotation: rot,
                        spin,
                        color: [0.76 * shade, 0.74 * shade, 0.72 * shade],
                        base_alpha: if use_small { 0.66 } else { 0.72 },
                        phase,
                        freq,
                        base_sprite_idx: group_start + variant_offset,
                        age: 0.0,
                        max_age,
                        alive: true,
                    };
                    pool.spawn(particle);
                }
            } else if matches!(state.voxel, Voxel::GeothermalVent | Voxel::Magma)
                && voxel_world
                    .get_voxel(pos + IVec3::Y)
                    .is_some_and(Voxel::is_water)
            {
                let rate = if state.voxel == Voxel::GeothermalVent {
                    5.5
                } else {
                    2.8
                };
                if pool.next_f32() < (rate * dt).min(1.0) {
                    let speed_mult = if state.voxel == Voxel::GeothermalVent {
                        1.15
                    } else {
                        0.95
                    };
                    spawn_bubble_at(pool, pos, speed_mult);
                    if pool.next_f32() < 0.42 {
                        spawn_steam_at(pool, pos);
                    }
                }
            }
        }
    }

    // Extra lightweight underwater probes so dense hydrothermal/magma clusters beyond
    // per-chunk point-light cap also emit rising bubbles.
    if status.is_camera_in_water || cam_pos.y <= 24.0 {
        for _ in 0..16 {
            let ox = ((pool.next_f32() - 0.5) * 34.0).round() as i32;
            let oz = ((pool.next_f32() - 0.5) * 34.0).round() as i32;
            let oy = (-14.0 + pool.next_f32() * 20.0).round() as i32;
            let pos = IVec3::new(
                cam_pos.x.floor() as i32 + ox,
                cam_pos.y.floor() as i32 + oy,
                cam_pos.z.floor() as i32 + oz,
            );
            if let Some(v) = voxel_world.get_voxel(pos)
                && matches!(v, Voxel::GeothermalVent | Voxel::Magma)
                && voxel_world
                    .get_voxel(pos + IVec3::Y)
                    .is_some_and(Voxel::is_water)
                && pool.next_f32() < 0.45
            {
                spawn_bubble_at(pool, pos, 1.0);
                if pool.next_f32() < 0.35 {
                    spawn_steam_at(pool, pos);
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 3. Blinking Fireflies (Evening & Night in Forest / Swamp Biomes)
    // -------------------------------------------------------------
    let is_firefly_time = matches!(day_phase, DayPhase::Evening | DayPhase::Night);
    if is_firefly_time && !status.is_underground && !status.is_camera_in_water && !is_stormy {
        let firefly_rate = match status.current_biome {
            BiomeType::OrchidSwamp => 7.5,
            BiomeType::YosemiteLowlands
            | BiomeType::Shield
            | BiomeType::CloudForest
            | BiomeType::BloomingValley
            | BiomeType::LavenderValley
            | BiomeType::SakuraGrove
            | BiomeType::TropicalJungle
            | BiomeType::SiberianTaiga
            | BiomeType::WintryForest => 4.0,
            _ => 0.0,
        };

        if firefly_rate > 0.0 && pool.next_f32() < (firefly_rate * dt).min(1.0) {
            let wx = (cam_pos.x + (pool.next_f32() - 0.5) * 44.0).floor() as i32;
            let wz = (cam_pos.z + (pool.next_f32() - 0.5) * 44.0).floor() as i32;
            let scan_top = (cam_pos.y + 8.0).floor() as i32;
            let scan_bot = (cam_pos.y - 10.0).floor() as i32;

            for y in (scan_bot..=scan_top).rev() {
                let ground_pos = IVec3::new(wx, y, wz);
                if let Some(v) = voxel_world.get_voxel(ground_pos)
                    && v.is_collidable()
                    && !v.is_leaves()
                    && voxel_world.get_voxel(ground_pos + IVec3::Y) == Some(Voxel::Air)
                {
                    let spawn_pos = Vec3::new(
                        wx as f32 + 0.5 + (pool.next_f32() - 0.5) * 0.6,
                        y as f32 + 1.3 + pool.next_f32() * 2.5,
                        wz as f32 + 0.5 + (pool.next_f32() - 0.5) * 0.6,
                    );
                    let variant = SPRITE_FIREFLY_START + (pool.next_u32() as usize % 3);
                    let warm_mix = pool.next_f32();
                    let tint = [
                        0.82 + 0.18 * warm_mix,
                        0.98 - 0.08 * warm_mix,
                        0.28 + 0.08 * (1.0 - warm_mix),
                    ];
                    let vel = Vec3::new(
                        (pool.next_f32() - 0.5) * 0.35,
                        (pool.next_f32() - 0.5) * 0.14,
                        (pool.next_f32() - 0.5) * 0.35,
                    );
                    let size = 0.38 + pool.next_f32() * 0.14;
                    let phase = pool.next_f32() * TAU;
                    let freq = 1.4 + pool.next_f32() * 1.4;
                    let max_age = 8.0 + pool.next_f32() * 6.0;
                    let particle = WorldParticle {
                        kind: WorldParticleKind::Firefly,
                        position: spawn_pos,
                        velocity: vel,
                        size,
                        grow_rate: 0.0,
                        rotation: 0.0,
                        spin: 0.0,
                        color: tint,
                        base_alpha: 1.0,
                        phase,
                        freq,
                        base_sprite_idx: variant,
                        age: 0.0,
                        max_age,
                        alive: true,
                    };
                    pool.spawn(particle);
                    break;
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 4. Daytime Ambient Pollen (Morning & Noon in Floral / Lush Biomes)
    // -------------------------------------------------------------
    let is_pollen_time = matches!(day_phase, DayPhase::Morning | DayPhase::Noon);
    if is_pollen_time && !status.is_underground && !status.is_camera_in_water && !is_stormy {
        let pollen_rate = match status.current_biome {
            BiomeType::BloomingValley | BiomeType::LavenderValley | BiomeType::SakuraGrove => 9.0,
            BiomeType::YosemiteLowlands
            | BiomeType::Shield
            | BiomeType::CloudForest
            | BiomeType::TropicalJungle
            | BiomeType::OrchidSwamp => 4.8,
            _ => 0.0,
        };

        if pollen_rate > 0.0 && pool.next_f32() < (pollen_rate * dt).min(1.0) {
            let wx = (cam_pos.x + (pool.next_f32() - 0.5) * 38.0).floor() as i32;
            let wz = (cam_pos.z + (pool.next_f32() - 0.5) * 38.0).floor() as i32;
            let wy = (cam_pos.y - 1.5 + pool.next_f32() * 6.5).floor() as i32;
            let spawn_voxel = IVec3::new(wx, wy, wz);

            if voxel_world.get_voxel(spawn_voxel) == Some(Voxel::Air) {
                let has_ground_below = (1..=8).any(|dy| {
                    voxel_world
                        .get_voxel(spawn_voxel - IVec3::Y * dy)
                        .is_some_and(Voxel::is_collidable)
                });
                if has_ground_below {
                    let variant = SPRITE_POLLEN_START + (pool.next_u32() as usize % 3);
                    let pos = Vec3::new(
                        wx as f32 + pool.next_f32(),
                        wy as f32 + pool.next_f32(),
                        wz as f32 + pool.next_f32(),
                    );
                    let vel = Vec3::new(
                        wind_vec.x * 0.24 + (pool.next_f32() - 0.5) * 0.22,
                        (pool.next_f32() - 0.45) * 0.14,
                        wind_vec.z * 0.24 + (pool.next_f32() - 0.5) * 0.22,
                    );
                    let size = 0.34 + pool.next_f32() * 0.14;
                    let phase = pool.next_f32() * TAU;
                    let freq = 1.5 + pool.next_f32() * 1.2;
                    let max_age = 6.0 + pool.next_f32() * 4.0;
                    let particle = WorldParticle {
                        kind: WorldParticleKind::Pollen,
                        position: pos,
                        velocity: vel,
                        size,
                        grow_rate: 0.0,
                        rotation: 0.0,
                        spin: 0.0,
                        color: [1.0, 0.98, 0.85],
                        base_alpha: 0.82,
                        phase,
                        freq,
                        base_sprite_idx: variant,
                        age: 0.0,
                        max_age,
                        alive: true,
                    };
                    pool.spawn(particle);
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 5. Simulate All Active Particles & Build Single Batched Mesh
    // -------------------------------------------------------------
    let cam_right = *cam_transform.right();
    let cam_up = *cam_transform.up();
    let uv_rects = pool.uv_rects;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut cursor_v = 0u32;

    for p in pool.particles.iter_mut() {
        if !p.alive {
            continue;
        }

        p.age += dt;
        if p.age >= p.max_age {
            p.alive = false;
            continue;
        }

        match p.kind {
            WorldParticleKind::Bubble => {
                let wobble_x = (p.age * p.freq + p.phase).sin() * 0.22;
                let wobble_z = (p.age * p.freq * 0.87 + p.phase + 1.1).cos() * 0.22;
                let next_pos = p.position + Vec3::new(wobble_x, p.velocity.y, wobble_z) * dt;
                let check_iv = next_pos.floor().as_ivec3();
                if let Some(v) = voxel_world.get_voxel(check_iv)
                    && v.is_water()
                {
                    p.position = next_pos;
                } else {
                    // Convert to animated bubble pop right at the water surface
                    p.kind = WorldParticleKind::BubblePop;
                    p.position = Vec3::new(next_pos.x, check_iv.y as f32 - 0.06, next_pos.z);
                    p.velocity = Vec3::ZERO;
                    p.size = 0.40;
                    p.age = 0.0;
                    p.max_age = 0.24;
                    p.base_sprite_idx = SPRITE_BUBBLE_POP_START;
                }
            }
            WorldParticleKind::BubblePop => {
                // Stationary surface pop animation (3 frames)
            }
            WorldParticleKind::FootstepDust => {
                p.velocity.x *= (1.0 - 4.0 * dt).max(0.0);
                p.velocity.z *= (1.0 - 4.0 * dt).max(0.0);
                p.velocity.y -= 0.45 * dt;
                p.position += p.velocity * dt;
                p.size += p.grow_rate * dt;
                p.rotation += p.spin * dt;
            }
            WorldParticleKind::Firefly => {
                let wander_x = (p.age * 0.7 + p.phase).sin() * 0.42;
                let wander_y = (p.age * 1.1 + p.phase * 1.7).cos() * 0.16;
                let wander_z = (p.age * 0.63 + p.phase * 2.3).cos() * 0.42;
                let next_pos = p.position
                    + (p.velocity + Vec3::new(wander_x, wander_y, wander_z) + wind_vec * 0.08) * dt;
                let iv = next_pos.floor().as_ivec3();
                if voxel_world
                    .get_voxel(iv)
                    .is_some_and(|v| v.is_collidable() || v.is_water())
                {
                    p.velocity = -p.velocity;
                } else {
                    p.position = next_pos;
                }
            }
            WorldParticleKind::Pollen => {
                let drift_x = (p.age * p.freq + p.phase).sin() * 0.18;
                let drift_y = (p.age * p.freq * 0.8 + p.phase).cos() * 0.10;
                let drift_z = (p.age * p.freq * 0.9 + p.phase + 1.4).sin() * 0.18;
                p.position += (p.velocity + Vec3::new(drift_x, drift_y, drift_z)) * dt;
            }
            WorldParticleKind::CampfireSmoke => {
                let sway_x = (p.age * p.freq + p.phase).sin() * 0.16;
                let sway_z = (p.age * p.freq * 0.85 + p.phase + 1.2).cos() * 0.16;
                let next_pos = p.position
                    + Vec3::new(
                        p.velocity.x + wind_vec.x * 0.16 * dt + sway_x,
                        p.velocity.y,
                        p.velocity.z + wind_vec.z * 0.16 * dt + sway_z,
                    ) * dt;
                if voxel_world
                    .get_voxel(next_pos.floor().as_ivec3())
                    .is_some_and(Voxel::is_solid_opaque)
                {
                    p.alive = false;
                    continue;
                }
                p.position = next_pos;
                p.size += p.grow_rate * dt;
                p.rotation += p.spin * dt;
            }
            WorldParticleKind::WaterSplash => {
                p.velocity.y -= 11.5 * dt;
                p.position += p.velocity * dt;
                p.rotation += p.spin * dt;
            }
        }

        let progress = (p.age / p.max_age).clamp(0.0, 1.0);
        let sprite_idx = match p.kind {
            WorldParticleKind::BubblePop => {
                SPRITE_BUBBLE_POP_START + ((progress * 3.0).floor() as usize).min(2)
            }
            WorldParticleKind::FootstepDust => {
                p.base_sprite_idx + ((progress * 6.0).floor() as usize).min(5)
            }
            WorldParticleKind::CampfireSmoke => {
                if p.base_sprite_idx >= SPRITE_STEAM_START {
                    p.base_sprite_idx
                } else {
                    let group_start = if p.base_sprite_idx >= SPRITE_SMOKE_SMALL_START {
                        SPRITE_SMOKE_SMALL_START
                    } else {
                        SPRITE_SMOKE_START
                    };
                    let local =
                        (p.base_sprite_idx - group_start) + ((progress * 4.0).floor() as usize);
                    group_start + local.min(5)
                }
            }
            _ => p.base_sprite_idx,
        };

        let (rgb, alpha) = match p.kind {
            WorldParticleKind::Bubble => {
                let light = (0.35 + 0.65 * day_light).min(1.0);
                let a = p.base_alpha * smooth_in_out_alpha(p.age, p.max_age, 0.12, 0.4);
                (
                    [p.color[0] * light, p.color[1] * light, p.color[2] * light],
                    a,
                )
            }
            WorldParticleKind::BubblePop => {
                let light = (0.35 + 0.65 * day_light).min(1.0);
                let a = 1.0 - progress * 0.35;
                (
                    [p.color[0] * light, p.color[1] * light, p.color[2] * light],
                    a,
                )
            }
            WorldParticleKind::FootstepDust => {
                let t = 1.0 - progress;
                let a = p.base_alpha * (t * t * (3.0 - 2.0 * t));
                (
                    [
                        p.color[0] * day_light,
                        p.color[1] * day_light,
                        p.color[2] * day_light,
                    ],
                    a,
                )
            }
            WorldParticleKind::Firefly => {
                // Pulsing emissive blink glow — never dimmed by night `day_light`!
                let blink = (0.5 + 0.5 * (p.age * p.freq + p.phase).sin()).powf(2.2);
                let glow = 0.25 + 1.35 * blink;
                let fade = smooth_in_out_alpha(p.age, p.max_age, 0.9, 1.4);
                let a = fade * (0.20 + 0.80 * blink);
                ([p.color[0] * glow, p.color[1] * glow, p.color[2] * glow], a)
            }
            WorldParticleKind::Pollen => {
                let a = p.base_alpha * smooth_in_out_alpha(p.age, p.max_age, 0.8, 1.4);
                (
                    [
                        p.color[0] * day_light,
                        p.color[1] * day_light,
                        p.color[2] * day_light,
                    ],
                    a,
                )
            }
            WorldParticleKind::CampfireSmoke => {
                let smoke_light = 0.42 + 0.58 * day_light;
                let a = p.base_alpha * smooth_in_out_alpha(p.age, p.max_age, 0.25, 2.4);
                (
                    [
                        p.color[0] * smoke_light,
                        p.color[1] * smoke_light,
                        p.color[2] * smoke_light,
                    ],
                    a,
                )
            }
            WorldParticleKind::WaterSplash => {
                let a = p.base_alpha * (1.0 - progress * progress);
                (
                    [
                        p.color[0] * day_light,
                        p.color[1] * day_light,
                        p.color[2] * day_light,
                    ],
                    a,
                )
            }
        };

        if alpha <= 0.01 {
            continue;
        }

        let half = p.size * 0.5;
        let (sin_r, cos_r) = p.rotation.sin_cos();
        let rot_r = (cam_right * cos_r + cam_up * sin_r) * half;
        let rot_u = (-cam_right * sin_r + cam_up * cos_r) * half;

        let c0 = p.position - rot_r - rot_u;
        let c1 = p.position + rot_r - rot_u;
        let c2 = p.position + rot_r + rot_u;
        let c3 = p.position - rot_r + rot_u;

        let uv = uv_rects[sprite_idx];
        let col = [rgb[0], rgb[1], rgb[2], alpha];

        positions.extend_from_slice(&[c0.to_array(), c1.to_array(), c2.to_array(), c3.to_array()]);
        normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
        uvs.extend_from_slice(&[
            [uv[0], uv[3]],
            [uv[2], uv[3]],
            [uv[2], uv[1]],
            [uv[0], uv[1]],
        ]);
        colors.extend_from_slice(&[col; 4]);
        indices.extend_from_slice(&[
            cursor_v,
            cursor_v + 1,
            cursor_v + 2,
            cursor_v,
            cursor_v + 2,
            cursor_v + 3,
        ]);
        cursor_v += 4;
    }

    if positions.is_empty() {
        if *batch_vis != Visibility::Hidden {
            *batch_vis = Visibility::Hidden;
        }
        return;
    }

    if *batch_vis != Visibility::Visible {
        *batch_vis = Visibility::Visible;
    }

    if let Some(mut mesh) = meshes.get_mut(&pool.mesh_handle) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
    }
}
