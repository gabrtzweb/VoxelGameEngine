use bevy::prelude::*;
use voxel_game_engine::{
    meshing::{
        ChunkMeshRegistry,
        culling::{
            FACE_OFFSETS, OPPOSITE_FACES, get_camera_reachable_faces, get_chunk_permeability,
            is_position_underground,
        },
    },
    world::{Chunk, Voxel, VoxelWorld},
};

#[test]
fn test_face_offsets_and_opposite_faces() {
    for face in 0..6 {
        let opp = OPPOSITE_FACES[face];
        assert_eq!(
            OPPOSITE_FACES[opp], face,
            "Opposite of opposite face {face} must equal itself"
        );
        assert_eq!(
            FACE_OFFSETS[face] + FACE_OFFSETS[opp],
            IVec3::ZERO,
            "Offsets for face {face} and opposite {opp} must cancel out to ZERO"
        );
    }
}

#[test]
fn test_camera_reachable_faces_empty_chunk() {
    let mut world = VoxelWorld::new();
    let chunk_coord = IVec3::new(0, 0, 0);
    world.insert_chunk(chunk_coord, Chunk::new());

    let cam_pos = Vec3::new(8.0, 8.0, 8.0);
    let reachable = get_camera_reachable_faces(cam_pos, chunk_coord, 0x3F, Some(&world));
    assert_eq!(
        reachable, 0x3F,
        "All 6 faces must be reachable from center of empty chunk"
    );
}

#[test]
fn test_camera_reachable_faces_solid_chunk() {
    let mut world = VoxelWorld::new();
    let chunk_coord = IVec3::new(0, 0, 0);
    world.insert_chunk(chunk_coord, Chunk::filled(Voxel::Granite));

    let cam_pos = Vec3::new(8.0, 8.0, 8.0);
    let reachable = get_camera_reachable_faces(cam_pos, chunk_coord, 0, Some(&world));
    assert_eq!(
        reachable, 0,
        "No faces must be reachable when camera is inside solid chunk"
    );
}

#[test]
fn test_is_position_underground_detection() {
    let mut world = VoxelWorld::new();

    // 1. Surface chunk with open air
    let surface_chunk_coord = IVec3::new(0, 2, 0);
    let mut surface_chunk = Chunk::new();
    // Ground at bottom Y=0..=2
    for y in 0..=2 {
        for z in 0..16 {
            for x in 0..16 {
                surface_chunk.set(x, y, z, Voxel::Granite);
            }
        }
    }
    world.insert_chunk(surface_chunk_coord, surface_chunk);

    // Player standing at Y = 40 (inside surface chunk at local Y = 8, well above ground Y=2)
    let player_on_surface = Vec3::new(8.0, 40.0, 8.0);
    assert!(
        !is_position_underground(player_on_surface, Some(&world)),
        "Player under open sky must be identified as on surface"
    );

    // 2. Subterranean chunk
    let sub_chunk_coord = IVec3::new(0, -2, 0);
    let mut sub_chunk = Chunk::new();
    sub_chunk.set_subterranean(true);
    world.insert_chunk(sub_chunk_coord, sub_chunk);

    let player_in_subterranean = Vec3::new(8.0, -25.0, 8.0);
    assert!(
        is_position_underground(player_in_subterranean, Some(&world)),
        "Player inside subterranean chunk must be identified as underground"
    );

    // 3. Player under solid roof
    let cave_chunk_coord = IVec3::new(5, 1, 5);
    let mut cave_chunk = Chunk::new();
    // Solid roof across Y=6..15
    for y in 6..16 {
        for z in 0..16 {
            for x in 0..16 {
                cave_chunk.set(x, y, z, Voxel::Granite);
            }
        }
    }
    world.insert_chunk(cave_chunk_coord, cave_chunk);

    // Player inside the cave at local Y=2 (world Y = 18) under the solid stone roof
    let player_under_roof = Vec3::new(5.0 * 16.0 + 8.0, 18.0, 5.0 * 16.0 + 8.0);
    assert!(
        is_position_underground(player_under_roof, Some(&world)),
        "Player under solid ceiling blocks must be identified as underground"
    );
}

#[test]
fn test_subterranean_chunk_permeability_culling() {
    let mut world = VoxelWorld::new();
    let registry = ChunkMeshRegistry::default();

    let sub_chunk_coord = IVec3::new(0, -3, 0);
    let mut sub_chunk = Chunk::new();
    sub_chunk.set_subterranean(true);
    world.insert_chunk(sub_chunk_coord, sub_chunk);

    // When player is on the surface, permeability must be 0 (culled from BFS traversal)
    let perm_surface = get_chunk_permeability(sub_chunk_coord, &registry, Some(&world), false);
    assert_eq!(
        perm_surface, 0,
        "Subterranean chunk must have 0 permeability when player is on the surface"
    );

    // When player is underground, permeability allows traversal through open air
    let perm_underground = get_chunk_permeability(sub_chunk_coord, &registry, Some(&world), true);
    assert_ne!(
        perm_underground, 0,
        "Subterranean chunk with air must have non-zero permeability when player is underground"
    );
}
