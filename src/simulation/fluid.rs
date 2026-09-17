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

pub const MAX_FULL_WATER_SPREAD: u8 = 8;
pub const MAX_SINGLE_WATER_SPREAD: u8 = 4;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaterInfo {
    pub distance: u8,
    pub is_full_block: bool,
}

impl WaterInfo {
    pub fn max_spread(self) -> u8 {
        if self.is_full_block {
            MAX_FULL_WATER_SPREAD
        } else {
            MAX_SINGLE_WATER_SPREAD
        }
    }
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

#[allow(clippy::too_many_arguments)]
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

    let mut edited_voxels = Vec::new();
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

        if let Some((player_chunk, sim_dist)) = player_sim_ctx {
            if !is_in_simulation_radius(pos, player_chunk, sim_dist) {
                // Beyond decoupled simulation radius: drop non-visible dynamic fluid tick
                continue;
            }
        }

        let Some(current_voxel) = world.get_voxel(pos) else {
            continue;
        };

        match current_voxel {
            Voxel::Water => {
                let changed = process_water_source(&mut world, &mut modifications, &mut queue, pos);
                edited_voxels.extend(changed);
            }
            Voxel::WaterFlowing => {
                let changed =
                    process_water_flowing(&mut world, &mut modifications, &mut queue, pos);
                edited_voxels.extend(changed);
            }
            Voxel::WaterOccupied => {
                let changed =
                    process_water_occupied(&mut world, &mut modifications, &mut queue, pos);
                edited_voxels.extend(changed);
            }
            Voxel::Air => {
                let changed =
                    check_infinite_source(&mut world, &mut modifications, &mut queue, pos);
                edited_voxels.extend(changed);
            }
            _ => {}
        }

        updates_this_tick += 1;
    }

    if edited_voxels.is_empty() {
        return;
    }

    let mut dirty_chunks = Vec::new();

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

fn process_water_source(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    queue: &mut FluidUpdateQueue,
    pos: IVec3,
) -> Vec<IVec3> {
    let mut edited = Vec::new();

    let below = pos - IVec3::Y;
    if let Some(voxel_below) = world.get_voxel(below) {
        if voxel_below == Voxel::Air {
            world.set_voxel(below, Voxel::WaterFlowing);
            modifications.record(below, Voxel::WaterFlowing);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        } else if voxel_below == Voxel::Occupied {
            world.set_voxel(below, Voxel::WaterOccupied);
            modifications.record(below, Voxel::WaterOccupied);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        }
    }

    if !is_supported_by_ground(world, pos) {
        return edited;
    }

    let horizontals = [
        pos + IVec3::X,
        pos - IVec3::X,
        pos + IVec3::Z,
        pos - IVec3::Z,
    ];

    for neighbor in horizontals {
        if let Some(v) = world.get_voxel(neighbor) {
            if v == Voxel::Air {
                world.set_voxel(neighbor, Voxel::WaterFlowing);
                modifications.record(neighbor, Voxel::WaterFlowing);
                queue.enqueue_with_neighbors(neighbor);
                edited.push(neighbor);
            } else if v == Voxel::Occupied {
                world.set_voxel(neighbor, Voxel::WaterOccupied);
                modifications.record(neighbor, Voxel::WaterOccupied);
                queue.enqueue_with_neighbors(neighbor);
                edited.push(neighbor);
            }
        }
    }

    edited
}

fn process_water_flowing(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    queue: &mut FluidUpdateQueue,
    pos: IVec3,
) -> Vec<IVec3> {
    let mut edited = Vec::new();

    if !has_adjacent_water(world, pos) {
        world.set_voxel(pos, Voxel::Air);
        modifications.record(pos, Voxel::Air);
        queue.enqueue_with_neighbors(pos);
        edited.push(pos);
        return edited;
    }

    let Some(info) = compute_water_info(world, pos) else {
        world.set_voxel(pos, Voxel::Air);
        modifications.record(pos, Voxel::Air);
        queue.enqueue_with_neighbors(pos);
        edited.push(pos);
        return edited;
    };

    let below = pos - IVec3::Y;
    if let Some(voxel_below) = world.get_voxel(below) {
        if voxel_below == Voxel::Air {
            world.set_voxel(below, Voxel::WaterFlowing);
            modifications.record(below, Voxel::WaterFlowing);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        } else if voxel_below == Voxel::Occupied {
            world.set_voxel(below, Voxel::WaterOccupied);
            modifications.record(below, Voxel::WaterOccupied);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        }
    }

    if !is_supported_by_ground(world, pos) {
        return edited;
    }

    if info.distance < info.max_spread() {
        let horizontals = [
            pos + IVec3::X,
            pos - IVec3::X,
            pos + IVec3::Z,
            pos - IVec3::Z,
        ];

        for neighbor in horizontals {
            if let Some(v) = world.get_voxel(neighbor) {
                if v == Voxel::Air {
                    world.set_voxel(neighbor, Voxel::WaterFlowing);
                    modifications.record(neighbor, Voxel::WaterFlowing);
                    queue.enqueue_with_neighbors(neighbor);
                    edited.push(neighbor);
                } else if v == Voxel::Occupied {
                    world.set_voxel(neighbor, Voxel::WaterOccupied);
                    modifications.record(neighbor, Voxel::WaterOccupied);
                    queue.enqueue_with_neighbors(neighbor);
                    edited.push(neighbor);
                }
            }
        }
    }

    edited
}

fn process_water_occupied(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    queue: &mut FluidUpdateQueue,
    pos: IVec3,
) -> Vec<IVec3> {
    let mut edited = Vec::new();

    if !has_adjacent_water(world, pos) {
        world.set_voxel(pos, Voxel::Occupied);
        modifications.record(pos, Voxel::Occupied);
        queue.enqueue_with_neighbors(pos);
        edited.push(pos);
        return edited;
    }

    let Some(info) = compute_water_info(world, pos) else {
        world.set_voxel(pos, Voxel::Occupied);
        modifications.record(pos, Voxel::Occupied);
        queue.enqueue_with_neighbors(pos);
        edited.push(pos);
        return edited;
    };

    let below = pos - IVec3::Y;
    if let Some(voxel_below) = world.get_voxel(below) {
        if voxel_below == Voxel::Air {
            world.set_voxel(below, Voxel::WaterFlowing);
            modifications.record(below, Voxel::WaterFlowing);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        } else if voxel_below == Voxel::Occupied {
            world.set_voxel(below, Voxel::WaterOccupied);
            modifications.record(below, Voxel::WaterOccupied);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        }
    }

    if !is_supported_by_ground(world, pos) {
        return edited;
    }

    if info.distance < info.max_spread() {
        let horizontals = [
            pos + IVec3::X,
            pos - IVec3::X,
            pos + IVec3::Z,
            pos - IVec3::Z,
        ];

        for neighbor in horizontals {
            if let Some(v) = world.get_voxel(neighbor) {
                if v == Voxel::Air {
                    world.set_voxel(neighbor, Voxel::WaterFlowing);
                    modifications.record(neighbor, Voxel::WaterFlowing);
                    queue.enqueue_with_neighbors(neighbor);
                    edited.push(neighbor);
                } else if v == Voxel::Occupied {
                    world.set_voxel(neighbor, Voxel::WaterOccupied);
                    modifications.record(neighbor, Voxel::WaterOccupied);
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
    let mut edited = Vec::new();

    let below = pos - IVec3::Y;
    let Some(v_below) = world.get_voxel(below) else {
        return edited;
    };
    if v_below == Voxel::Air {
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
        .filter(|&&n| world.get_voxel(n) == Some(Voxel::Water))
        .count();

    if source_count >= 2 {
        world.set_voxel(pos, Voxel::Water);
        modifications.record(pos, Voxel::Water);
        queue.enqueue_with_neighbors(pos);
        edited.push(pos);
    }

    edited
}

fn has_adjacent_water(world: &VoxelWorld, pos: IVec3) -> bool {
    let neighbors = [
        pos + IVec3::Y,
        pos - IVec3::Y,
        pos + IVec3::X,
        pos - IVec3::X,
        pos + IVec3::Z,
        pos - IVec3::Z,
    ];

    neighbors.iter().any(|&n| {
        world
            .get_voxel(n)
            .is_some_and(|v| v == Voxel::Water || v == Voxel::WaterFlowing)
    })
}

fn is_supported_by_ground(world: &VoxelWorld, pos: IVec3) -> bool {
    let below = pos - IVec3::Y;
    let Some(v_below) = world.get_voxel(below) else {
        return false;
    };

    if v_below.is_empty() {
        return false;
    }

    if !v_below.is_water() && v_below != Voxel::Occupied && v_below != Voxel::WaterOccupied {
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
        if !v.is_water() && v != Voxel::Occupied && v != Voxel::WaterOccupied {
            return true;
        }
        check_pos -= IVec3::Y;
    }

    false
}

pub fn is_full_block_source(world: &impl VoxelAccess, pos: IVec3) -> bool {
    let above = pos + IVec3::Y;
    let below = pos - IVec3::Y;
    world.get_voxel(above).is_some_and(Voxel::is_water)
        || world.get_voxel(below).is_some_and(Voxel::is_water)
}

pub fn compute_water_info(world: &impl VoxelAccess, pos: IVec3) -> Option<WaterInfo> {
    let above = pos + IVec3::Y;
    if let Some(v_above) = world.get_voxel(above)
        && (v_above == Voxel::Water || v_above == Voxel::WaterFlowing)
    {
        let two_above = pos + IVec3::new(0, 2, 0);
        if world.get_voxel(two_above).is_some_and(Voxel::is_water) {
            return Some(WaterInfo {
                distance: 1,
                is_full_block: true,
            });
        }
    }

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back((pos, 0u8));
    visited.insert(pos);

    while let Some((curr, dist)) = queue.pop_front() {
        if dist >= MAX_FULL_WATER_SPREAD {
            continue;
        }

        let horizontals = [
            curr + IVec3::X,
            curr - IVec3::X,
            curr + IVec3::Z,
            curr - IVec3::Z,
        ];

        for neighbor in horizontals {
            if visited.contains(&neighbor) {
                continue;
            }

            let Some(v) = world.get_voxel(neighbor) else {
                continue;
            };

            let next_dist = dist + 1;

            match v {
                Voxel::Water => {
                    let is_full = is_full_block_source(world, neighbor);
                    let info = WaterInfo {
                        distance: next_dist,
                        is_full_block: is_full,
                    };
                    if next_dist <= info.max_spread() {
                        return Some(info);
                    }
                }
                Voxel::WaterFlowing | Voxel::WaterOccupied => {
                    let n_above = neighbor + IVec3::Y;
                    let n_two_above = neighbor + IVec3::new(0, 2, 0);
                    if world.get_voxel(n_above).is_some_and(Voxel::is_water)
                        && world.get_voxel(n_two_above).is_some_and(Voxel::is_water)
                    {
                        let info = WaterInfo {
                            distance: next_dist,
                            is_full_block: true,
                        };
                        if next_dist <= MAX_FULL_WATER_SPREAD {
                            return Some(info);
                        }
                    }

                    visited.insert(neighbor);
                    queue.push_back((neighbor, next_dist));
                }
                _ => {}
            }
        }
    }

    None
}

#[allow(dead_code)]
pub fn compute_water_distance(world: &impl VoxelAccess, pos: IVec3) -> u8 {
    compute_water_info(world, pos).map_or(u8::MAX, |info| info.distance)
}

pub fn water_surface_height_offset(world: &impl VoxelAccess, world_voxel: IVec3) -> f32 {
    let voxel = world.get_voxel(world_voxel).unwrap_or(Voxel::Air);
    if !voxel.is_water() {
        return 0.0;
    }

    if voxel == Voxel::Water {
        let is_full = is_full_block_source(world, world_voxel);
        return if is_full { 0.10 } else { 0.05 };
    }

    let Some(info) = compute_water_info(world, world_voxel) else {
        return 0.40;
    };

    let below = world_voxel - IVec3::Y;
    let is_top_layer = world.get_voxel(below).is_some_and(Voxel::is_water);

    if info.is_full_block {
        let total_height = (0.90 - info.distance as f32 * 0.10).clamp(0.10, 0.90);
        if is_top_layer {
            let voxel_height = (total_height - 0.50).max(0.10);
            (VOXEL_SIZE - voxel_height).clamp(0.0, 0.40)
        } else {
            let voxel_height = total_height.min(VOXEL_SIZE);
            (VOXEL_SIZE - voxel_height).clamp(0.0, 0.40)
        }
    } else {
        let total_height = (0.50 - info.distance as f32 * 0.10).clamp(0.10, 0.50);
        let voxel_height = total_height.min(VOXEL_SIZE);
        (VOXEL_SIZE - voxel_height).clamp(0.0, 0.40)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Chunk;

    #[test]
    fn infinite_source_creates_water_with_two_sources_over_floor() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());
        let mut modifications = WorldModificationStore::default();
        let mut queue = FluidUpdateQueue::default();

        let center = IVec3::new(2, 1, 2);
        world.set_voxel(center - IVec3::Y, Voxel::Stone);

        world.set_voxel(center + IVec3::X, Voxel::Water);
        world.set_voxel(center + IVec3::Z, Voxel::Water);

        let changed = check_infinite_source(&mut world, &mut modifications, &mut queue, center);
        assert_eq!(changed, vec![center]);
        assert_eq!(world.get_voxel(center), Some(Voxel::Water));
    }

    #[test]
    fn water_falls_downward_first() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());
        let mut modifications = WorldModificationStore::default();
        let mut queue = FluidUpdateQueue::default();

        let source = IVec3::new(2, 3, 2);
        world.set_voxel(source, Voxel::Water);

        let changed = process_water_source(&mut world, &mut modifications, &mut queue, source);
        assert_eq!(changed, vec![source - IVec3::Y]);
        assert_eq!(
            world.get_voxel(source - IVec3::Y),
            Some(Voxel::WaterFlowing)
        );
    }

    #[test]
    fn compute_water_distance_finds_multi_block_source() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());

        world.set_voxel(IVec3::new(1, 1, 1), Voxel::Water);
        world.set_voxel(IVec3::new(2, 1, 1), Voxel::WaterFlowing);
        world.set_voxel(IVec3::new(3, 1, 1), Voxel::WaterFlowing);
        world.set_voxel(IVec3::new(4, 1, 1), Voxel::WaterFlowing);

        assert_eq!(compute_water_distance(&world, IVec3::new(2, 1, 1)), 1);
        assert_eq!(compute_water_distance(&world, IVec3::new(3, 1, 1)), 2);
        assert_eq!(compute_water_distance(&world, IVec3::new(4, 1, 1)), 3);
        assert_eq!(compute_water_distance(&world, IVec3::new(5, 1, 1)), 4);
        assert_eq!(compute_water_distance(&world, IVec3::new(6, 1, 1)), u8::MAX);
    }

    #[test]
    fn full_block_water_spreads_up_to_eight_voxels() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());

        world.set_voxel(IVec3::new(1, 1, 1), Voxel::Water);
        world.set_voxel(IVec3::new(1, 2, 1), Voxel::Water);

        for x in 2..=8 {
            world.set_voxel(IVec3::new(x, 1, 1), Voxel::WaterFlowing);
        }

        assert_eq!(compute_water_distance(&world, IVec3::new(8, 1, 1)), 7);
        assert_eq!(compute_water_distance(&world, IVec3::new(9, 1, 1)), 8);
        assert_eq!(
            compute_water_distance(&world, IVec3::new(10, 1, 1)),
            u8::MAX
        );
    }

    #[test]
    fn water_surface_height_decreases_by_ten_cm_per_step() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());

        world.set_voxel(IVec3::new(1, 1, 1), Voxel::Water);
        for x in 2..=5 {
            world.set_voxel(IVec3::new(x, 1, 1), Voxel::WaterFlowing);
        }

        assert!((water_surface_height_offset(&world, IVec3::new(1, 1, 1)) - 0.05).abs() < 1e-4);
        assert!((water_surface_height_offset(&world, IVec3::new(2, 1, 1)) - 0.10).abs() < 1e-4);
        assert!((water_surface_height_offset(&world, IVec3::new(3, 1, 1)) - 0.20).abs() < 1e-4);
        assert!((water_surface_height_offset(&world, IVec3::new(4, 1, 1)) - 0.30).abs() < 1e-4);
        assert!((water_surface_height_offset(&world, IVec3::new(5, 1, 1)) - 0.40).abs() < 1e-4);
    }

    #[test]
    fn water_in_midair_does_not_spread_horizontally() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());
        let mut modifications = WorldModificationStore::default();
        let mut queue = FluidUpdateQueue::default();

        let source = IVec3::new(2, 5, 2);
        world.set_voxel(source, Voxel::Water);

        let changed = process_water_source(&mut world, &mut modifications, &mut queue, source);
        assert_eq!(changed, vec![source - IVec3::Y]);
        assert_eq!(world.get_voxel(source + IVec3::X), Some(Voxel::Air));
        assert_eq!(world.get_voxel(source + IVec3::Z), Some(Voxel::Air));

        let changed2 = process_water_source(&mut world, &mut modifications, &mut queue, source);
        assert!(changed2.is_empty());
        assert_eq!(world.get_voxel(source + IVec3::X), Some(Voxel::Air));
        assert_eq!(world.get_voxel(source + IVec3::Z), Some(Voxel::Air));
    }

    #[test]
    fn simulation_radius_bounds_check() {
        let player_chunk = IVec3::new(0, 0, 0);
        let sim_dist = 4;

        // Voxel in player's chunk (0, 0, 0) is inside
        assert!(is_in_simulation_radius(
            IVec3::new(5, 5, 5),
            player_chunk,
            sim_dist
        ));

        // Voxel in chunk (3, 0, 2): dx=3, dz=2 -> 9+4=13 <= 16 -> inside
        let inside_voxel = IVec3::new(3 * 16 + 2, 8, 2 * 16 + 4);
        assert!(is_in_simulation_radius(
            inside_voxel,
            player_chunk,
            sim_dist
        ));

        // Voxel in chunk (4, 0, 0): dx=4, dz=0 -> 16 <= 16 -> inside boundary
        let boundary_voxel = IVec3::new(4 * 16 + 1, 8, 4);
        assert!(is_in_simulation_radius(
            boundary_voxel,
            player_chunk,
            sim_dist
        ));

        // Voxel in chunk (4, 0, 4): dx=4, dz=4 -> 16+16=32 > 16 -> outside
        let outside_diagonal = IVec3::new(4 * 16 + 1, 8, 4 * 16 + 1);
        assert!(!is_in_simulation_radius(
            outside_diagonal,
            player_chunk,
            sim_dist
        ));

        // Voxel in chunk (0, 5, 0): dy=5 > sim_dist=4 -> outside vertical
        let outside_vertical = IVec3::new(5, 5 * 16 + 2, 5);
        assert!(!is_in_simulation_radius(
            outside_vertical,
            player_chunk,
            sim_dist
        ));
    }
}
