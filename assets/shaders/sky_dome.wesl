import bevy_pbr::render::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
};

struct SkyDomeUniform {
    up_color: vec4<f32>,
    middle_color: vec4<f32>,
    down_color: vec4<f32>,
    sunset_color: vec4<f32>,
    sun_dir_and_factor: vec4<f32>,
    params: vec4<f32>,
    glare_color: vec4<f32>,
    rainbow_params: vec4<f32>,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sky: SkyDomeUniform;

fn ign_dither(pixel_pos: vec2<f32>) -> f32 {
    let magic = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
    return fract(magic.z * fract(dot(pixel_pos, magic.xy)));
}

fn aurora_hash(p: vec2<f32>) -> f32 {
    let p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    let p4 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p4.x + p4.y) * p4.z);
}

fn aurora_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(aurora_hash(i + vec2<f32>(0.0, 0.0)), aurora_hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(aurora_hash(i + vec2<f32>(0.0, 1.0)), aurora_hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_position.xyz - view.world_position);
    let v_dot_u = dir.y;
    let sun_dir = normalize(sky.sun_dir_and_factor.xyz);
    let sun_factor = sky.sun_dir_and_factor.w;
    let v_dot_s = dot(dir, sun_dir);

    let sunset_strength = sky.params.x;
    let rain_factor = sky.params.y;
    let underwater = sky.params.z;
    let underground = sky.params.w;

    let v_dot_u_pos = max(v_dot_u, 0.0);
    let v_dot_s_pos = max(v_dot_s, 0.0);
    let v_dot_s2 = v_dot_s_pos * v_dot_s_pos;
    let v_dot_s4 = v_dot_s2 * v_dot_s2;

    // 1. Procedural 3-zone sky gradient (Zenith -> Bright Horizon -> Lower Horizon)
    let horiz_t_raw = (1.0 - v_dot_u_pos) * (1.0 - v_dot_u_pos);
    let horiz_exp = (1.0 - v_dot_s4 * 0.35) * (1.15 - 0.15 * rain_factor * rain_factor);
    var horiz_t = pow(max(horiz_t_raw, 0.0001), horiz_exp);
    horiz_t = mix(horiz_t, 1.0, rain_factor * rain_factor * 0.18);

    let middle_boosted = sky.middle_color.rgb * (1.0 + v_dot_s4 * 0.28 * sun_factor * (1.0 - rain_factor * 0.75));
    var final_sky = mix(sky.up_color.rgb, middle_boosted, horiz_t);

    // 2. Dedicated dawn/dusk sunset ribbon concentrated at the horizon
    if (sunset_strength > 0.001) {
        let band = 1.0 - abs(v_dot_u);
        let band2 = band * band;
        let band_smooth = band2 * band2 * (3.0 - 2.0 * band2);
        let sun_bias = 0.55 + 0.45 * v_dot_s2;
        let sunset_mix = clamp(band_smooth * sun_bias * sunset_strength * (1.0 - rain_factor * 0.75), 0.0, 1.0);
        final_sky = mix(final_sky, sky.sunset_color.rgb * (1.0 + v_dot_s2 * 0.30), sunset_mix);
    }

    // 3. Lower horizon ground-bounce transition below the horizon line
    let lower_t = clamp((-v_dot_u + 0.06) / 0.34, 0.0, 1.0);
    let lower_smooth = lower_t * lower_t * (3.0 - 2.0 * lower_t);
    final_sky = mix(final_sky, sky.down_color.rgb, lower_smooth);

    // 4. Forward-scattered Sun / Moon Glare halo
    let active_dot = select(-v_dot_s, v_dot_s, sun_factor > 0.25);
    if (active_dot > 0.0 && sky.glare_color.a > 0.001) {
        let glare_scatter = 4.5 * (1.0 - 0.55 * rain_factor * rain_factor);
        let dot_pow = pow(active_dot, max(glare_scatter, 1.2));
        let vis = 0.075;
        let glare = (vis / max(1.0 - (1.0 - vis) * dot_pow, 0.01) - vis) * 0.65;
        let rain_atten = 1.0 - 0.72 * rain_factor;
        final_sky += sky.glare_color.rgb * glare * sky.glare_color.a * rain_atten;
    }

    // 4b. Post-rain atmospheric rainbow opposite the sun (~42 deg antisolar arch)
    let rainbow_strength = sky.rainbow_params.x;
    if (rainbow_strength > 0.002 && v_dot_u > 0.0 && underwater < 0.5 && underground < 0.5) {
        let anti_dot = dot(dir, -sun_dir);
        let rainbow_coord = clamp(1.0 - (anti_dot - 0.715) / 0.062, 0.0, 1.0);
        let arch = rainbow_coord * (1.0 - rainbow_coord);
        let arch_mask = pow(arch * 4.0, 2.5) * smoothstep(0.0, 0.14, v_dot_u);
        if (arch_mask > 0.001) {
            let hue = smoothstep(0.0, 1.0, rainbow_coord) * 0.84;
            let k = abs(((hue * 6.0 + vec3<f32>(-0.55, 4.3, 2.2)) % 6.0 + 6.0) % 6.0 - 3.0) - 1.0;
            let spec = pow(clamp(k, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(2.0));
            final_sky += spec * vec3<f32>(0.55, 0.42, 0.55) * arch_mask * rainbow_strength * (1.0 - underground);
        }
    }

    // 4c. Procedural Aurora Borealis curtains in polar/cold biomes at night (smooth analytic 8-layer integration)
    let aurora_strength = sky.rainbow_params.y;
    let aurora_time = sky.rainbow_params.z;
    if (aurora_strength > 0.002 && v_dot_u > 0.06 && underwater < 0.5 && underground < 0.5) {
        let horizon_fade = smoothstep(0.06, 0.24, v_dot_u) * (1.0 - smoothstep(0.55, 0.96, v_dot_u));
        if (horizon_fade > 0.001) {
            let sky_proj = dir.xz / max(v_dot_u, 0.10);
            var aurora_acc = vec3<f32>(0.0);
            let STEPS: i32 = 8;
            for (var i: i32 = 0; i < STEPS; i = i + 1) {
                let h = (f32(i) + 0.5) / f32(STEPS);
                let plane = sky_proj * (0.75 + h * 0.95) * 1.45 + vec2<f32>(aurora_time * 0.025, aurora_time * 0.012);
                let warp = aurora_noise(plane * 0.55 + vec2<f32>(0.0, aurora_time * 0.03)) * 1.8;
                let ribbon = sin(plane.x * 1.6 + warp + sin(plane.y * 0.9 - aurora_time * 0.05) * 1.4);
                let band = pow(max(1.0 - abs(ribbon), 0.0), 5.5);
                let shimmer = 0.50 + 0.50 * aurora_noise(plane * 2.4 - vec2<f32>(aurora_time * 0.05, 0.0));
                let cur_col = mix(vec3<f32>(0.14, 1.35, 0.68), vec3<f32>(0.58, 0.22, 1.18), pow(h, 1.35));
                aurora_acc += cur_col * band * shimmer * (1.0 - h * 0.65);
            }
            final_sky += (aurora_acc / f32(STEPS)) * horizon_fade * aurora_strength * 0.85 * (1.0 - underground);
        }
    }

    // 5. Underwater / underground overrides
    if (underwater > 0.5) {
        final_sky = vec3<f32>(0.02, 0.14, 0.26);
    } else if (underground > 0.001) {
        let cave_col = vec3<f32>(0.025, 0.032, 0.052);
        final_sky = mix(final_sky, cave_col, underground);
    }

    // 6. Subtle 1/128 anti-banding dither
    let dither = (ign_dither(in.position.xy) - 0.5) / 128.0;
    final_sky = max(final_sky + vec3<f32>(dither), vec3<f32>(0.0));

    return vec4<f32>(final_sky, 1.0);
}
