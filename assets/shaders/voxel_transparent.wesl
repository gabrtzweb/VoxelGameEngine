import bevy_pbr::{
    render::{
        mesh_functions,
        forward_io::{Vertex, VertexOutput, FragmentOutput},
        view_transformations::{depth_ndc_to_view_z, position_world_to_view, position_world_to_clip},
        pbr_types,
        pbr_fragment::pbr_input_from_standard_material,
        pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
        mesh_view_bindings::{globals, view, view_transmission_texture, view_transmission_sampler},
    },
    prepass::utils as prepass_utils,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var voxel_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var<uniform> sky_up_color: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var<uniform> sky_middle_color: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var<uniform> sky_sunset_color: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var<uniform> sun_dir_and_factor: vec4<f32>;

// Lightweight procedural sky reflection matching SkyDome (zenith, horizon, sunset ribbon, sun glare)
fn sample_water_sky_reflection(refl_dir: vec3<f32>) -> vec3<f32> {
    let dir = normalize(refl_dir);
    let v_dot_u = max(dir.y, 0.0);
    let sun_dir = normalize(sun_dir_and_factor.xyz);
    let sun_factor = sun_dir_and_factor.w;
    let rain_factor = sky_up_color.a;
    let sunset_strength = sky_sunset_color.a;
    let v_dot_s = max(dot(dir, sun_dir), 0.0);
    let v_dot_s2 = v_dot_s * v_dot_s;
    let v_dot_s4 = v_dot_s2 * v_dot_s2;

    let horiz_t = pow(max((1.0 - v_dot_u) * (1.0 - v_dot_u), 0.0001), 1.05);
    let mid_boost = sky_middle_color.rgb * (1.0 + v_dot_s4 * 0.28 * sun_factor);
    var sky_col = mix(sky_up_color.rgb, mid_boost, horiz_t);

    if (sunset_strength > 0.001) {
        let band = 1.0 - abs(dir.y);
        let band2 = band * band;
        let sunset_mix = clamp(band2 * band2 * (0.55 + 0.45 * v_dot_s2) * sunset_strength * (1.0 - rain_factor * 0.75), 0.0, 1.0);
        sky_col = mix(sky_col, sky_sunset_color.rgb * (1.0 + v_dot_s2 * 0.30), sunset_mix);
    }

    // Crisp sun specular sparkle reflected off wave ripples
    let sun_spec = pow(v_dot_s, 96.0) * 2.4 * sun_factor * (1.0 - rain_factor * 0.85);
    sky_col += mix(vec3<f32>(1.0, 0.92, 0.74), sky_sunset_color.rgb, sunset_strength * 0.6) * sun_spec;
    return sky_col;
}

fn ssr_dither(pixel_pos: vec2<f32>) -> f32 {
    let magic = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
    return fract(magic.z * fract(dot(pixel_pos, magic.xy)));
}

// Screen-Space Water Reflections (jittered raymarch against opaque depth prepass)
// Returns vec4(reflected_rgb, blend_weight [0..1]); falls back seamlessly to Sky Dome on miss/screen-edge.
fn trace_water_ssr(world_pos: vec3<f32>, refl_dir: vec3<f32>, screen_pos: vec2<f32>) -> vec4<f32> {
    if (sky_middle_color.a < 0.5) {
        return vec4<f32>(0.0);
    }
@if(DEPTH_PREPASS) {
    let jitter = ssr_dither(screen_pos);
    var dist: f32 = 0.45 + jitter * 0.45;
    var step_len: f32 = 0.55;
    var prev_pos = world_pos;

    for (var i: i32 = 0; i < 20; i = i + 1) {
        let sample_pos = world_pos + refl_dir * dist;
        let clip = position_world_to_clip(sample_pos);
        if (clip.w <= 0.001) {
            break;
        }
        let ndc = clip.xyz / clip.w;
        let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
        if (uv.x < 0.002 || uv.x > 0.998 || uv.y < 0.002 || uv.y > 0.998) {
            break;
        }

        let frag_coord = vec4<f32>(uv * view.viewport.zw + view.viewport.xy, 0.0, 0.0);
        let scene_ndc = prepass_utils::prepass_depth(frag_coord, 0u);
        if (scene_ndc > 0.00001) {
            let scene_view_z = depth_ndc_to_view_z(scene_ndc);
            let ray_view_z = position_world_to_view(sample_pos).z;
            let depth_diff = scene_view_z - ray_view_z;
            let thickness = max(step_len * 2.4, 1.1);
            if (depth_diff > 0.02 && depth_diff < thickness) {
                // Midpoint refinement for crisp terrain/tree reflections
                let mid_pos = (prev_pos + sample_pos) * 0.5;
                let mid_clip = position_world_to_clip(mid_pos);
                var hit_uv = uv;
                if (mid_clip.w > 0.001) {
                    let mid_ndc = mid_clip.xyz / mid_clip.w;
                    let mid_uv = vec2<f32>(mid_ndc.x * 0.5 + 0.5, 0.5 - mid_ndc.y * 0.5);
                    if (mid_uv.x > 0.002 && mid_uv.x < 0.998 && mid_uv.y > 0.002 && mid_uv.y < 0.998) {
                        let mid_fc = vec4<f32>(mid_uv * view.viewport.zw + view.viewport.xy, 0.0, 0.0);
                        let mid_scene_ndc = prepass_utils::prepass_depth(mid_fc, 0u);
                        if (mid_scene_ndc > 0.00001) {
                            let mid_diff = depth_ndc_to_view_z(mid_scene_ndc) - position_world_to_view(mid_pos).z;
                            if (mid_diff > 0.0) {
                                hit_uv = mid_uv;
                            }
                        }
                    }
                }

                let edge_fade = smoothstep(0.0, 0.08, hit_uv.x)
                    * smoothstep(1.0, 0.92, hit_uv.x)
                    * smoothstep(0.0, 0.08, hit_uv.y)
                    * smoothstep(1.0, 0.88, hit_uv.y);
                let dist_fade = 1.0 - smoothstep(48.0, 125.0, dist);
                let ssr_rgb = textureSampleLevel(view_transmission_texture, view_transmission_sampler, hit_uv, 0.0).rgb;
                return vec4<f32>(ssr_rgb, edge_fade * dist_fade);
            }
        }

        prev_pos = sample_pos;
        dist += step_len;
        step_len *= 1.24;
    }
}
    return vec4<f32>(0.0);
}

// ==============================================================================
// 1. Procedural Simplex 2D Noise (Triangular Lattice - Zero Checkerboard/Grid)
// ==============================================================================

fn mod289_2(x: vec2<f32>) -> vec2<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute3(x: vec3<f32>) -> vec3<f32> {
    return mod289_3(((x * 34.0) + 1.0) * x);
}

// Stefan Gustavson's Simplex 2D Noise: Evaluated on an equilateral triangular lattice
fn water_hash(p: vec2<f32>) -> f32 {
    let p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    let p4 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p4.x + p4.y) * p4.z);
}

// Crisp 16x16 voxel-snapped non-repetitive pixel-art raindrop ripples for all liquid surfaces (visible from above & below)
fn compute_fluid_rain_ripple(world_xz: vec2<f32>, time: f32, rain_active: f32) -> f32 {
    if (rain_active <= 0.05) {
        return 0.0;
    }
    let texel_coord = floor(world_xz * 16.0);
    let cell_size = 8.0;
    let cell = floor(texel_coord / cell_size);
    let local_tx = texel_coord - cell * cell_size;
    let h1 = water_hash(cell);
    let raw_t = time * 1.35 + h1;
    let cycle_id = floor(raw_t);
    let cycle = fract(raw_t);
    let seed = cell + vec2<f32>(cycle_id * 13.7, cycle_id * 29.3);
    let h2 = water_hash(seed + vec2<f32>(19.7, 43.1));
    let frame = floor(cycle * 5.0);
    if (h2 < 0.38 && frame < 3.5) {
        let center_tx = vec2<f32>(
            floor(2.0 + water_hash(seed + vec2<f32>(3.1, 7.4)) * 4.0),
            floor(2.0 + water_hash(seed + vec2<f32>(11.2, 5.8)) * 4.0)
        );
        let delta_tx = local_tx - center_tx;
        let cheb_dist = max(abs(delta_tx.x), abs(delta_tx.y));
        let manh_dist = abs(delta_tx.x) + abs(delta_tx.y);
        let ring_r = frame;
        let on_ring = (abs(cheb_dist - ring_r) < 0.25) && (manh_dist <= ring_r * 2.0 - select(0.0, 1.0, ring_r >= 2.0));
        if (on_ring) {
            return (1.0 - frame / 4.0) * rain_active;
        }
    }
    return 0.0;
}

fn simplex_noise2(v: vec2<f32>) -> f32 {
    let C = vec4<f32>(
        0.211324865405187,  // (3.0 - sqrt(3.0)) / 6.0
        0.366025403784439,  // 0.5 * (sqrt(3.0) - 1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439   // 1.0 / 41.0
    );

    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    var i1: vec2<f32>;
    if (x0.x > x0.y) {
        i1 = vec2<f32>(1.0, 0.0);
    } else {
        i1 = vec2<f32>(0.0, 1.0);
    }

    let x1 = x0.xy + C.xx - i1;
    let x2 = x0.xy + C.zz;

    i = mod289_2(i);
    let p = permute3(
        permute3(i.y + vec3<f32>(0.0, i1.y, 1.0))
        + i.x + vec3<f32>(0.0, i1.x, 1.0)
    );

    var m = max(0.5 - vec3<f32>(dot(x0, x0), dot(x1, x1), dot(x2, x2)), vec3<f32>(0.0));
    m = m * m;
    m = m * m;

    let x = 2.0 * fract(p * C.www) - 1.0;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    m = m * (1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h));

    let g = vec3<f32>(
        a0.x * x0.x + h.x * x0.y,
        a0.y * x1.x + h.y * x1.y,
        a0.z * x2.x + h.z * x2.y
    );

    return 130.0 * dot(m, g); // [-1.0 .. 1.0]
}

// Rotation matrices with irrational angles to break any spatial grid alignment
const ROT_FOAM_1: mat2x2<f32> = mat2x2<f32>(0.832, -0.555, 0.555, 0.832); // ~33.7 degrees
const ROT_FOAM_2: mat2x2<f32> = mat2x2<f32>(0.573, 0.819, -0.819, 0.573);  // ~55.0 degrees

// Multi-octave organic foam noise with domain rotation and fluid drift
fn organic_foam_noise(pos: vec2<f32>, time: f32) -> f32 {
    let p = pos * 1.6;
    let drift1 = vec2<f32>(time * 0.16, time * 0.10);
    let drift2 = vec2<f32>(-time * 0.12, time * 0.18);

    let n1 = simplex_noise2(ROT_FOAM_1 * p + drift1);
    let n2 = simplex_noise2(ROT_FOAM_2 * (p * 2.2) + drift2) * 0.5;
    let n3 = simplex_noise2((ROT_FOAM_1 * ROT_FOAM_2) * (p * 4.6) - drift1 * 1.4) * 0.25;

    return clamp((n1 + n2 + n3) * 0.35 + 0.5, 0.0, 1.0);
}

// Multi-layer domain-rotated organic caustics web projected onto shallow submerged terrain
fn calculate_caustics(pos_xz: vec2<f32>, time: f32) -> f32 {
    let p1 = (ROT_FOAM_1 * pos_xz) * 0.22 + vec2<f32>(time * 0.14, time * 0.09);
    let p2 = (ROT_FOAM_2 * pos_xz) * 0.31 - vec2<f32>(time * 0.11, -time * 0.15);
    let warp = simplex_noise2(pos_xz * 0.12 + vec2<f32>(time * 0.05, -time * 0.04)) * 0.45;

    let n1 = simplex_noise2(p1 + vec2<f32>(warp, -warp));
    let n2 = simplex_noise2(p2 - vec2<f32>(warp * 0.6, warp * 0.8));
    let c1 = 1.0 - abs(n1);
    let c2 = 1.0 - abs(n2);
    return pow(c1 * c2, 2.4) * 1.35;
}

// ==============================================================================
// 2. Procedural Wave Math (Shared between Vertex & Fragment)
// ==============================================================================

struct WaveData {
    height: f32,
    gradient: vec2<f32>, // (dh/dx, dh/dz) for surface normal perturbation
};

// Coordinated directional swells travelling in a shared wind quadrant
fn calculate_wave_height(pos_xz: vec2<f32>, time: f32) -> f32 {
    var h: f32 = 0.0;

    // Octave 1: Primary ocean roller (~30 deg)
    let dir1 = vec2<f32>(0.866, 0.500);
    h += 0.052 * sin(dot(dir1, pos_xz) * 0.60 + time * 1.35);

    // Octave 2: Secondary swell (~45 deg)
    let dir2 = vec2<f32>(0.707, 0.707);
    h += 0.028 * sin(dot(dir2, pos_xz) * 1.20 + time * 1.75);

    // Octave 3: Wind chop (~15 deg)
    let dir3 = vec2<f32>(0.966, 0.259);
    h += 0.015 * sin(dot(dir3, pos_xz) * 2.40 + time * 2.50);

    // Octave 4: Capillary ripples (~60 deg)
    let dir4 = vec2<f32>(0.500, 0.866);
    h += 0.007 * sin(dot(dir4, pos_xz) * 4.80 + time * 3.30);

    return h;
}

fn calculate_wave_data(pos_xz: vec2<f32>, time: f32) -> WaveData {
    var h: f32 = 0.0;
    var grad: vec2<f32> = vec2<f32>(0.0, 0.0);

    // Octave 1
    let dir1 = vec2<f32>(0.866, 0.500);
    let freq1 = 0.60;
    let amp1 = 0.052;
    let phase1 = dot(dir1, pos_xz) * freq1 + time * 1.35;
    let s1 = sin(phase1);
    let c1 = cos(phase1);
    h += amp1 * s1;
    grad += (amp1 * freq1 * c1) * dir1;

    // Octave 2
    let dir2 = vec2<f32>(0.707, 0.707);
    let freq2 = 1.20;
    let amp2 = 0.028;
    let phase2 = dot(dir2, pos_xz) * freq2 + time * 1.75;
    let s2 = sin(phase2);
    let c2 = cos(phase2);
    h += amp2 * s2;
    grad += (amp2 * freq2 * c2) * dir2;

    // Octave 3
    let dir3 = vec2<f32>(0.966, 0.259);
    let freq3 = 2.40;
    let amp3 = 0.015;
    let phase3 = dot(dir3, pos_xz) * freq3 + time * 2.50;
    let s3 = sin(phase3);
    let c3 = cos(phase3);
    h += amp3 * s3;
    grad += (amp3 * freq3 * c3) * dir3;

    // Octave 4
    let dir4 = vec2<f32>(0.500, 0.866);
    let freq4 = 4.80;
    let amp4 = 0.007;
    let phase4 = dot(dir4, pos_xz) * freq4 + time * 3.30;
    let s4 = sin(phase4);
    let c4 = cos(phase4);
    h += amp4 * s4;
    grad += (amp4 * freq4 * c4) * dir4;

    return WaveData(h, grad);
}

// Toned-down, viscous fluid wave math for generic liquids (Acid, Lava, Tar, Sludge, etc.)
fn calculate_fluid_wave_height(pos_xz: vec2<f32>, time: f32) -> f32 {
    var h: f32 = 0.0;
    let t = time * 0.7;

    // Primary gentle swell
    let dir1 = vec2<f32>(0.866, 0.500);
    h += 0.016 * sin(dot(dir1, pos_xz) * 0.75 + t * 0.9);

    // Secondary cross swell
    let dir2 = vec2<f32>(-0.600, 0.800);
    h += 0.009 * sin(dot(dir2, pos_xz) * 1.30 + t * 1.2);

    // Soft surface heave
    let dir3 = vec2<f32>(0.350, -0.937);
    h += 0.005 * sin(dot(dir3, pos_xz) * 2.10 + t * 1.5);

    return h;
}

fn calculate_fluid_wave_data(pos_xz: vec2<f32>, time: f32) -> WaveData {
    var h: f32 = 0.0;
    var grad: vec2<f32> = vec2<f32>(0.0, 0.0);
    let t = time * 0.7;

    let dir1 = vec2<f32>(0.866, 0.500);
    let freq1 = 0.75;
    let amp1 = 0.016;
    let phase1 = dot(dir1, pos_xz) * freq1 + t * 0.9;
    h += amp1 * sin(phase1);
    grad += (amp1 * freq1 * cos(phase1)) * dir1;

    let dir2 = vec2<f32>(-0.600, 0.800);
    let freq2 = 1.30;
    let amp2 = 0.009;
    let phase2 = dot(dir2, pos_xz) * freq2 + t * 1.2;
    h += amp2 * sin(phase2);
    grad += (amp2 * freq2 * cos(phase2)) * dir2;

    let dir3 = vec2<f32>(0.350, -0.937);
    let freq3 = 2.10;
    let amp3 = 0.005;
    let phase3 = dot(dir3, pos_xz) * freq3 + t * 1.5;
    h += amp3 * sin(phase3);
    grad += (amp3 * freq3 * cos(phase3)) * dir3;

    return WaveData(h, grad);
}

// ==============================================================================
// 3. Vertex Shader: Water Surface Displacement
// ==============================================================================

// Block-aligned travelling swell: a pure function of WORLD xz + time, so neighbouring quads and
// chunks always agree on the surface height.
fn surface_wave_offset(world_xz: vec2<f32>, time: f32) -> f32 {
    let wave_time = time * 5.5;
    let p = world_xz * 13.0;
    var wave = sin(wave_time * 0.72 - p.y * 0.135 + p.x * 0.065);
    wave += sin(wave_time * 0.48 - p.y * 0.095 + p.x * 0.052);
    wave *= 0.1;
    return wave * 0.125 - 0.05;
}


@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
    let world_from_local = mesh_world_from_local;

@if(VERTEX_NORMALS)
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex_no_morph.normal,
        vertex_no_morph.instance_index
    );
@else
    out.world_normal = vec3<f32>(0.0, 1.0, 0.0);

@if(VERTEX_POSITIONS) {
    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex_no_morph.position, 1.0)
    );

    let time = max(globals.time, 0.0);
    let surface_type = round(vertex_no_morph.color.a);

    // Surface wave: applied only to vertices that are not on a block floor line (fract(y) > 0.06).
    // Because fluid quads are meshed 1x1 every corner samples the same field, so waves are seamless
    // across blocks and chunk borders.
    if (surface_type > 0.5 && surface_type < 2.5) {
        if (fract(out.world_position.y + 0.005) > 0.06) {
            var wave_scale = 1.0;
            if (surface_type > 1.5) {
                wave_scale = 0.4; // viscous non-water fluids ripple less
            }
            out.world_position.y += surface_wave_offset(out.world_position.xz, time) * wave_scale;
        }
    }

    // Clip position projected from the world coordinates
    out.position = position_world_to_clip(out.world_position.xyz);
}

@if(VERTEX_UVS_A)
    out.uv = vertex_no_morph.uv;
@if(VERTEX_UVS_B)
    out.uv_b = vertex_no_morph.uv_b;
@if(VERTEX_COLORS)
    out.color = vertex_no_morph.color;
@if(VERTEX_OUTPUT_INSTANCE_INDEX)
    out.instance_index = vertex_no_morph.instance_index;

    return out;
}

// ==============================================================================
// 4. Fragment Shader: PBR Lighting, Volumetric Depth & Organic Seafoam
// ==============================================================================

fn deep_color_blend(deep_water: vec3<f32>, depth_dist: f32) -> vec3<f32> {
    // Exponential Beer-Lambert depth absorption for deep ocean
    let absorption = exp(-depth_dist * 0.08);
    return deep_water * mix(0.55, 1.0, absorption);
}

@fragment
fn fragment(
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_vertex = vertex_output;
    pbr_vertex.color = vec4<f32>(1.0);
    var pbr_input: pbr_types::PbrInput = pbr_input_from_standard_material(pbr_vertex, is_front);
    let time = max(globals.time, 0.0);
    let surface_type = round(vertex_output.color.a);

    let orig_normal = normalize(vertex_output.world_normal);
    let is_horizontal = abs(orig_normal.y) > 0.5;

    var wave_normal: vec3<f32>;

    if (surface_type < 0.5) {
        // --- 0. Solid Transparent (Glass, Ice, Fragile Ice, Black Ice) ---
        // Rigid, unperturbed surface normal - completely static, no waves or waterfalls!
        wave_normal = orig_normal;
        pbr_input.material.perceptual_roughness = 0.08;
        pbr_input.material.reflectance = vec3<f32>(0.50);
    } else if (surface_type > 1.5) {
        // --- 2. Generic Fluid (Acid, Lava, Molten, Tar, Blood, Sludge, Ooze, Null) ---
        // Toned-down, gentle fluid undulation without aggressive ocean chop
        let fluid_wave = calculate_fluid_wave_data(vertex_output.world_position.xz, time);
        if (is_horizontal) {
            let normal_sign = sign(orig_normal.y);
            wave_normal = normalize(vec3<f32>(-fluid_wave.gradient.x, normal_sign, -fluid_wave.gradient.y));
        } else {
            // Gentle, slow vertical creeping flow on side faces
            let flow = 0.02 * cos(vertex_output.world_position.y * 3.0 - time * 2.0 + (vertex_output.world_position.x + vertex_output.world_position.z) * 1.2);
            wave_normal = normalize(orig_normal + vec3<f32>(0.0, flow, 0.0));
        }
        pbr_input.material.perceptual_roughness = 0.18;
        pbr_input.material.reflectance = vec3<f32>(0.65);
    } else {
        // --- 1. Water (Liquid_Water, WaterOccupied) ---
        // Full ocean wave gradients and rapid cascading waterfalls
        let wave = calculate_wave_data(vertex_output.world_position.xz, time);
        if (is_horizontal) {
            let normal_sign = sign(orig_normal.y);
            wave_normal = normalize(vec3<f32>(-wave.gradient.x, normal_sign, -wave.gradient.y));
        } else {
            let flow = 0.05 * cos(vertex_output.world_position.y * 5.0 - time * 6.0 + (vertex_output.world_position.x + vertex_output.world_position.z) * 2.0);
            wave_normal = normalize(orig_normal + vec3<f32>(0.0, flow, 0.0));
        }
        pbr_input.material.perceptual_roughness = 0.04;
        pbr_input.material.reflectance = vec3<f32>(0.90);
    }

    if (!is_front) {
        wave_normal = -wave_normal;
    }

    pbr_input.world_normal = wave_normal;
    pbr_input.N = wave_normal;

    // 2. Original Animated Pixel Art Texture (Core Base Color)
    let uv = fract(vertex_output.uv);
    let base_layer = vertex_output.uv_b.x;
    let frame_count = vertex_output.uv_b.y;

    var layer = i32(round(base_layer));
    var next_layer = layer;
    var frame_blend = 0.0;
    let is_emissive = frame_count < -0.5;
    let actual_frame_count = abs(frame_count);
    if (actual_frame_count > 1.5) {
        let fps = 2.0;
        let count = max(1, i32(round(actual_frame_count)));
        let t = max(time, 0.0) * fps;
        let frame_cur = (i32(floor(t)) % count + count) % count;
        let frame_next = (frame_cur + 1) % count;
        frame_blend = fract(t);
        layer = layer + frame_cur;
        next_layer = i32(round(base_layer)) + frame_next;
    }

    var tex_color = textureSample(voxel_texture_array, voxel_sampler, uv, layer);
    if (frame_blend > 0.001) {
        let tex_next = textureSample(voxel_texture_array, voxel_sampler, uv, next_layer);
        tex_color = mix(tex_color, tex_next, frame_blend);
    }

    // Biome / Vertex Tint
    var biome_tint = vertex_output.color.rgb;
    var base_pixel_color: vec3<f32>;
    if (length(biome_tint) < 0.001) {
        base_pixel_color = tex_color.rgb;
    } else if (surface_type < 0.5 && (biome_tint.r < 0.99 || biome_tint.g < 0.99 || biome_tint.b < 0.99)) {
        let tex_srgb = pow(max(tex_color.rgb, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
        let highlight = clamp((tex_srgb - vec3<f32>(0.78)) / 0.22, vec3<f32>(0.0), vec3<f32>(1.0));
        let painted_srgb = tex_srgb * biome_tint + highlight * 0.08 * max(vec3<f32>(1.0) - biome_tint, vec3<f32>(0.0));
        base_pixel_color = pow(clamp(painted_srgb, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(2.2));
    } else {
        base_pixel_color = tex_color.rgb * biome_tint;
    }

    var final_color = base_pixel_color;
    var final_alpha = tex_color.a;
    var water_fresnel_refl: f32 = 0.0;
    var water_sky_refl_col = vec3<f32>(0.0);

    if (surface_type > 0.5 && surface_type < 1.5) {
        // --- 1. Water: Visible Pixel-Art Texture, Progressive Depth Darkening & Foam ---
        var depth_dist: f32 = 16.0;
        var y_pos_dif: f32 = -16.0;
@if(DEPTH_PREPASS) {
        let floor_ndc = prepass_utils::prepass_depth(vertex_output.position, 0u);
        if (floor_ndc <= 0.00001) {
            depth_dist = 1.2;
            y_pos_dif = -1.0;
        } else {
            let floor_view_z = depth_ndc_to_view_z(floor_ndc);
            let water_view_z = position_world_to_view(vertex_output.world_position.xyz).z;
            depth_dist = clamp(max(0.0, water_view_z - floor_view_z), 0.0, 32.0);

            let cam_pos = view.world_position;
            let to_frag = vertex_output.world_position.xyz - cam_pos;
            if (abs(water_view_z) > 0.0001) {
                let ratio = max(floor_view_z / water_view_z, 1.0);
                let floor_y = cam_pos.y + to_frag.y * ratio;
                y_pos_dif = floor_y - vertex_output.world_position.y;
            }
        }
}

        // Texture Preservation:
        // Boost the animated ripple contrast so the authentic pixel-art water texture is
        // crisp, vivid and clearly visible on the surface.
        let luma = dot(base_pixel_color, vec3<f32>(0.299, 0.587, 0.114));
        let texture_detail = mix(vec3<f32>(luma), base_pixel_color, 1.35) * 1.15;

        // Progressive Depth Absorption:
        // Shallows (0..2m): alpha ~0.62..0.80, bright clear texture with seabed visible below.
        // Medium depth (2..5m): alpha ~0.80..0.94, water shifts to richer deep blue.
        // Deep water (5m+ / open ocean): alpha ~0.98, heavily darkened into oceanic navy abyss,
        // completely concealing the seabed from the surface.
        let depth_factor = 1.0 - exp(-depth_dist * 0.45);
        let water_alpha_base = mix(0.62, 0.98, depth_factor);
        let depth_darkening = mix(1.0, 0.12, pow(depth_factor, 1.25));
        let depth_chroma = mix(vec3<f32>(1.0, 1.02, 1.05), vec3<f32>(0.35, 0.65, 1.18), depth_factor);

        var water_base = texture_detail * depth_darkening * depth_chroma;

        // Subtle sunlit seabed caustics shimmer in very shallow water
        if (is_front && depth_dist < 4.5 && sun_dir_and_factor.w > 0.05) {
            let shallow_fade = smoothstep(4.5, 0.45, depth_dist) * smoothstep(0.08, 0.50, depth_dist);
            let caustic_val = calculate_caustics(vertex_output.world_position.xz, time);
            let caustic_tint = vec3<f32>(0.68, 0.94, 1.04) * (0.075 * caustic_val * shallow_fade * sun_dir_and_factor.w);
            water_base += caustic_tint;
        }

        // Fresnel reflection: grazing angles reflect dynamic sky/sunset/sun and become opaque
        let cam_pos = view.world_position;
        let to_frag = vertex_output.world_position.xyz - cam_pos;
        let view_dir = normalize(to_frag);
        let fresnel = clamp(1.0 + dot(wave_normal, view_dir), 0.0, 1.0);
        let fresnel4 = fresnel * fresnel * fresnel * fresnel;
        var water_alpha = mix(water_alpha_base, 1.0, fresnel4 * 0.65);

        // Organic Animated Shoreline Foam (activates organic_foam_noise)
        var foam: f32 = 0.0;
        if (orig_normal.y > 0.99 && is_front) {
            let foam_threshold = 0.24;
            let raw_edge = clamp((foam_threshold + y_pos_dif) / foam_threshold, 0.0, 1.0);
            let foam_noise = organic_foam_noise(vertex_output.world_position.xz, time);
            let organic_edge = smoothstep(0.28, 0.82, raw_edge * (0.55 + foam_noise * 0.85));
            foam = organic_edge * 0.72;
            foam *= clamp((fract(vertex_output.world_position.y) - 0.7) * 10.0, 0.0, 1.0);
        }
        let foam_color = vec3<f32>(0.92, 0.96, 1.02);
        water_base = mix(water_base, foam_color, foam);
        water_alpha = mix(water_alpha, 0.92, foam);

        // Crisp 16x16 voxel-snapped pixel-art raindrop ripples on water (visible both from above and looking up underwater!)
        let water_rain_glint = select(
            0.0,
            compute_fluid_rain_ripple(vertex_output.world_position.xz, time, sky_up_color.w),
            orig_normal.y > 0.95
        );

        if (is_front) {
            if (water_rain_glint > 0.001) {
                water_base = mix(water_base, vec3<f32>(0.84, 0.92, 0.98), water_rain_glint * 0.28);
                water_alpha = max(water_alpha, water_rain_glint * 0.40);
            }
            let refl_dir = reflect(view_dir, wave_normal);
            water_sky_refl_col = sample_water_sky_reflection(refl_dir);
            let ssr = trace_water_ssr(vertex_output.world_position.xyz, refl_dir, vertex_output.position.xy);
            if (ssr.a > 0.001) {
                water_sky_refl_col = mix(water_sky_refl_col, ssr.rgb, ssr.a * 0.92);
            }
            water_fresnel_refl = clamp(0.10 + fresnel4 * 0.76, 0.0, 0.84) * (1.0 - foam * 0.85);
        } else {
            // Looking up at water surface from underwater: flip normal downward with balanced translucent pixel-art texture + visible rain ripples
            pbr_input.N = -wave_normal;
            pbr_input.world_normal = -wave_normal;
            water_alpha = 0.42;
            water_base = base_pixel_color * 1.35;
            if (water_rain_glint > 0.001) {
                water_base = mix(water_base, vec3<f32>(0.86, 0.94, 1.00), water_rain_glint * 0.42);
                water_alpha = max(water_alpha, 0.56);
            }
        }

        final_color = water_base;
        final_alpha = clamp(water_alpha, 0.0, 0.98);
        pbr_input.material.perceptual_roughness = mix(0.05, 0.36, foam);
        pbr_input.material.reflectance = vec3<f32>(0.85);
    } else if (surface_type > 1.5) {
        // --- 2. Generic Fluid (Lava, CoreMagma, Molten, Acid, Tar, Blood, Sludge, Ooze) ---
        let fluid_rain_glint = select(
            0.0,
            compute_fluid_rain_ripple(vertex_output.world_position.xz, time, sky_up_color.w),
            orig_normal.y > 0.95
        );
        if (!is_front) {
            // Looking up from inside non-water liquids: flip normal downward & ensure clear texture visibility
            // even in dark/non-emissive liquids (Blood, Ooze, Sludge, Tar) and less transparent Acid
            pbr_input.N = -orig_normal;
            pbr_input.world_normal = -orig_normal;
            final_color = base_pixel_color * 1.30;
            final_alpha = 0.84;
            if (fluid_rain_glint > 0.001) {
                final_color = mix(final_color, final_color * 1.55 + vec3<f32>(0.18), fluid_rain_glint * 0.35);
            }
        } else {
            final_color = base_pixel_color;
            if (fluid_rain_glint > 0.001) {
                final_color = mix(final_color, final_color * 1.45 + vec3<f32>(0.20), fluid_rain_glint * 0.30);
            }
            final_alpha = max(tex_color.a, 0.92);
        }
    } else {
        // --- 0. Solid Transparent ---
        // Pure texture base color and exact texture alpha (glass, ice, black ice)
        final_color = base_pixel_color;
        final_alpha = tex_color.a;
    }

    pbr_input.material.base_color = vec4<f32>(final_color, final_alpha);

    // 5. Final Lighting Evaluation
    var out: FragmentOutput;
    if (is_emissive) {
        // Emissive light-emitting fluids (Lava, Molten) boosted into HDR bloom range (> 1.0)
        out.color = vec4<f32>(final_color * 2.2, final_alpha);
    } else {
        out.color = apply_pbr_lighting(pbr_input);
        if (water_fresnel_refl > 0.001) {
            out.color = vec4<f32>(mix(out.color.rgb, water_sky_refl_col, water_fresnel_refl), out.color.a);
        }
        // Ensure fluid undersides remain visibly textured when looking up (including water at dusk/night)
        if (!is_front) {
            if (surface_type > 1.5) {
                out.color = vec4<f32>(max(out.color.rgb, final_color * 0.65), out.color.a);
            } else if (surface_type > 0.5) {
                out.color = vec4<f32>(max(out.color.rgb, final_color * 0.30), out.color.a);
            }
        }
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    out.color.a = final_alpha;

    // Raised Specular Highlight Ceiling for fluids / water (allows vibrant sun glints & HDR Bloom)
    if (surface_type > 0.5) {
        let fluid_luma = dot(out.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        let max_fluid_luma: f32 = 3.20;
        if (fluid_luma > max_fluid_luma) {
            out.color = vec4<f32>(out.color.rgb * (max_fluid_luma / fluid_luma), out.color.a);
        }
    }

    return out;
}
