use bevy::prelude::*;

use crate::{
    menu::MenuState,
    player::Player,
    simulation::lighting::{VoxelLightRegistry, sync_voxel_light},
    world::{
        BlockShape, ChunkStreamingQueues, Voxel, VoxelWorld, WorldModificationStore,
        affected_chunks,
    },
};

/// Returns true if the given voxel is a tilled soil substrate suitable for planting.
#[inline(always)]
pub fn is_tilled_soil(voxel: Voxel) -> bool {
    matches!(
        voxel,
        Voxel::LoamTilled | Voxel::PeatTilled | Voxel::SiltTilled
    )
}

/// Converts a seed or agricultural planting item into its Stage 0 (or seedling) crop voxel.
pub fn seed_to_crop(seed: Voxel) -> Option<Voxel> {
    match seed {
        Voxel::ItemWheatSeeds | Voxel::ItemWheat => Some(Voxel::CropWheatStage0),
        Voxel::ItemBarleySeeds | Voxel::ItemBarley => Some(Voxel::CropBarleyStage0),
        Voxel::ItemRice => Some(Voxel::CropRiceStage0),
        Voxel::ItemCorn => Some(Voxel::CropCornBuddingStage0),
        Voxel::ItemCarrot => Some(Voxel::CropCarrotsStage0),
        Voxel::ItemPotato => Some(Voxel::CropPotatoesStage0),
        Voxel::ItemBeetrootSeeds | Voxel::ItemBeetroot => Some(Voxel::CropBeetrootsStage0),
        Voxel::ItemOnion => Some(Voxel::CropOnionsStage0),
        Voxel::ItemGarlicBulb => Some(Voxel::CropGarlicStage0),
        Voxel::ItemCabbageSeeds | Voxel::ItemCabbage => Some(Voxel::CropCabbagesStage0),
        Voxel::ItemLettuce => Some(Voxel::CropLettuceStage0),
        Voxel::ItemPepperSeeds | Voxel::ItemPepper => Some(Voxel::CropPepperStage0),
        Voxel::ItemTomatoSeeds | Voxel::ItemTomato => Some(Voxel::CropTomatoesBuddingStage0),
        Voxel::ItemCoffeeSeeds | Voxel::ItemCoffeeBerries => Some(Voxel::CropCoffeeBuddingStage0),
        Voxel::ItemMelonSeeds => Some(Voxel::CropMelonStemStage0),
        Voxel::ItemPumpkinSeeds => Some(Voxel::CropPumpkinStemStage0),
        _ => None,
    }
}

/// Attempts to advance a growing crop at `pos` to its next developmental stage.
/// Returns true if the crop advanced, or false if it is already mature/blocked.
pub fn advance_crop(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    pos: IVec3,
) -> bool {
    let Some(current_voxel) = world.get_voxel(pos) else {
        return false;
    };

    // If target is the upper block of a double-tall plant, redirect to the bottom base
    if current_voxel.is_double_tall_top() {
        let bottom_pos = pos - IVec3::Y;
        return advance_crop(world, modifications, bottom_pos);
    }

    match current_voxel {
        // Single-block cultivars (stages 0 -> 1 -> 2 -> 3)
        Voxel::CropCarrotsStage0 => set_block(world, modifications, pos, Voxel::CropCarrotsStage1),
        Voxel::CropCarrotsStage1 => set_block(world, modifications, pos, Voxel::CropCarrotsStage2),
        Voxel::CropCarrotsStage2 => set_block(world, modifications, pos, Voxel::CropCarrotsStage3),

        Voxel::CropPotatoesStage0 => set_block(world, modifications, pos, Voxel::CropPotatoesStage1),
        Voxel::CropPotatoesStage1 => set_block(world, modifications, pos, Voxel::CropPotatoesStage2),
        Voxel::CropPotatoesStage2 => set_block(world, modifications, pos, Voxel::CropPotatoesStage3),

        Voxel::CropBeetrootsStage0 => set_block(world, modifications, pos, Voxel::CropBeetrootsStage1),
        Voxel::CropBeetrootsStage1 => set_block(world, modifications, pos, Voxel::CropBeetrootsStage2),
        Voxel::CropBeetrootsStage2 => set_block(world, modifications, pos, Voxel::CropBeetrootsStage3),

        Voxel::CropOnionsStage0 => set_block(world, modifications, pos, Voxel::CropOnionsStage1),
        Voxel::CropOnionsStage1 => set_block(world, modifications, pos, Voxel::CropOnionsStage2),
        Voxel::CropOnionsStage2 => set_block(world, modifications, pos, Voxel::CropOnionsStage3),

        Voxel::CropGarlicStage0 => set_block(world, modifications, pos, Voxel::CropGarlicStage1),
        Voxel::CropGarlicStage1 => set_block(world, modifications, pos, Voxel::CropGarlicStage2),
        Voxel::CropGarlicStage2 => set_block(world, modifications, pos, Voxel::CropGarlicStage3),

        Voxel::CropCabbagesStage0 => set_block(world, modifications, pos, Voxel::CropCabbagesStage1),
        Voxel::CropCabbagesStage1 => set_block(world, modifications, pos, Voxel::CropCabbagesStage2),
        Voxel::CropCabbagesStage2 => set_block(world, modifications, pos, Voxel::CropCabbagesStage3),

        Voxel::CropLettuceStage0 => set_block(world, modifications, pos, Voxel::CropLettuceStage1),
        Voxel::CropLettuceStage1 => set_block(world, modifications, pos, Voxel::CropLettuceStage2),
        Voxel::CropLettuceStage2 => set_block(world, modifications, pos, Voxel::CropLettuceStage3),

        Voxel::CropPepperStage0 => set_block(world, modifications, pos, Voxel::CropPepperStage1),
        Voxel::CropPepperStage1 => set_block(world, modifications, pos, Voxel::CropPepperStage2),
        Voxel::CropPepperStage2 => set_block(world, modifications, pos, Voxel::CropPepperStage3),

        Voxel::CropWheatStage0 => set_block(world, modifications, pos, Voxel::CropWheatStage1),
        Voxel::CropWheatStage1 => set_block(world, modifications, pos, Voxel::CropWheatStage2),
        Voxel::CropWheatStage2 => set_block(world, modifications, pos, Voxel::CropWheatStage3),

        // Barley: stage 0 -> 1 (single-block) -> 2 -> 3 (double-tall)
        Voxel::CropBarleyStage0 => set_block(world, modifications, pos, Voxel::CropBarleyStage1),
        Voxel::CropBarleyStage1 => {
            let top_pos = pos + IVec3::Y;
            if is_empty_for_plant(world, top_pos) {
                set_double_tall(
                    world,
                    modifications,
                    pos,
                    Voxel::CropBarleyBottomStage2,
                    top_pos,
                    Voxel::CropBarleyTopStage2,
                )
            } else {
                false
            }
        }
        Voxel::CropBarleyBottomStage2 => {
            let top_pos = pos + IVec3::Y;
            set_double_tall(
                world,
                modifications,
                pos,
                Voxel::CropBarleyBottomStage3,
                top_pos,
                Voxel::CropBarleyTopStage3,
            )
        }

        // Corn: budding 0..3 -> tall bottom/top 0..3
        Voxel::CropCornBuddingStage0 => set_block(world, modifications, pos, Voxel::CropCornBuddingStage1),
        Voxel::CropCornBuddingStage1 => set_block(world, modifications, pos, Voxel::CropCornBuddingStage2),
        Voxel::CropCornBuddingStage2 => set_block(world, modifications, pos, Voxel::CropCornBuddingStage3),
        Voxel::CropCornBuddingStage3 => {
            let top_pos = pos + IVec3::Y;
            if is_empty_for_plant(world, top_pos) {
                set_double_tall(
                    world,
                    modifications,
                    pos,
                    Voxel::CropCornBottomStage0,
                    top_pos,
                    Voxel::CropCornTopStage0,
                )
            } else {
                false
            }
        }
        Voxel::CropCornBottomStage0 => {
            set_double_tall(
                world,
                modifications,
                pos,
                Voxel::CropCornBottomStage1,
                pos + IVec3::Y,
                Voxel::CropCornTopStage1,
            )
        }
        Voxel::CropCornBottomStage1 => {
            set_double_tall(
                world,
                modifications,
                pos,
                Voxel::CropCornBottomStage2,
                pos + IVec3::Y,
                Voxel::CropCornTopStage2,
            )
        }
        Voxel::CropCornBottomStage2 => {
            set_double_tall(
                world,
                modifications,
                pos,
                Voxel::CropCornBottomStage3,
                pos + IVec3::Y,
                Voxel::CropCornTopStage3,
            )
        }

        // Coffee: budding 0..3 -> shrub bottom/top 0..3
        Voxel::CropCoffeeBuddingStage0 => set_block(world, modifications, pos, Voxel::CropCoffeeBuddingStage1),
        Voxel::CropCoffeeBuddingStage1 => set_block(world, modifications, pos, Voxel::CropCoffeeBuddingStage2),
        Voxel::CropCoffeeBuddingStage2 => set_block(world, modifications, pos, Voxel::CropCoffeeBuddingStage3),
        Voxel::CropCoffeeBuddingStage3 => {
            let top_pos = pos + IVec3::Y;
            if is_empty_for_plant(world, top_pos) {
                set_double_tall(
                    world,
                    modifications,
                    pos,
                    Voxel::CropCoffeeBottomStage0,
                    top_pos,
                    Voxel::CropCoffeeTopStage0,
                )
            } else {
                false
            }
        }
        Voxel::CropCoffeeBottomStage0 => {
            set_double_tall(
                world,
                modifications,
                pos,
                Voxel::CropCoffeeBottomStage1,
                pos + IVec3::Y,
                Voxel::CropCoffeeTopStage1,
            )
        }
        Voxel::CropCoffeeBottomStage1 => {
            set_double_tall(
                world,
                modifications,
                pos,
                Voxel::CropCoffeeBottomStage2,
                pos + IVec3::Y,
                Voxel::CropCoffeeTopStage2,
            )
        }
        Voxel::CropCoffeeBottomStage2 => {
            set_double_tall(
                world,
                modifications,
                pos,
                Voxel::CropCoffeeBottomStage3,
                pos + IVec3::Y,
                Voxel::CropCoffeeTopStage3,
            )
        }

        // Tomatoes: trellis budding 0..3 -> climbing vine top 0..3
        Voxel::CropTomatoesBuddingStage0 => set_block(world, modifications, pos, Voxel::CropTomatoesBuddingStage1),
        Voxel::CropTomatoesBuddingStage1 => set_block(world, modifications, pos, Voxel::CropTomatoesBuddingStage2),
        Voxel::CropTomatoesBuddingStage2 => set_block(world, modifications, pos, Voxel::CropTomatoesBuddingStage3),
        Voxel::CropTomatoesBuddingStage3 => {
            let top_pos = pos + IVec3::Y;
            let top_vox = world.get_voxel(top_pos);
            match top_vox {
                None | Some(Voxel::Air) => {
                    set_double_tall(
                        world,
                        modifications,
                        pos,
                        Voxel::CropTomatoesBuddingStage3,
                        top_pos,
                        Voxel::CropTomatoesVineStage0,
                    )
                }
                Some(Voxel::CropTomatoesVineStage0) => {
                    set_block(world, modifications, top_pos, Voxel::CropTomatoesVineStage1)
                }
                Some(Voxel::CropTomatoesVineStage1) => {
                    set_block(world, modifications, top_pos, Voxel::CropTomatoesVineStage2)
                }
                Some(Voxel::CropTomatoesVineStage2) => {
                    set_block(world, modifications, top_pos, Voxel::CropTomatoesVineStage3)
                }
                _ => false,
            }
        }

        // Rice: lower shoot 0..3 -> supporting foliage + panicles top 0..3
        Voxel::CropRiceStage0 => set_block(world, modifications, pos, Voxel::CropRiceStage1),
        Voxel::CropRiceStage1 => set_block(world, modifications, pos, Voxel::CropRiceStage2),
        Voxel::CropRiceStage2 => set_block(world, modifications, pos, Voxel::CropRiceStage3),
        Voxel::CropRiceStage3 => {
            let top_pos = pos + IVec3::Y;
            if is_empty_for_plant(world, top_pos) {
                set_double_tall(
                    world,
                    modifications,
                    pos,
                    Voxel::CropRiceSupporting,
                    top_pos,
                    Voxel::CropRicePaniclesStage0,
                )
            } else {
                false
            }
        }
        Voxel::CropRiceSupporting => {
            let top_pos = pos + IVec3::Y;
            match world.get_voxel(top_pos) {
                Some(Voxel::CropRicePaniclesStage0) => {
                    set_block(world, modifications, top_pos, Voxel::CropRicePaniclesStage1)
                }
                Some(Voxel::CropRicePaniclesStage1) => {
                    set_block(world, modifications, top_pos, Voxel::CropRicePaniclesStage2)
                }
                Some(Voxel::CropRicePaniclesStage2) => {
                    set_block(world, modifications, top_pos, Voxel::CropRicePaniclesStage3)
                }
                _ => false,
            }
        }

        // Melon & Pumpkin Stems (stages 0 -> 1 -> 2 -> spawn fruit)
        Voxel::CropMelonStemStage0 => set_block(world, modifications, pos, Voxel::CropMelonStemStage1),
        Voxel::CropMelonStemStage1 => set_block(world, modifications, pos, Voxel::CropMelonStemStage2),
        Voxel::CropMelonStemStage2 => try_spawn_cucurbit_fruit(
            world,
            modifications,
            pos,
            Voxel::CropMelonStemAttached,
            Voxel::MelonBlock,
        ),

        Voxel::CropPumpkinStemStage0 => set_block(world, modifications, pos, Voxel::CropPumpkinStemStage1),
        Voxel::CropPumpkinStemStage1 => set_block(world, modifications, pos, Voxel::CropPumpkinStemStage2),
        Voxel::CropPumpkinStemStage2 => try_spawn_cucurbit_fruit(
            world,
            modifications,
            pos,
            Voxel::CropPumpkinStemAttached,
            Voxel::PumpkinBlock,
        ),

        // Arboreal Cocoa Pods (preserve trunk attachment orientation!)
        Voxel::CropCocoaStage0 => {
            let orient = world.get_shape(pos).1;
            set_shaped_block(world, modifications, pos, Voxel::CropCocoaStage1, BlockShape::CocoaPod, orient)
        }
        Voxel::CropCocoaStage1 => {
            let orient = world.get_shape(pos).1;
            set_shaped_block(world, modifications, pos, Voxel::CropCocoaStage2, BlockShape::CocoaPod, orient)
        }

        // Arboreal Hanging Coconuts
        Voxel::CropCoconutStage0 => {
            set_shaped_block(world, modifications, pos, Voxel::CropCoconutStage1, BlockShape::HangingFruit, 0)
        }
        Voxel::CropCoconutStage1 => {
            set_shaped_block(world, modifications, pos, Voxel::CropCoconutStage2, BlockShape::HangingFruit, 0)
        }

        _ => false,
    }
}

/// Evaluates survival drops when a crop or wild plant is broken.
pub fn evaluate_crop_break_drops(voxel: Voxel, rng: &mut u64) -> Vec<(Voxel, u32)> {
    let mut next_rand = || {
        if *rng == 0 {
            *rng = 0x85EB_CA6B_C2B2_AE35;
        }
        *rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*rng >> 33) as u32
    };

    match voxel {
        // Mature root / tuber crops (Stage 3)
        Voxel::CropCarrotsStage3 => {
            let bonus = (next_rand() % 3) + 1; // 1..3
            vec![(Voxel::ItemCarrot, 1 + bonus)]
        }
        Voxel::CropPotatoesStage3 => {
            let bonus = (next_rand() % 3) + 1;
            vec![(Voxel::ItemPotato, 1 + bonus)]
        }
        Voxel::CropBeetrootsStage3 => {
            let seeds = (next_rand() % 3) + 1;
            vec![(Voxel::ItemBeetroot, 1), (Voxel::ItemBeetrootSeeds, seeds)]
        }
        Voxel::CropOnionsStage3 => {
            let bonus = (next_rand() % 2) + 1;
            vec![(Voxel::ItemOnion, 1 + bonus)]
        }
        Voxel::CropGarlicStage3 => {
            let bonus = (next_rand() % 3) + 1;
            vec![(Voxel::ItemGarlicBulb, 1 + bonus)]
        }
        Voxel::CropCabbagesStage3 => {
            let seeds = (next_rand() % 3) + 1;
            vec![(Voxel::ItemCabbage, 1), (Voxel::ItemCabbageSeeds, seeds)]
        }
        Voxel::CropLettuceStage3 => {
            let bonus = (next_rand() % 2) + 1;
            vec![(Voxel::ItemLettuce, 1 + bonus)]
        }
        Voxel::CropPepperStage3 => {
            let seeds = (next_rand() % 3) + 1;
            vec![(Voxel::ItemPepper, 1), (Voxel::ItemPepperSeeds, seeds)]
        }
        Voxel::CropWheatStage3 => {
            let seeds = (next_rand() % 3) + 1;
            vec![(Voxel::ItemWheat, 1), (Voxel::ItemWheatSeeds, seeds)]
        }

        // Mature double-tall crops
        Voxel::CropBarleyBottomStage3 | Voxel::CropBarleyTopStage3 => {
            let seeds = (next_rand() % 3) + 1;
            vec![(Voxel::ItemBarley, 1), (Voxel::ItemBarleySeeds, seeds)]
        }
        Voxel::CropCornBottomStage3 | Voxel::CropCornTopStage3 => {
            let ears = (next_rand() % 2) + 1;
            vec![(Voxel::ItemCorn, ears + 1)]
        }
        Voxel::CropCoffeeBottomStage3 | Voxel::CropCoffeeTopStage3 => {
            let berries = (next_rand() % 3) + 1;
            let seeds = (next_rand() % 2) + 1;
            vec![(Voxel::ItemCoffeeBerries, berries), (Voxel::ItemCoffeeSeeds, seeds)]
        }
        Voxel::CropTomatoesVineStage3 | Voxel::CropTomatoesBuddingStage3 => {
            let tomatoes = (next_rand() % 3) + 1;
            let seeds = (next_rand() % 2) + 1;
            vec![(Voxel::ItemTomato, tomatoes), (Voxel::ItemTomatoSeeds, seeds)]
        }
        Voxel::CropRicePaniclesStage3 => {
            let rice = (next_rand() % 3) + 2;
            vec![(Voxel::ItemRice, rice)]
        }

        // Immature crops (drop seed recovery)
        Voxel::CropCarrotsStage0 | Voxel::CropCarrotsStage1 | Voxel::CropCarrotsStage2 => {
            vec![(Voxel::ItemCarrot, 1)]
        }
        Voxel::CropPotatoesStage0 | Voxel::CropPotatoesStage1 | Voxel::CropPotatoesStage2 => {
            vec![(Voxel::ItemPotato, 1)]
        }
        Voxel::CropBeetrootsStage0 | Voxel::CropBeetrootsStage1 | Voxel::CropBeetrootsStage2 => {
            vec![(Voxel::ItemBeetrootSeeds, 1)]
        }
        Voxel::CropOnionsStage0 | Voxel::CropOnionsStage1 | Voxel::CropOnionsStage2 => {
            vec![(Voxel::ItemOnion, 1)]
        }
        Voxel::CropGarlicStage0 | Voxel::CropGarlicStage1 | Voxel::CropGarlicStage2 => {
            vec![(Voxel::ItemGarlicBulb, 1)]
        }
        Voxel::CropCabbagesStage0 | Voxel::CropCabbagesStage1 | Voxel::CropCabbagesStage2 => {
            vec![(Voxel::ItemCabbageSeeds, 1)]
        }
        Voxel::CropLettuceStage0 | Voxel::CropLettuceStage1 | Voxel::CropLettuceStage2 => {
            vec![(Voxel::ItemLettuce, 1)]
        }
        Voxel::CropPepperStage0 | Voxel::CropPepperStage1 | Voxel::CropPepperStage2 => {
            vec![(Voxel::ItemPepperSeeds, 1)]
        }
        Voxel::CropWheatStage0 | Voxel::CropWheatStage1 | Voxel::CropWheatStage2 => {
            vec![(Voxel::ItemWheatSeeds, 1)]
        }
        Voxel::CropBarleyStage0
        | Voxel::CropBarleyStage1
        | Voxel::CropBarleyBottomStage2
        | Voxel::CropBarleyTopStage2 => vec![(Voxel::ItemBarleySeeds, 1)],
        Voxel::CropCornBuddingStage0
        | Voxel::CropCornBuddingStage1
        | Voxel::CropCornBuddingStage2
        | Voxel::CropCornBuddingStage3
        | Voxel::CropCornBottomStage0
        | Voxel::CropCornBottomStage1
        | Voxel::CropCornBottomStage2
        | Voxel::CropCornTopStage0
        | Voxel::CropCornTopStage1
        | Voxel::CropCornTopStage2 => vec![(Voxel::ItemCorn, 1)],
        Voxel::CropCoffeeBuddingStage0
        | Voxel::CropCoffeeBuddingStage1
        | Voxel::CropCoffeeBuddingStage2
        | Voxel::CropCoffeeBuddingStage3
        | Voxel::CropCoffeeBottomStage0
        | Voxel::CropCoffeeBottomStage1
        | Voxel::CropCoffeeBottomStage2
        | Voxel::CropCoffeeTopStage0
        | Voxel::CropCoffeeTopStage1
        | Voxel::CropCoffeeTopStage2 => vec![(Voxel::ItemCoffeeSeeds, 1)],
        Voxel::CropTomatoesBuddingStage0
        | Voxel::CropTomatoesBuddingStage1
        | Voxel::CropTomatoesBuddingStage2
        | Voxel::CropTomatoesVineStage0
        | Voxel::CropTomatoesVineStage1
        | Voxel::CropTomatoesVineStage2 => vec![(Voxel::ItemTomatoSeeds, 1)],
        Voxel::CropRiceStage0
        | Voxel::CropRiceStage1
        | Voxel::CropRiceStage2
        | Voxel::CropRiceSupporting
        | Voxel::CropRicePaniclesStage0
        | Voxel::CropRicePaniclesStage1
        | Voxel::CropRicePaniclesStage2 => vec![(Voxel::ItemRice, 1)],

        // Cucurbit Stems
        Voxel::CropMelonStemStage0
        | Voxel::CropMelonStemStage1
        | Voxel::CropMelonStemStage2
        | Voxel::CropMelonStemAttached => vec![(Voxel::ItemMelonSeeds, 1)],
        Voxel::CropPumpkinStemStage0
        | Voxel::CropPumpkinStemStage1
        | Voxel::CropPumpkinStemStage2
        | Voxel::CropPumpkinStemAttached => vec![(Voxel::ItemPumpkinSeeds, 1)],

        // Cucurbit Fruit Blocks
        Voxel::MelonBlock | Voxel::WildMelon => {
            let slices = (next_rand() % 5) + 3; // 3..7 slices
            vec![(Voxel::ItemMelonSlice, slices)]
        }
        Voxel::PumpkinBlock | Voxel::WildPumpkin => {
            let slices = (next_rand() % 3) + 2; // 2..4 slices
            let seeds = (next_rand() % 3) + 1;  // 1..3 seeds
            vec![(Voxel::ItemPumpkinSlice, slices), (Voxel::ItemPumpkinSeeds, seeds)]
        }

        // Ancestral Wild Varieties
        Voxel::WildCarrots => vec![(Voxel::ItemCarrot, 2)],
        Voxel::WildPotatoes => vec![(Voxel::ItemPotato, 2)],
        Voxel::WildBeetroots => vec![(Voxel::ItemBeetroot, 1), (Voxel::ItemBeetrootSeeds, 1)],
        Voxel::WildOnions => vec![(Voxel::ItemOnion, 2)],
        Voxel::WildGarlic => vec![(Voxel::ItemGarlicBulb, 2)],
        Voxel::WildCabbages => vec![(Voxel::ItemCabbage, 1), (Voxel::ItemCabbageSeeds, 1)],
        Voxel::WildLettuce => vec![(Voxel::ItemLettuce, 2)],
        Voxel::WildPepper => vec![(Voxel::ItemPepper, 1), (Voxel::ItemPepperSeeds, 1)],
        Voxel::WildTomatoes => vec![(Voxel::ItemTomato, 2), (Voxel::ItemTomatoSeeds, 1)],
        Voxel::WildCoffee => vec![(Voxel::ItemCoffeeBerries, 2), (Voxel::ItemCoffeeSeeds, 1)],
        Voxel::WildWheatBottom | Voxel::WildWheatTop => {
            vec![(Voxel::ItemWheat, 1), (Voxel::ItemWheatSeeds, 1)]
        }
        Voxel::WildBarleyBottom | Voxel::WildBarleyTop => {
            vec![(Voxel::ItemBarley, 1), (Voxel::ItemBarleySeeds, 1)]
        }
        Voxel::WildRiceBottom | Voxel::WildRiceTop => vec![(Voxel::ItemRice, 2)],
        Voxel::WildCornBottom | Voxel::WildCornTop => vec![(Voxel::ItemCorn, 2)],
        Voxel::WildSugarCaneBottom | Voxel::WildSugarCaneTop => vec![(Voxel::ItemSugarCane, 2)],

        // Forest Floor Ground Loot
        Voxel::StickPile => {
            let count = (next_rand() % 3) + 1; // 1..3 sticks
            vec![(Voxel::ItemStick, count)]
        }

        // Arboreal Crops (Cocoa & Coconut)
        Voxel::CropCocoaStage2 => {
            let count = (next_rand() % 2) + 2; // 2..3 cocoa beans
            vec![(Voxel::ItemCocoaBeans, count)]
        }
        Voxel::CropCocoaStage0 | Voxel::CropCocoaStage1 => vec![(Voxel::ItemCocoaBeans, 1)],
        Voxel::CropCoconutStage2 => {
            let count = (next_rand() % 2) + 1; // 1..2 coconuts
            vec![(Voxel::ItemCoconut, count)]
        }
        Voxel::CropCoconutStage0 | Voxel::CropCoconutStage1 => vec![(Voxel::ItemCoconut, 1)],

        // Default: block drops itself
        _ => vec![(voxel, 1)],
    }
}

/// Checks adjacent neighbors of a removed block. If an adjacent neighbor was an attached cucurbit stem,
/// it detaches and reverts to mature Stage 2.
pub fn handle_cucurbit_fruit_harvest(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    pos: IVec3,
) {
    const OFFSETS: [IVec3; 4] = [IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z];
    for offset in OFFSETS {
        let n_pos = pos + offset;
        if let Some(n_vox) = world.get_voxel(n_pos) {
            if n_vox == Voxel::CropMelonStemAttached {
                set_block(world, modifications, n_pos, Voxel::CropMelonStemStage2);
            } else if n_vox == Voxel::CropPumpkinStemAttached {
                set_block(world, modifications, n_pos, Voxel::CropPumpkinStemStage2);
            }
        }
    }
}

fn set_block(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    pos: IVec3,
    voxel: Voxel,
) -> bool {
    set_block_oriented(world, modifications, pos, voxel, 0)
}

fn set_block_oriented(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    pos: IVec3,
    voxel: Voxel,
    orientation: u8,
) -> bool {
    set_shaped_block(world, modifications, pos, voxel, BlockShape::CrossBillboard, orientation)
}

fn set_shaped_block(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    pos: IVec3,
    voxel: Voxel,
    shape: BlockShape,
    orientation: u8,
) -> bool {
    world.set_voxel(pos, voxel);
    world.set_fluid_level(pos, 0);
    world.set_shape(pos, shape, orientation);
    world.set_extra_slab(pos, None);
    modifications.record(pos, voxel);
    modifications.record_fluid_level(pos, 0);
    modifications.record_shape(pos, shape, orientation);
    modifications.record_extra_slab(pos, None);
    true
}

fn set_double_tall(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    bottom_pos: IVec3,
    bottom_voxel: Voxel,
    top_pos: IVec3,
    top_voxel: Voxel,
) -> bool {
    set_block_oriented(world, modifications, bottom_pos, bottom_voxel, 0);
    set_block_oriented(world, modifications, top_pos, top_voxel, 0);
    true
}

fn is_empty_for_plant(world: &VoxelWorld, pos: IVec3) -> bool {
    world
        .get_voxel(pos)
        .map_or(false, |v| v.is_empty() || v.is_fluid())
}

fn try_spawn_cucurbit_fruit(
    world: &mut VoxelWorld,
    modifications: &mut WorldModificationStore,
    stem_pos: IVec3,
    attached_stem_voxel: Voxel,
    fruit_voxel: Voxel,
) -> bool {
    // Orientation mapping for attached stems:
    // 0 = +X (East), 1 = -X (West), 2 = +Z (South), 3 = -Z (North)
    const NEIGHBORS: [(IVec3, u8); 4] = [
        (IVec3::X, 0),
        (IVec3::NEG_X, 1),
        (IVec3::Z, 2),
        (IVec3::NEG_Z, 3),
    ];
    for (offset, stem_orient) in NEIGHBORS {
        let fruit_pos = stem_pos + offset;
        let ground_pos = fruit_pos - IVec3::Y;

        let fruit_empty = is_empty_for_plant(world, fruit_pos);
        let ground_solid = world
            .get_voxel(ground_pos)
            .map_or(false, |v| v.is_collidable() && !v.is_empty() && !v.is_fluid());

        if fruit_empty && ground_solid {
            // Place solid fruit block
            world.set_voxel(fruit_pos, fruit_voxel);
            world.set_fluid_level(fruit_pos, 0);
            world.set_shape(fruit_pos, BlockShape::Full, 0);
            world.set_extra_slab(fruit_pos, None);
            modifications.record(fruit_pos, fruit_voxel);
            modifications.record_fluid_level(fruit_pos, 0);
            modifications.record_shape(fruit_pos, BlockShape::Full, 0);
            modifications.record_extra_slab(fruit_pos, None);

            // Connect stem oriented directly toward the spawned fruit
            set_block_oriented(world, modifications, stem_pos, attached_stem_voxel, stem_orient);
            return true;
        }
    }
    false
}

#[derive(Resource)]
pub struct CropGrowthTimer(pub Timer);

impl Default for CropGrowthTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.2, TimerMode::Repeating))
    }
}

pub fn update_crop_growth(
    mut commands: Commands,
    time: Res<Time>,
    menu_state: Option<Res<State<MenuState>>>,
    mut timer: Local<CropGrowthTimer>,
    mut rng: Local<u64>,
    world: Option<ResMut<VoxelWorld>>,
    modifications: Option<ResMut<WorldModificationStore>>,
    queues: Option<ResMut<ChunkStreamingQueues>>,
    light_registry: Option<ResMut<VoxelLightRegistry>>,
    player_query: Query<&Transform, With<Player>>,
) {
    if menu_state.is_some_and(|s| *s.get() != MenuState::None) {
        return;
    }
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let Some(player_tf) = player_query.iter().next() else {
        return;
    };
    let Some(mut world) = world else {
        return;
    };
    let Some(mut modifications) = modifications else {
        return;
    };
    let Some(mut queues) = queues else {
        return;
    };
    let Some(mut light_registry) = light_registry else {
        return;
    };

    if *rng == 0 {
        *rng = 0xA738_495B_12DE_903C;
    }

    let player_base = player_tf.translation.floor().as_ivec3();
    let mut edited_voxels = Vec::new();

    // Sample several candidate positions near the player for natural agricultural ticks
    for _ in 0..12 {
        *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let rx = ((*rng >> 32) as i32 % 24) - 12;
        *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let rz = ((*rng >> 32) as i32 % 24) - 12;
        *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let ry = ((*rng >> 32) as i32 % 8) - 4;

        let sample_pos = player_base + IVec3::new(rx, ry, rz);
        if let Some(vox) = world.get_voxel(sample_pos) {
            if vox.is_crop_stage() {
                // Verify supporting block below is tilled soil
                let floor_vox = world.get_voxel(sample_pos - IVec3::Y);
                if floor_vox.is_some_and(is_tilled_soil) {
                    *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    // 35% chance to advance stage when selected
                    if (*rng % 100) < 35 {
                        if advance_crop(&mut world, &mut modifications, sample_pos) {
                            edited_voxels.push(sample_pos);
                            edited_voxels.push(sample_pos + IVec3::Y);
                        }
                    }
                }
            }
        }
    }

    if edited_voxels.is_empty() {
        return;
    }

    let mut dirty_chunks = Vec::new();
    for &pos in &edited_voxels {
        sync_voxel_light(&mut commands, &world, pos, &mut light_registry);
        dirty_chunks.extend(affected_chunks(pos));
    }
    dirty_chunks.sort_by_key(|c| (c.x, c.y, c.z));
    dirty_chunks.dedup();
    for coord in dirty_chunks {
        if world.get_chunk(coord).is_some() {
            queues.enqueue_priority_remesh(coord);
        }
    }
}

pub struct CropPlugin;

impl Plugin for CropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_crop_growth);
    }
}
