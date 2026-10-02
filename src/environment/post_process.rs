use std::borrow::Cow;

use bevy::{
    core_pipeline::{
        FullscreenShader,
        schedule::{Core3d, Core3dSystems},
        tonemapping::tonemapping,
    },
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        texture::TextureCache,
        view::{ExtractedView, ViewTarget},
    },
};

use crate::{
    environment::{
        celestial::{CELESTIAL_DISTANCE, calculate_sun_direction},
        time::EnvironmentState,
    },
    player::{InspectorInteraction, PlayerCamera},
};

pub const POST_PROCESS_SHADER_PATH: &str = "shaders/post_process.wgsl";

/// Uniform settings for screen-space crepuscular rays (god rays) and lens flare supporting Sun and Moon.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Reflect, Debug)]
pub struct PostProcessSettings {
    pub light_ndc_position: Vec2,
    pub light_screen_uv: Vec2,
    pub ray_tint: Vec3,
    pub aspect_ratio: f32,
    pub exposure: f32,
    pub decay: f32,
    pub density: f32,
    pub weight: f32,
    pub light_visible: f32,
    pub threshold: f32,
    pub max_radius: f32,
    pub padding: f32,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            light_ndc_position: Vec2::ZERO,
            light_screen_uv: Vec2::new(0.5, 0.5),
            ray_tint: Vec3::new(1.04, 0.92, 0.76),
            aspect_ratio: 16.0 / 9.0,
            exposure: 0.45,
            decay: 0.985,
            density: 0.95,
            weight: 0.04,
            light_visible: 1.0,
            threshold: 1.95,
            max_radius: 0.22,
            padding: 0.0,
        }
    }
}

pub struct PostProcessPlugin;

impl Plugin for PostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<PostProcessSettings>::default(),
            UniformComponentPlugin::<PostProcessSettings>::default(),
        ))
        .add_systems(
            Update,
            update_post_process_light_position.after(crate::player::PlayerSet::Movement),
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_god_rays_pipeline)
            .add_systems(
                Core3d,
                post_process_system
                    .in_set(Core3dSystems::PostProcess)
                    .before(tonemapping),
            );
    }
}

/// Pipeline resource storing layouts, samplers, and pipeline IDs for the two-pass downsampling architecture.
#[derive(Resource)]
pub struct GodRaysPipelines {
    scatter_layout: BindGroupLayoutDescriptor,
    composite_layout: BindGroupLayoutDescriptor,
    linear_sampler: Sampler,
    scatter_pipeline_id: CachedRenderPipelineId,
    composite_hdr_pipeline_id: CachedRenderPipelineId,
    composite_srgb_pipeline_id: CachedRenderPipelineId,
    composite_bgra_pipeline_id: CachedRenderPipelineId,
}

fn init_god_rays_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    // 1. Pass 1 Bind Group Layout: Scatter into downscaled buffer
    let scatter_layout = BindGroupLayoutDescriptor::new(
        "god_rays_scatter_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<PostProcessSettings>(true),
            ),
        ),
    );

    // 2. Pass 2 Bind Group Layout: High-res composite over native screen texture
    let composite_layout = BindGroupLayoutDescriptor::new(
        "god_rays_composite_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<PostProcessSettings>(true),
            ),
        ),
    );

    let linear_sampler = render_device.create_sampler(&SamplerDescriptor {
        label: Some("god_rays_linear_sampler"),
        min_filter: FilterMode::Linear,
        mag_filter: FilterMode::Linear,
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        ..default()
    });

    let shader = asset_server.load(POST_PROCESS_SHADER_PATH);

    // Pass 1 pipeline: renders scattering into half-resolution Rgba16Float intermediate texture
    let scatter_pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("god_rays_scatter_pipeline".into()),
        layout: vec![scatter_layout.clone()],
        vertex: fullscreen_shader.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: shader.clone(),
            entry_point: Some(Cow::Borrowed("fragment_scatter")),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba16Float,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    // Pass 2 pipelines: composite over native destination (specialized for target texture formats)
    let create_composite_pipeline = |format: TextureFormat, label: &'static str| {
        pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(label.into()),
            layout: vec![composite_layout.clone()],
            vertex: fullscreen_shader.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: shader.clone(),
                entry_point: Some(Cow::Borrowed("fragment_composite")),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        })
    };

    let composite_hdr_pipeline_id =
        create_composite_pipeline(TextureFormat::Rgba16Float, "god_rays_composite_hdr");
    let composite_srgb_pipeline_id =
        create_composite_pipeline(TextureFormat::Rgba8UnormSrgb, "god_rays_composite_srgb");
    let composite_bgra_pipeline_id =
        create_composite_pipeline(TextureFormat::Bgra8UnormSrgb, "god_rays_composite_bgra");

    commands.insert_resource(GodRaysPipelines {
        scatter_layout,
        composite_layout,
        linear_sampler,
        scatter_pipeline_id,
        composite_hdr_pipeline_id,
        composite_srgb_pipeline_id,
        composite_bgra_pipeline_id,
    });
}

fn post_process_system(
    view: ViewQuery<(
        &ViewTarget,
        &ExtractedView,
        &PostProcessSettings,
        &DynamicUniformIndex<PostProcessSettings>,
    )>,
    pipelines: Option<Res<GodRaysPipelines>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<PostProcessSettings>>,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    mut ctx: RenderContext,
) {
    let Some(pipelines) = pipelines else {
        return;
    };

    let (view_target, extracted_view, settings, settings_index) = view.into_inner();

    // Early exit if rays are faded to near-zero (night without moon, looking away, or offscreen)
    if settings.light_visible < 0.05 || settings.exposure <= 0.001 {
        return;
    }

    let Some(scatter_pipeline) = pipeline_cache.get_render_pipeline(pipelines.scatter_pipeline_id)
    else {
        return;
    };

    let composite_pipeline_id = match extracted_view.target_format {
        TextureFormat::Rgba16Float => pipelines.composite_hdr_pipeline_id,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm => {
            pipelines.composite_srgb_pipeline_id
        }
        TextureFormat::Bgra8UnormSrgb | TextureFormat::Bgra8Unorm => {
            pipelines.composite_bgra_pipeline_id
        }
        _ => pipelines.composite_hdr_pipeline_id,
    };

    let Some(composite_pipeline) = pipeline_cache.get_render_pipeline(composite_pipeline_id) else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    // 1. Allocate / fetch half-resolution intermediate texture from TextureCache
    let main_size = view_target.main_texture().size();
    let downscaled_width = (main_size.width / 2).max(1);
    let downscaled_height = (main_size.height / 2).max(1);

    let downscaled_texture = texture_cache.get(
        &render_device,
        TextureDescriptor {
            label: Some("god_rays_downscaled_texture"),
            size: Extent3d {
                width: downscaled_width,
                height: downscaled_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
    );

    // Flip view target to advance ping-pong state and obtain source/destination
    let post_process = view_target.post_process_write();

    // 2. Pass 1: Volumetric light scattering into downscaled buffer (half-resolution)
    let scatter_bind_group = ctx.render_device().create_bind_group(
        "god_rays_scatter_bind_group",
        &pipeline_cache.get_bind_group_layout(&pipelines.scatter_layout),
        &BindGroupEntries::sequential((
            post_process.source,
            &pipelines.linear_sampler,
            settings_binding.clone(),
        )),
    );

    {
        let mut scatter_pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("god_rays_scatter_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &downscaled_texture.default_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        scatter_pass.set_pipeline(scatter_pipeline);
        scatter_pass.set_bind_group(0, &scatter_bind_group, &[settings_index.index()]);
        scatter_pass.draw(0..3, 0..1);
    }

    // 3. Pass 2: High-resolution native additive composite with bilinear upsampling
    let composite_bind_group = ctx.render_device().create_bind_group(
        "god_rays_composite_bind_group",
        &pipeline_cache.get_bind_group_layout(&pipelines.composite_layout),
        &BindGroupEntries::sequential((
            post_process.source,
            &pipelines.linear_sampler,
            &downscaled_texture.default_view,
            &pipelines.linear_sampler,
            settings_binding,
        )),
    );

    {
        let mut composite_pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("god_rays_composite_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: post_process.destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        composite_pass.set_pipeline(composite_pipeline);
        composite_pass.set_bind_group(0, &composite_bind_group, &[settings_index.index()]);
        composite_pass.draw(0..3, 0..1);
    }
}

/// Computes the celestial light (Sun or Moon) 2D screen/NDC projection every frame, with aspect ratio and radial edge fade.
pub fn update_post_process_light_position(
    state: Res<EnvironmentState>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    camera_query: Query<(&Camera, &GlobalTransform), (With<Camera3d>, With<PlayerCamera>)>,
    mut settings_query: Query<&mut PostProcessSettings, (With<Camera3d>, With<PlayerCamera>)>,
) {
    let Ok((camera, camera_global_transform)) = camera_query.single() else {
        return;
    };
    let Ok(mut settings) = settings_query.single_mut() else {
        return;
    };

    // If game is in a menu or the inspector is open, disable sun rays to prevent post-process flicker with UI
    let in_menu = menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None);
    let inspector_active = inspector.is_some_and(|i| i.active);
    if in_menu || inspector_active {
        settings.light_visible = 0.0;
        settings.exposure = 0.0;
        return;
    }

    // 1. Calculate dynamic aspect ratio from camera viewport to correct circular flares
    let aspect_ratio = camera
        .logical_viewport_size()
        .map(|s| if s.y > 0.0 { s.x / s.y } else { 16.0 / 9.0 })
        .unwrap_or(16.0 / 9.0);
    settings.aspect_ratio = aspect_ratio;

    let camera_translation = camera_global_transform.translation();
    let sun_dir = calculate_sun_direction(state.time_of_day);
    let moon_dir = -sun_dir;

    // 2. Dual celestial body targeting: Sun during day/twilight, Moon at night
    let (target_dir, base_exposure, target_tint, target_threshold, target_max_radius) =
        if sun_dir.y > -0.05 {
            // Sun is active (HDR value ~2.2; threshold 1.95 filters out specular water reflections; radius 0.22 isolates celestial disc)
            let day_factor = (sun_dir.y / 0.15).clamp(0.0, 1.0);
            (
                sun_dir,
                0.45 * day_factor,
                Vec3::new(1.04, 0.92, 0.76),
                1.95,
                0.22,
            )
        } else if moon_dir.y > 0.0 {
            // Sun has set and Moon is active (HDR value ~1.35; threshold 1.20 filters out night terrain; radius 0.18 isolates moon disc)
            let moon_elevation_factor = (moon_dir.y / 0.15).clamp(0.0, 1.0);
            let phase_factor = state.moon_phase().illuminance_factor();
            // Subtle, ethereal moonlight rays
            let moon_exposure = 0.10 * moon_elevation_factor * phase_factor;
            (
                moon_dir,
                moon_exposure,
                Vec3::new(0.45, 0.55, 0.85),
                1.20,
                0.18,
            )
        } else {
            settings.light_visible = 0.0;
            settings.exposure = 0.0;
            return;
        };

    settings.ray_tint = target_tint;
    settings.threshold = target_threshold;
    settings.max_radius = target_max_radius;

    // 3. Frustum and projection check
    let forward = camera_global_transform.forward();
    let to_light = target_dir.normalize();
    let dot_forward = forward.dot(to_light);

    if dot_forward > 0.0 {
        let light_world_pos = camera_translation + target_dir * CELESTIAL_DISTANCE;
        if let Some(ndc) = camera.world_to_ndc(camera_global_transform, light_world_pos) {
            let ndc_2d = Vec2::new(ndc.x, ndc.y);
            settings.light_ndc_position = ndc_2d;

            // Convert NDC [-1, 1] to Screen UV [0, 1]:
            // Bevy NDC has -1 at bottom and +1 at top; UV has 0 at top and 1 at bottom.
            settings.light_screen_uv = Vec2::new(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);

            // 4. Smooth radial fade from screen center:
            // Calculate distance from dead center (0.0, 0.0) in NDC space.
            // Tightened radial fade: starts fading when ndc_dist > 0.8 and reaches 0.0 at ndc_dist >= 1.2,
            // completely masking peripheral projection warping.
            let ndc_dist = ndc_2d.length();
            let radial_fade = (1.0 - ((ndc_dist - 0.8) / 0.4)).clamp(0.0, 1.0);
            let smooth_radial_fade = radial_fade * radial_fade * (3.0 - 2.0 * radial_fade);

            let final_exposure = base_exposure * smooth_radial_fade;
            settings.exposure = final_exposure;
            settings.light_visible = if final_exposure > 0.002 { 1.0 } else { 0.0 };
        } else {
            settings.light_visible = 0.0;
            settings.exposure = 0.0;
        }
    } else {
        settings.light_visible = 0.0;
        settings.exposure = 0.0;
    }
}
