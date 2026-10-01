use crate::world::Voxel;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeSpecies {
    Oak,
    Birch,
    Pine,
    Acacia,
    Mahogany,
    Mangrove,
    Maple,
    Palm,
    Willow,
    Yew,
    Charred,
    Dead,
    Cactus,
}

#[allow(dead_code)]
impl TreeSpecies {
    pub const ALL: [TreeSpecies; 13] = [
        Self::Oak,
        Self::Birch,
        Self::Pine,
        Self::Acacia,
        Self::Mahogany,
        Self::Mangrove,
        Self::Maple,
        Self::Palm,
        Self::Willow,
        Self::Yew,
        Self::Charred,
        Self::Dead,
        Self::Cactus,
    ];

    pub fn log_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::Tree_Oak_Log,
            Self::Birch => Voxel::Tree_Birch_Log,
            Self::Pine => Voxel::Tree_Pine_Log,
            Self::Acacia => Voxel::Tree_Acacia_Log,
            Self::Mahogany => Voxel::Tree_Mahogany_Log,
            Self::Mangrove => Voxel::Tree_Mangrove_Log,
            Self::Maple => Voxel::Tree_Maple_Log,
            Self::Palm => Voxel::Tree_Palm_Log,
            Self::Willow => Voxel::Tree_Willow_Log,
            Self::Yew => Voxel::Tree_Yew_Log,
            Self::Charred => Voxel::Tree_Charred_Log,
            Self::Dead => Voxel::Tree_Dead_Log,
            Self::Cactus => Voxel::Tree_Cactus,
        }
    }

    pub fn wood_voxel(self) -> Voxel {
        match self {
            Self::Oak => Voxel::Tree_Oak_Bark,
            Self::Birch => Voxel::Tree_Birch_Bark,
            Self::Pine => Voxel::Tree_Pine_Bark,
            Self::Acacia => Voxel::Tree_Acacia_Bark,
            Self::Mahogany => Voxel::Tree_Mahogany_Bark,
            Self::Mangrove => Voxel::Tree_Mangrove_Bark,
            Self::Maple => Voxel::Tree_Maple_Bark,
            Self::Palm => Voxel::Tree_Palm_Bark,
            Self::Willow => Voxel::Tree_Willow_Bark,
            Self::Yew => Voxel::Tree_Yew_Bark,
            Self::Charred => Voxel::Tree_Charred_Bark,
            Self::Dead => Voxel::Tree_Dead_Bark,
            Self::Cactus => Voxel::Tree_Cactus,
        }
    }

    pub fn leaves_voxel(self) -> Option<Voxel> {
        match self {
            Self::Oak => Some(Voxel::Tree_Oak_Leaves),
            Self::Birch => Some(Voxel::Tree_Birch_Leaves),
            Self::Pine => Some(Voxel::Tree_Pine_Leaves),
            Self::Acacia => Some(Voxel::Tree_Acacia_Leaves),
            Self::Mahogany => Some(Voxel::Tree_Mahogany_Leaves),
            Self::Mangrove => Some(Voxel::Tree_Mangrove_Leaves),
            Self::Maple => Some(Voxel::Tree_Maple_Leaves_Red),
            Self::Palm => Some(Voxel::Tree_Palm_Leaves),
            Self::Willow => Some(Voxel::Tree_Willow_Leaves),
            Self::Yew => Some(Voxel::Tree_Yew_Leaves),
            Self::Charred | Self::Dead | Self::Cactus => None,
        }
    }

    pub fn planks_voxel(self) -> Option<Voxel> {
        match self {
            Self::Oak => Some(Voxel::Tree_Oak_Planks),
            Self::Birch => Some(Voxel::Tree_Birch_Planks),
            Self::Pine => Some(Voxel::Tree_Pine_Planks),
            Self::Acacia => Some(Voxel::Tree_Acacia_Planks),
            Self::Mahogany => Some(Voxel::Tree_Mahogany_Planks),
            Self::Mangrove => Some(Voxel::Tree_Mangrove_Planks),
            Self::Maple => Some(Voxel::Tree_Maple_Planks),
            Self::Palm => Some(Voxel::Tree_Palm_Planks),
            Self::Willow => Some(Voxel::Tree_Willow_Planks),
            Self::Yew => Some(Voxel::Tree_Yew_Planks),
            Self::Charred => Some(Voxel::Tree_Charred_Planks),
            Self::Dead => Some(Voxel::Tree_Dead_Planks),
            Self::Cactus => None,
        }
    }
}
