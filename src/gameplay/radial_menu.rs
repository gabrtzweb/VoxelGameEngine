use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;

use super::shaping::BlockShape;

#[derive(Resource, Default)]
pub struct RadialMenuState {
    pub is_open: bool,
    pub pressing: bool,
    pub hold_timer: f32,
    pub mouse_offset: Vec2,
    pub target_origin: Option<IVec3>,
    pub target_material: Option<crate::world::Voxel>,
    pub initial_shape: BlockShape,
    pub selected_shape: BlockShape,
}

#[derive(Component)]
pub struct RadialMenuRoot;

#[derive(Component)]
pub struct RadialMenuSlice(pub usize);

#[derive(Component)]
pub struct RadialMenuTitleText;

#[derive(Component)]
pub struct RadialMenuSubtitleText;

pub fn spawn_radial_menu(commands: &mut Commands, selected_shape: BlockShape) {
    let shapes = BlockShape::all();

    commands
        .spawn((
            RadialMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                top: px(0.0),
                right: px(0.0),
                bottom: px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ZIndex(250),
        ))
        .with_children(|root| {
            // Center wheel hub
            root.spawn(Node {
                position_type: PositionType::Relative,
                width: px(0.0),
                height: px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|hub| {
                // Central Preview Card
                hub.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(-75.0),
                        top: px(-65.0),
                        width: px(150.0),
                        height: px(130.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: px(4.0),
                        padding: UiRect::all(px(8.0)),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.95)),
                    BorderColor::all(Color::srgba(0.35, 0.65, 0.95, 0.8)),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new(selected_shape.name()),
                        TextFont {
                            font_size: FontSize::Px(15.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.4)),
                        RadialMenuTitleText,
                    ));

                    card.spawn((
                        Text::new(format!("{} / 8 Sub-voxels", selected_shape.voxel_count())),
                        TextFont {
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.75, 0.8, 0.9)),
                        RadialMenuSubtitleText,
                    ));

                    card.spawn((
                        Text::new("Hold R + Move Mouse\nRelease R to Select"),
                        TextFont {
                            font_size: FontSize::Px(10.0),
                            ..default()
                        },
                        TextColor(Color::srgba(0.6, 0.8, 1.0, 0.7)),
                    ));
                });

                // 10 Radial Slices
                let radius = 155.0;
                let card_size = 56.0;
                let slice_step = TAU / 10.0;

                for (i, &shape) in shapes.iter().enumerate() {
                    let angle = -FRAC_PI_2 + (i as f32) * slice_step;
                    let cx = angle.cos() * radius - (card_size * 0.5);
                    let cy = angle.sin() * radius - (card_size * 0.5);
                    let is_selected = shape == selected_shape;

                    hub.spawn((
                        RadialMenuSlice(i),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(cx),
                            top: px(cy),
                            width: px(card_size),
                            height: px(card_size),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(if is_selected { px(2.5) } else { px(1.5) }),
                            border_radius: BorderRadius::all(px(8.0)),
                            ..default()
                        },
                        BackgroundColor(if is_selected {
                            Color::srgba(0.2, 0.5, 0.85, 0.95)
                        } else {
                            Color::srgba(0.07, 0.09, 0.13, 0.88)
                        }),
                        BorderColor::all(if is_selected {
                            Color::srgb(0.5, 0.9, 1.0)
                        } else {
                            Color::srgba(0.3, 0.35, 0.45, 0.6)
                        }),
                    ))
                    .with_children(|slice_card| {
                        slice_card.spawn((
                            Text::new(shape.short_name()),
                            TextFont {
                                font_size: FontSize::Px(10.0),
                                ..default()
                            },
                            TextColor(if is_selected {
                                Color::srgb(1.0, 1.0, 1.0)
                            } else {
                                Color::srgb(0.8, 0.85, 0.9)
                            }),
                        ));

                        slice_card.spawn((
                            Text::new(format!("{}v", shape.voxel_count())),
                            TextFont {
                                font_size: FontSize::Px(9.0),
                                ..default()
                            },
                            TextColor(Color::srgba(0.6, 0.65, 0.75, 0.8)),
                        ));
                    });
                }
            });
        });
}

pub fn update_radial_menu_ui(
    radial_state: Res<RadialMenuState>,
    mut slice_query: Query<(&RadialMenuSlice, &mut BackgroundColor, &mut BorderColor)>,
    mut title_query: Query<&mut Text, (With<RadialMenuTitleText>, Without<RadialMenuSubtitleText>)>,
    mut sub_query: Query<&mut Text, (With<RadialMenuSubtitleText>, Without<RadialMenuTitleText>)>,
) {
    if !radial_state.is_open {
        return;
    }

    let shapes = BlockShape::all();
    let selected_idx = shapes
        .iter()
        .position(|&s| s == radial_state.selected_shape)
        .unwrap_or(0);

    for (slice, mut bg, mut border) in &mut slice_query {
        let is_selected = slice.0 == selected_idx;
        if is_selected {
            bg.0 = Color::srgba(0.2, 0.5, 0.85, 0.95);
            *border = BorderColor::all(Color::srgb(0.5, 0.9, 1.0));
        } else {
            bg.0 = Color::srgba(0.07, 0.09, 0.13, 0.88);
            *border = BorderColor::all(Color::srgba(0.3, 0.35, 0.45, 0.6));
        }
    }

    for mut text in &mut title_query {
        text.0 = radial_state.selected_shape.name().to_string();
    }

    for mut text in &mut sub_query {
        text.0 = format!(
            "{} / 8 Sub-voxels",
            radial_state.selected_shape.voxel_count()
        );
    }
}
