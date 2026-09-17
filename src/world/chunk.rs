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

#[derive(Clone)]
pub struct Chunk {
    voxels: Vec<Voxel>,
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

        let voxels = match homogeneity {
            ChunkHomogeneity::Empty | ChunkHomogeneity::Solid(_) => Vec::new(),
            ChunkHomogeneity::Mixed => vec![voxel; CHUNK_VOLUME],
        };

        Self {
            voxels,
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

        let stored_voxels = match homogeneity {
            ChunkHomogeneity::Empty | ChunkHomogeneity::Solid(_) => Vec::new(),
            ChunkHomogeneity::Mixed => voxels,
        };

        Self {
            voxels: stored_voxels,
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
    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        match self.homogeneity {
            ChunkHomogeneity::Empty => Voxel::Air,
            ChunkHomogeneity::Solid(voxel) => voxel,
            ChunkHomogeneity::Mixed => self.voxels[Self::index(x, y, z)],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let old = self.get(x, y, z);
        if old == voxel {
            return;
        }

        // Ensure voxel storage is materialized if it was lazily empty
        if self.voxels.is_empty() {
            let fill = match self.homogeneity {
                ChunkHomogeneity::Empty => Voxel::Air,
                ChunkHomogeneity::Solid(v) => v,
                ChunkHomogeneity::Mixed => Voxel::Air,
            };
            self.voxels = vec![fill; CHUNK_VOLUME];
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

        self.voxels[index] = voxel;

        self.homogeneity = if self.non_air_count == 0 {
            self.voxels.clear();
            ChunkHomogeneity::Empty
        } else if self.unique_voxel_count == 1 && self.solid_opaque_count == CHUNK_VOLUME {
            self.voxels.clear();
            ChunkHomogeneity::Solid(voxel)
        } else {
            ChunkHomogeneity::Mixed
        };
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
        assert!(!chunk.is_fully_solid_opaque());
        assert_eq!(chunk.non_air_count(), 0);
        assert_eq!(chunk.solid_opaque_count(), 0);

        // Add 1 solid opaque voxel
        chunk.set(0, 0, 0, Voxel::Stone);
        assert!(!chunk.is_empty());
        assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Mixed);
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

        // Remove the water voxel back to Air
        chunk.set(1, 0, 0, Voxel::Air);
        assert!(chunk.is_empty());
        assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Empty);
        assert_eq!(chunk.non_air_count(), 0);

        // Filled with solid stone
        let full_chunk = Chunk::filled(Voxel::Stone);
        assert!(!full_chunk.is_empty());
        assert_eq!(
            full_chunk.homogeneity(),
            ChunkHomogeneity::Solid(Voxel::Stone)
        );
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

        // Edit one voxel to air -> Mixed
        chunk.set(0, 0, 0, Voxel::Air);
        assert_eq!(chunk.homogeneity(), ChunkHomogeneity::Mixed);
        assert_eq!(chunk.get(0, 0, 0), Voxel::Air);
        assert_eq!(chunk.get(0, 0, 1), Voxel::Stone);

        // Edit it back to stone -> Solid(Stone)
        chunk.set(0, 0, 0, Voxel::Stone);
        assert_eq!(
            chunk.homogeneity(),
            ChunkHomogeneity::Solid(Voxel::Stone)
        );

        // Edit all to Granite via from_voxels
        let granite_voxels = vec![Voxel::Granite; CHUNK_VOLUME];
        let granite_chunk = Chunk::from_voxels(granite_voxels);
        assert_eq!(
            granite_chunk.homogeneity(),
            ChunkHomogeneity::Solid(Voxel::Granite)
        );
        assert_eq!(granite_chunk.get(8, 8, 8), Voxel::Granite);
    }
}
