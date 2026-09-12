use bevy::prelude::*;

use crate::player::GameMode;

use super::{
    InteractionMode,
    chunk::Voxel,
    fluid::FluidUpdateQueue,
    interaction::affected_chunks,
    light::{VoxelLightRegistry, sync_voxel_light},
    modifications::WorldModificationStore,
    render::{ChunkMaterial, ChunkMeshRegistry, sync_chunk_render},
    targeting::CurrentTarget,
    world::VoxelWorld,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockShape {
    Full,
    Stair,
    StairUpsideDown,
    CornerStair,
    SlabBottom,
    SlabTop,
    VerticalSlab,
    Column,
    CenteredColumn,
}

pub struct ShapingPlugin;

impl Plugin for ShapingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_block_shaping, handle_block_rotation));
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_block_shaping(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_mode: Res<GameMode>,
    interaction_mode: Res<InteractionMode>,
    current_target: Res<CurrentTarget>,
    material: Res<ChunkMaterial>,
    mut fluid_queue: Option<ResMut<FluidUpdateQueue>>,
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut registry: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if *game_mode != GameMode::Creative {
        return;
    }

    if *interaction_mode != InteractionMode::Block {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    let Some(target) = current_target.hit else {
        return;
    };

    let origin = target.block_origin;
    let voxels = get_block_voxels(&world, origin);

    // Find the primary non-empty voxel to preserve block material
    let Some(&block_material) = voxels.iter().find(|v| {
        !v.is_empty() && !v.is_water() && **v != Voxel::Occupied && **v != Voxel::WaterOccupied
    }) else {
        return;
    };

    let next_shape = detect_current_shape(&voxels).next();
    let new_voxels = generate_shape_voxels(next_shape, block_material);

    apply_block_subvoxels(
        &mut commands,
        &mut world,
        &mut modifications,
        &mut light_registry,
        &mut registry,
        &mut meshes,
        &material,
        &mut fluid_queue,
        origin,
        new_voxels,
    );
}

#[allow(clippy::too_many_arguments)]
fn handle_block_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_mode: Res<GameMode>,
    interaction_mode: Res<InteractionMode>,
    current_target: Res<CurrentTarget>,
    material: Res<ChunkMaterial>,
    mut fluid_queue: Option<ResMut<FluidUpdateQueue>>,
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut registry: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if *game_mode != GameMode::Creative {
        return;
    }

    if *interaction_mode != InteractionMode::Block {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyT) {
        return;
    }

    let Some(target) = current_target.hit else {
        return;
    };

    let origin = target.block_origin;
    let voxels = get_block_voxels(&world, origin);

    if voxels.iter().all(|v| v.is_empty() || v.is_water()) {
        return;
    }

    let rotated_voxels = rotate_block_90_y(&voxels);

    apply_block_subvoxels(
        &mut commands,
        &mut world,
        &mut modifications,
        &mut light_registry,
        &mut registry,
        &mut meshes,
        &material,
        &mut fluid_queue,
        origin,
        rotated_voxels,
    );
}

pub fn is_block_submerged_or_adjacent_to_water(world: &VoxelWorld, origin: IVec3) -> bool {
    // 1. Any voxel currently within the block is water
    for dy in 0..2 {
        for dz in 0..2 {
            for dx in 0..2 {
                if world
                    .get_voxel(origin + IVec3::new(dx, dy, dz))
                    .is_some_and(Voxel::is_water)
                {
                    return true;
                }
            }
        }
    }

    // 2. Or any immediate outer neighbor of the 2x2x2 block is water
    for dy in 0..2 {
        for dz in 0..2 {
            for dx in 0..2 {
                let pos = origin + IVec3::new(dx, dy, dz);
                let neighbors = [
                    pos + IVec3::X,
                    pos - IVec3::X,
                    pos + IVec3::Y,
                    pos - IVec3::Y,
                    pos + IVec3::Z,
                    pos - IVec3::Z,
                ];
                for n in neighbors {
                    let local = n - origin;
                    if (0..2).contains(&local.x)
                        && (0..2).contains(&local.y)
                        && (0..2).contains(&local.z)
                    {
                        continue;
                    }
                    if world.get_voxel(n).is_some_and(Voxel::is_water) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[allow(clippy::too_many_arguments)]
fn apply_block_subvoxels(
    commands: &mut Commands,
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    light_registry: &mut VoxelLightRegistry,
    registry: &mut ChunkMeshRegistry,
    meshes: &mut Assets<Mesh>,
    material: &ChunkMaterial,
    fluid_queue: &mut Option<ResMut<FluidUpdateQueue>>,
    block_origin: IVec3,
    mut new_voxels: [Voxel; 8],
) {
    let is_waterlogged = is_block_submerged_or_adjacent_to_water(world, block_origin);
    if is_waterlogged {
        for v in &mut new_voxels {
            if *v == Voxel::Air {
                *v = Voxel::Water;
            } else if *v == Voxel::Occupied {
                *v = Voxel::WaterOccupied;
            }
        }
    }
    let mut edited_voxels = Vec::with_capacity(8);

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..2 {
                let idx = (y * 4 + z * 2 + x) as usize;
                let position = block_origin + IVec3::new(x, y, z);
                let new_voxel = new_voxels[idx];

                if let Some(current) = world.get_voxel(position)
                    && current != new_voxel
                {
                    world.set_voxel(position, new_voxel);
                    modifications.record(position, new_voxel);
                    edited_voxels.push(position);
                }
            }
        }
    }

    if edited_voxels.is_empty() {
        return;
    }

    if let Some(queue) = fluid_queue.as_deref_mut() {
        for &pos in &edited_voxels {
            queue.enqueue_with_neighbors(pos);
        }
    }

    let mut dirty_chunks = Vec::new();

    for edited_voxel in edited_voxels {
        sync_voxel_light(commands, world, edited_voxel, light_registry);
        dirty_chunks.extend(affected_chunks(edited_voxel));
    }

    dirty_chunks.sort_by_key(|chunk| (chunk.x, chunk.y, chunk.z));
    dirty_chunks.dedup();

    for coordinate in dirty_chunks {
        if world.get_chunk(coordinate).is_none() {
            continue;
        }

        sync_chunk_render(commands, world, coordinate, registry, meshes, material);
    }
}

pub fn get_block_voxels(world: &VoxelWorld, block_origin: IVec3) -> [Voxel; 8] {
    let mut voxels = [Voxel::Air; 8];

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..2 {
                let idx = (y * 4 + z * 2 + x) as usize;
                let position = block_origin + IVec3::new(x, y, z);
                voxels[idx] = world.get_voxel(position).unwrap_or(Voxel::Air);
            }
        }
    }

    voxels
}

pub fn centered_layer_coordinates(world_voxel: IVec3) -> [IVec3; 4] {
    let bx = world_voxel.x.div_euclid(2) * 2;
    let bz = world_voxel.z.div_euclid(2) * 2;
    let y = world_voxel.y;
    [
        IVec3::new(bx, y, bz),
        IVec3::new(bx + 1, y, bz),
        IVec3::new(bx, y, bz + 1),
        IVec3::new(bx + 1, y, bz + 1),
    ]
}

pub fn is_centered_layer(world: &VoxelWorld, world_voxel: IVec3) -> bool {
    let coords = centered_layer_coordinates(world_voxel);
    coords.iter().any(|&pos| {
        world.get_voxel(pos) == Some(Voxel::Occupied)
            || world.get_voxel(pos) == Some(Voxel::WaterOccupied)
    })
}

pub fn get_centered_layer_material(world: &VoxelWorld, world_voxel: IVec3) -> Option<Voxel> {
    let coords = centered_layer_coordinates(world_voxel);
    for pos in coords {
        if let Some(voxel) = world.get_voxel(pos)
            && !voxel.is_empty()
            && voxel != Voxel::Occupied
            && voxel != Voxel::WaterOccupied
        {
            return Some(voxel);
        }
    }
    None
}

pub fn is_layer_centered(layer_voxels: &[Voxel]) -> bool {
    layer_voxels.contains(&Voxel::Occupied) || layer_voxels.contains(&Voxel::WaterOccupied)
}

pub fn is_centered_column(voxels: &[Voxel; 8]) -> bool {
    is_layer_centered(&voxels[0..4]) || is_layer_centered(&voxels[4..8])
}

pub fn detect_current_shape(voxels: &[Voxel; 8]) -> BlockShape {
    if is_centered_column(voxels) {
        return BlockShape::CenteredColumn;
    }

    let is_solid = |v: &Voxel| {
        !v.is_empty() && !v.is_water() && *v != Voxel::Occupied && *v != Voxel::WaterOccupied
    };

    let solid_count = voxels.iter().filter(|v| is_solid(v)).count();

    match solid_count {
        8 => BlockShape::Full,
        6 => {
            let bottom_count = voxels[0..4].iter().filter(|v| is_solid(v)).count();
            let top_count = voxels[4..8].iter().filter(|v| is_solid(v)).count();
            if top_count == 4 && bottom_count == 2 {
                BlockShape::StairUpsideDown
            } else {
                BlockShape::Stair
            }
        }
        5 => BlockShape::CornerStair,
        4 => {
            let bottom_count = voxels[0..4].iter().filter(|v| is_solid(v)).count();
            let top_count = voxels[4..8].iter().filter(|v| is_solid(v)).count();
            if bottom_count == 4 {
                BlockShape::SlabBottom
            } else if top_count == 4 {
                BlockShape::SlabTop
            } else {
                BlockShape::VerticalSlab
            }
        }
        2 => BlockShape::Column,
        _ => BlockShape::Full,
    }
}

impl BlockShape {
    pub fn next(self) -> Self {
        match self {
            Self::Full => Self::Stair,
            Self::Stair => Self::StairUpsideDown,
            Self::StairUpsideDown => Self::CornerStair,
            Self::CornerStair => Self::SlabBottom,
            Self::SlabBottom => Self::SlabTop,
            Self::SlabTop => Self::VerticalSlab,
            Self::VerticalSlab => Self::Column,
            Self::Column => Self::CenteredColumn,
            Self::CenteredColumn => Self::Full,
        }
    }
}

pub fn generate_shape_voxels(shape: BlockShape, material: Voxel) -> [Voxel; 8] {
    let mut result = [Voxel::Air; 8];
    let fill = material;

    match shape {
        BlockShape::Full => {
            result = [fill; 8];
        }
        BlockShape::Stair => {
            // Bottom 4 voxels solid
            result[0] = fill;
            result[1] = fill;
            result[2] = fill;
            result[3] = fill;
            // Top back 2 voxels solid (y=1, z=1)
            result[6] = fill;
            result[7] = fill;
        }
        BlockShape::StairUpsideDown => {
            // Top 4 voxels solid (y=1)
            result[4] = fill;
            result[5] = fill;
            result[6] = fill;
            result[7] = fill;
            // Bottom back 2 voxels solid (y=0, z=1)
            result[2] = fill;
            result[3] = fill;
        }
        BlockShape::CornerStair => {
            // Bottom 4 voxels solid (y=0)
            result[0] = fill;
            result[1] = fill;
            result[2] = fill;
            result[3] = fill;
            // Top back-right 1 voxel solid (y=1, z=1, x=1)
            result[7] = fill;
        }
        BlockShape::SlabBottom => {
            // Bottom 4 voxels solid (y=0)
            result[0] = fill;
            result[1] = fill;
            result[2] = fill;
            result[3] = fill;
        }
        BlockShape::SlabTop => {
            // Top 4 voxels solid (y=1)
            result[4] = fill;
            result[5] = fill;
            result[6] = fill;
            result[7] = fill;
        }
        BlockShape::VerticalSlab => {
            // Back 4 voxels solid (z=1, both y=0 and y=1)
            result[2] = fill;
            result[3] = fill;
            result[6] = fill;
            result[7] = fill;
        }
        BlockShape::Column => {
            // 2 voxels tall vertically at x=0, z=0
            result[0] = fill;
            result[4] = fill;
        }
        BlockShape::CenteredColumn => {
            // Bottom centered voxel: layer y=0
            result[0] = fill;
            result[1] = Voxel::Occupied;
            result[2] = Voxel::Occupied;
            result[3] = Voxel::Occupied;

            // Top centered voxel: layer y=1
            result[4] = fill;
            result[5] = Voxel::Occupied;
            result[6] = Voxel::Occupied;
            result[7] = Voxel::Occupied;
        }
    }

    result
}

pub fn rotate_block_90_y(voxels: &[Voxel; 8]) -> [Voxel; 8] {
    let mut rotated = [Voxel::Air; 8];

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..2 {
                let old_idx = (y * 4 + z * 2 + x) as usize;
                let new_x = 1 - z;
                let new_z = x;
                let new_idx = (y * 4 + new_z * 2 + new_x) as usize;
                rotated[new_idx] = voxels[old_idx];
            }
        }
    }

    rotated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_generation_produces_correct_solid_counts() {
        let mat = Voxel::Stone;
        let count_solid = |arr: [Voxel; 8]| arr.iter().filter(|v| !v.is_empty()).count();

        assert_eq!(count_solid(generate_shape_voxels(BlockShape::Full, mat)), 8);
        assert_eq!(
            count_solid(generate_shape_voxels(BlockShape::Stair, mat)),
            6
        );
        assert_eq!(
            count_solid(generate_shape_voxels(BlockShape::StairUpsideDown, mat)),
            6
        );
        assert_eq!(
            count_solid(generate_shape_voxels(BlockShape::CornerStair, mat)),
            5
        );
        assert_eq!(
            count_solid(generate_shape_voxels(BlockShape::SlabBottom, mat)),
            4
        );
        assert_eq!(
            count_solid(generate_shape_voxels(BlockShape::SlabTop, mat)),
            4
        );
        assert_eq!(
            count_solid(generate_shape_voxels(BlockShape::VerticalSlab, mat)),
            4
        );
        assert_eq!(
            count_solid(generate_shape_voxels(BlockShape::Column, mat)),
            2
        );
        assert_eq!(
            count_solid(generate_shape_voxels(BlockShape::CenteredColumn, mat)),
            8
        );
    }

    #[test]
    fn shape_progression_cycles_fully() {
        let mut shape = BlockShape::Full;
        let expected = [
            BlockShape::Stair,
            BlockShape::StairUpsideDown,
            BlockShape::CornerStair,
            BlockShape::SlabBottom,
            BlockShape::SlabTop,
            BlockShape::VerticalSlab,
            BlockShape::Column,
            BlockShape::CenteredColumn,
            BlockShape::Full,
        ];

        for exp in expected {
            shape = shape.next();
            assert_eq!(shape, exp);
        }
    }

    #[test]
    fn block_rotation_four_times_returns_to_original() {
        let stair = generate_shape_voxels(BlockShape::Stair, Voxel::Stone);
        let r1 = rotate_block_90_y(&stair);
        let r2 = rotate_block_90_y(&r1);
        let r3 = rotate_block_90_y(&r2);
        let r4 = rotate_block_90_y(&r3);

        assert_eq!(stair, r4);
        assert_ne!(stair, r1);

        let corner = generate_shape_voxels(BlockShape::CornerStair, Voxel::Stone);
        let cr1 = rotate_block_90_y(&corner);
        let cr2 = rotate_block_90_y(&cr1);
        let cr3 = rotate_block_90_y(&cr2);
        let cr4 = rotate_block_90_y(&cr3);

        assert_eq!(corner, cr4);
        assert_ne!(corner, cr1);

        let upside_down = generate_shape_voxels(BlockShape::StairUpsideDown, Voxel::Stone);
        let ur1 = rotate_block_90_y(&upside_down);
        let ur2 = rotate_block_90_y(&ur1);
        let ur3 = rotate_block_90_y(&ur2);
        let ur4 = rotate_block_90_y(&ur3);

        assert_eq!(upside_down, ur4);
        assert_ne!(upside_down, ur1);
    }

    #[test]
    fn centered_column_detected_and_cycled() {
        let col = generate_shape_voxels(BlockShape::CenteredColumn, Voxel::Stone);
        assert_eq!(detect_current_shape(&col), BlockShape::CenteredColumn);
        assert_eq!(detect_current_shape(&col).next(), BlockShape::Full);
    }

    #[test]
    fn new_stairs_shapes_detected_correctly() {
        let upside_down = generate_shape_voxels(BlockShape::StairUpsideDown, Voxel::Stone);
        assert_eq!(
            detect_current_shape(&upside_down),
            BlockShape::StairUpsideDown
        );

        let corner = generate_shape_voxels(BlockShape::CornerStair, Voxel::Stone);
        assert_eq!(detect_current_shape(&corner), BlockShape::CornerStair);
    }

    #[test]
    fn submerged_block_detected_as_waterlogged() {
        use crate::voxel::chunk::Chunk;
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());

        let origin = IVec3::new(2, 2, 2);
        // Initially dry
        assert!(!is_block_submerged_or_adjacent_to_water(&world, origin));

        // Water above the block (submerged)
        world.set_voxel(origin + IVec3::new(0, 2, 0), Voxel::Water);
        assert!(is_block_submerged_or_adjacent_to_water(&world, origin));
    }

    #[test]
    fn waterlogged_stair_detected_as_stair() {
        let mut stair = generate_shape_voxels(BlockShape::Stair, Voxel::Stone);
        for v in &mut stair {
            if *v == Voxel::Air {
                *v = Voxel::Water;
            }
        }
        assert_eq!(detect_current_shape(&stair), BlockShape::Stair);
        assert_eq!(
            detect_current_shape(&stair).next(),
            BlockShape::StairUpsideDown
        );
    }
}
