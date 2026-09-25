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

/// Canonical 3D gradient directions (12 edge midpoints of a cube).
const GRADIENTS_3D: [[f32; 3]; 12] = [
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
];

#[inline(always)]
fn hash_2d(x: i32, z: i32, seed: u32) -> u32 {
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
fn hash_3d(x: i32, y: i32, z: i32, seed: u32) -> u32 {
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

#[inline(always)]
fn quintic_fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline(always)]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// Computes smooth 2D gradient noise in the range `[-1.0, 1.0]`.
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

    let g000 = GRADIENTS_3D[(hash_3d(x0, y0, z0, seed) % 12) as usize];
    let g100 = GRADIENTS_3D[(hash_3d(x1, y0, z0, seed) % 12) as usize];
    let g010 = GRADIENTS_3D[(hash_3d(x0, y1, z0, seed) % 12) as usize];
    let g110 = GRADIENTS_3D[(hash_3d(x1, y1, z0, seed) % 12) as usize];
    let g001 = GRADIENTS_3D[(hash_3d(x0, y0, z1, seed) % 12) as usize];
    let g101 = GRADIENTS_3D[(hash_3d(x1, y0, z1, seed) % 12) as usize];
    let g011 = GRADIENTS_3D[(hash_3d(x0, y1, z1, seed) % 12) as usize];
    let g111 = GRADIENTS_3D[(hash_3d(x1, y1, z1, seed) % 12) as usize];

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
#[allow(clippy::too_many_arguments)]
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
