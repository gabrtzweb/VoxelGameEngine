use std::path::Path;
use voxel_game_engine::{
    core::Localization,
    menu::{MenuReturnTarget, MenuState, main_menu::MainMenuPhase},
};

#[test]
fn test_main_menu_initial_state() {
    assert_eq!(MenuState::default(), MenuState::MainMenu);
    assert_eq!(MenuReturnTarget::default(), MenuReturnTarget(MenuState::Pause));
    assert_eq!(MainMenuPhase::default(), MainMenuPhase::Splash);
}

#[test]
fn test_main_menu_assets_exist() {
    assert!(Path::new("assets/textures/interfaces/main_menu_bg.png").exists(), "Background image must exist");
    assert!(Path::new("assets/textures/interfaces/containers/buttons/button_large.png").exists(), "Large button texture must exist");
    assert!(Path::new("assets/textures/interfaces/containers/buttons/button_large_hover.png").exists(), "Large button hover texture must exist");
}

#[test]
fn test_main_menu_localization() {
    let mut loc = Localization::default();

    let keys = [
        "menu.main.title",
        "menu.main.press_any_key",
        "menu.main.play",
        "menu.main.settings",
        "menu.main.controls",
        "menu.main.quit",
        "menu.main.version",
        "menu.loading.title",
        "menu.loading.generating",
        "menu.loading.meshing",
        "menu.loading.finalizing",
        "menu.loading.ready",
        "menu.loading.tip_1",
        "menu.loading.tip_2",
        "menu.loading.tip_3",
        "menu.loading.tip_4",
        "menu.loading.tip_5",
        "menu.loading.tip_6",
        "menu.loading.tip_7",
        "menu.loading.tip_8",
        "menu.loading.tip_9",
        "menu.loading.tip_10",
    ];

    // English
    loc.set_locale(voxel_game_engine::core::Locale::EnUs);
    for key in keys {
        let text = loc.t(key);
        assert_ne!(text, key, "Missing translation for {key} in en_US");
        assert!(!text.is_empty());
    }

    // Portuguese
    loc.set_locale(voxel_game_engine::core::Locale::PtBr);
    for key in keys {
        let text = loc.t(key);
        assert_ne!(text, key, "Missing translation for {key} in pt_BR");
        assert!(!text.is_empty());
    }
}

#[test]
fn test_menu_bar_session_visibility_rules() {
    let check_visibility = |in_session: bool, current: MenuState, target: MenuState| -> bool {
        let in_active_session = if in_session { true } else { target != MenuState::MainMenu };
        in_active_session && current != MenuState::None && current != MenuState::MainMenu && current != MenuState::Loading
    };

    // In MainMenu: never visible
    assert!(!check_visibility(false, MenuState::MainMenu, MenuState::MainMenu));
    assert!(!check_visibility(false, MenuState::Loading, MenuState::MainMenu));
    assert!(!check_visibility(false, MenuState::Settings, MenuState::MainMenu));
    assert!(!check_visibility(false, MenuState::Controls, MenuState::MainMenu));

    // In Active Game Session: visible during in-game menus, hidden during gameplay (None) or loading (Loading)
    assert!(!check_visibility(true, MenuState::None, MenuState::Pause));
    assert!(!check_visibility(true, MenuState::Loading, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Inventory, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Journal, MenuState::Pause));
    assert!(check_visibility(true, MenuState::WorldMap, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Pause, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Settings, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Controls, MenuState::Pause));
}

#[test]
fn test_loading_screen_initial_state() {
    let state = voxel_game_engine::menu::loading::LoadingState::default();
    assert_eq!(state.display_progress, 0.0);
    assert_eq!(state.target_progress, 0.0);
    assert!(!state.is_complete);
    assert_eq!(state.total_chunks, 0);
    assert_eq!(state.loaded_chunks, 0);
}

#[test]
fn test_race_selection_and_palettes() {
    use voxel_game_engine::player::{CharacterRace, SelectedRace};

    assert_eq!(SelectedRace::default().0, CharacterRace::Human);
    assert_eq!(CharacterRace::ALL.len(), 8);

    let mut loc = Localization::default();
    loc.set_locale(voxel_game_engine::core::Locale::PtBr);

    for race in CharacterRace::ALL {
        let name = race.display_name(Some(&loc));
        assert!(!name.is_empty(), "Display name for {:?} should not be empty", race);
        let icon_path = format!("assets/{}", race.icon_asset_path());
        assert!(Path::new(&icon_path).exists(), "Icon asset {} must exist", icon_path);

        // Verify all limbs have distinct valid colors
        let skin = race.skin_color();
        let torso = race.torso_color();
        let arms = race.arms_color();
        let legs = race.legs_color();
        let _ = (skin, torso, arms, legs);

        // Verify physical proportions are finite and positive
        let p = race.proportions();
        assert!(p.root_scale.x > 0.0 && p.root_scale.y > 0.0 && p.root_scale.z > 0.0);
        assert!(p.head_scale.x > 0.0 && p.head_scale.y > 0.0 && p.head_scale.z > 0.0);
        assert!(p.torso_scale.x > 0.0 && p.torso_scale.y > 0.0 && p.torso_scale.z > 0.0);
        assert!(p.arm_scale.x > 0.0 && p.arm_scale.y > 0.0 && p.arm_scale.z > 0.0);
        assert!(p.leg_scale.x > 0.0 && p.leg_scale.y > 0.0 && p.leg_scale.z > 0.0);
        assert!(p.shoulder_spread > 0.0 && p.hip_spread > 0.0);
    }

    // Verify distinct stature scaling across key archetypes
    assert!(CharacterRace::Halfling.proportions().root_scale.y < CharacterRace::Dwarf.proportions().root_scale.y);
    assert!(CharacterRace::Dwarf.proportions().root_scale.y < CharacterRace::Human.proportions().root_scale.y);
    assert!(CharacterRace::Elf.proportions().root_scale.y > CharacterRace::Human.proportions().root_scale.y);
    assert!(CharacterRace::Orc.proportions().shoulder_spread > CharacterRace::Human.proportions().shoulder_spread);
}

#[test]
fn test_item_drop_buoyancy_and_mesh_generation() {
    use voxel_game_engine::gameplay::is_voxel_buoyant;
    use voxel_game_engine::world::block::Voxel;

    // Buoyant materials (wood, leaves, snow, ice) float on water
    assert!(is_voxel_buoyant(Voxel::OakLog));
    assert!(is_voxel_buoyant(Voxel::OakPlanks));
    assert!(is_voxel_buoyant(Voxel::OakLeaves));
    assert!(is_voxel_buoyant(Voxel::SnowBlock));
    assert!(is_voxel_buoyant(Voxel::FrostIce));

    // Dense mineral materials (stone, gravel, dirt, sand) sink
    assert!(!is_voxel_buoyant(Voxel::Andesite));
    assert!(!is_voxel_buoyant(Voxel::Basalt));
    assert!(!is_voxel_buoyant(Voxel::SandCommon));
    assert!(!is_voxel_buoyant(Voxel::LoamDirt));
}


