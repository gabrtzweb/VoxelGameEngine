import bevy_pbr::render::{
    mesh_functions,
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    view_transformations::position_world_to_clip,
    pbr_types,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::{globals, lights},
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var voxel_texture_array: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var voxel_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var noise_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var noise_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var<uniform> voxel_params: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var<uniform> anim_params: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(106) var mask_texture_array: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(107) var mask_sampler: sampler;

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

fn ambient_environment_variation(pos_xz: vec2<f32>, base_tint: vec3<f32>) -> vec3<f32> {
    let n_macro = smooth_noise2(pos_xz * 0.025);
    let n_detail = smooth_noise2(pos_xz * 0.10);
    let combined = n_macro * 0.7 + n_detail * 0.3;
    let brightness = 0.88 + combined * 0.24;
    let warm_factor = clamp((0.48 - combined) * 2.2, -0.35, 0.45);
    var shifted = base_tint * brightness;
    shifted.r += warm_factor * 0.12;
    shifted.g += warm_factor * 0.03;
    shifted.b -= warm_factor * 0.08;
    return clamp(shifted, vec3<f32>(0.02), vec3<f32>(1.0));
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

// Safe tile UV calculation:
// Tiles raw UVs repeating [0.0005, 0.9995] per block without ever wrapping
// across integer boundaries. When raw_uv reaches an integer boundary (>= 1.0),
// it is clamped to 0.9995 (texel 15, the dirt bottom or block edge), preventing
// row 0 (snow/grass) from bleeding into the bottom of vertical faces.
fn safe_tile_uv(raw_uv: vec2<f32>) -> vec2<f32> {
    let cell = floor(raw_uv);
    var frac = raw_uv - cell;
    if (frac.x < 0.001 && raw_uv.x > 0.5) {
        frac.x = 0.9995;
    }
    if (frac.y < 0.001 && raw_uv.y > 0.5) {
        frac.y = 0.9995;
    }
    return clamp(frac, vec2<f32>(0.0005), vec2<f32>(0.9995));
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

// Screen-Space Texel Offset & Snapping
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
    return val + clamp(val_offset, vec3<f32>(-0.0625), vec3<f32>(0.0625));
}

@fragment
fn fragment(
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_vertex = vertex_output;
    pbr_vertex.color = vec4<f32>(1.0);
    var pbr_input: pbr_types::PbrInput = pbr_input_from_standard_material(pbr_vertex, is_front);

    let abs_frame_full = abs(vertex_output.uv_b.y);
    let mask_index = i32(floor(abs_frame_full / 16.0 + 0.01));
    let raw_frame_param = sign(vertex_output.uv_b.y) * (abs_frame_full - f32(mask_index) * 16.0);

    // Connected-texture faces: vertex color rgb = AO only, alpha = -(1 + packed) where
    // packed = tint(7:7:7) | base_is_tinted << 21 | uv_swapped << 22 (see greedy.rs push_quad).
    let is_mask_face = mask_index > 0 && vertex_output.color.a < -0.5;
    var ct_packed = 0u;
    if (is_mask_face) {
        ct_packed = u32(round(-vertex_output.color.a - 1.0));
    }
    let ct_tint_bits = ct_packed & 0x1FFFFFu;
    let ct_base_tinted = ((ct_packed >> 21u) & 1u) == 1u;
    let ct_uv_swapped = ((ct_packed >> 22u) & 1u) == 1u;
    let ct_tint = vec3<f32>(
        f32((ct_tint_bits >> 14u) & 127u),
        f32((ct_tint_bits >> 7u) & 127u),
        f32(ct_tint_bits & 127u)
    ) / 127.0;

    let raw_base = round(vertex_output.uv_b.x);
    let base_layer = i32(raw_base) % 2048;
    let overlay_layer = i32(raw_base) / 2048;

    var layer = base_layer;
    var next_layer = layer;
    var frame_blend = 0.0;
    let is_emissive = raw_frame_param < -0.5;
    let abs_frame_param = abs(raw_frame_param);
    let actual_frame_count = floor(abs_frame_param + 0.1);
    let rot_mode = i32(round((abs_frame_param - actual_frame_count) * 4.0));

    if (actual_frame_count > 1.5) {
        let fps = 8.0;
        let count = max(1, i32(round(actual_frame_count)));
        let t = max(globals.time, 0.0) * fps;
        let frame_cur = (i32(floor(t)) % count + count) % count;
        let frame_next = (frame_cur + 1) % count;
        frame_blend = fract(t);
        layer = layer + frame_cur;
        next_layer = base_layer + frame_next;
    }

    let normal = vertex_output.world_normal;
    let is_horizontal = abs(normal.y) > 0.5;
    let block_pos = vec3<i32>(floor(vertex_output.world_position.xyz - normal * 0.1));

    var uv: vec2<f32>;
    var mask_uv: vec2<f32>;
    if (is_mask_face) {
        // Connected texture faces are strictly 1x1 quads with normalized [eps, 1-eps] coordinates.
        // No fract or modulo wrapping: mask_uv and uv are guaranteed within [0.0005, 0.9995].
        uv = clamp(vertex_output.uv, vec2<f32>(0.0005), vec2<f32>(0.9995));
        mask_uv = uv;
    } else {
        uv = safe_tile_uv(vertex_output.uv);
        mask_uv = uv;
    }

    // Natural blocks pseudo-random rotation and flipping (OptiFine style)
    if (rot_mode > 0 && !is_mask_face) {
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

    // Organic Connected Textures: Mask-Overlay blending.
    // Base and overlay are tinted independently with exactly the same formula the regular
    // (non-transition) path applies to that block, so spilled grass matches the grass block and
    // snow spilled over grass stays white. Vertex color rgb holds only AO on these faces.
    if (is_mask_face) {
        let mask_layer = mask_index - 1;
        let mask_sample = textureSample(mask_texture_array, mask_sampler, mask_uv, mask_layer);

        let ao = vertex_output.color.r;
        let ambient_factor = 0.92 + ambient_environment_noise(vertex_output.world_position.xz) * 0.16;
        // Regular path: tinted (foliage) colors get vc * vc * ambient with vc = tint * ao;
        // untinted colors get ao * ao * ambient only when AO darkens the vertex, else 1.0.
        let tinted_mul = ct_tint * ct_tint * ao * ao * ambient_factor;
        var white_mul = vec3<f32>(1.0);
        if (ao < 0.99) {
            white_mul = vec3<f32>(ao * ao * ambient_factor);
        }
        let has_tint = ct_tint_bits != 0u;
        var base_mul = white_mul;
        var overlay_mul = white_mul;
        if (has_tint && ct_base_tinted) {
            base_mul = tinted_mul;
        } else if (has_tint) {
            overlay_mul = tinted_mul;
        }

        // Overlay color source:
        // Mask layers:
        //   0..=16:  Grass (pure white mask -> sample voxel_texture_array with overlay_layer + biome tint)
        //  17..=33:  Sand  (pure white mask -> sample voxel_texture_array with overlay_layer)
        //  34..=50:  Rocks (pure white mask -> sample voxel_texture_array with overlay_layer)
        //  51..=67:  Snow  (pre-colored mask -> mask_sample.rgb already contains snow colors)
        //  68..=84:  Moss  (pre-colored mask -> mask_sample.rgb already contains moss colors)
        var overlay_rgb = tex_color.rgb * base_mul;
        if (mask_layer >= 51) {
            overlay_rgb = mask_sample.rgb * overlay_mul;
            let blended = mix(tex_color.rgb * base_mul, overlay_rgb, mask_sample.a);
            tex_color = vec4<f32>(blended, 1.0);
        } else if (overlay_layer > 0) {
            let sampled_overlay = textureSample(voxel_texture_array, voxel_sampler, mask_uv, overlay_layer);
            if (sampled_overlay.a > 0.1) {
                overlay_rgb = sampled_overlay.rgb * overlay_mul;
                let blended = mix(tex_color.rgb * base_mul, overlay_rgb, mask_sample.a);
                tex_color = vec4<f32>(blended, 1.0);
            }
        }
    }

    // Alpha Cutout: Discard see-through pixels (e.g. foliage leaves, alpha-cutout quads)
    // NEVER discard solid blocks or connected-texture transition quads
    if (tex_color.a < 0.5 && !is_mask_face) {
        discard;
    }

    // Dynamic item / held-block tag: vertex_output.color.a > 6.5 (7.0 = normal/tinted, 7.6 = painted).
    // Held blocks and dynamic 3D item drops rotate freely in space, so they must not use world-space
    // normal orientation checks, world-space coated noise projection, or screen-space texel snapping.
    let is_item_entity = vertex_output.color.a > 6.5;

    // Cross-billboard / fixture cutout model detection:
    // 1) Explicit vertex anim/model tag: 5.0 (flora) or 6.0 (rigid cross/speleothem/crystal)
    // 2) Geometric normal vs vertex normal: cross billboards are vertical planes (geo normal Y ~ 0)
    //    with an upward lighting normal (vertex normal Y = 1).
    let dp_x = dpdx(vertex_output.world_position.xyz);
    let dp_y = dpdy(vertex_output.world_position.xyz);
    let geo_norm = cross(dp_x, dp_y);
    let geo_len_sq = dot(geo_norm, geo_norm);
    let is_diagonal_plane = !is_item_entity && geo_len_sq > 0.00001 && (abs(geo_norm.x) * abs(geo_norm.z) > 0.05 * geo_len_sq);
    let is_cross_mesh = (vertex_output.color.a > 3.5 && vertex_output.color.a < 6.5) || is_diagonal_plane;

    // Ambient Environment modulation (Minecraft mod style):
    // Breaks up monotonous large expanses of grass and leaves with multi-scale noise and natural hue shifts
    var tint_color = vertex_output.color;
    if (is_mask_face) {
        tint_color = vec4<f32>(1.0);
    }
    let is_painted_block = ((vertex_output.color.a > 0.52) && (vertex_output.color.a < 0.72))
        || ((vertex_output.color.a > 7.4) && (vertex_output.color.a < 7.8));
    let is_foliage_tinted = !is_mask_face && !is_painted_block && (vertex_output.color.a > 0.75) && (tint_color.r < 0.99 || tint_color.g < 0.99 || tint_color.b < 0.99);
    if (is_foliage_tinted && !is_item_entity) {
        let varied_tint = ambient_environment_variation(vertex_output.world_position.xz, tint_color.rgb * vertex_output.color.rgb);
        tint_color = vec4<f32>(varied_tint, tint_color.a);
    }

    // Dynamic grass side fringe tinting:
    // On vertical side faces, only tint the green grass fringe hanging down so it matches the top face,
    // leaving natural dirt and stone underneath completely untouched.
    // On held items and 3D drops (is_item_entity), per-face vertex colors are already assigned directly
    // (top face = biome tint, side/bottom faces = white with pre-baked side overlay), so we preserve
    // final_tint regardless of how the player pitches or rolls the camera!
    var final_tint = tint_color.rgb;
    if (!is_mask_face && !is_horizontal && !is_item_entity) {
        let is_grass_fringe = (uv.y < 0.55) && (tex_color.g > tex_color.r * 1.02) && (tex_color.g > tex_color.b * 1.15);
        if (is_grass_fringe && is_foliage_tinted) {
            let fringe_lum = dot(tex_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
            let tinted_fringe = fringe_lum * 1.45 * tint_color.rgb;
            tex_color = vec4<f32>(tinted_fringe, tex_color.a);
        }
        final_tint = vec3<f32>(1.0);
    }

    // Coated Textures (custom style):
    // Samples authentic noise.png seamlessly across continuous world-space coordinates.
    // Zero block-edge grid lines; natural organic weathering and patina.
    // Disabled on held items / 3D drops so moving the camera doesn't slide a reflection/noise grid across the block.
    let is_solid_terrain = (tex_color.a >= 0.99) && !is_emissive && !is_item_entity;
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

    if (is_painted_block) {
        // Match the 2D inventory icon sRGB dye blend 1:1 before converting back to linear PBR space:
        let tex_srgb = pow(max(tex_color.rgb, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
        let highlight = clamp((tex_srgb - vec3<f32>(0.78)) / 0.22, vec3<f32>(0.0), vec3<f32>(1.0));
        let painted_srgb = tex_srgb * tint_color.rgb + highlight * 0.08 * max(vec3<f32>(1.0) - tint_color.rgb, vec3<f32>(0.0));
        let painted_linear = pow(clamp(painted_srgb, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(2.2));
        pbr_input.material.base_color = vec4<f32>(painted_linear * dir_shading, 1.0);
    } else {
        pbr_input.material.base_color = tex_color * vec4<f32>(final_tint * dir_shading, 1.0);
    }

    // Pixelated Shadows & Shadow Stabilization:
    // Snaps world position to 16x16 texel centers along screen derivative tangents on solid block faces.
    // Cross meshes and dynamic item meshes are exempted so non-orthogonal screen derivatives don't flicker surface lighting.
    let pixelated_shadows_enabled = voxel_params.z > 0.5;
    if (pixelated_shadows_enabled && !is_cross_mesh && !is_item_entity) {
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
        let texel_size = vec4<f32>(1.0 / 16.0, 1.0 / 16.0, 16.0, 16.0);
        let texel_offset = compute_texel_offset(planar_coord, texel_size);
        pbr_input.world_position = vec4<f32>(
            texel_snap_vec3(pbr_input.world_position.xyz, texel_offset),
            pbr_input.world_position.w
        );
    }

    if (is_cross_mesh) {
        pbr_input.world_normal = vec3<f32>(0.0, 1.0, 0.0);
        pbr_input.N = vec3<f32>(0.0, 1.0, 0.0);
        // Disable shadow receiving on crossmesh billboard fragments:
        // Paper-thin 2D diagonal quads cannot physically self-shadow. Clearing MESH_FLAGS_SHADOW_RECEIVER_BIT
        // (1u << 29u) ensures crossmeshes never evaluate shadow map depth on themselves, completely eliminating
        // shadow acne, pulsing vertical stripes, and top-edge 'X' artifacts. Meanwhile, voxel_prepass still writes
        // crossmesh geometry into the shadow map so they cast full, sharp shadows onto the ground and solid terrain.
        pbr_input.flags = pbr_input.flags & ~(1u << 29u);
    }

    var out: FragmentOutput;
    if (is_emissive) {
        // Light-emitting blocks are self-illuminated into HDR bloom range (> 1.0):
        out.color = vec4<f32>(tex_color.rgb * vertex_output.color.rgb * 2.5, 1.0);
    } else {
        out.color = apply_pbr_lighting(pbr_input);
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
