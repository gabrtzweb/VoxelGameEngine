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

