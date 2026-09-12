use std::collections::{HashSet, VecDeque};

use bevy::prelude::*;

use super::{
    chunk::{VOXEL_SIZE, Voxel},
    interaction::affected_chunks,
    light::{VoxelLightRegistry, sync_voxel_light},
    modifications::WorldModificationStore,
    render::{ChunkMaterial, ChunkMeshRegistry, sync_chunk_render},
    world::VoxelWorld,
};

pub const MAX_FULL_WATER_SPREAD: u8 = 8;
pub const MAX_SINGLE_WATER_SPREAD: u8 = 4;
const FLUID_TICK_SECONDS: f32 = 0.25;
const MAX_UPDATES_PER_TICK: usize = 256;

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
    mut registry: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ChunkMaterial>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    if queue.queue.is_empty() {
        return;
    }

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

        let Some(current_voxel) = world.get_voxel(pos) else {
            continue;
        };

        match current_voxel {
            Voxel::Water => {
                // Water source block: spreads downward, or horizontally if supported
                let changed = process_water_source(&mut world, &mut modifications, &mut queue, pos);
                edited_voxels.extend(changed);
            }

            Voxel::WaterFlowing => {
                // Flowing water: checks if source still supplies it, otherwise evaporates; spreads downward/horizontally
                let changed =
                    process_water_flowing(&mut world, &mut modifications, &mut queue, pos);
                edited_voxels.extend(changed);
            }

            Voxel::WaterOccupied => {
                // Waterlogged occupied space in centered column: checks if water is still present
                let changed =
                    process_water_occupied(&mut world, &mut modifications, &mut queue, pos);
                edited_voxels.extend(changed);
            }

            Voxel::Air => {
                // Check infinite source creation: 2+ adjacent source blocks over solid/water
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

fn process_water_source(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    queue: &mut FluidUpdateQueue,
    pos: IVec3,
) -> Vec<IVec3> {
    let mut edited = Vec::new();

    // 1. Downward spread priority
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

    // 2. Horizontal spread if downward is blocked or already water, AND supported by ground
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

    let below = pos - IVec3::Y;
    let voxel_below = world.get_voxel(below);
    let is_top_layer = voxel_below.is_some_and(Voxel::is_water);

    let Some(info) = compute_water_info(world, pos) else {
        // Evaporate / recede
        world.set_voxel(pos, Voxel::Air);
        modifications.record(pos, Voxel::Air);
        queue.enqueue_with_neighbors(pos);
        edited.push(pos);
        return edited;
    };

    // If this is a top layer of a stream, it only exists for full block sources at distance <= 3
    if is_top_layer && (!info.is_full_block || info.distance > 3) {
        let two_below = pos - IVec3::new(0, 2, 0);
        let is_falling_column = world.get_voxel(two_below) == Some(Voxel::Air);
        if !is_falling_column {
            world.set_voxel(pos, Voxel::Air);
            modifications.record(pos, Voxel::Air);
            queue.enqueue_with_neighbors(pos);
            edited.push(pos);
            return edited;
        }
    }

    if !is_top_layer && info.distance > info.max_spread() {
        world.set_voxel(pos, Voxel::Air);
        modifications.record(pos, Voxel::Air);
        queue.enqueue_with_neighbors(pos);
        edited.push(pos);
        return edited;
    }

    // Downward spread priority
    if let Some(v_below) = voxel_below {
        if v_below == Voxel::Air {
            world.set_voxel(below, Voxel::WaterFlowing);
            modifications.record(below, Voxel::WaterFlowing);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        } else if v_below == Voxel::Occupied {
            world.set_voxel(below, Voxel::WaterOccupied);
            modifications.record(below, Voxel::WaterOccupied);
            queue.enqueue_with_neighbors(below);
            edited.push(below);
            return edited;
        } else if v_below == Voxel::WaterFlowing {
            let two_below = pos - IVec3::new(0, 2, 0);
            if world.get_voxel(two_below) == Some(Voxel::Air) {
                // Falling column in mid-air
                return edited;
            }
        }
    }

    // Horizontal spread: allowed only if supported by solid ground (not mid-air)
    let can_spread = is_supported_by_ground(world, pos)
        && if is_top_layer {
            info.is_full_block && info.distance < 3
        } else {
            info.distance < info.max_spread()
        };

    if can_spread {
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
    let has_water_neighbor = has_adjacent_water(world, pos);

    if !has_water_neighbor {
        world.set_voxel(pos, Voxel::Occupied);
        modifications.record(pos, Voxel::Occupied);
        queue.enqueue_with_neighbors(pos);
        edited.push(pos);
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

    // Check floor below: must be solid or water
    let below = pos - IVec3::Y;
    let Some(v_below) = world.get_voxel(below) else {
        return edited;
    };
    if v_below == Voxel::Air {
        return edited;
    }

    // Count horizontal source water neighbors
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

pub fn is_full_block_source(world: &VoxelWorld, pos: IVec3) -> bool {
    let above = pos + IVec3::Y;
    let below = pos - IVec3::Y;
    world.get_voxel(above).is_some_and(Voxel::is_water)
        || world.get_voxel(below).is_some_and(Voxel::is_water)
}

pub fn compute_water_info(world: &VoxelWorld, pos: IVec3) -> Option<WaterInfo> {
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
pub fn compute_water_distance(world: &VoxelWorld, pos: IVec3) -> u8 {
    compute_water_info(world, pos).map_or(u8::MAX, |info| info.distance)
}

pub fn water_surface_height_offset(world: &VoxelWorld, world_voxel: IVec3) -> f32 {
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
    use crate::voxel::chunk::Chunk;

    #[test]
    fn infinite_source_creates_water_with_two_sources_over_floor() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());
        let mut modifications = WorldModificationStore::default();
        let mut queue = FluidUpdateQueue::default();

        let center = IVec3::new(2, 1, 2);
        // Floor at y=0
        world.set_voxel(center - IVec3::Y, Voxel::Stone);

        // Sources at +X and +Z
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

        // Place single-voxel source at (1, 1, 1)
        world.set_voxel(IVec3::new(1, 1, 1), Voxel::Water);
        // Place flowing water in a line along +X
        world.set_voxel(IVec3::new(2, 1, 1), Voxel::WaterFlowing);
        world.set_voxel(IVec3::new(3, 1, 1), Voxel::WaterFlowing);
        world.set_voxel(IVec3::new(4, 1, 1), Voxel::WaterFlowing);

        assert_eq!(compute_water_distance(&world, IVec3::new(2, 1, 1)), 1);
        assert_eq!(compute_water_distance(&world, IVec3::new(3, 1, 1)), 2);
        assert_eq!(compute_water_distance(&world, IVec3::new(4, 1, 1)), 3);
        // Position 5 is air next to 4, its distance to nearest source is 4
        assert_eq!(compute_water_distance(&world, IVec3::new(5, 1, 1)), 4);
        // Position 6 is air, exceeds single-voxel max spread of 4
        assert_eq!(compute_water_distance(&world, IVec3::new(6, 1, 1)), u8::MAX);
    }

    #[test]
    fn full_block_water_spreads_up_to_eight_voxels() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());

        // Place full block source (2 voxels tall) at (1, 1, 1) and (1, 2, 1)
        world.set_voxel(IVec3::new(1, 1, 1), Voxel::Water);
        world.set_voxel(IVec3::new(1, 2, 1), Voxel::Water);

        // Place flowing water along +X from 2 to 8
        for x in 2..=8 {
            world.set_voxel(IVec3::new(x, 1, 1), Voxel::WaterFlowing);
        }

        assert_eq!(compute_water_distance(&world, IVec3::new(8, 1, 1)), 7);
        // Position 9 is air next to 8, distance 8 is within full-block spread (8)
        assert_eq!(compute_water_distance(&world, IVec3::new(9, 1, 1)), 8);
        // Position 10 exceeds max spread of 8
        assert_eq!(
            compute_water_distance(&world, IVec3::new(10, 1, 1)),
            u8::MAX
        );
    }

    #[test]
    fn water_surface_height_decreases_by_ten_cm_per_step() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());

        // Single-voxel stream along +X: source at (1, 1, 1)
        world.set_voxel(IVec3::new(1, 1, 1), Voxel::Water);
        for x in 2..=5 {
            world.set_voxel(IVec3::new(x, 1, 1), Voxel::WaterFlowing);
        }

        // Source has 0.05 offset (height 45cm)
        assert!((water_surface_height_offset(&world, IVec3::new(1, 1, 1)) - 0.05).abs() < 1e-4);
        // Distance 1: 0.10 offset (height 40cm)
        assert!((water_surface_height_offset(&world, IVec3::new(2, 1, 1)) - 0.10).abs() < 1e-4);
        // Distance 2: 0.20 offset (height 30cm)
        assert!((water_surface_height_offset(&world, IVec3::new(3, 1, 1)) - 0.20).abs() < 1e-4);
        // Distance 3: 0.30 offset (height 20cm)
        assert!((water_surface_height_offset(&world, IVec3::new(4, 1, 1)) - 0.30).abs() < 1e-4);
        // Distance 4: 0.40 offset (height 10cm)
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

        // When source is processed again while hanging over air, it must not spread horizontally
        let changed2 = process_water_source(&mut world, &mut modifications, &mut queue, source);
        assert!(changed2.is_empty());
        assert_eq!(world.get_voxel(source + IVec3::X), Some(Voxel::Air));
        assert_eq!(world.get_voxel(source + IVec3::Z), Some(Voxel::Air));
    }
}
