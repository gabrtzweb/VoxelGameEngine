pub const VOXEL_SIZE: f32 = 0.5;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Voxel {
    #[default]
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
    Water = 5,
}

impl Voxel {
    pub fn is_empty(self) -> bool {
        self == Self::Air
    }

    pub fn is_collidable(self) -> bool {
        match self {
            Self::Air => false,

            Self::Grass | Self::Dirt | Self::Stone | Self::Sand | Self::Water => true,
        }
    }

    pub fn occludes_faces(self) -> bool {
        match self {
            Self::Air => false,

            Self::Grass | Self::Dirt | Self::Stone | Self::Sand | Self::Water => true,
        }
    }

    pub fn display_color(self) -> [f32; 4] {
        match self {
            Self::Air => [0.0, 0.0, 0.0, 0.0],

            Self::Grass => [0.32, 0.62, 0.25, 1.0],

            Self::Dirt => [0.42, 0.26, 0.13, 1.0],

            Self::Stone => [0.48, 0.50, 0.52, 1.0],

            Self::Sand => [0.82, 0.76, 0.50, 1.0],

            Self::Water => [0.15, 0.38, 0.85, 1.0],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Air => "Air",
            Self::Grass => "Grass",
            Self::Dirt => "Dirt",
            Self::Stone => "Stone",
            Self::Sand => "Sand",
            Self::Water => "Water",
        }
    }
}

#[derive(Clone)]
pub struct Chunk {
    voxels: Vec<Voxel>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::filled(Voxel::Air)
    }

    pub fn filled(voxel: Voxel) -> Self {
        Self {
            voxels: vec![voxel; CHUNK_VOLUME],
        }
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
