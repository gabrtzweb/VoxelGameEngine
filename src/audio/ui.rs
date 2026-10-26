use bevy::{picking::hover::Hovered, prelude::*, ui::Pressed, ui_widgets::Button};
use crate::{
    audio::PlaySfxEvent,
    player::hotbar::Hotbar,
};

/// Types of user interface tactile acoustic feedback.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiSound {
    Click,
    MouseClick,
    MouseRelease,
    Rollover,
    Switch,
}

pub type PlayUiSoundEvent = UiSound;


#[derive(Resource)]
struct UiAudioCache {
    click: Handle<AudioSource>,
    mouseclick: Handle<AudioSource>,
    mouserelease: Handle<AudioSource>,
    rollover: Handle<AudioSource>,
    switch: Handle<AudioSource>,
    last_hotbar_slot: usize,
}

impl FromWorld for UiAudioCache {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            click: asset_server.load("sounds/ui/click1.ogg"),
            mouseclick: asset_server.load("sounds/ui/mouseclick1.ogg"),
            mouserelease: asset_server.load("sounds/ui/mouserelease1.ogg"),
            rollover: asset_server.load("sounds/ui/rollover1.ogg"),
            switch: asset_server.load("sounds/ui/switch1.ogg"),
            last_hotbar_slot: 0,
        }
    }
}

pub struct UiAudioPlugin;

impl Plugin for UiAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAudioCache>()
            .add_observer(handle_play_ui_sound)
            .add_systems(
                Update,
                (
                    detect_hotbar_switch_sound,
                    detect_button_click_sound,
                ),
            );
    }
}

fn handle_play_ui_sound(
    trigger: On<UiSound>,
    cache: Res<UiAudioCache>,
    mut commands: Commands,
) {
    let sound_type = *trigger.event();
    let (sound, volume) = match sound_type {
        UiSound::Click => (cache.click.clone(), 0.85),
        UiSound::MouseClick => (cache.mouseclick.clone(), 0.80),
        UiSound::MouseRelease => (cache.mouserelease.clone(), 0.75),
        UiSound::Rollover => (cache.rollover.clone(), 0.45),
        UiSound::Switch => (cache.switch.clone(), 0.70),
    };

    commands.trigger(PlaySfxEvent::new(sound, volume));
}

fn detect_hotbar_switch_sound(
    hotbar: Option<Res<Hotbar>>,
    mut cache: ResMut<UiAudioCache>,
    mut commands: Commands,
) {
    let Some(hotbar) = hotbar else {
        return;
    };

    if hotbar.is_changed() && hotbar.active_slot != cache.last_hotbar_slot {
        cache.last_hotbar_slot = hotbar.active_slot;
        commands.trigger(UiSound::Switch);
    }
}

fn detect_button_click_sound(
    mouse: Res<ButtonInput<MouseButton>>,
    buttons_query: Query<(&Hovered, Has<Pressed>), With<Button>>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    for (hovered, is_pressed) in &buttons_query {
        if hovered.get() || is_pressed {
            commands.trigger(UiSound::Click);
            break;
        }
    }
}

