pub const VOXEL_SIZE: f32 = 0.5;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub use super::block::Voxel;

#[derive(Clone)]
pub struct Chunk {
    voxels: Vec<Voxel>,
    non_air_count: usize,
    solid_opaque_count: usize,
}

impl Chunk {
    pub fn new() -> Self {
        Self::filled(Voxel::Air)
    }

    pub fn filled(voxel: Voxel) -> Self {
        let non_air_count = if voxel.is_empty() { 0 } else { CHUNK_VOLUME };
        let solid_opaque_count = if !voxel.is_empty() && !voxel.is_transparent() {
            CHUNK_VOLUME
        } else {
            0
        };

        Self {
            voxels: vec![voxel; CHUNK_VOLUME],
            non_air_count,
            solid_opaque_count,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.non_air_count == 0
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

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let index = Self::index(x, y, z);
        let old = self.voxels[index];
        if old != voxel {
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
        assert!(!chunk.is_fully_solid_opaque());
        assert_eq!(chunk.non_air_count(), 0);
        assert_eq!(chunk.solid_opaque_count(), 0);

        // Add 1 solid opaque voxel
        chunk.set(0, 0, 0, Voxel::Stone);
        assert!(!chunk.is_empty());
        assert_eq!(chunk.non_air_count(), 1);
        assert_eq!(chunk.solid_opaque_count(), 1);

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
        assert_eq!(chunk.non_air_count(), 0);

        // Filled with solid stone
        let full_chunk = Chunk::filled(Voxel::Stone);
        assert!(!full_chunk.is_empty());
        assert!(full_chunk.is_fully_solid_opaque());
        assert_eq!(full_chunk.non_air_count(), CHUNK_VOLUME);
        assert_eq!(full_chunk.solid_opaque_count(), CHUNK_VOLUME);
    }
}
