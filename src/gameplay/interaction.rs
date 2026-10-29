use bevy::prelude::*;

use super::{
    feedback::{BlockBreakEvent, BlockPlaceEvent},
    targeting::{CurrentTarget, TargetingSet},
};
use crate::{
    menu::MenuState,
    player::{GameMode, InspectorInteraction, PlayerCamera, hotbar::Hotbar},
    simulation::{
        fluid::FluidUpdateQueue,
        lighting::{VoxelLightRegistry, sync_voxel_light},
    },
    world::{
        BlockShape, ChunkStreamingQueues, Voxel, VoxelWorld, WorldModificationStore,
        affected_chunks,
    },
};

const HOLD_DELAY: f32 = 0.25;
const REPEAT_INTERVAL: f32 = 0.16;

type WorldEditResources<'w> = (
    ResMut<'w, VoxelWorld>,
    ResMut<'w, WorldModificationStore>,
    ResMut<'w, VoxelLightRegistry>,
    ResMut<'w, ChunkStreamingQueues>,
);

type OptionalInteractionParams<'w> = (
    Option<ResMut<'w, FluidUpdateQueue>>,
    Option<Res<'w, State<MenuState>>>,
    Option<Res<'w, InspectorInteraction>>,
    Option<ResMut<'w, crate::map::MapCache>>,
);

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedVoxel(pub Option<Voxel>);

impl Default for SelectedVoxel {
    fn default() -> Self {
        Self(Some(Voxel::LoamGrass))
    }
}

#[derive(Default)]
struct HoldActionState {
    hold_time: f32,
    repeat_time: f32,
}

impl HoldActionState {
    fn update(&mut self, pressed: bool, just_pressed: bool, delta_seconds: f32) -> bool {
        if just_pressed {
            self.hold_time = 0.0;
            self.repeat_time = 0.0;
            return true;
        }

        if !pressed {
            self.hold_time = 0.0;
            self.repeat_time = 0.0;
            return false;
        }

        self.hold_time += delta_seconds;

        if self.hold_time < HOLD_DELAY {
            return false;
        }

        self.repeat_time += delta_seconds;

        if self.repeat_time >= REPEAT_INTERVAL {
            self.repeat_time -= REPEAT_INTERVAL;
            return true;
        }

        false
    }
}

#[derive(Default)]
struct InteractionState {
    break_action: HoldActionState,
    place_action: HoldActionState,
}

pub struct VoxelInteractionPlugin;

impl Plugin for VoxelInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedVoxel>().add_systems(
            Update,
            (pick_targeted_voxel, edit_voxels)
                .chain()
                .after(TargetingSet::UpdateTarget),
        );
    }
}

fn pick_targeted_voxel(
    (game_mode, current_target): (Res<GameMode>, Res<CurrentTarget>),
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    control_settings: Option<Res<crate::core::ControlSettings>>,
    (menu_state, inspector): (
        Option<Res<State<MenuState>>>,
        Option<Res<InspectorInteraction>>,
    ),
    world: Res<VoxelWorld>,
    mut selected: ResMut<SelectedVoxel>,
    hotbar: Option<ResMut<Hotbar>>,
) {
    if menu_state.is_some_and(|s| *s.get() != MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }

    if *game_mode != GameMode::Creative {
        return;
    }

    let is_pick = control_settings.as_ref().map_or(
        mouse.just_pressed(MouseButton::Middle),
        |c| c.is_action_just_pressed(crate::core::InputAction::PickBlock, &keyboard, &mouse),
    );

    if !is_pick {
        return;
    }

    let Some(target) = current_target.hit else {
        return;
    };

    let Some(voxel) = world.get_voxel(target.hit_voxel) else {
        return;
    };

    if voxel.is_empty() {
        return;
    }

    let pick_voxel = voxel.double_tall_bottom_partner().unwrap_or(voxel);

    selected.0 = Some(pick_voxel);
    if let Some(mut hotbar) = hotbar {
        let active = hotbar.active_slot;
        hotbar.slots[active] = Some(pick_voxel);
    }

    info!("Selected voxel: {}", pick_voxel.label());
}

fn edit_voxels(
    (game_mode, selected): (Res<GameMode>, Res<SelectedVoxel>),
    mut commands: Commands,
    (keyboard, mouse, time, current_target): (
        Res<ButtonInput<KeyCode>>,
        Res<ButtonInput<MouseButton>>,
        Res<Time>,
        Res<CurrentTarget>,
    ),
    control_settings: Option<Res<crate::core::ControlSettings>>,
    (mut world, mut modifications, mut light_registry, mut queues): WorldEditResources,
    (fluid_queue, menu_state, inspector, mut map_cache): OptionalInteractionParams,
    camera_query: Query<&Transform, With<PlayerCamera>>,
    mut interaction_state: Local<InteractionState>,
) {
    if menu_state.is_some_and(|s| *s.get() != MenuState::None)
        || inspector.is_some_and(|i| i.active)
    {
        return;
    }

    if *game_mode == GameMode::Spectator {
        return;
    }

    let delta_seconds = time.delta_secs();

    let break_pressed = control_settings.as_ref().map_or(
        mouse.pressed(MouseButton::Left),
        |c| c.is_action_pressed(crate::core::InputAction::PrimaryAction, &keyboard, &mouse),
    );
    let break_just_pressed = control_settings.as_ref().map_or(
        mouse.just_pressed(MouseButton::Left),
        |c| c.is_action_just_pressed(crate::core::InputAction::PrimaryAction, &keyboard, &mouse),
    );

    let place_pressed = control_settings.as_ref().map_or(
        mouse.pressed(MouseButton::Right),
        |c| c.is_action_pressed(crate::core::InputAction::SecondaryAction, &keyboard, &mouse),
    );
    let place_just_pressed = control_settings.as_ref().map_or(
        mouse.just_pressed(MouseButton::Right),
        |c| c.is_action_just_pressed(crate::core::InputAction::SecondaryAction, &keyboard, &mouse),
    );

    let break_action = interaction_state.break_action.update(
        break_pressed,
        break_just_pressed,
        delta_seconds,
    );

    let place_action = interaction_state.place_action.update(
        place_pressed,
        place_just_pressed,
        delta_seconds,
    );

    if !break_action && !place_action {
        return;
    }

    let Some(target) = current_target.hit else {
        return;
    };

    let (edited_voxels, broken_voxel) = if break_action {
        let prev_voxel = world.get_voxel(target.hit_voxel);
        let has_extra = world.get_extra_slab(target.hit_voxel).is_some();
        let (edited, broken) = if has_extra {
            // Remove the extra slab first, leaving the base slab intact
            let extra = world.get_extra_slab(target.hit_voxel).map(|(v, _)| v);
            world.set_extra_slab(target.hit_voxel, None);
            modifications.record_extra_slab(target.hit_voxel, None);
            (vec![target.hit_voxel], extra)
        } else {
            let edited = remove_block(&mut world, &mut modifications, target.hit_voxel);
            (edited, prev_voxel)
        };
        (edited, broken)
    } else if place_action {
        let Some(place_voxel_type) = selected.0 else {
            return;
        };

        let Some(place_position) = target.place_voxel else {
            return;
        };

        let (hit_shape, hit_orientation) = world.get_shape(target.hit_voxel);
        let existing_voxel = world.get_voxel(target.hit_voxel);
        let existing_extra = world.get_extra_slab(target.hit_voxel);

        if place_voxel_type == Voxel::VegeLilyPad {
            // Lily pads can ONLY be placed on water surface (+Y face of a water voxel)
            let is_on_water = target.face_normal == IVec3::Y
                && existing_voxel.is_some_and(Voxel::is_water);
            if !is_on_water {
                return;
            }
        }

        let edited = if existing_voxel.is_some_and(Voxel::is_torch) {
            // Cannot stack or place anything against or on top of a torch
            Vec::new()
        } else if target.face_normal == IVec3::Y
            && hit_shape == BlockShape::Slab
            && hit_orientation == 0
            && existing_extra.is_none()
        {
            // Clicking on top of a bottom slab that has no upper slab yet
            if Some(place_voxel_type) == existing_voxel {
                // Same material: completes the bottom slab into a full block of this material
                world.set_shape(target.hit_voxel, BlockShape::Full, 0);
                modifications.record_shape(target.hit_voxel, BlockShape::Full, 0);
            } else {
                // Different material: place new material as a top slab within the exact same block space!
                world.set_extra_slab(target.hit_voxel, Some((place_voxel_type, 1)));
                modifications.record_extra_slab(target.hit_voxel, Some((place_voxel_type, 1)));
            }
            vec![target.hit_voxel]
        } else if target.face_normal == -IVec3::Y
            && hit_shape == BlockShape::Slab
            && hit_orientation == 1
            && existing_extra.is_none()
        {
            // Clicking underneath a ceiling slab that has no bottom slab yet
            if Some(place_voxel_type) == existing_voxel {
                world.set_shape(target.hit_voxel, BlockShape::Full, 0);
                modifications.record_shape(target.hit_voxel, BlockShape::Full, 0);
            } else {
                world.set_extra_slab(target.hit_voxel, Some((place_voxel_type, 0)));
                modifications.record_extra_slab(target.hit_voxel, Some((place_voxel_type, 0)));
            }
            vec![target.hit_voxel]
        } else if place_voxel_type.is_torch() {
            // Torch placement: check orientation and support
            let supporting_solid =
                existing_voxel.is_some_and(|v| !v.is_empty() && !v.is_fluid() && !v.is_torch());
            if !supporting_solid {
                Vec::new()
            } else if target.face_normal == -IVec3::Y {
                // Cannot place torch on ceiling
                Vec::new()
            } else {
                let torch_orient = match target.face_normal {
                    IVec3::Y => 0,     // Floor standing
                    IVec3::Z => 1,     // Wall North (-Z)
                    IVec3::NEG_Z => 2, // Wall South (+Z)
                    IVec3::X => 3,     // Wall West (-X)
                    IVec3::NEG_X => 4, // Wall East (+X)
                    _ => 0,
                };
                let placed = place_block(
                    &mut world,
                    &mut modifications,
                    place_position,
                    place_voxel_type,
                );
                if !placed.is_empty() {
                    world.set_shape(place_position, BlockShape::Torch, torch_orient);
                    modifications.record_shape(place_position, BlockShape::Torch, torch_orient);
                }
                placed
            }
        } else if place_voxel_type == Voxel::VegeCaveVines || place_voxel_type == Voxel::VegeCaveVinesPlant {
            if target.face_normal == IVec3::Y {
                return; // cannot place hanging vines on floor
            }
            let above_v = world.get_voxel(place_position + IVec3::Y);
            let has_support = above_v.is_some_and(|v| {
                v.is_solid_opaque() || v == Voxel::VegeCaveVines || v == Voxel::VegeCaveVinesPlant
            });
            if !has_support {
                return;
            }
            let mut placed = place_block(
                &mut world,
                &mut modifications,
                place_position,
                Voxel::VegeCaveVines, // bottom tip
            );
            if !placed.is_empty() {
                world.set_shape(place_position, BlockShape::CrossBillboard, 0);
                modifications.record_shape(place_position, BlockShape::CrossBillboard, 0);
                if let Some(Voxel::VegeCaveVines) = above_v {
                    let above_pos = place_position + IVec3::Y;
                    world.set_voxel(above_pos, Voxel::VegeCaveVinesPlant);
                    modifications.record(above_pos, Voxel::VegeCaveVinesPlant);
                    placed.push(above_pos);
                }
            }
            placed
        } else if place_voxel_type == Voxel::Kelp || place_voxel_type == Voxel::KelpPlant
            || place_voxel_type == Voxel::BullKelp || place_voxel_type == Voxel::BullKelpPlant
        {
            let below_v = world.get_voxel(place_position - IVec3::Y);
            let is_kelp = place_voxel_type == Voxel::Kelp || place_voxel_type == Voxel::KelpPlant;
            let (tip_voxel, plant_voxel) = if is_kelp {
                (Voxel::Kelp, Voxel::KelpPlant)
            } else {
                (Voxel::BullKelp, Voxel::BullKelpPlant)
            };
            let has_support = below_v.is_some_and(|v| {
                v.is_solid_opaque() || v == tip_voxel || v == plant_voxel
            });
            if !has_support {
                return;
            }
            let mut placed = place_block(
                &mut world,
                &mut modifications,
                place_position,
                tip_voxel,
            );
            if !placed.is_empty() {
                world.set_shape(place_position, BlockShape::CrossBillboard, 0);
                modifications.record_shape(place_position, BlockShape::CrossBillboard, 0);
                if let Some(bv) = below_v {
                    if bv == tip_voxel {
                        let below_pos = place_position - IVec3::Y;
                        world.set_voxel(below_pos, plant_voxel);
                        modifications.record(below_pos, plant_voxel);
                        placed.push(below_pos);
                    }
                }
            }
            placed
        } else if place_voxel_type.default_shape() == BlockShape::WallDecal {
            let orient = match target.face_normal {
                IVec3::Y => 0,     // Floor
                IVec3::NEG_Y => 1, // Ceiling
                IVec3::Z => 2,     // Attached to South face of hit_voxel -> North wall (-Z) of place_position
                IVec3::NEG_Z => 3, // Attached to North face of hit_voxel -> South wall (+Z) of place_position
                IVec3::X => 4,     // Attached to East face of hit_voxel -> West wall (-X) of place_position
                IVec3::NEG_X => 5, // Attached to West face of hit_voxel -> East wall (+X) of place_position
                _ => 2,
            };
            let placed = place_block(
                &mut world,
                &mut modifications,
                place_position,
                place_voxel_type,
            );
            if !placed.is_empty() {
                world.set_shape(place_position, BlockShape::WallDecal, orient);
                modifications.record_shape(place_position, BlockShape::WallDecal, orient);
            }
            placed
        } else if place_voxel_type.is_basket() {
            let placed = place_block(
                &mut world,
                &mut modifications,
                place_position,
                place_voxel_type,
            );
            if !placed.is_empty() {
                world.set_shape(place_position, BlockShape::Basket, 0);
                modifications.record_shape(place_position, BlockShape::Basket, 0);
            }
            placed
        } else if place_voxel_type.is_pebble() {
            if target.face_normal != IVec3::Y {
                return;
            }
            let placed = place_block(
                &mut world,
                &mut modifications,
                place_position,
                place_voxel_type,
            );
            if !placed.is_empty() {
                let pebble_shape = place_voxel_type.default_shape();
                let orient = ((place_position.x.wrapping_mul(31) ^ place_position.z.wrapping_mul(17)) & 3).unsigned_abs() as u8;
                world.set_shape(place_position, pebble_shape, orient);
                modifications.record_shape(place_position, pebble_shape, orient);
            }
            placed
        } else if place_voxel_type.is_bamboo() {
            if target.face_normal != IVec3::Y {
                return;
            }
            let placed = place_block(
                &mut world,
                &mut modifications,
                place_position,
                place_voxel_type,
            );
            if !placed.is_empty() {
                world.set_shape(place_position, BlockShape::Bamboo, 0);
                modifications.record_shape(place_position, BlockShape::Bamboo, 0);
            }
            placed
        } else if place_voxel_type.is_axis_aligned() {
            let placed = place_block(
                &mut world,
                &mut modifications,
                place_position,
                place_voxel_type,
            );
            if !placed.is_empty() {
                let orientation = if target.face_normal.x != 0 {
                    1 // Axis X (horizontal East-West)
                } else if target.face_normal.z != 0 {
                    2 // Axis Z (horizontal North-South)
                } else {
                    0 // Axis Y (vertical Upright)
                };
                world.set_shape(place_position, BlockShape::Full, orientation);
                modifications.record_shape(place_position, BlockShape::Full, orientation);
            }
            placed
        } else if place_voxel_type.front_texture_name().is_some() {
            let placed = place_block(
                &mut world,
                &mut modifications,
                place_position,
                place_voxel_type,
            );
            if !placed.is_empty() {
                let orientation = if let Ok(cam_tf) = camera_query.single() {
                    let fwd = cam_tf.forward();
                    // Block front face points towards the player (opposite to camera horizontal forward)
                    if fwd.z.abs() >= fwd.x.abs() {
                        if fwd.z < 0.0 {
                            0 // Player looks North (-Z) -> block front faces South (+Z, towards player)
                        } else {
                            2 // Player looks South (+Z) -> block front faces North (-Z, towards player)
                        }
                    } else if fwd.x > 0.0 {
                        3 // Player looks East (+X) -> block front faces West (-X, towards player)
                    } else {
                        1 // Player looks West (-X) -> block front faces East (+X, towards player)
                    }
                } else if target.face_normal.x > 0 {
                    1
                } else if target.face_normal.x < 0 {
                    3
                } else if target.face_normal.z < 0 {
                    2
                } else {
                    0
                };
                world.set_shape(place_position, BlockShape::Full, orientation);
                modifications.record_shape(place_position, BlockShape::Full, orientation);
            }
            placed
        } else {
            let placed = place_block(
                &mut world,
                &mut modifications,
                place_position,
                place_voxel_type,
            );
            if !placed.is_empty() {
                for &p in &placed {
                    if let Some(v) = world.get_voxel(p) {
                        let def_shape = v.default_shape();
                        if def_shape != BlockShape::Full {
                            world.set_shape(p, def_shape, 0);
                            modifications.record_shape(p, def_shape, 0);
                        }
                    }
                }
            }
            placed
        };
        (edited, None)
    } else {
        (Vec::new(), None)
    };

    if edited_voxels.is_empty() {
        return;
    }

    if break_action {
        if let Some(v) = broken_voxel {
            commands.trigger(BlockBreakEvent {
                position: target.hit_voxel,
                voxel: v,
            });
        }
    } else if place_action
        && let (Some(place_voxel_type), Some(place_position)) = (selected.0, target.place_voxel)
    {
        let event_pos = if edited_voxels.contains(&target.hit_voxel)
            && !edited_voxels.contains(&place_position)
        {
            target.hit_voxel
        } else {
            place_position
        };
        commands.trigger(BlockPlaceEvent {
            position: event_pos,
            voxel: place_voxel_type,
        });
    }

    if let Some(mut fq) = fluid_queue {
        for &edited_voxel in &edited_voxels {
            fq.enqueue_with_neighbors(edited_voxel);
        }
    }

    let mut dirty_chunks = Vec::new();

    if let Some(ref mut cache) = map_cache {
        for &edited_voxel in &edited_voxels {
            cache.mark_block_dirty(edited_voxel);
        }
    }

    for &edited_voxel in &edited_voxels {
        sync_voxel_light(&mut commands, &world, edited_voxel, &mut light_registry);
        dirty_chunks.extend(affected_chunks(edited_voxel));
    }

    dirty_chunks.sort_by_key(|chunk| (chunk.x, chunk.y, chunk.z));
    dirty_chunks.dedup();

    for coordinate in dirty_chunks {
        if world.get_chunk(coordinate).is_some() {
            queues.enqueue_priority_remesh(coordinate);
        }
    }
}

pub fn remove_voxel(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    world_voxel: IVec3,
) -> bool {
    let Some(current_voxel) = world.get_voxel(world_voxel) else {
        return false;
    };

    if current_voxel.is_empty() || current_voxel.is_unbreakable() {
        return false;
    }

    let replacement = if current_voxel == Voxel::WaterOccupied {
        Voxel::LiquidWater
    } else {
        Voxel::Air
    };

    world.set_voxel(world_voxel, replacement);
    world.set_fluid_level(world_voxel, 0);
    world.set_shape(world_voxel, BlockShape::Full, 0);
    world.set_extra_slab(world_voxel, None);
    modifications.record(world_voxel, replacement);
    modifications.record_fluid_level(world_voxel, 0);
    modifications.record_shape(world_voxel, BlockShape::Full, 0);
    modifications.record_extra_slab(world_voxel, None);

    true
}

pub fn place_voxel(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    world_voxel: IVec3,
    voxel: Voxel,
) -> bool {
    let Some(current_voxel) = world.get_voxel(world_voxel) else {
        return false;
    };

    if !current_voxel.is_empty() && !current_voxel.is_fluid() {
        return false;
    }

    let final_voxel = if current_voxel.is_water() && voxel == Voxel::Occupied {
        Voxel::WaterOccupied
    } else {
        voxel
    };

    world.set_voxel(world_voxel, final_voxel);
    world.set_fluid_level(world_voxel, 0);
    modifications.record(world_voxel, final_voxel);
    modifications.record_fluid_level(world_voxel, 0);

    true
}

pub fn remove_block(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    block_pos: IVec3,
) -> Vec<IVec3> {
    let target_voxel = world.get_voxel(block_pos);
    if !remove_voxel(world, modifications, block_pos) {
        return Vec::new();
    }

    let mut removed = vec![block_pos];

    // If breaking the top half of a double-tall plant, also break the bottom partner
    if let Some(tv) = target_voxel {
        if tv.is_double_tall_top() {
            let bottom_pos = block_pos - IVec3::Y;
            if let Some(bv) = world.get_voxel(bottom_pos) {
                if tv.double_tall_bottom_partner() == Some(bv) {
                    if remove_voxel(world, modifications, bottom_pos) {
                        removed.push(bottom_pos);
                    }
                }
            }
        }
    }

    // 1. Blocks resting on top of this block (and cascading upwards for stacks like bamboo)
    let mut check_pos = block_pos + IVec3::Y;
    while let Some(v) = world.get_voxel(check_pos) {
        if v.requires_floor_support() {
            if remove_voxel(world, modifications, check_pos) {
                removed.push(check_pos);
                check_pos += IVec3::Y;
            } else {
                break;
            }
        } else if v.is_torch() {
            let (s, o) = world.get_shape(check_pos);
            if s == BlockShape::Torch && o == 0 && remove_voxel(world, modifications, check_pos) {
                removed.push(check_pos);
            }
            break;
        } else if v.is_bamboo() {
            if remove_voxel(world, modifications, check_pos) {
                removed.push(check_pos);
                check_pos += IVec3::Y;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Kelp cascading upwards
    let mut kelp_up_pos = block_pos + IVec3::Y;
    while let Some(v) = world.get_voxel(kelp_up_pos) {
        if v == Voxel::Kelp || v == Voxel::KelpPlant || v == Voxel::BullKelp || v == Voxel::BullKelpPlant {
            if remove_voxel(world, modifications, kelp_up_pos) {
                removed.push(kelp_up_pos);
                kelp_up_pos += IVec3::Y;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // If block directly below this was a kelp plant, it is now the top tip
    let kelp_below_pos = block_pos - IVec3::Y;
    if let Some(v) = world.get_voxel(kelp_below_pos) {
        if v == Voxel::KelpPlant {
            world.set_voxel(kelp_below_pos, Voxel::Kelp);
            modifications.record(kelp_below_pos, Voxel::Kelp);
            removed.push(kelp_below_pos);
        } else if v == Voxel::BullKelpPlant {
            world.set_voxel(kelp_below_pos, Voxel::BullKelp);
            modifications.record(kelp_below_pos, Voxel::BullKelp);
            removed.push(kelp_below_pos);
        }
    }

    // Cave vines hanging below this block (cascade downwards)
    let mut vine_down_pos = block_pos - IVec3::Y;
    while let Some(v) = world.get_voxel(vine_down_pos) {
        if v == Voxel::VegeCaveVines || v == Voxel::VegeCaveVinesPlant {
            if remove_voxel(world, modifications, vine_down_pos) {
                removed.push(vine_down_pos);
                vine_down_pos -= IVec3::Y;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // If block directly above this was a cave vines plant, it is now the bottom tip
    let vine_above_pos = block_pos + IVec3::Y;
    if let Some(Voxel::VegeCaveVinesPlant) = world.get_voxel(vine_above_pos) {
        world.set_voxel(vine_above_pos, Voxel::VegeCaveVines);
        modifications.record(vine_above_pos, Voxel::VegeCaveVines);
        removed.push(vine_above_pos);
    }

    // 2. Wall torches attached to this block:
    // South neighbor (+Z) has torch attached to North wall of block_pos (orientation 1)
    let south_pos = block_pos + IVec3::Z;
    if let Some(v) = world.get_voxel(south_pos)
        && v.is_torch()
    {
        let (s, o) = world.get_shape(south_pos);
        if s == BlockShape::Torch && o == 1 && remove_voxel(world, modifications, south_pos) {
            removed.push(south_pos);
        }
    }

    // North neighbor (-Z) has torch attached to South wall of block_pos (orientation 2)
    let north_pos = block_pos - IVec3::Z;
    if let Some(v) = world.get_voxel(north_pos)
        && v.is_torch()
    {
        let (s, o) = world.get_shape(north_pos);
        if s == BlockShape::Torch && o == 2 && remove_voxel(world, modifications, north_pos) {
            removed.push(north_pos);
        }
    }

    // East neighbor (+X) has torch attached to West wall of block_pos (orientation 3)
    let east_pos = block_pos + IVec3::X;
    if let Some(v) = world.get_voxel(east_pos)
        && v.is_torch()
    {
        let (s, o) = world.get_shape(east_pos);
        if s == BlockShape::Torch && o == 3 && remove_voxel(world, modifications, east_pos) {
            removed.push(east_pos);
        }
    }

    // West neighbor (-X) has torch attached to East wall of block_pos (orientation 4)
    let west_pos = block_pos - IVec3::X;
    if let Some(v) = world.get_voxel(west_pos)
        && v.is_torch()
    {
        let (s, o) = world.get_shape(west_pos);
        if s == BlockShape::Torch && o == 4 && remove_voxel(world, modifications, west_pos) {
            removed.push(west_pos);
        }
    }

    removed
}

pub fn place_block(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    block_pos: IVec3,
    voxel: Voxel,
) -> Vec<IVec3> {
    if let Some(top_partner) = voxel.double_tall_top_partner() {
        let top_pos = block_pos + IVec3::Y;
        let Some(above_voxel) = world.get_voxel(top_pos) else {
            return Vec::new();
        };
        if !above_voxel.is_empty() && !above_voxel.is_fluid() {
            return Vec::new();
        }

        if place_voxel(world, modifications, block_pos, voxel) {
            if place_voxel(world, modifications, top_pos, top_partner) {
                return vec![block_pos, top_pos];
            } else {
                remove_voxel(world, modifications, block_pos);
                return Vec::new();
            }
        }
        return Vec::new();
    }

    if place_voxel(world, modifications, block_pos, voxel) {
        vec![block_pos]
    } else {
        Vec::new()
    }
}
