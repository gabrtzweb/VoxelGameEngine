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
@group(constants::MATERIAL_BIND_GROUP) @binding(108) var<uniform> sun_dir_and_factor: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(109) var<uniform> sun_color_and_cloud: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(110) var<uniform> player_push_pos: vec4<f32>;

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

// Animated Voronoi cell-ridge caustic layer for projected underwater light webs
fn caustic_layer(p: vec2<f32>, t: f32) -> f32 {
    let i = floor(p);
    let f = fract(p);
    var min_d1 = 8.0;
    var min_d2 = 8.0;
    for (var y: i32 = -1; y <= 1; y = y + 1) {
        for (var x: i32 = -1; x <= 1; x = x + 1) {
            let nb = vec2<f32>(f32(x), f32(y));
            let h1 = hash2_noise(i + nb);
            let h2 = hash2_noise(i + nb + vec2<f32>(37.2, 17.9));
            let pt = nb + 0.5 + 0.36 * vec2<f32>(
                sin(t * 0.95 + h1 * 6.28318),
                cos(t * 0.85 + h2 * 6.28318)
            ) - f;
            let d = dot(pt, pt);
            if (d < min_d1) {
                min_d2 = min_d1;
                min_d1 = d;
            } else if (d < min_d2) {
                min_d2 = d;
            }
        }
    }
    let edge = sqrt(min_d2) - sqrt(min_d1);
    return smoothstep(0.22, 0.0, edge);
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
        let time = max(globals.time, 0.0);
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

            // Gentle player parting / push when walking through cross-mesh flora & crops
            if (player_push_pos.w > 0.5) {
                let d_xz = pos.xz - player_push_pos.xz;
                let dist = length(d_xz);
                let dy = abs(pos.y - (player_push_pos.y + 0.65));
                if (dist < 1.30 && dist > 0.001 && dy < 1.35) {
                    let push = smoothstep(1.30, 0.12, dist) * 0.42 * height;
                    let push_dir = d_xz / dist;
                    out.world_position.x += push_dir.x * push;
                    out.world_position.z += push_dir.y * push;
                    out.world_position.y -= push * 0.28;
                }
            }

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

    let raw_abs_uvb_y = abs(vertex_output.uv_b.y);
    let is_underwater_face = raw_abs_uvb_y >= 4000.0;
    let abs_frame_full = select(raw_abs_uvb_y, raw_abs_uvb_y - 4096.0, is_underwater_face);
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

    // Natural blocks pseudo-random rotation and flipping
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
        //  68..=84:  Moss  (alpha mask -> sample voxel_texture_array with overlay_layer)
        var overlay_rgb = tex_color.rgb * base_mul;
        if (mask_layer >= 51 && mask_layer <= 67) {
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

    // Distant LOD tree canopies (color.a ~ 2.0): render as solid opaque foliage
    // (blending transparent leaf holes into dark interior foliage) so they preserve
    // GPU Early-Z, avoid distant alpha-cutout shimmer, and skip wind animation.
    let is_lod_leaves = (vertex_output.color.a > 1.8) && (vertex_output.color.a < 2.2);
    if (is_lod_leaves && tex_color.a < 0.99) {
        tex_color = vec4<f32>(mix(vec3<f32>(0.22, 0.26, 0.20), tex_color.rgb, tex_color.a), 1.0);
    }

    // Alpha Cutout: Discard see-through pixels (e.g. foliage leaves, alpha-cutout quads)
    // NEVER discard solid blocks, LOD canopies, or connected-texture transition quads
    if (tex_color.a < 0.5 && !is_mask_face && !is_lod_leaves) {
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

    // Ambient environment color variation:
    // Breaks up monotonous large expanses of grass and leaves with multi-scale noise and natural hue shifts
    var tint_color = vertex_output.color;
    if (is_mask_face) {
        tint_color = vec4<f32>(1.0);
    }
    let is_painted_block = ((vertex_output.color.a > 0.52) && (vertex_output.color.a < 0.72))
        || ((vertex_output.color.a > 7.4) && (vertex_output.color.a < 7.8));
    // Only treat vertex colors with actual chroma (biome tint) as foliage-tinted;
    // pure greyscale AO on non-tinted blocks (e.g. melons, moss, stone) must never trigger grass fringe tinting.
    let has_chroma_tint = (abs(tint_color.r - tint_color.g) > 0.015) || (abs(tint_color.g - tint_color.b) > 0.015);
    let is_foliage_tinted = !is_mask_face && !is_painted_block && (vertex_output.color.a > 0.75) && has_chroma_tint;
    if (is_foliage_tinted && !is_item_entity) {
        let varied_tint = ambient_environment_variation(vertex_output.world_position.xz, tint_color.rgb * vertex_output.color.rgb);
        tint_color = vec4<f32>(varied_tint, tint_color.a);
    }

    // Dynamic grass side fringe tinting:
    // On vertical side faces of foliage-tinted terrain blocks (grass/turf, NOT tree leaves),
    // only tint the green grass fringe hanging down so it matches the top face, leaving natural
    // dirt and stone underneath untouched. Tree leaves (color.a ~ 2.0 or 3.0) keep full biome tint on all 6 faces.
    let is_leaves_block = (vertex_output.color.a > 1.5) && (vertex_output.color.a < 3.5);
    let is_full_grass_side = (vertex_output.color.a > 1.2) && (vertex_output.color.a < 1.5);
    var final_tint = tint_color.rgb;
    if (!is_mask_face && !is_horizontal && !is_item_entity && is_foliage_tinted && !is_leaves_block && !is_full_grass_side) {
        let is_grass_fringe = (uv.y < 0.55) && (tex_color.g > tex_color.r * 1.02) && (tex_color.g > tex_color.b * 1.15);
        if (is_grass_fringe) {
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

    // Directional shading floor (Top 1.0, Bottom 0.65, N/S 0.90, E/W 0.84):
    // Subtle geometric face distinction complementing real warm-sun vs. cool-ambient shadows.
    let norm = normalize(vertex_output.world_normal);
    var dir_shading = 1.0;
    if (is_cross_mesh) {
        dir_shading = 0.88;
    } else if (abs(norm.y) > 0.5) {
        if (norm.y < 0.0) {
            dir_shading = 0.65;
        }
    } else if (abs(norm.z) > 0.5) {
        dir_shading = 0.90;
    } else {
        dir_shading = 0.84;
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
        // Lift the shadow-sampling position above double-tall cross-mesh canopies so intersecting 'X'
        // planes never self-shadow (eliminating animated shadow-acne stripes), while still casting
        // crisp alpha-cutout shadows onto the ground and receiving shadows from trees, cliffs, and caves.
        pbr_input.world_position.y += 2.15;

        if (vertex_output.color.a > 5.5 && vertex_output.color.a < 6.5) {
            // Rigid fixtures / speleothems / crystals: use a horizontal viewer-facing normal
            // so both crossed planes share uniform shading AND match the vertical side faces
            // of the supporting cave ceiling/floor blocks (Calcite, Karst, etc.).
            let horiz_v = normalize(vec3<f32>(pbr_input.V.x, 0.12, pbr_input.V.z));
            pbr_input.world_normal = horiz_v;
            pbr_input.N = horiz_v;
        } else {
            pbr_input.world_normal = vec3<f32>(0.0, 1.0, 0.0);
            pbr_input.N = vec3<f32>(0.0, 1.0, 0.0);
        }
    }

    // Rain Puddles & Wet Ground Reflections (active during and after rain, dry land only):
    let rain_wetness = voxel_params.w;
    let active_rain = anim_params.w;
    var puddle_mask = 0.0;
    var pixel_ripple_glint = 0.0;
    if (is_solid_terrain && !is_underwater_face && !is_cross_mesh && !is_leaves_block && norm.y > 0.65 && rain_wetness > 0.01) {
        let p_xz = vertex_output.world_position.xz;
        let n_macro = smooth_noise2(p_xz * 0.14);
        let n_mid = smooth_noise2(p_xz * 0.37 + vec2<f32>(17.3, 9.1));
        let n_fine = smooth_noise2(p_xz * 0.95 + vec2<f32>(4.2, 23.8));
        let puddle_noise = n_macro * 0.58 + n_mid * 0.30 + n_fine * 0.12;
        let puddle_thresh = mix(0.68, 0.44, rain_wetness);
        puddle_mask = smoothstep(puddle_thresh, puddle_thresh + 0.14, puddle_noise) * smoothstep(0.05, 0.35, rain_wetness);

        // Damp porous terrain darkens slightly; puddles deepen contrast further
        let wet_darken = mix(1.0, 0.74, rain_wetness * 0.55 + puddle_mask * 0.45);
        pbr_input.material.base_color = vec4<f32>(pbr_input.material.base_color.rgb * wet_darken, pbr_input.material.base_color.a);

        // Balanced wet PBR glossiness: soft realistic sheen without 1-pixel GGX sun laser lines at sunrise/sunset
        let damp_roughness = mix(pbr_input.material.perceptual_roughness, 0.52, rain_wetness * 0.70);
        pbr_input.material.perceptual_roughness = mix(damp_roughness, 0.24, puddle_mask);
        pbr_input.material.reflectance = mix(pbr_input.material.reflectance, vec3<f32>(0.62), puddle_mask * 0.75 + rain_wetness * 0.15);

        // Pixel-art texel-snapped non-repetitive raindrop ripples inside puddles while actively raining
        if (puddle_mask > 0.15 && active_rain > 0.05) {
            let texel_coord = floor(p_xz * 16.0);
            let cell_size = 8.0;
            let cell = floor(texel_coord / cell_size);
            let local_tx = texel_coord - cell * cell_size;
            let h1 = hash2_noise(cell);
            let raw_t = globals.time * 1.35 + h1;
            let cycle_id = floor(raw_t);
            let cycle = fract(raw_t);
            // Mix cycle_id into the seed so every drop lands at a fresh random pixel each cycle
            let seed = cell + vec2<f32>(cycle_id * 13.7, cycle_id * 29.3);
            let h2 = hash2_noise(seed + vec2<f32>(19.7, 43.1));
            let frame = floor(cycle * 5.0);
            if (h2 < 0.38 && frame < 3.5) {
                let center_tx = vec2<f32>(
                    floor(2.0 + hash2_noise(seed + vec2<f32>(3.1, 7.4)) * 4.0),
                    floor(2.0 + hash2_noise(seed + vec2<f32>(11.2, 5.8)) * 4.0)
                );
                let delta_tx = local_tx - center_tx;
                let cheb_dist = max(abs(delta_tx.x), abs(delta_tx.y));
                let manh_dist = abs(delta_tx.x) + abs(delta_tx.y);
                // Crisp 1-texel wide octagonal/square pixel ring expanding 0 -> 1 -> 2 texels
                let ring_r = frame;
                let on_ring = (abs(cheb_dist - ring_r) < 0.25) && (manh_dist <= ring_r * 2.0 - select(0.0, 1.0, ring_r >= 2.0));
                if (on_ring) {
                    let fade = (1.0 - frame / 4.0) * puddle_mask * active_rain;
                    pixel_ripple_glint = fade * 0.22;
                }
            }
        }
    }

    var out: FragmentOutput;
    if (is_emissive) {
        // Light-emitting blocks are self-illuminated into HDR bloom range (> 1.0):
        out.color = vec4<f32>(tex_color.rgb * vertex_output.color.rgb * 2.5, 1.0);
    } else {
        out.color = apply_pbr_lighting(pbr_input);

        // Grazing-angle sky reflection sheen & subtle pixel-art ripple glint on rain puddles
        if (puddle_mask > 0.01) {
            let n_dot_v = max(dot(pbr_input.N, pbr_input.V), 0.0);
            let fresnel = 0.04 + 0.96 * pow(1.0 - n_dot_v, 4.0);
            let sky_luma = clamp(0.18 + max(sun_dir_and_factor.y, 0.0) * 1.4, 0.14, 0.95);
            let puddle_sky_tint = mix(vec3<f32>(0.10, 0.14, 0.22), vec3<f32>(0.42, 0.50, 0.60), sky_luma);
            var wet_rgb = mix(out.color.rgb, puddle_sky_tint, fresnel * puddle_mask * 0.62);
            if (pixel_ripple_glint > 0.001) {
                wet_rgb = mix(wet_rgb, vec3<f32>(0.78, 0.85, 0.92), pixel_ripple_glint);
            }
            out.color = vec4<f32>(wet_rgb, out.color.a);
        }

        // Projected Pixel-Art Underwater Caustics (16x16 texel-snapped, slow organic shimmer matching voxel style)
        let is_aquatic_flora = (vertex_output.color.a > 3.5) && (vertex_output.color.a < 4.5);
        if ((is_underwater_face || is_aquatic_flora) && !is_item_entity) {
            let sun_dir = normalize(sun_dir_and_factor.xyz);
            // Physically dim caustics smoothly through late afternoon (16:00-17:30) and vanish before dusk
            let sun_up = pow(smoothstep(0.10, 0.68, sun_dir.y), 1.85);
            // Smoothly fade caustics from 1.0 in shallow reefs (Y >= 10) down to 0.0 in deep abyssal trenches (Y <= -34)
            let depth_fade = smoothstep(-34.0, 10.0, vertex_output.world_position.y);
            if (sun_up > 0.005 && depth_fade > 0.005) {
                // Snap projected coordinates to 16x16 voxel texel grid for authentic pixel-art caustic charm
                let raw_proj = vertex_output.world_position.xz
                    - sun_dir.xz * (vertex_output.world_position.y / max(sun_dir.y, 0.35)) * 0.45;
                let proj_xz = floor(raw_proj * 16.0) / 16.0;
                let t = max(globals.time, 0.0);
                // Rotate second Voronoi layer by ~37 deg so the web never tiles, with gentle slow motion
                let rot_xz = vec2<f32>(
                    proj_xz.x * 0.7986 - proj_xz.y * 0.6018,
                    proj_xz.x * 0.6018 + proj_xz.y * 0.7986
                );
                let c1 = caustic_layer(proj_xz * 0.34 + vec2<f32>(t * 0.045, t * 0.028), t * 0.36);
                let c2 = caustic_layer(rot_xz * 0.52 - vec2<f32>(t * 0.032, -t * 0.040) + vec2<f32>(11.3, 5.7), t * 0.42);
                let caustic_web = smoothstep(0.18, 0.92, clamp(c1 * 0.65 + c2 * 0.45, 0.0, 1.15));
                let up_weight = select(clamp(norm.y * 0.80 + 0.20, 0.10, 1.0), 0.55, is_aquatic_flora);
                let caustic_color = vec3<f32>(0.36, 0.86, 0.98) * sun_color_and_cloud.rgb;
                let caustic_strength = caustic_web * sun_up * up_weight * pow(depth_fade, 1.25) * 0.30;
                out.color = vec4<f32>(
                    out.color.rgb + caustic_color * caustic_strength * (0.25 + pbr_input.material.base_color.rgb * 0.95),
                    out.color.a
                );
            }
        }

        // Backlit Subsurface Scattering (translucency) on tree leaves & flora
        let sun_strength = sun_dir_and_factor.w;
        if ((is_leaves_block || (is_cross_mesh && vertex_output.color.a < 5.5)) && !is_item_entity && sun_strength > 0.02) {
            let sun_dir = normalize(sun_dir_and_factor.xyz);
            let view_ray = normalize(-pbr_input.V);
            let v_dot_l = max(dot(view_ray, sun_dir), 0.0);
            let sss_peak = pow(v_dot_l, 8.0) * 0.72;
            let sss_wrap = pow(v_dot_l, 2.0) * 0.14 + 0.06;
            let sss_tint = pbr_input.material.base_color.rgb * sun_color_and_cloud.rgb * vec3<f32>(1.18, 1.24, 0.80);
            out.color = vec4<f32>(out.color.rgb + sss_tint * (sss_peak + sss_wrap) * sun_strength * 0.58, out.color.a);
        }

        // Lightweight procedural cloud shadows drifting across sunlit terrain
        let cloud_strength = sun_color_and_cloud.a;
        if (!is_item_entity && cloud_strength > 0.005) {
            let sun_dir = normalize(sun_dir_and_factor.xyz);
            let proj_scale = 140.0 / max(sun_dir.y, 0.25);
            let cloud_uv = (vertex_output.world_position.xz - sun_dir.xz * proj_scale) * 0.0085
                + vec2<f32>(globals.time * 0.0055, globals.time * 0.0022);
            let cn = smooth_noise2(cloud_uv) * 0.68 + smooth_noise2(cloud_uv * 2.15 + vec2<f32>(13.7, 7.3)) * 0.32;
            let cloud_mask = smoothstep(0.47, 0.74, cn);
            out.color = vec4<f32>(out.color.rgb * (1.0 - cloud_mask * cloud_strength), out.color.a);
        }
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
