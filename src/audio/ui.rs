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
    Pickup,
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
    pickup_rng: u64,
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
            pickup_rng: 0x9E37_79B9_7F4A_7C15,
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
    mut cache: ResMut<UiAudioCache>,
    mut commands: Commands,
) {
    let sound_type = *trigger.event();
    if sound_type == UiSound::Pickup {
        cache.pickup_rng = cache
            .pickup_rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let rand = ((cache.pickup_rng >> 32) as u32 as f32) / 4294967295.0;
        let speed = 1.65 + rand * 0.35;
        commands.trigger(PlaySfxEvent::new(cache.rollover.clone(), 0.38).with_speed(speed));
        return;
    }

    let (sound, volume) = match sound_type {
        UiSound::Click => (cache.click.clone(), 0.85),
        UiSound::MouseClick => (cache.mouseclick.clone(), 0.80),
        UiSound::MouseRelease => (cache.mouserelease.clone(), 0.75),
        UiSound::Rollover => (cache.rollover.clone(), 0.45),
        UiSound::Switch | UiSound::Pickup => (cache.switch.clone(), 0.70),
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

