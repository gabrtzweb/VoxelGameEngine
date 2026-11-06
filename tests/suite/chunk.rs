use voxel_game_engine::world::{CHUNK_VOLUME, Chunk, ChunkHomogeneity, Voxel};

#[test]
fn test_chunk_memory_size() {
    let chunk_size = std::mem::size_of::<Chunk>();
    assert!(
        chunk_size < 384,
        "Chunk size must be small for cache efficiency, was {chunk_size} bytes"
    );
}

#[test]
fn test_chunk_storage_uniform_and_set() {
    let mut chunk = Chunk::new();
    assert!(chunk.is_empty());
    assert_eq!(chunk.get(0, 0, 0), Voxel::Air);
    assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Empty);

    chunk.set(0, 0, 0, Voxel::LoamGrass);
    assert_eq!(chunk.get(0, 0, 0), Voxel::LoamGrass);
    assert!(!chunk.is_empty());
    assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Mixed);

    chunk.set(0, 0, 0, Voxel::Air);
    assert!(chunk.is_empty());
    assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Empty);
}

#[test]
fn test_chunk_from_voxels() {
    let voxels = vec![Voxel::Air; CHUNK_VOLUME];
    let chunk = Chunk::from_voxels(voxels);
    assert!(chunk.is_empty());
    assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Empty);
}

#[test]
fn test_voxel_ambient_occlusion_and_flipping() {
    use bevy::prelude::IVec3;
    use voxel_game_engine::meshing::greedy::{
        compute_vertex_ao, vertex_uv_sign, FaceDirection, AO_TABLE,
    };
    use voxel_game_engine::world::VoxelWorld;

    // AO multiplier levels
    assert_eq!(AO_TABLE[3], 1.00);
    assert_eq!(AO_TABLE[2], 0.92);
    assert_eq!(AO_TABLE[1], 0.84);
    assert_eq!(AO_TABLE[0], 0.76);

    // UV signs verify winding orientation for all 6 faces
    for direction in [
        FaceDirection::PositiveX,
        FaceDirection::NegativeX,
        FaceDirection::PositiveY,
        FaceDirection::NegativeY,
        FaceDirection::PositiveZ,
        FaceDirection::NegativeZ,
    ] {
        for v_idx in 0..4 {
            let (su, sv) = vertex_uv_sign(direction, v_idx);
            assert!(su == -1 || su == 1);
            assert!(sv == -1 || sv == 1);
        }
    }

    // In an empty world, all exposed vertices must evaluate to level 3 (unoccluded)
    let world = VoxelWorld::default();
    let pos = IVec3::new(10, 10, 10);
    for v_idx in 0..4 {
        let ao = compute_vertex_ao(&world, pos, FaceDirection::PositiveY, v_idx);
        assert_eq!(ao, 3, "Empty space should have maximum brightness (AO level 3)");
    }
}

#[test]
fn test_smooth_flow_fluid_corner_heights() {
    use bevy::prelude::IVec3;
    use voxel_game_engine::simulation::fluid::compute_fluid_corner_offsets;
    use voxel_game_engine::world::{Chunk, VoxelWorld};

    let mut world = VoxelWorld::default();
    world.insert_chunk(IVec3::ZERO, Chunk::new());
    let center = IVec3::new(10, 10, 10);
    world.set_voxel(center, Voxel::LiquidWater);
    world.set_fluid_level(center, 0);

    let corners = compute_fluid_corner_offsets(&world, center);
    for &c in &corners {
        assert!(
            (c - 0.10).abs() < 1e-4,
            "Full water block nominal surface drop must be 0.10m, got {c}"
        );
    }
}

#[test]
fn test_organic_river_hydrology_and_flow() {
    use voxel_game_engine::generation::TerrainGenerator;

    let generator = TerrainGenerator::default();

    // Test a sweep across coordinates to verify smooth continuous response without panics
    let mut river_detected = false;
    for x in (0..1000).step_by(50) {
        for z in (0..1000).step_by(50) {
            let (factor, is_river) = generator.organic_river_sample(x as f32, z as f32);
            assert!(factor >= 0.0 && factor <= 1.0);
            if is_river {
                river_detected = true;
                let vel = generator.river_flow_velocity(x as f32, z as f32);
                if factor > 0.10 {
                    assert!(
                        vel.length() > 0.0,
                        "Active river channel must have non-zero downhill flow velocity"
                    );
                }

                // Verify that any submerged river column always has water higher than its bed
                let col = generator.sample_column(x, z);
                if col.is_river && col.water_level.is_some() {
                    let wl = col.water_level.unwrap();
                    assert!(
                        wl > col.terrain_height,
                        "River water level ({}) must be strictly higher than carved bed ({}) at ({}, {})",
                        wl, col.terrain_height, x, z
                    );
                }
            }
        }
    }
    assert!(river_detected, "Sweep across 1000x1000 area must encounter organic river channels");
}

