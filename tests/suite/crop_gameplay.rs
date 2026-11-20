use bevy::prelude::*;
use voxel_game_engine::{
    gameplay::crops::{
        advance_crop, evaluate_crop_break_drops, handle_cucurbit_fruit_harvest, is_tilled_soil,
        seed_to_crop,
    },
    world::{Chunk, Voxel, VoxelWorld, WorldModificationStore},
};

#[test]
fn test_tilled_soil_recognition() {
    assert!(is_tilled_soil(Voxel::LoamTilled));
    assert!(is_tilled_soil(Voxel::PeatTilled));
    assert!(is_tilled_soil(Voxel::SiltTilled));

    assert!(!is_tilled_soil(Voxel::LoamDirt));
    assert!(!is_tilled_soil(Voxel::PeatGrass));
    assert!(!is_tilled_soil(Voxel::Air));
}

#[test]
fn test_seed_mapping() {
    assert_eq!(seed_to_crop(Voxel::ItemCarrot), Some(Voxel::CropCarrotsStage0));
    assert_eq!(seed_to_crop(Voxel::ItemPotato), Some(Voxel::CropPotatoesStage0));
    assert_eq!(seed_to_crop(Voxel::ItemBeetrootSeeds), Some(Voxel::CropBeetrootsStage0));
    assert_eq!(seed_to_crop(Voxel::ItemCabbageSeeds), Some(Voxel::CropCabbagesStage0));
    assert_eq!(seed_to_crop(Voxel::ItemGarlicBulb), Some(Voxel::CropGarlicStage0));
    assert_eq!(seed_to_crop(Voxel::ItemOnion), Some(Voxel::CropOnionsStage0));
    assert_eq!(seed_to_crop(Voxel::ItemPepperSeeds), Some(Voxel::CropPepperStage0));
    assert_eq!(seed_to_crop(Voxel::ItemLettuce), Some(Voxel::CropLettuceStage0));
    assert_eq!(seed_to_crop(Voxel::ItemTomatoSeeds), Some(Voxel::CropTomatoesBuddingStage0));
    assert_eq!(seed_to_crop(Voxel::ItemWheatSeeds), Some(Voxel::CropWheatStage0));
    assert_eq!(seed_to_crop(Voxel::ItemCorn), Some(Voxel::CropCornBuddingStage0));
    assert_eq!(seed_to_crop(Voxel::ItemBarleySeeds), Some(Voxel::CropBarleyStage0));
    assert_eq!(seed_to_crop(Voxel::ItemRice), Some(Voxel::CropRiceStage0));
    assert_eq!(seed_to_crop(Voxel::ItemCoffeeSeeds), Some(Voxel::CropCoffeeBuddingStage0));
    assert_eq!(seed_to_crop(Voxel::ItemMelonSeeds), Some(Voxel::CropMelonStemStage0));
    assert_eq!(seed_to_crop(Voxel::ItemPumpkinSeeds), Some(Voxel::CropPumpkinStemStage0));

    assert_eq!(seed_to_crop(Voxel::Air), None);
    assert_eq!(seed_to_crop(Voxel::LoamDirt), None);
}

#[test]
fn test_single_block_crop_advancement() {
    let mut world = VoxelWorld::new();
    world.insert_chunk(IVec3::ZERO, Chunk::new());
    let mut modifications = WorldModificationStore::default();
    let crop_pos = IVec3::new(10, 5, 10);
    let soil_pos = IVec3::new(10, 4, 10);

    world.set_voxel(soil_pos, Voxel::LoamTilled);
    world.set_voxel(crop_pos, Voxel::CropCarrotsStage0);

    // Stage 0 -> 1
    assert!(advance_crop(&mut world, &mut modifications, crop_pos));
    assert_eq!(world.get_voxel(crop_pos), Some(Voxel::CropCarrotsStage1));

    // Stage 1 -> 2
    assert!(advance_crop(&mut world, &mut modifications, crop_pos));
    assert_eq!(world.get_voxel(crop_pos), Some(Voxel::CropCarrotsStage2));

    // Stage 2 -> 3
    assert!(advance_crop(&mut world, &mut modifications, crop_pos));
    assert_eq!(world.get_voxel(crop_pos), Some(Voxel::CropCarrotsStage3));

    // Stage 3 is fully mature, should not advance further
    assert!(!advance_crop(&mut world, &mut modifications, crop_pos));
    assert_eq!(world.get_voxel(crop_pos), Some(Voxel::CropCarrotsStage3));
}

#[test]
fn test_double_tall_crop_advancement() {
    let mut world = VoxelWorld::new();
    world.insert_chunk(IVec3::ZERO, Chunk::new());
    let mut modifications = WorldModificationStore::default();
    let bot_pos = IVec3::new(8, 5, 8);
    let top_pos = IVec3::new(8, 6, 8);
    let soil_pos = IVec3::new(8, 4, 8);

    world.set_voxel(soil_pos, Voxel::PeatTilled);
    world.set_voxel(bot_pos, Voxel::CropBarleyStage0);
    world.set_voxel(top_pos, Voxel::Air);

    // Stage 0 -> 1: single block
    assert!(advance_crop(&mut world, &mut modifications, bot_pos));
    assert_eq!(world.get_voxel(bot_pos), Some(Voxel::CropBarleyStage1));
    assert_eq!(world.get_voxel(top_pos), Some(Voxel::Air));

    // Stage 1 -> 2: bottom becomes CropBarleyBottomStage2, top becomes CropBarleyTopStage2
    assert!(advance_crop(&mut world, &mut modifications, bot_pos));
    assert_eq!(world.get_voxel(bot_pos), Some(Voxel::CropBarleyBottomStage2));
    assert_eq!(world.get_voxel(top_pos), Some(Voxel::CropBarleyTopStage2));

    // Stage 2 -> 3: bottom becomes stage 3, top becomes CropBarleyTopStage3
    assert!(advance_crop(&mut world, &mut modifications, bot_pos));
    assert_eq!(world.get_voxel(bot_pos), Some(Voxel::CropBarleyBottomStage3));
    assert_eq!(world.get_voxel(top_pos), Some(Voxel::CropBarleyTopStage3));

    // Already mature
    assert!(!advance_crop(&mut world, &mut modifications, bot_pos));
}

#[test]
fn test_cucurbit_fruit_growth_and_harvest() {
    let mut world = VoxelWorld::new();
    world.insert_chunk(IVec3::ZERO, Chunk::new());
    let mut modifications = WorldModificationStore::default();
    let stem_pos = IVec3::new(5, 10, 5);
    let soil_stem = IVec3::new(5, 9, 5);

    // Provide supporting ground around stem
    world.set_voxel(soil_stem, Voxel::SiltTilled);
    world.set_voxel(stem_pos, Voxel::CropMelonStemStage0);

    let soil_east = IVec3::new(6, 9, 5);
    let fruit_pos = IVec3::new(6, 10, 5);
    world.set_voxel(soil_east, Voxel::LoamDirt);
    world.set_voxel(fruit_pos, Voxel::Air);

    // Stage 0 -> 1
    assert!(advance_crop(&mut world, &mut modifications, stem_pos));
    assert_eq!(world.get_voxel(stem_pos), Some(Voxel::CropMelonStemStage1));

    // Stage 1 -> 2
    assert!(advance_crop(&mut world, &mut modifications, stem_pos));
    assert_eq!(world.get_voxel(stem_pos), Some(Voxel::CropMelonStemStage2));

    // Stage 2 -> spawns fruit on adjacent ground and attaches stem!
    assert!(advance_crop(&mut world, &mut modifications, stem_pos));
    assert_eq!(world.get_voxel(stem_pos), Some(Voxel::CropMelonStemAttached));
    assert_eq!(world.get_voxel(fruit_pos), Some(Voxel::MelonBlock));

    // Now simulate harvesting the fruit: stem should revert to CropMelonStemStage2
    world.set_voxel(fruit_pos, Voxel::Air);
    handle_cucurbit_fruit_harvest(&mut world, &mut modifications, fruit_pos);
    assert_eq!(world.get_voxel(stem_pos), Some(Voxel::CropMelonStemStage2));
}

#[test]
fn test_crop_break_drops() {
    let mut rng = 123456789u64;

    // Immature crop yields only 1 seed back
    let immature_drops = evaluate_crop_break_drops(Voxel::CropCarrotsStage1, &mut rng);
    assert_eq!(immature_drops, vec![(Voxel::ItemCarrot, 1)]);

    // Mature single-block crop yields produce + seeds
    let mature_carrot = evaluate_crop_break_drops(Voxel::CropCarrotsStage3, &mut rng);
    assert!(mature_carrot.iter().any(|(v, c)| *v == Voxel::ItemCarrot && *c >= 1));

    // Mature wheat yields wheat item + seeds
    let mature_wheat = evaluate_crop_break_drops(Voxel::CropWheatStage3, &mut rng);
    assert!(mature_wheat.iter().any(|(v, c)| *v == Voxel::ItemWheat && *c >= 1));
    assert!(mature_wheat.iter().any(|(v, c)| *v == Voxel::ItemWheatSeeds && *c >= 1));

    // Immature wheat
    let immature_wheat = evaluate_crop_break_drops(Voxel::CropWheatStage0, &mut rng);
    assert_eq!(immature_wheat, vec![(Voxel::ItemWheatSeeds, 1)]);

    // Melon fruit yields melon slices
    let melon_drops = evaluate_crop_break_drops(Voxel::MelonBlock, &mut rng);
    assert!(melon_drops.iter().any(|(v, c)| *v == Voxel::ItemMelonSlice && *c >= 3));

    // Pumpkin block drops pumpkin slices and seeds
    let pumpkin_drops = evaluate_crop_break_drops(Voxel::PumpkinBlock, &mut rng);
    assert!(pumpkin_drops.iter().any(|(v, _)| *v == Voxel::ItemPumpkinSlice));
    assert!(pumpkin_drops.iter().any(|(v, _)| *v == Voxel::ItemPumpkinSeeds));

    // Stick pile drops 1..3 stick items
    let stick_drops = evaluate_crop_break_drops(Voxel::StickPile, &mut rng);
    assert!(stick_drops.iter().any(|(v, c)| *v == Voxel::ItemStick && *c >= 1 && *c <= 3));
}

#[test]
fn test_stick_pile_properties() {
    use voxel_game_engine::world::BlockShape;

    assert_eq!(Voxel::StickPile.default_shape(), BlockShape::StickPile);
    assert_eq!(BlockShape::StickPile.orientation_count(), 4);
    assert!(voxel_game_engine::gameplay::is_voxel_buoyant(Voxel::StickPile));
    assert!(voxel_game_engine::gameplay::is_voxel_buoyant(Voxel::ItemStick));
}

#[test]
fn test_double_tall_crop_orientation_and_break_cascade() {
    use voxel_game_engine::gameplay::interaction::{place_block, remove_block};

    let mut world = VoxelWorld::new();
    world.insert_chunk(IVec3::ZERO, Chunk::new());
    let mut modifications = WorldModificationStore::default();

    // 1. Planting Tomato Stage 0 must only place a single block (no premature top vine)
    let tomato_pos = IVec3::new(4, 5, 4);
    world.set_voxel(tomato_pos - IVec3::Y, Voxel::LoamTilled);
    let placed = place_block(
        &mut world,
        &mut modifications,
        tomato_pos,
        Voxel::CropTomatoesBuddingStage0,
    );
    assert_eq!(placed, vec![tomato_pos]);
    assert_eq!(world.get_voxel(tomato_pos + IVec3::Y), Some(Voxel::Air));

    // 2. Advancing Barley into double-tall Stage 2 must set top orientation = 0 (Floor, upright)
    let barley_pos = IVec3::new(6, 5, 6);
    let barley_top = barley_pos + IVec3::Y;
    world.set_voxel(barley_pos - IVec3::Y, Voxel::LoamTilled);
    world.set_voxel(barley_pos, Voxel::CropBarleyStage1);
    assert!(advance_crop(&mut world, &mut modifications, barley_pos));
    assert_eq!(world.get_voxel(barley_pos), Some(Voxel::CropBarleyBottomStage2));
    assert_eq!(world.get_voxel(barley_top), Some(Voxel::CropBarleyTopStage2));
    assert_eq!(world.get_shape(barley_top).1, 0);

    // 3. Breaking the bottom half must also remove the top half (no floating top)
    let removed = remove_block(&mut world, &mut modifications, barley_pos);
    assert!(removed.contains(&barley_pos));
    assert!(removed.contains(&barley_top));
    assert_eq!(world.get_voxel(barley_top), Some(Voxel::Air));
}

#[test]
fn test_attached_stem_orientation_toward_fruit() {
    let mut world = VoxelWorld::new();
    world.insert_chunk(IVec3::ZERO, Chunk::new());
    let mut modifications = WorldModificationStore::default();

    // Block +X so fruit must spawn at -X (orientation 1)
    let stem_pos = IVec3::new(8, 5, 8);
    world.set_voxel(stem_pos - IVec3::Y, Voxel::SiltTilled);
    world.set_voxel(stem_pos, Voxel::CropPumpkinStemStage2);
    world.set_voxel(stem_pos + IVec3::X, Voxel::LoamDirt); // blocks +X
    world.set_voxel(stem_pos + IVec3::NEG_X - IVec3::Y, Voxel::LoamDirt); // solid ground at -X

    assert!(advance_crop(&mut world, &mut modifications, stem_pos));
    assert_eq!(world.get_voxel(stem_pos), Some(Voxel::CropPumpkinStemAttached));
    assert_eq!(world.get_voxel(stem_pos + IVec3::NEG_X), Some(Voxel::PumpkinBlock));
    assert_eq!(world.get_shape(stem_pos).1, 1); // -X orientation
}

#[test]
fn test_inventory_only_items_cannot_be_placed() {
    assert!(Voxel::ItemStick.is_inventory_only_item());
    assert!(Voxel::ItemMelonSlice.is_inventory_only_item());
    assert!(Voxel::ItemPumpkinSlice.is_inventory_only_item());
    assert!(Voxel::ItemSugarCane.is_inventory_only_item());
    assert!(Voxel::ItemSugar.is_inventory_only_item());
    assert!(Voxel::ItemBoneMeal.is_inventory_only_item());

    assert!(!Voxel::StickPile.is_inventory_only_item());
    assert!(!Voxel::MelonBlock.is_inventory_only_item());
    assert!(!Voxel::PumpkinBlock.is_inventory_only_item());
    assert!(!Voxel::WildSugarCaneBottom.is_inventory_only_item());
}

#[test]
fn test_flora_substrate_placement_rules() {
    // Woodland / shade-loving flora requires fertile soil (dirt, grass, dirty gravel, clay, moss)
    assert!(Voxel::VegeFern.can_be_placed_on_ground(Voxel::LoamGrass));
    assert!(Voxel::VegeFern.can_be_placed_on_ground(Voxel::LoamDirt));
    assert!(Voxel::VegeFern.can_be_placed_on_ground(Voxel::GravelDirty));
    assert!(!Voxel::VegeFern.can_be_placed_on_ground(Voxel::SandCommon));
    assert!(!Voxel::WildPotatoes.can_be_placed_on_ground(Voxel::SandCommon));

    // Hardy grasses, rustic flowers, and adaptable wild crops allowed on both sand and fertile soil
    assert!(Voxel::VegeWildgrass.can_be_placed_on_ground(Voxel::SandCommon));
    assert!(Voxel::WildWheatBottom.can_be_placed_on_ground(Voxel::SandCommon));
    assert!(Voxel::WildWheatBottom.can_be_placed_on_ground(Voxel::LoamGrass));
    assert!(Voxel::VegeSeaOatsBottom.can_be_placed_on_ground(Voxel::SandCommon));
    assert!(Voxel::VegeCactus.can_be_placed_on_ground(Voxel::SandDune));
    assert!(Voxel::VegeDeadBush.can_be_placed_on_ground(Voxel::SandCommon));
    assert!(Voxel::WildSugarCaneBottom.can_be_placed_on_ground(Voxel::SandCommon));
}

#[test]
fn test_arboreal_cocoa_and_coconut_crops() {
    use voxel_game_engine::world::BlockShape;

    let mut world = VoxelWorld::new();
    world.insert_chunk(IVec3::ZERO, Chunk::new());
    let mut modifications = WorldModificationStore::default();
    let mut rng = 987654321u64;

    // 1. Cocoa Pod: shape, orientation preservation across Bone Meal / growth, drops
    let cocoa_pos = IVec3::new(4, 6, 4);
    world.set_voxel(cocoa_pos, Voxel::CropCocoaStage0);
    world.set_shape(cocoa_pos, BlockShape::CocoaPod, 2);
    assert_eq!(Voxel::CropCocoaStage0.default_shape(), BlockShape::CocoaPod);

    assert!(advance_crop(&mut world, &mut modifications, cocoa_pos));
    assert_eq!(world.get_voxel(cocoa_pos), Some(Voxel::CropCocoaStage1));
    assert_eq!(world.get_shape(cocoa_pos), (BlockShape::CocoaPod, 2));

    assert!(advance_crop(&mut world, &mut modifications, cocoa_pos));
    assert_eq!(world.get_voxel(cocoa_pos), Some(Voxel::CropCocoaStage2));
    assert_eq!(world.get_shape(cocoa_pos), (BlockShape::CocoaPod, 2));
    assert!(!advance_crop(&mut world, &mut modifications, cocoa_pos));

    let cocoa_drops = evaluate_crop_break_drops(Voxel::CropCocoaStage2, &mut rng);
    assert!(cocoa_drops.iter().any(|(v, c)| *v == Voxel::ItemCocoaBeans && *c >= 2));

    // 2. Hanging Coconut: shape, growth stages, drops
    let coco_pos = IVec3::new(6, 6, 6);
    world.set_voxel(coco_pos, Voxel::CropCoconutStage0);
    world.set_shape(coco_pos, BlockShape::HangingFruit, 0);
    assert_eq!(Voxel::CropCoconutStage0.default_shape(), BlockShape::HangingFruit);

    assert!(advance_crop(&mut world, &mut modifications, coco_pos));
    assert_eq!(world.get_voxel(coco_pos), Some(Voxel::CropCoconutStage1));
    assert!(advance_crop(&mut world, &mut modifications, coco_pos));
    assert_eq!(world.get_voxel(coco_pos), Some(Voxel::CropCoconutStage2));
    assert!(!advance_crop(&mut world, &mut modifications, coco_pos));

    let coconut_drops = evaluate_crop_break_drops(Voxel::CropCoconutStage2, &mut rng);
    assert!(coconut_drops.iter().any(|(v, c)| *v == Voxel::ItemCoconut && *c >= 1));
}
