//! Phase 20: World Surface, Marine & Subterranean Feature Generation.
//!
//! Populates chunks with light, open, biome-authentic features:
//! - 20.1: Surface geological scatter (rubble matching host strata, pebbles/clusters, forest stick piles)
//! - 20.2: Herbaceous flora, flowers, petal carpets, bamboo, cacti, lily pads, and cave flora
//! - 20.3: Ancestral wild crops clustered in climate-appropriate patches
//! - 20.4: Marine meadows (seagrass, kelp, bull kelp) and warm coral reef ecosystems

use bevy::prelude::IVec3;

use crate::{
    generation::{biome::BiomeType, generator::TerrainColumn},
    world::{BlockShape, CHUNK_SIZE, Voxel},
};

#[inline(always)]
fn chunk_idx(x: usize, y: usize, z: usize) -> usize {
    x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
}

/// Fast deterministic 2D integer hash.
#[inline(always)]
fn hash2d(x: i32, z: i32, seed: u32) -> u32 {
    let mut h = seed
        .wrapping_add((x as u32).wrapping_mul(0x9E37_79B1))
        .wrapping_add((z as u32).wrapping_mul(0x85EB_CA77));
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    h
}

/// Fast deterministic 3D integer hash.
#[inline(always)]
fn hash3d(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = seed
        .wrapping_add((x as u32).wrapping_mul(0x9E37_79B1))
        .wrapping_add((y as u32).wrapping_mul(0xC2B2_AE3D))
        .wrapping_add((z as u32).wrapping_mul(0x85EB_CA77));
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    h
}

/// Converts a hash value into a uniform float in `[0.0, 1.0)`.
#[inline(always)]
fn hash_f32(h: u32) -> f32 {
    (h & 0x00FF_FFFF) as f32 / 16_777_216.0
}

/// Smooth 2D value noise in `[0.0, 1.0]` used for organic feature patches (flower meadows,
/// wild crop clusters, seagrass beds, and coral reef mounds).
#[inline]
fn patch_noise(x: i32, z: i32, scale: f32, seed: u32) -> f32 {
    let fx = x as f32 / scale;
    let fz = z as f32 / scale;
    let x0 = fx.floor() as i32;
    let z0 = fz.floor() as i32;
    let tx = fx - x0 as f32;
    let tz = fz - z0 as f32;
    let sx = tx * tx * (3.0 - 2.0 * tx);
    let sz = tz * tz * (3.0 - 2.0 * tz);

    let v00 = hash_f32(hash2d(x0, z0, seed));
    let v10 = hash_f32(hash2d(x0 + 1, z0, seed));
    let v01 = hash_f32(hash2d(x0, z0 + 1, seed));
    let v11 = hash_f32(hash2d(x0 + 1, z0 + 1, seed));

    let a = v00 + (v10 - v00) * sx;
    let b = v01 + (v11 - v01) * sx;
    a + (b - a) * sz
}

#[inline(always)]
fn set_voxel_in_chunk(
    voxels: &mut [Voxel],
    chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
    chunk_origin_y: i32,
    lx: usize,
    world_y: i32,
    lz: usize,
    voxel: Voxel,
    shape: BlockShape,
    orientation: u8,
) {
    let ly = world_y - chunk_origin_y;
    if ly < 0 || ly >= CHUNK_SIZE as i32 {
        return;
    }
    let idx = chunk_idx(lx, ly as usize, lz);
    if !voxels[idx].is_empty() && !voxels[idx].is_water() {
        return;
    }
    voxels[idx] = voxel;
    if shape != BlockShape::Full || orientation != 0 {
        chunk_shapes.push((idx, shape, orientation));
    }
}

#[inline(always)]
fn set_air_only_in_chunk(
    voxels: &mut [Voxel],
    chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
    chunk_origin_y: i32,
    lx: usize,
    world_y: i32,
    lz: usize,
    voxel: Voxel,
    shape: BlockShape,
    orientation: u8,
) {
    let ly = world_y - chunk_origin_y;
    if ly < 0 || ly >= CHUNK_SIZE as i32 {
        return;
    }
    let idx = chunk_idx(lx, ly as usize, lz);
    if !voxels[idx].is_empty() {
        return;
    }
    voxels[idx] = voxel;
    if shape != BlockShape::Full || orientation != 0 {
        chunk_shapes.push((idx, shape, orientation));
    }
}

/// Maps a host rock or soil block to its matching surface rubble variant.
fn matching_rubble_for(ground: Voxel) -> Voxel {
    match ground {
        Voxel::Andesite | Voxel::AndesiteGrass | Voxel::AndesiteSnow | Voxel::CobbledAndesite => {
            Voxel::AndesiteRubble
        }
        Voxel::Argillite | Voxel::ArgilliteGrass | Voxel::ArgilliteSnow => Voxel::ArgilliteRubble,
        Voxel::Basalt | Voxel::BasaltGrass | Voxel::BasaltSnow | Voxel::CobbledBasalt => {
            Voxel::BasaltRubble
        }
        Voxel::Chalk | Voxel::ChalkGrass | Voxel::ChalkSnow => Voxel::ChalkRubble,
        Voxel::Dolomite | Voxel::DolomiteGrass | Voxel::DolomiteSnow => Voxel::DolomiteRubble,
        Voxel::Gabbro | Voxel::CobbledGabbro => Voxel::GabbroRubble,
        Voxel::Granite | Voxel::GraniteGrass | Voxel::GraniteSnow | Voxel::CobbledGranite => {
            Voxel::GraniteRubble
        }
        Voxel::Limestone | Voxel::LimestoneGrass | Voxel::LimestoneSnow => Voxel::LimestoneRubble,
        Voxel::Marble | Voxel::CobbledMarble => Voxel::MarbleRubble,
        Voxel::SandstoneCommon | Voxel::SandCommon => Voxel::SandstoneRubble,
        Voxel::SandstoneDune | Voxel::SandDune => Voxel::SandstoneDuneRubble,
        Voxel::SandstoneRed | Voxel::SandRed => Voxel::SandstoneRedRubble,
        Voxel::SandstoneWhite | Voxel::SandWhite => Voxel::SandstoneWhiteRubble,
        Voxel::Slate | Voxel::CobbledSlate => Voxel::SlateRubble,
        _ => Voxel::AndesiteRubble,
    }
}

/// Selects a biome-authentic wild crop for a cluster patch.
/// Returns `(bottom_voxel, Option<top_voxel>)`.
fn select_wild_crop(
    biome: BiomeType,
    is_wet_bank: bool,
    patch_id: u32,
) -> (Voxel, Option<Voxel>) {
    if is_wet_bank {
        return match patch_id % 3 {
            0 => (Voxel::WildSugarCaneBottom, Some(Voxel::WildSugarCaneTop)),
            1 => (Voxel::WildRiceBottom, Some(Voxel::WildRiceTop)),
            _ => (Voxel::WildSugarCaneBottom, Some(Voxel::WildSugarCaneTop)),
        };
    }

    match biome {
        // Cold / Alpine / Taiga
        BiomeType::SiberianTaiga
        | BiomeType::WintryForest
        | BiomeType::PineHighlands
        | BiomeType::ColdShrubland
        | BiomeType::AlpineHighlands
        | BiomeType::EmeraldPeaks => match patch_id % 4 {
            0 => (Voxel::WildPotatoes, None),
            1 => (Voxel::WildBeetroots, None),
            2 => (Voxel::WildGarlic, None),
            _ => (Voxel::WildBarleyBottom, Some(Voxel::WildBarleyTop)),
        },
        // Warm / Tropical / Savanna / Oasis
        BiomeType::TropicalJungle
        | BiomeType::OrchidSwamp
        | BiomeType::FracturedSavanna
        | BiomeType::SavannaBadlands
        | BiomeType::DesertOasis
        | BiomeType::MirageIsles => match patch_id % 5 {
            0 => (Voxel::WildCornBottom, Some(Voxel::WildCornTop)),
            1 => (Voxel::WildTomatoes, None),
            2 => (Voxel::WildPepper, None),
            3 => (Voxel::WildCoffee, None),
            _ => (Voxel::WildMelon, None),
        },
        // Temperate Meadows / Forests / Valleys
        _ => match patch_id % 7 {
            0 => (Voxel::WildWheatBottom, Some(Voxel::WildWheatTop)),
            1 => (Voxel::WildBarleyBottom, Some(Voxel::WildBarleyTop)),
            2 => (Voxel::WildCarrots, None),
            3 => (Voxel::WildOnions, None),
            4 => (Voxel::WildCabbages, None),
            5 => (Voxel::WildLettuce, None),
            _ => (Voxel::WildPumpkin, None),
        },
    }
}

/// Returns true for forested or woodland biomes where forest-floor stick piles naturally occur.
fn is_woodland_biome(biome: BiomeType) -> bool {
    matches!(
        biome,
        BiomeType::YosemiteLowlands
            | BiomeType::CloudForest
            | BiomeType::SakuraGrove
            | BiomeType::BloomingValley
            | BiomeType::LavenderValley
            | BiomeType::PineHighlands
            | BiomeType::SiberianTaiga
            | BiomeType::WintryForest
            | BiomeType::TropicalJungle
            | BiomeType::OrchidSwamp
            | BiomeType::Shield
            | BiomeType::FracturedSavanna
    )
}

/// Returns true for rocky, mountain, or canyon biomes with increased surface rubble.
fn is_rocky_biome(biome: BiomeType) -> bool {
    matches!(
        biome,
        BiomeType::AlpineHighlands
            | BiomeType::EmeraldPeaks
            | BiomeType::VolcanicPeaks
            | BiomeType::PaintedMountains
            | BiomeType::RockyMountains
            | BiomeType::ScarletMountains
            | BiomeType::BryceCanyon
            | BiomeType::AmethystCanyon
            | BiomeType::BasaltCliffs
            | BiomeType::GraniteCliffs
            | BiomeType::WhiteCliffs
            | BiomeType::YosemiteCliffs
            | BiomeType::Caldera
            | BiomeType::DesertSpires
            | BiomeType::AshenSavanna
    )
}

/// Populates a generated chunk with surface scatter, flora, wild crops, marine flora, and cave flora.
pub fn populate_chunk_flora<F>(
    voxels: &mut [Voxel],
    chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
    columns: &[TerrainColumn; CHUNK_SIZE * CHUNK_SIZE],
    chunk_origin: IVec3,
    seed: u32,
    voxel_at: F,
) where
    F: Fn(i32, i32, i32, TerrainColumn) -> Voxel,
{
    let chunk_min_y = chunk_origin.y;
    let chunk_max_y = chunk_origin.y + CHUNK_SIZE as i32 - 1;

    // 1. Surface & Marine Features (evaluated per-column deterministically for vertical chunk seamlessness)
    for lz in 0..CHUNK_SIZE {
        let wz = chunk_origin.z + lz as i32;
        for lx in 0..CHUNK_SIZE {
            let wx = chunk_origin.x + lx as i32;
            let col = columns[lz * CHUNK_SIZE + lx];
            let surface_y = col.terrain_height;

            // Skip columns whose entire surface/marine/bamboo/cactus span cannot touch this vertical chunk
            let water_top = col.water_level.unwrap_or(surface_y);
            let max_feature_y = surface_y.max(water_top) + 7;
            if chunk_min_y > max_feature_y || chunk_max_y < surface_y - 1 {
                continue;
            }

            // Never place surface features on sculpted terrain slopes (slabs/stairs)
            if col.surface_shape != BlockShape::Full {
                continue;
            }

            let ground = voxel_at(wx, surface_y, wz, col);
            if ground.is_empty() || ground.is_fluid() || !ground.is_solid_opaque() {
                continue;
            }

            // --- 20.4: MARINE & AQUATIC SCATTER ---
            if let Some(wl) = col.water_level {
                if surface_y < wl {
                    let depth = wl - surface_y;
                    populate_marine_column(
                        voxels,
                        chunk_shapes,
                        chunk_min_y,
                        lx,
                        lz,
                        wx,
                        wz,
                        surface_y,
                        wl,
                        depth,
                        ground,
                        col,
                        seed,
                    );
                    continue;
                }
            }

            // --- SURFACE FEATURES (above water) ---
            let h = hash2d(wx, wz, seed ^ 0x4F1B_BCDC);
            let roll = hash_f32(h);
            let orient = ((wx.wrapping_mul(31) ^ wz.wrapping_mul(17)) & 3).unsigned_abs() as u8;
            let place_y = surface_y + 1;

            // Check if the air slot immediately above surface is actually air (skip cave openings)
            let above_ground = voxel_at(wx, place_y, wz, col);
            if !above_ground.is_empty() {
                continue;
            }

            // Skip if trees, buttress roots, or under-canopy Leaf Litter already occupied `place_y`
            // (prevents 2-block-tall flora top halves from spawning floating at `place_y + 1`)
            let place_ly = place_y - chunk_min_y;
            if place_ly >= 0 && place_ly < CHUNK_SIZE as i32 {
                let idx = lx + lz * CHUNK_SIZE + place_ly as usize * CHUNK_SIZE * CHUNK_SIZE;
                if !voxels[idx].is_empty() {
                    continue;
                }
            }

            // --- 20.1: GEOLOGICAL SCATTER (Rubble, Pebbles, Stick Piles) ---
            // Surface Rubble on cliffs, rocky biomes, bare stone, and light scatter across plains
            let rubble_chance = surface_rubble_chance(col, ground);
            if roll < rubble_chance {
                let rubble = matching_rubble_for(ground);
                set_air_only_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    place_y,
                    lz,
                    rubble,
                    BlockShape::Rubble,
                    orient,
                );
                continue;
            }

            // Loose Pebbles & Clusters on beaches, gravel beaches, riverbanks, and sparse plains
            let pebble_chance = surface_pebble_chance(col);
            let h_peb = hash2d(wx, wz, seed ^ 0x1337_BEEF);
            if hash_f32(h_peb) < pebble_chance {
                let is_cluster = ((h_peb >> 8) % 4) == 0;
                let pebble = match (h_peb >> 12) % 3 {
                    0 => {
                        if is_cluster {
                            Voxel::PebbleChertCluster
                        } else {
                            Voxel::PebbleChert
                        }
                    }
                    1 => {
                        if is_cluster {
                            Voxel::PebbleFlintCluster
                        } else {
                            Voxel::PebbleFlint
                        }
                    }
                    _ => {
                        if is_cluster {
                            Voxel::PebbleQuartzCluster
                        } else {
                            Voxel::PebbleQuartz
                        }
                    }
                };
                set_air_only_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    place_y,
                    lz,
                    pebble,
                    pebble.default_shape(),
                    orient,
                );
                continue;
            }

            // Forest Floor Stick Piles & Leaf Litter patches in woodland biomes
            if is_woodland_biome(col.biome) && ground.is_fertile_soil() {
                let h_stick = hash2d(wx, wz, seed ^ 0x571C_C0DE);
                if hash_f32(h_stick) < 0.0055 {
                    set_air_only_in_chunk(
                        voxels,
                        chunk_shapes,
                        chunk_min_y,
                        lx,
                        place_y,
                        lz,
                        Voxel::StickPile,
                        BlockShape::StickPile,
                        orient,
                    );
                    continue;
                }

                // Clustered Minecraft-style Leaf Litter patches on temperate/deciduous woodland floors
                if matches!(
                    col.biome,
                    BiomeType::YosemiteLowlands
                        | BiomeType::CloudForest
                        | BiomeType::BloomingValley
                        | BiomeType::LavenderValley
                        | BiomeType::OrchidSwamp
                        | BiomeType::Shield
                        | BiomeType::TropicalJungle
                ) {
                    let litter_patch = patch_noise(wx, wz, 14.0, seed ^ 0x1EAF_F00D);
                    if litter_patch > 0.56 {
                        let h_litter = hash2d(wx, wz, seed ^ 0x1EAF_5EED);
                        if hash_f32(h_litter) < 0.22 {
                            let facing = (h_litter & 0x03) as u8;
                            let seg_idx: u8 = if litter_patch > 0.78 {
                                3
                            } else if litter_patch > 0.69 {
                                2
                            } else if litter_patch > 0.62 {
                                1
                            } else {
                                0
                            };
                            set_air_only_in_chunk(
                                voxels,
                                chunk_shapes,
                                chunk_min_y,
                                lx,
                                place_y,
                                lz,
                                Voxel::VegeLeafLitter,
                                BlockShape::FlatPad,
                                facing | (seg_idx << 2),
                            );
                            continue;
                        }
                    }
                }
            }

            // --- 20.3: ANCESTRAL WILD CROPS (clustered rare patches) ---
            // Never spawn dryland wild crops (like pumpkins/wheat) in the middle of beaches or gravel beaches!
            if let Some((bot, top_opt)) = sample_wild_crop_column(wx, wz, ground, col, seed) {
                let shape = bot.default_shape();
                set_air_only_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    place_y,
                    lz,
                    bot,
                    shape,
                    0,
                );
                if let Some(top) = top_opt {
                    set_air_only_in_chunk(
                        voxels,
                        chunk_shapes,
                        chunk_min_y,
                        lx,
                        place_y + 1,
                        lz,
                        top,
                        BlockShape::CrossBillboard,
                        0,
                    );
                }
                continue;
            }

            // --- 20.2: HERBACEOUS FLORA, FLOWERS, CACTI & BAMBOO (light & open density) ---
            populate_surface_flora_column(
                voxels,
                chunk_shapes,
                chunk_min_y,
                lx,
                lz,
                wx,
                wz,
                place_y,
                ground,
                col,
                seed,
            );
        }
    }

    // 2. Subterranean Cave Flora & Hanging Growths (light density)
    populate_cave_flora(voxels, chunk_shapes, columns, chunk_origin, seed);
}

#[allow(clippy::too_many_arguments)]
fn populate_marine_column(
    voxels: &mut [Voxel],
    chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
    chunk_min_y: i32,
    lx: usize,
    lz: usize,
    wx: i32,
    wz: i32,
    surface_y: i32,
    water_level: i32,
    depth: i32,
    ground: Voxel,
    col: TerrainColumn,
    seed: u32,
) {
    let h = hash2d(wx, wz, seed ^ 0xA0AA_71C5);
    let r = hash_f32(h);
    let orient = ((wx.wrapping_mul(31) ^ wz.wrapping_mul(17)) & 3).unsigned_abs() as u8;

    // Floating Lily Pads on shallow calm swamp / lake / oasis water
    if (col.is_lake || col.biome == BiomeType::OrchidSwamp || col.biome == BiomeType::DesertOasis)
        && depth <= 6
    {
        let lily_patch = patch_noise(wx, wz, 14.0, seed ^ 0x1117_FAD0);
        if lily_patch > 0.65 && r < 0.055 {
            set_air_only_in_chunk(
                voxels,
                chunk_shapes,
                chunk_min_y,
                lx,
                water_level + 1,
                lz,
                Voxel::VegeLilyPad,
                BlockShape::FlatPad,
                0,
            );
        }
    }

    if depth < 2 {
        return;
    }

    let floor_y = surface_y + 1;

    // Underwater Rubble scattered across ocean floors, riverbeds, and lakebeds
    let h_rub = hash2d(wx, wz, seed ^ 0x50B3_70B1);
    if hash_f32(h_rub) < 0.025 {
        let rubble = matching_rubble_for(ground);
        set_voxel_in_chunk(
            voxels,
            chunk_shapes,
            chunk_min_y,
            lx,
            floor_y,
            lz,
            rubble,
            BlockShape::Rubble,
            orient,
        );
        return;
    }

    // Vibrant Coral Reefs (dense in CoralReef, coastal/warm ocean clusters in temperate/warm waters)
    let is_coral_zone = matches!(col.biome, BiomeType::CoralReef | BiomeType::DeepWarmOcean)
        || (!col.is_river && col.climate.temperature > -0.15);
    if is_coral_zone && depth <= 22 {
        let reef_patch = patch_noise(wx, wz, 18.0, seed ^ 0xC07A_1FEF);
        let (threshold, chance) = if col.biome == BiomeType::CoralReef {
            (0.35, 0.24)
        } else if col.biome == BiomeType::DeepWarmOcean {
            (0.50, 0.16)
        } else {
            (0.58, 0.14)
        };
        if reef_patch > threshold && r < chance {
            let variant = (h >> 8) % 10;
            let (coral_block, coral_fan) = match variant {
                0 => (Voxel::BrainCoral, Voxel::BrainCoralFan),
                1 => (Voxel::BubbleCoral, Voxel::BubbleCoralFan),
                2 => (Voxel::FireCoral, Voxel::FireCoralFan),
                3 => (Voxel::HeartCoral, Voxel::HeartCoralFan),
                4 => (Voxel::HornCoral, Voxel::HornCoralFan),
                5 => (Voxel::LeafCoral, Voxel::LeafCoralFan),
                6 => (Voxel::ReedCoral, Voxel::ReedCoralFan),
                7 => (Voxel::StaghornCoral, Voxel::StaghornCoralFan),
                8 => (Voxel::SunCoral, Voxel::SunCoralFan),
                _ => (Voxel::TubeCoral, Voxel::TubeCoralFan),
            };
            if ((h >> 16) % 3) == 0 && depth >= 3 {
                // Small coral mound block + fan on top
                set_voxel_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    floor_y,
                    lz,
                    coral_block,
                    BlockShape::Full,
                    0,
                );
                set_voxel_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    floor_y + 1,
                    lz,
                    coral_fan,
                    BlockShape::CrossBillboard,
                    0,
                );
            } else {
                set_voxel_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    floor_y,
                    lz,
                    coral_fan,
                    BlockShape::CrossBillboard,
                    0,
                );
            }
            return;
        }
    }

    // Kelp & Bull Kelp underwater forests in any deep coastal water or ocean (depth >= 4)
    if depth >= 4 && !col.is_river {
        let kelp_patch = patch_noise(wx, wz, 18.0, seed ^ 0x0E1F_4422);
        if kelp_patch > 0.50 && r < 0.11 {
            let is_bull = col.biome == BiomeType::ColdOcean
                || col.climate.temperature < 0.1
                || ((h >> 12) & 1) == 0;
            let max_h = (depth - 2).clamp(2, 9);
            let stalk_h = 2 + ((h >> 16) as i32 % (max_h - 1).max(1));
            let (plant_vox, tip_vox) = if is_bull {
                (Voxel::BullKelpPlant, Voxel::BullKelp)
            } else {
                (Voxel::KelpPlant, Voxel::Kelp)
            };
            for dy in 0..stalk_h {
                let v = if dy + 1 == stalk_h { tip_vox } else { plant_vox };
                set_voxel_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    floor_y + dy,
                    lz,
                    v,
                    BlockShape::CrossBillboard,
                    0,
                );
            }
            return;
        }
    }

    // Lush Seagrass & Red Algae underwater meadows
    let sg_patch = patch_noise(wx, wz, 14.0, seed ^ 0x5EA6_7A55);
    if sg_patch > 0.35 && r < 0.22 {
        let vox = if ((h >> 10) % 6) == 0 {
            Voxel::RedAlgae
        } else {
            Voxel::Seagrass
        };
        set_voxel_in_chunk(
            voxels,
            chunk_shapes,
            chunk_min_y,
            lx,
            floor_y,
            lz,
            vox,
            BlockShape::CrossBillboard,
            0,
        );
    }
}

/// Lightweight surface feature classification for horizon LOD meshes.
#[derive(Clone, Copy, Debug)]
pub enum LodFloraKind {
    RubbleBox(Voxel),
    TallBox {
        voxel: Voxel,
        height: f32,
        half_width: f32,
    },
    CrossPlant {
        bot: Voxel,
        top: Option<Voxel>,
    },
    FlatCarpet(Voxel),
}

#[inline(always)]
fn surface_rubble_chance(col: TerrainColumn, ground: Voxel) -> f32 {
    if col.is_cliff || is_rocky_biome(col.biome) {
        0.048
    } else if !ground.is_fertile_soil() && !ground.is_sandy_soil() {
        0.030
    } else {
        0.009
    }
}

#[inline(always)]
fn surface_pebble_chance(col: TerrainColumn) -> f32 {
    if col.biome == BiomeType::GravelBeach {
        0.028
    } else if col.is_beach || col.river_factor > 0.12 {
        0.012
    } else if is_rocky_biome(col.biome) {
        0.008
    } else {
        0.002
    }
}

#[inline]
fn sample_wild_crop_column(
    wx: i32,
    wz: i32,
    ground: Voxel,
    col: TerrainColumn,
    seed: u32,
) -> Option<(Voxel, Option<Voxel>)> {
    let is_beach_surface = col.is_beach || col.biome == BiomeType::GravelBeach;
    let crop_patch = patch_noise(wx, wz, 22.0, seed ^ 0xC095_F00D);
    if crop_patch > 0.78 && (!is_beach_surface || col.river_factor > 0.10) {
        let h_crop = hash2d(wx, wz, seed ^ 0x7A31_5512);
        if hash_f32(h_crop) < 0.065 {
            let is_wet_bank = col.river_factor > 0.10
                || col.is_lake
                || matches!(col.biome, BiomeType::OrchidSwamp | BiomeType::DesertOasis);
            // Use coarse patch cell so nearby wild crops belong to the same species
            let species_id = hash2d(wx.div_euclid(24), wz.div_euclid(24), seed ^ 0xAA11_9922);
            let (bot, top_opt) = select_wild_crop(col.biome, is_wet_bank, species_id);
            if bot.can_be_placed_on_ground(ground) {
                return Some((bot, top_opt));
            }
        }
    }
    None
}

#[derive(Clone, Copy)]
enum SurfaceFloraDecision {
    Cactus(i32),
    Bamboo(i32),
    FlatPad(Voxel),
    Cross { bot: Voxel, top: Option<Voxel> },
}

fn select_surface_flora_column(
    wx: i32,
    wz: i32,
    ground: Voxel,
    col: TerrainColumn,
    seed: u32,
    skip_small_patches: bool,
) -> Option<(SurfaceFloraDecision, u32)> {
    let h = hash2d(wx, wz, seed ^ 0xF107_A123);
    let roll = hash_f32(h);

    if skip_small_patches {
        return match col.biome {
            BiomeType::DuneSea
            | BiomeType::AncientSands
            | BiomeType::DesertSpires
            | BiomeType::DesertOasis
            | BiomeType::BryceCanyon
            | BiomeType::AmethystCanyon => {
                if ground.is_sandy_soil() && roll < 0.0025 {
                    let height = 1 + ((h >> 8) % 3) as i32;
                    Some((SurfaceFloraDecision::Cactus(height), h))
                } else {
                    None
                }
            }
            BiomeType::TropicalJungle => {
                if ground.is_fertile_soil() {
                    let bamboo_patch = patch_noise(wx, wz, 20.0, seed ^ 0xBA3B_0000);
                    if bamboo_patch > 0.74 && roll < 0.025 {
                        let height = 3 + ((h >> 8) % 4) as i32;
                        Some((SurfaceFloraDecision::Bamboo(height), h))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
    }

    let flower_patch = patch_noise(wx, wz, 16.0, seed ^ 0xF10E_5555);
    let grass_patch = patch_noise(wx, wz, 18.0, seed ^ 0x67A5_5111);

    // Coastal beaches & GravelBeach: sparse Sea Oats on sand, plus hardy grass patches on dirty gravel
    if col.is_beach || col.biome == BiomeType::MirageIsles || col.biome == BiomeType::GravelBeach {
        if matches!(ground, Voxel::GravelDirty | Voxel::GravelAshy) && grass_patch > 0.36 && roll < 0.095 {
            let vox = match (h >> 8) % 4 {
                0 | 1 => Voxel::VegeWildgrass,
                2 => Voxel::VegeDrygrass,
                _ => Voxel::VegeBluffGrass,
            };
            return Some((SurfaceFloraDecision::Cross { bot: vox, top: None }, h));
        } else if ground.is_sandy_soil() && grass_patch > 0.60 && roll < 0.018 {
            return Some((
                SurfaceFloraDecision::Cross {
                    bot: Voxel::VegeSeaOatsBottom,
                    top: Some(Voxel::VegeSeaOatsTop),
                },
                h,
            ));
        }
        return None;
    }

    // Riverbank / Wetland Cattails
    if (col.river_factor > 0.14 || col.is_lake || col.biome == BiomeType::OrchidSwamp)
        && roll < 0.024
        && Voxel::VegeCattailBottom.can_be_placed_on_ground(ground)
    {
        return Some((
            SurfaceFloraDecision::Cross {
                bot: Voxel::VegeCattailBottom,
                top: Some(Voxel::VegeCattailTop),
            },
            h,
        ));
    }

    match col.biome {
        // --- DESERTS & ARID BADLANDS ---
        BiomeType::DuneSea
        | BiomeType::AncientSands
        | BiomeType::DesertSpires
        | BiomeType::DesertOasis
        | BiomeType::BryceCanyon
        | BiomeType::AmethystCanyon => {
            if ground.is_sandy_soil() {
                if roll < 0.0025 {
                    let height = 1 + ((h >> 8) % 3) as i32;
                    return Some((SurfaceFloraDecision::Cactus(height), h));
                } else if roll < 0.012 {
                    let vox = match (h >> 12) % 5 {
                        0 => Voxel::VegeDeadBush,
                        1 => Voxel::VegeBarrelCactus,
                        2 => Voxel::VegePricklyPearCactus,
                        3 => Voxel::FlowerCactusFlower,
                        _ => Voxel::VegeDrygrass,
                    };
                    return Some((SurfaceFloraDecision::Cross { bot: vox, top: None }, h));
                }
            }
            None
        }

        // --- SAVANNAS & DRY STEPPES ---
        BiomeType::FracturedSavanna | BiomeType::SavannaBadlands | BiomeType::AshenSavanna => {
            if roll < 0.085 && grass_patch > 0.34 {
                let vox = match (h >> 8) % 5 {
                    0 | 1 => Voxel::VegeDrygrass,
                    2 => Voxel::VegeBluffGrass,
                    3 => Voxel::VegeRedShrub,
                    _ => Voxel::FlowerGlobeMallow,
                };
                if vox.can_be_placed_on_ground(ground) {
                    return Some((SurfaceFloraDecision::Cross { bot: vox, top: None }, h));
                }
            }
            None
        }

        // --- BLOOMING VALLEY ---
        BiomeType::BloomingValley => {
            if flower_patch > 0.50 && roll < 0.055 {
                let species = (hash2d(wx.div_euclid(8), wz.div_euclid(8), seed ^ 0xB100_0001)
                    + ((h >> 10) % 2))
                    % 10;
                let (bot, top_opt) = match species {
                    0 => (Voxel::FlowerPoppy, None),
                    1 => (Voxel::FlowerDandelion, None),
                    2 => (Voxel::FlowerCornflower, None),
                    3 => (Voxel::FlowerOxeyeDaisy, None),
                    4 => (Voxel::FlowerOrangeTulip, None),
                    5 => (Voxel::FlowerPinkTulip, None),
                    6 => (Voxel::FlowerAllium, None),
                    7 => (Voxel::FlowerLilacBottom, Some(Voxel::FlowerLilacTop)),
                    8 => (Voxel::FlowerPeonyBottom, Some(Voxel::FlowerPeonyTop)),
                    _ => (Voxel::FlowerRoseBushBottom, Some(Voxel::FlowerRoseBushTop)),
                };
                if bot.can_be_placed_on_ground(ground) {
                    return Some((SurfaceFloraDecision::Cross { bot, top: top_opt }, h));
                }
            } else if roll < 0.085 && grass_patch > 0.34 && ground.is_fertile_soil() {
                let vox = if ((h >> 12) % 4) == 0 {
                    Voxel::FlowerWildflowers
                } else {
                    Voxel::VegeWildgrass
                };
                return Some((SurfaceFloraDecision::Cross { bot: vox, top: None }, h));
            }
            None
        }

        // --- LAVENDER VALLEY ---
        BiomeType::LavenderValley => {
            if flower_patch > 0.45 && roll < 0.055 && ground.is_fertile_soil() {
                let vox = match (h >> 8) % 5 {
                    0..=2 => Voxel::FlowerLavender,
                    3 => Voxel::FlowerAllium,
                    _ => Voxel::FlowerIris,
                };
                return Some((SurfaceFloraDecision::Cross { bot: vox, top: None }, h));
            } else if roll < 0.075 && grass_patch > 0.34 && ground.is_fertile_soil() {
                return Some((
                    SurfaceFloraDecision::Cross {
                        bot: Voxel::VegeWildgrass,
                        top: None,
                    },
                    h,
                ));
            }
            None
        }

        // --- SAKURA GROVE ---
        BiomeType::SakuraGrove => {
            if flower_patch > 0.50 && roll < 0.060 && ground.is_fertile_soil() {
                let vox = if ((h >> 8) % 4) == 0 {
                    Voxel::FlowerWhitePetals
                } else {
                    Voxel::FlowerPinkPetals
                };
                return Some((SurfaceFloraDecision::FlatPad(vox), h));
            } else if roll < 0.070 && grass_patch > 0.34 && ground.is_fertile_soil() {
                let vox = if ((h >> 12) % 3) == 0 {
                    Voxel::FlowerClover
                } else {
                    Voxel::VegeWildgrass
                };
                return Some((SurfaceFloraDecision::Cross { bot: vox, top: None }, h));
            }
            None
        }

        // --- TROPICAL JUNGLE ---
        BiomeType::TropicalJungle => {
            if ground.is_fertile_soil() {
                let bamboo_patch = patch_noise(wx, wz, 20.0, seed ^ 0xBA3B_0000);
                if bamboo_patch > 0.74 && roll < 0.025 {
                    let height = 3 + ((h >> 8) % 4) as i32;
                    return Some((SurfaceFloraDecision::Bamboo(height), h));
                } else if roll < 0.11 {
                    let decision = match (h >> 10) % 5 {
                        0 => SurfaceFloraDecision::Cross {
                            bot: Voxel::VegeTallJungleFernBottom,
                            top: Some(Voxel::VegeTallJungleFernTop),
                        },
                        1 => SurfaceFloraDecision::Cross {
                            bot: Voxel::VegeFern,
                            top: None,
                        },
                        2 => SurfaceFloraDecision::Cross {
                            bot: Voxel::FlowerHibiscus,
                            top: None,
                        },
                        3 => SurfaceFloraDecision::Cross {
                            bot: Voxel::FlowerOrchid,
                            top: None,
                        },
                        _ => SurfaceFloraDecision::Cross {
                            bot: Voxel::VegeBush,
                            top: None,
                        },
                    };
                    return Some((decision, h));
                }
            }
            None
        }

        // --- ORCHID SWAMP ---
        BiomeType::OrchidSwamp => {
            if ground.is_fertile_soil() && roll < 0.085 {
                let vox = match (h >> 8) % 5 {
                    0 | 1 => Voxel::FlowerBlueOrchid,
                    2 => Voxel::VegeFern,
                    3 => Voxel::VegeMossgrass,
                    _ => Voxel::FungiVerdigris,
                };
                return Some((SurfaceFloraDecision::Cross { bot: vox, top: None }, h));
            }
            None
        }

        // --- TAIGA / COLD FORESTS / PINE HIGHLANDS ---
        BiomeType::SiberianTaiga
        | BiomeType::WintryForest
        | BiomeType::PineHighlands
        | BiomeType::ColdShrubland => {
            if ground.is_fertile_soil() && roll < 0.075 {
                let vox = match (h >> 8) % 6 {
                    0 | 1 => Voxel::VegeFern,
                    2 => Voxel::VegeBracken,
                    3 => Voxel::VegeMossgrass,
                    4 => Voxel::FungiAmanita,
                    _ => Voxel::FungiPorcini,
                };
                return Some((SurfaceFloraDecision::Cross { bot: vox, top: None }, h));
            }
            None
        }

        // --- FROZEN / GLACIAL / VOLCANIC (sparse or barren) ---
        BiomeType::GlacialChasm
        | BiomeType::FrozenCliffs
        | BiomeType::VolcanicPeaks
        | BiomeType::Caldera
        | BiomeType::ColdOcean
        | BiomeType::DeepWarmOcean
        | BiomeType::CoralReef
        | BiomeType::OceanTrench => None,

        // --- TEMPERATE FORESTS, LOWLANDS, SHIELD & MOUNTAINS (default open scatter) ---
        _ => {
            if !ground.is_fertile_soil() {
                return None;
            }
            if flower_patch > 0.68 && roll < 0.025 {
                let (bot, top_opt) = match (h >> 8) % 8 {
                    0 => (Voxel::FlowerDandelion, None),
                    1 => (Voxel::FlowerPoppy, None),
                    2 => (Voxel::FlowerClover, None),
                    3 => (Voxel::FlowerMarigold, None),
                    4 => (Voxel::FlowerLilyOfTheValley, None),
                    5 => (Voxel::FlowerFoxgloveBottom, Some(Voxel::FlowerFoxgloveTop)),
                    6 => (
                        Voxel::FlowerBlueHydrangeaBottom,
                        Some(Voxel::FlowerBlueHydrangeaTop),
                    ),
                    _ => (Voxel::FlowerSnapdragonBottom, Some(Voxel::FlowerSnapdragonTop)),
                };
                return Some((SurfaceFloraDecision::Cross { bot, top: top_opt }, h));
            } else if grass_patch > 0.32 && roll < 0.095 {
                let decision = match (h >> 12) % 6 {
                    0 | 1 => SurfaceFloraDecision::Cross {
                        bot: Voxel::VegeWildgrass,
                        top: None,
                    },
                    2 => SurfaceFloraDecision::Cross {
                        bot: Voxel::VegeTallWildgrassBottom,
                        top: Some(Voxel::VegeTallWildgrassTop),
                    },
                    3 => SurfaceFloraDecision::Cross {
                        bot: Voxel::VegeFern,
                        top: None,
                    },
                    4 => SurfaceFloraDecision::Cross {
                        bot: Voxel::VegeTallFernBottom,
                        top: Some(Voxel::VegeTallFernTop),
                    },
                    _ => SurfaceFloraDecision::Cross {
                        bot: Voxel::VegeBush,
                        top: None,
                    },
                };
                return Some((decision, h));
            }
            None
        }
    }
}

/// Evaluates lightweight LOD props (rubble, boulders/crops, cacti, bamboo, bushes, and flowers)
/// for horizon chunks without generating full 3D chunks.
pub fn sample_lod_surface_prop(
    wx: i32,
    wz: i32,
    ground: Voxel,
    col: TerrainColumn,
    seed: u32,
    is_lod1: bool,
) -> Option<LodFloraKind> {
    if col.surface_shape != BlockShape::Full
        || col.water_level.is_some_and(|wl| col.terrain_height < wl)
        || !ground.is_solid_opaque()
        || ground.is_fluid()
        || ground.is_empty()
    {
        return None;
    }

    let h = hash2d(wx, wz, seed ^ 0x4F1B_BCDC);
    let roll = hash_f32(h);

    if roll < surface_rubble_chance(col, ground) {
        return if is_lod1 {
            Some(LodFloraKind::RubbleBox(matching_rubble_for(ground)))
        } else {
            None
        };
    }

    if is_lod1 {
        let h_peb = hash2d(wx, wz, seed ^ 0x1337_BEEF);
        if hash_f32(h_peb) < surface_pebble_chance(col) {
            return None;
        }
        if is_woodland_biome(col.biome) && ground.is_fertile_soil() {
            let h_stick = hash2d(wx, wz, seed ^ 0x571C_C0DE);
            if hash_f32(h_stick) < 0.0055 {
                return None;
            }
        }
        if let Some((bot, top_opt)) = sample_wild_crop_column(wx, wz, ground, col, seed) {
            if matches!(bot, Voxel::WildPumpkin | Voxel::WildMelon) {
                return Some(LodFloraKind::TallBox {
                    voxel: bot,
                    height: 0.75,
                    half_width: 0.38,
                });
            }
            return Some(LodFloraKind::CrossPlant { bot, top: top_opt });
        }
    }

    let (decision, h_flora) = select_surface_flora_column(wx, wz, ground, col, seed, !is_lod1)?;

    match decision {
        SurfaceFloraDecision::Cactus(height) => Some(LodFloraKind::TallBox {
            voxel: Voxel::VegeCactus,
            height: height as f32,
            half_width: if is_lod1 { 0.38 } else { 0.42 },
        }),
        SurfaceFloraDecision::Bamboo(height) => Some(LodFloraKind::TallBox {
            voxel: Voxel::BambooLeaves,
            height: height as f32,
            half_width: if is_lod1 { 0.35 } else { 0.38 },
        }),
        SurfaceFloraDecision::FlatPad(voxel) => {
            is_lod1.then_some(LodFloraKind::FlatCarpet(voxel))
        }
        SurfaceFloraDecision::Cross { bot, top } => {
            if !is_lod1 {
                return None;
            }
            // Thin single-block wildgrass tufts by 50% in Lod1 while keeping 100% of bushes, ferns, and flowers
            if top.is_none()
                && matches!(
                    bot,
                    Voxel::VegeWildgrass
                        | Voxel::VegeDrygrass
                        | Voxel::VegeBluffGrass
                        | Voxel::VegeMossgrass
                )
                && ((h_flora >> 15) & 1) != 0
            {
                return None;
            }
            Some(LodFloraKind::CrossPlant { bot, top })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn populate_surface_flora_column(
    voxels: &mut [Voxel],
    chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
    chunk_min_y: i32,
    lx: usize,
    lz: usize,
    wx: i32,
    wz: i32,
    place_y: i32,
    ground: Voxel,
    col: TerrainColumn,
    seed: u32,
) {
    let Some((decision, _)) = select_surface_flora_column(wx, wz, ground, col, seed, false) else {
        return;
    };

    match decision {
        SurfaceFloraDecision::Cactus(height) => {
            for dy in 0..height {
                set_air_only_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    place_y + dy,
                    lz,
                    Voxel::VegeCactus,
                    BlockShape::Cactus,
                    0,
                );
            }
        }
        SurfaceFloraDecision::Bamboo(height) => {
            for dy in 0..height {
                let vox = if dy + 1 >= height - 1 {
                    Voxel::BambooLeaves
                } else {
                    Voxel::BambooStalk
                };
                set_air_only_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    place_y + dy,
                    lz,
                    vox,
                    BlockShape::Bamboo,
                    0,
                );
            }
        }
        SurfaceFloraDecision::FlatPad(vox) => {
            set_air_only_in_chunk(
                voxels,
                chunk_shapes,
                chunk_min_y,
                lx,
                place_y,
                lz,
                vox,
                BlockShape::FlatPad,
                0,
            );
        }
        SurfaceFloraDecision::Cross { bot, top } => {
            set_air_only_in_chunk(
                voxels,
                chunk_shapes,
                chunk_min_y,
                lx,
                place_y,
                lz,
                bot,
                BlockShape::CrossBillboard,
                0,
            );
            if let Some(top_vox) = top {
                set_air_only_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    place_y + 1,
                    lz,
                    top_vox,
                    BlockShape::CrossBillboard,
                    0,
                );
            }
        }
    }
}

/// Populates subterranean cave floors and ceilings with light growths, mushrooms, and hanging vines.
fn populate_cave_flora(
    voxels: &mut [Voxel],
    chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
    columns: &[TerrainColumn; CHUNK_SIZE * CHUNK_SIZE],
    chunk_origin: IVec3,
    seed: u32,
) {
    for lz in 0..CHUNK_SIZE {
        let wz = chunk_origin.z + lz as i32;
        for lx in 0..CHUNK_SIZE {
            let wx = chunk_origin.x + lx as i32;
            let col = columns[lz * CHUNK_SIZE + lx];
            let max_cave_y = col.terrain_height - 5;

            if chunk_origin.y > max_cave_y {
                continue;
            }

            let is_mushroom_cave = col.biome == BiomeType::MushroomCaves;
            let is_lush_cave = col.biome == BiomeType::UndergroundJungle;

            for ly in 1..(CHUNK_SIZE - 1) {
                let wy = chunk_origin.y + ly as i32;
                if wy > max_cave_y {
                    break;
                }

                let idx = chunk_idx(lx, ly, lz);
                if !voxels[idx].is_empty() {
                    continue;
                }

                let below = voxels[chunk_idx(lx, ly - 1, lz)];
                let above = voxels[chunk_idx(lx, ly + 1, lz)];

                // Cave floor growths & mushrooms
                if below.is_solid_opaque() {
                    let h = hash3d(wx, wy, wz, seed ^ 0xCA7E_F107);
                    let r = hash_f32(h);
                    let chance = if is_mushroom_cave || is_lush_cave {
                        0.035
                    } else {
                        0.008
                    };
                    if r < chance {
                        let vox = if is_mushroom_cave || ((h >> 8) % 4) == 0 {
                            match (h >> 12) % 5 {
                                0 => Voxel::FungiAmanita,
                                1 => Voxel::FungiPorcini,
                                2 => Voxel::FungiRussula,
                                3 => Voxel::FungiShiitake,
                                _ => Voxel::FungiVerdigris,
                            }
                        } else {
                            match (h >> 12) % 3 {
                                0 => Voxel::VegeCaveGrowths,
                                1 => Voxel::VegeGrainyCaveGrowths,
                                _ => Voxel::VegeCaveHyssop,
                            }
                        };
                        voxels[idx] = vox;
                        chunk_shapes.push((idx, BlockShape::CrossBillboard, 0));
                    }
                } else if above.is_solid_opaque() {
                    // Cave ceiling hanging moss / roots / short cave vines
                    let h = hash3d(wx, wy, wz, seed ^ 0xCA7E_CE11);
                    let r = hash_f32(h);
                    let chance = if is_lush_cave { 0.045 } else { 0.009 };
                    if r < chance {
                        let vox = match (h >> 8) % 3 {
                            0 => Voxel::VegeHangingMoss,
                            1 => Voxel::VegeHangingRoots,
                            _ => Voxel::VegeCaveVines,
                        };
                        voxels[idx] = vox;
                        // Orientation 1 = Ceiling (Hanging)
                        chunk_shapes.push((idx, BlockShape::CrossBillboard, 1));
                    }
                }
            }
        }
    }
}
