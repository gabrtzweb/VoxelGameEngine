use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

use super::{
    radial_menu::{RadialMenuRoot, RadialMenuState, spawn_radial_menu, update_radial_menu_ui},
    targeting::CurrentTarget,
};
use crate::{
    menu::MenuState,
    player::GameMode,
    simulation::{
        fluid::FluidUpdateQueue,
        lighting::{VoxelLightRegistry, sync_voxel_light},
    },
    world::{
        ChunkStreamingQueues, Voxel, VoxelAccess, VoxelWorld, WorldModificationStore,
        affected_chunks,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockShape {
    #[default]
    Full,
    Stair,
    StairUpsideDown,
    CornerStair,
    CornerStairInverted,
    SlabBottom,
    SlabTop,
    VerticalSlab,
    Column,
    CenteredColumn,
}

impl BlockShape {
    pub const fn all() -> [BlockShape; 10] {
        [
            BlockShape::Full,
            BlockShape::Stair,
            BlockShape::StairUpsideDown,
            BlockShape::CornerStair,
            BlockShape::CornerStairInverted,
            BlockShape::SlabBottom,
            BlockShape::SlabTop,
            BlockShape::VerticalSlab,
            BlockShape::Column,
            BlockShape::CenteredColumn,
        ]
    }

    pub const fn index(self) -> usize {
        match self {
            Self::Full => 0,
            Self::Stair => 1,
            Self::StairUpsideDown => 2,
            Self::CornerStair => 3,
            Self::CornerStairInverted => 4,
            Self::SlabBottom => 5,
            Self::SlabTop => 6,
            Self::VerticalSlab => 7,
            Self::Column => 8,
            Self::CenteredColumn => 9,
        }
    }

    #[allow(dead_code)]
    pub const fn from_index(index: usize) -> Self {
        Self::all()[index % 10]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "Full Block",
            Self::Stair => "Stairs",
            Self::StairUpsideDown => "Upside-Down Stairs",
            Self::CornerStair => "Corner Stairs",
            Self::CornerStairInverted => "Inverted Corner Stairs",
            Self::SlabBottom => "Bottom Slab",
            Self::SlabTop => "Top Slab",
            Self::VerticalSlab => "Vertical Slab",
            Self::Column => "Column",
            Self::CenteredColumn => "Centered Column",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Full => "FULL",
            Self::Stair => "STAIR",
            Self::StairUpsideDown => "STAIR-UD",
            Self::CornerStair => "CORNER",
            Self::CornerStairInverted => "INV-CORNER",
            Self::SlabBottom => "SLAB-B",
            Self::SlabTop => "SLAB-T",
            Self::VerticalSlab => "VERT-SLAB",
            Self::Column => "COLUMN",
            Self::CenteredColumn => "CTR-COL",
        }
    }

    pub fn voxel_count(self) -> usize {
        match self {
            Self::Full => 8,
            Self::Stair => 6,
            Self::StairUpsideDown => 6,
            Self::CornerStair => 5,
            Self::CornerStairInverted => 7,
            Self::SlabBottom => 4,
            Self::SlabTop => 4,
            Self::VerticalSlab => 4,
            Self::Column => 2,
            Self::CenteredColumn => 8,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Full => Self::Stair,
            Self::Stair => Self::StairUpsideDown,
            Self::StairUpsideDown => Self::CornerStair,
            Self::CornerStair => Self::CornerStairInverted,
            Self::CornerStairInverted => Self::SlabBottom,
            Self::SlabBottom => Self::SlabTop,
            Self::SlabTop => Self::VerticalSlab,
            Self::VerticalSlab => Self::Column,
            Self::Column => Self::CenteredColumn,
            Self::CenteredColumn => Self::Full,
        }
    }
}

pub struct ShapingPlugin;

impl Plugin for ShapingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RadialMenuState>().add_systems(
            Update,
            (
                handle_block_shaping,
                handle_block_rotation,
                update_radial_menu_ui,
            ),
        );
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_block_shaping(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    game_mode: Res<GameMode>,
    current_target: Res<CurrentTarget>,
    mut fluid_queue: Option<ResMut<FluidUpdateQueue>>,
    mut commands: Commands,
    (mut world, mut modifications, mut light_registry, mut queues): (
        ResMut<VoxelWorld>,
        ResMut<WorldModificationStore>,
        ResMut<VoxelLightRegistry>,
        ResMut<ChunkStreamingQueues>,
    ),
    mut radial_state: ResMut<RadialMenuState>,
    menu_state: Option<Res<State<MenuState>>>,
    radial_root_query: Query<Entity, With<RadialMenuRoot>>,
) {
    if menu_state
        .as_ref()
        .is_some_and(|s| *s.get() != MenuState::None)
    {
        if radial_state.is_open {
            for entity in &radial_root_query {
                commands.entity(entity).despawn();
            }
        }
        *radial_state = RadialMenuState::default();
        return;
    }

    if *game_mode != GameMode::Creative {
        if radial_state.is_open {
            for entity in &radial_root_query {
                commands.entity(entity).despawn();
            }
        }
        *radial_state = RadialMenuState::default();
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) && radial_state.is_open {
        for entity in &radial_root_query {
            commands.entity(entity).despawn();
        }
        *radial_state = RadialMenuState::default();
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR)
        && let Some(target) = current_target.hit
    {
        let origin = target.block_origin;
        let voxels = get_block_voxels(&world, origin);

        if let Some(&block_mat) = voxels.iter().find(|v| {
            !v.is_empty() && !v.is_water() && **v != Voxel::Occupied && **v != Voxel::WaterOccupied
        }) {
            let current_shape = detect_current_shape(&voxels);
            radial_state.pressing = true;
            radial_state.hold_timer = 0.0;
            radial_state.mouse_offset = Vec2::ZERO;
            radial_state.target_origin = Some(origin);
            radial_state.target_material = Some(block_mat);
            radial_state.initial_shape = current_shape;
            radial_state.selected_shape = current_shape;
        }
    }

    if keyboard.pressed(KeyCode::KeyR) && radial_state.pressing {
        radial_state.hold_timer += time.delta_secs();

        if radial_state.hold_timer >= 0.20 && !radial_state.is_open {
            radial_state.is_open = true;
            spawn_radial_menu(&mut commands, radial_state.selected_shape);
        }

        if radial_state.is_open {
            radial_state.mouse_offset += mouse_motion.delta;

            if radial_state.mouse_offset.length() > 20.0 {
                let angle = radial_state
                    .mouse_offset
                    .y
                    .atan2(radial_state.mouse_offset.x);
                let mut rel_angle = angle - (-FRAC_PI_2);
                while rel_angle < 0.0 {
                    rel_angle += TAU;
                }
                while rel_angle >= TAU {
                    rel_angle -= TAU;
                }
                let slice_step = TAU / 10.0;
                let slice_idx =
                    ((rel_angle + (slice_step * 0.5)) / slice_step).floor() as usize % 10;
                radial_state.selected_shape = BlockShape::all()[slice_idx];
            }
        }
    }

    if keyboard.just_released(KeyCode::KeyR) && radial_state.pressing {
        if radial_state.is_open {
            for entity in &radial_root_query {
                commands.entity(entity).despawn();
            }

            if let (Some(origin), Some(mat)) =
                (radial_state.target_origin, radial_state.target_material)
            {
                let new_voxels = generate_shape_voxels(radial_state.selected_shape, mat);
                apply_block_subvoxels(
                    &mut commands,
                    &mut world,
                    &mut modifications,
                    &mut light_registry,
                    &mut queues,
                    &mut fluid_queue,
                    origin,
                    new_voxels,
                );
            }
        } else if radial_state.hold_timer < 0.20
            && let (Some(origin), Some(mat)) =
                (radial_state.target_origin, radial_state.target_material)
        {
            let next_shape = radial_state.initial_shape.next();
            let new_voxels = generate_shape_voxels(next_shape, mat);
            apply_block_subvoxels(
                &mut commands,
                &mut world,
                &mut modifications,
                &mut light_registry,
                &mut queues,
                &mut fluid_queue,
                origin,
                new_voxels,
            );
        }

        *radial_state = RadialMenuState::default();
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_block_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_mode: Res<GameMode>,
    current_target: Res<CurrentTarget>,
    mut fluid_queue: Option<ResMut<FluidUpdateQueue>>,
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut queues: ResMut<ChunkStreamingQueues>,
    menu_state: Option<Res<State<MenuState>>>,
) {
    if menu_state.is_some_and(|s| *s.get() != MenuState::None) {
        return;
    }

    if *game_mode != GameMode::Creative {
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
        &mut queues,
        &mut fluid_queue,
        origin,
        rotated_voxels,
    );
}

#[allow(dead_code)]
pub fn is_block_submerged_or_adjacent_to_water(world: &impl VoxelAccess, origin: IVec3) -> bool {
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
    queues: &mut ChunkStreamingQueues,
    fluid_queue: &mut Option<ResMut<FluidUpdateQueue>>,
    block_origin: IVec3,
    new_voxels: [Voxel; 8],
) {
    let Some(current) = world.get_voxel(block_origin) else {
        return;
    };
    if current.is_unbreakable() {
        return;
    }

    let primary = new_voxels
        .iter()
        .copied()
        .find(|v| {
            !v.is_empty() && !v.is_water() && *v != Voxel::Occupied && *v != Voxel::WaterOccupied
        })
        .unwrap_or(current);

    if current != primary {
        world.set_voxel(block_origin, primary);
        modifications.record(block_origin, primary);

        if let Some(queue) = fluid_queue.as_deref_mut() {
            queue.enqueue_with_neighbors(block_origin);
        }

        sync_voxel_light(commands, world, block_origin, light_registry);

        for coordinate in affected_chunks(block_origin) {
            if world.get_chunk(coordinate).is_some() {
                queues.enqueue_priority_remesh(coordinate);
            }
        }
    }
}

pub fn get_block_voxels(world: &impl VoxelAccess, block_origin: IVec3) -> [Voxel; 8] {
    let v = world.get_voxel(block_origin).unwrap_or(Voxel::Air);
    [v; 8]
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

pub fn is_centered_layer(world: &impl VoxelAccess, world_voxel: IVec3) -> bool {
    let coords = centered_layer_coordinates(world_voxel);
    coords.iter().any(|&pos| {
        world.get_voxel(pos) == Some(Voxel::Occupied)
            || world.get_voxel(pos) == Some(Voxel::WaterOccupied)
    })
}

pub fn get_centered_layer_material(world: &impl VoxelAccess, world_voxel: IVec3) -> Option<Voxel> {
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
        7 => BlockShape::CornerStairInverted,
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

/// Represents the 8 sub-voxels of a logical block.
/// Bit i is set if sub-voxel i is filled with the primary material.
/// Centered column sub-voxels also track Occupied companion voxels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubVoxelMask {
    pub fill_mask: u8,
    pub occupied_mask: u8,
}

impl SubVoxelMask {
    pub const fn new(fill_mask: u8, occupied_mask: u8) -> Self {
        Self {
            fill_mask,
            occupied_mask,
        }
    }

    pub const fn rotate_90(self) -> Self {
        Self {
            fill_mask: rotate_mask_90(self.fill_mask),
            occupied_mask: rotate_mask_90(self.occupied_mask),
        }
    }

    pub fn to_voxels(self, material: Voxel) -> [Voxel; 8] {
        let mut result = [Voxel::Air; 8];
        let mut i = 0;
        while i < 8 {
            let bit = 1 << i;
            if (self.fill_mask & bit) != 0 {
                result[i] = material;
            } else if (self.occupied_mask & bit) != 0 {
                result[i] = Voxel::Occupied;
            }
            i += 1;
        }
        result
    }
}

pub const fn rotate_mask_90(mask: u8) -> u8 {
    let mut rot = 0u8;
    let map = [1, 3, 0, 2, 5, 7, 4, 6];
    let mut i = 0;
    while i < 8 {
        if (mask & (1 << i)) != 0 {
            rot |= 1 << map[i];
        }
        i += 1;
    }
    rot
}

const fn make_shape_rotations(fill_mask: u8, occupied_mask: u8) -> [SubVoxelMask; 4] {
    let m0 = SubVoxelMask::new(fill_mask, occupied_mask);
    let m1 = m0.rotate_90();
    let m2 = m1.rotate_90();
    let m3 = m2.rotate_90();
    [m0, m1, m2, m3]
}

/// Static Lookup Table for all 10 BlockShapes across 4 rotation orientations (0°, 90°, 180°, 270°).
pub const SHAPE_ROTATION_MASKS: [[SubVoxelMask; 4]; 10] = [
    // 0: Full
    make_shape_rotations(0xFF, 0x00),
    // 1: Stair
    make_shape_rotations(0xCF, 0x00),
    // 2: StairUpsideDown
    make_shape_rotations(0xFC, 0x00),
    // 3: CornerStair
    make_shape_rotations(0x8F, 0x00),
    // 4: CornerStairInverted
    make_shape_rotations(0xEF, 0x00),
    // 5: SlabBottom
    make_shape_rotations(0x0F, 0x00),
    // 6: SlabTop
    make_shape_rotations(0xF0, 0x00),
    // 7: VerticalSlab
    make_shape_rotations(0xCC, 0x00),
    // 8: Column
    make_shape_rotations(0x11, 0x00),
    // 9: CenteredColumn
    make_shape_rotations(0x11, 0xEE),
];

pub const ROTATION_PERMUTATION_Y: [usize; 8] = [2, 0, 3, 1, 6, 4, 7, 5];

#[inline]
pub fn generate_shape_voxels(shape: BlockShape, material: Voxel) -> [Voxel; 8] {
    SHAPE_ROTATION_MASKS[shape.index()][0].to_voxels(material)
}

#[inline]
#[allow(dead_code)]
pub fn generate_shape_voxels_rotated(
    shape: BlockShape,
    rotation: usize,
    material: Voxel,
) -> [Voxel; 8] {
    SHAPE_ROTATION_MASKS[shape.index()][rotation % 4].to_voxels(material)
}

#[inline]
pub fn rotate_block_90_y(voxels: &[Voxel; 8]) -> [Voxel; 8] {
    [
        voxels[ROTATION_PERMUTATION_Y[0]],
        voxels[ROTATION_PERMUTATION_Y[1]],
        voxels[ROTATION_PERMUTATION_Y[2]],
        voxels[ROTATION_PERMUTATION_Y[3]],
        voxels[ROTATION_PERMUTATION_Y[4]],
        voxels[ROTATION_PERMUTATION_Y[5]],
        voxels[ROTATION_PERMUTATION_Y[6]],
        voxels[ROTATION_PERMUTATION_Y[7]],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Chunk;

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
            count_solid(generate_shape_voxels(BlockShape::CornerStairInverted, mat)),
            7
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
            BlockShape::CornerStairInverted,
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

        let inv_corner = generate_shape_voxels(BlockShape::CornerStairInverted, Voxel::Stone);
        let icr1 = rotate_block_90_y(&inv_corner);
        let icr2 = rotate_block_90_y(&icr1);
        let icr3 = rotate_block_90_y(&icr2);
        let icr4 = rotate_block_90_y(&icr3);

        assert_eq!(inv_corner, icr4);
        assert_ne!(inv_corner, icr1);

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

        let inv_corner = generate_shape_voxels(BlockShape::CornerStairInverted, Voxel::Stone);
        assert_eq!(
            detect_current_shape(&inv_corner),
            BlockShape::CornerStairInverted
        );
    }

    #[test]
    fn submerged_block_detected_as_waterlogged() {
        let mut world = VoxelWorld::default();
        world.insert_chunk(IVec3::ZERO, Chunk::new());

        let origin = IVec3::new(2, 2, 2);
        assert!(!is_block_submerged_or_adjacent_to_water(&world, origin));

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

    #[test]
    fn test_shape_rotation_luts_match_rotations() {
        for shape in BlockShape::all() {
            let base = generate_shape_voxels(shape, Voxel::Stone);
            let mut rotated = base;
            for rot in 0..4 {
                let from_lut = generate_shape_voxels_rotated(shape, rot, Voxel::Stone);
                assert_eq!(
                    from_lut, rotated,
                    "Mismatch for shape {shape:?} at rotation {rot}"
                );
                rotated = rotate_block_90_y(&rotated);
            }
        }
    }

    #[test]
    fn test_subvoxel_mask_coverage() {
        assert_eq!(SHAPE_ROTATION_MASKS.len(), 10);
        for row in SHAPE_ROTATION_MASKS {
            assert_eq!(row.len(), 4);
        }
    }
}
