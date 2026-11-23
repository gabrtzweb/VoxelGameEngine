use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{
    asset::RenderAssetUsages,
    picking::Pickable,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::{
    core::{AppFont, Localization, text_shadow_default},
    menu::{MenuState, theme::palette},
    world::WorldSession,
};

use super::{
    GameMode, InspectorInteraction, PLAYER_EYE_HEIGHT, Player, PlayerEnvironmentStatus,
    PlayerInventory, PlayerMotion, SelectedSubrace, WorldSpawnPoint, controller::PlayerCamera,
};

const RING_TEX_SIZE: u32 = 64;
const BASE_RING_DISPLAY_SIZE: f32 = 60.0;
const BASE_RING_ICON_SIZE: f32 = 28.0;
const BASE_HEALTH_WIDTH: f32 = 360.0;
const BASE_HEALTH_HEIGHT: f32 = 28.0;
const BASE_STAMINA_WIDTH: f32 = 316.0;
const BASE_STAMINA_HEIGHT: f32 = 20.0;
const BASE_CLIMATE_BADGE_SIZE: f32 = 38.0;
const BASE_CLIMATE_ICON_SIZE: f32 = 22.0;
const BASE_HUD_LEFT: f32 = 36.0;
// Hotbar vertical center sits at (270 - 236) * 4 = 136px from bottom;
// Health (28) + gap (10) + Stamina (20) = 58px -> Stamina bottom at 107px;
// Climate row below Stamina adds gap (10) + badge (38) = 48px -> root bottom = 59px.
const BASE_HUD_BOTTOM: f32 = 59.0;

#[derive(Resource, Debug, Clone)]
pub struct PlayerVitals {
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub stamina_regen_delay: f32,
    pub stamina_exhausted: bool,
    pub is_sprinting: bool,
    pub hunger: f32,
    pub thirst: f32,
    pub temperature: f32,
    pub oxygen: f32,
    pub max_oxygen: f32,
    drown_tick_timer: f32,
    fall_peak_y: Option<f32>,
    was_grounded: bool,
    initialized: bool,
    last_subrace: Option<super::CharacterSubrace>,
}

impl Default for PlayerVitals {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            stamina_regen_delay: 0.0,
            stamina_exhausted: false,
            is_sprinting: false,
            hunger: 100.0,
            thirst: 100.0,
            temperature: 50.0,
            oxygen: 100.0,
            max_oxygen: 100.0,
            drown_tick_timer: 0.0,
            fall_peak_y: None,
            was_grounded: true,
            initialized: false,
            last_subrace: None,
        }
    }
}

#[derive(Resource)]
struct VitalRingTextures {
    hunger: Handle<Image>,
    thirst: Handle<Image>,
    oxygen: Handle<Image>,
    last_hunger_pct: i32,
    last_thirst_pct: i32,
    last_oxy_pct: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ClimateBadgeKind {
    Cold,
    Neutral,
    Heat,
}

#[derive(Resource, Default)]
struct GameModeToastState {
    timer: f32,
    last_mode: Option<GameMode>,
    pending_custom_message: Option<String>,
}

const TOAST_HOLD_SECS: f32 = 1.6;
const TOAST_FADE_SECS: f32 = 0.6;
const TOAST_TOTAL_SECS: f32 = TOAST_HOLD_SECS + TOAST_FADE_SECS;

#[derive(Component)]
struct SurvivalHudRoot;

#[derive(Component)]
struct HudRingColumn;

#[derive(Component)]
struct HudRingBox;

#[derive(Component)]
struct HudRingIcon;

#[derive(Component)]
struct HudHealthTrack;

#[derive(Component)]
struct HudStaminaTrack;

#[derive(Component)]
struct HealthBarFill;

#[derive(Component)]
struct HealthBarText;

#[derive(Component)]
struct StaminaBarFill;

#[derive(Component)]
struct StaminaBarText;

#[derive(Component)]
struct OxygenRingContainer;

#[derive(Component)]
struct HudClimateRow;

#[derive(Component)]
struct HudClimateBadge(ClimateBadgeKind);

#[derive(Component)]
struct HudClimateIcon(ClimateBadgeKind);

#[derive(Component)]
struct GameModeToastRoot;

#[derive(Component)]
struct GameModeToastPill;

#[derive(Component)]
struct GameModeToastText;

#[derive(Component)]
struct MiningProgressRoot;

#[derive(Component)]
struct MiningProgressFill;

pub struct SurvivalHudPlugin;

impl Plugin for SurvivalHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerVitals>()
            .init_resource::<GameModeToastState>()
            .add_systems(Startup, spawn_survival_hud_and_toast)
            .add_systems(
                Update,
                (
                    sync_vitals_with_subrace_and_movement,
                    update_player_temperature,
                    sync_survival_hud_scale,
                    update_survival_hud_ui,
                    update_mining_progress_bar,
                    update_game_mode_toast,
                ),
            );
    }
}

fn create_ring_image(fraction: f32, arc_rgb: [u8; 3]) -> Image {
    let mut data = vec![0u8; (RING_TEX_SIZE * RING_TEX_SIZE * 4) as usize];
    write_ring_pixels(&mut data, fraction, arc_rgb);
    Image::new(
        Extent3d {
            width: RING_TEX_SIZE,
            height: RING_TEX_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

fn write_ring_pixels(data: &mut [u8], fraction: f32, arc_rgb: [u8; 3]) {
    let size = RING_TEX_SIZE as f32;
    let center = (size - 1.0) * 0.5;
    let inner_r = 21.5;
    let outer_r = 29.0;
    let border_r = 30.8;
    let clamped = fraction.clamp(0.0, 1.0);
    let target_angle = clamped * TAU;

    for y in 0..RING_TEX_SIZE {
        for x in 0..RING_TEX_SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * RING_TEX_SIZE + x) * 4) as usize;

            if dist > border_r + 1.0 {
                data[idx] = 0;
                data[idx + 1] = 0;
                data[idx + 2] = 0;
                data[idx + 3] = 0;
                continue;
            }

            // Clockwise angle starting from 12 o'clock (top: dx=0, dy<0)
            let mut angle = dy.atan2(dx) + FRAC_PI_2;
            if angle < 0.0 {
                angle += TAU;
            }

            let (r, g, b, a) = if dist < inner_r {
                (12u8, 15u8, 22u8, 185u8)
            } else if dist <= outer_r {
                let in_arc = clamped >= 0.999 || (clamped > 0.001 && angle <= target_angle);
                if in_arc {
                    (arc_rgb[0], arc_rgb[1], arc_rgb[2], 245u8)
                } else {
                    (32u8, 36u8, 48u8, 210u8)
                }
            } else {
                let edge = (1.0 - (dist - outer_r) / (border_r - outer_r)).clamp(0.0, 1.0);
                (8u8, 10u8, 14u8, (edge * 210.0) as u8)
            };

            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = a;
        }
    }
}

fn spawn_ring_node(
    parent: &mut ChildSpawnerCommands,
    ring_handle: Handle<Image>,
    icon_handle: Handle<Image>,
) {
    parent
        .spawn((
            HudRingBox,
            Pickable::IGNORE,
            Node {
                width: px(BASE_RING_DISPLAY_SIZE),
                height: px(BASE_RING_DISPLAY_SIZE),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|slot| {
            slot.spawn((
                HudRingBox,
                Pickable::IGNORE,
                ImageNode {
                    image: ring_handle,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: px(0.0),
                    width: px(BASE_RING_DISPLAY_SIZE),
                    height: px(BASE_RING_DISPLAY_SIZE),
                    ..default()
                },
            ));
            slot.spawn((
                HudRingIcon,
                Pickable::IGNORE,
                ImageNode {
                    image: icon_handle,
                    ..default()
                },
                Node {
                    width: px(BASE_RING_ICON_SIZE),
                    height: px(BASE_RING_ICON_SIZE),
                    ..default()
                },
                ZIndex(2),
            ));
        });
}

fn spawn_survival_hud_and_toast(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    app_font: Option<Res<AppFont>>,
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

    let hunger_ring = images.add(create_ring_image(1.0, [232, 168, 52]));
    let thirst_ring = images.add(create_ring_image(1.0, [64, 182, 242]));
    let oxy_ring = images.add(create_ring_image(1.0, [82, 214, 250]));

    commands.insert_resource(VitalRingTextures {
        hunger: hunger_ring.clone(),
        thirst: thirst_ring.clone(),
        oxygen: oxy_ring.clone(),
        last_hunger_pct: 100,
        last_thirst_pct: 100,
        last_oxy_pct: 100,
    });

    let hunger_icon: Handle<Image> =
        asset_server.load("textures/interfaces/icons/hud/hud_hunger.png");
    let thirst_icon: Handle<Image> =
        asset_server.load("textures/interfaces/icons/hud/hud_thirst.png");
    let oxy_icon: Handle<Image> = asset_server.load("textures/interfaces/icons/hud/hud_oxygen.png");
    let cold_icon: Handle<Image> =
        asset_server.load("textures/interfaces/icons/hud/hud_temp_cold.png");
    let neutral_icon: Handle<Image> =
        asset_server.load("textures/interfaces/icons/hud/hud_temp_neutral.png");
    let heat_icon: Handle<Image> =
        asset_server.load("textures/interfaces/icons/hud/hud_temp_heat.png");

    // 1. Bottom-Left Floating Survival Vitals HUD (no container background, aligned with Hotbar & scaled with gui_scale)
    commands
        .spawn((
            SurvivalHudRoot,
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: px(BASE_HUD_LEFT),
                bottom: px(BASE_HUD_BOTTOM),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(8.0),
                ..default()
            },
            Visibility::Hidden,
            ZIndex(90),
        ))
        .with_children(|root| {
            // Vertical column of circular progress rings above Health (space to the right reserved for buff text)
            root.spawn((
                HudRingColumn,
                Pickable::IGNORE,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: px(7.0),
                    margin: UiRect::bottom(px(2.0)),
                    ..default()
                },
            ))
            .with_children(|col| {
                // Dynamic Oxygen Ring (top of column, visible only when submerged / recovering)
                col.spawn((
                    OxygenRingContainer,
                    Pickable::IGNORE,
                    Node {
                        display: Display::None,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|row| {
                    spawn_ring_node(row, oxy_ring, oxy_icon);
                });

                spawn_ring_node(col, hunger_ring, hunger_icon);
                spawn_ring_node(col, thirst_ring, thirst_icon);
            });

            // Health Bar Row (larger, crisp floating bar)
            root.spawn((
                Pickable::IGNORE,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(10.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                row.spawn((
                    HudHealthTrack,
                    Pickable::IGNORE,
                    Node {
                        width: px(BASE_HEALTH_WIDTH),
                        height: px(BASE_HEALTH_HEIGHT),
                        border: UiRect::all(px(1.5)),
                        border_radius: BorderRadius::all(px(4.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.82)),
                    BorderColor::all(Color::srgba(0.34, 0.14, 0.16, 0.90)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        HealthBarFill,
                        Pickable::IGNORE,
                        Node {
                            width: percent(100.0),
                            height: percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.80, 0.20, 0.24)),
                    ));
                });

                row.spawn((
                    HealthBarText,
                    Pickable::IGNORE,
                    Text::new("100 / 100"),
                    make_font(14.0),
                    TextColor(palette::TEXT_PRIMARY),
                    text_shadow_default(),
                ));
            });

            // Stamina Bar Row
            root.spawn((
                Pickable::IGNORE,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(10.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                row.spawn((
                    HudStaminaTrack,
                    Pickable::IGNORE,
                    Node {
                        width: px(BASE_STAMINA_WIDTH),
                        height: px(BASE_STAMINA_HEIGHT),
                        border: UiRect::all(px(1.5)),
                        border_radius: BorderRadius::all(px(3.5)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.82)),
                    BorderColor::all(Color::srgba(0.42, 0.34, 0.14, 0.90)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        StaminaBarFill,
                        Pickable::IGNORE,
                        Node {
                            width: percent(100.0),
                            height: percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.92, 0.74, 0.22)),
                    ));
                });

                row.spawn((
                    StaminaBarText,
                    Pickable::IGNORE,
                    Text::new("100 / 100"),
                    make_font(12.5),
                    TextColor(palette::TEXT_MUTED),
                    text_shadow_default(),
                ));
            });

            // Climate State Badges Row (Cold / Neutral / Heat) below Stamina
            root.spawn((
                HudClimateRow,
                Pickable::IGNORE,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(10.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                for (kind, icon_handle) in [
                    (ClimateBadgeKind::Cold, cold_icon),
                    (ClimateBadgeKind::Neutral, neutral_icon),
                    (ClimateBadgeKind::Heat, heat_icon),
                ] {
                    row.spawn((
                        HudClimateBadge(kind),
                        Pickable::IGNORE,
                        Node {
                            width: px(BASE_CLIMATE_BADGE_SIZE),
                            height: px(BASE_CLIMATE_BADGE_SIZE),
                            border: UiRect::all(px(2.0)),
                            border_radius: BorderRadius::all(px(BASE_CLIMATE_BADGE_SIZE * 0.5)),
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.58)),
                        BorderColor::all(Color::srgba(0.22, 0.25, 0.32, 0.55)),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            HudClimateIcon(kind),
                            Pickable::IGNORE,
                            ImageNode {
                                image: icon_handle,
                                color: Color::srgba(1.0, 1.0, 1.0, 0.30),
                                ..default()
                            },
                            Node {
                                width: px(BASE_CLIMATE_ICON_SIZE),
                                height: px(BASE_CLIMATE_ICON_SIZE),
                                ..default()
                            },
                        ));
                    });
                }
            });
        });

    // 2. Screen-Center Game Mode Switch Toast Pill (Dark Iron UI style)
    commands
        .spawn((
            GameModeToastRoot,
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                right: px(0.0),
                top: px(0.0),
                bottom: px(0.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            ZIndex(220),
        ))
        .with_children(|center| {
            center
                .spawn((
                    GameModeToastPill,
                    Pickable::IGNORE,
                    Node {
                        padding: UiRect::axes(px(24.0), px(12.0)),
                        border: UiRect::all(px(1.5)),
                        border_radius: BorderRadius::all(px(6.0)),
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.08, 0.11, 0.0)),
                    BorderColor::all(Color::srgba(0.28, 0.31, 0.38, 0.0)),
                ))
                .with_children(|toast| {
                    toast.spawn((
                        GameModeToastText,
                        Pickable::IGNORE,
                        Text::new(""),
                        make_font(16.0),
                        TextColor(Color::srgba(0.92, 0.93, 0.96, 0.0)),
                        text_shadow_default(),
                    ));
                });
        });

    // 3. Under-Crosshair Mining Progress Bar (Survival block breaking indicator)
    commands
        .spawn((
            MiningProgressRoot,
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50.0),
                top: percent(50.0),
                margin: UiRect {
                    left: px(-32.0),
                    top: px(18.0),
                    ..default()
                },
                width: px(64.0),
                height: px(6.0),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(2.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.84)),
            BorderColor::all(Color::srgba(0.32, 0.35, 0.44, 0.88)),
            Visibility::Hidden,
            ZIndex(210),
        ))
        .with_children(|bar| {
            bar.spawn((
                MiningProgressFill,
                Pickable::IGNORE,
                Node {
                    width: percent(0.0),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.80, 0.34)),
            ));
        });
}

#[allow(clippy::too_many_arguments)]
fn sync_vitals_with_subrace_and_movement(
    time: Res<Time>,
    game_mode: Res<GameMode>,
    world_session: Option<Res<WorldSession>>,
    menu_state: Option<Res<State<MenuState>>>,
    selected_subrace: Option<Res<SelectedSubrace>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    world_spawn: Option<Res<WorldSpawnPoint>>,
    loc: Option<Res<Localization>>,
    mut player_q: Query<(&mut Transform, &mut PlayerMotion), With<Player>>,
    mut camera_q: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    mut vitals: ResMut<PlayerVitals>,
    mut toast_state: ResMut<GameModeToastState>,
    mut player_inv: Option<ResMut<PlayerInventory>>,
    mut journal: Option<ResMut<crate::menu::journal::JournalProgress>>,
) {
    let subrace = selected_subrace.map_or(super::CharacterSubrace::default(), |s| s.0);
    let attrs = subrace.attributes();
    let target_max_hp = attrs.max_health as f32;
    let target_max_stam = attrs.max_stamina as f32;
    let target_max_oxy = if subrace == super::CharacterSubrace::CoastalHuman {
        150.0
    } else {
        100.0
    };

    if !vitals.initialized || vitals.last_subrace != Some(subrace) {
        vitals.last_subrace = Some(subrace);
        if let Some(ref mut inv) = player_inv {
            inv.unlocked_slots = (attrs.carry_slots as usize).clamp(
                super::inventory::BASE_UNLOCKED_INVENTORY_SLOTS,
                super::inventory::PLAYER_INVENTORY_SLOTS,
            );
        }
    }

    if !vitals.initialized || (vitals.max_health - target_max_hp).abs() > 0.1 {
        vitals.max_health = target_max_hp;
        vitals.health = target_max_hp;
    }
    if !vitals.initialized || (vitals.max_stamina - target_max_stam).abs() > 0.1 {
        vitals.max_stamina = target_max_stam;
        vitals.stamina = target_max_stam;
    }
    if !vitals.initialized || (vitals.max_oxygen - target_max_oxy).abs() > 0.1 {
        vitals.max_oxygen = target_max_oxy;
        vitals.oxygen = target_max_oxy;
    }
    vitals.initialized = true;

    let is_paused = menu_state.as_ref().is_some_and(|s| {
        matches!(
            *s.get(),
            MenuState::Pause
                | MenuState::Settings
                | MenuState::Controls
                | MenuState::MainMenu
                | MenuState::RaceSelection
                | MenuState::Loading
        )
    });

    if *game_mode != GameMode::Survival
        || !world_session.as_ref().is_some_and(|s| s.is_active)
        || is_paused
    {
        vitals.fall_peak_y = None;
        vitals.drown_tick_timer = 0.0;
        return;
    }

    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let mut damage_this_frame = 0.0f32;

    // 1. Stamina drain (sprinting on ground or mid-air) & regeneration
    if vitals.stamina_regen_delay > 0.0 {
        vitals.stamina_regen_delay = (vitals.stamina_regen_delay - dt).max(0.0);
    }
    if vitals.is_sprinting {
        vitals.stamina = (vitals.stamina - 16.0 * dt).max(0.0);
        vitals.stamina_regen_delay = 0.55;
        if vitals.stamina <= 0.0 {
            vitals.stamina_exhausted = true;
        }
    } else if vitals.stamina_regen_delay <= 0.0 {
        vitals.stamina = (vitals.stamina + 22.0 * dt).min(vitals.max_stamina);
        if vitals.stamina_exhausted && vitals.stamina >= 15.0 {
            vitals.stamina_exhausted = false;
        }
    }

    // 2. Oxygen drain & drowning damage when empty
    let in_water = env_status.as_ref().is_some_and(|e| e.submersion > 0.25);
    let submerged = env_status
        .as_ref()
        .is_some_and(|e| e.is_camera_in_water || e.submersion > 0.85);
    if submerged {
        vitals.oxygen = (vitals.oxygen - 12.0 * dt).max(0.0);
        if vitals.oxygen <= 0.0 {
            vitals.drown_tick_timer += dt;
            if vitals.drown_tick_timer >= 1.0 {
                vitals.drown_tick_timer -= 1.0;
                damage_this_frame += 12.0;
            }
        } else {
            vitals.drown_tick_timer = 0.0;
        }
    } else {
        vitals.oxygen = (vitals.oxygen + 35.0 * dt).min(vitals.max_oxygen);
        vitals.drown_tick_timer = 0.0;
    }

    // 3. Fall damage (uses racial fall_damage_reduction & safe margin above racial jump height)
    if let Ok((mut player_tf, mut motion)) = player_q.single_mut() {
        let player_y = player_tf.translation.y;
        if motion.flying || in_water {
            vitals.fall_peak_y = None;
        } else if !motion.grounded {
            vitals.fall_peak_y = Some(vitals.fall_peak_y.map_or(player_y, |p| p.max(player_y)));
        } else if !vitals.was_grounded
            && let Some(peak_y) = vitals.fall_peak_y.take()
        {
            let fall_dist = peak_y - player_y;
            let safe_height = 3.5 + attrs.jump_height_blocks;
            if fall_dist > safe_height {
                let raw_dmg = (fall_dist - safe_height) * 10.0;
                let mitigated = raw_dmg * (1.0 - attrs.fall_damage_reduction.clamp(0.0, 0.9));
                damage_this_frame += mitigated;
            }
        }
        vitals.was_grounded = motion.grounded;

        if damage_this_frame > 0.0 {
            vitals.health = (vitals.health - damage_this_frame).max(0.0);
            if let Some(ref mut j) = journal {
                j.damage_taken += damage_this_frame;
            }
        }

        // 4. Death & Respawn at world spawn
        if vitals.health <= 0.0 {
            if let Some(ref mut j) = journal {
                j.deaths += 1;
            }
            let spawn_pos = world_spawn.map_or(Vec3::new(-10.0, 20.0, 14.0), |s| s.0);
            player_tf.translation = spawn_pos;
            motion.velocity = Vec3::ZERO;
            motion.flying = false;
            motion.grounded = false;
            if let Ok(mut cam_tf) = camera_q.single_mut() {
                cam_tf.translation = spawn_pos + Vec3::Y * PLAYER_EYE_HEIGHT;
            }
            vitals.health = vitals.max_health;
            vitals.stamina = vitals.max_stamina;
            vitals.oxygen = vitals.max_oxygen;
            vitals.stamina_exhausted = false;
            vitals.stamina_regen_delay = 0.0;
            vitals.drown_tick_timer = 0.0;
            vitals.fall_peak_y = None;
            vitals.was_grounded = false;

            let msg = loc
                .as_ref()
                .map_or("You died! Respawning at world spawn...", |l| {
                    l.t("hud.survival.respawn")
                });
            toast_state.pending_custom_message = Some(msg.to_string());
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_player_temperature(
    time: Res<Time>,
    game_mode: Res<GameMode>,
    world_session: Option<Res<WorldSession>>,
    menu_state: Option<Res<State<MenuState>>>,
    selected_subrace: Option<Res<SelectedSubrace>>,
    env_state: Option<Res<crate::environment::EnvironmentState>>,
    weather: Option<Res<crate::environment::WeatherManager>>,
    env_status: Option<Res<PlayerEnvironmentStatus>>,
    terrain_gen: Option<Res<crate::generation::TerrainGenerator>>,
    world: Option<Res<crate::world::VoxelWorld>>,
    player_q: Query<&Transform, With<Player>>,
    mut vitals: ResMut<PlayerVitals>,
    mut sample_timer: Local<f32>,
    mut cached_target_temp: Local<Option<f32>>,
) {
    let is_paused = menu_state.as_ref().is_some_and(|s| {
        matches!(
            *s.get(),
            MenuState::Pause
                | MenuState::Settings
                | MenuState::Controls
                | MenuState::MainMenu
                | MenuState::RaceSelection
                | MenuState::Loading
        )
    });
    if *game_mode != GameMode::Survival
        || !world_session.as_ref().is_some_and(|s| s.is_active)
        || is_paused
    {
        return;
    }

    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    *sample_timer -= dt;
    if *sample_timer <= 0.0 || cached_target_temp.is_none() {
        *sample_timer = 0.45;

        let Ok(player_tf) = player_q.single() else {
            return;
        };
        let pos = player_tf.translation;
        let ix = pos.x.floor() as i32;
        let iy = pos.y.floor() as i32;
        let iz = pos.z.floor() as i32;

        // 1. Biome climate baseline (-1..+1 mapped around comfortable 50)
        let biome_temp = terrain_gen
            .as_ref()
            .map_or(0.0, |g| g.sample_column(ix, iz).climate.temperature);
        let mut target = 50.0 + biome_temp * 34.0;

        // 2. Season & Day/Night cycle
        if let Some(ref env) = env_state {
            target += match env.season() {
                crate::environment::time::Season::Winter => -12.0,
                crate::environment::time::Season::Summer => 10.0,
                crate::environment::time::Season::Autumn => -3.0,
                crate::environment::time::Season::Spring => 0.0,
            };
            target += (env.time_of_day * TAU).sin() * 8.0 - 1.0;
        }

        // 3. Altitude cooling & subterranean moderation
        if pos.y > 40.0 {
            target -= ((pos.y - 40.0) * 0.22).min(22.0);
        }
        let underground = env_status.as_ref().map_or(0.0, |e| e.underground_factor);
        if underground > 0.1 {
            target = target * (1.0 - underground * 0.65) + 40.0 * (underground * 0.65);
        }

        // 4. Weather & water immersion
        if let Some(ref w) = weather {
            target -= w.rain_intensity * 5.0 + w.snow_intensity * 10.0;
        }
        if env_status.as_ref().is_some_and(|e| e.submersion > 0.25) {
            target -= 14.0;
        }

        // 5. Nearby heat sources (lit campfires, fire, lava, magma within 4 blocks)
        if let Some(ref w) = world {
            let mut near_heat = 0.0f32;
            'heat_scan: for dy in -1..=2 {
                for dz in -4..=4 {
                    for dx in -4..=4 {
                        if let Some(v) = w.get_voxel(IVec3::new(ix + dx, iy + dy, iz + dz)) {
                            let heat = match v {
                                crate::world::Voxel::Campfire
                                | crate::world::Voxel::ElementFire => 26.0,
                                crate::world::Voxel::Magma => 34.0,
                                _ if v.id().contains("lava") => 34.0,
                                _ => 0.0,
                            };
                            if heat > near_heat {
                                near_heat = heat;
                                if near_heat >= 34.0 {
                                    break 'heat_scan;
                                }
                            }
                        }
                    }
                }
            }
            target += near_heat;
        }

        // 6. Apply racial Cold / Heat Resistance toward comfortable 50.0
        let attrs = selected_subrace
            .map_or(super::CharacterSubrace::default(), |s| s.0)
            .attributes();
        if target < 50.0 {
            let cold_delta = 50.0 - target;
            target = 50.0 - cold_delta * (1.0 - attrs.cold_resistance.clamp(0.0, 0.9));
        } else if target > 50.0 {
            let heat_delta = target - 50.0;
            target = 50.0 + heat_delta * (1.0 - attrs.heat_resistance.clamp(0.0, 0.9));
        }

        *cached_target_temp = Some(target.clamp(0.0, 100.0));
    }

    if let Some(target) = *cached_target_temp {
        let blend = 1.0 - (-0.30 * dt).exp();
        vitals.temperature =
            (vitals.temperature + (target - vitals.temperature) * blend).clamp(0.0, 100.0);
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_survival_hud_ui(
    time: Res<Time>,
    game_mode: Res<GameMode>,
    vitals: Res<PlayerVitals>,
    menu_state: Option<Res<State<MenuState>>>,
    inspector: Option<Res<InspectorInteraction>>,
    mut rings: ResMut<VitalRingTextures>,
    mut images: ResMut<Assets<Image>>,
    mut root_q: Query<&mut Visibility, With<SurvivalHudRoot>>,
    mut hp_fill_q: Query<
        &mut Node,
        (
            With<HealthBarFill>,
            Without<StaminaBarFill>,
            Without<OxygenRingContainer>,
        ),
    >,
    mut stam_fill_q: Query<
        &mut Node,
        (
            With<StaminaBarFill>,
            Without<HealthBarFill>,
            Without<OxygenRingContainer>,
        ),
    >,
    mut oxy_cont_q: Query<
        &mut Node,
        (
            With<OxygenRingContainer>,
            Without<HealthBarFill>,
            Without<StaminaBarFill>,
        ),
    >,
    mut hp_text_q: Query<&mut Text, (With<HealthBarText>, Without<StaminaBarText>)>,
    mut stam_text_q: Query<&mut Text, (With<StaminaBarText>, Without<HealthBarText>)>,
    mut climate_badge_q: Query<(&HudClimateBadge, &mut BackgroundColor, &mut BorderColor)>,
    mut climate_icon_q: Query<(&HudClimateIcon, &mut ImageNode)>,
) {
    let show = *game_mode == GameMode::Survival
        && menu_state.is_none_or(|s| *s.get() == MenuState::None)
        && !inspector.is_some_and(|i| i.active);

    for mut vis in &mut root_q {
        let next = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *vis != next {
            *vis = next;
        }
    }

    if !show {
        return;
    }

    let hp_pct = ((vitals.health / vitals.max_health.max(1.0)) * 100.0).clamp(0.0, 100.0);
    let stam_pct = ((vitals.stamina / vitals.max_stamina.max(1.0)) * 100.0).clamp(0.0, 100.0);
    let oxy_pct = ((vitals.oxygen / vitals.max_oxygen.max(1.0)) * 100.0).clamp(0.0, 100.0);

    for mut n in &mut hp_fill_q {
        n.width = percent(hp_pct);
    }
    for mut n in &mut stam_fill_q {
        n.width = percent(stam_pct);
    }
    for mut n in &mut oxy_cont_q {
        n.display = if oxy_pct < 99.5 {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Update circular progress ring textures only when integer percentage changes
    let hunger_i = vitals.hunger.round().clamp(0.0, 100.0) as i32;
    if hunger_i != rings.last_hunger_pct {
        rings.last_hunger_pct = hunger_i;
        let rgb = if hunger_i < 25 {
            [224, 82, 54]
        } else {
            [232, 168, 52]
        };
        if let Some(mut img) = images.get_mut(&rings.hunger)
            && let Some(ref mut data) = img.data
        {
            write_ring_pixels(data, hunger_i as f32 / 100.0, rgb);
        }
    }

    let thirst_i = vitals.thirst.round().clamp(0.0, 100.0) as i32;
    if thirst_i != rings.last_thirst_pct {
        rings.last_thirst_pct = thirst_i;
        let rgb = if thirst_i < 25 {
            [214, 92, 68]
        } else {
            [64, 182, 242]
        };
        if let Some(mut img) = images.get_mut(&rings.thirst)
            && let Some(ref mut data) = img.data
        {
            write_ring_pixels(data, thirst_i as f32 / 100.0, rgb);
        }
    }

    // Update 3 circular Climate State badges below Stamina (active highlight + pulse on Cold/Heat)
    let active_climate = if vitals.temperature < 30.0 {
        ClimateBadgeKind::Cold
    } else if vitals.temperature > 70.0 {
        ClimateBadgeKind::Heat
    } else {
        ClimateBadgeKind::Neutral
    };
    let elapsed = time.elapsed_secs();
    let cold_sev = ((30.0 - vitals.temperature) / 30.0).clamp(0.0, 1.0);
    let heat_sev = ((vitals.temperature - 70.0) / 30.0).clamp(0.0, 1.0);
    let cold_pulse = 0.5 + 0.5 * (elapsed * (3.2 + cold_sev * 3.8)).sin();
    let heat_pulse = 0.5 + 0.5 * (elapsed * (3.2 + heat_sev * 3.8)).sin();

    for (badge, mut bg, mut border) in &mut climate_badge_q {
        if badge.0 != active_climate {
            bg.0 = Color::srgba(0.05, 0.06, 0.09, 0.55);
            *border = BorderColor::all(Color::srgba(0.22, 0.25, 0.32, 0.50));
            continue;
        }
        match badge.0 {
            ClimateBadgeKind::Cold => {
                bg.0 = Color::srgba(
                    0.07 + 0.10 * cold_pulse,
                    0.18 + 0.18 * cold_pulse,
                    0.28 + 0.22 * cold_pulse,
                    0.86 + 0.12 * cold_pulse,
                );
                *border = BorderColor::all(Color::srgba(
                    0.34 + 0.38 * cold_pulse,
                    0.78 + 0.20 * cold_pulse,
                    0.99,
                    0.76 + 0.24 * cold_pulse,
                ));
            }
            ClimateBadgeKind::Neutral => {
                bg.0 = Color::srgba(0.09, 0.22, 0.16, 0.90);
                *border = BorderColor::all(Color::srgba(0.36, 0.84, 0.54, 0.95));
            }
            ClimateBadgeKind::Heat => {
                bg.0 = Color::srgba(
                    0.28 + 0.24 * heat_pulse,
                    0.10 + 0.10 * heat_pulse,
                    0.05 + 0.04 * heat_pulse,
                    0.86 + 0.12 * heat_pulse,
                );
                *border = BorderColor::all(Color::srgba(
                    0.99,
                    0.42 + 0.34 * heat_pulse,
                    0.16 + 0.14 * heat_pulse,
                    0.76 + 0.24 * heat_pulse,
                ));
            }
        }
    }

    for (icon, mut img) in &mut climate_icon_q {
        img.color = if icon.0 != active_climate {
            Color::srgba(1.0, 1.0, 1.0, 0.28)
        } else {
            match icon.0 {
                ClimateBadgeKind::Cold => Color::srgba(0.92, 0.98, 1.0, 0.82 + 0.18 * cold_pulse),
                ClimateBadgeKind::Neutral => Color::WHITE,
                ClimateBadgeKind::Heat => Color::srgba(1.0, 0.95, 0.90, 0.82 + 0.18 * heat_pulse),
            }
        };
    }

    let oxy_i = oxy_pct.round().clamp(0.0, 100.0) as i32;
    if oxy_i != rings.last_oxy_pct {
        rings.last_oxy_pct = oxy_i;
        let rgb = if oxy_i < 25 {
            [238, 86, 66]
        } else {
            [82, 214, 250]
        };
        if let Some(mut img) = images.get_mut(&rings.oxygen)
            && let Some(ref mut data) = img.data
        {
            write_ring_pixels(data, oxy_i as f32 / 100.0, rgb);
        }
    }

    let hp_str = format!(
        "{} / {}",
        vitals.health.round() as u32,
        vitals.max_health.round() as u32
    );
    for mut t in &mut hp_text_q {
        if t.0 != hp_str {
            t.0 = hp_str.clone();
        }
    }

    let stam_str = format!(
        "{} / {}",
        vitals.stamina.round() as u32,
        vitals.max_stamina.round() as u32
    );
    for mut t in &mut stam_text_q {
        if t.0 != stam_str {
            t.0 = stam_str.clone();
        }
    }
}

fn update_mining_progress_bar(
    breaking: Option<Res<crate::gameplay::BlockBreakingState>>,
    mut root_q: Query<&mut Visibility, With<MiningProgressRoot>>,
    mut fill_q: Query<&mut Node, With<MiningProgressFill>>,
) {
    let progress = breaking
        .as_ref()
        .filter(|b| b.target.is_some())
        .map_or(0.0, |b| b.progress.clamp(0.0, 1.0));
    let show = progress > 0.01;

    for mut vis in &mut root_q {
        let next = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *vis != next {
            *vis = next;
        }
    }

    if show {
        for mut n in &mut fill_q {
            n.width = percent(progress * 100.0);
        }
    }
}

fn update_game_mode_toast(
    time: Res<Time>,
    game_mode: Res<GameMode>,
    loc: Option<Res<Localization>>,
    mut state: ResMut<GameModeToastState>,
    mut root_q: Query<&mut Visibility, With<GameModeToastRoot>>,
    mut pill_q: Query<(&mut BackgroundColor, &mut BorderColor), With<GameModeToastPill>>,
    mut text_q: Query<(&mut Text, &mut TextColor), With<GameModeToastText>>,
) {
    if let Some(custom) = state.pending_custom_message.take() {
        state.timer = TOAST_TOTAL_SECS;
        for (mut txt, _) in &mut text_q {
            txt.0 = custom.clone();
        }
    } else if state.last_mode.is_none() {
        state.last_mode = Some(*game_mode);
        return;
    } else if state.last_mode != Some(*game_mode) {
        state.last_mode = Some(*game_mode);
        state.timer = TOAST_TOTAL_SECS;

        let prefix = loc
            .as_ref()
            .map_or("Game Mode", |l| l.t("hud.gamemode.prefix"));
        let mode_name = loc
            .as_ref()
            .map_or(game_mode.label(), |l| l.t(game_mode.loc_key()));
        let msg = format!("{prefix}: {mode_name}");
        for (mut txt, _) in &mut text_q {
            txt.0 = msg.clone();
        }
    }

    if state.timer <= 0.0 {
        for mut vis in &mut root_q {
            if *vis != Visibility::Hidden {
                *vis = Visibility::Hidden;
            }
        }
        return;
    }

    state.timer = (state.timer - time.delta_secs()).max(0.0);
    let alpha = if state.timer > TOAST_FADE_SECS {
        1.0
    } else {
        (state.timer / TOAST_FADE_SECS).clamp(0.0, 1.0)
    };

    for mut vis in &mut root_q {
        *vis = Visibility::Visible;
    }
    for (mut bg, mut border) in &mut pill_q {
        bg.0 = Color::srgba(0.07, 0.08, 0.11, 0.92 * alpha);
        *border = BorderColor::all(Color::srgba(0.28, 0.31, 0.38, 0.92 * alpha));
    }
    for (_, mut col) in &mut text_q {
        col.0 = Color::srgba(0.92, 0.93, 0.96, alpha);
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_survival_hud_scale(
    game_settings: Option<Res<crate::menu::GameSettings>>,
    mut last_scale: Local<Option<u32>>,
    mut root_q: Query<
        &mut Node,
        (
            With<SurvivalHudRoot>,
            Without<HudRingColumn>,
            Without<HudRingBox>,
            Without<HudRingIcon>,
            Without<HudHealthTrack>,
            Without<HudStaminaTrack>,
            Without<HudClimateRow>,
            Without<HudClimateBadge>,
            Without<HudClimateIcon>,
        ),
    >,
    mut ring_col_q: Query<
        &mut Node,
        (
            With<HudRingColumn>,
            Without<SurvivalHudRoot>,
            Without<HudRingBox>,
            Without<HudRingIcon>,
            Without<HudHealthTrack>,
            Without<HudStaminaTrack>,
            Without<HudClimateRow>,
            Without<HudClimateBadge>,
            Without<HudClimateIcon>,
        ),
    >,
    mut ring_box_q: Query<
        &mut Node,
        (
            With<HudRingBox>,
            Without<SurvivalHudRoot>,
            Without<HudRingColumn>,
            Without<HudRingIcon>,
            Without<HudHealthTrack>,
            Without<HudStaminaTrack>,
            Without<HudClimateRow>,
            Without<HudClimateBadge>,
            Without<HudClimateIcon>,
        ),
    >,
    mut ring_icon_q: Query<
        &mut Node,
        (
            With<HudRingIcon>,
            Without<SurvivalHudRoot>,
            Without<HudRingColumn>,
            Without<HudRingBox>,
            Without<HudHealthTrack>,
            Without<HudStaminaTrack>,
            Without<HudClimateRow>,
            Without<HudClimateBadge>,
            Without<HudClimateIcon>,
        ),
    >,
    mut hp_track_q: Query<
        &mut Node,
        (
            With<HudHealthTrack>,
            Without<SurvivalHudRoot>,
            Without<HudRingColumn>,
            Without<HudRingBox>,
            Without<HudRingIcon>,
            Without<HudStaminaTrack>,
            Without<HudClimateRow>,
            Without<HudClimateBadge>,
            Without<HudClimateIcon>,
        ),
    >,
    mut stam_track_q: Query<
        &mut Node,
        (
            With<HudStaminaTrack>,
            Without<SurvivalHudRoot>,
            Without<HudRingColumn>,
            Without<HudRingBox>,
            Without<HudRingIcon>,
            Without<HudHealthTrack>,
            Without<HudClimateRow>,
            Without<HudClimateBadge>,
            Without<HudClimateIcon>,
        ),
    >,
    mut climate_row_q: Query<
        &mut Node,
        (
            With<HudClimateRow>,
            Without<SurvivalHudRoot>,
            Without<HudRingColumn>,
            Without<HudRingBox>,
            Without<HudRingIcon>,
            Without<HudHealthTrack>,
            Without<HudStaminaTrack>,
            Without<HudClimateBadge>,
            Without<HudClimateIcon>,
        ),
    >,
    mut climate_badge_q: Query<
        &mut Node,
        (
            With<HudClimateBadge>,
            Without<SurvivalHudRoot>,
            Without<HudRingColumn>,
            Without<HudRingBox>,
            Without<HudRingIcon>,
            Without<HudHealthTrack>,
            Without<HudStaminaTrack>,
            Without<HudClimateRow>,
            Without<HudClimateIcon>,
        ),
    >,
    mut climate_icon_q: Query<
        &mut Node,
        (
            With<HudClimateIcon>,
            Without<SurvivalHudRoot>,
            Without<HudRingColumn>,
            Without<HudRingBox>,
            Without<HudRingIcon>,
            Without<HudHealthTrack>,
            Without<HudStaminaTrack>,
            Without<HudClimateRow>,
            Without<HudClimateBadge>,
        ),
    >,
    mut hp_font_q: Query<&mut TextFont, (With<HealthBarText>, Without<StaminaBarText>)>,
    mut stam_font_q: Query<&mut TextFont, (With<StaminaBarText>, Without<HealthBarText>)>,
) {
    let Some(settings) = game_settings else {
        return;
    };
    if *last_scale == Some(settings.gui_scale) {
        return;
    }
    *last_scale = Some(settings.gui_scale);

    let factor = (settings.gui_scale as f32 / 4.0).clamp(0.5, 1.0);

    for mut n in &mut root_q {
        n.left = px(BASE_HUD_LEFT * factor);
        n.bottom = px(BASE_HUD_BOTTOM * factor);
        n.row_gap = px(10.0 * factor);
    }
    for mut n in &mut ring_col_q {
        n.row_gap = px(8.0 * factor);
    }
    for mut n in &mut ring_box_q {
        n.width = px(BASE_RING_DISPLAY_SIZE * factor);
        n.height = px(BASE_RING_DISPLAY_SIZE * factor);
    }
    for mut n in &mut ring_icon_q {
        n.width = px(BASE_RING_ICON_SIZE * factor);
        n.height = px(BASE_RING_ICON_SIZE * factor);
    }
    for mut n in &mut hp_track_q {
        n.width = px(BASE_HEALTH_WIDTH * factor);
        n.height = px(BASE_HEALTH_HEIGHT * factor);
    }
    for mut n in &mut stam_track_q {
        n.width = px(BASE_STAMINA_WIDTH * factor);
        n.height = px(BASE_STAMINA_HEIGHT * factor);
    }
    for mut n in &mut climate_row_q {
        n.column_gap = px(10.0 * factor);
    }
    let badge_size = BASE_CLIMATE_BADGE_SIZE * factor;
    for mut n in &mut climate_badge_q {
        n.width = px(badge_size);
        n.height = px(badge_size);
        n.border_radius = BorderRadius::all(px(badge_size * 0.5));
    }
    for mut n in &mut climate_icon_q {
        n.width = px(BASE_CLIMATE_ICON_SIZE * factor);
        n.height = px(BASE_CLIMATE_ICON_SIZE * factor);
    }
    for mut tf in &mut hp_font_q {
        tf.font_size = FontSize::Px((19.0 * factor).max(12.0));
    }
    for mut tf in &mut stam_font_q {
        tf.font_size = FontSize::Px((16.5 * factor).max(11.0));
    }
}
