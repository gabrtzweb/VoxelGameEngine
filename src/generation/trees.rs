//! Phase 20.5: Procedural Multi-Chunk Tree Generation & Arboreal Crops.
//!
//! Uses a deterministic 2D cellular grid evaluated across neighboring cells so tree trunks,
//! sculpted bark stair/column branches, organic multi-cluster canopies, hanging coconuts, and
//! lateral cocoa pods remain 100% seamless across chunk borders.

use bevy::prelude::IVec3;

use crate::{
    generation::{biome::BiomeType, generator::TerrainColumn},
    world::{BlockShape, CHUNK_SIZE, Voxel},
};

/// Identifies trunk and structural wood blocks.
#[inline]
pub fn is_trunk(voxel: Voxel) -> bool {
    voxel.is_wood_block()
}

/// Identifies leaf voxels.
#[inline]
pub fn is_leaves(voxel: Voxel) -> bool {
    voxel.is_leaves()
}

/// Fast, deterministic hash function for 2D cellular grid sampling.
#[inline]
pub fn hash_tree_cell(cell_x: i32, cell_z: i32, seed: u32) -> u32 {
    let mut h = seed
        .wrapping_add((cell_x as u32).wrapping_mul(0x85ebca6b))
        .wrapping_add((cell_z as u32).wrapping_mul(0xc2b2ae35));
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

#[inline(always)]
fn hash_f32(h: u32) -> f32 {
    (h & 0x00FF_FFFF) as f32 / 16_777_216.0
}

#[inline(always)]
fn chunk_idx(x: usize, y: usize, z: usize) -> usize {
    x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
}

#[derive(Clone, Copy)]
enum TreeArchetype {
    Oak,
    Birch,
    Maple(Voxel),
    Cherry,
    Pine,
    Yew,
    Redwood,
    Acacia,
    Palm,
    Mahogany,
    Willow,
    Mangrove,
    Charred,
}

/// Determines tree spawn probability and archetype for a given biome/column.
fn select_tree_for_column(col: TerrainColumn, ground: Voxel, h: u32) -> Option<(f32, TreeArchetype)> {
    // Gravel Beach: sparse coastal palms only (no pines on beaches!)
    if col.biome == BiomeType::GravelBeach {
        if col.climate.temperature > -0.25 {
            let on_dirty_gravel = matches!(ground, Voxel::GravelDirty | Voxel::GravelAshy);
            let chance = if on_dirty_gravel { 0.16 } else { 0.08 };
            return Some((chance, TreeArchetype::Palm));
        }
        return None;
    }

    // Coastal beaches & Mirage Isles: evenly distributed Palms across warm & temperate coastlines
    if col.is_beach || col.biome == BiomeType::MirageIsles {
        if col.climate.temperature > -0.25 {
            return Some((0.16, TreeArchetype::Palm));
        }
        return None;
    }

    match col.biome {
        BiomeType::YosemiteLowlands | BiomeType::Shield => {
            let arch = match (h >> 8) % 5 {
                0 | 1 => TreeArchetype::Oak,
                2 => TreeArchetype::Birch,
                3 => TreeArchetype::Pine,
                _ => TreeArchetype::Maple(match (h >> 12) % 3 {
                    0 => Voxel::MapleLeavesOrange,
                    1 => Voxel::MapleLeavesRed,
                    _ => Voxel::MapleLeavesYellow,
                }),
            };
            Some((0.64, arch))
        }
        BiomeType::CloudForest => {
            let arch = match (h >> 8) % 5 {
                0 => TreeArchetype::Redwood,
                1 | 2 => TreeArchetype::Oak,
                3 => TreeArchetype::Maple(Voxel::MapleLeavesOrange),
                _ => TreeArchetype::Birch,
            };
            Some((0.76, arch))
        }
        BiomeType::BloomingValley => {
            let arch = match (h >> 8) % 4 {
                0 | 1 => TreeArchetype::Birch,
                2 => TreeArchetype::Oak,
                _ => TreeArchetype::Maple(Voxel::MapleLeavesYellow),
            };
            Some((0.44, arch))
        }
        BiomeType::LavenderValley => {
            let arch = if ((h >> 8) & 1) == 0 {
                TreeArchetype::Birch
            } else {
                TreeArchetype::Oak
            };
            Some((0.36, arch))
        }
        BiomeType::SakuraGrove => Some((0.68, TreeArchetype::Cherry)),
        BiomeType::PineHighlands | BiomeType::SiberianTaiga | BiomeType::WintryForest => {
            let arch = match (h >> 8) % 6 {
                0 => TreeArchetype::Redwood,
                1 => TreeArchetype::Yew,
                _ => TreeArchetype::Pine,
            };
            Some((0.72, arch))
        }
        BiomeType::ColdShrubland | BiomeType::AlpineHighlands | BiomeType::EmeraldPeaks => {
            Some((0.34, TreeArchetype::Pine))
        }
        BiomeType::FracturedSavanna | BiomeType::SavannaBadlands => {
            Some((0.36, TreeArchetype::Acacia))
        }
        BiomeType::DesertOasis => Some((0.62, TreeArchetype::Palm)),
        BiomeType::TropicalJungle => {
            let arch = if ((h >> 8) % 5) == 0 {
                TreeArchetype::Palm
            } else {
                TreeArchetype::Mahogany
            };
            Some((0.82, arch))
        }
        BiomeType::OrchidSwamp => {
            let arch = if ((h >> 8) & 1) == 0 {
                TreeArchetype::Willow
            } else {
                TreeArchetype::Mangrove
            };
            Some((0.62, arch))
        }
        BiomeType::AshenSavanna | BiomeType::VolcanicPeaks => Some((0.24, TreeArchetype::Charred)),
        _ => None,
    }
}

struct ChunkWriter<'a> {
    voxels: &'a mut [Voxel],
    chunk_shapes: &'a mut Vec<(usize, BlockShape, u8)>,
    origin: IVec3,
    wrote_any: bool,
}

impl<'a> ChunkWriter<'a> {
    #[inline(always)]
    fn set_log_shaped(
        &mut self,
        wx: i32,
        wy: i32,
        wz: i32,
        log: Voxel,
        shape: BlockShape,
        orient: u8,
    ) {
        let lx = wx - self.origin.x;
        let ly = wy - self.origin.y;
        let lz = wz - self.origin.z;
        if !(0..CHUNK_SIZE as i32).contains(&lx)
            || !(0..CHUNK_SIZE as i32).contains(&ly)
            || !(0..CHUNK_SIZE as i32).contains(&lz)
        {
            return;
        }
        let idx = chunk_idx(lx as usize, ly as usize, lz as usize);
        let cur = self.voxels[idx];
        if cur.is_empty() || cur.is_leaves() || cur.is_flora_billboard() || cur.is_crop_billboard()
        {
            self.voxels[idx] = log;
            if shape != BlockShape::Full || orient != 0 {
                self.chunk_shapes.push((idx, shape, orient));
            }
            self.wrote_any = true;
        }
    }

    #[inline(always)]
    fn set_log(&mut self, wx: i32, wy: i32, wz: i32, log: Voxel) {
        self.set_log_shaped(wx, wy, wz, log, BlockShape::Full, 0);
    }

    #[inline(always)]
    fn set_leaf_shaped(
        &mut self,
        wx: i32,
        wy: i32,
        wz: i32,
        leaf: Voxel,
        shape: BlockShape,
        orient: u8,
    ) {
        let lx = wx - self.origin.x;
        let ly = wy - self.origin.y;
        let lz = wz - self.origin.z;
        if !(0..CHUNK_SIZE as i32).contains(&lx)
            || !(0..CHUNK_SIZE as i32).contains(&ly)
            || !(0..CHUNK_SIZE as i32).contains(&lz)
        {
            return;
        }
        let idx = chunk_idx(lx as usize, ly as usize, lz as usize);
        if self.voxels[idx].is_empty() {
            self.voxels[idx] = leaf;
            if shape != BlockShape::Full || orient != 0 {
                self.chunk_shapes.push((idx, shape, orient));
            }
            self.wrote_any = true;
        }
    }

    #[inline(always)]
    fn set_leaf(&mut self, wx: i32, wy: i32, wz: i32, leaf: Voxel) {
        self.set_leaf_shaped(wx, wy, wz, leaf, BlockShape::Full, 0);
    }

    #[inline(always)]
    fn set_shaped(
        &mut self,
        wx: i32,
        wy: i32,
        wz: i32,
        voxel: Voxel,
        shape: BlockShape,
        orient: u8,
    ) {
        let lx = wx - self.origin.x;
        let ly = wy - self.origin.y;
        let lz = wz - self.origin.z;
        if !(0..CHUNK_SIZE as i32).contains(&lx)
            || !(0..CHUNK_SIZE as i32).contains(&ly)
            || !(0..CHUNK_SIZE as i32).contains(&lz)
        {
            return;
        }
        let idx = chunk_idx(lx as usize, ly as usize, lz as usize);
        if self.voxels[idx].is_empty() {
            self.voxels[idx] = voxel;
            self.chunk_shapes.push((idx, shape, orient));
            self.wrote_any = true;
        }
    }

    /// Replaces existing solid soil/grass/dirt in-chunk (used for underground trunk anchoring
    /// and regional Rooted Dirt patches beneath and around tree roots).
    #[inline(always)]
    fn replace_ground(&mut self, wx: i32, wy: i32, wz: i32, replacement: Voxel) {
        let lx = wx - self.origin.x;
        let ly = wy - self.origin.y;
        let lz = wz - self.origin.z;
        if !(0..CHUNK_SIZE as i32).contains(&lx)
            || !(0..CHUNK_SIZE as i32).contains(&ly)
            || !(0..CHUNK_SIZE as i32).contains(&lz)
        {
            return;
        }
        let idx = chunk_idx(lx as usize, ly as usize, lz as usize);
        let cur = self.voxels[idx];
        if cur.is_solid_opaque() && !cur.is_wood_block() {
            self.voxels[idx] = replacement;
            self.wrote_any = true;
        }
    }
}

/// Resolves the regional `(rooted, coarse, mulch)` soil tuple matching the local surface soil family.
/// Returns `None` on pure beach sands or bare stone so beaches stay clean.
fn soil_collar_variants_for(ground: Voxel) -> Option<(Voxel, Voxel, Voxel)> {
    match ground {
        Voxel::PeatGrass
        | Voxel::PeatDirt
        | Voxel::PeatCoarse
        | Voxel::PeatMud
        | Voxel::PeatMulch
        | Voxel::PeatSnow
        | Voxel::PeatRooted => Some((Voxel::PeatRooted, Voxel::PeatCoarse, Voxel::PeatMulch)),
        Voxel::SiltGrass
        | Voxel::SiltDirt
        | Voxel::SiltCoarse
        | Voxel::SiltMud
        | Voxel::SiltMulch
        | Voxel::SiltSnow
        | Voxel::SiltRooted => Some((Voxel::SiltRooted, Voxel::SiltCoarse, Voxel::SiltMulch)),
        _ if ground.is_fertile_soil() => Some((Voxel::LoamRooted, Voxel::LoamCoarse, Voxel::LoamMulch)),
        _ => None,
    }
}

/// Anchors the tree trunk 1-2 blocks deep into the ground, places regional Rooted Dirt
/// underneath the trunk in the subsoil, and blends subtle Coarse/Mulch patches around the base.
fn stamp_anchored_root_collar(
    w: &mut ChunkWriter,
    rx: i32,
    ground_y: i32,
    rz: i32,
    bark: Voxel,
    ground: Voxel,
    h: u32,
) {
    // Sink the trunk core 1..2 blocks into the ground so trees feel deeply planted
    let depth = 1 + ((h >> 3) & 1) as i32;
    for d in 0..depth {
        w.replace_ground(rx, ground_y - d, rz, bark);
    }

    // Keep Rooted Dirt mainly underground beneath the trunk, and lightly blend Coarse & Mulch on the surface
    if let Some((rooted, coarse, mulch)) = soil_collar_variants_for(ground) {
        w.replace_ground(rx, ground_y - depth, rz, rooted);
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            w.replace_ground(rx + dx, ground_y - 1, rz + dz, rooted);
        }
        for dz in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let d2 = dx * dx + dz * dz;
                let rh = hash_tree_cell(rx + dx * 5, rz + dz * 7, h ^ 0x7007);
                let prob = if d2 == 1 { 0.34 } else { 0.16 };
                if hash_f32(rh) < prob {
                    let surface_soil = match (rh >> 8) % 10 {
                        0..=4 => coarse,
                        5..=8 => mulch,
                        _ => rooted,
                    };
                    w.replace_ground(rx + dx, ground_y, rz + dz, surface_soil);
                }
            }
        }
    }
}

/// Returns `(upright_stair_orient, inverted_stair_orient)` for a slender facing-stair bend
/// shifting by `(dx, dz)` on the same Y level (upright on old col, inverted facing it on new col).
#[inline]
fn curve_stair_orientations(dx: i32, dz: i32) -> (u8, u8) {
    match (dx, dz) {
        (1, 0) => (0, 5),  // Upright +X on old col, Inverted -X on new col
        (-1, 0) => (1, 4), // Upright -X on old col, Inverted +X on new col
        (0, 1) => (2, 7),  // Upright +Z on old col, Inverted -Z on new col
        _ => (3, 6),       // Upright -Z on old col, Inverted +Z on new col
    }
}

/// Stamps natural all-bark buttress roots around the base of a mature trunk using Stairs & Slabs.
fn stamp_buttress_roots(w: &mut ChunkWriter, rx: i32, ry: i32, rz: i32, bark: Voxel, h: u32) {
    const ROOTS: [(i32, i32, u8); 4] = [(-1, 0, 0), (1, 0, 1), (0, -1, 2), (0, 1, 3)];
    for (i, &(dx, dz, orient)) in ROOTS.iter().enumerate() {
        let r = (h >> (6 + i * 2)) & 3;
        if r == 0 || r == 1 {
            w.set_log_shaped(rx + dx, ry, rz + dz, bark, BlockShape::Stair, orient);
        } else if r == 2 {
            w.set_log_shaped(rx + dx, ry, rz + dz, bark, BlockShape::Slab, 0);
        }
    }
}

/// Stamps a horizontal centered-column branch extending `len` blocks along `(dx, dz)` using all-bark wood.
/// Orientation 5 = Centered Horizontal (East-West / X), 6 = Centered Horizontal (North-South / Z).
fn stamp_horizontal_branch(
    w: &mut ChunkWriter,
    sx: i32,
    sy: i32,
    sz: i32,
    dx: i32,
    dz: i32,
    len: i32,
    bark: Voxel,
) {
    let orient = if dx != 0 { 5 } else { 6 };
    for step in 1..=len {
        w.set_log_shaped(
            sx + dx * step,
            sy,
            sz + dz * step,
            bark,
            BlockShape::Column,
            orient,
        );
    }
}

/// Stamps an organic, non-blocky ellipsoidal leaf cloud with deterministic edge feathering.
fn stamp_leaf_cloud(
    w: &mut ChunkWriter,
    cx: i32,
    cy: i32,
    cz: i32,
    rx_f: f32,
    ry_f: f32,
    rz_f: f32,
    leaves: Voxel,
    h: u32,
) {
    let min_x = -(rx_f.ceil() as i32);
    let max_x = rx_f.ceil() as i32;
    let min_y = -(ry_f.ceil() as i32);
    let max_y = ry_f.ceil() as i32;
    let min_z = -(rz_f.ceil() as i32);
    let max_z = rz_f.ceil() as i32;

    for dy in min_y..=max_y {
        let ny = dy as f32 / ry_f;
        for dz in min_z..=max_z {
            let nz = dz as f32 / rz_f;
            for dx in min_x..=max_x {
                let nx = dx as f32 / rx_f;
                let dist_sq = nx * nx + ny * ny + nz * nz;
                if dist_sq > 1.05 {
                    continue;
                }
                if dist_sq > 0.72 {
                    let cell_h = hash_tree_cell(cx + dx * 3 + dy * 7, cz + dz * 5 - dy * 11, h);
                    let keep_prob = 1.0 - (dist_sq - 0.72) / 0.36;
                    if hash_f32(cell_h) > keep_prob {
                        continue;
                    }
                }
                w.set_leaf(cx + dx, cy + dy, cz + dz, leaves);
            }
        }
    }
}

/// Populates procedural multi-chunk trees and arboreal crops (cocoa pods & hanging coconuts)
/// into the given chunk. Returns `true` if any tree voxel was written inside this chunk.
pub fn populate_chunk_trees<F, G>(
    voxels: &mut [Voxel],
    chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
    chunk_origin: IVec3,
    seed: u32,
    column_at: F,
    ground_at: G,
) -> bool
where
    F: Fn(i32, i32) -> TerrainColumn,
    G: Fn(i32, i32, i32, TerrainColumn) -> Option<Voxel>,
{
    const CELL_SIZE: i32 = 7;
    const MAX_CANOPY_RADIUS: i32 = 8;

    let min_cell_x = (chunk_origin.x - MAX_CANOPY_RADIUS).div_euclid(CELL_SIZE);
    let max_cell_x =
        (chunk_origin.x + CHUNK_SIZE as i32 - 1 + MAX_CANOPY_RADIUS).div_euclid(CELL_SIZE);
    let min_cell_z = (chunk_origin.z - MAX_CANOPY_RADIUS).div_euclid(CELL_SIZE);
    let max_cell_z =
        (chunk_origin.z + CHUNK_SIZE as i32 - 1 + MAX_CANOPY_RADIUS).div_euclid(CELL_SIZE);

    let chunk_min_y = chunk_origin.y;
    let chunk_max_y = chunk_origin.y + CHUNK_SIZE as i32 - 1;

    let mut writer = ChunkWriter {
        voxels,
        chunk_shapes,
        origin: chunk_origin,
        wrote_any: false,
    };

    for cell_z in min_cell_z..=max_cell_z {
        for cell_x in min_cell_x..=max_cell_x {
            let h = hash_tree_cell(cell_x, cell_z, seed ^ 0x77EE_1234);
            let jx = 1 + ((h & 15) as i32 % (CELL_SIZE - 2));
            let jz = 1 + (((h >> 4) & 15) as i32 % (CELL_SIZE - 2));
            let root_x = cell_x * CELL_SIZE + jx;
            let root_z = cell_z * CELL_SIZE + jz;

            let col = column_at(root_x, root_z);
            let root_y = col.terrain_height;

            // Vertical culling with generous clearance for tall Redwoods & Pines (and 2-block root depth)
            if chunk_min_y > root_y + 24 || chunk_max_y < root_y - 3 {
                continue;
            }

            if col.is_cliff
                || col.is_river
                || col.surface_shape != BlockShape::Full
                || col.water_level.is_some_and(|wl| root_y <= wl)
            {
                continue;
            }

            // Reject hollow cave entrances & non-solid ground so trees never float
            let Some(ground) = ground_at(root_x, root_y, root_z, col) else {
                continue;
            };

            let Some((chance, archetype)) = select_tree_for_column(col, ground, h) else {
                continue;
            };

            if hash_f32(hash_tree_cell(cell_x, cell_z, seed ^ 0x99AB_CDEF)) >= chance {
                continue;
            }

            let root_bark = archetype_bark(archetype);
            stamp_anchored_root_collar(&mut writer, root_x, root_y, root_z, root_bark, ground, h);
            stamp_tree(&mut writer, root_x, root_y + 1, root_z, archetype, h);
        }
    }

    writer.wrote_any
}

fn archetype_bark(archetype: TreeArchetype) -> Voxel {
    match archetype {
        TreeArchetype::Oak => Voxel::OakWood,
        TreeArchetype::Birch => Voxel::BirchWood,
        TreeArchetype::Maple(_) => Voxel::MapleWood,
        TreeArchetype::Cherry => Voxel::CherryWood,
        TreeArchetype::Pine => Voxel::PineWood,
        TreeArchetype::Yew => Voxel::YewWood,
        TreeArchetype::Redwood => Voxel::RedwoodWood,
        TreeArchetype::Acacia => Voxel::AcaciaWood,
        TreeArchetype::Palm => Voxel::PalmWood,
        TreeArchetype::Mahogany => Voxel::MahoganyWood,
        TreeArchetype::Willow => Voxel::WillowWood,
        TreeArchetype::Mangrove => Voxel::MangroveWood,
        TreeArchetype::Charred => Voxel::TreeCharredBark,
    }
}

fn stamp_tree(
    w: &mut ChunkWriter,
    rx: i32,
    ry: i32,
    rz: i32,
    archetype: TreeArchetype,
    h: u32,
) {
    // ~35% of eligible trees spawn as slender young variants using Centered Vertical Column bark trunks
    let is_slender = ((h >> 28) % 3) == 0;

    match archetype {
        TreeArchetype::Oak => {
            if is_slender {
                let height = 6 + ((h >> 16) % 2) as i32;
                for dy in 0..height {
                    w.set_log_shaped(rx, ry + dy, rz, Voxel::OakWood, BlockShape::Column, 0);
                }
                let bx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
                stamp_horizontal_branch(w, rx, ry + height - 2, rz, bx, 0, 1, Voxel::OakWood);
                stamp_leaf_cloud(w, rx, ry + height, rz, 2.5, 2.2, 2.5, Voxel::OakLeaves, h);
                stamp_leaf_cloud(
                    w,
                    rx + bx,
                    ry + height - 1,
                    rz,
                    2.0,
                    1.6,
                    2.2,
                    Voxel::OakLeaves,
                    h ^ 0x123,
                );
            } else {
                let height = 7 + ((h >> 16) % 3) as i32;
                stamp_buttress_roots(w, rx, ry, rz, Voxel::OakWood, h);
                for dy in 0..height {
                    w.set_log(rx, ry + dy, rz, Voxel::OakLog);
                }
                // Horizontal centered-column branches (both X and Z axes!) using all-bark OakWood
                let bx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
                let bz = if ((h >> 11) & 1) == 0 { 1 } else { -1 };
                stamp_horizontal_branch(w, rx, ry + height - 2, rz, bx, 0, 2, Voxel::OakWood);
                stamp_horizontal_branch(w, rx, ry + height - 3, rz, 0, bz, 2, Voxel::OakWood);
                stamp_leaf_cloud(w, rx, ry + height + 1, rz, 3.2, 2.3, 3.2, Voxel::OakLeaves, h);
                stamp_leaf_cloud(
                    w,
                    rx + bx * 2,
                    ry + height - 1,
                    rz,
                    2.6,
                    1.8,
                    2.5,
                    Voxel::OakLeaves,
                    h ^ 0x331,
                );
                stamp_leaf_cloud(
                    w,
                    rx,
                    ry + height - 2,
                    rz + bz * 2,
                    2.5,
                    1.8,
                    2.6,
                    Voxel::OakLeaves,
                    h ^ 0x779,
                );
            }
        }
        TreeArchetype::Birch => {
            let height = 7 + ((h >> 16) % 4) as i32;
            let (vox, trunk_shape) = if is_slender {
                (Voxel::BirchWood, BlockShape::Column)
            } else {
                (Voxel::BirchLog, BlockShape::Full)
            };
            for dy in 0..height {
                w.set_log_shaped(rx, ry + dy, rz, vox, trunk_shape, 0);
            }
            let bz = if ((h >> 11) & 1) == 0 { 1 } else { -1 };
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, 0, bz, 1, Voxel::BirchWood);
            stamp_leaf_cloud(w, rx, ry + height - 1, rz, 2.6, 2.4, 2.6, Voxel::BirchLeaves, h);
            stamp_leaf_cloud(
                w,
                rx,
                ry + height + 1,
                rz,
                1.8,
                1.8,
                1.8,
                Voxel::BirchLeaves,
                h ^ 0x412,
            );
        }
        TreeArchetype::Maple(leaves) => {
            let height = 7 + ((h >> 16) % 3) as i32;
            if !is_slender {
                stamp_buttress_roots(w, rx, ry, rz, Voxel::MapleWood, h);
            }
            let (vox, trunk_shape) = if is_slender {
                (Voxel::MapleWood, BlockShape::Column)
            } else {
                (Voxel::MapleLog, BlockShape::Full)
            };
            for dy in 0..height {
                w.set_log_shaped(rx, ry + dy, rz, vox, trunk_shape, 0);
            }
            let dx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
            let dz = if ((h >> 11) & 1) == 0 { 1 } else { -1 };
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, dx, 0, 2, Voxel::MapleWood);
            stamp_horizontal_branch(w, rx, ry + height - 3, rz, 0, -dz, 2, Voxel::MapleWood);
            stamp_leaf_cloud(w, rx, ry + height + 1, rz, 3.0, 2.2, 3.0, leaves, h);
            stamp_leaf_cloud(w, rx + dx * 2, ry + height - 1, rz + dz, 2.5, 1.7, 2.5, leaves, h ^ 0x222);
            stamp_leaf_cloud(w, rx - dx, ry + height - 1, rz - dz * 2, 2.4, 1.7, 2.4, leaves, h ^ 0x555);
        }
        TreeArchetype::Cherry => {
            let height = 6 + ((h >> 16) % 3) as i32;
            let lean_x = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
            let lean_z = if ((h >> 11) & 1) == 0 { 1 } else { -1 };
            if !is_slender {
                stamp_buttress_roots(w, rx, ry, rz, Voxel::CherryWood, h);
            }
            let (up_st, inv_st) = curve_stair_orientations(lean_x, 0);
            let mut tx = rx;
            for dy in 0..height {
                if dy == 3 {
                    // Slender facing-stair curve using all-bark CherryWood
                    w.set_log_shaped(tx, ry + dy, rz, Voxel::CherryWood, BlockShape::Stair, up_st);
                    tx += lean_x;
                    w.set_log_shaped(tx, ry + dy, rz, Voxel::CherryWood, BlockShape::Stair, inv_st);
                } else {
                    w.set_log(tx, ry + dy, rz, Voxel::CherryLog);
                }
            }
            // Horizontal spreading cherry boughs
            stamp_horizontal_branch(w, tx, ry + height - 1, rz, -lean_x, 0, 2, Voxel::CherryWood);
            stamp_horizontal_branch(w, tx, ry + height - 2, rz, 0, lean_z, 2, Voxel::CherryWood);
            stamp_leaf_cloud(w, tx + lean_x, ry + height + 1, rz, 3.4, 1.9, 3.2, Voxel::CherryLeaves, h);
            stamp_leaf_cloud(
                w,
                tx - lean_x * 2,
                ry + height,
                rz + lean_z * 2,
                2.8,
                1.7,
                2.8,
                Voxel::CherryLeaves,
                h ^ 0x919,
            );
        }
        TreeArchetype::Pine => {
            let height = 10 + ((h >> 16) % 5) as i32;
            let (vox, trunk_shape) = if is_slender {
                (Voxel::PineWood, BlockShape::Column)
            } else {
                stamp_buttress_roots(w, rx, ry, rz, Voxel::PineWood, h);
                (Voxel::PineLog, BlockShape::Full)
            };
            stamp_conifer(w, rx, ry, rz, height, 3, trunk_shape, vox, Voxel::PineLeaves, h);
        }
        TreeArchetype::Yew => {
            let height = 8 + ((h >> 16) % 4) as i32;
            let (vox, trunk_shape) = if is_slender {
                (Voxel::YewWood, BlockShape::Column)
            } else {
                (Voxel::YewLog, BlockShape::Full)
            };
            stamp_conifer(w, rx, ry, rz, height, 3, trunk_shape, vox, Voxel::YewLeaves, h);
        }
        TreeArchetype::Redwood => {
            let height = 15 + ((h >> 16) % 5) as i32;
            stamp_buttress_roots(w, rx, ry, rz, Voxel::RedwoodWood, h);
            stamp_conifer(
                w,
                rx,
                ry,
                rz,
                height,
                5,
                BlockShape::Full,
                Voxel::RedwoodLog,
                Voxel::RedwoodLeaves,
                h,
            );
        }
        TreeArchetype::Acacia => {
            let height = 6 + ((h >> 16) % 3) as i32;
            let (lean_x, lean_z) = match (h >> 10) % 4 {
                0 => (1, 0),
                1 => (-1, 0),
                2 => (0, 1),
                _ => (0, -1),
            };
            let (up_st, inv_st) = curve_stair_orientations(lean_x, lean_z);
            let mut tx = rx;
            let mut tz = rz;
            for dy in 0..height {
                if dy == 2 || dy == 4 {
                    // Slender stair-to-stair bend using all-bark AcaciaWood
                    w.set_log_shaped(tx, ry + dy, tz, Voxel::AcaciaWood, BlockShape::Stair, up_st);
                    tx += lean_x;
                    tz += lean_z;
                    w.set_log_shaped(tx, ry + dy, tz, Voxel::AcaciaWood, BlockShape::Stair, inv_st);
                } else {
                    w.set_log(tx, ry + dy, tz, Voxel::AcaciaLog);
                }
            }
            let cy = ry + height;
            stamp_acacia_umbrella(w, tx, cy, tz, 3.6, Voxel::AcaciaLeaves, h);
            // Horizontal fork branch using centered Column
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, -lean_x, -lean_z, 2, Voxel::AcaciaWood);
            stamp_acacia_umbrella(
                w,
                rx - lean_x * 2,
                cy - 1,
                rz - lean_z * 2,
                2.4,
                Voxel::AcaciaLeaves,
                h ^ 0x333,
            );
        }
        TreeArchetype::Palm => {
            let (lean_x, lean_z) = match (h >> 10) % 4 {
                0 => (1, 0),
                1 => (-1, 0),
                2 => (0, 1),
                _ => (0, -1),
            };
            let (up_st, inv_st) = curve_stair_orientations(lean_x, lean_z);
            let mut tx = rx;
            let mut tz = rz;

            // 4 varied palm trunk styles:
            // 0 = Straight shorter slender Column palm (5..=6 blocks),
            // 1 = Gentle 1-bend, 2 = Classic 2-bend, 3 = Dramatic sweeping 3-bend coastal palm
            let palm_style = (h >> 13) % 4;
            let height = if palm_style == 0 {
                5 + ((h >> 16) % 2) as i32
            } else {
                7 + ((h >> 16) % 4) as i32
            };
            for dy in 0..height {
                let bend_step = match palm_style {
                    0 => false,
                    1 => dy == (height / 2),
                    2 => dy == 2 || dy == 5,
                    _ => dy == 1 || dy == 3 || dy == 5,
                };
                if bend_step {
                    w.set_log_shaped(tx, ry + dy, tz, Voxel::PalmWood, BlockShape::Stair, up_st);
                    tx += lean_x;
                    tz += lean_z;
                    w.set_log_shaped(tx, ry + dy, tz, Voxel::PalmWood, BlockShape::Stair, inv_st);
                } else if palm_style == 0 {
                    w.set_log_shaped(tx, ry + dy, tz, Voxel::PalmWood, BlockShape::Column, 0);
                } else {
                    w.set_log(tx, ry + dy, tz, Voxel::PalmWood);
                }
            }

            let top_y = ry + height;
            // Slightly fuller rounded palm crown
            w.set_leaf(tx, top_y, tz, Voxel::PalmLeaves);
            w.set_leaf(tx, top_y + 1, tz, Voxel::PalmLeaves);

            // 4 graceful arching cardinal fronds with tapered Slab tips + Hanging Coconuts
            const DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            for (i, (dx, dz)) in DIRS.iter().enumerate() {
                w.set_leaf(tx + dx, top_y, tz + dz, Voxel::PalmLeaves);
                w.set_leaf_shaped(
                    tx + dx,
                    top_y + 1,
                    tz + dz,
                    Voxel::PalmLeaves,
                    BlockShape::Slab,
                    0,
                );
                w.set_leaf(tx + dx * 2, top_y, tz + dz * 2, Voxel::PalmLeaves);
                w.set_leaf_shaped(
                    tx + dx * 2,
                    top_y + 1,
                    tz + dz * 2,
                    Voxel::PalmLeaves,
                    BlockShape::Slab,
                    0,
                );
                w.set_leaf(tx + dx * 3, top_y - 1, tz + dz * 3, Voxel::PalmLeaves);
                w.set_leaf_shaped(
                    tx + dx * 4,
                    top_y - 2,
                    tz + dz * 4,
                    Voxel::PalmLeaves,
                    BlockShape::Slab,
                    1,
                );

                // Hanging Coconuts underneath the inner palm fronds
                let coco_roll = (h >> (18 + i * 2)) & 3;
                if coco_roll == 0 || (i == 0 && ((h >> 26) & 1) == 0) {
                    let stage_vox = if ((h >> (22 + i)) & 1) == 0 {
                        Voxel::CropCoconutStage2
                    } else {
                        Voxel::CropCoconutStage1
                    };
                    w.set_shaped(
                        tx + dx,
                        top_y - 1,
                        tz + dz,
                        stage_vox,
                        BlockShape::HangingFruit,
                        0,
                    );
                }
            }
            // 4 diagonal fronds
            for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                w.set_leaf(tx + dx, top_y, tz + dz, Voxel::PalmLeaves);
                w.set_leaf(tx + dx * 2, top_y - 1, tz + dz * 2, Voxel::PalmLeaves);
                w.set_leaf_shaped(
                    tx + dx * 3,
                    top_y - 2,
                    tz + dz * 3,
                    Voxel::PalmLeaves,
                    BlockShape::Slab,
                    1,
                );
            }
        }
        TreeArchetype::Mahogany => {
            let height = 9 + ((h >> 16) % 5) as i32;
            stamp_buttress_roots(w, rx, ry, rz, Voxel::MahoganyWood, h);
            for dy in 0..height {
                w.set_log(rx, ry + dy, rz, Voxel::MahoganyLog);
            }
            let bx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
            let bz = if ((h >> 11) & 1) == 0 { 1 } else { -1 };
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, bx, 0, 2, Voxel::MahoganyWood);
            stamp_horizontal_branch(w, rx, ry + height - 3, rz, 0, bz, 2, Voxel::MahoganyWood);
            stamp_leaf_cloud(w, rx, ry + height + 1, rz, 3.8, 2.2, 3.8, Voxel::MahoganyLeaves, h);
            stamp_leaf_cloud(w, rx + bx * 2, ry + height - 1, rz, 2.8, 1.8, 2.8, Voxel::MahoganyLeaves, h ^ 0x101);
            stamp_leaf_cloud(w, rx, ry + height - 1, rz + bz * 2, 2.8, 1.8, 2.8, Voxel::MahoganyLeaves, h ^ 0x202);

            // Lateral Cocoa Pods attached directly to the Mahogany trunk
            const SIDES: [(i32, i32, u8); 4] = [(0, 1, 0), (-1, 0, 1), (0, -1, 2), (1, 0, 3)];
            for (i, &(dx, dz, orient)) in SIDES.iter().enumerate() {
                if ((h >> (18 + i * 2)) % 3) == 0 {
                    let pod_y = ry + 3 + ((h >> (12 + i)) % (height - 5).max(1) as u32) as i32;
                    let stage_vox = match (h >> (24 + i)) % 3 {
                        0 => Voxel::CropCocoaStage1,
                        _ => Voxel::CropCocoaStage2,
                    };
                    w.set_shaped(
                        rx + dx,
                        pod_y,
                        rz + dz,
                        stage_vox,
                        BlockShape::CocoaPod,
                        orient,
                    );
                }
            }
        }
        TreeArchetype::Willow => {
            let height = 7 + ((h >> 16) % 3) as i32;
            stamp_buttress_roots(w, rx, ry, rz, Voxel::WillowWood, h);
            for dy in 0..height {
                w.set_log(rx, ry + dy, rz, Voxel::WillowLog);
            }
            let bx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
            let bz = if ((h >> 11) & 1) == 0 { 1 } else { -1 };
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, bx, 0, 2, Voxel::WillowWood);
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, 0, bz, 2, Voxel::WillowWood);
            let cy = ry + height;
            stamp_leaf_cloud(w, rx, cy + 1, rz, 3.6, 2.2, 3.6, Voxel::WillowLeaves, h);
            for dz in -4..=4 {
                for dx in -4..=4 {
                    let d2 = dx * dx + dz * dz;
                    if (9..=15).contains(&d2) && ((dx * 7 + dz * 13 + h as i32) & 1) == 0 {
                        let drop = 2 + ((dx.abs() + dz.abs() + h as i32) % 3);
                        for dd in 0..drop {
                            w.set_leaf(rx + dx, cy - dd, rz + dz, Voxel::WillowLeaves);
                        }
                    }
                }
            }
        }
        TreeArchetype::Mangrove => {
            let height = 7 + ((h >> 16) % 3) as i32;
            for &(dx, dz, orient) in &[(-1, 0, 0u8), (1, 0, 1), (0, -1, 2), (0, 1, 3)] {
                w.set_log_shaped(rx + dx, ry, rz + dz, Voxel::MangroveWood, BlockShape::Column, 0);
                w.set_log_shaped(rx + dx, ry + 1, rz + dz, Voxel::MangroveWood, BlockShape::Column, 0);
                w.set_log_shaped(rx + dx, ry + 2, rz + dz, Voxel::MangroveWood, BlockShape::Stair, orient);
            }
            for dy in 2..height {
                w.set_log(rx, ry + dy, rz, Voxel::MangroveLog);
            }
            stamp_leaf_cloud(w, rx, ry + height + 1, rz, 3.4, 2.2, 3.4, Voxel::MangroveLeaves, h);
        }
        TreeArchetype::Charred => {
            let height = 4 + ((h >> 16) % 4) as i32;
            let (vox, trunk_shape) = if is_slender {
                (Voxel::TreeCharredBark, BlockShape::Column)
            } else {
                (Voxel::TreeCharredLog, BlockShape::Full)
            };
            for dy in 0..height {
                w.set_log_shaped(rx, ry + dy, rz, vox, trunk_shape, 0);
            }
            if height >= 5 {
                let bx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
                stamp_horizontal_branch(w, rx, ry + height - 2, rz, bx, 0, 1, Voxel::TreeCharredBark);
            }
        }
    }
}

fn stamp_acacia_umbrella(
    w: &mut ChunkWriter,
    cx: i32,
    cy: i32,
    cz: i32,
    radius: f32,
    leaves: Voxel,
    h: u32,
) {
    let r_i = radius.ceil() as i32;
    let r2 = radius * radius;
    for dz in -r_i..=r_i {
        for dx in -r_i..=r_i {
            let d2 = (dx * dx + dz * dz) as f32;
            if d2 <= r2 {
                if d2 > r2 * 0.72 {
                    let ch = hash_tree_cell(cx + dx * 5, cz + dz * 7, h);
                    if (ch & 3) == 0 {
                        continue;
                    }
                    w.set_leaf_shaped(cx + dx, cy, cz + dz, leaves, BlockShape::Slab, 1);
                } else {
                    w.set_leaf(cx + dx, cy, cz + dz, leaves);
                }
            }
            if d2 <= r2 * 0.42 {
                w.set_leaf(cx + dx, cy + 1, cz + dz, leaves);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stamp_conifer(
    w: &mut ChunkWriter,
    rx: i32,
    ry: i32,
    rz: i32,
    height: i32,
    canopy_start: i32,
    trunk_shape: BlockShape,
    log: Voxel,
    leaves: Voxel,
    h: u32,
) {
    for dy in 0..height {
        w.set_log_shaped(rx, ry + dy, rz, log, trunk_shape, 0);
    }
    w.set_leaf(rx, ry + height, rz, leaves);
    w.set_leaf(rx, ry + height + 1, rz, leaves);
    w.set_leaf_shaped(rx, ry + height + 2, rz, leaves, BlockShape::Slab, 0);
    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        w.set_leaf(rx + dx, ry + height, rz + dz, leaves);
    }

    let canopy_h = (height - canopy_start).max(1);
    let max_r = if height >= 14 { 4.0f32 } else { 3.0f32 };

    for dy in canopy_start..height {
        let progress = (dy - canopy_start) as f32 / canopy_h as f32;
        let tier_wave = if ((dy - canopy_start) % 3) == 0 {
            1.0
        } else if ((dy - canopy_start) % 3) == 1 {
            0.78
        } else {
            0.55
        };
        let r = ((1.0 - progress * 0.85) * max_r * tier_wave).max(1.0);
        let r_i = r.ceil() as i32;
        let r2 = r * r;

        for dz in -r_i..=r_i {
            for dx in -r_i..=r_i {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let d2 = (dx * dx + dz * dz) as f32;
                if d2 > r2 + 0.35 {
                    continue;
                }
                if d2 > r2 * 0.72 {
                    let ch = hash_tree_cell(rx + dx * 3 + dy * 7, rz + dz * 5, h);
                    if (ch & 3) == 0 {
                        continue;
                    }
                }
                w.set_leaf(rx + dx, ry + dy, rz + dz, leaves);
            }
        }
    }
}

/// Lightweight proxy descriptor for rendering trees in distant LOD1 / LOD2 chunks.
#[derive(Clone, Copy, Debug)]
pub struct LodTreeDesc {
    pub root_x: i32,
    pub root_y: i32,
    pub root_z: i32,
    pub trunk_height: f32,
    pub canopy_radius: f32,
    pub canopy_height: f32,
    pub log_voxel: Voxel,
    pub leaf_voxel: Option<Voxel>,
    pub is_conifer: bool,
}

/// Samples all procedural trees whose roots fall inside `[min_x..max_x, min_z..max_z]`
/// using the exact same cellular grid and biome rules as LOD0 `populate_chunk_trees`.
pub fn sample_lod_trees_in_region<F, G>(
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
    seed: u32,
    column_at: F,
    ground_at: G,
) -> Vec<LodTreeDesc>
where
    F: Fn(i32, i32) -> TerrainColumn,
    G: Fn(i32, i32, i32, TerrainColumn) -> Option<Voxel>,
{
    const CELL_SIZE: i32 = 7;

    let min_cell_x = min_x.div_euclid(CELL_SIZE);
    let max_cell_x = max_x.div_euclid(CELL_SIZE);
    let min_cell_z = min_z.div_euclid(CELL_SIZE);
    let max_cell_z = max_z.div_euclid(CELL_SIZE);

    let mut out = Vec::new();

    for cell_z in min_cell_z..=max_cell_z {
        for cell_x in min_cell_x..=max_cell_x {
            let h = hash_tree_cell(cell_x, cell_z, seed ^ 0x77EE_1234);
            let jx = 1 + ((h & 15) as i32 % (CELL_SIZE - 2));
            let jz = 1 + (((h >> 4) & 15) as i32 % (CELL_SIZE - 2));
            let root_x = cell_x * CELL_SIZE + jx;
            let root_z = cell_z * CELL_SIZE + jz;

            if root_x < min_x || root_x > max_x || root_z < min_z || root_z > max_z {
                continue;
            }

            let col = column_at(root_x, root_z);
            let root_y = col.terrain_height;

            if col.is_cliff
                || col.is_river
                || col.surface_shape != BlockShape::Full
                || col.water_level.is_some_and(|wl| root_y <= wl)
            {
                continue;
            }

            let Some(ground) = ground_at(root_x, root_y, root_z, col) else {
                continue;
            };

            let Some((chance, archetype)) = select_tree_for_column(col, ground, h) else {
                continue;
            };

            if hash_f32(hash_tree_cell(cell_x, cell_z, seed ^ 0x99AB_CDEF)) >= chance {
                continue;
            }

            let (trunk_h, canopy_r, canopy_h, log_v, leaf_v, is_conifer) = match archetype {
                TreeArchetype::Oak => (6.5, 3.0, 4.0, Voxel::OakWood, Some(Voxel::OakLeaves), false),
                TreeArchetype::Birch => (7.5, 2.5, 4.2, Voxel::BirchWood, Some(Voxel::BirchLeaves), false),
                TreeArchetype::Maple(l) => (7.0, 3.2, 4.0, Voxel::MapleWood, Some(l), false),
                TreeArchetype::Cherry => (6.5, 3.4, 3.5, Voxel::CherryWood, Some(Voxel::CherryLeaves), false),
                TreeArchetype::Pine => (11.0, 3.0, 8.0, Voxel::PineWood, Some(Voxel::PineLeaves), true),
                TreeArchetype::Yew => (9.0, 2.6, 6.0, Voxel::YewWood, Some(Voxel::YewLeaves), true),
                TreeArchetype::Redwood => (16.0, 3.6, 11.5, Voxel::RedwoodWood, Some(Voxel::RedwoodLeaves), true),
                TreeArchetype::Acacia => (6.5, 3.6, 2.2, Voxel::AcaciaWood, Some(Voxel::AcaciaLeaves), false),
                TreeArchetype::Palm => (6.5, 2.8, 2.0, Voxel::PalmWood, Some(Voxel::PalmLeaves), false),
                TreeArchetype::Mahogany => (10.5, 3.8, 4.5, Voxel::MahoganyWood, Some(Voxel::MahoganyLeaves), false),
                TreeArchetype::Willow => (7.0, 3.8, 5.0, Voxel::WillowWood, Some(Voxel::WillowLeaves), false),
                TreeArchetype::Mangrove => (7.0, 3.4, 4.0, Voxel::MangroveWood, Some(Voxel::MangroveLeaves), false),
                TreeArchetype::Charred => (5.0, 0.0, 0.0, Voxel::TreeCharredBark, None, false),
            };

            out.push(LodTreeDesc {
                root_x,
                root_y,
                root_z,
                trunk_height: trunk_h,
                canopy_radius: canopy_r,
                canopy_height: canopy_h,
                log_voxel: log_v,
                leaf_voxel: leaf_v,
                is_conifer,
            });
        }
    }

    out
}
