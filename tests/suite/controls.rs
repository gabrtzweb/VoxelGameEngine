use bevy::prelude::KeyCode;
use voxel_game_engine::core::{ControlSettings, InputAction, InputBinding};

#[test]
fn test_default_bindings_roundtrip() {
    let controls = ControlSettings::default();
    for &action in InputAction::ALL {
        let binding = controls.get_binding(action);
        let s = binding.to_config_str();
        let parsed = InputBinding::from_config_str(&s).expect("Valid parse");
        assert_eq!(binding, parsed);
    }
}

#[test]
fn test_conflict_detection() {
    let mut controls = ControlSettings::default();
    assert!(!controls.has_conflict(InputAction::MoveForward));

    // Force conflict in Movement
    controls.set_binding(InputAction::MoveBackward, InputBinding::Key(KeyCode::KeyW));
    assert!(controls.has_conflict(InputAction::MoveForward));
    assert!(controls.has_conflict(InputAction::MoveBackward));
}
