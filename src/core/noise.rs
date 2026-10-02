//! Fast, deterministic 2D and 3D gradient noise and Fractal Brownian Motion (FBM).
//! Designed for procedural voxel generation without external dependencies.

/// Canonical 2D gradient directions (8 uniformly spaced directions).
const GRADIENTS_2D: [[f32; 2]; 8] = [
    [1.0, 0.0],
    [-1.0, 0.0],
    [0.0, 1.0],
    [0.0, -1.0],
    [0.70710677, 0.70710677],
    [-0.70710677, 0.70710677],
    [0.70710677, -0.70710677],
    [-0.70710677, -0.70710677],
];

/// Canonical 3D gradient directions (16 directions: 12 cube edge midpoints + 4 tetrahedron diagonals).
/// Sized to 16 to allow branchless bitwise masking (`& 15`) instead of expensive integer modulo (`% 12`).
const GRADIENTS_3D: [[f32; 3]; 16] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
    // 4 canonical directions completing the 16-element power-of-two table (Ken Perlin Improved Noise):
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [0.0, -1.0, 1.0],
    [0.0, -1.0, -1.0],
];

#[inline(always)]
pub fn hash_2d(x: i32, z: i32, seed: u32) -> u32 {
    let mut h = seed;
    h ^= (x as u32).wrapping_mul(0x27D4_EB2D);
    h ^= (z as u32).wrapping_mul(0x1656_67B1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    h
}

#[inline(always)]
pub fn hash_2d_f32(x: i32, z: i32, seed: u32) -> f32 {
    let normalized = hash_2d(x, z, seed) as f32 / u32::MAX as f32;
    normalized * 2.0 - 1.0
}

#[inline(always)]
pub fn hash_3d(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = seed;
    h ^= (x as u32).wrapping_mul(0x27D4_EB2D);
    h ^= (y as u32).wrapping_mul(0x9E37_79B9);
    h ^= (z as u32).wrapping_mul(0x1656_67B1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    h
}

pub use super::math::{lerp, quintic_fade, smoothstep};

/// Computes smooth 2D value noise in the range `[-1.0, 1.0]`.
#[inline]
pub fn value_noise_2d(x: f32, z: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;

    let x1 = x0 + 1;
    let z1 = z0 + 1;

    let tx = smoothstep(x - x0 as f32);
    let tz = smoothstep(z - z0 as f32);

    let v00 = hash_2d_f32(x0, z0, seed);
    let v10 = hash_2d_f32(x1, z0, seed);
    let v01 = hash_2d_f32(x0, z1, seed);
    let v11 = hash_2d_f32(x1, z1, seed);

    let top = lerp(v00, v10, tx);
    let bottom = lerp(v01, v11, tx);

    lerp(top, bottom, tz)
}

/// Computes multi-octave 2D value fractal noise in the range `[-1.0, 1.0]`.
#[inline]
pub fn fractal_noise_2d(
    world_x: f32,
    world_z: f32,
    base_frequency: f32,
    octaves: u32,
    persistence: f32,
    seed: u32,
) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut amplitude_sum = 0.0;

    for octave in 0..octaves {
        let x = world_x * base_frequency * frequency;
        let z = world_z * base_frequency * frequency;

        let octave_seed = seed.wrapping_add(octave.wrapping_mul(10_007));
        value += value_noise_2d(x, z, octave_seed) * amplitude;

        amplitude_sum += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    if amplitude_sum > 0.0 {
        value / amplitude_sum
    } else {
        0.0
    }
}

/// Computes smooth 2D gradient noise in the range `[-1.0, 1.0]`.
#[inline]
pub fn gradient_noise_2d(x: f32, z: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let x1 = x0 + 1;
    let z1 = z0 + 1;

    let fx0 = x - x0 as f32;
    let fz0 = z - z0 as f32;
    let fx1 = fx0 - 1.0;
    let fz1 = fz0 - 1.0;

    let u = quintic_fade(fx0);
    let v = quintic_fade(fz0);

    let g00 = GRADIENTS_2D[(hash_2d(x0, z0, seed) & 7) as usize];
    let g10 = GRADIENTS_2D[(hash_2d(x1, z0, seed) & 7) as usize];
    let g01 = GRADIENTS_2D[(hash_2d(x0, z1, seed) & 7) as usize];
    let g11 = GRADIENTS_2D[(hash_2d(x1, z1, seed) & 7) as usize];

    let d00 = g00[0] * fx0 + g00[1] * fz0;
    let d10 = g10[0] * fx1 + g10[1] * fz0;
    let d01 = g01[0] * fx0 + g01[1] * fz1;
    let d11 = g11[0] * fx1 + g11[1] * fz1;

    let nx0 = lerp(d00, d10, u);
    let nx1 = lerp(d01, d11, u);

    // Scale by SQRT_2 to normalize the range close to [-1.0, 1.0]
    (lerp(nx0, nx1, v) * std::f32::consts::SQRT_2).clamp(-1.0, 1.0)
}

/// Computes 2D Fractal Brownian Motion (FBM) with multiple octaves in range `[-1.0, 1.0]`.
#[inline]
pub fn fbm_2d(
    x: f32,
    z: f32,
    base_frequency: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
    seed: u32,
) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = base_frequency;
    let mut max_amplitude = 0.0;

    for i in 0..octaves {
        let octave_seed = seed.wrapping_add(i.wrapping_mul(31_337));
        total += gradient_noise_2d(x * frequency, z * frequency, octave_seed) * amplitude;
        max_amplitude += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    if max_amplitude > 0.0 {
        total / max_amplitude
    } else {
        0.0
    }
}

/// Computes smooth 3D gradient noise in the range `[-1.0, 1.0]`.
#[inline]
pub fn gradient_noise_3d(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let z0 = z.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let z1 = z0 + 1;

    let fx0 = x - x0 as f32;
    let fy0 = y - y0 as f32;
    let fz0 = z - z0 as f32;
    let fx1 = fx0 - 1.0;
    let fy1 = fy0 - 1.0;
    let fz1 = fz0 - 1.0;

    let u = quintic_fade(fx0);
    let v = quintic_fade(fy0);
    let w = quintic_fade(fz0);

    let g000 = GRADIENTS_3D[(hash_3d(x0, y0, z0, seed) & 15) as usize];
    let g100 = GRADIENTS_3D[(hash_3d(x1, y0, z0, seed) & 15) as usize];
    let g010 = GRADIENTS_3D[(hash_3d(x0, y1, z0, seed) & 15) as usize];
    let g110 = GRADIENTS_3D[(hash_3d(x1, y1, z0, seed) & 15) as usize];
    let g001 = GRADIENTS_3D[(hash_3d(x0, y0, z1, seed) & 15) as usize];
    let g101 = GRADIENTS_3D[(hash_3d(x1, y0, z1, seed) & 15) as usize];
    let g011 = GRADIENTS_3D[(hash_3d(x0, y1, z1, seed) & 15) as usize];
    let g111 = GRADIENTS_3D[(hash_3d(x1, y1, z1, seed) & 15) as usize];

    let d000 = g000[0] * fx0 + g000[1] * fy0 + g000[2] * fz0;
    let d100 = g100[0] * fx1 + g100[1] * fy0 + g100[2] * fz0;
    let d010 = g010[0] * fx0 + g010[1] * fy1 + g010[2] * fz0;
    let d110 = g110[0] * fx1 + g110[1] * fy1 + g110[2] * fz0;
    let d001 = g001[0] * fx0 + g001[1] * fy0 + g001[2] * fz1;
    let d101 = g101[0] * fx1 + g101[1] * fy0 + g101[2] * fz1;
    let d011 = g011[0] * fx0 + g011[1] * fy1 + g011[2] * fz1;
    let d111 = g111[0] * fx1 + g111[1] * fy1 + g111[2] * fz1;

    let nx00 = lerp(d000, d100, u);
    let nx10 = lerp(d010, d110, u);
    let nx01 = lerp(d001, d101, u);
    let nx11 = lerp(d011, d111, u);

    let ny0 = lerp(nx00, nx10, v);
    let ny1 = lerp(nx01, nx11, v);

    // Scale factor to map roughly into [-1.0, 1.0]
    (lerp(ny0, ny1, w) * 1.1547005).clamp(-1.0, 1.0)
}

/// Computes 3D Fractal Brownian Motion (FBM) with multiple octaves in range `[-1.0, 1.0]`.
#[inline]
pub fn fbm_3d(
    x: f32,
    y: f32,
    z: f32,
    base_frequency: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
    seed: u32,
) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = base_frequency;
    let mut max_amplitude = 0.0;

    for i in 0..octaves {
        let octave_seed = seed.wrapping_add(i.wrapping_mul(54_321));
        total +=
            gradient_noise_3d(x * frequency, y * frequency, z * frequency, octave_seed) * amplitude;
        max_amplitude += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    if max_amplitude > 0.0 {
        total / max_amplitude
    } else {
        0.0
    }
}

// ==============================================================================
// 4-Wide Vector SIMD Noise Primitives (Phase 5)
// ==============================================================================

/// 4-wide 32-bit floating point SIMD vector aligned to 16 bytes.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, align(16))]
pub struct Simd4f(pub [f32; 4]);

impl From<[f32; 4]> for Simd4f {
    #[inline(always)]
    fn from(arr: [f32; 4]) -> Self {
        Self(arr)
    }
}

impl From<Simd4f> for [f32; 4] {
    #[inline(always)]
    fn from(simd: Simd4f) -> Self {
        simd.0
    }
}

impl Simd4f {
    #[inline(always)]
    pub const fn splat(val: f32) -> Self {
        Self([val, val, val, val])
    }

    #[inline(always)]
    pub fn add(self, other: Self) -> Self {
        Self([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ])
    }

    #[inline(always)]
    pub fn add_scalar(self, val: f32) -> Self {
        self.add(Self::splat(val))
    }

    #[inline(always)]
    pub fn sub(self, other: Self) -> Self {
        Self([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
            self.0[3] - other.0[3],
        ])
    }

    #[inline(always)]
    pub fn sub_scalar(self, val: f32) -> Self {
        self.sub(Self::splat(val))
    }

    #[inline(always)]
    pub fn mul(self, other: Self) -> Self {
        Self([
            self.0[0] * other.0[0],
            self.0[1] * other.0[1],
            self.0[2] * other.0[2],
            self.0[3] * other.0[3],
        ])
    }

    #[inline(always)]
    pub fn scale(self, val: f32) -> Self {
        self.mul(Self::splat(val))
    }

    #[inline(always)]
    pub fn floor(self) -> Self {
        Self([
            self.0[0].floor(),
            self.0[1].floor(),
            self.0[2].floor(),
            self.0[3].floor(),
        ])
    }

    #[inline(always)]
    pub fn to_simd4i(self) -> Simd4i {
        Simd4i([
            self.0[0] as i32,
            self.0[1] as i32,
            self.0[2] as i32,
            self.0[3] as i32,
        ])
    }

    #[inline(always)]
    pub fn quintic_fade(self) -> Self {
        Self([
            quintic_fade(self.0[0]),
            quintic_fade(self.0[1]),
            quintic_fade(self.0[2]),
            quintic_fade(self.0[3]),
        ])
    }

    #[inline(always)]
    pub fn smoothstep(self) -> Self {
        Self([
            smoothstep(self.0[0]),
            smoothstep(self.0[1]),
            smoothstep(self.0[2]),
            smoothstep(self.0[3]),
        ])
    }

    #[inline(always)]
    pub fn lerp(self, other: Self, t: Self) -> Self {
        Self([
            lerp(self.0[0], other.0[0], t.0[0]),
            lerp(self.0[1], other.0[1], t.0[1]),
            lerp(self.0[2], other.0[2], t.0[2]),
            lerp(self.0[3], other.0[3], t.0[3]),
        ])
    }

    #[inline(always)]
    pub fn clamp(self, min_val: f32, max_val: f32) -> Self {
        Self([
            self.0[0].clamp(min_val, max_val),
            self.0[1].clamp(min_val, max_val),
            self.0[2].clamp(min_val, max_val),
            self.0[3].clamp(min_val, max_val),
        ])
    }

    #[inline(always)]
    pub fn abs(self) -> Self {
        Self([
            self.0[0].abs(),
            self.0[1].abs(),
            self.0[2].abs(),
            self.0[3].abs(),
        ])
    }

    #[inline(always)]
    pub fn gather_grad2d(h: Simd4u) -> (Self, Self) {
        let g0 = GRADIENTS_2D[(h.0[0] & 7) as usize];
        let g1 = GRADIENTS_2D[(h.0[1] & 7) as usize];
        let g2 = GRADIENTS_2D[(h.0[2] & 7) as usize];
        let g3 = GRADIENTS_2D[(h.0[3] & 7) as usize];
        (
            Self([g0[0], g1[0], g2[0], g3[0]]),
            Self([g0[1], g1[1], g2[1], g3[1]]),
        )
    }
}

/// 4-wide 32-bit unsigned integer SIMD vector aligned to 16 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C, align(16))]
pub struct Simd4u(pub [u32; 4]);

impl Simd4u {
    #[inline(always)]
    pub const fn splat(val: u32) -> Self {
        Self([val, val, val, val])
    }

    #[inline(always)]
    pub fn wrapping_mul(self, other: Self) -> Self {
        Self([
            self.0[0].wrapping_mul(other.0[0]),
            self.0[1].wrapping_mul(other.0[1]),
            self.0[2].wrapping_mul(other.0[2]),
            self.0[3].wrapping_mul(other.0[3]),
        ])
    }

    #[inline(always)]
    pub fn bitxor(self, other: Self) -> Self {
        Self([
            self.0[0] ^ other.0[0],
            self.0[1] ^ other.0[1],
            self.0[2] ^ other.0[2],
            self.0[3] ^ other.0[3],
        ])
    }

    #[inline(always)]
    pub fn shr(self, shift: u32) -> Self {
        Self([
            self.0[0] >> shift,
            self.0[1] >> shift,
            self.0[2] >> shift,
            self.0[3] >> shift,
        ])
    }
}

/// 4-wide 32-bit signed integer SIMD vector aligned to 16 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C, align(16))]
pub struct Simd4i(pub [i32; 4]);

impl Simd4i {
    #[inline(always)]
    pub const fn splat(val: i32) -> Self {
        Self([val, val, val, val])
    }

    #[inline(always)]
    pub fn add_scalar(self, val: i32) -> Self {
        Self([
            self.0[0] + val,
            self.0[1] + val,
            self.0[2] + val,
            self.0[3] + val,
        ])
    }

    #[inline(always)]
    pub fn as_simd4u(self) -> Simd4u {
        Simd4u([
            self.0[0] as u32,
            self.0[1] as u32,
            self.0[2] as u32,
            self.0[3] as u32,
        ])
    }
}

#[inline(always)]
pub fn hash_2d_x4(x: Simd4i, z: Simd4i, seed: u32) -> Simd4u {
    let mut h = Simd4u::splat(seed);
    h = h.bitxor(x.as_simd4u().wrapping_mul(Simd4u::splat(0x27D4_EB2D)));
    h = h.bitxor(z.as_simd4u().wrapping_mul(Simd4u::splat(0x1656_67B1)));
    h = h.bitxor(h.shr(15));
    h = h.wrapping_mul(Simd4u::splat(0x85EB_CA6B));
    h = h.bitxor(h.shr(13));
    h = h.wrapping_mul(Simd4u::splat(0xC2B2_AE35));
    h = h.bitxor(h.shr(16));
    h
}

#[inline(always)]
pub fn hash_2d_f32_x4(x: Simd4i, z: Simd4i, seed: u32) -> Simd4f {
    let h = hash_2d_x4(x, z, seed);
    let inv_max = 1.0 / u32::MAX as f32;
    Simd4f([
        (h.0[0] as f32 * inv_max) * 2.0 - 1.0,
        (h.0[1] as f32 * inv_max) * 2.0 - 1.0,
        (h.0[2] as f32 * inv_max) * 2.0 - 1.0,
        (h.0[3] as f32 * inv_max) * 2.0 - 1.0,
    ])
}

/// Computes smooth 2D gradient noise for 4 vector lanes simultaneously in the range `[-1.0, 1.0]`.
#[inline]
pub fn gradient_noise_2d_x4(x: Simd4f, z: Simd4f, seed: u32) -> Simd4f {
    let x0 = x.floor();
    let z0 = z.floor();
    let x0_i = x0.to_simd4i();
    let z0_i = z0.to_simd4i();
    let x1_i = x0_i.add_scalar(1);
    let z1_i = z0_i.add_scalar(1);

    let fx0 = x.sub(x0);
    let fz0 = z.sub(z0);
    let fx1 = fx0.sub_scalar(1.0);
    let fz1 = fz0.sub_scalar(1.0);

    let u = fx0.quintic_fade();
    let v = fz0.quintic_fade();

    let h00 = hash_2d_x4(x0_i, z0_i, seed);
    let h10 = hash_2d_x4(x1_i, z0_i, seed);
    let h01 = hash_2d_x4(x0_i, z1_i, seed);
    let h11 = hash_2d_x4(x1_i, z1_i, seed);

    let g00 = Simd4f::gather_grad2d(h00);
    let g10 = Simd4f::gather_grad2d(h10);
    let g01 = Simd4f::gather_grad2d(h01);
    let g11 = Simd4f::gather_grad2d(h11);

    let d00 = g00.0.mul(fx0).add(g00.1.mul(fz0));
    let d10 = g10.0.mul(fx1).add(g10.1.mul(fz0));
    let d01 = g01.0.mul(fx0).add(g01.1.mul(fz1));
    let d11 = g11.0.mul(fx1).add(g11.1.mul(fz1));

    let nx0 = d00.lerp(d10, u);
    let nx1 = d01.lerp(d11, u);

    let res = nx0.lerp(nx1, v).scale(std::f32::consts::SQRT_2);
    res.clamp(-1.0, 1.0)
}

/// Computes smooth 2D value noise for 4 vector lanes simultaneously in the range `[-1.0, 1.0]`.
#[inline]
pub fn value_noise_2d_x4(x: Simd4f, z: Simd4f, seed: u32) -> Simd4f {
    let x0 = x.floor();
    let z0 = z.floor();
    let x0_i = x0.to_simd4i();
    let z0_i = z0.to_simd4i();
    let x1_i = x0_i.add_scalar(1);
    let z1_i = z0_i.add_scalar(1);

    let tx = x.sub(x0).smoothstep();
    let tz = z.sub(z0).smoothstep();

    let v00 = hash_2d_f32_x4(x0_i, z0_i, seed);
    let v10 = hash_2d_f32_x4(x1_i, z0_i, seed);
    let v01 = hash_2d_f32_x4(x0_i, z1_i, seed);
    let v11 = hash_2d_f32_x4(x1_i, z1_i, seed);

    let top = v00.lerp(v10, tx);
    let bottom = v01.lerp(v11, tx);

    top.lerp(bottom, tz)
}

/// Computes 2D Fractal Brownian Motion (FBM) for 4 vector lanes simultaneously in range `[-1.0, 1.0]`.
#[inline]
pub fn fbm_2d_x4(
    x: Simd4f,
    z: Simd4f,
    base_frequency: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
    seed: u32,
) -> Simd4f {
    let mut total = Simd4f::splat(0.0);
    let mut amplitude = 1.0;
    let mut frequency = base_frequency;
    let mut max_amplitude = 0.0;

    for i in 0..octaves {
        let octave_seed = seed.wrapping_add(i.wrapping_mul(31_337));
        total = total.add(
            gradient_noise_2d_x4(x.scale(frequency), z.scale(frequency), octave_seed)
                .scale(amplitude),
        );
        max_amplitude += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    if max_amplitude > 0.0 {
        total.scale(1.0 / max_amplitude)
    } else {
        Simd4f::splat(0.0)
    }
}

/// Computes multi-octave 2D value fractal noise for 4 vector lanes simultaneously in range `[-1.0, 1.0]`.
#[inline]
pub fn fractal_noise_2d_x4(
    world_x: Simd4f,
    world_z: Simd4f,
    base_frequency: f32,
    octaves: u32,
    persistence: f32,
    seed: u32,
) -> Simd4f {
    let mut value = Simd4f::splat(0.0);
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut amplitude_sum = 0.0;

    for octave in 0..octaves {
        let x = world_x.scale(base_frequency * frequency);
        let z = world_z.scale(base_frequency * frequency);

        let octave_seed = seed.wrapping_add(octave.wrapping_mul(10_007));
        value = value.add(value_noise_2d_x4(x, z, octave_seed).scale(amplitude));

        amplitude_sum += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    if amplitude_sum > 0.0 {
        value.scale(1.0 / amplitude_sum)
    } else {
        Simd4f::splat(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_noise_matches_scalar() {
        let seed = 1337;
        let xs = [12.34, -45.67, 89.01, -0.25];
        let zs = [98.76, 54.32, -21.09, 100.5];

        let simd_grad = gradient_noise_2d_x4(Simd4f(xs), Simd4f(zs), seed);
        for i in 0..4 {
            let scalar = gradient_noise_2d(xs[i], zs[i], seed);
            assert!(
                (simd_grad.0[i] - scalar).abs() < 1e-6,
                "Gradient noise lane {} mismatch: simd={}, scalar={}",
                i,
                simd_grad.0[i],
                scalar
            );
        }

        let simd_val = value_noise_2d_x4(Simd4f(xs), Simd4f(zs), seed);
        for i in 0..4 {
            let scalar = value_noise_2d(xs[i], zs[i], seed);
            assert!(
                (simd_val.0[i] - scalar).abs() < 1e-6,
                "Value noise lane {} mismatch: simd={}, scalar={}",
                i,
                simd_val.0[i],
                scalar
            );
        }

        let simd_fbm = fbm_2d_x4(Simd4f(xs), Simd4f(zs), 0.05, 3, 0.5, 2.0, seed);
        for i in 0..4 {
            let scalar = fbm_2d(xs[i], zs[i], 0.05, 3, 0.5, 2.0, seed);
            assert!(
                (simd_fbm.0[i] - scalar).abs() < 1e-6,
                "FBM noise lane {} mismatch: simd={}, scalar={}",
                i,
                simd_fbm.0[i],
                scalar
            );
        }

        let simd_frac = fractal_noise_2d_x4(Simd4f(xs), Simd4f(zs), 0.05, 3, 0.5, seed);
        for i in 0..4 {
            let scalar = fractal_noise_2d(xs[i], zs[i], 0.05, 3, 0.5, seed);
            assert!(
                (simd_frac.0[i] - scalar).abs() < 1e-6,
                "Fractal noise lane {} mismatch: simd={}, scalar={}",
                i,
                simd_frac.0[i],
                scalar
            );
        }
    }
}
