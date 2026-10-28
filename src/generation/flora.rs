use bevy::prelude::*;
use crate::{
    generation::{biome::BiomeType, generator::TerrainColumn},
    world::{BlockShape, CHUNK_SIZE, Voxel},
};

pub struct FloraPlacement {
    pub bottom: Voxel,
    pub top: Option<Voxel>,
}

impl FloraPlacement {
    #[inline(always)]
    pub fn single(voxel: Voxel) -> Self {
        Self {
            bottom: voxel,
            top: None,
        }
    }

    #[inline(always)]
    pub fn tall(bottom: Voxel, top: Voxel) -> Self {
        Self {
            bottom,
            top: Some(top),
        }
    }
}

/// Deterministic hash combining coordinate integers with a salt.
#[inline(always)]
fn hash_coords(x: i32, y: i32, z: i32, seed: u32, salt: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x85EB_CA6B);
    h ^= (y as u32).wrapping_mul(0xC2B2_AE35);
    h ^= (z as u32).wrapping_mul(0x27D4_EB2D);
    h ^= seed.wrapping_mul(0x1656_67B1);
    h ^= salt;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h
}

/// Determines the flora to place on top of a surface ground block.
pub fn select_surface_flora(
    biome: BiomeType,
    ground: Voxel,
    is_beach: bool,
    hash: u32,
    can_place_tall: bool,
) -> Option<FloraPlacement> {
    let is_grass = ground.has_grass_turf();
    let is_dirt = matches!(
        ground,
        Voxel::LoamDirt
            | Voxel::PeatDirt
            | Voxel::SiltDirt
            | Voxel::LateriteDirt
            | Voxel::PermafrostDirt
            | Voxel::LoamCoarse
            | Voxel::PeatCoarse
            | Voxel::SiltCoarse
    );

    // 1. Coastal Beach / Dunes (Strictly NO cacti, NO flowers, NO dead bushes)
    if is_beach || biome == BiomeType::CoastalBeachReef {
        // Very sparse coastal grass: 2% density
        if (hash % 100) < 2 {
            let roll = (hash >> 8) % 3;
            return Some(match roll {
                0..=1 if can_place_tall => FloraPlacement::tall(
                    Voxel::VegeSeaOatsBottom,
                    Voxel::VegeSeaOatsTop,
                ),
                _ => FloraPlacement::single(Voxel::VegeDrygrass),
            });
        }
        return None;
    }

    // 2. Arid Desert & Badlands Flora (Only in true desert biome, never on beaches)
    if biome == BiomeType::Desert {
        // Sparse desert vegetation: 2% density
        if (hash % 100) < 2 {
            let roll = (hash >> 8) % 4;
            return Some(match roll {
                0 => FloraPlacement::single(Voxel::VegeDeadBush),
                1 => FloraPlacement::single(Voxel::VegeBarrelCactus),
                2 => FloraPlacement::single(Voxel::VegePricklyPearCactus),
                _ => FloraPlacement::single(Voxel::FlowerGlobeMallow),
            });
        }
        return None;
    }

    // From here, terrestrial flora requires viable soil/grass turf
    if !is_grass && !is_dirt {
        return None;
    }

    match biome {
        BiomeType::Plains => {
            // Meadow & Plains: 9% total density (80% wildgrass, 20% flowers)
            if (hash % 100) >= 9 {
                return None;
            }
            let roll = (hash >> 8) % 100;
            Some(match roll {
                // Wildgrasses (80%)
                0..=50 => FloraPlacement::single(Voxel::VegeWildgrass),
                51..=68 => FloraPlacement::single(Voxel::VegeBluffGrass),
                69..=79 if can_place_tall => FloraPlacement::tall(
                    Voxel::VegeTallWildgrassBottom,
                    Voxel::VegeTallWildgrassTop,
                ),
                69..=79 => FloraPlacement::single(Voxel::VegeWildgrass),

                // Meadow Flowers (20%)
                80..=82 => FloraPlacement::single(Voxel::FlowerDandelion),
                83..=85 => FloraPlacement::single(Voxel::FlowerPoppy),
                86..=87 => FloraPlacement::single(Voxel::FlowerCornflower),
                88..=89 => FloraPlacement::single(Voxel::FlowerOxeyeDaisy),
                90..=91 => FloraPlacement::single(Voxel::FlowerClover),
                92 => FloraPlacement::single(Voxel::FlowerIris),
                93 => FloraPlacement::single(Voxel::FlowerOrangeTulip),
                94 => FloraPlacement::single(Voxel::FlowerPinkTulip),
                95 => FloraPlacement::single(Voxel::FlowerWildflowers),
                96..=97 if can_place_tall => FloraPlacement::tall(
                    Voxel::FlowerSunflowerBottom,
                    Voxel::FlowerSunflowerTop,
                ),
                98..=99 if can_place_tall => FloraPlacement::tall(
                    Voxel::FlowerPeonyBottom,
                    Voxel::FlowerPeonyTop,
                ),
                _ => FloraPlacement::single(Voxel::VegeWildgrass),
            })
        }

        BiomeType::Forest => {
            // Dense Forest Understory: 8% density (85% ferns/bushes, 15% flowers)
            if (hash % 100) >= 8 {
                return None;
            }
            let roll = (hash >> 8) % 100;
            Some(match roll {
                // Ferns & Bushes (85%)
                0..=38 => FloraPlacement::single(Voxel::VegeFern),
                39..=58 => FloraPlacement::single(Voxel::VegeBracken),
                59..=72 => FloraPlacement::single(Voxel::VegeBush),
                73..=84 if can_place_tall => FloraPlacement::tall(
                    Voxel::VegeTallFernBottom,
                    Voxel::VegeTallFernTop,
                ),
                73..=84 => FloraPlacement::single(Voxel::VegeFern),

                // Forest Flowers & Shaded Understory (15%)
                85..=87 => FloraPlacement::single(Voxel::FlowerLilyOfTheValley),
                88..=90 => FloraPlacement::single(Voxel::FlowerPinkPetals),
                91..=93 => FloraPlacement::single(Voxel::FlowerWhitePetals),
                94 => FloraPlacement::single(Voxel::FlowerOrchid),
                95..=96 if can_place_tall => FloraPlacement::tall(
                    Voxel::FlowerFoxgloveBottom,
                    Voxel::FlowerFoxgloveTop,
                ),
                97..=98 if can_place_tall => FloraPlacement::tall(
                    Voxel::FlowerBlueHydrangeaBottom,
                    Voxel::FlowerBlueHydrangeaTop,
                ),
                99 if can_place_tall => FloraPlacement::tall(
                    Voxel::FlowerRoseBushBottom,
                    Voxel::FlowerRoseBushTop,
                ),
                _ => FloraPlacement::single(Voxel::VegeFern),
            })
        }

        BiomeType::Taiga => {
            // Coniferous Boreal Undergrowth: 6% density (85% moss/ferns, 15% flowers)
            if (hash % 100) >= 6 {
                return None;
            }
            let roll = (hash >> 8) % 100;
            Some(match roll {
                0..=45 => FloraPlacement::single(Voxel::VegeMossgrass),
                46..=65 => FloraPlacement::single(Voxel::VegeFern),
                66..=84 => FloraPlacement::single(Voxel::VegeBracken),
                85..=90 => FloraPlacement::single(Voxel::FlowerAllium),
                91..=95 => FloraPlacement::single(Voxel::FlowerLavender),
                _ if can_place_tall => FloraPlacement::tall(
                    Voxel::FlowerSnapdragonBottom,
                    Voxel::FlowerSnapdragonTop,
                ),
                _ => FloraPlacement::single(Voxel::VegeMossgrass),
            })
        }

        BiomeType::Highlands => {
            // Alpine Tundra & Highlands: 4% density
            if (hash % 100) >= 4 {
                return None;
            }
            let roll = (hash >> 8) % 100;
            Some(match roll {
                0..=55 => FloraPlacement::single(Voxel::VegeBluffGrass),
                56..=85 => FloraPlacement::single(Voxel::VegeMossgrass),
                86..=93 => FloraPlacement::single(Voxel::FlowerOxeyeDaisy),
                _ => FloraPlacement::single(Voxel::FlowerAllium),
            })
        }

        BiomeType::Savanna => {
            // Dry Grassland Savanna: 7% density
            if (hash % 100) >= 7 {
                return None;
            }
            let roll = (hash >> 8) % 100;
            Some(match roll {
                0..=55 => FloraPlacement::single(Voxel::VegeDrygrass),
                56..=75 if can_place_tall => FloraPlacement::tall(
                    Voxel::VegeTallWildgrassBottom,
                    Voxel::VegeTallWildgrassTop,
                ),
                76..=88 => FloraPlacement::single(Voxel::VegeRedShrub),
                _ => FloraPlacement::single(Voxel::FlowerMarigold),
            })
        }

        BiomeType::RiversLakes | BiomeType::MangroveWetland => {
            // Wetlands & Riverbanks: 8% density
            if (hash % 100) >= 8 {
                return None;
            }
            let roll = (hash >> 8) % 100;
            Some(match roll {
                0..=35 if can_place_tall => FloraPlacement::tall(
                    Voxel::VegeCattailBottom,
                    Voxel::VegeCattailTop,
                ),
                36..=55 => FloraPlacement::single(Voxel::VegeBush),
                56..=75 => FloraPlacement::single(Voxel::VegeMossgrass),
                76..=87 if can_place_tall => FloraPlacement::tall(
                    Voxel::VegeTallJungleFernBottom,
                    Voxel::VegeTallJungleFernTop,
                ),
                88..=93 => FloraPlacement::single(Voxel::FlowerIris),
                _ => FloraPlacement::single(Voxel::FlowerMarigold),
            })
        }

        _ => None,
    }
}

/// Populates procedural flora (wildgrass, ferns, bushes, flowers, succulents, cave flora)
/// for a single chunk's voxel grid.
pub fn populate_chunk_flora(
    voxels: &mut [Voxel],
    columns: &[TerrainColumn],
    chunk_origin: IVec3,
    seed: u32,
) {
    let chunk_min_y = chunk_origin.y;
    let chunk_max_y = chunk_min_y + CHUNK_SIZE as i32 - 1;

    let get_v = |voxels: &[Voxel], lx: usize, ly: usize, lz: usize| -> Voxel {
        voxels[lx + lz * CHUNK_SIZE + ly * CHUNK_SIZE * CHUNK_SIZE]
    };

    // 1. Surface flora passes
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let col = columns[x + z * CHUNK_SIZE];

            // Never spawn flora on slabs, stairs, or steep cliffs:
            if col.surface_shape != BlockShape::Full || col.is_cliff {
                continue;
            }

            let surface_y = col.terrain_height;

            if surface_y >= chunk_min_y && surface_y < chunk_max_y {
                let ly = (surface_y - chunk_min_y) as usize;
                let ground_voxel = get_v(voxels, x, ly, z);
                let air_above = get_v(voxels, x, ly + 1, z);

                if air_above == Voxel::Air {
                    let wx = chunk_origin.x + x as i32;
                    let wy = surface_y + 1;
                    let wz = chunk_origin.z + z as i32;
                    let h = hash_coords(wx, wy, wz, seed, 0x9E37_79B9);

                    let can_place_tall = ly + 2 < CHUNK_SIZE && get_v(voxels, x, ly + 2, z) == Voxel::Air;

                    if let Some(placed) = select_surface_flora(col.biome, ground_voxel, col.is_beach, h, can_place_tall) {
                        let idx = x + z * CHUNK_SIZE + (ly + 1) * CHUNK_SIZE * CHUNK_SIZE;
                        voxels[idx] = placed.bottom;

                        if let Some(top) = placed.top {
                            if can_place_tall {
                                let top_idx = x + z * CHUNK_SIZE + (ly + 2) * CHUNK_SIZE * CHUNK_SIZE;
                                voxels[top_idx] = top;
                            }
                        }
                    }
                }
            } else if surface_y + 1 == chunk_min_y {
                // Ground was top of chunk below, so ly = 0 of this chunk is surface air!
                let air_above = get_v(voxels, x, 0, z);
                if air_above == Voxel::Air {
                    let ground_voxel = if col.biome == BiomeType::Desert {
                        Voxel::SandDune
                    } else if col.is_beach {
                        Voxel::SandCommon
                    } else {
                        Voxel::LoamGrass
                    };
                    let wx = chunk_origin.x + x as i32;
                    let wy = chunk_min_y;
                    let wz = chunk_origin.z + z as i32;
                    let h = hash_coords(wx, wy, wz, seed, 0x9E37_79B9);

                    let can_place_tall = CHUNK_SIZE >= 2 && get_v(voxels, x, 1, z) == Voxel::Air;
                    if let Some(placed) = select_surface_flora(col.biome, ground_voxel, col.is_beach, h, can_place_tall) {
                        let idx = x + z * CHUNK_SIZE;
                        voxels[idx] = placed.bottom;

                        if let Some(top) = placed.top {
                            if can_place_tall {
                                let top_idx = x + z * CHUNK_SIZE + CHUNK_SIZE * CHUNK_SIZE;
                                voxels[top_idx] = top;
                            }
                        }
                    }
                }
            }

            // Water surface flora (e.g. Water Lily Pads, rare natural clusters ~1%)
            if let Some(water_y) = col.water_level {
                if water_y >= col.terrain_height && water_y >= chunk_min_y && water_y < chunk_max_y {
                    let ly = (water_y - chunk_min_y) as usize;
                    if get_v(voxels, x, ly, z) == Voxel::LiquidWater && get_v(voxels, x, ly + 1, z) == Voxel::Air {
                        let wx = chunk_origin.x + x as i32;
                        let wz = chunk_origin.z + z as i32;
                        let h = hash_coords(wx, water_y, wz, seed, 0x1234_5678);

                        if (col.biome == BiomeType::RiversLakes
                            || col.biome == BiomeType::MangroveWetland
                            || col.biome == BiomeType::Plains)
                            && (h % 100 == 0)
                        {
                            let idx = x + z * CHUNK_SIZE + (ly + 1) * CHUNK_SIZE * CHUNK_SIZE;
                            voxels[idx] = Voxel::VegeLilyPad;
                        }
                    }
                }
            }
        }
    }

    // 2. Subterranean cave flora passes (Ceiling hanging moss/vines & floor growths)
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let col = columns[x + z * CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                let world_y = chunk_min_y + y as i32;
                if world_y >= col.terrain_height {
                    continue;
                }

                let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                if voxels[idx] != Voxel::Air {
                    continue;
                }

                let wx = chunk_origin.x + x as i32;
                let wz = chunk_origin.z + z as i32;

                // Cave Ceiling check: hanging vines, moss, roots (~1% density)
                if y + 1 < CHUNK_SIZE {
                    let ceiling_voxel = get_v(voxels, x, y + 1, z);
                    if ceiling_voxel.is_solid_opaque() {
                        let h = hash_coords(wx, world_y, wz, seed, 0xCAFE_BABE);
                        if (h % 90) == 0 {
                            let roll = (h >> 6) % 5;
                            match roll {
                                0 | 1 => {
                                    if y >= 1 && get_v(voxels, x, y - 1, z) == Voxel::Air {
                                        voxels[idx] = Voxel::VegeCaveVinesPlant;
                                        let tip_idx = x + z * CHUNK_SIZE + (y - 1) * CHUNK_SIZE * CHUNK_SIZE;
                                        voxels[tip_idx] = Voxel::VegeCaveVines;
                                    } else {
                                        voxels[idx] = Voxel::VegeCaveVines;
                                    }
                                }
                                2 => voxels[idx] = Voxel::VegeHangingMoss,
                                3 => voxels[idx] = Voxel::VegeHangingRoots,
                                4 => voxels[idx] = Voxel::VegeVines,
                                _ => {}
                            }
                            continue;
                        }
                    }
                }

                // Cave Floor check: growths, lichen, hyssop (~1% density)
                if y >= 1 {
                    let floor_voxel = get_v(voxels, x, y - 1, z);
                    if floor_voxel.is_solid_opaque() && !floor_voxel.is_empty() {
                        let h = hash_coords(wx, world_y, wz, seed, 0xD00D_FEED);
                        if (h % 100) == 0 {
                            let roll = (h >> 7) % 4;
                            voxels[idx] = match roll {
                                0 => Voxel::VegeCaveGrowths,
                                1 => Voxel::VegeGrainyCaveGrowths,
                                2 => Voxel::VegeCaveHyssop,
                                _ => Voxel::VegeLichen,
                            };
                        }
                    }
                }
            }
        }
    }
}
