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
