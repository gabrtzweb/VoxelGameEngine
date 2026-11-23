use bevy::prelude::*;
use bevy::{picking::hover::Hovered, ui::Pressed, ui_widgets::Button};

use crate::{
    core::{
        AppFont, ControlSettings, InputAction, InputBinding, LocalConfig, Localization,
        text_shadow_default,
    },
    menu::{GameSettings, MenuReturnTarget, MenuState, ThemedButton, theme::palette},
};

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlsTab {
    #[default]
    Movement,
    Gameplay,
    Interfaces,
    MouseCamera,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlsDetailOpen(pub bool);

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RebindState(pub Option<InputAction>);

#[derive(Component)]
pub struct ControlsMenuRoot;

#[derive(Component)]
pub struct ControlsMenuCard;

#[derive(Component)]
struct ControlsDetailPanel;

#[derive(Component)]
struct ControlsTabLabel;

#[derive(Component)]
struct ControlsTabAccent;

#[derive(Component)]
struct MovementSection;

#[derive(Component)]
struct GameplaySection;

#[derive(Component)]
struct InterfacesSection;

#[derive(Component)]
struct MouseCameraSection;

#[derive(Component)]
struct ControlsTabButton(ControlsTab);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlsAction {
    SwitchTab(ControlsTab),
    StartRebind(InputAction),
    CancelRebind,
    ResetDefaults,
    Back,

    DecSensitivity,
    IncSensitivity,
    ToggleInvertY,
}

#[derive(Component)]
struct ActionBindingLabel(InputAction);

#[derive(Component)]
struct ActionBindingButton(#[allow(dead_code)] InputAction);

#[derive(Component)]
struct MouseSensitivityLabel;

#[derive(Component)]
struct InvertYLabel;

pub struct ControlsMenuPlugin;

impl Plugin for ControlsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlsTab>()
            .init_resource::<ControlsDetailOpen>()
            .init_resource::<RebindState>()
            .add_systems(OnEnter(MenuState::Controls), spawn_controls_menu)
            .add_systems(OnExit(MenuState::Controls), despawn_controls_menu)
            .add_systems(
                Update,
                (
                    handle_controls_buttons,
                    capture_rebind_input,
                    update_controls_tab_view,
                    update_controls_card_scale,
                    update_action_binding_labels,
                )
                    .run_if(in_state(MenuState::Controls)),
            );
    }
}

fn spawn_controls_menu(
    mut commands: Commands,
    controls: Res<ControlSettings>,
    _game_settings: Res<GameSettings>,
    active_tab: Res<ControlsTab>,
    mut detail_open: ResMut<ControlsDetailOpen>,
    rebind_state: Res<RebindState>,
    loc: Res<Localization>,
    app_font: Option<Res<AppFont>>,
    asset_server: Res<AssetServer>,
    menu_return_target: Res<MenuReturnTarget>,
) {
    use crate::menu::menu_bar::MENU_BAR_HEIGHT;

    detail_open.0 = false;
    let font_handle = app_font.as_ref().map(|f| f.source());

    let mut header_font = TextFont {
        font_size: FontSize::Px(13.0),
        ..default()
    };
    if let Some(ref font) = font_handle {
        header_font.font = font.clone();
    }

    let is_from_main_menu = menu_return_target.0 == MenuState::MainMenu;
    let top_offset = if is_from_main_menu {
        0.0
    } else {
        MENU_BAR_HEIGHT
    };
    let left_offset = if is_from_main_menu {
        0.0
    } else {
        palette::DRAWER_COL1_WIDTH
    };

    commands
        .spawn((
            ControlsMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(left_offset),
                top: px(top_offset),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                ..default()
            },
            BackgroundColor(Color::NONE),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            if is_from_main_menu {
                let bg_texture = asset_server.load("textures/interfaces/main_menu_bg.png");
                backdrop.spawn((
                    ImageNode {
                        image: bg_texture,
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    ZIndex(0),
                ));
                backdrop.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                    ZIndex(1),
                ));
            }

            backdrop
                .spawn((
                    ControlsMenuCard,
                    ZIndex(10),
                    Node {
                        height: percent(100.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                ))
                .with_children(|drawers| {
                    // Column 2: Controls Categories Drawer
                    drawers
                        .spawn((
                            Node {
                                width: px(palette::DRAWER_COL2_WIDTH),
                                min_width: px(palette::DRAWER_COL2_WIDTH),
                                flex_shrink: 0.0,
                                height: percent(100.0),
                                border: UiRect::right(px(1.5)),
                                padding: UiRect::vertical(px(28.0)),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Stretch,
                                row_gap: px(4.0),
                                ..default()
                            },
                            BackgroundColor(palette::DRAWER_COL2_BG),
                            BorderColor::all(palette::DRAWER_DIVIDER),
                        ))
                        .with_children(|cat_col| {
                            if is_from_main_menu {
                                let mut back_font = TextFont {
                                    font_size: FontSize::Px(15.0),
                                    ..default()
                                };
                                if let Some(ref font) = font_handle {
                                    back_font.font = font.clone();
                                }
                                let back_theme = ThemedButton {
                                    normal_bg: Color::NONE,
                                    normal_border: Color::NONE,
                                    hover_bg: palette::NAV_ITEM_HOVER_BG,
                                    hover_border: Color::NONE,
                                    pressed_bg: palette::NAV_ITEM_ACTIVE_BG,
                                };
                                cat_col
                                    .spawn((
                                        Button,
                                        Hovered::default(),
                                        ControlsAction::Back,
                                        back_theme,
                                        BackgroundColor(Color::NONE),
                                        BorderColor::all(Color::NONE),
                                        Node {
                                            width: percent(100.0),
                                            height: px(44.0),
                                            padding: UiRect::new(
                                                px(24.0),
                                                px(20.0),
                                                px(0.0),
                                                px(0.0),
                                            ),
                                            margin: UiRect::bottom(px(8.0)),
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new(format!(
                                                "<  {}",
                                                loc.t("menu.settings.back")
                                            )),
                                            back_font,
                                            TextColor(palette::TEXT_MUTED),
                                            text_shadow_default(),
                                        ));
                                    });
                            }

                            let tabs = [
                                (ControlsTab::Movement, "menu.controls.tab_movement"),
                                (ControlsTab::Gameplay, "menu.controls.tab_gameplay"),
                                (ControlsTab::Interfaces, "menu.controls.tab_interfaces"),
                                (ControlsTab::MouseCamera, "menu.controls.tab_mouse"),
                            ];

                            for (tab, label_key) in tabs {
                                cat_col
                                    .spawn((
                                        Button,
                                        Hovered::default(),
                                        ControlsTabButton(tab),
                                        ControlsAction::SwitchTab(tab),
                                        Node {
                                            width: percent(100.0),
                                            height: px(46.0),
                                            padding: UiRect::new(
                                                px(24.0),
                                                px(20.0),
                                                px(0.0),
                                                px(0.0),
                                            ),
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Row,
                                            justify_content: JustifyContent::SpaceBetween,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                        BorderColor::all(Color::NONE),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            ControlsTabAccent,
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: px(0.0),
                                                top: px(8.0),
                                                bottom: px(8.0),
                                                width: px(3.0),
                                                border_radius: BorderRadius::right(px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(palette::ACCENT_LINE),
                                            Visibility::Hidden,
                                        ));

                                        let mut tf = TextFont {
                                            font_size: FontSize::Px(15.0),
                                            ..default()
                                        };
                                        if let Some(ref font) = font_handle {
                                            tf.font = font.clone();
                                        }
                                        btn.spawn((
                                            ControlsTabLabel,
                                            Text::new(loc.t(label_key)),
                                            tf.clone(),
                                            TextColor(palette::TEXT_PRIMARY),
                                            text_shadow_default(),
                                        ));
                                        btn.spawn((
                                            Text::new(">"),
                                            tf,
                                            TextColor(palette::TEXT_MUTED),
                                        ));
                                    });
                            }

                            // Push bottom action buttons to foot of Column 2
                            cat_col.spawn(Node {
                                flex_grow: 1.0,
                                ..default()
                            });

                            let action_theme = ThemedButton::subtle();
                            cat_col
                                .spawn(Node {
                                    padding: UiRect::horizontal(px(20.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(8.0),
                                    ..default()
                                })
                                .with_children(|footer| {
                                    let mut tf = TextFont {
                                        font_size: FontSize::Px(14.0),
                                        ..default()
                                    };
                                    if let Some(ref font) = font_handle {
                                        tf.font = font.clone();
                                    }

                                    // Reset Defaults Button
                                    footer
                                        .spawn((
                                            Button,
                                            Hovered::default(),
                                            ControlsAction::ResetDefaults,
                                            action_theme,
                                            BackgroundColor(action_theme.normal_bg),
                                            BorderColor::all(action_theme.normal_border),
                                            Node {
                                                width: percent(100.0),
                                                height: px(38.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(px(1.5)),
                                                border_radius: BorderRadius::all(px(4.0)),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|btn| {
                                            btn.spawn((
                                                Text::new(loc.t("menu.controls.reset_defaults")),
                                                tf,
                                                TextColor(Color::srgb(0.92, 0.65, 0.65)),
                                                text_shadow_default(),
                                            ));
                                        });
                                });
                        });

                    // Column 3: Controls Detail Panel (opens when a category in Column 2 is selected)
                    drawers
                        .spawn((
                            ControlsDetailPanel,
                            Node {
                                width: px(palette::DRAWER_COL3_WIDTH),
                                height: percent(100.0),
                                border: UiRect::right(px(1.5)),
                                padding: UiRect::all(px(24.0)),
                                display: Display::None,
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Stretch,
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            BackgroundColor(palette::DRAWER_COL3_BG),
                            BorderColor::all(palette::DRAWER_DIVIDER),
                        ))
                        .with_children(|sections| {
                            // 1. Movement section
                            sections
                                .spawn((
                                    MovementSection,
                                    Node {
                                        display: if *active_tab == ControlsTab::Movement {
                                            Display::Flex
                                        } else {
                                            Display::None
                                        },
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(6.0),
                                        width: percent(100.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|sec| {
                                    let movement_actions = [
                                        InputAction::MoveForward,
                                        InputAction::MoveBackward,
                                        InputAction::MoveLeft,
                                        InputAction::MoveRight,
                                        InputAction::Jump,
                                        InputAction::Crouch,
                                        InputAction::Sprint,
                                    ];
                                    for action in movement_actions {
                                        spawn_action_row(
                                            sec,
                                            action,
                                            &controls,
                                            &rebind_state,
                                            &loc,
                                            font_handle.as_ref(),
                                        );
                                    }
                                });

                            // 2. Gameplay section
                            sections
                                .spawn((
                                    GameplaySection,
                                    Node {
                                        display: if *active_tab == ControlsTab::Gameplay {
                                            Display::Flex
                                        } else {
                                            Display::None
                                        },
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(6.0),
                                        width: percent(100.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|sec| {
                                    let gameplay_actions = [
                                        InputAction::PrimaryAction,
                                        InputAction::SecondaryAction,
                                        InputAction::PickBlock,
                                        InputAction::DropItem,
                                        InputAction::ShapeCycle,
                                        InputAction::ShapeInvert,
                                        InputAction::Zoom,
                                    ];
                                    for action in gameplay_actions {
                                        spawn_action_row(
                                            sec,
                                            action,
                                            &controls,
                                            &rebind_state,
                                            &loc,
                                            font_handle.as_ref(),
                                        );
                                    }
                                });

                            // 3. Interfaces section
                            sections
                                .spawn((
                                    InterfacesSection,
                                    Node {
                                        display: if *active_tab == ControlsTab::Interfaces {
                                            Display::Flex
                                        } else {
                                            Display::None
                                        },
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(6.0),
                                        width: percent(100.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|sec| {
                                    let interface_actions = [
                                        InputAction::Inventory,
                                        InputAction::Skills,
                                        InputAction::Journal,
                                        InputAction::Quests,
                                        InputAction::WorldMap,
                                        InputAction::MenuTabPrev,
                                        InputAction::MenuTabNext,
                                    ];
                                    for action in interface_actions {
                                        spawn_action_row(
                                            sec,
                                            action,
                                            &controls,
                                            &rebind_state,
                                            &loc,
                                            font_handle.as_ref(),
                                        );
                                    }
                                });

                            // 4. Mouse & Camera section
                            sections
                                .spawn((
                                    MouseCameraSection,
                                    Node {
                                        display: if *active_tab == ControlsTab::MouseCamera {
                                            Display::Flex
                                        } else {
                                            Display::None
                                        },
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(16.0),
                                        width: percent(100.0),
                                        align_items: AlignItems::Center,
                                        padding: UiRect::top(px(24.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|sec| {
                                    // Mouse sensitivity stepper row
                                    sec.spawn(Node {
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        width: percent(100.0),
                                        height: px(40.0),
                                        padding: UiRect::axes(px(4.0), px(2.0)),
                                        ..default()
                                    })
                                    .with_children(|row| {
                                        let mut tf = TextFont {
                                            font_size: FontSize::Px(15.0),
                                            ..default()
                                        };
                                        if let Some(ref f) = font_handle {
                                            tf.font = f.clone();
                                        }
                                        row.spawn((
                                            Text::new(loc.t("controls.mouse.sensitivity")),
                                            tf.clone(),
                                            TextColor(Color::srgb(0.88, 0.90, 0.94)),
                                            text_shadow_default(),
                                        ));

                                        // Stepper container: [-] Value [+]
                                        row.spawn(Node {
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            column_gap: px(6.0),
                                            ..default()
                                        })
                                        .with_children(
                                            |st| {
                                                // Left button
                                                let small_theme = ThemedButton::subtle();
                                                st.spawn((
                                                    Button,
                                                    Hovered::default(),
                                                    ControlsAction::DecSensitivity,
                                                    small_theme,
                                                    BackgroundColor(small_theme.normal_bg),
                                                    BorderColor::all(small_theme.normal_border),
                                                    Node {
                                                        width: px(28.0),
                                                        height: px(28.0),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        border: UiRect::all(px(1.5)),
                                                        border_radius: BorderRadius::all(px(4.0)),
                                                        ..default()
                                                    },
                                                ))
                                                .with_children(|b| {
                                                    b.spawn((
                                                        Text::new("-"),
                                                        tf.clone(),
                                                        TextColor(Color::WHITE),
                                                        text_shadow_default(),
                                                    ));
                                                });

                                                // Label
                                                st.spawn((
                                                    MouseSensitivityLabel,
                                                    Text::new(format!(
                                                        "{:.1}x",
                                                        controls.mouse_sensitivity
                                                    )),
                                                    tf.clone(),
                                                    TextColor(Color::srgb(1.0, 0.88, 0.45)),
                                                    text_shadow_default(),
                                                    Node {
                                                        width: px(76.0),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        ..default()
                                                    },
                                                ));

                                                // Right button
                                                st.spawn((
                                                    Button,
                                                    Hovered::default(),
                                                    ControlsAction::IncSensitivity,
                                                    small_theme,
                                                    BackgroundColor(small_theme.normal_bg),
                                                    BorderColor::all(small_theme.normal_border),
                                                    Node {
                                                        width: px(28.0),
                                                        height: px(28.0),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        border: UiRect::all(px(1.5)),
                                                        border_radius: BorderRadius::all(px(4.0)),
                                                        ..default()
                                                    },
                                                ))
                                                .with_children(|b| {
                                                    b.spawn((
                                                        Text::new("+"),
                                                        tf.clone(),
                                                        TextColor(Color::WHITE),
                                                        text_shadow_default(),
                                                    ));
                                                });
                                            },
                                        );
                                    });

                                    // Invert Y Toggle row
                                    sec.spawn(Node {
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        width: percent(100.0),
                                        height: px(40.0),
                                        padding: UiRect::axes(px(4.0), px(2.0)),
                                        ..default()
                                    })
                                    .with_children(|row| {
                                        let mut tf = TextFont {
                                            font_size: FontSize::Px(15.0),
                                            ..default()
                                        };
                                        if let Some(ref f) = font_handle {
                                            tf.font = f.clone();
                                        }
                                        row.spawn((
                                            Text::new(loc.t("controls.mouse.invert_y")),
                                            tf.clone(),
                                            TextColor(Color::srgb(0.88, 0.90, 0.94)),
                                            text_shadow_default(),
                                        ));

                                        // Toggle button
                                        let toggle_theme = ThemedButton::subtle();
                                        row.spawn((
                                            Button,
                                            Hovered::default(),
                                            ControlsAction::ToggleInvertY,
                                            toggle_theme,
                                            BackgroundColor(toggle_theme.normal_bg),
                                            BorderColor::all(toggle_theme.normal_border),
                                            Node {
                                                width: px(144.0),
                                                height: px(34.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(px(1.5)),
                                                border_radius: BorderRadius::all(px(4.0)),
                                                ..default()
                                            },
                                        ))
                                        .with_children(
                                            |b| {
                                                let status_str = if controls.invert_mouse_y {
                                                    loc.t("common.enabled")
                                                } else {
                                                    loc.t("common.disabled")
                                                };
                                                b.spawn((
                                                    InvertYLabel,
                                                    Text::new(status_str),
                                                    tf.clone(),
                                                    TextColor(if controls.invert_mouse_y {
                                                        Color::srgb(0.4, 0.9, 0.5)
                                                    } else {
                                                        Color::srgb(0.8, 0.8, 0.8)
                                                    }),
                                                    text_shadow_default(),
                                                ));
                                            },
                                        );
                                    });
                                });
                        });
                });
        });
}

fn spawn_action_row(
    parent: &mut ChildSpawnerCommands,
    action: InputAction,
    controls: &ControlSettings,
    rebind_state: &RebindState,
    loc: &Localization,
    font: Option<&FontSource>,
) {
    let binding = controls.get_binding(action);
    let is_rebinding = rebind_state.0 == Some(action);
    let has_conflict = controls.has_conflict(action);

    let display_str = if is_rebinding {
        format!("> {} <", loc.t("menu.controls.press_key"))
    } else {
        let name = binding.display_name(Some(loc));
        if has_conflict {
            format!("{name} (!)")
        } else {
            name
        }
    };

    let btn_text_color = if is_rebinding {
        Color::srgb(1.0, 0.90, 0.30)
    } else if has_conflict {
        Color::srgb(1.0, 0.40, 0.35)
    } else {
        Color::srgb(0.85, 0.90, 0.96)
    };

    let mut tf = TextFont {
        font_size: FontSize::Px(16.0),
        ..default()
    };
    if let Some(f) = font {
        tf.font = f.clone();
    }

    parent
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: percent(100.0),
            height: px(40.0),
            padding: UiRect::axes(px(4.0), px(2.0)),
            ..default()
        })
        .with_children(|row| {
            // Action Label
            row.spawn((
                Text::new(loc.t(action.label_key())),
                tf.clone(),
                TextColor(Color::srgb(0.85, 0.88, 0.92)),
                text_shadow_default(),
            ));

            // Rebind Button
            let rebind_theme = ThemedButton::subtle();
            row.spawn((
                Button,
                Hovered::default(),
                ActionBindingButton(action),
                ControlsAction::StartRebind(action),
                rebind_theme,
                BackgroundColor(rebind_theme.normal_bg),
                BorderColor::all(rebind_theme.normal_border),
                Node {
                    width: px(152.0),
                    height: px(34.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(1.5)),
                    border_radius: BorderRadius::all(px(4.0)),
                    ..default()
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    ActionBindingLabel(action),
                    Text::new(display_str),
                    tf.clone(),
                    TextColor(btn_text_color),
                    text_shadow_default(),
                ));
            });
        });
}

fn despawn_controls_menu(mut commands: Commands, query: Query<Entity, With<ControlsMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_controls_buttons(
    mut buttons_query: Query<
        (
            &Hovered,
            Has<Pressed>,
            &ControlsAction,
            Option<&ThemedButton>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mouse: Res<ButtonInput<MouseButton>>,
    menu_return_target: Res<MenuReturnTarget>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut active_tab: ResMut<ControlsTab>,
    mut detail_open: ResMut<ControlsDetailOpen>,
    mut rebind_state: ResMut<RebindState>,
    mut controls: ResMut<ControlSettings>,
    mut local_config: ResMut<LocalConfig>,
) {
    if rebind_state.0.is_some() {
        return;
    }

    let left_clicked = mouse.just_pressed(MouseButton::Left);

    for (hovered, is_pressed, action, theme, mut bg, mut border) in &mut buttons_query {
        if let Some(theme) = theme {
            let (next_bg, next_border) = theme.colors(hovered.get(), is_pressed);
            bg.0 = next_bg;
            *border = BorderColor::all(next_border);
        }
        if hovered.get() && left_clicked {
            match action {
                ControlsAction::SwitchTab(tab) => {
                    if detail_open.0 && *active_tab == *tab {
                        detail_open.0 = false;
                    } else {
                        *active_tab = *tab;
                        detail_open.0 = true;
                    }
                    rebind_state.0 = None;
                }
                ControlsAction::StartRebind(act) => {
                    rebind_state.0 = Some(*act);
                }
                ControlsAction::CancelRebind => {
                    rebind_state.0 = None;
                }
                ControlsAction::ResetDefaults => {
                    controls.reset_to_defaults();
                    local_config.update_from_control_settings(&controls);
                    let _ = local_config.save();
                    rebind_state.0 = None;
                }
                ControlsAction::Back => {
                    rebind_state.0 = None;
                    next_state.set(menu_return_target.0);
                }
                ControlsAction::DecSensitivity => {
                    controls.mouse_sensitivity =
                        ((controls.mouse_sensitivity * 10.0).round() - 1.0) / 10.0;
                    controls.mouse_sensitivity = controls.mouse_sensitivity.clamp(0.2, 3.0);
                    local_config.update_from_control_settings(&controls);
                    let _ = local_config.save();
                }
                ControlsAction::IncSensitivity => {
                    controls.mouse_sensitivity =
                        ((controls.mouse_sensitivity * 10.0).round() + 1.0) / 10.0;
                    controls.mouse_sensitivity = controls.mouse_sensitivity.clamp(0.2, 3.0);
                    local_config.update_from_control_settings(&controls);
                    let _ = local_config.save();
                }
                ControlsAction::ToggleInvertY => {
                    controls.invert_mouse_y = !controls.invert_mouse_y;
                    local_config.update_from_control_settings(&controls);
                    let _ = local_config.save();
                }
            }
        }
    }
}

/// Listens for any pressed keyboard key or mouse button while in rebind mode.
fn capture_rebind_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut rebind_state: ResMut<RebindState>,
    mut controls: ResMut<ControlSettings>,
    mut local_config: ResMut<LocalConfig>,
) {
    if rebind_state.is_changed() {
        return;
    }

    let Some(action) = rebind_state.0 else {
        return;
    };

    // Escape cancels rebind without altering the key
    if keyboard.just_pressed(KeyCode::Escape) {
        rebind_state.0 = None;
        return;
    }

    // Check keyboard keys
    for key in keyboard.get_just_pressed() {
        if *key == KeyCode::Escape {
            continue;
        }
        controls.set_binding(action, InputBinding::Key(*key));
        local_config.update_from_control_settings(&controls);
        let _ = local_config.save();
        rebind_state.0 = None;
        return;
    }

    // Check mouse buttons
    if let Some(m) = mouse.get_just_pressed().next() {
        controls.set_binding(action, InputBinding::Mouse(*m));
        local_config.update_from_control_settings(&controls);
        let _ = local_config.save();
        rebind_state.0 = None;
    }
}

fn update_controls_tab_view(
    active_tab: Res<ControlsTab>,
    detail_open: Res<ControlsDetailOpen>,
    mut tab_buttons_query: Query<(
        &ControlsTabButton,
        &Hovered,
        &Children,
        &mut BackgroundColor,
    )>,
    mut text_color_query: Query<&mut TextColor, With<ControlsTabLabel>>,
    mut accent_query: Query<&mut Visibility, With<ControlsTabAccent>>,
    mut detail_panel_query: Query<
        &mut Node,
        (
            With<ControlsDetailPanel>,
            Without<MovementSection>,
            Without<GameplaySection>,
            Without<InterfacesSection>,
            Without<MouseCameraSection>,
        ),
    >,
    mut movement_query: Query<
        &mut Node,
        (
            With<MovementSection>,
            Without<GameplaySection>,
            Without<InterfacesSection>,
            Without<MouseCameraSection>,
            Without<ControlsDetailPanel>,
        ),
    >,
    mut gameplay_query: Query<
        &mut Node,
        (
            With<GameplaySection>,
            Without<MovementSection>,
            Without<InterfacesSection>,
            Without<MouseCameraSection>,
            Without<ControlsDetailPanel>,
        ),
    >,
    mut interfaces_query: Query<
        &mut Node,
        (
            With<InterfacesSection>,
            Without<MovementSection>,
            Without<GameplaySection>,
            Without<MouseCameraSection>,
            Without<ControlsDetailPanel>,
        ),
    >,
    mut mouse_camera_query: Query<
        &mut Node,
        (
            With<MouseCameraSection>,
            Without<MovementSection>,
            Without<GameplaySection>,
            Without<InterfacesSection>,
            Without<ControlsDetailPanel>,
        ),
    >,
) {
    for (tab_btn, hovered, children, mut bg) in &mut tab_buttons_query {
        let is_active = detail_open.0 && tab_btn.0 == *active_tab;
        let is_hovered = hovered.get();
        bg.0 = if is_active {
            palette::NAV_ITEM_ACTIVE_BG
        } else if is_hovered {
            palette::NAV_ITEM_HOVER_BG
        } else {
            Color::NONE
        };
        let color = if is_active || is_hovered {
            palette::TEXT_ACCENT
        } else {
            palette::TEXT_PRIMARY
        };
        for child in children.iter() {
            if let Ok(mut text_color) = text_color_query.get_mut(child) {
                text_color.0 = color;
            }
            if let Ok(mut vis) = accent_query.get_mut(child) {
                *vis = if is_active || is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }

    if !active_tab.is_changed() && !detail_open.is_changed() {
        return;
    }

    for mut n in &mut detail_panel_query {
        n.display = if detail_open.0 {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut n in &mut movement_query {
        n.display = if *active_tab == ControlsTab::Movement {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut n in &mut gameplay_query {
        n.display = if *active_tab == ControlsTab::Gameplay {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut n in &mut interfaces_query {
        n.display = if *active_tab == ControlsTab::Interfaces {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut n in &mut mouse_camera_query {
        n.display = if *active_tab == ControlsTab::MouseCamera {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn update_controls_card_scale(
    _game_settings: Res<GameSettings>,
    _card_query: Query<&mut UiTransform, With<ControlsMenuCard>>,
) {
}

fn update_action_binding_labels(
    controls: Res<ControlSettings>,
    rebind_state: Res<RebindState>,
    loc: Res<Localization>,
    mut label_query: Query<(&ActionBindingLabel, &mut Text, &mut TextColor)>,
    mut sens_query: Query<
        &mut Text,
        (
            With<MouseSensitivityLabel>,
            Without<ActionBindingLabel>,
            Without<InvertYLabel>,
        ),
    >,
    mut invert_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<InvertYLabel>,
            Without<ActionBindingLabel>,
            Without<MouseSensitivityLabel>,
        ),
    >,
) {
    if !controls.is_changed() && !rebind_state.is_changed() && !loc.is_changed() {
        return;
    }

    for (lbl, mut text, mut color) in &mut label_query {
        let action = lbl.0;
        let is_rebinding = rebind_state.0 == Some(action);
        let has_conflict = controls.has_conflict(action);
        let binding = controls.get_binding(action);

        if is_rebinding {
            text.0 = format!("> {} <", loc.t("menu.controls.press_key"));
            color.0 = Color::srgb(1.0, 0.90, 0.30);
        } else {
            let name = binding.display_name(Some(&loc));
            if has_conflict {
                text.0 = format!("{name} (!)");
                color.0 = Color::srgb(1.0, 0.40, 0.35);
            } else {
                text.0 = name;
                color.0 = Color::srgb(0.85, 0.90, 0.96);
            }
        }
    }

    for mut text in &mut sens_query {
        text.0 = format!("{:.1}x", controls.mouse_sensitivity);
    }

    for (mut text, mut color) in &mut invert_query {
        text.0 = if controls.invert_mouse_y {
            loc.t("common.enabled").to_string()
        } else {
            loc.t("common.disabled").to_string()
        };
        color.0 = if controls.invert_mouse_y {
            Color::srgb(0.4, 0.9, 0.5)
        } else {
            Color::srgb(0.8, 0.8, 0.8)
        };
    }
}
