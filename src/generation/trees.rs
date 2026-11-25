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
    GiantMushroom(Voxel),
}

/// Determines tree spawn probability and archetype for a given biome/column.
fn select_tree_for_column(
    col: TerrainColumn,
    ground: Voxel,
    h: u32,
) -> Option<(f32, TreeArchetype)> {
    // Gravel Beach & Craggy Beach: sparse coastal palms only (no pines on beaches!)
    if matches!(col.biome, BiomeType::GravelBeach | BiomeType::CraggyBeach) {
        if col.climate.temperature > -0.25 {
            let on_dirty_gravel = matches!(ground, Voxel::GravelDirty | Voxel::GravelAshy);
            let chance = if on_dirty_gravel { 0.16 } else { 0.08 };
            return Some((chance, TreeArchetype::Palm));
        }
        return None;
    }

    // Coastal beaches & Tropical/Coral Islands: evenly distributed Palms across warm & temperate coastlines
    if col.is_beach
        || matches!(
            col.biome,
            BiomeType::SandyShores
                | BiomeType::DuneBeach
                | BiomeType::TropicalIsland
                | BiomeType::CoralIsland
        )
    {
        if col.climate.temperature > -0.25 {
            return Some((0.18, TreeArchetype::Palm));
        }
        return None;
    }

    match col.biome {
        BiomeType::VerdantWoods
        | BiomeType::TemperateIsland
        | BiomeType::ShieldForest => {
            if (h >> 18).is_multiple_of(20) {
                let cap = match (h >> 12) % 4 {
                    0 => Voxel::FungiAmanitaCap,
                    1 => Voxel::FungiPorciniCap,
                    2 => Voxel::FungiRussulaCap,
                    _ => Voxel::FungiShiitakeCap,
                };
                return Some((0.64, TreeArchetype::GiantMushroom(cap)));
            }
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
        BiomeType::AutumnForest => {
            if (h >> 18).is_multiple_of(18) {
                let cap = match (h >> 12) % 3 {
                    0 => Voxel::FungiAmanitaCap,
                    1 => Voxel::FungiPorciniCap,
                    _ => Voxel::FungiRussulaCap,
                };
                return Some((0.68, TreeArchetype::GiantMushroom(cap)));
            }
            let arch = match (h >> 8) % 5 {
                0 => TreeArchetype::Birch,
                1 => TreeArchetype::Oak,
                _ => TreeArchetype::Maple(match (h >> 12) % 3 {
                    0 => Voxel::MapleLeavesOrange,
                    1 => Voxel::MapleLeavesRed,
                    _ => Voxel::MapleLeavesYellow,
                }),
            };
            Some((0.68, arch))
        }
        BiomeType::MistyWoods => {
            if (h >> 18).is_multiple_of(11) {
                let cap = match (h >> 12) % 4 {
                    0 => Voxel::FungiAmanitaCap,
                    1 => Voxel::FungiPorciniCap,
                    2 => Voxel::FungiShiitakeCap,
                    _ => Voxel::FungiVerdigrisCap,
                };
                return Some((0.76, TreeArchetype::GiantMushroom(cap)));
            }
            let arch = match (h >> 8) % 5 {
                0 => TreeArchetype::Redwood,
                1 | 2 => TreeArchetype::Oak,
                3 => TreeArchetype::Maple(Voxel::MapleLeavesOrange),
                _ => TreeArchetype::Birch,
            };
            Some((0.76, arch))
        }
        BiomeType::FertileValley | BiomeType::FlowerMeadow => {
            let arch = match (h >> 8) % 4 {
                0 | 1 => TreeArchetype::Birch,
                2 => TreeArchetype::Oak,
                _ => TreeArchetype::Maple(Voxel::MapleLeavesYellow),
            };
            let chance = if col.biome == BiomeType::FlowerMeadow {
                0.24
            } else {
                0.44
            };
            Some((chance, arch))
        }
        BiomeType::Grasslands | BiomeType::RollingSteppe => {
            let arch = if ((h >> 8) & 1) == 0 {
                TreeArchetype::Oak
            } else {
                TreeArchetype::Birch
            };
            Some((0.12, arch))
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
        BiomeType::PineHighlands
        | BiomeType::BleakTaiga
        | BiomeType::WintryForest
        | BiomeType::FrostPinewoods
        | BiomeType::IceboundRidge => {
            if !matches!(
                col.biome,
                BiomeType::WintryForest | BiomeType::FrostPinewoods | BiomeType::IceboundRidge
            ) && (h >> 18).is_multiple_of(20)
            {
                let cap = match (h >> 12) % 3 {
                    0 => Voxel::FungiAmanitaCap,
                    1 => Voxel::FungiPorciniCap,
                    _ => Voxel::FungiRussulaCap,
                };
                return Some((0.72, TreeArchetype::GiantMushroom(cap)));
            }
            let arch = match (h >> 8) % 6 {
                0 => TreeArchetype::Redwood,
                1 => TreeArchetype::Yew,
                _ => TreeArchetype::Pine,
            };
            let chance = if col.biome == BiomeType::IceboundRidge {
                0.42
            } else {
                0.72
            };
            Some((chance, arch))
        }
        BiomeType::ColdShrubland
        | BiomeType::PermafrostSteppe
        | BiomeType::TundraIsland
        | BiomeType::AlpinePeaks
        | BiomeType::JaggedPeaks
        | BiomeType::EmeraldFoothills
        | BiomeType::GranitePlateau
        | BiomeType::WindyPass => Some((0.28, TreeArchetype::Pine)),
        BiomeType::AcaciaPlains
        | BiomeType::FracturedSavanna
        | BiomeType::SavannaBadlands => Some((0.36, TreeArchetype::Acacia)),
        BiomeType::DesertOasis => Some((0.62, TreeArchetype::Palm)),
        BiomeType::DenseRainforest
        | BiomeType::BambooGrove
        | BiomeType::JungleWetlands
        | BiomeType::VolcanicJungle => {
            if (h >> 18).is_multiple_of(16) {
                let cap = match (h >> 12) % 3 {
                    0 => Voxel::FungiVerdigrisCap,
                    1 => Voxel::FungiShiitakeCap,
                    _ => Voxel::FungiRussulaCap,
                };
                return Some((0.82, TreeArchetype::GiantMushroom(cap)));
            }
            let arch = if (h >> 8).is_multiple_of(5) {
                TreeArchetype::Palm
            } else {
                TreeArchetype::Mahogany
            };
            let chance = if col.biome == BiomeType::BambooGrove {
                0.48
            } else {
                0.82
            };
            Some((chance, arch))
        }
        BiomeType::WillowMarsh
        | BiomeType::MangroveSwamp
        | BiomeType::OrchidSwamp
        | BiomeType::SwampyShore => {
            if (h >> 18).is_multiple_of(10) {
                let cap = match (h >> 12) % 4 {
                    0 => Voxel::FungiVerdigrisCap,
                    1 => Voxel::FungiShiitakeCap,
                    2 => Voxel::FungiAmanitaCap,
                    _ => Voxel::FungiPorciniCap,
                };
                return Some((0.62, TreeArchetype::GiantMushroom(cap)));
            }
            let arch = match col.biome {
                BiomeType::WillowMarsh => TreeArchetype::Willow,
                BiomeType::MangroveSwamp => TreeArchetype::Mangrove,
                _ => {
                    if ((h >> 8) & 1) == 0 {
                        TreeArchetype::Willow
                    } else {
                        TreeArchetype::Mangrove
                    }
                }
            };
            Some((0.62, arch))
        }
        BiomeType::WitheredWoods
        | BiomeType::AshenWastes
        | BiomeType::ObsidianMire
        | BiomeType::VolcanicPeaks
        | BiomeType::ScorchedFlats => {
            let chance = if col.biome == BiomeType::WitheredWoods {
                0.46
            } else {
                0.22
            };
            Some((chance, TreeArchetype::Charred))
        }
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

    /// Places a solid or shaped rock voxel into empty air or water (used for underwater
    /// Coralstone clusters and coastal Reefstone boulders & sea-stack spires).
    #[inline(always)]
    fn set_rock_shaped(
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
        let cur = self.voxels[idx];
        if cur.is_empty() || cur.is_water() {
            self.voxels[idx] = voxel;
            if shape != BlockShape::Full || orient != 0 {
                self.chunk_shapes.push((idx, shape, orient));
            }
            self.wrote_any = true;
        }
    }

    #[inline(always)]
    fn set_rock(&mut self, wx: i32, wy: i32, wz: i32, voxel: Voxel) {
        self.set_rock_shaped(wx, wy, wz, voxel, BlockShape::Full, 0);
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
        _ if ground.is_fertile_soil() => {
            Some((Voxel::LoamRooted, Voxel::LoamCoarse, Voxel::LoamMulch))
        }
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
                // Keep the dense interior 100% Full blocks for greedy meshing, and sculpt
                // the outer silhouette primarily with inward-facing Stairs (+ subtle Slab tips).
                if dist_sq > 0.56 {
                    let cell_h = hash_tree_cell(cx + dx * 3 + dy * 7, cz + dz * 5 - dy * 11, h);
                    let inward_orient: u8 = if dx.abs() >= dz.abs() {
                        if dx < 0 { 0 } else { 1 }
                    } else if dz < 0 {
                        2
                    } else {
                        3
                    };
                    if dy > 0 && ny > 0.25 {
                        if (dx == 0 && dz == 0 && dist_sq > 0.74) || dist_sq > 0.95 {
                            w.set_leaf_shaped(
                                cx + dx,
                                cy + dy,
                                cz + dz,
                                leaves,
                                BlockShape::Slab,
                                0,
                            );
                        } else if !(cell_h >> 6).is_multiple_of(5) {
                            w.set_leaf_shaped(
                                cx + dx,
                                cy + dy,
                                cz + dz,
                                leaves,
                                BlockShape::Stair,
                                inward_orient,
                            );
                        } else {
                            w.set_leaf(cx + dx, cy + dy, cz + dz, leaves);
                        }
                        continue;
                    } else if dy < 0 && ny < -0.22 {
                        if (dx == 0 && dz == 0 && dist_sq > 0.74) || dist_sq > 0.95 {
                            w.set_leaf_shaped(
                                cx + dx,
                                cy + dy,
                                cz + dz,
                                leaves,
                                BlockShape::Slab,
                                1,
                            );
                        } else if !(cell_h >> 6).is_multiple_of(5) {
                            w.set_leaf_shaped(
                                cx + dx,
                                cy + dy,
                                cz + dz,
                                leaves,
                                BlockShape::Stair,
                                inward_orient + 4,
                            );
                        } else {
                            w.set_leaf(cx + dx, cy + dy, cz + dz, leaves);
                        }
                        continue;
                    } else if dist_sq > 0.76 && !(cell_h >> 8).is_multiple_of(3) {
                        let stair_orient = if ny >= 0.0 {
                            inward_orient
                        } else {
                            inward_orient + 4
                        };
                        w.set_leaf_shaped(
                            cx + dx,
                            cy + dy,
                            cz + dz,
                            leaves,
                            BlockShape::Stair,
                            stair_orient,
                        );
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

            if matches!(
                archetype,
                TreeArchetype::Oak
                    | TreeArchetype::Birch
                    | TreeArchetype::Maple(_)
                    | TreeArchetype::Willow
                    | TreeArchetype::Mahogany
            ) {
                stamp_under_canopy_leaf_litter(
                    &mut writer,
                    root_x,
                    root_z,
                    seed,
                    &column_at,
                    &ground_at,
                );
            } else if matches!(archetype, TreeArchetype::Palm) {
                stamp_under_palm_beach_grass(
                    &mut writer,
                    root_x,
                    root_z,
                    seed,
                    &column_at,
                    &ground_at,
                );
            } else if let TreeArchetype::GiantMushroom(cap_voxel) = archetype {
                stamp_under_giant_mushroom(
                    &mut writer,
                    root_x,
                    root_z,
                    cap_voxel,
                    seed,
                    &column_at,
                    &ground_at,
                );
            }
        }
    }

    writer.wrote_any
}

/// Maps a biome's primary host rock to its `(cobble, mossy_cobble)` surface boulder pair.
fn rock_to_cobble_pair(primary_stone: Voxel) -> (Voxel, Voxel) {
    match primary_stone {
        Voxel::Andesite => (Voxel::CobbledAndesite, Voxel::MossyCobbledAndesite),
        Voxel::Argillite => (Voxel::CobbledArgillite, Voxel::MossyCobbledArgillite),
        Voxel::Basalt => (Voxel::CobbledBasalt, Voxel::MossyCobbledBasalt),
        Voxel::Chalk => (Voxel::CobbledChalk, Voxel::MossyCobbledChalk),
        Voxel::Dolomite => (Voxel::CobbledDolomite, Voxel::MossyCobbledDolomite),
        Voxel::Granite => (Voxel::CobbledGranite, Voxel::MossyCobbledGranite),
        Voxel::Limestone => (Voxel::CobbledLimestone, Voxel::MossyCobbledLimestone),
        Voxel::Gabbro => (Voxel::CobbledGabbro, Voxel::MossyCobbledBasalt),
        Voxel::Slate => (Voxel::CobbledSlate, Voxel::MossyCobbledArgillite),
        Voxel::Marble => (Voxel::CobbledMarble, Voxel::MossyCobbledLimestone),
        _ => (Voxel::CobbledAndesite, Voxel::MossyCobbledAndesite),
    }
}

/// Selects a biome-appropriate `(log, leaves)` pair and spawn probability for ground leaf bushes.
fn select_bush_for_column(col: TerrainColumn, h: u32) -> Option<(f32, Voxel, Voxel)> {
    if col.is_beach || col.is_cliff || col.is_river {
        return None;
    }

    let (chance, log, leaves) = match col.biome {
        BiomeType::VerdantWoods
        | BiomeType::TemperateIsland
        | BiomeType::ShieldForest
        | BiomeType::Grasslands => {
            let (l, lf) = match (h >> 8) % 4 {
                0 | 1 => (Voxel::OakLog, Voxel::OakLeaves),
                2 => (Voxel::BirchLog, Voxel::BirchLeaves),
                _ => (Voxel::PineLog, Voxel::PineLeaves),
            };
            (0.22, l, lf)
        }
        BiomeType::MistyWoods | BiomeType::AutumnForest => {
            let (l, lf) = match (h >> 8) % 3 {
                0 => (Voxel::OakLog, Voxel::OakLeaves),
                1 => (Voxel::BirchLog, Voxel::BirchLeaves),
                _ => (Voxel::MapleLog, Voxel::MapleLeavesOrange),
            };
            (0.24, l, lf)
        }
        BiomeType::FertileValley
        | BiomeType::FlowerMeadow
        | BiomeType::LavenderValley
        | BiomeType::SakuraGrove => {
            let (l, lf) = match (h >> 8) % 3 {
                0 => (Voxel::BirchLog, Voxel::BirchLeaves),
                1 => (Voxel::OakLog, Voxel::OakLeaves),
                _ => (Voxel::CherryLog, Voxel::CherryLeaves),
            };
            (0.18, l, lf)
        }
        BiomeType::DenseRainforest
        | BiomeType::BambooGrove
        | BiomeType::JungleWetlands
        | BiomeType::VolcanicJungle
        | BiomeType::TropicalIsland => {
            let (l, lf) = if ((h >> 8) & 1) == 0 {
                (Voxel::MahoganyLog, Voxel::MahoganyLeaves)
            } else {
                (Voxel::OakLog, Voxel::OakLeaves)
            };
            (0.26, l, lf)
        }
        BiomeType::BleakTaiga
        | BiomeType::WintryForest
        | BiomeType::FrostPinewoods
        | BiomeType::ColdShrubland
        | BiomeType::PermafrostSteppe
        | BiomeType::TundraIsland => {
            let (l, lf) = if (h >> 8).is_multiple_of(3) {
                (Voxel::BirchLog, Voxel::BirchLeaves)
            } else {
                (Voxel::PineLog, Voxel::PineLeaves)
            };
            (0.20, l, lf)
        }
        BiomeType::WillowMarsh
        | BiomeType::MangroveSwamp
        | BiomeType::OrchidSwamp
        | BiomeType::SwampyShore => (0.20, Voxel::WillowLog, Voxel::WillowLeaves),
        BiomeType::AcaciaPlains
        | BiomeType::FracturedSavanna
        | BiomeType::SavannaBadlands
        | BiomeType::DesertOasis => (0.12, Voxel::AcaciaLog, Voxel::AcaciaLeaves),
        BiomeType::AlpinePeaks
        | BiomeType::JaggedPeaks
        | BiomeType::EmeraldFoothills
        | BiomeType::GranitePlateau
        | BiomeType::MistyCrags => (0.12, Voxel::PineLog, Voxel::PineLeaves),
        _ => return None,
    };

    Some((chance, log, leaves))
}

/// Selects spawn probability and mossiness ratio for surface cobble / mossy cobble rock clusters.
fn boulder_chance_for_biome(col: TerrainColumn) -> Option<(f32, f32)> {
    if col.is_beach || col.is_cliff || col.is_river {
        return None;
    }
    match col.biome {
        BiomeType::VerdantWoods
        | BiomeType::AutumnForest
        | BiomeType::ShieldForest
        | BiomeType::BleakTaiga
        | BiomeType::WintryForest
        | BiomeType::FrostPinewoods
        | BiomeType::MistyWoods => Some((0.18, 0.52)),
        BiomeType::FertileValley
        | BiomeType::FlowerMeadow
        | BiomeType::Grasslands
        | BiomeType::RollingSteppe
        | BiomeType::LavenderValley
        | BiomeType::SakuraGrove
        | BiomeType::ColdShrubland
        | BiomeType::PermafrostSteppe
        | BiomeType::TundraIsland => Some((0.14, 0.42)),
        BiomeType::DenseRainforest
        | BiomeType::BambooGrove
        | BiomeType::JungleWetlands
        | BiomeType::VolcanicJungle
        | BiomeType::WillowMarsh
        | BiomeType::MangroveSwamp
        | BiomeType::OrchidSwamp => Some((0.16, 0.65)),
        BiomeType::AcaciaPlains
        | BiomeType::FracturedSavanna
        | BiomeType::SavannaBadlands
        | BiomeType::WitheredWoods
        | BiomeType::AshenWastes
        | BiomeType::ScorchedFlats
        | BiomeType::JaggedPeaks
        | BiomeType::AlpinePeaks
        | BiomeType::EmeraldFoothills
        | BiomeType::GranitePlateau
        | BiomeType::MistyCrags
        | BiomeType::MonolithCliffs
        | BiomeType::DeepGorge
        | BiomeType::WindyPass
        | BiomeType::RockyIsland => Some((0.14, 0.25)),
        _ => None,
    }
}

/// Populates multi-chunk surface cobble/mossy cobble rock clusters and ground leaf bushes
/// seamlessly across chunk boundaries. Runs after trees and before flora.
pub fn populate_chunk_boulders_and_bushes<F, G>(
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
    let chunk_min_y = chunk_origin.y;
    let chunk_max_y = chunk_origin.y + CHUNK_SIZE as i32 - 1;

    let mut writer = ChunkWriter {
        voxels,
        chunk_shapes,
        origin: chunk_origin,
        wrote_any: false,
    };

    // --- 1. Surface Cobble & Mossy Cobble Rock Clusters ---
    {
        const BOULDER_CELL: i32 = 10;
        const BOULDER_RADIUS: i32 = 2;

        let min_cx = (chunk_origin.x - BOULDER_RADIUS).div_euclid(BOULDER_CELL);
        let max_cx =
            (chunk_origin.x + CHUNK_SIZE as i32 - 1 + BOULDER_RADIUS).div_euclid(BOULDER_CELL);
        let min_cz = (chunk_origin.z - BOULDER_RADIUS).div_euclid(BOULDER_CELL);
        let max_cz =
            (chunk_origin.z + CHUNK_SIZE as i32 - 1 + BOULDER_RADIUS).div_euclid(BOULDER_CELL);

        for cz in min_cz..=max_cz {
            for cx in min_cx..=max_cx {
                let h = hash_tree_cell(cx, cz, seed ^ 0xB01D_E211);
                let jx = 2 + ((h & 15) as i32 % (BOULDER_CELL - 4));
                let jz = 2 + (((h >> 4) & 15) as i32 % (BOULDER_CELL - 4));
                let rx = cx * BOULDER_CELL + jx;
                let rz = cz * BOULDER_CELL + jz;

                let col = column_at(rx, rz);
                let ry = col.terrain_height;

                if chunk_min_y > ry + 4 || chunk_max_y < ry - 1 {
                    continue;
                }
                if col.surface_shape != BlockShape::Full
                    || col.water_level.is_some_and(|wl| ry <= wl)
                {
                    continue;
                }
                let Some((chance, moss_prob)) = boulder_chance_for_biome(col) else {
                    continue;
                };
                if hash_f32(hash_tree_cell(cx, cz, seed ^ 0xB01D_9933)) >= chance {
                    continue;
                }
                let Some(ground) = ground_at(rx, ry, rz, col) else {
                    continue;
                };
                // ~24% of surface boulders are mineral Quartz / Flint / Chert boulders
                let is_mineral_boulder = ((h >> 18) % 25) < 6;
                if !is_mineral_boulder && !ground.is_fertile_soil() && !ground.is_grass_soil() {
                    continue;
                }

                let mineral_triple: Option<(Voxel, Voxel, Voxel)> = if is_mineral_boulder {
                    let kind = if (h >> 22).is_multiple_of(3) {
                        (h >> 24) % 3
                    } else {
                        match col.biome.config().primary_stone {
                            Voxel::Granite | Voxel::Marble | Voxel::Chalk | Voxel::Limestone => 0, // Quartz
                            Voxel::Basalt | Voxel::Gabbro | Voxel::Slate | Voxel::Argillite => 1, // Flint
                            _ => 2, // Chert
                        }
                    };
                    Some(match kind {
                        0 => (
                            Voxel::BoulderQuartz,
                            Voxel::PebbleQuartz,
                            Voxel::PebbleQuartzCluster,
                        ),
                        1 => (
                            Voxel::BoulderFlint,
                            Voxel::PebbleFlint,
                            Voxel::PebbleFlintCluster,
                        ),
                        _ => (
                            Voxel::BoulderChert,
                            Voxel::PebbleChert,
                            Voxel::PebbleChertCluster,
                        ),
                    })
                } else {
                    None
                };

                let (cobble, mossy) = rock_to_cobble_pair(col.biome.config().primary_stone);
                let pick_mat = |dx: i32, dy: i32, dz: i32| -> Voxel {
                    if let Some((boulder_vox, _, _)) = mineral_triple {
                        return boulder_vox;
                    }
                    let mh = hash_tree_cell(rx + dx * 7 + dy * 13, rz + dz * 11, h ^ 0x44A1);
                    if hash_f32(mh) < moss_prob {
                        mossy
                    } else {
                        cobble
                    }
                };

                // Center core at ry + 1
                writer.set_leaf(rx, ry + 1, rz, pick_mat(0, 1, 0));

                // Top cap at ry + 2: 55% full block, 45% bottom slab
                let top_h = hash_f32(hash_tree_cell(rx, rz, h ^ 0x7102));
                if top_h < 0.55 {
                    writer.set_leaf(rx, ry + 2, rz, pick_mat(0, 2, 0));
                    if let Some((_, peb_single, peb_cluster)) = mineral_triple {
                        let ph = hash_tree_cell(rx, rz, h ^ 0x9EB1);
                        if hash_f32(ph) < 0.65 {
                            let (pvox, pshape) = if (ph >> 8).is_multiple_of(3) {
                                (peb_cluster, BlockShape::PebbleCluster)
                            } else {
                                (peb_single, BlockShape::Pebble)
                            };
                            writer.set_shaped(rx, ry + 3, rz, pvox, pshape, ((ph >> 12) & 3) as u8);
                        }
                    } else if moss_prob >= 0.35
                        && hash_f32(hash_tree_cell(rx, rz, h ^ 0x4055)) < 0.42
                    {
                        writer.set_shaped(rx, ry + 3, rz, Voxel::MossCarpet, BlockShape::Carpet, 0);
                    }
                } else {
                    writer.set_leaf_shaped(rx, ry + 2, rz, pick_mat(0, 2, 0), BlockShape::Slab, 0);
                }

                // Surrounding side stones (2..=4 cardinal + optional diagonal) with slope footing
                let dirs: [(i32, i32, u8); 4] = [
                    (1, 0, 1),  // +X: stair ascends toward -X (center)
                    (-1, 0, 0), // -X: stair ascends toward +X (center)
                    (0, 1, 3),  // +Z: stair ascends toward -Z (center)
                    (0, -1, 2), // -Z: stair ascends toward +Z (center)
                ];

                let mut placed_sides = 0;
                for (i, &(dx, dz, stair_orient)) in dirs.iter().enumerate() {
                    let sh = hash_tree_cell(rx + dx * 5, rz + dz * 9, h ^ (0x1234 + i as u32));
                    let roll = hash_f32(sh);
                    if roll > 0.68 && placed_sides >= 2 {
                        continue;
                    }
                    let ncol = column_at(rx + dx, rz + dz);
                    let dy_diff = ncol.terrain_height - ry;
                    if !(-1..=1).contains(&dy_diff) || ncol.water_level.is_some() {
                        continue;
                    }
                    // Anchor into 1-block slope dip so side stone never floats
                    if dy_diff == -1 {
                        writer.set_leaf(rx + dx, ry, rz + dz, pick_mat(dx, 0, dz));
                    }
                    let mat = pick_mat(dx, 1, dz);
                    let shape_roll = hash_f32(sh.rotate_right(9));
                    if shape_roll < 0.50 {
                        writer.set_leaf(rx + dx, ry + 1, rz + dz, mat);
                        if let Some((_, peb_single, peb_cluster)) = mineral_triple
                            && hash_f32(sh.rotate_right(17)) < 0.38 {
                                let (pvox, pshape) = if (sh >> 12).is_multiple_of(3) {
                                    (peb_cluster, BlockShape::PebbleCluster)
                                } else {
                                    (peb_single, BlockShape::Pebble)
                                };
                                writer.set_shaped(
                                    rx + dx,
                                    ry + 2,
                                    rz + dz,
                                    pvox,
                                    pshape,
                                    ((sh >> 14) & 3) as u8,
                                );
                            }
                    } else if shape_roll < 0.80 {
                        writer.set_leaf_shaped(rx + dx, ry + 1, rz + dz, mat, BlockShape::Slab, 0);
                    } else {
                        writer.set_leaf_shaped(
                            rx + dx,
                            ry + 1,
                            rz + dz,
                            mat,
                            BlockShape::Stair,
                            stair_orient,
                        );
                    }
                    placed_sides += 1;
                }

                // Occasional diagonal shoulder stone (creates wider 2x2 mossy/mineral clusters)
                if ((h >> 12) & 3) == 0 {
                    let diag_idx = ((h >> 14) & 3) as usize;
                    let (dx, dz) = [(1, 1), (1, -1), (-1, 1), (-1, -1)][diag_idx];
                    let ncol = column_at(rx + dx, rz + dz);
                    let dy_diff = ncol.terrain_height - ry;
                    if (-1..=1).contains(&dy_diff) && ncol.water_level.is_none() {
                        if dy_diff == -1 {
                            writer.set_leaf(rx + dx, ry, rz + dz, pick_mat(dx, 0, dz));
                        }
                        writer.set_leaf_shaped(
                            rx + dx,
                            ry + 1,
                            rz + dz,
                            pick_mat(dx, 1, dz),
                            BlockShape::Slab,
                            0,
                        );
                    }
                }

                // Scatter matching mineral pebbles around Quartz / Flint / Chert boulders
                if let Some((_, peb_single, peb_cluster)) = mineral_triple {
                    for pdz in -3..=3 {
                        for pdx in -3..=3 {
                            let d2 = pdx * pdx + pdz * pdz;
                            if !(2..=9).contains(&d2) {
                                continue;
                            }
                            let ph = hash_tree_cell(rx + pdx * 11, rz + pdz * 7, h ^ 0x9EB5);
                            if hash_f32(ph) < 0.24 {
                                let pcol = column_at(rx + pdx, rz + pdz);
                                if pcol.surface_shape == BlockShape::Full
                                    && pcol.water_level.is_none_or(|wl| pcol.terrain_height >= wl)
                                {
                                    let (pvox, pshape) = if (ph >> 8).is_multiple_of(3) {
                                        (peb_cluster, BlockShape::PebbleCluster)
                                    } else {
                                        (peb_single, BlockShape::Pebble)
                                    };
                                    writer.set_shaped(
                                        rx + pdx,
                                        pcol.terrain_height + 1,
                                        rz + pdz,
                                        pvox,
                                        pshape,
                                        ((ph >> 10) & 3) as u8,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // --- 2. Ground Leaf Bushes (1 Center Log + Surrounding Leaf Blocks) ---
    {
        const BUSH_CELL: i32 = 9;
        const BUSH_RADIUS: i32 = 2;

        let min_cx = (chunk_origin.x - BUSH_RADIUS).div_euclid(BUSH_CELL);
        let max_cx = (chunk_origin.x + CHUNK_SIZE as i32 - 1 + BUSH_RADIUS).div_euclid(BUSH_CELL);
        let min_cz = (chunk_origin.z - BUSH_RADIUS).div_euclid(BUSH_CELL);
        let max_cz = (chunk_origin.z + CHUNK_SIZE as i32 - 1 + BUSH_RADIUS).div_euclid(BUSH_CELL);

        for cz in min_cz..=max_cz {
            for cx in min_cx..=max_cx {
                let h = hash_tree_cell(cx, cz, seed ^ 0xB05B_7721);
                let jx = 2 + ((h & 15) as i32 % (BUSH_CELL - 4));
                let jz = 2 + (((h >> 4) & 15) as i32 % (BUSH_CELL - 4));
                let rx = cx * BUSH_CELL + jx;
                let rz = cz * BUSH_CELL + jz;

                let col = column_at(rx, rz);
                let ry = col.terrain_height;

                if chunk_min_y > ry + 4 || chunk_max_y < ry - 1 {
                    continue;
                }
                if col.surface_shape != BlockShape::Full
                    || col.water_level.is_some_and(|wl| ry <= wl)
                {
                    continue;
                }
                let Some((chance, log, leaves)) = select_bush_for_column(col, h) else {
                    continue;
                };
                if hash_f32(hash_tree_cell(cx, cz, seed ^ 0xB05B_AA55)) >= chance {
                    continue;
                }
                let Some(ground) = ground_at(rx, ry, rz, col) else {
                    continue;
                };
                if !ground.is_grass_soil() && !ground.is_fertile_soil() {
                    continue;
                }

                // Center single log block (only placed into empty air so trees/boulders stay intact)
                writer.set_leaf(rx, ry + 1, rz, log);
                // Top leaf cap directly above center log
                writer.set_leaf(rx, ry + 2, rz, leaves);

                // 4 cardinal leaf blocks wrapping the center log
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let ncol = column_at(rx + dx, rz + dz);
                    let dy_diff = ncol.terrain_height - ry;
                    if !(-1..=1).contains(&dy_diff) || ncol.water_level.is_some() {
                        continue;
                    }
                    let leaf_y = if dy_diff == -1 { ry } else { ry + 1 };
                    writer.set_leaf(rx + dx, leaf_y, rz + dz, leaves);
                    if dy_diff == -1 {
                        writer.set_leaf(rx + dx, ry + 1, rz + dz, leaves);
                    }
                }

                // Optional 1-2 diagonal corner leaves (~35% chance) for subtle fullness variety
                if ((h >> 12) % 10) < 4 {
                    for (i, &(dx, dz)) in [(1, 1), (1, -1), (-1, 1), (-1, -1)].iter().enumerate() {
                        let ch = hash_tree_cell(rx + dx * 3, rz + dz * 5, h ^ (0x5500 + i as u32));
                        if hash_f32(ch) < 0.35 {
                            let ncol = column_at(rx + dx, rz + dz);
                            let dy_diff = ncol.terrain_height - ry;
                            if (0..=1).contains(&dy_diff) && ncol.water_level.is_none() {
                                writer.set_leaf(rx + dx, ry + 1, rz + dz, leaves);
                            }
                        }
                    }
                }
            }
        }
    }

    // --- 3. Underwater Coralstone Boulder Clusters & Floor Patches ---
    {
        const CORAL_BOULDER_CELL: i32 = 11;
        const CORAL_BOULDER_RADIUS: i32 = 3;

        let min_cx = (chunk_origin.x - CORAL_BOULDER_RADIUS).div_euclid(CORAL_BOULDER_CELL);
        let max_cx = (chunk_origin.x + CHUNK_SIZE as i32 - 1 + CORAL_BOULDER_RADIUS)
            .div_euclid(CORAL_BOULDER_CELL);
        let min_cz = (chunk_origin.z - CORAL_BOULDER_RADIUS).div_euclid(CORAL_BOULDER_CELL);
        let max_cz = (chunk_origin.z + CHUNK_SIZE as i32 - 1 + CORAL_BOULDER_RADIUS)
            .div_euclid(CORAL_BOULDER_CELL);

        for cz in min_cz..=max_cz {
            for cx in min_cx..=max_cx {
                let h = hash_tree_cell(cx, cz, seed ^ 0xC057_B01D);
                let jx = 2 + ((h & 15) as i32 % (CORAL_BOULDER_CELL - 4));
                let jz = 2 + (((h >> 4) & 15) as i32 % (CORAL_BOULDER_CELL - 4));
                let rx = cx * CORAL_BOULDER_CELL + jx;
                let rz = cz * CORAL_BOULDER_CELL + jz;

                let col = column_at(rx, rz);
                let ry = col.terrain_height;

                if chunk_min_y > ry + 4 || chunk_max_y < ry - 2 {
                    continue;
                }
                let Some(wl) = col.water_level else {
                    continue;
                };
                if col.is_river || col.is_lake {
                    continue;
                }
                let depth = wl - ry;
                if !(3..=22).contains(&depth) {
                    continue;
                }
                let is_coral_zone = matches!(
                    col.biome,
                    BiomeType::CoralReefs | BiomeType::CoralIsland | BiomeType::WarmOcean
                ) || col.climate.temperature > -0.15;
                if !is_coral_zone {
                    continue;
                }

                let chance = if matches!(col.biome, BiomeType::CoralReefs | BiomeType::CoralIsland)
                {
                    0.46
                } else if col.biome == BiomeType::WarmOcean {
                    0.26
                } else {
                    0.16
                };
                if hash_f32(hash_tree_cell(cx, cz, seed ^ 0xC057_99A1)) >= chance {
                    continue;
                }

                // Organic Coralstone floor patch beneath the boulder cluster
                for pdz in -2..=2 {
                    for pdx in -2..=2 {
                        let d2 = pdx * pdx + pdz * pdz;
                        if d2 > 5 {
                            continue;
                        }
                        let ph = hash_tree_cell(rx + pdx * 7, rz + pdz * 11, h ^ 0x3319);
                        if d2 <= 2 || hash_f32(ph) < 0.62 {
                            let pcol = column_at(rx + pdx, rz + pdz);
                            if (pcol.terrain_height - ry).abs() <= 1 {
                                writer.replace_ground(
                                    rx + pdx,
                                    pcol.terrain_height,
                                    rz + pdz,
                                    Voxel::Coralstone,
                                );
                            }
                        }
                    }
                }

                // Stepped 2..=3 block tall Coralstone core
                let core_h = if depth >= 5 && ((h >> 8) & 1) == 0 {
                    3
                } else {
                    2
                };
                for dy in 1..core_h {
                    writer.set_rock(rx, ry + dy, rz, Voxel::Coralstone);
                }
                if ((h >> 9) & 3) == 0 {
                    writer.set_rock_shaped(
                        rx,
                        ry + core_h,
                        rz,
                        Voxel::Coralstone,
                        BlockShape::Slab,
                        0,
                    );
                } else {
                    writer.set_rock(rx, ry + core_h, rz, Voxel::Coralstone);
                }

                // Cardinal shoulder blocks with slope anchoring
                let dirs: [(i32, i32, u8); 4] = [(1, 0, 1), (-1, 0, 0), (0, 1, 3), (0, -1, 2)];
                let mut placed_sides = 0;
                for (i, &(dx, dz, stair_orient)) in dirs.iter().enumerate() {
                    let sh = hash_tree_cell(rx + dx * 5, rz + dz * 9, h ^ (0x2100 + i as u32));
                    if hash_f32(sh) > 0.72 && placed_sides >= 2 {
                        continue;
                    }
                    let ncol = column_at(rx + dx, rz + dz);
                    let dy_diff = ncol.terrain_height - ry;
                    if !(-1..=1).contains(&dy_diff) {
                        continue;
                    }
                    if dy_diff == -1 {
                        writer.set_rock(rx + dx, ry, rz + dz, Voxel::Coralstone);
                    }
                    let shape_roll = hash_f32(sh.rotate_right(9));
                    if shape_roll < 0.62 {
                        writer.set_rock(rx + dx, ry + 1, rz + dz, Voxel::Coralstone);
                        if core_h >= 3 && shape_roll < 0.24 {
                            writer.set_rock(rx + dx, ry + 2, rz + dz, Voxel::Coralstone);
                        }
                    } else if shape_roll < 0.84 {
                        writer.set_rock_shaped(
                            rx + dx,
                            ry + 1,
                            rz + dz,
                            Voxel::Coralstone,
                            BlockShape::Slab,
                            0,
                        );
                    } else {
                        writer.set_rock_shaped(
                            rx + dx,
                            ry + 1,
                            rz + dz,
                            Voxel::Coralstone,
                            BlockShape::Stair,
                            stair_orient,
                        );
                    }
                    placed_sides += 1;
                }

                // Optional diagonal coralstone shoulder
                if ((h >> 12) & 1) == 0 {
                    let diag_idx = ((h >> 13) & 3) as usize;
                    let (dx, dz) = [(1, 1), (1, -1), (-1, 1), (-1, -1)][diag_idx];
                    let ncol = column_at(rx + dx, rz + dz);
                    let dy_diff = ncol.terrain_height - ry;
                    if (-1..=1).contains(&dy_diff) {
                        if dy_diff == -1 {
                            writer.set_rock(rx + dx, ry, rz + dz, Voxel::Coralstone);
                        }
                        writer.set_rock(rx + dx, ry + 1, rz + dz, Voxel::Coralstone);
                    }
                }
            }
        }
    }

    // --- 4. Coastal & Shallow-Water Reefstone Boulders (much more common in water than on dry sand) ---
    {
        const COAST_BOULDER_CELL: i32 = 10;
        const COAST_BOULDER_RADIUS: i32 = 3;

        let min_cx = (chunk_origin.x - COAST_BOULDER_RADIUS).div_euclid(COAST_BOULDER_CELL);
        let max_cx = (chunk_origin.x + CHUNK_SIZE as i32 - 1 + COAST_BOULDER_RADIUS)
            .div_euclid(COAST_BOULDER_CELL);
        let min_cz = (chunk_origin.z - COAST_BOULDER_RADIUS).div_euclid(COAST_BOULDER_CELL);
        let max_cz = (chunk_origin.z + CHUNK_SIZE as i32 - 1 + COAST_BOULDER_RADIUS)
            .div_euclid(COAST_BOULDER_CELL);

        for cz in min_cz..=max_cz {
            for cx in min_cx..=max_cx {
                let h = hash_tree_cell(cx, cz, seed ^ 0x5EEF_B01D);
                let jx = 2 + ((h & 15) as i32 % (COAST_BOULDER_CELL - 4));
                let jz = 2 + (((h >> 4) & 15) as i32 % (COAST_BOULDER_CELL - 4));
                let rx = cx * COAST_BOULDER_CELL + jx;
                let rz = cz * COAST_BOULDER_CELL + jz;

                let col = column_at(rx, rz);
                let ry = col.terrain_height;

                if chunk_min_y > ry + 4 || chunk_max_y < ry - 2 {
                    continue;
                }
                if col.is_river || col.is_lake {
                    continue;
                }

                let is_dry_beach = (col.is_beach
                    || matches!(
                        col.biome,
                        BiomeType::SandyShores
                            | BiomeType::GravelBeach
                            | BiomeType::DuneBeach
                            | BiomeType::CraggyBeach
                            | BiomeType::TidePools
                            | BiomeType::TropicalIsland
                            | BiomeType::CoralIsland
                    ))
                    && col.water_level.is_none_or(|wl| ry >= wl)
                    && col.surface_shape == BlockShape::Full;
                let is_coastal_water = col.water_level.is_some_and(|wl| ry < wl && (wl - ry) <= 8);

                if !is_dry_beach && !is_coastal_water {
                    continue;
                }

                // Significantly rarer on dry beaches, abundant in coastal & shallow-to-submerged waters
                let chance = if is_dry_beach { 0.075 } else { 0.50 };
                if hash_f32(hash_tree_cell(cx, cz, seed ^ 0x5EEF_4419)) >= chance {
                    continue;
                }

                // For water boulders, verify proximity to coastline so deep open ocean stays clean
                if is_coastal_water {
                    let near_land = [
                        (14, 0),
                        (-14, 0),
                        (0, 14),
                        (0, -14),
                        (22, 0),
                        (-22, 0),
                        (0, 22),
                        (0, -22),
                    ]
                    .iter()
                    .any(|&(dx, dz)| {
                        let p = column_at(rx + dx, rz + dz);
                        p.water_level.is_none_or(|pwl| p.terrain_height >= pwl)
                    });
                    if !near_land {
                        continue;
                    }
                }

                // Anchor base solidly into seabed/sand so nothing ever floats
                writer.replace_ground(rx, ry, rz, Voxel::Reefstone);
                writer.set_rock(rx, ry + 1, rz, Voxel::Reefstone);

                // Sculpted top cap favoring slabs & stairs
                if !(h >> 8).is_multiple_of(3) {
                    let cap_roll = (h >> 10) % 10;
                    if cap_roll < 5 {
                        writer.set_rock_shaped(
                            rx,
                            ry + 2,
                            rz,
                            Voxel::Reefstone,
                            BlockShape::Slab,
                            0,
                        );
                    } else if cap_roll < 8 {
                        writer.set_rock_shaped(
                            rx,
                            ry + 2,
                            rz,
                            Voxel::Reefstone,
                            BlockShape::Stair,
                            ((h >> 14) & 3) as u8,
                        );
                    } else {
                        writer.set_rock(rx, ry + 2, rz, Voxel::Reefstone);
                    }
                }

                let dirs: [(i32, i32, u8); 4] = [(1, 0, 1), (-1, 0, 0), (0, 1, 3), (0, -1, 2)];
                for (i, &(dx, dz, stair_orient)) in dirs.iter().enumerate() {
                    let sh = hash_tree_cell(rx + dx * 5, rz + dz * 7, h ^ (0x3400 + i as u32));
                    if hash_f32(sh) > 0.68 {
                        continue;
                    }
                    let ncol = column_at(rx + dx, rz + dz);
                    let dy_diff = ncol.terrain_height - ry;
                    if !(-1..=0).contains(&dy_diff) || ncol.surface_shape != BlockShape::Full {
                        continue;
                    }
                    // Anchor all the way down to terrain_height
                    for fy in ncol.terrain_height..=ry {
                        writer.replace_ground(rx + dx, fy, rz + dz, Voxel::Reefstone);
                        writer.set_rock(rx + dx, fy, rz + dz, Voxel::Reefstone);
                    }
                    let shape_roll = hash_f32(sh.rotate_right(7));
                    if shape_roll < 0.42 {
                        writer.set_rock_shaped(
                            rx + dx,
                            ry + 1,
                            rz + dz,
                            Voxel::Reefstone,
                            BlockShape::Stair,
                            stair_orient,
                        );
                    } else if shape_roll < 0.78 {
                        writer.set_rock_shaped(
                            rx + dx,
                            ry + 1,
                            rz + dz,
                            Voxel::Reefstone,
                            BlockShape::Slab,
                            0,
                        );
                    } else {
                        writer.set_rock(rx + dx, ry + 1, rz + dz, Voxel::Reefstone);
                    }
                }

                // Scatter dark matching rubble around dry surface Reefstone boulders
                if is_dry_beach {
                    for rdz in -3..=3 {
                        for rdx in -3..=3 {
                            let d2 = rdx * rdx + rdz * rdz;
                            if !(2..=9).contains(&d2) {
                                continue;
                            }
                            let rh = hash_tree_cell(rx + rdx * 9, rz + rdz * 13, h ^ 0xD4B1);
                            if hash_f32(rh) < 0.24 {
                                let rcol = column_at(rx + rdx, rz + rdz);
                                if rcol.surface_shape == BlockShape::Full
                                    && rcol.water_level.is_none_or(|wl| rcol.terrain_height >= wl)
                                {
                                    let rubble = match (rh >> 8) % 3 {
                                        0 => Voxel::BasaltRubble,
                                        1 => Voxel::GabbroRubble,
                                        _ => Voxel::SlateRubble,
                                    };
                                    writer.set_shaped(
                                        rx + rdx,
                                        rcol.terrain_height + 1,
                                        rz + rdz,
                                        rubble,
                                        BlockShape::Rubble,
                                        ((rh >> 12) & 3) as u8,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // --- 5. Coastal Reefstone Sea Stacks & Jagged Spires (in coastal water, sculpted with stairs/slabs) ---
    {
        const SEA_STACK_CELL: i32 = 19;
        const SEA_STACK_RADIUS: i32 = 6;

        let min_cx = (chunk_origin.x - SEA_STACK_RADIUS).div_euclid(SEA_STACK_CELL);
        let max_cx =
            (chunk_origin.x + CHUNK_SIZE as i32 - 1 + SEA_STACK_RADIUS).div_euclid(SEA_STACK_CELL);
        let min_cz = (chunk_origin.z - SEA_STACK_RADIUS).div_euclid(SEA_STACK_CELL);
        let max_cz =
            (chunk_origin.z + CHUNK_SIZE as i32 - 1 + SEA_STACK_RADIUS).div_euclid(SEA_STACK_CELL);

        for cz in min_cz..=max_cz {
            for cx in min_cx..=max_cx {
                let h = hash_tree_cell(cx, cz, seed ^ 0x57AC_4EEF);
                let jx = 4 + ((h & 31) as i32 % (SEA_STACK_CELL - 8));
                let jz = 4 + (((h >> 5) & 31) as i32 % (SEA_STACK_CELL - 8));
                let rx = cx * SEA_STACK_CELL + jx;
                let rz = cz * SEA_STACK_CELL + jz;

                let col = column_at(rx, rz);
                let ry = col.terrain_height;

                if chunk_min_y > ry + 16 || chunk_max_y < ry - 3 {
                    continue;
                }
                if col.is_river || col.is_lake {
                    continue;
                }

                // Spawn strictly in coastal water (1..=10 blocks deep) so dry beaches stay clean
                let Some(water_level) = col.water_level else {
                    continue;
                };
                let water_depth = water_level - ry;
                if !(1..=10).contains(&water_depth) {
                    continue;
                }

                let macro_patch = crate::core::noise::gradient_noise_2d(
                    rx as f32 * 0.018,
                    rz as f32 * 0.018,
                    seed.wrapping_add(0x57AC_0099),
                );
                if macro_patch < -0.18 {
                    continue;
                }
                if hash_f32(hash_tree_cell(cx, cz, seed ^ 0x57AC_BB21)) >= 0.62 {
                    continue;
                }

                // Verify shoreline proximity (dry land nearby)
                let near_land = [
                    (14, 0),
                    (-14, 0),
                    (0, 14),
                    (0, -14),
                    (24, 0),
                    (-24, 0),
                    (0, 24),
                    (0, -24),
                ]
                .iter()
                .any(|&(dx, dz)| {
                    let p = column_at(rx + dx, rz + dz);
                    p.water_level.is_none_or(|pwl| p.terrain_height >= pwl)
                });
                if !near_land {
                    continue;
                }

                let angle = hash_f32(h.rotate_right(11)) * std::f32::consts::TAU;
                let lean_mag = 0.12 + hash_f32(h.rotate_right(17)) * 0.18;
                let lean_x = angle.cos() * lean_mag;
                let lean_z = angle.sin() * lean_mag;

                let above_water = 4 + ((h >> 10) % 6) as i32;
                let main_height = (water_depth + above_water).clamp(5, 14);
                let main_radius = 1.85 + hash_f32(h.rotate_right(13)) * 1.1;

                stamp_reefstone_spire(
                    &mut writer,
                    rx,
                    ry,
                    rz,
                    main_height,
                    main_radius,
                    lean_x,
                    lean_z,
                    angle,
                    h,
                    &column_at,
                );

                // 70% chance of 1..=2 smaller satellite spikes in the water nearby
                if ((h >> 14) % 100) < 70 {
                    let sat_count = if ((h >> 16) & 1) == 0 { 2 } else { 1 };
                    for s in 0..sat_count {
                        let sh = hash_tree_cell(rx + s * 17, rz - s * 13, h ^ 0x9911);
                        let sat_angle = angle
                            + std::f32::consts::FRAC_PI_2
                            + (s as f32) * 2.4
                            + hash_f32(sh) * 0.8;
                        let sat_dist = 3.2 + hash_f32(sh.rotate_right(7)) * 2.4;
                        let sx = rx + (sat_angle.cos() * sat_dist).round() as i32;
                        let sz = rz + (sat_angle.sin() * sat_dist).round() as i32;
                        let scol = column_at(sx, sz);
                        let sry = scol.terrain_height;
                        if scol.water_level.is_some_and(|wl| sry < wl) && (sry - ry).abs() <= 4 {
                            let sat_h =
                                (3 + ((sh >> 8) % 5) as i32 + (water_level - sry).max(0) / 2)
                                    .clamp(3, 8);
                            let sat_r = 1.15 + hash_f32(sh.rotate_right(15)) * 0.65;
                            stamp_reefstone_spire(
                                &mut writer,
                                sx,
                                sry,
                                sz,
                                sat_h,
                                sat_r,
                                lean_x * 1.1,
                                lean_z * 1.1,
                                angle,
                                sh,
                                &column_at,
                            );
                        }
                    }
                }
            }
        }
    }

    // --- 6. Fallen Forest Logs, Stumps & Top Mushrooms ---
    {
        const FALLEN_CELL: i32 = 16;
        const FALLEN_RADIUS: i32 = 5;

        let min_cx = (chunk_origin.x - FALLEN_RADIUS).div_euclid(FALLEN_CELL);
        let max_cx =
            (chunk_origin.x + CHUNK_SIZE as i32 - 1 + FALLEN_RADIUS).div_euclid(FALLEN_CELL);
        let min_cz = (chunk_origin.z - FALLEN_RADIUS).div_euclid(FALLEN_CELL);
        let max_cz =
            (chunk_origin.z + CHUNK_SIZE as i32 - 1 + FALLEN_RADIUS).div_euclid(FALLEN_CELL);

        for cz in min_cz..=max_cz {
            for cx in min_cx..=max_cx {
                let h = hash_tree_cell(cx, cz, seed ^ 0xFA11_E910);
                let jx = 3 + ((h & 15) as i32 % (FALLEN_CELL - 6));
                let jz = 3 + (((h >> 4) & 15) as i32 % (FALLEN_CELL - 6));
                let rx = cx * FALLEN_CELL + jx;
                let rz = cz * FALLEN_CELL + jz;

                let col = column_at(rx, rz);
                let ry = col.terrain_height;

                if chunk_min_y > ry + 3 || chunk_max_y < ry - 1 {
                    continue;
                }
                if col.is_beach
                    || col.is_cliff
                    || col.is_river
                    || col.surface_shape != BlockShape::Full
                    || col.water_level.is_some_and(|wl| ry <= wl)
                {
                    continue;
                }

                let Some((chance, log)) = select_fallen_log_for_biome(col.biome, h) else {
                    continue;
                };
                if hash_f32(hash_tree_cell(cx, cz, seed ^ 0xFA11_7723)) >= chance {
                    continue;
                }
                let Some(ground) = ground_at(rx, ry, rz, col) else {
                    continue;
                };
                if !ground.is_grass_soil() && !ground.is_fertile_soil() {
                    continue;
                }

                let along_x = ((h >> 8) & 1) == 0;
                let (dx, dz, axis_orient): (i32, i32, u8) =
                    if along_x { (1, 0, 1) } else { (0, 1, 2) };
                let log_len = 3 + ((h >> 9) % 3) as i32; // 3..=5 blocks

                // Verify flat, dry, full-block ground under every segment so the log never floats
                let mut valid_span = true;
                for i in 0..log_len {
                    let sx = rx + dx * i;
                    let sz = rz + dz * i;
                    let scol = column_at(sx, sz);
                    if scol.terrain_height != ry
                        || scol.surface_shape != BlockShape::Full
                        || scol.water_level.is_some_and(|wl| scol.terrain_height <= wl)
                    {
                        valid_span = false;
                        break;
                    }
                }
                if !valid_span {
                    continue;
                }

                // Optional upright stump separated by 1 air gap (~50% chance)
                if ((h >> 11) & 1) == 0 {
                    let stx = rx - dx * 2;
                    let stz = rz - dz * 2;
                    let stcol = column_at(stx, stz);
                    if stcol.terrain_height == ry
                        && stcol.surface_shape == BlockShape::Full
                        && stcol.water_level.is_none_or(|wl| stcol.terrain_height > wl)
                    {
                        writer.set_leaf(stx, ry + 1, stz, log);
                    }
                }

                // Place horizontal fallen trunk + optional mushrooms on top + mossy floor accents
                for i in 0..log_len {
                    let sx = rx + dx * i;
                    let sz = rz + dz * i;
                    writer.set_leaf_shaped(sx, ry + 1, sz, log, BlockShape::Full, axis_orient);

                    // Mushrooms or Moss Carpet sprouting on top of the decaying log
                    let mh = hash_tree_cell(sx * 7, sz * 11, h ^ 0x3058);
                    let mroll = hash_f32(mh);
                    if mroll < 0.24 {
                        let mushroom = match (mh >> 8) % 4 {
                            0 => Voxel::FungiAmanita,
                            1 => Voxel::FungiPorcini,
                            2 => Voxel::FungiRussula,
                            _ => Voxel::FungiShiitake,
                        };
                        writer.set_shaped(sx, ry + 2, sz, mushroom, BlockShape::CrossBillboard, 0);
                    } else if mroll < 0.54 {
                        writer.set_shaped(sx, ry + 2, sz, Voxel::MossCarpet, BlockShape::Carpet, 0);
                    }

                    // Occasional Forest Moss patch right next to the fallen log
                    if (mh >> 12).is_multiple_of(3) {
                        let side_sign = if ((mh >> 14) & 1) == 0 { 1 } else { -1 };
                        let mx = sx + dz * side_sign;
                        let mz = sz + dx * side_sign;
                        let mcol = column_at(mx, mz);
                        if (mcol.terrain_height - ry).abs() <= 1 {
                            writer.replace_ground(mx, mcol.terrain_height, mz, Voxel::MossForest);
                        }
                    }
                }
            }
        }
    }

    writer.wrote_any
}

/// Selects fallen log chance and biome-appropriate log species for forested biomes.
fn select_fallen_log_for_biome(biome: BiomeType, h: u32) -> Option<(f32, Voxel)> {
    let (chance, log) = match biome {
        BiomeType::VerdantWoods
        | BiomeType::AutumnForest
        | BiomeType::TemperateIsland
        | BiomeType::ShieldForest => {
            let l = match (h >> 14) % 4 {
                0 => Voxel::OakLog,
                1 => Voxel::BirchLog,
                2 => Voxel::PineLog,
                _ => Voxel::MapleLog,
            };
            (0.30, l)
        }
        BiomeType::MistyWoods | BiomeType::PineHighlands => {
            let l = if ((h >> 14) & 1) == 0 {
                Voxel::PineLog
            } else {
                Voxel::RedwoodLog
            };
            (0.34, l)
        }
        BiomeType::BleakTaiga | BiomeType::WintryForest | BiomeType::FrostPinewoods => {
            (0.28, Voxel::PineLog)
        }
        BiomeType::FertileValley | BiomeType::FlowerMeadow | BiomeType::LavenderValley => {
            let l = if ((h >> 14) & 1) == 0 {
                Voxel::OakLog
            } else {
                Voxel::BirchLog
            };
            (0.22, l)
        }
        BiomeType::SakuraGrove => (0.24, Voxel::CherryLog),
        BiomeType::DenseRainforest
        | BiomeType::BambooGrove
        | BiomeType::JungleWetlands
        | BiomeType::VolcanicJungle => (0.32, Voxel::MahoganyLog),
        BiomeType::WillowMarsh
        | BiomeType::MangroveSwamp
        | BiomeType::OrchidSwamp
        | BiomeType::SwampyShore => (0.34, Voxel::WillowLog),
        _ => return None,
    };
    Some((chance, log))
}

/// Stamps a column-solid, tapered, leaning Reefstone sea stack sculpted with stairs and slabs.
/// Every column runs continuously down into the seabed so zero voxels can ever float.
#[allow(clippy::too_many_arguments)]
fn stamp_reefstone_spire<F>(
    w: &mut ChunkWriter,
    rx: i32,
    ry: i32,
    rz: i32,
    height: i32,
    base_radius: f32,
    lean_x: f32,
    lean_z: f32,
    fin_angle: f32,
    h: u32,
    column_at: &F,
) where
    F: Fn(i32, i32) -> TerrainColumn,
{
    let cos_a = fin_angle.cos();
    let sin_a = fin_angle.sin();
    let reach =
        (base_radius * 1.45 + height as f32 * (lean_x.abs().max(lean_z.abs()))).ceil() as i32 + 1;

    for dz in -reach..=reach {
        for dx in -reach..=reach {
            let wx = rx + dx;
            let wz = rz + dz;
            let col_jitter = (hash_f32(hash_tree_cell(wx * 3, wz * 5, h ^ 0x77AA)) - 0.5) * 0.28;

            // Find the highest continuous layer `top_dy` this column reaches inside the tapered spire
            let mut top_dy: i32 = -1;
            let mut top_ellip: f32 = 2.0;
            for dy in 0..=height {
                let t = (dy as f32) / (height.max(1) as f32);
                let radius = (base_radius * (1.0 - t).powf(0.80)).max(0.45);
                let cx = rx as f32 + lean_x * (dy as f32);
                let cz = rz as f32 + lean_z * (dy as f32);
                let dxf = wx as f32 - cx;
                let dzf = wz as f32 - cz;

                let along = dxf * cos_a + dzf * sin_a;
                let across = -dxf * sin_a + dzf * cos_a;
                let ellip = (along / (radius * 1.30)).powi(2) + (across / (radius * 0.78)).powi(2);

                if ellip <= 1.0 + col_jitter {
                    top_dy = dy;
                    top_ellip = ellip;
                }
            }

            if top_dy < 0 {
                continue;
            }

            let col = column_at(wx, wz);
            let col_base_y = (col.terrain_height - 1).min(ry - 1);
            let col_top_y = ry + top_dy;

            // Fill solid Reefstone from below the seabed up to col_top_y - 1 (zero floating blocks!)
            for wy in col_base_y..col_top_y {
                if wy <= col.terrain_height {
                    w.replace_ground(wx, wy, wz, Voxel::Reefstone);
                }
                w.set_rock(wx, wy, wz, Voxel::Reefstone);
            }

            // Sculpt the top block of this column with a Slab or inward-facing Stair for jagged sea-stack flanks
            let cx_top = rx as f32 + lean_x * (top_dy as f32);
            let cz_top = rz as f32 + lean_z * (top_dy as f32);
            let to_cx = cx_top - wx as f32;
            let to_cz = cz_top - wz as f32;
            let stair_orient: u8 = if to_cx.abs() >= to_cz.abs() {
                if to_cx >= 0.0 { 0 } else { 1 }
            } else if to_cz >= 0.0 {
                2
            } else {
                3
            };

            if top_dy < height && top_ellip > 0.58 {
                if top_ellip > 0.82 {
                    w.set_rock_shaped(wx, col_top_y, wz, Voxel::Reefstone, BlockShape::Slab, 0);
                } else {
                    w.set_rock_shaped(
                        wx,
                        col_top_y,
                        wz,
                        Voxel::Reefstone,
                        BlockShape::Stair,
                        stair_orient,
                    );
                }
            } else {
                w.set_rock(wx, col_top_y, wz, Voxel::Reefstone);
            }
        }
    }

    // Guarantee a sharp pinnacle tip at the exact top of the lean axis
    let tip_x = (rx as f32 + lean_x * height as f32).round() as i32;
    let tip_z = (rz as f32 + lean_z * height as f32).round() as i32;
    let tip_col = column_at(tip_x, tip_z);
    for wy in (tip_col.terrain_height - 1)..=(ry + height) {
        if wy <= tip_col.terrain_height {
            w.replace_ground(tip_x, wy, tip_z, Voxel::Reefstone);
        }
        w.set_rock(tip_x, wy, tip_z, Voxel::Reefstone);
    }
    if ((h >> 19) & 1) == 0 {
        w.set_rock_shaped(
            tip_x,
            ry + height + 1,
            tip_z,
            Voxel::Reefstone,
            BlockShape::Slab,
            0,
        );
    }
}

/// Scatters green Wildgrass (and occasional Drygrass) on beach sand right around Palm tree bases.
fn stamp_under_palm_beach_grass<F, G>(
    w: &mut ChunkWriter,
    root_x: i32,
    root_z: i32,
    seed: u32,
    column_at: &F,
    ground_at: &G,
) where
    F: Fn(i32, i32) -> TerrainColumn,
    G: Fn(i32, i32, i32, TerrainColumn) -> Option<Voxel>,
{
    for dz in -3..=3 {
        for dx in -3..=3 {
            let d2 = dx * dx + dz * dz;
            if d2 == 0 || d2 > 10 {
                continue;
            }
            let wx = root_x + dx;
            let wz = root_z + dz;
            let gh = hash_tree_cell(wx, wz, seed ^ 0x9A13_67A5);
            let prob = if d2 <= 2 {
                0.42
            } else if d2 <= 5 {
                0.24
            } else {
                0.12
            };
            if hash_f32(gh) >= prob {
                continue;
            }
            let col = column_at(wx, wz);
            if col.surface_shape != BlockShape::Full
                || col.water_level.is_some_and(|wl| col.terrain_height <= wl)
            {
                continue;
            }
            let Some(ground) = ground_at(wx, col.terrain_height, wz, col) else {
                continue;
            };
            if !ground.is_sandy_soil() && !ground.is_fertile_soil() {
                continue;
            }
            let grass = if (gh >> 8).is_multiple_of(4) {
                Voxel::VegeDrygrass
            } else {
                Voxel::VegeWildgrass
            };
            w.set_shaped(
                wx,
                col.terrain_height + 1,
                wz,
                grass,
                BlockShape::CrossBillboard,
                0,
            );
        }
    }
}

/// Scatters natural Leaf Litter (1..=4 quadrants, random facing) beneath deciduous canopies.
fn stamp_under_canopy_leaf_litter<F, G>(
    w: &mut ChunkWriter,
    root_x: i32,
    root_z: i32,
    seed: u32,
    column_at: &F,
    ground_at: &G,
) where
    F: Fn(i32, i32) -> TerrainColumn,
    G: Fn(i32, i32, i32, TerrainColumn) -> Option<Voxel>,
{
    for dz in -3..=3 {
        for dx in -3..=3 {
            let d2 = dx * dx + dz * dz;
            if d2 == 0 || d2 > 10 {
                continue;
            }
            let wx = root_x + dx;
            let wz = root_z + dz;
            let lh = hash_tree_cell(wx, wz, seed ^ 0x1EAF_1177);
            let prob = if d2 <= 4 { 0.16 } else { 0.08 };
            if hash_f32(lh) >= prob {
                continue;
            }
            let lcol = column_at(wx, wz);
            if lcol.is_cliff
                || lcol.is_river
                || lcol.surface_shape != BlockShape::Full
                || lcol.water_level.is_some_and(|wl| lcol.terrain_height <= wl)
            {
                continue;
            }
            let Some(ground) = ground_at(wx, lcol.terrain_height, wz, lcol) else {
                continue;
            };
            if !ground.is_fertile_soil() {
                continue;
            }
            let facing = (lh & 0x03) as u8;
            let seg_idx = if d2 <= 2 {
                1 + (((lh >> 2) % 3) as u8) // 2..=4 quadrants right under trunk
            } else {
                ((lh >> 2) % 3) as u8 // 1..=3 quadrants near canopy edge
            };
            let orient = facing | (seg_idx << 2);
            w.set_leaf_shaped(
                wx,
                lcol.terrain_height + 1,
                wz,
                Voxel::VegeLeafLitter,
                BlockShape::FlatPad,
                orient,
            );
        }
    }
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
        TreeArchetype::GiantMushroom(_) => Voxel::FungiStem,
    }
}

fn stamp_tree(w: &mut ChunkWriter, rx: i32, ry: i32, rz: i32, archetype: TreeArchetype, h: u32) {
    // ~35% of eligible trees spawn as slender young variants using Centered Vertical Column log trunks
    let is_slender = (h >> 28).is_multiple_of(3);

    match archetype {
        TreeArchetype::Oak => {
            if is_slender {
                let height = 6 + ((h >> 16) % 2) as i32;
                for dy in 0..height {
                    w.set_log_shaped(rx, ry + dy, rz, Voxel::OakLog, BlockShape::Column, 0);
                }
                let bx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
                stamp_horizontal_branch(w, rx, ry + height - 2, rz, bx, 0, 1, Voxel::OakLog);
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
                // Horizontal centered-column branches (both X and Z axes) using OakLog
                let bx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
                let bz = if ((h >> 11) & 1) == 0 { 1 } else { -1 };
                stamp_horizontal_branch(w, rx, ry + height - 2, rz, bx, 0, 2, Voxel::OakLog);
                stamp_horizontal_branch(w, rx, ry + height - 3, rz, 0, bz, 2, Voxel::OakLog);
                stamp_leaf_cloud(
                    w,
                    rx,
                    ry + height + 1,
                    rz,
                    3.2,
                    2.3,
                    3.2,
                    Voxel::OakLeaves,
                    h,
                );
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
            let trunk_shape = if is_slender {
                BlockShape::Column
            } else {
                BlockShape::Full
            };
            for dy in 0..height {
                w.set_log_shaped(rx, ry + dy, rz, Voxel::BirchLog, trunk_shape, 0);
            }
            let bz = if ((h >> 11) & 1) == 0 { 1 } else { -1 };
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, 0, bz, 1, Voxel::BirchLog);
            stamp_leaf_cloud(
                w,
                rx,
                ry + height - 1,
                rz,
                2.6,
                2.4,
                2.6,
                Voxel::BirchLeaves,
                h,
            );
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
            let trunk_shape = if is_slender {
                BlockShape::Column
            } else {
                BlockShape::Full
            };
            for dy in 0..height {
                w.set_log_shaped(rx, ry + dy, rz, Voxel::MapleLog, trunk_shape, 0);
            }
            let dx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
            let dz = if ((h >> 11) & 1) == 0 { 1 } else { -1 };
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, dx, 0, 2, Voxel::MapleLog);
            stamp_horizontal_branch(w, rx, ry + height - 3, rz, 0, -dz, 2, Voxel::MapleLog);
            stamp_leaf_cloud(w, rx, ry + height + 1, rz, 3.0, 2.2, 3.0, leaves, h);
            stamp_leaf_cloud(
                w,
                rx + dx * 2,
                ry + height - 1,
                rz + dz,
                2.5,
                1.7,
                2.5,
                leaves,
                h ^ 0x222,
            );
            stamp_leaf_cloud(
                w,
                rx - dx,
                ry + height - 1,
                rz - dz * 2,
                2.4,
                1.7,
                2.4,
                leaves,
                h ^ 0x555,
            );
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
                    w.set_log_shaped(
                        tx,
                        ry + dy,
                        rz,
                        Voxel::CherryWood,
                        BlockShape::Stair,
                        inv_st,
                    );
                } else {
                    w.set_log(tx, ry + dy, rz, Voxel::CherryLog);
                }
            }
            // Horizontal spreading cherry boughs
            stamp_horizontal_branch(w, tx, ry + height - 1, rz, -lean_x, 0, 2, Voxel::CherryLog);
            stamp_horizontal_branch(w, tx, ry + height - 2, rz, 0, lean_z, 2, Voxel::CherryLog);
            stamp_leaf_cloud(
                w,
                tx + lean_x,
                ry + height + 1,
                rz,
                3.4,
                1.9,
                3.2,
                Voxel::CherryLeaves,
                h,
            );
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
            let trunk_shape = if is_slender {
                BlockShape::Column
            } else {
                stamp_buttress_roots(w, rx, ry, rz, Voxel::PineWood, h);
                BlockShape::Full
            };
            stamp_conifer(
                w,
                rx,
                ry,
                rz,
                height,
                3,
                trunk_shape,
                Voxel::PineLog,
                Voxel::PineLeaves,
                h,
            );
        }
        TreeArchetype::Yew => {
            let height = 8 + ((h >> 16) % 4) as i32;
            let trunk_shape = if is_slender {
                BlockShape::Column
            } else {
                BlockShape::Full
            };
            stamp_conifer(
                w,
                rx,
                ry,
                rz,
                height,
                3,
                trunk_shape,
                Voxel::YewLog,
                Voxel::YewLeaves,
                h,
            );
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
                    w.set_log_shaped(
                        tx,
                        ry + dy,
                        tz,
                        Voxel::AcaciaWood,
                        BlockShape::Stair,
                        inv_st,
                    );
                } else {
                    w.set_log(tx, ry + dy, tz, Voxel::AcaciaLog);
                }
            }
            let cy = ry + height;
            stamp_acacia_umbrella(w, tx, cy, tz, 3.6, Voxel::AcaciaLeaves, h);
            // Horizontal fork branch attached directly to the upper bent trunk `(tx, tz)`
            stamp_horizontal_branch(
                w,
                tx,
                ry + height - 1,
                tz,
                -lean_x,
                -lean_z,
                2,
                Voxel::AcaciaLog,
            );
            stamp_acacia_umbrella(
                w,
                tx - lean_x * 2,
                cy,
                tz - lean_z * 2,
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
                    w.set_log_shaped(tx, ry + dy, tz, Voxel::PalmLog, BlockShape::Column, 0);
                } else {
                    w.set_log(tx, ry + dy, tz, Voxel::PalmLog);
                }
            }

            let top_y = ry + height;
            // Slightly fuller rounded palm crown
            w.set_leaf(tx, top_y, tz, Voxel::PalmLeaves);
            w.set_leaf(tx, top_y + 1, tz, Voxel::PalmLeaves);

            // 4 graceful arching cardinal fronds: stacked upright/inverted Stairs + bottom tip Slab
            const DIRS: [(i32, i32, u8, u8); 4] = [
                (1, 0, 1, 4),  // +X: inward upright -X (1), outward inverted +X (4)
                (-1, 0, 0, 5), // -X: inward upright +X (0), outward inverted -X (5)
                (0, 1, 3, 6),  // +Z: inward upright -Z (3), outward inverted +Z (6)
                (0, -1, 2, 7), // -Z: inward upright +Z (2), outward inverted -Z (7)
            ];
            for (i, &(dx, dz, inward_orient, inv_outward_orient)) in DIRS.iter().enumerate() {
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
                w.set_leaf_shaped(
                    tx + dx * 3,
                    top_y,
                    tz + dz * 3,
                    Voxel::PalmLeaves,
                    BlockShape::Stair,
                    inward_orient,
                );
                w.set_leaf_shaped(
                    tx + dx * 3,
                    top_y - 1,
                    tz + dz * 3,
                    Voxel::PalmLeaves,
                    BlockShape::Stair,
                    inv_outward_orient,
                );
                w.set_leaf_shaped(
                    tx + dx * 4,
                    top_y - 1,
                    tz + dz * 4,
                    Voxel::PalmLeaves,
                    BlockShape::Stair,
                    inward_orient,
                );
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
            // 4 diagonal fronds (no stairs)
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
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, bx, 0, 2, Voxel::MahoganyLog);
            stamp_horizontal_branch(w, rx, ry + height - 3, rz, 0, bz, 2, Voxel::MahoganyLog);
            stamp_leaf_cloud(
                w,
                rx,
                ry + height + 1,
                rz,
                3.8,
                2.2,
                3.8,
                Voxel::MahoganyLeaves,
                h,
            );
            stamp_leaf_cloud(
                w,
                rx + bx * 2,
                ry + height - 1,
                rz,
                2.8,
                1.8,
                2.8,
                Voxel::MahoganyLeaves,
                h ^ 0x101,
            );
            stamp_leaf_cloud(
                w,
                rx,
                ry + height - 1,
                rz + bz * 2,
                2.8,
                1.8,
                2.8,
                Voxel::MahoganyLeaves,
                h ^ 0x202,
            );

            // Lateral Cocoa Pods attached directly to the Mahogany trunk
            const SIDES: [(i32, i32, u8); 4] = [(0, 1, 0), (-1, 0, 1), (0, -1, 2), (1, 0, 3)];
            for (i, &(dx, dz, orient)) in SIDES.iter().enumerate() {
                if (h >> (18 + i * 2)).is_multiple_of(3) {
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
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, bx, 0, 2, Voxel::WillowLog);
            stamp_horizontal_branch(w, rx, ry + height - 2, rz, 0, bz, 2, Voxel::WillowLog);
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
                w.set_log_shaped(
                    rx + dx,
                    ry,
                    rz + dz,
                    Voxel::MangroveLog,
                    BlockShape::Column,
                    0,
                );
                w.set_log_shaped(
                    rx + dx,
                    ry + 1,
                    rz + dz,
                    Voxel::MangroveLog,
                    BlockShape::Column,
                    0,
                );
                w.set_log_shaped(
                    rx + dx,
                    ry + 2,
                    rz + dz,
                    Voxel::MangroveWood,
                    BlockShape::Stair,
                    orient,
                );
            }
            for dy in 2..height {
                w.set_log(rx, ry + dy, rz, Voxel::MangroveLog);
            }
            stamp_leaf_cloud(
                w,
                rx,
                ry + height + 1,
                rz,
                3.4,
                2.2,
                3.4,
                Voxel::MangroveLeaves,
                h,
            );
        }
        TreeArchetype::Charred => {
            let height = 4 + ((h >> 16) % 4) as i32;
            let trunk_shape = if is_slender {
                BlockShape::Column
            } else {
                BlockShape::Full
            };
            for dy in 0..height {
                w.set_log_shaped(rx, ry + dy, rz, Voxel::TreeCharredLog, trunk_shape, 0);
            }
            if height >= 5 {
                let bx = if ((h >> 10) & 1) == 0 { 1 } else { -1 };
                stamp_horizontal_branch(
                    w,
                    rx,
                    ry + height - 2,
                    rz,
                    bx,
                    0,
                    1,
                    Voxel::TreeCharredLog,
                );
            }
        }
        TreeArchetype::GiantMushroom(cap_voxel) => {
            stamp_giant_mushroom(w, rx, ry, rz, cap_voxel, h);
        }
    }
}

/// Stamps a forest mushroom: ~70% compact small/medium mushrooms (often with `Column` stems)
/// and ~30% rare true Giant Mushrooms with hollow recessed `FungiInside` gill cavities.
fn stamp_giant_mushroom(w: &mut ChunkWriter, rx: i32, ry: i32, rz: i32, cap: Voxel, h: u32) {
    let is_giant = ((h >> 22) % 10) < 3; // 30% rare giant, 70% small/medium

    if !is_giant {
        // Small sculpted mushroom matching the plus-shaped silhouette:
        // 1..=2 block stem (Column or Full), 5-block plus-shaped cap base,
        // 4 cardinal Slab(0) + 1 Full center crown.
        let stem_h = 1 + ((h >> 16) & 1) as i32; // 1..=2 blocks
        let stem_shape = if ((h >> 17) % 5) < 3 {
            BlockShape::Column
        } else {
            BlockShape::Full
        };

        for dy in 0..stem_h {
            w.set_log_shaped(rx, ry + dy, rz, Voxel::FungiStem, stem_shape, 0);
        }

        let cap_y = ry + stem_h;

        // Plus-shaped cap base (5 blocks)
        w.set_leaf(rx, cap_y, rz, cap);
        w.set_leaf(rx + 1, cap_y, rz, cap);
        w.set_leaf(rx - 1, cap_y, rz, cap);
        w.set_leaf(rx, cap_y, rz + 1, cap);
        w.set_leaf(rx, cap_y, rz - 1, cap);

        // Upper crown
        w.set_leaf(rx, cap_y + 1, rz, cap);

        // Cardinal slabs
        for &(dx, dz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            w.set_leaf_shaped(rx + dx, cap_y + 1, rz + dz, cap, BlockShape::Slab, 0);
        }

        return;
    }

    // Rare Giant Mushroom (5..=8 blocks tall) with recessed hollow underside & `FungiInside` ceiling
    let height = 5 + ((h >> 16) % 4) as i32;
    let is_wide_umbrella = matches!(cap, Voxel::FungiPorciniCap | Voxel::FungiShiitakeCap)
        || (cap == Voxel::FungiVerdigrisCap && ((h >> 20) & 1) == 0);

    let mut tx = rx;
    let mut tz = rz;
    let bends = height >= 6 && ((h >> 9) & 1) == 0;
    let (lean_x, lean_z) = match (h >> 10) % 4 {
        0 => (1, 0),
        1 => (-1, 0),
        2 => (0, 1),
        _ => (0, -1),
    };
    let (up_st, inv_st) = curve_stair_orientations(lean_x, lean_z);

    for dy in 0..height {
        if bends && dy == 2 {
            w.set_log_shaped(tx, ry + dy, tz, Voxel::FungiStem, BlockShape::Stair, up_st);
            tx += lean_x;
            tz += lean_z;
            w.set_log_shaped(tx, ry + dy, tz, Voxel::FungiStem, BlockShape::Stair, inv_st);
        } else {
            w.set_log(tx, ry + dy, tz, Voxel::FungiStem);
        }
    }

    if is_wide_umbrella {
        // Wide umbrella cap with recessed hollow underside: drooping outer lip at `cap_y`,
        // `FungiInside` gill ceiling tucked up at `cap_y + 1`, and sculpted `cap` dome on top
        let cap_y = ry + height - 1;
        let radius = 3i32;

        // Outer drooping skirt lip at `cap_y` (interior stays OPEN/hollow!)
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let d2 = dx * dx + dz * dz;
                if (6..=10).contains(&d2) {
                    w.set_leaf_shaped(tx + dx, cap_y, tz + dz, cap, BlockShape::Slab, 1);
                }
            }
        }

        // Recessed `FungiInside` gill ceiling + outer `cap` rim at `cap_y + 1`
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let d2 = dx * dx + dz * dz;
                if d2 > 10 {
                    continue;
                }
                if d2 <= 5 {
                    w.set_leaf(tx + dx, cap_y + 1, tz + dz, Voxel::FungiInside);
                } else {
                    let inward_orient: u8 = if dx.abs() >= dz.abs() {
                        if dx < 0 { 0 } else { 1 }
                    } else if dz < 0 {
                        2
                    } else {
                        3
                    };
                    w.set_leaf_shaped(
                        tx + dx,
                        cap_y + 1,
                        tz + dz,
                        cap,
                        BlockShape::Stair,
                        inward_orient,
                    );
                }
            }
        }

        // Upper cap crust at `cap_y + 2`
        for dz in -2..=2 {
            for dx in -2..=2 {
                let d2 = dx * dx + dz * dz;
                if d2 > 5 {
                    continue;
                }
                if d2 <= 2 {
                    w.set_leaf(tx + dx, cap_y + 2, tz + dz, cap);
                } else {
                    w.set_leaf_shaped(tx + dx, cap_y + 2, tz + dz, cap, BlockShape::Slab, 0);
                }
            }
        }
    } else {
        // Classic domed bell cap with OPEN hollow lower skirt & recessed `FungiInside` ceiling
        let cap_base_y = ry + height - 2;
        let radius = 2i32;

        // Lower skirt (`cap_base_y`): ONLY the outer `cap` ring — interior stays OPEN air!
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let adx = dx.abs();
                let adz = dz.abs();
                if adx == radius && adz == radius {
                    continue;
                }
                if adx == radius || adz == radius {
                    w.set_leaf(tx + dx, cap_base_y, tz + dz, cap);
                }
            }
        }

        // Recessed upper interior (`cap_base_y + 1`): outer `cap` wall + `FungiInside` gill ceiling
        let mid_y = cap_base_y + 1;
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let adx = dx.abs();
                let adz = dz.abs();
                if adx == radius && adz == radius {
                    continue;
                }
                if adx == radius || adz == radius {
                    w.set_leaf(tx + dx, mid_y, tz + dz, cap);
                } else {
                    w.set_leaf(tx + dx, mid_y, tz + dz, Voxel::FungiInside);
                }
            }
        }

        // Top dome (`cap_base_y + 2`): full center 3x3 + beveled outer stairs
        let top_y = cap_base_y + 2;
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let d2 = dx * dx + dz * dz;
                if d2 > 5 {
                    continue;
                }
                if d2 <= 2 {
                    w.set_leaf(tx + dx, top_y, tz + dz, cap);
                } else {
                    let inward_orient: u8 = if dx.abs() >= dz.abs() {
                        if dx < 0 { 0 } else { 1 }
                    } else if dz < 0 {
                        2
                    } else {
                        3
                    };
                    w.set_leaf_shaped(
                        tx + dx,
                        top_y,
                        tz + dz,
                        cap,
                        BlockShape::Stair,
                        inward_orient,
                    );
                }
            }
        }
        w.set_leaf_shaped(tx, top_y + 1, tz, cap, BlockShape::Slab, 0);
    }
}

/// Scatters matching small mushrooms and a touch of Forest Moss around a Giant Mushroom base.
fn stamp_under_giant_mushroom<F, G>(
    w: &mut ChunkWriter,
    root_x: i32,
    root_z: i32,
    cap_voxel: Voxel,
    seed: u32,
    column_at: &F,
    ground_at: &G,
) where
    F: Fn(i32, i32) -> TerrainColumn,
    G: Fn(i32, i32, i32, TerrainColumn) -> Option<Voxel>,
{
    let small_shroom = match cap_voxel {
        Voxel::FungiAmanitaCap => Voxel::FungiAmanita,
        Voxel::FungiPorciniCap => Voxel::FungiPorcini,
        Voxel::FungiRussulaCap => Voxel::FungiRussula,
        Voxel::FungiShiitakeCap => Voxel::FungiShiitake,
        _ => Voxel::FungiVerdigris,
    };

    for dz in -2..=2 {
        for dx in -2..=2 {
            let d2 = dx * dx + dz * dz;
            if d2 == 0 || d2 > 5 {
                continue;
            }
            let wx = root_x + dx;
            let wz = root_z + dz;
            let mh = hash_tree_cell(wx, wz, seed ^ 0x3058_61A7);
            let col = column_at(wx, wz);
            if col.water_level.is_some_and(|wl| col.terrain_height <= wl) {
                continue;
            }
            if hash_f32(mh) < 0.45 {
                w.replace_ground(wx, col.terrain_height, wz, Voxel::MossForest);
            }
            if col.surface_shape == BlockShape::Full && hash_f32(mh.rotate_right(9)) < 0.22
                && ground_at(wx, col.terrain_height, wz, col).is_some() {
                    w.set_shaped(
                        wx,
                        col.terrain_height + 1,
                        wz,
                        small_shroom,
                        BlockShape::CrossBillboard,
                        0,
                    );
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
            let inward_orient: u8 = if dx.abs() >= dz.abs() {
                if dx < 0 { 0 } else { 1 }
            } else if dz < 0 {
                2
            } else {
                3
            };
            if d2 <= r2 {
                if d2 > r2 * 0.64 {
                    let ch = hash_tree_cell(cx + dx * 5, cz + dz * 7, h);
                    if d2 > r2 * 0.72 && (ch & 3) == 0 {
                        continue;
                    }
                    if d2 > r2 * 0.88 {
                        w.set_leaf_shaped(cx + dx, cy, cz + dz, leaves, BlockShape::Slab, 1);
                    } else {
                        w.set_leaf_shaped(
                            cx + dx,
                            cy,
                            cz + dz,
                            leaves,
                            BlockShape::Stair,
                            inward_orient + 4,
                        );
                    }
                } else {
                    w.set_leaf(cx + dx, cy, cz + dz, leaves);
                }
            }
            if d2 <= r2 * 0.48 {
                if d2 > r2 * 0.18 {
                    if d2 > r2 * 0.40 {
                        w.set_leaf_shaped(cx + dx, cy + 1, cz + dz, leaves, BlockShape::Slab, 0);
                    } else {
                        w.set_leaf_shaped(
                            cx + dx,
                            cy + 1,
                            cz + dz,
                            leaves,
                            BlockShape::Stair,
                            inward_orient,
                        );
                    }
                } else {
                    w.set_leaf(cx + dx, cy + 1, cz + dz, leaves);
                }
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
    for &(dx, dz, orient) in &[(1, 0, 1u8), (-1, 0, 0), (0, 1, 3), (0, -1, 2)] {
        w.set_leaf_shaped(
            rx + dx,
            ry + height,
            rz + dz,
            leaves,
            BlockShape::Stair,
            orient,
        );
    }

    let canopy_h = (height - canopy_start).max(1);
    let max_r = if height >= 14 { 4.0f32 } else { 3.0f32 };

    for dy in canopy_start..height {
        let progress = (dy - canopy_start) as f32 / canopy_h as f32;
        let wave_phase = (dy - canopy_start) % 3;
        let tier_wave = if wave_phase == 0 {
            1.0
        } else if wave_phase == 1 {
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
                if d2 > r2 * 0.50 {
                    let ch = hash_tree_cell(rx + dx * 3 + dy * 7, rz + dz * 5, h);
                    if d2 > r2 * 0.72 && (ch & 3) == 0 {
                        continue;
                    }
                    let inward_orient: u8 = if dx.abs() >= dz.abs() {
                        if dx < 0 { 0 } else { 1 }
                    } else if dz < 0 {
                        2
                    } else {
                        3
                    };
                    if d2 > r2 * 0.92 {
                        let slab_orient = if wave_phase == 0 { 1 } else { 0 };
                        w.set_leaf_shaped(
                            rx + dx,
                            ry + dy,
                            rz + dz,
                            leaves,
                            BlockShape::Slab,
                            slab_orient,
                        );
                    } else {
                        let stair_orient = if wave_phase == 0 && d2 > r2 * 0.78 {
                            inward_orient + 4
                        } else {
                            inward_orient
                        };
                        w.set_leaf_shaped(
                            rx + dx,
                            ry + dy,
                            rz + dz,
                            leaves,
                            BlockShape::Stair,
                            stair_orient,
                        );
                    }
                    continue;
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
                TreeArchetype::Oak => {
                    (6.5, 3.0, 4.0, Voxel::OakWood, Some(Voxel::OakLeaves), false)
                }
                TreeArchetype::Birch => (
                    7.5,
                    2.5,
                    4.2,
                    Voxel::BirchWood,
                    Some(Voxel::BirchLeaves),
                    false,
                ),
                TreeArchetype::Maple(l) => (7.0, 3.2, 4.0, Voxel::MapleWood, Some(l), false),
                TreeArchetype::Cherry => (
                    6.5,
                    3.4,
                    3.5,
                    Voxel::CherryWood,
                    Some(Voxel::CherryLeaves),
                    false,
                ),
                TreeArchetype::Pine => (
                    11.0,
                    3.0,
                    8.0,
                    Voxel::PineWood,
                    Some(Voxel::PineLeaves),
                    true,
                ),
                TreeArchetype::Yew => (9.0, 2.6, 6.0, Voxel::YewWood, Some(Voxel::YewLeaves), true),
                TreeArchetype::Redwood => (
                    16.0,
                    3.6,
                    11.5,
                    Voxel::RedwoodWood,
                    Some(Voxel::RedwoodLeaves),
                    true,
                ),
                TreeArchetype::Acacia => (
                    6.5,
                    3.6,
                    2.2,
                    Voxel::AcaciaWood,
                    Some(Voxel::AcaciaLeaves),
                    false,
                ),
                TreeArchetype::Palm => (
                    6.5,
                    2.8,
                    2.0,
                    Voxel::PalmWood,
                    Some(Voxel::PalmLeaves),
                    false,
                ),
                TreeArchetype::Mahogany => (
                    10.5,
                    3.8,
                    4.5,
                    Voxel::MahoganyWood,
                    Some(Voxel::MahoganyLeaves),
                    false,
                ),
                TreeArchetype::Willow => (
                    7.0,
                    3.8,
                    5.0,
                    Voxel::WillowWood,
                    Some(Voxel::WillowLeaves),
                    false,
                ),
                TreeArchetype::Mangrove => (
                    7.0,
                    3.4,
                    4.0,
                    Voxel::MangroveWood,
                    Some(Voxel::MangroveLeaves),
                    false,
                ),
                TreeArchetype::Charred => (5.0, 0.0, 0.0, Voxel::TreeCharredBark, None, false),
                TreeArchetype::GiantMushroom(cap) => {
                    if ((h >> 22) % 10) >= 3 {
                        continue;
                    }
                    (6.0, 2.8, 2.4, Voxel::FungiStem, Some(cap), false)
                }
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
