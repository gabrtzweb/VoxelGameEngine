use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::Button,
};

use crate::{
    core::{AppFont, text_shadow_default},
    player::{CharacterRace, SelectedRace},
};

use super::{GameSettings, MenuState, menu_bar::MENU_BAR_HEIGHT, theme::palette};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkillTreeTab {
    #[default]
    Combat,
    Gathering,
    Crafting,
    Exploration,
    Racial,
}

impl SkillTreeTab {
    const ALL: [Self; 5] = [
        Self::Combat,
        Self::Gathering,
        Self::Crafting,
        Self::Exploration,
        Self::Racial,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Combat => "COMBAT",
            Self::Gathering => "GATHERING",
            Self::Crafting => "CRAFTING",
            Self::Exploration => "EXPLORATION",
            Self::Racial => "RACIAL",
        }
    }

    fn nodes(self) -> [(&'static str, &'static str, &'static str); 6] {
        match self {
            Self::Combat => [
                ("Steady Grip", "I", "+8% melee swing stability and reduced recoil."),
                ("Iron Guard", "I", "+10% block mitigation when holding off-hand gear."),
                ("Precision Strike", "II", "Critical hits deal +15% bonus damage."),
                ("Second Wind", "II", "Recover stamina faster after dodging or sprinting."),
                ("Warlord Resolve", "III", "+15% max health and stagger resistance."),
                ("Executioner", "III", "Bonus damage against weakened foes."),
            ],
            Self::Gathering => [
                ("Keen Pick", "I", "+10% mining speed on stone and mineral strata."),
                ("Woodsman", "I", "+12% axe efficiency when felling timber."),
                ("Green Thumb", "II", "Increased yield when harvesting mature crops."),
                ("Prospector", "II", "Higher chance of bonus ore shards from veins."),
                ("Master Harvester", "III", "Rare seeds and botanical specimens drop more often."),
                ("Deep Delver", "III", "Reduced tool wear in subterranean layers."),
            ],
            Self::Crafting => [
                ("Neat Joinery", "I", "Slightly lower material waste on timber recipes."),
                ("Stonecutter", "I", "Unlock decorative carved masonry patterns."),
                ("Smelter", "II", "Faster smelting and fuel efficiency in furnaces."),
                ("Artisan", "II", "Crafted tools gain +15% base durability."),
                ("Masterwork", "III", "Chance to forge superior quality equipment."),
                ("Alchemist", "III", "Longer duration on brewed tonics and salves."),
            ],
            Self::Exploration => [
                ("Lightfooted", "I", "+6% base movement speed on natural terrain."),
                ("Pathfinder", "I", "Wider map reveal radius while exploring."),
                ("Climber", "II", "Reduced stamina drain while jumping and climbing."),
                ("Weatherproof", "II", "Better resilience in storms and cold biomes."),
                ("Wayfarer", "III", "Faster stamina recovery when traveling outdoors."),
                ("Cartographer", "III", "Reveals landmarks and geological outcrops on map."),
            ],
            Self::Racial => [
                ("Ancestral Focus", "I", "Enhances your race's signature innate aptitude."),
                ("Bloodline Vigor", "I", "Improves core vitality and environmental comfort."),
                ("Heritage Craft", "II", "Bonus efficiency with traditional racial tools."),
                ("Kinship Aura", "II", "Subtle passive bonus when exploring native biomes."),
                ("Paragon Form", "III", "Major boost to primary racial attribute."),
                ("Legacy Awakening", "III", "Unlocks ultimate racial passive mastery."),
            ],
        }
    }
}

#[derive(Component)]
struct SkillsMenuRoot;

#[derive(Component, Clone, Copy)]
struct SkillTabButton(SkillTreeTab);

#[derive(Component)]
struct SkillTabLabel;

#[derive(Component)]
struct SkillTabUnderline;

#[derive(Component, Clone, Copy)]
struct SkillTreeSection(SkillTreeTab);

pub struct SkillsMenuPlugin;

impl Plugin for SkillsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkillTreeTab>()
            .add_systems(OnEnter(MenuState::Skills), spawn_skills_menu)
            .add_systems(OnExit(MenuState::Skills), despawn_skills_menu)
            .add_systems(
                Update,
                (handle_skill_tab_clicks, update_skill_tab_view)
                    .run_if(in_state(MenuState::Skills)),
            );
    }
}

fn race_info(race: CharacterRace) -> (&'static str, [(&'static str, &'static str); 2]) {
    match race {
        CharacterRace::Human => (
            "Human",
            [
                ("Adaptable", "+5% XP gain across all skill trees."),
                ("Versatile", "Balanced attributes with no terrain penalties."),
            ],
        ),
        CharacterRace::Elf => (
            "Elf",
            [
                ("Forest Stride", "+8% movement speed in wooded biomes."),
                ("Keen Sight", "Wider exploration and foraging awareness."),
            ],
        ),
        CharacterRace::Dwarf => (
            "Dwarf",
            [
                ("Stoneborn", "+12% mining speed in deep stone strata."),
                ("Stout Frame", "+15% resistance to knockback and cold."),
            ],
        ),
        CharacterRace::Orc => (
            "Orc",
            [
                ("Iron Blood", "+15% base health and heavy melee power."),
                ("Relentless", "Faster stamina recovery in combat."),
            ],
        ),
        _ => (
            "Wanderer",
            [
                ("Resilient", "+5% stamina regeneration while exploring."),
                ("Innate Gift", "Unique racial talents unlock as you level."),
            ],
        ),
    }
}

fn race_icon_path(race: CharacterRace) -> &'static str {
    match race {
        CharacterRace::Human => "textures/interfaces/icons/race_human.png",
        CharacterRace::Elf => "textures/interfaces/icons/race_elf.png",
        CharacterRace::Dwarf => "textures/interfaces/icons/race_dwarf.png",
        CharacterRace::Orc => "textures/interfaces/icons/race_orc.png",
        CharacterRace::Halfling => "textures/interfaces/icons/race_halfling.png",
        CharacterRace::Drow => "textures/interfaces/icons/race_drow.png",
        CharacterRace::Tiefling => "textures/interfaces/icons/race_tiefling.png",
        CharacterRace::Dragonborn => "textures/interfaces/icons/race_dragonborn.png",
    }
}

fn spawn_skills_menu(
    mut commands: Commands,
    active_tab: Res<SkillTreeTab>,
    selected_race: Option<Res<SelectedRace>>,
    asset_server: Res<AssetServer>,
    app_font: Option<Res<AppFont>>,
    game_settings: Option<Res<GameSettings>>,
) {
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

    let race = selected_race.map_or(CharacterRace::Human, |r| r.0);
    let (race_name, traits) = race_info(race);
    let race_icon = asset_server.load(race_icon_path(race));
    let player_name = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "Adventurer".to_string());
    let dimming_alpha = game_settings.map_or(0.65, |s| s.background_dimming.alpha());

    commands
        .spawn((
            SkillsMenuRoot,
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
            // Left Column: Character & Race Sheet (standardized 310px width)
            root.spawn((
                Node {
                    width: px(palette::DRAWER_COL1_WIDTH),
                    min_width: px(palette::DRAWER_COL1_WIDTH),
                    flex_shrink: 0.0,
                    height: percent(100.0),
                    border: UiRect::right(px(1.5)),
                    padding: UiRect::all(px(24.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(18.0),
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL1_BG),
                BorderColor::all(palette::DRAWER_DIVIDER),
            ))
            .with_children(|sheet| {
                // Player Name, Race Icon & Level Header
                sheet
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8.0),
                        padding: UiRect::bottom(px(14.0)),
                        border: UiRect::bottom(px(1.0)),
                        ..default()
                    })
                    .insert(BorderColor::all(palette::PANEL_BORDER_SUBTLE))
                    .with_children(|hdr| {
                        hdr.spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn(Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(10.0),
                                ..default()
                            })
                            .with_children(|id| {
                                id.spawn((
                                    ImageNode {
                                        image: race_icon,
                                        ..default()
                                    },
                                    Node {
                                        width: px(32.0),
                                        height: px(32.0),
                                        ..default()
                                    },
                                ));
                                id.spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(2.0),
                                    ..default()
                                })
                                .with_children(|col| {
                                    col.spawn((
                                        Text::new(player_name),
                                        make_font(16.0),
                                        TextColor(palette::TEXT_PRIMARY),
                                        text_shadow_default(),
                                    ));
                                    col.spawn((
                                        Text::new(race_name.to_uppercase()),
                                        make_font(12.0),
                                        TextColor(palette::TEXT_ACCENT),
                                    ));
                                });
                            });

                            row.spawn((
                                Text::new("Lv. 1"),
                                make_font(14.0),
                                TextColor(palette::TEXT_PRIMARY),
                            ));
                        });

                        // XP Bar
                        hdr.spawn((
                            Node {
                                width: percent(100.0),
                                height: px(6.0),
                                margin: UiRect::top(px(4.0)),
                                border_radius: BorderRadius::all(px(3.0)),
                                ..default()
                            },
                            BackgroundColor(palette::SLOT_EMPTY_BG),
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                Node {
                                    width: percent(45.0),
                                    height: percent(100.0),
                                    border_radius: BorderRadius::all(px(3.0)),
                                    ..default()
                                },
                                BackgroundColor(palette::HEALTH_FILL),
                            ));
                        });
                    });

                // Core Attributes
                sheet
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8.0),
                        ..default()
                    })
                    .with_children(|attrs| {
                        attrs.spawn((
                            Text::new("ATTRIBUTES"),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        let stats = [
                            ("Health", "100 / 100"),
                            ("Stamina", "100 / 100"),
                            ("Defense", "5"),
                            ("Move Speed", "100%"),
                            ("Mining Speed", "100%"),
                            ("Carry Slots", "36"),
                        ];

                        for (k, v) in stats {
                            attrs
                                .spawn(Node {
                                    width: percent(100.0),
                                    height: px(28.0),
                                    padding: UiRect::horizontal(px(10.0)),
                                    border_radius: BorderRadius::all(px(4.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .insert(BackgroundColor(palette::PANEL_INNER_BG))
                                .with_children(|row| {
                                    row.spawn((
                                        Text::new(k),
                                        make_font(13.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));
                                    row.spawn((
                                        Text::new(v),
                                        make_font(13.0),
                                        TextColor(palette::TEXT_PRIMARY),
                                    ));
                                });
                        }
                    });

                // Racial Traits
                sheet
                    .spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8.0),
                        margin: UiRect::top(px(6.0)),
                        ..default()
                    })
                    .with_children(|tr| {
                        tr.spawn((
                            Text::new("RACIAL TRAITS"),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        for (title, desc) in traits {
                            tr.spawn((
                                Node {
                                    width: percent(100.0),
                                    padding: UiRect::all(px(12.0)),
                                    border: UiRect::all(px(1.0)),
                                    border_radius: BorderRadius::all(px(5.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(4.0),
                                    ..default()
                                },
                                BackgroundColor(palette::PANEL_INNER_BG),
                                BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                            ))
                            .with_children(|card| {
                                card.spawn((
                                    Text::new(title),
                                    make_font(14.0),
                                    TextColor(palette::TEXT_ACCENT),
                                ));
                                card.spawn((
                                    Text::new(desc),
                                    make_font(12.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                            });
                        }
                    });
            });

            // Right Main Panel: Skill Trees
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    height: percent(100.0),
                    padding: UiRect::all(px(28.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(22.0),
                    ..default()
                },
                BackgroundColor(palette::DRAWER_COL2_BG),
            ))
            .with_children(|main| {
                // Sub-tabs bar + Skill Points counter
                main.spawn(Node {
                    width: percent(100.0),
                    height: px(44.0),
                    border: UiRect::bottom(px(1.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .insert(BorderColor::all(palette::PANEL_BORDER_SUBTLE))
                .with_children(|header| {
                    header
                        .spawn(Node {
                            height: percent(100.0),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(36.0),
                            ..default()
                        })
                        .with_children(|tabs| {
                            for tab in SkillTreeTab::ALL {
                                let is_active = tab == *active_tab;
                                tabs.spawn((
                                    Button,
                                    Hovered::default(),
                                    SkillTabButton(tab),
                                    Node {
                                        height: percent(100.0),
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        SkillTabLabel,
                                        Text::new(tab.label()),
                                        make_font(14.0),
                                        TextColor(if is_active {
                                            palette::TEXT_ACCENT
                                        } else {
                                            palette::TEXT_MUTED
                                        }),
                                        text_shadow_default(),
                                    ));
                                    btn.spawn((
                                        SkillTabUnderline,
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(0.0),
                                            right: px(0.0),
                                            bottom: px(0.0),
                                            height: px(2.5),
                                            border_radius: BorderRadius::all(px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(palette::ACCENT_LINE),
                                        if is_active {
                                            Visibility::Visible
                                        } else {
                                            Visibility::Hidden
                                        },
                                    ));
                                });
                            }
                        });

                    header.spawn((
                        Text::new("Skill Points: 0"),
                        make_font(14.0),
                        TextColor(palette::TEXT_ACCENT),
                        text_shadow_default(),
                    ));
                });

                // Skill Nodes Grid per Tab
                for tab in SkillTreeTab::ALL {
                    main.spawn((
                        SkillTreeSection(tab),
                        Node {
                            width: percent(100.0),
                            display: if tab == *active_tab {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(16.0),
                            row_gap: px(16.0),
                            ..default()
                        },
                    ))
                    .with_children(|grid| {
                        for (title, tier, desc) in tab.nodes() {
                            grid.spawn((
                                Node {
                                    width: px(270.0),
                                    padding: UiRect::all(px(16.0)),
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::all(px(6.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(8.0),
                                    ..default()
                                },
                                BackgroundColor(palette::PANEL_INNER_BG),
                                BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                            ))
                            .with_children(|card| {
                                card.spawn(Node {
                                    width: percent(100.0),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .with_children(|top| {
                                    top.spawn((
                                        Text::new(title),
                                        make_font(15.0),
                                        TextColor(palette::TEXT_PRIMARY),
                                        text_shadow_default(),
                                    ));
                                    top.spawn((
                                        Text::new(format!("Tier {tier}  •  0/3")),
                                        make_font(12.0),
                                        TextColor(palette::TEXT_ACCENT),
                                    ));
                                });

                                card.spawn((
                                    Text::new(desc),
                                    make_font(13.0),
                                    TextColor(palette::TEXT_MUTED),
                                ));
                            });
                        }
                    });
                }
            });
        });
}

fn despawn_skills_menu(mut commands: Commands, query: Query<Entity, With<SkillsMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_skill_tab_clicks(
    query: Query<(&Hovered, &SkillTabButton), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut active_tab: ResMut<SkillTreeTab>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    for (hovered, tab_btn) in &query {
        if hovered.get() {
            *active_tab = tab_btn.0;
        }
    }
}

fn update_skill_tab_view(
    active_tab: Res<SkillTreeTab>,
    tabs_query: Query<(&SkillTabButton, &Hovered, &Children)>,
    mut labels_query: Query<&mut TextColor, With<SkillTabLabel>>,
    mut underlines_query: Query<&mut Visibility, With<SkillTabUnderline>>,
    mut sections_query: Query<(&SkillTreeSection, &mut Node)>,
) {
    for (tab_btn, hovered, children) in &tabs_query {
        let is_active = tab_btn.0 == *active_tab;
        for child in children.iter() {
            if let Ok(mut color) = labels_query.get_mut(child) {
                color.0 = if is_active {
                    palette::TEXT_ACCENT
                } else if hovered.get() {
                    palette::TEXT_PRIMARY
                } else {
                    palette::TEXT_MUTED
                };
            }
            if let Ok(mut vis) = underlines_query.get_mut(child) {
                *vis = if is_active {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }

    if !active_tab.is_changed() {
        return;
    }

    for (sec, mut node) in &mut sections_query {
        node.display = if sec.0 == *active_tab {
            Display::Flex
        } else {
            Display::None
        };
    }
}
