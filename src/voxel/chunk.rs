pub const VOXEL_SIZE: f32 = 0.5;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Voxel {
    #[default]
    Air = 0,
    Solid = 1,
}

pub struct Chunk {
    voxels: Vec<Voxel>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            voxels: vec![Voxel::Air; CHUNK_VOLUME],
        }
    }

    pub fn new_half_solid() -> Self {
        let mut chunk = Self::new();

        for y in 0..CHUNK_SIZE / 2 {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    chunk.set(x, y, z, Voxel::Solid);
                }
            }
        }

        chunk
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let index = Self::index(x, y, z);
        self.voxels[index] = voxel;
    }

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
