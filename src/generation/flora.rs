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
fn select_wild_crop(biome: BiomeType, is_wet_bank: bool, patch_id: u32) -> (Voxel, Option<Voxel>) {
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

            let ground = voxel_at(wx, surface_y, wz, col);
            if ground.is_empty() || ground.is_fluid() || !ground.is_solid_opaque() {
                continue;
            }

            // Forest Moss ground patches (`MossForest`) across wooded & lush biomes
            // Evaluated before the `surface_shape != Full` skip so patches flow seamlessly
            // across sculpted terrain slabs and stairs too.
            let mut is_moss_patch = false;
            if !col.is_beach
                && !col.is_cliff
                && col.water_level.is_none_or(|wl| surface_y >= wl)
                && ground.is_grass_soil()
                && matches!(
                    col.biome,
                    BiomeType::YosemiteLowlands
                        | BiomeType::CloudForest
                        | BiomeType::SakuraGrove
                        | BiomeType::BloomingValley
                        | BiomeType::LavenderValley
                        | BiomeType::OrchidSwamp
                        | BiomeType::Shield
                        | BiomeType::TropicalJungle
                        | BiomeType::PineHighlands
                        | BiomeType::SiberianTaiga
                        | BiomeType::WintryForest
                )
            {
                let moss_macro = patch_noise(wx, wz, 18.0, seed ^ 0x4055_A11C);
                let moss_detail = patch_noise(wx, wz, 5.5, seed ^ 0x4055_B22D);
                let macro_thresh = if matches!(
                    col.biome,
                    BiomeType::OrchidSwamp | BiomeType::CloudForest | BiomeType::TropicalJungle
                ) {
                    0.60
                } else {
                    0.70
                };
                if moss_macro > macro_thresh && moss_detail > 0.50 {
                    let surf_ly = surface_y - chunk_min_y;
                    if (0..CHUNK_SIZE as i32).contains(&surf_ly) {
                        let sidx = chunk_idx(lx, surf_ly as usize, lz);
                        if voxels[sidx].is_solid_opaque() {
                            voxels[sidx] = Voxel::MossForest;
                            is_moss_patch = true;
                        }
                    }
                }
            }

            // Never place surface features on sculpted terrain slopes (slabs/stairs)
            if col.surface_shape != BlockShape::Full {
                continue;
            }

            // --- 20.4: MARINE & AQUATIC SCATTER ---
            if let Some(wl) = col.water_level
                && surface_y < wl {
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
                let is_cluster = (h_peb >> 8).is_multiple_of(4);
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

                // Clustered Leaf Litter patches on temperate/deciduous woodland floors
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
                    if litter_patch > 0.66 {
                        let h_litter = hash2d(wx, wz, seed ^ 0x1EAF_5EED);
                        if hash_f32(h_litter) < 0.09 {
                            let facing = (h_litter & 0x03) as u8;
                            let seg_idx: u8 = if litter_patch > 0.84 {
                                3
                            } else if litter_patch > 0.77 {
                                2
                            } else if litter_patch > 0.71 {
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

            // Spawn Moss Carpet & Mossgrass naturally on top of Forest Moss patches
            if is_moss_patch || ground == Voxel::MossForest {
                let h_mg = hash2d(wx, wz, seed ^ 0x4055_67A5);
                let mg_roll = hash_f32(h_mg);
                if mg_roll < 0.16 {
                    set_air_only_in_chunk(
                        voxels,
                        chunk_shapes,
                        chunk_min_y,
                        lx,
                        place_y,
                        lz,
                        Voxel::MossCarpet,
                        BlockShape::Carpet,
                        0,
                    );
                    continue;
                } else if mg_roll < 0.42 {
                    set_air_only_in_chunk(
                        voxels,
                        chunk_shapes,
                        chunk_min_y,
                        lx,
                        place_y,
                        lz,
                        Voxel::VegeMossgrass,
                        BlockShape::CrossBillboard,
                        0,
                    );
                    continue;
                }
            } else if matches!(
                ground,
                Voxel::MossyAndesite
                    | Voxel::MossyArgillite
                    | Voxel::MossyBasalt
                    | Voxel::MossyChalk
                    | Voxel::MossyDolomite
                    | Voxel::MossyGranite
                    | Voxel::MossyLimestone
                    | Voxel::MossyCobbledAndesite
                    | Voxel::MossyCobbledArgillite
                    | Voxel::MossyCobbledBasalt
                    | Voxel::MossyCobbledChalk
                    | Voxel::MossyCobbledDolomite
                    | Voxel::MossyCobbledGranite
                    | Voxel::MossyCobbledLimestone
            ) {
                let h_mc = hash2d(wx, wz, seed ^ 0x4055_CA49);
                if hash_f32(h_mc) < 0.18 {
                    set_air_only_in_chunk(
                        voxels,
                        chunk_shapes,
                        chunk_min_y,
                        lx,
                        place_y,
                        lz,
                        Voxel::MossCarpet,
                        BlockShape::Carpet,
                        0,
                    );
                    continue;
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

    // Underwater Hydrothermal Vents (`GeothermalVent`) + Volcanic `Magma` pockets on deep seabeds
    if !col.is_river && !col.is_lake && depth >= 6 && floor_y + 2 < water_level {
        let vent_patch = patch_noise(wx, wz, 22.0, seed ^ 0x6E07_3A11);
        let h_vent = hash2d(wx, wz, seed ^ 0x6E07_9944);
        let r_vent = hash_f32(h_vent);
        let should_spawn_vent = match col.biome {
            BiomeType::OceanTrench | BiomeType::Caldera => vent_patch > 0.58 && r_vent < 0.032,
            BiomeType::DeepWarmOcean | BiomeType::ColdOcean if depth >= 8 => {
                vent_patch > 0.68 && r_vent < 0.012
            }
            _ => false,
        };
        if should_spawn_vent {
            let above_height: i32 = if depth >= 8 && ((h_vent >> 8) & 1) == 0 {
                2
            } else {
                1
            };
            let underground_depth: i32 = 2 + ((h_vent >> 9) & 1) as i32;

            for y in (floor_y - underground_depth)..(floor_y + above_height) {
                set_voxel_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    y,
                    lz,
                    Voxel::GeothermalVent,
                    BlockShape::Full,
                    0,
                );
            }

            let magma_top_y = floor_y - underground_depth - 1;
            for y in (magma_top_y - 1)..=magma_top_y {
                set_voxel_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    lx,
                    y,
                    lz,
                    Voxel::Magma,
                    BlockShape::Full,
                    0,
                );
            }

            // Surround chimney with 1-2 submerged Magma / Basalt pockets inside chunk bounds
            for (i, (dx, dz)) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)].iter().enumerate() {
                let nlx = lx as i32 + dx;
                let nlz = lz as i32 + dz;
                if !(0..CHUNK_SIZE as i32).contains(&nlx) || !(0..CHUNK_SIZE as i32).contains(&nlz)
                {
                    continue;
                }
                if (h_vent >> (12 + i * 2)).is_multiple_of(3) {
                    let accent = if ((h_vent >> (10 + i)) & 1) == 0 {
                        Voxel::Magma
                    } else {
                        Voxel::Basalt
                    };
                    set_voxel_in_chunk(
                        voxels,
                        chunk_shapes,
                        chunk_min_y,
                        nlx as usize,
                        floor_y - 1,
                        nlz as usize,
                        accent,
                        BlockShape::Full,
                        0,
                    );
                }
            }
            return;
        }
    }

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

    // Vibrant Coral Reefs: dense, multi-block branching colonies with 6-face wall/top coral fans
    let is_coral_zone = matches!(col.biome, BiomeType::CoralReef | BiomeType::DeepWarmOcean)
        || (!col.is_river && col.climate.temperature > -0.15);
    if is_coral_zone && depth <= 22 {
        let reef_patch = patch_noise(wx, wz, 20.0, seed ^ 0xC07A_1FEF);
        let reef_cluster = patch_noise(wx, wz, 7.0, seed ^ 0xC07A_8841);
        let (threshold, chance) = if col.biome == BiomeType::CoralReef {
            (0.30, 0.34)
        } else if col.biome == BiomeType::DeepWarmOcean {
            (0.44, 0.24)
        } else {
            (0.54, 0.18)
        };
        // Organic Coralstone seabed patches underneath and around coral reef clusters
        if reef_patch > threshold - 0.04 && reef_cluster > 0.30 {
            let cs_patch = patch_noise(wx, wz, 5.5, seed ^ 0xC057_09E1);
            if cs_patch > 0.44 {
                let ly = surface_y - chunk_min_y;
                if (0..CHUNK_SIZE as i32).contains(&ly) {
                    let idx = chunk_idx(lx, ly as usize, lz);
                    if voxels[idx].is_solid_opaque() {
                        voxels[idx] = Voxel::Coralstone;
                    }
                }
            }
        }
        if reef_patch > threshold && reef_cluster > 0.36 && r < chance {
            // Ensure seabed directly beneath coral colonies frequently transitions to Coralstone
            if !(h >> 3).is_multiple_of(4) {
                let ly = surface_y - chunk_min_y;
                if (0..CHUNK_SIZE as i32).contains(&ly) {
                    let idx = chunk_idx(lx, ly as usize, lz);
                    if voxels[idx].is_solid_opaque() {
                        voxels[idx] = Voxel::Coralstone;
                    }
                }
            }
            // Pick a coherent colony species per small 5x5 cell so nearby corals form lush reefs
            let colony_h = hash2d(wx.div_euclid(5), wz.div_euclid(5), seed ^ 0xC07A_5512);
            let variant = if (h >> 6).is_multiple_of(5) {
                (h >> 8) % 10
            } else {
                (colony_h >> 4) % 10
            };
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

            let chunk_origin_x = wx - lx as i32;
            let chunk_origin_z = wz - lz as i32;
            let try_place_coral = |voxels: &mut [Voxel],
                                   chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
                                   tx: i32,
                                   ty: i32,
                                   tz: i32,
                                   vox: Voxel,
                                   shape: BlockShape,
                                   orient: u8| {
                let nlx = tx - chunk_origin_x;
                let nlz = tz - chunk_origin_z;
                if !(0..CHUNK_SIZE as i32).contains(&nlx) || !(0..CHUNK_SIZE as i32).contains(&nlz)
                {
                    return;
                }
                if ty >= water_level {
                    return;
                }
                set_voxel_in_chunk(
                    voxels,
                    chunk_shapes,
                    chunk_min_y,
                    nlx as usize,
                    ty,
                    nlz as usize,
                    vox,
                    shape,
                    orient,
                );
            };

            let decorate_coral_block = |voxels: &mut [Voxel],
                                        chunk_shapes: &mut Vec<(usize, BlockShape, u8)>,
                                        bx: i32,
                                        by: i32,
                                        bz: i32,
                                        dh: u32| {
                // Top fan (orientation 0: floor +Y)
                if !(dh >> 2).is_multiple_of(3) {
                    try_place_coral(
                        voxels,
                        chunk_shapes,
                        bx,
                        by + 1,
                        bz,
                        coral_fan,
                        BlockShape::CrossBillboard,
                        0,
                    );
                }
                // Lateral wall coral fans (orientations 2..=5, identical to cave crystal clusters!)
                // 2: +Z (North wall), 3: -Z (South wall), 4: +X (West wall), 5: -X (East wall)
                let sides: [(i32, i32, u8); 4] = [(0, 1, 2), (0, -1, 3), (1, 0, 4), (-1, 0, 5)];
                for (si, &(sdx, sdz, sorient)) in sides.iter().enumerate() {
                    if ((dh >> (6 + si * 3)) % 5) < 2 {
                        try_place_coral(
                            voxels,
                            chunk_shapes,
                            bx + sdx,
                            by,
                            bz + sdz,
                            coral_fan,
                            BlockShape::CrossBillboard,
                            sorient,
                        );
                    }
                }
            };

            let structure_roll = (h >> 16) % 10;
            if structure_roll < 6 && depth >= 4 {
                // Multi-block 3D Coral Structure (Spire, Branching Tree, or Reef Mound)
                let arch = (h >> 20) % 3;
                match arch {
                    0 => {
                        // Tall Coral Spire / Pillar (2..=4 blocks stacked vertically)
                        let spire_h = (2 + ((h >> 12) as i32 % 3)).min(depth - 1);
                        for dy in 0..spire_h {
                            try_place_coral(
                                voxels,
                                chunk_shapes,
                                wx,
                                floor_y + dy,
                                wz,
                                coral_block,
                                BlockShape::Full,
                                0,
                            );
                        }
                        for dy in 0..spire_h {
                            decorate_coral_block(
                                voxels,
                                chunk_shapes,
                                wx,
                                floor_y + dy,
                                wz,
                                h.wrapping_add(dy as u32 * 97),
                            );
                        }
                    }
                    1 => {
                        // Branching Coral Tree (vertical stem + 2..=3 outward/upward arms)
                        let stem_h = (2 + ((h >> 12) as i32 % 2)).min(depth - 2);
                        for dy in 0..stem_h {
                            try_place_coral(
                                voxels,
                                chunk_shapes,
                                wx,
                                floor_y + dy,
                                wz,
                                coral_block,
                                BlockShape::Full,
                                0,
                            );
                        }
                        let arm_dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
                        for (ai, &(adx, adz)) in arm_dirs.iter().enumerate() {
                            if !(h >> (10 + ai * 2)).is_multiple_of(3) {
                                let arm_y = floor_y + stem_h - 1;
                                try_place_coral(
                                    voxels,
                                    chunk_shapes,
                                    wx + adx,
                                    arm_y,
                                    wz + adz,
                                    coral_block,
                                    BlockShape::Full,
                                    0,
                                );
                                // Tip rises +1 block like staghorn/branching coral
                                let tip_y = arm_y + 1;
                                try_place_coral(
                                    voxels,
                                    chunk_shapes,
                                    wx + adx,
                                    tip_y,
                                    wz + adz,
                                    coral_block,
                                    BlockShape::Full,
                                    0,
                                );
                                decorate_coral_block(
                                    voxels,
                                    chunk_shapes,
                                    wx + adx,
                                    tip_y,
                                    wz + adz,
                                    h.wrapping_add(ai as u32 * 131),
                                );
                            }
                        }
                        decorate_coral_block(
                            voxels,
                            chunk_shapes,
                            wx,
                            floor_y + stem_h - 1,
                            wz,
                            h ^ 0x3311,
                        );
                    }
                    _ => {
                        // Compact Coral Reef Mound (2..=3 wide base + stacked center peak)
                        try_place_coral(
                            voxels,
                            chunk_shapes,
                            wx,
                            floor_y,
                            wz,
                            coral_block,
                            BlockShape::Full,
                            0,
                        );
                        try_place_coral(
                            voxels,
                            chunk_shapes,
                            wx,
                            floor_y + 1,
                            wz,
                            coral_block,
                            BlockShape::Full,
                            0,
                        );
                        for (ai, &(adx, adz)) in
                            [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().enumerate()
                        {
                            if (h >> (10 + ai * 2)).is_multiple_of(2) {
                                try_place_coral(
                                    voxels,
                                    chunk_shapes,
                                    wx + adx,
                                    floor_y,
                                    wz + adz,
                                    coral_block,
                                    BlockShape::Full,
                                    0,
                                );
                                decorate_coral_block(
                                    voxels,
                                    chunk_shapes,
                                    wx + adx,
                                    floor_y,
                                    wz + adz,
                                    h.wrapping_add(ai as u32 * 53),
                                );
                            }
                        }
                        decorate_coral_block(voxels, chunk_shapes, wx, floor_y + 1, wz, h ^ 0x7722);
                    }
                }
            } else {
                // Standalone floor coral fan around the colony
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
                let v = if dy + 1 == stalk_h {
                    tip_vox
                } else {
                    plant_vox
                };
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
        let vox = if (h >> 10).is_multiple_of(6) {
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

    // Coastal beaches & GravelBeach: Dry Wildgrass & sparse Sea Oats on sand, plus hardy grass patches on dirty gravel
    if col.is_beach || col.biome == BiomeType::MirageIsles || col.biome == BiomeType::GravelBeach {
        if matches!(ground, Voxel::GravelDirty | Voxel::GravelAshy)
            && grass_patch > 0.36
            && roll < 0.095
        {
            let vox = match (h >> 8) % 4 {
                0 | 1 => Voxel::VegeWildgrass,
                2 => Voxel::VegeDrygrass,
                _ => Voxel::VegeBluffGrass,
            };
            return Some((
                SurfaceFloraDecision::Cross {
                    bot: vox,
                    top: None,
                },
                h,
            ));
        } else if ground.is_sandy_soil() && grass_patch > 0.60 && roll < 0.015 {
            return Some((
                SurfaceFloraDecision::Cross {
                    bot: Voxel::VegeSeaOatsBottom,
                    top: Some(Voxel::VegeSeaOatsTop),
                },
                h,
            ));
        } else if ground.is_sandy_soil() && grass_patch > 0.30 && roll < 0.075 {
            return Some((
                SurfaceFloraDecision::Cross {
                    bot: Voxel::VegeDrygrass,
                    top: None,
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
                    return Some((
                        SurfaceFloraDecision::Cross {
                            bot: vox,
                            top: None,
                        },
                        h,
                    ));
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
                    return Some((
                        SurfaceFloraDecision::Cross {
                            bot: vox,
                            top: None,
                        },
                        h,
                    ));
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
                let vox = if (h >> 12).is_multiple_of(4) {
                    Voxel::FlowerWildflowers
                } else {
                    Voxel::VegeWildgrass
                };
                return Some((
                    SurfaceFloraDecision::Cross {
                        bot: vox,
                        top: None,
                    },
                    h,
                ));
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
                return Some((
                    SurfaceFloraDecision::Cross {
                        bot: vox,
                        top: None,
                    },
                    h,
                ));
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
                let vox = if (h >> 8).is_multiple_of(4) {
                    Voxel::FlowerWhitePetals
                } else {
                    Voxel::FlowerPinkPetals
                };
                return Some((SurfaceFloraDecision::FlatPad(vox), h));
            } else if roll < 0.070 && grass_patch > 0.34 && ground.is_fertile_soil() {
                if (h >> 12).is_multiple_of(3) {
                    return Some((SurfaceFloraDecision::FlatPad(Voxel::FlowerClover), h));
                }
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
                return Some((
                    SurfaceFloraDecision::Cross {
                        bot: vox,
                        top: None,
                    },
                    h,
                ));
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
                return Some((
                    SurfaceFloraDecision::Cross {
                        bot: vox,
                        top: None,
                    },
                    h,
                ));
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
                if ((h >> 8) % 8) == 2 {
                    return Some((SurfaceFloraDecision::FlatPad(Voxel::FlowerClover), h));
                }
                let (bot, top_opt) = match (h >> 8) % 8 {
                    0 => (Voxel::FlowerDandelion, None),
                    1 => (Voxel::FlowerPoppy, None),
                    3 => (Voxel::FlowerMarigold, None),
                    4 => (Voxel::FlowerLilyOfTheValley, None),
                    5 => (Voxel::FlowerFoxgloveBottom, Some(Voxel::FlowerFoxgloveTop)),
                    6 => (
                        Voxel::FlowerBlueHydrangeaBottom,
                        Some(Voxel::FlowerBlueHydrangeaTop),
                    ),
                    _ => (
                        Voxel::FlowerSnapdragonBottom,
                        Some(Voxel::FlowerSnapdragonTop),
                    ),
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
        SurfaceFloraDecision::FlatPad(voxel) => is_lod1.then_some(LodFloraKind::FlatCarpet(voxel)),
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
                        let vox = if is_mushroom_cave || (h >> 8).is_multiple_of(4) {
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
