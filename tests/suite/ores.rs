use voxel_game_engine::{
    generation::{
        biome::BiomeType,
        ores::{self, CrystalGeodeType, MineralType},
        strata::StrataGenerator,
    },
    world::Voxel,
};

#[test]
fn test_all_54_ore_vein_mappings() {
    // 1. Coal (6 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::SandstoneCommon), Some(Voxel::OreCoalSandstoneCommon));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Granite), Some(Voxel::OreCoalGranite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Dolomite), Some(Voxel::OreCoalDolomite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Limestone), Some(Voxel::OreCoalLimestone));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Chalk), Some(Voxel::OreCoalChalk));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Argillite), Some(Voxel::OreCoalArgillite));

    // 2. Copper (6 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::SandstoneCommon), Some(Voxel::OreCopperSandstoneCommon));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::SandstoneRed), Some(Voxel::OreCopperSandstoneRed));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::SandstoneWhite), Some(Voxel::OreCopperSandstoneWhite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::Granite), Some(Voxel::OreCopperGranite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::Andesite), Some(Voxel::OreCopperAndesite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::Dolomite), Some(Voxel::OreCopperDolomite));

    // 3. Tin (4 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Tin, Voxel::Granite), Some(Voxel::OreTinGranite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Tin, Voxel::Andesite), Some(Voxel::OreTinAndesite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Tin, Voxel::Dolomite), Some(Voxel::OreTinDolomite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Tin, Voxel::Chalk), Some(Voxel::OreTinChalk));

    // 4. Iron (6 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Granite), Some(Voxel::OreIronGranite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Andesite), Some(Voxel::OreIronAndesite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Dolomite), Some(Voxel::OreIronDolomite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Slate), Some(Voxel::OreIronSlate));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Gabbro), Some(Voxel::OreIronGabbro));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Basalt), Some(Voxel::OreIronBasalt));

    // 5. Zinc (4 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Zinc, Voxel::Andesite), Some(Voxel::OreZincAndesite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Zinc, Voxel::Dolomite), Some(Voxel::OreZincDolomite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Zinc, Voxel::Argillite), Some(Voxel::OreZincArgillite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Zinc, Voxel::Slate), Some(Voxel::OreZincSlate));

    // 6. Gold (4 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Gold, Voxel::SandstoneRed), Some(Voxel::OreGoldSandstoneRed));
    assert_eq!(ores::host_to_ore_vein(MineralType::Gold, Voxel::Granite), Some(Voxel::OreGoldGranite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Gold, Voxel::Slate), Some(Voxel::OreGoldSlate));
    assert_eq!(ores::host_to_ore_vein(MineralType::Gold, Voxel::Gabbro), Some(Voxel::OreGoldGabbro));

    // 7. Silver (3 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Silver, Voxel::Andesite), Some(Voxel::OreSilverAndesite));
    assert_eq!(ores::host_to_ore_vein(MineralType::Silver, Voxel::Slate), Some(Voxel::OreSilverSlate));
    assert_eq!(ores::host_to_ore_vein(MineralType::Silver, Voxel::Marble), Some(Voxel::OreSilverMarble));

    // 8. Platinum (2 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Platinum, Voxel::Gabbro), Some(Voxel::OrePlatinumGabbro));
    assert_eq!(ores::host_to_ore_vein(MineralType::Platinum, Voxel::Basalt), Some(Voxel::OrePlatinumBasalt));

    // 9. Uranium (2 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Uranium, Voxel::Slate), Some(Voxel::OreUraniumSlate));
    assert_eq!(ores::host_to_ore_vein(MineralType::Uranium, Voxel::Gabbro), Some(Voxel::OreUraniumGabbro));

    // 10. Lazurite (2 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Lazurite, Voxel::Limestone), Some(Voxel::OreLazuriteLimestone));
    assert_eq!(ores::host_to_ore_vein(MineralType::Lazurite, Voxel::Marble), Some(Voxel::OreLazuriteMarble));

    // 11. Diamond (2 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Diamond, Voxel::Gabbro), Some(Voxel::GemDiamondGabbro));
    assert_eq!(ores::host_to_ore_vein(MineralType::Diamond, Voxel::Basalt), Some(Voxel::GemDiamondBasalt));

    // 12. Emerald (2 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Emerald, Voxel::Slate), Some(Voxel::GemEmeraldSlate));
    assert_eq!(ores::host_to_ore_vein(MineralType::Emerald, Voxel::Marble), Some(Voxel::GemEmeraldMarble));

    // 13. Sulfur (1 host rock)
    assert_eq!(ores::host_to_ore_vein(MineralType::Sulfur, Voxel::Basalt), Some(Voxel::OreSulfurBasalt));

    // 14. Cinnabar (1 host rock)
    assert_eq!(ores::host_to_ore_vein(MineralType::Cinnabar, Voxel::Basalt), Some(Voxel::OreCinnabarBasalt));

    // 15. Opal (1 host rock, exclusive)
    assert_eq!(ores::host_to_ore_vein(MineralType::Opal, Voxel::SandstoneWhite), Some(Voxel::GemOpalSandstoneWhite));

    // 16. Onyx (1 host rock, exclusive)
    assert_eq!(ores::host_to_ore_vein(MineralType::Onyx, Voxel::Slate), Some(Voxel::GemOnyxSlate));

    // 17. Gem Vein (7 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Dolomite), Some(Voxel::GemVeinDolomite));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Granite), Some(Voxel::GemVeinGranite));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Andesite), Some(Voxel::GemVeinAndesite));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Slate), Some(Voxel::GemVeinSlate));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Gabbro), Some(Voxel::GemVeinGabbro));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Basalt), Some(Voxel::GemVeinBasalt));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Marble), Some(Voxel::GemVeinMarble));
}

#[test]
fn test_crystal_geodes() {
    assert_eq!(CrystalGeodeType::Amethyst.shell_voxel(), Voxel::Basalt);
    assert_eq!(CrystalGeodeType::Amethyst.crystal_voxel(), Voxel::CrystalAmethystBlock);

    assert_eq!(CrystalGeodeType::Aquamarine.shell_voxel(), Voxel::CaveCalcite);
    assert_eq!(CrystalGeodeType::Aquamarine.crystal_voxel(), Voxel::CrystalAquamarineBlock);

    assert_eq!(CrystalGeodeType::Citrine.shell_voxel(), Voxel::Basalt);
    assert_eq!(CrystalGeodeType::Citrine.crystal_voxel(), Voxel::CrystalCitrineBlock);

    assert_eq!(CrystalGeodeType::Fluorite.shell_voxel(), Voxel::CaveCalcite);
    assert_eq!(CrystalGeodeType::Fluorite.crystal_voxel(), Voxel::CrystalFluoriteBlock);

    assert_eq!(CrystalGeodeType::PinkSalt.shell_voxel(), Voxel::CaveCalcite);
    assert_eq!(CrystalGeodeType::PinkSalt.crystal_voxel(), Voxel::CrystalPinkSaltBlock);

    assert_eq!(CrystalGeodeType::Selenite.shell_voxel(), Voxel::CaveCalcite);
    assert_eq!(CrystalGeodeType::Selenite.crystal_voxel(), Voxel::CrystalSeleniteBlock);
}

#[test]
fn test_subterranean_strata_depths() {
    let strata = StrataGenerator::default();
    let plains = BiomeType::Plains.config();

    // Bedrock floor strictly Mantle at -254
    let mantle_block = strata.solid_voxel_at(0, -254, 0, 100, &plains, 12345);
    assert_eq!(mantle_block, Voxel::Mantle);

    // Deep crust at -150 should be Slate or Gabbro
    let deep_block = strata.solid_voxel_at(0, -150, 0, 100, &plains, 12345);
    assert!(
        matches!(deep_block, Voxel::Slate | Voxel::Gabbro | Voxel::Marble),
        "Expected deep crust rock (Slate/Gabbro/Marble), got {:?}",
        deep_block
    );

    // Middle crust at -70 should be Dolomite, Andesite, Limestone, Argillite, or Granite/Marble pocket
    let mid_block = strata.solid_voxel_at(0, -70, 0, 100, &plains, 12345);
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
    let taiga = BiomeType::Taiga.config();
    let taiga_crust = strata.solid_voxel_at(0, 50, 0, 6, &taiga, 12345);
    assert_eq!(
        taiga_crust,
        Voxel::Andesite,
        "Taiga permafrost surface crust must yield Andesite, got {:?}",
        taiga_crust
    );

    // 2. Plains (Loam soil -> Granite mother rock)
    let plains = BiomeType::Plains.config();
    let plains_crust = strata.solid_voxel_at(0, 50, 0, 6, &plains, 12345);
    assert_eq!(
        plains_crust,
        Voxel::Granite,
        "Plains loam surface crust must yield Granite, got {:?}",
        plains_crust
    );

    // 3. Savanna (Laterite soil -> Basalt mother rock)
    let savanna = BiomeType::Savanna.config();
    let savanna_crust = strata.solid_voxel_at(0, 50, 0, 6, &savanna, 12345);
    assert_eq!(
        savanna_crust,
        Voxel::Basalt,
        "Savanna laterite surface crust must yield Basalt, got {:?}",
        savanna_crust
    );

    // 4. Desert (Sand -> SandstoneCommon)
    let desert = BiomeType::Desert.config();
    let desert_crust = strata.solid_voxel_at(0, 50, 0, 6, &desert, 12345);
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


