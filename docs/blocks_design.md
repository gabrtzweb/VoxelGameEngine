# Voxel Game Engine Master Block Catalog (Blocks Design)

This master document catalogs **all blocks and items currently implemented in the Voxel Engine**, synchronized directly with `src/world/block/voxel.rs` and `src/world/block/data.rs`.

> [!IMPORTANT]
> **Project Grand Total: 693 Registry Entries (3 Internal System Markers + 690 Active Gameplay Blocks & Items in `Voxel::ALL`)**  
> Backed by **1557** block textures in `assets/textures/blocks/` and **62** item icons in `assets/textures/items/`.

- **Stochastic variations** (suffixes `1`, `2`, `3`, e.g., `vege_leaf_litter.png`, `vege_leaf_litter1.png`, `vege_leaf_litter2.png`) are sampled per-block by `src/meshing/textures.rs` and are **not** registered as duplicate voxel IDs.
- **Connected texture overlays** live in `assets/textures/blocks/overlays/` and are composited in the voxel shader without adding extra voxel IDs.

---

## Master Summary & Block Census

| Section | Domain | Entries | Navigation |
| :---: | :--- | :---: | :--- |
| **01** | Technical & System Markers (`systems`) | **3** | [Go to Section 01](#01-technical--system-markers-systems--3-entries) |
| **02** | Hydrosphere & Cryology (`fluids`) | **18** | [Go to Section 02](#02-hydrosphere--cryology-fluids--18-entries) |
| **03** | Pedology & Soils (`soils`) | **50** | [Go to Section 03](#03-pedology--soils-soils--50-entries) |
| **04** | Geology & Rocks (`rocks`) | **83** | [Go to Section 04](#04-geology--rocks-rocks--83-entries) |
| **05** | Trees, Forestry & Dendrology (`trees`) | **69** | [Go to Section 05](#05-trees-forestry--dendrology-trees--69-entries) |
| **06** | Ores, Minerals & Gemology (`ores`) | **13** | [Go to Section 06](#06-ores-minerals--gemology-ores--13-entries) |
| **07** | Speleology & Crystals (`caverns`) | **33** | [Go to Section 07](#07-speleology--crystals-caverns--33-entries) |
| **08** | Ocean & Benthic Ecosystems (`oceans`) | **45** | [Go to Section 08](#08-ocean--benthic-ecosystems-oceans--45-entries) |
| **09** | Biomaterials, Fauna & Organic Matter (`organics`) | **26** | [Go to Section 09](#09-biomaterials-fauna--organic-matter-organics--26-entries) |
| **10** | Decorations, Lighting & Architecture (`decorations`) | **121** | [Go to Section 10](#10-decorations-lighting--architecture-decorations--121-entries) |
| **11** | Mycology: Mushrooms (`mushrooms`) | **12** | [Go to Section 11](#11-mycology-mushrooms-mushrooms--12-entries) |
| **12** | Botany: Groundcover & Herbaceous Flora (`vegetation`) | **33** | [Go to Section 12](#12-botany-groundcover--herbaceous-flora-vegetation--33-entries) |
| **13** | Botany: Flowers & Flowering Flora (`flowers`) | **34** | [Go to Section 13](#13-botany-flowers--flowering-flora-flowers--34-entries) |
| **14** | Agriculture: Cultivated & Wild Crops (`crops`) | **121** | [Go to Section 14](#14-agriculture-cultivated--wild-crops-crops--121-entries) |
| **15** | Harvested Items, Seeds & Materials (`items`) | **32** | [Go to Section 15](#15-harvested-items-seeds--materials-items--32-entries) |
| **TOTAL** | **All `VOXEL_DATA` Entries (`0..=692`)** | **693** | — |

---

## 01. Technical & System Markers (`systems`) — 3 Entries

Internal non-placeable engine state markers (`air`, `occupied`, `water_occupied`) excluded from `Voxel::ALL`.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `air` | Air | **System** | *(Internal engine marker)* | Internal spatial state (non-rendered) |
| **02** | `occupied` | Occupied | **System** | *(Internal engine marker)* | Internal spatial state (non-rendered) |
| **03** | `water_occupied` | Waterlogged Occupied | **System** | *(Internal engine marker)* | Internal spatial state (non-rendered) |

---

## 02. Hydrosphere & Cryology (`fluids`) — 18 Entries

Cryogenic ice variants and dynamic physical liquids (still/flow animated strips).

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `frost_blue_ice` | Blue Ice | **Cryology** | `frost_blue_ice.png` | 6 uniform faces *(100% Opaque deep cobalt glacial)* |
| **02** | `frost_blue_ice_cracked` | Cracked Blue Ice | **Cryology** | `frost_blue_ice_cracked.png` | 6 uniform faces *(100% Opaque cellular frost veins & bubbles)* |
| **03** | `frost_ice` | Ice | **Cryology** | `frost_ice.png` | 6 uniform faces *(Translucent, alpha=153)* |
| **04** | `frost_ice_black` | Black Ice | **Cryology** | `frost_ice_black.png` | 6 uniform faces *(Translucent abyssal dark, alpha=160)* |
| **05** | `frost_ice_black_fragile` | Fragile Black Ice | **Cryology** | `frost_ice_black_fragile.png` | 6 uniform faces *(Translucent dark with bold cleavage fractures)* |
| **06** | `frost_ice_black_packed` | Packed Black Ice | **Cryology** | `frost_ice_black_packed.png` | 6 uniform faces *(100% Opaque solid dark permafrost)* |
| **07** | `frost_ice_fragile` | Fragile Ice | **Cryology** | `frost_ice_fragile.png` | 6 uniform faces *(Translucent with subtle hairline cracks)* |
| **08** | `frost_ice_packed` | Packed Ice | **Cryology** | `frost_ice_packed.png` | 6 uniform faces *(100% Opaque solid firn ice)* |
| **09** | `liquid_acid` | Acid | **Chemical** | Block: `liquid_acid.png`<br>Still: `liquid_acid_still.png`<br>Flow: `liquid_acid_flow.png` | Top: Still animated strip *(Neon green corrosive)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **10** | `liquid_blood` | Blood | **Biological** | Block: `liquid_blood.png`<br>Still: `liquid_blood_still.png`<br>Flow: `liquid_blood_flow.png` | Top: Still animated strip *(Crimson arterial)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **11** | `liquid_core_magma` | Core Magma | **Magmatic** | Block: `liquid_core_magma.png`<br>Still: `liquid_core_magma_still.png`<br>Flow: `liquid_core_magma_flow.png` | Top: Still animated strip *(Deep mantle incandescent)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **12** | `liquid_lava` | Lava | **Magmatic** | Block: `liquid_lava.png`<br>Still: `liquid_lava_still.png`<br>Flow: `liquid_lava_flow.png` | Top: Still animated strip *(1000°C–1200°C basaltic)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **13** | `liquid_molten` | Molten | **Magmatic** | Block: `liquid_molten.png`<br>Still: `liquid_molten_still.png`<br>Flow: `liquid_molten_flow.png` | Top: Still animated strip *(Liquefied metallurgical metal)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **14** | `liquid_ooze` | Ooze | **Biological** | Block: `liquid_ooze.png`<br>Still: `liquid_ooze_still.png`<br>Flow: `liquid_ooze_flow.png` | Top: Still animated strip *(Olive green bubbling slime)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **15** | `liquid_sludge` | Sludge | **Alluvial Mire** | Block: `liquid_sludge.png`<br>Still: `liquid_sludge_still.png`<br>Flow: `liquid_sludge_flow.png` | Top: Still animated strip *(Dark brown muddy mire)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **16** | `liquid_spring_water` | Spring Water | **Aquatic** | Block: `liquid_spring_water.png`<br>Still: `liquid_spring_water_still.png`<br>Flow: `liquid_spring_water_flow.png` | Top: Still animated strip *(Turquoise mineral)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **17** | `liquid_tar` | Tar | **Bituminous** | Block: `liquid_tar.png`<br>Still: `liquid_tar_still.png`<br>Flow: `liquid_tar_flow.png` | Top: Still animated strip *(Viscous black bitumen, alpha=216)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **18** | `liquid_water` | Water | **Aquatic** | Block: `liquid_water.png`<br>Still: `liquid_water_still.png`<br>Flow: `liquid_water_flow.png` | Top: Still animated strip *(Dynamic biome tint)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |

---

## 03. Pedology & Soils (`soils`) — 50 Entries

Zonal agricultural soils, climatic earths, turf/grass top-soils, sands, clays, and snow covers.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `caliche_coarse` | Caliche Coarse | **Climatic** | `soil_caliche_coarse.png` | 6 uniform faces |
| **02** | `caliche_cracked` | Caliche Cracked | **Climatic** | `soil_caliche_cracked.png` | 6 uniform faces |
| **03** | `caliche_dirt` | Caliche Dirt | **Climatic** | `soil_caliche_dirt.png` | 6 uniform faces |
| **04** | `caliche_grass` | Caliche Grass | **Climatic** | Top: `soil_grass.png`<br>Bottom: `soil_caliche_dirt.png`<br>Sides: `soil_caliche_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `assets/textures/blocks/overlays/soil_grass_side_overlay.png`) |
| **05** | `clay_blue` | Blue Clay | **Clay** | `soil_clay_blue.png` | 6 uniform faces |
| **06** | `clay_red` | Red Clay | **Clay** | `soil_clay_red.png` | 6 uniform faces |
| **07** | `clay_white` | White Clay | **Clay** | `soil_clay_white.png` | 6 uniform faces |
| **08** | `clay_yellow` | Yellow Clay | **Clay** | `soil_clay_yellow.png` | 6 uniform faces |
| **09** | `gravel_ashy` | Ashy Gravel | **Gravel** | `soil_gravel_ashy.png` | 6 uniform faces |
| **10** | `gravel_dirty` | Dirty Gravel | **Gravel** | `soil_gravel_dirty.png` | 6 uniform faces |
| **11** | `gravel_sandy` | Sandy Gravel | **Gravel** | `soil_gravel_sandy.png` | 6 uniform faces |
| **12** | `gravel_snowy` | Snowy Gravel | **Gravel** | `soil_gravel_snowy.png` | 6 uniform faces |
| **13** | `laterite_coarse` | Laterite Coarse | **Climatic** | `soil_laterite_coarse.png` | 6 uniform faces |
| **14** | `laterite_dirt` | Laterite Dirt | **Climatic** | `soil_laterite_dirt.png` | 6 uniform faces |
| **15** | `laterite_grass` | Laterite Grass | **Climatic** | Top: `soil_grass.png`<br>Bottom: `soil_laterite_dirt.png`<br>Sides: `soil_laterite_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `assets/textures/blocks/overlays/soil_grass_side_overlay.png`) |
| **16** | `laterite_hardpan` | Laterite Hardpan | **Climatic** | `soil_laterite_hardpan.png` | 6 uniform faces |
| **17** | `loam_coarse` | Loam Coarse | **Biological** | `soil_loam_coarse.png` | 6 uniform faces |
| **18** | `loam_dirt` | Loam Dirt | **Biological** | `soil_loam_dirt.png` | 6 uniform faces |
| **19** | `loam_grass` | Loam Grass | **Biological** | Top: `soil_grass.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `assets/textures/blocks/overlays/soil_grass_side_overlay.png`) |
| **20** | `loam_mud` | Loam Mud | **Biological** | `soil_loam_mud.png` | 6 uniform faces |
| **21** | `loam_mulch` | Loam Mulch | **Biological** | Top: `soil_loam_mulch.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_mulch_side.png` | Top + Bottom + 4 sides |
| **22** | `loam_rooted` | Loam Rooted | **Biological** | `soil_loam_rooted.png` | 6 uniform faces |
| **23** | `loam_snow` | Loam Snow | **Biological** | Top: `soil_snow.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_snow_side.png` | Top + Bottom + 4 sides |
| **24** | `loam_tilled` | Loam Tilled | **Biological** | Top: `soil_loam_tilled_top.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_tilled_side.png` | Top + Bottom + 4 sides |
| **25** | `moss_forest` | Moss | **Ground Moss** | `soil_moss_forest.png` | 6 uniform faces |
| **26** | `peat_coarse` | Peat Coarse | **Biological** | `soil_peat_coarse.png` | 6 uniform faces |
| **27** | `peat_dirt` | Peat Dirt | **Biological** | `soil_peat_dirt.png` | 6 uniform faces |
| **28** | `peat_grass` | Peat Grass | **Biological** | Top: `soil_grass.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `assets/textures/blocks/overlays/soil_grass_side_overlay.png`) |
| **29** | `peat_mud` | Peat Mud | **Biological** | `soil_peat_mud.png` | 6 uniform faces |
| **30** | `peat_mulch` | Peat Mulch | **Biological** | Top: `soil_peat_mulch.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_mulch_side.png` | Top + Bottom + 4 sides |
| **31** | `peat_rooted` | Peat Rooted | **Biological** | `soil_peat_rooted.png` | 6 uniform faces |
| **32** | `peat_snow` | Peat Snow | **Biological** | Top: `soil_snow.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_snow_side.png` | Top + Bottom + 4 sides |
| **33** | `peat_tilled` | Peat Tilled | **Biological** | Top: `soil_peat_tilled_top.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_tilled_side.png` | Top + Bottom + 4 sides |
| **34** | `permafrost_coarse` | Permafrost Coarse | **Climatic** | `soil_permafrost_coarse.png` | 6 uniform faces |
| **35** | `permafrost_dirt` | Permafrost Dirt | **Climatic** | `soil_permafrost_dirt.png` | 6 uniform faces |
| **36** | `permafrost_grass` | Permafrost Grass | **Climatic** | Top: `soil_grass.png`<br>Bottom: `soil_permafrost_dirt.png`<br>Sides: `soil_permafrost_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `assets/textures/blocks/overlays/soil_grass_side_overlay.png`) |
| **37** | `permafrost_icy` | Permafrost Icy | **Climatic** | `soil_permafrost_icy.png` | 6 uniform faces |
| **38** | `sand_common` | Sand | **Sand** | `soil_sand_common.png` | 6 uniform faces |
| **39** | `sand_dune` | Dune Sand | **Sand** | `soil_sand_dune.png` | 6 uniform faces |
| **40** | `sand_red` | Red Sand | **Sand** | `soil_sand_red.png` | 6 uniform faces |
| **41** | `sand_white` | White Sand | **Sand** | `soil_sand_white.png` | 6 uniform faces |
| **42** | `silt_coarse` | Silt Coarse | **Biological** | `soil_silt_coarse.png` | 6 uniform faces |
| **43** | `silt_dirt` | Silt Dirt | **Biological** | `soil_silt_dirt.png` | 6 uniform faces |
| **44** | `silt_grass` | Silt Grass | **Biological** | Top: `soil_grass.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `assets/textures/blocks/overlays/soil_grass_side_overlay.png`) |
| **45** | `silt_mud` | Silt Mud | **Biological** | `soil_silt_mud.png` | 6 uniform faces |
| **46** | `silt_mulch` | Silt Mulch | **Biological** | Top: `soil_silt_mulch.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_mulch_side.png` | Top + Bottom + 4 sides |
| **47** | `silt_rooted` | Silt Rooted | **Biological** | `soil_silt_rooted.png` | 6 uniform faces |
| **48** | `silt_snow` | Silt Snow | **Biological** | Top: `soil_snow.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_snow_side.png` | Top + Bottom + 4 sides |
| **49** | `silt_tilled` | Silt Tilled | **Biological** | Top: `soil_silt_tilled_top.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_tilled_side.png` | Top + Bottom + 4 sides |
| **50** | `snow_block` | Snow | **Cryosphere** | `soil_snow.png` | 6 uniform faces |

---

## 04. Geology & Rocks (`rocks`) — 83 Entries

Bedrock strata, cobbled/mossy stones, sedimentary/igneous cliffs, boulders, pebbles, and surface rubble.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `andesite` | Andesite | **Continental** | `rock_andesite.png` | 6 uniform faces |
| **02** | `andesite_grass` | Andesite Grass Outcrop | **Continental** | Top: `soil_grass.png`<br>Bot: `rock_andesite.png`<br>Sides: `rock_andesite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **03** | `andesite_snow` | Andesite Snow Outcrop | **Continental** | Top: `soil_snow.png`<br>Bot: `rock_andesite.png`<br>Sides: `rock_andesite_snow_side.png` | Top + Bot + 4 sides |
| **04** | `argillite` | Argillite | **Sedimentary** | `rock_argillite.png` | 6 uniform faces |
| **05** | `argillite_grass` | Argillite Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_argillite.png`<br>Sides: `rock_argillite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **06** | `argillite_snow` | Argillite Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_argillite.png`<br>Sides: `rock_argillite_snow_side.png` | Top + Bot + 4 sides |
| **07** | `basalt` | Basalt | **Volcanic** | `rock_basalt.png` | 6 uniform faces |
| **08** | `basalt_grass` | Basalt Grass Outcrop | **Volcanic** | Top: `soil_grass.png`<br>Bot: `rock_basalt.png`<br>Sides: `rock_basalt_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **09** | `basalt_snow` | Basalt Snow Outcrop | **Volcanic** | Top: `soil_snow.png`<br>Bot: `rock_basalt.png`<br>Sides: `rock_basalt_snow_side.png` | Top + Bot + 4 sides |
| **10** | `chalk` | Chalk | **Sedimentary** | `rock_chalk.png` | 6 uniform faces |
| **11** | `chalk_grass` | Chalk Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_chalk.png`<br>Sides: `rock_chalk_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **12** | `chalk_snow` | Chalk Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_chalk.png`<br>Sides: `rock_chalk_snow_side.png` | Top + Bot + 4 sides |
| **13** | `cobbled_andesite` | Cobbled Andesite | **Continental** | `rock_andesite_cobbled.png` | 6 uniform faces |
| **14** | `cobbled_argillite` | Cobbled Argillite | **Sedimentary** | `rock_argillite_cobbled.png` | 6 uniform faces |
| **15** | `cobbled_basalt` | Cobbled Basalt | **Volcanic** | `rock_basalt_cobbled.png` | 6 uniform faces |
| **16** | `cobbled_chalk` | Cobbled Chalk | **Sedimentary** | `rock_chalk_cobbled.png` | 6 uniform faces |
| **17** | `cobbled_dolomite` | Cobbled Dolomite | **Sedimentary** | `rock_dolomite_cobbled.png` | 6 uniform faces |
| **18** | `cobbled_gabbro` | Cobbled Gabbro | **Plutonic** | `rock_gabbro_cobbled.png` | 6 uniform faces |
| **19** | `cobbled_granite` | Cobbled Granite | **Plutonic** | `rock_granite_cobbled.png` | 6 uniform faces |
| **20** | `cobbled_limestone` | Cobbled Limestone | **Sedimentary** | `rock_limestone_cobbled.png` | 6 uniform faces |
| **21** | `cobbled_magma` | Cobbled Magma | **Geothermal** | `rock_magma_cobbled.png` | 6 uniform faces *(Animated incandescent veins)* |
| **22** | `cobbled_mantle` | Cobbled Mantle | **Bedrock** | `rock_mantle_cobbled.png` | 6 uniform faces *(Fractured primordial bedrock seams, unbreakable)* |
| **23** | `cobbled_marble` | Cobbled Marble | **Metamorphic** | `rock_marble_cobbled.png` | 6 uniform faces |
| **24** | `cobbled_sandstone_common` | Cobbled Sandstone | **Sedimentary** | `rock_sandstone_common_cobbled.png` | 6 uniform faces |
| **25** | `cobbled_sandstone_dune` | Cobbled Dune Sandstone | **Sedimentary** | `rock_sandstone_dune_cobbled.png` | 6 uniform faces |
| **26** | `cobbled_sandstone_red` | Cobbled Red Sandstone | **Sedimentary** | `rock_sandstone_red_cobbled.png` | 6 uniform faces |
| **27** | `cobbled_sandstone_white` | Cobbled White Sandstone | **Sedimentary** | `rock_sandstone_white_cobbled.png` | 6 uniform faces |
| **28** | `cobbled_slate` | Cobbled Slate | **Metamorphic** | `rock_slate_cobbled.png` | 6 uniform faces |
| **29** | `dolomite` | Dolomite | **Sedimentary** | `rock_dolomite.png` | 6 uniform faces |
| **30** | `dolomite_grass` | Dolomite Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_dolomite.png`<br>Sides: `rock_dolomite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **31** | `dolomite_snow` | Dolomite Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_dolomite.png`<br>Sides: `rock_dolomite_snow_side.png` | Top + Bot + 4 sides |
| **32** | `gabbro` | Gabbro | **Plutonic** | `rock_gabbro.png` | 6 uniform faces |
| **33** | `granite` | Granite | **Plutonic** | `rock_granite.png` | 6 uniform faces |
| **34** | `granite_grass` | Granite Grass Outcrop | **Plutonic** | Top: `soil_grass.png`<br>Bot: `rock_granite.png`<br>Sides: `rock_granite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **35** | `granite_snow` | Granite Snow Outcrop | **Plutonic** | Top: `soil_snow.png`<br>Bot: `rock_granite.png`<br>Sides: `rock_granite_snow_side.png` | Top + Bot + 4 sides |
| **36** | `limestone` | Limestone | **Sedimentary** | `rock_limestone.png` | 6 uniform faces |
| **37** | `limestone_grass` | Limestone Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_limestone.png`<br>Sides: `rock_limestone_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **38** | `limestone_snow` | Limestone Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_limestone.png`<br>Sides: `rock_limestone_snow_side.png` | Top + Bot + 4 sides |
| **39** | `magma` | Magma | **Geothermal** | `rock_magma.png` *(+ `1`, `2`)* | 6 uniform faces *(Animated incandescent veins, emits light)* |
| **40** | `mantle` | Mantle | **Bedrock** | `rock_mantle.png` *(+ `1`, `2`)* | 6 uniform faces *(Unbreakable)* |
| **41** | `marble` | Marble | **Metamorphic** | `rock_marble.png` | 6 uniform faces |
| **42** | `mossy_andesite` | Mossy Andesite | **Continental** | `rock_andesite.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **43** | `mossy_argillite` | Mossy Argillite | **Sedimentary** | `rock_argillite.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **44** | `mossy_basalt` | Mossy Basalt | **Volcanic** | `rock_basalt.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **45** | `mossy_chalk` | Mossy Chalk | **Sedimentary** | `rock_chalk.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **46** | `mossy_cobbled_andesite` | Mossy Cobbled Andesite | **Continental** | `rock_andesite_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **47** | `mossy_cobbled_argillite` | Mossy Cobbled Argillite | **Sedimentary** | `rock_argillite_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **48** | `mossy_cobbled_basalt` | Mossy Cobbled Basalt | **Volcanic** | `rock_basalt_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **49** | `mossy_cobbled_chalk` | Mossy Cobbled Chalk | **Sedimentary** | `rock_chalk_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **50** | `mossy_cobbled_dolomite` | Mossy Cobbled Dolomite | **Sedimentary** | `rock_dolomite_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **51** | `mossy_cobbled_granite` | Mossy Cobbled Granite | **Plutonic** | `rock_granite_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **52** | `mossy_cobbled_limestone` | Mossy Cobbled Limestone | **Sedimentary** | `rock_limestone_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **53** | `mossy_dolomite` | Mossy Dolomite | **Sedimentary** | `rock_dolomite.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **54** | `mossy_granite` | Mossy Granite | **Plutonic** | `rock_granite.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **55** | `mossy_limestone` | Mossy Limestone | **Sedimentary** | `rock_limestone.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **56** | `pebble_chert` | Chert Pebble | **Pebble (Single)** | `pebble_chert.png` *(+ `1`, `2`)* | Low sub-voxel ground pebble |
| **57** | `pebble_chert_cluster` | Chert Pebble Cluster | **Pebble (Cluster)** | `pebble_chert_cluster.png` *(+ `1`, `2`)* | Low sub-voxel cluster pebbles |
| **58** | `pebble_flint` | Flint Pebble | **Pebble (Single)** | `pebble_flint.png` *(+ `1`, `2`)* | Low sub-voxel ground pebble |
| **59** | `pebble_flint_cluster` | Flint Pebble Cluster | **Pebble (Cluster)** | `pebble_flint_cluster.png` *(+ `1`, `2`)* | Low sub-voxel cluster pebbles |
| **60** | `pebble_quartz` | Quartz Pebble | **Pebble (Single)** | `pebble_quartz.png` *(+ `1`, `2`)* | Low sub-voxel ground pebble |
| **61** | `pebble_quartz_cluster` | Quartz Pebble Cluster | **Pebble (Cluster)** | `pebble_quartz_cluster.png` *(+ `1`, `2`)* | Low sub-voxel cluster pebbles |
| **62** | `sandstone_common` | Sandstone | **Sedimentary** | `rock_sandstone_common.png` | 6 uniform faces |
| **63** | `sandstone_dune` | Dune Sandstone | **Sedimentary** | `rock_sandstone_dune.png` | 6 uniform faces |
| **64** | `sandstone_red` | Red Sandstone | **Sedimentary** | `rock_sandstone_red.png` | 6 uniform faces |
| **65** | `sandstone_white` | White Sandstone | **Sedimentary** | `rock_sandstone_white.png` | 6 uniform faces |
| **66** | `slate` | Slate | **Metamorphic** | `rock_slate.png` | 6 uniform faces |
| **67** | `andesite_rubble` | Andesite Rubble | **Surface Rubble** | `rock_andesite.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **68** | `argillite_rubble` | Argillite Rubble | **Surface Rubble** | `rock_argillite.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **69** | `basalt_rubble` | Basalt Rubble | **Surface Rubble** | `rock_basalt.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **70** | `chalk_rubble` | Chalk Rubble | **Surface Rubble** | `rock_chalk.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **71** | `dolomite_rubble` | Dolomite Rubble | **Surface Rubble** | `rock_dolomite.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **72** | `gabbro_rubble` | Gabbro Rubble | **Surface Rubble** | `rock_gabbro.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **73** | `granite_rubble` | Granite Rubble | **Surface Rubble** | `rock_granite.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **74** | `limestone_rubble` | Limestone Rubble | **Surface Rubble** | `rock_limestone.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **75** | `marble_rubble` | Marble Rubble | **Surface Rubble** | `rock_marble.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **76** | `sandstone_rubble` | Sandstone Rubble | **Surface Rubble** | `rock_sandstone_common.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **77** | `sandstone_dune_rubble` | Dune Sandstone Rubble | **Surface Rubble** | `rock_sandstone_dune.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **78** | `sandstone_red_rubble` | Red Sandstone Rubble | **Surface Rubble** | `rock_sandstone_red.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **79** | `sandstone_white_rubble` | White Sandstone Rubble | **Surface Rubble** | `rock_sandstone_white.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **80** | `slate_rubble` | Slate Rubble | **Surface Rubble** | `rock_slate.png` *(+2 variants)* | 3D scattered stone rubble (`Rubble`) |
| **81** | `rock_chert_boulder` | Chert Boulder | **Geology** | `rock_chert_boulder.png` *(+2 variants)* | 6 faces / Natural rock |
| **82** | `rock_flint_boulder` | Flint Boulder | **Geology** | `rock_flint_boulder.png` *(+2 variants)* | 6 faces / Natural rock |
| **83** | `rock_quartz_boulder` | Quartz Boulder | **Geology** | `rock_quartz_boulder.png` *(+2 variants)* | 6 faces / Natural rock |

---

## 05. Trees, Forestry & Dendrology (`trees`) — 69 Entries

Logs, all-bark woods, planks, live/dead leaves, and charred timber across all tree species.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `acacia_leaves` | Acacia Leaves | **Foliage** | `tree_acacia_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **02** | `acacia_leaves_dead` | Dead Acacia Leaves | **Foliage** | `tree_acacia_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **03** | `acacia_log` | Acacia Log | **Wood** | Top/Bottom: `tree_acacia_log.png`<br>Sides: `tree_acacia_bark.png` | Directional (Top/Bottom + 4 sides) |
| **04** | `acacia_planks` | Acacia Planks | **Planks** | `tree_acacia_planks.png` | 6 uniform faces |
| **05** | `acacia_wood` | Acacia Wood | **Wood** | `tree_acacia_bark.png` | 6 uniform faces |
| **06** | `bamboo_leaves` | Bamboo Leaves | **Foliage** | `tree_bamboo_leaves.png` *(+ `1`, `2`)* | Stalk column with cross-billboard leaves |
| **07** | `bamboo_planks` | Bamboo Planks | **Planks** | `tree_bamboo_planks.png` | 6 uniform faces |
| **08** | `bamboo_stalk` | Bamboo Stalk | **Wood / Stalk** | Top/Bottom: `tree_bamboo_stalk_top.png`<br>Sides: `tree_bamboo_stalk.png` | Directional column (3×3 top/bottom, 3×16 sides) |
| **09** | `birch_leaves` | Birch Leaves | **Foliage** | `tree_birch_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **10** | `birch_leaves_dead` | Dead Birch Leaves | **Foliage** | `tree_birch_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **11** | `birch_log` | Birch Log | **Wood** | Top/Bottom: `tree_birch_log.png`<br>Sides: `tree_birch_bark.png` | Directional (Top/Bottom + 4 sides) |
| **12** | `birch_planks` | Birch Planks | **Planks** | `tree_birch_planks.png` | 6 uniform faces |
| **13** | `birch_wood` | Birch Wood | **Wood** | `tree_birch_bark.png` | 6 uniform faces |
| **14** | `cherry_leaves` | Cherry Blossom Leaves | **Foliage** | `tree_cherry_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **15** | `cherry_leaves_dead` | Dead Cherry Leaves | **Foliage** | `tree_cherry_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **16** | `cherry_log` | Cherry Log | **Wood** | Top/Bottom: `tree_cherry_log.png`<br>Sides: `tree_cherry_bark.png` | Directional (Top/Bottom + 4 sides) |
| **17** | `cherry_planks` | Cherry Planks | **Planks** | `tree_cherry_planks.png` | 6 uniform faces |
| **18** | `cherry_wood` | Cherry Wood | **Wood** | `tree_cherry_bark.png` | 6 uniform faces |
| **19** | `mahogany_leaves` | Mahogany Leaves | **Foliage** | `tree_mahogany_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **20** | `mahogany_leaves_dead` | Dead Mahogany Leaves | **Foliage** | `tree_mahogany_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **21** | `mahogany_log` | Mahogany Log | **Wood** | Top/Bottom: `tree_mahogany_log.png`<br>Sides: `tree_mahogany_bark.png` | Directional (Top/Bottom + 4 sides) |
| **22** | `mahogany_planks` | Mahogany Planks | **Planks** | `tree_mahogany_planks.png` | 6 uniform faces |
| **23** | `mahogany_wood` | Mahogany Wood | **Wood** | `tree_mahogany_bark.png` | 6 uniform faces |
| **24** | `mangrove_leaves` | Mangrove Leaves | **Foliage** | `tree_mangrove_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **25** | `mangrove_leaves_dead` | Dead Mangrove Leaves | **Foliage** | `tree_mangrove_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **26** | `mangrove_log` | Mangrove Log | **Wood** | Top/Bottom: `tree_mangrove_log.png`<br>Sides: `tree_mangrove_bark.png` | Directional (Top/Bottom + 4 sides) |
| **27** | `mangrove_planks` | Mangrove Planks | **Planks** | `tree_mangrove_planks.png` | 6 uniform faces |
| **28** | `mangrove_roots` | Mangrove Aerial Roots | **Root** | Top/Bottom: `tree_mangrove_roots_top.png`<br>Sides: `tree_mangrove_roots.png` | Directional (Top/Bottom + 4 sides, Cutout) |
| **29** | `mangrove_wood` | Mangrove Wood | **Wood** | `tree_mangrove_bark.png` | 6 uniform faces |
| **30** | `maple_leaves_dead` | Dead Maple Leaves | **Foliage** | `tree_maple_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **31** | `maple_leaves_orange` | Orange Maple Leaves | **Foliage** | `tree_maple_leaves_orange.png` | 6 uniform faces (Cutout alpha transparency) |
| **32** | `maple_leaves_red` | Red Maple Leaves | **Foliage** | `tree_maple_leaves_red.png` | 6 uniform faces (Cutout alpha transparency) |
| **33** | `maple_leaves_yellow` | Yellow Maple Leaves | **Foliage** | `tree_maple_leaves_yellow.png` | 6 uniform faces (Cutout alpha transparency) |
| **34** | `maple_log` | Maple Log | **Wood** | Top/Bottom: `tree_maple_log.png`<br>Sides: `tree_maple_bark.png` | Directional (Top/Bottom + 4 sides) |
| **35** | `maple_planks` | Maple Planks | **Planks** | `tree_maple_planks.png` | 6 uniform faces |
| **36** | `maple_wood` | Maple Wood | **Wood** | `tree_maple_bark.png` | 6 uniform faces |
| **37** | `oak_leaves` | Oak Leaves | **Foliage** | `tree_oak_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **38** | `oak_leaves_dead` | Dead Oak Leaves | **Foliage** | `tree_oak_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **39** | `oak_log` | Oak Log | **Wood** | Top/Bottom: `tree_oak_log.png`<br>Sides: `tree_oak_bark.png` | Directional (Top/Bottom + 4 sides) |
| **40** | `oak_planks` | Oak Planks | **Planks** | `tree_oak_planks.png` | 6 uniform faces |
| **41** | `oak_wood` | Oak Wood | **Wood** | `tree_oak_bark.png` | 6 uniform faces |
| **42** | `palm_leaves` | Palm Leaves | **Foliage** | `tree_palm_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **43** | `palm_leaves_dead` | Dead Palm Leaves | **Foliage** | `tree_palm_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **44** | `palm_log` | Palm Log | **Wood** | Top/Bottom: `tree_palm_log.png`<br>Sides: `tree_palm_bark.png` | Directional (Top/Bottom + 4 sides) |
| **45** | `palm_planks` | Palm Planks | **Planks** | `tree_palm_planks.png` | 6 uniform faces |
| **46** | `palm_wood` | Palm Wood | **Wood** | `tree_palm_bark.png` | 6 uniform faces |
| **47** | `pine_leaves` | Pine Needles | **Foliage** | `tree_pine_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **48** | `pine_leaves_dead` | Dead Pine Needles | **Foliage** | `tree_pine_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **49** | `pine_log` | Pine Log | **Wood** | Top/Bottom: `tree_pine_log.png`<br>Sides: `tree_pine_bark.png` | Directional (Top/Bottom + 4 sides) |
| **50** | `pine_planks` | Pine Planks | **Planks** | `tree_pine_planks.png` | 6 uniform faces |
| **51** | `pine_wood` | Pine Wood | **Wood** | `tree_pine_bark.png` | 6 uniform faces |
| **52** | `redwood_leaves` | Redwood Foliage | **Foliage** | `tree_redwood_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **53** | `redwood_leaves_dead` | Dead Redwood Foliage | **Foliage** | `tree_redwood_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **54** | `redwood_log` | Redwood Log | **Wood** | Top/Bottom: `tree_redwood_log.png`<br>Sides: `tree_redwood_bark.png` | Directional (Top/Bottom + 4 sides) |
| **55** | `redwood_planks` | Redwood Planks | **Planks** | `tree_redwood_planks.png` | 6 uniform faces |
| **56** | `redwood_wood` | Redwood Wood | **Wood** | `tree_redwood_bark.png` | 6 uniform faces |
| **57** | `willow_leaves` | Weeping Willow Leaves | **Foliage** | `tree_willow_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **58** | `willow_leaves_dead` | Dead Willow Leaves | **Foliage** | `tree_willow_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **59** | `willow_log` | Willow Log | **Wood** | Top/Bottom: `tree_willow_log.png`<br>Sides: `tree_willow_bark.png` | Directional (Top/Bottom + 4 sides) |
| **60** | `willow_planks` | Willow Planks | **Planks** | `tree_willow_planks.png` | 6 uniform faces |
| **61** | `willow_wood` | Willow Wood | **Wood** | `tree_willow_bark.png` | 6 uniform faces |
| **62** | `yew_leaves` | Yew Foliage | **Foliage** | `tree_yew_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **63** | `yew_leaves_dead` | Dead Yew Foliage | **Foliage** | `tree_yew_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **64** | `yew_log` | Yew Log | **Wood** | Top/Bottom: `tree_yew_log.png`<br>Sides: `tree_yew_bark.png` | Directional (Top/Bottom + 4 sides) |
| **65** | `yew_planks` | Yew Planks | **Planks** | `tree_yew_planks.png` | 6 uniform faces |
| **66** | `yew_wood` | Yew Wood | **Wood** | `tree_yew_bark.png` | 6 uniform faces |
| **67** | `tree_charred_log` | Charred Log | **Dendrology** | `tree_charred_log_lit.png` *(+2 variants)* | Log / Bark / Planks / Leaves |
| **68** | `tree_charred_bark` | Charred Wood | **Dendrology** | `tree_charred_bark_lit.png` *(+2 variants)* | Log / Bark / Planks / Leaves |
| **69** | `tree_charred_planks` | Charred Planks | **Dendrology** | `tree_charred_planks_lit.png` *(+2 variants)* | Log / Bark / Planks / Leaves |

---

## 06. Ores, Minerals & Gemology (`ores`) — 13 Entries

Metalliferous ores, fossil matrices, coal/gem deposits, and refined ore strata.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `gem_diamond_rough` | Rough Diamond Block | **Gemstone** | `gem_diamond_rough.png` | Uniform (6 faces) |
| **02** | `gem_emerald_rough` | Rough Emerald Block | **Gemstone** | `gem_emerald_rough.png` | Uniform (6 faces) |
| **03** | `gem_jade_rough` | Rough Jade Block | **Gemstone** | `gem_jade_rough.png` | Uniform (6 faces) |
| **04** | `gem_ruby_rough` | Rough Ruby Block | **Gemstone** | `gem_ruby_rough.png` | Uniform (6 faces) |
| **05** | `gem_sapphire_rough` | Rough Sapphire Block | **Gemstone** | `gem_sapphire_rough.png` | Uniform (6 faces) |
| **06** | `gem_topaz_rough` | Rough Topaz Block | **Gemstone** | `gem_topaz_rough.png` | Uniform (6 faces) |
| **07** | `ore_coal_raw` | Raw Coal Block | **Chemical** | `ore_coal_raw.png` | Uniform (6 faces) |
| **08** | `ore_copper_raw` | Raw Copper Block | **Metal** | `ore_copper_raw.png` | Uniform (6 faces) |
| **09** | `ore_gold_raw` | Raw Gold Block | **Metal** | `ore_gold_raw.png` | Uniform (6 faces) |
| **10** | `ore_iron_raw` | Raw Iron Block | **Metal** | `ore_iron_raw.png` | Uniform (6 faces) |
| **11** | `ore_silver_raw` | Raw Silver Block | **Metal** | `ore_silver_raw.png` | Uniform (6 faces) |
| **12** | `ore_sulfur_raw` | Raw Sulfur Block | **Chemical** | `ore_sulfur_raw.png` | Uniform (6 faces) |
| **13** | `ore_uranium_raw` | Raw Uranium Block | **Heavy Metal** | `ore_uranium_raw.png` | Uniform (6 faces) |

---

## 07. Speleology & Crystals (`caverns`) — 33 Entries

Cave strata (karst, calcite, obsidian), speleothems, geothermal vents, and crystalline blocks/clusters.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `cave_calcite` | Calcite | **Speleological** | `cave_calcite.png` *(+ `1`, `2`)* | 6 uniform faces |
| **02** | `cave_karst` | Karst | **Speleological** | `cave_karst.png` *(+ `1`, `2`)* | 6 uniform faces |
| **03** | `cave_obsidian` | Obsidian | **Cavern** | `cave_obsidian.png` *(+2 variants)* | Subterranean / Crystal geometry |
| **04** | `fractured_cave_calcite` | Fractured Calcite | **Cavern** | `cave_calcite_fractured.png` *(+2 variants)* | Subterranean / Crystal geometry |
| **05** | `fractured_cave_karst` | Fractured Karst | **Cavern** | `cave_karst_fractured.png` *(+2 variants)* | Subterranean / Crystal geometry |
| **06** | `crystal_amethyst_block` | Amethyst Block | **Crystal** | `crystal_solid_amethyst.png` *(+ `1`, `2`)* | 6 uniform faces |
| **07** | `crystal_amethyst_cluster` | Amethyst Cluster | **Cluster** | `crystal_amethyst_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **08** | `crystal_amethyst_glinted` | Glinted Amethyst Cluster | **Emissive** | `crystal_glinted_cluster_amethyst.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **09** | `crystal_aquamarine_block` | Aquamarine Block | **Crystal** | `crystal_solid_aquamarine.png` *(+ `1`, `2`)* | 6 uniform faces |
| **10** | `crystal_aquamarine_cluster` | Aquamarine Cluster | **Cluster** | `crystal_aquamarine_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **11** | `crystal_aquamarine_glinted` | Glinted Aquamarine Cluster | **Emissive** | `crystal_glinted_cluster_aquamarine.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **12** | `crystal_citrine_block` | Citrine Block | **Crystal** | `crystal_solid_citrine.png` *(+ `1`, `2`)* | 6 uniform faces |
| **13** | `crystal_citrine_cluster` | Citrine Cluster | **Cluster** | `crystal_citrine_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **14** | `crystal_citrine_glinted` | Glinted Citrine Cluster | **Emissive** | `crystal_glinted_cluster_citrine.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **15** | `crystal_fluorite_block` | Fluorite Block | **Crystal** | `crystal_solid_fluorite.png` *(+ `1`, `2`)* | 6 uniform faces |
| **16** | `crystal_fluorite_cluster` | Fluorite Cluster | **Cluster** | `crystal_fluorite_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **17** | `crystal_fluorite_glinted` | Glinted Fluorite Cluster | **Emissive** | `crystal_glinted_cluster_fluorite.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **18** | `crystal_pink_salt_block` | Pink Salt Block | **Crystal** | `crystal_solid_pink_salt.png` *(+ `1`, `2`)* | 6 uniform faces |
| **19** | `crystal_pink_salt_cluster` | Pink Salt Cluster | **Cluster** | `crystal_pink_salt_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **20** | `crystal_pink_salt_glinted` | Glinted Pink Salt Cluster | **Emissive** | `crystal_glinted_cluster_pink_salt.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **21** | `crystal_selenite_block` | Selenite Block | **Crystal** | `crystal_solid_selenite.png` *(+ `1`, `2`)* | 6 uniform faces |
| **22** | `crystal_selenite_cluster` | Selenite Cluster | **Cluster** | `crystal_selenite_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **23** | `crystal_selenite_glinted` | Glinted Selenite Cluster | **Emissive** | `crystal_glinted_cluster_selenite.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **24** | `fossil_amber` | Fossil Amber | **Fossil** | `fossil_amber.png` *(+ `1`, `2`)* | 6 uniform faces |
| **25** | `fossil_bitumen` | Fossil Bitumen | **Fossil** | `fossil_bitumen.png` *(+ `1`, `2`)* | 6 uniform faces |
| **26** | `fossil_bones` | Fossil Bones | **Fossil** | `fossil_bones.png` *(+ `1`, `2`)* | 6 uniform faces |
| **27** | `fossil_matrix` | Fossil Matrix | **Fossil** | `fossil_matrix.png` *(+ `1`, `2`)* | 6 uniform faces |
| **28** | `speleothem_blue_ice` | Blue Ice Icicle | **Speleothem** | `speleothem_blue_ice_*.png` | Modular system (**10** pieces) |
| **29** | `speleothem_calcite` | Calcite Flowstone | **Speleothem** | `speleothem_calcite_*.png` | Modular system (**10** pieces) |
| **30** | `speleothem_karst` | Karst Dripstone | **Speleothem** | `speleothem_karst_*.png` | Modular system (**10** pieces) |
| **31** | `speleothem_salt_straw` | Pink Salt Straw | **Speleothem** | `speleothem_pink_salt_straw_*.png` | Modular system (**6** pieces) |
| **32** | `obsidian_spike` | Obsidian Spike | **Speleothem** | `speleothem_obsidian_spike_*.png` | Modular system (**6** pieces) |
| **33** | `fractured_cave_obsidian` | Fractured Obsidian | **Cavern** | `cave_obsidian_fractured.png` *(+2 variants)* | Subterranean / Crystal geometry |

---

## 08. Ocean & Benthic Ecosystems (`oceans`) — 45 Entries

Coral blocks, coral fans, kelp/algae stalks, marine flora, and seafloor sediments.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `bone_worms` | Bone Worms | **Siboglinid Annelid** | `vege_bone_worms.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **02** | `brain_coral` | Brain Coral | **Massive Coral** | `aqua_coral_brain.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **03** | `brain_coral_fan` | Brain Coral Fan | **Coral Fan** | `aqua_brain_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **04** | `bubble_coral` | Bubble Coral | **Massive Coral** | `aqua_coral_bubble.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **05** | `bubble_coral_fan` | Bubble Coral Fan | **Coral Fan** | `aqua_bubble_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **06** | `bull_kelp` | Bull Kelp | **Laminar Macroalga** | `vege_bull_kelp.png` *(Top)*<br>`vege_bull_kelp_plant.png` *(Body)* | Columnar (Multi-block) |
| **07** | `coralstone` | Coralstone | **Biogenic Rock** | `aqua_coralstone.png` *(+ `1`, `2`)* | 6 uniform faces |
| **08** | `dead_coral` | Dead Coral | **Dead Coral** | `aqua_coral_dead.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **09** | `dead_coral_fan` | Dead Coral Fan | **Dead Coral** | `aqua_dead_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **10** | `delesseria` | Delesseria | **Rhodophyte** | `vege_delesseria.png` *(Top)*<br>`vege_delesseria_plant.png` *(Body)* | Columnar (Multi-block) |
| **11** | `dying_coral` | Dying Coral | **Senescent Coral** | `aqua_coral_dying.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **12** | `dying_coral_fan` | Dying Coral Fan | **Senescent Coral** | `aqua_dying_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **13** | `fire_coral` | Fire Coral | **Massive Coral** | `aqua_coral_fire.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **14** | `fire_coral_fan` | Fire Coral Fan | **Coral Fan** | `aqua_fire_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **15** | `geothermal_vent` | Geothermal Vent | **Hydrothermal** | `aqua_geothermal_vent_top.png`<br>`aqua_geothermal_vent_side.png`<br>`aqua_geothermal_vent_bot.png` | Directional (Top / Sides / Bottom) |
| **16** | `heart_coral` | Heart Coral | **Massive Coral** | `aqua_coral_heart.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **17** | `heart_coral_fan` | Heart Coral Fan | **Coral Fan** | `aqua_heart_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **18** | `horn_coral` | Horn Coral | **Massive Coral** | `aqua_coral_horn.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **19** | `horn_coral_fan` | Horn Coral Fan | **Coral Fan** | `aqua_horn_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **20** | `kelp` | Giant Kelp | **Laminar Macroalga** | `vege_kelp.png` *(Top)*<br>`vege_kelp_plant.png` *(Body)* | Columnar (Multi-block) |
| **21** | `leaf_coral` | Leaf Coral | **Massive Coral** | `aqua_coral_leaf.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **22** | `leaf_coral_fan` | Leaf Coral Fan | **Coral Fan** | `aqua_leaf_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **23** | `red_algae` | Red Algae | **Rhodophyte** | `vege_red_algae.png` *(+ `1`, `2`)* | Animated strip (3 variants) |
| **24** | `reed_coral` | Reed Coral | **Massive Coral** | `aqua_coral_reed.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **25** | `reed_coral_fan` | Reed Coral Fan | **Coral Fan** | `aqua_reed_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **26** | `reefstone` | Reefstone | **Volcanic Rock** | `aqua_reefstone.png` *(+ `1`, `2`)* | 6 uniform faces |
| **27** | `sargassum` | Sargassum | **Pelagic Phaeophyte** | `vege_sargassum.png` *(Top)*<br>`vege_sargassum_plant.png` *(Body)* | Columnar (Multi-block) |
| **28** | `vege_kelp_plant` | Giant Kelp Stalk | **Marine Flora** | `vege_kelp_plant.png` | Submerged aquatic cross-billboard |
| **29** | `seagrass` | Seagrass | **Marine Angiosperm** | `vege_seagrass.png` *(+ `1`, `2`)* | Animated strip (3 variants) |
| **30** | `seaweed` | Seaweed | **Littoral Macroalga** | `vege_seaweed.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **31** | `spike_coral` | Spike Coral | **Massive Coral** | `aqua_coral_spike.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **32** | `spike_coral_fan` | Spike Coral Fan | **Coral Fan** | `aqua_spike_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **33** | `vege_bull_kelp_plant` | Bull Kelp Stalk | **Marine Flora** | `vege_bull_kelp_plant.png` | Submerged aquatic cross-billboard |
| **34** | `staghorn_coral` | Staghorn Coral | **Massive Coral** | `aqua_coral_staghorn.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **35** | `staghorn_coral_fan` | Staghorn Coral Fan | **Coral Fan** | `aqua_staghorn_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **36** | `sun_coral` | Sun Coral | **Massive Coral** | `aqua_coral_sun.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **37** | `sun_coral_fan` | Sun Coral Fan | **Coral Fan** | `aqua_sun_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **38** | `thorn_coral` | Thorn Coral | **Massive Coral** | `aqua_coral_thorn.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **39** | `thorn_coral_fan` | Thorn Coral Fan | **Coral Fan** | `aqua_thorn_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **40** | `tube_coral` | Tube Coral | **Massive Coral** | `aqua_coral_tube.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **41** | `tube_coral_fan` | Tube Coral Fan | **Coral Fan** | `aqua_tube_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **42** | `wax_coral` | Wax Coral | **Massive Coral** | `aqua_coral_wax.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **43** | `wax_coral_fan` | Wax Coral Fan | **Coral Fan** | `aqua_wax_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **44** | `zigzag_coral` | Zigzag Coral | **Massive Coral** | `aqua_coral_zigzag.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **45** | `zigzag_coral_fan` | Zigzag Coral Fan | **Coral Fan** | `aqua_zigzag_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |

---

## 09. Biomaterials, Fauna & Organic Matter (`organics`) — 26 Entries

Organic matter, bones, nests, hives, cocoons, cobwebs, bamboo, stick piles, and living tissue blocks.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `ant_hill` | Ant Hill | **Colony** | `nest_ant_hill.png` | Uniform (6 faces) |
| **02** | `ant_hill_open` | Ant Hill Open | **Colony** | `nest_ant_hill_open.png` | Uniform (6 faces) |
| **03** | `bee_nest` | Wild Bee Nest | **Hive** | `nest_bee_nest_bot...top.png`<br>`nest_bee_nest_front.png` | Directional (4 faces) |
| **04** | `bird_nest` | Bird Nest | **Nest** | `nest_bird_nest_bot...top.png` | Half-height Slab |
| **05** | `bloody_bone` | Bloody Bone | **Osteology** | Top/Bot: `bio_bone_top.png`<br>Sides: `bio_bone_side.png` *(+ `bloody_overlay.png`, `1`, `2`)* | Axial Column (2 faces) + Shader Bloody Overlay |
| **06** | `bloody_brain` | Bloody Brain Block | **Neural** | `bio_brain.png` *(+ `bloody_overlay.png`, `1`, `2`)* | Uniform (6 faces) + Shader Bloody Overlay |
| **07** | `bloody_flesh` | Bloody Flesh Block | **Tissue** | Top/Bot: `bio_flesh_top.png`<br>Sides: `bio_flesh_side.png` *(+ `bloody_overlay.png`, `1`, `2`)* | Directional (2 faces) + Lateral Shader Bloody Overlay |
| **08** | `bone_block` | Bone | **Osteology** | `bio_bone_top.png`<br>`bio_bone_side.png` *(+ `1`, `2`)* | Axial Column (2 faces) |
| **09** | `brain_block` | Brain Block | **Neural** | `bio_brain.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **10** | `cobweb` | Cobweb | **Silk** | `organic_cobweb.png` *(+ `1`, `2`)* | Volumetric 3D Cross |
| **11** | `cobweb_arch` | Cobweb Arch | **Silk** | `organic_cobweb_arch.png` *(+ `1`, `2`)* | Flat Vertical Quad |
| **12** | `cobweb_hanging` | Hanging Cobweb | **Silk** | `organic_cobweb_hanging.png` *(+ `1`, `2`)* | Flat Vertical Quad |
| **13** | `cobweb_stretched` | Stretched Cobweb | **Silk** | `organic_cobweb_stretched.png` *(+ `1`, `2`)* | Flat Vertical Quad |
| **14** | `dust_block` | Dust | **Detritus** | `organic_dust.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **15** | `flesh_block` | Raw Flesh Block | **Tissue** | `bio_flesh_top.png`<br>`bio_flesh_side.png` *(+ `1`, `2`)* | Directional (2 faces) |
| **16** | `hay_nest` | Hay Nest | **Nest** | `nest_hay_nest_bot...top.png` | Half-height Slab |
| **17** | `honeycomb_block` | Honeycomb | **Wax Matrix** | `organic_honeycomb.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **18** | `jelly_block` | Royal Jelly Block | **Colloid** | `organic_jelly_top.png`<br>`organic_jelly_side.png`<br>`organic_jelly_bot.png` | Translucent (3 faces) |
| **19** | `poop_block` | Poop | **Dropping** | `organic_poop.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **20** | `silk_cocoon` | Wild Silk Cocoon | **Pupa** | `nest_silk_cocoon_bot...top.png` | Directional (3 faces) |
| **21** | `slime_block` | Slime Block | **Colloid** | `organic_slime_top.png`<br>`organic_slime_side.png`<br>`organic_slime_bot.png` | Translucent (3 faces) |
| **22** | `spider_egg` | Spider Egg Sac | **Ootheca** | `nest_spider_egg_bot...top.png` | Directional (3 faces) |
| **23** | `termite_mound` | Termite Mound | **Colony** | `nest_termite_mound.png` | Uniform (6 faces) |
| **24** | `termite_mound_open` | Termite Mound Open | **Colony** | `nest_termite_mound_open.png` | Uniform (6 faces) |
| **25** | `wasp_nest` | Wild Wasp Nest | **Hive** | `nest_wasp_nest_bot...top.png` | Directional (3 faces) |
| **26** | `stick_pile` | Pile of Sticks | **Forest Floor** | `stick_pile.png` *(+2 variants)* | 3D crossed twigs (`StickPile`, drops 1–3 `item_stick`) |

---

## 10. Decorations, Lighting & Architecture (`decorations`) — 121 Entries

Stone/sandstone bricks, painted architectural palettes (bricks, fabric, plaster, shingles, wool), containers, torches, and lamps.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `andesite_bricks` | Andesite Bricks | **Masonry** | `deco_andesite_bricks.png` *(+2 variants)* | 6 uniform faces |
| **02** | `argillite_bricks` | Argillite Bricks | **Masonry** | `deco_argillite_bricks.png` *(+2 variants)* | 6 uniform faces |
| **03** | `bag` | Fabric Bag | **Utility** | `util_bag_top.png`<br>`util_bag_side.png`<br>`util_bag_side_front.png`<br>`util_bag_bot.png` | Top top + Bottom bot + 3 sides side + 1 front `util_bag_side_front.png` |
| **04** | `barrel` | Barrel | **Utility** | `util_barrel_top.png`<br>`util_barrel_side.png`<br>`util_barrel_bot.png` | Top top + Bottom bot + 4 sides side |
| **05** | `basalt_bricks` | Basalt Bricks | **Masonry** | `deco_basalt_bricks.png` *(+2 variants)* | 6 uniform faces |
| **06** | `basket` | Basket | **Utility** | `util_basket_top.png`<br>`util_basket_side.png`<br>`util_basket_bot.png` | Top top + Bottom bot + 4 sides side |
| **07** | `bookshelf` | Bookshelf | **Utility** | `util_bookshelf_top.png`<br>`util_bookshelf_side.png`<br>`util_bookshelf_front.png` | Top & Bottom top + 3 sides side + 1 front `util_bookshelf_front.png` |
| **08** | `bookshelf_empty` | Empty Bookshelf | **Utility** | `util_bookshelf_top.png`<br>`util_bookshelf_side.png`<br>`util_bookshelf_front_empty.png` | Top & Bottom top + 3 sides side + 1 front `util_bookshelf_front_empty.png` |
| **09** | `brick_black` | Black Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **10** | `brick_blue` | Blue Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **11** | `brick_brown` | Brown Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **12** | `brick_dark_blue` | Dark Blue Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **13** | `brick_dark_grey` | Dark Grey Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **14** | `brick_green` | Green Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **15** | `brick_light_grey` | Light Grey Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **16** | `brick_light_pink` | Light Pink Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **17** | `brick_lime` | Lime Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **18** | `brick_orange` | Orange Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **19** | `brick_pink` | Pink Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **20** | `brick_purple` | Purple Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **21** | `brick_red` | Red Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **22** | `brick_turquoise` | Turquoise Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **23** | `brick_white` | White Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **24** | `brick_yellow` | Yellow Bricks | **Painted Deco** | `deco_bricks_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **25** | `bricks` | Bricks | **Masonry** | `deco_bricks.png` | 6 uniform faces |
| **26** | `chalk_bricks` | Chalk Bricks | **Masonry** | `deco_chalk_bricks.png` *(+2 variants)* | 6 uniform faces |
| **27** | `crate` | Empty Crate | **Utility** | `util_crate_top.png`<br>`util_crate_side.png`<br>`util_crate_bot.png` | Top top + Bottom bot + 4 sides side |
| **28** | `dolomite_bricks` | Dolomite Bricks | **Masonry** | `deco_dolomite_bricks.png` *(+2 variants)* | 6 uniform faces |
| **29** | `emit_lamp_blue` | Blue Lamp | **Decoration** | `emit_lamp_blue_side.png` | Architectural / Lighting fixture |
| **30** | `emit_lamp_green` | Green Lamp | **Decoration** | `emit_lamp_green_side.png` | Architectural / Lighting fixture |
| **31** | `emit_lamp_red` | Red Lamp | **Decoration** | `emit_lamp_red_side.png` | Architectural / Lighting fixture |
| **32** | `emit_lamp_warm` | Warm Lamp | **Decoration** | `emit_lamp_warm_side.png` | Architectural / Lighting fixture |
| **33** | `emit_torch_blue` | Blue Torch | **Lighting** | `emit_torch_blue.png` | Vertical stem model *(Sapphire blue diode)* |
| **34** | `emit_torch_fire` | Fire Torch | **Lighting** | `emit_torch_fire.png` | Vertical stem model *(5 animated fire frames)* |
| **35** | `emit_torch_green` | Green Torch | **Lighting** | `emit_torch_green.png` | Vertical stem model *(Emerald green diode)* |
| **36** | `emit_torch_red` | Red Torch | **Lighting** | `emit_torch_red.png` | Vertical stem model *(Ruby red diode)* |
| **37** | `emit_torch_warm` | Warm Torch | **Lighting** | `emit_torch_warm.png` | Vertical stem model *(Warm light 2700K)* |
| **38** | `fabric` | Fabric | **Textile** | `deco_fabric.png` *(+ `1`, `2`)* | 6 uniform faces |
| **39** | `fabric_black` | Black Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **40** | `fabric_blue` | Blue Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **41** | `fabric_brown` | Brown Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **42** | `fabric_dark_blue` | Dark Blue Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **43** | `fabric_dark_grey` | Dark Grey Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **44** | `fabric_green` | Green Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **45** | `fabric_light_grey` | Light Grey Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **46** | `fabric_light_pink` | Light Pink Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **47** | `fabric_lime` | Lime Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **48** | `fabric_orange` | Orange Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **49** | `fabric_pink` | Pink Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **50** | `fabric_purple` | Purple Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **51** | `fabric_red` | Red Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **52** | `fabric_turquoise` | Turquoise Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **53** | `fabric_white` | White Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **54** | `fabric_yellow` | Yellow Fabric | **Painted Deco** | `deco_fabric_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **55** | `gabbro_bricks` | Gabbro Bricks | **Masonry** | `deco_gabbro_bricks.png` *(+2 variants)* | 6 uniform faces |
| **56** | `granite_bricks` | Granite Bricks | **Masonry** | `deco_granite_bricks.png` *(+2 variants)* | 6 uniform faces |
| **57** | `limestone_bricks` | Limestone Bricks | **Masonry** | `deco_limestone_bricks.png` *(+2 variants)* | 6 uniform faces |
| **58** | `marble_bricks` | Marble Bricks | **Masonry** | `deco_marble_bricks.png` *(+2 variants)* | 6 uniform faces |
| **59** | `path_dirty` | Dirty Path | **Paths** | `deco_path_dirty.png` | 6 uniform faces |
| **60** | `path_sandy` | Sandy Path | **Paths** | `deco_path_sandy.png` | 6 uniform faces |
| **61** | `path_snowy` | Snowy Path | **Paths** | `deco_path_snowy.png` | 6 uniform faces |
| **62** | `path_stony` | Stony Path | **Paths** | `deco_path_stony.png` | 6 uniform faces |
| **63** | `plaster` | Plaster | **Masonry** | `deco_plaster.png` | 6 uniform faces |
| **64** | `plaster_black` | Black Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **65** | `plaster_blue` | Blue Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **66** | `plaster_brown` | Brown Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **67** | `plaster_dark_blue` | Dark Blue Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **68** | `plaster_dark_grey` | Dark Grey Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **69** | `plaster_green` | Green Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **70** | `plaster_light_grey` | Light Grey Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **71** | `plaster_light_pink` | Light Pink Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **72** | `plaster_lime` | Lime Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **73** | `plaster_orange` | Orange Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **74** | `plaster_pink` | Pink Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **75** | `plaster_purple` | Purple Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **76** | `plaster_red` | Red Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **77** | `plaster_turquoise` | Turquoise Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **78** | `plaster_white` | White Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **79** | `plaster_yellow` | Yellow Plaster | **Painted Deco** | `deco_plaster_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **80** | `sandstone_common_bricks` | Sandstone Bricks | **Masonry** | `deco_sandstone_common_bricks.png` *(+2 variants)* | 6 uniform faces |
| **81** | `sandstone_dune_bricks` | Dune Sandstone Bricks | **Masonry** | `deco_sandstone_dune_bricks.png` *(+2 variants)* | 6 uniform faces |
| **82** | `sandstone_red_bricks` | Red Sandstone Bricks | **Masonry** | `deco_sandstone_red_bricks.png` *(+2 variants)* | 6 uniform faces |
| **83** | `sandstone_white_bricks` | White Sandstone Bricks | **Masonry** | `deco_sandstone_white_bricks.png` *(+2 variants)* | 6 uniform faces |
| **84** | `shingles` | Shingles | **Roofing** | `deco_shingles.png` | 6 uniform faces |
| **85** | `shingles_black` | Black Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **86** | `shingles_blue` | Blue Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **87** | `shingles_brown` | Brown Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **88** | `shingles_dark_blue` | Dark Blue Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **89** | `shingles_dark_grey` | Dark Grey Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **90** | `shingles_green` | Green Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **91** | `shingles_light_grey` | Light Grey Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **92** | `shingles_light_pink` | Light Pink Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **93** | `shingles_lime` | Lime Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **94** | `shingles_orange` | Orange Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **95** | `shingles_pink` | Pink Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **96** | `shingles_purple` | Purple Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **97** | `shingles_red` | Red Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **98** | `shingles_turquoise` | Turquoise Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **99** | `shingles_white` | White Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **100** | `shingles_yellow` | Yellow Shingles | **Painted Deco** | `deco_shingles_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **101** | `slate_bricks` | Slate Bricks | **Masonry** | `deco_slate_bricks.png` *(+2 variants)* | 6 uniform faces |
| **102** | `wool` | Wool | **Textile** | `deco_wool.png` | 6 uniform faces |
| **103** | `wool_black` | Black Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **104** | `wool_blue` | Blue Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **105** | `wool_brown` | Brown Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **106** | `wool_dark_blue` | Dark Blue Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **107** | `wool_dark_grey` | Dark Grey Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **108** | `wool_green` | Green Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **109** | `wool_light_grey` | Light Grey Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **110** | `wool_light_pink` | Light Pink Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **111** | `wool_lime` | Lime Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **112** | `wool_orange` | Orange Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **113** | `wool_pink` | Pink Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **114** | `wool_purple` | Purple Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **115** | `wool_red` | Red Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **116** | `wool_turquoise` | Turquoise Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **117** | `wool_white` | White Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **118** | `wool_yellow` | Yellow Wool | **Painted Deco** | `deco_wool_painted.png` *(+2 variants)* | 6 uniform faces *(16-color sRGB dye palette tint)* |
| **119** | `element_fire` | Fire | **Decoration** | `element_fire.png` *(+2 variants)* | Architectural / Lighting fixture |
| **120** | `bag_rice` | Rice Bag | **Decoration** | `util_bag_side.png` | Architectural / Lighting fixture |
| **121** | `bag_sugar` | Sugar Bag | **Decoration** | `util_bag_side.png` | Architectural / Lighting fixture |

---

## 11. Mycology: Mushrooms (`mushrooms`) — 12 Entries

Surface and subterranean fungi caps, stalks, and cross-billboard mushrooms.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `fungi_amanita` | Amanita | **Ground Mushroom** | `fungi_amanita.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **02** | `fungi_amanita_cap` | Amanita Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_amanita_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **03** | `fungi_inside` | Fungi Inside Flesh | **Macrofungal Structure** | `fungi_inside.png` *(+ `1`, `2`)* | 6 uniform faces |
| **04** | `fungi_porcini` | Porcini | **Ground Mushroom** | `fungi_porcini.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **05** | `fungi_porcini_cap` | Porcini Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_porcini_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **06** | `fungi_russula` | Russula | **Ground Mushroom** | `fungi_russula.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **07** | `fungi_russula_cap` | Russula Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_russula_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **08** | `fungi_shiitake` | Shiitake | **Ground Mushroom** | `fungi_shiitake.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **09** | `fungi_shiitake_cap` | Shiitake Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_shiitake_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **10** | `fungi_stem` | Fungi Stem | **Macrofungal Structure** | `fungi_stem.png` *(+ `1`, `2`)* | 6 uniform faces |
| **11** | `fungi_verdigris` | Verdigris | **Ground Mushroom** | `fungi_verdigris.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **12** | `fungi_verdigris_cap` | Verdigris Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_verdigris_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |

---

## 12. Botany: Groundcover & Herbaceous Flora (`vegetation`) — 33 Entries

Wildgrass, ferns, bushes, cacti, vines, moss, lily pads, and segmented `vege_leaf_litter`.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `vege_wildgrass` | Wildgrass | **Grass** | `vege_wildgrass.png` *(+ `1`, `2`)* | Single Cross-Mesh (Plains, Meadows & Forests) |
| **02** | `vege_drygrass` | Drygrass | **Grass** | `vege_drygrass.png` *(+ `1`, `2`)* | Single Cross-Mesh (Savannas & Scrublands) |
| **03** | `vege_bluff_grass` | Bluff Grass | **Grass** | `vege_bluff_grass.png` *(+ `1`, `2`)* | Single Cross-Mesh (Rocky Crags & Cliffs) |
| **04** | `vege_mossgrass` | Mossgrass | **Grass** | `vege_mossgrass.png` *(+ `1`, `2`)* | Low Dense Tuft (Damp Woodlands & River Glades) |
| **05** | `vege_tall_wildgrass_bottom` | Tall Wildgrass | **Vegetation** | `vege_tall_wildgrass_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **06** | `vege_tall_wildgrass_top` | Tall Wildgrass | **Vegetation** | `vege_tall_wildgrass_top.png` *(+2 variants)* | Cross-billboard cutout |
| **07** | `vege_sea_oats_bottom` | Sea Oats | **Vegetation** | `vege_sea_oats_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **08** | `vege_sea_oats_top` | Sea Oats | **Vegetation** | `vege_sea_oats_top.png` *(+2 variants)* | Cross-billboard cutout |
| **09** | `vege_fern` | Common Fern | **Fern** | `vege_fern.png` *(+ `1`, `2`)* | Single Cross-Mesh (Shaded Woodlands & Taigas) |
| **10** | `vege_bracken` | Bracken Fern | **Fern** | `vege_bracken.png` *(+ `1`, `2`)* | Spreading Frond Mesh (Moorlands & Open Woods) |
| **11** | `vege_tall_fern_bottom` | Tall Fern | **Vegetation** | `vege_tall_fern_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **12** | `vege_tall_fern_top` | Tall Fern | **Vegetation** | `vege_tall_fern_top.png` *(+2 variants)* | Cross-billboard cutout |
| **13** | `vege_tall_jungle_fern_bottom` | Tall Jungle Fern | **Vegetation** | `vege_tall_jungle_fern_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **14** | `vege_tall_jungle_fern_top` | Tall Jungle Fern | **Vegetation** | `vege_tall_jungle_fern_top.png` *(+2 variants)* | Cross-billboard cutout |
| **15** | `vege_bush` | Temperate Bush | **Shrub** | `vege_bush.png` *(+ `1`, `2`)* | Dense Rounded Mesh (Forest Edges & Meadows) |
| **16** | `vege_red_shrub` | Red Shrub | **Shrub** | `vege_red_shrub.png` *(+ `1`, `2`)* | Red Pigmented Mesh (Autumn Woods & Moors) |
| **17** | `vege_dead_bush` | Dead Bush | **Shrub** | `vege_dead_bush.png` *(+ `1`, `2`)* | Withered Woody Sticks (Arid Deserts & Badlands) |
| **18** | `vege_barrel_cactus` | Barrel Cactus | **Cactus** | `vege_barrel_cactus.png` *(+ `1`, `2`)* | Warm Deserts, Scrub & Chaparrals (Spiny Succulent Mesh) |
| **19** | `vege_prickly_pear_cactus` | Prickly Pear Cactus | **Cactus** | `vege_prickly_pear_cactus.png` *(+ `1`, `2`)* | Fleshy Pad Cross-Mesh (Semiarid Slopes & Scrub) |
| **20** | `vege_cactus` | Saguaro Cactus | **Cactus** | `vege_cactus_top.png`<br>`vege_cactus_side.png`<br>`vege_cactus_bottom.png` | 3D Full Column Block (3 sides, 3 tops, 1 bottom) |
| **21** | `vege_cattail_bottom` | Cattail | **Vegetation** | `vege_cattail_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **22** | `vege_cattail_top` | Cattail | **Vegetation** | `vege_cattail_top.png` *(+2 variants)* | Cross-billboard cutout |
| **23** | `vege_lily_pad` | Water Lily Pad | **Floating Pad** | `vege_lily_pad.png` *(+ `1`, `2`)* | Floating Water Surface Mesh (Lakes & Swamps) |
| **24** | `vege_vines` | Vines | **Epiphyte** | `vege_vines.png` *(+ `1`, `2`)* | Vertical Wall Climber (Canopies & Cliff Walls) |
| **25** | `vege_hanging_moss` | Hanging Moss | **Epiphyte** | `vege_hanging_moss.png` *(+ `1`, `2`)* | Ceiling Hanging Decal (Swamps & Gallery Forests) |
| **26** | `vege_hanging_roots` | Hanging Roots | **Epiphyte** | `vege_hanging_roots.png` *(+ `1`, `2`)* | Ceiling Root Danglers (Overhangs & Ravines) |
| **27** | `vege_lichen` | Lichen | **Epiphyte** | `vege_lichen.png` *(+ `1`, `2`)* | Crustose Surface Patch (Barks, Rocks & Boulders) |
| **28** | `vege_cave_growths` | Cave Growths | **Cave Flora** | `vege_cave_growths.png` *(+ `1`, `2`)* | Ground Umbrophilic Thallus (Subterranean Caverns) |
| **29** | `vege_grainy_cave_growths` | Grainy Cave Growths | **Cave Flora** | `vege_grainy_cave_growths.png` *(+ `1`, `2`)* | Ferruginous Granular Crust (Subterranean Walls) |
| **30** | `vege_cave_hyssop` | Cave Hyssop | **Cave Flora** | `vege_cave_hyssop.png` *(+ `1`, `2`)* | Subterranean Flowering Herb (Geodes & Crystals) |
| **31** | `vege_cave_vines` | Cave Vines | **Cave Flora** | `vege_cave_vines.png`<br>`vege_cave_vines_plant.png` | Ceiling Hanging Multi-Block Vine (3 tip, 3 plant) |
| **32** | `vege_cave_vines_plant` | Cave Vines (Plant) | **Vegetation** | `vege_cave_vines_plant.png` *(+2 variants)* | Cross-billboard cutout |
| **33** | `vege_leaf_litter` | Leaf Litter | **Groundcover** | `vege_leaf_litter.png` *(+2 variants)* | Segmented 1–4 quadrant extruded `FlatPad` (biome dry-foliage tint) |

---

## 13. Botany: Flowers & Flowering Flora (`flowers`) — 34 Entries

Single and double-tall flowering plants, clover, and flat petal carpets.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `flower_allium` | Allium | **Wildflower** | `flower_allium.png` to `2` | Cross-billboard (3 variants) |
| **02** | `flower_blue_hydrangea_bottom` | Blue Hydrangea | **Flower** | `flower_blue_hydrangea_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **03** | `flower_blue_hydrangea_top` | Blue Hydrangea | **Flower** | `flower_blue_hydrangea_top.png` *(+2 variants)* | Cross-billboard cutout |
| **04** | `flower_blue_orchid` | Blue Orchid | **Wildflower** | `flower_blue_orchid.png` to `2` | Cross-billboard (3 variants) |
| **05** | `flower_cactus_flower` | Cactus Flower | **Arid Flora** | `flower_cactus_flower.png` to `2` | Single Cross / Perch (3 variants) |
| **06** | `flower_clover` | Clover | **Carpet** | `flower_clover.png` to `2`<br>`flower_carpet_stem.png` to `2` | Flat Quad + Central Upright Stem |
| **07** | `flower_cornflower` | Cornflower | **Wildflower** | `flower_cornflower.png` to `2` | Cross-billboard (3 variants) |
| **08** | `flower_dandelion` | Dandelion | **Wildflower** | `flower_dandelion.png` to `2` | Cross-billboard (3 variants) |
| **09** | `flower_foxglove_bottom` | Foxglove | **Flower** | `flower_foxglove_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **10** | `flower_foxglove_top` | Foxglove | **Flower** | `flower_foxglove_top.png` *(+2 variants)* | Cross-billboard cutout |
| **11** | `flower_gardenia_bottom` | Gardenia | **Flower** | `flower_gardenia_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **12** | `flower_gardenia_top` | Gardenia | **Flower** | `flower_gardenia_top.png` *(+2 variants)* | Cross-billboard cutout |
| **13** | `flower_globe_mallow` | Globe Mallow | **Arid Flora** | `flower_globe_mallow.png` to `2` | Single Cross (3 variants) |
| **14** | `flower_hibiscus` | Hibiscus | **Wildflower** | `flower_hibiscus.png` to `2` | Cross-billboard (3 variants) |
| **15** | `flower_iris` | Iris | **Wildflower** | `flower_iris.png` to `2` | Cross-billboard (3 variants) |
| **16** | `flower_lavender` | Lavender | **Wildflower** | `flower_lavender.png` to `2` | Cross-billboard (3 variants) |
| **17** | `flower_lilac_bottom` | Lilac | **Flower** | `flower_lilac_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **18** | `flower_lilac_top` | Lilac | **Flower** | `flower_lilac_top.png` *(+2 variants)* | Cross-billboard cutout |
| **19** | `flower_lily_of_the_valley` | Lily of the Valley | **Wildflower** | `flower_lily_of_the_valley.png` to `2` | Cross-billboard (3 variants) |
| **20** | `flower_marigold` | Marigold | **Wildflower** | `flower_marigold.png` to `2` | Cross-billboard (3 variants) |
| **21** | `flower_orange_tulip` | Orange Tulip | **Tulip** | `flower_orange_tulip.png` to `2` | Cross-billboard (3 variants) |
| **22** | `flower_orchid` | Classic Orchid | **Wildflower** | `flower_orchid.png` to `2` | Cross-billboard (3 variants) |
| **23** | `flower_oxeye_daisy` | Oxeye Daisy | **Wildflower** | `flower_oxeye_daisy.png` to `2` | Cross-billboard (3 variants) |
| **24** | `flower_peony_bottom` | Peony | **Flower** | `flower_peony_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **25** | `flower_peony_top` | Peony | **Flower** | `flower_peony_top.png` *(+2 variants)* | Cross-billboard cutout |
| **26** | `flower_pink_petals` | Pink Petals | **Carpet** | `flower_pink_petals.png` to `2`<br>`flower_carpet_stem.png` to `2` | Flat Quad + Central Upright Stem |
| **27** | `flower_pink_tulip` | Pink Tulip | **Tulip** | `flower_pink_tulip.png` to `2` | Cross-billboard (3 variants) |
| **28** | `flower_poppy` | Poppy | **Wildflower** | `flower_poppy.png` to `2` | Cross-billboard (3 variants) |
| **29** | `flower_rose_bush_bottom` | Rose Bush | **Flower** | `flower_rose_bush_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **30** | `flower_rose_bush_top` | Rose Bush | **Flower** | `flower_rose_bush_top.png` *(+2 variants)* | Cross-billboard cutout |
| **31** | `flower_snapdragon_bottom` | Snapdragon | **Flower** | `flower_snapdragon_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **32** | `flower_snapdragon_top` | Snapdragon | **Flower** | `flower_snapdragon_top.png` *(+2 variants)* | Cross-billboard cutout |
| **33** | `flower_white_petals` | White Petals | **Carpet** | `flower_white_petals.png` to `2`<br>`flower_carpet_stem.png` to `2` | Flat Quad + Central Upright Stem |
| **34** | `flower_wildflowers` | Yellow Petals | **Carpet** | `flower_wildflowers.png` to `2`<br>`flower_carpet_stem.png` to `2` | Flat Quad + Central Upright Stem |

---

## 14. Agriculture: Cultivated & Wild Crops (`crops`) — 121 Entries

Multi-stage agricultural crops, arboreal cocoa/coconut pods, cucurbit fruits/stems, and ancestral wild crops.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `crop_carrots_stage0` | Carrot Crop Stage 0 | **Cultivated Crop** | `crops_carrots_stage0.png` | Cross-billboard cutout |
| **02** | `crop_carrots_stage1` | Carrot Crop Stage 1 | **Cultivated Crop** | `crops_carrots_stage1.png` | Cross-billboard cutout |
| **03** | `crop_carrots_stage2` | Carrot Crop Stage 2 | **Cultivated Crop** | `crops_carrots_stage2.png` | Cross-billboard cutout |
| **04** | `crop_carrots_stage3` | Carrot Crop Stage 3 | **Cultivated Crop** | `crops_carrots_stage3.png` | Cross-billboard cutout |
| **05** | `crop_potatoes_stage0` | Potato Crop Stage 0 | **Cultivated Crop** | `crops_potatoes_stage0.png` | Cross-billboard cutout |
| **06** | `crop_potatoes_stage1` | Potato Crop Stage 1 | **Cultivated Crop** | `crops_potatoes_stage1.png` | Cross-billboard cutout |
| **07** | `crop_potatoes_stage2` | Potato Crop Stage 2 | **Cultivated Crop** | `crops_potatoes_stage2.png` | Cross-billboard cutout |
| **08** | `crop_potatoes_stage3` | Potato Crop Stage 3 | **Cultivated Crop** | `crops_potatoes_stage3.png` | Cross-billboard cutout |
| **09** | `crop_beetroots_stage0` | Beetroot Crop Stage 0 | **Cultivated Crop** | `crops_beetroots_stage0.png` | Cross-billboard cutout |
| **10** | `crop_beetroots_stage1` | Beetroot Crop Stage 1 | **Cultivated Crop** | `crops_beetroots_stage1.png` | Cross-billboard cutout |
| **11** | `crop_beetroots_stage2` | Beetroot Crop Stage 2 | **Cultivated Crop** | `crops_beetroots_stage2.png` | Cross-billboard cutout |
| **12** | `crop_beetroots_stage3` | Beetroot Crop Stage 3 | **Cultivated Crop** | `crops_beetroots_stage3.png` | Cross-billboard cutout |
| **13** | `crop_onions_stage0` | Onion Crop Stage 0 | **Cultivated Crop** | `crops_onions_stage0.png` | Cross-billboard cutout |
| **14** | `crop_onions_stage1` | Onion Crop Stage 1 | **Cultivated Crop** | `crops_onions_stage1.png` | Cross-billboard cutout |
| **15** | `crop_onions_stage2` | Onion Crop Stage 2 | **Cultivated Crop** | `crops_onions_stage2.png` | Cross-billboard cutout |
| **16** | `crop_onions_stage3` | Onion Crop Stage 3 | **Cultivated Crop** | `crops_onions_stage3.png` | Cross-billboard cutout |
| **17** | `crop_garlic_stage0` | Garlic Crop Stage 0 | **Cultivated Crop** | `crops_garlic_stage0.png` | Cross-billboard cutout |
| **18** | `crop_garlic_stage1` | Garlic Crop Stage 1 | **Cultivated Crop** | `crops_garlic_stage1.png` | Cross-billboard cutout |
| **19** | `crop_garlic_stage2` | Garlic Crop Stage 2 | **Cultivated Crop** | `crops_garlic_stage2.png` | Cross-billboard cutout |
| **20** | `crop_garlic_stage3` | Garlic Crop Stage 3 | **Cultivated Crop** | `crops_garlic_stage3.png` | Cross-billboard cutout |
| **21** | `crop_cabbages_stage0` | Cabbage Crop Stage 0 | **Cultivated Crop** | `crops_cabbages_stage0.png` | Cross-billboard cutout |
| **22** | `crop_cabbages_stage1` | Cabbage Crop Stage 1 | **Cultivated Crop** | `crops_cabbages_stage1.png` | Cross-billboard cutout |
| **23** | `crop_cabbages_stage2` | Cabbage Crop Stage 2 | **Cultivated Crop** | `crops_cabbages_stage2.png` | Cross-billboard cutout |
| **24** | `crop_cabbages_stage3` | Cabbage Crop Stage 3 | **Cultivated Crop** | `crops_cabbages_stage3.png` | Cross-billboard cutout |
| **25** | `crop_lettuce_stage0` | Lettuce Crop Stage 0 | **Cultivated Crop** | `crops_lettuce_stage0.png` | Cross-billboard cutout |
| **26** | `crop_lettuce_stage1` | Lettuce Crop Stage 1 | **Cultivated Crop** | `crops_lettuce_stage1.png` | Cross-billboard cutout |
| **27** | `crop_lettuce_stage2` | Lettuce Crop Stage 2 | **Cultivated Crop** | `crops_lettuce_stage2.png` | Cross-billboard cutout |
| **28** | `crop_lettuce_stage3` | Lettuce Crop Stage 3 | **Cultivated Crop** | `crops_lettuce_stage3.png` | Cross-billboard cutout |
| **29** | `crop_pepper_stage0` | Pepper Crop Stage 0 | **Cultivated Crop** | `crops_pepper_stage0.png` | Cross-billboard cutout |
| **30** | `crop_pepper_stage1` | Pepper Crop Stage 1 | **Cultivated Crop** | `crops_pepper_stage1.png` | Cross-billboard cutout |
| **31** | `crop_pepper_stage2` | Pepper Crop Stage 2 | **Cultivated Crop** | `crops_pepper_stage2.png` | Cross-billboard cutout |
| **32** | `crop_pepper_stage3` | Pepper Crop Stage 3 | **Cultivated Crop** | `crops_pepper_stage3.png` | Cross-billboard cutout |
| **33** | `crop_wheat_stage0` | Wheat Crop Stage 0 | **Cultivated Crop** | `crops_wheat_stage0.png` | Cross-billboard cutout |
| **34** | `crop_wheat_stage1` | Wheat Crop Stage 1 | **Cultivated Crop** | `crops_wheat_stage1.png` | Cross-billboard cutout |
| **35** | `crop_wheat_stage2` | Wheat Crop Stage 2 | **Cultivated Crop** | `crops_wheat_stage2.png` | Cross-billboard cutout |
| **36** | `crop_wheat_stage3` | Wheat Crop Stage 3 | **Cultivated Crop** | `crops_wheat_stage3.png` | Cross-billboard cutout |
| **37** | `crop_barley_bottom_stage0` | Barley Crop Stage 0 | **Cultivated Crop** | `crops_barley_bottom_stage0.png` | Cross-billboard cutout |
| **38** | `crop_barley_bottom_stage1` | Barley Crop Stage 1 | **Cultivated Crop** | `crops_barley_bottom_stage1.png` | Cross-billboard cutout |
| **39** | `crop_barley_bottom_stage2` | Barley Crop Lower Stage 2 | **Cultivated Crop** | `crops_barley_bottom_stage2.png` | Cross-billboard cutout |
| **40** | `crop_barley_top_stage2` | Barley Crop Upper Stage 2 | **Cultivated Crop** | `crops_barley_top_stage2.png` | Cross-billboard cutout |
| **41** | `crop_barley_bottom_stage3` | Barley Crop Lower Stage 3 | **Cultivated Crop** | `crops_barley_bottom_stage3.png` | Cross-billboard cutout |
| **42** | `crop_barley_top_stage3` | Barley Crop Upper Stage 3 | **Cultivated Crop** | `crops_barley_top_stage3.png` | Cross-billboard cutout |
| **43** | `crop_corn_budding_stage0` | Corn Budding Stage 0 | **Cultivated Crop** | `crops_corn_budding_stage0.png` | Cross-billboard cutout |
| **44** | `crop_corn_budding_stage1` | Corn Budding Stage 1 | **Cultivated Crop** | `crops_corn_budding_stage1.png` | Cross-billboard cutout |
| **45** | `crop_corn_budding_stage2` | Corn Budding Stage 2 | **Cultivated Crop** | `crops_corn_budding_stage2.png` | Cross-billboard cutout |
| **46** | `crop_corn_budding_stage3` | Corn Budding Stage 3 | **Cultivated Crop** | `crops_corn_budding_stage3.png` | Cross-billboard cutout |
| **47** | `crop_corn_bottom_stage0` | Corn Stalk Lower Stage 0 | **Cultivated Crop** | `crops_corn_bottom_stage0.png` | Cross-billboard cutout |
| **48** | `crop_corn_top_stage0` | Corn Stalk Upper Stage 0 | **Cultivated Crop** | `crops_corn_top_stage0.png` | Cross-billboard cutout |
| **49** | `crop_corn_bottom_stage1` | Corn Stalk Lower Stage 1 | **Cultivated Crop** | `crops_corn_bottom_stage1.png` | Cross-billboard cutout |
| **50** | `crop_corn_top_stage1` | Corn Stalk Upper Stage 1 | **Cultivated Crop** | `crops_corn_top_stage1.png` | Cross-billboard cutout |
| **51** | `crop_corn_bottom_stage2` | Corn Stalk Lower Stage 2 | **Cultivated Crop** | `crops_corn_bottom_stage2.png` | Cross-billboard cutout |
| **52** | `crop_corn_top_stage2` | Corn Stalk Upper Stage 2 | **Cultivated Crop** | `crops_corn_top_stage2.png` | Cross-billboard cutout |
| **53** | `crop_corn_bottom_stage3` | Corn Stalk Lower Stage 3 | **Cultivated Crop** | `crops_corn_bottom_stage3.png` | Cross-billboard cutout |
| **54** | `crop_corn_top_stage3` | Corn Stalk Upper Stage 3 | **Cultivated Crop** | `crops_corn_top_stage3.png` | Cross-billboard cutout |
| **55** | `crop_coffee_budding_stage0` | Coffee Budding Stage 0 | **Cultivated Crop** | `crops_coffee_budding_stage0.png` | Cross-billboard cutout |
| **56** | `crop_coffee_budding_stage1` | Coffee Budding Stage 1 | **Cultivated Crop** | `crops_coffee_budding_stage1.png` | Cross-billboard cutout |
| **57** | `crop_coffee_budding_stage2` | Coffee Budding Stage 2 | **Cultivated Crop** | `crops_coffee_budding_stage2.png` | Cross-billboard cutout |
| **58** | `crop_coffee_budding_stage3` | Coffee Budding Stage 3 | **Cultivated Crop** | `crops_coffee_budding_stage3.png` | Cross-billboard cutout |
| **59** | `crop_coffee_bottom_stage0` | Coffee Shrub Lower Stage 0 | **Cultivated Crop** | `crops_coffee_bottom_stage0.png` | Cross-billboard cutout |
| **60** | `crop_coffee_top_stage0` | Coffee Shrub Upper Stage 0 | **Cultivated Crop** | `crops_coffee_top_stage0.png` | Cross-billboard cutout |
| **61** | `crop_coffee_bottom_stage1` | Coffee Shrub Lower Stage 1 | **Cultivated Crop** | `crops_coffee_bottom_stage1.png` | Cross-billboard cutout |
| **62** | `crop_coffee_top_stage1` | Coffee Shrub Upper Stage 1 | **Cultivated Crop** | `crops_coffee_top_stage1.png` | Cross-billboard cutout |
| **63** | `crop_coffee_bottom_stage2` | Coffee Shrub Lower Stage 2 | **Cultivated Crop** | `crops_coffee_bottom_stage2.png` | Cross-billboard cutout |
| **64** | `crop_coffee_top_stage2` | Coffee Shrub Upper Stage 2 | **Cultivated Crop** | `crops_coffee_top_stage2.png` | Cross-billboard cutout |
| **65** | `crop_coffee_bottom_stage3` | Coffee Shrub Lower Stage 3 | **Cultivated Crop** | `crops_coffee_bottom_stage3.png` | Cross-billboard cutout |
| **66** | `crop_coffee_top_stage3` | Coffee Shrub Upper Stage 3 | **Cultivated Crop** | `crops_coffee_top_stage3.png` | Cross-billboard cutout |
| **67** | `crop_tomatoes_budding_stage0` | Tomato Trellis Stage 0 | **Cultivated Crop** | `crops_budding_tomatoes_stage0.png` | Cross-billboard cutout |
| **68** | `crop_tomatoes_budding_stage1` | Tomato Trellis Stage 1 | **Cultivated Crop** | `crops_budding_tomatoes_stage1.png` | Cross-billboard cutout |
| **69** | `crop_tomatoes_budding_stage2` | Tomato Trellis Stage 2 | **Cultivated Crop** | `crops_budding_tomatoes_stage2.png` | Cross-billboard cutout |
| **70** | `crop_tomatoes_budding_stage3` | Tomato Trellis Stage 3 | **Cultivated Crop** | `crops_budding_tomatoes_stage3.png` | Cross-billboard cutout |
| **71** | `crop_tomatoes_vine_stage0` | Tomato Fruiting Vine Stage 0 | **Cultivated Crop** | `crops_tomatoes_stage0.png` | Cross-billboard cutout |
| **72** | `crop_tomatoes_vine_stage1` | Tomato Fruiting Vine Stage 1 | **Cultivated Crop** | `crops_tomatoes_stage1.png` | Cross-billboard cutout |
| **73** | `crop_tomatoes_vine_stage2` | Tomato Fruiting Vine Stage 2 | **Cultivated Crop** | `crops_tomatoes_stage2.png` | Cross-billboard cutout |
| **74** | `crop_tomatoes_vine_stage3` | Tomato Fruiting Vine Stage 3 | **Cultivated Crop** | `crops_tomatoes_stage3.png` | Cross-billboard cutout |
| **75** | `crop_rice_stage0` | Rice Paddy Shoot Stage 0 | **Cultivated Crop** | `crops_rice_stage0.png` | Cross-billboard cutout |
| **76** | `crop_rice_stage1` | Rice Paddy Shoot Stage 1 | **Cultivated Crop** | `crops_rice_stage1.png` | Cross-billboard cutout |
| **77** | `crop_rice_stage2` | Rice Paddy Shoot Stage 2 | **Cultivated Crop** | `crops_rice_stage2.png` | Cross-billboard cutout |
| **78** | `crop_rice_stage3` | Rice Paddy Shoot Stage 3 | **Cultivated Crop** | `crops_rice_stage3.png` | Cross-billboard cutout |
| **79** | `crop_rice_supporting` | Rice Supporting Foliage | **Cultivated Crop** | `crops_rice_supporting.png` | Cross-billboard cutout |
| **80** | `crop_rice_panicles_stage0` | Rice Panicle Stage 0 | **Cultivated Crop** | `crops_rice_panicles_stage0.png` | Cross-billboard cutout |
| **81** | `crop_rice_panicles_stage1` | Rice Panicle Stage 1 | **Cultivated Crop** | `crops_rice_panicles_stage1.png` | Cross-billboard cutout |
| **82** | `crop_rice_panicles_stage2` | Rice Panicle Stage 2 | **Cultivated Crop** | `crops_rice_panicles_stage2.png` | Cross-billboard cutout |
| **83** | `crop_rice_panicles_stage3` | Rice Panicle Stage 3 | **Cultivated Crop** | `crops_rice_panicles_stage3.png` | Cross-billboard cutout |
| **84** | `crop_melon_stem_stage0` | Melon Stem Stage 0 | **Cultivated Crop** | `crops_melon_stem_stage0.png` | Cross-billboard cutout |
| **85** | `crop_melon_stem_stage1` | Melon Stem Stage 1 | **Cultivated Crop** | `crops_melon_stem_stage1.png` | Cross-billboard cutout |
| **86** | `crop_melon_stem_stage2` | Melon Stem Stage 2 | **Cultivated Crop** | `crops_melon_stem_stage2.png` | Cross-billboard cutout |
| **87** | `crop_melon_stem_attached` | Attached Melon Stem | **Cultivated Crop** | `crops_melon_stem_attached.png` | Cross-billboard cutout |
| **88** | `crop_pumpkin_stem_stage0` | Pumpkin Stem Stage 0 | **Cultivated Crop** | `crops_pumpkin_stem_stage0.png` | Cross-billboard cutout |
| **89** | `crop_pumpkin_stem_stage1` | Pumpkin Stem Stage 1 | **Cultivated Crop** | `crops_pumpkin_stem_stage1.png` | Cross-billboard cutout |
| **90** | `crop_pumpkin_stem_stage2` | Pumpkin Stem Stage 2 | **Cultivated Crop** | `crops_pumpkin_stem_stage2.png` | Cross-billboard cutout |
| **91** | `crop_pumpkin_stem_attached` | Attached Pumpkin Stem | **Cultivated Crop** | `crops_pumpkin_stem_attached.png` | Cross-billboard cutout |
| **92** | `melon_block` | Melon | **Cultivated Crop** | `crops_melon_side.png` | Full solid fruit cube |
| **93** | `pumpkin_block` | Pumpkin | **Cultivated Crop** | `crops_pumpkin_side.png` | Full solid fruit cube |
| **94** | `wild_carrots` | Wild Carrots | **Wild Crop** | `wild_carrots.png` *(+2 variants)* | Cross-billboard cutout |
| **95** | `wild_potatoes` | Wild Potatoes | **Wild Crop** | `wild_potatoes.png` *(+2 variants)* | Cross-billboard cutout |
| **96** | `wild_beetroots` | Wild Beetroots | **Wild Crop** | `wild_beetroots.png` *(+2 variants)* | Cross-billboard cutout |
| **97** | `wild_onions` | Wild Onions | **Wild Crop** | `wild_onions.png` *(+2 variants)* | Cross-billboard cutout |
| **98** | `wild_garlic` | Wild Garlic | **Wild Crop** | `wild_garlic.png` *(+2 variants)* | Cross-billboard cutout |
| **99** | `wild_cabbages` | Wild Cabbages | **Wild Crop** | `wild_cabbages.png` *(+2 variants)* | Cross-billboard cutout |
| **100** | `wild_lettuce` | Wild Lettuce | **Wild Crop** | `wild_lettuce.png` *(+2 variants)* | Cross-billboard cutout |
| **101** | `wild_pepper` | Wild Pepper | **Wild Crop** | `wild_pepper.png` *(+2 variants)* | Cross-billboard cutout |
| **102** | `wild_tomatoes` | Wild Tomatoes | **Wild Crop** | `wild_tomatoes.png` *(+2 variants)* | Cross-billboard cutout |
| **103** | `wild_coffee` | Wild Coffee | **Wild Crop** | `wild_coffee.png` *(+2 variants)* | Cross-billboard cutout |
| **104** | `wild_wheat_bottom` | Wild Wheat | **Wild Crop** | `wild_wheat_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **105** | `wild_wheat_top` | Wild Wheat | **Wild Crop** | `wild_wheat_top.png` *(+2 variants)* | Cross-billboard cutout |
| **106** | `wild_barley_bottom` | Wild Barley | **Wild Crop** | `wild_barley_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **107** | `wild_barley_top` | Wild Barley | **Wild Crop** | `wild_barley_top.png` *(+2 variants)* | Cross-billboard cutout |
| **108** | `wild_rice_bottom` | Wild Rice | **Wild Crop** | `wild_rice_bottom.png` | Cross-billboard cutout |
| **109** | `wild_rice_top` | Wild Rice | **Wild Crop** | `wild_rice_top.png` | Cross-billboard cutout |
| **110** | `wild_corn_bottom` | Wild Corn | **Wild Crop** | `wild_corn_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **111** | `wild_corn_top` | Wild Corn | **Wild Crop** | `wild_corn_top.png` *(+2 variants)* | Cross-billboard cutout |
| **112** | `wild_sugar_cane_bottom` | Wild Sugar Cane | **Wild Crop** | `wild_sugar_cane_bottom.png` *(+2 variants)* | Cross-billboard cutout |
| **113** | `wild_sugar_cane_top` | Wild Sugar Cane | **Wild Crop** | `wild_sugar_cane_top.png` *(+2 variants)* | Cross-billboard cutout |
| **114** | `wild_melon` | Wild Melon | **Wild Crop** | `crops_melon_side.png` | Full solid fruit cube |
| **115** | `wild_pumpkin` | Wild Pumpkin | **Wild Crop** | `crops_pumpkin_side.png` | Full solid fruit cube |
| **116** | `crop_cocoa_stage0` | Cocoa Pod | **Arboreal Crop** | `crops_cocoa_stage0.png` | Hanging/Lateral pod |
| **117** | `crop_cocoa_stage1` | Cocoa Pod | **Arboreal Crop** | `crops_cocoa_stage1.png` | Hanging/Lateral pod |
| **118** | `crop_cocoa_stage2` | Cocoa Pod | **Arboreal Crop** | `crops_cocoa_stage2.png` | Hanging/Lateral pod |
| **119** | `crop_coconut_stage0` | Coconut | **Arboreal Crop** | `crops_coconut_hanging_stage1.png` | Hanging/Lateral pod |
| **120** | `crop_coconut_stage1` | Coconut | **Arboreal Crop** | `crops_coconut_hanging_stage2.png` | Hanging/Lateral pod |
| **121** | `crop_coconut_stage2` | Coconut | **Arboreal Crop** | `crops_coconut_hanging_stage3.png` | Hanging/Lateral pod |

---

## 15. Harvested Items, Seeds & Materials (`items`) — 32 Entries

Inventory-only harvested produce, seeds, sticks, bone meal, and raw materials.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `item_wheat_seeds` | Wheat Seeds | **Item** | `items/wheat_seeds_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **02** | `item_barley_seeds` | Barley Seeds | **Item** | `items/barley_seeds_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **03** | `item_beetroot_seeds` | Beetroot Seeds | **Item** | `items/beetroot_seeds_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **04** | `item_cabbage_seeds` | Cabbage Seeds | **Item** | `items/cabbage_seeds_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **05** | `item_pepper_seeds` | Pepper Seeds | **Item** | `items/pepper_seeds_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **06** | `item_tomato_seeds` | Tomato Seeds | **Item** | `items/tomato_seeds_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **07** | `item_coffee_seeds` | Coffee Seeds | **Item** | `items/coffee_seeds_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **08** | `item_melon_seeds` | Melon Seeds | **Item** | `items/melon_seeds_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **09** | `item_pumpkin_seeds` | Pumpkin Seeds | **Item** | `items/pumpkin_seeds_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **10** | `item_wheat` | Wheat | **Item** | `items/wheat_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **11** | `item_barley` | Barley | **Item** | `items/barley_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **12** | `item_rice` | Rice | **Item** | `items/rice_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **13** | `item_corn` | Corn | **Item** | `items/corn_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **14** | `item_carrot` | Carrot | **Item** | `items/carrot_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **15** | `item_potato` | Potato | **Item** | `items/potato_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **16** | `item_beetroot` | Beetroot | **Item** | `items/beetroot_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **17** | `item_onion` | Onion | **Item** | `items/onion_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **18** | `item_garlic_bulb` | Garlic Bulb | **Item** | `items/garlic_bulb_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **19** | `item_cabbage` | Cabbage | **Item** | `items/cabbage_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **20** | `item_lettuce` | Lettuce | **Item** | `items/lettuce_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **21** | `item_pepper` | Pepper | **Item** | `items/pepper_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **22** | `item_tomato` | Tomato | **Item** | `items/tomato_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **23** | `item_coffee_berries` | Coffee Berries | **Item** | `items/coffee_berries_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **24** | `item_coffee_beans` | Coffee Beans | **Item** | `items/coffee_beans_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **25** | `item_melon_slice` | Melon Slice | **Item** | `items/melon_slice_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **26** | `item_pumpkin_slice` | Pumpkin Slice | **Item** | `items/pumpkin_slice_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **27** | `item_sugar_cane` | Sugar Cane | **Item** | `items/sugar_cane_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **28** | `item_sugar` | Sugar | **Item** | `items/sugar_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **29** | `item_bone_meal` | Bone Meal | **Item** | `items/bone_meal_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **30** | `item_stick` | Stick | **Item** | `items/stick_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **31** | `item_cocoa_beans` | Cocoa Beans | **Item** | `items/cocoa_beans_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
| **32** | `item_coconut` | Coconut | **Item** | `items/coconut_item.png` | 2D inventory item icon (non-placeable or seed/plantable) |
