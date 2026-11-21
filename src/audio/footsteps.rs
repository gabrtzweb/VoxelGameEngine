use bevy::{platform::collections::HashMap, prelude::*};
use crate::{
    audio::{PlaySfxEvent, material::SoundMaterial},
    environment::WorldParticleBurstQueue,
    player::{Player, PlayerEnvironmentStatus, PlayerMotion, PlayerStance},
    world::{VOXEL_SIZE, Voxel, VoxelWorld},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum StepSoundCategory {
    Walk(SoundMaterial),
    Run(SoundMaterial),
    Land(SoundMaterial),
    WaterWade,
    Swim,
    SplashEnter,
    SplashExit,
}

#[derive(Resource)]
pub struct FootstepAudioState {
    distance_accum: f32,
    was_grounded: bool,
    was_in_water: bool,
    prev_velocity_y: f32,
    swim_timer: f32,
    rng_state: u64,
    handles: HashMap<StepSoundCategory, Vec<Handle<AudioSource>>>,
}

impl Default for FootstepAudioState {
    fn default() -> Self {
        Self {
            distance_accum: 0.0,
            was_grounded: true,
            was_in_water: false,
            prev_velocity_y: 0.0,
            swim_timer: 0.0,
            rng_state: 0x9E37_79B9_7F4A_7C15,
            handles: HashMap::default(),
        }
    }
}

impl FootstepAudioState {
    fn next_float(&mut self) -> f32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.rng_state >> 32) as u32 as f32) / 4294967295.0
    }

    fn get_sound(
        &mut self,
        category: StepSoundCategory,
        asset_server: &AssetServer,
    ) -> Option<Handle<AudioSource>> {
        self.handles.entry(category).or_insert_with(|| {
            let paths: &[&str] = match category {
                StepSoundCategory::Walk(mat) => mat.walk_sounds(),
                StepSoundCategory::Run(mat) => mat.run_sounds(),
                StepSoundCategory::Land(mat) => mat.land_sounds(),
                StepSoundCategory::WaterWade => SoundMaterial::Water.walk_sounds(),
                StepSoundCategory::Swim => &[
                    "sounds/liquids/swim1.ogg",
                    "sounds/liquids/swim2.ogg",
                    "sounds/liquids/swim3.ogg",
                    "sounds/liquids/swim4.ogg",
                    "sounds/liquids/swim5.ogg",
                    "sounds/liquids/swim6.ogg",
                    "sounds/liquids/swim7.ogg",
                    "sounds/liquids/swim8.ogg",
                    "sounds/liquids/swim9.ogg",
                    "sounds/liquids/swim10.ogg",
                    "sounds/liquids/swim11.ogg",
                    "sounds/liquids/swim12.ogg",
                    "sounds/liquids/swim13.ogg",
                    "sounds/liquids/swim14.ogg",
                    "sounds/liquids/swim15.ogg",
                    "sounds/liquids/swim16.ogg",
                    "sounds/liquids/swim17.ogg",
                    "sounds/liquids/swim18.ogg",
                ],
                StepSoundCategory::SplashEnter => &["sounds/liquids/splash.ogg"],
                StepSoundCategory::SplashExit => &["sounds/liquids/splash2.ogg"],
            };
            paths.iter().map(|p| asset_server.load(*p)).collect()
        });

        let rand = self.next_float();
        let list = self.handles.get(&category)?;
        if list.is_empty() {
            None
        } else {
            let idx = (rand * list.len() as f32) as usize % list.len();
            Some(list[idx].clone())
        }
    }
}

pub struct FootstepAudioPlugin;

impl Plugin for FootstepAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FootstepAudioState>()
            .add_systems(Update, update_player_footstep_audio);
    }
}

fn detect_floor_voxel(world: &VoxelWorld, player_pos: Vec3) -> Voxel {
    let probe_pos = player_pos - Vec3::new(0.0, 0.15, 0.0);
    let mut block_pos = (probe_pos / VOXEL_SIZE).floor().as_ivec3();

    if let Some(v) = world.get_voxel(block_pos) {
        if !v.is_empty() && v.is_collidable() {
            return v;
        }
    }

    // Check one block lower in case player is standing on half-slab or top edge
    block_pos.y -= 1;
    if let Some(v) = world.get_voxel(block_pos) {
        if !v.is_empty() && v.is_collidable() {
            return v;
        }
    }

    Voxel::LoamGrass
}

#[inline]
fn dust_color_for_voxel(v: Voxel) -> [f32; 3] {
    let c = v.fallback_color();
    let r = c[0] as f32 / 255.0;
    let g = c[1] as f32 / 255.0;
    let b = c[2] as f32 / 255.0;
    [
        r * 0.65 + 0.86 * 0.35,
        g * 0.65 + 0.83 * 0.35,
        b * 0.65 + 0.77 * 0.35,
    ]
}

fn update_player_footstep_audio(
    time: Res<Time>,
    world_res: Option<Res<VoxelWorld>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    player_query: Query<(&Transform, &PlayerMotion), With<Player>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<FootstepAudioState>,
    mut particle_bursts: Option<ResMut<WorldParticleBurstQueue>>,
    mut commands: Commands,
) {
    let Ok((transform, motion)) = player_query.single() else {
        return;
    };
    let Some(world) = world_res else {
        return;
    };

    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let submersion = env_status.as_ref().map_or(0.0, |s| s.submersion);
    let is_cam_water = env_status.as_ref().map_or(false, |s| s.is_camera_in_water);
    let water_tint_raw = env_status
        .as_ref()
        .map_or([0.32, 0.62, 0.88, 1.0], |s| s.current_biome.water_color());
    let water_tint = [water_tint_raw[0], water_tint_raw[1], water_tint_raw[2]];
    let in_water = submersion > 0.08 || is_cam_water;

    // 1. Water entry & exit splash detection
    if in_water != state.was_in_water {
        let category = if in_water {
            if let Some(ref mut bursts) = particle_bursts {
                let intensity = ((-state.prev_velocity_y).clamp(1.2, 6.0) / 3.0).max(0.6);
                bursts.push_splash(
                    transform.translation + Vec3::new(0.0, 0.25, 0.0),
                    intensity,
                    water_tint,
                );
            }
            StepSoundCategory::SplashEnter
        } else {
            StepSoundCategory::SplashExit
        };
        let rand_pitch = 0.94 + state.next_float() * 0.12;
        if let Some(sound) = state.get_sound(category, &asset_server) {
            commands.trigger(PlaySfxEvent::new(sound, 0.58).with_speed(rand_pitch));
        }
        state.was_in_water = in_water;
    }

    // 2. Landing detection
    if !state.was_grounded && motion.grounded && state.prev_velocity_y < -2.2 {
        let floor_voxel = detect_floor_voxel(&world, transform.translation);
        let mat = SoundMaterial::from_voxel(floor_voxel);
        let fall_severity = ((-state.prev_velocity_y - 2.2) / 8.0).clamp(0.4, 1.0);
        let category = if in_water {
            if let Some(ref mut bursts) = particle_bursts {
                bursts.push_splash(
                    transform.translation + Vec3::new(0.0, 0.20, 0.0),
                    fall_severity * 1.5,
                    water_tint,
                );
            }
            StepSoundCategory::SplashEnter
        } else {
            if let Some(ref mut bursts) = particle_bursts {
                bursts.push_dust(
                    transform.translation,
                    (6.0 + fall_severity * 5.0).round() as u8,
                    dust_color_for_voxel(floor_voxel),
                    0.42,
                );
            }
            StepSoundCategory::Land(mat)
        };

        let rand_pitch = 0.92 + state.next_float() * 0.16;

        if let Some(sound) = state.get_sound(category, &asset_server) {
            commands.trigger(
                PlaySfxEvent::new(sound, 0.44 * fall_severity).with_speed(rand_pitch),
            );
        }
    }

    state.was_grounded = motion.grounded;
    state.prev_velocity_y = motion.velocity.y;

    // 3. Movement audio: ground walking/running or water swimming/wading
    let horiz_velocity = Vec2::new(motion.velocity.x, motion.velocity.z);
    let horiz_speed = horiz_velocity.length();

    if in_water && submersion > 0.35 && !motion.grounded {
        // Swimming audio
        let move_speed = motion.velocity.length();
        if move_speed > 0.6 {
            state.swim_timer += dt;
            if state.swim_timer >= 0.82 {
                state.swim_timer = 0.0;
                let rand_pitch = 0.93 + state.next_float() * 0.14;
                if let Some(sound) = state.get_sound(StepSoundCategory::Swim, &asset_server) {
                    commands.trigger(PlaySfxEvent::new(sound, 0.44).with_speed(rand_pitch));
                }
            }
        } else {
            state.swim_timer = 0.70; // Ready for immediate stroke when swimming starts
        }
        state.distance_accum = 0.0;
    } else if motion.grounded && !motion.flying && horiz_speed > 0.35 {
        // Grounded movement: calculate step cadence interval
        let is_running = horiz_speed > 5.8;
        let is_crouching = motion.stance == PlayerStance::Crouching;
        let is_crawling = motion.stance == PlayerStance::Crawling;

        let step_distance = if is_running {
            2.25
        } else if is_crouching {
            1.85
        } else if is_crawling {
            1.30
        } else {
            2.55
        };

        state.distance_accum += horiz_speed * dt;

        if state.distance_accum >= step_distance {
            state.distance_accum = 0.0;

            let rand_pitch = 0.94 + state.next_float() * 0.12;

            if in_water && submersion > 0.05 {
                // Wading in shallow water
                if let Some(ref mut bursts) = particle_bursts {
                    bursts.push_splash(
                        transform.translation + Vec3::new(0.0, 0.15, 0.0),
                        0.45,
                        water_tint,
                    );
                }
                if let Some(sound) = state.get_sound(StepSoundCategory::WaterWade, &asset_server) {
                    commands.trigger(PlaySfxEvent::new(sound, 0.48).with_speed(rand_pitch));
                }
            } else {
                let floor_voxel = detect_floor_voxel(&world, transform.translation);
                if !is_crouching && !is_crawling
                    && let Some(ref mut bursts) = particle_bursts
                {
                    bursts.push_dust(
                        transform.translation,
                        if is_running { 4 } else { 2 },
                        dust_color_for_voxel(floor_voxel),
                        if is_running { 0.28 } else { 0.16 },
                    );
                }
                let mat = SoundMaterial::from_voxel(floor_voxel);
                let category = if is_running {
                    StepSoundCategory::Run(mat)
                } else {
                    StepSoundCategory::Walk(mat)
                };

                // Attenuated ~32% for comfortable auditory mix
                let step_volume = if is_crouching || is_crawling {
                    0.20
                } else if is_running {
                    0.54
                } else {
                    0.44
                };

                if let Some(sound) = state.get_sound(category, &asset_server) {
                    commands.trigger(PlaySfxEvent::new(sound, step_volume).with_speed(rand_pitch));
                }
            }
        }
    } else {
        // Decay distance accumulator slightly when stopped
        state.distance_accum = (state.distance_accum - dt * 2.0).max(0.0);
    }
}
