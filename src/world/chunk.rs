pub const VOXEL_SIZE: f32 = 1.0;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
pub const MAX_VOXEL_TYPES: usize = 256;

use bevy::platform::collections::HashMap;

pub use super::block::{BlockShape, Voxel};

/// Homogeneity classification of a chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkHomogeneity {
    /// 100% Air (no voxels present).
    Empty,
    /// 100% filled with a single solid opaque voxel material.
    Solid(Voxel),
    /// Contains mixed materials, partial air, or fluids.
    Mixed,
}

/// Two-tiered paletted and compressed chunk voxel storage.
#[derive(Clone, Debug, PartialEq)]
pub enum ChunkStorage {
    /// 100% single voxel material (e.g. 100% Air, 100% Stone). 0 heap bytes allocated.
    Uniform(Voxel),
    /// Up to 16 distinct block types packed as 4-bit nibbles (2 voxels per byte = 2,048 heap bytes).
    Paletted {
        palette: [Voxel; 16],
        palette_len: u8,
        indices: Box<[u8; CHUNK_VOLUME / 2]>,
    },
    /// Flat uncompressed buffer (4,096 heap bytes) for complex chunks with > 16 distinct materials.
    Dense(Box<[Voxel; CHUNK_VOLUME]>),
}

impl ChunkStorage {
    /// Branchless fast-path voxel extraction.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Voxel {
        match self {
            Self::Uniform(v) => *v,
            Self::Paletted {
                palette, indices, ..
            } => {
                let packed = indices[index >> 1];
                let shift = (index & 1) * 4;
                let pal_idx = (packed >> shift) & 0x0F;
                palette[pal_idx as usize]
            }
            Self::Dense(voxels) => voxels[index],
        }
    }

    /// Updates a single voxel, promoting/demoting storage tiers if necessary.
    pub fn set(&mut self, index: usize, new_voxel: Voxel, unique_count: u16) {
        if unique_count == 1 {
            *self = Self::Uniform(new_voxel);
            return;
        }

        match self {
            Self::Uniform(old_voxel) => {
                let old = *old_voxel;
                let mut palette = [Voxel::Air; 16];
                palette[0] = old;
                palette[1] = new_voxel;
                let mut indices = Box::new([0u8; CHUNK_VOLUME / 2]);
                let shift = (index & 1) * 4;
                indices[index >> 1] = 1 << shift;
                *self = Self::Paletted {
                    palette,
                    palette_len: 2,
                    indices,
                };
            }
            Self::Paletted {
                palette,
                palette_len,
                indices,
            } => {
                let mut existing_pal_idx = None;
                for (i, v) in palette[..*palette_len as usize].iter().enumerate() {
                    if *v == new_voxel {
                        existing_pal_idx = Some(i as u8);
                        break;
                    }
                }

                if let Some(pal_idx) = existing_pal_idx {
                    let byte = &mut indices[index >> 1];
                    let shift = (index & 1) * 4;
                    let mask = 0x0F << shift;
                    *byte = (*byte & !mask) | ((pal_idx & 0x0F) << shift);
                } else if (*palette_len as usize) < 16 {
                    let pal_idx = *palette_len;
                    palette[pal_idx as usize] = new_voxel;
                    *palette_len += 1;

                    let byte = &mut indices[index >> 1];
                    let shift = (index & 1) * 4;
                    let mask = 0x0F << shift;
                    *byte = (*byte & !mask) | ((pal_idx & 0x0F) << shift);
                } else {
                    // Overflow > 16 materials: promote to Dense
                    let mut dense = Box::new([Voxel::Air; CHUNK_VOLUME]);
                    for i in 0..CHUNK_VOLUME {
                        let packed = indices[i >> 1];
                        let shift = (i & 1) * 4;
                        let pal_idx = (packed >> shift) & 0x0F;
                        dense[i] = palette[pal_idx as usize];
                    }
                    dense[index] = new_voxel;
                    *self = Self::Dense(dense);
                }
            }
            Self::Dense(voxels) => {
                voxels[index] = new_voxel;
            }
        }
    }

    /// Constructs the most compact storage representation for a slice of 4,096 voxels.
    pub fn from_voxels_slice(voxels: &[Voxel]) -> Self {
        assert_eq!(voxels.len(), CHUNK_VOLUME);

        let mut palette = [Voxel::Air; 16];
        let mut palette_len: u8 = 0;
        let mut voxel_to_pal = [0xFFu8; MAX_VOXEL_TYPES];

        for &v in voxels {
            let idx = v as usize;
            if voxel_to_pal[idx] == 0xFF {
                if (palette_len as usize) < 16 {
                    voxel_to_pal[idx] = palette_len;
                    palette[palette_len as usize] = v;
                    palette_len += 1;
                } else {
                    let mut dense = Box::new([Voxel::Air; CHUNK_VOLUME]);
                    dense.copy_from_slice(voxels);
                    return Self::Dense(dense);
                }
            }
        }

        if palette_len <= 1 {
            return Self::Uniform(voxels[0]);
        }

        let mut indices = Box::new([0u8; CHUNK_VOLUME / 2]);
        for (i, &v) in voxels.iter().enumerate() {
            let pal_idx = voxel_to_pal[v as usize];
            let shift = (i & 1) * 4;
            indices[i >> 1] |= (pal_idx & 0x0F) << shift;
        }

        Self::Paletted {
            palette,
            palette_len,
            indices,
        }
    }
}

#[derive(Clone)]
pub struct Chunk {
    storage: ChunkStorage,
    non_air_count: usize,
    solid_opaque_count: usize,
    variant_counts: [u16; MAX_VOXEL_TYPES],
    unique_voxel_count: u16,
    homogeneity: ChunkHomogeneity,
    shapes: HashMap<usize, (BlockShape, u8)>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::filled(Voxel::Air)
    }

    pub fn filled(voxel: Voxel) -> Self {
        let mut variant_counts = [0u16; MAX_VOXEL_TYPES];
        variant_counts[voxel as usize] = CHUNK_VOLUME as u16;
        let non_air_count = if voxel.is_empty() { 0 } else { CHUNK_VOLUME };
        let solid_opaque_count = if voxel.is_solid_opaque() {
            CHUNK_VOLUME
        } else {
            0
        };

        let homogeneity = if non_air_count == 0 {
            ChunkHomogeneity::Empty
        } else if solid_opaque_count == CHUNK_VOLUME {
            ChunkHomogeneity::Solid(voxel)
        } else {
            ChunkHomogeneity::Mixed
        };

        Self {
            storage: ChunkStorage::Uniform(voxel),
            non_air_count,
            solid_opaque_count,
            variant_counts,
            unique_voxel_count: 1,
            homogeneity,
            shapes: HashMap::default(),
        }
    }

    /// Constructs a chunk directly from a raw 4096-voxel vector in a single linear pass.
    pub fn from_voxels(voxels: Vec<Voxel>) -> Self {
        assert_eq!(voxels.len(), CHUNK_VOLUME);
        let mut non_air_count = 0;
        let mut solid_opaque_count = 0;
        let mut variant_counts = [0u16; MAX_VOXEL_TYPES];
        let mut unique_voxel_count = 0;

        for &v in &voxels {
            let idx = v as usize;
            if variant_counts[idx] == 0 {
                unique_voxel_count += 1;
            }
            variant_counts[idx] += 1;

            if !v.is_empty() {
                non_air_count += 1;
                if v.is_solid_opaque() {
                    solid_opaque_count += 1;
                }
            }
        }

        let homogeneity = if non_air_count == 0 {
            ChunkHomogeneity::Empty
        } else if unique_voxel_count == 1 && solid_opaque_count == CHUNK_VOLUME {
            ChunkHomogeneity::Solid(voxels[0])
        } else {
            ChunkHomogeneity::Mixed
        };

        let storage = ChunkStorage::from_voxels_slice(&voxels);

        Self {
            storage,
            non_air_count,
            solid_opaque_count,
            variant_counts,
            unique_voxel_count,
            homogeneity,
            shapes: HashMap::default(),
        }
    }

    #[inline]
    pub fn homogeneity(&self) -> ChunkHomogeneity {
        self.homogeneity
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.homogeneity == ChunkHomogeneity::Empty
    }

    #[inline]
    pub fn is_fully_solid_opaque(&self) -> bool {
        self.solid_opaque_count == CHUNK_VOLUME
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        match self.homogeneity {
            ChunkHomogeneity::Empty => Voxel::Air,
            ChunkHomogeneity::Solid(voxel) => voxel,
            ChunkHomogeneity::Mixed => self.storage.get(Self::index(x, y, z)),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let old = self.get(x, y, z);
        if old == voxel {
            return;
        }

        let index = Self::index(x, y, z);
        let old_idx = old as usize;
        let new_idx = voxel as usize;

        self.variant_counts[old_idx] -= 1;
        if self.variant_counts[old_idx] == 0 {
            self.unique_voxel_count -= 1;
        }

        if self.variant_counts[new_idx] == 0 {
            self.unique_voxel_count += 1;
        }
        self.variant_counts[new_idx] += 1;

        if old.is_empty() {
            self.non_air_count += 1;
        } else if voxel.is_empty() {
            self.non_air_count -= 1;
        }

        let old_is_solid_opaque = old.is_solid_opaque();
        let new_is_solid_opaque = voxel.is_solid_opaque();
        if !old_is_solid_opaque && new_is_solid_opaque {
            self.solid_opaque_count += 1;
        } else if old_is_solid_opaque && !new_is_solid_opaque {
            self.solid_opaque_count -= 1;
        }

        self.storage.set(index, voxel, self.unique_voxel_count);

        if voxel.is_empty() {
            self.shapes.remove(&index);
        }

        self.homogeneity = if self.non_air_count == 0 {
            self.storage = ChunkStorage::Uniform(Voxel::Air);
            ChunkHomogeneity::Empty
        } else if self.unique_voxel_count == 1 && self.solid_opaque_count == CHUNK_VOLUME {
            self.storage = ChunkStorage::Uniform(voxel);
            ChunkHomogeneity::Solid(voxel)
        } else {
            ChunkHomogeneity::Mixed
        };
    }

    #[inline]
    pub fn get_shape(&self, x: usize, y: usize, z: usize) -> (BlockShape, u8) {
        if self.shapes.is_empty() {
            (BlockShape::Full, 0)
        } else {
            self.shapes
                .get(&Self::index(x, y, z))
                .copied()
                .unwrap_or((BlockShape::Full, 0))
        }
    }

    pub fn set_shape(&mut self, x: usize, y: usize, z: usize, shape: BlockShape, orientation: u8) {
        let index = Self::index(x, y, z);
        if shape == BlockShape::Full {
            self.shapes.remove(&index);
        } else {
            self.shapes.insert(index, (shape, orientation));
        }
    }

    #[inline]
    pub fn has_shapes(&self) -> bool {
        !self.shapes.is_empty()
    }

    #[inline]
    pub fn shapes(&self) -> &HashMap<usize, (BlockShape, u8)> {
        &self.shapes
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE);
        debug_assert!(y < CHUNK_SIZE);
        debug_assert!(z < CHUNK_SIZE);

        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline]
    pub fn index_to_xyz(index: usize) -> (usize, usize, usize) {
        let x = index % CHUNK_SIZE;
        let z = (index / CHUNK_SIZE) % CHUNK_SIZE;
        let y = index / (CHUNK_SIZE * CHUNK_SIZE);
        (x, y, z)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
