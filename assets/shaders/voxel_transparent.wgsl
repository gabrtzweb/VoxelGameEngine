#import bevy_pbr::{
    mesh_functions,
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    view_transformations::{depth_ndc_to_view_z, position_world_to_view, position_world_to_clip},
    pbr_types,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::globals,
    prepass_utils,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var voxel_sampler: sampler;

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
// 3. Vertex Shader: Physical 3D Water & Fluid Surface Displacement
// ==============================================================================

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
    let world_from_local = mesh_world_from_local;

#ifdef VERTEX_NORMALS
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex_no_morph.normal,
        vertex_no_morph.instance_index
    );
#else
    out.world_normal = vec3<f32>(0.0, 1.0, 0.0);
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex_no_morph.position, 1.0)
    );

    let time = max(globals.time, 0.0);
    let surface_type = round(vertex_no_morph.color.a);

    // Apply physical Y-axis wave displacement to upward-facing fluid surfaces ONLY
    if (out.world_normal.y > 0.5) {
        if (surface_type > 0.5 && surface_type < 1.5) {
            // Water: full physical waves
            let wave_h = calculate_wave_height(out.world_position.xz, time);
            out.world_position.y += wave_h;
        } else if (surface_type > 1.5) {
            // Generic Fluid: toned-down, gentle viscous undulation
            let wave_h = calculate_fluid_wave_height(out.world_position.xz, time);
            out.world_position.y += wave_h;
        }
        // surface_type < 0.5 (Solid Transparent): strictly ZERO displacement
    }

    // Clip position projected from the physically displaced world coordinates
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex_no_morph.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex_no_morph.uv_b;
#endif
#ifdef VERTEX_COLORS
    out.color = vertex_no_morph.color;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex_no_morph.instance_index;
#endif

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
    var pbr_input: pbr_types::PbrInput = pbr_input_from_standard_material(vertex_output, is_front);
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
    let is_emissive = frame_count < -0.5;
    let actual_frame_count = abs(frame_count);
    if (actual_frame_count > 1.5) {
        let fps = 6.0;
        let count = max(1, i32(round(actual_frame_count)));
        let frame = (i32(floor(time * fps)) % count + count) % count;
        layer = layer + frame;
    }

    let tex_color = textureSample(voxel_texture_array, voxel_sampler, uv, layer);

    // Biome / Vertex Tint
    var biome_tint = vertex_output.color.rgb;
    if (length(biome_tint) < 0.1) {
        biome_tint = vec3<f32>(1.0);
    }

    let base_pixel_color = tex_color.rgb * biome_tint;

    var final_color = base_pixel_color;
    var final_alpha = tex_color.a;

    if (surface_type > 0.5 && surface_type < 1.5) {
        // --- 1. Water: Volumetric Depth & Organic Seafoam ---
#ifdef DEPTH_PREPASS
        let floor_ndc = prepass_utils::prepass_depth(vertex_output.position, 0u);
        var depth_dist: f32;
        // In reverse-Z depth prepass, floor_ndc <= 0.00001 indicates clear depth / far plane (unrendered background)
        if (floor_ndc <= 0.00001) {
            depth_dist = 12.0;
        } else {
            let floor_view_z = depth_ndc_to_view_z(floor_ndc);
            let water_view_z = position_world_to_view(vertex_output.world_position.xyz).z;
            depth_dist = clamp(max(0.0, water_view_z - floor_view_z), 0.0, 24.0);
        }
#else
        let depth_dist = 5.0;
#endif

        let shore_fade = smoothstep(0.0, 1.2, depth_dist);
        let tex_a = max(tex_color.a, 0.1);
        let shallow_alpha = mix(0.20, tex_a, shore_fade);
        let depth_opacity = smoothstep(1.5, 12.0, depth_dist);
        let base_water_alpha = mix(shallow_alpha, 0.98, depth_opacity);

        let shallow_tint = vec3<f32>(1.00, 1.04, 1.08);
        let mid_tint     = vec3<f32>(0.45, 0.70, 0.88);
        let deep_tint    = vec3<f32>(0.06, 0.16, 0.32);

        let t1 = smoothstep(0.8, 3.8, depth_dist);
        let t2 = smoothstep(3.8, 11.5, depth_dist);
        let depth_tint = mix(mix(shallow_tint, mid_tint, t1), deep_color_blend(deep_tint, depth_dist), t2);
        let water_base = base_pixel_color * depth_tint;

        var shore_foam: f32 = 0.0;
#ifdef DEPTH_PREPASS
        let shore_pulse = 0.5 + 0.5 * sin(time * 2.6 + dot(vertex_output.world_position.xz, vec2<f32>(0.7, 0.4)));
        let shore_foam_threshold = 0.36 + 0.12 * shore_pulse;
        let foam_sample = organic_foam_noise(vertex_output.world_position.xz, time);

        let shore_proximity = 1.0 - smoothstep(0.0, shore_foam_threshold, depth_dist);
        shore_foam = shore_proximity * smoothstep(0.28, 0.62, foam_sample + 0.22 * shore_pulse);
#endif

        let total_foam = clamp(shore_foam * 1.30, 0.0, 1.0);
        let foam_color = vec3<f32>(0.94, 0.97, 1.0);

        final_color = mix(water_base, foam_color, total_foam);
        final_alpha = clamp(mix(base_water_alpha, 0.98, total_foam), 0.0, 1.0);
        pbr_input.material.perceptual_roughness = mix(0.04, 0.32, total_foam);
    } else if (surface_type > 1.5) {
        // --- 2. Generic Fluid ---
        // Pure vibrant liquid colors (no ocean depth tint or foam)
        final_color = base_pixel_color;
        final_alpha = max(tex_color.a, 0.88);
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
        // Emissive light-emitting fluids (Lava, Molten)
        out.color = vec4<f32>(final_color * 1.25, final_alpha);
    } else {
        out.color = apply_pbr_lighting(pbr_input);
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    out.color.a = final_alpha;

    // Specular Highlight Ceiling for fluids / water
    if (surface_type > 0.5) {
        let fluid_luma = dot(out.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        let max_fluid_luma: f32 = 1.10;
        if (fluid_luma > max_fluid_luma) {
            out.color = vec4<f32>(out.color.rgb * (max_fluid_luma / fluid_luma), out.color.a);
        }
    }

    return out;
}
