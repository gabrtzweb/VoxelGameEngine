use bevy::prelude::*;

use super::{
    adventure_map::AdventureWorldMap,
    biome::{BiomeType, ClimateGenerator, ClimateSample},
    caves::{CaveGenerator, CaveNoiseSample},
    flora, ores,
    strata::StrataGenerator,
    trees,
};
use crate::core::math::lerp;
use crate::core::noise::{Simd4f, fractal_noise_2d as fractal_noise, fractal_noise_2d_x4};
use crate::world::{BlockShape, CHUNK_SIZE, CHUNK_VOLUME, Chunk, Voxel};

pub const LOGICAL_BLOCK_VOXELS: i32 = 1;

pub const SEED_OFFSET_BEACH_NOISE: u32 = 88_411;
pub const SEED_OFFSET_SURFACE_DITHER: u32 = 45_678;
pub const SEED_OFFSET_SNOWLINE_JITTER: u32 = 91_111;
pub const SEED_OFFSET_SNOW_SLOPE: u32 = 82_222;
pub const SEED_OFFSET_OUTCROP_TURF: u32 = 73_333;

#[derive(Clone, Copy, Debug)]
pub struct TerrainColumn {
    pub terrain_height: i32,
    pub water_level: Option<i32>,
    pub biome: BiomeType,
    pub is_beach: bool,
    pub is_cliff: bool,
    pub is_underground_river: bool,
    pub is_river: bool,
    pub is_lake: bool,
    pub river_factor: f32,
    pub climate: ClimateSample,
    pub surface_shape: BlockShape,
    pub shape_orientation: u8,
}

impl Default for TerrainColumn {
    fn default() -> Self {
        Self {
            terrain_height: 0,
            water_level: None,
            biome: BiomeType::default(),
            is_beach: false,
            is_cliff: false,
            is_underground_river: false,
            is_river: false,
            is_lake: false,
            river_factor: 0.0,
            climate: ClimateSample {
                continentalness: 0.0,
                erosion: 0.0,
                ridges: 0.0,
                temperature: 0.0,
                humidity: 0.0,
                depth: 0.0,
                biome: BiomeType::default(),
            },
            surface_shape: BlockShape::Full,
            shape_orientation: 0,
        }
    }
}

#[derive(Resource, Clone, Reflect)]
pub struct TerrainGenerator {
    pub seed: u32,

    // Large-scale terrain.
    pub base_height: f32,
    pub macro_amplitude: f32,
    pub macro_frequency: f32,

    // Smaller terrain details.
    pub detail_amplitude: f32,
    pub detail_frequency: f32,
    pub detail_octaves: u32,
    pub persistence: f32,

    // Water, rivers & oceans.
    pub sea_level: i32,
    pub river_frequency: f32,
    pub river_width: f32,
    pub river_depth: f32,
    pub tributary_abundance: f32,
    pub reach_flatness: f32,
    pub stream_flow_speed: f32,

    // Organic lakes & ponds
    pub lake_abundance: f32,
    pub lake_size: f32,
    pub lake_depth: f32,

    // Mountain ridges and rolling hills
    pub mountain_ridge_height: f32,
    pub rolling_hills_amplitude: f32,

    // Advanced Phase 6 procedural systems
    pub climate: ClimateGenerator,
    pub caves: CaveGenerator,
    pub strata: StrataGenerator,

    // Phase 12 procedural vegetation
    pub tree_density: f32,

    // Phase 12 Adventure Mode handcrafted world map (None = infinite procedural Sandbox)
    #[reflect(ignore)]
    pub adventure_map: Option<std::sync::Arc<AdventureWorldMap>>,

    // Generation version tracker to signal runtime remeshing when tweaked in inspector
    pub version: u32,
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self {
            seed: 1337,

            // Baseline elevation in voxels
            base_height: 16.0,

            // Broad continental landforms
            macro_amplitude: 4.0,
            macro_frequency: 0.0260,

            // Local terrain relief
            detail_amplitude: 5.4,
            detail_frequency: 0.032,
            detail_octaves: 3,
            persistence: 0.5,

            // Mountain ridges and rolling hills
            mountain_ridge_height: 54.0,
            rolling_hills_amplitude: 10.0,

            // Sea level in voxels
            sea_level: 12,

            river_frequency: 0.0038,
            river_width: 0.055,
            river_depth: 3.5,
            tributary_abundance: 0.35,
            reach_flatness: 1.0,
            stream_flow_speed: 2.5,

            lake_abundance: 0.25,
            lake_size: 38.0,
            lake_depth: 5.8,

            climate: ClimateGenerator::default(),
            caves: CaveGenerator::default(),
            strata: StrataGenerator::default(),

            // Tree generation disabled for initial terrain validation
            tree_density: 0.0,

            adventure_map: None,

            version: 0,
        }
    }
}

const SHARED_COLUMN_CACHE_CAP: usize = 1024;

type ColumnGridArc = std::sync::Arc<[TerrainColumn; CHUNK_SIZE * CHUNK_SIZE]>;

struct SharedColumnCache {
    map: std::collections::HashMap<(u64, i32, i32), ColumnGridArc>,
    order: std::collections::VecDeque<(u64, i32, i32)>,
}

impl SharedColumnCache {
    fn new() -> Self {
        Self {
            map: std::collections::HashMap::with_capacity(SHARED_COLUMN_CACHE_CAP),
            order: std::collections::VecDeque::with_capacity(SHARED_COLUMN_CACHE_CAP),
        }
    }

    fn get(&self, key: &(u64, i32, i32)) -> Option<ColumnGridArc> {
        self.map.get(key).cloned()
    }

    fn insert(&mut self, key: (u64, i32, i32), val: ColumnGridArc) {
        if self.map.contains_key(&key) {
            return;
        }
        while self.map.len() >= SHARED_COLUMN_CACHE_CAP {
            if let Some(old_key) = self.order.pop_front() {
                self.map.remove(&old_key);
            } else {
                break;
            }
        }
        self.order.push_back(key);
        self.map.insert(key, val);
    }
}

static SHARED_COLUMN_CACHE: std::sync::OnceLock<std::sync::Mutex<SharedColumnCache>> =
    std::sync::OnceLock::new();

impl TerrainGenerator {
    #[inline]
    fn column_cache_fingerprint(&self) -> u64 {
        let mut h = (self.seed as u64) | ((self.version as u64) << 32);
        h ^= (self.sea_level as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^= (self.base_height.to_bits() as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= (self.macro_amplitude.to_bits() as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= (self.detail_amplitude.to_bits() as u64).rotate_left(17);
        h ^= (self.mountain_ridge_height.to_bits() as u64).rotate_left(31);
        h ^= (self.river_frequency.to_bits() as u64).rotate_left(43);
        h ^= (self.lake_abundance.to_bits() as u64).rotate_left(53);
        if let Some(ref map) = self.adventure_map {
            h ^= map.fingerprint();
        }
        h
    }

    fn get_or_build_chunk_columns(&self, chunk_x: i32, chunk_z: i32) -> ColumnGridArc {
        let key = (self.column_cache_fingerprint(), chunk_x, chunk_z);
        let mutex =
            SHARED_COLUMN_CACHE.get_or_init(|| std::sync::Mutex::new(SharedColumnCache::new()));

        if let Ok(guard) = mutex.lock()
            && let Some(cached) = guard.get(&key)
        {
            return cached;
        }

        let origin_x = chunk_x * CHUNK_SIZE as i32;
        let origin_z = chunk_z * CHUNK_SIZE as i32;
        let mut columns = [TerrainColumn::default(); CHUNK_SIZE * CHUNK_SIZE];

        for z in 0..CHUNK_SIZE {
            for x_start in (0..CHUNK_SIZE).step_by(4) {
                let wx = [
                    origin_x + x_start as i32,
                    origin_x + x_start as i32 + 1,
                    origin_x + x_start as i32 + 2,
                    origin_x + x_start as i32 + 3,
                ];
                let wz = [origin_z + z as i32; 4];
                let cols = self.sample_columns_x4(wx, wz);
                for (lane, col) in cols.into_iter().enumerate() {
                    columns[column_index(x_start + lane, z)] = col;
                }
            }
        }

        let arc: ColumnGridArc = std::sync::Arc::new(columns);
        if let Ok(mut guard) = mutex.lock() {
            guard.insert(key, arc.clone());
        }
        arc
    }

    #[inline]
    pub fn sample_column_cached(&self, world_x: i32, world_z: i32) -> TerrainColumn {
        let cx = world_x.div_euclid(CHUNK_SIZE as i32);
        let cz = world_z.div_euclid(CHUNK_SIZE as i32);
        let lx = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let lz = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;
        let key = (self.column_cache_fingerprint(), cx, cz);
        let mutex =
            SHARED_COLUMN_CACHE.get_or_init(|| std::sync::Mutex::new(SharedColumnCache::new()));
        if let Ok(guard) = mutex.lock()
            && let Some(cached) = guard.get(&key)
        {
            return cached[column_index(lx, lz)];
        }
        self.sample_column(world_x, world_z)
    }

    /// Fast 5-point bounding evaluation (4 corners + center) providing a safe upper-bound
    /// on the maximum solid terrain, water, and tree canopy height anywhere within chunk column (chunk_x, chunk_z).
    /// Fast 5-point bounding evaluation returning `(min_height, max_height)` in a single pass.
    /// Used by chunk streaming to avoid evaluating the same 5 noise points twice per column.
    pub fn estimate_chunk_height_bounds(&self, chunk_x: i32, chunk_z: i32) -> (i32, i32) {
        let chunk_origin_x = chunk_x * CHUNK_SIZE as i32;
        let chunk_origin_z = chunk_z * CHUNK_SIZE as i32;

        let samples = [
            (chunk_origin_x, chunk_origin_z),
            (chunk_origin_x + (CHUNK_SIZE as i32 - 1), chunk_origin_z),
            (chunk_origin_x, chunk_origin_z + (CHUNK_SIZE as i32 - 1)),
            (
                chunk_origin_x + (CHUNK_SIZE as i32 - 1),
                chunk_origin_z + (CHUNK_SIZE as i32 - 1),
            ),
            (
                chunk_origin_x + (CHUNK_SIZE as i32 / 2),
                chunk_origin_z + (CHUNK_SIZE as i32 / 2),
            ),
        ];

        let mut min_h = f32::MAX;
        let mut max_h = f32::MIN;
        for (sx, sz) in samples {
            let (raw_h, _, _) = self.continuous_height_and_biome(sx, sz);
            min_h = min_h.min(raw_h);
            max_h = max_h.max(raw_h);
        }

        let sea_level = self.effective_sea_level() as f32;
        let bound_min = (min_h.min(sea_level) - 16.0).floor() as i32;
        let bound_max = (max_h.max(sea_level) + 24.0).ceil() as i32;
        (bound_min, bound_max)
    }

    /// Used for O(1) early rejection of empty sky chunks.
    pub fn estimate_chunk_max_height(&self, chunk_x: i32, chunk_z: i32) -> i32 {
        self.estimate_chunk_height_bounds(chunk_x, chunk_z).1
    }

    /// Fast 5-point bounding evaluation providing a safe lower-bound
    /// on the minimum solid terrain or seabed height anywhere within chunk column (chunk_x, chunk_z).
    /// Used to ensure visible surface terrain and water chunks stay loaded when player is flying high.
    pub fn estimate_chunk_surface_min_height(&self, chunk_x: i32, chunk_z: i32) -> i32 {
        self.estimate_chunk_height_bounds(chunk_x, chunk_z).0
    }

    pub fn generate_chunk(&self, chunk_coordinate: IVec3) -> Chunk {
        let chunk_origin = chunk_coordinate * CHUNK_SIZE as i32;
        let chunk_min_y = chunk_origin.y;
        let chunk_max_y = chunk_min_y + CHUNK_SIZE as i32 - 1;

        // 1. Bedrock floor fast path: bottom layers below bedrock are 100% solid Dreadstone
        if chunk_max_y <= self.strata.bedrock_min_block_y {
            let mut chunk = Chunk::filled(Voxel::Mantle);
            chunk.set_subterranean(true);
            return chunk;
        }

        // 2. Sky fast path: if chunk is strictly above estimated max terrain height + canopy, return Air in O(1)
        let est_max = self.estimate_chunk_max_height(chunk_coordinate.x, chunk_coordinate.z);
        if chunk_min_y > est_max {
            return Chunk::filled(Voxel::Air);
        }

        let columns = self.get_or_build_chunk_columns(chunk_coordinate.x, chunk_coordinate.z);
        let mut maximum_filled_height = i32::MIN;
        let mut minimum_filled_height = i32::MAX;

        for col in columns.iter() {
            let col_top = col.terrain_height;
            let filled_height = col.water_level.unwrap_or(col_top).max(col_top) + 16;
            maximum_filled_height = maximum_filled_height.max(filled_height);
            minimum_filled_height = minimum_filled_height.min(col_top);
        }

        // Entire chunk is above terrain, water, AND any potential tree canopies
        if chunk_min_y > maximum_filled_height + 16 {
            return Chunk::filled(Voxel::Air);
        }

        let mut voxels = vec![Voxel::Air; CHUNK_VOLUME];
        let mut chunk_shapes: Vec<(usize, BlockShape, u8)> = Vec::new();
        let mut has_solid_voxels = false;
        let mut has_subterranean_air = false;
        let mut has_surface_overhang = false;

        if chunk_min_y <= maximum_filled_height {
            has_solid_voxels = true;
            let chunk_caves = self.caves.build_chunk_sampler(chunk_origin, self.seed);

            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let column = columns[column_index(x, z)];

                    for y in 0..CHUNK_SIZE {
                        let world_y = chunk_origin.y + y as i32;
                        let world_x = chunk_origin.x + x as i32;
                        let world_z = chunk_origin.z + z as i32;

                        let cave_sample = chunk_caves.sample(x, y, z);
                        let voxel =
                            self.voxel_at_sampled(column, world_x, world_y, world_z, cave_sample);
                        if voxel != Voxel::Air {
                            let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                            voxels[idx] = voxel;

                            if world_y > column.terrain_height
                                && voxel.is_collidable()
                                && !voxel.is_fluid()
                            {
                                has_surface_overhang = true;
                            }

                            if world_y == column.terrain_height
                                && column.surface_shape != BlockShape::Full
                            {
                                chunk_shapes.push((
                                    idx,
                                    column.surface_shape,
                                    column.shape_orientation,
                                ));
                            }
                        } else if world_y < column.terrain_height {
                            has_subterranean_air = true;
                        }
                    }
                }
            }

            // Surface Overhang & Basalt Outcrop Shaping: sculpt 3D surface rock mounds/outcrops
            // with stairs and slabs along curves, crests, and overhang undersides.
            if has_surface_overhang {
                let is_solid_world = |voxels_buf: &[Voxel], wx: i32, wy: i32, wz: i32| -> bool {
                    let lx = wx - chunk_origin.x;
                    let ly = wy - chunk_origin.y;
                    let lz = wz - chunk_origin.z;
                    if (0..CHUNK_SIZE as i32).contains(&lx)
                        && (0..CHUNK_SIZE as i32).contains(&ly)
                        && (0..CHUNK_SIZE as i32).contains(&lz)
                    {
                        let v = voxels_buf[lx as usize
                            + lz as usize * CHUNK_SIZE
                            + ly as usize * CHUNK_SIZE * CHUNK_SIZE];
                        v.is_collidable() && !v.is_fluid()
                    } else {
                        let col = if (0..CHUNK_SIZE as i32).contains(&lx)
                            && (0..CHUNK_SIZE as i32).contains(&lz)
                        {
                            columns[column_index(lx as usize, lz as usize)]
                        } else {
                            self.sample_column_cached(wx, wz)
                        };
                        let v = self.voxel_at(col, wx, wy, wz);
                        v.is_collidable() && !v.is_fluid()
                    }
                };

                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let column = columns[column_index(x, z)];
                        let world_x = chunk_origin.x + x as i32;
                        let world_z = chunk_origin.z + z as i32;

                        for y in 0..CHUNK_SIZE {
                            let world_y = chunk_origin.y + y as i32;
                            if world_y <= column.terrain_height {
                                continue;
                            }
                            let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                            let voxel = voxels[idx];
                            if !voxel.is_collidable()
                                || voxel.is_fluid()
                                || voxel.is_shape_locked()
                                || voxel.is_crystal_solid_block()
                            {
                                continue;
                            }

                            let above_solid =
                                is_solid_world(&voxels, world_x, world_y + 1, world_z);
                            let below_solid =
                                is_solid_world(&voxels, world_x, world_y - 1, world_z);
                            let air_east = !is_solid_world(&voxels, world_x + 1, world_y, world_z);
                            let air_west = !is_solid_world(&voxels, world_x - 1, world_y, world_z);
                            let air_south = !is_solid_world(&voxels, world_x, world_y, world_z + 1);
                            let air_north = !is_solid_world(&voxels, world_x, world_y, world_z - 1);
                            let air_count = (air_east as u8)
                                + (air_west as u8)
                                + (air_south as u8)
                                + (air_north as u8);

                            if air_count == 0 {
                                continue;
                            }

                            let h = (world_x.wrapping_mul(374_761_393)
                                ^ world_y.wrapping_mul(265_443_576)
                                ^ world_z.wrapping_mul(668_265_263)
                                ^ (self.seed as i32))
                                .unsigned_abs()
                                % 100;

                            if !above_solid && below_solid {
                                if air_count == 1 {
                                    let stair_orient = if air_east {
                                        1 // Air at +X, step ascends toward -X
                                    } else if air_west {
                                        0 // Air at -X, step ascends toward +X
                                    } else if air_south {
                                        3 // Air at +Z, step ascends toward -Z
                                    } else {
                                        2 // Air at -Z, step ascends toward +Z
                                    };
                                    if h < 45 {
                                        chunk_shapes.push((idx, BlockShape::Stair, stair_orient));
                                    } else if h < 80 {
                                        chunk_shapes.push((idx, BlockShape::Slab, 0));
                                    }
                                } else if air_count == 2 {
                                    // Outer corner (adjacent air faces, not a thin wall)
                                    let is_opposite =
                                        (air_east && air_west) || (air_north && air_south);
                                    if !is_opposite && h < 75 {
                                        chunk_shapes.push((idx, BlockShape::Slab, 0));
                                    }
                                } else if air_count >= 3 && h < 65 {
                                    chunk_shapes.push((idx, BlockShape::Slab, 0));
                                }
                            } else if above_solid && !below_solid && air_count == 1 && h < 40 {
                                let inv_orient = if air_east {
                                    5
                                } else if air_west {
                                    4
                                } else if air_south {
                                    7
                                } else {
                                    6
                                };
                                if h < 25 {
                                    chunk_shapes.push((idx, BlockShape::Stair, inv_orient));
                                } else {
                                    chunk_shapes.push((idx, BlockShape::Slab, 1));
                                }
                            }
                        }
                    }
                }
            }

            // Subterranean Ores & Crystal Geodes Generation
            ores::populate_chunk_ores(&mut voxels, chunk_origin, self.seed);
            ores::populate_chunk_geodes(&mut voxels, chunk_origin, self.seed);

            // Cave interior shaping, crystal clusters, and speleothems only run when
            // subterranean air actually exists in this chunk (skips 100% solid stone chunks).
            if has_subterranean_air {
                // Cave interior shaping: clamp border reads directly to chunk array bounds
                let get_v = |voxels_buf: &[Voxel], lx: i32, ly: i32, lz: i32| -> Voxel {
                    let cx = lx.clamp(0, CHUNK_SIZE as i32 - 1) as usize;
                    let cy = ly.clamp(0, CHUNK_SIZE as i32 - 1) as usize;
                    let cz = lz.clamp(0, CHUNK_SIZE as i32 - 1) as usize;
                    voxels_buf[cx + cz * CHUNK_SIZE + cy * CHUNK_SIZE * CHUNK_SIZE]
                };

                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let column = columns[column_index(x, z)];

                        for y in 0..CHUNK_SIZE {
                            let world_y = chunk_origin.y + y as i32;
                            // Subterranean spaces strictly below surface terrain height
                            if world_y >= column.terrain_height {
                                continue;
                            }

                            let idx = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE;
                            let voxel = voxels[idx];
                            if !voxel.is_collidable()
                                || voxel.is_unbreakable()
                                || voxel.is_crystal_solid_block()
                            {
                                continue;
                            }

                            let world_x = chunk_origin.x + x as i32;
                            let world_z = chunk_origin.z + z as i32;

                            let above = get_v(&voxels, x as i32, y as i32 + 1, z as i32);
                            let has_air_neighbor = above == Voxel::Air
                                || get_v(&voxels, x as i32 - 1, y as i32, z as i32) == Voxel::Air
                                || get_v(&voxels, x as i32 + 1, y as i32, z as i32) == Voxel::Air
                                || get_v(&voxels, x as i32, y as i32, z as i32 - 1) == Voxel::Air
                                || get_v(&voxels, x as i32, y as i32, z as i32 + 1) == Voxel::Air;

                            if !has_air_neighbor {
                                continue;
                            }

                            // Unique Subterranean Cavern Chamber Lining (Cave Karst & Cave Calcite)
                            if matches!(
                                voxel,
                                Voxel::Dolomite
                                    | Voxel::Limestone
                                    | Voxel::Marble
                                    | Voxel::Andesite
                            ) {
                                let cavern_noise = crate::core::noise::gradient_noise_3d(
                                    world_x as f32 * 0.024,
                                    world_y as f32 * 0.032,
                                    world_z as f32 * 0.024,
                                    self.seed.wrapping_add(88_112),
                                );

                                if cavern_noise > 0.45
                                    && matches!(
                                        voxel,
                                        Voxel::Dolomite | Voxel::Limestone | Voxel::Marble
                                    )
                                {
                                    let rubble_roll = (world_x ^ world_y ^ world_z) & 7;
                                    voxels[idx] = if rubble_roll == 0 {
                                        Voxel::FracturedCaveKarst
                                    } else {
                                        Voxel::CaveKarst
                                    };
                                } else {
                                    let calcite_noise = crate::core::noise::gradient_noise_3d(
                                        world_x as f32 * 0.026,
                                        world_y as f32 * 0.036,
                                        world_z as f32 * 0.026,
                                        self.seed.wrapping_add(66_331),
                                    );
                                    if calcite_noise > 0.52
                                        && matches!(
                                            voxel,
                                            Voxel::Dolomite | Voxel::Limestone | Voxel::Andesite
                                        )
                                    {
                                        let rubble_roll = (world_x ^ world_y ^ world_z) & 7;
                                        voxels[idx] = if rubble_roll == 0 {
                                            Voxel::FracturedCaveCalcite
                                        } else {
                                            Voxel::CaveCalcite
                                        };
                                    }
                                }
                            }

                            if above == Voxel::Air {
                                // Cave floor / shelf: use bottom slabs on gentle 1-block steps
                                let d_west = if !get_v(&voxels, x as i32 - 1, y as i32, z as i32)
                                    .is_collidable()
                                {
                                    if get_v(&voxels, x as i32 - 1, y as i32 - 1, z as i32)
                                        .is_collidable()
                                    {
                                        -1
                                    } else {
                                        -2
                                    }
                                } else {
                                    0
                                };

                                let d_east = if !get_v(&voxels, x as i32 + 1, y as i32, z as i32)
                                    .is_collidable()
                                {
                                    if get_v(&voxels, x as i32 + 1, y as i32 - 1, z as i32)
                                        .is_collidable()
                                    {
                                        -1
                                    } else {
                                        -2
                                    }
                                } else {
                                    0
                                };

                                let d_north = if !get_v(&voxels, x as i32, y as i32, z as i32 - 1)
                                    .is_collidable()
                                {
                                    if get_v(&voxels, x as i32, y as i32 - 1, z as i32 - 1)
                                        .is_collidable()
                                    {
                                        -1
                                    } else {
                                        -2
                                    }
                                } else {
                                    0
                                };

                                let d_south = if !get_v(&voxels, x as i32, y as i32, z as i32 + 1)
                                    .is_collidable()
                                {
                                    if get_v(&voxels, x as i32, y as i32 - 1, z as i32 + 1)
                                        .is_collidable()
                                    {
                                        -1
                                    } else {
                                        -2
                                    }
                                } else {
                                    0
                                };

                                let has_cliff =
                                    d_west <= -2 || d_east <= -2 || d_north <= -2 || d_south <= -2;
                                if !has_cliff {
                                    let lower_count = (if d_west == -1 { 1 } else { 0 })
                                        + (if d_east == -1 { 1 } else { 0 })
                                        + (if d_north == -1 { 1 } else { 0 })
                                        + (if d_south == -1 { 1 } else { 0 });

                                    if lower_count == 1 || lower_count == 2 {
                                        // Slabs for cave floor transitions (avoids excessive staircases)
                                        chunk_shapes.push((idx, BlockShape::Slab, 0));
                                    }
                                }
                            } else {
                                // Cave wall shaping: soften wall bases, arched ceiling junctions, vertical wall recesses, and chamfered corners
                                let air_west =
                                    get_v(&voxels, x as i32 - 1, y as i32, z as i32) == Voxel::Air;
                                let air_east =
                                    get_v(&voxels, x as i32 + 1, y as i32, z as i32) == Voxel::Air;
                                let air_north =
                                    get_v(&voxels, x as i32, y as i32, z as i32 - 1) == Voxel::Air;
                                let air_south =
                                    get_v(&voxels, x as i32, y as i32, z as i32 + 1) == Voxel::Air;

                                let wall_openings = air_west as u8
                                    + air_east as u8
                                    + air_north as u8
                                    + air_south as u8;
                                let wall_noise = crate::core::noise::gradient_noise_3d(
                                    world_x as f32 * 0.18,
                                    world_y as f32 * 0.18,
                                    world_z as f32 * 0.18,
                                    self.seed.wrapping_add(44_221),
                                );

                                if wall_openings == 1 {
                                    let (
                                        air_dx,
                                        air_dz,
                                        stair_orient,
                                        inv_stair_orient,
                                        vert_slab_orient,
                                    ) = if air_west {
                                        (-1, 0, 0, 4, 5) // air is -X, slab attached to +X rock wall -> orient 5
                                    } else if air_east {
                                        (1, 0, 1, 5, 4) // air is +X, slab attached to -X rock wall -> orient 4
                                    } else if air_north {
                                        (0, -1, 2, 6, 3) // air is -Z, slab attached to +Z rock wall -> orient 3
                                    } else {
                                        (0, 1, 3, 7, 2) // air is +Z, slab attached to -Z rock wall -> orient 2
                                    };

                                    let below_cave_floor = get_v(
                                        &voxels,
                                        x as i32 + air_dx,
                                        y as i32 - 1,
                                        z as i32 + air_dz,
                                    )
                                    .is_collidable();
                                    let above_cave_roof = get_v(
                                        &voxels,
                                        x as i32 + air_dx,
                                        y as i32 + 1,
                                        z as i32 + air_dz,
                                    )
                                    .is_collidable();

                                    if below_cave_floor && wall_noise > 0.10 {
                                        // Wall base: rock footing using stairs or slabs
                                        if wall_noise > 0.45 {
                                            chunk_shapes.push((
                                                idx,
                                                BlockShape::Stair,
                                                stair_orient,
                                            ));
                                        } else {
                                            chunk_shapes.push((idx, BlockShape::Slab, 0));
                                        }
                                    } else if above_cave_roof && wall_noise > 0.25 {
                                        // Wall top: arched ceiling overhang
                                        if wall_noise > 0.50 {
                                            chunk_shapes.push((
                                                idx,
                                                BlockShape::Stair,
                                                inv_stair_orient,
                                            ));
                                        } else {
                                            chunk_shapes.push((idx, BlockShape::Slab, 1));
                                        }
                                    } else if (0.35..0.68).contains(&wall_noise) {
                                        // Mid-wall: vertical slabs create rocky recesses and wall relief in caves & ravines!
                                        chunk_shapes.push((
                                            idx,
                                            BlockShape::Slab,
                                            vert_slab_orient,
                                        ));
                                    }
                                } else if wall_openings == 2 {
                                    // Outer turning corners in caves and ravines: side-aligned column slabs chamfer sharp 90-degree rock edges
                                    let corner_orient = if air_west && air_north {
                                        Some(4) // Solid corner at MaxX, MaxZ (+X, +Z)
                                    } else if air_east && air_north {
                                        Some(3) // Solid corner at MinX, MaxZ (-X, +Z)
                                    } else if air_west && air_south {
                                        Some(2) // Solid corner at MaxX, MinZ (+X, -Z)
                                    } else if air_east && air_south {
                                        Some(1) // Solid corner at MinX, MinZ (-X, -Z)
                                    } else {
                                        None // Opposite walls (corridor)
                                    };

                                    if let Some(orient) = corner_orient
                                        && wall_noise > 0.15
                                    {
                                        chunk_shapes.push((idx, BlockShape::Column, orient));
                                    }
                                }
                            }
                        }
                    }
                }

                // Populate 6-direction crystal clusters (floor, ceiling, and horizontal on walls)
                // inside crystal geode chambers and pink salt caverns
                ores::populate_chunk_crystal_clusters(
                    &mut voxels,
                    &mut chunk_shapes,
                    chunk_origin,
                    self.seed,
                );

                // Populate stalactites and stalagmites strictly on/under their matching host rock
                // (Calcite on Calcite, Karst on Karst, BlueIce on Ice, SaltStraw on PinkSalt, ObsidianSpike on Obsidian)
                ores::populate_chunk_speleothems(
                    &mut voxels,
                    &mut chunk_shapes,
                    chunk_origin,
                    self.seed,
                );
            }
        }

        // Phase 20.5: Multi-Chunk Procedural Trees & Arboreal Crops (run BEFORE flora so pebbles,
        // rubble, and wild crops never spawn underneath tree trunks or buttress roots!)
        let wrote_trees = trees::populate_chunk_trees(
            &mut voxels,
            &mut chunk_shapes,
            chunk_origin,
            self.seed,
            |wx, wz| {
                let lx = wx - chunk_origin.x;
                let lz = wz - chunk_origin.z;
                if (0..CHUNK_SIZE as i32).contains(&lx) && (0..CHUNK_SIZE as i32).contains(&lz) {
                    columns[column_index(lx as usize, lz as usize)]
                } else {
                    self.sample_column_cached(wx, wz)
                }
            },
            |wx, wy, wz, col| {
                // Reject cave entrances / hollow overhangs so trees never float over caves
                if self
                    .caves
                    .is_cave_at_point(wx as f32, wy as f32, wz as f32, self.seed)
                    || self
                        .caves
                        .is_cave_at_point(wx as f32, (wy - 1) as f32, wz as f32, self.seed)
                {
                    return None;
                }
                let g = self.voxel_at(col, wx, wy, wz);
                if g.is_solid_opaque() && !g.is_fluid() && !g.is_empty() {
                    Some(g)
                } else {
                    None
                }
            },
        );
        if wrote_trees {
            has_solid_voxels = true;
        }

        // Surface Cobble/Mossy Rock Clusters & Ground Leaf Bushes (after trees so they never
        // collide with tree trunks, before flora so grass/flowers scatter cleanly around them)
        let wrote_surface_features = trees::populate_chunk_boulders_and_bushes(
            &mut voxels,
            &mut chunk_shapes,
            chunk_origin,
            self.seed,
            |wx, wz| {
                let lx = wx - chunk_origin.x;
                let lz = wz - chunk_origin.z;
                if (0..CHUNK_SIZE as i32).contains(&lx) && (0..CHUNK_SIZE as i32).contains(&lz) {
                    columns[column_index(lx as usize, lz as usize)]
                } else {
                    self.sample_column_cached(wx, wz)
                }
            },
            |wx, wy, wz, col| {
                if self
                    .caves
                    .is_cave_at_point(wx as f32, wy as f32, wz as f32, self.seed)
                    || self
                        .caves
                        .is_cave_at_point(wx as f32, (wy - 1) as f32, wz as f32, self.seed)
                {
                    return None;
                }
                let g = self.voxel_at(col, wx, wy, wz);
                if g.is_solid_opaque() && !g.is_fluid() && !g.is_empty() {
                    Some(g)
                } else {
                    None
                }
            },
        );
        if wrote_surface_features {
            has_solid_voxels = true;
        }

        // Phase 20 (Blocks 1..4): Surface Scatter, Flora, Wild Crops, Marine & Cave Features
        let shapes_before_flora = chunk_shapes.len();
        flora::populate_chunk_flora(
            &mut voxels,
            &mut chunk_shapes,
            &columns,
            chunk_origin,
            self.seed,
            |wx, wy, wz, col| self.voxel_at(col, wx, wy, wz),
        );
        if !has_solid_voxels && chunk_shapes.len() > shapes_before_flora {
            has_solid_voxels = true;
        }

        if !has_solid_voxels {
            return Chunk::filled(Voxel::Air);
        }

        let mut chunk = Chunk::from_voxels(voxels);
        for (idx, shape, orientation) in chunk_shapes {
            let (lx, ly, lz) = Chunk::index_to_xyz(idx);
            chunk.set_shape(lx, ly, lz, shape, orientation);
        }

        let is_subterranean = chunk_max_y < minimum_filled_height;
        chunk.set_subterranean(is_subterranean);

        chunk
    }

    pub fn effective_sea_level(&self) -> i32 {
        logical_block_top(self.sea_level)
    }

    pub fn continuous_height_and_biome_with_climate(
        &self,
        world_x: i32,
        world_z: i32,
    ) -> (f32, BiomeType, bool, ClimateSample) {
        if let Some(ref map) = self.adventure_map {
            return map.sample_column_data(world_x, world_z, self.seed);
        }

        let logical_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS);
        let logical_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS);
        let sample_x = logical_block_sample_position(logical_x);
        let sample_z = logical_block_sample_position(logical_z);

        let mut climate = self.climate.sample_dithered(sample_x, sample_z, self.seed);

        let mut raw_height = self.natural_height_at(world_x as f32, world_z as f32, &climate);

        let (mut river_factor, is_river_path, _center_dx, _center_dz) =
            self.river_sample_with_offset(sample_x, sample_z);
        let sea_level = self.effective_sea_level();
        let sea_level_f = sea_level as f32;

        // Rivers ONLY generate in flat, lowland terrain (plains, valleys, meadows, coastal areas).
        // Strictly forbidden on mountains, hills, and high terrain to prevent deep chasms/wormholes.
        let height_suppress = ((raw_height - 23.0) / 8.0).clamp(0.0, 1.0);
        let mountain_suppress = ((climate.continentalness - 0.25) / 0.10).clamp(0.0, 1.0);
        let ridge_suppress = ((climate.ridges - 0.25) / 0.15).clamp(0.0, 1.0);
        let total_suppress =
            (height_suppress.max(mountain_suppress).max(ridge_suppress)).clamp(0.0, 1.0);
        river_factor *= 1.0 - total_suppress;

        // Procedural Rivers:
        // Rivers strictly generate inland (continentalness >= -0.04) and carve valleys down to sea level.
        // River water is strictly at sea level (Y = 12), ensuring 100% horizontal, flat, stable water with zero uphill bugs!
        let is_inland = climate.continentalness >= -0.04;
        let is_river = is_river_path && is_inland && river_factor > 0.04;
        let is_underground_river = false;

        if is_river {
            let water_y = sea_level_f;
            let bed_depth = 2.5 + river_factor * self.river_depth;
            let target_bed = water_y - bed_depth;

            if river_factor > 0.12 {
                let channel_t = ((river_factor - 0.12) / 0.88).clamp(0.0, 1.0);
                let carve_t = channel_t * channel_t * (3.0 - 2.0 * channel_t);
                let carved_bed = lerp(raw_height, target_bed, (carve_t * 1.35).min(1.0));
                raw_height = raw_height.min(carved_bed);
            } else {
                // Bank levee: terrain bordering river is guaranteed above sea level water
                raw_height = raw_height.max(water_y + 1.0);
            }

            if climate.continentalness < 0.18
                && climate.humidity > 0.35
                && raw_height <= (sea_level + 2) as f32
            {
                climate.biome = BiomeType::OrchidSwamp;
            }
        } else if let Some((oasis_water_y, carve_depth, t_dist)) =
            self.sample_desert_oasis(world_x, world_z, raw_height, &climate)
        {
            if t_dist < 0.90 {
                let target_bed = oasis_water_y as f32 - carve_depth;
                raw_height = raw_height.min(target_bed);
            } else if t_dist <= 1.25 {
                raw_height = raw_height.max((oasis_water_y + 1) as f32);
            }
            climate.biome = BiomeType::DesertOasis;
        } else if let Some((lake_water_y, carve_depth, t_dist)) =
            self.sample_organic_lake(world_x, world_z, raw_height, &climate)
        {
            if t_dist < 0.90 {
                // Lake interior: carve bowl for deep water
                let target_bed = lake_water_y as f32 - carve_depth;
                raw_height = raw_height.min(target_bed);
            } else {
                // Lake rim / levee (0.90 <= t_dist <= 1.25):
                // Enforce that the entire perimeter of the lake stands at least 1 block ABOVE water level!
                // This eliminates open corners and keeps water completely enclosed.
                raw_height = raw_height.max((lake_water_y + 1) as f32);
            }
        }

        (raw_height, climate.biome, is_underground_river, climate)
    }

    /// Computes both the target river water surface elevation and carved bed elevation.
    /// In nature, rivers flow downhill along the continental gradient towards the sea.
    /// Water level is strictly monotonically decreasing towards the ocean (never flows uphill over local hills).
    pub fn river_water_and_bed(
        &self,
        _centerline_h: f32,
        river_factor: f32,
        _uncarved_h: f32,
        _climate: &ClimateSample,
    ) -> (f32, f32) {
        let sea_level_f = self.effective_sea_level() as f32; // Strictly flat sea_level (12.0)
        let water_y = sea_level_f;
        let bed_depth = 2.5 + river_factor * self.river_depth;
        let target_bed = water_y - bed_depth;
        (water_y, target_bed)
    }

    /// Computes the stabilized hydraulic reach water elevation for an inland river.
    pub fn river_water_level(&self, raw_height: f32, climate: &ClimateSample) -> f32 {
        let (water_y, _) = self.river_water_and_bed(raw_height, 0.5, raw_height, climate);
        water_y
    }

    pub fn continuous_height_and_biome(
        &self,
        world_x: i32,
        world_z: i32,
    ) -> (f32, BiomeType, bool) {
        let (raw_height, final_biome, is_underground_river, _) =
            self.continuous_height_and_biome_with_climate(world_x, world_z);
        (raw_height, final_biome, is_underground_river)
    }

    /// Evaluates 4 terrain column heights, biomes, and climates simultaneously using 4-wide SIMD noise.
    pub fn continuous_height_and_biome_x4(
        &self,
        world_xs: [i32; 4],
        world_zs: [i32; 4],
    ) -> [(f32, BiomeType, bool, ClimateSample); 4] {
        if let Some(ref map) = self.adventure_map {
            return [
                map.sample_column_data(world_xs[0], world_zs[0], self.seed),
                map.sample_column_data(world_xs[1], world_zs[1], self.seed),
                map.sample_column_data(world_xs[2], world_zs[2], self.seed),
                map.sample_column_data(world_xs[3], world_zs[3], self.seed),
            ];
        }

        let logical_x = [
            world_xs[0].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_xs[1].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_xs[2].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_xs[3].div_euclid(LOGICAL_BLOCK_VOXELS),
        ];
        let logical_z = [
            world_zs[0].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_zs[1].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_zs[2].div_euclid(LOGICAL_BLOCK_VOXELS),
            world_zs[3].div_euclid(LOGICAL_BLOCK_VOXELS),
        ];

        let sample_x = Simd4f([
            logical_block_sample_position(logical_x[0]),
            logical_block_sample_position(logical_x[1]),
            logical_block_sample_position(logical_x[2]),
            logical_block_sample_position(logical_x[3]),
        ]);
        let sample_z = Simd4f([
            logical_block_sample_position(logical_z[0]),
            logical_block_sample_position(logical_z[1]),
            logical_block_sample_position(logical_z[2]),
            logical_block_sample_position(logical_z[3]),
        ]);

        let climates = self
            .climate
            .sample_dithered_x4(sample_x, sample_z, self.seed);
        let sea_level = self.effective_sea_level();

        let wx_f32 = Simd4f([
            world_xs[0] as f32,
            world_xs[1] as f32,
            world_xs[2] as f32,
            world_xs[3] as f32,
        ]);
        let wz_f32 = Simd4f([
            world_zs[0] as f32,
            world_zs[1] as f32,
            world_zs[2] as f32,
            world_zs[3] as f32,
        ]);

        let raw_heights = self.natural_height_at_x4(wx_f32, wz_f32, &climates);
        let (river_factors, is_river_paths, _center_dxs, _center_dzs) =
            self.river_sample_with_offset_x4(sample_x, sample_z);

        let mut res = [(0.0f32, BiomeType::default(), false, climates[0]); 4];

        for i in 0..4 {
            let mut raw_height = raw_heights.0[i];
            let mut climate = climates[i];
            let mut river_factor = river_factors[i];
            let is_river_path = is_river_paths[i];

            let sea_level_f = sea_level as f32;
            let height_suppress = ((raw_height - 23.0) / 8.0).clamp(0.0, 1.0);
            let mountain_suppress = ((climate.continentalness - 0.25) / 0.10).clamp(0.0, 1.0);
            let ridge_suppress = ((climate.ridges - 0.25) / 0.15).clamp(0.0, 1.0);
            let total_suppress =
                (height_suppress.max(mountain_suppress).max(ridge_suppress)).clamp(0.0, 1.0);
            river_factor *= 1.0 - total_suppress;

            let is_inland = climate.continentalness >= -0.04;
            let is_river = is_river_path && is_inland && river_factor > 0.04;
            let is_underground_river = false;

            if is_river {
                let water_y = sea_level_f;
                let bed_depth = 2.5 + river_factor * self.river_depth;
                let target_bed = water_y - bed_depth;

                if river_factor > 0.12 {
                    let channel_t = ((river_factor - 0.12) / 0.88).clamp(0.0, 1.0);
                    let carve_t = channel_t * channel_t * (3.0 - 2.0 * channel_t);
                    let carved_bed = lerp(raw_height, target_bed, (carve_t * 1.35).min(1.0));
                    raw_height = raw_height.min(carved_bed);
                } else {
                    raw_height = raw_height.max(water_y + 1.0);
                }

                if climate.continentalness < 0.18
                    && climate.humidity > 0.35
                    && raw_height <= (sea_level + 2) as f32
                {
                    climate.biome = BiomeType::OrchidSwamp;
                }
            } else if let Some((oasis_water_y, carve_depth, t_dist)) =
                self.sample_desert_oasis(world_xs[i], world_zs[i], raw_height, &climate)
            {
                if t_dist < 0.90 {
                    let target_bed = oasis_water_y as f32 - carve_depth;
                    raw_height = raw_height.min(target_bed);
                } else if t_dist <= 1.25 {
                    raw_height = raw_height.max((oasis_water_y + 1) as f32);
                }
                climate.biome = BiomeType::DesertOasis;
            } else if let Some((lake_water_y, carve_depth, t_dist)) =
                self.sample_organic_lake(world_xs[i], world_zs[i], raw_height, &climate)
            {
                if t_dist < 0.90 {
                    let target_bed = lake_water_y as f32 - carve_depth;
                    raw_height = raw_height.min(target_bed);
                } else {
                    raw_height = raw_height.max((lake_water_y + 1) as f32);
                }
            }

            res[i] = (raw_height, climate.biome, is_underground_river, climate);
        }

        res
    }

    pub fn build_terrain_column(
        &self,
        world_x: i32,
        world_z: i32,
        raw_height: f32,
        final_biome: BiomeType,
        is_underground_river: bool,
        climate: ClimateSample,
    ) -> TerrainColumn {
        let sea_level = self.effective_sea_level();
        let terrain_height = raw_height.round() as i32;
        let mut surface_shape = BlockShape::Full;
        let mut shape_orientation = 0u8;

        let (is_river, is_lake, water_level, river_factor, lake_sample) =
            if let Some(ref map) = self.adventure_map {
                let is_river = matches!(final_biome, BiomeType::Rapids | BiomeType::Estuary);
                let is_lake = final_biome == BiomeType::CalmLake;
                let water_level = if let Some(adv_water_y) = map.sample_water_level(world_x, world_z)
                {
                    if terrain_height < adv_water_y {
                        Some(adv_water_y)
                    } else {
                        None
                    }
                } else if terrain_height < sea_level {
                    Some(sea_level)
                } else {
                    None
                };
                let river_factor = if is_river { 0.85 } else { 0.0 };
                let lake_sample = if is_lake {
                    water_level.map(|wy| (wy, 4.5, 0.25))
                } else {
                    None
                };
                (is_river, is_lake, water_level, river_factor, lake_sample)
            } else {
                let logical_x = world_x.div_euclid(LOGICAL_BLOCK_VOXELS);
                let logical_z = world_z.div_euclid(LOGICAL_BLOCK_VOXELS);
                let sample_x = logical_block_sample_position(logical_x);
                let sample_z = logical_block_sample_position(logical_z);

                let (mut river_factor, is_river_path, _center_dx, _center_dz) =
                    self.river_sample_with_offset(sample_x, sample_z);
                let uncarved_height =
                    self.natural_height_at(world_x as f32, world_z as f32, &climate);
                let height_suppress = ((uncarved_height - 23.0) / 8.0).clamp(0.0, 1.0);
                let mountain_suppress = ((climate.continentalness - 0.25) / 0.10).clamp(0.0, 1.0);
                let ridge_suppress = ((climate.ridges - 0.25) / 0.15).clamp(0.0, 1.0);
                let total_suppress =
                    (height_suppress.max(mountain_suppress).max(ridge_suppress)).clamp(0.0, 1.0);
                river_factor *= 1.0 - total_suppress;

                let is_inland = climate.continentalness >= -0.04;
                let is_river =
                    is_river_path && is_inland && river_factor > 0.04 && !is_underground_river;
                let oasis_sample =
                    self.sample_desert_oasis(world_x, world_z, raw_height, &climate);
                let lake_sample = oasis_sample
                    .or_else(|| self.sample_organic_lake(world_x, world_z, raw_height, &climate));
                let is_lake = lake_sample.is_some_and(|(_, _, t_dist)| t_dist <= 1.25);

                let water_level = if terrain_height < sea_level {
                    Some(sea_level)
                } else if let Some((lake_water_y, _, t_dist)) = lake_sample {
                    if t_dist < 0.90 && terrain_height < lake_water_y {
                        Some(lake_water_y)
                    } else {
                        None
                    }
                } else {
                    None
                };
                (is_river, is_lake, water_level, river_factor, lake_sample)
            };

        // Ensure open ocean/sea bodies always resolve to an authentic ocean biome
        // (instead of reporting the nearest terrestrial surface biome on the HUD/map).
        let is_already_aquatic = matches!(
            final_biome,
            BiomeType::WarmOcean
                | BiomeType::ColdOcean
                | BiomeType::CoralReefs
                | BiomeType::SeagrassMeadows
                | BiomeType::KelpForests
                | BiomeType::OceanTrench
                | BiomeType::Rapids
                | BiomeType::CalmLake
                | BiomeType::Estuary
                | BiomeType::Caldera
                | BiomeType::GlacialChasm
                | BiomeType::WillowMarsh
                | BiomeType::MangroveSwamp
                | BiomeType::OrchidSwamp
        );
        let final_biome =
            if terrain_height < sea_level && !is_river && !is_lake && !is_already_aquatic {
                let ocean_depth = sea_level - terrain_height;
                if ocean_depth >= 18 {
                    BiomeType::OceanTrench
                } else if climate.temperature < -0.15 {
                    if ocean_depth <= 15 {
                        BiomeType::KelpForests
                    } else {
                        BiomeType::ColdOcean
                    }
                } else {
                    let reef_noise = crate::core::noise::gradient_noise_2d(
                        world_x as f32 * 0.012,
                        world_z as f32 * 0.012,
                        self.seed.wrapping_add(0xC07A_1001),
                    );
                    if climate.temperature > 0.05 && ocean_depth <= 14 && reef_noise > 0.22 {
                        BiomeType::CoralReefs
                    } else if ocean_depth <= 14 && reef_noise < -0.10 {
                        BiomeType::SeagrassMeadows
                    } else {
                        BiomeType::WarmOcean
                    }
                }
            } else {
                final_biome
            };

        // Horizontal beach width: coastal transition strip directly bordering water
        let shore_noise = crate::core::noise::gradient_noise_2d(
            world_x as f32 * 0.12,
            world_z as f32 * 0.12,
            self.seed.wrapping_add(SEED_OFFSET_BEACH_NOISE),
        );
        let beach_band = if self.adventure_map.is_some() { 7 } else { 5 };
        let max_beach_y = sea_level + beach_band + (shore_noise * 3.0).round() as i32;

        let is_coastal_continentalness =
            climate.continentalness >= -0.18 && climate.continentalness <= 0.05;
        let is_coastal_zone = matches!(
            final_biome,
            BiomeType::SandyShores
                | BiomeType::GravelBeach
                | BiomeType::DuneBeach
                | BiomeType::CraggyBeach
                | BiomeType::TidePools
                | BiomeType::TropicalIsland
                | BiomeType::CoralIsland
        ) || is_coastal_continentalness;

        let is_beach =
            is_coastal_zone && terrain_height >= sea_level - 2 && terrain_height <= max_beach_y;

        let mut is_cliff = false;

        let is_submerged = water_level.is_some()
            || lake_sample.is_some_and(|(ly, _, td)| td < 0.95 && terrain_height <= ly);

        // Universal smooth transitions across ALL terrain (dry land, beaches, riverbanks,
        // shallow waterlines, and submerged riverbeds/lakebeds/seafloors). Underwater slabs
        // and stairs are automatically waterlogged by the mesher so the water surface flows
        // seamlessly over them without air pockets.
        {
            let get_effective_height = |(h, _, _): (f32, BiomeType, bool)| -> i32 {
                let th = h.round() as i32;
                if !is_submerged && terrain_height > sea_level && th < sea_level {
                    sea_level
                } else {
                    th
                }
            };

            let h_east =
                get_effective_height(self.continuous_height_and_biome(world_x + 1, world_z));
            let h_west =
                get_effective_height(self.continuous_height_and_biome(world_x - 1, world_z));
            let h_south =
                get_effective_height(self.continuous_height_and_biome(world_x, world_z + 1));
            let h_north =
                get_effective_height(self.continuous_height_and_biome(world_x, world_z - 1));

            let d_east = h_east - terrain_height;
            let d_west = h_west - terrain_height;
            let d_south = h_south - terrain_height;
            let d_north = h_north - terrain_height;

            let max_delta = d_east
                .abs()
                .max(d_west.abs())
                .max(d_south.abs())
                .max(d_north.abs());

            let is_cliff_biome = matches!(
                final_biome,
                BiomeType::CoastalCliffs
                    | BiomeType::MonolithCliffs
                    | BiomeType::FrozenCliffs
                    | BiomeType::RedrockCanyon
                    | BiomeType::PaintedMesas
                    | BiomeType::AmethystCanyon
                    | BiomeType::DeepGorge
                    | BiomeType::WindyPass
                    | BiomeType::DesertSpires
                    | BiomeType::EmeraldFoothills
                    | BiomeType::GranitePlateau
                    | BiomeType::MistyCrags
                    | BiomeType::AlpinePeaks
                    | BiomeType::JaggedPeaks
                    | BiomeType::VolcanicPeaks
            );

            // Slope-Aware Surface Rules apply on dry land above water level
            if !is_submerged && terrain_height > sea_level {
                is_cliff = if is_cliff_biome {
                    max_delta >= 2
                } else {
                    max_delta >= 3
                };
            }

            // Retain solid blocks on sections of higher terrain for blocky cliffs and stepped terraces
            let edge_noise = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.08,
                world_z as f32 * 0.08,
                self.seed.wrapping_add(88_991),
            );

            // Balance crisp full-block ledges (~45%) and smooth slab/stair slopes (~55%)
            // consistently across all elevations (forests, beaches, riverbanks, and ocean floors).
            let keep_solid = edge_noise > 0.05;

            if !is_cliff && !keep_solid {
                let lower_count = (if d_west == -1 { 1 } else { 0 })
                    + (if d_east == -1 { 1 } else { 0 })
                    + (if d_north == -1 { 1 } else { 0 })
                    + (if d_south == -1 { 1 } else { 0 });

                if lower_count == 1 {
                    // Surface stairs restored at a natural frequency (~35%), slabs for remainder
                    let stair_hash = ((world_x.wrapping_mul(374_761_393)
                        ^ world_z.wrapping_mul(668_265_263)
                        ^ (self.seed as i32))
                        .abs()
                        % 100) as f32
                        / 100.0;

                    if stair_hash < 0.35 {
                        surface_shape = BlockShape::Stair;
                        // Orient step towards the higher side so it ascends away from the lower neighbor
                        shape_orientation = if d_east == -1 {
                            1 // Drop to East (+X): step is at -X
                        } else if d_west == -1 {
                            0 // Drop to West (-X): step is at +X
                        } else if d_south == -1 {
                            3 // Drop to South (+Z): step is at -Z
                        } else {
                            2 // Drop to North (-Z): step is at +Z
                        };
                    } else {
                        surface_shape = BlockShape::Slab;
                        shape_orientation = 0; // Bottom Slab
                    }
                } else if lower_count == 2 {
                    // Corner drops use clean bottom slabs
                    surface_shape = BlockShape::Slab;
                    shape_orientation = 0; // Bottom Slab
                }
            }
        }

        let is_beach = is_beach && !is_cliff;

        TerrainColumn {
            terrain_height,
            water_level,
            biome: final_biome,
            is_beach,
            is_cliff,
            is_underground_river,
            is_river,
            is_lake,
            river_factor,
            climate,
            surface_shape,
            shape_orientation,
        }
    }

    pub fn sample_column(&self, world_x: i32, world_z: i32) -> TerrainColumn {
        let (raw_height, final_biome, is_underground_river, climate) =
            self.continuous_height_and_biome_with_climate(world_x, world_z);
        self.build_terrain_column(
            world_x,
            world_z,
            raw_height,
            final_biome,
            is_underground_river,
            climate,
        )
    }

    /// Evaluates 4 terrain columns simultaneously using 4-wide SIMD noise.
    pub fn sample_columns_x4(&self, world_xs: [i32; 4], world_zs: [i32; 4]) -> [TerrainColumn; 4] {
        let samples = self.continuous_height_and_biome_x4(world_xs, world_zs);
        let mut cols = [TerrainColumn::default(); 4];
        for i in 0..4 {
            let (raw_height, final_biome, is_underground_river, climate) = samples[i];
            cols[i] = self.build_terrain_column(
                world_xs[i],
                world_zs[i],
                raw_height,
                final_biome,
                is_underground_river,
                climate,
            );
        }
        cols
    }

    /// Evaluates organic, non-circular natural lakes and ponds.
    /// Lakes only form in natural topographic depressions, valley basins, or flat plateaus.
    /// Never generates harsh circular craters on steep hillsides.
    pub fn sample_organic_lake(
        &self,
        world_x: i32,
        world_z: i32,
        natural_height: f32,
        _climate: &ClimateSample,
    ) -> Option<(i32, f32, f32)> {
        if self.lake_abundance <= 0.01 {
            return None;
        }

        let sea_level = self.effective_sea_level();
        let sea_level_f = sea_level as f32;
        if natural_height <= sea_level_f + 2.0 {
            return None; // Already ocean / coastal water
        }

        let cell_size = 180;
        let cx = world_x.div_euclid(cell_size);
        let cz = world_z.div_euclid(cell_size);

        let h = crate::core::noise::hash_2d(cx, cz, self.seed.wrapping_add(444_222));
        let prob_thresh = (self.lake_abundance.clamp(0.0, 1.0) * 100.0) as u32;
        if (h % 100) >= prob_thresh {
            return None;
        }

        let offset_x = 45 + ((h >> 8) % (cell_size as u32 - 90)) as i32;
        let offset_z = 45 + ((h >> 16) % (cell_size as u32 - 90)) as i32;
        let center_x = cx * cell_size + offset_x;
        let center_z = cz * cell_size + offset_z;

        // Check local terrain slope: lakes can only exist in flat valley basins or meadows,
        // never on steep mountainsides or sharp hills (reject if slope > 2.2 blocks across 20m)
        let check_dist = 14.0;
        let c_climate = self
            .climate
            .sample_dithered(center_x as f32, center_z as f32, self.seed);
        let center_h = self.natural_height_at(center_x as f32, center_z as f32, &c_climate);
        let h_east =
            self.natural_height_at(center_x as f32 + check_dist, center_z as f32, &c_climate);
        let h_south =
            self.natural_height_at(center_x as f32, center_z as f32 + check_dist, &c_climate);

        let slope_x = (h_east - center_h).abs();
        let slope_z = (h_south - center_h).abs();
        if slope_x > 2.2 || slope_z > 2.2 {
            return None; // Steep slope: physics forbids natural pond from perching on a hillside
        }

        // Domain-warp the lake coordinates with smooth noise to create
        // completely organic, non-circular shorelines with natural bays and coves.
        let wx = world_x as f32;
        let wz = world_z as f32;

        let warp_x = crate::core::noise::gradient_noise_2d(
            wx * 0.025,
            wz * 0.025,
            self.seed.wrapping_add(888_111),
        ) * (self.lake_size * 0.45);

        let warp_z = crate::core::noise::gradient_noise_2d(
            wx * 0.025 + 67.0,
            wz * 0.025 + 67.0,
            self.seed.wrapping_add(888_222),
        ) * (self.lake_size * 0.45);

        let dx = (world_x - center_x) as f32 + warp_x;
        let dz = (world_z - center_z) as f32 + warp_z;
        let dist = (dx * dx + dz * dz).sqrt();

        let shape_noise = crate::core::noise::gradient_noise_2d(
            wx * 0.045,
            wz * 0.045,
            self.seed.wrapping_add(888_333),
        );
        let organic_radius = self.lake_size * (1.0 + shape_noise * 0.35);

        let t_dist = dist / organic_radius;
        if t_dist <= 1.25 {
            let lake_water_y = (center_h - 0.8).round() as i32;
            let carve_depth = if t_dist < 0.90 {
                let inner_t = t_dist / 0.90;
                (1.0 - inner_t * inner_t) * self.lake_depth
            } else {
                0.0
            };
            Some((lake_water_y, carve_depth, t_dist))
        } else {
            None
        }
    }

    /// Evaluates small, water-centered Desert Oasis pockets exclusively inside desert biomes
    /// (`AncientSands`, `DuneSea`, `DesertSpires`). Returns `(water_y, carve_depth, t_dist)`:
    /// - `t_dist < 0.90`: carved oasis water pool
    /// - `0.90..=1.55`: lush palm & grass oasis shoreline ring surrounded by desert sand
    pub fn sample_desert_oasis(
        &self,
        world_x: i32,
        world_z: i32,
        natural_height: f32,
        climate: &ClimateSample,
    ) -> Option<(i32, f32, f32)> {
        if !matches!(
            climate.biome,
            BiomeType::AncientSands
                | BiomeType::DuneSea
                | BiomeType::DesertSpires
                | BiomeType::DesertOasis
        ) {
            return None;
        }

        let sea_level = self.effective_sea_level();
        if natural_height <= sea_level as f32 + 2.0 {
            return None;
        }

        let cell_size = 140;
        let cx = world_x.div_euclid(cell_size);
        let cz = world_z.div_euclid(cell_size);

        let h = crate::core::noise::hash_2d(cx, cz, self.seed.wrapping_add(0x0A51_5777));
        if (h % 100) >= 60 {
            return None;
        }

        let offset_x = 34 + ((h >> 8) % (cell_size as u32 - 68)) as i32;
        let offset_z = 34 + ((h >> 16) % (cell_size as u32 - 68)) as i32;
        let center_x = cx * cell_size + offset_x;
        let center_z = cz * cell_size + offset_z;

        // Fast bounding-box reject before sampling center climate/slope
        if (world_x - center_x).abs() > 36 || (world_z - center_z).abs() > 36 {
            return None;
        }

        let c_climate = self
            .climate
            .sample_dithered(center_x as f32, center_z as f32, self.seed);
        if !matches!(
            c_climate.biome,
            BiomeType::AncientSands | BiomeType::DuneSea | BiomeType::DesertSpires
        ) {
            return None;
        }

        let check_dist = 12.0;
        let center_h = self.natural_height_at(center_x as f32, center_z as f32, &c_climate);
        let h_east =
            self.natural_height_at(center_x as f32 + check_dist, center_z as f32, &c_climate);
        let h_south =
            self.natural_height_at(center_x as f32, center_z as f32 + check_dist, &c_climate);
        if (h_east - center_h).abs() > 3.5 || (h_south - center_h).abs() > 3.5 {
            return None;
        }

        let wx = world_x as f32;
        let wz = world_z as f32;
        let pool_radius = 9.0 + (((h >> 24) & 0xFF) as f32 / 255.0) * 5.0; // 9..14 blocks

        let warp_x = crate::core::noise::gradient_noise_2d(
            wx * 0.04,
            wz * 0.04,
            self.seed.wrapping_add(0x0A51_1111),
        ) * (pool_radius * 0.35);
        let warp_z = crate::core::noise::gradient_noise_2d(
            wx * 0.04 + 41.0,
            wz * 0.04 + 41.0,
            self.seed.wrapping_add(0x0A51_2222),
        ) * (pool_radius * 0.35);

        let dx = (world_x - center_x) as f32 + warp_x;
        let dz = (world_z - center_z) as f32 + warp_z;
        let dist = (dx * dx + dz * dz).sqrt();

        let shape_noise = crate::core::noise::gradient_noise_2d(
            wx * 0.06,
            wz * 0.06,
            self.seed.wrapping_add(0x0A51_3333),
        );
        let organic_radius = pool_radius * (1.0 + shape_noise * 0.28);
        let t_dist = dist / organic_radius;

        if t_dist <= 1.55 {
            let oasis_water_y = (center_h - 0.8).round() as i32;
            let carve_depth = if t_dist < 0.90 {
                let inner_t = t_dist / 0.90;
                (1.0 - inner_t * inner_t) * 3.8
            } else {
                0.0
            };
            Some((oasis_water_y, carve_depth, t_dist))
        } else {
            None
        }
    }

    /// Backwards compatibility alias for sample_organic_lake.
    #[inline]
    pub fn sample_flat_pond(
        &self,
        world_x: i32,
        world_z: i32,
        natural_height: f32,
        climate: &ClimateSample,
    ) -> Option<(i32, f32)> {
        self.sample_organic_lake(world_x, world_z, natural_height, climate)
            .map(|(wy, cd, _)| (wy, cd))
    }

    /// Evaluates organic meandering river channels and mountain tributaries using continuous
    /// domain-warped ridged multi-octave noise.
    /// Never produces polygonal wireframes, straight lines, or circular loops around hills.
    /// Evaluates organic meandering river channels using multi-octave fractal noise.
    pub fn organic_river_sample(&self, world_x: f32, world_z: f32) -> (f32, bool) {
        let warp_x = crate::core::noise::gradient_noise_2d(
            world_x * 0.0016,
            world_z * 0.0016,
            self.seed.wrapping_add(101_333),
        ) * 32.0;
        let warp_z = crate::core::noise::gradient_noise_2d(
            world_x * 0.0016 + 137.0,
            world_z * 0.0016 + 137.0,
            self.seed.wrapping_add(202_555),
        ) * 32.0;

        let wx = world_x + warp_x;
        let wz = world_z + warp_z;

        let river_noise = fractal_noise(
            wx,
            wz,
            self.river_frequency,
            3,
            0.50,
            self.seed.wrapping_add(555_123),
        );
        let dist = river_noise.abs();
        if dist < self.river_width {
            let factor = 1.0 - (dist / self.river_width);
            (factor, true)
        } else {
            (0.0, false)
        }
    }

    pub fn river_sample(&self, world_x: f32, world_z: f32) -> (f32, bool) {
        self.organic_river_sample(world_x, world_z)
    }

    pub fn river_sample_x4(&self, world_x: Simd4f, world_z: Simd4f) -> ([f32; 4], [bool; 4]) {
        let mut depth_factors = [0.0f32; 4];
        let mut is_river = [false; 4];
        for i in 0..4 {
            let (df, ir) = self.organic_river_sample(world_x.0[i], world_z.0[i]);
            depth_factors[i] = df;
            is_river[i] = ir;
        }
        (depth_factors, is_river)
    }

    pub fn organic_river_sample_with_offset(
        &self,
        world_x: f32,
        world_z: f32,
    ) -> (f32, bool, f32, f32) {
        let (factor, is_river) = self.organic_river_sample(world_x, world_z);
        if !is_river || factor <= 0.01 {
            return (factor, false, 0.0, 0.0);
        }

        let eps = 2.0;
        let (f_px, _) = self.organic_river_sample(world_x + eps, world_z);
        let (f_mx, _) = self.organic_river_sample(world_x - eps, world_z);
        let (f_pz, _) = self.organic_river_sample(world_x, world_z + eps);
        let (f_mz, _) = self.organic_river_sample(world_x, world_z - eps);

        let df_dx = (f_px - f_mx) / (2.0 * eps);
        let df_dz = (f_pz - f_mz) / (2.0 * eps);
        let grad_len = (df_dx * df_dx + df_dz * df_dz).sqrt();

        let (center_dx, center_dz) = if grad_len > 1e-6 {
            let dist_to_center = ((1.0 - factor) * self.river_width * 150.0).clamp(-25.0, 25.0);
            (
                (df_dx / grad_len) * dist_to_center,
                (df_dz / grad_len) * dist_to_center,
            )
        } else {
            (0.0, 0.0)
        };

        (factor, is_river, center_dx, center_dz)
    }

    pub fn river_sample_with_offset(&self, world_x: f32, world_z: f32) -> (f32, bool, f32, f32) {
        self.organic_river_sample_with_offset(world_x, world_z)
    }

    pub fn river_sample_with_offset_x4(
        &self,
        world_x: Simd4f,
        world_z: Simd4f,
    ) -> ([f32; 4], [bool; 4], [f32; 4], [f32; 4]) {
        let mut depth_factors = [0.0f32; 4];
        let mut is_river = [false; 4];
        let mut center_dxs = [0.0f32; 4];
        let mut center_dzs = [0.0f32; 4];
        for i in 0..4 {
            let (df, ir, cdx, cdz) =
                self.organic_river_sample_with_offset(world_x.0[i], world_z.0[i]);
            depth_factors[i] = df;
            is_river[i] = ir;
            center_dxs[i] = cdx;
            center_dzs[i] = cdz;
        }
        (depth_factors, is_river, center_dxs, center_dzs)
    }

    /// Returns the downhill river flow velocity vector (in world units/sec) at a given (x, z) coordinate.
    /// If not in a river channel, returns Vec3::ZERO.
    pub fn river_flow_velocity(&self, world_x: f32, world_z: f32) -> Vec3 {
        let (depth_factor, is_river) = self.organic_river_sample(world_x, world_z);
        if !is_river || depth_factor <= 0.02 {
            return Vec3::ZERO;
        }

        let eps = 2.0;
        let (f_px, _) = self.organic_river_sample(world_x + eps, world_z);
        let (f_mx, _) = self.organic_river_sample(world_x - eps, world_z);
        let (f_pz, _) = self.organic_river_sample(world_x, world_z + eps);
        let (f_mz, _) = self.organic_river_sample(world_x, world_z - eps);

        let df_dx = (f_px - f_mx) / (2.0 * eps);
        let df_dz = (f_pz - f_mz) / (2.0 * eps);
        let grad_len = (df_dx * df_dx + df_dz * df_dz).sqrt();

        let (tangent_x, tangent_z) = if grad_len > 1e-5 {
            // Perpendicular to channel cross-section gradient is along the river flow
            (-df_dz / grad_len, df_dx / grad_len)
        } else {
            (1.0, 0.0)
        };

        // Downhill direction towards the ocean (decreasing continentalness)
        let c_plus = self
            .climate
            .sample_dithered(
                world_x + tangent_x * 16.0,
                world_z + tangent_z * 16.0,
                self.seed,
            )
            .continentalness;
        let c_minus = self
            .climate
            .sample_dithered(
                world_x - tangent_x * 16.0,
                world_z - tangent_z * 16.0,
                self.seed,
            )
            .continentalness;

        let (flow_x, flow_z) = if c_plus < c_minus {
            (tangent_x, tangent_z)
        } else {
            (-tangent_x, -tangent_z)
        };

        let speed = self.stream_flow_speed + depth_factor * 1.2;
        Vec3::new(flow_x * speed, 0.0, flow_z * speed)
    }

    #[inline]
    fn is_overhang_rugged(column: TerrainColumn) -> bool {
        column.climate.ridges > 0.20
            || matches!(
                column.biome,
                BiomeType::CoastalCliffs
                    | BiomeType::MonolithCliffs
                    | BiomeType::FrozenCliffs
                    | BiomeType::DesertSpires
                    | BiomeType::RedrockCanyon
                    | BiomeType::PaintedMesas
                    | BiomeType::EmeraldFoothills
                    | BiomeType::GranitePlateau
                    | BiomeType::MistyCrags
                    | BiomeType::AlpinePeaks
                    | BiomeType::JaggedPeaks
                    | BiomeType::VolcanicPeaks
                    | BiomeType::FracturedSavanna
                    | BiomeType::DeepGorge
            )
    }

    #[inline]
    fn overhang_voxel_material(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        column: TerrainColumn,
    ) -> Voxel {
        if matches!(
            column.biome,
            BiomeType::RedrockCanyon
                | BiomeType::PaintedMesas
                | BiomeType::SavannaBadlands
                | BiomeType::DesertSpires
        ) {
            crate::generation::strata::canyon_strata_voxel(world_y, world_x, world_z, self.seed)
        } else {
            column.biome.config().cliff_material
        }
    }

    /// Evaluates the effective LOD surface elevation and material for a column, including
    /// 3D surface rock overhangs, boulders, and cliffs generated by `sample_surface_overhang`.
    pub fn lod_surface_height_and_material(
        &self,
        world_x: i32,
        world_z: i32,
        column: TerrainColumn,
    ) -> (i32, Voxel) {
        let base_mat = self.surface_material_at(world_x, column.terrain_height, world_z, column);

        if column.water_level.is_some()
            || column.terrain_height <= self.effective_sea_level() + 2
            || !Self::is_overhang_rugged(column)
        {
            return (column.terrain_height, base_mat);
        }

        // Fast early-out at dy = 1: if 3D overhang density is well below threshold here,
        // higher layers (- dy * 0.075) cannot solidify into a surface boulder/overhang.
        let world_y1 = column.terrain_height + 1;
        let noise_a1 = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.05,
            world_y1 as f32 * 0.07,
            world_z as f32 * 0.05,
            self.seed.wrapping_add(88_871),
        );
        let noise_b1 = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.09,
            world_y1 as f32 * 0.09,
            world_z as f32 * 0.09,
            self.seed.wrapping_add(99_931),
        );
        let base_density1 = noise_a1 + noise_b1 * 0.35;
        if base_density1 < 0.22 {
            return (column.terrain_height, base_mat);
        }

        for dy in (2..=16).rev() {
            let wy = column.terrain_height + dy;
            if let Some(vox) = self.sample_surface_overhang(world_x, wy, world_z, column, dy) {
                return (wy, vox);
            }
        }

        if base_density1 - 0.075 > 0.32 {
            return (
                world_y1,
                self.overhang_voxel_material(world_x, world_y1, world_z, column),
            );
        }

        (column.terrain_height, base_mat)
    }

    /// Evaluates 3D volumetric density for surface rock arches and overhangs.
    fn sample_surface_overhang(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        column: TerrainColumn,
        dy: i32,
    ) -> Option<Voxel> {
        if !Self::is_overhang_rugged(column) || dy > 16 {
            return None;
        }

        let noise_a = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.05,
            world_y as f32 * 0.07,
            world_z as f32 * 0.05,
            self.seed.wrapping_add(88_871),
        );
        let noise_b = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.09,
            world_y as f32 * 0.09,
            world_z as f32 * 0.09,
            self.seed.wrapping_add(99_931),
        );

        let density = noise_a + noise_b * 0.35 - (dy as f32 * 0.075);
        if density > 0.32 {
            Some(self.overhang_voxel_material(world_x, world_y, world_z, column))
        } else {
            None
        }
    }

    /// Evaluates if an arch opening should hollow out under a mountain cliff surface.
    fn sample_surface_arch_opening(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        column: TerrainColumn,
        depth: i32,
    ) -> bool {
        let is_rugged = column.climate.ridges > 0.25
            || matches!(
                column.biome,
                BiomeType::CoastalCliffs
                    | BiomeType::MonolithCliffs
                    | BiomeType::FrozenCliffs
                    | BiomeType::DesertSpires
                    | BiomeType::RedrockCanyon
                    | BiomeType::PaintedMesas
                    | BiomeType::EmeraldFoothills
                    | BiomeType::GranitePlateau
                    | BiomeType::MistyCrags
                    | BiomeType::AlpinePeaks
                    | BiomeType::JaggedPeaks
                    | BiomeType::VolcanicPeaks
                    | BiomeType::DeepGorge
            );

        if !is_rugged || !(1..=6).contains(&depth) {
            return false;
        }

        let noise_a = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.05,
            world_y as f32 * 0.07,
            world_z as f32 * 0.05,
            self.seed.wrapping_add(88_871),
        );
        let noise_b = crate::core::noise::gradient_noise_3d(
            world_x as f32 * 0.09,
            world_y as f32 * 0.09,
            world_z as f32 * 0.09,
            self.seed.wrapping_add(99_931),
        );

        noise_a > 0.30 && noise_b < -0.15
    }

    /// Converts a mother rock into its corresponding surface outcrop turf block.
    pub fn rock_to_outcrop_grass(stone: Voxel) -> Voxel {
        match stone {
            Voxel::Andesite => Voxel::AndesiteGrass,
            Voxel::Argillite => Voxel::ArgilliteGrass,
            Voxel::Basalt => Voxel::BasaltGrass,
            Voxel::Chalk => Voxel::ChalkGrass,
            Voxel::Dolomite => Voxel::DolomiteGrass,
            Voxel::Granite => Voxel::GraniteGrass,
            Voxel::Limestone => Voxel::LimestoneGrass,
            _ => Voxel::GraniteGrass,
        }
    }

    /// Converts a mother rock into its corresponding snow outcrop block for alpine transitions.
    pub fn rock_to_outcrop_snow(stone: Voxel) -> Voxel {
        match stone {
            Voxel::Andesite => Voxel::AndesiteSnow,
            Voxel::Argillite => Voxel::ArgilliteSnow,
            Voxel::Basalt => Voxel::BasaltSnow,
            Voxel::Chalk => Voxel::ChalkSnow,
            Voxel::Dolomite => Voxel::DolomiteSnow,
            Voxel::Granite => Voxel::GraniteSnow,
            Voxel::Limestone => Voxel::LimestoneSnow,
            _ => Voxel::GraniteSnow,
        }
    }

    pub fn surface_material_at(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        column: TerrainColumn,
    ) -> Voxel {
        let cfg = column.biome.config();
        let dither_noise = crate::core::noise::gradient_noise_2d(
            world_x as f32 * 0.15,
            world_z as f32 * 0.15,
            self.seed.wrapping_add(SEED_OFFSET_SURFACE_DITHER),
        );

        // 1. Submerged terrain (underwater) floor materials
        let is_submerged = column.water_level.is_some_and(|wl| world_y <= wl);
        if is_submerged {
            // River Bed Styling (authentic fluvial sediment lining channel floor)
            if column.is_river {
                return if column.climate.temperature < -0.15 {
                    if dither_noise > 0.15 {
                        Voxel::GravelDirty
                    } else if dither_noise > -0.20 {
                        Voxel::ClayBlue
                    } else {
                        Voxel::GravelSandy
                    }
                } else if column.climate.humidity > 0.35 {
                    if dither_noise > 0.10 {
                        Voxel::SiltMud
                    } else {
                        Voxel::ClayBlue
                    }
                } else if matches!(
                    column.biome,
                    BiomeType::RedrockCanyon
                        | BiomeType::PaintedMesas
                        | BiomeType::AncientSands
                        | BiomeType::DesertSpires
                        | BiomeType::SavannaBadlands
                ) {
                    if dither_noise > 0.10 {
                        Voxel::SandRed
                    } else {
                        Voxel::ClayRed
                    }
                } else {
                    if dither_noise > 0.20 {
                        Voxel::SandCommon
                    } else if dither_noise > -0.20 {
                        Voxel::GravelSandy
                    } else {
                        Voxel::ClayBlue
                    }
                };
            }

            // Lake Bed Styling (authentic multi-layered lacustrine sediment)
            if column.is_lake {
                let lake_depth = column.water_level.map(|wl| wl - world_y).unwrap_or(0);
                return if lake_depth >= 3 {
                    if dither_noise > 0.15 {
                        Voxel::ClayBlue
                    } else if dither_noise > -0.15 {
                        Voxel::SiltMud
                    } else {
                        Voxel::PeatMud
                    }
                } else if lake_depth >= 1 {
                    if dither_noise > 0.20 {
                        Voxel::GravelDirty
                    } else if dither_noise > -0.20 {
                        Voxel::GravelSandy
                    } else {
                        Voxel::ClayBlue
                    }
                } else {
                    if dither_noise > 0.15 {
                        Voxel::SandCommon
                    } else if dither_noise > -0.20 {
                        Voxel::GravelSandy
                    } else {
                        Voxel::LoamMud
                    }
                };
            }

            let seabed_macro = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.042,
                world_z as f32 * 0.042,
                self.seed.wrapping_add(0x5EA8_ED01),
            );
            let seabed_pocket = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.11,
                world_z as f32 * 0.11,
                self.seed.wrapping_add(0x5EA8_ED02),
            );
            let ocean_depth = column.water_level.map(|wl| wl - world_y).unwrap_or(0);

            return match column.biome {
                BiomeType::SeagrassMeadows => {
                    if seabed_macro > -0.28 {
                        if seabed_pocket > 0.42 {
                            Voxel::SandCommon
                        } else if seabed_pocket < -0.45 {
                            Voxel::GravelSandy
                        } else {
                            Voxel::SeagrassSand
                        }
                    } else if seabed_pocket > 0.10 {
                        Voxel::SandCommon
                    } else {
                        Voxel::SeagrassSand
                    }
                }
                BiomeType::WarmOcean => {
                    if seabed_macro > 0.34 {
                        if seabed_pocket > 0.15 {
                            Voxel::SandWhite
                        } else {
                            Voxel::SandCommon
                        }
                    } else if seabed_macro > 0.06 {
                        if seabed_pocket > 0.28 {
                            Voxel::GravelSandy
                        } else if seabed_pocket < -0.22 {
                            Voxel::ClayYellow
                        } else {
                            Voxel::SandCommon
                        }
                    } else if seabed_macro > -0.22 {
                        if seabed_pocket > 0.20 {
                            Voxel::ClayBlue
                        } else if seabed_pocket < -0.20 {
                            Voxel::GravelDirty
                        } else if ocean_depth >= 7 {
                            Voxel::SiltMud
                        } else {
                            Voxel::SandDune
                        }
                    } else if seabed_pocket > 0.18 {
                        Voxel::LoamDirt
                    } else if seabed_pocket < -0.15 {
                        Voxel::GravelDirty
                    } else {
                        Voxel::ClayBlue
                    }
                }
                BiomeType::ColdOcean | BiomeType::KelpForests => {
                    if seabed_macro > 0.28 {
                        if seabed_pocket > 0.10 {
                            Voxel::GravelSandy
                        } else {
                            Voxel::SandCommon
                        }
                    } else if seabed_macro > -0.05 {
                        if seabed_pocket > 0.25 {
                            Voxel::ClayBlue
                        } else if seabed_pocket < -0.25 {
                            Voxel::SiltMud
                        } else {
                            Voxel::GravelDirty
                        }
                    } else if seabed_pocket > 0.22 {
                        Voxel::LoamDirt
                    } else if seabed_pocket < -0.20 && ocean_depth >= 8 {
                        Voxel::CobbledAndesite
                    } else {
                        Voxel::PeatMud
                    }
                }
                BiomeType::CoralReefs | BiomeType::CoralIsland => {
                    if seabed_macro > 0.25 {
                        if seabed_pocket > 0.10 {
                            Voxel::SandWhite
                        } else {
                            Voxel::ClayWhite
                        }
                    } else if seabed_macro > -0.15 {
                        if seabed_pocket > 0.22 {
                            Voxel::Reefstone
                        } else if seabed_pocket < -0.22 {
                            Voxel::GravelSandy
                        } else {
                            Voxel::SandCommon
                        }
                    } else if seabed_pocket > 0.15 {
                        Voxel::SandDune
                    } else {
                        Voxel::Reefstone
                    }
                }
                BiomeType::OceanTrench => {
                    if seabed_macro > 0.25 {
                        if dither_noise > 0.0 {
                            Voxel::Basalt
                        } else {
                            Voxel::CobbledBasalt
                        }
                    } else if seabed_macro > -0.15 {
                        if seabed_pocket > 0.15 {
                            Voxel::Gabbro
                        } else {
                            Voxel::GravelDirty
                        }
                    } else if seabed_pocket > 0.0 {
                        Voxel::ClayBlue
                    } else {
                        Voxel::SiltMud
                    }
                }
                BiomeType::Caldera => Voxel::Basalt,
                BiomeType::WillowMarsh
                | BiomeType::MangroveSwamp
                | BiomeType::OrchidSwamp
                | BiomeType::ObsidianMire
                | BiomeType::Estuary => Voxel::PeatMud,
                BiomeType::GlacialChasm
                | BiomeType::IceSheet
                | BiomeType::FrozenCliffs
                | BiomeType::FrozenIsland
                | BiomeType::IceCaves => Voxel::FrostIcePacked,
                _ => {
                    if column.climate.temperature < -0.15 {
                        if seabed_macro > 0.15 {
                            Voxel::GravelSandy
                        } else if seabed_pocket > 0.15 {
                            Voxel::ClayBlue
                        } else {
                            Voxel::GravelDirty
                        }
                    } else if seabed_macro > 0.25 {
                        Voxel::SandCommon
                    } else if seabed_macro > -0.10 {
                        if seabed_pocket > 0.20 {
                            Voxel::GravelSandy
                        } else if seabed_pocket < -0.20 {
                            Voxel::ClayBlue
                        } else {
                            Voxel::SandDune
                        }
                    } else if seabed_pocket > 0.15 {
                        Voxel::LoamDirt
                    } else {
                        Voxel::SiltMud
                    }
                }
            };
        }

        // Riverbank waterline rings (muddy, coarse transition strip bordering the river)
        if column.is_river && column.river_factor > 0.18 {
            return match column.biome.config().subsoil_material {
                Voxel::LateriteDirt => Voxel::LateriteDirt,
                Voxel::PeatDirt => Voxel::PeatMud,
                _ => {
                    if dither_noise > 0.25 {
                        Voxel::GravelDirty
                    } else if dither_noise > -0.15 {
                        Voxel::SiltMud
                    } else {
                        Voxel::LoamMud
                    }
                }
            };
        }

        // Lake Shoreline Levee Rim (organic vegetated and pebble transition strip surrounding lakes)
        if column.is_lake {
            return match column.biome.config().subsoil_material {
                Voxel::LateriteDirt => Voxel::LateriteDirt,
                Voxel::PeatDirt => Voxel::PeatMud,
                _ => {
                    if dither_noise > 0.35 {
                        Voxel::GravelDirty
                    } else if dither_noise > 0.05 {
                        Voxel::LoamCoarse
                    } else if dither_noise > -0.25 {
                        Voxel::SiltMud
                    } else {
                        Voxel::LoamRooted
                    }
                }
            };
        }

        // 2. Coastal Shoreline & Beach Surface (predominantly sand with organic scattered gravel patches)
        if column.is_beach {
            let beach_noise = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.16,
                world_z as f32 * 0.16,
                self.seed.wrapping_add(SEED_OFFSET_BEACH_NOISE),
            );

            // Dedicated gravel beach biome or extreme subarctic cold coast:
            // ~70% sand with natural gravel pockets (not 99% solid gravel!)
            let is_gravel_specialist = matches!(
                column.biome,
                BiomeType::GravelBeach | BiomeType::CraggyBeach
            ) || (column.climate.temperature < -0.40
                && !matches!(
                    column.biome,
                    BiomeType::FrozenCliffs | BiomeType::IceSheet | BiomeType::FrozenIsland
                ));

            if is_gravel_specialist {
                return if beach_noise > 0.35 {
                    if dither_noise > 0.0 {
                        Voxel::GravelSandy
                    } else {
                        Voxel::GravelDirty
                    }
                } else {
                    Voxel::SandCommon
                };
            }

            // Red sand with gravel patches on arid canyon / badlands shores
            if matches!(
                column.biome,
                BiomeType::RedrockCanyon
                    | BiomeType::PaintedMesas
                    | BiomeType::AncientSands
                    | BiomeType::DesertSpires
                    | BiomeType::SavannaBadlands
            ) {
                return if beach_noise > 0.48 {
                    Voxel::GravelSandy
                } else {
                    Voxel::SandRed
                };
            }

            // Standard beaches: predominantly sand (~80%) with natural scattered patches of gravel (~20%)
            let has_gravel_patch = beach_noise > 0.48;
            if has_gravel_patch {
                return if dither_noise > 0.0 {
                    Voxel::GravelSandy
                } else {
                    Voxel::GravelDirty
                };
            }

            // Clean beach sand: white coral sand in tropics, golden common sand in temperate zones
            return if column.climate.temperature > 0.40 {
                Voxel::SandWhite
            } else {
                Voxel::SandCommon
            };
        }

        // 3. High alpine elevation snowline: cold/temperate mountains above y >= 50 receive snowcaps
        let is_warm = matches!(
            column.biome,
            BiomeType::DesertSpires
                | BiomeType::AncientSands
                | BiomeType::DesertOasis
                | BiomeType::AcaciaPlains
                | BiomeType::FracturedSavanna
                | BiomeType::SavannaBadlands
                | BiomeType::WitheredWoods
                | BiomeType::AshenWastes
                | BiomeType::ObsidianMire
                | BiomeType::AcidPools
                | BiomeType::SulfurVents
                | BiomeType::ScorchedFlats
                | BiomeType::DuneSea
                | BiomeType::RedrockCanyon
                | BiomeType::PaintedMesas
                | BiomeType::VolcanicPeaks
                | BiomeType::VolcanicJungle
                | BiomeType::Caldera
                | BiomeType::SandyShores
                | BiomeType::GravelBeach
                | BiomeType::DuneBeach
                | BiomeType::TropicalIsland
                | BiomeType::CoralIsland
        );

        if !is_warm && column.climate.temperature < 0.10 {
            let snowline_jitter = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.05,
                world_z as f32 * 0.05,
                self.seed.wrapping_add(SEED_OFFSET_SNOWLINE_JITTER),
            ) * 4.0;
            let snowline = 75.0 + snowline_jitter;

            if world_y as f32 >= snowline {
                let slope_noise = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.15,
                    world_z as f32 * 0.15,
                    self.seed.wrapping_add(SEED_OFFSET_SNOW_SLOPE),
                );
                if world_y as f32 >= snowline + 8.0 && slope_noise > 0.35 {
                    return column.biome.config().primary_stone;
                }
                return Voxel::SnowBlock;
            } else if world_y as f32 >= snowline - 10.0 {
                // Organic Alpine Transition Zone: gradual snow patches and snow-dusted rock outcrops
                let trans_t = (world_y as f32 - (snowline - 10.0)) / 10.0;
                let trans_noise = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.18,
                    world_z as f32 * 0.18,
                    self.seed.wrapping_add(SEED_OFFSET_SNOW_SLOPE),
                );
                let snow_threshold = 0.65 - trans_t * 0.50;
                if trans_noise > snow_threshold {
                    if trans_noise > snow_threshold + 0.18 {
                        return Voxel::SnowBlock;
                    } else {
                        return Self::rock_to_outcrop_snow(cfg.primary_stone);
                    }
                }
            }
        }

        // 4. Biome Surface Block Selection with organic dual-surface dithering
        if column.is_cliff {
            if matches!(
                column.biome,
                BiomeType::RedrockCanyon
                    | BiomeType::PaintedMesas
                    | BiomeType::SavannaBadlands
                    | BiomeType::DesertSpires
            ) {
                crate::generation::strata::canyon_strata_voxel(world_y, world_x, world_z, self.seed)
            } else {
                cfg.cliff_material
            }
        } else {
            // Natural complementary forest & tundra floor patches:
            // Medium-scale organic patches of Tundra Moss, Mulch, Coarse Dirt, and Dirt
            if matches!(
                column.biome,
                BiomeType::BleakTaiga
                    | BiomeType::ColdShrubland
                    | BiomeType::PermafrostSteppe
                    | BiomeType::TundraIsland
                    | BiomeType::SnowyPlains
            ) {
                let icy_patch = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.068,
                    world_z as f32 * 0.068,
                    self.seed.wrapping_add(0x1C3F_1111),
                ) + dither_noise * 0.22;
                if icy_patch > 0.54 {
                    return Voxel::PermafrostIcy;
                }

                let moss_patch = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.055,
                    world_z as f32 * 0.055,
                    self.seed.wrapping_add(0x709D_7A01),
                ) + dither_noise * 0.18;
                if moss_patch > 0.12 && column.biome != BiomeType::SnowyPlains {
                    return if moss_patch > 0.45 {
                        Voxel::LoamTundraMoss
                    } else if moss_patch > 0.28 {
                        Voxel::PeatTundraMoss
                    } else {
                        Voxel::SiltTundraMoss
                    };
                } else if moss_patch > 0.42 && column.biome == BiomeType::SnowyPlains {
                    return Voxel::SiltTundraMoss;
                }

                let coarse_patch = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.072,
                    world_z as f32 * 0.072,
                    self.seed.wrapping_add(0xC0A5_3333),
                ) + dither_noise * 0.20;
                if coarse_patch > 0.48 {
                    if coarse_patch > 0.64 || dither_noise > 0.15 {
                        return Voxel::PermafrostCoarse;
                    } else {
                        return Voxel::PermafrostDirt;
                    }
                }
            } else if matches!(
                column.biome,
                BiomeType::MistyWoods
                    | BiomeType::AutumnForest
                    | BiomeType::SakuraGrove
                    | BiomeType::VerdantWoods
                    | BiomeType::TemperateIsland
                    | BiomeType::DenseRainforest
                    | BiomeType::BambooGrove
                    | BiomeType::JungleWetlands
                    | BiomeType::VolcanicJungle
                    | BiomeType::FertileValley
                    | BiomeType::LavenderValley
                    | BiomeType::ShieldForest
                    | BiomeType::PineHighlands
                    | BiomeType::WintryForest
                    | BiomeType::FrostPinewoods
            ) {
                let (dirt, coarse, mulch) = match column.biome {
                    BiomeType::ShieldForest
                    | BiomeType::WintryForest
                    | BiomeType::FrostPinewoods => {
                        (Voxel::PeatDirt, Voxel::PeatCoarse, Voxel::PeatMulch)
                    }
                    BiomeType::PineHighlands | BiomeType::LavenderValley => {
                        (Voxel::SiltDirt, Voxel::SiltCoarse, Voxel::SiltMulch)
                    }
                    _ => (Voxel::LoamDirt, Voxel::LoamCoarse, Voxel::LoamMulch),
                };

                let mulch_patch = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.068,
                    world_z as f32 * 0.068,
                    self.seed.wrapping_add(0x401C_1111),
                ) + dither_noise * 0.22;
                if mulch_patch > 0.48 {
                    return mulch;
                }

                let soil_patch = crate::core::noise::gradient_noise_2d(
                    world_x as f32 * 0.072,
                    world_z as f32 * 0.072,
                    self.seed.wrapping_add(0xC0A5_2222),
                ) + dither_noise * 0.20;
                if soil_patch > 0.52 {
                    if soil_patch > 0.66 || dither_noise > 0.15 {
                        return coarse;
                    } else {
                        return dirt;
                    }
                }
            } else if matches!(
                column.biome,
                BiomeType::SavannaBadlands
                    | BiomeType::FracturedSavanna
                    | BiomeType::AcaciaPlains
            ) && dither_noise > 0.60
            {
                return Voxel::LateriteDirt;
            }

            // Small scattered rock outcrops on flat terrain using organic high-frequency noise
            let outcrop_noise = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.16,
                world_z as f32 * 0.16,
                self.seed.wrapping_add(SEED_OFFSET_OUTCROP_TURF),
            );
            if outcrop_noise > 0.68 {
                Self::rock_to_outcrop_grass(cfg.primary_stone)
            } else {
                cfg.surface_material
            }
        }
    }

    fn voxel_at_sampled(
        &self,
        column: TerrainColumn,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        cave_sample: CaveNoiseSample,
    ) -> Voxel {
        // Underground mountain river tunnel
        if column.is_underground_river {
            let river_floor = self.effective_sea_level() - 4;
            let river_roof = self.effective_sea_level() + 5;
            if world_y >= river_floor && world_y <= river_roof {
                if world_y <= self.effective_sea_level() {
                    return Voxel::LiquidWater;
                } else {
                    return Voxel::Air;
                }
            }
        }

        if world_y > column.terrain_height {
            if let Some(water_level) = column.water_level
                && world_y <= water_level
            {
                return Voxel::LiquidWater;
            }

            // Surface rock arches and overhangs strictly generate on dry land above water
            if column.water_level.is_none()
                && column.terrain_height > self.effective_sea_level() + 2
            {
                let dy = world_y - column.terrain_height;
                if dy <= 16
                    && let Some(overhang_voxel) =
                        self.sample_surface_overhang(world_x, world_y, world_z, column, dy)
                    {
                        return overhang_voxel;
                    }
            }

            return Voxel::Air;
        }

        // Absolute bedrock floor: bottom layers of the world are strictly solid Mantle
        if world_y <= self.strata.bedrock_min_block_y {
            return Voxel::Mantle;
        }

        let is_underwater =
            column.water_level.is_some() || column.terrain_height <= self.effective_sea_level() + 2;

        // Impervious aquatic bed and bank barrier: protect riverbed, lakebed, water columns, and surrounding banks
        // from being hollowed out or breached by underground caves and ravines.
        let is_near_water_basin = (column.is_river
            || column.is_lake
            || column.river_factor > 0.02
            || column.water_level.is_some())
            && world_y >= (column.water_level.unwrap_or(column.terrain_height) - 5);

        // Subterranean volcanic lava basins:
        // Discrete volcanic pools enclosed by Basalt, Obsidian, and Magma (never floats, never leaks)
        if !is_near_water_basin && world_y <= self.caves.deep_lava_y && world_y > -246
            && let Some(basin_voxel) = self
                .caves
                .subterranean_lava_basin_at(world_x, world_y, world_z, self.seed)
            {
                return basin_voxel;
            }

        if !is_near_water_basin
            && self.caves.is_cave_sampled(
                world_x,
                world_y,
                world_z,
                cave_sample,
                column.terrain_height,
                is_underwater,
                self.effective_sea_level(),
                self.seed,
            )
        {
            // Subterranean fluid basins tailored to depth and regional cave biomes:
            // All subterranean pools are strictly flat, horizontal basins (no fluids on walls/ceilings).
            let biome_type = column.biome;
            if biome_type == BiomeType::ThermalCaves && world_y <= -30 {
                if let Some(pool_y) =
                    self.subterranean_pool_level(world_x, world_z, 889_121, 42, 25, -35, 18, 5, 5.0)
                    && world_y <= pool_y && world_y >= pool_y - 2 {
                        return Voxel::LiquidAcid;
                    }
            } else if (biome_type == BiomeType::MushroomCaves
                || biome_type == BiomeType::OrchidSwamp)
                && world_y <= -20
            {
                if let Some(pool_y) =
                    self.subterranean_pool_level(world_x, world_z, 441_889, 40, 28, -25, 16, 5, 6.0)
                    && world_y <= pool_y && world_y >= pool_y - 2 {
                        return Voxel::LiquidSludge;
                    }
            } else if biome_type == BiomeType::ThermalCaves && (-160..=-40).contains(&world_y) {
                if let Some(pool_y) =
                    self.subterranean_pool_level(world_x, world_z, 66_331, 40, 30, -50, 16, 6, 6.0)
                    && world_y <= pool_y && world_y >= pool_y - 2 {
                        return Voxel::LiquidSpringWater;
                    }
            } else if matches!(
                biome_type,
                BiomeType::AncientSands | BiomeType::SavannaBadlands | BiomeType::DesertSpires
            ) && (-90..=-15).contains(&world_y)
                && let Some(pool_y) =
                    self.subterranean_pool_level(world_x, world_z, 881_204, 44, 28, -20, 14, 5, 5.0)
                    && world_y <= pool_y && world_y >= pool_y - 3 {
                        return Voxel::LiquidTar;
                    }

            return self.caves.cave_voxel_sampled(
                world_y,
                world_x,
                world_z,
                cave_sample,
                self.effective_sea_level(),
                self.seed,
            );
        }

        let depth = column.terrain_height - world_y;
        if depth <= 0 {
            return self.surface_material_at(world_x, world_y, world_z, column);
        }

        // 3D Rock Arch Openings (hollow window under mountain cliff)
        if !column.is_river
            && column.water_level.is_none()
            && self.sample_surface_arch_opening(world_x, world_y, world_z, column, depth)
        {
            return Voxel::Air;
        }

        let logical_depth = depth / LOGICAL_BLOCK_VOXELS;
        let biome_cfg = column.biome.config();

        if column.is_beach && logical_depth <= 5 {
            let beach_noise = crate::core::noise::gradient_noise_2d(
                world_x as f32 * 0.16,
                world_z as f32 * 0.16,
                self.seed.wrapping_add(SEED_OFFSET_BEACH_NOISE),
            );

            let is_gravel_specialist = matches!(
                column.biome,
                BiomeType::GravelBeach | BiomeType::CraggyBeach
            ) || (column.climate.temperature < -0.40
                && !matches!(
                    column.biome,
                    BiomeType::FrozenCliffs | BiomeType::IceSheet | BiomeType::FrozenIsland
                ));

            if is_gravel_specialist {
                return if beach_noise > 0.40 {
                    Voxel::SandCommon
                } else {
                    Voxel::GravelDirty
                };
            }

            if matches!(
                column.biome,
                BiomeType::RedrockCanyon
                    | BiomeType::PaintedMesas
                    | BiomeType::AncientSands
                    | BiomeType::DesertSpires
                    | BiomeType::SavannaBadlands
            ) {
                return if beach_noise > 0.32 {
                    Voxel::GravelSandy
                } else {
                    Voxel::SandRed
                };
            }

            let has_gravel_patch = beach_noise > 0.28;
            if has_gravel_patch {
                return Voxel::GravelDirty;
            }

            return if column.climate.temperature > 0.40 {
                Voxel::SandWhite
            } else {
                Voxel::SandCommon
            };
        }

        if logical_depth <= 2 {
            let surface = self.surface_material_at(world_x, column.terrain_height, world_z, column);
            if matches!(
                surface,
                Voxel::SandCommon
                    | Voxel::SandDune
                    | Voxel::SandRed
                    | Voxel::SandWhite
                    | Voxel::GravelSandy
                    | Voxel::GravelDirty
                    | Voxel::GravelAshy
                    | Voxel::GravelSnowy
                    | Voxel::LoamMud
                    | Voxel::PeatMud
                    | Voxel::SiltMud
                    | Voxel::SnowBlock
                    | Voxel::FrostIce
                    | Voxel::FrostIcePacked
                    | Voxel::FrostIceBlack
            ) {
                return surface;
            }
        }

        // Volcanic thermal halo: rock in the immediate vicinity of subterranean lava basins
        // is naturally transmuted into authentic volcanic geology (Basalt, Obsidian, Magma, CobbledBasalt).
        if world_y <= self.caves.deep_lava_y && world_y > -246
            && let Some(volcanic_rock) = self
                .caves
                .volcanic_halo_at(world_x, world_y, world_z, self.seed)
            {
                return volcanic_rock;
            }

        self.strata.solid_voxel_at(
            world_x,
            world_y,
            world_z,
            logical_depth,
            &biome_cfg,
            self.seed,
        )
    }

    pub fn voxel_at(
        &self,
        column: TerrainColumn,
        world_x: i32,
        world_y: i32,
        world_z: i32,
    ) -> Voxel {
        let sample = self.caves.sample_noise_point(
            world_x as f32,
            world_y as f32,
            world_z as f32,
            self.seed,
        );
        self.voxel_at_sampled(column, world_x, world_y, world_z, sample)
    }

    pub fn continental_elevation(continentalness: f32) -> f32 {
        const SPLINE_NODES: [(f32, f32); 10] = [
            (-1.00, -48.0), // Deep abyssal ocean trench
            (-0.65, -30.0), // Deep ocean basin
            (-0.35, -20.0), // Open ocean floor
            (-0.15, -12.0), // Outer continental shelf
            (-0.04, -6.0),  // Shallow coastal shelf / beach transition
            (0.10, 5.0),    // Low coastal plains (y = 21)
            (0.22, 16.0),   // Inland rolling plains (y = 32)
            (0.35, 42.0),   // Highlands & foothills (y = 58)
            (0.50, 84.0),   // Rugged mountain chains (y = 100)
            (0.70, 140.0),  // Grand alpine peaks (y = 156)
        ];

        let c = continentalness.clamp(-1.0, 1.0);

        for i in 0..(SPLINE_NODES.len() - 1) {
            let (c0, y0) = SPLINE_NODES[i];
            let (c1, y1) = SPLINE_NODES[i + 1];
            if c <= c1 {
                let t = (c - c0) / (c1 - c0);
                let t_smooth = t * t * (3.0 - 2.0 * t);
                return y0 + (y1 - y0) * t_smooth;
            }
        }

        SPLINE_NODES[SPLINE_NODES.len() - 1].1
    }

    pub fn continental_roughness(continentalness: f32) -> f32 {
        if continentalness < 0.0 {
            0.55
        } else if continentalness < 0.35 {
            let t = continentalness / 0.35;
            let t_smooth = t * t * (3.0 - 2.0 * t);
            0.55 + 0.50 * t_smooth
        } else {
            let t = ((continentalness - 0.35) / 0.50).min(1.0);
            let t_smooth = t * t * (3.0 - 2.0 * t);
            1.05 + 1.75 * t_smooth
        }
    }

    fn natural_height_at(&self, world_x: f32, world_z: f32, climate: &ClimateSample) -> f32 {
        let macro_noise = fractal_noise(
            world_x,
            world_z,
            self.macro_frequency,
            3,
            0.55,
            self.seed.wrapping_add(31_337),
        );

        let rolling_hills = fractal_noise(
            world_x,
            world_z,
            0.012,
            2,
            0.5,
            self.seed.wrapping_add(45_117),
        ) * self.rolling_hills_amplitude;

        let detail_noise = fractal_noise(
            world_x,
            world_z,
            self.detail_frequency,
            self.detail_octaves,
            self.persistence,
            self.seed.wrapping_add(81_731),
        );

        let cont_base = Self::continental_elevation(climate.continentalness);
        let roughness = Self::continental_roughness(climate.continentalness);

        let mountain_factor = ((climate.continentalness - 0.22) / 0.35).clamp(0.0, 1.0);
        let ridge =
            (1.0 - macro_noise.abs()).powi(2) * self.mountain_ridge_height * mountain_factor;

        let steep_variation = ((climate.ridges - 0.10) / 0.45).clamp(0.0, 1.0);
        let abrupt_peak = if steep_variation > 0.0 && mountain_factor > 0.0 {
            let peak_form = (1.0 - (macro_noise * 1.35).abs().min(1.0)).powf(1.6);
            peak_form * 16.0 * steep_variation * mountain_factor
        } else {
            0.0
        };

        let swamp_depression = if climate.continentalness > 0.02
            && climate.continentalness < 0.25
            && climate.humidity > 0.15
        {
            let wetness = ((climate.humidity - 0.15) / 0.20).clamp(0.0, 1.0);
            let inland_factor =
                (1.0 - ((climate.continentalness - 0.12).abs() / 0.12)).clamp(0.0, 1.0);
            wetness * inland_factor * 3.5
        } else {
            0.0
        };

        let base = self.base_height + cont_base - swamp_depression;
        let amplitude = (self.macro_amplitude * macro_noise
            + rolling_hills
            + self.detail_amplitude * detail_noise)
            * roughness
            + ridge
            + abrupt_peak;

        base + amplitude
    }

    fn natural_height_at_x4(
        &self,
        world_x: Simd4f,
        world_z: Simd4f,
        climates: &[ClimateSample; 4],
    ) -> Simd4f {
        let macro_noise = fractal_noise_2d_x4(
            world_x,
            world_z,
            self.macro_frequency,
            3,
            0.55,
            self.seed.wrapping_add(31_337),
        );

        let rolling_hills = fractal_noise_2d_x4(
            world_x,
            world_z,
            0.012,
            2,
            0.5,
            self.seed.wrapping_add(45_117),
        )
        .scale(self.rolling_hills_amplitude);

        let detail_noise = fractal_noise_2d_x4(
            world_x,
            world_z,
            self.detail_frequency,
            self.detail_octaves,
            self.persistence,
            self.seed.wrapping_add(81_731),
        );

        let mut res = [0.0f32; 4];
        for i in 0..4 {
            let m_n = macro_noise.0[i];
            let r_h = rolling_hills.0[i];
            let d_n = detail_noise.0[i];
            let climate = &climates[i];

            let cont_base = Self::continental_elevation(climate.continentalness);
            let roughness = Self::continental_roughness(climate.continentalness);

            let mountain_factor = ((climate.continentalness - 0.22) / 0.35).clamp(0.0, 1.0);
            let ridge = (1.0 - m_n.abs()).powi(2) * self.mountain_ridge_height * mountain_factor;

            let steep_variation = ((climate.ridges - 0.10) / 0.45).clamp(0.0, 1.0);
            let abrupt_peak = if steep_variation > 0.0 && mountain_factor > 0.0 {
                let peak_form = (1.0 - (m_n * 1.35).abs().min(1.0)).powf(1.6);
                peak_form * 16.0 * steep_variation * mountain_factor
            } else {
                0.0
            };

            let swamp_depression = if climate.continentalness > 0.02
                && climate.continentalness < 0.25
                && climate.humidity > 0.15
            {
                let wetness = ((climate.humidity - 0.15) / 0.20).clamp(0.0, 1.0);
                let inland_factor =
                    (1.0 - ((climate.continentalness - 0.12).abs() / 0.12)).clamp(0.0, 1.0);
                wetness * inland_factor * 3.5
            } else {
                0.0
            };

            let base = self.base_height + cont_base - swamp_depression;
            let amplitude = (self.macro_amplitude * m_n + r_h + self.detail_amplitude * d_n)
                * roughness
                + ridge
                + abrupt_peak;

            res[i] = base + amplitude;
        }

        Simd4f(res)
    }

    #[inline]
    pub fn subterranean_pool_level(
        &self,
        world_x: i32,
        world_z: i32,
        seed_offset: u32,
        cell_size: i32,
        chance_percent: u32,
        base_y: i32,
        step_y: i32,
        num_bands: u32,
        base_radius: f32,
    ) -> Option<i32> {
        let cell_x = world_x.div_euclid(cell_size);
        let cell_z = world_z.div_euclid(cell_size);

        for dx in -1..=1 {
            for dz in -1..=1 {
                let cx = cell_x + dx;
                let cz = cell_z + dz;
                let h = crate::core::noise::hash_2d(cx, cz, self.seed.wrapping_add(seed_offset));
                if (h % 100) >= chance_percent {
                    continue;
                }
                let margin = (cell_size / 4).max(4) as u32;
                let center_x =
                    cx * cell_size + ((h % (cell_size as u32 - margin * 2)) + margin) as i32;
                let center_z =
                    cz * cell_size + (((h >> 8) % (cell_size as u32 - margin * 2)) + margin) as i32;
                let band = ((h >> 16) % num_bands) as i32;
                let pool_surface_y = base_y - band * step_y;
                let radius = base_radius + ((h >> 20) % 5) as f32;
                let dist_sq = ((world_x - center_x) * (world_x - center_x)
                    + (world_z - center_z) * (world_z - center_z))
                    as f32;
                if dist_sq <= radius * radius {
                    return Some(pool_surface_y);
                }
            }
        }
        None
    }

    pub fn terrain_height_at(&self, world_x: f32, world_z: f32, climate: &ClimateSample) -> i32 {
        self.natural_height_at(world_x, world_z, climate).round() as i32
    }
}

pub fn logical_block_sample_position(logical_coordinate: i32) -> f32 {
    logical_coordinate as f32 * LOGICAL_BLOCK_VOXELS as f32 + 0.5
}

pub fn logical_block_bottom(world_y: i32) -> i32 {
    world_y.div_euclid(LOGICAL_BLOCK_VOXELS) * LOGICAL_BLOCK_VOXELS
}

pub fn logical_block_top(world_y: i32) -> i32 {
    logical_block_bottom(world_y) + LOGICAL_BLOCK_VOXELS - 1
}

fn column_index(x: usize, z: usize) -> usize {
    x + z * CHUNK_SIZE
}
