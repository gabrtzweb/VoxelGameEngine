pub mod inventory;
pub mod pause;
pub mod settings;

use bevy::{
    post_process::dof::{DepthOfField, DepthOfFieldMode},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    player::InspectorInteraction,
    voxel::{chunk::Voxel, icon::BlockIcons},
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
                    update_custom_cursor,
                    sync_camera_fov,
                    manage_menu_blur,
                    manage_menu_time_pause,
                ),
            );
    }
}

fn setup_custom_cursor(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cursor_texture = asset_server.load("textures/gui/cursor_default.png");
    commands.insert_resource(GuiTextures {
        cursor: cursor_texture.clone(),
    });

    commands
        .spawn((
            CustomCursor,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                width: px(32.0),
                height: px(32.0),
                ..default()
            },
            ImageNode {
                image: cursor_texture,
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
                    left: px(14.0),
                    top: px(14.0),
                    width: px(24.0),
                    height: px(24.0),
                    ..default()
                },
                ImageNode::default(),
                Visibility::Hidden,
            ));
        });
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

#[allow(clippy::type_complexity)]
fn update_custom_cursor(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    menu_state: Res<State<MenuState>>,
    held_item: Res<HeldInventoryItem>,
    block_icons: Option<Res<BlockIcons>>,
    cursor_query: Single<(&mut Node, &mut Visibility), With<CustomCursor>>,
    held_icon_query: Single<
        (&mut ImageNode, &mut Visibility),
        (With<CustomCursorHeldIcon>, Without<CustomCursor>),
    >,
) {
    let (mut cursor_node, mut cursor_vis) = cursor_query.into_inner();
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
}
