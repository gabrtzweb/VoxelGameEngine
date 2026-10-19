use voxel_game_engine::core::noise::{
    Simd4f, fbm_2d, fbm_2d_x4, fractal_noise_2d, fractal_noise_2d_x4, gradient_noise_2d,
    gradient_noise_2d_x4, value_noise_2d, value_noise_2d_x4,
};

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
