use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::*,
    shader::ShaderRef,
};

use super::{
    config::{GrassConfig, GrassStyle},
    trail::GrassTrailField,
    uniforms::{FoliageAnimationState, GrassShaderUniformsGpu},
};

pub const GRASS_BLADES_SHADER_PATH: &str = "shaders/grass_blades.wesl";
pub const GRASS_PLANT_SHADER_PATH: &str = "shaders/grass_plant.wesl";

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct GrassVoxelMaterial {
    #[uniform(100)]
    pub uniforms: GrassShaderUniformsGpu,
    #[texture(101)]
    #[sampler(102)]
    pub shape_texture: Handle<Image>,
    #[texture(103)]
    #[sampler(104)]
    pub noise_texture: Handle<Image>,
    #[texture(105)]
    #[sampler(106)]
    pub trail_texture: Handle<Image>,
}

impl MaterialExtension for GrassVoxelMaterial {
    fn vertex_shader() -> ShaderRef {
        GRASS_BLADES_SHADER_PATH.into()
    }

    fn deferred_vertex_shader() -> ShaderRef {
        GRASS_BLADES_SHADER_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        GRASS_BLADES_SHADER_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        GRASS_BLADES_SHADER_PATH.into()
    }

    // Grass deformation is simulated in forward vertex shader; disable default undeformed prepass and shadows
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }
}

pub type GrassChunkMaterial = ExtendedMaterial<StandardMaterial, GrassVoxelMaterial>;

/// Global handle storing the shared foliage material and loaded textures.
#[derive(Resource, Clone)]
pub struct GrassMaterialHandle {
    pub material: Handle<GrassChunkMaterial>,
    pub tapered_shape_texture: Handle<Image>,
    pub segmented_shape_texture: Handle<Image>,
    pub noise_texture: Handle<Image>,
    pub grainy_texture: Handle<Image>,
}

pub fn setup_grass_materials(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<GrassChunkMaterial>>,
    config: Option<Res<GrassConfig>>,
) {
    let tapered_shape_texture = asset_server.load("textures/foliage/grass_tapered.png");
    let segmented_shape_texture = asset_server.load("textures/foliage/grass_bands.png");
    let noise_texture = asset_server.load("textures/foliage/noise.png");
    let grainy_texture = asset_server.load("textures/foliage/grainy.png");

    let trail = GrassTrailField::new(&mut images);
    let trail_handle = trail.image_handle.clone();
    commands.insert_resource(trail);

    let default_cfg = GrassConfig::default();
    let cfg = config.as_ref().map(|c| &**c).unwrap_or(&default_cfg);

    let active_shape = if cfg.grass_style == GrassStyle::Segmented {
        segmented_shape_texture.clone()
    } else {
        tapered_shape_texture.clone()
    };

    let material_handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Mask(0.1),
            perceptual_roughness: 0.85,
            cull_mode: None, // Double sided grass quads
            double_sided: true,
            ..default()
        },
        extension: GrassVoxelMaterial {
            uniforms: GrassShaderUniformsGpu::default(),
            shape_texture: active_shape,
            noise_texture: noise_texture.clone(),
            trail_texture: trail_handle,
        },
    });

    commands.insert_resource(GrassMaterialHandle {
        material: material_handle,
        tapered_shape_texture,
        segmented_shape_texture,
        noise_texture,
        grainy_texture,
    });
}

pub fn sync_grass_material_uniforms(
    anim_state: Res<FoliageAnimationState>,
    grass_mat_handle: Option<Res<GrassMaterialHandle>>,
    mut materials: ResMut<Assets<GrassChunkMaterial>>,
    config: Option<Res<GrassConfig>>,
) {
    let (Some(handle), Some(cfg)) = (grass_mat_handle, config) else {
        return;
    };
    if let Some(mut mat) = materials.get_mut(&handle.material) {
        mat.extension.uniforms = anim_state.gpu_uniforms;

        let target_shape = if cfg.grass_style == GrassStyle::Segmented {
            &handle.segmented_shape_texture
        } else {
            &handle.tapered_shape_texture
        };

        if &mat.extension.shape_texture != target_shape {
            mat.extension.shape_texture = target_shape.clone();
        }
    }
}
