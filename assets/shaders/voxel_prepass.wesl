import bevy_pbr::{
    render::{
        mesh_functions,
        view_transformations::position_world_to_clip,
    },
    prepass::io::{Vertex, VertexOutput, FragmentOutput},
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var voxel_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var<uniform> voxel_params: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var<uniform> anim_params: vec4<f32>;

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

@if(NORMAL_PREPASS_OR_DEFERRED_PREPASS) {
    @if(VERTEX_NORMALS) {
        out.world_normal = mesh_functions::mesh_normal_local_to_world(
            vertex_no_morph.normal,
            vertex_no_morph.instance_index
        );
    }
}

    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex_no_morph.position, 1.0)
    );

    let waving_enabled = voxel_params.y > 0.5;
    if (waving_enabled) {
        let time = max(anim_params.x, 0.0);
        let anim_type = round(vertex_no_morph.color.a);
        let wind_intensity = anim_params.y;
        let wind_angle = anim_params.z;
        let wind_dir = vec2<f32>(cos(wind_angle), sin(wind_angle));
        let pos = out.world_position.xyz;

        if (anim_type > 4.5 && anim_type < 5.5) {
            // Type 5: Terrestrial Flora & Flowers (Base-anchored wind sway)
            // UV.y is ~1.0 at root, ~0.0 at tip. Smoothly anchor root to 0 displacement:
            let height = smoothstep(0.1, 0.9, 1.0 - vertex_no_morph.uv.y);
            let h2 = height * height;

            // Travelling wind gusts + high-frequency flutter:
            let gust = sin(time * 2.2 + pos.x * 0.28 + pos.z * 0.22) * 0.5 + 0.5;
            let flutter = sin(time * 5.0 + pos.x * 1.5 + pos.z * 1.7) * 0.035;
            let sway = (gust * 0.09 + 0.03 + flutter) * h2 * wind_intensity;

            // Cross-wind gentle oscillation:
            let cross_sway = sin(time * 1.7 + pos.x * 0.6) * 0.025 * h2 * wind_intensity;

            out.world_position.x += wind_dir.x * sway - wind_dir.y * cross_sway;
            out.world_position.z += wind_dir.y * sway + wind_dir.x * cross_sway;
            out.world_position.y -= (abs(sway) * 0.15) * height;

        } else if (anim_type > 3.5 && anim_type < 4.5) {
            // Type 4: Aquatic Flora (Kelp, Seagrass)
            // Harmonically undulating wave mimicking viscous water currents:
            let height = smoothstep(0.1, 0.9, 1.0 - vertex_no_morph.uv.y);
            let current = sin(time * 1.4 + pos.y * 1.2 + pos.x * 0.35) * 0.055 * height;
            let cross = cos(time * 0.95 + pos.z * 0.55) * 0.030 * height;

            out.world_position.x += current * 0.85 + cross * 0.15;
            out.world_position.z += cross * 0.85 + current * 0.15;
            out.world_position.y -= abs(current) * 0.18 * height;

        } else if (anim_type > 2.5 && anim_type < 3.5) {
            // Type 3: Tree Leaves (ambient canopy rustle)
            // Subtle, continuous spatial wave so voxel edges remain tightly cohesive:
            let leaf_wave = sin(time * 2.5 + pos.x * 0.75 + pos.y * 0.60 + pos.z * 0.75) * 0.020 * wind_intensity;
            let leaf_flutter = cos(time * 4.4 + pos.x * 2.2 - pos.z * 1.9) * 0.009 * wind_intensity;

            out.world_position.x += (wind_dir.x * leaf_wave + leaf_flutter);
            out.world_position.z += (wind_dir.y * leaf_wave - leaf_flutter);
            out.world_position.y += sin(time * 1.9 + pos.x + pos.z) * 0.010 * wind_intensity;
        }
    }

    out.position = position_world_to_clip(out.world_position.xyz);

@if(VERTEX_UVS_A)
    out.uv = vertex_no_morph.uv;
@if(VERTEX_UVS_B)
    out.uv_b = vertex_no_morph.uv_b;
@if(VERTEX_COLORS)
    out.color = vertex_no_morph.color;

    return out;
}

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
@if(NORMAL_PREPASS)
    out.normal = vec4<f32>(in.world_normal * 0.5 + vec3<f32>(0.5), 1.0);
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
