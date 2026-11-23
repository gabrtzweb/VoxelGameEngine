use crate::{audio::PlaySfxEvent, menu::MenuState};
use bevy::{picking::hover::Hovered, prelude::*, ui::Pressed, ui_widgets::Button};

/// Types of user interface tactile acoustic feedback (menu button clicks only).
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiSound {
    Click,
}

pub type PlayUiSoundEvent = UiSound;

#[derive(Resource)]
struct UiAudioCache {
    click: Handle<AudioSource>,
}

impl FromWorld for UiAudioCache {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            click: asset_server.load("sounds/ui/click1.ogg"),
        }
    }
}

pub struct UiAudioPlugin;

impl Plugin for UiAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAudioCache>()
            .add_observer(handle_play_ui_sound)
            .add_systems(Update, detect_button_click_sound);
    }
}

fn handle_play_ui_sound(trigger: On<UiSound>, cache: Res<UiAudioCache>, mut commands: Commands) {
    let UiSound::Click = *trigger.event();
    commands.trigger(PlaySfxEvent::new(cache.click.clone(), 0.85));
}

fn detect_button_click_sound(
    mouse: Res<ButtonInput<MouseButton>>,
    menu_state: Option<Res<State<MenuState>>>,
    buttons_query: Query<(&Hovered, Has<Pressed>), With<Button>>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Never play UI click sounds during gameplay (hotbar) or inside the Inventory / Creative menu
    if menu_state.is_some_and(|s| matches!(*s.get(), MenuState::None | MenuState::Inventory)) {
        return;
    }

    for (hovered, is_pressed) in &buttons_query {
        if hovered.get() || is_pressed {
            commands.trigger(UiSound::Click);
            break;
        }
    }
}
