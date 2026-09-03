use bevy::prelude::*;

use crate::player::GameMode;

use super::{
    chunk::{CHUNK_SIZE, Voxel},
    light::{VoxelLightRegistry, sync_voxel_light},
    modifications::WorldModificationStore,
    render::{ChunkMaterial, ChunkMeshRegistry, sync_chunk_render},
    targeting::{CurrentTarget, TargetingSet},
    world::VoxelWorld,
};

const HOLD_DELAY: f32 = 0.25;
const REPEAT_INTERVAL: f32 = 0.16;

#[derive(Resource, Clone, Copy)]
pub struct SelectedVoxel(pub Voxel);

impl Default for SelectedVoxel {
    fn default() -> Self {
        Self(Voxel::Stone)
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
            .add_systems(Update, select_voxel_type)
            .add_systems(
                Update,
                (pick_targeted_voxel, edit_voxels)
                    .chain()
                    .after(TargetingSet::UpdateTarget),
            );
    }
}

fn select_voxel_type(keyboard: Res<ButtonInput<KeyCode>>, mut selected: ResMut<SelectedVoxel>) {
    let next = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(Voxel::Grass)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(Voxel::Dirt)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(Voxel::Stone)
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some(Voxel::Sand)
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        Some(Voxel::Water)
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        Some(Voxel::Light)
    } else {
        None
    };

    let Some(next) = next else {
        return;
    };

    set_selected_voxel(&mut selected, next);
}

fn pick_targeted_voxel(
    game_mode: Res<GameMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    current_target: Res<CurrentTarget>,
    world: Res<VoxelWorld>,
    mut selected: ResMut<SelectedVoxel>,
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

    let Some(voxel) = world.get_voxel(target.hit_voxel) else {
        return;
    };

    if voxel.is_empty() {
        return;
    }

    set_selected_voxel(&mut selected, voxel);
}

fn set_selected_voxel(selected: &mut SelectedVoxel, voxel: Voxel) {
    if selected.0 == voxel {
        return;
    }

    selected.0 = voxel;

    info!("Selected voxel: {}", voxel.label(),);
}

#[allow(clippy::too_many_arguments)]
fn edit_voxels(
    game_mode: Res<GameMode>,
    selected: Res<SelectedVoxel>,
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

    let edited_voxel = if break_action {
        if remove_voxel(&mut world, &mut modifications, target.hit_voxel) {
            Some(target.hit_voxel)
        } else {
            None
        }
    } else if place_action {
        target.place_voxel.filter(|&place_position| {
            place_voxel(&mut world, &mut modifications, place_position, selected.0)
        })
    } else {
        None
    };

    let Some(edited_voxel) = edited_voxel else {
        return;
    };

    sync_voxel_light(&mut commands, &world, edited_voxel, &mut light_registry);

    let dirty_chunks = affected_chunks(edited_voxel);

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

fn affected_chunks(world_voxel: IVec3) -> Vec<IVec3> {
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
