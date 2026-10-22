use bevy::prelude::*;
use voxel_game_engine::world::block::{BlockShape, NaturalRotationMode, PillarAxis, Voxel};

#[test]
fn test_natural_rotation_modes() {
    // Natural isotropic blocks: rock, sand, gravel, dirt
    assert_eq!(
        Voxel::Granite.natural_rotation_mode(),
        NaturalRotationMode::Isotropic
    );
    assert_eq!(
        Voxel::Basalt.natural_rotation_mode(),
        NaturalRotationMode::Isotropic
    );
    assert_eq!(
        Voxel::Andesite.natural_rotation_mode(),
        NaturalRotationMode::Isotropic
    );
    assert_eq!(
        Voxel::SandCommon.natural_rotation_mode(),
        NaturalRotationMode::Isotropic
    );
    assert_eq!(
        Voxel::LoamDirt.natural_rotation_mode(),
        NaturalRotationMode::Isotropic
    );
    assert_eq!(
        Voxel::GravelSandy.natural_rotation_mode(),
        NaturalRotationMode::Isotropic
    );

    // Natural soils with side overlays: top rotated 4-way, side flipped horizontally only
    assert_eq!(
        Voxel::LoamGrass.natural_rotation_mode(),
        NaturalRotationMode::SoilSide
    );
    assert_eq!(
        Voxel::PeatGrass.natural_rotation_mode(),
        NaturalRotationMode::SoilSide
    );
    assert_eq!(
        Voxel::BasaltSnow.natural_rotation_mode(),
        NaturalRotationMode::SoilSide
    );
    assert_eq!(
        Voxel::LoamSnow.natural_rotation_mode(),
        NaturalRotationMode::SoilSide
    );

    // Directional / architectural blocks: must NOT rotate
    assert_eq!(
        Voxel::OakLog.natural_rotation_mode(),
        NaturalRotationMode::None
    );
    assert_eq!(
        Voxel::BirchPlanks.natural_rotation_mode(),
        NaturalRotationMode::None
    );
    assert_eq!(
        Voxel::Bricks.natural_rotation_mode(),
        NaturalRotationMode::None
    );
    assert_eq!(
        Voxel::BrickRed.natural_rotation_mode(),
        NaturalRotationMode::None
    );
    assert_eq!(
        Voxel::Barrel.natural_rotation_mode(),
        NaturalRotationMode::None
    );
    assert_eq!(
        Voxel::EmitTorchFire.natural_rotation_mode(),
        NaturalRotationMode::None
    );
}

#[test]
fn test_axis_aligned_pillar_properties() {
    // Normal to axis conversion
    assert_eq!(PillarAxis::from_normal(IVec3::new(1, 0, 0)), PillarAxis::X);
    assert_eq!(PillarAxis::from_normal(IVec3::new(-1, 0, 0)), PillarAxis::X);
    assert_eq!(PillarAxis::from_normal(IVec3::new(0, 0, 1)), PillarAxis::Z);
    assert_eq!(PillarAxis::from_normal(IVec3::new(0, 0, -1)), PillarAxis::Z);
    assert_eq!(PillarAxis::from_normal(IVec3::new(0, 1, 0)), PillarAxis::Y);
    assert_eq!(PillarAxis::from_normal(IVec3::new(0, -1, 0)), PillarAxis::Y);

    // Axis-aligned pillar detection
    assert!(Voxel::OakLog.is_axis_aligned());
    assert!(Voxel::BaobabLog.is_axis_aligned());
    assert!(Voxel::OakWood.is_axis_aligned());
    assert!(Voxel::BaobabWood.is_axis_aligned());
    assert!(Voxel::BoneBlock.is_axis_aligned());
    assert!(Voxel::FleshBlock.is_axis_aligned());
    assert!(Voxel::HayBale.is_axis_aligned());
    assert!(!Voxel::BambooStalk.is_axis_aligned());
    assert_eq!(Voxel::BambooStalk.default_shape(), BlockShape::Bamboo);
    assert_eq!(Voxel::BambooLeaves.default_shape(), BlockShape::Bamboo);

    // Non-pillar blocks
    assert!(!Voxel::OakPlanks.is_axis_aligned());
    assert!(!Voxel::Granite.is_axis_aligned());
    assert!(!Voxel::LoamDirt.is_axis_aligned());

    // BlockShape::Full has 3 orientations (Y, X, Z)
    assert_eq!(BlockShape::Full.orientation_count(), 3);
    assert_eq!(BlockShape::Full.orientation_name(0), "Vertical (Y)");
    assert_eq!(BlockShape::Full.orientation_name(1), "Horizontal (X)");
    assert_eq!(BlockShape::Full.orientation_name(2), "Horizontal (Z)");
}

#[test]
fn test_voxel_discriminants_continuity() {
    for id in 0..=613u16 {
        let maybe_voxel = Voxel::from_u16(id);
        assert!(
            maybe_voxel.is_some(),
            "Voxel discriminant {id} must be valid"
        );
        assert_eq!(maybe_voxel.unwrap() as u16, id);
    }
    assert_eq!(Voxel::from_u16(614), None);
    assert_eq!(Voxel::from_u16(1000), None);

    assert_eq!(Voxel::ALL.len(), 611);
    assert_eq!(Voxel::ALL[0], Voxel::DebugAccept);
    assert_eq!(Voxel::ALL[610], Voxel::FungiVerdigrisCap);
}

#[test]
fn test_block_shapes_all() {
    assert_eq!(BlockShape::all().len(), 10);
    assert_eq!(BlockShape::shapable().len(), 4);
}

#[test]
fn test_flora_support_and_collision() {
    assert!(Voxel::PebbleFlint.requires_floor_support());
    assert!(Voxel::PebbleFlintCluster.requires_floor_support());
    assert!(Voxel::PebbleQuartz.requires_floor_support());
    assert!(Voxel::PebbleQuartzCluster.requires_floor_support());
    assert!(Voxel::PebbleChert.requires_floor_support());
    assert!(Voxel::PebbleChertCluster.requires_floor_support());
    assert_eq!(Voxel::PebbleChert.default_shape(), BlockShape::Pebble);
    assert_eq!(
        Voxel::PebbleChertCluster.default_shape(),
        BlockShape::PebbleCluster
    );
    assert!(Voxel::FungiAmanita.requires_floor_support());
    assert!(Voxel::FungiPorcini.requires_floor_support());
    assert!(Voxel::Seagrass.requires_floor_support());
    assert!(Voxel::BrainCoralFan.requires_floor_support());
    assert!(Voxel::BubbleCoralFan.requires_floor_support());
    assert!(Voxel::Kelp.requires_floor_support());
    assert!(!Voxel::LoamGrass.requires_floor_support());

    // Mushrooms have no solid player collision
    assert!(!Voxel::FungiAmanita.is_collidable());
    assert!(!Voxel::FungiPorcini.is_collidable());
    // Regular leaves remain solid blocks
    assert!(Voxel::OakLeaves.is_collidable());
}
