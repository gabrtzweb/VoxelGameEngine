use bevy::prelude::*;

use crate::world::Voxel;

pub const PLAYER_INVENTORY_COLS: usize = 8;
pub const PLAYER_INVENTORY_ROWS: usize = 4;
pub const PLAYER_INVENTORY_SLOTS: usize = PLAYER_INVENTORY_COLS * PLAYER_INVENTORY_ROWS; // 32 slots

/// Represents the player's personal persistent inventory storage (8 columns x 4 rows).
#[derive(Resource, Debug, Clone)]
pub struct PlayerInventory {
    pub slots: [Option<Voxel>; PLAYER_INVENTORY_SLOTS],
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self {
            slots: [None; PLAYER_INVENTORY_SLOTS],
        }
    }
}

impl PlayerInventory {
    #[inline]
    pub fn get(&self, slot: usize) -> Option<Voxel> {
        self.slots.get(slot).copied().flatten()
    }

    #[inline]
    pub fn set(&mut self, slot: usize, voxel: Option<Voxel>) {
        if slot < PLAYER_INVENTORY_SLOTS {
            self.slots[slot] = voxel;
        }
    }

    #[inline]
    pub fn first_empty_slot(&self) -> Option<usize> {
        self.slots.iter().position(|s| s.is_none())
    }

    pub fn add_item(&mut self, voxel: Voxel) -> bool {
        if let Some(slot) = self.first_empty_slot() {
            self.slots[slot] = Some(voxel);
            true
        } else {
            false
        }
    }
}
