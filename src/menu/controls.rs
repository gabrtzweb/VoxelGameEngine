use bevy::prelude::*;
use bevy::{picking::hover::Hovered, ui::Pressed, ui_widgets::Button};

use crate::{
    core::{
        AppFont, ControlSettings, InputAction, InputBinding, LocalConfig, Localization,
        text_shadow_default,
    },
    menu::{GameSettings, MenuState},
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
pub struct RebindState(pub Option<InputAction>);

#[derive(Component)]
pub struct ControlsMenuRoot;

#[derive(Component)]
pub struct ControlsMenuCard;

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
    game_settings: Res<GameSettings>,
    active_tab: Res<ControlsTab>,
    rebind_state: Res<RebindState>,
    loc: Res<Localization>,
    app_font: Option<Res<AppFont>>,
    asset_server: Res<AssetServer>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source());

    let btn_normal = asset_server.load("textures/interfaces/containers/buttons/button_medium.png");
    let btn_hover = asset_server.load("textures/interfaces/containers/buttons/button_medium_hover.png");
    let small_normal = asset_server.load("textures/interfaces/containers/buttons/button_small.png");

    let mut header_font = TextFont {
        font_size: FontSize::Px(32.0),
        ..default()
    };
    if let Some(ref font) = font_handle {
        header_font.font = font.clone();
    }

    let dimming_alpha = game_settings.background_dimming.alpha();

    commands
        .spawn((
            ControlsMenuRoot,
            crate::menu::MenuBackdrop,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(80.0),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, dimming_alpha)),
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            let scale_factor = (game_settings.gui_scale as f32 / 4.0).clamp(0.5, 1.0);
            backdrop
                .spawn((
                    ControlsMenuCard,
                    UiTransform {
                        scale: Vec2::splat(scale_factor),
                        ..UiTransform::IDENTITY
                    },
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(14.0),
                        width: px(960.0),
                        ..default()
                    },
                ))
                .with_children(|card| {
                    // Header title
                    card.spawn((
                        Text::new(loc.t("menu.controls.title")),
                        header_font,
                        TextColor(Color::srgb(0.92, 0.92, 0.95)),
                        text_shadow_default(),
                    ));

                    // Category Tabs
                    card.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        column_gap: px(12.0),
                        ..default()
                    })
                    .with_children(|tab_row| {
                        let tabs = [
                            (ControlsTab::Movement, "menu.controls.tab_movement"),
                            (ControlsTab::Gameplay, "menu.controls.tab_gameplay"),
                            (ControlsTab::Interfaces, "menu.controls.tab_interfaces"),
                            (ControlsTab::MouseCamera, "menu.controls.tab_mouse"),
                        ];

                        for (tab, label_key) in tabs {
                            let is_active = tab == *active_tab;
                            let text_color = if is_active {
                                Color::srgb(1.0, 0.88, 0.45)
                            } else {
                                Color::srgb(0.78, 0.78, 0.82)
                            };

                            tab_row
                                .spawn((
                                    Button,
                                    Hovered::default(),
                                    ControlsTabButton(tab),
                                    ControlsAction::SwitchTab(tab),
                                    Node {
                                        width: px(170.0),
                                        height: px(38.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ImageNode {
                                        image: if is_active {
                                            btn_hover.clone()
                                        } else {
                                            btn_normal.clone()
                                        },
                                        ..default()
                                    },
                                ))
                                .with_children(|btn| {
                                    let mut tf = TextFont {
                                        font_size: FontSize::Px(16.0),
                                        ..default()
                                    };
                                    if let Some(ref font) = font_handle {
                                        tf.font = font.clone();
                                    }
                                    btn.spawn((
                                        Text::new(loc.t(label_key)),
                                        tf,
                                        TextColor(text_color),
                                        text_shadow_default(),
                                    ));
                                });
                        }
                    });

                    // Sections Container (Fixed height with dark border background)
                    card.spawn(Node {
                        width: px(920.0),
                        height: px(360.0),
                        padding: UiRect::axes(px(20.0), px(16.0)),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        border: UiRect::all(px(2.0)),
                        ..default()
                    })
                    .insert(BorderColor::all(Color::srgba(0.20, 0.22, 0.28, 0.80)))
                    .insert(BackgroundColor(Color::srgba(0.06, 0.07, 0.09, 0.85)))
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
                                    width: px(880.0),
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
                                        &small_normal,
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
                                    width: px(880.0),
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
                                        &small_normal,
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
                                    width: px(880.0),
                                    ..default()
                                },
                            ))
                            .with_children(|sec| {
                                let interface_actions = [
                                    InputAction::Inventory,
                                    InputAction::WorldMap,
                                    InputAction::Journal,
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
                                        &small_normal,
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
                                    width: px(880.0),
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
                                    width: px(680.0),
                                    height: px(40.0),
                                    padding: UiRect::axes(px(16.0), px(4.0)),
                                    ..default()
                                })
                                .with_children(|row| {
                                    let mut tf = TextFont {
                                        font_size: FontSize::Px(18.0),
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

                                    // Stepper container
                                    row.spawn(Node {
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: px(8.0),
                                        ..default()
                                    })
                                    .with_children(|st| {
                                        // Left button
                                        st.spawn((
                                            Button,
                                            Hovered::default(),
                                            ControlsAction::DecSensitivity,
                                            Node {
                                                width: px(36.0),
                                                height: px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            ImageNode {
                                                image: small_normal.clone(),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|b| {
                                            b.spawn((
                                                Text::new("◀"),
                                                tf.clone(),
                                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                                text_shadow_default(),
                                            ));
                                        });

                                        // Label
                                        st.spawn((
                                            MouseSensitivityLabel,
                                            Text::new(format!("{:.1}x", controls.mouse_sensitivity)),
                                            tf.clone(),
                                            TextColor(Color::srgb(1.0, 0.88, 0.45)),
                                            text_shadow_default(),
                                            Node {
                                                width: px(100.0),
                                                justify_content: JustifyContent::Center,
                                                ..default()
                                            },
                                        ));

                                        // Right button
                                        st.spawn((
                                            Button,
                                            Hovered::default(),
                                            ControlsAction::IncSensitivity,
                                            Node {
                                                width: px(36.0),
                                                height: px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            ImageNode {
                                                image: small_normal.clone(),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|b| {
                                            b.spawn((
                                                Text::new("▶"),
                                                tf.clone(),
                                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                                text_shadow_default(),
                                            ));
                                        });
                                    });
                                });

                                // Invert Y Toggle row
                                sec.spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    width: px(680.0),
                                    height: px(40.0),
                                    padding: UiRect::axes(px(16.0), px(4.0)),
                                    ..default()
                                })
                                .with_children(|row| {
                                    let mut tf = TextFont {
                                        font_size: FontSize::Px(18.0),
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
                                    row.spawn((
                                        Button,
                                        Hovered::default(),
                                        ControlsAction::ToggleInvertY,
                                        Node {
                                            width: px(140.0),
                                            height: px(36.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        ImageNode {
                                            image: small_normal.clone(),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|b| {
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
                                    });
                                });
                            });
                    });

                    // Bottom Bar (Reset to defaults & Back)
                    card.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        width: px(920.0),
                        column_gap: px(24.0),
                        margin: UiRect::top(px(8.0)),
                        ..default()
                    })
                    .with_children(|bottom| {
                        // Reset Defaults Button
                        bottom
                            .spawn((
                                Button,
                                Hovered::default(),
                                ControlsAction::ResetDefaults,
                                Node {
                                    width: px(240.0),
                                    height: px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ImageNode {
                                    image: btn_normal.clone(),
                                    ..default()
                                },
                            ))
                            .with_children(|btn| {
                                let mut tf = TextFont {
                                    font_size: FontSize::Px(18.0),
                                    ..default()
                                };
                                if let Some(ref font) = font_handle {
                                    tf.font = font.clone();
                                }
                                btn.spawn((
                                    Text::new(loc.t("menu.controls.reset_defaults")),
                                    tf,
                                    TextColor(Color::srgb(0.92, 0.65, 0.65)),
                                    text_shadow_default(),
                                ));
                            });

                        // Back Button
                        bottom
                            .spawn((
                                Button,
                                Hovered::default(),
                                ControlsAction::Back,
                                Node {
                                    width: px(200.0),
                                    height: px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ImageNode {
                                    image: btn_normal.clone(),
                                    ..default()
                                },
                            ))
                            .with_children(|btn| {
                                let mut tf = TextFont {
                                    font_size: FontSize::Px(20.0),
                                    ..default()
                                };
                                if let Some(ref font) = font_handle {
                                    tf.font = font.clone();
                                }
                                btn.spawn((
                                    Text::new(loc.t("menu.settings.back")),
                                    tf,
                                    TextColor(Color::srgb(0.90, 0.92, 0.96)),
                                    text_shadow_default(),
                                ));
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
    btn_texture: &Handle<Image>,
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
            width: px(860.0),
            height: px(38.0),
            padding: UiRect::axes(px(16.0), px(2.0)),
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
            row.spawn((
                Button,
                Hovered::default(),
                ActionBindingButton(action),
                ControlsAction::StartRebind(action),
                Node {
                    width: px(180.0),
                    height: px(34.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: btn_texture.clone(),
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
    buttons_query: Query<(&Hovered, Has<Pressed>, &ControlsAction), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut active_tab: ResMut<ControlsTab>,
    mut rebind_state: ResMut<RebindState>,
    mut controls: ResMut<ControlSettings>,
    mut local_config: ResMut<LocalConfig>,
) {
    if rebind_state.0.is_some() {
        return;
    }

    let left_clicked = mouse.just_pressed(MouseButton::Left);

    for (hovered, is_pressed, action) in &buttons_query {
        if hovered.get() && (left_clicked || is_pressed) {
            match action {
                ControlsAction::SwitchTab(tab) => {
                    *active_tab = *tab;
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
                    next_state.set(MenuState::Pause);
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
    for m in mouse.get_just_pressed() {
        controls.set_binding(action, InputBinding::Mouse(*m));
        local_config.update_from_control_settings(&controls);
        let _ = local_config.save();
        rebind_state.0 = None;
        return;
    }
}

fn update_controls_tab_view(
    active_tab: Res<ControlsTab>,
    asset_server: Res<AssetServer>,
    mut tab_buttons_query: Query<(&ControlsTabButton, &Children, &mut ImageNode)>,
    mut text_color_query: Query<&mut TextColor>,
    mut movement_query: Query<&mut Node, (With<MovementSection>, Without<GameplaySection>, Without<InterfacesSection>, Without<MouseCameraSection>)>,
    mut gameplay_query: Query<&mut Node, (With<GameplaySection>, Without<MovementSection>, Without<InterfacesSection>, Without<MouseCameraSection>)>,
    mut interfaces_query: Query<&mut Node, (With<InterfacesSection>, Without<MovementSection>, Without<GameplaySection>, Without<MouseCameraSection>)>,
    mut mouse_camera_query: Query<&mut Node, (With<MouseCameraSection>, Without<MovementSection>, Without<GameplaySection>, Without<InterfacesSection>)>,
) {
    if !active_tab.is_changed() {
        return;
    }

    for mut n in &mut movement_query {
        n.display = if *active_tab == ControlsTab::Movement { Display::Flex } else { Display::None };
    }
    for mut n in &mut gameplay_query {
        n.display = if *active_tab == ControlsTab::Gameplay { Display::Flex } else { Display::None };
    }
    for mut n in &mut interfaces_query {
        n.display = if *active_tab == ControlsTab::Interfaces { Display::Flex } else { Display::None };
    }
    for mut n in &mut mouse_camera_query {
        n.display = if *active_tab == ControlsTab::MouseCamera { Display::Flex } else { Display::None };
    }

    let btn_normal: Handle<Image> = asset_server.load("textures/interfaces/containers/buttons/button_medium.png");
    let btn_hover: Handle<Image> = asset_server.load("textures/interfaces/containers/buttons/button_medium_hover.png");

    for (tab_btn, children, mut img) in &mut tab_buttons_query {
        let is_active = tab_btn.0 == *active_tab;
        img.image = if is_active {
            btn_hover.clone()
        } else {
            btn_normal.clone()
        };
        let color = if is_active {
            Color::srgb(1.0, 0.88, 0.45)
        } else {
            Color::srgb(0.78, 0.78, 0.82)
        };
        for child in children.iter() {
            if let Ok(mut text_color) = text_color_query.get_mut(child) {
                text_color.0 = color;
            }
        }
    }
}

fn update_controls_card_scale(
    game_settings: Res<GameSettings>,
    mut card_query: Query<&mut UiTransform, With<ControlsMenuCard>>,
) {
    if !game_settings.is_changed() {
        return;
    }
    let factor = (game_settings.gui_scale as f32 / 4.0).clamp(0.5, 1.0);
    for mut card_tf in &mut card_query {
        card_tf.scale = Vec2::splat(factor);
    }
}

fn update_action_binding_labels(
    controls: Res<ControlSettings>,
    rebind_state: Res<RebindState>,
    loc: Res<Localization>,
    mut label_query: Query<(&ActionBindingLabel, &mut Text, &mut TextColor)>,
    mut sens_query: Query<&mut Text, (With<MouseSensitivityLabel>, Without<ActionBindingLabel>, Without<InvertYLabel>)>,
    mut invert_query: Query<(&mut Text, &mut TextColor), (With<InvertYLabel>, Without<ActionBindingLabel>, Without<MouseSensitivityLabel>)>,
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
