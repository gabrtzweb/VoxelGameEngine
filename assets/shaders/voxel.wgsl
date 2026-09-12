#import bevy_pbr::{
    pbr_types,
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::globals,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var voxel_sampler: sampler;

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

    pbr_input.material.base_color = tex_color * pbr_input.material.base_color * vertex_output.color;

    if (frame_count > 1.5) {
        pbr_input.material.base_color.a = max(pbr_input.material.base_color.a, 0.72);
        let water_tint = tex_color.rgb * vertex_output.color.rgb;
        pbr_input.material.emissive = vec4<f32>(water_tint * 0.55, 1.0);
    }

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
