use bevy::prelude::*;

use super::{
    interaction_mode::InteractionMode,
    shaping::{centered_layer_coordinates, get_centered_layer_material, is_centered_layer},
    targeting::{CurrentTarget, TargetingSet, adjacent_block_origin},
};
use crate::{
    menu::MenuState,
    player::{GameMode, hotbar::Hotbar},
    simulation::{
        fluid::FluidUpdateQueue,
        lighting::{VoxelLightRegistry, sync_voxel_light},
    },
    world::{
        ChunkStreamingQueues, Voxel, VoxelWorld, WorldModificationStore, affected_chunks,
    },
};

const HOLD_DELAY: f32 = 0.25;
const REPEAT_INTERVAL: f32 = 0.16;

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
        app.init_resource::<SelectedVoxel>()
            .init_resource::<InteractionMode>()
            .add_systems(
                Update,
                toggle_interaction_mode.before(TargetingSet::UpdateTarget),
            )
            .add_systems(
                Update,
                (pick_targeted_voxel, edit_voxels)
                    .chain()
                    .after(TargetingSet::UpdateTarget),
            );
    }
}

fn toggle_interaction_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    menu_state: Option<Res<State<MenuState>>>,
    mut interaction_mode: ResMut<InteractionMode>,
) {
    if menu_state.is_some_and(|s| *s.get() != MenuState::None) {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyB) {
        return;
    }

    interaction_mode.toggle();
    info!("Interaction mode: {}", interaction_mode.label());
}

fn pick_targeted_voxel(
    game_mode: Res<GameMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    menu_state: Option<Res<State<MenuState>>>,
    current_target: Res<CurrentTarget>,
    world: Res<VoxelWorld>,
    mut selected: ResMut<SelectedVoxel>,
    hotbar: Option<ResMut<Hotbar>>,
) {
    if menu_state.is_some_and(|s| *s.get() != MenuState::None) {
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

    let Some(mut voxel) = world.get_voxel(target.hit_voxel) else {
        return;
    };

    if voxel.is_empty() {
        return;
    }

    if voxel == Voxel::Occupied || voxel == Voxel::WaterOccupied {
        if let Some(material) = get_centered_layer_material(&world, target.hit_voxel) {
            voxel = material;
        } else {
            return;
        }
    }

    selected.0 = Some(voxel);
    if let Some(mut hotbar) = hotbar {
        let active = hotbar.active_slot;
        hotbar.slots[active] = Some(voxel);
    }

    info!("Selected voxel: {}", voxel.label());
}

#[allow(clippy::too_many_arguments)]
fn edit_voxels(
    game_mode: Res<GameMode>,
    selected: Res<SelectedVoxel>,
    interaction_mode: Res<InteractionMode>,
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    current_target: Res<CurrentTarget>,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut queues: ResMut<ChunkStreamingQueues>,
    mut interaction_state: Local<InteractionState>,
    fluid_queue: Option<ResMut<FluidUpdateQueue>>,
    menu_state: Option<Res<State<MenuState>>>,
) {
    if menu_state.is_some_and(|s| *s.get() != MenuState::None) {
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

    let edited_voxels = if break_action {
        match *interaction_mode {
            InteractionMode::Block => {
                remove_block(&mut world, &mut modifications, target.block_origin)
            }
            InteractionMode::Voxel => {
                if is_centered_layer(&world, target.hit_voxel) {
                    let coords = centered_layer_coordinates(target.hit_voxel);
                    let mut removed = Vec::new();
                    for pos in coords {
                        if remove_voxel(&mut world, &mut modifications, pos) {
                            removed.push(pos);
                        }
                    }
                    removed
                } else if remove_voxel(&mut world, &mut modifications, target.hit_voxel) {
                    vec![target.hit_voxel]
                } else {
                    Vec::new()
                }
            }
        }
    } else if place_action {
        let Some(place_voxel_type) = selected.0 else {
            return;
        };

        let Some(place_position) = target.place_voxel else {
            return;
        };

        match *interaction_mode {
            InteractionMode::Block => {
                if target.face_normal == IVec3::ZERO {
                    Vec::new()
                } else {
                    let block_origin = adjacent_block_origin(
                        target.block_origin,
                        target.hit_voxel,
                        target.face_normal,
                    );

                    place_block(
                        &mut world,
                        &mut modifications,
                        block_origin,
                        place_voxel_type,
                    )
                }
            }

            InteractionMode::Voxel => {
                let is_centered_target = is_centered_layer(&world, place_position + IVec3::Y)
                    || is_centered_layer(&world, place_position - IVec3::Y);

                if is_centered_target {
                    let coords = centered_layer_coordinates(place_position);
                    if coords.iter().all(|&pos| {
                        world
                            .get_voxel(pos)
                            .is_some_and(|v| v.is_empty() || v.is_water())
                    }) {
                        let is_waterlogged = coords
                            .iter()
                            .any(|&pos| world.get_voxel(pos).is_some_and(Voxel::is_water));
                        let occ_type = if is_waterlogged {
                            Voxel::WaterOccupied
                        } else {
                            Voxel::Occupied
                        };

                        world.set_voxel(coords[0], place_voxel_type);
                        modifications.record(coords[0], place_voxel_type);
                        for &pos in &coords[1..4] {
                            world.set_voxel(pos, occ_type);
                            modifications.record(pos, occ_type);
                        }
                        coords.to_vec()
                    } else if place_voxel(
                        &mut world,
                        &mut modifications,
                        place_position,
                        place_voxel_type,
                    ) {
                        vec![place_position]
                    } else {
                        Vec::new()
                    }
                } else if place_voxel(
                    &mut world,
                    &mut modifications,
                    place_position,
                    place_voxel_type,
                ) {
                    vec![place_position]
                } else {
                    Vec::new()
                }
            }
        }
    } else {
        Vec::new()
    };

    if edited_voxels.is_empty() {
        return;
    }

    if let Some(mut fq) = fluid_queue {
        for &edited_voxel in &edited_voxels {
            fq.enqueue_with_neighbors(edited_voxel);
        }
    }

    let mut dirty_chunks = Vec::new();

    for edited_voxel in edited_voxels {
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

fn remove_voxel(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    world_voxel: IVec3,
) -> bool {
    let Some(current_voxel) = world.get_voxel(world_voxel) else {
        return false;
    };

    if current_voxel.is_empty() {
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

fn place_voxel(
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

fn remove_block(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    block_origin: IVec3,
) -> Vec<IVec3> {
    let mut removed_voxels = Vec::with_capacity(8);

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..2 {
                let position = block_origin + IVec3::new(x, y, z);
                if remove_voxel(world, modifications, position) {
                    removed_voxels.push(position);
                }
            }
        }
    }

    removed_voxels
}

pub fn place_block(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    block_origin: IVec3,
    voxel: Voxel,
) -> Vec<IVec3> {
    let mut positions = [IVec3::ZERO; 8];
    let mut index = 0;

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..2 {
                let position = block_origin + IVec3::new(x, y, z);
                let Some(current_voxel) = world.get_voxel(position) else {
                    return Vec::new();
                };

                if !current_voxel.is_empty() && !current_voxel.is_water() {
                    return Vec::new();
                }

                positions[index] = position;
                index += 1;
            }
        }
    }

    let mut edited_voxels = Vec::with_capacity(8);

    for position in positions {
        if place_voxel(world, modifications, position, voxel) {
            edited_voxels.push(position);
        }
    }

    edited_voxels
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Chunk, VoxelWorld};
    use bevy::prelude::IVec3;

    #[test]
    fn placing_a_block_fills_all_eight_voxels() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());
        let mut modifications = WorldModificationStore::default();
        let origin = IVec3::new(2, 2, 2);

        let edited = place_block(&mut world, &mut modifications, origin, Voxel::Stone);

        assert_eq!(edited.len(), 8);
        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    assert_eq!(
                        world.get_voxel(origin + IVec3::new(x, y, z)),
                        Some(Voxel::Stone)
                    );
                }
            }
        }
    }

    #[test]
    fn placing_a_block_is_atomic_when_any_voxel_is_occupied() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());
        let mut modifications = WorldModificationStore::default();
        let origin = IVec3::new(2, 2, 2);
        world.set_voxel(origin + IVec3::ONE, Voxel::Dirt);

        let edited = place_block(&mut world, &mut modifications, origin, Voxel::Stone);

        assert!(edited.is_empty());
        assert_eq!(world.get_voxel(origin), Some(Voxel::Air));
        assert_eq!(world.get_voxel(origin + IVec3::ONE), Some(Voxel::Dirt));
    }

    #[test]
    fn placing_on_top_of_centered_voxel_creates_centered_layer() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());

        world.set_voxel(IVec3::new(0, 0, 0), Voxel::Stone);
        world.set_voxel(IVec3::new(1, 0, 0), Voxel::Occupied);
        world.set_voxel(IVec3::new(0, 0, 1), Voxel::Occupied);
        world.set_voxel(IVec3::new(1, 0, 1), Voxel::Occupied);

        assert!(is_centered_layer(&world, IVec3::new(0, 0, 0)));

        let coords = centered_layer_coordinates(IVec3::new(0, 1, 0));
        world.set_voxel(coords[0], Voxel::Stone);
        for &pos in &coords[1..4] {
            world.set_voxel(pos, Voxel::Occupied);
        }

        assert!(is_centered_layer(&world, IVec3::new(0, 1, 0)));
        assert_eq!(world.get_voxel(IVec3::new(0, 1, 0)), Some(Voxel::Stone));
        assert_eq!(world.get_voxel(IVec3::new(1, 1, 0)), Some(Voxel::Occupied));
    }

    #[test]
    fn placing_on_floor_beneath_hanging_centered_column_creates_centered_layer() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());
        let mut modifications = WorldModificationStore::default();

        world.set_voxel(IVec3::new(0, 0, 0), Voxel::Stone);
        world.set_voxel(IVec3::new(1, 0, 0), Voxel::Stone);
        world.set_voxel(IVec3::new(0, 0, 1), Voxel::Stone);
        world.set_voxel(IVec3::new(1, 0, 1), Voxel::Stone);

        world.set_voxel(IVec3::new(0, 2, 0), Voxel::Sand);
        world.set_voxel(IVec3::new(1, 2, 0), Voxel::Occupied);
        world.set_voxel(IVec3::new(0, 2, 1), Voxel::Occupied);
        world.set_voxel(IVec3::new(1, 2, 1), Voxel::Occupied);

        let place_position = IVec3::new(0, 1, 0);
        let is_centered_target = is_centered_layer(&world, place_position + IVec3::Y)
            || is_centered_layer(&world, place_position - IVec3::Y);

        assert!(is_centered_target);

        let coords = centered_layer_coordinates(place_position);
        assert!(
            coords
                .iter()
                .all(|&pos| world.get_voxel(pos).is_some_and(|v| v.is_empty()))
        );

        world.set_voxel(coords[0], Voxel::Sand);
        modifications.record(coords[0], Voxel::Sand);
        for &pos in &coords[1..4] {
            world.set_voxel(pos, Voxel::Occupied);
            modifications.record(pos, Voxel::Occupied);
        }

        assert!(is_centered_layer(&world, place_position));
        assert_eq!(world.get_voxel(coords[0]), Some(Voxel::Sand));
        assert_eq!(world.get_voxel(coords[1]), Some(Voxel::Occupied));
    }
}
