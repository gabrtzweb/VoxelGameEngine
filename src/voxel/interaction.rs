use bevy::prelude::*;

use crate::player::GameMode;

use super::{
    InteractionMode,
    chunk::{CHUNK_SIZE, Voxel},
    light::{VoxelLightRegistry, sync_voxel_light},
    modifications::WorldModificationStore,
    render::{ChunkMaterial, ChunkMeshRegistry, sync_chunk_render},
    targeting::{CurrentTarget, TargetingSet, adjacent_block_origin},
    world::VoxelWorld,
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
    mut interaction_mode: ResMut<InteractionMode>,
) {
    if !keyboard.just_pressed(KeyCode::KeyB) {
        return;
    }

    interaction_mode.toggle();

    info!("Interaction mode: {}", interaction_mode.label(),);
}

fn pick_targeted_voxel(
    game_mode: Res<GameMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    current_target: Res<CurrentTarget>,
    world: Res<VoxelWorld>,
    mut selected: ResMut<SelectedVoxel>,
    hotbar: Option<ResMut<crate::player::hotbar::Hotbar>>,
) {
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

    if voxel == Voxel::Occupied {
        if let Some(material) =
            crate::voxel::shaping::get_centered_layer_material(&world, target.hit_voxel)
        {
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
    material: Res<ChunkMaterial>,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut registry: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut interaction_state: Local<InteractionState>,
) {
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
                if crate::voxel::shaping::is_centered_layer(&world, target.hit_voxel) {
                    let coords =
                        crate::voxel::shaping::centered_layer_coordinates(target.hit_voxel);
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
                let is_centered_target =
                    crate::voxel::shaping::is_centered_layer(&world, place_position + IVec3::Y)
                        || crate::voxel::shaping::is_centered_layer(
                            &world,
                            place_position - IVec3::Y,
                        );

                if is_centered_target {
                    let coords = crate::voxel::shaping::centered_layer_coordinates(place_position);
                    if coords
                        .iter()
                        .all(|&pos| world.get_voxel(pos).is_some_and(|v| v.is_empty()))
                    {
                        world.set_voxel(coords[0], place_voxel_type);
                        modifications.record(coords[0], place_voxel_type);
                        for &pos in &coords[1..4] {
                            world.set_voxel(pos, Voxel::Occupied);
                            modifications.record(pos, Voxel::Occupied);
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

    let mut dirty_chunks = Vec::new();

    for edited_voxel in edited_voxels {
        sync_voxel_light(&mut commands, &world, edited_voxel, &mut light_registry);

        dirty_chunks.extend(affected_chunks(edited_voxel));
    }

    dirty_chunks.sort_by_key(|chunk| (chunk.x, chunk.y, chunk.z));
    dirty_chunks.dedup();

    for coordinate in dirty_chunks {
        if world.get_chunk(coordinate).is_none() {
            continue;
        }

        sync_chunk_render(
            &mut commands,
            &world,
            coordinate,
            &mut registry,
            &mut meshes,
            &material,
        );
    }
}

fn remove_voxel(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    position: IVec3,
) -> bool {
    let Some(voxel) = world.get_voxel(position) else {
        return false;
    };

    if voxel.is_empty() {
        return false;
    }

    if world.set_voxel(position, Voxel::Air).is_none() {
        return false;
    }

    modifications.record(position, Voxel::Air);

    true
}

fn remove_block(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    block_origin: IVec3,
) -> Vec<IVec3> {
    let mut edited_voxels = Vec::with_capacity(8);

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..2 {
                let position = block_origin + IVec3::new(x, y, z);

                if remove_voxel(world, modifications, position) {
                    edited_voxels.push(position);
                }
            }
        }
    }

    edited_voxels
}

fn place_voxel(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    position: IVec3,
    voxel: Voxel,
) -> bool {
    let Some(current_voxel) = world.get_voxel(position) else {
        return false;
    };

    if !current_voxel.is_empty() {
        return false;
    }

    if world.set_voxel(position, voxel).is_none() {
        return false;
    }

    modifications.record(position, voxel);

    true
}

fn place_block(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    block_origin: IVec3,
    voxel: Voxel,
) -> Vec<IVec3> {
    let mut positions = Vec::with_capacity(8);

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..2 {
                positions.push(block_origin + IVec3::new(x, y, z));
            }
        }
    }

    if positions.iter().any(|&position| {
        world
            .get_voxel(position)
            .is_none_or(|current_voxel| !current_voxel.is_empty())
    }) {
        return Vec::new();
    }

    let mut edited_voxels = Vec::with_capacity(8);

    for position in positions {
        if place_voxel(world, modifications, position, voxel) {
            edited_voxels.push(position);
        }
    }

    edited_voxels
}

pub fn affected_chunks(world_voxel: IVec3) -> Vec<IVec3> {
    let (chunk_coordinate, local_coordinate) = VoxelWorld::world_voxel_to_chunk(world_voxel);

    let mut chunks = Vec::with_capacity(4);

    chunks.push(chunk_coordinate);

    let max_local = (CHUNK_SIZE - 1) as u32;

    if local_coordinate.x == 0 {
        chunks.push(chunk_coordinate + IVec3::new(-1, 0, 0));
    } else if local_coordinate.x == max_local {
        chunks.push(chunk_coordinate + IVec3::new(1, 0, 0));
    }

    if local_coordinate.y == 0 {
        chunks.push(chunk_coordinate + IVec3::new(0, -1, 0));
    } else if local_coordinate.y == max_local {
        chunks.push(chunk_coordinate + IVec3::new(0, 1, 0));
    }

    if local_coordinate.z == 0 {
        chunks.push(chunk_coordinate + IVec3::new(0, 0, -1));
    } else if local_coordinate.z == max_local {
        chunks.push(chunk_coordinate + IVec3::new(0, 0, 1));
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::{Voxel, WorldModificationStore, place_block};
    use crate::voxel::{chunk::Chunk, world::VoxelWorld};
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

        // Setup a bottom centered voxel at y=0
        world.set_voxel(IVec3::new(0, 0, 0), Voxel::Stone);
        world.set_voxel(IVec3::new(1, 0, 0), Voxel::Occupied);
        world.set_voxel(IVec3::new(0, 0, 1), Voxel::Occupied);
        world.set_voxel(IVec3::new(1, 0, 1), Voxel::Occupied);

        assert!(crate::voxel::shaping::is_centered_layer(
            &world,
            IVec3::new(0, 0, 0)
        ));

        // Simulate placing a centered voxel on top at y=1
        let coords = crate::voxel::shaping::centered_layer_coordinates(IVec3::new(0, 1, 0));
        world.set_voxel(coords[0], Voxel::Stone);
        for &pos in &coords[1..4] {
            world.set_voxel(pos, Voxel::Occupied);
        }

        assert!(crate::voxel::shaping::is_centered_layer(
            &world,
            IVec3::new(0, 1, 0)
        ));
        assert_eq!(world.get_voxel(IVec3::new(0, 1, 0)), Some(Voxel::Stone));
        assert_eq!(world.get_voxel(IVec3::new(1, 1, 0)), Some(Voxel::Occupied));
    }

    #[test]
    fn placing_on_floor_beneath_hanging_centered_column_creates_centered_layer() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());
        let mut modifications = WorldModificationStore::default();

        // Floor at y=0 (normal stone block)
        world.set_voxel(IVec3::new(0, 0, 0), Voxel::Stone);
        world.set_voxel(IVec3::new(1, 0, 0), Voxel::Stone);
        world.set_voxel(IVec3::new(0, 0, 1), Voxel::Stone);
        world.set_voxel(IVec3::new(1, 0, 1), Voxel::Stone);

        // Hanging centered column at y=2
        world.set_voxel(IVec3::new(0, 2, 0), Voxel::Sand);
        world.set_voxel(IVec3::new(1, 2, 0), Voxel::Occupied);
        world.set_voxel(IVec3::new(0, 2, 1), Voxel::Occupied);
        world.set_voxel(IVec3::new(1, 2, 1), Voxel::Occupied);

        // Space at y=1 is empty (broken bottom voxel)
        let place_position = IVec3::new(0, 1, 0);
        let is_centered_target =
            crate::voxel::shaping::is_centered_layer(&world, place_position + IVec3::Y)
                || crate::voxel::shaping::is_centered_layer(&world, place_position - IVec3::Y);

        assert!(is_centered_target);

        let coords = crate::voxel::shaping::centered_layer_coordinates(place_position);
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

        assert!(crate::voxel::shaping::is_centered_layer(
            &world,
            place_position
        ));
        assert_eq!(world.get_voxel(coords[0]), Some(Voxel::Sand));
        assert_eq!(world.get_voxel(coords[1]), Some(Voxel::Occupied));
    }
}
