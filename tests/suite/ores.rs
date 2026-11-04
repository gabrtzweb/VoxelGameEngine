use voxel_game_engine::{
    generation::{
        biome::BiomeType,
        ores::{self, CrystalGeodeType, MineralType},
        strata::StrataGenerator,
    },
    world::Voxel,
};

#[test]
fn test_all_ore_vein_mappings() {
    // 1. Coal (6 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::SandstoneCommon), Some(Voxel::OreCoalRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Granite), Some(Voxel::OreCoalRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Dolomite), Some(Voxel::OreCoalRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Limestone), Some(Voxel::OreCoalRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Chalk), Some(Voxel::OreCoalRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Coal, Voxel::Argillite), Some(Voxel::OreCoalRaw));

    // 2. Copper (6 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::SandstoneCommon), Some(Voxel::OreCopperRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::SandstoneRed), Some(Voxel::OreCopperRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::SandstoneWhite), Some(Voxel::OreCopperRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::Granite), Some(Voxel::OreCopperRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::Andesite), Some(Voxel::OreCopperRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Copper, Voxel::Dolomite), Some(Voxel::OreCopperRaw));

    // 3. Iron (6 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Granite), Some(Voxel::OreIronRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Andesite), Some(Voxel::OreIronRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Dolomite), Some(Voxel::OreIronRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Slate), Some(Voxel::OreIronRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Gabbro), Some(Voxel::OreIronRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Iron, Voxel::Basalt), Some(Voxel::OreIronRaw));

    // 4. Gold (4 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Gold, Voxel::SandstoneRed), Some(Voxel::OreGoldRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Gold, Voxel::Granite), Some(Voxel::OreGoldRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Gold, Voxel::Slate), Some(Voxel::OreGoldRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Gold, Voxel::Gabbro), Some(Voxel::OreGoldRaw));

    // 5. Silver (3 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Silver, Voxel::Andesite), Some(Voxel::OreSilverRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Silver, Voxel::Slate), Some(Voxel::OreSilverRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Silver, Voxel::Marble), Some(Voxel::OreSilverRaw));

    // 6. Uranium (2 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Uranium, Voxel::Slate), Some(Voxel::OreUraniumRaw));
    assert_eq!(ores::host_to_ore_vein(MineralType::Uranium, Voxel::Gabbro), Some(Voxel::OreUraniumRaw));

    // 7. Diamond (2 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Diamond, Voxel::Gabbro), Some(Voxel::GemDiamondRough));
    assert_eq!(ores::host_to_ore_vein(MineralType::Diamond, Voxel::Basalt), Some(Voxel::GemDiamondRough));

    // 8. Emerald (2 host rocks)
    assert_eq!(ores::host_to_ore_vein(MineralType::Emerald, Voxel::Slate), Some(Voxel::GemEmeraldRough));
    assert_eq!(ores::host_to_ore_vein(MineralType::Emerald, Voxel::Marble), Some(Voxel::GemEmeraldRough));

    // 9. Sulfur (1 host rock)
    assert_eq!(ores::host_to_ore_vein(MineralType::Sulfur, Voxel::Basalt), Some(Voxel::OreSulfurRaw));

    // 10. Gem Vein (7 host rocks -> Rough Gems)
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Dolomite), Some(Voxel::GemTopazRough));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Granite), Some(Voxel::GemTopazRough));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Andesite), Some(Voxel::GemJadeRough));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Slate), Some(Voxel::GemJadeRough));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Gabbro), Some(Voxel::GemSapphireRough));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Basalt), Some(Voxel::GemSapphireRough));
    assert_eq!(ores::host_to_ore_vein(MineralType::GemVein, Voxel::Marble), Some(Voxel::GemRubyRough));
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


