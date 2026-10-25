use std::f32::consts::PI;

use bevy::prelude::*;

use crate::meshing::greedy::MeshBuffers;
use crate::world::BlockShape;

/// Simple, ultra-fast 64-bit SplitMix pseudo-random generator.
pub struct BladePrng {
    state: u64,
}

impl BladePrng {
    #[inline(always)]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    #[inline(always)]
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 * 5.9604645e-8 // 1 / 2^24
    }
}

/// Computes the surface top Y offset within the block [0.0 .. 1.0] for a given (local_x, local_z) on a shaped block.
/// Returns None if there is no solid surface underneath this position (e.g. empty side of a wall slab or column).
pub fn block_surface_top_y(
    shape: BlockShape,
    orientation: u8,
    local_x: f32,
    local_z: f32,
) -> Option<f32> {
    if shape == BlockShape::Full {
        return Some(1.0);
    }
    let (box_a, box_b) = shape.local_boxes(orientation);
    let in_box = |b: &[Vec3; 2]| -> Option<f32> {
        let eps = 0.001;
        if local_x >= b[0].x - eps
            && local_x <= b[1].x + eps
            && local_z >= b[0].z - eps
            && local_z <= b[1].z + eps
        {
            Some(b[1].y)
        } else {
            None
        }
    };
    let top_a = in_box(&box_a);
    let top_b = box_b.as_ref().and_then(in_box);
    match (top_a, top_b) {
        (Some(ya), Some(yb)) => Some(ya.max(yb)),
        (Some(ya), None) => Some(ya),
        (None, Some(yb)) => Some(yb),
        (None, None) => None,
    }
}

/// Emits the stacked quad geometry for procedural grass blades sprouting on top of a voxel.
pub fn emit_block_grass_blades(
    buffers: &mut MeshBuffers,
    block_local: IVec3,
    chunk_origin: IVec3,
    blade_count: u32,
    lod_quads: usize,
    base_tint: [f32; 3],
    snow_blend: f32,
    shape: BlockShape,
    orientation: u8,
) {
    let world_block_x = chunk_origin.x + block_local.x;
    let world_block_y = chunk_origin.y + block_local.y;
    let world_block_z = chunk_origin.z + block_local.z;

    let seed = ((world_block_x as i64) as u64)
        ^ (((world_block_z as i64) as u64).wrapping_shl(32))
        ^ (((world_block_y as i64) as u64).wrapping_shl(16))
        ^ 0xA5A5A5A55A5A5A5A;

    let mut prng = BladePrng::new(seed);

    for _blade_idx in 0..blade_count {
        // Position on the block face (slight overhang 0.08..1.08 to break grid edges)
        let off_x = 0.08 + prng.next_f32() * 0.84;
        let off_z = 0.08 + prng.next_f32() * 0.84;

        let Some(top_y_offset) = block_surface_top_y(shape, orientation, off_x, off_z) else {
            continue;
        };
        let block_top_y = block_local.y as f32 + top_y_offset;

        let blade_local_x = block_local.x as f32 + off_x;
        let blade_local_z = block_local.z as f32 + off_z;

        let world_x = world_block_x as f32 + off_x;
        let world_z = world_block_z as f32 + off_z;

        // Quantized blade orientation in 32 angular buckets (5 bits)
        let raw_angle = prng.next_f32() * 2.0 * PI;
        let angle_bucket = (((raw_angle / (2.0 * PI)) * 31.0).round() as u32).min(31);
        let quant_angle = (angle_bucket as f32 / 31.0) * (2.0 * PI);

        let side_dir_x = quant_angle.cos();
        let side_dir_z = quant_angle.sin();

        // World noise coordinates [-1.0 .. 1.0] packed in Normal.xy
        let norm_x = ((world_x % 32.0 + 32.0) % 32.0) / 32.0 * 2.0 - 1.0;
        let norm_z = ((world_z % 32.0 + 32.0) % 32.0) / 32.0 * 2.0 - 1.0;

        // Hue jitter based on angle bucket
        let jitter = (angle_bucket as f32 / 31.0 - 0.5) * 0.06;
        let blade_color_rgb = [
            (base_tint[0] + jitter).clamp(0.0, 1.0),
            (base_tint[1] + jitter * 0.5).clamp(0.0, 1.0),
            (base_tint[2] - jitter * 0.5).clamp(0.0, 1.0),
        ];

        let height_class: u32 = 0; // 0 = standard grass turf

        let left_alpha = ((0u32 << 7) | (height_class << 5) | angle_bucket) as f32 / 255.0;
        let right_alpha = ((1u32 << 7) | (height_class << 5) | angle_bucket) as f32 / 255.0;

        // Quad subdivisions: 4 (full), 2 (medium), 1 (far)
        let quads = lod_quads.clamp(1, 4);
        let step_h = 1.0 / quads as f32;

        let base_width = 0.040;
        let mid_width = 0.050;
        let tip_width = 0.013;

        let width_at = |h: f32| -> f32 {
            if h <= 0.666667 {
                base_width + (mid_width - base_width) * (h / 0.666667)
            } else {
                mid_width + (tip_width - mid_width) * ((h - 0.666667) / 0.333333)
            }
        };

        for q in 0..quads {
            let h0 = q as f32 * step_h;
            let h1 = (q + 1) as f32 * step_h;

            let w0 = width_at(h0);
            let w1 = width_at(h1);

            let v_idx_start = buffers.positions.len() as u32;

            // 4 vertices of the quad
            // v0: bottom-left
            let p0 = [
                blade_local_x - side_dir_x * w0,
                block_top_y + h0 * 0.435,
                blade_local_z - side_dir_z * w0,
            ];
            // v1: bottom-right
            let p1 = [
                blade_local_x + side_dir_x * w0,
                block_top_y + h0 * 0.435,
                blade_local_z + side_dir_z * w0,
            ];
            // v2: top-right
            let p2 = [
                blade_local_x + side_dir_x * w1,
                block_top_y + h1 * 0.435,
                blade_local_z + side_dir_z * w1,
            ];
            // v3: top-left
            let p3 = [
                blade_local_x - side_dir_x * w1,
                block_top_y + h1 * 0.435,
                blade_local_z - side_dir_z * w1,
            ];

            buffers.positions.push(p0);
            buffers.positions.push(p1);
            buffers.positions.push(p2);
            buffers.positions.push(p3);

            // Normal packing: [norm_x, norm_z, height_fraction]
            buffers.normals.push([norm_x, norm_z, h0]);
            buffers.normals.push([norm_x, norm_z, h0]);
            buffers.normals.push([norm_x, norm_z, h1]);
            buffers.normals.push([norm_x, norm_z, h1]);

            // UV0: [side_u (0 or 1), snow_blend]
            buffers.uvs.push([0.0, snow_blend]);
            buffers.uvs.push([1.0, snow_blend]);
            buffers.uvs.push([1.0, snow_blend]);
            buffers.uvs.push([0.0, snow_blend]);

            // UV_B unused by input, initialized to zero
            buffers.uv_bs.push([0.0, 0.0]);
            buffers.uv_bs.push([0.0, 0.0]);
            buffers.uv_bs.push([0.0, 0.0]);
            buffers.uv_bs.push([0.0, 0.0]);

            // Colors: RGB = tint, A = packed side & angle
            buffers.colors.push([blade_color_rgb[0], blade_color_rgb[1], blade_color_rgb[2], left_alpha]);
            buffers.colors.push([blade_color_rgb[0], blade_color_rgb[1], blade_color_rgb[2], right_alpha]);
            buffers.colors.push([blade_color_rgb[0], blade_color_rgb[1], blade_color_rgb[2], right_alpha]);
            buffers.colors.push([blade_color_rgb[0], blade_color_rgb[1], blade_color_rgb[2], left_alpha]);

            // Indices (two triangles per quad)
            buffers.indices.push(v_idx_start);
            buffers.indices.push(v_idx_start + 1);
            buffers.indices.push(v_idx_start + 2);

            buffers.indices.push(v_idx_start);
            buffers.indices.push(v_idx_start + 2);
            buffers.indices.push(v_idx_start + 3);
        }
    }
}
