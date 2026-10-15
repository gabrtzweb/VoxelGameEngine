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

// Fast deterministic integer block face hash (Murmur3 style bit avalanche)
fn hash_block_face(p: vec3<i32>, normal: vec3<f32>) -> u32 {
    var face_id = 0u;
    if (abs(normal.y) > 0.5) {
        if (normal.y > 0.0) { face_id = 0u; } else { face_id = 1u; }
    } else if (abs(normal.x) > 0.5) {
        if (normal.x > 0.0) { face_id = 2u; } else { face_id = 3u; }
    } else {
        if (normal.z > 0.0) { face_id = 4u; } else { face_id = 5u; }
    }

    var h = (u32(p.x) * 73856093u) ^ (u32(p.y) * 19349663u) ^ (u32(p.z) * 83492791u) ^ (face_id * 2654435761u);
    h = h ^ (h >> 16u);
    h = h * 0x45d9f3bu;
    h = h ^ (h >> 16u);
    return h;
}

@fragment
fn fragment(
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input: pbr_types::PbrInput = pbr_input_from_standard_material(vertex_output, is_front);

    let base_layer = vertex_output.uv_b.x;
    let raw_frame_param = vertex_output.uv_b.y;

    var layer = i32(round(base_layer));
    let is_emissive = raw_frame_param < -0.5;
    let abs_frame_param = abs(raw_frame_param);
    let actual_frame_count = floor(abs_frame_param + 0.1);
    let rot_mode = i32(round((abs_frame_param - actual_frame_count) * 4.0));

    if (actual_frame_count > 1.5) {
        let fps = 6.0;
        let count = max(1, i32(round(actual_frame_count)));
        let frame = (i32(floor(max(globals.time, 0.0) * fps)) % count + count) % count;
        layer = layer + frame;
    }

    var uv = fract(vertex_output.uv);

    // Natural blocks pseudo-random rotation and flipping (OptiFine style)
    if (rot_mode > 0) {
        let normal = vertex_output.world_normal;
        let is_horizontal = abs(normal.y) > 0.5;

        // Deterministic integer voxel coordinates (offset inward against normal to stay strictly inside the voxel)
        let block_pos = vec3<i32>(floor(vertex_output.world_position.xyz - normal * 0.1));
        let hash = hash_block_face(block_pos, normal);

        if (is_horizontal) {
            // Horizontal faces (Top / Bottom): 4-way 90° step rotation + optional mirror
            let rot = hash & 3u;
            var rot_uv: vec2<f32>;
            switch (rot) {
                case 1u: { rot_uv = vec2<f32>(1.0 - uv.y, uv.x); }
                case 2u: { rot_uv = vec2<f32>(1.0 - uv.x, 1.0 - uv.y); }
                case 3u: { rot_uv = vec2<f32>(uv.y, 1.0 - uv.x); }
                default: { rot_uv = uv; }
            }
            if ((hash & 4u) != 0u) {
                rot_uv.x = 1.0 - rot_uv.x;
            }
            uv = clamp(rot_uv, vec2<f32>(0.0), vec2<f32>(0.99999));
        } else {
            // Vertical side faces (Walls)
            if (rot_mode == 1) {
                // Natural isotropic stone, rock, sand, gravel: 4-way 90° step rotation + flip
                let rot = hash & 3u;
                var rot_uv: vec2<f32>;
                switch (rot) {
                    case 1u: { rot_uv = vec2<f32>(1.0 - uv.y, uv.x); }
                    case 2u: { rot_uv = vec2<f32>(1.0 - uv.x, 1.0 - uv.y); }
                    case 3u: { rot_uv = vec2<f32>(uv.y, 1.0 - uv.x); }
                    default: { rot_uv = uv; }
                }
                if ((hash & 4u) != 0u) {
                    rot_uv.x = 1.0 - rot_uv.x;
                }
                uv = clamp(rot_uv, vec2<f32>(0.0), vec2<f32>(0.99999));
            } else {
                // Soils with grass/snow or directional side features:
                // Only horizontal flipping (mirror U) so textures don't appear upside-down or sideways
                if ((hash & 1u) != 0u) {
                    uv.x = clamp(1.0 - uv.x, 0.0, 0.99999);
                }
            }
        }
    }

    let tex_color = textureSample(voxel_texture_array, voxel_sampler, uv, layer);

    // Alpha Cutout: Discard see-through pixels (e.g. foliage leaves, alpha-cutout quads)
    if (tex_color.a < 0.5) {
        discard;
    }

    // Ambient Environment modulation: breaks up monotonous large expanses of grass and leaves (foliage has alpha > 0.75)
    var tint_color = vertex_output.color;
    let is_foliage_tinted = (vertex_output.color.a > 0.75) && (tint_color.r < 0.99 || tint_color.g < 0.99 || tint_color.b < 0.99);
    if (is_foliage_tinted) {
        let noise = ambient_environment_noise(vertex_output.world_position.xz);
        // Subtle organic variation: ±8% darker and lighter natural patches
        let ambient_factor = 0.92 + noise * 0.16;
        tint_color = vec4<f32>(tint_color.rgb * ambient_factor, tint_color.a);
    }

    pbr_input.material.base_color = tex_color * pbr_input.material.base_color * vec4<f32>(tint_color.rgb, 1.0);

    var out: FragmentOutput;
    if (is_emissive) {
        // Light-emitting blocks are self-illuminated:
        out.color = vec4<f32>(tex_color.rgb * vertex_output.color.rgb * 1.15, 1.0);
    } else {
        out.color = apply_pbr_lighting(pbr_input);
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
