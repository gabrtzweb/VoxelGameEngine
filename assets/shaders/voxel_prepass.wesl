import bevy_pbr::prepass::io::{VertexOutput, FragmentOutput};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var voxel_sampler: sampler;

@if(PREPASS_FRAGMENT)
@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let uv = fract(in.uv);
    let layer = i32(round(in.uv_b.x));

    let tex_color = textureSample(voxel_texture_array, voxel_sampler, uv, layer);
    if (tex_color.a < 0.5) {
        discard;
    }

    var out: FragmentOutput;
    return out;
}
@else
@fragment
fn fragment(in: VertexOutput) {
    let uv = fract(in.uv);
    let layer = i32(round(in.uv_b.x));

    let tex_color = textureSample(voxel_texture_array, voxel_sampler, uv, layer);
    if (tex_color.a < 0.5) {
        discard;
    }
}
