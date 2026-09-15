use bevy::prelude::*;

use crate::{
    player::hotbar::{HOTBAR_SLOT_COUNT, Hotbar, HotbarTextures},
    voxel::chunk::Voxel,
};

use super::{HeldInventoryItem, MenuState};

pub const INVENTORY_TOTAL_SLOTS: usize = 32; // 8 wide x 4 high
pub const INVENTORY_COLS: usize = 8;
pub const INVENTORY_ROWS: usize = 4;

pub const AVAILABLE_BLOCKS: [Voxel; 6] = [
    Voxel::Grass,
    Voxel::Dirt,
    Voxel::Stone,
    Voxel::Sand,
    Voxel::Water,
    Voxel::Light,
];

#[derive(Component)]
struct InventoryMenuRoot;

#[derive(Component)]
struct InventoryPaletteSlot {
    _index: usize,
    voxel: Option<Voxel>,
}

#[derive(Component)]
struct InventoryHotbarSlot {
    index: usize,
}

#[derive(Component)]
struct InventoryHotbarSlotIcon {
    index: usize,
}

#[derive(Component)]
struct InventoryItemTooltip;

pub struct InventoryMenuPlugin;

impl Plugin for InventoryMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Inventory), spawn_inventory_menu)
            .add_systems(OnExit(MenuState::Inventory), despawn_inventory_menu)
            .add_systems(
                Update,
                (handle_inventory_interaction, sync_inventory_hotbar_icons)
                    .run_if(in_state(MenuState::Inventory)),
            );
    }
}

fn spawn_inventory_menu(
    mut commands: Commands,
    hotbar: Res<Hotbar>,
    textures: Res<HotbarTextures>,
) {
    commands
        .spawn((
            InventoryMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(12.0),
                        width: px(450.0),
                        padding: UiRect::all(px(20.0)),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.11, 0.96)),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.42, 0.80)),
                ))
                .with_children(|panel| {
                    // Header Title
                    panel.spawn((
                        Text::new("CREATIVE INVENTORY"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.95, 0.98)),
                        Node {
                            margin: UiRect::bottom(px(2.0)),
                            ..default()
                        },
                    ));

                    // Instructions / Tooltip text
                    panel.spawn((
                        InventoryItemTooltip,
                        Text::new(
                            "Left-click to hold • Keys 1–8 to assign • Middle-click clears slot",
                        ),
                        TextFont {
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.75, 0.78, 0.85)),
                        Node {
                            margin: UiRect::bottom(px(6.0)),
                            ..default()
                        },
                    ));

                    // 8x4 Grid Container
                    panel
                        .spawn((
                            Node {
                                display: Display::Grid,
                                grid_template_columns: RepeatedGridTrack::px(
                                    INVENTORY_COLS as u16,
                                    44.0,
                                ),
                                grid_template_rows: RepeatedGridTrack::px(
                                    INVENTORY_ROWS as u16,
                                    44.0,
                                ),
                                row_gap: px(6.0),
                                column_gap: px(6.0),
                                padding: UiRect::all(px(8.0)),
                                border: UiRect::all(px(1.5)),
                                border_radius: BorderRadius::all(px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.85)),
                            BorderColor::all(Color::srgba(0.22, 0.22, 0.26, 0.70)),
                        ))
                        .with_children(|grid| {
                            for slot_idx in 0..INVENTORY_TOTAL_SLOTS {
                                let voxel = AVAILABLE_BLOCKS.get(slot_idx).copied();

                                let icon_handle = match voxel {
                                    Some(Voxel::Grass) => Some(textures.grass.clone()),
                                    Some(Voxel::Dirt) => Some(textures.dirt.clone()),
                                    Some(Voxel::Stone) => Some(textures.stone.clone()),
                                    Some(Voxel::Sand) => Some(textures.sand.clone()),
                                    Some(Voxel::Water) => Some(textures.water.clone()),
                                    Some(Voxel::Light) => Some(textures.light.clone()),
                                    _ => None,
                                };

                                grid.spawn((
                                    Button,
                                    InventoryPaletteSlot {
                                        _index: slot_idx,
                                        voxel,
                                    },
                                    Node {
                                        width: px(44.0),
                                        height: px(44.0),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.85)),
                                    BorderColor::all(Color::srgba(0.25, 0.25, 0.30, 0.60)),
                                ))
                                .with_children(|slot| {
                                    if let Some(handle) = icon_handle {
                                        slot.spawn((
                                            ImageNode {
                                                image: handle,
                                                ..default()
                                            },
                                            Node {
                                                width: px(32.0),
                                                height: px(32.0),
                                                ..default()
                                            },
                                        ));
                                    }
                                });
                            }
                        });

                    // Divider
                    panel.spawn(Node {
                        width: px(390.0),
                        height: px(1.5),
                        margin: UiRect::axes(px(0.0), px(4.0)),
                        border: UiRect::all(px(1.0)),
                        ..default()
                    });

                    // Hotbar Label
                    panel.spawn((
                        Text::new("HOTBAR"),
                        TextFont {
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.70, 0.72, 0.78)),
                        Node {
                            margin: UiRect::bottom(px(2.0)),
                            ..default()
                        },
                    ));

                    // 8-slot Hotbar mirror row
                    panel
                        .spawn((
                            Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                column_gap: px(6.0),
                                padding: UiRect::all(px(6.0)),
                                border: UiRect::all(px(1.5)),
                                border_radius: BorderRadius::all(px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.85)),
                            BorderColor::all(Color::srgba(0.22, 0.22, 0.26, 0.70)),
                        ))
                        .with_children(|tray| {
                            for slot_idx in 0..HOTBAR_SLOT_COUNT {
                                let initial_voxel = hotbar.slots[slot_idx];
                                let icon_handle = match initial_voxel {
                                    Some(Voxel::Grass) => textures.grass.clone(),
                                    Some(Voxel::Dirt) => textures.dirt.clone(),
                                    Some(Voxel::Stone) => textures.stone.clone(),
                                    Some(Voxel::Sand) => textures.sand.clone(),
                                    Some(Voxel::Water) => textures.water.clone(),
                                    Some(Voxel::Light) => textures.light.clone(),
                                    _ => textures.stone.clone(),
                                };

                                let icon_vis = if initial_voxel.is_some() {
                                    Visibility::Visible
                                } else {
                                    Visibility::Hidden
                                };

                                tray.spawn((
                                    Button,
                                    InventoryHotbarSlot { index: slot_idx },
                                    Node {
                                        width: px(44.0),
                                        height: px(44.0),
                                        display: Display::Flex,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        position_type: PositionType::Relative,
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.85)),
                                    BorderColor::all(Color::srgba(0.25, 0.25, 0.30, 0.60)),
                                ))
                                .with_children(|slot| {
                                    // Number badge 1..8
                                    slot.spawn((
                                        Text::new(format!("{}", slot_idx + 1)),
                                        TextFont {
                                            font_size: FontSize::Px(10.0),
                                            ..default()
                                        },
                                        TextColor(Color::srgba(0.85, 0.85, 0.85, 0.60)),
                                        Node {
                                            position_type: PositionType::Absolute,
                                            top: px(2.0),
                                            left: px(3.0),
                                            ..default()
                                        },
                                    ));

                                    // Item icon
                                    slot.spawn((
                                        InventoryHotbarSlotIcon { index: slot_idx },
                                        ImageNode {
                                            image: icon_handle,
                                            ..default()
                                        },
                                        Node {
                                            width: px(30.0),
                                            height: px(30.0),
                                            ..default()
                                        },
                                        icon_vis,
                                    ));
                                });
                            }
                        });
                });
        });
}

fn despawn_inventory_menu(
    mut commands: Commands,
    query: Query<Entity, With<InventoryMenuRoot>>,
    mut held_item: ResMut<HeldInventoryItem>,
) {
    held_item.voxel = None;
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_inventory_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut held_item: ResMut<HeldInventoryItem>,
    mut hotbar: ResMut<Hotbar>,
    mut palette_query: Query<
        (
            &Interaction,
            &InventoryPaletteSlot,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        With<Button>,
    >,
    mut hotbar_slot_query: Query<
        (
            &Interaction,
            &InventoryHotbarSlot,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        (With<Button>, Without<InventoryPaletteSlot>),
    >,
    mut tooltip_query: Query<&mut Text, With<InventoryItemTooltip>>,
) {
    let mut hovered_item_name: Option<&'static str> = None;

    // Right-click clears currently held item
    if mouse.just_pressed(MouseButton::Right) {
        held_item.voxel = None;
    }

    // Check digit keys 1..8 for quick assignment
    let digit_pressed = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some(3)
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        Some(4)
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        Some(5)
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        Some(6)
    } else if keyboard.just_pressed(KeyCode::Digit8) {
        Some(7)
    } else {
        None
    };

    // 1. Process Palette Slots
    for (interaction, slot, mut border, mut bg) in &mut palette_query {
        let is_hovered = *interaction == Interaction::Hovered;
        let is_pressed = *interaction == Interaction::Pressed;

        if is_hovered || is_pressed {
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            *bg = BackgroundColor(Color::srgba(0.20, 0.20, 0.25, 0.95));

            if let Some(v) = slot.voxel {
                hovered_item_name = Some(v.label());

                // Quick assign via keys 1..8
                if let Some(target_slot) = digit_pressed {
                    hotbar.slots[target_slot] = Some(v);
                }

                // Left-click picks up item onto cursor
                if is_pressed {
                    held_item.voxel = Some(v);
                }
            }
        } else {
            *border = BorderColor::all(Color::srgba(0.25, 0.25, 0.30, 0.60));
            *bg = BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.85));
        }
    }

    // 2. Process Hotbar Slots
    for (interaction, hotbar_slot, mut border, mut bg) in &mut hotbar_slot_query {
        let is_hovered = *interaction == Interaction::Hovered;
        let is_pressed = *interaction == Interaction::Pressed;
        let idx = hotbar_slot.index;

        if is_hovered || is_pressed {
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            *bg = BackgroundColor(Color::srgba(0.20, 0.20, 0.25, 0.95));

            if let Some(v) = hotbar.slots[idx]
                && hovered_item_name.is_none()
            {
                hovered_item_name = Some(v.label());
            }

            // Middle click clears hotbar slot
            if mouse.just_pressed(MouseButton::Middle) {
                hotbar.slots[idx] = None;
            }

            // Left click transfers or swaps
            if is_pressed {
                if let Some(held) = held_item.voxel {
                    // Place held item into slot
                    hotbar.slots[idx] = Some(held);
                } else if hotbar.slots[idx].is_some() {
                    // Pick up slot item into hand
                    held_item.voxel = hotbar.slots[idx];
                    hotbar.slots[idx] = None;
                }
            }
        } else {
            *border = BorderColor::all(Color::srgba(0.25, 0.25, 0.30, 0.60));
            *bg = BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.85));
        }
    }

    // 3. Update tooltip text
    for mut tooltip in &mut tooltip_query {
        if let Some(name) = hovered_item_name {
            tooltip.0 = format!("Block: {name} (Keys 1–8 to quick-assign)");
        } else if let Some(held) = held_item.voxel {
            tooltip.0 = format!("Holding: {} (Click hotbar slot to place)", held.label());
        } else {
            tooltip.0 =
                "Left-click to hold • Keys 1–8 to assign • Middle-click clears slot".to_string();
        }
    }
}

fn sync_inventory_hotbar_icons(
    hotbar: Res<Hotbar>,
    textures: Res<HotbarTextures>,
    mut icons_query: Query<(&InventoryHotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    if !hotbar.is_changed() {
        return;
    }

    for (icon, mut img, mut vis) in &mut icons_query {
        let slot_voxel = hotbar.slots[icon.index];
        if let Some(voxel) = slot_voxel {
            *vis = Visibility::Visible;
            img.image = match voxel {
                Voxel::Grass => textures.grass.clone(),
                Voxel::Dirt => textures.dirt.clone(),
                Voxel::Stone => textures.stone.clone(),
                Voxel::Sand => textures.sand.clone(),
                Voxel::Water => textures.water.clone(),
                Voxel::Light => textures.light.clone(),
                _ => textures.stone.clone(),
            };
        } else {
            *vis = Visibility::Hidden;
        }
    }
}
