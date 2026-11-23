use bevy::{picking::hover::Hovered, prelude::*, ui_widgets::Button};

use crate::core::{AppFont, Localization, text_shadow_default};

use super::{GameSettings, MenuState, menu_bar::MENU_BAR_HEIGHT, theme::palette};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectedQuestIndex(pub usize);

#[derive(Component)]
struct QuestsMenuRoot;

#[derive(Component, Clone, Copy)]
struct QuestNavButton(usize);

#[derive(Component)]
struct QuestNavLabel;

#[derive(Component)]
struct QuestNavAccent;

#[derive(Component, Clone, Copy)]
struct QuestDetailCard(usize);

struct QuestEntry {
    title: &'static str,
    category: &'static str,
    description: &'static str,
    objectives: &'static [&'static str],
    reward: &'static str,
}

const SAMPLE_QUESTS: [QuestEntry; 3] = [
    QuestEntry {
        title: "menu.quests.q1.title",
        category: "menu.quests.q1.cat",
        description: "menu.quests.q1.desc",
        objectives: &[
            "menu.quests.q1.obj1",
            "menu.quests.q1.obj2",
            "menu.quests.q1.obj3",
        ],
        reward: "menu.quests.q1.reward",
    },
    QuestEntry {
        title: "menu.quests.q2.title",
        category: "menu.quests.q2.cat",
        description: "menu.quests.q2.desc",
        objectives: &[
            "menu.quests.q2.obj1",
            "menu.quests.q2.obj2",
            "menu.quests.q2.obj3",
        ],
        reward: "menu.quests.q2.reward",
    },
    QuestEntry {
        title: "menu.quests.q3.title",
        category: "menu.quests.q3.cat",
        description: "menu.quests.q3.desc",
        objectives: &[
            "menu.quests.q3.obj1",
            "menu.quests.q3.obj2",
            "menu.quests.q3.obj3",
        ],
        reward: "menu.quests.q3.reward",
    },
];

pub struct QuestsMenuPlugin;

impl Plugin for QuestsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedQuestIndex>()
            .add_systems(OnEnter(MenuState::Quests), spawn_quests_menu)
            .add_systems(OnExit(MenuState::Quests), despawn_quests_menu)
            .add_systems(
                Update,
                (handle_quest_clicks, update_quest_view).run_if(in_state(MenuState::Quests)),
            );
    }
}

fn spawn_quests_menu(
    mut commands: Commands,
    selected: Res<SelectedQuestIndex>,
    loc: Option<Res<Localization>>,
    app_font: Option<Res<AppFont>>,
    game_settings: Option<Res<GameSettings>>,
) {
    let t = |key: &'static str| -> String {
        loc.as_ref()
            .map_or_else(|| key.to_string(), |l| l.t(key).to_string())
    };
    let font_handle = app_font.as_ref().map(|f| f.source());
    let make_font = |size: f32| {
        let mut tf = TextFont {
            font_size: FontSize::Px(size),
            ..default()
        };
        if let Some(ref font) = font_handle {
            tf.font = font.clone();
        }
        tf
    };

    let dimming_alpha = game_settings.map_or(0.65, |s| s.background_dimming.alpha());

    commands
        .spawn((
            QuestsMenuRoot,
            crate::menu::MenuBackdrop,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(MENU_BAR_HEIGHT),
                right: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, dimming_alpha)),
            ZIndex(300),
        ))
        .with_children(|root| {
            // Left Column: Active Quests List (standardized 310px width)
            root.spawn((
                Node {
                    width: px(palette::DRAWER_COL1_WIDTH),
                    min_width: px(palette::DRAWER_COL1_WIDTH),
                    flex_shrink: 0.0,
                    height: percent(100.0),
                    border: UiRect::right(px(1.5)),
                    padding: UiRect::vertical(px(24.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4.0),
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL1_BG),
                BorderColor::all(palette::DRAWER_DIVIDER),
            ))
            .with_children(|list| {
                list.spawn(Node {
                    padding: UiRect::new(px(24.0), px(20.0), px(0.0), px(12.0)),
                    ..default()
                })
                .with_children(|hdr| {
                    hdr.spawn((
                        Text::new(t("menu.quests.active_header")),
                        make_font(12.0),
                        TextColor(palette::TEXT_MUTED),
                    ));
                });

                for (idx, quest) in SAMPLE_QUESTS.iter().enumerate() {
                    let is_active = idx == selected.0;
                    list.spawn((
                        Button,
                        Hovered::default(),
                        QuestNavButton(idx),
                        Node {
                            width: percent(100.0),
                            height: px(54.0),
                            padding: UiRect::new(px(24.0), px(20.0), px(0.0), px(0.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        BackgroundColor(if is_active {
                            palette::NAV_ITEM_ACTIVE_BG
                        } else {
                            Color::NONE
                        }),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            QuestNavAccent,
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
                            if is_active {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            },
                        ));

                        btn.spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(2.0),
                            ..default()
                        })
                        .with_children(|col| {
                            col.spawn((
                                QuestNavLabel,
                                Text::new(t(quest.title)),
                                make_font(15.0),
                                TextColor(if is_active {
                                    palette::TEXT_ACCENT
                                } else {
                                    palette::TEXT_PRIMARY
                                }),
                                text_shadow_default(),
                            ));
                            col.spawn((
                                Text::new(t(quest.category)),
                                make_font(11.0),
                                TextColor(palette::TEXT_MUTED),
                            ));
                        });

                        btn.spawn((
                            Text::new(">"),
                            make_font(14.0),
                            TextColor(palette::TEXT_MUTED),
                        ));
                    });
                }
            });

            // Right Main Panel: Selected Quest Details
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    height: percent(100.0),
                    padding: UiRect::all(px(32.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL2_BG),
            ))
            .with_children(|main| {
                for (idx, quest) in SAMPLE_QUESTS.iter().enumerate() {
                    main.spawn((
                        QuestDetailCard(idx),
                        Node {
                            width: px(620.0),
                            display: if idx == selected.0 {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            flex_direction: FlexDirection::Column,
                            row_gap: px(18.0),
                            ..default()
                        },
                    ))
                    .with_children(|detail| {
                        detail
                            .spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                row_gap: px(4.0),
                                padding: UiRect::bottom(px(14.0)),
                                border: UiRect::bottom(px(1.0)),
                                ..default()
                            })
                            .insert(BorderColor::all(palette::PANEL_BORDER_SUBTLE))
                            .with_children(|hdr| {
                                hdr.spawn((
                                    Text::new(t(quest.category)),
                                    make_font(12.0),
                                    TextColor(palette::TEXT_ACCENT),
                                ));
                                hdr.spawn((
                                    Text::new(t(quest.title)),
                                    make_font(22.0),
                                    TextColor(palette::TEXT_PRIMARY),
                                    text_shadow_default(),
                                ));
                            });

                        detail.spawn((
                            Text::new(t(quest.description)),
                            make_font(14.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        // Objectives Card
                        detail
                            .spawn((
                                Node {
                                    width: percent(100.0),
                                    padding: UiRect::all(px(16.0)),
                                    border: UiRect::all(px(1.0)),
                                    border_radius: BorderRadius::all(px(6.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(10.0),
                                    ..default()
                                },
                                BackgroundColor(palette::PANEL_INNER_BG),
                                BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                            ))
                            .with_children(|obj_box| {
                                obj_box.spawn((
                                    Text::new(t("menu.quests.objectives_header")),
                                    make_font(12.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                                for obj_key in quest.objectives {
                                    let obj_text = t(obj_key);
                                    let done = obj_text.starts_with("[x]");
                                    obj_box.spawn((
                                        Text::new(obj_text),
                                        make_font(14.0),
                                        TextColor(if done {
                                            palette::HEALTH_FILL
                                        } else {
                                            palette::TEXT_PRIMARY
                                        }),
                                    ));
                                }
                            });

                        // Rewards
                        detail
                            .spawn((
                                Node {
                                    width: percent(100.0),
                                    padding: UiRect::all(px(14.0)),
                                    border: UiRect::all(px(1.0)),
                                    border_radius: BorderRadius::all(px(6.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(palette::PANEL_INNER_BG),
                                BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                            ))
                            .with_children(|rew| {
                                rew.spawn((
                                    Text::new(t("menu.quests.rewards_header")),
                                    make_font(12.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                                rew.spawn((
                                    Text::new(t(quest.reward)),
                                    make_font(13.0),
                                    TextColor(palette::TEXT_ACCENT),
                                ));
                            });
                    });
                }
            });
        });
}

fn despawn_quests_menu(mut commands: Commands, query: Query<Entity, With<QuestsMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_quest_clicks(
    query: Query<(&Hovered, &QuestNavButton), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut selected: ResMut<SelectedQuestIndex>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    for (hovered, btn) in &query {
        if hovered.get() {
            selected.0 = btn.0;
        }
    }
}

fn update_quest_view(
    selected: Res<SelectedQuestIndex>,
    mut nav_query: Query<(&QuestNavButton, &Hovered, &mut BackgroundColor, &Children)>,
    mut accents_query: Query<&mut Visibility, With<QuestNavAccent>>,
    mut cards_query: Query<(&QuestDetailCard, &mut Node)>,
) {
    for (btn, hovered, mut bg, children) in &mut nav_query {
        let is_active = btn.0 == selected.0;
        let is_hovered = hovered.get();
        bg.0 = if is_active {
            palette::NAV_ITEM_ACTIVE_BG
        } else if is_hovered {
            palette::NAV_ITEM_HOVER_BG
        } else {
            Color::NONE
        };
        for child in children.iter() {
            if let Ok(mut vis) = accents_query.get_mut(child) {
                *vis = if is_active || is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }

    if !selected.is_changed() {
        return;
    }

    for (card, mut node) in &mut cards_query {
        node.display = if card.0 == selected.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}
