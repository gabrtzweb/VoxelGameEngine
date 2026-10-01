use bevy::prelude::*;

use super::{
    feedback::{BlockBreakEvent, BlockPlaceEvent},
    targeting::{CurrentTarget, TargetingSet},
};
use crate::{
    menu::MenuState,
    player::{GameMode, InspectorInteraction, hotbar::Hotbar},
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
        Self(Some(Voxel::Soil_Grass))
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
    mouse: Res<ButtonInput<MouseButton>>,
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

    if !mouse.just_pressed(MouseButton::Middle) {
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

    selected.0 = Some(voxel);
    if let Some(mut hotbar) = hotbar {
        let active = hotbar.active_slot;
        hotbar.slots[active] = Some(voxel);
    }

    info!("Selected voxel: {}", voxel.label());
}

fn edit_voxels(
    (game_mode, selected): (Res<GameMode>, Res<SelectedVoxel>),
    mut commands: Commands,
    (mouse, time, current_target): (Res<ButtonInput<MouseButton>>, Res<Time>, Res<CurrentTarget>),
    (mut world, mut modifications, mut light_registry, mut queues): WorldEditResources,
    (fluid_queue, menu_state, inspector, mut map_cache): OptionalInteractionParams,
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

    let break_action = interaction_state.break_action.update(
        mouse.pressed(MouseButton::Left),
        mouse.just_pressed(MouseButton::Left),
        delta_seconds,
    );

    let place_action = interaction_state.place_action.update(
        mouse.pressed(MouseButton::Right),
        mouse.just_pressed(MouseButton::Right),
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
            let supporting_solid = existing_voxel.is_some_and(|v| !v.is_empty() && !v.is_fluid() && !v.is_torch());
            if !supporting_solid {
                Vec::new()
            } else if target.face_normal == -IVec3::Y {
                // Cannot place torch on ceiling
                Vec::new()
            } else {
                let torch_orient = match target.face_normal {
                    IVec3::Y => 0,      // Floor standing
                    IVec3::Z => 1,      // Wall North (-Z)
                    IVec3::NEG_Z => 2,  // Wall South (+Z)
                    IVec3::X => 3,      // Wall West (-X)
                    IVec3::NEG_X => 4,  // Wall East (+X)
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
        } else {
            place_block(
                &mut world,
                &mut modifications,
                place_position,
                place_voxel_type,
            )
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
        Voxel::Liquid_Water
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
    if !remove_voxel(world, modifications, block_pos) {
        return Vec::new();
    }

    let mut removed = vec![block_pos];

    // Check if any attached torches lost their support:
    // 1. Floor torch sitting on top of this block (block_pos + Y)
    let top_pos = block_pos + IVec3::Y;
    if let Some(v) = world.get_voxel(top_pos)
        && v.is_torch()
    {
        let (s, o) = world.get_shape(top_pos);
        if s == BlockShape::Torch && o == 0 && remove_voxel(world, modifications, top_pos) {
            removed.push(top_pos);
        }
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
    if place_voxel(world, modifications, block_pos, voxel) {
        vec![block_pos]
    } else {
        Vec::new()
    }
}
