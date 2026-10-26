import bevy_pbr::{
    render::{
        mesh_functions,
        forward_io::{Vertex, VertexOutput, FragmentOutput},
        view_transformations::{position_world_to_clip},
        pbr_types,
        pbr_fragment::pbr_input_from_standard_material,
        pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
        mesh_view_bindings::globals,
    },
};

struct GrassInfo {
    wind_params: vec4<f32>,       // xy = wind direction, z = flutter phase, w = plant bob phase
    wind_params2: vec4<f32>,      // xy = noise scroll offset, z = noise scale, w = blade width multiplier
    style_params: vec4<f32>,      // x = style (0.0 Tapered, 1.0 Segmented), y = wind intensity, z = blade visual height, w = grass brightness
    gradient_params: vec4<f32>,   // x = base multiplier, y = tip multiplier, z = curve, w = height variation scale
    trail_params2: vec4<f32>,     // xy = trail noise origin, zw = reserved
    entity_pushers: array<vec4<f32>, 8>, // pusher 0: x = trail origin x, y = trail origin z, z = inverse world size, w = trail strength
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> grass_info: GrassInfo;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var shape_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var shape_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var noise_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var noise_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var trail_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(106) var trail_sampler: sampler;

const PI2: f32 = 6.2831853;
const MAX_WIND_INTENSITY: f32 = 5.0;
const MAX_ROTATION_RADIANS: f32 = 1.05;

const COLOR_WAVE_CONTRAST: f32 = 1.0;
const COLOR_WAVE_BIAS: f32 = 0.5;

const BEND_MIN_HEIGHT_NOISE: f32 = 0.0;
const BEND_FULL_HEIGHT_NOISE: f32 = 0.1;
const BEND_AMOUNT: f32 = 0.1;
const BEND_LOWER_CONTROL_HEIGHT: f32 = 0.45;
const BEND_UPPER_CONTROL_HEIGHT: f32 = 0.82;
const BEND_LOWER_CONTROL_FRACTION: f32 = 0.0;
const BEND_UPPER_CONTROL_FRACTION: f32 = 0.22;
const BEND_TIP_FRACTION: f32 = 1.0;

const WIND_CURVE_ROTATION: f32 = 0.4;
const WIND_CURVE_PEAK_ROTATION: f32 = 0.52;
const WIND_CURVE_LOWER_ROTATION_FRACTION: f32 = 0.30;
const WIND_CURVE_UPPER_ROTATION_FRACTION: f32 = 0.72;

const REST_CURVE_ROTATION_MIN: f32 = -0.52;
const REST_CURVE_ROTATION_MAX: f32 = 0.52;
const REST_CURVE_FADE_END: f32 = 0.35;

const HEIGHT_NOISE_SCALE: f32 = 1.0;
const HEIGHT_NOISE_CONTRAST: f32 = 1.35;
const HEIGHT_NOISE_BIAS: f32 = 0.5;
const HEIGHT_NOISE_THIN_WIDTH: f32 = 1.0;
const HEIGHT_NOISE_SHORT_MULTIPLIER: f32 = 0.1;
const HEIGHT_NOISE_TALL_MULTIPLIER: f32 = 1.28;
const REGIONAL_LENGTH_FRACTION: f32 = 0.59;
const DEFAULT_ANIMATION_BLADE_LENGTH: f32 = 0.5075;
const TALL_PLANT_LENGTH_MULTIPLIER: f32 = 3.0;

const SEGMENT_MAX_COUNT: f32 = 4.0;
const SEGMENT_MIN_BAND_HEIGHT: f32 = 0.14;
const SEGMENT_BAND_VARIANTS: f32 = 8.0;

const GEO_MAX_SEGMENTS: f32 = 5.0;
const GEO_MIN_QUAD_HEIGHT: f32 = 0.115;

const LEAN_RESPONSE_MIN: f32 = 0.3;
const LEAN_RESPONSE_MAX: f32 = 0.58;

const TRAIL_WIND_SUPPRESSION: f32 = 0.95;
const TRAIL_CURVE_SUPPRESSION: f32 = 0.9;
const TRAIL_FLUTTER_SUPPRESSION: f32 = 1.0;

const FLUTTER_SPEED_STEPS: f32 = 64.0;
const FLUTTER_SPEED_MIN_STEP: f32 = 58.0;
const FLUTTER_SPEED_MAX_STEP: f32 = 70.0;

const MIN_BLADE_LENGTH: f32 = 0.02;
const PER_BLADE_HEIGHT_MIN: f32 = 0.88;
const PER_BLADE_HEIGHT_MAX: f32 = 1.14;
const PER_BLADE_WIDTH_COMPENSATION: f32 = 0.12;

const CLUMP_COORD_SCALE: f32 = 2.0;
const CLUMP_WIDTH_MIN: f32 = 0.92;
const CLUMP_WIDTH_MAX: f32 = 1.08;
const CLUMP_LEAN_COHESION: f32 = 0.42;
const CLUMP_REST_COHESION: f32 = 0.55;
const CLUMP_INWARD_ROTATION: f32 = 0.21;
const CLUMP_INWARD_DISTANCE_CURVE: f32 = 0.55;

const CAMERA_FACE_STRENGTH: f32 = 0.38;
const CAMERA_FACE_START_DISTANCE: f32 = 2.0;
const CAMERA_FACE_END_DISTANCE: f32 = 42.0;
const CAMERA_FACE_MIN_HEIGHT: f32 = 0.08;

const BAKED_BASE_WIDTH: f32 = 0.040;
const BAKED_MIDDLE_WIDTH: f32 = 0.050;
const BAKED_TIP_WIDTH: f32 = 0.013;
const BAKED_SPLIT_FRACTION: f32 = 0.6666667;
const BAKED_SEGMENTED_WIDTH: f32 = 0.050;
const TAPERED_WIDTH_SCALE: f32 = 0.8947368;

fn hash2d(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn sample_noise(uv: vec2<f32>) -> f32 {
    let dims = vec2<f32>(textureDimensions(noise_texture, 0));
    let pixel = fract(uv) * dims - 0.5;
    let pixel_floor = floor(pixel);
    let blend = smoothstep(vec2<f32>(0.0), vec2<f32>(1.0), fract(pixel));

    let u0 = (pixel_floor + 0.5) / dims;
    let u1 = (pixel_floor + vec2<f32>(1.0, 0.0) + 0.5) / dims;
    let u2 = (pixel_floor + vec2<f32>(0.0, 1.0) + 0.5) / dims;
    let u3 = (pixel_floor + vec2<f32>(1.0, 1.0) + 0.5) / dims;

    let bl = textureSampleLevel(noise_texture, noise_sampler, fract(u0), 0.0).r;
    let br = textureSampleLevel(noise_texture, noise_sampler, fract(u1), 0.0).r;
    let tl = textureSampleLevel(noise_texture, noise_sampler, fract(u2), 0.0).r;
    let tr = textureSampleLevel(noise_texture, noise_sampler, fract(u3), 0.0).r;

    let bottom = mix(bl, br, blend.x);
    let top = mix(tl, tr, blend.x);
    return mix(bottom, top, blend.y);
}

fn stationary_height_noise(coord: vec2<f32>) -> f32 {
    let n = sample_noise(coord * HEIGHT_NOISE_SCALE);
    let shaped = clamp((n - HEIGHT_NOISE_BIAS) * HEIGHT_NOISE_CONTRAST + HEIGHT_NOISE_BIAS, 0.0, 1.0);
    return smoothstep(0.0, 1.0, shaped);
}

fn sample_clump(coord: vec2<f32>) -> vec3<f32> {
    let dims = vec2<f32>(textureDimensions(noise_texture, 0));
    let pixel = floor(fract(coord) * dims);
    let uv = (pixel + 0.5) / dims;
    let c = textureSampleLevel(noise_texture, noise_sampler, uv, 0.0);
    return c.gba; // G: rest cohesion, B: lean cohesion, A: center distance
}

fn clump_to_center_dir(coord: vec2<f32>) -> vec2<f32> {
    let dims = vec2<f32>(textureDimensions(noise_texture, 0));
    let step_uv = 2.0 / dims;
    let zxp = sample_clump(coord + vec2<f32>(step_uv.x, 0.0)).z;
    let zxn = sample_clump(coord - vec2<f32>(step_uv.x, 0.0)).z;
    let zyp = sample_clump(coord + vec2<f32>(0.0, step_uv.y)).z;
    let zyn = sample_clump(coord - vec2<f32>(0.0, step_uv.y)).z;
    let grad = vec2<f32>(zxp - zxn, zyp - zyn);
    let len = length(grad);
    if (len > 0.0001) {
        return -grad / len;
    }
    return vec2<f32>(0.0);
}

fn mix_direction(from_dir: vec2<f32>, to_dir: vec2<f32>, amount: f32) -> vec2<f32> {
    let dot_val = clamp(dot(from_dir, to_dir), -1.0, 1.0);
    let cross_val = from_dir.x * to_dir.y - from_dir.y * to_dir.x;
    if (abs(cross_val) < 0.0001 && dot_val > 0.999) {
        return from_dir;
    }
    let angle = atan2(cross_val, dot_val) * clamp(amount, 0.0, 1.0);
    let s = sin(angle);
    let c = cos(angle);
    return vec2<f32>(
        from_dir.x * c - from_dir.y * s,
        from_dir.x * s + from_dir.y * c
    );
}

fn cubic_bezier(p0: vec3<f32>, p1: vec3<f32>, p2: vec3<f32>, p3: vec3<f32>, t: f32) -> vec3<f32> {
    let inv_t = 1.0 - t;
    return inv_t * inv_t * inv_t * p0
         + 3.0 * inv_t * inv_t * t * p1
         + 3.0 * inv_t * t * t * p2
         + t * t * t * p3;
}

fn rotated_spine_point(
    base_point: vec3<f32>,
    dir: vec2<f32>,
    visual_length: f32,
    anim_length: f32,
    h: f32,
    rotation: f32,
) -> vec3<f32> {
    var pt = base_point;
    pt.x += dir.x * (sin(rotation) * anim_length * h);
    pt.z += dir.y * (sin(rotation) * anim_length * h);
    pt.y += cos(rotation) * visual_length * h;
    return pt;
}

fn wind_curve_response(rot: f32) -> f32 {
    let rise = smoothstep(0.0, WIND_CURVE_PEAK_ROTATION, rot);
    let fall = 1.0 - smoothstep(WIND_CURVE_PEAK_ROTATION, MAX_ROTATION_RADIANS, rot);
    return rise * fall;
}

fn camera_facing_side_dir(center: vec3<f32>, orig_side: vec2<f32>, h: f32, cam_rel_xz: vec2<f32>) -> vec2<f32> {
    let dist = length(cam_rel_xz);
    if (dist < 0.001 || h < CAMERA_FACE_MIN_HEIGHT) {
        return orig_side;
    }
    let to_cam = normalize(cam_rel_xz);
    var cam_side = vec2<f32>(-to_cam.y, to_cam.x);
    if (dot(cam_side, orig_side) < 0.0) {
        cam_side = -cam_side;
    }
    let edge_on = 1.0 - abs(dot(orig_side, cam_side));
    let dist_fade = 1.0 - smoothstep(CAMERA_FACE_START_DISTANCE, CAMERA_FACE_END_DISTANCE, dist);
    let amt = CAMERA_FACE_STRENGTH * edge_on * dist_fade;
    let blended = mix(orig_side, cam_side, amt);
    let len = length(blended);
    if (len > 0.001) {
        return blended / len;
    }
    return orig_side;
}

fn blade_width(h: f32, tip_w: f32, reg_w: f32) -> f32 {
    if (grass_info.style_params.x > 0.5) {
        return BAKED_SEGMENTED_WIDTH * grass_info.wind_params2.w * reg_w;
    }
    let tapered_w = grass_info.wind_params2.w * TAPERED_WIDTH_SCALE * reg_w;
    if (h <= BAKED_SPLIT_FRACTION) {
        return mix(BAKED_BASE_WIDTH, BAKED_MIDDLE_WIDTH, h / BAKED_SPLIT_FRACTION) * tapered_w;
    }
    return mix(BAKED_MIDDLE_WIDTH, tip_w, (h - BAKED_SPLIT_FRACTION) / (1.0 - BAKED_SPLIT_FRACTION)) * tapered_w;
}

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
    var world_pos = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex_no_morph.position, 1.0)
    ).xyz;

    // STEP 1: Decode baked vertex attributes
    // vertex_no_morph.normal: xy = world noise coordinates [-1.0, 1.0], z = baked height fraction [0.0, 1.0]
    let height = clamp(vertex_no_morph.normal.z, 0.0, 1.0);
    let noise_coord = vertex_no_morph.normal.xy * 0.5 + 0.5;

    // vertex_no_morph.color.a packs: 1 side bit, 2 height-class bits, 5 angle bits
    let packed_blade_data = floor(vertex_no_morph.color.a * 255.0 + 0.5);
    let right_side = step(128.0, packed_blade_data);
    let side_sign = mix(-1.0, 1.0, right_side);

    let data_without_side = packed_blade_data - right_side * 128.0;
    let height_class = floor(data_without_side / 32.0);
    let angle_bucket = data_without_side - height_class * 32.0;

    let side_angle = (angle_bucket / 31.0) * PI2;
    let original_side_dir = vec2<f32>(cos(side_angle), sin(side_angle));

    // STEP 2: Regional + per-blade height and width calculation
    let height_noise = stationary_height_noise(noise_coord);
    let clump_coord = noise_coord * CLUMP_COORD_SCALE;
    let clump = sample_clump(clump_coord);
    let clump_influence = 0.6 + 0.4 * (1.0 - smoothstep(0.25, 0.85, clump.z));
    let clump_width_mult = mix(1.0, mix(CLUMP_WIDTH_MIN, CLUMP_WIDTH_MAX, smoothstep(0.0, 1.0, clump.y)), clump_influence);
    let height_multiplier = mix(HEIGHT_NOISE_SHORT_MULTIPLIER, HEIGHT_NOISE_TALL_MULTIPLIER, height_noise);

    var blade_length_mult = 1.0;
    if (height_class >= 1.5) {
        blade_length_mult = TALL_PLANT_LENGTH_MULTIPLIER;
    }
    let base_blade_len = grass_info.style_params.z * blade_length_mult;

    let blade_rnd = hash2d(noise_coord + vec2<f32>(angle_bucket * 0.071, height_class * 13.37));
    let shaped_rnd = smoothstep(0.0, 1.0, blade_rnd);
    let per_blade_height_mult = mix(PER_BLADE_HEIGHT_MIN, PER_BLADE_HEIGHT_MAX, shaped_rnd);
    let per_blade_width_mult = mix(1.0 + PER_BLADE_WIDTH_COMPENSATION, 1.0 - PER_BLADE_WIDTH_COMPENSATION, shaped_rnd);

    let regional_width = mix(HEIGHT_NOISE_THIN_WIDTH, 1.0, height_noise) * clump_width_mult * per_blade_width_mult;

    let reg_extra = base_blade_len * (height_multiplier - 1.0) * REGIONAL_LENGTH_FRACTION;
    let blade_extra = base_blade_len * (per_blade_height_mult - 1.0);
    let total_extra = (reg_extra + blade_extra) * grass_info.gradient_params.w;

    let blade_len = max(base_blade_len + total_extra, MIN_BLADE_LENGTH);
    let effective_extra = blade_len - base_blade_len;

    // GPU decimation for short blades
    let geo_segments = clamp(floor(blade_len / GEO_MIN_QUAD_HEIGHT), 1.0, GEO_MAX_SEGMENTS);
    let geo_height = floor(height * geo_segments + 0.5) / geo_segments;

    world_pos.y += effective_extra * height;

    // STEP 3: Centerline recovery
    let baked_width = blade_width(height, BAKED_TIP_WIDTH, 1.0);
    var drawn_width: f32;
    if (grass_info.style_params.x > 0.5) {
        drawn_width = BAKED_SEGMENTED_WIDTH * grass_info.wind_params2.w;
    } else {
        drawn_width = BAKED_BASE_WIDTH * grass_info.wind_params2.w * TAPERED_WIDTH_SCALE;
    }
    let final_width = drawn_width * regional_width;

    var source_center = world_pos;
    source_center.x -= original_side_dir.x * side_sign * baked_width;
    source_center.z -= original_side_dir.y * side_sign * baked_width;

    // STEP 4: Scrolling wind field
    let wind_intensity = max(grass_info.style_params.y, 0.0);
    let wind_amount = clamp(wind_intensity, 0.0, 1.0);
    let noise_bias = clamp(wind_intensity / MAX_WIND_INTENSITY, 0.0, 1.0);

    var wind_dir = grass_info.wind_params.xy;
    if (length(wind_dir) > 0.001) {
        wind_dir = normalize(wind_dir);
    } else {
        wind_dir = vec2<f32>(1.0, 0.0);
    }

    let noise_uv = noise_coord * grass_info.wind_params2.z + grass_info.wind_params2.xy;
    let wind_noise = sample_noise(noise_uv);

    let shaped_wave = clamp((wind_noise - COLOR_WAVE_BIAS) * COLOR_WAVE_CONTRAST + COLOR_WAVE_BIAS, 0.0, 1.0);
    let color_wave = smoothstep(0.0, 1.0, shaped_wave);

    let local_wind = mix(wind_noise, 1.0, noise_bias);
    let lean_rnd = mix(hash2d(noise_coord + vec2<f32>(43.17, 11.91)), clump.y, CLUMP_LEAN_COHESION * clump_influence);
    let lean_resp = mix(LEAN_RESPONSE_MIN, LEAN_RESPONSE_MAX, lean_rnd);

    var resisted_wind = local_wind * wind_amount * lean_resp;
    var wind_rot = clamp(resisted_wind, 0.0, 1.0) * MAX_ROTATION_RADIANS;

    // STEP 5: Entity interaction trail field
    let trail_params = grass_info.entity_pushers[0];
    var trail_dir = wind_dir;
    var trail_effect: f32 = 0.0;
    if (trail_params.w > 0.0) {
        let trail_uv = (source_center.xz - trail_params.xy) * trail_params.z;
        let inside = step(vec2<f32>(0.0), trail_uv) * step(trail_uv, vec2<f32>(1.0));
        let mask = inside.x * inside.y;

        let sample = textureSampleLevel(trail_texture, trail_sampler, trail_uv, 0.0);
        let sampled_dir = sample.rg * 2.0 - 1.0;
        let sampled_len = length(sampled_dir);
        if (sampled_len > 0.001) {
            trail_dir = sampled_dir / sampled_len;
        }
        trail_effect = clamp(sample.b * mask * trail_params.w, 0.0, 1.0);
    }

    let trail_mult = 1.0 - trail_effect * TRAIL_WIND_SUPPRESSION;
    resisted_wind *= trail_mult;
    wind_rot *= trail_mult;

    let trail_rot = trail_effect * MAX_ROTATION_RADIANS;

    // Inward tuft curl (Voronoi clump center lean)
    let to_clump_center = clump_to_center_dir(clump_coord);
    let curved_clump_dist = pow(clamp(clump.z, 0.0, 1.0), CLUMP_INWARD_DISTANCE_CURVE);
    let inward_rot = curved_clump_dist * CLUMP_INWARD_ROTATION * (1.0 - wind_amount);

    let lean_vec = wind_dir * wind_rot + trail_dir * trail_rot + to_clump_center * inward_rot;
    let rotation = clamp(length(lean_vec), 0.0, MAX_ROTATION_RADIANS);

    let active_rot = wind_rot + trail_rot;
    let trail_mix = trail_rot / max(active_rot, 0.0001);
    let active_dir = mix_direction(wind_dir, trail_dir, trail_mix);
    var rest_dir = active_dir;
    if (length(to_clump_center) > 0.001) {
        rest_dir = normalize(to_clump_center);
    }
    let act_mix = active_rot / max(inward_rot + active_rot, 0.0001);
    let lean_dir = mix_direction(rest_dir, active_dir, act_mix);
    let cross_lean_dir = vec2<f32>(-lean_dir.y, lean_dir.x);

    let wind_force_mag = clamp(max(resisted_wind, trail_effect), 0.0, 1.0);

    // STEP 6: Cubic Bezier spline deformation
    let segment_count = clamp(floor(blade_len / SEGMENT_MIN_BAND_HEIGHT), 1.0, SEGMENT_MAX_COUNT);
    let anim_blade_len = max(blade_len, DEFAULT_ANIMATION_BLADE_LENGTH * blade_length_mult);

    var base_pt = source_center;
    base_pt.y -= (0.435 + effective_extra) * height;

    let noise_bend_elig = smoothstep(BEND_MIN_HEIGHT_NOISE, BEND_FULL_HEIGHT_NOISE, height_noise);
    let plant_bend_elig = step(0.5, height_class);
    let bend_elig = max(noise_bend_elig, plant_bend_elig);

    var wind_curve_rot = WIND_CURVE_ROTATION * wind_curve_response(rotation) * bend_elig;
    wind_curve_rot *= 1.0 - trail_effect * TRAIL_CURVE_SUPPRESSION;

    let rest_curve_rnd = mix(hash2d(noise_coord + vec2<f32>(7.13, 91.77)), clump.x, CLUMP_REST_COHESION * clump_influence);
    let rest_curve_fade = 1.0 - smoothstep(0.0, REST_CURVE_FADE_END, wind_force_mag);
    var rest_curve_rot = mix(REST_CURVE_ROTATION_MIN, REST_CURVE_ROTATION_MAX, rest_curve_rnd) * rest_curve_fade * (bend_elig + 0.25);
    rest_curve_rot *= 1.0 - trail_effect * TRAIL_CURVE_SUPPRESSION;

    let tip_rot = rotation + wind_curve_rot + rest_curve_rot;
    var tip_pt = rotated_spine_point(base_pt, lean_dir, blade_len, anim_blade_len, 1.0, tip_rot);

    // Tip flutter
    let bend_calmness = 1.0 - smoothstep(0.25, 0.85, wind_force_mag);
    let bend_phase = dot(noise_coord, vec2<f32>(97.0, 151.0));
    let speed_rnd = hash2d(noise_coord + vec2<f32>(angle_bucket * 1.913 + 73.17, height_class * 11.71 + 29.43));
    let speed_step = floor(mix(FLUTTER_SPEED_MIN_STEP, FLUTTER_SPEED_MAX_STEP + 1.0, speed_rnd));
    let bend_wave = sin(grass_info.wind_params.z * (speed_step / FLUTTER_SPEED_STEPS) + bend_phase);

    var bend = bend_wave * BEND_AMOUNT * anim_blade_len * bend_elig * bend_calmness;
    bend *= 1.0 - trail_effect * TRAIL_FLUTTER_SUPPRESSION;
    let bend_offset = cross_lean_dir * bend;

    var lower_ctrl = rotated_spine_point(
        base_pt,
        lean_dir,
        blade_len,
        anim_blade_len,
        BEND_LOWER_CONTROL_HEIGHT,
        rotation + (wind_curve_rot + rest_curve_rot) * WIND_CURVE_LOWER_ROTATION_FRACTION
    );
    lower_ctrl.x += bend_offset.x * BEND_LOWER_CONTROL_FRACTION;
    lower_ctrl.z += bend_offset.y * BEND_LOWER_CONTROL_FRACTION;

    var upper_ctrl = rotated_spine_point(
        base_pt,
        lean_dir,
        blade_len,
        anim_blade_len,
        BEND_UPPER_CONTROL_HEIGHT,
        rotation + (wind_curve_rot + rest_curve_rot) * WIND_CURVE_UPPER_ROTATION_FRACTION
    );
    upper_ctrl.x += bend_offset.x * BEND_UPPER_CONTROL_FRACTION;
    upper_ctrl.z += bend_offset.y * BEND_UPPER_CONTROL_FRACTION;

    tip_pt.x += bend_offset.x * BEND_TIP_FRACTION;
    tip_pt.z += bend_offset.y * BEND_TIP_FRACTION;

    let blade_center = cubic_bezier(base_pt, lower_ctrl, upper_ctrl, tip_pt, geo_height);

    // STEP 7: Restore final visual width around curved spine with camera facing
    var final_pos = blade_center;
    let to_cam_xz = (grass_info.entity_pushers[0].xy + vec2<f32>(24.0, 24.0)) - source_center.xz;
    let visual_side_dir = camera_facing_side_dir(source_center, original_side_dir, height, to_cam_xz);
    final_pos.x += visual_side_dir.x * side_sign * final_width;
    final_pos.z += visual_side_dir.y * side_sign * final_width;

    // STEP 8: Emit outputs
    out.world_position = vec4<f32>(final_pos, 1.0);
    out.position = position_world_to_clip(final_pos);

    // Compute texture UV based on active style:
    var tex_uv: vec2<f32>;
    if (grass_info.style_params.x > 0.5) {
        let variant = fract(sin(dot(noise_coord, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        let band_u = (floor(variant * SEGMENT_BAND_VARIANTS) + 0.5) / SEGMENT_BAND_VARIANTS;
        let band_v = min(geo_height * segment_count / SEGMENT_MAX_COUNT, 0.999);
        tex_uv = vec2<f32>(band_u, band_v);
    } else {
        tex_uv = vec2<f32>(vertex_no_morph.uv.x, 1.0 - geo_height);
    }

    // Emit outputs compatible with Bevy VertexOutput
    out.uv = tex_uv;
    // Pack: uv_b.x = color_wave shimmer, uv_b.y = snow_blend (from vertex UV0.y)
    out.uv_b = vec2<f32>(color_wave, vertex_no_morph.uv.y);
    out.world_normal = vec3<f32>(0.0, 1.0, 0.0);
    // Pack geo_height in out.color.a so fragment shader can read it for gradient
    out.color = vec4<f32>(vertex_no_morph.color.rgb, geo_height);

    return out;
}

fn hash2_noise(p: vec2<f32>) -> f32 {
    let p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    let p4 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p4.x + p4.y) * p4.z);
}

fn smooth_noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash2_noise(i + vec2<f32>(0.0, 0.0));
    let b = hash2_noise(i + vec2<f32>(1.0, 0.0));
    let c = hash2_noise(i + vec2<f32>(0.0, 1.0));
    let d = hash2_noise(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn ambient_environment_noise(pos_xz: vec2<f32>) -> f32 {
    return smooth_noise2(pos_xz * 0.04);
}

@fragment
fn fragment(
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let height = clamp(vertex_output.color.a, 0.0, 1.0);
    let color_wave = clamp(vertex_output.uv_b.x, 0.0, 1.0);
    let snow_amount = clamp(vertex_output.uv_b.y, 0.0, 1.0);

    let shape = textureSample(shape_texture, shape_sampler, vertex_output.uv);

    if (shape.a < 0.1) {
        discard;
    }

    // Match the grass block surface texture baseline (soil_grass.png mean luminance ~0.5395)
    // grass_bands.png has mean ~0.745; grass_tapered.png has mean 1.0
    let shape_texture_factor = select(0.5395, 0.5395 / 0.745, grass_info.style_params.x > 0.5);
    let shape_tex = shape.rgb * shape_texture_factor;

    // Ambient environment noise matching voxel_opaque.wesl exactly
    let ambient_noise = ambient_environment_noise(vertex_output.world_position.xz);
    let ambient_factor = 0.92 + ambient_noise * 0.16;

    // Subtle blade gradient: grounding slightly at the base, gentle catching of light at the tip
    let gradient = mix(0.96, 1.04, pow(height, 1.5));
    let wave_mult = mix(0.96, 1.04, color_wave);

    // Voxel opaque shader squares the tint color: pbr_input.base_color (which has vertex_color) * tint_color.rgb
    let block_tint = vertex_output.color.rgb * vertex_output.color.rgb;

    var blade_rgb = block_tint * ambient_factor * shape_tex * gradient * wave_mult;
    blade_rgb = mix(blade_rgb, vec3<f32>(1.0), snow_amount);
    blade_rgb = clamp(blade_rgb * grass_info.style_params.w, vec3<f32>(0.0), vec3<f32>(1.0));

    var pbr_input: pbr_types::PbrInput = pbr_input_from_standard_material(vertex_output, is_front);
    pbr_input.N = vec3<f32>(0.0, 1.0, 0.0);
    pbr_input.world_normal = vec3<f32>(0.0, 1.0, 0.0);
    pbr_input.material.base_color = vec4<f32>(blade_rgb, shape.a);
    pbr_input.material.perceptual_roughness = 0.90;
    pbr_input.material.reflectance = vec3<f32>(0.5);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
