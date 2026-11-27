import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

struct PostProcessSettings {
    light_ndc_position: vec2<f32>,
    light_screen_uv: vec2<f32>,
    ray_tint: vec3<f32>,
    aspect_ratio: f32,
    exposure: f32,
    decay: f32,
    density: f32,
    weight: f32,
    light_visible: f32,
    threshold: f32,
    max_radius: f32,
    padding: f32,
    underwater_params: vec4<f32>,
};

// Interleaved Gradient Noise (IGN) by Jorge Jimenez
// Produces high-frequency triangular dither per pixel to turn radial banding into cinematic grain
fn interleaved_gradient_noise(pixel_pos: vec2<f32>) -> f32 {
    let magic = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
    return fract(magic.z * fract(dot(pixel_pos, magic.xy)));
}

// ==============================================================================
// 1. Pass 1: Downscaled Volumetric Light Scattering (God Rays & Lens Flare)
// ==============================================================================

@group(0) @binding(0) var scatter_input_texture: texture_2d<f32>;
@group(0) @binding(1) var scatter_input_sampler: sampler;
@group(0) @binding(2) var<uniform> scatter_settings: PostProcessSettings;

@fragment
fn fragment_scatter(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let light_uv = scatter_settings.light_screen_uv;
    let is_underwater = scatter_settings.underwater_params.x > 0.5;

    // Safety guard: skip work if celestial light is not active or exposure faded to zero
    if (scatter_settings.light_visible <= 0.0 || scatter_settings.exposure <= 0.0001) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Vector from current pixel towards the celestial light source (Sun or Moon)
    var delta_uv = light_uv - in.uv;

    // High quality ray marching: 24 samples with Interleaved Gradient Noise jitter
    let NUM_SAMPLES: i32 = 24;
    delta_uv = delta_uv * (scatter_settings.density / f32(NUM_SAMPLES));

    // Interleaved Gradient Noise jitter: staggers the starting ray offset per pixel
    let jitter = interleaved_gradient_noise(in.position.xy);
    var sample_uv = in.uv + delta_uv * jitter;

    var illumination_decay: f32 = 1.0;
    var accumulated_rays: vec3<f32> = vec3<f32>(0.0);

    for (var i: i32 = 0; i < NUM_SAMPLES; i = i + 1) {
        sample_uv = sample_uv + delta_uv;

        // Clip to valid texture coordinate space
        if (sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0) {
            break;
        }

        // Distance from current sample to celestial body center (aspect-ratio corrected)
        let sample_aspect_delta = vec2<f32>((sample_uv.x - light_uv.x) * scatter_settings.aspect_ratio, sample_uv.y - light_uv.y);
        let sample_dist = length(sample_aspect_delta);

        // Spatial Emitter Mask:
        // Above water: strictly restrict to the sun disc & tight lower rim so water glints never shoot a laser beam.
        // Underwater: allow full radial scattering from the sunlit water surface / Snell's window around the sun.
        let radial_mask = smoothstep(scatter_settings.max_radius, scatter_settings.max_radius * 0.50, sample_dist);
        let vertical_mask = select(
            1.0 - smoothstep(0.015, 0.048, max(0.0, sample_uv.y - light_uv.y)),
            1.0,
            is_underwater
        );
        let emitter_mask = radial_mask * vertical_mask;

        if (emitter_mask > 0.001) {
            // Subtle spectral dispersion across RGB taps for cinematic atmospheric light
            let sample_r = textureSample(scatter_input_texture, scatter_input_sampler, sample_uv + delta_uv * 0.0018);
            let sample_g = textureSample(scatter_input_texture, scatter_input_sampler, sample_uv);
            let sample_b = textureSample(scatter_input_texture, scatter_input_sampler, sample_uv - delta_uv * 0.0018);
            let sample_rgb = vec3<f32>(sample_r.r, sample_g.g, sample_b.b);

            // Smooth quadratic knee thresholding: eliminates harsh cutoff flickering across threshold boundaries
            let luma = dot(sample_rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
            let knee: f32 = 0.35;
            let diff = luma - scatter_settings.threshold;
            var excess: f32 = 0.0;
            if (diff > knee) {
                excess = diff;
            } else if (diff > -knee) {
                let soft = diff + knee;
                excess = (soft * soft) / (4.0 * knee);
            }
            excess = excess * emitter_mask;

            let light_sample = sample_rgb * excess;
            accumulated_rays = accumulated_rays + light_sample * illumination_decay * scatter_settings.weight;
        }

        // Exponential decay per step
        illumination_decay = illumination_decay * scatter_settings.decay;
    }

    // Aspect-Ratio-Corrected distance for perfectly circular lens flare & corona halo
    let aspect_delta = vec2<f32>((in.uv.x - light_uv.x) * scatter_settings.aspect_ratio, in.uv.y - light_uv.y);
    let dist_to_light = length(aspect_delta);

    // Forward Mie scattering phase function approximation (Henyey-Greenstein)
    let cos_theta = clamp(1.0 - dist_to_light * 0.85, -1.0, 1.0);
    let g: f32 = 0.60;
    let mie_phase = (1.0 - g * g) / pow(max(0.01, 1.0 + g * g - 2.0 * g * cos_theta), 1.5);
    let phase_factor = clamp(mie_phase * 0.40, 0.60, 2.20);

    // Subtle Lens Flare & Celestial Corona Halo (perfectly circular isotropic glow)
    // Physically occluded if the celestial disc center is blocked by geometry in the scene buffer
    let clamped_uv = clamp(light_uv, vec2<f32>(0.001), vec2<f32>(0.999));
    let center_sample = textureSample(scatter_input_texture, scatter_input_sampler, clamped_uv);
    let center_luma = dot(center_sample.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let flare_visibility = smoothstep(scatter_settings.threshold * 0.75, scatter_settings.threshold, center_luma);
    let flare_halo = exp(-dist_to_light * 4.2) * 0.20 * flare_visibility;

    // Atmospheric distance attenuation: smoothly dissolves rays as they travel away from the light source
    let dist_attenuation = smoothstep(1.6, 0.0, dist_to_light);
    let soft_rays = accumulated_rays * dist_attenuation * phase_factor;

    // Dynamic tinting (warm golden for Sun, cool silver/blue for Moon)
    let final_rays = (soft_rays + flare_halo * scatter_settings.ray_tint) * (scatter_settings.exposure * scatter_settings.ray_tint);

    return vec4<f32>(final_rays, 1.0);
}

// ==============================================================================
// 2. Pass 2: High-Resolution Native Additive Composite
// ==============================================================================

@group(0) @binding(0) var comp_main_texture: texture_2d<f32>;
@group(0) @binding(1) var comp_main_sampler: sampler;
@group(0) @binding(2) var comp_rays_texture: texture_2d<f32>;
@group(0) @binding(3) var comp_rays_sampler: sampler; // Bilinear filtering sampler for smooth upsample
@group(0) @binding(4) var<uniform> comp_settings: PostProcessSettings;

@fragment
fn fragment_composite(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uw_factor = comp_settings.underwater_params.x;

    // Balanced gentle underwater refraction shimmer (visibly aquatic yet comfortable for long dives)
    var sample_uv = in.uv;
    if (uw_factor > 0.001) {
        let t = comp_settings.underwater_params.y;
        let wobble_x = sin(in.uv.y * 15.0 + t * 1.35) * 0.00088 + cos(in.uv.y * 8.0 - t * 0.95) * 0.00042;
        let wobble_y = cos(in.uv.x * 12.0 + t * 1.20) * 0.00072;
        sample_uv = clamp(in.uv + vec2<f32>(wobble_x, wobble_y) * uw_factor, vec2<f32>(0.001), vec2<f32>(0.999));
    }

    let main_color = textureSample(comp_main_texture, comp_main_sampler, sample_uv);

    if (uw_factor <= 0.001 && (comp_settings.light_visible <= 0.0 || comp_settings.exposure <= 0.0001)) {
        return main_color;
    }

    // 4-tap tent bilinear upsample filter:
    // Completely dissolves any dither grain into soft, continuous volumetric shafts
    let tex_size = vec2<f32>(textureDimensions(comp_rays_texture));
    let inv_size = 1.0 / tex_size;
    let offset = inv_size * 0.5;

    let r0 = textureSample(comp_rays_texture, comp_rays_sampler, sample_uv + vec2<f32>(-offset.x, -offset.y));
    let r1 = textureSample(comp_rays_texture, comp_rays_sampler, sample_uv + vec2<f32>(offset.x, -offset.y));
    let r2 = textureSample(comp_rays_texture, comp_rays_sampler, sample_uv + vec2<f32>(-offset.x, offset.y));
    let r3 = textureSample(comp_rays_texture, comp_rays_sampler, sample_uv + vec2<f32>(offset.x, offset.y));
    let rays_color = (r0 + r1 + r2 + r3) * 0.25;

    var final_rgb = main_color.rgb + rays_color.rgb;

    if (uw_factor > 0.001) {
        let depth_atten = comp_settings.underwater_params.w;

        // Subtle deep-water peripheral vignette (deepens slightly in abyssal trenches)
        let vig_dist = length((in.uv - vec2<f32>(0.5)) * vec2<f32>(1.15, 1.0));
        let vig_strength = mix(0.28, 0.16, depth_atten);
        let vignette = 1.0 - smoothstep(0.42, 0.92, vig_dist) * vig_strength * uw_factor;
        final_rgb *= vignette;
    }

    return vec4<f32>(final_rgb, main_color.a);
}
