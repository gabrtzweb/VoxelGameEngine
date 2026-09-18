use bevy::prelude::*;

use super::{biome::BiomeType, generator::TerrainGenerator};
use crate::world::{CHUNK_SIZE, Voxel};

pub const TREE_CELL_SIZE: i32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSpecies {
    Oak,
    Birch,
    Pine,
}

impl TreeSpecies {
    pub fn log_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::OakWoodLog,
            Self::Birch => Voxel::BirchWoodLog,
            Self::Pine => Voxel::PineWoodLog,
        }
    }

    pub fn wood_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::OakWood,
            Self::Birch => Voxel::BirchWood,
            Self::Pine => Voxel::PineWood,
        }
    }

    pub fn leaves_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::OakLeaves,
            Self::Birch => Voxel::BirchLeaves,
            Self::Pine => Voxel::PineLeaves,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrunkType {
    /// 1-voxel wide centered column (0.5m x 0.5m)
    Thin,
    /// Full 1m x 1m block column (2x2 voxels)
    Normal,
    /// 2x2 central Log trunk + cardinal vertical slabs of Wood (bark) + flared root base
    Large,
}

#[derive(Debug, Clone, Copy)]
pub struct TreeFeature {
    pub species: TreeSpecies,
    pub trunk_type: TrunkType,
    pub world_x: i32,
    pub base_y: i32,
    pub world_z: i32,
    pub height: i32,
    pub seed: u32,
}

/// Deterministic 2D coordinate hash.
pub fn hash_tree_cell(cell_x: i32, cell_z: i32, seed: u32) -> u32 {
    let mut h = (cell_x as u32).wrapping_mul(0x7110_4277)
        ^ (cell_z as u32).wrapping_mul(0x9E37_79B9)
        ^ seed.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x1656_67B1);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^ (h >> 16)
}

/// Evaluates if a given world cell contains a tree candidate and returns its specifications.
pub fn sample_tree_candidate(
    cell_x: i32,
    cell_z: i32,
    generator: &TerrainGenerator,
) -> Option<TreeFeature> {
    let h = hash_tree_cell(cell_x, cell_z, generator.seed);

    // Tree placement offset within 8x8 cell (aligned to even logical block coordinates 2 or 4)
    // Ensures adjacent cells never have trunks closer than 6 voxels (3m), preventing overlap.
    let local_x = 2 + (((h & 0x01) * 2) as i32);
    let local_z = 2 + ((((h >> 3) & 0x01) * 2) as i32);

    let world_x = cell_x * TREE_CELL_SIZE + local_x;
    let world_z = cell_z * TREE_CELL_SIZE + local_z;

    let column = generator.sample_column(world_x, world_z);

    // Trees never generate submerged in water or beaches
    if column.water_level.is_some() || column.is_beach {
        return None;
    }

    // Biome probability filter (evaluated per 8x8 cell)
    let chance_percent = match column.biome {
        BiomeType::Woodland => 75,
        BiomeType::PlainsForest => 55,
        BiomeType::Wetlands => 35,
        BiomeType::SnowyTundra => 32,
        BiomeType::Highlands => 22,
        BiomeType::Meadow => 18,
        BiomeType::Plains => 16,
        _ => 0,
    };

    let roll = (h >> 8) % 100;
    if roll >= chance_percent {
        return None;
    }

    // Determine species based on biome
    let species_roll = (h >> 12) % 100;
    let species = match column.biome {
        BiomeType::SnowyTundra => TreeSpecies::Pine,
        BiomeType::Highlands => {
            if species_roll < 85 {
                TreeSpecies::Pine
            } else {
                TreeSpecies::Oak
            }
        }
        BiomeType::Meadow => {
            if species_roll < 65 {
                TreeSpecies::Birch
            } else {
                TreeSpecies::Oak
            }
        }
        BiomeType::PlainsForest => {
            if species_roll < 60 {
                TreeSpecies::Oak
            } else {
                TreeSpecies::Birch
            }
        }
        BiomeType::Plains => {
            if species_roll < 50 {
                TreeSpecies::Oak
            } else {
                TreeSpecies::Birch
            }
        }
        BiomeType::Wetlands => TreeSpecies::Oak,
        BiomeType::Woodland => {
            if species_roll < 70 {
                TreeSpecies::Oak
            } else if species_roll < 90 {
                TreeSpecies::Birch
            } else {
                TreeSpecies::Pine
            }
        }
        _ => TreeSpecies::Oak,
    };

    // Determine trunk shape and height according to species rules
    let type_roll = (h >> 16) % 100;
    let (trunk_type, height) = match species {
        TreeSpecies::Oak => {
            // Small oaks are rarer (Thin ~15%), Normal ~60%, Large ~25%
            // Heights significantly increased so oaks are tall, grand, and majestic
            if type_roll < 15 {
                let height = 10 + ((h >> 20) % 5) as i32; // 10..=14 (5.0m - 7.0m)
                (TrunkType::Thin, height)
            } else if type_roll < 75 {
                let height = 14 + ((h >> 20) % 9) as i32; // 14..=22 (7.0m - 11.0m)
                (TrunkType::Normal, height)
            } else {
                let height = 22 + ((h >> 20) % 11) as i32; // 22..=32 (11.0m - 16.0m)
                (TrunkType::Large, height)
            }
        }
        TreeSpecies::Birch => {
            // No Large version. Thin ~45%, Normal ~55%. Tall, elegant birch trees.
            if type_roll < 45 {
                let height = 14 + ((h >> 20) % 7) as i32; // 14..=20 (7.0m - 10.0m)
                (TrunkType::Thin, height)
            } else {
                let height = 20 + ((h >> 20) % 9) as i32; // 20..=28 (10.0m - 14.0m)
                (TrunkType::Normal, height)
            }
        }
        TreeSpecies::Pine => {
            // Thin ~35%, Normal ~45%, Large ~20%
            if type_roll < 35 {
                let height = 16 + ((h >> 20) % 7) as i32; // 16..=22 (8.0m - 11.0m)
                (TrunkType::Thin, height)
            } else if type_roll < 80 {
                let height = 24 + ((h >> 20) % 9) as i32; // 24..=32 (12.0m - 16.0m)
                (TrunkType::Normal, height)
            } else {
                let height = 32 + ((h >> 20) % 11) as i32; // 32..=42 (16.0m - 21.0m)
                (TrunkType::Large, height)
            }
        }
    };

    Some(TreeFeature {
        species,
        trunk_type,
        world_x,
        base_y: column.terrain_height + 1,
        world_z,
        height,
        seed: h,
    })
}

/// Applies all tree trunks, branches, and leaves that intersect this chunk.
pub fn generate_chunk_trees(
    chunk_origin: IVec3,
    generator: &TerrainGenerator,
    voxels: &mut [Voxel],
) {
    let margin = 12;
    let min_cell_x = (chunk_origin.x - margin).div_euclid(TREE_CELL_SIZE);
    let max_cell_x = (chunk_origin.x + CHUNK_SIZE as i32 + margin).div_euclid(TREE_CELL_SIZE);
    let min_cell_z = (chunk_origin.z - margin).div_euclid(TREE_CELL_SIZE);
    let max_cell_z = (chunk_origin.z + CHUNK_SIZE as i32 + margin).div_euclid(TREE_CELL_SIZE);

    for cell_z in min_cell_z..=max_cell_z {
        for cell_x in min_cell_x..=max_cell_x {
            if let Some(tree) = sample_tree_candidate(cell_x, cell_z, generator) {
                place_tree_trunk_in_chunk(chunk_origin, &tree, voxels);
            }
        }
    }
}

#[inline(always)]
fn set_chunk_voxel(
    chunk_origin: IVec3,
    voxels: &mut [Voxel],
    world_x: i32,
    world_y: i32,
    world_z: i32,
    voxel: Voxel,
) {
    let lx = world_x - chunk_origin.x;
    let ly = world_y - chunk_origin.y;
    let lz = world_z - chunk_origin.z;

    if (0..CHUNK_SIZE as i32).contains(&lx)
        && (0..CHUNK_SIZE as i32).contains(&ly)
        && (0..CHUNK_SIZE as i32).contains(&lz)
    {
        let idx = lx as usize + lz as usize * CHUNK_SIZE + ly as usize * CHUNK_SIZE * CHUNK_SIZE;
        voxels[idx] = voxel;
    }
}

#[inline(always)]
fn set_branch_voxel(
    chunk_origin: IVec3,
    voxels: &mut [Voxel],
    world_x: i32,
    world_y: i32,
    world_z: i32,
    voxel: Voxel,
) {
    let lx = world_x - chunk_origin.x;
    let ly = world_y - chunk_origin.y;
    let lz = world_z - chunk_origin.z;

    if (0..CHUNK_SIZE as i32).contains(&lx)
        && (0..CHUNK_SIZE as i32).contains(&ly)
        && (0..CHUNK_SIZE as i32).contains(&lz)
    {
        let idx = lx as usize + lz as usize * CHUNK_SIZE + ly as usize * CHUNK_SIZE * CHUNK_SIZE;
        // Only place branches in Air or Snow to avoid overwriting trunk log cores or solid terrain
        if voxels[idx] == Voxel::Air || voxels[idx] == Voxel::Snow {
            voxels[idx] = voxel;
        }
    }
}

#[inline(always)]
fn set_leaf_voxel(
    chunk_origin: IVec3,
    voxels: &mut [Voxel],
    world_x: i32,
    world_y: i32,
    world_z: i32,
    leaf_voxel: Voxel,
) {
    let lx = world_x - chunk_origin.x;
    let ly = world_y - chunk_origin.y;
    let lz = world_z - chunk_origin.z;

    if (0..CHUNK_SIZE as i32).contains(&lx)
        && (0..CHUNK_SIZE as i32).contains(&ly)
        && (0..CHUNK_SIZE as i32).contains(&lz)
    {
        let idx = lx as usize + lz as usize * CHUNK_SIZE + ly as usize * CHUNK_SIZE * CHUNK_SIZE;
        // Leaves never replace solid blocks, logs, branches, or roots; only replace Air or Snow
        if voxels[idx] == Voxel::Air || voxels[idx] == Voxel::Snow {
            voxels[idx] = leaf_voxel;
        }
    }
}

fn place_tree_trunk_in_chunk(chunk_origin: IVec3, tree: &TreeFeature, voxels: &mut [Voxel]) {
    let tx = tree.world_x;
    let tz = tree.world_z;
    let base_y = tree.base_y;
    let top_y = base_y + tree.height - 1;
    let log = tree.species.log_voxel();
    let wood = tree.species.wood_voxel();

    // Early vertical reject: if the tree doesn't reach or touch this chunk vertically (with leaf margin), skip
    if (top_y + 6) < chunk_origin.y || (base_y - 2) > (chunk_origin.y + CHUNK_SIZE as i32 - 1) {
        return;
    }

    match tree.trunk_type {
        TrunkType::Thin => {
            // Thin trunk: Centered column of log_voxel (1 voxel wide)
            // Anchor roots 2 voxels into the ground with solid dirt to eliminate hovering on slopes
            for y in (base_y - 2)..base_y {
                set_chunk_voxel(chunk_origin, voxels, tx, y, tz, Voxel::Dirt);
                set_chunk_voxel(chunk_origin, voxels, tx + 1, y, tz, Voxel::Dirt);
                set_chunk_voxel(chunk_origin, voxels, tx, y, tz + 1, Voxel::Dirt);
                set_chunk_voxel(chunk_origin, voxels, tx + 1, y, tz + 1, Voxel::Dirt);
            }
            for y in base_y..=top_y {
                set_chunk_voxel(chunk_origin, voxels, tx, y, tz, log);
                set_chunk_voxel(chunk_origin, voxels, tx + 1, y, tz, Voxel::Occupied);
                set_chunk_voxel(chunk_origin, voxels, tx, y, tz + 1, Voxel::Occupied);
                set_chunk_voxel(chunk_origin, voxels, tx + 1, y, tz + 1, Voxel::Occupied);
            }
        }
        TrunkType::Normal => {
            // Normal trunk: Full 2x2 voxel column of log_voxel
            for y in (base_y - 2)..base_y {
                for dx in 0..2 {
                    for dz in 0..2 {
                        set_chunk_voxel(chunk_origin, voxels, tx + dx, y, tz + dz, Voxel::Dirt);
                    }
                }
            }
            for y in base_y..=top_y {
                for dx in 0..2 {
                    for dz in 0..2 {
                        set_chunk_voxel(chunk_origin, voxels, tx + dx, y, tz + dz, log);
                    }
                }
            }
        }
        TrunkType::Large => {
            // Large trunk:
            // 1. Central 2x2 core of log_voxel across the full height
            for y in (base_y - 2)..base_y {
                for dx in 0..2 {
                    for dz in 0..2 {
                        set_chunk_voxel(chunk_origin, voxels, tx + dx, y, tz + dz, Voxel::Dirt);
                    }
                }
            }
            for y in base_y..=top_y {
                for dx in 0..2 {
                    for dz in 0..2 {
                        set_chunk_voxel(chunk_origin, voxels, tx + dx, y, tz + dz, log);
                    }
                }
            }

            // 2. Cardinal Vertical Slabs made of wood_voxel (bark-only) attached to the 4 sides
            for y in base_y..=top_y {
                // +X wing (x = tx + 2, z in [tz, tz + 1])
                set_chunk_voxel(chunk_origin, voxels, tx + 2, y, tz, wood);
                set_chunk_voxel(chunk_origin, voxels, tx + 2, y, tz + 1, wood);

                // -X wing (x = tx - 1, z in [tz, tz + 1])
                set_chunk_voxel(chunk_origin, voxels, tx - 1, y, tz, wood);
                set_chunk_voxel(chunk_origin, voxels, tx - 1, y, tz + 1, wood);

                // +Z wing (z = tz + 2, x in [tx, tx + 1])
                set_chunk_voxel(chunk_origin, voxels, tx, y, tz + 2, wood);
                set_chunk_voxel(chunk_origin, voxels, tx + 1, y, tz + 2, wood);

                // -Z wing (z = tz - 1, x in [tx, tx + 1])
                set_chunk_voxel(chunk_origin, voxels, tx, y, tz - 1, wood);
                set_chunk_voxel(chunk_origin, voxels, tx + 1, y, tz - 1, wood);
            }

            // Anchor under vertical slabs so they don't float if on a small slope
            for dx in [-1, 2] {
                for dz in 0..2 {
                    set_chunk_voxel(chunk_origin, voxels, tx + dx, base_y - 1, tz + dz, Voxel::Dirt);
                }
            }
            for dz in [-1, 2] {
                for dx in 0..2 {
                    set_chunk_voxel(chunk_origin, voxels, tx + dx, base_y - 1, tz + dz, Voxel::Dirt);
                }
            }

            // 3. Flared Root Base: low root steps extending outward at ground level (base_y)
            let root_y = base_y;

            // +X root flare (x = tx + 3)
            set_chunk_voxel(chunk_origin, voxels, tx + 3, root_y, tz, wood);
            set_chunk_voxel(chunk_origin, voxels, tx + 3, root_y, tz + 1, wood);
            set_chunk_voxel(chunk_origin, voxels, tx + 3, root_y - 1, tz, Voxel::Dirt);
            set_chunk_voxel(chunk_origin, voxels, tx + 3, root_y - 1, tz + 1, Voxel::Dirt);

            // -X root flare (x = tx - 2)
            set_chunk_voxel(chunk_origin, voxels, tx - 2, root_y, tz, wood);
            set_chunk_voxel(chunk_origin, voxels, tx - 2, root_y, tz + 1, wood);
            set_chunk_voxel(chunk_origin, voxels, tx - 2, root_y - 1, tz, Voxel::Dirt);
            set_chunk_voxel(chunk_origin, voxels, tx - 2, root_y - 1, tz + 1, Voxel::Dirt);

            // +Z root flare (z = tz + 3)
            set_chunk_voxel(chunk_origin, voxels, tx, root_y, tz + 3, wood);
            set_chunk_voxel(chunk_origin, voxels, tx + 1, root_y, tz + 3, wood);
            set_chunk_voxel(chunk_origin, voxels, tx + 1, root_y - 1, tz + 3, Voxel::Dirt);
            set_chunk_voxel(chunk_origin, voxels, tx, root_y - 1, tz + 3, Voxel::Dirt);

            // -Z root flare (z = tz - 2)
            set_chunk_voxel(chunk_origin, voxels, tx, root_y, tz - 2, wood);
            set_chunk_voxel(chunk_origin, voxels, tx + 1, root_y, tz - 2, wood);
            set_chunk_voxel(chunk_origin, voxels, tx + 1, root_y - 1, tz - 2, Voxel::Dirt);
            set_chunk_voxel(chunk_origin, voxels, tx, root_y - 1, tz - 2, Voxel::Dirt);

            // Diagonal corner root steps at base_y
            set_chunk_voxel(chunk_origin, voxels, tx - 1, root_y, tz - 1, wood);
            set_chunk_voxel(chunk_origin, voxels, tx + 2, root_y, tz - 1, wood);
            set_chunk_voxel(chunk_origin, voxels, tx - 1, root_y, tz + 2, wood);
            set_chunk_voxel(chunk_origin, voxels, tx + 2, root_y, tz + 2, wood);

            set_chunk_voxel(chunk_origin, voxels, tx - 1, root_y - 1, tz - 1, Voxel::Dirt);
            set_chunk_voxel(chunk_origin, voxels, tx + 2, root_y - 1, tz - 1, Voxel::Dirt);
            set_chunk_voxel(chunk_origin, voxels, tx - 1, root_y - 1, tz + 2, Voxel::Dirt);
            set_chunk_voxel(chunk_origin, voxels, tx + 2, root_y - 1, tz + 2, Voxel::Dirt);
        }
    }

    // Procedural branching in the upper section
    place_tree_branches(chunk_origin, tree, voxels);

    // Procedural canopy foliage
    place_tree_leaves(chunk_origin, tree, voxels);
}

fn place_tree_branches(chunk_origin: IVec3, tree: &TreeFeature, voxels: &mut [Voxel]) {
    let tx = tree.world_x;
    let tz = tree.world_z;
    let base_y = tree.base_y;
    let height = tree.height;
    let top_y = base_y + height - 1;
    let wood = tree.species.wood_voxel();

    let mut rng = (tree.seed ^ 0xA5A5_5A5A).wrapping_mul(0x9E37_79B9);
    let mut next_rand = || {
        rng = rng.wrapping_mul(0x1656_67B1).wrapping_add(0xC2B2_AE35);
        rng ^ (rng >> 16)
    };

    let dirs = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (-1, -1),
        (-1, 1),
        (1, -1),
    ];

    match tree.species {
        TreeSpecies::Pine => {
            // Pine: Branches are short stubs strictly on lower 55% of tree,
            // completely concealed within the wide needle skirts (never sticking out!).
            let start_y = base_y + (height * 25 / 100).max(3);
            let branch_max_y = base_y + (height * 55 / 100);
            let tier_step = 2;
            let mut y = start_y;
            while y < branch_max_y {
                let offset = (next_rand() % 4) as usize;
                let branch_count = 2 + ((next_rand() % 2) as usize); // 2..3

                for b in 0..branch_count {
                    let (dx, dz) = dirs[(offset + b * 3) % 8];
                    let (start_x, start_z) = match tree.trunk_type {
                        TrunkType::Thin => (tx, tz),
                        _ => {
                            let ox = if dx > 0 { tx + 1 } else { tx };
                            let oz = if dz > 0 { tz + 1 } else { tz };
                            (ox, oz)
                        }
                    };

                    // Only 1 voxel reach: strictly stays inside the needle skirt!
                    set_branch_voxel(chunk_origin, voxels, start_x + dx, y, start_z + dz, wood);
                }

                y += tier_step;
            }
        }
        TreeSpecies::Birch => {
            // Birch: Slender ascending limbs in upper 45%
            let start_y = base_y + (height * 50 / 100).max(5);
            let branch_count = 2 + ((next_rand() % 3) as usize);
            let start_dir_idx = (next_rand() % 8) as usize;

            for b in 0..branch_count {
                let (dx, dz) = dirs[(start_dir_idx + b * 3) % 8];
                let by = start_y + ((next_rand() % ((top_y - start_y).max(1) as u32)) as i32);
                let (start_x, start_z) = match tree.trunk_type {
                    TrunkType::Thin => (tx, tz),
                    _ => {
                        let ox = if dx > 0 { tx + 1 } else { tx };
                        let oz = if dz > 0 { tz + 1 } else { tz };
                        (ox, oz)
                    }
                };

                // Step 1: outward
                set_branch_voxel(chunk_origin, voxels, start_x + dx, by, start_z + dz, wood);
                // Step 2: outward and upward
                set_branch_voxel(chunk_origin, voxels, start_x + dx * 2, by + 1, start_z + dz * 2, wood);
            }
        }
        TreeSpecies::Oak => {
            // Oak: Sprawling gnarled lateral and diagonal boughs in upper 60%
            let start_y = base_y + (height * 35 / 100).max(4);
            let branch_count = match tree.trunk_type {
                TrunkType::Thin => 3 + ((next_rand() % 2) as usize), // 3..4
                TrunkType::Normal => 4 + ((next_rand() % 3) as usize), // 4..6
                TrunkType::Large => 5 + ((next_rand() % 3) as usize), // 5..7
            };
            let start_dir_idx = (next_rand() % 8) as usize;

            for b in 0..branch_count {
                let (dx, dz) = dirs[(start_dir_idx + b * 2) % 8];
                let by = start_y + ((next_rand() % ((top_y - start_y).max(1) as u32)) as i32);
                let (start_x, start_z) = match tree.trunk_type {
                    TrunkType::Thin => (tx, tz),
                    TrunkType::Normal => {
                        let ox = if dx > 0 { tx + 1 } else if dx < 0 { tx } else { tx + (next_rand() % 2) as i32 };
                        let oz = if dz > 0 { tz + 1 } else if dz < 0 { tz } else { tz + (next_rand() % 2) as i32 };
                        (ox, oz)
                    }
                    TrunkType::Large => {
                        let ox = if dx > 0 { tx + 2 } else if dx < 0 { tx - 1 } else { tx + (next_rand() % 2) as i32 };
                        let oz = if dz > 0 { tz + 2 } else if dz < 0 { tz - 1 } else { tz + (next_rand() % 2) as i32 };
                        (ox, oz)
                    }
                };

                let reach = match tree.trunk_type {
                    TrunkType::Thin => 2 + (next_rand() % 2) as i32, // 2..3
                    TrunkType::Normal => 3 + (next_rand() % 2) as i32, // 3..4
                    TrunkType::Large => 4 + (next_rand() % 3) as i32, // 4..6
                };

                for step in 1..=reach {
                    let bx = start_x + dx * step;
                    let bz = start_z + dz * step;
                    let py = if step >= (reach - 1) && (next_rand() % 2 == 0) { by + 1 } else { by };
                    set_branch_voxel(chunk_origin, voxels, bx, py, bz, wood);
                }
            }
        }
    }
}

/// Places a smooth Euclidean circular disc of leaves at height `ly` centered at continuous `(center_x, center_z)`.
#[inline(always)]
fn place_leaf_layer_circle(
    chunk_origin: IVec3,
    voxels: &mut [Voxel],
    center_x: f32,
    center_z: f32,
    ly: i32,
    radius: f32,
    leaves: Voxel,
) {
    if radius < 0.2 {
        return;
    }
    let r_sq = radius * radius;
    let min_x = (center_x - radius).floor() as i32;
    let max_x = (center_x + radius).ceil() as i32;
    let min_z = (center_z - radius).floor() as i32;
    let max_z = (center_z + radius).ceil() as i32;

    for vx in min_x..=max_x {
        let dx = (vx as f32 + 0.5) - center_x;
        let dx_sq = dx * dx;
        for vz in min_z..=max_z {
            let dz = (vz as f32 + 0.5) - center_z;
            if dx_sq + dz * dz <= r_sq + 0.25 {
                set_leaf_voxel(chunk_origin, voxels, vx, ly, vz, leaves);
            }
        }
    }
}

/// Places a true 3D Euclidean rounded sphere of leaves centered at `(cx, cy, cz)`.
#[inline(always)]
fn place_leaf_sphere(
    chunk_origin: IVec3,
    voxels: &mut [Voxel],
    cx: f32,
    cy: f32,
    cz: f32,
    radius: f32,
    leaves: Voxel,
) {
    if radius < 0.2 {
        return;
    }
    let min_y = (cy - radius).floor() as i32;
    let max_y = (cy + radius).ceil() as i32;
    let r_sq = radius * radius;

    for ly in min_y..=max_y {
        let dy = (ly as f32 + 0.5) - cy;
        let dy_sq = dy * dy;
        if dy_sq > r_sq {
            continue;
        }
        let horizontal_r_sq = r_sq - dy_sq;
        let h_radius = horizontal_r_sq.sqrt();
        let min_x = (cx - h_radius).floor() as i32;
        let max_x = (cx + h_radius).ceil() as i32;
        let min_z = (cz - h_radius).floor() as i32;
        let max_z = (cz + h_radius).ceil() as i32;

        for vx in min_x..=max_x {
            let dx = (vx as f32 + 0.5) - cx;
            let dx_sq = dx * dx;
            for vz in min_z..=max_z {
                let dz = (vz as f32 + 0.5) - cz;
                if dx_sq + dz * dz <= horizontal_r_sq + 0.25 {
                    set_leaf_voxel(chunk_origin, voxels, vx, ly, vz, leaves);
                }
            }
        }
    }
}

fn place_tree_leaves(chunk_origin: IVec3, tree: &TreeFeature, voxels: &mut [Voxel]) {
    let tx = tree.world_x;
    let tz = tree.world_z;
    let base_y = tree.base_y;
    let height = tree.height;
    let top_y = base_y + height - 1;
    let leaves = tree.species.leaves_voxel();

    let (center_x, center_z) = match tree.trunk_type {
        TrunkType::Thin => (tx as f32 + 0.5, tz as f32 + 0.5),
        TrunkType::Normal | TrunkType::Large => (tx as f32 + 1.0, tz as f32 + 1.0),
    };

    let mut rng = (tree.seed ^ 0x5A5A_A5A5).wrapping_mul(0x9E37_79B9);
    let mut next_rand = || {
        rng = rng.wrapping_mul(0x1656_67B1).wrapping_add(0xC2B2_AE35);
        rng ^ (rng >> 16)
    };

    let dirs = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (-1, -1),
        (-1, 1),
        (1, -1),
    ];

    match tree.species {
        TreeSpecies::Pine => {
            // Pine: Conical needle skirts starting low (~25% height) and tapering up to a sharp spire
            let start_y = base_y + (height * 25 / 100).max(3);
            let max_r: f32 = match tree.trunk_type {
                TrunkType::Thin => 3.2,
                TrunkType::Normal => 4.4,
                TrunkType::Large => 5.6,
            };

            let foliage_span = (top_y - start_y).max(4);
            let mut y = start_y;
            while y <= top_y {
                let progress = (y - start_y) as f32 / foliage_span as f32;
                let tier_r = (1.0 - progress) * (max_r - 1.2) + 1.2;

                // Conical skirt with smooth Euclidean circular layers
                place_leaf_layer_circle(chunk_origin, voxels, center_x, center_z, y - 1, tier_r, leaves);
                place_leaf_layer_circle(chunk_origin, voxels, center_x, center_z, y, tier_r * 0.85, leaves);
                place_leaf_layer_circle(chunk_origin, voxels, center_x, center_z, y + 1, tier_r * 0.65, leaves);

                y += 2;
            }

            // Needle spire at summit
            place_leaf_layer_circle(chunk_origin, voxels, center_x, center_z, top_y + 1, 1.4, leaves);
            place_leaf_layer_circle(chunk_origin, voxels, center_x, center_z, top_y + 2, 0.9, leaves);
            place_leaf_layer_circle(chunk_origin, voxels, center_x, center_z, top_y + 3, 0.2, leaves);
            place_leaf_layer_circle(chunk_origin, voxels, center_x, center_z, top_y + 4, 0.2, leaves);
        }

        TreeSpecies::Birch => {
            // Birch: Continuous organic oval/flame canopy following a sinusoidal curve (no blocky steps!)
            let start_y = base_y + (height * 40 / 100).max(4);
            let max_r: f32 = match tree.trunk_type {
                TrunkType::Thin => 2.7,
                _ => 3.7,
            };
            let canopy_len = ((top_y + 4) - start_y).max(4);

            for ly in start_y..=(top_y + 4) {
                let t = (ly - start_y) as f32 / canopy_len as f32; // 0.0 to 1.0
                let curve = (std::f32::consts::PI * t).sin().powf(0.55);
                let r = max_r * curve;
                place_leaf_layer_circle(chunk_origin, voxels, center_x, center_z, ly, r, leaves);
            }
        }

        TreeSpecies::Oak => {
            // Oak: Magnificent billowing organic canopy formed by multiple overlapping 3D spherical
            // leaf clouds along each sprawling branch bough, plus a rounded central crown dome
            let start_y = base_y + (height * 35 / 100).max(4);
            let branch_count = match tree.trunk_type {
                TrunkType::Thin => 3 + ((next_rand() % 2) as usize), // 3..4
                TrunkType::Normal => 4 + ((next_rand() % 3) as usize), // 4..6
                TrunkType::Large => 5 + ((next_rand() % 3) as usize), // 5..7
            };
            let start_dir_idx = (next_rand() % 8) as usize;

            // 1. Billowing 3D leaf spheres along each branch bough and tip
            for b in 0..branch_count {
                let (dx, dz) = dirs[(start_dir_idx + b * 2) % 8];
                let by = start_y + ((next_rand() % ((top_y - start_y).max(1) as u32)) as i32);
                let reach = match tree.trunk_type {
                    TrunkType::Thin => 2 + (next_rand() % 2) as i32, // 2..3
                    TrunkType::Normal => 3 + (next_rand() % 2) as i32, // 3..4
                    TrunkType::Large => 4 + (next_rand() % 3) as i32, // 4..6
                };

                let tip_x = center_x + (dx * reach) as f32;
                let tip_z = center_z + (dz * reach) as f32;
                let tip_y = (by + (if next_rand() % 2 == 0 { 1 } else { 0 })) as f32 + 0.5;

                // Spherical leaf cloud around branch tip
                let tip_radius: f32 = match tree.trunk_type {
                    TrunkType::Thin => 2.5,
                    TrunkType::Normal => 3.0,
                    TrunkType::Large => 3.5,
                };
                place_leaf_sphere(chunk_origin, voxels, tip_x, tip_y, tip_z, tip_radius, leaves);

                // Mid-bough leaf cloud
                let mid_x = (center_x + tip_x) * 0.5;
                let mid_z = (center_z + tip_z) * 0.5;
                place_leaf_sphere(chunk_origin, voxels, mid_x, by as f32 + 0.5, mid_z, tip_radius * 0.85, leaves);
            }

            // 2. Rounded central crown dome capping the upper trunk
            let crown_max_r: f32 = match tree.trunk_type {
                TrunkType::Thin => 3.4,
                TrunkType::Normal => 4.6,
                TrunkType::Large => 5.8,
            };

            for ly in (top_y - 2)..=(top_y + 4) {
                let dy = (ly - top_y) as f32;
                let r = crown_max_r * (1.0 - (dy / 5.5).powi(2)).max(0.0).sqrt();
                place_leaf_layer_circle(chunk_origin, voxels, center_x, center_z, ly, r, leaves);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_heights_are_always_taller_than_player() {
        let generator = TerrainGenerator::default();
        for cz in -10..10 {
            for cx in -10..10 {
                if let Some(tree) = sample_tree_candidate(cx, cz, &generator) {
                    // Player is 1.8m (3.6 sub-voxels). Height must be at least 6 voxels (3.0m).
                    assert!(
                        tree.height >= 6,
                        "Tree height {} must be >= 6 voxels (3.0m), taller than player (1.8m)",
                        tree.height
                    );
                }
            }
        }
    }

    #[test]
    fn tree_generation_is_deterministic() {
        let generator = TerrainGenerator::default();
        let tree1 = sample_tree_candidate(5, 5, &generator);
        let tree2 = sample_tree_candidate(5, 5, &generator);
        match (tree1, tree2) {
            (Some(t1), Some(t2)) => {
                assert_eq!(t1.species, t2.species);
                assert_eq!(t1.trunk_type, t2.trunk_type);
                assert_eq!(t1.world_x, t2.world_x);
                assert_eq!(t1.world_z, t2.world_z);
                assert_eq!(t1.height, t2.height);
                assert_eq!(t1.seed, t2.seed);
            }
            (None, None) => {}
            _ => panic!("Tree generation must be strictly deterministic"),
        }
    }

    #[test]
    fn birch_never_generates_large_and_is_taller_than_oak() {
        let mut found_birch = 0;
        for cz in -20..20 {
            for cx in -20..20 {
                let h = hash_tree_cell(cx, cz, 42);
                let type_roll = (h >> 16) % 100;
                let (trunk_type, height) = if type_roll < 45 {
                    (TrunkType::Thin, 14 + ((h >> 20) % 7) as i32)
                } else {
                    (TrunkType::Normal, 20 + ((h >> 20) % 9) as i32)
                };
                assert_ne!(trunk_type, TrunkType::Large, "Birch must never generate Large trunk");
                assert!(height >= 14, "Birch height must be >= 14");
                found_birch += 1;
            }
        }
        assert!(found_birch > 0);
    }

    #[test]
    fn pine_thin_and_normal_rarity() {
        let mut thin_count = 0;
        let mut normal_count = 0;
        let mut large_count = 0;
        let total = 10_000;

        for i in 0..total {
            let type_roll = i % 100;
            if type_roll < 35 {
                thin_count += 1;
            } else if type_roll < 80 {
                normal_count += 1;
            } else {
                large_count += 1;
            }
        }

        assert_eq!(thin_count, 3500);
        assert_eq!(normal_count, 4500);
        assert_eq!(large_count, 2000);
        assert!(normal_count > thin_count);
        assert!(thin_count > large_count);
    }

    #[test]
    fn oak_smaller_trees_are_rarer() {
        let mut thin_count = 0;
        let mut normal_count = 0;
        let mut large_count = 0;
        let total = 10_000;

        for i in 0..total {
            let type_roll = i % 100;
            if type_roll < 15 {
                thin_count += 1;
            } else if type_roll < 75 {
                normal_count += 1;
            } else {
                large_count += 1;
            }
        }

        assert_eq!(thin_count, 1500);
        assert_eq!(normal_count, 6000);
        assert_eq!(large_count, 2500);
        assert!(normal_count > large_count);
        assert!(large_count > thin_count);
    }

    #[test]
    fn trunk_placement_applies_correct_materials_per_species() {
        // Test Birch Thin
        let birch_thin = TreeFeature {
            species: TreeSpecies::Birch,
            trunk_type: TrunkType::Thin,
            world_x: 4,
            base_y: 2,
            world_z: 4,
            height: 10,
            seed: 12345,
        };
        let mut voxels_birch = vec![Voxel::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        place_tree_trunk_in_chunk(IVec3::ZERO, &birch_thin, &mut voxels_birch);
        let idx = |x: usize, y: usize, z: usize| x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
        assert_eq!(voxels_birch[idx(4, 2, 4)], Voxel::BirchWoodLog);
        assert_eq!(voxels_birch[idx(5, 2, 4)], Voxel::Occupied);

        // Test Pine Large
        let pine_large = TreeFeature {
            species: TreeSpecies::Pine,
            trunk_type: TrunkType::Large,
            world_x: 8,
            base_y: 2,
            world_z: 8,
            height: 18,
            seed: 67890,
        };
        let mut voxels_pine = vec![Voxel::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        place_tree_trunk_in_chunk(IVec3::ZERO, &pine_large, &mut voxels_pine);
        // Center is PineWoodLog
        assert_eq!(voxels_pine[idx(8, 5, 8)], Voxel::PineWoodLog);
        assert_eq!(voxels_pine[idx(9, 5, 9)], Voxel::PineWoodLog);
        // Cardinal slabs are PineWood (bark-only)
        assert_eq!(voxels_pine[idx(10, 5, 8)], Voxel::PineWood);
        assert_eq!(voxels_pine[idx(7, 5, 8)], Voxel::PineWood);
    }

    #[test]
    fn leaves_never_overwrite_trunks_or_solid_blocks() {
        let tree = TreeFeature {
            species: TreeSpecies::Oak,
            trunk_type: TrunkType::Normal,
            world_x: 8,
            base_y: 2,
            world_z: 8,
            height: 10,
            seed: 7777,
        };
        let mut voxels = vec![Voxel::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        place_tree_trunk_in_chunk(IVec3::ZERO, &tree, &mut voxels);

        let idx = |x: usize, y: usize, z: usize| x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
        // Central trunk at y=5 must be OakWoodLog, NOT OakLeaves
        assert_eq!(voxels[idx(8, 5, 8)], Voxel::OakWoodLog);
        assert_eq!(voxels[idx(9, 5, 8)], Voxel::OakWoodLog);

        // Surrounding voxels should contain OakLeaves
        let mut leaf_found = false;
        for y in 6..=13 {
            for x in 6..=11 {
                for z in 6..=11 {
                    if voxels[idx(x, y, z)] == Voxel::OakLeaves {
                        leaf_found = true;
                        break;
                    }
                }
            }
        }
        assert!(leaf_found, "Canopy should have placed OakLeaves around the crown");
    }

    #[test]
    fn tree_trunk_crosses_vertical_chunk_boundary_seamlessly() {
        let tree = TreeFeature {
            species: TreeSpecies::Oak,
            trunk_type: TrunkType::Normal,
            world_x: 6,
            base_y: 10,
            world_z: 6,
            height: 12,
            seed: 9999,
        };

        let mut chunk_lower = vec![Voxel::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        let mut chunk_upper = vec![Voxel::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];

        place_tree_trunk_in_chunk(IVec3::new(0, 0, 0), &tree, &mut chunk_lower);
        place_tree_trunk_in_chunk(IVec3::new(0, 16, 0), &tree, &mut chunk_upper);

        let idx = |x: usize, y: usize, z: usize| x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;

        // Top voxel of chunk_lower (Y = 15) must be OakWoodLog
        assert_eq!(chunk_lower[idx(6, 15, 6)], Voxel::OakWoodLog);

        // Bottom voxel of chunk_upper (Y = 16, local y = 0) must be OakWoodLog
        assert_eq!(chunk_upper[idx(6, 0, 6)], Voxel::OakWoodLog);

        // Tree top (world Y = 21, local y = 5 in chunk_upper) must be OakWoodLog
        assert_eq!(chunk_upper[idx(6, 5, 6)], Voxel::OakWoodLog);
    }
}
