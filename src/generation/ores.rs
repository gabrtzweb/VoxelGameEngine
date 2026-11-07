use bevy::prelude::*;
use crate::world::{BlockShape, CHUNK_SIZE, Voxel};

/// Stub: procedural ore vein generation is temporarily bypassed to showcase pure terrain landforms.
#[inline]
pub fn populate_chunk_ores(_voxels: &mut [Voxel], _chunk_origin: IVec3, _seed: u32) {
    // Temporarily disabled: no-op stub
}

/// Stub: procedural geode generation is temporarily bypassed to showcase pure terrain landforms.
#[inline]
pub fn populate_chunk_geodes(_voxels: &mut [Voxel], _chunk_origin: IVec3, _seed: u32) {
    // Temporarily disabled: no-op stub
}

/// Populates crystal clusters on cave floors (orientation 0), ceilings (orientation 1),
/// and horizontal walls (orientations 2..=5) wherever solid crystal blocks border cave air.
pub fn populate_chunk_crystal_clusters(
    voxels: &mut [Voxel],
    chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
    chunk_origin: IVec3,
    seed: u32,
) {
    let crystal_cluster_for_block = |support: Voxel, glinted: bool| -> Option<Voxel> {
        match support {
            Voxel::CrystalAmethystBlock => Some(if glinted {
                Voxel::CrystalAmethystGlinted
            } else {
                Voxel::CrystalAmethystCluster
            }),
            Voxel::CrystalAquamarineBlock => Some(if glinted {
                Voxel::CrystalAquamarineGlinted
            } else {
                Voxel::CrystalAquamarineCluster
            }),
            Voxel::CrystalCitrineBlock => Some(if glinted {
                Voxel::CrystalCitrineGlinted
            } else {
                Voxel::CrystalCitrineCluster
            }),
            Voxel::CrystalFluoriteBlock => Some(if glinted {
                Voxel::CrystalFluoriteGlinted
            } else {
                Voxel::CrystalFluoriteCluster
            }),
            Voxel::CrystalPinkSaltBlock => Some(if glinted {
                Voxel::CrystalPinkSaltGlinted
            } else {
                Voxel::CrystalPinkSaltCluster
            }),
            Voxel::CrystalSeleniteBlock => Some(if glinted {
                Voxel::CrystalSeleniteGlinted
            } else {
                Voxel::CrystalSeleniteCluster
            }),
            _ => None,
        }
    };

    // Directions from the Air cell to the supporting neighbor block, paired with the resulting
    // CrossBillboard orientation:
    // (0, -1, 0) -> support is below (-Y), cluster stands on floor (orientation 0, growing +Y)
    // (0, +1, 0) -> support is above (+Y), cluster hangs from ceiling (orientation 1, growing -Y)
    // (0, 0, -1) -> support is North (-Z), cluster grows horizontally South (+Z, orientation 2)
    // (0, 0, +1) -> support is South (+Z), cluster grows horizontally North (-Z, orientation 3)
    // (-1, 0, 0) -> support is West (-X), cluster grows horizontally East (+X, orientation 4)
    // (+1, 0, 0) -> support is East (+X), cluster grows horizontally West (-X, orientation 5)
    const ATTACHMENTS: [(i32, i32, i32, u8); 6] = [
        (0, -1, 0, 0),
        (0, 1, 0, 1),
        (0, 0, -1, 2),
        (0, 0, 1, 3),
        (-1, 0, 0, 4),
        (1, 0, 0, 5),
    ];

    let mut placements: Vec<(usize, Voxel, u8)> = Vec::new();

    for y in 1..(CHUNK_SIZE - 1) {
        for z in 1..(CHUNK_SIZE - 1) {
            for x in 1..(CHUNK_SIZE - 1) {
                let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                if voxels[idx] != Voxel::Air {
                    continue;
                }

                let wx = chunk_origin.x + x as i32;
                let wy = chunk_origin.y + y as i32;
                let wz = chunk_origin.z + z as i32;
                let h = crate::core::noise::hash_2d(
                    wx ^ (wy.wrapping_mul(73)),
                    wz ^ (wy.wrapping_mul(131)),
                    seed.wrapping_add(919_331),
                );

                // ~28% chance to sprout a crystal cluster on an exposed crystal geode surface
                if (h % 100) >= 28 {
                    continue;
                }

                let start_dir = ((h >> 8) as usize) % 6;
                let glinted = ((h >> 16) % 100) < 35;

                for step in 0..6 {
                    let (dx, dy, dz, orient) = ATTACHMENTS[(start_dir + step) % 6];
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    let nz = (z as i32 + dz) as usize;
                    let n_idx = nx + nz * CHUNK_SIZE + ny * CHUNK_SIZE * CHUNK_SIZE;
                    let support_voxel = voxels[n_idx];

                    if let Some(cluster_voxel) = crystal_cluster_for_block(support_voxel, glinted) {
                        // Ensure the supporting block wasn't carved into a partial slab/stair
                        let is_shaped = chunk_shapes.iter().any(|&(s_idx, _, _)| s_idx == n_idx);
                        if !is_shaped {
                            placements.push((idx, cluster_voxel, orient));
                            break;
                        }
                    }
                }
            }
        }
    }

    for (idx, cluster_voxel, orient) in placements {
        voxels[idx] = cluster_voxel;
        chunk_shapes.push((idx, BlockShape::CrossBillboard, orient));
    }
}

/// Maps a supporting cave ceiling or floor block strictly to its matching speleothem type:
/// - `CaveCalcite` / `FracturedCaveCalcite` -> `SpeleothemCalcite`
/// - `CaveKarst` / `FracturedCaveKarst` -> `SpeleothemKarst`
/// - `FrostBlueIce` / `FrostIcePacked` / `FrostIce` -> `SpeleothemBlueIce`
/// - `CrystalPinkSaltBlock` -> `SpeleothemSaltStraw`
/// - `Obsidian` / `FracturedCaveObsidian` -> `ObsidianSpike`
#[inline]
pub fn speleothem_for_host_rock(host: Voxel) -> Option<Voxel> {
    match host {
        Voxel::CaveCalcite | Voxel::FracturedCaveCalcite => Some(Voxel::SpeleothemCalcite),
        Voxel::CaveKarst | Voxel::FracturedCaveKarst => Some(Voxel::SpeleothemKarst),
        Voxel::FrostBlueIce | Voxel::FrostIcePacked | Voxel::FrostIce => {
            Some(Voxel::SpeleothemBlueIce)
        }
        Voxel::CrystalPinkSaltBlock => Some(Voxel::SpeleothemSaltStraw),
        Voxel::Obsidian | Voxel::FracturedCaveObsidian => Some(Voxel::ObsidianSpike),
        _ => None,
    }
}

/// Populates vertical stalactites (hanging down from matching ceiling rock) and stalagmites
/// (standing up on matching floor rock) inside caves. Speleothems strictly require their
/// corresponding host block (e.g. Calcite only on/under Calcite, Karst only on/under Karst).
pub fn populate_chunk_speleothems(
    voxels: &mut [Voxel],
    chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
    chunk_origin: IVec3,
    seed: u32,
) {
    let mut placements: Vec<(usize, Voxel)> = Vec::new();

    for z in 1..(CHUNK_SIZE - 1) {
        for x in 1..(CHUNK_SIZE - 1) {
            let wx = chunk_origin.x + x as i32;
            let wz = chunk_origin.z + z as i32;

            // 1. Hanging stalactites (growing downward from ceiling y + 1 into cave air)
            let mut y = CHUNK_SIZE - 2;
            while y >= 1 {
                let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                let ceil_idx = x + z * CHUNK_SIZE + (y + 1) * CHUNK_SIZE * CHUNK_SIZE;
                if voxels[idx] == Voxel::Air
                    && let Some(speleothem) = speleothem_for_host_rock(voxels[ceil_idx])
                {
                    let is_ceil_shaped =
                        chunk_shapes.iter().any(|&(s_idx, _, _)| s_idx == ceil_idx);
                    if !is_ceil_shaped {
                        let wy = chunk_origin.y + y as i32;
                        let h = crate::core::noise::hash_2d(
                            wx ^ (wy.wrapping_mul(59)),
                            wz ^ (wy.wrapping_mul(107)),
                            seed.wrapping_add(512_987),
                        );
                        if (h % 100) < 22 {
                            let max_len = (((h >> 8) % 3) + 1) as usize;
                            let mut grown = 0;
                            while grown < max_len && y >= grown {
                                let cur_y = y - grown;
                                let cur_idx = x + z * CHUNK_SIZE + cur_y * CHUNK_SIZE * CHUNK_SIZE;
                                if voxels[cur_idx] != Voxel::Air {
                                    break;
                                }
                                placements.push((cur_idx, speleothem));
                                grown += 1;
                            }
                            if y >= grown {
                                y = y.saturating_sub(grown);
                                continue;
                            } else {
                                break;
                            }
                        }
                    }
                }
                if y == 0 {
                    break;
                }
                y -= 1;
            }

            // 2. Standing stalagmites (growing upward from floor y - 1 into cave air)
            let mut y = 1;
            while y < CHUNK_SIZE - 1 {
                let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                let floor_idx = x + z * CHUNK_SIZE + (y - 1) * CHUNK_SIZE * CHUNK_SIZE;
                if voxels[idx] == Voxel::Air
                    && let Some(speleothem) = speleothem_for_host_rock(voxels[floor_idx])
                {
                    let is_floor_shaped =
                        chunk_shapes.iter().any(|&(s_idx, _, _)| s_idx == floor_idx);
                    if !is_floor_shaped {
                        let wy = chunk_origin.y + y as i32;
                        let h = crate::core::noise::hash_2d(
                            wx ^ (wy.wrapping_mul(83)),
                            wz ^ (wy.wrapping_mul(139)),
                            seed.wrapping_add(618_233),
                        );
                        if (h % 100) < 18 {
                            let max_len = (((h >> 8) % 3) + 1) as usize;
                            let mut grown = 0;
                            while grown < max_len && (y + grown) < CHUNK_SIZE {
                                let cur_y = y + grown;
                                let cur_idx = x + z * CHUNK_SIZE + cur_y * CHUNK_SIZE * CHUNK_SIZE;
                                if voxels[cur_idx] != Voxel::Air
                                    || placements.iter().any(|&(p_idx, _)| p_idx == cur_idx)
                                {
                                    break;
                                }
                                placements.push((cur_idx, speleothem));
                                grown += 1;
                            }
                            y += grown.max(1);
                            continue;
                        }
                    }
                }
                y += 1;
            }
        }
    }

    for (idx, speleothem) in placements {
        voxels[idx] = speleothem;
        chunk_shapes.push((idx, BlockShape::Speleothem, 0));
    }
}
