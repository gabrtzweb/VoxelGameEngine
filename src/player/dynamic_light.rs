use bevy::prelude::*;
use crate::{
    core::config::LocalConfig,
    gameplay::SelectedVoxel,
    player::{Player, PlayerCamera},
};

#[derive(Component)]
pub struct HandheldDynamicLight;

pub fn update_player_dynamic_light(
    mut commands: Commands,
    local_config: Option<Res<LocalConfig>>,
    selected_voxel: Option<Res<SelectedVoxel>>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, With<PlayerCamera>>,
    mut light_query: Query<
        (Entity, &mut PointLight, &mut Transform),
        (With<HandheldDynamicLight>, Without<Player>, Without<PlayerCamera>),
    >,
) {
    let dynamic_enabled = local_config.map_or(true, |c| c.dynamic_lighting);
    let held_voxel = if dynamic_enabled {
        selected_voxel
            .and_then(|s| s.0)
            .filter(|v| v.is_light() || v.emission_level() > 0)
    } else {
        None
    };

    if let Some(voxel) = held_voxel {
        // Calculate emission factor and light parameters
        let emission = (voxel.emission_level().max(1) as f32) / 15.0;
        let color = voxel.light_color();
        let intensity = 400_000.0 * emission;
        let range = 18.0 * emission.max(0.65);

        // Position light near player's hand/chest in world space
        let light_pos = if let Ok(cam_tf) = camera_query.single() {
            cam_tf.translation
                + cam_tf.forward() * 0.45
                + cam_tf.right() * 0.35
                + Vec3::new(0.0, -0.25, 0.0)
        } else if let Ok(player_tf) = player_query.single() {
            player_tf.translation + Vec3::new(0.3, 1.2, 0.2)
        } else {
            return;
        };

        if let Ok((_entity, mut light, mut transform)) = light_query.single_mut() {
            // Smoothly track position and update light properties
            transform.translation = transform.translation.lerp(light_pos, 0.5);
            light.color = color;
            light.intensity = intensity;
            light.range = range;
        } else {
            // Spawn handheld dynamic light
            commands.spawn((
                PointLight {
                    color,
                    intensity,
                    range,
                    radius: 0.15,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_translation(light_pos),
                HandheldDynamicLight,
            ));
        }
    } else {
        // Despawn light when not holding a light-emitting item
        for (entity, _, _) in light_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
