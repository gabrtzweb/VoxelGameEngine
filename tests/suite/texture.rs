use bevy::prelude::*;
use voxel_game_engine::{
    meshing::{
        VoxelTextureRegistry,
        greedy::FaceDirection,
        textures::{FaceTextureInfo, build_voxel_texture_array},
    },
    world::Voxel,
};

#[test]
fn test_build_array() {
    let (img, _reg) = build_voxel_texture_array();
    let depth = img.texture_descriptor.size.depth_or_array_layers;
    let data_len = img.data.as_ref().map_or(0, |d| d.len());
    println!("TOTAL LAYERS: {}", depth);
    println!("DATA LEN: {}", data_len);
    println!("EXPECTED DATA LEN: {}", depth as usize * 16 * 16 * 4);
    assert!(
        depth <= 2048,
        "Texture array depth must be <= 2048 to fit GPU limits, got {}",
        depth
    );
}

#[test]
fn test_oriented_pillar_textures() {
    let mut registry = VoxelTextureRegistry::default();
    let side = FaceTextureInfo {
        start_layer: 10,
        variant_count: 1,
        frame_count: 1,
    };
    let top = FaceTextureInfo {
        start_layer: 20,
        variant_count: 1,
        frame_count: 1,
    };
    let bottom = FaceTextureInfo {
        start_layer: 21,
        variant_count: 1,
        frame_count: 1,
    };

    registry.register_multi_face(Voxel::OakLog, side, top, bottom);

    // Axis Y (orientation 0): Top/Bottom get caps, lateral get side
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveY, 0)
            .0,
        20
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeY, 0)
            .0,
        21
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveX, 0)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveZ, 0)
            .0,
        10
    );

    // Axis X (orientation 1): PositiveX/NegativeX get caps, Y/Z get side
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveX, 1)
            .0,
        20
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeX, 1)
            .0,
        21
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveY, 1)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeY, 1)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveZ, 1)
            .0,
        10
    );

    // Axis Z (orientation 2): PositiveZ/NegativeZ get caps, X/Y get side
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveZ, 2)
            .0,
        20
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeZ, 2)
            .0,
        21
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveX, 2)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeX, 2)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveY, 2)
            .0,
        10
    );
}

#[test]
fn test_pillar_and_leaves_textures_and_tinting() {
    // 1. Pillar blocks must have distinct bark/side textures, never top rings on sides!
    assert_eq!(Voxel::OakLog.side_texture_name(), Some("tree_oak_bark"));
    assert_eq!(Voxel::OakLog.top_texture_name(), Some("tree_oak_log"));
    assert_eq!(Voxel::BoneBlock.side_texture_name(), Some("bio_bone_side"));
    assert_eq!(Voxel::BoneBlock.top_texture_name(), Some("bio_bone_top"));
    assert_eq!(Voxel::FleshBlock.side_texture_name(), Some("bio_flesh_side"));
    assert_eq!(Voxel::FleshBlock.top_texture_name(), Some("bio_flesh_top"));
    assert_eq!(Voxel::CornBale.side_texture_name(), Some("deco_corn_bale_side"));
    assert_eq!(Voxel::CornBale.top_texture_name(), Some("deco_corn_bale_top"));
    assert_eq!(Voxel::BambooStalk.side_texture_name(), Some("tree_bamboo_stalk"));
    assert_eq!(Voxel::BambooStalk.top_texture_name(), Some("tree_bamboo_stalk_top"));

    // 2. Dead leaves, seasonal maple leaves, cherry, and aspen must NOT be tinted!
    assert!(!Voxel::OakLeavesDead.is_tinted());
    assert!(!Voxel::BirchLeavesDead.is_tinted());
    assert!(!Voxel::PineLeavesDead.is_tinted());
    assert!(!Voxel::MapleLeavesOrange.is_tinted());
    assert!(!Voxel::MapleLeavesRed.is_tinted());
    assert!(!Voxel::MapleLeavesYellow.is_tinted());
    assert!(!Voxel::MapleLeavesDead.is_tinted());
    assert!(!Voxel::CherryLeaves.is_tinted());
    assert!(!Voxel::AspenLeaves.is_tinted());
    assert!(!Voxel::FirLeaves.is_tinted());
    assert!(!Voxel::CypressLeaves.is_tinted());
    assert!(!Voxel::RedwoodLeaves.is_tinted());

    // 3. Living grayscale canopy species MUST be tinted
    assert!(Voxel::OakLeaves.is_tinted());
    assert!(Voxel::BirchLeaves.is_tinted());
    assert!(Voxel::PalmLeaves.is_tinted());
    assert!(Voxel::AcaciaLeaves.is_tinted());

    // 4. Bloody blocks must have the bloody overlay
    assert_eq!(Voxel::BloodyBrain.overlay_texture_name(), Some("bloody_overlay"));
    assert_eq!(Voxel::BloodyFlesh.overlay_texture_name(), Some("bloody_overlay"));
}

