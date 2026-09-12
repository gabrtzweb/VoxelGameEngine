use bevy::prelude::IVec3;

pub const VOXEL_SIZE: f32 = 0.5;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Voxel {
    #[default]
    Air = 0,

    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
    Water = 5,
    Light = 6,
    Occupied = 7,
    WaterFlowing = 8,
    WaterOccupied = 9,
}

impl Voxel {
    pub const ALL: [Voxel; 7] = [
        Voxel::Grass,
        Voxel::Dirt,
        Voxel::Stone,
        Voxel::Sand,
        Voxel::Water,
        Voxel::WaterFlowing,
        Voxel::Light,
    ];

    pub fn texture_name(self) -> Option<&'static str> {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => None,
            Self::Grass => Some("terr_grass"),
            Self::Dirt => Some("terr_dirt"),
            Self::Stone => Some("rock_stone"),
            Self::Sand => Some("terr_sand"),
            Self::Water => Some("liqd_water_still"),
            Self::WaterFlowing => Some("liqd_water_flow"),
            Self::Light => Some("emit_light"),
        }
    }

    pub fn tint_color(self) -> [f32; 4] {
        match self {
            Self::Grass => [0.58, 0.90, 0.44, 1.0],
            Self::Water | Self::WaterFlowing | Self::WaterOccupied => [0.40, 0.80, 1.0, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn tint_color_at(self, _world_voxel: IVec3) -> [f32; 4] {
        self.tint_color()
    }

    pub fn fallback_color(self) -> [u8; 4] {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => [0, 0, 0, 0],
            Self::Grass => [255, 255, 255, 255],
            Self::Dirt => [107, 66, 33, 255],
            Self::Stone => [122, 128, 133, 255],
            Self::Sand => [209, 194, 128, 255],
            Self::Water | Self::WaterFlowing => [255, 255, 255, 255],
            Self::Light => [255, 199, 64, 255],
        }
    }

    pub fn is_empty(self) -> bool {
        self == Self::Air
    }

    pub fn is_water(self) -> bool {
        matches!(self, Self::Water | Self::WaterFlowing | Self::WaterOccupied)
    }

    pub fn is_collidable(self) -> bool {
        match self {
            Self::Air | Self::Water | Self::WaterFlowing => false,

            Self::Grass
            | Self::Dirt
            | Self::Stone
            | Self::Sand
            | Self::Light
            | Self::Occupied
            | Self::WaterOccupied => true,
        }
    }

    pub fn is_transparent(self) -> bool {
        self.is_water()
    }

    #[allow(dead_code)]
    pub fn display_color(self) -> [f32; 4] {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => [0.0, 0.0, 0.0, 0.0],

            Self::Grass => [0.58, 0.90, 0.44, 1.0],

            Self::Dirt => [0.42, 0.26, 0.13, 1.0],

            Self::Stone => [0.48, 0.50, 0.52, 1.0],

            Self::Sand => [0.82, 0.76, 0.50, 1.0],

            Self::Water | Self::WaterFlowing => [0.40, 0.80, 1.0, 1.0],

            Self::Light => [1.0, 0.78, 0.25, 1.0],
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
            Self::WaterFlowing => "Flowing Water",
            Self::Light => "Light",
            Self::Occupied => "Occupied",
            Self::WaterOccupied => "Waterlogged Occupied",
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
