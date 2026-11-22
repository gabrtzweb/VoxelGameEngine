use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    ui::Pressed,
    ui_widgets::Button,
};

use crate::{
    audio::UiSound,
    core::{AppFont, Localization, text_shadow_default},
    player::{CharacterRace, CharacterReady, CharacterSubrace, SelectedRace, SelectedSubrace},
};

use super::{
    MenuBackdrop, MenuState, ThemedButton, main_menu::MainMenuPhase, theme::palette,
};

#[derive(Component)]
pub struct RaceSelectionRoot;

#[derive(Component)]
struct RaceColumnsContainer;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct CoreRaceCardButton(pub CharacterRace);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct SubraceCardButton(pub CharacterSubrace);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum RaceMenuAction {
    Back,
    Confirm,
}

#[derive(Component)]
struct CoreRaceCardLabel;

#[derive(Component)]
struct SubraceCardLabel;

#[derive(Component)]
struct RaceActionButtonLabel;

pub struct RaceSelectionMenuPlugin;

impl Plugin for RaceSelectionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::RaceSelection), spawn_race_selection_menu)
            .add_systems(
                OnExit(MenuState::RaceSelection),
                despawn_race_selection_menu,
            )
            .add_systems(
                Update,
                (
                    handle_race_selection_buttons,
                    rebuild_race_selection_columns,
                    update_core_race_card_hover,
                    update_subrace_card_hover,
                    update_race_action_button_hover,
                )
                    .chain()
                    .run_if(in_state(MenuState::RaceSelection)),
            );
    }
}

fn make_text_font(font_handle: Option<&FontSource>, size: f32) -> TextFont {
    let mut tf = TextFont {
        font_size: FontSize::Px(size),
        ..default()
    };
    if let Some(font) = font_handle {
        tf.font = font.clone();
    }
    tf
}

fn spawn_race_selection_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loc: Option<Res<Localization>>,
    app_font: Option<Res<AppFont>>,
    selected_subrace: Option<Res<SelectedSubrace>>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source());
    let bg_texture = asset_server.load("textures/interfaces/main_menu_bg.png");
    let current_subrace = selected_subrace.map_or(CharacterSubrace::default(), |s| s.0);

    let title_str = loc
        .as_ref()
        .map_or("CHOOSE YOUR RACE", |l| l.t("menu.race_selection.title"));
    let subtitle_str = loc.as_ref().map_or(
        "Select your lineage and subrace to shape your attributes and traits",
        |l| l.t("menu.race_selection.subtitle"),
    );
    let back_str = loc
        .as_ref()
        .map_or("Back", |l| l.t("menu.race_selection.back"));
    let confirm_str = loc
        .as_ref()
        .map_or("Confirm", |l| l.t("menu.race_selection.confirm"));

    commands
        .spawn((
            RaceSelectionRoot,
            MenuBackdrop,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                right: px(0.0),
                bottom: px(0.0),
                width: percent(100.0),
                height: percent(100.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(300),
        ))
        .with_children(|backdrop| {
            backdrop.spawn((
                Pickable::IGNORE,
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
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.58)),
                ZIndex(1),
            ));

            backdrop
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        justify_content: JustifyContent::SpaceBetween,
                        width: px(1080.0),
                        max_width: percent(94.0),
                        padding: UiRect::axes(px(28.0), px(24.0)),
                        border: UiRect::all(px(1.5)),
                        border_radius: BorderRadius::all(px(8.0)),
                        row_gap: px(18.0),
                        ..default()
                    },
                    BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                    BackgroundColor(palette::DRAWER_COL1_BG),
                    ZIndex(10),
                ))
                .with_children(|card| {
                    // Header
                    card.spawn((
                        Pickable::IGNORE,
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(4.0),
                            padding: UiRect::bottom(px(14.0)),
                            border: UiRect::bottom(px(1.0)),
                            ..default()
                        },
                        BorderColor::all(palette::DRAWER_DIVIDER),
                    ))
                    .with_children(|header| {
                        header.spawn((
                            Pickable::IGNORE,
                            Text::new(title_str),
                            make_text_font(font_handle.as_ref(), 24.0),
                            TextColor(palette::TEXT_PRIMARY),
                            text_shadow_default(),
                        ));

                        header.spawn((
                            Pickable::IGNORE,
                            Text::new(subtitle_str),
                            make_text_font(font_handle.as_ref(), 13.0),
                            TextColor(palette::TEXT_MUTED),
                            text_shadow_default(),
                        ));
                    });

                    // 3-Column Body
                    card.spawn((
                        RaceColumnsContainer,
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Stretch,
                            width: percent(100.0),
                            column_gap: px(18.0),
                            min_height: px(420.0),
                            ..default()
                        },
                    ))
                    .with_children(|cols| {
                        spawn_three_columns(
                            cols,
                            current_subrace,
                            &asset_server,
                            loc.as_deref(),
                            font_handle.as_ref(),
                        );
                    });

                    // Footer: Back & Confirm
                    card.spawn((
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::top(px(14.0)),
                            border: UiRect::top(px(1.0)),
                            ..default()
                        },
                        BorderColor::all(palette::DRAWER_DIVIDER),
                    ))
                    .with_children(|bottom| {
                        let actions = [
                            (RaceMenuAction::Back, back_str),
                            (RaceMenuAction::Confirm, confirm_str),
                        ];

                        for (action, label) in actions {
                            let theme = ThemedButton::default();
                            bottom
                                .spawn((
                                    Button,
                                    Hovered::default(),
                                    action,
                                    theme,
                                    BackgroundColor(theme.normal_bg),
                                    BorderColor::all(theme.normal_border),
                                    Node {
                                        width: px(200.0),
                                        height: px(44.0),
                                        display: Display::Flex,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        border: UiRect::all(px(1.5)),
                                        border_radius: BorderRadius::all(px(5.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Pickable::IGNORE,
                                        RaceActionButtonLabel,
                                        Text::new(label),
                                        make_text_font(font_handle.as_ref(), 16.0),
                                        TextColor(palette::TEXT_PRIMARY),
                                        text_shadow_default(),
                                    ));
                                });
                        }
                    });
                });
        });
}

fn spawn_three_columns(
    cols: &mut ChildSpawnerCommands,
    current_subrace: CharacterSubrace,
    asset_server: &AssetServer,
    loc: Option<&Localization>,
    font_handle: Option<&FontSource>,
) {
    let active_race = current_subrace.parent_race();

    let select_race_lbl = loc.map_or("CORE RACE", |l| l.t("menu.race_selection.select_race"));
    let select_subrace_lbl = loc.map_or("SUBRACE", |l| l.t("menu.race_selection.select_subrace"));
    let attrs_lbl = loc.map_or("ATTRIBUTES", |l| l.t("menu.race_selection.attributes"));
    let traits_lbl = loc.map_or("RACIAL TRAITS", |l| l.t("menu.race_selection.traits"));

    // Column 1: Core Races (Human, Elf, Dwarf)
    cols.spawn((
        Node {
            width: px(230.0),
            min_width: px(230.0),
            flex_shrink: 0.0,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            padding: UiRect::right(px(16.0)),
            border: UiRect::right(px(1.0)),
            ..default()
        },
        BorderColor::all(palette::DRAWER_DIVIDER),
    ))
    .with_children(|col1| {
        col1.spawn((
            Pickable::IGNORE,
            Text::new(select_race_lbl.to_uppercase()),
            make_text_font(font_handle, 12.0),
            TextColor(palette::TEXT_MUTED),
        ));

        for race in CharacterRace::PLAYABLE {
            let is_selected = race == active_race;
            let icon = asset_server.load(race.icon_asset_path());
            let name = race.display_name(loc);

            let (bg, border) = if is_selected {
                (palette::BTN_HOVER_BG, palette::ACCENT_LINE)
            } else {
                (palette::PANEL_INNER_BG, palette::PANEL_BORDER_SUBTLE)
            };

            col1.spawn((
                Button,
                Hovered::default(),
                CoreRaceCardButton(race),
                Node {
                    width: percent(100.0),
                    padding: UiRect::axes(px(14.0), px(12.0)),
                    border: UiRect::all(px(1.5)),
                    border_radius: BorderRadius::all(px(6.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(12.0),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(border),
            ))
            .with_children(|card| {
                card.spawn((
                    Pickable::IGNORE,
                    ImageNode {
                        image: icon,
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Node {
                        width: px(36.0),
                        height: px(36.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                ));

                card.spawn((
                    Pickable::IGNORE,
                    CoreRaceCardLabel,
                    Text::new(name),
                    make_text_font(font_handle, 15.0),
                    TextColor(if is_selected {
                        palette::TEXT_ACCENT
                    } else {
                        palette::TEXT_PRIMARY
                    }),
                    text_shadow_default(),
                ));
            });
        }
    });

    // Column 2: Active Core Race Lore + Subraces
    cols.spawn((
        Node {
            width: px(290.0),
            min_width: px(290.0),
            flex_shrink: 0.0,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(12.0),
            padding: UiRect::right(px(16.0)),
            border: UiRect::right(px(1.0)),
            ..default()
        },
        BorderColor::all(palette::DRAWER_DIVIDER),
    ))
    .with_children(|col2| {
        let race_name = active_race.display_name(loc);
        let race_desc = loc.map_or("", |l| l.t(active_race.desc_key()));

        // Core race lore card
        col2.spawn((
            Pickable::IGNORE,
            Node {
                width: percent(100.0),
                padding: UiRect::all(px(12.0)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(6.0)),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: px(6.0),
                ..default()
            },
            BackgroundColor(palette::PANEL_INNER_BG),
            BorderColor::all(palette::PANEL_BORDER_SUBTLE),
        ))
        .with_children(|lore| {
            lore.spawn((
                Pickable::IGNORE,
                Text::new(race_name.to_uppercase()),
                make_text_font(font_handle, 13.0),
                TextColor(palette::TEXT_ACCENT),
            ));
            lore.spawn((
                Pickable::IGNORE,
                Text::new(race_desc),
                make_text_font(font_handle, 12.0),
                TextColor(palette::TEXT_MUTED),
            ));
        });

        col2.spawn((
            Pickable::IGNORE,
            Text::new(select_subrace_lbl.to_uppercase()),
            make_text_font(font_handle, 12.0),
            TextColor(palette::TEXT_MUTED),
            Node {
                margin: UiRect::top(px(4.0)),
                ..default()
            },
        ));

        for subrace in active_race.subraces() {
            let is_selected = subrace == current_subrace;
            let icon = asset_server.load(subrace.icon_asset_path());
            let name = subrace.display_name(loc);

            let (bg, border) = if is_selected {
                (palette::BTN_HOVER_BG, palette::ACCENT_LINE)
            } else {
                (palette::PANEL_INNER_BG, palette::PANEL_BORDER_SUBTLE)
            };

            col2.spawn((
                Button,
                Hovered::default(),
                SubraceCardButton(subrace),
                Node {
                    width: percent(100.0),
                    padding: UiRect::axes(px(14.0), px(12.0)),
                    border: UiRect::all(px(1.5)),
                    border_radius: BorderRadius::all(px(6.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(12.0),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(border),
            ))
            .with_children(|card| {
                card.spawn((
                    Pickable::IGNORE,
                    ImageNode {
                        image: icon,
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Node {
                        width: px(40.0),
                        height: px(40.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                ));

                card.spawn((
                    Pickable::IGNORE,
                    SubraceCardLabel,
                    Text::new(name),
                    make_text_font(font_handle, 15.0),
                    TextColor(if is_selected {
                        palette::TEXT_ACCENT
                    } else {
                        palette::TEXT_PRIMARY
                    }),
                    text_shadow_default(),
                ));
            });
        }
    });

    // Column 3: Selected Subrace Details, Attributes & Traits
    cols.spawn(Node {
        flex_grow: 1.0,
        min_width: px(360.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        row_gap: px(14.0),
        ..default()
    })
    .with_children(|col3| {
        let sub_icon = asset_server.load(current_subrace.icon_asset_path());
        let sub_name = current_subrace.display_name(loc);
        let parent_name = active_race.display_name(loc);
        let sub_desc = loc.map_or("", |l| l.t(current_subrace.desc_key()));
        let attrs = current_subrace.attributes();

        // Identity header
        col3.spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(14.0),
            ..default()
        })
        .with_children(|hdr| {
            hdr.spawn((
                Pickable::IGNORE,
                ImageNode {
                    image: sub_icon,
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    width: px(48.0),
                    height: px(48.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));

            hdr.spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: px(2.0),
                ..default()
            })
            .with_children(|title_col| {
                title_col.spawn((
                    Pickable::IGNORE,
                    Text::new(sub_name),
                    make_text_font(font_handle, 20.0),
                    TextColor(palette::TEXT_ACCENT),
                    text_shadow_default(),
                ));
                title_col.spawn((
                    Pickable::IGNORE,
                    Text::new(parent_name.to_uppercase()),
                    make_text_font(font_handle, 12.0),
                    TextColor(palette::TEXT_MUTED),
                ));
            });
        });

        // Subrace description
        col3.spawn((
            Pickable::IGNORE,
            Text::new(sub_desc),
            make_text_font(font_handle, 13.0),
            TextColor(palette::TEXT_PRIMARY),
        ));

        // Attributes Grid (2 columns)
        col3.spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(6.0),
            ..default()
        })
        .with_children(|attr_sec| {
            attr_sec.spawn((
                Pickable::IGNORE,
                Text::new(attrs_lbl.to_uppercase()),
                make_text_font(font_handle, 12.0),
                TextColor(palette::TEXT_MUTED),
            ));

            let stat_entries = [
                (
                    loc.map_or("Health", |l| l.t("attr.health")),
                    format!("{}", attrs.max_health),
                    attrs.max_health > 100,
                    attrs.max_health < 100,
                ),
                (
                    loc.map_or("Stamina", |l| l.t("attr.stamina")),
                    format!("{}", attrs.max_stamina),
                    attrs.max_stamina > 105,
                    attrs.max_stamina < 105,
                ),
                (
                    loc.map_or("Defense", |l| l.t("attr.defense")),
                    format!("{}", attrs.defense),
                    attrs.defense > 5,
                    attrs.defense < 5,
                ),
                (
                    loc.map_or("Move Speed", |l| l.t("attr.move_speed")),
                    format!("{}%", (attrs.move_speed_multiplier * 100.0).round() as i32),
                    attrs.move_speed_multiplier > 1.001,
                    attrs.move_speed_multiplier < 0.999,
                ),
                (
                    loc.map_or("Jump Height", |l| l.t("attr.jump_height")),
                    format!("{:.2}b", attrs.jump_height_blocks),
                    attrs.jump_height_blocks > 1.35,
                    attrs.jump_height_blocks < 1.20,
                ),
                (
                    loc.map_or("Step Noise", |l| l.t("attr.footstep_noise")),
                    format!(
                        "{}%",
                        (attrs.footstep_noise_multiplier * 100.0).round() as i32
                    ),
                    attrs.footstep_noise_multiplier < 0.99,
                    attrs.footstep_noise_multiplier > 1.01,
                ),
                (
                    loc.map_or("Swim Speed", |l| l.t("attr.swim_speed")),
                    format!("{}%", (attrs.swim_speed_multiplier * 100.0).round() as i32),
                    attrs.swim_speed_multiplier > 1.001,
                    attrs.swim_speed_multiplier < 0.999,
                ),
                (
                    loc.map_or("Mining Speed", |l| l.t("attr.mining_speed")),
                    format!(
                        "{}%",
                        (attrs.mining_speed_multiplier * 100.0).round() as i32
                    ),
                    attrs.mining_speed_multiplier > 1.001,
                    attrs.mining_speed_multiplier < 0.999,
                ),
                (
                    loc.map_or("Mining Reach", |l| l.t("attr.reach_bonus")),
                    format!("+{:.2}b", attrs.reach_bonus_blocks),
                    attrs.reach_bonus_blocks > 0.01,
                    false,
                ),
                (
                    loc.map_or("Cave Vision", |l| l.t("attr.cave_vision")),
                    format!("+{}%", (attrs.cave_vision_bonus * 100.0).round() as i32),
                    attrs.cave_vision_bonus > 0.01,
                    false,
                ),
                (
                    loc.map_or("Fall Resistance", |l| l.t("attr.fall_resistance")),
                    format!(
                        "+{}%",
                        (attrs.fall_damage_reduction * 100.0).round() as i32
                    ),
                    attrs.fall_damage_reduction > 0.01,
                    false,
                ),
                (
                    loc.map_or("Knockback Res.", |l| l.t("attr.knockback_res")),
                    format!("+{}%", (attrs.knockback_resistance * 100.0).round() as i32),
                    attrs.knockback_resistance > 0.01,
                    false,
                ),
                (
                    loc.map_or("Cold Resistance", |l| l.t("attr.cold_res")),
                    format!("+{}%", (attrs.cold_resistance * 100.0).round() as i32),
                    attrs.cold_resistance > 0.01,
                    false,
                ),
                (
                    loc.map_or("Heat Resistance", |l| l.t("attr.heat_res")),
                    format!("+{}%", (attrs.heat_resistance * 100.0).round() as i32),
                    attrs.heat_resistance > 0.01,
                    false,
                ),
                (
                    loc.map_or("XP Gain", |l| l.t("attr.xp_gain")),
                    format!(
                        "+{}%",
                        ((attrs.xp_gain_multiplier - 1.0) * 100.0).round() as i32
                    ),
                    attrs.xp_gain_multiplier > 1.001,
                    false,
                ),
                (
                    loc.map_or("Inventory Slots", |l| l.t("attr.inventory_slots")),
                    format!("{} / 36", attrs.carry_slots),
                    attrs.carry_slots > 27,
                    false,
                ),
            ];

            attr_sec
                .spawn(Node {
                    display: Display::Grid,
                    grid_template_columns: vec![GridTrack::fr(1.0), GridTrack::fr(1.0)],
                    column_gap: px(8.0),
                    row_gap: px(6.0),
                    width: percent(100.0),
                    ..default()
                })
                .with_children(|grid| {
                    for (label, value, is_bonus, is_penalty) in stat_entries {
                        let val_color = if is_bonus {
                            palette::TEXT_ACCENT
                        } else if is_penalty {
                            Color::srgb(0.88, 0.56, 0.50)
                        } else {
                            palette::TEXT_PRIMARY
                        };

                        grid.spawn((
                            Pickable::IGNORE,
                            Node {
                                width: percent(100.0),
                                height: px(28.0),
                                padding: UiRect::horizontal(px(10.0)),
                                border_radius: BorderRadius::all(px(4.0)),
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(palette::PANEL_INNER_BG),
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Pickable::IGNORE,
                                Text::new(label),
                                make_text_font(font_handle, 12.0),
                                TextColor(palette::TEXT_MUTED),
                            ));
                            row.spawn((
                                Pickable::IGNORE,
                                Text::new(value),
                                make_text_font(font_handle, 13.0),
                                TextColor(val_color),
                            ));
                        });
                    }
                });
        });

        // Racial Traits
        col3.spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(6.0),
            ..default()
        })
        .with_children(|trait_sec| {
            trait_sec.spawn((
                Pickable::IGNORE,
                Text::new(traits_lbl.to_uppercase()),
                make_text_font(font_handle, 12.0),
                TextColor(palette::TEXT_MUTED),
            ));

            for (name_key, desc_key) in current_subrace.trait_keys() {
                let t_name = loc.map_or(name_key, |l| l.t(name_key));
                let t_desc = loc.map_or(desc_key, |l| l.t(desc_key));

                trait_sec
                    .spawn((
                        Pickable::IGNORE,
                        Node {
                            width: percent(100.0),
                            padding: UiRect::axes(px(12.0), px(9.0)),
                            border: UiRect::all(px(1.0)),
                            border_radius: BorderRadius::all(px(5.0)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(3.0),
                            ..default()
                        },
                        BackgroundColor(palette::PANEL_INNER_BG),
                        BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Pickable::IGNORE,
                            Text::new(t_name),
                            make_text_font(font_handle, 13.0),
                            TextColor(palette::TEXT_ACCENT),
                        ));
                        card.spawn((
                            Pickable::IGNORE,
                            Text::new(t_desc),
                            make_text_font(font_handle, 12.0),
                            TextColor(palette::TEXT_MUTED),
                        ));
                    });
            }
        });
    });
}

fn rebuild_race_selection_columns(
    mut commands: Commands,
    selected_subrace: Res<SelectedSubrace>,
    asset_server: Res<AssetServer>,
    loc: Option<Res<Localization>>,
    app_font: Option<Res<AppFont>>,
    container_query: Query<(Entity, Option<&Children>), With<RaceColumnsContainer>>,
) {
    if !selected_subrace.is_changed() {
        return;
    }

    let font_handle = app_font.as_ref().map(|f| f.source());
    for (container, children) in &container_query {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        commands.entity(container).with_children(|cols| {
            spawn_three_columns(
                cols,
                selected_subrace.0,
                &asset_server,
                loc.as_deref(),
                font_handle.as_ref(),
            );
        });
    }
}

fn despawn_race_selection_menu(
    mut commands: Commands,
    query: Query<Entity, With<RaceSelectionRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_race_selection_buttons(
    core_race_buttons: Query<(&Hovered, &CoreRaceCardButton), With<Button>>,
    subrace_buttons: Query<(&Hovered, &SubraceCardButton), With<Button>>,
    action_buttons: Query<(&Hovered, &RaceMenuAction), With<Button>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected_race: ResMut<SelectedRace>,
    mut selected_subrace: ResMut<SelectedSubrace>,
    mut character_ready: ResMut<CharacterReady>,
    mut main_menu_phase: ResMut<MainMenuPhase>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut commands: Commands,
) {
    let left_clicked = mouse.just_pressed(MouseButton::Left);
    let escape_pressed = keyboard.just_pressed(KeyCode::Escape);
    let enter_pressed =
        keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter);

    if escape_pressed {
        commands.trigger(UiSound::Click);
        *main_menu_phase = MainMenuPhase::ActionMenu;
        next_state.set(MenuState::MainMenu);
        return;
    }

    if enter_pressed {
        commands.trigger(UiSound::Click);
        selected_race.0 = selected_subrace.0.rig_race();
        character_ready.0 = true;
        *main_menu_phase = MainMenuPhase::ActionMenu;
        next_state.set(MenuState::MainMenu);
        return;
    }

    if !left_clicked {
        return;
    }

    for (hovered, core_btn) in &core_race_buttons {
        if hovered.get() {
            let default_sub = core_btn.0.default_subrace();
            if selected_subrace.0.parent_race() != core_btn.0 {
                selected_subrace.0 = default_sub;
                selected_race.0 = default_sub.rig_race();
            }
            return;
        }
    }

    for (hovered, sub_btn) in &subrace_buttons {
        if hovered.get() {
            if selected_subrace.0 != sub_btn.0 {
                selected_subrace.0 = sub_btn.0;
                selected_race.0 = sub_btn.0.rig_race();
            }
            return;
        }
    }

    for (hovered, action) in &action_buttons {
        if hovered.get() {
            match action {
                RaceMenuAction::Back => {
                    *main_menu_phase = MainMenuPhase::ActionMenu;
                    next_state.set(MenuState::MainMenu);
                }
                RaceMenuAction::Confirm => {
                    selected_race.0 = selected_subrace.0.rig_race();
                    character_ready.0 = true;
                    *main_menu_phase = MainMenuPhase::ActionMenu;
                    next_state.set(MenuState::MainMenu);
                }
            }
            return;
        }
    }
}

fn update_core_race_card_hover(
    selected_subrace: Res<SelectedSubrace>,
    mut cards: Query<
        (
            &Hovered,
            Has<Pressed>,
            &CoreRaceCardButton,
            &mut BorderColor,
            &mut BackgroundColor,
            &Children,
        ),
        With<Button>,
    >,
    mut labels: Query<&mut TextColor, With<CoreRaceCardLabel>>,
) {
    let active_race = selected_subrace.0.parent_race();
    for (hovered, is_pressed, btn, mut border, mut bg, children) in &mut cards {
        let is_current = btn.0 == active_race;
        let is_active = hovered.get() || is_pressed;

        if is_active {
            *border = BorderColor::all(palette::ACCENT_LINE);
            *bg = BackgroundColor(palette::BTN_HOVER_BG);
        } else if is_current {
            *border = BorderColor::all(palette::ACCENT_LINE);
            *bg = BackgroundColor(palette::BTN_HOVER_BG);
        } else {
            *border = BorderColor::all(palette::PANEL_BORDER_SUBTLE);
            *bg = BackgroundColor(palette::PANEL_INNER_BG);
        }

        for child in children.iter() {
            if let Ok(mut text_color) = labels.get_mut(child) {
                text_color.0 = if is_active || is_current {
                    palette::TEXT_ACCENT
                } else {
                    palette::TEXT_PRIMARY
                };
            }
        }
    }
}

fn update_subrace_card_hover(
    selected_subrace: Res<SelectedSubrace>,
    mut cards: Query<
        (
            &Hovered,
            Has<Pressed>,
            &SubraceCardButton,
            &mut BorderColor,
            &mut BackgroundColor,
            &Children,
        ),
        With<Button>,
    >,
    mut labels: Query<&mut TextColor, With<SubraceCardLabel>>,
) {
    for (hovered, is_pressed, btn, mut border, mut bg, children) in &mut cards {
        let is_current = btn.0 == selected_subrace.0;
        let is_active = hovered.get() || is_pressed;

        if is_active || is_current {
            *border = BorderColor::all(palette::ACCENT_LINE);
            *bg = BackgroundColor(palette::BTN_HOVER_BG);
        } else {
            *border = BorderColor::all(palette::PANEL_BORDER_SUBTLE);
            *bg = BackgroundColor(palette::PANEL_INNER_BG);
        }

        for child in children.iter() {
            if let Ok(mut text_color) = labels.get_mut(child) {
                text_color.0 = if is_active || is_current {
                    palette::TEXT_ACCENT
                } else {
                    palette::TEXT_PRIMARY
                };
            }
        }
    }
}

fn update_race_action_button_hover(
    mut buttons_query: Query<
        (
            &Hovered,
            Has<Pressed>,
            &ThemedButton,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (With<Button>, With<RaceMenuAction>),
    >,
    mut labels_query: Query<&mut TextColor, With<RaceActionButtonLabel>>,
) {
    for (hovered, is_pressed, theme, mut bg, mut border, children) in &mut buttons_query {
        let is_active = hovered.get() || is_pressed;
        let (next_bg, next_border) = theme.colors(hovered.get(), is_pressed);
        bg.0 = next_bg;
        *border = BorderColor::all(next_border);

        for child in children.iter() {
            if let Ok(mut text_color) = labels_query.get_mut(child) {
                text_color.0 = if is_active {
                    palette::TEXT_ACCENT
                } else {
                    palette::TEXT_PRIMARY
                };
            }
        }
    }
}
