pub mod inventory;
pub mod pause;
pub mod settings;

use bevy::{
    post_process::dof::{DepthOfField, DepthOfFieldMode},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    gameplay::BlockIcons,
    player::InspectorInteraction,
    world::Voxel,
};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MenuState {
    #[default]
    None,
    Pause,
    Settings,
    Inventory,
}

#[derive(Resource, Debug, Clone)]
pub struct GameSettings {
    pub fov_degrees: f32,
    pub fog_enabled: bool,
    pub view_bobbing: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fov_degrees: 90.0,
            fog_enabled: true,
            view_bobbing: true,
        }
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HeldInventoryItem {
    pub voxel: Option<Voxel>,
}

#[derive(Component)]
pub struct CustomCursor;

#[derive(Component)]
pub struct CustomCursorHeldIcon;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum CursorMode {
    #[default]
    Default,
    PointingHand,
    Grabbing,
    Crosshair,
    IBeam,
    NotAllowed,
    ResizeAll,
    ResizeEw,
    ResizeNs,
    ResizeNesw,
    ResizeNwse,
    Shift,
    Busy,
}

#[derive(Resource, Clone)]
#[allow(dead_code)]
pub struct CursorTextures {
    pub default: Handle<Image>,
    pub pointing_hand: Handle<Image>,
    pub grabbing: Handle<Image>,
    pub crosshair: Handle<Image>,
    pub ibeam: Handle<Image>,
    pub not_allowed: Handle<Image>,
    pub resize_all: Handle<Image>,
    pub resize_ew: Handle<Image>,
    pub resize_ns: Handle<Image>,
    pub resize_nesw: Handle<Image>,
    pub resize_nwse: Handle<Image>,
    pub shift: Handle<Image>,
    pub busy: Handle<Image>,
}

impl CursorTextures {
    pub fn get(&self, mode: CursorMode) -> Handle<Image> {
        match mode {
            CursorMode::Default => self.default.clone(),
            CursorMode::PointingHand => self.pointing_hand.clone(),
            CursorMode::Grabbing => self.grabbing.clone(),
            CursorMode::Crosshair => self.crosshair.clone(),
            CursorMode::IBeam => self.ibeam.clone(),
            CursorMode::NotAllowed => self.not_allowed.clone(),
            CursorMode::ResizeAll => self.resize_all.clone(),
            CursorMode::ResizeEw => self.resize_ew.clone(),
            CursorMode::ResizeNs => self.resize_ns.clone(),
            CursorMode::ResizeNesw => self.resize_nesw.clone(),
            CursorMode::ResizeNwse => self.resize_nwse.clone(),
            CursorMode::Shift => self.shift.clone(),
            CursorMode::Busy => self.busy.clone(),
        }
    }
}

#[derive(Resource)]
pub struct BusyCursorAnimation {
    pub timer: Timer,
    pub frame: usize,
}

impl Default for BusyCursorAnimation {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.08, TimerMode::Repeating),
            frame: 0,
        }
    }
}

#[derive(Resource)]
#[allow(dead_code)]
pub struct GuiTextures {
    pub cursor: Handle<Image>,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .init_resource::<GameSettings>()
            .init_resource::<HeldInventoryItem>()
            .init_resource::<CursorMode>()
            .init_resource::<BusyCursorAnimation>()
            .add_plugins((
                pause::PauseMenuPlugin,
                settings::SettingsMenuPlugin,
                inventory::InventoryMenuPlugin,
            ))
            .add_systems(Startup, setup_custom_cursor)
            .add_systems(
                Update,
                (
                    handle_menu_key_inputs,
                    manage_cursor_grab_mode,
                    manage_cursor_mode,
                    animate_busy_cursor,
                    update_custom_cursor,
                    sync_camera_fov,
                    manage_menu_blur,
                    manage_menu_time_pause,
                ),
            );
    }
}

fn setup_custom_cursor(mut commands: Commands, asset_server: Res<AssetServer>) {
    let textures = CursorTextures {
        default: asset_server.load("textures/gui/cursors/default.png"),
        pointing_hand: asset_server.load("textures/gui/cursors/pointing_hand.png"),
        grabbing: asset_server.load("textures/gui/cursors/grabbing.png"),
        crosshair: asset_server.load("textures/gui/cursors/crosshair.png"),
        ibeam: asset_server.load("textures/gui/cursors/ibeam.png"),
        not_allowed: asset_server.load("textures/gui/cursors/not_allowed.png"),
        resize_all: asset_server.load("textures/gui/cursors/resize_all.png"),
        resize_ew: asset_server.load("textures/gui/cursors/resize_ew.png"),
        resize_ns: asset_server.load("textures/gui/cursors/resize_ns.png"),
        resize_nesw: asset_server.load("textures/gui/cursors/resize_nesw.png"),
        resize_nwse: asset_server.load("textures/gui/cursors/resize_nwse.png"),
        shift: asset_server.load("textures/gui/cursors/shift.png"),
        busy: asset_server.load("textures/gui/cursors/busy.png"),
    };

    commands.insert_resource(GuiTextures {
        cursor: textures.default.clone(),
    });
    commands.insert_resource(textures.clone());

    commands
        .spawn((
            CustomCursor,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                width: px(64.0),
                height: px(64.0),
                ..default()
            },
            ImageNode {
                image: textures.default,
                ..default()
            },
            ZIndex(1000),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                CustomCursorHeldIcon,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(28.0),
                    top: px(28.0),
                    width: px(32.0),
                    height: px(32.0),
                    ..default()
                },
                ImageNode::default(),
                Visibility::Hidden,
            ));
        });
}

fn animate_busy_cursor(
    time: Res<Time>,
    cursor_mode: Res<CursorMode>,
    mut animation: ResMut<BusyCursorAnimation>,
) {
    if *cursor_mode == CursorMode::Busy {
        animation.timer.tick(time.delta());
        if animation.timer.just_finished() {
            animation.frame = (animation.frame + 1) % 13;
        }
    } else if animation.frame != 0 {
        animation.frame = 0;
    }
}

fn manage_cursor_mode(
    menu_state: Res<State<MenuState>>,
    held_item: Res<HeldInventoryItem>,
    keyboard: Res<ButtonInput<KeyCode>>,
    buttons_query: Query<&Interaction, With<Button>>,
    mut cursor_mode: ResMut<CursorMode>,
) {
    if *menu_state.get() == MenuState::None {
        if *cursor_mode != CursorMode::Default {
            *cursor_mode = CursorMode::Default;
        }
        return;
    }

    // Preserve external cursor mode requests (e.g. Busy, resize, not allowed)
    if matches!(
        *cursor_mode,
        CursorMode::Busy
            | CursorMode::ResizeAll
            | CursorMode::ResizeEw
            | CursorMode::ResizeNs
            | CursorMode::ResizeNesw
            | CursorMode::ResizeNwse
            | CursorMode::IBeam
            | CursorMode::Crosshair
            | CursorMode::NotAllowed
    ) {
        return;
    }

    let is_shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let is_hovering_button = buttons_query.iter().any(|interaction| {
        *interaction == Interaction::Hovered || *interaction == Interaction::Pressed
    });

    let next_mode = if held_item.voxel.is_some() {
        CursorMode::Grabbing
    } else if is_shift && *menu_state.get() == MenuState::Inventory {
        CursorMode::Shift
    } else if is_hovering_button {
        CursorMode::PointingHand
    } else {
        CursorMode::Default
    };

    if *cursor_mode != next_mode {
        *cursor_mode = next_mode;
    }
}

fn handle_menu_key_inputs(
    keyboard: Res<ButtonInput<KeyCode>>,
    menu_state: Res<State<MenuState>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut held_item: ResMut<HeldInventoryItem>,
) {
    let current = *menu_state.get();

    if keyboard.just_pressed(KeyCode::Escape) {
        match current {
            MenuState::None => next_state.set(MenuState::Pause),
            MenuState::Pause => next_state.set(MenuState::None),
            MenuState::Settings => next_state.set(MenuState::Pause),
            MenuState::Inventory => {
                held_item.voxel = None;
                next_state.set(MenuState::None);
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        match current {
            MenuState::None => next_state.set(MenuState::Inventory),
            MenuState::Inventory => {
                held_item.voxel = None;
                next_state.set(MenuState::None);
            }
            _ => {}
        }
    }
}

fn manage_cursor_grab_mode(
    menu_state: Res<State<MenuState>>,
    inspector: Res<InspectorInteraction>,
    cursor_options: Option<Single<&mut CursorOptions>>,
) {
    if !menu_state.is_changed() {
        return;
    }

    let Some(cursor_options) = cursor_options else {
        return;
    };
    let mut cursor_options = cursor_options.into_inner();

    if *menu_state.get() != MenuState::None {
        // Unlock mouse and hide OS cursor so custom cursor draws cleanly
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::None;
    } else if inspector.active {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    } else {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_custom_cursor(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    menu_state: Res<State<MenuState>>,
    held_item: Res<HeldInventoryItem>,
    block_icons: Option<Res<BlockIcons>>,
    cursor_query: Single<(&mut Node, &mut ImageNode, &mut Visibility), With<CustomCursor>>,
    held_icon_query: Single<
        (&mut ImageNode, &mut Visibility),
        (With<CustomCursorHeldIcon>, Without<CustomCursor>),
    >,
    cursor_mode: Res<CursorMode>,
    cursor_textures: Res<CursorTextures>,
    busy_animation: Res<BusyCursorAnimation>,
) {
    let (mut cursor_node, mut cursor_image, mut cursor_vis) = cursor_query.into_inner();
    let (mut held_icon, mut held_vis) = held_icon_query.into_inner();

    if *menu_state.get() == MenuState::None {
        *cursor_vis = Visibility::Hidden;
        *held_vis = Visibility::Hidden;
        return;
    }

    let Some(window) = window else {
        *cursor_vis = Visibility::Hidden;
        *held_vis = Visibility::Hidden;
        return;
    };

    if let Some(pos) = window.cursor_position() {
        *cursor_vis = Visibility::Visible;
        cursor_node.left = px(pos.x);
        cursor_node.top = px(pos.y);

        let current_mode = *cursor_mode;
        cursor_image.image = cursor_textures.get(current_mode);

        if current_mode == CursorMode::Busy {
            let frame_y = busy_animation.frame as f32 * 32.0;
            cursor_image.rect = Some(Rect::new(0.0, frame_y, 32.0, frame_y + 32.0));
        } else {
            cursor_image.rect = None;
        }

        if let (Some(voxel), Some(ref icons)) = (held_item.voxel, block_icons) {
            *held_vis = Visibility::Visible;
            held_icon.image = icons.get(voxel);
        } else {
            *held_vis = Visibility::Hidden;
        }
    } else {
        *cursor_vis = Visibility::Hidden;
        *held_vis = Visibility::Hidden;
    }
}

fn manage_menu_blur(
    mut commands: Commands,
    menu_state: Res<State<MenuState>>,
    camera_query: Query<Entity, With<Camera3d>>,
) {
    if !menu_state.is_changed() {
        return;
    }

    // World depth-of-field blur applies only to Pause and Settings menus, keeping the world
    // visually clear when interacting with the inventory.
    let is_in_pause_menu =
        *menu_state.get() == MenuState::Pause || *menu_state.get() == MenuState::Settings;

    for camera_entity in &camera_query {
        if is_in_pause_menu {
            commands.entity(camera_entity).insert(DepthOfField {
                mode: DepthOfFieldMode::Gaussian,
                focal_distance: 0.1,
                aperture_f_stops: 2.0,
                max_circle_of_confusion_diameter: 8.0,
                max_depth: 100.0,
                ..default()
            });
        } else {
            commands.entity(camera_entity).remove::<DepthOfField>();
        }
    }
}

fn manage_menu_time_pause(
    menu_state: Res<State<MenuState>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if !menu_state.is_changed() {
        return;
    }

    match *menu_state.get() {
        MenuState::Pause | MenuState::Settings => {
            virtual_time.pause();
        }
        MenuState::None | MenuState::Inventory => {
            virtual_time.unpause();
        }
    }
}

fn sync_camera_fov(
    settings: Res<GameSettings>,
    camera_projection: Option<Single<&mut Projection, With<Camera3d>>>,
) {
    if !settings.is_changed() {
        return;
    }

    let Some(camera_projection) = camera_projection else {
        return;
    };

    if let Projection::Perspective(ref mut perspective) = *camera_projection.into_inner() {
        perspective.fov = settings.fov_degrees.to_radians();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_settings_has_sensible_defaults() {
        let settings = GameSettings::default();
        assert_eq!(settings.fov_degrees, 90.0);
        assert!(settings.fog_enabled);
        assert!(settings.view_bobbing);
    }

    #[test]
    fn held_inventory_item_default_is_none() {
        let held = HeldInventoryItem::default();
        assert_eq!(held.voxel, None);
    }

    #[test]
    fn cursor_mode_default_is_default() {
        assert_eq!(CursorMode::default(), CursorMode::Default);
    }

    #[test]
    fn busy_cursor_animation_cycles_13_frames() {
        let mut anim = BusyCursorAnimation::default();
        assert_eq!(anim.frame, 0);

        for i in 1..=13 {
            anim.frame = (anim.frame + 1) % 13;
            assert_eq!(anim.frame, i % 13);
        }
        assert_eq!(anim.frame, 0);
    }
}
