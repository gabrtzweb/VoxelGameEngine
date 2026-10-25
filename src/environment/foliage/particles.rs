use std::f32::consts::PI;

use bevy::prelude::*;

use super::config::GrassConfig;
use crate::{
    environment::weather::wind::WindState,
    player::PlayerCamera,
};

#[derive(Component)]
pub struct WindBladeParticle {
    pub velocity: Vec3,
    pub lifetime_timer: Timer,
    pub roll_speed: f32,
    pub initial_scale: f32,
}

#[derive(Resource, Default)]
pub struct WindBladeParticleTimer {
    pub timer: Timer,
}

pub fn spawn_wind_blade_particles(
    mut commands: Commands,
    time: Res<Time>,
    wind: Option<Res<WindState>>,
    config: Option<Res<GrassConfig>>,
    mut timer: Local<f32>,
    asset_server: Res<AssetServer>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
) {
    let default_cfg = GrassConfig::default();
    let cfg = config.as_ref().map(|c| &**c).unwrap_or(&default_cfg);

    if !cfg.enabled || !cfg.blade_particles {
        return;
    }

    let Some(w) = wind.as_ref() else {
        return;
    };

    // Only shed grass blades if wind is sufficiently strong (>= 4.5 m/s)
    let speed = w.current_speed;
    if speed < 4.5 {
        return;
    }

    *timer += time.delta_secs();
    let spawn_interval = (1.5 / (speed / 4.0)).clamp(0.15, 1.2);
    if *timer < spawn_interval {
        return;
    }
    *timer = 0.0;

    let Some(cam_transform) = camera_query.iter().next() else {
        return;
    };

    let cam_pos = cam_transform.translation;
    let wind_norm = w.current_wind.normalize_or_zero();

    // Spawn 1-3 particles in a semi-circle upwind of the camera
    let spawn_count = ((speed / 6.0).ceil() as usize).clamp(1, 3);
    let texture = asset_server.load("textures/foliage/wind_blade.png");

    for i in 0..spawn_count {
        let angle_jitter = ((i as f32) - 1.0) * 0.4 + (time.elapsed_secs() * 3.7 + i as f32).sin() * 0.3;
        let c = angle_jitter.cos();
        let s = angle_jitter.sin();
        let offset_dir = Vec3::new(
            -wind_norm.x * c - -wind_norm.z * s,
            0.0,
            -wind_norm.x * s + -wind_norm.z * c,
        );

        let dist = 8.0 + (i as f32 * 4.0);
        let spawn_pos = cam_pos + offset_dir * dist + Vec3::new(0.0, 1.5 + (i as f32 * 0.8), 0.0);

        let particle_vel = w.current_wind * 0.85 + Vec3::new(0.0, -0.4, 0.0);

        commands.spawn((
            Sprite {
                image: texture.clone(),
                custom_size: Some(Vec2::splat(0.25)),
                color: Color::srgba(0.85, 0.95, 0.8, 0.9),
                ..default()
            },
            Transform::from_translation(spawn_pos).with_rotation(Quat::from_rotation_z(raw_angle(i))),
            WindBladeParticle {
                velocity: particle_vel,
                lifetime_timer: Timer::from_seconds(2.5, TimerMode::Once),
                roll_speed: 2.5 + (i as f32 * 1.2),
                initial_scale: 0.25,
            },
        ));
    }
}

fn raw_angle(idx: usize) -> f32 {
    (idx as f32 * 1.5) % (2.0 * PI)
}

pub fn update_wind_blade_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut WindBladeParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime_timer.tick(time.delta());

        if particle.lifetime_timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += particle.velocity * dt;
        transform.rotate_z(particle.roll_speed * dt);

        let progress = particle.lifetime_timer.fraction();
        let alpha = if progress < 0.2 {
            progress / 0.2
        } else if progress > 0.7 {
            1.0 - (progress - 0.7) / 0.3
        } else {
            1.0
        };

        sprite.color.set_alpha(alpha * 0.9);
    }
}
