use bevy::prelude::*;
use voxel_game_engine::{
    environment::foliage::{
        config::{GrassConfig, GrassStyle},
        generator::{BladePrng, block_surface_top_y, emit_block_grass_blades},
        trail::GrassTrailField,
    },
    meshing::greedy::MeshBuffers,
    world::{BlockShape, Voxel},
};

#[test]
fn test_grass_turf_voxels() {
    let grass_voxels = [
        Voxel::CalicheGrass,
        Voxel::LateriteGrass,
        Voxel::LoamGrass,
        Voxel::LoamBurrow,
        Voxel::PeatGrass,
        Voxel::PeatBurrow,
        Voxel::PermafrostGrass,
        Voxel::SiltGrass,
        Voxel::SiltBurrow,
        Voxel::AndesiteGrass,
        Voxel::ArgilliteGrass,
        Voxel::BasaltGrass,
        Voxel::ChalkGrass,
        Voxel::DolomiteGrass,
        Voxel::GraniteGrass,
        Voxel::LimestoneGrass,
    ];

    for voxel in grass_voxels {
        assert!(
            voxel.has_grass_turf(),
            "Voxel {:?} must have grass turf enabled",
            voxel
        );
    }

    assert!(!Voxel::AshPumice.has_grass_turf());
    assert!(!Voxel::LoamDirt.has_grass_turf());
    assert!(!Voxel::Air.has_grass_turf());
    assert!(!Voxel::LiquidWater.has_grass_turf());
}

#[test]
fn test_blade_prng_determinism() {
    let mut prng1 = BladePrng::new(0x12345678_ABCDEF01);
    let mut prng2 = BladePrng::new(0x12345678_ABCDEF01);

    for _ in 0..100 {
        assert_eq!(prng1.next_u64(), prng2.next_u64());
        assert_eq!(prng1.next_f32(), prng2.next_f32());
    }

    let mut prng3 = BladePrng::new(42);
    for _ in 0..100 {
        let val = prng3.next_f32();
        assert!(val >= 0.0 && val <= 1.0, "PRNG output out of bounds [0, 1]");
    }
}

#[test]
fn test_emit_block_grass_blades_geometry() {
    let mut buffers = MeshBuffers::with_capacity(64);
    let block_local = IVec3::new(4, 5, 6);
    let chunk_origin = IVec3::new(16, 0, 32);
    let blade_count = 16;
    let lod_quads = 4;
    let tint = [0.4, 0.8, 0.3];
    let snow_blend = 0.0;

    emit_block_grass_blades(
        &mut buffers,
        block_local,
        chunk_origin,
        blade_count,
        lod_quads,
        tint,
        snow_blend,
        BlockShape::Full,
        0,
    );

    // 16 blades * 4 quads = 64 quads
    // 64 quads * 4 vertices = 256 vertices
    // 64 quads * 6 indices (2 tris) = 384 indices
    assert_eq!(buffers.positions.len(), 256);
    assert_eq!(buffers.normals.len(), 256);
    assert_eq!(buffers.uvs.len(), 256);
    assert_eq!(buffers.colors.len(), 256);
    assert_eq!(buffers.indices.len(), 384);

    let mesh = buffers.into_mesh();
    assert!(mesh.is_some(), "into_mesh must return Some mesh for non-empty buffers");
    let mesh = mesh.unwrap();
    assert_eq!(mesh.count_vertices(), 256);
}

#[test]
fn test_emit_block_grass_blades_on_slab_and_stairs() {
    // 1. Bottom Slab (orientation 0): surface height should be 0.5 across the entire block
    assert_eq!(block_surface_top_y(BlockShape::Slab, 0, 0.2, 0.2), Some(0.5));
    assert_eq!(block_surface_top_y(BlockShape::Slab, 0, 0.8, 0.8), Some(0.5));

    // Top Slab (orientation 1): surface height should be 1.0
    assert_eq!(block_surface_top_y(BlockShape::Slab, 1, 0.5, 0.5), Some(1.0));

    // 2. Upright Stair (+X, orientation 0): lower step is x in 0.0..0.5 (y = 0.5), upper step is x in 0.5..1.0 (y = 1.0)
    assert_eq!(block_surface_top_y(BlockShape::Stair, 0, 0.2, 0.5), Some(0.5));
    assert_eq!(block_surface_top_y(BlockShape::Stair, 0, 0.8, 0.5), Some(1.0));

    // 3. Test blade vertex generation on a bottom slab: base vertices should sit at y = 5.5 (5 + 0.5), not 6.0!
    let mut buffers = MeshBuffers::with_capacity(32);
    let block_local = IVec3::new(4, 5, 6);
    emit_block_grass_blades(
        &mut buffers,
        block_local,
        IVec3::ZERO,
        8,
        2,
        [0.4, 0.8, 0.3],
        0.0,
        BlockShape::Slab,
        0,
    );

    assert!(!buffers.positions.is_empty());
    for pos in &buffers.positions {
        // Base vertices (h0 = 0) must be exactly at y = 5.5
        assert!(pos[1] >= 5.5, "Grass blade vertex Y ({}) is below slab surface (5.5)", pos[1]);
        assert!(pos[1] <= 6.0, "Grass blade vertex Y ({}) exceeds reasonable height above slab", pos[1]);
    }
}

#[test]
fn test_grass_trail_field_stamping_and_decay() {
    let mut images = Assets::<Image>::default();
    let mut trail = GrassTrailField::new(&mut images);

    assert_eq!(trail.pixels.len(), 48 * 48 * 4);

    // Initial state: all pixels zeroed (no footprint displacement)
    for byte in &trail.pixels {
        assert_eq!(*byte, 0);
    }

    // Stamp a player footprint at grid center (24m into 48m field)
    trail.stamp_entity(Vec3::new(24.0, 64.0, 24.0), Vec3::new(1.0, 0.0, 0.0), 1.2);

    // Verify center pixel has non-zero intensity
    let center_idx = (24 * 48 + 24) * 4;
    assert!(trail.pixels[center_idx + 2] > 0, "Intensity channel (blue) must be > 0 at footprint stamp center");
    assert!(trail.pixels[center_idx + 3] > 0, "Alpha channel must be > 0 at footprint stamp center");

    let initial_intensity = trail.pixels[center_idx + 2];
    let initial_alpha = trail.pixels[center_idx + 3];

    // Simulate decay over 1.0 second
    trail.decay(1.0);

    let decayed_intensity = trail.pixels[center_idx + 2];
    let decayed_alpha = trail.pixels[center_idx + 3];
    assert!(
        decayed_intensity < initial_intensity,
        "Footprint intensity must decay exponentially over time"
    );
    assert!(
        decayed_alpha < initial_alpha,
        "Footprint alpha must decay exponentially over time"
    );
}

#[test]
fn test_grass_config_presets_and_style() {
    let default_cfg = GrassConfig::default();
    assert!(default_cfg.enabled);
    assert_eq!(default_cfg.grass_style, GrassStyle::Tapered);
    assert_eq!(default_cfg.blades_per_block, 64);

    let mut local_cfg = voxel_game_engine::core::config::LocalConfig::default();
    assert!(local_cfg.grass_enabled);
    assert_eq!(local_cfg.grass_density, 32);
    assert_eq!(local_cfg.grass_style, "tapered");

    local_cfg.apply_preset(voxel_game_engine::core::config::PerformancePreset::Potato);
    assert!(!local_cfg.grass_enabled);
    assert_eq!(local_cfg.grass_density, 16);

    local_cfg.apply_preset(voxel_game_engine::core::config::PerformancePreset::Ultra);
    assert!(local_cfg.grass_enabled);
    assert_eq!(local_cfg.grass_density, 64);
    assert!(local_cfg.grass_particles);
}
