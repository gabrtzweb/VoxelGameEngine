# World Blocks Definition v0.5 (238 in total)

## Rocky Blocks (96 in total)
* **Andesite** -> Rock_Andesite, Cobbled_Andesite, Mossy_Andesite, Mossy_Cobbled_Andesite
* **Azurite** -> Rock_Azurite, Cobbled_Azurite, Mossy_Azurite, Mossy_Cobbled_Azurite
* **Basalt** -> Rock_Basalt, Cobbled_Basalt, Mossy_Basalt, Mossy_Cobbled_Basalt
* **Black Sandstone** -> Rock_Black_Sandstone, Cobbled_Black_Sandstone, Mossy_Black_Sandstone, Mossy_Cobbled_Black_Sandstone
* **Calcite** -> Rock_Calcite, Cobbled_Calcite, Mossy_Calcite, Mossy_Cobbled_Calcite
* **Chalk** -> Rock_Chalk, Cobbled_Chalk, Mossy_Chalk, Mossy_Cobbled_Chalk
* **Chert** -> Rock_Chert, Cobbled_Chert, Mossy_Chert, Mossy_Cobbled_Chert
* **Cinnabar** -> Rock_Cinnabar, Cobbled_Cinnabar, Mossy_Cinnabar, Mossy_Cobbled_Cinnabar
* **Diorite** -> Rock_Diorite, Cobbled_Diorite, Mossy_Diorite, Mossy_Cobbled_Diorite
* **Gabbro** -> Rock_Gabbro, Cobbled_Gabbro, Mossy_Gabbro, Mossy_Cobbled_Gabbro
* **Granite** -> Rock_Granite, Cobbled_Granite, Mossy_Granite, Mossy_Cobbled_Granite
* **Karst** -> Rock_Karst, Cobbled_Karst, Mossy_Karst, Mossy_Cobbled_Karst
* **Limestone** -> Rock_Limestone, Cobbled_Limestone, Mossy_Limestone, Mossy_Cobbled_Limestone
* **Marl** -> Rock_Marl, Cobbled_Marl, Mossy_Marl, Mossy_Cobbled_Marl
* **Pitchstone** -> Rock_Pitchstone, Cobbled_Pitchstone, Mossy_Pitchstone, Mossy_Cobbled_Pitchstone
* **Porphyry** -> Rock_Porphyry, Cobbled_Porphyry, Mossy_Porphyry, Mossy_Cobbled_Porphyry
* **Red Sandstone** -> Rock_Red_Sandstone, Cobbled_Red_Sandstone, Mossy_Red_Sandstone, Mossy_Cobbled_Red_Sandstone
* **Sandstone** -> Rock_Sandstone, Cobbled_Sandstone, Mossy_Sandstone, Mossy_Cobbled_Sandstone
* **Scoria** -> Rock_Scoria, Cobbled_Scoria, Mossy_Scoria, Mossy_Cobbled_Scoria
* **Serpentine** -> Rock_Serpentine, Cobbled_Serpentine, Mossy_Serpentine, Mossy_Cobbled_Serpentine
* **Slate** -> Rock_Slate, Cobbled_Slate, Mossy_Slate, Mossy_Cobbled_Slate
* **Stone** -> Rock_Stone, Cobbled_Stone, Mossy_Stone, Mossy_Cobbled_Stone
* **Tuffite** -> Rock_Tuffite, Cobbled_Tuffite, Mossy_Tuffite, Mossy_Cobbled_Tuffite
* **White Sandstone** -> Rock_White_Sandstone, Cobbled_White_Sandstone, Mossy_White_Sandstone, Mossy_Cobbled_White_Sandstone

Explaining them:
Let's take Andesite for example. It will have the first block: "Andesite", which uses the texture `/blocks/rock_andesite.png` on all 6 faces. Then, there is "Mossy Andesite", which uses the same base texture `/blocks/rock_andesite.png` combined with the `/blocks/mossy_rock_overlay.png` texture on top (on all 6 faces). The third block, "Cobbled Andesite", uses the texture `/blocks/cobbled_andesite.png` on all 6 faces. Following this pattern, the fourth block, "Mossy Cobbled Andesite", uses the `/blocks/cobbled_andesite.png` base combined with the `/blocks/mossy_cobbled_overlay.png` texture on top (on all 6 faces). 

This definition applies to all rocky blocks above, with the exception of Basalt. Basalt has different textures depending on the face, using `/blocks/rock_basalt_side.png` on its 4 sides and `/blocks/rock_basalt.png` on the top and bottom faces. Therefore, "Mossy Basalt" applies the `/blocks/mossy_rock_overlay.png` only on top of the side textures, not on the top and bottom faces. For "Cobbled Basalt", all 6 faces share the same `/blocks/cobbled_basalt.png` texture, so the `/blocks/mossy_cobbled_overlay.png` can be applied to all 6 faces.

### Special Rocky Blocks (8 in total)
* **Alabaster** -> Rock_Alabaster
* **Brimstone** -> Rock_Brimstone
* **Cryolite** -> Rock_Cryolite
* **Dreadstone** -> Rock_Dreadstone
* **Flint** -> Rock_Flint
* **Magma** -> Rock_Magma
* **Obsidian** -> Rock_Obsidian
* **Terracotta** -> Rock_Terracotta

Explaining them:
Those last 8 rock types don't follow the same pattern as the previous ones because they don't have variants (Cobbled, Mossy, Mossy Cobbled).

---

## Soil Blocks (32 in total)
* **Ash** -> Soil_Ash
* **Black Sand** -> Soil_Black_Sand
* **Clay** -> Soil_Clay
* **Dirt** -> Soil_Dirt
* **Grass** -> Soil_Grass
* **Gravel** -> Soil_Gravel
* **Moss** -> Soil_Moss
* **Mud** -> Soil_Mud
* **Mulch** -> Soil_Mulch
* **Packed Dirt** -> Soil_Packed_Dirt
* **Packed Mud** -> Soil_Packed_Mud
* **Packed Peat** -> Soil_Packed_Peat
* **Packed Silt** -> Soil_Packed_Silt
* **Peat** -> Soil_Peat
* **Peat Grass** -> Soil_Peat_Grass
* **Peat Mulch** -> Soil_Peat_Mulch
* **Red Moss** -> Soil_Red_Moss
* **Red Sand** -> Soil_Red_Sand
* **Rooted Dirt** -> Soil_Rooted_Dirt
* **Sand** -> Soil_Sand
* **Scorched Black Sand** -> Soil_Scorched_Black_Sand
* **Scorched Red Sand** -> Soil_Scorched_Red_Sand
* **Scorched Sand** -> Soil_Scorched_Sand
* **Scorched White Sand** -> Soil_Scorched_White_Sand
* **Silt** -> Soil_Silt
* **Silt Grass** -> Soil_Silt_Grass
* **Silt Mulch** -> Soil_Silt_Mulch
* **Snow** -> Soil_Snow
* **Snowy Grass** -> Soil_Snowy_Grass
* **Snowy Peat** -> Soil_Snowy_Peat
* **Snowy Silt** -> Soil_Snowy_Silt
* **White Sand** -> Soil_White_Sand

Explaining them:
A few of them will have different side, top, bottom, and even overlay textures. For example, the "Grass" block will have the "`/blocks/soil_grass_side.png`" texture on its 4 side faces combined with the "`/blocks/soil_grass_overlay.png`" texture on top of them. It will use "`/blocks/soil_grass.png`" on its top face and "`/blocks/soil_dirt.png`" on its bottom face. 

Keep in mind that the `/blocks/soil_grass_overlay.png` and `/blocks/soil_grass.png` textures are grayscale and will receive a tint (this mechanic is already implemented in the engine). 

Blocks following this same pattern include Silt Grass and Peat Grass. They will use the "`/blocks/soil_grass.png`" texture on top. Their sides will use their respective side versions ("`/blocks/soil_peat_grass_side.png`" and "`/blocks/soil_silt_grass_side.png`") combined with the `/blocks/soil_grass_overlay.png` for the tint. Their bottom faces will use their respective dirt types, meaning `soil_peat` and `soil_silt`.

The Mulch blocks will have a similar structure. The "Mulch" block will use "`/blocks/soil_mulch.png`" on top, "`/blocks/soil_mulch_side.png`" on the sides, and "`/blocks/soil_dirt.png`" on the bottom. Similarly, "Peat Mulch" will use "`/blocks/soil_peat_mulch.png`" on top, "`/blocks/soil_peat_mulch_side.png`" on the sides, and "`/blocks/soil_peat.png`" on the bottom. 

The process for the snowy blocks works the exact same way. They use the snow texture on top and their specific snowy side textures on the side faces. For the bottom faces, "Snowy Grass" uses dirt, "Snowy Peat" uses peat, and "Snowy Silt" uses silt. 

All the remaining soil blocks share the same texture across all 6 faces.

---

## Liquid Blocks (8 in total)
- **Acid** -> Liquid_Acid
- **Blood** -> Liquid_Blood
- **Lava** -> Liquid_Lava
- **Molten** -> Liquid_Molten
- **Ooze** -> Liquid_Ooze
- **Sludge** -> Liquid_Sludge
- **Tar** -> Liquid_Tar
- **Water** -> Liquid_Water

Explaining them:
All liquid blocks use their own unique animated textures. For example, water has "`/blocks/liquid_water.png`" and "`/blocks/liquid_water_still.png`". I removed the "`_flow.png`" variants because the engine will use the "`_still.png`" textures for both still and flowing liquids for now. All other liquids follow this exact same logic. The base texture without the "still" suffix is intended exclusively for the static "item" icon in the inventory.

---

## Frost Blocks (4 in total)
- **Black Ice** -> Frost_Black_Ice
- **Fragile Ice** -> Frost_Fragile_Ice
- **Ice** -> Frost_Ice
- **Packed Ice** -> Frost_Packed_Ice

Explaining them:
All frost blocks use their own unique textures on all 6 faces. 

---

## Testing/Debug Blocks (8 in total)
- **Null Block** -> Null_Block
- **Null Liquid** -> Null_Liquid
- **Test Accept** -> Test_Accept
- **Test Debug** -> Test_Debug
- **Test Numbers** -> Test_Numbers
- **Test Fail** -> Test_Fail
- **Test Instance** -> Test_Instance
- **Test Start** -> Test_Start

Explaining them:
All testing and debug blocks use their own unique textures on all 6 faces. The "Null" textures are not actual blocks, but rather placeholders to be used on blocks without a texture or with a broken/missing texture.

---

## Light Source Blocks (8 in total)
- **Blue Light Block** -> Emit_Blue_Light
- **Blue Torch** -> Emit_Blue_Torch
- **Cold Light Block** -> Emit_Cold_Light
- **Green Light Block** -> Emit_Green_Light
- **Green Torch** -> Emit_Green_Torch
- **Red Light Block** -> Emit_Red_Light
- **Red Torch** -> Emit_Red_Torch
- **Warm Light Block** -> Emit_Warm_Light

Explaining them:
All light source blocks use their own unique textures on all 6 faces. The "Light Block" variants are solid cubic blocks that emit colored lighting. The "Torch" variants currently use a standard solid cubic mesh with their respective torch textures applied to all faces. This serves as a structural placeholder in the engine until a custom, thinner 3D mesh is implemented for them (TODO list).

---

## Decoration Blocks (16 in total)
- **Barrel** -> Deco_Barrel
- **Basket** -> Deco_Basket
- **Bone Block** -> Deco_Bone
- **Bookshelf** -> Deco_Bookshelf
- **Brick** -> Deco_Brick
- **Fabric** -> Deco_Fabric
- **Flesh** -> Deco_Flesh
- **Glass** -> Deco_Glass
- **Hay** -> Deco_Hay
- **Plaster** -> Deco_Plaster
- **Slime** -> Deco_Slime
- **Stone Path** -> Deco_Stone_Path
- **Thatch** -> Deco_Thatch
- **Wax** -> Deco_Wax
- **Wicker** -> Deco_Wicker
- **Wool** -> Deco_Wool

Explaining them:
Decoration blocks are solid cubic structures used to detail the world. While many of these blocks apply a single texture uniformly across all 6 faces (like Brick or Wax), several items require specific face mapping to make sense visually. For example, blocks like the Barrel and the Bookshelf will use different textures for their top, bottom, and side faces to properly represent the objects. 

---

## Wood Blocks (50 in total)
- **Acacia** -> Tree_Acacia_Bark, Tree_Acacia_Log
- **Acacia Leaves** -> Tree_Acacia_Leaves
- **Acacia Planks** -> Tree_Acacia_Planks
- **Birch** -> Tree_Birch_Bark, Tree_Birch_Log
- **Birch Leaves** -> Tree_Birch_Leaves
- **Birch Planks** -> Tree_Birch_Planks
- **Cactus** -> Tree_Cactus
- **Charred Wood** -> Tree_Charred_Bark, Tree_Charred_Log
- **Charred Wood Planks** -> Tree_Charred_Planks
- **Dead Wood** -> Tree_Dead_Bark, Tree_Dead_Log
- **Dead Wood Planks** -> Tree_Dead_Planks
- **Mahogany** -> Tree_Mahogany_Bark, Tree_Mahogany_Log
- **Mahogany Leaves** -> Tree_Mahogany_Leaves
- **Mahogany Planks** -> Tree_Mahogany_Planks
- **Mangrove** -> Tree_Mangrove_Bark, Tree_Mangrove_Log
- **Mangrove Leaves** -> Tree_Mangrove_Leaves
- **Mangrove Planks** -> Tree_Mangrove_Planks
- **Mangrove Roots** -> Tree_Mangrove_Roots
- **Maple** -> Tree_Maple_Bark, Tree_Maple_Log
- **Maple Leaves** -> Tree_Maple_Leaves_Orange, Tree_Maple_Leaves_Red, Tree_Maple_Leaves_Yellow
- **Maple Planks** -> Tree_Maple_Planks
- **Oak** -> Tree_Oak_Bark, Tree_Oak_Log
- **Oak Leaves** -> Tree_Oak_Leaves
- **Oak Planks** -> Tree_Oak_Planks
- **Palm** -> Tree_Palm_Bark, Tree_Palm_Log
- **Palm Leaves** -> Tree_Palm_Leaves
- **Palm Planks** -> Tree_Palm_Planks
- **Pine** -> Tree_Pine_Bark, Tree_Pine_Log
- **Pine Leaves** -> Tree_Pine_Leaves
- **Pine Planks** -> Tree_Pine_Planks
- **Willow** -> Tree_Willow_Bark, Tree_Willow_Log
- **Willow Leaves** -> Tree_Willow_Leaves
- **Willow Planks** -> Tree_Willow_Planks
- **Yew** -> Tree_Yew_Bark, Tree_Yew_Log
- **Yew Leaves** -> Tree_Yew_Leaves
- **Yew Planks** -> Tree_Yew_Planks

Explaining them:
Each tree type, such as Oak, features two primary wood blocks. The first is the "Log" block (e.g., `Tree_Oak_Log`), which uses a bark texture like "`/blocks/tree_oak_bark.png`" on its four side faces and a cross-section log texture like "`/blocks/tree_oak_log.png`" on its top and bottom faces. The second is the solid wood block (e.g., `Tree_Oak_Bark`), which applies the bark texture "`/blocks/tree_oak_bark.png`" uniformly across all six faces. 

This dual-block logic applies to all tree varieties. The plank and leaf blocks are simpler, as they apply their respective unique textures uniformly across all six faces. 

Special blocks have their own mapping rules: "Mangrove Roots" uses "`/blocks/tree_mangrove_roots.png`" on its four side faces and "`/blocks/tree_mangrove_roots_top.png`" on its top and bottom faces. The "Cactus" block uses "`/blocks/tree_cactus_side.png`" on its four sides, "`/blocks/tree_cactus_top.png`" on the top face, and "`/blocks/tree_cactus_bot.png`" on the bottom face.

---
## Aquatic Blocks (8 in total)
- **Algae Mat** -> Aqua_Algae_Mat
- **Brain Coral** -> Aqua_Brain_Coral
- **Bubble Coral** -> Aqua_Bubble_Coral
- **Fire Coral** -> Aqua_Fire_Coral
- **Geothermal Vent** -> Aqua_Geothermal_Vent
- **Horn Coral** -> Aqua_Horn_Coral
- **Sponge** -> Aqua_Sponge
- **Tube Coral** -> Aqua_Tube_Coral

Explaining them:
All aquatic blocks are solid cubic blocks that function structurally like rock or soil blocks, occupying the full voxel to prevent water rendering issues. Most of them use a single unique texture across all 6 faces, with the exception of the "Algae Mat" and the "Geothermal Vent". The Algae Mat uses "`/blocks/aqua_algae_mat_top.png`" for its top and bottom faces, and "`/blocks/aqua_algae_mat.png`" for its side faces. The Geothermal Vent follows a similar rule, utilizing different textures for its top/bottom ( `/blocks/aqua_geothermal_vent.png`) and "`/blocks/aqua_geothermal_vent_side.png`" for the side faces.

---
## Future Blocks (Planned for later)
