use crate::world::Voxel;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSpecies {
    Oak,
    Birch,
    Pine,
    Cactus,
    Rainwood,
}

#[allow(dead_code)]
impl TreeSpecies {
    pub fn log_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::OakWoodLog,
            Self::Birch => Voxel::BirchWoodLog,
            Self::Pine => Voxel::PineWoodLog,
            Self::Cactus => Voxel::Cactus,
            Self::Rainwood => Voxel::RainwoodWoodLog,
        }
    }

    pub fn wood_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::OakWood,
            Self::Birch => Voxel::BirchWood,
            Self::Pine => Voxel::PineWood,
            Self::Cactus => Voxel::Cactus,
            Self::Rainwood => Voxel::RainwoodWood,
        }
    }

    pub fn leaves_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::OakLeaves,
            Self::Birch => Voxel::BirchLeaves,
            Self::Pine => Voxel::PineLeaves,
            Self::Cactus => Voxel::Cactus,
            Self::Rainwood => Voxel::RainwoodLeaves,
        }
    }
}
