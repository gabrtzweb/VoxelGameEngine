pub mod config;
pub mod generator;
pub mod material;
pub mod particles;
pub mod trail;
pub mod uniforms;

use bevy::prelude::*;

pub use config::{GrassConfig, GrassStyle};
pub use material::{
    GRASS_BLADES_SHADER_PATH, GRASS_PLANT_SHADER_PATH, GrassChunkMaterial, GrassMaterialHandle,
    GrassVoxelMaterial, setup_grass_materials, sync_grass_material_uniforms,
};
pub use trail::{GrassTrailField, update_grass_trail_field};
pub use uniforms::{FoliageAnimationState, GrassShaderUniformsGpu, update_foliage_uniforms};

pub struct FoliagePlugin;

impl Plugin for FoliagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GrassConfig>()
            .init_resource::<FoliageAnimationState>()
            .add_plugins(MaterialPlugin::<GrassChunkMaterial>::default())
            .add_systems(Startup, setup_grass_materials)
            .add_systems(
                Update,
                (
                    sync_local_config_to_grass_config,
                    update_foliage_uniforms,
                    update_grass_trail_field,
                    particles::spawn_wind_blade_particles,
                    particles::update_wind_blade_particles,
                ),
            )
            .add_systems(PostUpdate, sync_grass_material_uniforms);
    }
}

fn sync_local_config_to_grass_config(
    local_config: Option<Res<crate::core::config::LocalConfig>>,
    mut grass_config: ResMut<GrassConfig>,
) {
    let Some(local_config) = local_config else { return; };
    if local_config.is_changed() {
        grass_config.enabled = local_config.grass_enabled;
        grass_config.blades_per_block = local_config.grass_density;
        grass_config.blade_particles = local_config.grass_particles;
        grass_config.grass_style = match local_config.grass_style.as_str() {
            "segmented" | "Segmented" => GrassStyle::Segmented,
            _ => GrassStyle::Tapered,
        };
    }
}
