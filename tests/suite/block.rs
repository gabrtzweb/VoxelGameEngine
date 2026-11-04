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
    assert!(Voxel::OakWood.is_axis_aligned());
    assert!(Voxel::BloodyBone.is_axis_aligned());
    assert!(Voxel::BoneBlock.is_axis_aligned());
    assert!(Voxel::FleshBlock.is_axis_aligned());
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
    for id in 0..=531u16 {
        let maybe_voxel = Voxel::from_u16(id);
        assert!(
            maybe_voxel.is_some(),
            "Voxel discriminant {id} must be valid"
        );
        assert_eq!(maybe_voxel.unwrap() as u16, id);
    }
    assert_eq!(Voxel::from_u16(532), None);
    assert_eq!(Voxel::from_u16(1000), None);

    assert_eq!(Voxel::ALL.len(), 529);
    assert_eq!(Voxel::ALL[0], Voxel::FrostBlueIce);
    assert_eq!(Voxel::ALL[443], Voxel::FungiVerdigrisCap);
    assert_eq!(Voxel::ALL[444], Voxel::VegeWildgrass);
    assert_eq!(Voxel::ALL[513], Voxel::BullKelpPlant);
    assert_eq!(Voxel::ALL[527], Voxel::SlateRubble);
    assert_eq!(Voxel::ALL[528], Voxel::ObsidianSpike);
}

#[test]
fn test_block_shapes_all() {
    assert_eq!(BlockShape::all().len(), 16);
    assert_eq!(Voxel::AndesiteRubble.default_shape(), BlockShape::Rubble);
    assert!(Voxel::AndesiteRubble.is_shape_locked());
    assert!(Voxel::AndesiteRubble.requires_floor_support());
    assert_eq!(Voxel::SpeleothemKarst.default_shape(), BlockShape::Speleothem);
    assert!(Voxel::SpeleothemKarst.is_shape_locked());
    assert_eq!(Voxel::ObsidianSpike.default_shape(), BlockShape::Speleothem);
    assert!(Voxel::ObsidianSpike.is_shape_locked());
    assert_eq!(BlockShape::shapable().len(), 4);
    assert_eq!(Voxel::SpiderEgg.default_shape(), BlockShape::Cocoon);
    assert_eq!(Voxel::SilkCocoon.default_shape(), BlockShape::Cocoon);
    assert!(Voxel::SpiderEgg.is_shape_locked());
    assert!(Voxel::SilkCocoon.is_shape_locked());
    assert!(Voxel::BirdNest.is_shape_locked());
    assert!(Voxel::HayNest.is_shape_locked());
    assert_eq!(Voxel::Crate.default_shape(), BlockShape::Full);
    assert!(Voxel::Crate.is_solid_opaque());
    assert_eq!(Voxel::Basket.default_shape(), BlockShape::Basket);
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

    // Phase 6: Wild Flora & Flowers
    assert!(Voxel::VegeWildgrass.requires_floor_support());
    assert!(Voxel::FlowerDandelion.requires_floor_support());
    assert!(!Voxel::VegeCaveVines.requires_floor_support()); // hanging plant
    assert!(!Voxel::VegeWildgrass.is_collidable());
    assert!(!Voxel::FlowerPoppy.is_collidable());
    assert!(Voxel::VegeCactus.is_collidable()); // Saguaro column is solid!
    assert!(!Voxel::VegeCactus.is_solid_opaque());
    assert_eq!(Voxel::VegeWildgrass.default_shape(), BlockShape::CrossBillboard);
    assert_eq!(Voxel::FlowerDandelion.default_shape(), BlockShape::CrossBillboard);
    assert_eq!(Voxel::VegeCactus.default_shape(), BlockShape::Cactus);
    assert_eq!(Voxel::VegeLilyPad.default_shape(), BlockShape::FlatPad);
    assert_eq!(Voxel::FlowerWhitePetals.default_shape(), BlockShape::FlatPad);
    assert_eq!(Voxel::VegeVines.default_shape(), BlockShape::WallDecal);
    assert!(Voxel::CrystalAmethystCluster.is_collidable());
    assert!(!Voxel::BrainCoralFan.is_collidable());
}

#[test]
fn test_painted_blocks_and_solid_light_tint_colors() {
    let families = [
        (Voxel::BrickBlack as u16, Voxel::BrickYellow as u16),
        (Voxel::FabricBlack as u16, Voxel::FabricYellow as u16),
        (Voxel::PlasterBlack as u16, Voxel::PlasterYellow as u16),
        (Voxel::ShinglesBlack as u16, Voxel::ShinglesYellow as u16),
        (Voxel::WoolBlack as u16, Voxel::WoolYellow as u16),
    ];

    for (start, end) in families {
        assert_eq!(end - start, 15, "Each painted family must have 16 colors");
        for idx in 0..16u16 {
            let v = Voxel::from_u16(start + idx).unwrap();
            assert!(v.is_painted(), "{v:?} must be marked as painted");
            assert_eq!(
                v.paint_color(),
                Some(Voxel::PAINT_PALETTE[idx as usize]),
                "{v:?} must match palette index {idx}"
            );
            assert_eq!(v.tint_color(), Voxel::PAINT_PALETTE[idx as usize]);
        }
    }

    // Unpainted base blocks and lamp blocks must never be painted
    let unpainted = [
        Voxel::Bricks,
        Voxel::Fabric,
        Voxel::Plaster,
        Voxel::Shingles,
        Voxel::Wool,
        Voxel::Crate,
        Voxel::EmitLampWarm,
        Voxel::EmitLampBlue,
        Voxel::EmitLampGreen,
        Voxel::EmitLampRed,
    ];
    for v in unpainted {
        assert!(!v.is_painted(), "{v:?} must not be painted");
        assert_eq!(v.paint_color(), None, "{v:?} must have no paint color");
        assert_eq!(v.tint_color(), [1.0, 1.0, 1.0, 1.0]);
    }
}


#[test]
fn test_speleothem_properties_and_part_determination() {
    use bevy::prelude::IVec3;
    use voxel_game_engine::{
        meshing::shapes::fixtures::{SpeleothemDirection, SpeleothemPiece, determine_speleothem_part},
        world::{Chunk, VoxelWorld},
    };

    // 1. Check all 5 speleothems properties
    let speleos = [
        Voxel::SpeleothemKarst,
        Voxel::SpeleothemCalcite,
        Voxel::SpeleothemBlueIce,
        Voxel::SpeleothemSaltStraw,
        Voxel::ObsidianSpike,
    ];
    for s in speleos {
        assert!(s.is_speleothem(), "{s:?} must be speleothem");
        assert_eq!(s.default_shape(), BlockShape::Speleothem);
        assert!(s.is_shape_locked());
        assert!(!s.is_solid_opaque());
        assert!(s.has_custom_mesh());
        assert!(s.is_collidable());
    }

    // 2. Part determination in world
    let mut world = VoxelWorld::new();
    world.insert_chunk(IVec3::ZERO, Chunk::new());

    // Single hanging stalactite at Y=10 attached to ceiling at Y=11
    world.set_voxel(IVec3::new(0, 11, 0), Voxel::Andesite);
    world.set_voxel(IVec3::new(0, 10, 0), Voxel::SpeleothemKarst);
    let (dir, piece) = determine_speleothem_part(&world, IVec3::new(0, 10, 0), Voxel::SpeleothemKarst);
    assert_eq!(dir, SpeleothemDirection::Down);
    assert_eq!(piece, SpeleothemPiece::Tip);

    // 2-block hanging stalactite at Y=10 (frustum) and Y=9 (tip)
    world.set_voxel(IVec3::new(0, 9, 0), Voxel::SpeleothemKarst);
    let (dir_top, piece_top) = determine_speleothem_part(&world, IVec3::new(0, 10, 0), Voxel::SpeleothemKarst);
    let (dir_bot, piece_bot) = determine_speleothem_part(&world, IVec3::new(0, 9, 0), Voxel::SpeleothemKarst);
    assert_eq!(dir_top, SpeleothemDirection::Down);
    assert_eq!(piece_top, SpeleothemPiece::Frustum);
    assert_eq!(dir_bot, SpeleothemDirection::Down);
    assert_eq!(piece_bot, SpeleothemPiece::Tip);

    // Merged column of 4 blocks between ceiling (Y=11) and floor (Y=6): Y=10, 9, 8, 7
    world.set_voxel(IVec3::new(0, 6, 0), Voxel::Andesite);
    world.set_voxel(IVec3::new(0, 8, 0), Voxel::SpeleothemKarst);
    world.set_voxel(IVec3::new(0, 7, 0), Voxel::SpeleothemKarst);

    let (d10, p10) = determine_speleothem_part(&world, IVec3::new(0, 10, 0), Voxel::SpeleothemKarst);
    let (d9, p9) = determine_speleothem_part(&world, IVec3::new(0, 9, 0), Voxel::SpeleothemKarst);
    let (d8, p8) = determine_speleothem_part(&world, IVec3::new(0, 8, 0), Voxel::SpeleothemKarst);
    let (d7, p7) = determine_speleothem_part(&world, IVec3::new(0, 7, 0), Voxel::SpeleothemKarst);

    assert_eq!(d10, SpeleothemDirection::Down);
    assert_eq!(p10, SpeleothemPiece::Base);
    assert_eq!(d9, SpeleothemDirection::Down);
    assert_eq!(p9, SpeleothemPiece::TipMerge);

    assert_eq!(d8, SpeleothemDirection::Up);
    assert_eq!(p8, SpeleothemPiece::TipMerge);
    assert_eq!(d7, SpeleothemDirection::Up);
    assert_eq!(p7, SpeleothemPiece::Base);
}
