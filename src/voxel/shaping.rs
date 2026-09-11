use bevy::prelude::*;

use crate::player::GameMode;

use super::{
    InteractionMode,
    chunk::Voxel,
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
    SlabBottom,
    SlabTop,
    VerticalSlab,
    Column,
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
    let Some(&block_material) = voxels.iter().find(|v| !v.is_empty()) else {
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

    if voxels.iter().all(|v| v.is_empty()) {
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
        origin,
        rotated_voxels,
    );
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
    block_origin: IVec3,
    new_voxels: [Voxel; 8],
) {
    let mut edited_voxels = Vec::with_capacity(8);

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..2 {
                let idx = (y * 4 + z * 2 + x) as usize;
                let pos = block_origin + IVec3::new(x, y, z);
                let target_voxel = new_voxels[idx];
                let current_voxel = world.get_voxel(pos).unwrap_or(Voxel::Air);

                if current_voxel != target_voxel {
                    world.set_voxel(pos, target_voxel);
                    modifications.record(pos, target_voxel);
                    edited_voxels.push(pos);
                }
            }
        }
    }

    if edited_voxels.is_empty() {
        return;
    }

    let mut dirty_chunks = Vec::new();
    for &edited_voxel in &edited_voxels {
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
    let mut result = [Voxel::Air; 8];
    for y in 0..2 {
        for z in 0..2 {
            for x in 0..2 {
                let idx = (y * 4 + z * 2 + x) as usize;
                let pos = block_origin + IVec3::new(x, y, z);
                result[idx] = world.get_voxel(pos).unwrap_or(Voxel::Air);
            }
        }
    }
    result
}

pub fn detect_current_shape(voxels: &[Voxel; 8]) -> BlockShape {
    let solid_count = voxels.iter().filter(|v| !v.is_empty()).count();

    match solid_count {
        8 => BlockShape::Full,
        6 => BlockShape::Stair,
        4 => {
            let bottom_count = voxels[0..4].iter().filter(|v| !v.is_empty()).count();
            let top_count = voxels[4..8].iter().filter(|v| !v.is_empty()).count();
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
            Self::Stair => Self::SlabBottom,
            Self::SlabBottom => Self::SlabTop,
            Self::SlabTop => Self::VerticalSlab,
            Self::VerticalSlab => Self::Column,
            Self::Column => Self::Full,
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
    }

    #[test]
    fn shape_progression_cycles_fully() {
        let mut shape = BlockShape::Full;
        let expected = [
            BlockShape::Stair,
            BlockShape::SlabBottom,
            BlockShape::SlabTop,
            BlockShape::VerticalSlab,
            BlockShape::Column,
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
    }
}
