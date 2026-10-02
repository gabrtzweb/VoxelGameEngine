//! Common fast math and interpolation utilities for the voxel engine.

/// Standard linear interpolation between `a` and `b` by factor `t`.
#[inline(always)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Normalizes `value` within the range `[min, max]`.
#[allow(dead_code)]
#[inline(always)]
pub fn inverse_lerp(min: f32, max: f32, value: f32) -> f32 {
    if (max - min).abs() < f32::EPSILON {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

/// Remaps `value` from input range `[i_min, i_max]` to output range `[o_min, o_max]`.
#[allow(dead_code)]
#[inline(always)]
pub fn remap(i_min: f32, i_max: f32, o_min: f32, o_max: f32, value: f32) -> f32 {
    let t = inverse_lerp(i_min, i_max, value);
    lerp(o_min, o_max, t)
}

/// Standard cubic Hermite smoothstep in `[0.0, 1.0]`.
#[inline(always)]
pub fn smoothstep(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Quintic fade curve (Ken Perlin improved noise fade function): `6t^5 - 15t^4 + 10t^3`.
#[inline(always)]
pub fn quintic_fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
