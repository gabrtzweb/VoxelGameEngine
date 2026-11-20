use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::uniforms::FoliageAnimationState;
use crate::player::PlayerCamera;

pub const TRAIL_GRID_SIZE: usize = 48;
pub const TRAIL_WORLD_SIZE: f32 = 48.0;

/// Dynamic 48x48 footprint grid tracking player & entity movement over grass.
#[derive(Resource)]
pub struct GrassTrailField {
    pub image_handle: Handle<Image>,
    pub origin_world_xz: Vec2,
    pub pixels: Vec<u8>,
    pub last_player_pos: Vec3,
    pub upload_timer: f32,
    pub active_pixels: usize,
}

impl GrassTrailField {
    pub fn new(images: &mut Assets<Image>) -> Self {
        let pixels = vec![0u8; TRAIL_GRID_SIZE * TRAIL_GRID_SIZE * 4];
        let mut image = Image::new(
            Extent3d {
                width: TRAIL_GRID_SIZE as u32,
                height: TRAIL_GRID_SIZE as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels.clone(),
            TextureFormat::Rgba8Unorm,
            default(),
        );
        image.sampler = bevy::image::ImageSampler::linear();
        let image_handle = images.add(image);

        Self {
            image_handle,
            origin_world_xz: Vec2::ZERO,
            pixels,
            last_player_pos: Vec3::ZERO,
            upload_timer: 0.0,
            active_pixels: 0,
        }
    }

    pub fn decay(&mut self, dt: f32) {
        if self.active_pixels == 0 {
            return;
        }
        let decay_mult = (-dt * 1.5).exp();
        let mut active = 0usize;
        for pixel in self.pixels.chunks_exact_mut(4) {
            if pixel[2] == 0 {
                continue;
            }
            let strength = pixel[2] as f32 * decay_mult;
            pixel[2] = strength.round() as u8;
            let alpha = pixel[3] as f32 * decay_mult;
            pixel[3] = alpha.round() as u8;
            if pixel[2] == 0 {
                pixel[0] = 128;
                pixel[1] = 128;
                pixel[3] = 0;
            } else {
                active += 1;
            }
        }
        self.active_pixels = active;
    }

    pub fn stamp_entity(&mut self, entity_pos: Vec3, move_delta: Vec3, radius_m: f32) {
        let move_dir = Vec2::new(move_delta.x, move_delta.z).normalize_or_zero();
        let enc_x = ((move_dir.x * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
        let enc_z = ((move_dir.y * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;

        let center_x = (entity_pos.x - self.origin_world_xz.x) / TRAIL_WORLD_SIZE * (TRAIL_GRID_SIZE as f32);
        let center_z = (entity_pos.z - self.origin_world_xz.y) / TRAIL_WORLD_SIZE * (TRAIL_GRID_SIZE as f32);
        let radius = radius_m * (TRAIL_GRID_SIZE as f32 / TRAIL_WORLD_SIZE);

        let min_x = (center_x - radius).floor().max(0.0) as usize;
        let max_x = (center_x + radius).ceil().min((TRAIL_GRID_SIZE - 1) as f32) as usize;
        let min_z = (center_z - radius).floor().max(0.0) as usize;
        let max_z = (center_z + radius).ceil().min((TRAIL_GRID_SIZE - 1) as f32) as usize;

        for gz in min_z..=max_z {
            for gx in min_x..=max_x {
                let dx = gx as f32 - center_x;
                let dz = gz as f32 - center_z;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist <= radius {
                    let intensity = ((1.0 - (dist / radius)) * 255.0).clamp(0.0, 255.0) as u8;
                    let idx = (gz * TRAIL_GRID_SIZE + gx) * 4;
                    if intensity > self.pixels[idx + 2] {
                        if self.pixels[idx + 2] == 0 && intensity > 0 {
                            self.active_pixels += 1;
                        }
                        self.pixels[idx] = enc_x;
                        self.pixels[idx + 1] = enc_z;
                        self.pixels[idx + 2] = intensity;
                        self.pixels[idx + 3] = 255;
                    }
                }
            }
        }
    }
}

pub fn update_grass_trail_field(
    time: Res<Time>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut trail: ResMut<GrassTrailField>,
    mut images: ResMut<Assets<Image>>,
    mut anim_state: ResMut<FoliageAnimationState>,
) {
    let dt = time.delta_secs();
    let player_pos = camera_query
        .iter()
        .next()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    // Recenter origin around player camera
    let half_size = TRAIL_WORLD_SIZE * 0.5;
    trail.origin_world_xz = Vec2::new(player_pos.x - half_size, player_pos.z - half_size);

    // Decay existing footprint impression (recovery rate ~2.0 seconds)
    trail.decay(dt);

    // Measure player movement velocity
    let move_delta = player_pos - trail.last_player_pos;
    let horizontal_speed = (move_delta.x * move_delta.x + move_delta.z * move_delta.z).sqrt() / dt.max(0.001);
    trail.last_player_pos = player_pos;

    let had_active = trail.active_pixels > 0;
    // If player is moving, stamp directional footprint trail; if standing, bend grass under body
    if horizontal_speed > 0.05 {
        trail.stamp_entity(player_pos, move_delta, 1.25);
    } else if had_active || dt > 0.0 {
        trail.stamp_entity(player_pos, Vec3::new(0.001, 0.0, 0.001), 0.75);
    }

    // Throttle GPU image upload to ~25 Hz and copy in place without cloning Vec when active
    trail.upload_timer += dt;
    if (horizontal_speed > 0.05 || trail.active_pixels > 0 || had_active) && trail.upload_timer >= 0.04 {
        trail.upload_timer = 0.0;
        if let Some(mut image) = images.get_mut(&trail.image_handle) {
            if let Some(ref mut data) = image.data
                && data.len() == trail.pixels.len()
            {
                data.copy_from_slice(&trail.pixels);
            } else {
                image.data = Some(trail.pixels.clone());
            }
        }
    }

    // Update uniform pusher
    anim_state.gpu_uniforms.trail_params2 = Vec4::new(
        trail.origin_world_xz.x % 32.0,
        trail.origin_world_xz.y % 32.0,
        0.0,
        0.0,
    );
    anim_state.gpu_uniforms.entity_pushers[0] = Vec4::new(
        trail.origin_world_xz.x,
        trail.origin_world_xz.y,
        1.0 / TRAIL_WORLD_SIZE,
        1.0, // active trail strength
    );
}
