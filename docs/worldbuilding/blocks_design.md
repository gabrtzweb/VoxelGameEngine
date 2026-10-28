# Voxel Game Engine Master Block Catalog (Blocks Design)

This master document centralizes, unifies, and catalogs **all blocks implemented in the Voxel Engine**, organized across the 14 worldbuilding subdomains located in `docs/worldbuilding/blocks/`.

> [!IMPORTANT]
> **Project Grand Total: 709 Cataloged Unique Blocks**  
> Summing all 14 core domains of the engine's block ecosystem:  
> **8** (systems) + **18** (fluids) + **56** (soils) + **68** (rocks) + **91** (trees) + **74** (ores) + **30** (caverns) + **50** (oceans) + **36** (organics) + **162** (decorations) + **18** (mushrooms) + **26** (vegetation) + **32** (flowers) + **36** (crops) = **709 Total Blocks**.

For each block, the following properties are systematically specified:

1. **Unique Block Identifier (`Block ID`)**.
2. **Canonical Name and Functional/Ecological Role**.
3. **Face Texture Mapping** (`top`, `bottom` / `bot`, `sides`, `front`, etc.).
4. **Compositional Layers, Shader Dynamic Tinting & Procedural Overlays**.

> [!NOTE]
> In accordance with project architectural guidelines:
>
> - **Stochastic / procedural variations** (e.g. suffixes `1`, `2`, `3`) are dynamically sampled by shaders to break visual tiling repetition across terrain and are **not** registered as duplicate blocks.
> - **Procedural texture overlays & decal layers** are centralized in `docs/worldbuilding/blocks/overlays/` and documented in [overlays_design.md](blocks/overlays_design.md). They register **0 standalone blocks** and are composited via multi-pass shaders.
> - **Integrated compositional variants** (such as the 16 `mossy_rock` / `mossy_cobbled` variants, `bloody_brain` / `bloody_flesh`, `algae_reefstone` / `barnacles_reefstone`, and flowering foliage) generate distinct registered engine blocks cataloged in their respective domain specifications.

---

## Master Summary & Block Census

| Category | Domain / Directory | Block Count | Texture Count | Status | Primary Architectural Specification | Navigation Link |
| :---: | :--- | :--- | :--- | :--- | :--- | :--- |
| **01** | Technical & System Blocks (`systems`) | **8 blocks** | **8 textures** | ✅ **100% Complete** | [systems_design.md](blocks/systems_design.md) | [Go to Section 01](#01-technical--system-blocks-systems--8-blocks--100-complete) |
| **02** | Hydrosphere & Cryology (`fluids`) | **18 blocks** | **54 textures** | ✅ **100% Complete** | [fluids_design.md](blocks/fluids_design.md) | [Go to Section 02](#02-hydrosphere--cryology-fluids--18-blocks--100-complete) |
| **03** | Pedology & Soils (`soils`) | **56 blocks** | **178 textures** | ✅ **100% Complete** | [soils_design.md](blocks/soils_design.md) | [Go to Section 03](#03-pedology--soils-soils--56-blocks--100-complete) |
| **04** | Geology & Rocks (`rocks`) | **68 blocks** | **162 textures** | ✅ **100% Complete** | [rocks_design.md](blocks/rocks_design.md) | [Go to Section 04](#04-geology--rocks-rocks--68-blocks--100-complete) |
| **05** | Trees, Forestry & Dendrology (`trees`) | **91 blocks** | **243 textures** | ✅ **100% Complete** | [trees_design.md](blocks/trees_design.md) | [Go to Section 05](#05-trees-forestry--dendrology-trees--91-blocks--100-complete) |
| **06** | Ores, Minerals & Gemology (`ores`) | **74 blocks** | **111 textures** | ✅ **100% Complete** | [ores_design.md](blocks/ores_design.md) | [Go to Section 06](#06-ores-minerals--gemology-ores--74-blocks--100-complete) |
| **07** | Speleology & Crystals (`caverns`) | **30 blocks** | **114 textures** | ✅ **100% Complete** | [caverns_design.md](blocks/caverns_design.md) | [Go to Section 07](#07-speleology--crystals-caverns--30-blocks--100-complete) |
| **08** | Ocean & Benthic Ecosystems (`oceans`) | **50 blocks** | **140 textures** | ✅ **100% Complete** | [oceans_design.md](blocks/oceans_design.md) | [Go to Section 08](#08-ocean--benthic-ecosystems-oceans--50-blocks--100-complete) |
| **09** | Biomaterials, Fauna & Organic Matter (`organics`) | **36 blocks** | **94 textures** | ✅ **100% Complete** | [organics_design.md](blocks/organics_design.md) | [Go to Section 09](#09-biomaterials-fauna--organic-matter-organics--36-blocks--100-complete) |
| **10** | Decorations, Lighting & Architecture (`decorations`) | **162 blocks** | **96 textures** | ✅ **100% Complete** | [decorations_design.md](blocks/decorations_design.md) | [Go to Section 10](#10-decorations-lighting--architecture-decorations--162-blocks--100-complete) |
| **11** | Mycology: Mushrooms (`mushrooms`) | **18 blocks** | **54 textures** | ✅ **100% Complete** | [mushrooms_design.md](blocks/mushrooms_design.md) | [Go to Section 11](#11-mycology-mushrooms-mushrooms--18-blocks--100-complete) |
| **12** | Botany: Groundcover & Herbaceous Flora (`vegetation`) | **26 blocks** | **102 textures** | ✅ **100% Complete** | [vegetation_design.md](blocks/vegetation_design.md) | [Go to Section 12](#12-botany-groundcover--herbaceous-flora-vegetation--26-blocks--100-complete) |
| **13** | Botany: Flowers & Flowering Flora (`flowers`) | **28 blocks** | **113 textures** | ✅ **100% Complete** | [flowers_design.md](blocks/flowers_design.md) | [Go to Section 13](#13-botany-flowers--flowering-flora-flowers--28-blocks--100-complete) |
| **14** | Agriculture & Domestic Crops (`crops`) | **36 blocks** | **183 textures** | 🔄 *In Progress* | [crops_design.md](blocks/crops_design.md) | [Go to Section 14](#14-agriculture--domestic-crops-crops--36-blocks) |
| **TOTAL** | **Grand Master Sum** | **705 blocks** | **1,652 textures** | — | — | — |

01. [Technical & System Blocks (`systems`) — 8 Blocks [✅ 100% Complete]](#01-technical--system-blocks-systems--8-blocks--100-complete)
02. [Hydrosphere & Cryology (`fluids`) — 18 Blocks [✅ 100% Complete]](#02-hydrosphere--cryology-fluids--18-blocks--100-complete)
03. [Pedology & Soils (`soils`) — 56 Blocks [✅ 100% Complete]](#03-pedology--soils-soils--56-blocks--100-complete)
04. [Geology & Rocks (`rocks`) — 68 Blocks [✅ 100% Complete]](#04-geology--rocks-rocks--68-blocks--100-complete)
05. [Trees, Forestry & Dendrology (`trees`) — 91 Blocks [✅ 100% Complete]](#05-trees-forestry--dendrology-trees--91-blocks--100-complete)
06. [Ores, Minerals & Gemology (`ores`) — 74 Blocks [✅ 100% Complete]](#06-ores-minerals--gemology-ores--74-blocks--100-complete)
07. [Speleology & Crystals (`caverns`) — 30 Blocks [✅ 100% Complete]](#07-speleology--crystals-caverns--30-blocks--100-complete)
08. [Ocean & Benthic Ecosystems (`oceans`) — 50 Blocks [✅ 100% Complete]](#08-ocean--benthic-ecosystems-oceans--50-blocks--100-complete)
09. [Biomaterials, Fauna & Organic Matter (`organics`) — 36 Blocks [✅ 100% Complete]](#09-biomaterials-fauna--organic-matter-organics--36-blocks--100-complete)
10. [Decorations, Lighting & Architecture (`decorations`) — 162 Blocks [✅ 100% Complete]](#10-decorations-lighting--architecture-decorations--162-blocks--100-complete)
11. [Mycology: Mushrooms (`mushrooms`) — 18 Blocks [✅ 100% Complete]](#11-mycology-mushrooms-mushrooms--18-blocks--100-complete)
12. [Botany: Groundcover & Herbaceous Flora (`vegetation`) — 26 Blocks [✅ 100% Complete]](#12-botany-groundcover--herbaceous-flora-vegetation--26-blocks--100-complete)
13. [Botany: Flowers & Flowering Flora (`flowers`) — 28 Blocks [✅ 100% Complete]](#13-botany-flowers--flowering-flora-flowers--28-blocks--100-complete)
14. [Agriculture & Domestic Crops (`crops`) — 36 Blocks](#14-agriculture--domestic-crops-crops--36-blocks)

---

## 01. Technical & System Blocks (`systems`) — 8 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/systems`  
📄 **Comprehensive Specification**: [systems_design.md](blocks/systems_design.md)

The engine internal technical and validation domain comprises **4 engine fallback placeholders** (solid cube, dynamic liquid, cross-billboard, and generic 3D UV checkerboard) and **4 developer pipeline validation markers** (initialization, chunk instance, pass, and fail) = **8 Total System Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `debug_accept` | Debug Accept | **Development** | `debug_accept.png` | 6 uniform faces *(Visual indicator of passing assertion / OK)* |
| **02** | `debug_fail` | Debug Fail | **Development** | `debug_fail.png` | 6 uniform faces *(Visual indicator of failed assertion / Error)* |
| **03** | `debug_instance` | Debug Instance | **Development** | `debug_instance.png` | 6 uniform faces *(Chunk / structure instance validation)* |
| **04** | `debug_start` | Debug Start | **Development** | `debug_start.png` | 6 uniform faces *(Pipeline initialization validation)* |
| **05** | `null_block` | Null Block | **Engine Fallback** | `null_block.png` | 6 uniform faces *(Rendered when full block texture or ID is missing)* |
| **06** | `null_crossmesh` | Null Crossmesh | **Engine Fallback** | `null_crossmesh.png` | Cross-billboard model *(Rendered when cross-billboard or flora texture is missing)* |
| **07** | `null_generic` | Null Generic | **Engine Fallback** | `null_generic.png` | UV face mapping *(Rendered when custom 3D model, prop, or entity texture is missing)* |
| **08** | `null_liquid` | Null Liquid | **Engine Fallback** | `null_liquid.png` | Animated vertical scrolling strip *(Rendered when liquid texture is unassigned)* |

---

## 02. Hydrosphere & Cryology (`fluids`) — 18 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/fluids`  
📄 **Comprehensive Specification**: [fluids_design.md](blocks/fluids_design.md)

The hydrosphere, rheology, and cryosphere domain comprises **8 cryogenic ice blocks** (clear, fragile, packed, black, and blue varieties) and **10 dynamic physical liquids** (each backed by static block icon, still surface animation strip, and cascading flow strip) = **18 Total Fluid & Ice Blocks**.

---

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

## 03. Pedology & Soils (`soils`) — 56 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/soils`  
📄 **Comprehensive Specification**: [soils_design.md](blocks/soils_design.md)

The planetary pedology system classifies soils into **3 core functional divisions**: 24 biological zonal soils (Loam, Silt, Peat series across 8 functional agricultural/turf states), 12 extreme climatic zonal soils (Permafrost, Caliche, Laterite series), and 22 mineral granular sediments, cryosphere covers, and biological mats = **58 Total Pedological Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `ash_pumice` | Pumice Ash | **Pyroclastic** | `soil_ash_pumice.png` | 6 uniform faces |
| **02** | `ash_volcanic` | Volcanic Ash | **Pyroclastic** | `soil_ash_vulcanic.png` | 6 uniform faces |
| **03** | `caliche_coarse` | Caliche Coarse | **Climatic** | `soil_caliche_coarse.png` | 6 uniform faces |
| **04** | `caliche_cracked` | Caliche Cracked | **Climatic** | `soil_caliche_cracked.png` | 6 uniform faces |
| **05** | `caliche_dirt` | Caliche Dirt | **Climatic** | `soil_caliche_dirt.png` | 6 uniform faces |
| **06** | `caliche_grass` | Caliche Grass | **Climatic** | Top: `soil_grass.png`<br>Bottom: `soil_caliche_dirt.png`<br>Sides: `soil_caliche_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `blocks/overlays/soil_grass_side_overlay.png`) |
| **07** | `clay_blue` | Blue Clay | **Clay** | `soil_clay_blue.png` | 6 uniform faces |
| **08** | `clay_red` | Red Clay | **Clay** | `soil_clay_red.png` | 6 uniform faces |
| **09** | `clay_white` | White Clay | **Clay** | `soil_clay_white.png` | 6 uniform faces |
| **10** | `clay_yellow` | Yellow Clay | **Clay** | `soil_clay_yellow.png` | 6 uniform faces |
| **11** | `gravel_ashy` | Ashy Gravel | **Gravel** | `soil_gravel_ashy.png` | 6 uniform faces |
| **12** | `gravel_dirty` | Dirty Gravel | **Gravel** | `soil_gravel_dirty.png` | 6 uniform faces |
| **13** | `gravel_sandy` | Sandy Gravel | **Gravel** | `soil_gravel_sandy.png` | 6 uniform faces |
| **14** | `gravel_snowy` | Snowy Gravel | **Gravel** | `soil_gravel_snowy.png` | 6 uniform faces |
| **15** | `laterite_coarse` | Laterite Coarse | **Climatic** | `soil_laterite_coarse.png` | 6 uniform faces |
| **16** | `laterite_dirt` | Laterite Dirt | **Climatic** | `soil_laterite_dirt.png` | 6 uniform faces |
| **17** | `laterite_grass` | Laterite Grass | **Climatic** | Top: `soil_grass.png`<br>Bottom: `soil_laterite_dirt.png`<br>Sides: `soil_laterite_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `blocks/overlays/soil_grass_side_overlay.png`) |
| **18** | `laterite_hardpan` | Laterite Hardpan | **Climatic** | `soil_laterite_hardpan.png` | 6 uniform faces |
| **19** | `loam_coarse` | Loam Coarse | **Biological** | `soil_loam_coarse.png` | 6 uniform faces |
| **20** | `loam_dirt` | Loam Dirt | **Biological** | `soil_loam_dirt.png` | 6 uniform faces |
| **21** | `loam_grass` | Loam Grass | **Biological** | Top: `soil_grass.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `blocks/overlays/soil_grass_side_overlay.png`) |
| **22** | `loam_mud` | Loam Mud | **Biological** | `soil_loam_mud.png` | 6 uniform faces |
| **23** | `loam_mulch` | Loam Mulch | **Biological** | Top: `soil_loam_mulch.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_mulch_side.png` | Top + Bottom + 4 sides |
| **24** | `loam_rooted` | Loam Rooted | **Biological** | `soil_loam_rooted.png` | 6 uniform faces |
| **25** | `loam_snow` | Loam Snow | **Biological** | Top: `soil_snow.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_snow_side.png` | Top + Bottom + 4 sides |
| **26** | `loam_tilled` | Loam Tilled | **Biological** | Top: `soil_loam_tilled_top.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_tilled_side.png` | Top + Bottom + 4 sides |
| **27** | `moss_amber` | Amber Moss | **Ground Moss** | `soil_moss_amber.png` | 6 uniform faces |
| **28** | `moss_cave` | Cave Moss | **Ground Moss** | `soil_moss_cave.png` | 6 uniform faces |
| **29** | `moss_crimson` | Crimson Moss | **Ground Moss** | `soil_moss_crimson.png` | 6 uniform faces |
| **30** | `moss_forest` | Forest Moss | **Ground Moss** | `soil_moss_forest.png` | 6 uniform faces |
| **31** | `peat_coarse` | Peat Coarse | **Biological** | `soil_peat_coarse.png` | 6 uniform faces |
| **32** | `peat_dirt` | Peat Dirt | **Biological** | `soil_peat_dirt.png` | 6 uniform faces |
| **33** | `peat_grass` | Peat Grass | **Biological** | Top: `soil_grass.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `blocks/overlays/soil_grass_side_overlay.png`) |
| **34** | `peat_mud` | Peat Mud | **Biological** | `soil_peat_mud.png` | 6 uniform faces |
| **35** | `peat_mulch` | Peat Mulch | **Biological** | Top: `soil_peat_mulch.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_mulch_side.png` | Top + Bottom + 4 sides |
| **36** | `peat_rooted` | Peat Rooted | **Biological** | `soil_peat_rooted.png` | 6 uniform faces |
| **37** | `peat_snow` | Peat Snow | **Biological** | Top: `soil_snow.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_snow_side.png` | Top + Bottom + 4 sides |
| **38** | `peat_tilled` | Peat Tilled | **Biological** | Top: `soil_peat_tilled_top.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_tilled_side.png` | Top + Bottom + 4 sides |
| **39** | `permafrost_coarse` | Permafrost Coarse | **Climatic** | `soil_permafrost_coarse.png` | 6 uniform faces |
| **40** | `permafrost_dirt` | Permafrost Dirt | **Climatic** | `soil_permafrost_dirt.png` | 6 uniform faces |
| **41** | `permafrost_grass` | Permafrost Grass | **Climatic** | Top: `soil_grass.png`<br>Bottom: `soil_permafrost_dirt.png`<br>Sides: `soil_permafrost_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `blocks/overlays/soil_grass_side_overlay.png`) |
| **42** | `permafrost_icy` | Permafrost Icy | **Climatic** | `soil_permafrost_icy.png` | 6 uniform faces |
| **43** | `powder_snow` | Powder Snow | **Cryosphere** | `soil_snow_powder.png` | 6 uniform faces |
| **44** | `sand_common` | Sand | **Sand** | `soil_sand_common.png` | 6 uniform faces |
| **45** | `sand_dune` | Dune Sand | **Sand** | `soil_sand_dune.png` | 6 uniform faces |
| **46** | `sand_red` | Red Sand | **Sand** | `soil_sand_red.png` | 6 uniform faces |
| **47** | `sand_white` | White Sand | **Sand** | `soil_sand_white.png` | 6 uniform faces |
| **48** | `silt_coarse` | Silt Coarse | **Biological** | `soil_silt_coarse.png` | 6 uniform faces |
| **49** | `silt_dirt` | Silt Dirt | **Biological** | `soil_silt_dirt.png` | 6 uniform faces |
| **50** | `silt_grass` | Silt Grass | **Biological** | Top: `soil_grass.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `blocks/overlays/soil_grass_side_overlay.png`) |
| **51** | `silt_mud` | Silt Mud | **Biological** | `soil_silt_mud.png` | 6 uniform faces |
| **52** | `silt_mulch` | Silt Mulch | **Biological** | Top: `soil_silt_mulch.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_mulch_side.png` | Top + Bottom + 4 sides |
| **53** | `silt_rooted` | Silt Rooted | **Biological** | `soil_silt_rooted.png` | 6 uniform faces |
| **54** | `silt_snow` | Silt Snow | **Biological** | Top: `soil_snow.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_snow_side.png` | Top + Bottom + 4 sides |
| **55** | `silt_tilled` | Silt Tilled | **Biological** | Top: `soil_silt_tilled_top.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_tilled_side.png` | Top + Bottom + 4 sides |
| **56** | `snow_block` | Snow | **Cryosphere** | `soil_snow.png` | 6 uniform faces |

---

## 04. Geology & Rocks (`rocks`) — 68 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/rocks`  
📄 **Comprehensive Specification**: [rocks_design.md](blocks/rocks_design.md)

The geological framework comprises **17 mineable rock matrices** across 7 core petrological domains (yielding 17 solid stone and 17 mined cobbled stone = 34 structural blocks), **14 surface relief outcrops** (grass and snow transitions = 14 blocks), **6 hand-collectable surface pebbles**, and **14 humid biome mossy composite blocks** (7 solid mossy rocks + 7 mossy cobbled stones) = **68 Total Geological Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping / Geometry |
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
| **24** | `cobbled_obsidian` | Cobbled Obsidian | **Volcanic** | `rock_obsidian_cobbled.png` | 6 uniform faces |
| **25** | `cobbled_sandstone_common` | Cobbled Sandstone | **Sedimentary** | `rock_sandstone_common_cobbled.png` | 6 uniform faces |
| **26** | `cobbled_sandstone_dune` | Cobbled Dune Sandstone | **Sedimentary** | `rock_sandstone_dune_cobbled.png` | 6 uniform faces |
| **27** | `cobbled_sandstone_red` | Cobbled Red Sandstone | **Sedimentary** | `rock_sandstone_red_cobbled.png` | 6 uniform faces |
| **28** | `cobbled_sandstone_white` | Cobbled White Sandstone | **Sedimentary** | `rock_sandstone_white_cobbled.png` | 6 uniform faces |
| **29** | `cobbled_slate` | Cobbled Slate | **Metamorphic** | `rock_slate_cobbled.png` | 6 uniform faces |
| **30** | `dolomite` | Dolomite (Alpine Stone) | **Sedimentary** | `rock_dolomite.png` | 6 uniform faces |
| **31** | `dolomite_grass` | Dolomite Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_dolomite.png`<br>Sides: `rock_dolomite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **32** | `dolomite_snow` | Dolomite Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_dolomite.png`<br>Sides: `rock_dolomite_snow_side.png` | Top + Bot + 4 sides |
| **33** | `gabbro` | Gabbro | **Plutonic** | `rock_gabbro.png` | 6 uniform faces |
| **34** | `granite` | Granite | **Plutonic** | `rock_granite.png` | 6 uniform faces |
| **35** | `granite_grass` | Granite Grass Outcrop | **Plutonic** | Top: `soil_grass.png`<br>Bot: `rock_granite.png`<br>Sides: `rock_granite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **36** | `granite_snow` | Granite Snow Outcrop | **Plutonic** | Top: `soil_snow.png`<br>Bot: `rock_granite.png`<br>Sides: `rock_granite_snow_side.png` | Top + Bot + 4 sides |
| **37** | `limestone` | Limestone | **Sedimentary** | `rock_limestone.png` | 6 uniform faces |
| **38** | `limestone_grass` | Limestone Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_limestone.png`<br>Sides: `rock_limestone_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **39** | `limestone_snow` | Limestone Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_limestone.png`<br>Sides: `rock_limestone_snow_side.png` | Top + Bot + 4 sides |
| **40** | `magma` | Magma | **Geothermal** | `rock_magma.png` *(+ `1`, `2`)* | 6 uniform faces *(Animated incandescent veins, emits light)* |
| **41** | `mantle` | Mantle | **Bedrock** | `rock_mantle.png` *(+ `1`, `2`)* | 6 uniform faces *(Unbreakable)* |
| **42** | `marble` | Marble | **Metamorphic** | `rock_marble.png` | 6 uniform faces |
| **43** | `mossy_andesite` | Mossy Andesite | **Continental** | `rock_andesite.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **44** | `mossy_argillite` | Mossy Argillite | **Sedimentary** | `rock_argillite.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **45** | `mossy_basalt` | Mossy Basalt | **Volcanic** | `rock_basalt.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **46** | `mossy_chalk` | Mossy Chalk | **Sedimentary** | `rock_chalk.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **47** | `mossy_cobbled_andesite` | Mossy Cobbled Andesite | **Continental** | `rock_andesite_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **48** | `mossy_cobbled_argillite` | Mossy Cobbled Argillite | **Sedimentary** | `rock_argillite_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **49** | `mossy_cobbled_basalt` | Mossy Cobbled Basalt | **Volcanic** | `rock_basalt_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **50** | `mossy_cobbled_chalk` | Mossy Cobbled Chalk | **Sedimentary** | `rock_chalk_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **51** | `mossy_cobbled_dolomite` | Mossy Cobbled Dolomite | **Sedimentary** | `rock_dolomite_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **52** | `mossy_cobbled_granite` | Mossy Cobbled Granite | **Plutonic** | `rock_granite_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **53** | `mossy_cobbled_limestone` | Mossy Cobbled Limestone | **Sedimentary** | `rock_limestone_cobbled.png` *(+ `mossy_cobbled_overlay.png`)* | 6 faces + Shader Cobble Joint Moss (Stochastic: base, 1, 2) |
| **54** | `mossy_dolomite` | Mossy Dolomite | **Sedimentary** | `rock_dolomite.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **55** | `mossy_granite` | Mossy Granite | **Plutonic** | `rock_granite.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **56** | `mossy_limestone` | Mossy Limestone | **Sedimentary** | `rock_limestone.png` *(+ `mossy_rock_overlay.png`)* | 6 faces + Shader Moss Overlay (Stochastic: base, 1, 2) |
| **57** | `obsidian` | Obsidian | **Volcanic** | `rock_obsidian.png` | 6 uniform faces |
| **58** | `pebble_chert` | Chert Pebble | **Pebble (Single)** | `pebble_chert.png` *(+ `1`, `2`)* | Low sub-voxel ground pebble |
| **59** | `pebble_chert_cluster` | Chert Pebble Cluster | **Pebble (Cluster)** | `pebble_chert_cluster.png` *(+ `1`, `2`)* | Low sub-voxel cluster pebbles |
| **60** | `pebble_flint` | Flint Pebble | **Pebble (Single)** | `pebble_flint.png` *(+ `1`, `2`)* | Low sub-voxel ground pebble |
| **61** | `pebble_flint_cluster` | Flint Pebble Cluster | **Pebble (Cluster)** | `pebble_flint_cluster.png` *(+ `1`, `2`)* | Low sub-voxel cluster pebbles |
| **62** | `pebble_quartz` | Quartz Pebble | **Pebble (Single)** | `pebble_quartz.png` *(+ `1`, `2`)* | Low sub-voxel ground pebble |
| **63** | `pebble_quartz_cluster` | Quartz Pebble Cluster | **Pebble (Cluster)** | `pebble_quartz_cluster.png` *(+ `1`, `2`)* | Low sub-voxel cluster pebbles |
| **64** | `sandstone_common` | Sandstone | **Sedimentary** | `rock_sandstone_common.png` | 6 uniform faces |
| **65** | `sandstone_dune` | Dune Sandstone | **Sedimentary** | `rock_sandstone_dune.png` | 6 uniform faces |
| **66** | `sandstone_red` | Red Sandstone | **Sedimentary** | `rock_sandstone_red.png` | 6 uniform faces |
| **67** | `sandstone_white` | White Sandstone | **Sedimentary** | `rock_sandstone_white.png` | 6 uniform faces |
| **68** | `slate` | Slate | **Metamorphic** | `rock_slate.png` | 6 uniform faces |

---

## 05. Trees, Forestry & Dendrology (`trees`) — 91 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/trees`  
📄 **Comprehensive Specification**: [trees_design.md](blocks/trees_design.md)

Planetary dendrology encompasses **18 living forest tree species** distributed across 5 major bioclimatic zones. Tree blocks are systematically classified into: 35 wood/trunk blocks (34 logs and full bark blocks, plus 1 bamboo stalk), 18 dressed lumber building blocks (planks), 37 canopy foliage blocks (healthy and dead states), and 1 estuarine aerial root block = **91 Total Forestry Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `acacia_leaves` | Acacia Leaves | **Foliage** | `tree_acacia_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **02** | `acacia_leaves_dead` | Dead Acacia Leaves | **Foliage** | `tree_acacia_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **03** | `acacia_log` | Acacia Log | **Wood** | Top/Bottom: `tree_acacia_log.png`<br>Sides: `tree_acacia_bark.png` | Directional (Top/Bottom + 4 sides) |
| **04** | `acacia_planks` | Acacia Planks | **Planks** | `tree_acacia_planks.png` | 6 uniform faces |
| **05** | `acacia_wood` | Acacia Wood | **Wood** | `tree_acacia_bark.png` | 6 uniform faces |
| **06** | `aspen_leaves` | Golden Aspen Leaves | **Foliage** | `tree_aspen_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **07** | `aspen_leaves_dead` | Dead Aspen Leaves | **Foliage** | `tree_aspen_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **08** | `aspen_log` | Aspen Log | **Wood** | Top/Bottom: `tree_aspen_log.png`<br>Sides: `tree_aspen_bark.png` | Directional (Top/Bottom + 4 sides) |
| **09** | `aspen_planks` | Aspen Planks | **Planks** | `tree_aspen_planks.png` | 6 uniform faces |
| **10** | `aspen_wood` | Aspen Wood | **Wood** | `tree_aspen_bark.png` | 6 uniform faces |
| **11** | `bamboo_leaves` | Bamboo Leaves | **Foliage** | `tree_bamboo_leaves.png` *(+ `1`, `2`)* | Stalk column with cross-billboard leaves |
| **12** | `bamboo_planks` | Bamboo Planks | **Planks** | `tree_bamboo_planks.png` | 6 uniform faces |
| **13** | `bamboo_stalk` | Bamboo Stalk | **Wood / Stalk** | Top/Bottom: `tree_bamboo_stalk_top.png`<br>Sides: `tree_bamboo_stalk.png` | Directional column (3×3 top/bottom, 3×16 sides) |
| **14** | `baobab_leaves` | Baobab Leaves | **Foliage** | `tree_baobab_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **15** | `baobab_leaves_dead` | Dead Baobab Leaves | **Foliage** | `tree_baobab_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **16** | `baobab_log` | Baobab Log | **Wood** | Top/Bottom: `tree_baobab_log.png`<br>Sides: `tree_baobab_bark.png` | Directional (Top/Bottom + 4 sides) |
| **17** | `baobab_planks` | Baobab Planks | **Planks** | `tree_baobab_planks.png` | 6 uniform faces |
| **18** | `baobab_wood` | Baobab Wood | **Wood** | `tree_baobab_bark.png` | 6 uniform faces |
| **19** | `birch_leaves` | Birch Leaves | **Foliage** | `tree_birch_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **20** | `birch_leaves_dead` | Dead Birch Leaves | **Foliage** | `tree_birch_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **21** | `birch_log` | Birch Log | **Wood** | Top/Bottom: `tree_birch_log.png`<br>Sides: `tree_birch_bark.png` | Directional (Top/Bottom + 4 sides) |
| **22** | `birch_planks` | Birch Planks | **Planks** | `tree_birch_planks.png` | 6 uniform faces |
| **23** | `birch_wood` | Birch Wood | **Wood** | `tree_birch_bark.png` | 6 uniform faces |
| **24** | `cherry_leaves` | Cherry Blossom Leaves | **Foliage** | `tree_cherry_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **25** | `cherry_leaves_dead` | Dead Cherry Leaves | **Foliage** | `tree_cherry_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **26** | `cherry_log` | Cherry Log | **Wood** | Top/Bottom: `tree_cherry_log.png`<br>Sides: `tree_cherry_bark.png` | Directional (Top/Bottom + 4 sides) |
| **27** | `cherry_planks` | Cherry Planks | **Planks** | `tree_cherry_planks.png` | 6 uniform faces |
| **28** | `cherry_wood` | Cherry Wood | **Wood** | `tree_cherry_bark.png` | 6 uniform faces |
| **29** | `cypress_leaves` | Cypress Foliage | **Foliage** | `tree_cypress_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **30** | `cypress_leaves_dead` | Dead Cypress Foliage | **Foliage** | `tree_cypress_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **31** | `cypress_log` | Cypress Log | **Wood** | Top/Bottom: `tree_cypress_log.png`<br>Sides: `tree_cypress_bark.png` | Directional (Top/Bottom + 4 sides) |
| **32** | `cypress_planks` | Cypress Planks | **Planks** | `tree_cypress_planks.png` | 6 uniform faces |
| **33** | `cypress_wood` | Cypress Wood | **Wood** | `tree_cypress_bark.png` | 6 uniform faces |
| **34** | `fir_leaves` | Fir Needles | **Foliage** | `tree_fir_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **35** | `fir_leaves_dead` | Dead Fir Needles | **Foliage** | `tree_fir_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **36** | `fir_log` | Fir Log | **Wood** | Top/Bottom: `tree_fir_log.png`<br>Sides: `tree_fir_bark.png` | Directional (Top/Bottom + 4 sides) |
| **37** | `fir_planks` | Fir Planks | **Planks** | `tree_fir_planks.png` | 6 uniform faces |
| **38** | `fir_wood` | Fir Wood | **Wood** | `tree_fir_bark.png` | 6 uniform faces |
| **39** | `kapok_leaves` | Kapok Leaves | **Foliage** | `tree_kapok_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **40** | `kapok_leaves_dead` | Dead Kapok Leaves | **Foliage** | `tree_kapok_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **41** | `kapok_log` | Kapok Log | **Wood** | Top/Bottom: `tree_kapok_log.png`<br>Sides: `tree_kapok_bark.png` | Directional (Top/Bottom + 4 sides) |
| **42** | `kapok_planks` | Kapok Planks | **Planks** | `tree_kapok_planks.png` | 6 uniform faces |
| **43** | `kapok_wood` | Kapok Wood | **Wood** | `tree_kapok_bark.png` | 6 uniform faces |
| **44** | `mahogany_leaves` | Mahogany Leaves | **Foliage** | `tree_mahogany_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **45** | `mahogany_leaves_dead` | Dead Mahogany Leaves | **Foliage** | `tree_mahogany_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **46** | `mahogany_log` | Mahogany Log | **Wood** | Top/Bottom: `tree_mahogany_log.png`<br>Sides: `tree_mahogany_bark.png` | Directional (Top/Bottom + 4 sides) |
| **47** | `mahogany_planks` | Mahogany Planks | **Planks** | `tree_mahogany_planks.png` | 6 uniform faces |
| **48** | `mahogany_wood` | Mahogany Wood | **Wood** | `tree_mahogany_bark.png` | 6 uniform faces |
| **49** | `mangrove_leaves` | Mangrove Leaves | **Foliage** | `tree_mangrove_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **50** | `mangrove_leaves_dead` | Dead Mangrove Leaves | **Foliage** | `tree_mangrove_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **51** | `mangrove_log` | Mangrove Log | **Wood** | Top/Bottom: `tree_mangrove_log.png`<br>Sides: `tree_mangrove_bark.png` | Directional (Top/Bottom + 4 sides) |
| **52** | `mangrove_planks` | Mangrove Planks | **Planks** | `tree_mangrove_planks.png` | 6 uniform faces |
| **53** | `mangrove_roots` | Mangrove Aerial Roots | **Root** | Top/Bottom: `tree_mangrove_roots_top.png`<br>Sides: `tree_mangrove_roots.png` | Directional (Top/Bottom + 4 sides, Cutout) |
| **54** | `mangrove_wood` | Mangrove Wood | **Wood** | `tree_mangrove_bark.png` | 6 uniform faces |
| **55** | `maple_leaves_dead` | Dead Maple Leaves | **Foliage** | `tree_maple_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **56** | `maple_leaves_orange` | Orange Maple Leaves | **Foliage** | `tree_maple_leaves_orange.png` | 6 uniform faces (Cutout alpha transparency) |
| **57** | `maple_leaves_red` | Red Maple Leaves | **Foliage** | `tree_maple_leaves_red.png` | 6 uniform faces (Cutout alpha transparency) |
| **58** | `maple_leaves_yellow` | Yellow Maple Leaves | **Foliage** | `tree_maple_leaves_yellow.png` | 6 uniform faces (Cutout alpha transparency) |
| **59** | `maple_log` | Maple Log | **Wood** | Top/Bottom: `tree_maple_log.png`<br>Sides: `tree_maple_bark.png` | Directional (Top/Bottom + 4 sides) |
| **60** | `maple_planks` | Maple Planks | **Planks** | `tree_maple_planks.png` | 6 uniform faces |
| **61** | `maple_wood` | Maple Wood | **Wood** | `tree_maple_bark.png` | 6 uniform faces |
| **62** | `oak_leaves` | Oak Leaves | **Foliage** | `tree_oak_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **63** | `oak_leaves_dead` | Dead Oak Leaves | **Foliage** | `tree_oak_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **64** | `oak_log` | Oak Log | **Wood** | Top/Bottom: `tree_oak_log.png`<br>Sides: `tree_oak_bark.png` | Directional (Top/Bottom + 4 sides) |
| **65** | `oak_planks` | Oak Planks | **Planks** | `tree_oak_planks.png` | 6 uniform faces |
| **66** | `oak_wood` | Oak Wood | **Wood** | `tree_oak_bark.png` | 6 uniform faces |
| **67** | `palm_leaves` | Palm Leaves | **Foliage** | `tree_palm_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **68** | `palm_leaves_dead` | Dead Palm Leaves | **Foliage** | `tree_palm_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **69** | `palm_log` | Palm Log | **Wood** | Top/Bottom: `tree_palm_log.png`<br>Sides: `tree_palm_bark.png` | Directional (Top/Bottom + 4 sides) |
| **70** | `palm_planks` | Palm Planks | **Planks** | `tree_palm_planks.png` | 6 uniform faces |
| **71** | `palm_wood` | Palm Wood | **Wood** | `tree_palm_bark.png` | 6 uniform faces |
| **72** | `pine_leaves` | Pine Needles | **Foliage** | `tree_pine_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **73** | `pine_leaves_dead` | Dead Pine Needles | **Foliage** | `tree_pine_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **74** | `pine_log` | Pine Log | **Wood** | Top/Bottom: `tree_pine_log.png`<br>Sides: `tree_pine_bark.png` | Directional (Top/Bottom + 4 sides) |
| **75** | `pine_planks` | Pine Planks | **Planks** | `tree_pine_planks.png` | 6 uniform faces |
| **76** | `pine_wood` | Pine Wood | **Wood** | `tree_pine_bark.png` | 6 uniform faces |
| **77** | `redwood_leaves` | Redwood Foliage | **Foliage** | `tree_redwood_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **78** | `redwood_leaves_dead` | Dead Redwood Foliage | **Foliage** | `tree_redwood_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **79** | `redwood_log` | Redwood Log | **Wood** | Top/Bottom: `tree_redwood_log.png`<br>Sides: `tree_redwood_bark.png` | Directional (Top/Bottom + 4 sides) |
| **80** | `redwood_planks` | Redwood Planks | **Planks** | `tree_redwood_planks.png` | 6 uniform faces |
| **81** | `redwood_wood` | Redwood Wood | **Wood** | `tree_redwood_bark.png` | 6 uniform faces |
| **82** | `willow_leaves` | Weeping Willow Leaves | **Foliage** | `tree_willow_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **83** | `willow_leaves_dead` | Dead Willow Leaves | **Foliage** | `tree_willow_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **84** | `willow_log` | Willow Log | **Wood** | Top/Bottom: `tree_willow_log.png`<br>Sides: `tree_willow_bark.png` | Directional (Top/Bottom + 4 sides) |
| **85** | `willow_planks` | Willow Planks | **Planks** | `tree_willow_planks.png` | 6 uniform faces |
| **86** | `willow_wood` | Willow Wood | **Wood** | `tree_willow_bark.png` | 6 uniform faces |
| **87** | `yew_leaves` | Yew Foliage | **Foliage** | `tree_yew_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **88** | `yew_leaves_dead` | Dead Yew Foliage | **Foliage** | `tree_yew_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **89** | `yew_log` | Yew Log | **Wood** | Top/Bottom: `tree_yew_log.png`<br>Sides: `tree_yew_bark.png` | Directional (Top/Bottom + 4 sides) |
| **90** | `yew_planks` | Yew Planks | **Planks** | `tree_yew_planks.png` | 6 uniform faces |
| **91** | `yew_wood` | Yew Wood | **Wood** | `tree_yew_bark.png` | 6 uniform faces |

---

## 06. Ores, Minerals & Gemology (`ores`) — 74 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/ores`  
📄 **Comprehensive Specification**: [ores_design.md](blocks/ores_design.md)

The metallogenic and mineralogical domain comprises **20 solid unrefined raw/rough blocks** (8 native/precious metals + 4 chemicals + 8 rough gemstones) and **54 procedural ore veins** dynamically generated across 13 mineralized host rock matrices (featuring the 3 primary crustal macro-strata: Slate with 8 veins, Dolomite with 6 veins, and Gabbro with 6 veins, with sandstone_dune, obsidian, magma, and mantle remaining sterile) = **74 Total Mineral Blocks**. All blocks are organized in strict alphabetical order.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **001** | `gem_diamond_basalt` | Diamond Ore (Basalt) | **Gemstone** | `rock_basalt.png` | `gem_diamond_overlay.png` |
| **002** | `gem_diamond_gabbro` | Diamond Ore (Gabbro) | **Gemstone** | `rock_gabbro.png` | `gem_diamond_overlay.png` |
| **003** | `gem_diamond_rough` | Rough Diamond Block | **Gemstone** | `gem_diamond_rough.png` | Uniform (6 faces) |
| **004** | `gem_emerald_marble` | Emerald Ore (Marble) | **Gemstone** | `rock_marble.png` | `gem_emerald_overlay.png` |
| **005** | `gem_emerald_rough` | Rough Emerald Block | **Gemstone** | `gem_emerald_rough.png` | Uniform (6 faces) |
| **006** | `gem_emerald_slate` | Emerald Ore (Slate) | **Gemstone** | `rock_slate.png` | `gem_emerald_overlay.png` |
| **007** | `gem_jade_rough` | Rough Jade Block | **Gemstone** | `gem_jade_rough.png` | Uniform (6 faces) |
| **008** | `gem_onyx_rough` | Rough Onyx Block | **Gemstone** | `gem_onyx_rough.png` *(16×64)* | Animated (4 frames) |
| **009** | `gem_onyx_slate` | Onyx Ore (Slate) | **Gemstone** | `rock_slate.png` | `gem_onyx_overlay.png` |
| **010** | `gem_opal_rough` | Rough Opal Block | **Gemstone** | `gem_opal_rough.png` *(16×64)* | Animated (4 frames) |
| **011** | `gem_opal_sandstone_white` | Opal Ore (White Sandstone) | **Gemstone** | `rock_sandstone_white.png` | `gem_opal_overlay.png` |
| **012** | `gem_ruby_rough` | Rough Ruby Block | **Gemstone** | `gem_ruby_rough.png` | Uniform (6 faces) |
| **013** | `gem_sapphire_rough` | Rough Sapphire Block | **Gemstone** | `gem_sapphire_rough.png` | Uniform (6 faces) |
| **014** | `gem_topaz_rough` | Rough Topaz Block | **Gemstone** | `gem_topaz_rough.png` | Uniform (6 faces) |
| **015** | `gem_vein_andesite` | Gem Vein (Andesite) | **Gemstone** | `rock_andesite.png` | `gem_vein_overlay.png` |
| **016** | `gem_vein_basalt` | Gem Vein (Basalt) | **Gemstone** | `rock_basalt.png` | `gem_vein_overlay.png` |
| **017** | `gem_vein_dolomite` | Gem Vein (Dolomite) | **Gemstone** | `rock_dolomite.png` | `gem_vein_overlay.png` |
| **018** | `gem_vein_gabbro` | Gem Vein (Gabbro) | **Gemstone** | `rock_gabbro.png` | `gem_vein_overlay.png` |
| **019** | `gem_vein_granite` | Gem Vein (Granite) | **Gemstone** | `rock_granite.png` | `gem_vein_overlay.png` |
| **020** | `gem_vein_marble` | Gem Vein (Marble) | **Gemstone** | `rock_marble.png` | `gem_vein_overlay.png` |
| **021** | `gem_vein_slate` | Gem Vein (Slate) | **Gemstone** | `rock_slate.png` | `gem_vein_overlay.png` |
| **022** | `ore_cinnabar_basalt` | Cinnabar Ore (Basalt) | **Chemical** | `rock_basalt.png` | `ore_cinnabar_overlay.png` |
| **023** | `ore_cinnabar_raw` | Raw Cinnabar Block | **Chemical** | `ore_cinnabar_raw.png` | Uniform (6 faces) |
| **024** | `ore_coal_argillite` | Coal Ore (Argillite) | **Chemical** | `rock_argillite.png` | `ore_coal_overlay.png` |
| **025** | `ore_coal_chalk` | Coal Ore (Chalk) | **Chemical** | `rock_chalk.png` | `ore_coal_overlay.png` |
| **026** | `ore_coal_dolomite` | Coal Ore (Dolomite) | **Chemical** | `rock_dolomite.png` | `ore_coal_overlay.png` |
| **027** | `ore_coal_granite` | Coal Ore (Granite) | **Chemical** | `rock_granite.png` | `ore_coal_overlay.png` |
| **028** | `ore_coal_limestone` | Coal Ore (Limestone) | **Chemical** | `rock_limestone.png` | `ore_coal_overlay.png` |
| **029** | `ore_coal_raw` | Raw Coal Block | **Chemical** | `ore_coal_raw.png` | Uniform (6 faces) |
| **030** | `ore_coal_sandstone_common` | Coal Ore (Sandstone) | **Chemical** | `rock_sandstone_common.png` | `ore_coal_overlay.png` |
| **031** | `ore_copper_andesite` | Copper Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_copper_overlay.png` |
| **032** | `ore_copper_dolomite` | Copper Ore (Dolomite) | **Metal** | `rock_dolomite.png` | `ore_copper_overlay.png` |
| **033** | `ore_copper_granite` | Copper Ore (Granite) | **Metal** | `rock_granite.png` | `ore_copper_overlay.png` |
| **034** | `ore_copper_raw` | Raw Copper Block | **Metal** | `ore_copper_raw.png` | Uniform (6 faces) |
| **035** | `ore_copper_sandstone_common` | Copper Ore (Sandstone) | **Metal** | `rock_sandstone_common.png` | `ore_copper_overlay.png` |
| **036** | `ore_copper_sandstone_red` | Copper Ore (Red Sandstone) | **Metal** | `rock_sandstone_red.png` | `ore_copper_overlay.png` |
| **037** | `ore_copper_sandstone_white` | Copper Ore (White Sandstone) | **Metal** | `rock_sandstone_white.png` | `ore_copper_overlay.png` |
| **038** | `ore_gold_gabbro` | Gold Ore (Gabbro) | **Metal** | `rock_gabbro.png` | `ore_gold_overlay.png` |
| **039** | `ore_gold_granite` | Gold Ore (Granite) | **Metal** | `rock_granite.png` | `ore_gold_overlay.png` |
| **040** | `ore_gold_raw` | Raw Gold Block | **Metal** | `ore_gold_raw.png` | Uniform (6 faces) |
| **041** | `ore_gold_sandstone_red` | Gold Ore (Red Sandstone) | **Metal** | `rock_sandstone_red.png` | `ore_gold_overlay.png` |
| **042** | `ore_gold_slate` | Gold Ore (Slate) | **Metal** | `rock_slate.png` | `ore_gold_overlay.png` |
| **043** | `ore_iron_andesite` | Iron Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_iron_overlay.png` |
| **044** | `ore_iron_basalt` | Iron Ore (Basalt) | **Metal** | `rock_basalt.png` | `ore_iron_overlay.png` |
| **045** | `ore_iron_dolomite` | Iron Ore (Dolomite) | **Metal** | `rock_dolomite.png` | `ore_iron_overlay.png` |
| **046** | `ore_iron_gabbro` | Iron Ore (Gabbro) | **Metal** | `rock_gabbro.png` | `ore_iron_overlay.png` |
| **047** | `ore_iron_granite` | Iron Ore (Granite) | **Metal** | `rock_granite.png` | `ore_iron_overlay.png` |
| **048** | `ore_iron_raw` | Raw Iron Block | **Metal** | `ore_iron_raw.png` | Uniform (6 faces) |
| **049** | `ore_iron_slate` | Iron Ore (Slate) | **Metal** | `rock_slate.png` | `ore_iron_overlay.png` |
| **050** | `ore_lazurite_limestone` | Lazurite Ore (Limestone) | **Chemical** | `rock_limestone.png` | `ore_lazurite_overlay.png` |
| **051** | `ore_lazurite_marble` | Lazurite Ore (Marble) | **Chemical** | `rock_marble.png` | `ore_lazurite_overlay.png` |
| **052** | `ore_lazurite_raw` | Raw Lazurite Block | **Chemical** | `ore_lazurite_raw.png` | Uniform (6 faces) |
| **053** | `ore_platinum_basalt` | Platinum Ore (Basalt) | **Metal** | `rock_basalt.png` | `ore_platinum_overlay.png` |
| **054** | `ore_platinum_gabbro` | Platinum Ore (Gabbro) | **Metal** | `rock_gabbro.png` | `ore_platinum_overlay.png` |
| **055** | `ore_platinum_raw` | Raw Platinum Block | **Metal** | `ore_platinum_raw.png` | Uniform (6 faces) |
| **056** | `ore_silver_andesite` | Silver Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_silver_overlay.png` |
| **057** | `ore_silver_marble` | Silver Ore (Marble) | **Metal** | `rock_marble.png` | `ore_silver_overlay.png` |
| **058** | `ore_silver_raw` | Raw Silver Block | **Metal** | `ore_silver_raw.png` | Uniform (6 faces) |
| **059** | `ore_silver_slate` | Silver Ore (Slate) | **Metal** | `rock_slate.png` | `ore_silver_overlay.png` |
| **060** | `ore_sulfur_basalt` | Sulfur Ore (Basalt) | **Chemical** | `rock_basalt.png` | `ore_sulfur_overlay.png` |
| **061** | `ore_sulfur_raw` | Raw Sulfur Block | **Chemical** | `ore_sulfur_raw.png` | Uniform (6 faces) |
| **062** | `ore_tin_andesite` | Tin Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_tin_overlay.png` |
| **063** | `ore_tin_chalk` | Tin Ore (Chalk) | **Metal** | `rock_chalk.png` | `ore_tin_overlay.png` |
| **064** | `ore_tin_dolomite` | Tin Ore (Dolomite) | **Metal** | `rock_dolomite.png` | `ore_tin_overlay.png` |
| **065** | `ore_tin_granite` | Tin Ore (Granite) | **Metal** | `rock_granite.png` | `ore_tin_overlay.png` |
| **066** | `ore_tin_raw` | Raw Tin Block | **Metal** | `ore_tin_raw.png` | Uniform (6 faces) |
| **067** | `ore_uranium_gabbro` | Uranium Ore (Gabbro) | **Heavy Metal** | `rock_gabbro.png` | `ore_uranium_overlay.png` |
| **068** | `ore_uranium_raw` | Raw Uranium Block | **Heavy Metal** | `ore_uranium_raw.png` | Uniform (6 faces) |
| **069** | `ore_uranium_slate` | Uranium Ore (Slate) | **Heavy Metal** | `rock_slate.png` | `ore_uranium_overlay.png` |
| **070** | `ore_zinc_andesite` | Zinc Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_zinc_overlay.png` |
| **071** | `ore_zinc_argillite` | Zinc Ore (Argillite) | **Metal** | `rock_argillite.png` | `ore_zinc_overlay.png` |
| **072** | `ore_zinc_dolomite` | Zinc Ore (Dolomite) | **Metal** | `rock_dolomite.png` | `ore_zinc_overlay.png` |
| **073** | `ore_zinc_raw` | Raw Zinc Block | **Metal** | `ore_zinc_raw.png` | Uniform (6 faces) |
| **074** | `ore_zinc_slate` | Zinc Ore (Slate) | **Metal** | `rock_slate.png` | `ore_zinc_overlay.png` |

## 07. Speleology & Crystals (`caverns`) — 30 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/caverns`  
📄 **Comprehensive Specification**: [caverns_design.md](blocks/caverns_design.md)

The subterranean speleological domain comprises **6 solid mineral crystal blocks**, **6 crystal clusters**, **6 glinted emissive crystal clusters**, **4 modular speleothem/dripstone systems** (stalactites, stalagmites, columns, and Straws), **4 speleological cave stone strata** (Cave Karst and Cave Calcite solid and cobbled), and **4 paleontological fossil blocks** = **30 Total Cavern Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping / Geometry |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `cave_calcite` | Cave Calcite | **Speleological** | `cave_calcite.png` *(+ `1`, `2`)* | 6 uniform faces |
| **02** | `cave_karst` | Cave Karst | **Speleological** | `cave_karst.png` *(+ `1`, `2`)* | 6 uniform faces |
| **03** | `cobbled_cave_calcite` | Cobbled Cave Calcite | **Speleological** | `cave_calcite_cobbled.png` *(+ `1`, `2`)* | 6 uniform faces |
| **04** | `cobbled_cave_karst` | Cobbled Cave Karst | **Speleological** | `cave_karst_cobbled.png` *(+ `1`, `2`)* | 6 uniform faces |
| **05** | `crystal_amethyst_block` | Amethyst Block | **Crystal** | `crystal_amethyst.png` *(+ `1`, `2`)* | 6 uniform faces |
| **06** | `crystal_amethyst_cluster` | Amethyst Cluster | **Cluster** | `crystal_amethyst_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **07** | `crystal_amethyst_glinted` | Glinted Amethyst Cluster | **Emissive** | `crystal_amethyst_cluster_glinted.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **08** | `crystal_aquamarine_block` | Aquamarine Block | **Crystal** | `crystal_aquamarine.png` *(+ `1`, `2`)* | 6 uniform faces |
| **09** | `crystal_aquamarine_cluster` | Aquamarine Cluster | **Cluster** | `crystal_aquamarine_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **10** | `crystal_aquamarine_glinted` | Glinted Aquamarine Cluster | **Emissive** | `crystal_aquamarine_cluster_glinted.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **11** | `crystal_citrine_block` | Citrine Block | **Crystal** | `crystal_citrine.png` *(+ `1`, `2`)* | 6 uniform faces |
| **12** | `crystal_citrine_cluster` | Citrine Cluster | **Cluster** | `crystal_citrine_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **13** | `crystal_citrine_glinted` | Glinted Citrine Cluster | **Emissive** | `crystal_citrine_cluster_glinted.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **14** | `crystal_fluorite_block` | Fluorite Block | **Crystal** | `crystal_fluorite.png` *(+ `1`, `2`)* | 6 uniform faces |
| **15** | `crystal_fluorite_cluster` | Fluorite Cluster | **Cluster** | `crystal_fluorite_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **16** | `crystal_fluorite_glinted` | Glinted Fluorite Cluster | **Emissive** | `crystal_fluorite_cluster_glinted.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **17** | `crystal_pink_salt_block` | Pink Salt Block | **Crystal** | `crystal_pink_salt.png` *(+ `1`, `2`)* | 6 uniform faces |
| **18** | `crystal_pink_salt_cluster` | Pink Salt Cluster | **Cluster** | `crystal_pink_salt_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **19** | `crystal_pink_salt_glinted` | Glinted Pink Salt Cluster | **Emissive** | `crystal_pink_salt_cluster_glinted.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **20** | `crystal_selenite_block` | Selenite Block | **Crystal** | `crystal_selenite.png` *(+ `1`, `2`)* | 6 uniform faces |
| **21** | `crystal_selenite_cluster` | Selenite Cluster | **Cluster** | `crystal_selenite_cluster.png` *(+ `1`, `2`)* | Cross-billboard |
| **22** | `crystal_selenite_glinted` | Glinted Selenite Cluster | **Emissive** | `crystal_selenite_cluster_glinted.png` *(16×64 + `1`, `2`)* | Animated Cross-billboard (4 frames) |
| **23** | `fossil_amber` | Fossil Amber | **Fossil** | `fossil_amber.png` *(+ `1`, `2`)* | 6 uniform faces |
| **24** | `fossil_bitumen` | Fossil Bitumen | **Fossil** | `fossil_bitumen.png` *(+ `1`, `2`)* | 6 uniform faces |
| **25** | `fossil_bones` | Fossil Bones | **Fossil** | `fossil_bones.png` *(+ `1`, `2`)* | 6 uniform faces |
| **26** | `fossil_matrix` | Fossil Matrix | **Fossil** | `fossil_matrix.png` *(+ `1`, `2`)* | 6 uniform faces |
| **27** | `speleothem_blue_ice` | Blue Ice Icicle | **Speleothem** | `speleothem_blue_ice_*.png` | Modular system (**10** pieces) |
| **28** | `speleothem_calcite` | Calcite Flowstone | **Speleothem** | `speleothem_calcite_*.png` | Modular system (**10** pieces) |
| **29** | `speleothem_karst` | Karst Dripstone | **Speleothem** | `speleothem_karst_*.png` | Modular system (**10** pieces) |
| **30** | `speleothem_salt_straw` | Pink Salt Straw | **Speleothem** | `speleothem_pink_salt_straw_*.png` | Modular system (**6** pieces) |

---

## 08. Ocean & Benthic Ecosystems (`oceans`) — 50 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/oceans`  
📄 **Comprehensive Specification**: [oceans_design.md](blocks/oceans_design.md)

The benthic and subaquatic oceanic domain comprises **14 colonial massive coral blocks**, **14 branching coral fans**, **4 senescent & dead coral formations** (dying coral, dying fan, dead coral, dead fan), **10 benthic structures & biogenic substrates** (sponges, algae mats, vents, coralstone, reefstone, scutes, pearls, anemones), and **8 subaquatic pelagic plants** = **50 Total Oceanic Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping / Geometry |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `algae_mat` | Algae Mat | **Biofilm** | `aqua_algae_mat.png` *(+ `1`, `2`)* | 6 uniform faces |
| **02** | `algae_reefstone` | Algae Reefstone | **Encrusted Rock** | `aqua_reefstone.png` *(+ `algae_dead_overlay.png`)* | 6 uniform faces (Shader Overlay) |
| **03** | `barnacles_reefstone` | Barnacles Reefstone | **Crustacean Encrusted Rock** | `aqua_reefstone.png` *(+ `barnacles_overlay.png`)* | 6 uniform faces (Shader Overlay) |
| **04** | `bone_worms` | Bone Worms | **Siboglinid Annelid** | `vege_bone_worms.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **05** | `brain_coral` | Brain Coral | **Massive Coral** | `aqua_brain_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **06** | `brain_coral_fan` | Brain Coral Fan | **Coral Fan** | `vege_brain_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **07** | `bubble_coral` | Bubble Coral | **Massive Coral** | `aqua_bubble_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **08** | `bubble_coral_fan` | Bubble Coral Fan | **Coral Fan** | `vege_bubble_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **09** | `bull_kelp` | Bull Kelp | **Laminar Macroalga** | `vege_bull_kelp.png` *(Top)*<br>`vege_bull_kelp_plant.png` *(Body)* | Columnar (Multi-block) |
| **10** | `coralstone` | Coralstone | **Biogenic Rock** | `aqua_coralstone.png` *(+ `1`, `2`)* | 6 uniform faces |
| **11** | `dead_coral` | Dead Coral | **Dead Coral** | `aqua_dead_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **12** | `dead_coral_fan` | Dead Coral Fan | **Dead Coral** | `vege_dead_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **13** | `delesseria` | Delesseria | **Rhodophyte** | `vege_delesseria.png` *(Top)*<br>`vege_delesseria_plant.png` *(Body)* | Columnar (Multi-block) |
| **14** | `dying_coral` | Dying Coral | **Senescent Coral** | `aqua_dying_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **15** | `dying_coral_fan` | Dying Coral Fan | **Senescent Coral** | `vege_dying_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **16** | `fire_coral` | Fire Coral | **Massive Coral** | `aqua_fire_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **17** | `fire_coral_fan` | Fire Coral Fan | **Coral Fan** | `vege_fire_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **18** | `geothermal_vent` | Geothermal Vent | **Hydrothermal** | `aqua_geothermal_vent_top.png`<br>`aqua_geothermal_vent_side.png`<br>`aqua_geothermal_vent_bot.png` | Directional (Top / Sides / Bottom) |
| **19** | `heart_coral` | Heart Coral | **Massive Coral** | `aqua_heart_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **20** | `heart_coral_fan` | Heart Coral Fan | **Coral Fan** | `vege_heart_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **21** | `horn_coral` | Horn Coral | **Massive Coral** | `aqua_horn_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **22** | `horn_coral_fan` | Horn Coral Fan | **Coral Fan** | `vege_horn_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **23** | `kelp` | Giant Kelp | **Laminar Macroalga** | `vege_kelp.png` *(Top)*<br>`vege_kelp_plant.png` *(Body)* | Columnar (Multi-block) |
| **24** | `leaf_coral` | Leaf Coral | **Massive Coral** | `aqua_leaf_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **25** | `leaf_coral_fan` | Leaf Coral Fan | **Coral Fan** | `vege_leaf_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **26** | `pearl_block` | Pearl Block | **Biogenic Nacre** | `aqua_pearl_top.png`<br>`aqua_pearl_side.png`<br>`aqua_pearl_bot.png` | Directional (Top / Sides / Bottom) |
| **27** | `red_algae` | Red Algae | **Rhodophyte** | `vege_red_algae.png` *(+ `1`, `2`)* | Animated strip (3 variants) |
| **28** | `reed_coral` | Reed Coral | **Massive Coral** | `aqua_reed_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **29** | `reed_coral_fan` | Reed Coral Fan | **Coral Fan** | `vege_reed_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **30** | `reefstone` | Reefstone | **Volcanic Rock** | `aqua_reefstone.png` *(+ `1`, `2`)* | 6 uniform faces |
| **31** | `sargassum` | Sargassum | **Pelagic Phaeophyte** | `vege_sargassum.png` *(Top)*<br>`vege_sargassum_plant.png` *(Body)* | Columnar (Multi-block) |
| **32** | `scute_block` | Turtle Scute Block | **Biogenic Shell** | `aqua_scute_top.png`<br>`aqua_scute_side.png`<br>`aqua_scute_bot.png` | Directional (Top / Sides / Bottom) |
| **33** | `sea_anemone` | Sea Anemone | **Anthozoan** | `aqua_sea_anemone_top.png`<br>`aqua_sea_anemone_side.png`<br>`aqua_sea_anemone_bot.png` | Directional (Top / Sides / Bottom) |
| **34** | `seagrass` | Seagrass | **Marine Angiosperm** | `vege_seagrass.png` *(+ `1`, `2`)* | Animated strip (3 variants) |
| **35** | `seaweed` | Seaweed | **Littoral Macroalga** | `vege_seaweed.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **36** | `spike_coral` | Spike Coral | **Massive Coral** | `aqua_spike_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **37** | `spike_coral_fan` | Spike Coral Fan | **Coral Fan** | `vege_spike_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **38** | `sponge` | Natural Sponge | **Poriferan** | `aqua_sponge.png` *(+ `1`, `2`)* | 6 uniform faces |
| **39** | `staghorn_coral` | Staghorn Coral | **Massive Coral** | `aqua_staghorn_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **40** | `staghorn_coral_fan` | Staghorn Coral Fan | **Coral Fan** | `vege_staghorn_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **41** | `sun_coral` | Sun Coral | **Massive Coral** | `aqua_sun_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **42** | `sun_coral_fan` | Sun Coral Fan | **Coral Fan** | `vege_sun_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **43** | `thorn_coral` | Thorn Coral | **Massive Coral** | `aqua_thorn_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **44** | `thorn_coral_fan` | Thorn Coral Fan | **Coral Fan** | `vege_thorn_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **45** | `tube_coral` | Tube Coral | **Massive Coral** | `aqua_tube_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **46** | `tube_coral_fan` | Tube Coral Fan | **Coral Fan** | `vege_tube_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **47** | `wax_coral` | Wax Coral | **Massive Coral** | `aqua_wax_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **48** | `wax_coral_fan` | Wax Coral Fan | **Coral Fan** | `vege_wax_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |
| **49** | `zigzag_coral` | Zigzag Coral | **Massive Coral** | `aqua_zigzag_coral.png` *(+ `1`, `2`)* | 6 uniform faces (3 variants) |
| **50** | `zigzag_coral_fan` | Zigzag Coral Fan | **Coral Fan** | `vege_zigzag_coral_fan.png` *(+ `1`, `2`)* | Cross-billboard (3 variants) |

---

## 09. Biomaterials, Fauna & Organic Matter (`organics`) — 36 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/organics`  
📄 **Comprehensive Specification**: [organics_design.md](blocks/organics_design.md)

The biological biomaterials and organic domain comprises **14 insect colony and architecture blocks** (termites, ants, bees, wasps), **6 calcified skeletal fossil formations** (bones, skulls, ribs, tusks), and **16 visceral biological tissues, organ masses & arachnid webbings** (including bloody brain and bloody flesh) = **36 Total Organic Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `ant_hill` | Ant Hill | **Colony** | `nest_ant_hill.png` | Uniform (6 faces) |
| **02** | `ant_hill_open` | Ant Hill Open | **Colony** | `nest_ant_hill_open.png` | Uniform (6 faces) |
| **03** | `beaver_dam` | Beaver Dam | **Architecture** | `nest_beaver_dam.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **04** | `bee_nest` | Wild Bee Nest | **Hive** | `nest_bee_nest_bot...top.png`<br>`nest_bee_nest_front_honey.png` | Directional (5 faces) |
| **05** | `bird_nest` | Bird Nest | **Nest** | `nest_bird_nest_bot...top.png` | Half-height Slab |
| **06** | `bloody_brain` | Bloody Brain Block | **Neural** | `bio_brain.png` *(+ `bloody_overlay.png`, `1`, `2`)* | Uniform (6 faces) + Shader Bloody Overlay |
| **07** | `bloody_flesh` | Bloody Flesh Block | **Tissue** | Top/Bot: `bio_flesh_top.png`<br>Sides: `bio_flesh_side.png` *(+ `bloody_overlay.png`, `1`, `2`)* | Directional (2 faces) + Lateral Shader Bloody Overlay |
| **08** | `blubber_block` | Blubber | **Adipose** | `bio_blubber.png` *(+ `1`, `2`)* | Translucent (Alpha 204) |
| **09** | `bone_block` | Bone | **Osteology** | `bio_bone_top.png`<br>`bio_bone_side.png` *(+ `1`, `2`)* | Axial Column (2 faces) |
| **10** | `brain_block` | Brain Block | **Neural** | `bio_brain.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **11** | `calcined_bone` | Calcined Bone | **Osteology** | `bio_calcined_bone_top.png`<br>`bio_calcined_bone_side.png` *(+ `1`, `2`)* | Axial Column (2 faces) |
| **12** | `cobweb` | Cobweb | **Silk** | `organic_cobweb.png` *(+ `1`, `2`)* | Volumetric 3D Cross |
| **13** | `cobweb_arch` | Cobweb Arch | **Silk** | `organic_cobweb_arch.png` *(+ `1`, `2`)* | Flat Vertical Quad |
| **14** | `cobweb_hanging` | Hanging Cobweb | **Silk** | `organic_cobweb_hanging.png` *(+ `1`, `2`)* | Flat Vertical Quad |
| **15** | `cobweb_stretched` | Stretched Cobweb | **Silk** | `organic_cobweb_stretched.png` *(+ `1`, `2`)* | Flat Vertical Quad |
| **16** | `coprolite_block` | Coprolite | **Fossil** | `organic_coprolite.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **17** | `dust_block` | Dust | **Detritus** | `organic_dust.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **18** | `flesh_block` | Raw Flesh Block | **Tissue** | `bio_flesh_top.png`<br>`bio_flesh_side.png` *(+ `1`, `2`)* | Directional (2 faces) |
| **19** | `fur_block` | Animal Fur | **Integument** | `bio_fur_top.png`<br>`bio_fur_side.png`<br>`bio_fur_bot.png` | Directional (3 faces) |
| **20** | `guano_block` | Guano | **Dropping** | `organic_guano.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **21** | `hay_nest` | Hay Nest | **Nest** | `nest_hay_nest_bot...top.png` | Half-height Slab |
| **22** | `honey_block` | Honey Block | **Secretion** | `organic_honey_block_bot...top.png` | Translucent (3 faces) |
| **23** | `honeycomb_block` | Honeycomb | **Wax Matrix** | `organic_honeycomb_block.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **24** | `jelly_block` | Royal Jelly Block | **Colloid** | `organic_jelly_top.png`<br>`organic_jelly_side.png`<br>`organic_jelly_bot.png` | Translucent (3 faces) |
| **25** | `loam_burrow` | Loam Animal Burrow | **Burrow** | Top: `nest_burrow_top.png` + `layer.png` *(+ `1`, `2`)*<br>Bot: `soils/soil_loam_dirt.png`<br>Sides: `soils/soil_loam_grass_side.png` | Composite Top + Loam soil |
| **26** | `peat_burrow` | Peat Animal Burrow | **Burrow** | Top: `nest_burrow_top.png` + `layer.png` *(+ `1`, `2`)*<br>Bot: `soils/soil_peat_dirt.png`<br>Sides: `soils/soil_peat_grass_side.png` | Composite Top + Peat soil |
| **27** | `poop_block` | Poop | **Dropping** | `organic_poop.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **28** | `seashell_block` | Seashell Block | **Sediment** | `organic_seashell.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **29** | `silk_cocoon` | Wild Silk Cocoon | **Pupa** | `nest_silk_cocoon_bot...top.png` | Directional (3 faces) |
| **30** | `silt_burrow` | Silt Animal Burrow | **Burrow** | Top: `nest_burrow_top.png` + `layer.png` *(+ `1`, `2`)*<br>Bot: `soils/soil_silt_dirt.png`<br>Sides: `soils/soil_silt_grass_side.png` | Composite Top + Silt soil |
| **31** | `slime_block` | Slime Block | **Colloid** | `organic_slime_top.png`<br>`organic_slime_side.png`<br>`organic_slime_bot.png` | Translucent (3 faces) |
| **32** | `spider_egg` | Spider Egg Sac | **Ootheca** | `nest_spider_egg_bot...top.png` | Directional (3 faces) |
| **33** | `termite_mound` | Termite Mound | **Colony** | `nest_termite_mound.png` | Uniform (6 faces) |
| **34** | `termite_mound_open` | Termite Mound Open | **Colony** | `nest_termite_mound_open.png` | Uniform (6 faces) |
| **35** | `wasp_nest` | Wild Wasp Nest | **Hive** | `nest_wasp_nest_bot...top.png` | Directional (3 faces) |
| **36** | `wax_block` | Beeswax | **Wax** | `organic_wax.png` *(+ `1`, `2`)* | Uniform (6 faces) |

---

## 10. Decorations, Lighting & Architecture (`decorations`) — 162 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/decorations`  
📄 **Comprehensive Specification**: [decorations_design.md](blocks/decorations_design.md)

The architectural, utilitarian, and illumination domain comprises **6 utilitarian containers & structural furniture blocks**, **8 traditional pure masonry, textile, and glass blocks**, **128 dyed finishes in 16 colors** via shader tinting ($8 \times 16$), **8 agricultural fibers & rural pathways**, and **12 radiant emitters & torches** = **162 Total Architectural Blocks**.

> **16 Standard Dye Colors (IDs 1 to 16):**  
> `01. black` · `02. blue` · `03. brown` · `04. dark_blue` · `05. dark_grey` · `06. green` · `07. light_grey` · `08. light_pink` · `09. lime` · `10. orange` · `11. pink` · `12. purple` · `13. red` · `14. turquoise` · `15. white` · `16. yellow`

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **001** | `bag` | Fabric Bag | **Utility** | `util_bag_top.png`<br>`util_bag_side.png`<br>`util_bag_side_front.png`<br>`util_bag_bot.png` | Top top + Bottom bot + 3 sides side + 1 front `util_bag_side_front.png` |
| **002** | `barrel` | Barrel | **Utility** | `util_barrel_top.png`<br>`util_barrel_side.png`<br>`util_barrel_bot.png` | Top top + Bottom bot + 4 sides side |
| **003** | `basket` | Basket | **Utility** | `util_basket_top.png`<br>`util_basket_side.png`<br>`util_basket_bot.png` | Top top + Bottom bot + 4 sides side |
| **004** | `bookshelf` | Bookshelf | **Utility** | `util_bookshelf_top.png`<br>`util_bookshelf_side.png`<br>`util_bookshelf_front.png` | Top & Bottom top + 3 sides side + 1 front `util_bookshelf_front.png` |
| **005** | `bookshelf_empty` | Empty Bookshelf | **Utility** | `util_bookshelf_top.png`<br>`util_bookshelf_side.png`<br>`util_bookshelf_front_empty.png` | Top & Bottom top + 3 sides side + 1 front `util_bookshelf_front_empty.png` |
| **006–021** | `brick_<color>` | (Color) Bricks *(16 colors)* | **Masonry** | `deco_bricks_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **022** | `bricks` | Bricks | **Masonry** | `deco_bricks.png` | 6 uniform faces |
| **023** | `corn_bale` | Corn Bale | **Fibers** | `deco_corn_bale_top.png`<br>`deco_corn_bale_side.png`<br>`deco_corn_bale_bottom.png` | Top top + Bottom bottom + 4 sides side *(Orientable)* |
| **024** | `crate` | Empty Crate | **Utility** | `util_crate_top.png`<br>`util_crate_side.png`<br>`util_crate_bot.png` | Top top + Bottom bot + 4 sides side |
| **025** | `emit_lamp` | Lamp | **Lighting** | `emit_lamp.png`<br>`emit_lamp_top.png`<br>`emit_lamp_on.png`<br>`emit_lamp_top_on.png` | Top top + 4 sides + Bottom (`emit_lamp.png`) *(2 states: Normal / On)* |
| **026** | `emit_light_blue` | Blue Light | **Radiant Emitter** | `emit_light_blue.png` | 6 uniform faces |
| **027** | `emit_light_cold` | Cold Light | **Radiant Emitter** | `emit_light_cold.png` | 6 uniform faces |
| **028** | `emit_light_green` | Green Light | **Radiant Emitter** | `emit_light_green.png` | 6 uniform faces |
| **029** | `emit_light_red` | Red Light | **Radiant Emitter** | `emit_light_red.png` | 6 uniform faces |
| **030** | `emit_light_warm` | Warm Light | **Radiant Emitter** | `emit_light_warm.png` | 6 uniform faces |
| **031** | `emit_torch_blue` | Blue Torch | **Lighting** | `emit_torch_blue.png` | Vertical stem model *(Sapphire blue diode)* |
| **032** | `emit_torch_cold` | Cold Torch | **Lighting** | `emit_torch_cold.png` | Vertical stem model *(Cold light 6500K)* |
| **033** | `emit_torch_fire` | Fire Torch | **Lighting** | `emit_torch_fire.png` | Vertical stem model *(5 animated fire frames)* |
| **034** | `emit_torch_green` | Green Torch | **Lighting** | `emit_torch_green.png` | Vertical stem model *(Emerald green diode)* |
| **035** | `emit_torch_red` | Red Torch | **Lighting** | `emit_torch_red.png` | Vertical stem model *(Ruby red diode)* |
| **036** | `emit_torch_warm` | Warm Torch | **Lighting** | `emit_torch_warm.png` | Vertical stem model *(Warm light 2700K)* |
| **037** | `fabric` | Fabric | **Textile** | `deco_fabric.png` *(+ `1`, `2`)* | 6 uniform faces |
| **038–053** | `fabric_<color>` | (Color) Fabric *(16 colors)* | **Textile** | `deco_fabric_painted.png` *(+ `1`, `2`)* | 6 uniform faces (Shader Tint: colors 01–16) |
| **054** | `glass` | Glass | **Glasswork** | `deco_glass.png` | 6 uniform faces (Clear transparent) |
| **055–070** | `glass_<color>` | (Color) Glass *(16 colors)* | **Glasswork** | `deco_glass_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **071** | `hay_bale` | Hay Bale | **Fibers** | `deco_hay_bale_top.png`<br>`deco_hay_bale_side.png` | Top & Bottom top + 4 sides side *(Orientable)* |
| **072** | `linen` | Linen | **Textile** | `deco_linen.png` | 6 uniform faces |
| **073–088** | `linen_<color>` | (Color) Linen *(16 colors)* | **Textile** | `deco_linen_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **089** | `path_dirty` | Dirty Path | **Paths** | `deco_path_dirty.png` | 6 uniform faces |
| **090** | `path_sandy` | Sandy Path | **Paths** | `deco_path_sandy.png` | 6 uniform faces |
| **091** | `path_snowy` | Snowy Path | **Paths** | `deco_path_snowy.png` | 6 uniform faces |
| **092** | `path_stony` | Stony Path | **Paths** | `deco_path_stony.png` | 6 uniform faces |
| **093** | `plaster` | Plaster | **Masonry** | `deco_plaster.png` | 6 uniform faces |
| **094–109** | `plaster_<color>` | (Color) Plaster *(16 colors)* | **Masonry** | `deco_plaster_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **110** | `rice_bale` | Rice Bale | **Fibers** | `deco_rice_bale_top.png`<br>`deco_rice_bale_side.png`<br>`deco_rice_bale_bottom.png` | Top top + Bottom bottom + 4 sides side *(Orientable)* |
| **111** | `shingles` | Shingles | **Roofing** | `deco_shingles.png` | 6 uniform faces |
| **112–127** | `shingles_<color>` | (Color) Shingles *(16 colors)* | **Roofing** | `deco_shingles_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **128** | `straw_bale` | Straw Bale | **Fibers** | `deco_straw_bale_top.png`<br>`deco_straw_bale_side.png` | Top & Bottom top + 4 sides side *(Orientable)* |
| **129** | `tinted_glass` | Tinted Glass | **Glasswork** | `deco_tinted_glass.png` | 6 uniform faces (Smoked semi-translucent) |
| **130–145** | `tinted_glass_<color>` | (Color) Tinted Glass *(16 colors)* | **Glasswork** | `deco_tinted_glass_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **146** | `wool` | Wool | **Textile** | `deco_wool.png` | 6 uniform faces |
| **147–162** | `wool_<color>` | (Color) Wool *(16 colors)* | **Textile** | `deco_wool_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |

---

## 11. Mycology: Mushrooms (`mushrooms`) — 18 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/mushrooms`  
📄 **Comprehensive Specification**: [mushrooms_design.md](blocks/mushrooms_design.md)

The mycological domain encompasses **8 wild understory ground mushrooms** (cross-billboards with procedural variations), **8 giant macrofungal caps** (`top`/`side` cap with `bottom` porous gills), and **2 shared macrofungal structural blocks** (`stem` stipe column and `inside` flesh) = **18 Total Mycological Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `fungi_amanita` | Amanita | **Ground Mushroom** | `fungi_amanita.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **02** | `fungi_amanita_cap` | Amanita Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_amanita_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **03** | `fungi_chanterelle` | Chanterelle | **Ground Mushroom** | `fungi_chanterelle.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **04** | `fungi_chanterelle_cap` | Chanterelle Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_chanterelle_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **05** | `fungi_indigo` | Indigo | **Ground Mushroom** | `fungi_indigo.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **06** | `fungi_indigo_cap` | Indigo Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_indigo_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **07** | `fungi_inside` | Fungi Inside Flesh | **Macrofungal Structure** | `fungi_inside.png` *(+ `1`, `2`)* | 6 uniform faces |
| **08** | `fungi_mycena` | Mycena | **Ground Mushroom** | `fungi_mycena.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **09** | `fungi_mycena_cap` | Mycena Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_mycena_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **10** | `fungi_porcini` | Porcini | **Ground Mushroom** | `fungi_porcini.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **11** | `fungi_porcini_cap` | Porcini Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_porcini_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **12** | `fungi_russula` | Russula | **Ground Mushroom** | `fungi_russula.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **13** | `fungi_russula_cap` | Russula Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_russula_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **14** | `fungi_shiitake` | Shiitake | **Ground Mushroom** | `fungi_shiitake.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **15** | `fungi_shiitake_cap` | Shiitake Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_shiitake_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **16** | `fungi_stem` | Fungi Stem | **Macrofungal Structure** | `fungi_stem.png` *(+ `1`, `2`)* | 6 uniform faces |
| **17** | `fungi_verdigris` | Verdigris | **Ground Mushroom** | `fungi_verdigris.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **18** | `fungi_verdigris_cap` | Verdigris Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_verdigris_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |

---

## 12. Botany: Groundcover & Herbaceous Flora (`vegetation`) — 26 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/vegetation`  
📄 **Comprehensive Specification**: [vegetation_design.md](blocks/vegetation_design.md)

The planetary herbaceous flora and groundcover system encompasses **26 distinct herbaceous, shrub, fern, epiphyte, and cavern taxa** spanning 6 functional ecological divisions (each with strictly 3 procedural variants): 6 native grasses, 4 understory ferns, 6 shrubs & cacti, 2 wetland plants, 4 epiphytes, and 4 subterranean cavern flora = **26 Total Groundcover Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping / Geometry |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `vege_barrel_cactus` | Barrel Cactus | **Cactus** | `vege_barrel_cactus.png` *(+ `1`, `2`)* | Warm Deserts, Scrub & Chaparrals (Spiny Succulent Mesh) |
| **02** | `vege_bluff_grass` | Bluff Grass | **Grass** | `vege_bluff_grass.png` *(+ `1`, `2`)* | Single Cross-Mesh (Rocky Crags & Cliffs) |
| **03** | `vege_bracken` | Bracken Fern | **Fern** | `vege_bracken.png` *(+ `1`, `2`)* | Spreading Frond Mesh (Moorlands & Open Woods) |
| **04** | `vege_bush` | Temperate Bush | **Shrub** | `vege_bush.png` *(+ `1`, `2`)* | Dense Rounded Mesh (Forest Edges & Meadows) |
| **05** | `vege_cactus` | Saguaro Cactus | **Cactus** | `vege_cactus_top.png`<br>`vege_cactus_side.png`<br>`vege_cactus_bottom.png` | 3D Full Column Block (3 sides, 3 tops, 1 bottom) |
| **06** | `vege_cattail` | Cattail | **Wetland** | `vege_cattail_bottom.png`<br>`vege_cattail_top.png` | Double-Tall Emergent (3 pairs) |
| **07** | `vege_cave_growths` | Cave Growths | **Cave Flora** | `vege_cave_growths.png` *(+ `1`, `2`)* | Ground Umbrophilic Thallus (Subterranean Caverns) |
| **08** | `vege_cave_hyssop` | Cave Hyssop | **Cave Flora** | `vege_cave_hyssop.png` *(+ `1`, `2`)* | Subterranean Flowering Herb (Geodes & Crystals) |
| **09** | `vege_cave_vines` | Cave Vines | **Cave Flora** | `vege_cave_vines.png`<br>`vege_cave_vines_plant.png` | Ceiling Hanging Multi-Block Vine (3 tip, 3 plant) |
| **10** | `vege_dead_bush` | Dead Bush | **Shrub** | `vege_dead_bush.png` *(+ `1`, `2`)* | Withered Woody Sticks (Arid Deserts & Badlands) |
| **11** | `vege_drygrass` | Drygrass | **Grass** | `vege_drygrass.png` *(+ `1`, `2`)* | Single Cross-Mesh (Savannas & Scrublands) |
| **12** | `vege_fern` | Common Fern | **Fern** | `vege_fern.png` *(+ `1`, `2`)* | Single Cross-Mesh (Shaded Woodlands & Taigas) |
| **13** | `vege_grainy_cave_growths` | Grainy Cave Growths | **Cave Flora** | `vege_grainy_cave_growths.png` *(+ `1`, `2`)* | Ferruginous Granular Crust (Subterranean Walls) |
| **14** | `vege_hanging_moss` | Hanging Moss | **Epiphyte** | `vege_hanging_moss.png` *(+ `1`, `2`)* | Ceiling Hanging Decal (Swamps & Gallery Forests) |
| **15** | `vege_hanging_roots` | Hanging Roots | **Epiphyte** | `vege_hanging_roots.png` *(+ `1`, `2`)* | Ceiling Root Danglers (Overhangs & Ravines) |
| **16** | `vege_lichen` | Tree & Rock Lichen | **Epiphyte** | `vege_lichen.png` *(+ `1`, `2`)* | Crustose Surface Patch (Barks, Rocks & Boulders) |
| **17** | `vege_lily_pad` | Water Lily Pad | **Floating Pad** | `vege_lily_pad.png` *(+ `1`, `2`)* | Floating Water Surface Mesh (Lakes & Swamps) |
| **18** | `vege_mossgrass` | Mossgrass | **Grass** | `vege_mossgrass.png` *(+ `1`, `2`)* | Low Dense Tuft (Damp Woodlands & River Glades) |
| **19** | `vege_prickly_pear_cactus` | Prickly Pear Cactus | **Cactus** | `vege_prickly_pear_cactus.png` *(+ `1`, `2`)* | Fleshy Pad Cross-Mesh (Semiarid Slopes & Scrub) |
| **20** | `vege_red_shrub` | Red Shrub | **Shrub** | `vege_red_shrub.png` *(+ `1`, `2`)* | Red Pigmented Mesh (Autumn Woods & Moors) |
| **21** | `vege_sea_oats` | Sea Oats | **Tall Grass** | `vege_sea_oats_bottom.png`<br>`vege_sea_oats_top.png` | Double-Tall Dune Grass (3 pairs) |
| **22** | `vege_tall_fern` | Tall Fern | **Tall Fern** | `vege_tall_fern_bottom.png`<br>`vege_tall_fern_top.png` | Double-Tall Arborescent Fern (3 pairs) |
| **23** | `vege_tall_jungle_fern` | Tall Jungle Fern | **Tall Fern** | `vege_tall_jungle_fern_bottom.png`<br>`vege_tall_jungle_fern_top.png` | Double-Tall Jungle Fern (3 pairs) |
| **24** | `vege_tall_wildgrass` | Tall Wildgrass | **Tall Grass** | `vege_tall_wildgrass_bottom.png`<br>`vege_tall_wildgrass_top.png` | Double-Tall Prairie Grass (3 pairs) |
| **25** | `vege_vines` | Vines (Lianas) | **Epiphyte** | `vege_vines.png` *(+ `1`, `2`)* | Vertical Wall Climber (Canopies & Cliff Walls) |
| **26** | `vege_wildgrass` | Wildgrass | **Grass** | `vege_wildgrass.png` *(+ `1`, `2`)* | Single Cross-Mesh (Plains, Meadows & Forests) |

---

## 13. Botany: Flowers & Flowering Flora (`flowers`) — 28 Blocks [✅ 100% Complete]

📂 **Domain Directory**: `docs/worldbuilding/blocks/flowers`  
📄 **Comprehensive Specification**: [flowers_design.md](blocks/flowers_design.md)

The angiosperm flowering flora domain comprises **28 distinct flowering plant species** distributed across temperate meadows, alpine glades, wetlands, gardens, arid deserts, and forest groundcovers: 12 wild meadow & woodland flowers, 2 tulip cultivars, 8 double-tall shrubs/flowers, 4 groundcover carpets, and 2 arid desert blooms = **28 Total Flowering Blocks** (113 active textures, 100% standardized with 3 variants per block/part).

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping / Geometry |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `flower_allium` | Allium | **Wildflower** | `flower_allium.png` to `2` | Cross-billboard (3 variants) |
| **02** | `flower_blue_hydrangea` | Blue Hydrangea | **Double-Tall** | `flower_blue_hydrangea_bottom.png` to `2`<br>`flower_blue_hydrangea_top.png` to `2` | Double-Tall (2 Blocks, 3 variants) |
| **03** | `flower_blue_orchid` | Blue Orchid | **Wildflower** | `flower_blue_orchid.png` to `2` | Cross-billboard (3 variants) |
| **04** | `flower_cactus_flower` | Cactus Flower | **Arid Flora** | `flower_cactus_flower.png` to `2` | Single Cross / Perch (3 variants) |
| **05** | `flower_clover` | Clover Carpet | **Carpet** | `flower_clover.png` to `2`<br>`flower_carpet_stem.png` to `2` | Flat Quad + Central Upright Stem |
| **06** | `flower_cornflower` | Cornflower | **Wildflower** | `flower_cornflower.png` to `2` | Cross-billboard (3 variants) |
| **07** | `flower_dandelion` | Dandelion | **Wildflower** | `flower_dandelion.png` to `2` | Cross-billboard (3 variants) |
| **08** | `flower_foxglove` | Foxglove | **Double-Tall** | `flower_foxglove_bottom.png` to `2`<br>`flower_foxglove_top.png` to `2` | Double-Tall (2 Blocks, 3 variants) |
| **09** | `flower_gardenia` | Gardenia | **Double-Tall** | `flower_gardenia_bottom.png` to `2`<br>`flower_gardenia_top.png` to `2` | Double-Tall (2 Blocks, 3 variants) |
| **10** | `flower_globe_mallow` | Globe Mallow | **Arid Flora** | `flower_globe_mallow.png` to `2` | Single Cross (3 variants) |
| **11** | `flower_hibiscus` | Hibiscus | **Wildflower** | `flower_hibiscus.png` to `2` | Cross-billboard (3 variants) |
| **12** | `flower_iris` | Iris | **Wildflower** | `flower_iris.png` to `2` | Cross-billboard (3 variants) |
| **13** | `flower_lavender` | Lavender | **Wildflower** | `flower_lavender.png` to `2` | Cross-billboard (3 variants) |
| **14** | `flower_lilac` | Lilac | **Double-Tall** | `flower_lilac_bottom.png` to `2`<br>`flower_lilac_top.png` to `2` | Double-Tall (2 Blocks, 3 variants) |
| **15** | `flower_lily_of_the_valley` | Lily of the Valley | **Wildflower** | `flower_lily_of_the_valley.png` to `2` | Cross-billboard (3 variants) |
| **16** | `flower_marigold` | Marigold | **Wildflower** | `flower_marigold.png` to `2` | Cross-billboard (3 variants) |
| **17** | `flower_orange_tulip` | Orange Tulip | **Tulip** | `flower_orange_tulip.png` to `2` | Cross-billboard (3 variants) |
| **18** | `flower_orchid` | Classic Orchid | **Wildflower** | `flower_orchid.png` to `2` | Cross-billboard (3 variants) |
| **19** | `flower_oxeye_daisy` | Oxeye Daisy | **Wildflower** | `flower_oxeye_daisy.png` to `2` | Cross-billboard (3 variants) |
| **20** | `flower_peony` | Peony | **Double-Tall** | `flower_peony_bottom.png` to `2`<br>`flower_peony_top.png` to `2` | Double-Tall (2 Blocks, 3 variants) |
| **21** | `flower_pink_petals` | Pink Petals | **Carpet** | `flower_pink_petals.png` to `2`<br>`flower_carpet_stem.png` to `2` | Flat Quad + Central Upright Stem |
| **22** | `flower_pink_tulip` | Pink Tulip | **Tulip** | `flower_pink_tulip.png` to `2` | Cross-billboard (3 variants) |
| **23** | `flower_poppy` | Poppy | **Wildflower** | `flower_poppy.png` to `2` | Cross-billboard (3 variants) |
| **24** | `flower_rose_bush` | Rose Bush | **Double-Tall** | `flower_rose_bush_bottom.png` to `2`<br>`flower_rose_bush_top.png` to `2` | Double-Tall (2 Blocks, 3 variants) |
| **25** | `flower_snapdragon` | Snapdragon | **Double-Tall** | `flower_snapdragon_bottom.png` to `2`<br>`flower_snapdragon_top.png` to `2` | Double-Tall (2 Blocks, 3 variants) |
| **26** | `flower_sunflower` | Sunflower | **Double-Tall** | `flower_sunflower_bottom.png` to `2`<br>`flower_sunflower_top.png` to `2`<br>`flower_sunflower_front.png`, `back.png` | Double-Tall (Heliotropic Head, 3 variants) |
| **27** | `flower_white_petals` | White Petals | **Carpet** | `flower_white_petals.png` to `2`<br>`flower_carpet_stem.png` to `2` | Flat Quad + Central Upright Stem |
| **28** | `flower_wildflowers` | Wildflowers Carpet | **Carpet** | `flower_wildflowers.png` to `2`<br>`flower_carpet_stem.png` to `2` | Flat Quad + Central Upright Stem |

---

## 14. Agriculture & Domestic Crops (`crops`) — 36 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/crops`  
📄 **Comprehensive Specification**: [crops_design.md](blocks/crops_design.md)

The agricultural crop domain comprises **36 cultivated food crops, arboreal pods, and wild ancestral cultivars** (spanning 16 farmland & bush crops, 2 arboreal canopy/trunk pods, and 18 wild ancestral harvestable crops) featuring multi-stage developmental growth lifecycles = **36 Total Domestic & Wild Crop Blocks**.

---

| # | Block ID | Name | Category | Texture File(s) | Face Mapping / Lifecycle |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `crop_beetroots` | Beetroot Crop | **Taproot** | `crops_beetroots_stage0...3.png` | 4 Stages (0–3) |
| **02** | `crop_bell_peppers` | Bell Pepper Crop | **Solanaceae** | `crops_bell_peppers_stage0...7.png` | 8 Stages (0–7) |
| **03** | `crop_cabbages` | Cabbage Crop | **Brassica** | `crops_cabbages_stage0...7.png` | 8 Stages (0–7) |
| **04** | `crop_carrots` | Carrot Crop | **Root Crop** | `crops_carrots_stage0...3.png` | 4 Stages (0–3) |
| **05** | `crop_chili_pepper` | Chili Pepper Crop | **Solanaceae** | `crops_chili_pepper_stage0...7.png` | 8 Stages (0–7) |
| **06** | `crop_cocoa` | Cocoa Pod | **Arboreal Pod** | `crops_cocoa_stage0...2.png` | 3 Stages (0–2) |
| **07** | `crop_coconut` | Hanging Coconut | **Arboreal Drupe** | `crops_coconut_hanging_stage1...3.png` | 3 Stages (1–3) |
| **08** | `crop_coffee` | Coffee Crop | **Beverage Shrub** | Budding: `crops_coffee_budding_stage0...3.png`<br>Bottom: `crops_coffee_bottom_stage0...3.png`<br>Top: `crops_coffee_top_stage0...3.png` | Double-Tall Shrub (4 Budding + 4 Lower + 4 Upper Stages) |
| **09** | `crop_corn` | Corn Crop | **Cereal** | Budding: `crops_corn_budding_stage0...3.png`<br>Bottom: `crops_corn_bottom_stage0...3.png`<br>Top: `crops_corn_top_stage0...3.png` | Double-Tall Stalk (4 Budding + 4 Lower + 4 Upper Stages) |
| **10** | `crop_garlic` | Garlic Crop | **Bulb** | `crops_garlic_stage0...7.png` | 8 Stages (0–7) |
| **11** | `crop_lettuce` | Lettuce Crop | **Leafy Vegetable** | `crops_lettuce_stage0...4.png` | 5 Stages (0–4) |
| **12** | `crop_onions` | Onion Crop | **Bulb** | `crops_onions_stage0...3.png` | 4 Stages (0–3) |
| **13** | `crop_parsnips` | Parsnip Crop | **Root Crop** | `crops_parsnips_stage0...3.png` | 4 Stages (0–3) |
| **14** | `crop_potatoes` | Potato Crop | **Tuber** | `crops_potatoes_stage0...3.png` | 4 Stages (0–3) |
| **15** | `crop_rice` | Rice Crop | **Cereal** | Bottom: `crops_rice_stage0...3.png` *(+ `supporting`)*<br>Top: `crops_rice_panicles_stage0...3.png` | Double-Tall (4 Lower + Supporting + 4 Upper Stages) |
| **16** | `crop_sweet_potato` | Sweet Potato Crop | **Tuber** | `crops_sweet_potato_stage0...7.png` | 8 Stages (0–7) |
| **17** | `crop_tomatoes` | Tomato Crop | **Climbing Vine** | Bottom: `crops_budding_tomatoes_stage0...3.png`<br>Top: `crops_tomatoes_stage0...3.png` | Double-Tall (4 Lower + 4 Upper Stages) |
| **18** | `crop_wheat` | Wheat Crop | **Cereal** | `crops_wheat_stage0...5.png` | 6 Stages (0–5) |
| **19** | `wild_beetroots` | Wild Beetroots | **Wild Ancestor** | `wild_beetroots.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **20** | `wild_bell_peppers` | Wild Bell Peppers | **Wild Ancestor** | `wild_bell_peppers.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **21** | `wild_cabbages` | Wild Cabbages | **Wild Ancestor** | `wild_cabbages.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **22** | `wild_carrots` | Wild Carrots | **Wild Ancestor** | `wild_carrots.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **23** | `wild_chili_pepper` | Wild Chili Pepper | **Wild Ancestor** | `wild_chili_pepper.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **24** | `wild_coffee` | Wild Coffee | **Wild Ancestor** | `wild_coffee.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **25** | `wild_corn` | Wild Corn | **Wild Ancestor** | Bottom: `wild_corn_bottom.png` *(+ `1`, `2`)*<br>Top: `wild_corn_top.png` *(+ `1`, `2`)* | Double-Tall Cross-billboard (Harvestable Forb, 3 Variants) |
| **26** | `wild_garlic` | Wild Garlic | **Wild Ancestor** | `wild_garlic.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **27** | `wild_lettuce` | Wild Lettuce | **Wild Ancestor** | `wild_lettuce.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **28** | `wild_onions` | Wild Onions | **Wild Ancestor** | `wild_onions.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **29** | `wild_parsnips` | Wild Parsnips | **Wild Ancestor** | `wild_parsnips.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **30** | `wild_potatoes` | Wild Potatoes | **Wild Ancestor** | `wild_potatoes.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **31** | `wild_rice` | Wild Rice | **Wild Ancestor** | Bottom: `wild_rice_bottom.png`<br>Top: `wild_rice_top.png` | Double-Tall Cross-billboard (Harvestable Forb) |
| **32** | `wild_sweet_potato` | Wild Sweet Potato | **Wild Ancestor** | `wild_sweet_potato.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **33** | `wild_tomatoes` | Wild Tomatoes | **Wild Ancestor** | `wild_tomatoes.png` *(+ `1`, `2`)* | Cross-billboard (Harvestable Forb, 3 Variants) |
| **34** | `wild_wheat` | Wild Wheat | **Wild Ancestor** | Bottom: `wild_wheat_bottom.png` *(+ `1`, `2`)*<br>Top: `wild_wheat_top.png` *(+ `1`, `2`)* | Double-Tall Cross-billboard (Harvestable Grain, 3 Variants) |
