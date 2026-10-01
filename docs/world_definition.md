# World Blocks Definition v0.5 (244 in total)

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

## Wood Blocks (56 in total)
- **Acacia** -> Tree_Acacia_Bark, Tree_Acacia_Log
- **Acacia Leaves** -> Tree_Acacia_Leaves
- **Acacia Planks** -> Tree_Acacia_Planks
- **Birch** -> Tree_Birch_Bark, Tree_Birch_Log
- **Birch Leaves** -> Tree_Birch_Leaves
- **Birch Planks** -> Tree_Birch_Planks
- **Cactus** -> Tree_Cactus
- **Charred Wood** -> Tree_Charred_Bark, Tree_Charred_Log
- **Charred Wood Planks** -> Tree_Charred_Planks
- **Cherry** -> Tree_Cherry_Bark, Tree_Cherry_Log
- **Cherry Leaves** -> Tree_Cherry_Leaves
- **Cherry Planks** -> Tree_Cherry_Planks
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
- **Oak Leaves** -> Tree_Oak_Leaves, Tree_Oak_Leaves_Lush, Tree_Oak_Leaves_Flowering
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
## Future Vegetation Blocks (Planned for later) (48 slots in total)

*(NOT to be implement yet. They are non-solid vegetation blocks that require custom rendering pipelines and alpha transparency. Keep them out of the current Voxel enum.)*

### Terrestrial Flora (X-Mesh) (16 slots)
- **Bush** -> Vege_Bush 
- **Cactus Bush** -> Vege_Cactus_Bush 
- **Dead Bush** -> Vege_Dead_Bush 
- **Dry Grass** -> Vege_Drygrass 
- **Dune Grass** -> Vege_Dune_Grass [X]
- **Fern** -> Vege_Fern 
- **Frost Grass** -> Vege_Frost_Grass [X]
- **Red Shrub** -> Vege_Red_Shrub 
- **Wildgrass** -> Vege_Wildgrass
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*

### Tall Terrestrial Flora (2-Blocks High X-Mesh) (8 slots)
- **Tall Fern** -> Vege_Tall_Fern
- **Tall Wildgrass** -> Vege_Tall_Wildgrass
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*

### Surface & Wall Flora (Flat/Directional Mesh) (8 slots)
- **Lily Pad** -> Vege_Lily_Pad [X]
- **Vines** -> Vege_Vines
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*

### Aquatic Flora (X-Mesh) (8 slots)
- **Brain Coral Fan** -> Vege_Brain_Coral_Fan
- **Bubble Coral Fan** -> Vege_Bubble_Coral_Fan
- **Fire Coral Fan** -> Vege_Fire_Coral_Fan 
- **Horn Coral Fan** -> Vege_Horn_Coral_Fan 
- **Kelp** -> Vege_Kelp
- **Seagrass** -> Vege_Seagrass
- **Tube Coral Fan** -> Vege_Tube_Coral_Fan
- *[Empty Slot]*

### Fungi (X-Mesh) (8 slots)
- **Brown Mushroom** -> Fungi_Brown_Mushroom [X]
- **Red Mushroom** -> Fungi_Red_Mushroom [X]
- **Crimson Fungus** -> Fungi_Crimson [X]
- **Warped Fungus** -> Fungi_Warped [X]
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*
- *[Empty Slot]*

Explaining them:
Unlike standard cubic blocks, these vegetation blocks will bypass traditional voxel geometry. 

- **X-Mesh Blocks**: Most vegetation (grass, ferns, bushes, corals, and fungi) will use an "X-mesh" (or cross-model) rendering technique where two flat planes intersect diagonally inside the voxel. 
- **Custom Mesh Blocks**: "Lily Pad" will render as a flat horizontal quad slightly above the water level. "Vines" will render as directional flat quads placed flush against the side faces of adjacent solid blocks.
- **Tall Blocks**: Tall ferns and wildgrass occupy two vertical voxels. They will require specific rendering logic to pair a bottom texture (`_bottom`) with its corresponding top texture (`_top`).

Most of these plants feature multiple texture variants (e.g., `vege_wildgrass.png`, `vege_wildgrass1.png`, `vege_wildgrass2.png`) to provide natural, randomized visual variety in the world generation. 

Tinting: Blocks like `Vege_Wildgrass`, `Vege_Bush`, and `Vege_Fern` will utilize grayscale textures to receive dynamic biome color tinting (similar to the standard `Soil_Grass`). Other flora, such as dry grass, dead bushes, red shrubs, corals, and fungi, will have their final colors fully baked into their texture files.

---
# World Biomes Definition v0.5 (Phase 12) (48 in total)

## Biome Generation Anatomy

Explaining how the 244 blocks from the registry populate these 48 biomes.

### Forests & Woodlands (8 in total)
- **Ancient Weald:** A dense, classic magical forest. Surface: `Soil_Mulch` and `Soil_Grass`. Trees: `Tree_Oak` with `Tree_Oak_Leaves`, `Tree_Oak_Leaves_Lush`, and `Tree_Oak_Leaves_Flowering`.
- **Autumnal Forest:** A vibrant temperate forest. Surface: `Soil_Silt_Grass`. Trees: `Tree_Maple` featuring a mix of `Tree_Maple_Leaves_Red`, `Tree_Maple_Leaves_Orange`, and `Tree_Maple_Leaves_Yellow`.
- **Birch Copse:** A bright, airy woodland. Surface: `Soil_Grass`. Primary Rock: `Rock_Chalk`. Trees: `Tree_Birch`.
- **Blossom Grove:** A serene, pink-canopied grove. Surface: `Soil_Grass`. Primary Rock: `Rock_Calcite`. Trees: `Tree_Cherry`.
- **Boreal Taiga:** A cold, pine-dominated forest. Surface: `Soil_Snowy_Peat` and `Soil_Snowy_Grass`. Trees: `Tree_Pine`.
- **Deadwood Thicket:** A cursed, lifeless woods. Surface: `Soil_Ash` and `Soil_Black_Sand`. Trees: `Tree_Dead` and `Tree_Charred`. Primary Rock: `Rock_Pitchstone`.
- **Tropical Rainforest:** A hot, dense jungle. Surface: `Soil_Mud` and `Soil_Moss`. Trees: `Tree_Mahogany`. Primary Rock: `Rock_Serpentine`.
- **Yew Grove:** A dark, ancient, and solemn forest. Surface: `Soil_Peat_Mulch`. Trees: `Tree_Yew`. Primary Rock: `Rock_Gabbro`.

### Plains & Open Lands (8 in total)
- **Acacia Savanna:** Warm, dry grassland. Surface: `Soil_Grass` (warm tinted) and `Soil_Packed_Dirt`. Trees: Sparse `Tree_Acacia`.
- **Heath:** Cool, rugged plains. Surface: `Soil_Silt_Grass`. Subsoil: `Soil_Silt`. Primary Rock: `Rock_Tuffite`.
- **Moorland:** Wet, foggy highlands. Surface: `Soil_Peat_Grass`. Subsoil: `Soil_Peat`. Primary Rock: `Rock_Gabbro`.
- **Outback Scrubland:** Harsh, reddish dry plains. Surface: `Soil_Scorched_Red_Sand` and `Soil_Packed_Mud`.
- **Permafrost Steppe:** Deeply frozen flatlands. Surface: `Soil_Snowy_Silt`. Subsoil: `Soil_Packed_Silt`. Primary Rock: `Rock_Slate`.
- **Snowy Tundra:** Classic frozen plains. Surface: `Soil_Snowy_Grass`. Primary Rock: `Rock_Limestone`. 
- **Steppe:** Endless temperate grasslands. Surface: `Soil_Grass`. Primary Rock: `Rock_Sandstone`.
- **Volcanic Plains:** Flatlands covered in volcanic sediment. Surface: `Soil_Ash` and `Soil_Scorched_Black_Sand`. Primary Rock: `Rock_Basalt`.

### Wetlands & Swamps (8 in total)
- **Cypress Swamp:** Deep, murky waters. Surface: `Soil_Mud`. Trees: `Tree_Pine` (acting as Cypress) growing directly out of `Liquid_Water`.
- **Fungal Bog:** A highly toxic or magical swamp. Surface: `Soil_Red_Moss`. Liquids: Pools of `Liquid_Ooze` or `Liquid_Acid`. 
- **Mangrove Swamp:** Coastal or riverine wetlands. Surface: `Soil_Mud`. Trees: `Tree_Mangrove` supported by `Tree_Mangrove_Roots` over water.
- **Marshland:** Grassy, shallow wetlands. Surface: `Soil_Mud` and `Soil_Packed_Mud`. Primary Rock: `Rock_Serpentine`.
- **Peat Bog:** Spongy, wet terrain. Surface: `Soil_Peat_Grass`. Subsoil: `Soil_Peat`. Primary Rock: `Rock_Marl`.
- **Sludge Wastes:** Highly polluted or corrupted wetlands. Surface: `Soil_Scorched_Sand`. Liquids: Ponds of `Liquid_Sludge`.
- **Tar Pits:** Dangerous prehistoric swamps. Surface: `Soil_Black_Sand`. Liquids: Deep pits of `Liquid_Tar`. Primary Rock: `Rock_Pitchstone`.
- **Weeping Bayou:** Calm, atmospheric flooded forests. Surface: `Soil_Silt_Mulch`. Trees: `Tree_Willow` lining the water channels.

### Arid & Warm Lands (8 in total)
- **Badlands:** Multi-colored, highly eroded terrain. Surface: `Soil_Red_Sand`. Primary Rock: `Rock_Red_Sandstone` featuring distinct horizontal bands of `Rock_Terracotta`.
- **Dune Desert:** Endless rolling dunes. Surface: `Soil_Sand`. Primary Rock: `Rock_Sandstone`.
- **Oasis:** Small, localized lush spots in deserts. Surface: `Soil_Grass` enclosing a pool of `Liquid_Water`. Trees: `Tree_Palm`.
- **Painted Desert:** Deserts with exotic geology. Surface: `Soil_White_Sand` and `Soil_Sand`. Primary Rock: `Rock_Ochrestone` or `Rock_Cinnabar`.
- **Rocky Scrubland:** Dry, harsh terrain. Surface: `Soil_Packed_Dirt` and `Soil_Sand`. Trees: `Tree_Cactus`. Primary Rock: `Rock_Granite`.
- **Scorched Wastes:** Extremely hot, barren plains. Surface: `Soil_Scorched_Sand`. Liquids: Rare pools of `Liquid_Lava`. Primary Rock: `Rock_Scoria`.
- **White Desert:** Blindingly bright deserts. Surface: `Soil_White_Sand`. Primary Rock: `Rock_White_Sandstone`.
- **Windswept Canyons:** Deep arid ravines. Surface: `Soil_Sand`. Primary Rock: `Rock_Chert` and `Rock_Porphyry` forming the canyon walls.

### Mountain Biomes (8 in total)
- **Alpine Tundra:** High altitude rocky plains. Surface: `Soil_Snowy_Grass` interspersed with exposed `Rock_Diorite`.
- **Frozen Caldera:** The collapsed summit of an extinct volcano. Surface: `Frost_Black_Ice`. Walls: `Rock_Obsidian` and `Rock_Basalt`.
- **Glacial Peaks:** The highest, coldest mountains. Surface: `Soil_Snow`. Subsoil: `Frost_Packed_Ice`.
- **Jagged Crags:** Dark, imposing, and hostile mountains. Surface: Exposed `Rock_Gabbro` and `Rock_Andesite`.
- **Karst Peaks:** Extreme, vertical spire-like mountains. Surface: Pure `Rock_Karst`.
- **Scree Slopes:** Dangerous, unstable mountain sides. Surface: Entirely composed of sliding `Soil_Gravel` resting on `Rock_Andesite`.
- **Shale Barrens:** Grey, lifeless ridges. Surface: `Rock_Slate` and `Cobbled_Slate`.
- **Volcanic Fields:** Active geothermal mountains. Surface: `Rock_Basalt` and `Rock_Scoria`. Liquids: Streams of `Liquid_Lava` and `Liquid_Molten`. Primary Rock: `Rock_Magma`.

### Coastal & Aquatic Biomes (8 in total)
- **Abyssal Trench:** The deepest, darkest parts of the ocean. Floor: `Rock_Obsidian` and `Rock_Pitchstone`. Features: `Aqua_Geothermal_Vent`.
- **Beach:** Classic coastal shorelines. Surface: `Soil_White_Sand` or `Soil_Sand`. Trees: Sparse `Tree_Palm`.
- **Brackish Estuary:** River mouths meeting the sea. Floor: `Soil_Silt` and `Soil_Mud`.
- **Chalk Cliffs:** Vertical coastal drops. Walls: Pure `Rock_Chalk`.
- **Coastal Crags:** Violent, rocky shores. Surface: `Rock_Porphyry`.
- **Deep Ocean:** The vast open ocean. Floor: `Soil_Gravel` and `Rock_Andesite`.
- **Temperate Ocean:** Shallow, lively seas. Floor: `Soil_White_Sand`. Features: Thriving coral reefs using `Aqua_Brain_Coral`, `Aqua_Bubble_Coral`, `Aqua_Fire_Coral`, `Aqua_Horn_Coral`, and `Aqua_Tube_Coral`.
- **Tidal Mudflats:** Flat coastal wetlands exposed at low tide. Surface: `Soil_Packed_Mud` and `Soil_Clay`.
