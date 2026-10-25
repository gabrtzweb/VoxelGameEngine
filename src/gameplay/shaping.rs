use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

use super::{
    radial_menu::{RadialMenuRoot, RadialMenuState, spawn_radial_menu, update_radial_menu_ui},
    targeting::CurrentTarget,
};
use crate::{
    menu::MenuState,
    player::GameMode,
    simulation::lighting::{VoxelLightRegistry, sync_voxel_light},
    world::{
        ChunkStreamingQueues, VoxelAccess, VoxelWorld, WorldModificationStore, affected_chunks,
    },
};

pub use crate::world::BlockShape;

pub struct ShapingPlugin;

impl Plugin for ShapingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RadialMenuState>().add_systems(
            Update,
            (
                handle_block_shaping,
                handle_block_rotation,
                update_radial_menu_ui,
            ),
        );
    }
}

fn handle_block_shaping(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    control_settings: Option<Res<crate::core::ControlSettings>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    game_mode: Res<GameMode>,
    current_target: Res<CurrentTarget>,
    mut commands: Commands,
    (mut world, mut modifications, mut light_registry, mut queues): (
        ResMut<VoxelWorld>,
        ResMut<WorldModificationStore>,
        ResMut<VoxelLightRegistry>,
        ResMut<ChunkStreamingQueues>,
    ),
    mut radial_state: ResMut<RadialMenuState>,
    menu_state: Option<Res<State<MenuState>>>,
    radial_root_query: Query<Entity, With<RadialMenuRoot>>,
    app_font: Option<Res<crate::core::AppFont>>,
) {
    if menu_state
        .as_ref()
        .is_some_and(|s| *s.get() != MenuState::None)
        || *game_mode != GameMode::Creative
    {
        if radial_state.is_open {
            for entity in &radial_root_query {
                commands.entity(entity).despawn();
            }
        }
        *radial_state = RadialMenuState::default();
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) && radial_state.is_open {
        for entity in &radial_root_query {
            commands.entity(entity).despawn();
        }
        *radial_state = RadialMenuState::default();
        return;
    }

    let cycle_just_pressed = control_settings.as_ref().map_or(
        keyboard.just_pressed(KeyCode::KeyR),
        |c| c.is_action_just_pressed(crate::core::InputAction::ShapeCycle, &keyboard, &mouse),
    );

    // Press key to initiate shaping
    if cycle_just_pressed
        && let Some(target) = current_target.hit
    {
        let origin = target.block_origin;
        if let Some(voxel) = world.get_voxel(origin)
            && !voxel.is_empty()
            && !voxel.is_fluid()
            && !voxel.is_unbreakable()
            && !voxel.is_shape_locked()
        {
            let (current_shape, current_orientation) = world.get_shape(origin);
            radial_state.pressing = true;
            radial_state.hold_timer = 0.0;
            radial_state.mouse_offset = Vec2::ZERO;
            radial_state.target_origin = Some(origin);
            radial_state.target_material = Some(voxel);
            radial_state.initial_shape = current_shape;
            radial_state.selected_shape = current_shape;
            radial_state.current_orientation = current_orientation;
        }
    }

    let cycle_pressed = control_settings.as_ref().map_or(
        keyboard.pressed(KeyCode::KeyR),
        |c| c.is_action_pressed(crate::core::InputAction::ShapeCycle, &keyboard, &mouse),
    );

    // Hold key to open Radial Menu
    if cycle_pressed && radial_state.pressing {
        radial_state.hold_timer += time.delta_secs();

        if radial_state.hold_timer >= 0.20 && !radial_state.is_open {
            radial_state.is_open = true;
            let font_handle = app_font.as_ref().map(|f| f.source());
            spawn_radial_menu(
                &mut commands,
                radial_state.selected_shape,
                font_handle.as_ref(),
            );
        }

        if radial_state.is_open {
            radial_state.mouse_offset += mouse_motion.delta;

            if radial_state.mouse_offset.length() > 20.0 {
                let angle = radial_state
                    .mouse_offset
                    .y
                    .atan2(radial_state.mouse_offset.x);
                let mut rel_angle = angle - (-FRAC_PI_2);
                while rel_angle < 0.0 {
                    rel_angle += TAU;
                }
                while rel_angle >= TAU {
                    rel_angle -= TAU;
                }
                let slice_step = TAU / 4.0;
                let slice_idx =
                    ((rel_angle + (slice_step * 0.5)) / slice_step).floor() as usize % 4;
                radial_state.selected_shape = BlockShape::shapable()[slice_idx];
            }
        }
    }

    let cycle_just_released = control_settings.as_ref().map_or(
        keyboard.just_released(KeyCode::KeyR),
        |c| c.is_action_just_released(crate::core::InputAction::ShapeCycle, &keyboard, &mouse),
    );

    // Release key to apply selection
    if cycle_just_released && radial_state.pressing {
        if radial_state.is_open {
            for entity in &radial_root_query {
                commands.entity(entity).despawn();
            }

            if let Some(origin) = radial_state.target_origin {
                apply_block_shape(
                    &mut commands,
                    &mut world,
                    &mut modifications,
                    &mut light_registry,
                    &mut queues,
                    ShapeModification {
                        origin,
                        shape: radial_state.selected_shape,
                        orientation: 0,
                    },
                );
            }
        } else if radial_state.hold_timer < 0.20
            && let Some(origin) = radial_state.target_origin
        {
            let next_shape = radial_state.initial_shape.next();
            apply_block_shape(
                &mut commands,
                &mut world,
                &mut modifications,
                &mut light_registry,
                &mut queues,
                ShapeModification {
                    origin,
                    shape: next_shape,
                    orientation: 0,
                },
            );
        }

        *radial_state = RadialMenuState::default();
    }
}

fn handle_block_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    control_settings: Option<Res<crate::core::ControlSettings>>,
    game_mode: Res<GameMode>,
    current_target: Res<CurrentTarget>,
    mut commands: Commands,
    (mut world, mut modifications, mut light_registry, mut queues): (
        ResMut<VoxelWorld>,
        ResMut<WorldModificationStore>,
        ResMut<VoxelLightRegistry>,
        ResMut<ChunkStreamingQueues>,
    ),
    menu_state: Option<Res<State<MenuState>>>,
) {
    if menu_state.is_some_and(|s| *s.get() != MenuState::None) || *game_mode != GameMode::Creative {
        return;
    }

    let invert_just_pressed = control_settings.as_ref().map_or(
        keyboard.just_pressed(KeyCode::KeyT),
        |c| c.is_action_just_pressed(crate::core::InputAction::ShapeInvert, &keyboard, &mouse),
    );

    if !invert_just_pressed {
        return;
    }

    let Some(target) = current_target.hit else {
        return;
    };

    let origin = target.block_origin;
    let Some(voxel) = world.get_voxel(origin) else {
        return;
    };
    if voxel.is_empty() || voxel.is_fluid() || voxel.is_unbreakable() || voxel.is_shape_locked() {
        return;
    }

    let (current_shape, current_orientation) = world.get_shape(origin);
    let next_orientation = (current_orientation + 1) % current_shape.orientation_count();

    apply_block_shape(
        &mut commands,
        &mut world,
        &mut modifications,
        &mut light_registry,
        &mut queues,
        ShapeModification {
            origin,
            shape: current_shape,
            orientation: next_orientation,
        },
    );
}

pub struct ShapeModification {
    pub origin: IVec3,
    pub shape: BlockShape,
    pub orientation: u8,
}

fn apply_block_shape(
    commands: &mut Commands,
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    light_registry: &mut VoxelLightRegistry,
    queues: &mut ChunkStreamingQueues,
    req: ShapeModification,
) {
    let Some(current) = world.get_voxel(req.origin) else {
        return;
    };
    if current.is_unbreakable()
        || current.is_empty()
        || current.is_fluid()
        || current.is_shape_locked()
    {
        return;
    }

    world.set_shape(req.origin, req.shape, req.orientation);
    modifications.record_shape(req.origin, req.shape, req.orientation);
    if req.shape != BlockShape::Slab {
        world.set_extra_slab(req.origin, None);
        modifications.record_extra_slab(req.origin, None);
    }

    sync_voxel_light(commands, world, req.origin, light_registry);

    for coordinate in affected_chunks(req.origin) {
        if world.get_chunk(coordinate).is_some() {
            queues.enqueue_priority_remesh(coordinate);
        }
    }
}
