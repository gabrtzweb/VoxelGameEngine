use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::Button,
};

use crate::{
    core::{AppFont, Localization, text_shadow_default},
    player::{CharacterSubrace, SelectedSubrace},
};

use super::{GameSettings, MenuState, menu_bar::MENU_BAR_HEIGHT, theme::palette};

#[derive(Debug, Clone, Copy)]
pub struct SkillDef {
    pub id: &'static str,
    pub tier: &'static str,
    pub name_key: &'static str,
    pub desc_key: &'static str,
}

impl SkillDef {
    pub const fn new(
        id: &'static str,
        tier: &'static str,
        name_key: &'static str,
        desc_key: &'static str,
    ) -> Self {
        Self {
            id,
            tier,
            name_key,
            desc_key,
        }
    }

    pub fn icon_path(self) -> String {
        format!("textures/interfaces/icons/skills/skill_{}.png", self.id)
    }
}

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

    fn loc_key(self) -> &'static str {
        match self {
            Self::Combat => "skills.tab.combat",
            Self::Gathering => "skills.tab.gathering",
            Self::Crafting => "skills.tab.crafting",
            Self::Exploration => "skills.tab.exploration",
            Self::Racial => "skills.tab.racial",
        }
    }

    fn fallback_label(self) -> &'static str {
        match self {
            Self::Combat => "COMBAT",
            Self::Gathering => "GATHERING",
            Self::Crafting => "CRAFTING",
            Self::Exploration => "EXPLORATION",
            Self::Racial => "RACIAL",
        }
    }

    pub fn skills_for(self, subrace: CharacterSubrace) -> [SkillDef; 6] {
        match self {
            Self::Combat => [
                SkillDef::new("steady_grip", "I", "skill.steady_grip.name", "skill.steady_grip.desc"),
                SkillDef::new("iron_guard", "I", "skill.iron_guard.name", "skill.iron_guard.desc"),
                SkillDef::new("precision_strike", "II", "skill.precision_strike.name", "skill.precision_strike.desc"),
                SkillDef::new("second_wind", "II", "skill.second_wind.name", "skill.second_wind.desc"),
                SkillDef::new("warlord_resolve", "III", "skill.warlord_resolve.name", "skill.warlord_resolve.desc"),
                SkillDef::new("executioner", "III", "skill.executioner.name", "skill.executioner.desc"),
            ],
            Self::Gathering => [
                SkillDef::new("keen_pick", "I", "skill.keen_pick.name", "skill.keen_pick.desc"),
                SkillDef::new("woodsman", "I", "skill.woodsman.name", "skill.woodsman.desc"),
                SkillDef::new("green_thumb", "II", "skill.green_thumb.name", "skill.green_thumb.desc"),
                SkillDef::new("prospector", "II", "skill.prospector.name", "skill.prospector.desc"),
                SkillDef::new("master_harvester", "III", "skill.master_harvester.name", "skill.master_harvester.desc"),
                SkillDef::new("deep_delver", "III", "skill.deep_delver.name", "skill.deep_delver.desc"),
            ],
            Self::Crafting => [
                SkillDef::new("neat_joinery", "I", "skill.neat_joinery.name", "skill.neat_joinery.desc"),
                SkillDef::new("stonecutter", "I", "skill.stonecutter.name", "skill.stonecutter.desc"),
                SkillDef::new("smelter", "II", "skill.smelter.name", "skill.smelter.desc"),
                SkillDef::new("artisan", "II", "skill.artisan.name", "skill.artisan.desc"),
                SkillDef::new("masterwork", "III", "skill.masterwork.name", "skill.masterwork.desc"),
                SkillDef::new("alchemist", "III", "skill.alchemist.name", "skill.alchemist.desc"),
            ],
            Self::Exploration => [
                SkillDef::new("lightfooted", "I", "skill.lightfooted.name", "skill.lightfooted.desc"),
                SkillDef::new("pathfinder", "I", "skill.pathfinder.name", "skill.pathfinder.desc"),
                SkillDef::new("climber", "II", "skill.climber.name", "skill.climber.desc"),
                SkillDef::new("weatherproof", "II", "skill.weatherproof.name", "skill.weatherproof.desc"),
                SkillDef::new("wayfarer", "III", "skill.wayfarer.name", "skill.wayfarer.desc"),
                SkillDef::new("cartographer", "III", "skill.cartographer.name", "skill.cartographer.desc"),
            ],
            Self::Racial => match subrace {
                CharacterSubrace::Nordling => [
                    SkillDef::new("versatility", "I", "skill.versatility.name", "skill.versatility.desc"),
                    SkillDef::new("resolve", "I", "skill.resolve.name", "skill.resolve.desc"),
                    SkillDef::new("pioneer", "II", "skill.pioneer.name", "skill.pioneer.desc"),
                    SkillDef::new("winter_blood", "III", "skill.winter_blood.name", "skill.winter_blood.desc"),
                    SkillDef::new("bearhide", "III", "skill.bearhide.name", "skill.bearhide.desc"),
                    SkillDef::new("highland_stamina", "III", "skill.highland_stamina.name", "skill.highland_stamina.desc"),
                ],
                CharacterSubrace::Sunlander => [
                    SkillDef::new("versatility", "I", "skill.versatility.name", "skill.versatility.desc"),
                    SkillDef::new("resolve", "I", "skill.resolve.name", "skill.resolve.desc"),
                    SkillDef::new("pioneer", "II", "skill.pioneer.name", "skill.pioneer.desc"),
                    SkillDef::new("quick_mind", "III", "skill.quick_mind.name", "skill.quick_mind.desc"),
                    SkillDef::new("dune_stride", "III", "skill.dune_stride.name", "skill.dune_stride.desc"),
                    SkillDef::new("sunborn", "III", "skill.sunborn.name", "skill.sunborn.desc"),
                ],
                CharacterSubrace::CoastalHuman => [
                    SkillDef::new("versatility", "I", "skill.versatility.name", "skill.versatility.desc"),
                    SkillDef::new("resolve", "I", "skill.resolve.name", "skill.resolve.desc"),
                    SkillDef::new("pioneer", "II", "skill.pioneer.name", "skill.pioneer.desc"),
                    SkillDef::new("tide_runner", "III", "skill.tide_runner.name", "skill.tide_runner.desc"),
                    SkillDef::new("deep_breath", "III", "skill.deep_breath.name", "skill.deep_breath.desc"),
                    SkillDef::new("mariner_fortune", "III", "skill.mariner_fortune.name", "skill.mariner_fortune.desc"),
                ],
                CharacterSubrace::WoodElf => [
                    SkillDef::new("graceful_leap", "I", "skill.graceful_leap.name", "skill.graceful_leap.desc"),
                    SkillDef::new("silent_tread", "I", "skill.silent_tread.name", "skill.silent_tread.desc"),
                    SkillDef::new("eagle_eye", "II", "skill.eagle_eye.name", "skill.eagle_eye.desc"),
                    SkillDef::new("canopy_sprint", "III", "skill.canopy_sprint.name", "skill.canopy_sprint.desc"),
                    SkillDef::new("forest_veil", "III", "skill.forest_veil.name", "skill.forest_veil.desc"),
                    SkillDef::new("ancestral_bow", "III", "skill.ancestral_bow.name", "skill.ancestral_bow.desc"),
                ],
                CharacterSubrace::DarkElf => [
                    SkillDef::new("graceful_leap", "I", "skill.graceful_leap.name", "skill.graceful_leap.desc"),
                    SkillDef::new("silent_tread", "I", "skill.silent_tread.name", "skill.silent_tread.desc"),
                    SkillDef::new("eagle_eye", "II", "skill.eagle_eye.name", "skill.eagle_eye.desc"),
                    SkillDef::new("abyssal_sight", "III", "skill.abyssal_sight.name", "skill.abyssal_sight.desc"),
                    SkillDef::new("shadow_ambush", "III", "skill.shadow_ambush.name", "skill.shadow_ambush.desc"),
                    SkillDef::new("crystal_affinity", "III", "skill.crystal_affinity.name", "skill.crystal_affinity.desc"),
                ],
                CharacterSubrace::HighElf => [
                    SkillDef::new("graceful_leap", "I", "skill.graceful_leap.name", "skill.graceful_leap.desc"),
                    SkillDef::new("silent_tread", "I", "skill.silent_tread.name", "skill.silent_tread.desc"),
                    SkillDef::new("eagle_eye", "II", "skill.eagle_eye.name", "skill.eagle_eye.desc"),
                    SkillDef::new("arcane_resonance", "III", "skill.arcane_resonance.name", "skill.arcane_resonance.desc"),
                    SkillDef::new("starlight_aim", "III", "skill.starlight_aim.name", "skill.starlight_aim.desc"),
                    SkillDef::new("aether_ward", "III", "skill.aether_ward.name", "skill.aether_ward.desc"),
                ],
                CharacterSubrace::MountainDwarf => [
                    SkillDef::new("stoneborn", "I", "skill.stoneborn.name", "skill.stoneborn.desc"),
                    SkillDef::new("stout_frame", "I", "skill.stout_frame.name", "skill.stout_frame.desc"),
                    SkillDef::new("forge_master", "II", "skill.forge_master.name", "skill.forge_master.desc"),
                    SkillDef::new("granite_bulwark", "III", "skill.granite_bulwark.name", "skill.granite_bulwark.desc"),
                    SkillDef::new("seismic_blow", "III", "skill.seismic_blow.name", "skill.seismic_blow.desc"),
                    SkillDef::new("peak_endurance", "III", "skill.peak_endurance.name", "skill.peak_endurance.desc"),
                ],
                CharacterSubrace::DeepDwarf => [
                    SkillDef::new("stoneborn", "I", "skill.stoneborn.name", "skill.stoneborn.desc"),
                    SkillDef::new("stout_frame", "I", "skill.stout_frame.name", "skill.stout_frame.desc"),
                    SkillDef::new("forge_master", "II", "skill.forge_master.name", "skill.forge_master.desc"),
                    SkillDef::new("ore_sense", "III", "skill.ore_sense.name", "skill.ore_sense.desc"),
                    SkillDef::new("magma_ward", "III", "skill.magma_ward.name", "skill.magma_ward.desc"),
                    SkillDef::new("relentless_pick", "III", "skill.relentless_pick.name", "skill.relentless_pick.desc"),
                ],
                CharacterSubrace::HillDwarf => [
                    SkillDef::new("stoneborn", "I", "skill.stoneborn.name", "skill.stoneborn.desc"),
                    SkillDef::new("stout_frame", "I", "skill.stout_frame.name", "skill.stout_frame.desc"),
                    SkillDef::new("forge_master", "II", "skill.forge_master.name", "skill.forge_master.desc"),
                    SkillDef::new("golden_hammer", "III", "skill.golden_hammer.name", "skill.golden_hammer.desc"),
                    SkillDef::new("hearty_brew", "III", "skill.hearty_brew.name", "skill.hearty_brew.desc"),
                    SkillDef::new("rich_vein", "III", "skill.rich_vein.name", "skill.rich_vein.desc"),
                ],
            },
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

fn spawn_skills_menu(
    mut commands: Commands,
    active_tab: Res<SkillTreeTab>,
    selected_subrace: Option<Res<SelectedSubrace>>,
    loc: Option<Res<Localization>>,
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

    let subrace = selected_subrace.map_or(CharacterSubrace::default(), |s| s.0);
    let subrace_name = subrace.display_name(loc.as_deref());
    let race_icon = asset_server.load(subrace.icon_asset_path());
    let attrs = subrace.attributes();
    let trait_keys = subrace.trait_keys();
    let player_name = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "Adventurer".to_string());
    let dimming_alpha = game_settings.map_or(0.65, |s| s.background_dimming.alpha());
    let points_lbl = loc.as_ref().map_or("Skill Points", |l| l.t("skills.points"));
    let tier_lbl = loc.as_ref().map_or("Tier", |l| l.t("skills.tier"));

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
                    row_gap: px(14.0),
                    overflow: Overflow::scroll_y(),
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
                                        Text::new(subrace_name.to_uppercase()),
                                        make_font(11.0),
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
                        row_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|attrs_node| {
                        attrs_node.spawn((
                            Text::new(
                                loc.as_ref()
                                    .map_or("ATTRIBUTES", |l| l.t("menu.race_selection.attributes"))
                                    .to_uppercase(),
                            ),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        let stats = [
                            (
                                loc.as_ref().map_or("Health", |l| l.t("attr.health")),
                                format!("{} / {}", attrs.max_health, attrs.max_health),
                            ),
                            (
                                loc.as_ref().map_or("Stamina", |l| l.t("attr.stamina")),
                                format!("{} / {}", attrs.max_stamina, attrs.max_stamina),
                            ),
                            (
                                loc.as_ref().map_or("Defense", |l| l.t("attr.defense")),
                                format!("{}", attrs.defense),
                            ),
                            (
                                loc.as_ref().map_or("Move Speed", |l| l.t("attr.move_speed")),
                                format!("{}%", (attrs.move_speed_multiplier * 100.0).round() as i32),
                            ),
                            (
                                loc.as_ref().map_or("Jump Height", |l| l.t("attr.jump_height")),
                                format!("{:.2}b", attrs.jump_height_blocks),
                            ),
                            (
                                loc.as_ref()
                                    .map_or("Step Noise", |l| l.t("attr.footstep_noise")),
                                format!(
                                    "{}%",
                                    (attrs.footstep_noise_multiplier * 100.0).round() as i32
                                ),
                            ),
                            (
                                loc.as_ref().map_or("Swim Speed", |l| l.t("attr.swim_speed")),
                                format!("{}%", (attrs.swim_speed_multiplier * 100.0).round() as i32),
                            ),
                            (
                                loc.as_ref()
                                    .map_or("Mining Speed", |l| l.t("attr.mining_speed")),
                                format!(
                                    "{}%",
                                    (attrs.mining_speed_multiplier * 100.0).round() as i32
                                ),
                            ),
                            (
                                loc.as_ref()
                                    .map_or("Mining Reach", |l| l.t("attr.reach_bonus")),
                                format!("+{:.2}b", attrs.reach_bonus_blocks),
                            ),
                            (
                                loc.as_ref()
                                    .map_or("Cave Vision", |l| l.t("attr.cave_vision")),
                                format!("+{}%", (attrs.cave_vision_bonus * 100.0).round() as i32),
                            ),
                            (
                                loc.as_ref()
                                    .map_or("Fall Resistance", |l| l.t("attr.fall_resistance")),
                                format!(
                                    "+{}%",
                                    (attrs.fall_damage_reduction * 100.0).round() as i32
                                ),
                            ),
                            (
                                loc.as_ref()
                                    .map_or("Knockback Res.", |l| l.t("attr.knockback_res")),
                                format!("+{}%", (attrs.knockback_resistance * 100.0).round() as i32),
                            ),
                            (
                                loc.as_ref()
                                    .map_or("Cold Resistance", |l| l.t("attr.cold_res")),
                                format!("+{}%", (attrs.cold_resistance * 100.0).round() as i32),
                            ),
                            (
                                loc.as_ref()
                                    .map_or("Heat Resistance", |l| l.t("attr.heat_res")),
                                format!("+{}%", (attrs.heat_resistance * 100.0).round() as i32),
                            ),
                            (
                                loc.as_ref().map_or("XP Gain", |l| l.t("attr.xp_gain")),
                                format!(
                                    "+{}%",
                                    ((attrs.xp_gain_multiplier - 1.0) * 100.0).round() as i32
                                ),
                            ),
                        ];

                        for (k, v) in stats {
                            attrs_node
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
                            Text::new(
                                loc.as_ref()
                                    .map_or("RACIAL TRAITS", |l| l.t("menu.race_selection.traits"))
                                    .to_uppercase(),
                            ),
                            make_font(12.0),
                            TextColor(palette::TEXT_MUTED),
                        ));

                        for (title_key, desc_key) in trait_keys {
                            let title = loc.as_ref().map_or(title_key, |l| l.t(title_key));
                            let desc = loc.as_ref().map_or(desc_key, |l| l.t(desc_key));
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
                                let tab_text = loc
                                    .as_ref()
                                    .map_or(tab.fallback_label(), |l| l.t(tab.loc_key()));
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
                                        Text::new(tab_text),
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
                        Text::new(format!("{points_lbl}: 0")),
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
                        for skill in tab.skills_for(subrace) {
                            let icon_handle = asset_server.load(skill.icon_path());
                            let title = loc
                                .as_ref()
                                .map_or(skill.id, |l| l.t(skill.name_key));
                            let desc = loc
                                .as_ref()
                                .map_or("", |l| l.t(skill.desc_key));

                            grid.spawn((
                                Node {
                                    width: px(310.0),
                                    padding: UiRect::all(px(14.0)),
                                    border: UiRect::all(px(1.5)),
                                    border_radius: BorderRadius::all(px(6.0)),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::FlexStart,
                                    column_gap: px(12.0),
                                    ..default()
                                },
                                BackgroundColor(palette::PANEL_INNER_BG),
                                BorderColor::all(palette::PANEL_BORDER_SUBTLE),
                            ))
                            .with_children(|card| {
                                // Dedicated 16x16 Skill Icon scaled crisply to 40x40
                                card.spawn((
                                    ImageNode {
                                        image: icon_handle,
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

                                card.spawn(Node {
                                    flex_grow: 1.0,
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(6.0),
                                    ..default()
                                })
                                .with_children(|info| {
                                    info.spawn(Node {
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
                                            make_font(14.0),
                                            TextColor(palette::TEXT_PRIMARY),
                                            text_shadow_default(),
                                        ));
                                        top.spawn((
                                            Text::new(format!("{tier_lbl} {}  •  0/3", skill.tier)),
                                            make_font(11.0),
                                            TextColor(palette::TEXT_ACCENT),
                                        ));
                                    });

                                    info.spawn((
                                        Text::new(desc),
                                        make_font(12.0),
                                        TextColor(palette::TEXT_MUTED),
                                    ));
                                });
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
