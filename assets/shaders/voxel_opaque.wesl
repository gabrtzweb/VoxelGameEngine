import bevy_pbr::render::{
    pbr_types,
    pbr_fragment::pbr_input_from_standard_material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::globals,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var voxel_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var noise_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var noise_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var<uniform> voxel_params: vec4<f32>;

// Fast procedural noise using Dave Hoskins' sine-free hash (ultra-fast on integrated GPUs)
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
    var next_layer = layer;
    var frame_blend = 0.0;
    let is_emissive = raw_frame_param < -0.5;
    let abs_frame_param = abs(raw_frame_param);
    let actual_frame_count = floor(abs_frame_param + 0.1);
    let rot_mode = i32(round((abs_frame_param - actual_frame_count) * 4.0));

    if (actual_frame_count > 1.5) {
        let fps = 1.25;
        let count = max(1, i32(round(actual_frame_count)));
        let t = max(globals.time, 0.0) * fps;
        let frame_cur = (i32(floor(t)) % count + count) % count;
        let frame_next = (frame_cur + 1) % count;
        frame_blend = fract(t);
        layer = layer + frame_cur;
        next_layer = i32(round(base_layer)) + frame_next;
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
            // Vertical side faces (Walls):
            // Never rotate by 90° or 270° so stratified rocks and geology blocks never appear sideways.
            // Only allow horizontal mirroring (flip U) to break tiling while preserving horizontal orientation.
            if ((hash & 1u) != 0u) {
                uv.x = clamp(1.0 - uv.x, 0.0, 0.99999);
            }
        }
    }

    var tex_color = textureSample(voxel_texture_array, voxel_sampler, uv, layer);

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

    // Coated Textures (Complementary Reimagined reverse-engineered style):
    // Samples authentic noise.png seamlessly across continuous world-space coordinates.
    // Zero block-edge grid lines; natural organic weathering and patina.
    let is_solid_terrain = (tex_color.a >= 0.99) && !is_emissive;
    let coat_strength = voxel_params.x;
    if (is_solid_terrain && coat_strength > 0.01) {
        let normal = normalize(vertex_output.world_normal);
        let abs_n = abs(normal);
        var planar_coord: vec2<f32>;
        if (abs_n.y > 0.5) {
            planar_coord = vertex_output.world_position.xz;
        } else if (abs_n.x > 0.5) {
            planar_coord = vertex_output.world_position.zy;
        } else {
            planar_coord = vertex_output.world_position.xy;
        }

        // Snap to 16-texels-per-meter global grid (pixel-perfect with block resolution, seamless across blocks):
        let texel_world = floor(planar_coord * 16.0) / 16.0;
        let noise_uv = fract(texel_world * (1.0 / 8.0));
        let noise_sample = textureSample(noise_texture, noise_sampler, noise_uv);
        let combined_noise = noise_sample.r * 0.75 + noise_sample.g * 0.25;
        let coat_factor = mix(1.0, 0.72 + combined_noise * 0.56, coat_strength);
        tex_color = vec4<f32>(tex_color.rgb * coat_factor, tex_color.a);
    }

    // Directional shading floor (Minecraft standard: Top 1.0, Bottom 0.55, N/S 0.86, E/W 0.78):
    // Preserves clear geometric distinction across faces under diffuse ambient light.
    let norm = normalize(vertex_output.world_normal);
    var dir_shading = 1.0;
    if (abs(norm.y) > 0.5) {
        if (norm.y < 0.0) {
            dir_shading = 0.55;
        }
    } else if (abs(norm.z) > 0.5) {
        dir_shading = 0.86;
    } else {
        dir_shading = 0.78;
    }

    pbr_input.material.base_color = tex_color * pbr_input.material.base_color * vec4<f32>(tint_color.rgb * dir_shading, 1.0);

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
