use bevy::prelude::*;

use crate::world::Voxel;

pub const PLAYER_INVENTORY_COLS: usize = 9;
pub const PLAYER_INVENTORY_ROWS: usize = 3;
pub const PLAYER_INVENTORY_SLOTS: usize = PLAYER_INVENTORY_COLS * PLAYER_INVENTORY_ROWS; // 27 slots

/// Represents the player's personal persistent inventory storage (9 columns x 4 rows)
/// plus equipment and auxiliary slots.
#[derive(Resource, Debug, Clone)]
pub struct PlayerInventory {
    pub slots: [Option<Voxel>; PLAYER_INVENTORY_SLOTS],
    pub offhand_slot: Option<Voxel>,
    pub utility_slot: Option<Voxel>,
    #[allow(dead_code)]
    pub armor_slots: [Option<Voxel>; 6],
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self {
            slots: [None; PLAYER_INVENTORY_SLOTS],
            offhand_slot: None,
            utility_slot: None,
            armor_slots: [None; 6],
        }
    }
}

#[allow(dead_code)]
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
