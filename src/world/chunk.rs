pub const VOXEL_SIZE: f32 = 0.5;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub use super::block::Voxel;

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
    /// Run-Length Encoded runs for compact disk serialization and hibernation.
    Rle(Vec<(Voxel, u16)>),
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
            Self::Rle(runs) => {
                let mut accumulated = 0;
                for &(voxel, len) in runs {
                    accumulated += len as usize;
                    if index < accumulated {
                        return voxel;
                    }
                }
                Voxel::Air
            }
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
            Self::Rle(runs) => {
                let mut dense = Box::new([Voxel::Air; CHUNK_VOLUME]);
                let mut idx = 0;
                for &(v, len) in runs.iter() {
                    for _ in 0..len {
                        if idx < CHUNK_VOLUME {
                            dense[idx] = v;
                            idx += 1;
                        }
                    }
                }
                dense[index] = new_voxel;
                *self = Self::Dense(dense);
            }
        }
    }

    /// Constructs the most compact storage representation for a slice of 4,096 voxels.
    pub fn from_voxels_slice(voxels: &[Voxel]) -> Self {
        assert_eq!(voxels.len(), CHUNK_VOLUME);

        let mut palette = [Voxel::Air; 16];
        let mut palette_len: u8 = 0;
        let mut voxel_to_pal = [0xFFu8; 64];

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

    /// Converts the current storage representation to Run-Length Encoded runs.
    pub fn to_rle(&self) -> Vec<(Voxel, u16)> {
        match self {
            Self::Uniform(v) => vec![(*v, CHUNK_VOLUME as u16)],
            Self::Rle(runs) => runs.clone(),
            _ => {
                let mut runs = Vec::new();
                let first = self.get(0);
                let mut current_voxel = first;
                let mut current_run: u16 = 1;

                for idx in 1..CHUNK_VOLUME {
                    let v = self.get(idx);
                    if v == current_voxel && current_run < u16::MAX {
                        current_run += 1;
                    } else {
                        runs.push((current_voxel, current_run));
                        current_voxel = v;
                        current_run = 1;
                    }
                }
                runs.push((current_voxel, current_run));
                runs
            }
        }
    }

    /// Reconstructs compact storage from Run-Length Encoded runs.
    pub fn from_rle(runs: &[(Voxel, u16)]) -> Self {
        if runs.len() == 1 {
            return Self::Uniform(runs[0].0);
        }

        let mut voxels = vec![Voxel::Air; CHUNK_VOLUME];
        let mut idx = 0;
        for &(v, len) in runs {
            for _ in 0..len {
                if idx < CHUNK_VOLUME {
                    voxels[idx] = v;
                    idx += 1;
                }
            }
        }

        Self::from_voxels_slice(&voxels)
    }

    /// Returns the exact heap memory allocated by this chunk storage in bytes.
    pub fn memory_size(&self) -> usize {
        match self {
            Self::Uniform(_) => 0,
            Self::Paletted { .. } => CHUNK_VOLUME / 2, // 2,048 bytes
            Self::Dense(_) => CHUNK_VOLUME,             // 4,096 bytes
            Self::Rle(runs) => runs.len() * std::mem::size_of::<(Voxel, u16)>(),
        }
    }
}

#[derive(Clone)]
pub struct Chunk {
    storage: ChunkStorage,
    non_air_count: usize,
    solid_opaque_count: usize,
    variant_counts: [u16; 64],
    unique_voxel_count: u16,
    homogeneity: ChunkHomogeneity,
}

impl Chunk {
    pub fn new() -> Self {
        Self::filled(Voxel::Air)
    }

    pub fn filled(voxel: Voxel) -> Self {
        let mut variant_counts = [0u16; 64];
        variant_counts[voxel as usize] = CHUNK_VOLUME as u16;
        let non_air_count = if voxel.is_empty() { 0 } else { CHUNK_VOLUME };
        let solid_opaque_count = if !voxel.is_empty() && !voxel.is_transparent() {
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
        }
    }

    /// Constructs a chunk directly from a raw 4096-voxel vector in a single linear pass.
    pub fn from_voxels(voxels: Vec<Voxel>) -> Self {
        assert_eq!(voxels.len(), CHUNK_VOLUME);
        let mut non_air_count = 0;
        let mut solid_opaque_count = 0;
        let mut variant_counts = [0u16; 64];
        let mut unique_voxel_count = 0;

        for &v in &voxels {
            let idx = v as usize;
            if variant_counts[idx] == 0 {
                unique_voxel_count += 1;
            }
            variant_counts[idx] += 1;

            if !v.is_empty() {
                non_air_count += 1;
                if !v.is_transparent() {
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
        }
    }

    #[inline]
    pub fn homogeneity(&self) -> ChunkHomogeneity {
        self.homogeneity
    }

    #[inline]
    pub fn storage(&self) -> &ChunkStorage {
        &self.storage
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.homogeneity == ChunkHomogeneity::Empty
    }

    #[inline]
    pub fn is_fully_solid_opaque(&self) -> bool {
        self.solid_opaque_count == CHUNK_VOLUME
    }

    #[allow(dead_code)]
    #[inline]
    pub fn non_air_count(&self) -> usize {
        self.non_air_count
    }

    #[allow(dead_code)]
    #[inline]
    pub fn solid_opaque_count(&self) -> usize {
        self.solid_opaque_count
    }

    #[allow(dead_code)]
    #[inline]
    pub fn unique_voxel_count(&self) -> u16 {
        self.unique_voxel_count
    }

    #[inline]
    pub fn memory_size(&self) -> usize {
        self.storage.memory_size()
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

        let old_is_solid_opaque = !old.is_empty() && !old.is_transparent();
        let new_is_solid_opaque = !voxel.is_empty() && !voxel.is_transparent();
        if !old_is_solid_opaque && new_is_solid_opaque {
            self.solid_opaque_count += 1;
        } else if old_is_solid_opaque && !new_is_solid_opaque {
            self.solid_opaque_count -= 1;
        }

        self.storage.set(index, voxel, self.unique_voxel_count);

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

    pub fn to_rle(&self) -> Vec<(Voxel, u16)> {
        self.storage.to_rle()
    }

    pub fn from_rle(runs: &[(Voxel, u16)]) -> Self {
        let storage = ChunkStorage::from_rle(runs);
        let mut voxels = Vec::with_capacity(CHUNK_VOLUME);
        for i in 0..CHUNK_VOLUME {
            voxels.push(storage.get(i));
        }
        Self::from_voxels(voxels)
    }

    #[allow(dead_code)]
    pub fn compress_rle(&mut self) {
        if !matches!(self.storage, ChunkStorage::Rle(_)) {
            let rle = self.to_rle();
            self.storage = ChunkStorage::Rle(rle);
        }
    }

    #[allow(dead_code)]
    pub fn decompress_rle(&mut self) {
        if let ChunkStorage::Rle(runs) = &self.storage {
            self.storage = ChunkStorage::from_rle(runs);
        }
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE);
        debug_assert!(y < CHUNK_SIZE);
        debug_assert!(z < CHUNK_SIZE);

        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_occupancy_and_solid_tracking() {
        let mut chunk = Chunk::new();
        assert!(chunk.is_empty());
        assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Empty);
        assert_eq!(chunk.memory_size(), 0);
        assert!(!chunk.is_fully_solid_opaque());
        assert_eq!(chunk.non_air_count(), 0);
        assert_eq!(chunk.solid_opaque_count(), 0);

        // Add 1 solid opaque voxel -> becomes Paletted
        chunk.set(0, 0, 0, Voxel::Stone);
        assert!(!chunk.is_empty());
        assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Mixed);
        assert_eq!(chunk.memory_size(), 2048);
        assert_eq!(chunk.non_air_count(), 1);
        assert_eq!(chunk.solid_opaque_count(), 1);
        assert_eq!(chunk.get(0, 0, 0), Voxel::Stone);
        assert_eq!(chunk.get(1, 0, 0), Voxel::Air);

        // Add 1 transparent fluid voxel
        chunk.set(1, 0, 0, Voxel::Water);
        assert_eq!(chunk.non_air_count(), 2);
        assert_eq!(chunk.solid_opaque_count(), 1);

        // Remove the solid voxel back to Air
        chunk.set(0, 0, 0, Voxel::Air);
        assert_eq!(chunk.non_air_count(), 1);
        assert_eq!(chunk.solid_opaque_count(), 0);

        // Remove the water voxel back to Air -> demotes back to Uniform (0 heap bytes)
        chunk.set(1, 0, 0, Voxel::Air);
        assert!(chunk.is_empty());
        assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Empty);
        assert_eq!(chunk.memory_size(), 0);
        assert_eq!(chunk.non_air_count(), 0);

        // Filled with solid stone
        let full_chunk = Chunk::filled(Voxel::Stone);
        assert!(!full_chunk.is_empty());
        assert_eq!(
            full_chunk.homogeneity(),
            ChunkHomogeneity::Solid(Voxel::Stone)
        );
        assert_eq!(full_chunk.memory_size(), 0);
        assert!(full_chunk.is_fully_solid_opaque());
        assert_eq!(full_chunk.non_air_count(), CHUNK_VOLUME);
        assert_eq!(full_chunk.solid_opaque_count(), CHUNK_VOLUME);
        assert_eq!(full_chunk.get(5, 5, 5), Voxel::Stone);
    }

    #[test]
    fn chunk_homogeneity_transitions() {
        let mut chunk = Chunk::filled(Voxel::Stone);
        assert_eq!(
            chunk.homogeneity(),
            ChunkHomogeneity::Solid(Voxel::Stone)
        );
        assert_eq!(chunk.memory_size(), 0);

        // Edit one voxel to air -> Mixed & Paletted (2048 bytes)
        chunk.set(0, 0, 0, Voxel::Air);
        assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Mixed);
        assert_eq!(chunk.memory_size(), 2048);
        assert_eq!(chunk.get(0, 0, 0), Voxel::Air);
        assert_eq!(chunk.get(0, 0, 1), Voxel::Stone);

        // Edit it back to stone -> Solid(Stone) & Uniform (0 bytes)
        chunk.set(0, 0, 0, Voxel::Stone);
        assert_eq!(
            chunk.homogeneity(),
            ChunkHomogeneity::Solid(Voxel::Stone)
        );
        assert_eq!(chunk.memory_size(), 0);

        // Edit all to Granite via from_voxels
        let granite_voxels = vec![Voxel::Granite; CHUNK_VOLUME];
        let granite_chunk = Chunk::from_voxels(granite_voxels);
        assert_eq!(
            granite_chunk.homogeneity(),
            ChunkHomogeneity::Solid(Voxel::Granite)
        );
        assert_eq!(granite_chunk.memory_size(), 0);
        assert_eq!(granite_chunk.get(8, 8, 8), Voxel::Granite);
    }

    #[test]
    fn test_paletted_branchless_access_and_packing() {
        let mut voxels = vec![Voxel::Stone; CHUNK_VOLUME];
        voxels[0] = Voxel::Grass;
        voxels[1] = Voxel::Dirt;
        voxels[2] = Voxel::Sand;
        voxels[3] = Voxel::Gravel;

        let chunk = Chunk::from_voxels(voxels);
        assert!(matches!(chunk.storage(), ChunkStorage::Paletted { .. }));
        assert_eq!(chunk.memory_size(), 2048);

        assert_eq!(chunk.get(0, 0, 0), Voxel::Grass);
        assert_eq!(chunk.get(1, 0, 0), Voxel::Dirt);
        assert_eq!(chunk.get(2, 0, 0), Voxel::Sand);
        assert_eq!(chunk.get(3, 0, 0), Voxel::Gravel);
        assert_eq!(chunk.get(4, 0, 0), Voxel::Stone);
    }

    #[test]
    fn test_dense_overflow_when_exceeding_16_materials() {
        let mut chunk = Chunk::filled(Voxel::Air);
        // Add 17 distinct materials to force overflow into Dense
        let materials = [
            Voxel::Grass,
            Voxel::Dirt,
            Voxel::Stone,
            Voxel::Cobblestone,
            Voxel::Sand,
            Voxel::Gravel,
            Voxel::Clay,
            Voxel::Mud,
            Voxel::Mulch,
            Voxel::Snow,
            Voxel::Ice,
            Voxel::Andesite,
            Voxel::Diorite,
            Voxel::Granite,
            Voxel::Tuff,
            Voxel::Sandstone,
            Voxel::Slate,
        ];

        for (i, &mat) in materials.iter().enumerate() {
            chunk.set(i % 16, i / 16, 0, mat);
        }

        assert!(matches!(chunk.storage(), ChunkStorage::Dense(_)));
        assert_eq!(chunk.memory_size(), 4096);

        for (i, &mat) in materials.iter().enumerate() {
            assert_eq!(chunk.get(i % 16, i / 16, 0), mat);
        }
    }

    #[test]
    fn test_rle_lossless_roundtrip() {
        let mut voxels = vec![Voxel::Air; CHUNK_VOLUME];
        for i in 0..100 {
            voxels[i] = Voxel::Stone;
        }
        for i in 100..500 {
            voxels[i] = Voxel::Dirt;
        }

        let chunk = Chunk::from_voxels(voxels.clone());
        let rle = chunk.to_rle();
        let reconstructed = Chunk::from_rle(&rle);

        for i in 0..CHUNK_VOLUME {
            assert_eq!(
                chunk.storage.get(i),
                reconstructed.storage.get(i),
                "Mismatch at index {i}"
            );
        }
    }
}
