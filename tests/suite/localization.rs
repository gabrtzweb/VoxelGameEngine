use voxel_game_engine::core::{Locale, Localization};
use voxel_game_engine::world::Voxel;

#[test]
fn test_locale_cycling() {
    assert_eq!(Locale::EnUs.next(), Locale::PtBr);
    assert_eq!(Locale::PtBr.next(), Locale::EnUs);
    assert_eq!(Locale::EnUs.prev(), Locale::PtBr);
    assert_eq!(Locale::PtBr.prev(), Locale::EnUs);
}

#[test]
fn test_locale_from_code() {
    assert_eq!(Locale::from_code("en_us"), Locale::EnUs);
    assert_eq!(Locale::from_code("pt_br"), Locale::PtBr);
    assert_eq!(Locale::from_code("pt-BR"), Locale::PtBr);
    assert_eq!(Locale::from_code("portugues"), Locale::PtBr);
    assert_eq!(Locale::from_code("unknown"), Locale::EnUs);
}

#[test]
fn test_format_translation() {
    let loc = Localization::new(Locale::EnUs);
    let formatted = loc.t_fmt("hud.coordinates", &["10", "64", "-25"]);
    assert_eq!(formatted, "Coordinates: XYZ: 10, 64, -25");
}

#[test]
fn test_fallback_and_block_translation() {
    let mut loc = Localization::new(Locale::PtBr);
    // Water translation in Portuguese
    let water_label = loc.t_block(Voxel::LiquidWater);
    assert_eq!(water_label, "Água");

    // Switch to English
    loc.set_locale(Locale::EnUs);
    let water_label_en = loc.t_block(Voxel::LiquidWater);
    assert_eq!(water_label_en, "Water");

    // Key fallback to key itself if not found
    assert_eq!(loc.t("non.existent.key"), "non.existent.key");
}
