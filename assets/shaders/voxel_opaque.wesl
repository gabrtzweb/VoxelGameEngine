import bevy_pbr::render::{
    mesh_functions,
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    view_transformations::position_world_to_clip,
    pbr_types,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::globals,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var voxel_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var noise_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var noise_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var<uniform> voxel_params: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var<uniform> anim_params: vec4<f32>;

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

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

@if(VERTEX_NORMALS)
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex_no_morph.normal,
        vertex_no_morph.instance_index
    );
@else
    out.world_normal = vec3<f32>(0.0, 1.0, 0.0);

@if(VERTEX_POSITIONS) {
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
            // Type 3: Tree Leaves (Complementary-style ambient canopy rustle)
            // Subtle, continuous spatial wave so voxel edges remain tightly cohesive:
            let leaf_wave = sin(time * 2.5 + pos.x * 0.75 + pos.y * 0.60 + pos.z * 0.75) * 0.020 * wind_intensity;
            let leaf_flutter = cos(time * 4.4 + pos.x * 2.2 - pos.z * 1.9) * 0.009 * wind_intensity;

            out.world_position.x += (wind_dir.x * leaf_wave + leaf_flutter);
            out.world_position.z += (wind_dir.y * leaf_wave - leaf_flutter);
            out.world_position.y += sin(time * 1.9 + pos.x + pos.z) * 0.010 * wind_intensity;
        }
    }

    out.position = position_world_to_clip(out.world_position.xyz);
}

@if(VERTEX_UVS_A)
    out.uv = vertex_no_morph.uv;
@if(VERTEX_UVS_B)
    out.uv_b = vertex_no_morph.uv_b;
@if(VERTEX_COLORS)
    out.color = vertex_no_morph.color;
@if(VERTEX_OUTPUT_INSTANCE_INDEX)
    out.instance_index = vertex_no_morph.instance_index;

    return out;
}

// Screen-Space Texel Offset & Snapping (Complementary Reimagined reverse-engineered)
// Projects UV derivatives into screen space to compute sub-pixel offset to the nearest
// 16x16 texel center, snapping world position to eliminate shadow crawling and jitter.
fn compute_texel_offset(uv: vec2<f32>, texel_size: vec4<f32>) -> vec2<f32> {
    let d_uv = (floor(uv * texel_size.zw) + 0.5) * texel_size.xy - uv;
    let duv_ds = dpdx(uv);
    let duv_dt = dpdy(uv);
    let a = dot(duv_ds, duv_ds);
    let b = dot(duv_ds, duv_dt);
    let c = dot(duv_dt, duv_dt);
    let det = a * c - b * b;
    if (det <= a * c * 0.01) {
        return vec2<f32>(0.0);
    }
    let inv = 1.0 / det;
    let proj = vec2<f32>(dot(d_uv, duv_ds), dot(d_uv, duv_dt));
    let offset = vec2<f32>(c * proj.x - b * proj.y, a * proj.y - b * proj.x) * inv;
    return clamp(offset, vec2<f32>(-8.0), vec2<f32>(8.0));
}

fn texel_snap_vec3(val: vec3<f32>, texel_offset: vec2<f32>) -> vec3<f32> {
    if (dot(texel_offset, texel_offset) < 0.000001) {
        return val;
    }
    let dx = dpdx(val);
    let dy = dpdy(val);
    let val_offset = dx * texel_offset.x + dy * texel_offset.y;
    return val + clamp(val_offset, vec3<f32>(-0.25), vec3<f32>(0.25));
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

    // Pixelated Shadows & Shadow Stabilization (Complementary Reimagined reverse-engineered):
    // Snaps world position to 16x16 texel centers along screen derivative tangents.
    // Eliminates sub-pixel shadow shimmering, crawling, and jitter while aligning shadows
    // strictly to the block texel grid.
    let pixelated_shadows_enabled = voxel_params.z > 0.5;
    if (pixelated_shadows_enabled) {
        let texel_size = vec4<f32>(1.0 / 16.0, 1.0 / 16.0, 16.0, 16.0);
        let texel_offset = compute_texel_offset(vertex_output.uv, texel_size);
        pbr_input.world_position = vec4<f32>(
            texel_snap_vec3(pbr_input.world_position.xyz, texel_offset),
            pbr_input.world_position.w
        );
    }

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
