use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InteractionMode {
    #[default]
    Block,
    Voxel,
}

impl InteractionMode {
    pub fn toggle(&mut self) {
        *self = match *self {
            Self::Block => Self::Voxel,
            Self::Voxel => Self::Block,
        };
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Block => "Block",
            Self::Voxel => "Voxel",
        }
    }
}
