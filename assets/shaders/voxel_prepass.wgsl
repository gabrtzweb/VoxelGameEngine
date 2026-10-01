#import bevy_pbr::prepass_io

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var voxel_sampler: sampler;

#ifdef PREPASS_FRAGMENT
@fragment
fn fragment(
    in: prepass_io::VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> prepass_io::FragmentOutput {
    let uv = fract(in.uv);
    let layer = i32(round(in.uv_b.x));

    let tex_color = textureSample(voxel_texture_array, voxel_sampler, uv, layer);
    if (tex_color.a < 0.5) {
        discard;
    }

    var out: prepass_io::FragmentOutput;
    return out;
}
#else
@fragment
fn fragment(in: prepass_io::VertexOutput) {
    let uv = fract(in.uv);
    let layer = i32(round(in.uv_b.x));

    let tex_color = textureSample(voxel_texture_array, voxel_sampler, uv, layer);
    if (tex_color.a < 0.5) {
        discard;
    }
}
#endif
