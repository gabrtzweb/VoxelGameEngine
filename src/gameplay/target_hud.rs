use bevy::prelude::*;

use super::{BlockIcons, CurrentTarget, TargetingSet, VoxelTarget};
use crate::{
    menu::MenuState,
    world::{BlockShape, Voxel, VoxelAccess, VoxelWorld},
};

#[derive(Component)]
pub struct TargetHudRoot;

#[derive(Component)]
#[allow(dead_code)]
pub struct TargetHudCard;

#[derive(Component)]
pub struct TargetHudIcon;

#[derive(Component)]
pub struct TargetHudTitle;

#[derive(Component)]
#[allow(dead_code)]
pub struct TargetHudSubtitle;

pub struct TargetHudPlugin;

impl Plugin for TargetHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_target_hud.after(crate::gameplay::setup_block_icons),
        )
        .add_systems(Update, update_target_hud.after(TargetingSet::UpdateTarget));
    }
}

/// Resolves the canonical voxel and detected shape for a given voxel target.
pub fn resolve_target_block_info(
    world: &impl VoxelAccess,
    target: VoxelTarget,
) -> Option<(Voxel, Option<(BlockShape, u8)>)> {
    let raw_voxel = world.get_voxel(target.hit_voxel)?;
    if raw_voxel.is_empty() {
        return None;
    }

    let shape = if raw_voxel.is_fluid() {
        None
    } else {
        let (shape, orientation) = world.get_shape(target.block_origin);
        Some((shape, orientation))
    };

    Some((raw_voxel, shape))
}

/// Formats the target block title, appending shape and orientation name if not a standard full block.
pub fn format_target_hud_title(voxel: Voxel, shape: Option<(BlockShape, u8)>) -> String {
    match shape {
        Some((s, orientation)) if s != BlockShape::Full && !voxel.is_fluid() => {
            format!(
                "{} ({} - {})",
                voxel.label(),
                s.name(),
                s.orientation_name(orientation)
            )
        }
        _ => voxel.label().to_string(),
    }
}

fn setup_target_hud(
    mut commands: Commands,
    icons: Res<BlockIcons>,
    app_font: Option<Res<crate::core::AppFont>>,
) {
    let font_handle = app_font.as_ref().map(|f| f.source());

    let mut title_font = TextFont {
        font_size: FontSize::Px(14.0),
        ..default()
    };
    let mut subtitle_font = TextFont {
        font_size: FontSize::Px(11.0),
        ..default()
    };
    if let Some(ref font) = font_handle {
        title_font.font = font.clone();
        subtitle_font.font = font.clone();
    }

    commands
        .spawn((
            TargetHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: px(16.0),
                left: px(0.0),
                right: px(0.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            ZIndex(100),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    TargetHudCard,
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(10.0),
                        padding: UiRect::axes(px(12.0), px(8.0)),
                        border: UiRect::all(px(1.5)),
                        border_radius: BorderRadius::all(px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.06, 0.09, 0.88)),
                    BorderColor::all(Color::srgba(0.28, 0.28, 0.34, 0.85)),
                ))
                .with_children(|card| {
                    // 3D Isometric block icon
                    card.spawn((
                        TargetHudIcon,
                        ImageNode {
                            image: icons.get(Voxel::Stone),
                            ..default()
                        },
                        Node {
                            width: px(32.0),
                            height: px(32.0),
                            ..default()
                        },
                    ));

                    // Title & Mod/Engine subtitle
                    card.spawn((Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        row_gap: px(2.0),
                        ..default()
                    },))
                        .with_children(|col| {
                            col.spawn((
                                TargetHudTitle,
                                Text::new(""),
                                title_font,
                                TextColor(Color::srgb(0.96, 0.96, 0.98)),
                                crate::core::text_shadow_default(),
                            ));

                            col.spawn((
                                TargetHudSubtitle,
                                Text::new("Voxel Engine"),
                                subtitle_font,
                                TextColor(Color::srgb(0.42, 0.65, 0.96)),
                                crate::core::text_shadow_default(),
                            ));
                        });
                });
        });
}

fn update_target_hud(
    current_target: Res<CurrentTarget>,
    world: Res<VoxelWorld>,
    icons: Res<BlockIcons>,
    menu_state: Option<Res<State<MenuState>>>,
    mut root_query: Query<&mut Visibility, With<TargetHudRoot>>,
    mut icon_query: Query<&mut ImageNode, With<TargetHudIcon>>,
    mut title_query: Query<&mut Text, With<TargetHudTitle>>,
) {
    // Hide HUD if a menu is open
    if menu_state.is_some_and(|s| *s.get() != MenuState::None) {
        for mut vis in &mut root_query {
            if *vis != Visibility::Hidden {
                *vis = Visibility::Hidden;
            }
        }
        return;
    }

    // Hide HUD if not targeting anything
    let Some(target) = current_target.hit else {
        for mut vis in &mut root_query {
            if *vis != Visibility::Hidden {
                *vis = Visibility::Hidden;
            }
        }
        return;
    };

    // Resolve targeted block info
    let Some((voxel, shape)) = resolve_target_block_info(&*world, target) else {
        for mut vis in &mut root_query {
            if *vis != Visibility::Hidden {
                *vis = Visibility::Hidden;
            }
        }
        return;
    };

    // Show HUD
    for mut vis in &mut root_query {
        if *vis != Visibility::Visible {
            *vis = Visibility::Visible;
        }
    }

    // Update icon image
    for mut icon in &mut icon_query {
        let handle = icons.get(voxel);
        if icon.image != handle {
            icon.image = handle;
        }
    }

    // Update title text
    let new_title = format_target_hud_title(voxel, shape);
    for mut text in &mut title_query {
        if text.0 != new_title {
            text.0 = new_title.clone();
        }
    }
}
