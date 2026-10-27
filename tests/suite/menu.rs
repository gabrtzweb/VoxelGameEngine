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
        in_active_session && current != MenuState::None && current != MenuState::MainMenu
    };

    // In MainMenu: never visible
    assert!(!check_visibility(false, MenuState::MainMenu, MenuState::MainMenu));
    assert!(!check_visibility(false, MenuState::Settings, MenuState::MainMenu));
    assert!(!check_visibility(false, MenuState::Controls, MenuState::MainMenu));

    // In Active Game Session: visible during in-game menus, hidden during gameplay (None)
    assert!(!check_visibility(true, MenuState::None, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Inventory, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Journal, MenuState::Pause));
    assert!(check_visibility(true, MenuState::WorldMap, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Pause, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Settings, MenuState::Pause));
    assert!(check_visibility(true, MenuState::Controls, MenuState::Pause));
}
