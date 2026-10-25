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
    style_params: vec4<f32>,      // x = style, y = wind intensity, z = blade visual height, w = grass brightness
    gradient_params: vec4<f32>,   // x = base multiplier, y = tip multiplier, z = curve, w = height variation scale
    trail_params2: vec4<f32>,     // xy = trail noise origin, zw = reserved
    entity_pushers: array<vec4<f32>, 8>, // pusher 0: x = trail origin x, y = trail origin z, z = inverse world size, w = trail strength
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> grass_info: GrassInfo;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var plant_texture_array: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var plant_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var noise_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var noise_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var trail_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(106) var trail_sampler: sampler;

const PLANT_SWAY_STRENGTH: f32 = 0.32;
const PLANT_SWAY_BASE_LEAN: f32 = 0.4;
const PLANT_SWAY_GUST_RANGE: f32 = 0.8;
const PLANT_BOB_STRENGTH: f32 = 0.06;
const PLANT_IDLE_BOB_STRENGTH: f32 = 0.14;
const WIND_FIELD_SCALE: f32 = 0.5;
const TRAIL_WIND_SUPPRESSION: f32 = 0.95;
const NOISE_TILE: f32 = 32.0;
const PLANT_TRAIL_PUSH: f32 = 1.2;

fn sample_smooth_noise(uv: vec2<f32>) -> f32 {
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

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
    var world_pos = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex_no_morph.position, 1.0)
    ).xyz;

    let height = clamp(vertex_no_morph.normal.z, 0.0, 1.0);
    let noise_coord = vertex_no_morph.normal.xy * 0.5 + 0.5;

    let wind_strength = grass_info.style_params.y;
    let bob_strength = max(wind_strength, PLANT_IDLE_BOB_STRENGTH);

    var wind_dir = grass_info.wind_params.xy;
    if (length(wind_dir) > 0.001) {
        wind_dir = normalize(wind_dir);
    } else {
        wind_dir = vec2<f32>(1.0, 0.0);
    }

    let trail_params = grass_info.entity_pushers[0];
    let vertex_trail = world_pos.xz - trail_params.xy;
    let center_mod = (NOISE_TILE * noise_coord - grass_info.trail_params2.xy) % NOISE_TILE;
    let plant_center = vertex_trail + ((center_mod - vertex_trail + 0.5 * NOISE_TILE) % NOISE_TILE) - 0.5 * NOISE_TILE;

    var trail_dir = wind_dir;
    var trail_strength: f32 = 0.0;
    var trail_wind_mult: f32 = 1.0;

    if (trail_params.w > 0.0) {
        let trail_uv = plant_center * trail_params.z;
        let inside = step(vec2<f32>(0.0), trail_uv) * step(trail_uv, vec2<f32>(1.0));
        let mask = inside.x * inside.y;

        let sample = textureSampleLevel(trail_texture, trail_sampler, trail_uv, 0.0);
        let sampled_dir = sample.rg * 2.0 - 1.0;
        let sampled_len = length(sampled_dir);
        if (sampled_len > 0.001) {
            trail_dir = sampled_dir / sampled_len;
        }
        trail_strength = sample.b * mask * trail_params.w;
        let trail_effect = clamp(trail_strength, 0.0, 1.0);
        trail_wind_mult = 1.0 - trail_effect * TRAIL_WIND_SUPPRESSION;
    }

    let wind_field_uv = noise_coord * WIND_FIELD_SCALE + grass_info.wind_params2.xy;
    let wind_field = sample_smooth_noise(wind_field_uv);

    let lean = PLANT_SWAY_BASE_LEAN + wind_field * PLANT_SWAY_GUST_RANGE;
    let sway = lean * wind_strength * trail_wind_mult * height * height * PLANT_SWAY_STRENGTH;
    world_pos.x += wind_dir.x * sway;
    world_pos.z += wind_dir.y * sway;

    let cross_wind_dir = vec2<f32>(-wind_dir.y, wind_dir.x);
    let phase = dot(noise_coord, vec2<f32>(97.0, 151.0));
    let bob = sin(grass_info.wind_params.w + phase) * PLANT_BOB_STRENGTH * height * height;
    world_pos.x += cross_wind_dir.x * (bob * bob_strength * trail_wind_mult);
    world_pos.z += cross_wind_dir.y * (bob * bob_strength * trail_wind_mult);

    world_pos.x += trail_dir.x * (trail_strength * height * height * PLANT_TRAIL_PUSH);
    world_pos.z += trail_dir.y * (trail_strength * height * height * PLANT_TRAIL_PUSH);

    out.world_position = vec4<f32>(world_pos, 1.0);
    out.position = position_world_to_clip(world_pos);
    out.uv = vertex_no_morph.uv;
    out.world_normal = vertex_no_morph.normal;
    out.color = vertex_no_morph.color;

    return out;
}

@fragment
fn fragment(
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let layer = i32(round(vertex_output.color.a * 255.0));
    var tex_color = textureSample(plant_texture_array, plant_sampler, vertex_output.uv, layer);

    if (tex_color.a < 0.1) {
        discard;
    }

    var plant_rgb = tex_color.rgb * vertex_output.color.rgb;
    plant_rgb = clamp(plant_rgb * grass_info.style_params.w, vec3<f32>(0.0), vec3<f32>(1.0));

    var pbr_input: pbr_types::PbrInput = pbr_input_from_standard_material(vertex_output, is_front);
    pbr_input.material.base_color = vec4<f32>(plant_rgb, tex_color.a);
    pbr_input.material.perceptual_roughness = 0.9;
    pbr_input.material.reflectance = vec3<f32>(0.1);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
