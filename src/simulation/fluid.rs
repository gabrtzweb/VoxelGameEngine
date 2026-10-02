use std::collections::{HashSet, VecDeque};

use bevy::prelude::*;

use super::lighting::{VoxelLightRegistry, sync_voxel_light};
use crate::{
    player::Player,
    world::{
        ChunkStreamingQueues, VOXEL_SIZE, Voxel, VoxelAccess, VoxelWorld, WorldModificationStore,
        affected_chunks,
        streaming::manager::{ChunkStreamingSettings, DEFAULT_SIMULATION_DISTANCE},
    },
};

const FLUID_TICK_SECONDS: f32 = 0.25;
const MAX_UPDATES_PER_TICK: usize = 256;

/// Returns true if `voxel_pos` is within `simulation_distance` chunks of `player_chunk`.
/// Uses horizontal Euclidean distance squared `dx*dx + dz*dz <= sim_dist*sim_dist` and vertical delta `dy <= sim_dist`.
pub fn is_in_simulation_radius(
    voxel_pos: IVec3,
    player_chunk: IVec3,
    simulation_distance: i32,
) -> bool {
    let (chunk_coord, _) = VoxelWorld::world_voxel_to_chunk(voxel_pos);
    let dx = chunk_coord.x - player_chunk.x;
    let dz = chunk_coord.z - player_chunk.z;
    let dy = (chunk_coord.y - player_chunk.y).abs();
    dx * dx + dz * dz <= simulation_distance * simulation_distance && dy <= simulation_distance
}

#[derive(Resource, Default)]
pub struct FluidUpdateQueue {
    queue: VecDeque<IVec3>,
    in_queue: HashSet<IVec3>,
}

impl FluidUpdateQueue {
    pub fn enqueue(&mut self, position: IVec3) {
        if self.in_queue.insert(position) {
            self.queue.push_back(position);
        }
    }

    pub fn enqueue_with_neighbors(&mut self, position: IVec3) {
        self.enqueue(position);
        self.enqueue(position + IVec3::X);
        self.enqueue(position - IVec3::X);
        self.enqueue(position + IVec3::Y);
        self.enqueue(position - IVec3::Y);
        self.enqueue(position + IVec3::Z);
        self.enqueue(position - IVec3::Z);
    }
}

#[derive(Resource)]
struct FluidTickTimer(Timer);

impl Default for FluidTickTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            FLUID_TICK_SECONDS,
            TimerMode::Repeating,
        ))
    }
}

pub struct FluidSimulationPlugin;

impl Plugin for FluidSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FluidUpdateQueue>()
            .init_resource::<FluidTickTimer>()
            .add_systems(Update, run_fluid_simulation);
    }
}

fn run_fluid_simulation(
    time: Res<Time>,
    mut timer: ResMut<FluidTickTimer>,
    mut queue: ResMut<FluidUpdateQueue>,
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut queues: ResMut<ChunkStreamingQueues>,
    player_query: Query<&Transform, With<Player>>,
    settings: Option<Res<ChunkStreamingSettings>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    if queue.queue.is_empty() {
        return;
    }

    let player_sim_ctx = player_query.iter().next().map(|transform| {
        let player_voxel = (transform.translation / VOXEL_SIZE).floor().as_ivec3();
        let (player_chunk, _) = VoxelWorld::world_voxel_to_chunk(player_voxel);
        let sim_distance = settings
            .as_ref()
            .map_or(DEFAULT_SIMULATION_DISTANCE, |s| s.simulation_distance);
        (player_chunk, sim_distance)
    });

    let mut edited_voxels = Vec::with_capacity(32);
    let count = queue.queue.len();
    let mut updates_this_tick = 0;

    for _ in 0..count {
        if updates_this_tick >= MAX_UPDATES_PER_TICK {
            break;
        }

        let Some(pos) = queue.queue.pop_front() else {
            break;
        };
        queue.in_queue.remove(&pos);

        if let Some((player_chunk, sim_dist)) = player_sim_ctx
            && !is_in_simulation_radius(pos, player_chunk, sim_dist)
        {
            // Beyond decoupled simulation radius: drop non-visible dynamic fluid tick
            continue;
        }

        let Some(current_voxel) = world.get_voxel(pos) else {
            continue;
        };

        if current_voxel.is_fluid() {
            let changed = process_fluid(
                &mut world,
                &mut modifications,
                &mut queue,
                pos,
                current_voxel,
            );
            edited_voxels.extend(changed);
        } else if current_voxel == Voxel::Air {
            let changed = check_infinite_source(&mut world, &mut modifications, &mut queue, pos);
            edited_voxels.extend(changed);
        }

        updates_this_tick += 1;
    }

    if edited_voxels.is_empty() {
        return;
    }

    let mut dirty_chunks = Vec::with_capacity(16);

    for edited in edited_voxels {
        sync_voxel_light(&mut commands, &world, edited, &mut light_registry);
        dirty_chunks.extend(affected_chunks(edited));
    }

    dirty_chunks.sort_by_key(|chunk| (chunk.x, chunk.y, chunk.z));
    dirty_chunks.dedup();

    for coordinate in dirty_chunks {
        if world.get_chunk(coordinate).is_some() {
            queues.enqueue_remesh(coordinate);
        }
    }
}

#[inline]
pub fn is_same_fluid_type(a: Voxel, b: Voxel) -> bool {
    (a.is_water() && b.is_water()) || (a.is_fluid() && a == b)
}

fn process_fluid(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    queue: &mut FluidUpdateQueue,
    pos: IVec3,
    current_voxel: Voxel,
) -> Vec<IVec3> {
    let mut edited = Vec::with_capacity(5);
    let current_level = world.get_fluid_level(pos);

    let horizontals = [
        pos + IVec3::X,
        pos - IVec3::X,
        pos + IVec3::Z,
        pos - IVec3::Z,
    ];

    // 1. Upstream validation for flowing blocks:
    // If this is a flowing block (level > 0), verify that it is still fed by an upstream source/column.
    // If not, drain back to Air (or Occupied for waterlogged slabs).
    if current_level > 0 {
        let has_upstream_above = world
            .get_voxel(pos + IVec3::Y)
            .is_some_and(|v| is_same_fluid_type(current_voxel, v));

        let has_upstream_horizontal = horizontals.iter().any(|&n| {
            world
                .get_voxel(n)
                .is_some_and(|v| is_same_fluid_type(current_voxel, v))
                && world.get_fluid_level(n) < current_level
        });

        if !has_upstream_above && !has_upstream_horizontal {
            let replacement = if current_voxel == Voxel::WaterOccupied {
                Voxel::Occupied
            } else {
                Voxel::Air
            };
            world.set_voxel(pos, replacement);
            world.set_fluid_level(pos, 0);
            modifications.record(pos, replacement);
            modifications.record_fluid_level(pos, 0);
            queue.enqueue_with_neighbors(pos);
            edited.push(pos);
            return edited;
        }
    }

    // 2. Downward flow (vertical gravity priority):
    let below = pos - IVec3::Y;
    if let Some(voxel_below) = world.get_voxel(below) {
        if voxel_below == Voxel::Air {
            let flow_voxel = if current_voxel == Voxel::WaterOccupied {
                Voxel::Liquid_Water
            } else {
                current_voxel
            };
            world.set_voxel(below, flow_voxel);
            world.set_fluid_level(below, 1);
            modifications.record(below, flow_voxel);
            modifications.record_fluid_level(below, 1);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        } else if voxel_below == Voxel::Occupied && current_voxel.is_water() {
            world.set_voxel(below, Voxel::WaterOccupied);
            world.set_fluid_level(below, 1);
            modifications.record(below, Voxel::WaterOccupied);
            modifications.record_fluid_level(below, 1);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        }
    }

    // 3. Falling column check:
    // A falling stream descending through mid-air must not branch sideways until it hits solid ground.
    let is_falling_column = world
        .get_voxel(pos + IVec3::Y)
        .is_some_and(|v| is_same_fluid_type(current_voxel, v));
    let below_is_solid = world
        .get_voxel(below)
        .is_some_and(|v| !v.is_empty() && !v.is_fluid());

    if is_falling_column && !below_is_solid {
        return edited;
    }

    // 4. Ground support check:
    if !is_supported_by_ground(world, pos) {
        return edited;
    }

    // 5. Horizontal spreading:
    let effective_level = if current_level == 0 {
        0
    } else if is_falling_column {
        1
    } else {
        current_level
    };

    let max_spread = current_voxel.max_fluid_spread();
    if effective_level < max_spread {
        let next_level = effective_level + 1;
        let spread_voxel = if current_voxel == Voxel::WaterOccupied {
            Voxel::Liquid_Water
        } else {
            current_voxel
        };

        for neighbor in horizontals {
            let Some(v) = world.get_voxel(neighbor) else {
                continue;
            };

            if v == Voxel::Air {
                world.set_voxel(neighbor, spread_voxel);
                world.set_fluid_level(neighbor, next_level);
                modifications.record(neighbor, spread_voxel);
                modifications.record_fluid_level(neighbor, next_level);
                queue.enqueue_with_neighbors(neighbor);
                edited.push(neighbor);
            } else if v == Voxel::Occupied && current_voxel.is_water() {
                world.set_voxel(neighbor, Voxel::WaterOccupied);
                world.set_fluid_level(neighbor, next_level);
                modifications.record(neighbor, Voxel::WaterOccupied);
                modifications.record_fluid_level(neighbor, next_level);
                queue.enqueue_with_neighbors(neighbor);
                edited.push(neighbor);
            } else if is_same_fluid_type(current_voxel, v) {
                let existing_level = world.get_fluid_level(neighbor);
                if existing_level > next_level {
                    world.set_fluid_level(neighbor, next_level);
                    modifications.record_fluid_level(neighbor, next_level);
                    queue.enqueue_with_neighbors(neighbor);
                    edited.push(neighbor);
                }
            }
        }
    }

    edited
}

fn check_infinite_source(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    queue: &mut FluidUpdateQueue,
    pos: IVec3,
) -> Vec<IVec3> {
    let mut edited = Vec::with_capacity(1);

    let below = pos - IVec3::Y;
    let Some(v_below) = world.get_voxel(below) else {
        return edited;
    };
    if v_below.is_empty() {
        return edited;
    }

    let horizontals = [
        pos + IVec3::X,
        pos - IVec3::X,
        pos + IVec3::Z,
        pos - IVec3::Z,
    ];

    let source_count = horizontals
        .iter()
        .filter(|&&n| {
            world.get_voxel(n) == Some(Voxel::Liquid_Water) && world.get_fluid_level(n) == 0
        })
        .count();

    if source_count >= 2 {
        world.set_voxel(pos, Voxel::Liquid_Water);
        world.set_fluid_level(pos, 0);
        modifications.record(pos, Voxel::Liquid_Water);
        modifications.record_fluid_level(pos, 0);
        queue.enqueue_with_neighbors(pos);
        edited.push(pos);
    }

    edited
}

fn is_supported_by_ground(world: &VoxelWorld, pos: IVec3) -> bool {
    let below = pos - IVec3::Y;
    let Some(v_below) = world.get_voxel(below) else {
        return false;
    };

    if v_below.is_empty() {
        return false;
    }

    if !v_below.is_fluid() {
        return true;
    }

    let mut check_pos = below - IVec3::Y;
    for _ in 0..32 {
        let Some(v) = world.get_voxel(check_pos) else {
            return false;
        };
        if v.is_empty() {
            return false;
        }
        if !v.is_fluid() {
            return true;
        }
        check_pos -= IVec3::Y;
    }

    false
}

pub fn water_surface_height_offset(world: &impl VoxelAccess, world_voxel: IVec3) -> f32 {
    let voxel = world.get_voxel(world_voxel).unwrap_or(Voxel::Air);
    if !voxel.is_fluid() {
        return 0.0;
    }

    // Submerged fluid blocks have no surface drop.
    let above = world_voxel + IVec3::Y;
    if world.get_voxel(above).is_some_and(Voxel::is_fluid) {
        return 0.0;
    }

    let level = world.get_fluid_level(world_voxel);
    if level == 0 {
        return 0.10;
    }

    let max_spread = voxel.max_fluid_spread().max(1);
    let ratio = (level as f32) / (max_spread as f32);
    let total_offset = 0.10 + ratio * 0.65;
    total_offset.clamp(0.10, 0.85)
}
