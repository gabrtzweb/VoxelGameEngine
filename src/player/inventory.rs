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

#[allow(dead_code)]
impl PlayerInventory {
    pub fn new() -> Self {
        Self::default()
    }

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
    pub fn swap(&mut self, a: usize, b: usize) {
        if a < PLAYER_INVENTORY_SLOTS && b < PLAYER_INVENTORY_SLOTS {
            self.slots.swap(a, b);
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

    pub fn clear(&mut self) {
        self.slots.fill(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_inventory_dimensions() {
        assert_eq!(PLAYER_INVENTORY_COLS, 8);
        assert_eq!(PLAYER_INVENTORY_ROWS, 4);
        assert_eq!(PLAYER_INVENTORY_SLOTS, 32);
        let inv = PlayerInventory::default();
        assert_eq!(inv.slots.len(), 32);
        assert!(inv.slots.iter().all(|s| s.is_none()));
    }

    #[test]
    fn test_player_inventory_add_and_swap() {
        let mut inv = PlayerInventory::default();
        assert_eq!(inv.first_empty_slot(), Some(0));

        assert!(inv.add_item(Voxel::Grass));
        assert_eq!(inv.get(0), Some(Voxel::Grass));
        assert_eq!(inv.first_empty_slot(), Some(1));

        assert!(inv.add_item(Voxel::Stone));
        assert_eq!(inv.get(1), Some(Voxel::Stone));

        inv.swap(0, 1);
        assert_eq!(inv.get(0), Some(Voxel::Stone));
        assert_eq!(inv.get(1), Some(Voxel::Grass));
    }

    #[test]
    fn test_player_inventory_full_capacity() {
        let mut inv = PlayerInventory::default();
        for i in 0..PLAYER_INVENTORY_SLOTS {
            assert!(inv.add_item(Voxel::Sand), "failed to add item at slot {i}");
        }

        assert_eq!(inv.first_empty_slot(), None);
        assert!(!inv.add_item(Voxel::Dirt));
    }
}
