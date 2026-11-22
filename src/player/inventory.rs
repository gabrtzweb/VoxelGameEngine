use bevy::prelude::*;

use crate::world::Voxel;

pub const PLAYER_INVENTORY_COLS: usize = 9;
pub const PLAYER_INVENTORY_ROWS: usize = 4;
pub const PLAYER_INVENTORY_SLOTS: usize = PLAYER_INVENTORY_COLS * PLAYER_INVENTORY_ROWS; // 36 slots
pub const BASE_UNLOCKED_INVENTORY_SLOTS: usize = 27;

#[derive(Resource, Debug, Clone)]
pub struct PlayerInventory {
    pub slots: [Option<Voxel>; PLAYER_INVENTORY_SLOTS],
    pub counts: [u16; PLAYER_INVENTORY_SLOTS],
    pub unlocked_slots: usize,
    pub offhand_slot: Option<Voxel>,
    pub offhand_count: u16,
    pub utility_slot: Option<Voxel>,
    pub utility_count: u16,
    #[allow(dead_code)]
    pub armor_slots: [Option<Voxel>; 6],
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self {
            slots: [None; PLAYER_INVENTORY_SLOTS],
            counts: [0; PLAYER_INVENTORY_SLOTS],
            unlocked_slots: BASE_UNLOCKED_INVENTORY_SLOTS,
            offhand_slot: None,
            offhand_count: 0,
            utility_slot: None,
            utility_count: 0,
            armor_slots: [None; 6],
        }
    }
}

#[allow(dead_code)]
impl PlayerInventory {
    #[inline]
    pub fn is_slot_unlocked(&self, slot: usize) -> bool {
        slot < self.unlocked_slots.min(PLAYER_INVENTORY_SLOTS)
    }

    #[inline]
    pub fn get(&self, slot: usize) -> Option<Voxel> {
        self.slots.get(slot).copied().flatten()
    }

    #[inline]
    pub fn set(&mut self, slot: usize, voxel: Option<Voxel>) {
        if self.is_slot_unlocked(slot) {
            self.slots[slot] = voxel;
            self.counts[slot] = if voxel.is_some() { 1 } else { 0 };
        }
    }

    #[inline]
    pub fn first_empty_slot(&self) -> Option<usize> {
        let limit = self.unlocked_slots.min(PLAYER_INVENTORY_SLOTS);
        self.slots[..limit].iter().position(|s| s.is_none())
    }

    pub fn try_stack_existing(&mut self, voxel: Voxel, mut amount: u16) -> u16 {
        let max_stack = voxel.max_stack_size();
        let limit = self.unlocked_slots.min(PLAYER_INVENTORY_SLOTS);
        for i in 0..limit {
            if amount == 0 {
                break;
            }
            if self.slots[i] == Some(voxel) && self.counts[i] < max_stack {
                let space = max_stack - self.counts[i];
                let add = amount.min(space);
                self.counts[i] += add;
                amount -= add;
            }
        }
        amount
    }

    pub fn add_item_stack(&mut self, voxel: Voxel, amount: u16) -> u16 {
        let mut remaining = self.try_stack_existing(voxel, amount);
        let max_stack = voxel.max_stack_size();
        let limit = self.unlocked_slots.min(PLAYER_INVENTORY_SLOTS);
        for i in 0..limit {
            if remaining == 0 {
                break;
            }
            if self.slots[i].is_none() {
                let add = remaining.min(max_stack);
                self.slots[i] = Some(voxel);
                self.counts[i] = add;
                remaining -= add;
            }
        }
        remaining
    }

    pub fn add_item(&mut self, voxel: Voxel) -> bool {
        self.add_item_stack(voxel, 1) == 0
    }
}
