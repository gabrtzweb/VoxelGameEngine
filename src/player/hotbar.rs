use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};

use crate::core::{AppFont, text_shadow_default};
use crate::gameplay::{BlockIcons, SelectedVoxel};
use crate::player::InspectorInteraction;
use crate::world::Voxel;

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
                Some(Voxel::LightWarm),
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

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hotbar>()
            .add_systems(
                Startup,
                setup_hotbar_ui.after(crate::gameplay::setup_block_icons),
            )
            .add_systems(Update, (handle_hotbar_input, sync_hotbar_ui).chain());
    }
}

fn setup_hotbar_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icons: Res<BlockIcons>,
    app_font: Option<Res<AppFont>>,
) {
    let initial_hotbar = Hotbar::default();
    let font_handle = app_font.as_ref().map(|f| f.source());
    let hotbar_texture = asset_server.load("textures/interfaces/containers/hotbar.png");

    // Scale multiplier for hotbar UI rendering (2x integer scaling = 512x64 px)
    const HOTBAR_SCALE: f32 = 2.0;
    let tray_width = 256.0 * HOTBAR_SCALE;
    let tray_height = 32.0 * HOTBAR_SCALE;
    let slot_stride = 20.0 * HOTBAR_SCALE;
    let slot_left_offset = 49.0 * HOTBAR_SCALE;
    let slot_top = 7.0 * HOTBAR_SCALE;
    let slot_size = 18.0 * HOTBAR_SCALE;
    let icon_size = 16.0 * HOTBAR_SCALE;
    let border_width = 1.0 * HOTBAR_SCALE;

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
            // Scaled hotbar texture (256x32 px at 2x = 512x64 px)
            parent
                .spawn((
                    ImageNode {
                        image: hotbar_texture,
                        ..default()
                    },
                    Node {
                        width: px(tray_width),
                        height: px(tray_height),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                ))
                .with_children(|tray| {
                    for index in 0..HOTBAR_SLOT_COUNT {
                        let is_active = index == initial_hotbar.active_slot;
                        let initial_voxel = initial_hotbar.slots[index];

                        let border_color = if is_active {
                            Color::srgb(1.0, 0.85, 0.30)
                        } else {
                            Color::NONE
                        };

                        let bg_color = if is_active {
                            Color::srgba(1.0, 1.0, 1.0, 0.12)
                        } else {
                            Color::NONE
                        };

                        let icon_handle = initial_voxel
                            .map(|v| icons.get(v))
                            .unwrap_or_else(|| icons.get(Voxel::Stone));

                        let icon_visibility = if initial_voxel.is_some() {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        };

                        tray.spawn((
                            HotbarSlotUi { index },
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(slot_left_offset + index as f32 * slot_stride),
                                top: px(slot_top),
                                width: px(slot_size),
                                height: px(slot_size),
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(border_width)),
                                border_radius: BorderRadius::all(px(border_width)),
                                ..default()
                            },
                            BackgroundColor(bg_color),
                            BorderColor::all(border_color),
                        ))
                        .with_children(|slot| {
                            // Slot index number (1 through 8)
                            let mut num_font = TextFont {
                                font_size: FontSize::Px(6.0 * HOTBAR_SCALE),
                                ..default()
                            };
                            if let Some(ref font) = font_handle {
                                num_font.font = font.clone();
                            }

                            slot.spawn((
                                Text::new(format!("{}", index + 1)),
                                num_font,
                                TextColor(Color::srgba(0.90, 0.90, 0.90, 0.85)),
                                text_shadow_default(),
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: px(1.0 * HOTBAR_SCALE),
                                    left: px(2.0 * HOTBAR_SCALE),
                                    ..default()
                                },
                            ));

                            // Centered 2D item icon
                            slot.spawn((
                                HotbarSlotIcon { index },
                                ImageNode {
                                    image: icon_handle,
                                    ..default()
                                },
                                Node {
                                    width: px(icon_size),
                                    height: px(icon_size),
                                    ..default()
                                },
                                icon_visibility,
                            ));
                        });
                    }
                });
        });
}

fn handle_hotbar_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    menu_state: Option<Res<State<crate::menu::MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    mut hotbar: ResMut<Hotbar>,
    mut selected: ResMut<SelectedVoxel>,
) {
    if menu_state.is_some_and(|s| *s.get() != crate::menu::MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }
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

    // When holding Z (camera zoom), mouse wheel scroll adjusts zoom level instead of cycling hotbar
    if !keyboard.pressed(KeyCode::KeyZ) {
        let scroll = mouse_scroll.delta.y;
        if scroll > 0.05 {
            // Scroll up = previous slot
            hotbar.active_slot = (hotbar.active_slot + HOTBAR_SLOT_COUNT - 1) % HOTBAR_SLOT_COUNT;
        } else if scroll < -0.05 {
            // Scroll down = next slot
            hotbar.active_slot = (hotbar.active_slot + 1) % HOTBAR_SLOT_COUNT;
        }
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
    icons: Res<BlockIcons>,
    mut slot_query: Query<(&HotbarSlotUi, &mut BorderColor, &mut BackgroundColor)>,
    mut icon_query: Query<(&HotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    if !hotbar.is_changed() {
        return;
    }

    for (slot, mut border_color, mut bg_color) in &mut slot_query {
        if slot.index == hotbar.active_slot {
            *border_color = BorderColor::all(Color::srgb(1.0, 0.85, 0.30));
            bg_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.12);
        } else {
            *border_color = BorderColor::all(Color::NONE);
            bg_color.0 = Color::NONE;
        }
    }

    for (icon, mut img_node, mut visibility) in &mut icon_query {
        match hotbar.slots[icon.index] {
            Some(voxel) => {
                img_node.image = icons.get(voxel);
                *visibility = Visibility::Visible;
            }
            None => {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
