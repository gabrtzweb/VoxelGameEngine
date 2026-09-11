use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};

use crate::voxel::{chunk::Voxel, interaction::SelectedVoxel};

pub const HOTBAR_SLOT_COUNT: usize = 8;

#[derive(Resource, Debug, Clone)]
pub struct Hotbar {
    pub slots: [Option<Voxel>; HOTBAR_SLOT_COUNT],
    pub active_slot: usize,
}

impl Default for Hotbar {
    fn default() -> Self {
        Self {
            slots: [
                Some(Voxel::Grass),
                Some(Voxel::Dirt),
                Some(Voxel::Stone),
                Some(Voxel::Sand),
                Some(Voxel::Water),
                Some(Voxel::Light),
                None,
                None,
            ],
            active_slot: 0,
        }
    }
}

#[derive(Component)]
pub struct HotbarSlotUi {
    pub index: usize,
}

#[derive(Component)]
pub struct HotbarSlotIcon {
    pub index: usize,
}

#[derive(Resource)]
pub struct HotbarTextures {
    pub grass: Handle<Image>,
    pub dirt: Handle<Image>,
    pub stone: Handle<Image>,
    pub sand: Handle<Image>,
    pub water: Handle<Image>,
    pub light: Handle<Image>,
}

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hotbar>()
            .add_systems(Startup, setup_hotbar_ui)
            .add_systems(Update, (handle_hotbar_input, sync_hotbar_ui).chain());
    }
}

fn setup_hotbar_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let textures = HotbarTextures {
        grass: asset_server.load("textures/items/item_grass.png"),
        dirt: asset_server.load("textures/items/item_dirt.png"),
        stone: asset_server.load("textures/items/item_stone.png"),
        sand: asset_server.load("textures/items/item_sand.png"),
        water: asset_server.load("textures/items/item_water.png"),
        light: asset_server.load("textures/items/item_light.png"),
    };

    let initial_hotbar = Hotbar::default();

    // Centered bottom container spanning screen width
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(16.0),
                left: px(0.0),
                right: px(0.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(150),
        ))
        .with_children(|parent| {
            // Hotbar tray with semi-transparent dark backing and subtle border
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        column_gap: px(6.0),
                        padding: UiRect::all(px(6.0)),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.06, 0.08, 0.85)),
                    BorderColor::all(Color::srgba(0.25, 0.25, 0.28, 0.85)),
                ))
                .with_children(|tray| {
                    for index in 0..HOTBAR_SLOT_COUNT {
                        let is_active = index == initial_hotbar.active_slot;
                        let initial_voxel = initial_hotbar.slots[index];

                        let border_color = if is_active {
                            Color::srgb(1.0, 0.85, 0.30)
                        } else {
                            Color::srgba(0.25, 0.25, 0.28, 0.60)
                        };

                        let bg_color = if is_active {
                            Color::srgba(0.24, 0.24, 0.28, 0.95)
                        } else {
                            Color::srgba(0.12, 0.12, 0.14, 0.75)
                        };

                        let icon_handle = match initial_voxel {
                            Some(Voxel::Grass) => textures.grass.clone(),
                            Some(Voxel::Dirt) => textures.dirt.clone(),
                            Some(Voxel::Stone) => textures.stone.clone(),
                            Some(Voxel::Sand) => textures.sand.clone(),
                            Some(Voxel::Water) => textures.water.clone(),
                            Some(Voxel::Light) => textures.light.clone(),
                            _ => textures.stone.clone(),
                        };

                        let icon_visibility = if initial_voxel.is_some() {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        };

                        tray.spawn((
                            HotbarSlotUi { index },
                            Node {
                                width: px(48.0),
                                height: px(48.0),
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(2.0)),
                                border_radius: BorderRadius::all(px(6.0)),
                                position_type: PositionType::Relative,
                                ..default()
                            },
                            BackgroundColor(bg_color),
                            BorderColor::all(border_color),
                        ))
                        .with_children(|slot| {
                            // Slot index number (1 through 8)
                            slot.spawn((
                                Text::new(format!("{}", index + 1)),
                                TextFont {
                                    font_size: FontSize::Px(11.0),
                                    ..default()
                                },
                                TextColor(Color::srgba(0.85, 0.85, 0.85, 0.70)),
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: px(2.0),
                                    left: px(4.0),
                                    ..default()
                                },
                            ));

                            // Centered 32x32 2D item icon
                            slot.spawn((
                                HotbarSlotIcon { index },
                                ImageNode {
                                    image: icon_handle,
                                    ..default()
                                },
                                Node {
                                    width: px(32.0),
                                    height: px(32.0),
                                    ..default()
                                },
                                icon_visibility,
                            ));
                        });
                    }
                });
        });

    commands.insert_resource(textures);
}

fn handle_hotbar_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut hotbar: ResMut<Hotbar>,
    mut selected: ResMut<SelectedVoxel>,
) {
    let digit_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ];

    for (idx, key) in digit_keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            hotbar.active_slot = idx;
        }
    }

    let scroll = mouse_scroll.delta.y;
    if scroll > 0.05 {
        // Scroll up = previous slot
        hotbar.active_slot = (hotbar.active_slot + HOTBAR_SLOT_COUNT - 1) % HOTBAR_SLOT_COUNT;
    } else if scroll < -0.05 {
        // Scroll down = next slot
        hotbar.active_slot = (hotbar.active_slot + 1) % HOTBAR_SLOT_COUNT;
    }

    // Q key clears the active hotbar slot (giving the player an empty hand)
    if keyboard.just_pressed(KeyCode::KeyQ) {
        let active = hotbar.active_slot;
        hotbar.slots[active] = None;
    }

    let active = hotbar.active_slot;
    selected.0 = hotbar.slots[active];
}

fn sync_hotbar_ui(
    hotbar: Res<Hotbar>,
    icons: Res<HotbarTextures>,
    mut slot_query: Query<(&HotbarSlotUi, &mut BorderColor, &mut BackgroundColor)>,
    mut icon_query: Query<(&HotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    if !hotbar.is_changed() {
        return;
    }

    for (slot, mut border_color, mut bg_color) in &mut slot_query {
        if slot.index == hotbar.active_slot {
            *border_color = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            bg_color.0 = Color::srgba(0.24, 0.24, 0.28, 0.95);
        } else {
            *border_color = BorderColor::all(Color::srgba(0.25, 0.25, 0.28, 0.60));
            bg_color.0 = Color::srgba(0.12, 0.12, 0.14, 0.75);
        }
    }

    for (icon, mut img_node, mut visibility) in &mut icon_query {
        match hotbar.slots[icon.index] {
            Some(voxel) => {
                let handle = match voxel {
                    Voxel::Grass => icons.grass.clone(),
                    Voxel::Dirt => icons.dirt.clone(),
                    Voxel::Stone => icons.stone.clone(),
                    Voxel::Sand => icons.sand.clone(),
                    Voxel::Water => icons.water.clone(),
                    Voxel::Light => icons.light.clone(),
                    _ => icons.stone.clone(),
                };
                img_node.image = handle;
                *visibility = Visibility::Visible;
            }
            None => {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotbar_slot_cycling_wraps_cleanly() {
        let mut active = 0;
        // Scroll up from 0 -> wraps to 7
        active = (active + HOTBAR_SLOT_COUNT - 1) % HOTBAR_SLOT_COUNT;
        assert_eq!(active, 7);

        // Scroll down from 7 -> wraps to 0
        active = (active + 1) % HOTBAR_SLOT_COUNT;
        assert_eq!(active, 0);
    }

    #[test]
    fn hotbar_default_has_eight_slots() {
        let hotbar = Hotbar::default();
        assert_eq!(hotbar.slots.len(), 8);
        assert_eq!(hotbar.slots[0], Some(Voxel::Grass));
        assert_eq!(hotbar.slots[5], Some(Voxel::Light));
        assert_eq!(hotbar.slots[6], None);
        assert_eq!(hotbar.slots[7], None);
    }

    #[test]
    fn clearing_slot_sets_it_to_none() {
        let mut hotbar = Hotbar::default();
        assert_eq!(hotbar.slots[hotbar.active_slot], Some(Voxel::Grass));
        // Simulate pressing Q
        hotbar.slots[hotbar.active_slot] = None;
        assert_eq!(hotbar.slots[hotbar.active_slot], None);
    }
}
