use voxel_game_engine::simulation::crops::{CropSimulator, CropStage, CropType, SoilMoisture};

#[test]
fn test_crop_stages_lifecycle() {
    let wheat = CropType::Wheat;
    assert_eq!(wheat.total_stages(), 8);
    assert_eq!(wheat.max_stage(), 7);

    let stage_0 = CropStage::seed(wheat);
    assert!(stage_0.is_seed());
    assert!(!stage_0.is_ripe());
    assert_eq!(stage_0.progress_ratio(), 0.0);

    let stage_1 = stage_0.advance();
    assert!(stage_1.is_sprout());
    assert!(!stage_1.is_ripe());

    // Advance all the way to ripe
    let mut current = stage_1;
    for _ in 0..10 {
        current = current.advance();
    }
    assert!(current.is_ripe());
    assert_eq!(current.stage, 7);
    assert_eq!(current.progress_ratio(), 1.0);
}

#[test]
fn test_soil_hydration_rules() {
    // Within 4 blocks of water -> Hydrated
    assert_eq!(
        SoilMoisture::calculate_hydration(0, false),
        SoilMoisture::Hydrated
    );
    assert_eq!(
        SoilMoisture::calculate_hydration(4, false),
        SoilMoisture::Hydrated
    );

    // 5-6 blocks without rain -> Moist
    assert_eq!(
        SoilMoisture::calculate_hydration(5, false),
        SoilMoisture::Moist
    );
    assert_eq!(
        SoilMoisture::calculate_hydration(6, false),
        SoilMoisture::Moist
    );

    // Far from water without rain -> Dry
    assert_eq!(
        SoilMoisture::calculate_hydration(7, false),
        SoilMoisture::Dry
    );
    assert_eq!(
        SoilMoisture::calculate_hydration(20, false),
        SoilMoisture::Dry
    );

    // Rain provides at least Moist even when far away
    assert_eq!(
        SoilMoisture::calculate_hydration(15, true),
        SoilMoisture::Moist
    );
}

#[test]
fn test_crop_growth_conditions_and_ticks() {
    let crop = CropType::Wheat;
    let seed = CropStage::seed(crop);

    // Insufficient light (< 9) -> cannot grow
    assert!(!CropSimulator::can_grow(
        crop,
        seed,
        SoilMoisture::Hydrated,
        5
    ));

    // Dry soil for wheat (requires Moist) -> cannot grow
    assert!(!CropSimulator::can_grow(crop, seed, SoilMoisture::Dry, 12));

    // Sufficient light and hydration -> can grow
    assert!(CropSimulator::can_grow(
        crop,
        seed,
        SoilMoisture::Hydrated,
        12
    ));

    // Evaluate tick with a successful roll
    let next = CropSimulator::evaluate_tick(crop, seed, SoilMoisture::Hydrated, 12, 0.1);
    assert_eq!(next.stage, 1);

    // Evaluate tick with an unsuccessful roll
    let same = CropSimulator::evaluate_tick(crop, seed, SoilMoisture::Hydrated, 12, 0.99);
    assert_eq!(same.stage, 0);
}

#[test]
fn test_crop_harvest_yield() {
    let crop = CropType::Carrot;
    let seed = CropStage::seed(crop);
    let ripe = CropStage::new(crop.max_stage(), crop.max_stage());

    // Immature crop drops only 1 seed
    let immature_yield = CropSimulator::harvest(crop, seed);
    assert_eq!(immature_yield.produce_count, 0);
    assert_eq!(immature_yield.seed_count, 1);

    // Ripe crop drops produce and seeds
    let ripe_yield = CropSimulator::harvest(crop, ripe);
    assert!(ripe_yield.produce_count >= 2);
    assert!(ripe_yield.seed_count >= 1);
}
