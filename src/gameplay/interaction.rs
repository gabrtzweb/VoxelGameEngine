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
    world::{ChunkStreamingQueues, Voxel, VoxelWorld, WorldModificationStore, affected_chunks},
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
        Self(Some(Voxel::Grass))
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
        let edited = remove_block(&mut world, &mut modifications, target.hit_voxel);
        (edited, prev_voxel)
    } else if place_action {
        let Some(place_voxel_type) = selected.0 else {
            return;
        };

        let Some(place_position) = target.place_voxel else {
            return;
        };

        let edited = place_block(
            &mut world,
            &mut modifications,
            place_position,
            place_voxel_type,
        );
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
        commands.trigger(BlockPlaceEvent {
            position: place_position,
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
        Voxel::Water
    } else {
        Voxel::Air
    };

    world.set_voxel(world_voxel, replacement);
    modifications.record(world_voxel, replacement);

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

    if !current_voxel.is_empty() && !current_voxel.is_water() {
        return false;
    }

    let final_voxel = if current_voxel.is_water() && voxel == Voxel::Occupied {
        Voxel::WaterOccupied
    } else {
        voxel
    };

    world.set_voxel(world_voxel, final_voxel);
    modifications.record(world_voxel, final_voxel);

    true
}

pub fn remove_block(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    block_pos: IVec3,
) -> Vec<IVec3> {
    if remove_voxel(world, modifications, block_pos) {
        vec![block_pos]
    } else {
        Vec::new()
    }
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
