#import bevy_pbr::{
    pbr_types,
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::globals,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var voxel_sampler: sampler;

// Procedural ambient color noise ("Ambient Environment" mod style)
fn hash2_noise(p: vec2<f32>) -> f32 {
    let q = fract(sin(vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)), dot(p, vec2<f32>(269.5, 183.3)))) * 43758.5453);
    return fract(q.x + q.y);
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
    // 2-octave smooth noise: broad undulating patches (freq ~0.04) and gentle local depth (freq ~0.09)
    let n1 = smooth_noise2(pos_xz * 0.04);
    let n2 = smooth_noise2(pos_xz * 0.09 + vec2<f32>(17.3, 31.7));
    return n1 * 0.7 + n2 * 0.3;
}

@fragment
fn fragment(
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input: pbr_types::PbrInput = pbr_input_from_standard_material(vertex_output, is_front);

    let uv = fract(vertex_output.uv);
    let base_layer = vertex_output.uv_b.x;
    let frame_count = vertex_output.uv_b.y;

    var layer = i32(round(base_layer));
    if (frame_count > 1.5) {
        let fps = 6.0;
        let count = max(1, i32(round(frame_count)));
        let frame = (i32(floor(max(globals.time, 0.0) * fps)) % count + count) % count;
        layer = layer + frame;
    }

    let tex_color = textureSample(voxel_texture_array, voxel_sampler, uv, layer);

    // Alpha Cutout: Discard see-through pixels (e.g. foliage leaves, alpha-cutout quads)
    if (tex_color.a < 0.5) {
        discard;
    }

    // Ambient Environment modulation: breaks up monotonous large expanses of grass, leaves, and water
    var tint_color = vertex_output.color;
    let is_tinted = (tint_color.r < 0.99 || tint_color.g < 0.99 || tint_color.b < 0.99);
    if (is_tinted || frame_count > 1.5) {
        let noise = ambient_environment_noise(vertex_output.world_position.xz);
        // Subtle organic variation: ±8% darker and lighter natural patches
        let ambient_factor = 0.92 + noise * 0.16;
        tint_color = vec4<f32>(tint_color.rgb * ambient_factor, tint_color.a);
    }

    pbr_input.material.base_color = tex_color * pbr_input.material.base_color * tint_color;

    var out: FragmentOutput;
    if (frame_count < -0.5) {
        // Light-emitting blocks are self-illuminated:
        // Always maintains full, vivid texture visibility day and night,
        // unaffected by external shadows or darkness.
        out.color = vec4<f32>(tex_color.rgb * vertex_output.color.rgb * 1.15, 1.0);
    } else {
        if (frame_count > 1.5) {
            pbr_input.material.base_color.a = max(pbr_input.material.base_color.a, 0.72);
            let water_tint = tex_color.rgb * tint_color.rgb;
            pbr_input.material.emissive = vec4<f32>(water_tint * 0.55, 1.0);
        }
        out.color = apply_pbr_lighting(pbr_input);
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
