use voxel_game_engine::{
    generation::{
        biome::BiomeType,
        strata::StrataGenerator,
    },
    world::Voxel,
};

#[test]
fn test_subterranean_strata_depths() {
    let strata = StrataGenerator::default();
    let config = BiomeType::AlpineHighlands.config();

    // Bedrock floor strictly Mantle at -254
    let mantle_block = strata.solid_voxel_at(0, -254, 0, 100, &config, 12345);
    assert_eq!(mantle_block, Voxel::Mantle);

    // Deep crust at -150 should be Slate or Gabbro
    let deep_block = strata.solid_voxel_at(0, -150, 0, 100, &config, 12345);
    assert!(
        matches!(deep_block, Voxel::Slate | Voxel::Gabbro | Voxel::Marble),
        "Expected deep crust rock (Slate/Gabbro/Marble), got {:?}",
        deep_block
    );

    // Middle crust at -70 should be Dolomite, Andesite, Limestone, Argillite, or Granite/Marble pocket
    let mid_block = strata.solid_voxel_at(0, -70, 0, 100, &config, 12345);
    assert!(
        matches!(
            mid_block,
            Voxel::Dolomite
                | Voxel::Andesite
                | Voxel::Limestone
                | Voxel::Argillite
                | Voxel::Granite
                | Voxel::Marble
        ),
        "Expected middle crust rock, got {:?}",
        mid_block
    );
}

#[test]
fn test_surface_crust_mother_rocks() {
    let strata = StrataGenerator::default();

    // 1. Taiga (Permafrost subsoil -> must yield Andesite mother rock in 4-8 block surface crust, never Slate!)
    let taiga = BiomeType::SiberianTaiga.config();
    let taiga_crust = strata.solid_voxel_at(0, 50, 0, 6, &taiga, 12345);
    assert_eq!(
        taiga_crust,
        Voxel::Andesite,
        "Taiga permafrost surface crust must yield Andesite, got {:?}",
        taiga_crust
    );

    // 2. Loam soil -> Granite mother rock
    let plains = BiomeType::EmeraldPeaks.config();
    let plains_crust = strata.solid_voxel_at(0, 50, 0, 6, &plains, 12345);
    assert_eq!(
        plains_crust,
        Voxel::Granite,
        "Plains loam surface crust must yield Granite, got {:?}",
        plains_crust
    );

    // 3. Savanna (Laterite soil -> Basalt mother rock)
    let savanna = BiomeType::FracturedSavanna.config();
    let savanna_crust = strata.solid_voxel_at(0, 50, 0, 6, &savanna, 12345);
    assert_eq!(
        savanna_crust,
        Voxel::Basalt,
        "Savanna laterite surface crust must yield Basalt, got {:?}",
        savanna_crust
    );

    // 4. Desert (Sand subsoil -> SandstoneCommon mother rock below subsoil depth)
    let desert = BiomeType::AncientSands.config();
    let desert_crust = strata.solid_voxel_at(0, 50, 0, desert.subsoil_depth + 1, &desert, 12345);
    assert_eq!(
        desert_crust,
        Voxel::SandstoneCommon,
        "Desert sand surface crust must yield SandstoneCommon, got {:?}",
        desert_crust
    );
}

#[test]
fn test_crossmesh_marine_flora() {
    assert!(Voxel::RedAlgae.is_cross_billboard());
    assert!(Voxel::Seaweed.is_cross_billboard());
    assert!(Voxel::Sargassum.is_cross_billboard());
}

#[test]
fn test_rock_outcrops_appear_in_both_categories() {
    use voxel_game_engine::menu::creative_inventory::CreativeCategory;

    assert!(CreativeCategory::Rocks.matches(Voxel::AndesiteGrass));
    assert!(CreativeCategory::Soils.matches(Voxel::AndesiteGrass));

    assert!(CreativeCategory::Rocks.matches(Voxel::GraniteGrass));
    assert!(CreativeCategory::Soils.matches(Voxel::GraniteGrass));

    assert!(CreativeCategory::Rocks.matches(Voxel::DolomiteSnow));
    assert!(CreativeCategory::Soils.matches(Voxel::DolomiteSnow));
}
