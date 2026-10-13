# Voxel Game Engine Master Block Catalog (Blocks Design)

This master document centralizes, unifies, and catalogs **all blocks implemented in the Voxel Engine**, organized across the 13 worldbuilding subdomains located in `docs/worldbuilding/blocks/`.

> [!IMPORTANT]
> **Project Grand Total: 762 Cataloged Unique Blocks**  
> Summing all 13 core domains of the engine's block ecosystem:  
> **76** (rocks) + **58** (soils) + **98** (trees) + **158** (ores) + **34** (organics) + **160** (decorations) + **32** (flowers) + **38** (vegetation) + **18** (mushrooms) + **18** (fluids) + **36** (oceans) + **28** (caverns) + **8** (systems) = **762 Total Blocks**.

For each block, the following properties are systematically specified:
1. **Unique Block Identifier (`Block ID`)**.
2. **Canonical Name and Functional/Ecological Role**.
3. **Face Texture Mapping** (`top`, `bottom` / `bot`, `sides`, `front`, etc.).
4. **Compositional Layers, Shader Dynamic Tinting & Procedural Overlays**.

> [!NOTE]
> In accordance with project architectural guidelines:
> - **Stochastic / procedural variations** (e.g. suffixes `1`, `2`, `3`) are dynamically sampled by shaders to break visual tiling repetition across terrain and are **not** registered as duplicate blocks.
> - **Non-autonomous decal/cosmetic overlays** (such as `lichen`, `bloody`, `slimy`, and `moss_*_side`) remain dynamic multi-texturing shader passes.
> - **Integrated compositional variants** (such as `mossy_rock`, `mossy_cobbled`, `mossy_tree`, `algae_overlay`, snow caps, and flower overlays) generate distinct registered engine blocks cataloged below.

---

## Master Summary & Block Census

| Category | Domain / Directory | Block Count | Primary Architectural Specification | Navigation Link |
| :---: | :--- | :---: | :--- | :--- |
| **01** | Geology & Rocks (`rocks`) | **76 blocks** | [rocks_design.md](blocks/rocks_design.md) | [Go to Section 01](#01-geology--rocks-rocks--76-blocks) |
| **02** | Pedology & Soils (`soils`) | **58 blocks** | [soils_design.md](blocks/soils_design.md) | [Go to Section 02](#02-pedology--soils-soils--58-blocks) |
| **03** | Trees, Forestry & Dendrology (`trees`) | **98 blocks** | [trees_design.md](blocks/trees_design.md) | [Go to Section 03](#03-trees-forestry--dendrology-trees--98-blocks) |
| **04** | Ores, Minerals & Gemology (`ores`) | **158 blocks** | [ores_design.md](blocks/ores_design.md) | [Go to Section 04](#04-ores-minerals--gemology-ores--158-blocks) |
| **05** | Biomaterials & Organics (`organics`) | **34 blocks** | [organics_design.md](blocks/organics_design.md) | [Go to Section 05](#05-biomaterials--organics-organics--34-blocks) |
| **06** | Decorations, Lighting & Architecture (`decorations`) | **160 blocks** | [decorations_design.md](blocks/decorations_design.md) | [Go to Section 06](#06-decorations-lighting--architecture-decorations--160-blocks) |
| **07** | Botany: Flowers (`flowers`) | **32 blocks** | [flowers_design.md](blocks/flowers_design.md) | [Go to Section 07](#07-botany-flowers-flowers--32-blocks) |
| **08** | Botany: Groundcover & Vegetation (`vegetation`) | **38 blocks** | [vegetation_design.md](blocks/vegetation_design.md) | [Go to Section 08](#08-botany-groundcover--vegetation-vegetation--38-blocks) |
| **09** | Mycology: Mushrooms (`mushrooms`) | **18 blocks** | [mushrooms_design.md](blocks/mushrooms_design.md) | [Go to Section 09](#09-mycology-mushrooms-mushrooms--18-blocks) |
| **10** | Hydrosphere & Cryology (`fluids`) | **18 blocks** | [fluids_design.md](blocks/fluids_design.md) | [Go to Section 10](#10-hydrosphere--cryology-fluids--18-blocks) |
| **11** | Ocean & Benthic Ecosystems (`oceans`) | **36 blocks** | [oceans_design.md](blocks/oceans_design.md) | [Go to Section 11](#11-ocean--benthic-ecosystems-oceans--36-blocks) |
| **12** | Speleology & Crystals (`caverns`) | **28 blocks** | [caverns_design.md](blocks/caverns_design.md) | [Go to Section 12](#12-speleology--crystals-caverns--28-blocks) |
| **13** | Technical & System Blocks (`systems`) | **8 blocks** | [systems_design.md](blocks/systems_design.md) | [Go to Section 13](#13-technical--system-blocks-systems--8-blocks) |
| **TOTAL** | **Grand Master Sum** | **762 blocks** | — | — |

01. [Geology & Rocks (`rocks`) — 76 Blocks](#01-geology--rocks-rocks--76-blocks)
02. [Pedology & Soils (`soils`) — 58 Blocks](#02-pedology--soils-soils--58-blocks)
03. [Trees, Forestry & Dendrology (`trees`) — 98 Blocks](#03-trees-forestry--dendrology-trees--98-blocks)
04. [Ores, Minerals & Gemology (`ores`) — 158 Blocks](#04-ores-minerals--gemology-ores--158-blocks)
05. [Biomaterials & Organics (`organics`) — 34 Blocks](#05-biomaterials--organics-organics--34-blocks)
06. [Decorations, Lighting & Architecture (`decorations`) — 160 Blocks](#06-decorations-lighting--architecture-decorations--160-blocks)
07. [Botany: Flowers (`flowers`) — 32 Blocks](#07-botany-flowers-flowers--32-blocks)
08. [Botany: Groundcover & Vegetation (`vegetation`) — 38 Blocks](#08-botany-groundcover--vegetation-vegetation--38-blocks)
09. [Mycology: Mushrooms (`mushrooms`) — 18 Blocks](#09-mycology-mushrooms-mushrooms--18-blocks)
10. [Hydrosphere & Cryology (`fluids`) — 18 Blocks](#10-hydrosphere--cryology-fluids--18-blocks)
11. [Ocean & Benthic Ecosystems (`oceans`) — 36 Blocks](#11-ocean--benthic-ecosystems-oceans--36-blocks)
12. [Speleology & Crystals (`caverns`) — 28 Blocks](#12-speleology--crystals-caverns--28-blocks)
13. [Technical & System Blocks (`systems`) — 8 Blocks](#13-technical--system-blocks-systems--8-blocks)
14. [Future Roadmap: Agriculture, Crops & Food (`crops` / `foods`)](#14-future-roadmap-agriculture-crops--food-crops--foods)

---

## 01. Geology & Rocks (`rocks`) — 76 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/rocks`  
📄 **Comprehensive Specification**: [rocks_design.md](blocks/rocks_design.md)

The geological framework comprises **28 mineable rock matrices** across 7 core petrological domains (each yielding native solid stone and mined cobbled stone = 56 blocks), **16 surface relief outcrops** (grass and snow transitions = 16 blocks), **2 primordial mantle bedrock blocks**, and **6 hand-collectable surface pebbles** = **76 Total Geological Blocks**.

---

### 3.1. Aeolian & Marine Arenites (12 Blocks: 6 Solid + 6 Cobbled)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `sandstone_common` | Common Sandstone | **Sedimentary** | `rock_sandstone_common.png` | 6 uniform faces |
| **02** | `cobbled_sandstone_common` | Cobbled Sandstone | **Sedimentary** | `rock_sandstone_common_cobbled.png` | 6 uniform faces |
| **03** | `sandstone_red` | Red Sandstone | **Sedimentary** | `rock_sandstone_red.png` | 6 uniform faces |
| **04** | `cobbled_sandstone_red` | Cobbled Red Sandstone | **Sedimentary** | `rock_sandstone_red_cobbled.png` | 6 uniform faces |
| **05** | `sandstone_dune` | Dune Sandstone | **Sedimentary** | `rock_sandstone_dune.png` | 6 uniform faces |
| **06** | `cobbled_sandstone_dune` | Cobbled Dune Sandstone | **Sedimentary** | `rock_sandstone_dune_cobbled.png` | 6 uniform faces |
| **07** | `sandstone_white` | White Sandstone | **Sedimentary** | `rock_sandstone_white.png` | 6 uniform faces |
| **08** | `cobbled_sandstone_white` | Cobbled White Sandstone | **Sedimentary** | `rock_sandstone_white_cobbled.png` | 6 uniform faces |
| **09** | `sandstone_pink` | Pink Sandstone | **Sedimentary** | `rock_sandstone_pink.png` | 6 uniform faces |
| **10** | `cobbled_sandstone_pink` | Cobbled Pink Sandstone | **Sedimentary** | `rock_sandstone_pink_cobbled.png` | 6 uniform faces |
| **11** | `sandstone_black` | Black Sandstone | **Sedimentary** | `rock_sandstone_black.png` | 6 uniform faces |
| **12** | `cobbled_sandstone_black` | Cobbled Black Sandstone | **Sedimentary** | `rock_sandstone_black_cobbled.png` | 6 uniform faces |

---

### 3.2. Continental Plutonics & Orogenics (12 Blocks: 4 Solid + 4 Cobbled + 4 Outcrops)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **13** | `granite` | Granite | **Plutonic** | `rock_granite.png` | 6 uniform faces |
| **14** | `cobbled_granite` | Cobbled Granite | **Plutonic** | `rock_granite_cobbled.png` | 6 uniform faces |
| **15** | `granite_grass` | Granite Grass Outcrop | **Plutonic** | Top: `soil_grass.png`<br>Bot: `rock_granite.png`<br>Sides: `rock_granite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **16** | `granite_snow` | Granite Snow Outcrop | **Plutonic** | Top: `soil_snow.png`<br>Bot: `rock_granite.png`<br>Sides: `rock_granite_snow_side.png` | Top + Bot + 4 sides |
| **17** | `andesite` | Andesite | **Continental** | `rock_andesite.png` | 6 uniform faces |
| **18** | `cobbled_andesite` | Cobbled Andesite | **Continental** | `rock_andesite_cobbled.png` | 6 uniform faces |
| **19** | `andesite_grass` | Andesite Grass Outcrop | **Continental** | Top: `soil_grass.png`<br>Bot: `rock_andesite.png`<br>Sides: `rock_andesite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **20** | `andesite_snow` | Andesite Snow Outcrop | **Continental** | Top: `soil_snow.png`<br>Bot: `rock_andesite.png`<br>Sides: `rock_andesite_snow_side.png` | Top + Bot + 4 sides |
| **21** | `diorite` | Diorite | **Plutonic** | `rock_diorite.png` | 6 uniform faces |
| **22** | `cobbled_diorite` | Cobbled Diorite | **Plutonic** | `rock_diorite_cobbled.png` | 6 uniform faces |
| **23** | `gabbro` | Gabbro | **Plutonic** | `rock_gabbro.png` | 6 uniform faces *(Deep intrusive mafic bedrock)* |
| **24** | `cobbled_gabbro` | Cobbled Gabbro | **Plutonic** | `rock_gabbro_cobbled.png` | 6 uniform faces |

---

### 3.3. Sedimentary Strata & Cliffs (16 Blocks: 4 Solid + 4 Cobbled + 8 Outcrops)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **25** | `dolomite` | Dolomite (Alpine Stone) | **Sedimentary** | `rock_dolomite.png` | 6 uniform faces *(The classic Stone baseline)* |
| **26** | `cobbled_dolomite` | Cobbled Dolomite | **Sedimentary** | `rock_dolomite_cobbled.png` | 6 uniform faces |
| **27** | `dolomite_grass` | Dolomite Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_dolomite.png`<br>Sides: `rock_dolomite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **28** | `dolomite_snow` | Dolomite Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_dolomite.png`<br>Sides: `rock_dolomite_snow_side.png` | Top + Bot + 4 sides |
| **29** | `limestone` | Limestone | **Sedimentary** | `rock_limestone.png` | 6 uniform faces *(Fibrous travertine matrix)* |
| **30** | `cobbled_limestone` | Cobbled Limestone | **Sedimentary** | `rock_limestone_cobbled.png` | 6 uniform faces |
| **31** | `limestone_grass` | Limestone Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_limestone.png`<br>Sides: `rock_limestone_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **32** | `limestone_snow` | Limestone Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_limestone.png`<br>Sides: `rock_limestone_snow_side.png` | Top + Bot + 4 sides |
| **33** | `chalk` | Chalk | **Sedimentary** | `rock_chalk.png` | 6 uniform faces *(Oceanic white cliff face)* |
| **34** | `cobbled_chalk` | Cobbled Chalk | **Sedimentary** | `rock_chalk_cobbled.png` | 6 uniform faces |
| **35** | `chalk_grass` | Chalk Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_chalk.png`<br>Sides: `rock_chalk_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **36** | `chalk_snow` | Chalk Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_chalk.png`<br>Sides: `rock_chalk_snow_side.png` | Top + Bot + 4 sides |
| **37** | `argillite` | Argillite | **Sedimentary** | `rock_argillite.png` | 6 uniform faces *(Clay bedrock)* |
| **38** | `cobbled_argillite` | Cobbled Argillite | **Sedimentary** | `rock_argillite_cobbled.png` | 6 uniform faces |
| **39** | `argillite_grass` | Argillite Grass Outcrop | **Sedimentary** | Top: `soil_grass.png`<br>Bot: `rock_argillite.png`<br>Sides: `rock_argillite_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **40** | `argillite_snow` | Argillite Snow Outcrop | **Sedimentary** | Top: `soil_snow.png`<br>Bot: `rock_argillite.png`<br>Sides: `rock_argillite_snow_side.png` | Top + Bot + 4 sides |

---

### 3.4. Volcanic & Pyroclastics (10 Blocks: 4 Solid + 4 Cobbled + 2 Outcrops)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **41** | `basalt` | Basalt | **Volcanic** | `rock_basalt.png` | 6 uniform faces |
| **42** | `cobbled_basalt` | Cobbled Basalt | **Volcanic** | `rock_basalt_cobbled.png` | 6 uniform faces |
| **43** | `basalt_grass` | Basalt Grass Outcrop | **Volcanic** | Top: `soil_grass.png`<br>Bot: `rock_basalt.png`<br>Sides: `rock_basalt_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **44** | `basalt_snow` | Basalt Snow Outcrop | **Volcanic** | Top: `soil_snow.png`<br>Bot: `rock_basalt.png`<br>Sides: `rock_basalt_snow_side.png` | Top + Bot + 4 sides |
| **45** | `obsidian` | Obsidian | **Volcanic** | `rock_obsidian.png` | 6 uniform faces *(Amorphous volcanic glass)* |
| **46** | `cobbled_obsidian` | Cobbled Obsidian | **Volcanic** | `rock_obsidian_cobbled.png` | 6 uniform faces |
| **47** | `tuff` | Tuff | **Pyroclastic** | `rock_tuff.png` | 6 uniform faces *(Compacted volcanic ash)* |
| **48** | `cobbled_tuff` | Cobbled Tuff | **Pyroclastic** | `rock_tuff_cobbled.png` | 6 uniform faces |
| **49** | `scoria` | Scoria | **Pyroclastic** | `rock_scoria.png` | 6 uniform faces *(Vesicular basaltic cinders)* |
| **50** | `cobbled_scoria` | Cobbled Scoria | **Pyroclastic** | `rock_scoria_cobbled.png` | 6 uniform faces |

---

### 3.5. Metamorphic Foundations (10 Blocks: 4 Solid + 4 Cobbled + 2 Outcrops)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **51** | `slate` | Slate | **Metamorphic** | `rock_slate.png` | 6 uniform faces *(Laminated deep underground stone)* |
| **52** | `cobbled_slate` | Cobbled Slate | **Metamorphic** | `rock_slate_cobbled.png` | 6 uniform faces |
| **53** | `marble` | Marble | **Metamorphic** | `rock_marble.png` | 6 uniform faces *(Pure white architectural stone)* |
| **54** | `cobbled_marble` | Cobbled Marble | **Metamorphic** | `rock_marble_cobbled.png` | 6 uniform faces |
| **55** | `gneiss` | Gneiss | **Metamorphic** | `rock_gneiss.png` | 6 uniform faces *(Ancient banded shield bedrock)* |
| **56** | `cobbled_gneiss` | Cobbled Gneiss | **Metamorphic** | `rock_gneiss_cobbled.png` | 6 uniform faces |
| **57** | `gneiss_grass` | Gneiss Grass Outcrop | **Metamorphic** | Top: `soil_grass.png`<br>Bot: `rock_gneiss.png`<br>Sides: `rock_gneiss_grass_side.png` | Top + Bot + 4 sides (overlay `soil_grass_side_overlay.png`) |
| **58** | `gneiss_snow` | Gneiss Snow Outcrop | **Metamorphic** | Top: `soil_snow.png`<br>Bot: `rock_gneiss.png`<br>Sides: `rock_gneiss_snow_side.png` | Top + Bot + 4 sides |
| **59** | `serpentinite` | Serpentinite | **Metamorphic** | `rock_serpentinite.png` | 6 uniform faces *(Deep muted olive-slate hydrated rock)* |
| **60** | `cobbled_serpentinite`| Cobbled Serpentinite | **Metamorphic** | `rock_serpentinite_cobbled.png` | 6 uniform faces |

---

### 3.6. Subterranean & Geothermal Specials (8 Blocks: 4 Solid + 4 Cobbled)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **61** | `magma` | Magma Rock | **Geothermal** | `rock_magma.png` | 6 uniform faces *(Animated incandescent veins, emits light)* |
| **62** | `cobbled_magma` | Cobbled Magma | **Geothermal** | `rock_magma_cobbled.png` | 6 uniform faces *(Animated incandescent veins)* |
| **63** | `brimstone` | Brimstone | **Hydrothermal** | `rock_brimstone.png` | 6 uniform faces *(Sulfur fumaroles, cave speleothems)* |
| **64** | `cobbled_brimstone` | Cobbled Brimstone | **Hydrothermal** | `rock_brimstone_cobbled.png` | 6 uniform faces |
| **65** | `calcite` | Calcite | **Mineral** | `rock_calcite.png` | 6 uniform faces *(Crystalline geode matrix, speleothems)* |
| **66** | `cobbled_calcite` | Cobbled Calcite | **Mineral** | `rock_calcite_cobbled.png` | 6 uniform faces |
| **67** | `karst` | Karst Limestone | **Speleological** | `rock_karst.png` | 6 uniform faces *(Porous cave limestone, speleothems)* |
| **68** | `cobbled_karst` | Cobbled Karst | **Speleological** | `rock_karst_cobbled.png` | 6 uniform faces |

---

### 3.7. Primordial Mantle (2 Unbreakable Bedrock Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **69** | `mantle` | Primordial Mantle | **Bedrock** | `rock_mantle.png` | 6 uniform faces *(Cold high-pressure world floor, unbreakable)* |
| **70** | `mantle_plume` | Mantle Plume | **Bedrock** | `rock_mantle_plume.png` | 6 uniform faces *(Thermal hotspot ascending plume, unbreakable)* |

---

### 3.8. Ground Pebbles & River Shingles (6 Sub-Voxel Cluster Blocks)

| # | Block ID | Name | Cluster Stage | Texture File | Model Geometry |
| :-: | :--- | :--- | :-: | :--- | :--- |
| **71** | `pebble_flint_one` | Flint Pebble (Single) | 1 Pebble | `pebble_flint_one.png` | Sub-voxel box `[7..11, 0..2, 5..10]` |
| **72** | `pebble_flint_two` | Flint Pebble (Pair) | 2 Pebbles | `pebble_flint_two.png` | Corner micro-box `[3..5, 0..1, 10..12]` |
| **73** | `pebble_flint_three`| Flint Pebble (Cluster)| 3 Pebbles | `pebble_flint_three.png` | Dual boxes `[7..12, 0..2, 8..13]` + `[5..8, 0..1, 6..10]` |
| **74** | `pebble_quartz_one` | Quartz Pebble (Single) | 1 Pebble | `pebble_quartz_one.png` | Sub-voxel box `[7..11, 0..2, 5..10]` |
| **75** | `pebble_quartz_two` | Quartz Pebble (Pair) | 2 Pebbles | `pebble_quartz_two.png` | Corner micro-box `[3..5, 0..1, 10..12]` |
| **76** | `pebble_quartz_three`| Quartz Pebble (Cluster)| 3 Pebbles | `pebble_quartz_three.png` | Dual boxes `[7..12, 0..2, 8..13]` + `[5..8, 0..1, 6..10]` |

---

## 02. Pedology & Soils (`soils`) — 58 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/soils`  
📄 **Comprehensive Specification**: [soils_design.md](blocks/soils_design.md)

The planetary pedology system classifies soils into **3 core functional divisions**: 24 biological zonal soils (Loam, Silt, Peat series across 8 functional agricultural/turf states), 12 extreme climatic zonal soils (Permafrost, Caliche, Laterite series), and 22 mineral granular sediments, cryosphere covers, and biological mats = **58 Total Pedological Blocks**.

---

### 3.1. Biological Soils (24 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `loam_dirt` | Loam Dirt | **Biological** | `soil_loam_dirt.png` | 6 uniform faces |
| **02** | `loam_coarse` | Loam Coarse | **Biological** | `soil_loam_coarse.png` | 6 uniform faces |
| **03** | `loam_mud` | Loam Mud | **Biological** | `soil_loam_mud.png` | 6 uniform faces |
| **04** | `loam_rooted` | Loam Rooted | **Biological** | `soil_loam_rooted.png` | 6 uniform faces |
| **05** | `loam_grass` | Loam Grass | **Biological** | Top: `soil_grass.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `overlays/soil_grass_side_overlay.png`) |
| **06** | `loam_snow` | Loam Snow | **Biological** | Top: `soil_snow.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_snow_side.png` | Top + Bottom + 4 sides |
| **07** | `loam_mulch` | Loam Mulch | **Biological** | Top: `soil_loam_mulch.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_mulch_side.png` | Top + Bottom + 4 sides |
| **08** | `loam_tilled` | Loam Tilled | **Biological** | Top: `soil_loam_tilled_top.png`<br>Bottom: `soil_loam_dirt.png`<br>Sides: `soil_loam_tilled_side.png` | Top + Bottom + 4 sides |
| **09** | `silt_dirt` | Silt Dirt | **Biological** | `soil_silt_dirt.png` | 6 uniform faces |
| **10** | `silt_coarse` | Silt Coarse | **Biological** | `soil_silt_coarse.png` | 6 uniform faces |
| **11** | `silt_mud` | Silt Mud | **Biological** | `soil_silt_mud.png` | 6 uniform faces |
| **12** | `silt_rooted` | Silt Rooted | **Biological** | `soil_silt_rooted.png` | 6 uniform faces |
| **13** | `silt_grass` | Silt Grass | **Biological** | Top: `soil_grass.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `overlays/soil_grass_side_overlay.png`) |
| **14** | `silt_snow` | Silt Snow | **Biological** | Top: `soil_snow.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_snow_side.png` | Top + Bottom + 4 sides |
| **15** | `silt_mulch` | Silt Mulch | **Biological** | Top: `soil_silt_mulch.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_mulch_side.png` | Top + Bottom + 4 sides |
| **16** | `silt_tilled` | Silt Tilled | **Biological** | Top: `soil_silt_tilled_top.png`<br>Bottom: `soil_silt_dirt.png`<br>Sides: `soil_silt_tilled_side.png` | Top + Bottom + 4 sides |
| **17** | `peat_dirt` | Peat Dirt | **Biological** | `soil_peat_dirt.png` | 6 uniform faces |
| **18** | `peat_coarse` | Peat Coarse | **Biological** | `soil_peat_coarse.png` | 6 uniform faces |
| **19** | `peat_mud` | Peat Mud | **Biological** | `soil_peat_mud.png` | 6 uniform faces |
| **20** | `peat_rooted` | Peat Rooted | **Biological** | `soil_peat_rooted.png` | 6 uniform faces |
| **21** | `peat_grass` | Peat Grass | **Biological** | Top: `soil_grass.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `overlays/soil_grass_side_overlay.png`) |
| **22** | `peat_snow` | Peat Snow | **Biological** | Top: `soil_snow.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_snow_side.png` | Top + Bottom + 4 sides |
| **23** | `peat_mulch` | Peat Mulch | **Biological** | Top: `soil_peat_mulch.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_mulch_side.png` | Top + Bottom + 4 sides |
| **24** | `peat_tilled` | Peat Tilled | **Biological** | Top: `soil_peat_tilled_top.png`<br>Bottom: `soil_peat_dirt.png`<br>Sides: `soil_peat_tilled_side.png` | Top + Bottom + 4 sides |

---

### 3.2. Climatic Soils (12 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **25** | `permafrost_dirt` | Permafrost Dirt | **Climatic** | `soil_permafrost_dirt.png` | 6 uniform faces |
| **26** | `permafrost_coarse` | Permafrost Coarse | **Climatic** | `soil_permafrost_coarse.png` | 6 uniform faces |
| **27** | `permafrost_grass` | Permafrost Grass | **Climatic** | Top: `soil_grass.png`<br>Bottom: `soil_permafrost_dirt.png`<br>Sides: `soil_permafrost_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `overlays/soil_grass_side_overlay.png`) |
| **28** | `permafrost_icy` | Permafrost Icy | **Climatic** | `soil_permafrost_icy.png` | 6 uniform faces |
| **29** | `caliche_dirt` | Caliche Dirt | **Climatic** | `soil_caliche_dirt.png` | 6 uniform faces |
| **30** | `caliche_coarse` | Caliche Coarse | **Climatic** | `soil_caliche_coarse.png` | 6 uniform faces |
| **31** | `caliche_cracked` | Caliche Cracked | **Climatic** | `soil_caliche_cracked.png` | 6 uniform faces |
| **32** | `caliche_grass` | Caliche Grass | **Climatic** | Top: `soil_grass.png`<br>Bottom: `soil_caliche_dirt.png`<br>Sides: `soil_caliche_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `overlays/soil_grass_side_overlay.png`) |
| **33** | `laterite_dirt` | Laterite Dirt | **Climatic** | `soil_laterite_dirt.png` | 6 uniform faces |
| **34** | `laterite_coarse` | Laterite Coarse | **Climatic** | `soil_laterite_coarse.png` | 6 uniform faces |
| **35** | `laterite_hardpan` | Laterite Hardpan | **Climatic** | `soil_laterite_hardpan.png` | 6 uniform faces |
| **36** | `laterite_grass` | Laterite Grass | **Climatic** | Top: `soil_grass.png`<br>Bottom: `soil_laterite_dirt.png`<br>Sides: `soil_laterite_grass_side.png` | Top (biome tint) + Bottom + 4 sides (overlay `overlays/soil_grass_side_overlay.png`) |

---

### 3.3. Ground Mosses (4 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **37** | `moss_forest` | Forest Moss | **Ground Moss** | `soil_moss_forest.png` | 6 uniform faces *(Overlay: `overlays/moss_forest_side_overlay.png`)* |
| **38** | `moss_crimson` | Crimson Moss | **Ground Moss** | `soil_moss_crimson.png` | 6 uniform faces *(Overlay: `overlays/moss_crimson_side_overlay.png`)* |
| **39** | `moss_amber` | Amber Moss | **Ground Moss** | `soil_moss_amber.png` | 6 uniform faces *(Overlay: `overlays/moss_amber_side_overlay.png`)* |
| **40** | `moss_cave` | Cave Moss | **Ground Moss** | `soil_moss_cave.png` | 6 uniform faces *(Overlay: `overlays/moss_cave_side_overlay.png`)* |

---

### 3.4. Clays (4 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **41** | `clay_blue` | Blue Clay | **Clay** | `soil_clay_blue.png` | 6 uniform faces |
| **42** | `clay_red` | Red Clay | **Clay** | `soil_clay_red.png` | 6 uniform faces |
| **43** | `clay_white` | White Clay | **Clay** | `soil_clay_white.png` | 6 uniform faces |
| **44** | `clay_yellow` | Yellow Clay | **Clay** | `soil_clay_yellow.png` | 6 uniform faces |

---

### 3.5. Sands (6 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **45** | `sand_common` | Common Sand | **Sand** | `soil_sand_common.png` | 6 uniform faces |
| **46** | `sand_red` | Red Sand | **Sand** | `soil_sand_red.png` | 6 uniform faces |
| **47** | `sand_dune` | Dune Sand | **Sand** | `soil_sand_dune.png` | 6 uniform faces |
| **48** | `sand_white` | White Sand | **Sand** | `soil_sand_white.png` | 6 uniform faces |
| **49** | `sand_pink` | Pink Sand | **Sand** | `soil_sand_pink.png` | 6 uniform faces |
| **50** | `sand_black` | Black Sand | **Sand** | `soil_sand_black.png` | 6 uniform faces |

---

### 3.6. Matrix Gravels (4 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **51** | `gravel_dirty` | Dirty Gravel | **Gravel** | `soil_gravel_dirty.png` | 6 uniform faces |
| **52** | `gravel_sandy` | Sandy Gravel | **Gravel** | `soil_gravel_sandy.png` | 6 uniform faces |
| **53** | `gravel_ashy` | Ashy Gravel | **Gravel** | `soil_gravel_ashy.png` | 6 uniform faces |
| **54** | `gravel_snowy` | Snowy Gravel | **Gravel** | `soil_gravel_snowy.png` | 6 uniform faces |

---

### 3.7. Volcanic Ashes (2 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **55** | `ash_volcanic` | Volcanic Ash | **Pyroclastic** | `soil_ash_vulcanic.png` | 6 uniform faces |
| **56** | `ash_pumice` | Pumice Ash | **Pyroclastic** | `soil_ash_pumice.png` | 6 uniform faces |

---

### 3.8. Snows (2 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **57** | `snow_block` | Snow Block | **Cryosphere** | `soil_snow.png` | 6 uniform faces |
| **58** | `powder_snow` | Powder Snow | **Cryosphere** | `soil_snow_powder.png` | 6 uniform faces |

---

## 03. Trees, Forestry & Dendrology (`trees`) — 98 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/trees`  
📄 **Comprehensive Specification**: [trees_design.md](blocks/trees_design.md)

Planetary dendrology encompasses **18 living forest tree species** distributed across 5 major bioclimatic zones. Tree blocks are systematically classified into: 36 wood/trunk blocks (`log` with concentric rings and `wood` with 6-sided bark), 18 dressed lumber building blocks (`planks`), 43 canopy foliage blocks (`leaves` in healthy, flowering, lush, seasonal autumn, snowy, and dead states), and 1 estuarine aerial root block = **98 Total Forestry Blocks**.

---

### 6.1. Temperate & Deciduous Forest Blocks (35 Blocks)

| # | Block ID | Name | Category | Top Face | Bottom Face | Side Faces | Properties |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **001** | `oak_log` | Oak Log | **Wood** | `tree_oak_log.png` | `tree_oak_log.png` | `tree_oak_bark.png` | 3D Axis, Flammable |
| **002** | `oak_wood` | Oak Wood | **Wood** | `tree_oak_bark.png` | `tree_oak_bark.png` | `tree_oak_bark.png` | Solid Bark, Flammable |
| **003** | `oak_planks` | Oak Planks | **Planks** | `tree_oak_planks.png` | `tree_oak_planks.png` | `tree_oak_planks.png` | Solid, Building Material |
| **004** | `oak_leaves` | Oak Leaves | **Foliage** | `tree_oak_leaves.png` | `tree_oak_leaves.png` | `tree_oak_leaves.png` | Cutout, Canopy Decay |
| **005** | `oak_leaves_lush_flowering` | Flowering Lush Oak Leaves | **Foliage** | `tree_oak_leaves_flowering.png` | `tree_oak_leaves_flowering.png` | `tree_oak_leaves_flowering.png` | Cutout, Canopy Decay |
| **006** | `oak_leaves_lush` | Lush Oak Leaves | **Foliage** | `tree_oak_leaves_lush.png` | `tree_oak_leaves_lush.png` | `tree_oak_leaves_lush.png` | Cutout, Canopy Decay |
| **007** | `oak_leaves_dead` | Dead Oak Leaves | **Foliage** | `tree_oak_leaves_dead.png` | `tree_oak_leaves_dead.png` | `tree_oak_leaves_dead.png` | Cutout, Brittle, Highly Flammable |
| **008** | `birch_log` | Birch Log | **Wood** | `tree_birch_log.png` | `tree_birch_log.png` | `tree_birch_bark.png` | 3D Axis, Flammable |
| **009** | `birch_wood` | Birch Wood | **Wood** | `tree_birch_bark.png` | `tree_birch_bark.png` | `tree_birch_bark.png` | Solid Bark, Flammable |
| **010** | `birch_planks` | Birch Planks | **Planks** | `tree_birch_planks.png` | `tree_birch_planks.png` | `tree_birch_planks.png` | Solid, Building Material |
| **011** | `birch_leaves` | Birch Leaves | **Foliage** | `tree_birch_leaves.png` | `tree_birch_leaves.png` | `tree_birch_leaves.png` | Cutout, Canopy Decay |
| **012** | `birch_leaves_dead` | Dead Birch Leaves | **Foliage** | `tree_birch_leaves_dead.png` | `tree_birch_leaves_dead.png` | `tree_birch_leaves_dead.png` | Cutout, Brittle, Highly Flammable |
| **013** | `maple_log` | Maple Log | **Wood** | `tree_maple_log.png` | `tree_maple_log.png` | `tree_maple_bark.png` | 3D Axis, Flammable |
| **014** | `maple_wood` | Maple Wood | **Wood** | `tree_maple_bark.png` | `tree_maple_bark.png` | `tree_maple_bark.png` | Solid Bark, Flammable |
| **015** | `maple_planks` | Maple Planks | **Planks** | `tree_maple_planks.png` | `tree_maple_planks.png` | `tree_maple_planks.png` | Solid, Building Material |
| **016** | `maple_leaves_orange` | Orange Maple Leaves | **Foliage** | `tree_maple_leaves_orange.png` | `tree_maple_leaves_orange.png` | `tree_maple_leaves_orange.png` | Cutout, Canopy Decay |
| **017** | `maple_leaves_red` | Red Maple Leaves | **Foliage** | `tree_maple_leaves_red.png` | `tree_maple_leaves_red.png` | `tree_maple_leaves_red.png` | Cutout, Canopy Decay |
| **018** | `maple_leaves_yellow` | Yellow Maple Leaves | **Foliage** | `tree_maple_leaves_yellow.png` | `tree_maple_leaves_yellow.png` | `tree_maple_leaves_yellow.png` | Cutout, Canopy Decay |
| **019** | `maple_leaves_dead` | Dead Maple Leaves | **Foliage** | `tree_maple_leaves_dead.png` | `tree_maple_leaves_dead.png` | `tree_maple_leaves_dead.png` | Cutout, Brittle, Highly Flammable |
| **020** | `cherry_log` | Cherry Log | **Wood** | `tree_cherry_log.png` | `tree_cherry_log.png` | `tree_cherry_bark.png` | 3D Axis, Flammable |
| **021** | `cherry_wood` | Cherry Wood | **Wood** | `tree_cherry_bark.png` | `tree_cherry_bark.png` | `tree_cherry_bark.png` | Solid Bark, Flammable |
| **022** | `cherry_planks` | Cherry Planks | **Planks** | `tree_cherry_planks.png` | `tree_cherry_planks.png` | `tree_cherry_planks.png` | Solid, Building Material |
| **023** | `cherry_leaves` | Cherry Blossom Leaves | **Foliage** | `tree_cherry_leaves.png` | `tree_cherry_leaves.png` | `tree_cherry_leaves.png` | Cutout, Sakura Canopy Decay |
| **024** | `cherry_leaves_dead` | Dead Cherry Leaves | **Foliage** | `tree_cherry_leaves_dead.png` | `tree_cherry_leaves_dead.png` | `tree_cherry_leaves_dead.png` | Cutout, Brittle, Highly Flammable |
| **025** | `aspen_log` | Aspen Log | **Wood** | `tree_aspen_log.png` | `tree_aspen_log.png` | `tree_aspen_bark.png` | 3D Axis, Flammable |
| **026** | `aspen_wood` | Aspen Wood | **Wood** | `tree_aspen_bark.png` | `tree_aspen_bark.png` | `tree_aspen_bark.png` | Solid Bark, Flammable |
| **027** | `aspen_planks` | Aspen Planks | **Planks** | `tree_aspen_planks.png` | `tree_aspen_planks.png` | `tree_aspen_planks.png` | Solid, Building Material |
| **028** | `aspen_leaves` | Golden Aspen Leaves | **Foliage** | `tree_aspen_leaves.png` | `tree_aspen_leaves.png` | `tree_aspen_leaves.png` | Cutout, Canopy Decay |
| **029** | `aspen_leaves_snowy` | Snowy Aspen Leaves | **Foliage** | `tree_aspen_leaves_snowy_top.png` | `tree_aspen_leaves.png` | `tree_aspen_leaves_snowy_side.png` | Cutout, Canopy Decay, Snowy |
| **030** | `aspen_leaves_dead` | Dead Aspen Leaves | **Foliage** | `tree_aspen_leaves_dead.png` | `tree_aspen_leaves_dead.png` | `tree_aspen_leaves_dead.png` | Cutout, Brittle, Highly Flammable |
| **031** | `willow_log` | Willow Log | **Wood** | `tree_willow_log.png` | `tree_willow_log.png` | `tree_willow_bark.png` | 3D Axis, Flammable |
| **032** | `willow_wood` | Willow Wood | **Wood** | `tree_willow_bark.png` | `tree_willow_bark.png` | `tree_willow_bark.png` | Solid Bark, Flammable |
| **033** | `willow_planks` | Willow Planks | **Planks** | `tree_willow_planks.png` | `tree_willow_planks.png` | `tree_willow_planks.png` | Solid, Building Material |
| **034** | `willow_leaves` | Weeping Willow Leaves | **Foliage** | `tree_willow_leaves.png` | `tree_willow_leaves.png` | `tree_willow_leaves.png` | Cutout, Drooping Canopy Decay |
| **035** | `willow_leaves_dead` | Dead Willow Leaves | **Foliage** | `tree_willow_leaves_dead.png` | `tree_willow_leaves_dead.png` | `tree_willow_leaves_dead.png` | Cutout, Brittle, Highly Flammable |

---

### 6.2. Boreal & Coniferous Taiga Blocks (23 Blocks)

| # | Block ID | Name | Category | Top Face | Bottom Face | Side Faces | Properties |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **036** | `pine_log` | Pine Log | **Wood** | `tree_pine_log.png` | `tree_pine_log.png` | `tree_pine_bark.png` | 3D Axis, Flammable |
| **037** | `pine_wood` | Pine Wood | **Wood** | `tree_pine_bark.png` | `tree_pine_bark.png` | `tree_pine_bark.png` | Solid Bark, Flammable |
| **038** | `pine_planks` | Pine Planks | **Planks** | `tree_pine_planks.png` | `tree_pine_planks.png` | `tree_pine_planks.png` | Solid, Building Material |
| **039** | `pine_leaves` | Pine Needles | **Foliage** | `tree_pine_leaves.png` | `tree_pine_leaves.png` | `tree_pine_leaves.png` | Cutout, Canopy Decay, Tintable |
| **040** | `pine_leaves_snowy` | Snowy Pine Needles | **Foliage** | `tree_pine_leaves_snowy_top.png` | `tree_pine_leaves.png` | `tree_pine_leaves.png` *(+ overlay)* | Cutout, Snowy |
| **041** | `pine_leaves_dead` | Dead Pine Needles | **Foliage** | `tree_pine_leaves_dead.png` | `tree_pine_leaves_dead.png` | `tree_pine_leaves_dead.png` | Cutout, Dry Needles, Flammable |
| **042** | `fir_log` | Fir Log | **Wood** | `tree_fir_log.png` | `tree_fir_log.png` | `tree_fir_bark.png` | 3D Axis, Flammable |
| **043** | `fir_wood` | Fir Wood | **Wood** | `tree_fir_bark.png` | `tree_fir_bark.png` | `tree_fir_bark.png` | Solid Bark, Flammable |
| **044** | `fir_planks` | Fir Planks | **Planks** | `tree_fir_planks.png` | `tree_fir_planks.png` | `tree_fir_planks.png` | Solid, Building Material |
| **045** | `fir_leaves` | Fir Needles | **Foliage** | `tree_fir_leaves.png` | `tree_fir_leaves.png` | `tree_fir_leaves.png` | Cutout, Canopy Decay |
| **046** | `fir_leaves_snowy` | Snowy Fir Needles | **Foliage** | `tree_fir_leaves_snowy.png` | `tree_fir_leaves_snowy.png` | `tree_fir_leaves_snowy.png` | Cutout, Canopy Decay, Snowy |
| **047** | `fir_leaves_dead` | Dead Fir Needles | **Foliage** | `tree_fir_leaves_dead.png` | `tree_fir_leaves_dead.png` | `tree_fir_leaves_dead.png` | Cutout, Dry Needles, Flammable |
| **048** | `redwood_log` | Redwood Log | **Wood** | `tree_redwood_log.png` | `tree_redwood_log.png` | `tree_redwood_bark.png` | 3D Axis, Colossal Trunk |
| **049** | `redwood_wood` | Redwood Wood | **Wood** | `tree_redwood_bark.png` | `tree_redwood_bark.png` | `tree_redwood_bark.png` | Solid Bark, Flammable |
| **050** | `redwood_planks` | Redwood Planks | **Planks** | `tree_redwood_planks.png` | `tree_redwood_planks.png` | `tree_redwood_planks.png` | Solid, Brick-Red Building Material |
| **051** | `redwood_leaves` | Redwood Foliage | **Foliage** | `tree_redwood_leaves.png` | `tree_redwood_leaves.png` | `tree_redwood_leaves.png` | Cutout, Canopy Decay |
| **052** | `redwood_leaves_dead` | Dead Redwood Foliage | **Foliage** | `tree_redwood_leaves_dead.png` | `tree_redwood_leaves_dead.png` | `tree_redwood_leaves_dead.png` | Cutout, Brittle, Highly Flammable |
| **053** | `yew_log` | Yew Log | **Wood** | `tree_yew_log.png` | `tree_yew_log.png` | `tree_yew_bark.png` | 3D Axis, Dark Heartwood |
| **054** | `yew_wood` | Yew Wood | **Wood** | `tree_yew_bark.png` | `tree_yew_bark.png` | `tree_yew_bark.png` | Solid Bark, Flammable |
| **055** | `yew_planks` | Yew Planks | **Planks** | `tree_yew_planks.png` | `tree_yew_planks.png` | `tree_yew_planks.png` | Solid, Dark Ebony Material |
| **056** | `yew_leaves` | Yew Foliage | **Foliage** | `tree_yew_leaves.png` | `tree_yew_leaves.png` | `tree_yew_leaves.png` | Cutout, Canopy Decay |
| **057** | `yew_leaves_snowy` | Snowy Yew Foliage | **Foliage** | `tree_yew_leaves_snowy_top.png` | `tree_yew_leaves.png` | `tree_yew_leaves.png` *(+ overlay)* | Cutout, Snowy |
| **058** | `yew_leaves_dead` | Dead Yew Foliage | **Foliage** | `tree_yew_leaves_dead.png` | `tree_yew_leaves_dead.png` | `tree_yew_leaves_dead.png` | Cutout, Brittle, Highly Flammable |

---

### 6.3. Tropical & Rainforest Jungle Blocks (20 Blocks)

| # | Block ID | Name | Category | Top Face | Bottom Face | Side Faces | Properties |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **059** | `mahogany_log` | Mahogany Log | **Wood** | `tree_mahogany_log.png` | `tree_mahogany_log.png` | `tree_mahogany_bark.png` | 3D Axis, Flammable |
| **060** | `mahogany_wood` | Mahogany Wood | **Wood** | `tree_mahogany_bark.png` | `tree_mahogany_bark.png` | `tree_mahogany_bark.png` | Solid Bark, Flammable |
| **061** | `mahogany_planks` | Mahogany Planks | **Planks** | `tree_mahogany_planks.png` | `tree_mahogany_planks.png` | `tree_mahogany_planks.png` | Solid, Deep Auburn Material |
| **062** | `mahogany_leaves` | Mahogany Leaves | **Foliage** | `tree_mahogany_leaves.png` | `tree_mahogany_leaves.png` | `tree_mahogany_leaves.png` | Cutout, Canopy Decay |
| **063** | `mahogany_leaves_dead` | Dead Mahogany Leaves | **Foliage** | `tree_mahogany_leaves_dead.png` | `tree_mahogany_leaves_dead.png` | `tree_mahogany_leaves_dead.png` | Cutout, Brittle, Highly Flammable |
| **064** | `bamboo_stalk` | Bamboo Stalk | **Wood / Stalk** | `tree_bamboo_stalk.png` | `tree_bamboo_stalk.png` | `tree_bamboo_stalk.png` | Slim Tube / Column, Orientable |
| **065** | `bamboo_planks` | Bamboo Planks | **Planks** | `tree_bamboo_planks.png` | `tree_bamboo_planks.png` | `tree_bamboo_planks.png` | Solid, Woven Mat Pattern |
| **066** | `bamboo_large_leaves` | Large Bamboo Leaves | **Foliage** | `tree_bamboo_large_leaves.png` | `tree_bamboo_large_leaves.png` | `tree_bamboo_large_leaves.png` | Cutout, High Canopy |
| **067** | `bamboo_small_leaves` | Small Bamboo Leaves | **Foliage** | `tree_bamboo_small_leaves.png` | `tree_bamboo_small_leaves.png` | `tree_bamboo_small_leaves.png` | Cutout, Low Shoot Leaves |
| **068** | `mangrove_log` | Mangrove Log | **Wood** | `tree_mangrove_log.png` | `tree_mangrove_log.png` | `tree_mangrove_bark.png` | 3D Axis, Salt-Tolerant |
| **069** | `mangrove_wood` | Mangrove Wood | **Wood** | `tree_mangrove_bark.png` | `tree_mangrove_bark.png` | `tree_mangrove_bark.png` | Solid Bark, Flammable |
| **070** | `mangrove_planks` | Mangrove Planks | **Planks** | `tree_mangrove_planks.png` | `tree_mangrove_planks.png` | `tree_mangrove_planks.png` | Solid, Coppery-Tan Material |
| **071** | `mangrove_leaves` | Mangrove Leaves | **Foliage** | `tree_mangrove_leaves.png` | `tree_mangrove_leaves.png` | `tree_mangrove_leaves.png` | Cutout, Canopy Decay |
| **072** | `mangrove_roots` | Mangrove Aerial Roots | **Root** | `tree_mangrove_roots_top.png` | `tree_mangrove_roots_top.png` | `tree_mangrove_roots.png` | Semi-Solid, Waterlogged Permeable |
| **073** | `mangrove_leaves_dead` | Dead Mangrove Leaves | **Foliage** | `tree_mangrove_leaves_dead.png` | `tree_mangrove_leaves_dead.png` | `tree_mangrove_leaves_dead.png` | Cutout, Brittle, Salted Decay |
| **074** | `kapok_log` | Kapok Log | **Wood** | `tree_kapok_log.png` | `tree_kapok_log.png` | `tree_kapok_bark.png` | 3D Axis, Colossal Emergent |
| **075** | `kapok_wood` | Kapok Wood | **Wood** | `tree_kapok_bark.png` | `tree_kapok_bark.png` | `tree_kapok_bark.png` | Solid Bark, Buttress Core |
| **076** | `kapok_planks` | Kapok Planks | **Planks** | `tree_kapok_planks.png` | `tree_kapok_planks.png` | `tree_kapok_planks.png` | Solid, Warm Pale Tropical Wood |
| **077** | `kapok_leaves` | Kapok Leaves | **Foliage** | `tree_kapok_leaves.png` | `tree_kapok_leaves.png` | `tree_kapok_leaves.png` | Cutout, Palmate Canopy Decay |
| **078** | `kapok_leaves_dead` | Dead Kapok Leaves | **Foliage** | `tree_kapok_leaves_dead.png` | `tree_kapok_leaves_dead.png` | `tree_kapok_leaves_dead.png` | Cutout, Brittle, Highly Flammable |

---

### 6.4. Arid Savanna & Woodland Blocks (10 Blocks)

| # | Block ID | Name | Category | Top Face | Bottom Face | Side Faces | Properties |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **079** | `acacia_log` | Acacia Log | **Wood** | `tree_acacia_log.png` | `tree_acacia_log.png` | `tree_acacia_bark.png` | 3D Axis, Flammable |
| **080** | `acacia_wood` | Acacia Wood | **Wood** | `tree_acacia_bark.png` | `tree_acacia_bark.png` | `tree_acacia_bark.png` | Solid Bark, Flammable |
| **081** | `acacia_planks` | Acacia Planks | **Planks** | `tree_acacia_planks.png` | `tree_acacia_planks.png` | `tree_acacia_planks.png` | Solid, Vibrant Terracotta Material |
| **082** | `acacia_leaves` | Acacia Leaves | **Foliage** | `tree_acacia_leaves.png` | `tree_acacia_leaves.png` | `tree_acacia_leaves.png` | Cutout, Umbrella Canopy Decay |
| **083** | `acacia_leaves_dead` | Dead Acacia Leaves | **Foliage** | `tree_acacia_leaves_dead.png` | `tree_acacia_leaves_dead.png` | `tree_acacia_leaves_dead.png` | Cutout, Brittle, Arid Drought Decay |
| **084** | `baobab_log` | Baobab Log | **Wood** | `tree_baobab_log.png` | `tree_baobab_log.png` | `tree_baobab_bark.png` | 3D Axis, Pachycaul Water-Rich |
| **085** | `baobab_wood` | Baobab Wood | **Wood** | `tree_baobab_bark.png` | `tree_baobab_bark.png` | `tree_baobab_bark.png` | Solid Bark, Massive Trunk Core |
| **086** | `baobab_planks` | Baobab Planks | **Planks** | `tree_baobab_planks.png` | `tree_baobab_planks.png` | `tree_baobab_planks.png` | Solid, Fibrous Copper-Orange |
| **087** | `baobab_leaves` | Baobab Leaves | **Foliage** | `tree_baobab_leaves.png` | `tree_baobab_leaves.png` | `tree_baobab_leaves.png` | Cutout, Tufted Canopy Decay |
| **088** | `baobab_leaves_dead` | Dead Baobab Leaves | **Foliage** | `tree_baobab_leaves_dead.png` | `tree_baobab_leaves_dead.png` | `tree_baobab_leaves_dead.png` | Cutout, Brittle, Severe Estivation Decay |

---

### 6.5. Coastal & Wetland Grove Blocks (10 Blocks)

| # | Block ID | Name | Category | Top Face | Bottom Face | Side Faces | Properties |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **089** | `palm_log` | Palm Log | **Wood** | `tree_palm_log.png` | `tree_palm_log.png` | `tree_palm_bark.png` | 3D Axis, Fibrous Stipe |
| **090** | `palm_wood` | Palm Wood | **Wood** | `tree_palm_bark.png` | `tree_palm_bark.png` | `tree_palm_bark.png` | Solid Stipe Bark, Flammable |
| **091** | `palm_planks` | Palm Planks | **Planks** | `tree_palm_planks.png` | `tree_palm_planks.png` | `tree_palm_planks.png` | Solid, Golden Blonde Material |
| **092** | `palm_leaves` | Palm Fronds | **Foliage** | `tree_palm_leaves.png` | `tree_palm_leaves.png` | `tree_palm_leaves.png` | Cutout, Broad Arching Fronds |
| **093** | `palm_leaves_dead` | Dead Palm Fronds | **Foliage** | `tree_palm_leaves_dead.png` | `tree_palm_leaves_dead.png` | `tree_palm_leaves_dead.png` | Cutout, Dry Thatch Straw Fronds |
| **094** | `cypress_log` | Cypress Log | **Wood** | `tree_cypress_log.png` | `tree_cypress_log.png` | `tree_cypress_bark.png` | 3D Axis, Rot-Resistant |
| **095** | `cypress_wood` | Cypress Wood | **Wood** | `tree_cypress_bark.png` | `tree_cypress_bark.png` | `tree_cypress_bark.png` | Solid Bark, Rot-Resistant |
| **096** | `cypress_planks` | Cypress Planks | **Planks** | `tree_cypress_planks.png` | `tree_cypress_planks.png` | `tree_cypress_planks.png` | Solid, Resilient Hazel Material |
| **097** | `cypress_leaves` | Cypress Foliage | **Foliage** | `tree_cypress_leaves.png` | `tree_cypress_leaves.png` | `tree_cypress_leaves.png` | Cutout, Dense Columnar Foliage |
| **098** | `cypress_leaves_dead` | Dead Cypress Foliage | **Foliage** | `tree_cypress_leaves_dead.png` | `tree_cypress_leaves_dead.png` | `tree_cypress_leaves_dead.png` | Cutout, Brittle, Swamp Decay |

---

## 04. Ores, Minerals & Gemology (`ores`) — 158 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/ores`  
📄 **Comprehensive Specification**: [ores_design.md](blocks/ores_design.md)

The metallogenic and mineralogical domain comprises **26 solid unrefined raw blocks** (18 native/heavy metals and chemicals + 8 rough gemstones) and **132 procedural ore veins** dynamically generated across 24 mineralized host rock matrices (with obsidian, magma, mantle, and mantle plume remaining sterile) = **158 Total Mineral Blocks**.

---

### Part A: Solid Unrefined Mineral Blocks (26 Blocks)

#### 4.1. Native, Precious & Structural Metals (8 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Host Rocks | Visual Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **001** | `ore_iron_raw` | Raw Iron Block | **Metal** | `ore_iron_raw.png` | Uniform (6 faces) | 12 Host Rocks | Dense unrefined block of hematite and magnetite ore with dark brown-orange mineral crust. |
| **002** | `ore_copper_raw` | Raw Copper Block | **Metal** | `ore_copper_raw.png` | Uniform (6 faces) | 14 Host Rocks | Aggregate of raw native copper and green malachite nodules with warm terracotta-bronze hues. |
| **003** | `ore_tin_raw` | Raw Tin Block | **Metal** | `ore_tin_raw.png` | Uniform (6 faces) | 6 Host Rocks | Heavy crystalline cassiterite block with mottled brownish-grey and silvery-white facets. |
| **004** | `ore_zinc_raw` | Raw Zinc Block | **Metal** | `ore_zinc_raw.png` | Uniform (6 faces) | 8 Host Rocks | Compact sphalerite ore block with dark sub-metallic luster and dull greyish-brown crystalline veins. |
| **005** | `ore_gold_raw` | Raw Gold Block | **Metal** | `ore_gold_raw.png` | Uniform (6 faces) | 4 Host Rocks | Brilliant unrefined native gold chunk mass with rich yellow metallic grains and quartz traces. |
| **006** | `ore_silver_raw` | Raw Silver Block | **Metal** | `ore_silver_raw.png` | Uniform (6 faces) | 4 Host Rocks | Dense tarnished argentite and native silver aggregate with shimmering grey-white metallic veins. |
| **007** | `ore_platinum_raw` | Raw Platinum Block | **Metal** | `ore_platinum_raw.png` | Uniform (6 faces) | 4 Host Rocks | Ultra-dense compact block of native platinum grains with cool silvery-white metallic sheen. |
| **008** | `ore_aluminum_raw` | Raw Aluminum Block | **Metal** | `ore_aluminum_raw.png` | Uniform (6 faces) | 4 Host Rocks | Earthy pisolitic bauxite block with characteristic reddish-brown and pale ochre clay nodules. |

#### 4.2. Heavy & Strategic Metals (4 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Host Rocks | Visual Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **009** | `ore_lead_raw` | Raw Lead Block | **Heavy Metal** | `ore_lead_raw.png` | Uniform (6 faces) | 8 Host Rocks | Heavy block of cubic galena ore with distinctive steely-grey metallic luster and cubic cleavage planes. |
| **010** | `ore_nickel_raw` | Raw Nickel Block | **Heavy Metal** | `ore_nickel_raw.png` | Uniform (6 faces) | 4 Host Rocks | Dense pentlandite and pyrrhotite aggregate with pale bronze-yellow to brassy metallic sheen. |
| **011** | `ore_manganese_raw` | Raw Manganese Block | **Heavy Metal** | `ore_manganese_raw.png` | Uniform (6 faces) | 6 Host Rocks | Raspberry-pink crystalline rhodochrosite mineral block with rose-red banded carbonate patterns. |
| **012** | `ore_uranium_raw` | Raw Uranium Block | **Heavy Metal** | `ore_uranium_raw.png` | Uniform (6 faces) | 4 Host Rocks | Heavy pitchblende uraninite block displaying mottled pitch-black surface with sulfur-yellow oxide flecks. |

#### 4.3. Precious Gemstones (8 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Host Rocks | Visual Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **013** | `gem_diamond_rough` | Rough Diamond Block | **Gemstone** | `gem_diamond_rough.png` | Uniform (6 faces) | Gabbro, Serpentinite | Uncut octahedral carbon diamond crystal cluster with brilliant icy-white and pale cyan highlights. |
| **014** | `gem_ruby_rough` | Rough Ruby Block | **Gemstone** | `gem_ruby_rough.png` | Uniform (6 faces) | Basalt, Marble | Deep crimson and scarlet corundum crystal mass with rich vitreous ruby cleavage facets. |
| **015** | `gem_sapphire_rough` | Rough Sapphire Block | **Gemstone** | `gem_sapphire_rough.png` | Uniform (6 faces) | Basalt, Marble | Royal ocean-blue corundum crystal mass with navy undertones and vitreous cleavage facets. |
| **016** | `gem_topaz_rough` | Rough Topaz Block | **Gemstone** | `gem_topaz_rough.png` | Uniform (6 faces) | Sandstone Pink, Granite | Radiant golden-amber and solar-orange prismatic nesosilicate crystal aggregate. |
| **017** | `gem_emerald_rough` | Rough Emerald Block | **Gemstone** | `gem_emerald_rough.png` | Uniform (6 faces) | Slate, Marble | Vivid deep forest-green hexagonal beryl crystals intergrown in a crystalline emerald aggregate. |
| **018** | `gem_jade_rough` | Rough Jade Block | **Gemstone** | `gem_jade_rough.png` | Uniform (6 faces) | Marble, Serpentinite | Smooth, fibrous nephrite/jadeite aggregate with silky apple-green to sage-green translucency. |
| **019** | `gem_opal_rough` | Rough Opal Block | **Gemstone** | `gem_opal_rough.png` *(16×64)* | Animated (4 frames) | Sandstone White, Sandstone Pink, Basalt, Tuff | Hydrated silica matrix with dynamic 4-phase animated rainbow play-of-color across faces. |
| **020** | `gem_onyx_rough` | Rough Onyx Block | **Gemstone** | `gem_onyx_rough.png` *(16×64)* | Animated (4 frames) | Sandstone Black, Slate, Marble, Calcite | Deep jet-black banded chalcedony with subtle parallel micro-layers and traveling silver glints. |

#### 4.4. Industrial Chemicals & Non-Metals (6 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Host Rocks | Visual Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **021** | `ore_coal_raw` | Raw Coal Block | **Chemical** | `ore_coal_raw.png` | Uniform (6 faces) | 12 Host Rocks | Dense, jet-black carbonaceous sedimentary block with vitreous to sub-metallic fracture surfaces. |
| **022** | `ore_sulfur_raw` | Raw Sulfur Block | **Chemical** | `ore_sulfur_raw.png` | Uniform (6 faces) | Basalt, Tuff, Scoria, Brimstone | Vivid canary-yellow crystalline native sulfur aggregate with resinous to greasy luster. |
| **023** | `ore_saltpeter_raw` | Raw Saltpeter Block | **Chemical** | `ore_saltpeter_raw.png` | Uniform (6 faces) | 4 Host Rocks | Powdery, chalky white to pale grey encrustation of crystalline potassium nitrate niter. |
| **024** | `ore_quartz_raw` | Raw Quartz Block | **Chemical** | `ore_quartz_raw.png` | Uniform (6 faces) | 6 Host Rocks | Crystalline mass of milky to vitreous clear hydrothermal quartz with interlocking trigonal prisms. |
| **025** | `ore_cinnabar_raw` | Raw Cinnabar Block | **Chemical** | `ore_cinnabar_raw.png` | Uniform (6 faces) | Basalt, Tuff, Brimstone, Karst | Bright cochineal to vermilion scarlet mercury sulfide mineral block with adamantine luster. |
| **026** | `ore_lazurite_raw` | Raw Lazurite Block | **Chemical** | `ore_lazurite_raw.png` | Uniform (6 faces) | Sandstone Black, Limestone, Marble, Calcite | Deep royal ultramarine blue sodalite mineral aggregate speckled with golden pyrite flecks. |

---

### Part B: Procedural Ore Vein Blocks (132 Blocks)

### 4.1. Aeolian & Marine Sandstone Ores (22 Blocks)

| # | Block ID | Name | Category | Host Rock | Overlay Texture | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **027** | `ore_coal_sandstone_common` | Coal Ore (Common Sandstone) | **Chemical** | `rock_sandstone_common.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **028** | `ore_copper_sandstone_common` | Copper Ore (Common Sandstone) | **Metal** | `rock_sandstone_common.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **029** | `ore_iron_sandstone_common` | Iron Ore (Common Sandstone) | **Metal** | `rock_sandstone_common.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **030** | `ore_lead_sandstone_common` | Lead Ore (Common Sandstone) | **Heavy Metal** | `rock_sandstone_common.png` | `ore_lead_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **031** | `ore_quartz_sandstone_common` | Quartz Ore (Common Sandstone) | **Chemical** | `rock_sandstone_common.png` | `ore_quartz_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **032** | `ore_tin_sandstone_common` | Tin Ore (Common Sandstone) | **Metal** | `rock_sandstone_common.png` | `ore_tin_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **033** | `ore_copper_sandstone_red` | Copper Ore (Red Sandstone) | **Metal** | `rock_sandstone_red.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **034** | `ore_gold_sandstone_red` | Gold Ore (Red Sandstone) | **Metal** | `rock_sandstone_red.png` | `ore_gold_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **035** | `ore_iron_sandstone_red` | Iron Ore (Red Sandstone) | **Metal** | `rock_sandstone_red.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **036** | `ore_quartz_sandstone_red` | Quartz Ore (Red Sandstone) | **Chemical** | `rock_sandstone_red.png` | `ore_quartz_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **037** | `ore_coal_sandstone_dune` | Coal Ore (Dune Sandstone) | **Chemical** | `rock_sandstone_dune.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **038** | `ore_saltpeter_sandstone_dune` | Saltpeter Ore (Dune Sandstone) | **Chemical** | `rock_sandstone_dune.png` | `ore_saltpeter_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **039** | `ore_aluminum_sandstone_white` | Aluminum Ore (White Sandstone) | **Metal** | `rock_sandstone_white.png` | `ore_aluminum_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **040** | `gem_opal_sandstone_white` | Opal Ore (White Sandstone) | **Gemstone** | `rock_sandstone_white.png` | `gem_opal_overlay.png` | Uniform 6 faces (Animated 16×64 Overlay) |
| **041** | `ore_quartz_sandstone_white` | Quartz Ore (White Sandstone) | **Chemical** | `rock_sandstone_white.png` | `ore_quartz_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **042** | `ore_saltpeter_sandstone_white` | Saltpeter Ore (White Sandstone) | **Chemical** | `rock_sandstone_white.png` | `ore_saltpeter_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **043** | `gem_opal_sandstone_pink` | Opal Ore (Pink Sandstone) | **Gemstone** | `rock_sandstone_pink.png` | `gem_opal_overlay.png` | Uniform 6 faces (Animated 16×64 Overlay) |
| **044** | `gem_topaz_sandstone_pink` | Topaz Ore (Pink Sandstone) | **Gemstone** | `rock_sandstone_pink.png` | `gem_topaz_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **045** | `ore_coal_sandstone_black` | Coal Ore (Black Sandstone) | **Chemical** | `rock_sandstone_black.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **046** | `ore_iron_sandstone_black` | Iron Ore (Black Sandstone) | **Metal** | `rock_sandstone_black.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **047** | `ore_lazurite_sandstone_black` | Lazurite Ore (Black Sandstone) | **Chemical** | `rock_sandstone_black.png` | `ore_lazurite_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **048** | `gem_onyx_sandstone_black` | Onyx Ore (Black Sandstone) | **Gemstone** | `rock_sandstone_black.png` | `gem_onyx_overlay.png` | Uniform 6 faces (Animated 16×64 Overlay) |

### 4.2. Continental & Orogenic Plutonic Ores (30 Blocks)

| # | Block ID | Name | Category | Host Rock | Overlay Texture | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **049** | `ore_coal_granite` | Coal Ore (Granite) | **Chemical** | `rock_granite.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **050** | `ore_copper_granite` | Copper Ore (Granite) | **Metal** | `rock_granite.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **051** | `ore_gold_granite` | Gold Ore (Granite) | **Metal** | `rock_granite.png` | `ore_gold_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **052** | `ore_iron_granite` | Iron Ore (Granite) | **Metal** | `rock_granite.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **053** | `ore_quartz_granite` | Quartz Ore (Granite) | **Chemical** | `rock_granite.png` | `ore_quartz_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **054** | `ore_tin_granite` | Tin Ore (Granite) | **Metal** | `rock_granite.png` | `ore_tin_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **055** | `gem_topaz_granite` | Topaz Ore (Granite) | **Gemstone** | `rock_granite.png` | `gem_topaz_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **056** | `ore_uranium_granite` | Uranium Ore (Granite) | **Heavy Metal** | `rock_granite.png` | `ore_uranium_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **057** | `ore_coal_diorite` | Coal Ore (Diorite) | **Chemical** | `rock_diorite.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **058** | `ore_copper_diorite` | Copper Ore (Diorite) | **Metal** | `rock_diorite.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **059** | `ore_iron_diorite` | Iron Ore (Diorite) | **Metal** | `rock_diorite.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **060** | `ore_lead_diorite` | Lead Ore (Diorite) | **Heavy Metal** | `rock_diorite.png` | `ore_lead_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **061** | `ore_silver_diorite` | Silver Ore (Diorite) | **Metal** | `rock_diorite.png` | `ore_silver_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **062** | `ore_tin_diorite` | Tin Ore (Diorite) | **Metal** | `rock_diorite.png` | `ore_tin_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **063** | `ore_uranium_diorite` | Uranium Ore (Diorite) | **Heavy Metal** | `rock_diorite.png` | `ore_uranium_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **064** | `ore_zinc_diorite` | Zinc Ore (Diorite) | **Metal** | `rock_diorite.png` | `ore_zinc_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **065** | `ore_coal_andesite` | Coal Ore (Andesite) | **Chemical** | `rock_andesite.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **066** | `ore_copper_andesite` | Copper Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **067** | `ore_iron_andesite` | Iron Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **068** | `ore_lead_andesite` | Lead Ore (Andesite) | **Heavy Metal** | `rock_andesite.png` | `ore_lead_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **069** | `ore_quartz_andesite` | Quartz Ore (Andesite) | **Chemical** | `rock_andesite.png` | `ore_quartz_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **070** | `ore_silver_andesite` | Silver Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_silver_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **071** | `ore_tin_andesite` | Tin Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_tin_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **072** | `ore_zinc_andesite` | Zinc Ore (Andesite) | **Metal** | `rock_andesite.png` | `ore_zinc_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **073** | `ore_copper_gabbro` | Copper Ore (Gabbro) | **Metal** | `rock_gabbro.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **074** | `gem_diamond_gabbro` | Diamond Ore (Gabbro) | **Gemstone** | `rock_gabbro.png` | `gem_diamond_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **075** | `ore_iron_gabbro` | Iron Ore (Gabbro) | **Metal** | `rock_gabbro.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **076** | `ore_nickel_gabbro` | Nickel Ore (Gabbro) | **Heavy Metal** | `rock_gabbro.png` | `ore_nickel_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **077** | `ore_platinum_gabbro` | Platinum Ore (Gabbro) | **Metal** | `rock_gabbro.png` | `ore_platinum_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **078** | `ore_uranium_gabbro` | Uranium Ore (Gabbro) | **Heavy Metal** | `rock_gabbro.png` | `ore_uranium_overlay.png` | Uniform 6 faces (Shader Overlay) |

### 4.3. Sedimentary Strata & Cliff Ores (26 Blocks)

| # | Block ID | Name | Category | Host Rock | Overlay Texture | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **079** | `ore_coal_dolomite` | Coal Ore (Dolomite) | **Chemical** | `rock_dolomite.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **080** | `ore_copper_dolomite` | Copper Ore (Dolomite) | **Metal** | `rock_dolomite.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **081** | `ore_iron_dolomite` | Iron Ore (Dolomite) | **Metal** | `rock_dolomite.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **082** | `ore_manganese_dolomite` | Manganese Ore (Dolomite) | **Heavy Metal** | `rock_dolomite.png` | `ore_manganese_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **083** | `ore_tin_dolomite` | Tin Ore (Dolomite) | **Metal** | `rock_dolomite.png` | `ore_tin_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **084** | `ore_zinc_dolomite` | Zinc Ore (Dolomite) | **Metal** | `rock_dolomite.png` | `ore_zinc_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **085** | `ore_aluminum_limestone` | Aluminum Ore (Limestone) | **Metal** | `rock_limestone.png` | `ore_aluminum_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **086** | `ore_coal_limestone` | Coal Ore (Limestone) | **Chemical** | `rock_limestone.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **087** | `ore_copper_limestone` | Copper Ore (Limestone) | **Metal** | `rock_limestone.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **088** | `ore_lazurite_limestone` | Lazurite Ore (Limestone) | **Chemical** | `rock_limestone.png` | `ore_lazurite_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **089** | `ore_lead_limestone` | Lead Ore (Limestone) | **Heavy Metal** | `rock_limestone.png` | `ore_lead_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **090** | `ore_manganese_limestone` | Manganese Ore (Limestone) | **Heavy Metal** | `rock_limestone.png` | `ore_manganese_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **091** | `ore_saltpeter_limestone` | Saltpeter Ore (Limestone) | **Chemical** | `rock_limestone.png` | `ore_saltpeter_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **092** | `ore_zinc_limestone` | Zinc Ore (Limestone) | **Metal** | `rock_limestone.png` | `ore_zinc_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **093** | `ore_aluminum_chalk` | Aluminum Ore (Chalk) | **Metal** | `rock_chalk.png` | `ore_aluminum_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **094** | `ore_coal_chalk` | Coal Ore (Chalk) | **Chemical** | `rock_chalk.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **095** | `ore_copper_chalk` | Copper Ore (Chalk) | **Metal** | `rock_chalk.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **096** | `ore_lead_chalk` | Lead Ore (Chalk) | **Heavy Metal** | `rock_chalk.png` | `ore_lead_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **097** | `ore_manganese_chalk` | Manganese Ore (Chalk) | **Heavy Metal** | `rock_chalk.png` | `ore_manganese_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **098** | `ore_zinc_chalk` | Zinc Ore (Chalk) | **Metal** | `rock_chalk.png` | `ore_zinc_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **099** | `ore_aluminum_argillite` | Aluminum Ore (Argillite) | **Metal** | `rock_argillite.png` | `ore_aluminum_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **100** | `ore_coal_argillite` | Coal Ore (Argillite) | **Chemical** | `rock_argillite.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **101** | `ore_copper_argillite` | Copper Ore (Argillite) | **Metal** | `rock_argillite.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **102** | `ore_lead_argillite` | Lead Ore (Argillite) | **Heavy Metal** | `rock_argillite.png` | `ore_lead_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **103** | `ore_manganese_argillite` | Manganese Ore (Argillite) | **Heavy Metal** | `rock_argillite.png` | `ore_manganese_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **104** | `ore_zinc_argillite` | Zinc Ore (Argillite) | **Metal** | `rock_argillite.png` | `ore_zinc_overlay.png` | Uniform 6 faces (Shader Overlay) |

### 4.4. Volcanic & Pyroclastic Ores (16 Blocks)

| # | Block ID | Name | Category | Host Rock | Overlay Texture | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **105** | `ore_cinnabar_basalt` | Cinnabar Ore (Basalt) | **Chemical** | `rock_basalt.png` | `ore_cinnabar_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **106** | `ore_iron_basalt` | Iron Ore (Basalt) | **Metal** | `rock_basalt.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **107** | `ore_nickel_basalt` | Nickel Ore (Basalt) | **Heavy Metal** | `rock_basalt.png` | `ore_nickel_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **108** | `gem_opal_basalt` | Opal Ore (Basalt) | **Gemstone** | `rock_basalt.png` | `gem_opal_overlay.png` | Uniform 6 faces (Animated 16×64 Overlay) |
| **109** | `ore_platinum_basalt` | Platinum Ore (Basalt) | **Metal** | `rock_basalt.png` | `ore_platinum_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **110** | `gem_ruby_basalt` | Ruby Ore (Basalt) | **Gemstone** | `rock_basalt.png` | `gem_ruby_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **111** | `gem_sapphire_basalt` | Sapphire Ore (Basalt) | **Gemstone** | `rock_basalt.png` | `gem_sapphire_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **112** | `ore_sulfur_basalt` | Sulfur Ore (Basalt) | **Chemical** | `rock_basalt.png` | `ore_sulfur_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **113** | `ore_cinnabar_tuff` | Cinnabar Ore (Tuff) | **Chemical** | `rock_tuff.png` | `ore_cinnabar_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **114** | `ore_copper_tuff` | Copper Ore (Tuff) | **Metal** | `rock_tuff.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **115** | `gem_opal_tuff` | Opal Ore (Tuff) | **Gemstone** | `rock_tuff.png` | `gem_opal_overlay.png` | Uniform 6 faces (Animated 16×64 Overlay) |
| **116** | `ore_sulfur_tuff` | Sulfur Ore (Tuff) | **Chemical** | `rock_tuff.png` | `ore_sulfur_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **117** | `ore_iron_scoria` | Iron Ore (Scoria) | **Metal** | `rock_scoria.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **118** | `ore_nickel_scoria` | Nickel Ore (Scoria) | **Heavy Metal** | `rock_scoria.png` | `ore_nickel_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **119** | `ore_platinum_scoria` | Platinum Ore (Scoria) | **Metal** | `rock_scoria.png` | `ore_platinum_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **120** | `ore_sulfur_scoria` | Sulfur Ore (Scoria) | **Chemical** | `rock_scoria.png` | `ore_sulfur_overlay.png` | Uniform 6 faces (Shader Overlay) |

### 4.5. Metamorphic Foundation Ores (30 Blocks)

| # | Block ID | Name | Category | Host Rock | Overlay Texture | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **121** | `ore_coal_slate` | Coal Ore (Slate) | **Chemical** | `rock_slate.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **122** | `ore_copper_slate` | Copper Ore (Slate) | **Metal** | `rock_slate.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **123** | `gem_emerald_slate` | Emerald Ore (Slate) | **Gemstone** | `rock_slate.png` | `gem_emerald_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **124** | `ore_gold_slate` | Gold Ore (Slate) | **Metal** | `rock_slate.png` | `ore_gold_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **125** | `ore_lead_slate` | Lead Ore (Slate) | **Heavy Metal** | `rock_slate.png` | `ore_lead_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **126** | `ore_manganese_slate` | Manganese Ore (Slate) | **Heavy Metal** | `rock_slate.png` | `ore_manganese_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **127** | `gem_onyx_slate` | Onyx Ore (Slate) | **Gemstone** | `rock_slate.png` | `gem_onyx_overlay.png` | Uniform 6 faces (Animated 16×64 Overlay) |
| **128** | `ore_uranium_slate` | Uranium Ore (Slate) | **Heavy Metal** | `rock_slate.png` | `ore_uranium_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **129** | `gem_emerald_marble` | Emerald Ore (Marble) | **Gemstone** | `rock_marble.png` | `gem_emerald_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **130** | `ore_gold_marble` | Gold Ore (Marble) | **Metal** | `rock_marble.png` | `ore_gold_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **131** | `gem_jade_marble` | Jade Ore (Marble) | **Gemstone** | `rock_marble.png` | `gem_jade_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **132** | `ore_lazurite_marble` | Lazurite Ore (Marble) | **Chemical** | `rock_marble.png` | `ore_lazurite_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **133** | `gem_onyx_marble` | Onyx Ore (Marble) | **Gemstone** | `rock_marble.png` | `gem_onyx_overlay.png` | Uniform 6 faces (Animated 16×64 Overlay) |
| **134** | `gem_ruby_marble` | Ruby Ore (Marble) | **Gemstone** | `rock_marble.png` | `gem_ruby_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **135** | `gem_sapphire_marble` | Sapphire Ore (Marble) | **Gemstone** | `rock_marble.png` | `gem_sapphire_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **136** | `ore_silver_marble` | Silver Ore (Marble) | **Metal** | `rock_marble.png` | `ore_silver_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **137** | `ore_coal_gneiss` | Coal Ore (Gneiss) | **Chemical** | `rock_gneiss.png` | `ore_coal_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **138** | `ore_copper_gneiss` | Copper Ore (Gneiss) | **Metal** | `rock_gneiss.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **139** | `ore_iron_gneiss` | Iron Ore (Gneiss) | **Metal** | `rock_gneiss.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **140** | `ore_manganese_gneiss` | Manganese Ore (Gneiss) | **Heavy Metal** | `rock_gneiss.png` | `ore_manganese_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **141** | `ore_quartz_gneiss` | Quartz Ore (Gneiss) | **Chemical** | `rock_gneiss.png` | `ore_quartz_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **142** | `ore_silver_gneiss` | Silver Ore (Gneiss) | **Metal** | `rock_gneiss.png` | `ore_silver_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **143** | `ore_tin_gneiss` | Tin Ore (Gneiss) | **Metal** | `rock_gneiss.png` | `ore_tin_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **144** | `ore_zinc_gneiss` | Zinc Ore (Gneiss) | **Metal** | `rock_gneiss.png` | `ore_zinc_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **145** | `ore_copper_serpentinite` | Copper Ore (Serpentinite) | **Metal** | `rock_serpentinite.png` | `ore_copper_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **146** | `gem_diamond_serpentinite` | Diamond Ore (Serpentinite) | **Gemstone** | `rock_serpentinite.png` | `gem_diamond_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **147** | `ore_iron_serpentinite` | Iron Ore (Serpentinite) | **Metal** | `rock_serpentinite.png` | `ore_iron_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **148** | `gem_jade_serpentinite` | Jade Ore (Serpentinite) | **Gemstone** | `rock_serpentinite.png` | `gem_jade_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **149** | `ore_nickel_serpentinite` | Nickel Ore (Serpentinite) | **Heavy Metal** | `rock_serpentinite.png` | `ore_nickel_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **150** | `ore_platinum_serpentinite` | Platinum Ore (Serpentinite) | **Metal** | `rock_serpentinite.png` | `ore_platinum_overlay.png` | Uniform 6 faces (Shader Overlay) |

### 4.6. Subterranean & Geothermal Special Ores (8 Blocks)

| # | Block ID | Name | Category | Host Rock | Overlay Texture | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **151** | `ore_cinnabar_brimstone` | Cinnabar Ore (Brimstone) | **Chemical** | `rock_brimstone.png` | `ore_cinnabar_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **152** | `ore_sulfur_brimstone` | Sulfur Ore (Brimstone) | **Chemical** | `rock_brimstone.png` | `ore_sulfur_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **153** | `ore_lazurite_calcite` | Lazurite Ore (Calcite) | **Chemical** | `rock_calcite.png` | `ore_lazurite_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **154** | `gem_onyx_calcite` | Onyx Ore (Calcite) | **Gemstone** | `rock_calcite.png` | `gem_onyx_overlay.png` | Uniform 6 faces (Animated 16×64 Overlay) |
| **155** | `ore_cinnabar_karst` | Cinnabar Ore (Karst) | **Chemical** | `rock_karst.png` | `ore_cinnabar_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **156** | `ore_lead_karst` | Lead Ore (Karst) | **Heavy Metal** | `rock_karst.png` | `ore_lead_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **157** | `ore_saltpeter_karst` | Saltpeter Ore (Karst) | **Chemical** | `rock_karst.png` | `ore_saltpeter_overlay.png` | Uniform 6 faces (Shader Overlay) |
| **158** | `ore_zinc_karst` | Zinc Ore (Karst) | **Metal** | `rock_karst.png` | `ore_zinc_overlay.png` | Uniform 6 faces (Shader Overlay) |

---

---

## 05. Biomaterials, Fauna & Organic Matter (`organics`) — 34 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/organics`  
📄 **Comprehensive Specification**: [organics_design.md](blocks/organics_design.md)

The zoological and organic matter domain encompasses **14 fauna structures & animal nests** (colonial insect mounds, avian/poultry nests, burrows, and pupae), **6 anatomical tissues, organs & osteology blocks**, and **14 secretions, waxes, gels, droppings & modular spiderwebs** = **34 Total Organic Blocks**.

---

### 4.1. Fauna Architecture & Animal Nests (14 Blocks)

Structures engineered by colonial insects, avian species, mammals, and arachnids.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Biome Occurrence | Biological Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **01** | `ant_hill` | Ant Hill (Closed) | **Colony** | `nest_ant_hill.png` | Uniform (6 faces) | Plains, Savannas | Cemented mound of mineral soil particles and salivary secretions in dormant closed state. |
| **02** | `ant_hill_open` | Ant Hill (Active) | **Colony** | `nest_ant_hill_open.png` | Uniform (6 faces) | Plains, Savannas | Active ant mound featuring excavated gallery entryways and active insect swarms. |
| **03** | `termite_mound` | Termite Mound (Closed) | **Colony** | `nest_termite_mound.png` | Uniform (6 faces) | Arid Savannas | Towering spire of red clay soil cemented by termites with monolithic flute ridges. |
| **04** | `termite_mound_open`| Termite Mound (Active) | **Colony** | `nest_termite_mound_open.png` | Uniform (6 faces) | Arid Savannas | Active termite mound displaying open thermal chimneys and ventilation tunnels. |
| **05** | `bee_nest` | Wild Bee Nest | **Hive** | `nest_bee_nest_bot...top.png`<br>`nest_bee_nest_front_honey.png` | Directional (5 faces) | Flower Forests, Meadows | Natural hive hanging from tree canopies, filled with wax comb cells and dripping wild honey. |
| **06** | `wasp_nest` | Paper Wasp Nest | **Hive** | `nest_wasp_nest_bot...top.png` | Directional (3 faces) | Dense Forests, Ruins | Papery cellulose nest chewed from plant fibers by social wasps, with inverted entry pore. |
| **07** | `bird_nest` | Songbird Nest | **Nest** | `nest_bird_nest_bot...top.png` | Half-height Slab | Forest Canopies, Cliffs | Woven bowl of twigs and moss featuring speckled cyan robin eggs; half-height slab geometry. |
| **08** | `hay_nest` | Hay Poultry Nest | **Nest** | `nest_hay_nest_bot...top.png` | Half-height Slab | Barnyards, Farmsteads | Agricultural laying nest woven from golden cured straw and dry hay; half-height slab geometry. |
| **09** | `beaver_dam` | Beaver Dam | **Architecture** | `nest_beaver_dam.png` | Uniform (6 faces) | Forest Rivers, Swamps | Hydraulic blockage matrix woven from debarked river branches, mud mortar, and river gravel. |
| **10** | `spider_egg` | Spider Egg Sac | **Ootheca** | `nest_spider_egg_bot...top.png` | Directional (3 faces) | Deep Caverns, Mines | Dense fibrous silk ootheca enclosing clutches of gestating scarlet spider eggs. |
| **11** | `silk_cocoon` | Wild Silk Cocoon | **Pupa** | `nest_silk_cocoon_bot...top.png` | Directional (3 faces) | Temperate Woods, Jungles | Compact ellipsoidal pupal shell of continuous pearlescent silk fibers; premium silk harvest. |
| **12** | `loam_burrow` | Loam Animal Burrow | **Burrow** | Top: `nest_burrow_top.png` + `layer.png`<br>Bot: `soils/soil_loam_dirt.png`<br>Sides: `soils/soil_loam_grass_side.png` | Composite Top + Loam soil | Fertile Plains, Meadows | Excavated wildlife den in rich loam soil with grass-fringed aperture and earth depth. |
| **13** | `silt_burrow` | Silt Animal Burrow | **Burrow** | Top: `nest_burrow_top.png` + `layer.png`<br>Bot: `soils/soil_silt_dirt.png`<br>Sides: `soils/soil_silt_grass_side.png` | Composite Top + Silt soil | Woodland Hills, Slopes | Excavated wildlife den in fine silt forest soils with shaded earthen entry tunnel. |
| **14** | `peat_burrow` | Peat Animal Burrow | **Burrow** | Top: `nest_burrow_top.png` + `layer.png`<br>Bot: `soils/soil_peat_dirt.png`<br>Sides: `soils/soil_peat_grass_side.png` | Composite Top + Peat soil | Bogs, Moorlands, Fens | Subterranean burrow dug into dense dark acidic peat moorland, sheltering hardy marsh fauna. |

---

### 4.2. Living Tissues, Organs & Osteology (6 Blocks)

Biological anatomical materials harvested from animal carcasses or skeletal deposits.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Biome Occurrence | Biological Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **15** | `bone_block` | Bone Block | **Osteology** | `bio_bone_top.png`<br>`bio_bone_side.png` | Axial Column (2 faces) | Deserts, Fossils, Caves | Dense calcium phosphate and hydroxyapatite fossil megafauna bone with central marrow canal. |
| **16** | `calcined_bone` | Calcined Bone | **Osteology** | `bio_calcined_bone_top.png`<br>`bio_calcined_bone_side.png` | Axial Column (2 faces) | Volcanic Ash, Kilns, Catacombs | Thermally altered, pyromineralized bone recrystallized into chalky pale grey hydroxyapatite. |
| **17** | `flesh_block` | Raw Flesh Block | **Tissue** | `bio_flesh_top.png`<br>`bio_flesh_side.png` | Directional (2 faces) | Butcheries, Lairs, Carcasses | Fresh striated skeletal muscle and fibrous connective tissue with membrane top and fibril sides. |
| **18** | `blubber_block` | Blubber Block | **Adipose** | `bio_blubber.png` | Translucent (Alpha 204) | Polar Oceans, Arctic Tundras | Thick subcutaneous marine adipose tissue; excellent thermal insulator and long-burning fuel. |
| **19** | `brain_block` | Brain Block | **Neural** | `bio_brain.png` | Uniform (6 faces) | Abyssal Depths, Lairs | Convoluted cerebral gray and pink neural matter exhibiting synaptic twitching. |
| **20** | `fur_block` | Animal Fur Block | **Integument** | `bio_fur_top.png`<br>`bio_fur_side.png` | Directional (2 faces) | Arctic Taigas, Pastures | Heavy mammal hide with dense vertical hair flow on sides and whorled rosette on top. |

---

### 4.3. Secretions, Waxes, Gels, Droppings & Spiderwebs (14 Blocks)

Physiological secretions, insect products, biological wastes, and structural spider silk.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Biome Occurrence | Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **21** | `honey_block` | Honey Block | **Secretion** | `organic_honey_block_bot...top.png` | Translucent (3 faces) | Beehives, Apiaries | Viscous colloidal nectar concentrate; cushions kinetic impact and adheres to surfaces. |
| **22** | `honeycomb_block`| Honeycomb Block | **Wax Matrix** | `organic_honeycomb_block.png` | Uniform (6 faces) | Apiaries, Tree Cavities | Regular hexagonal prismatic matrix of pure insect wax synthesized by honeybee workers. |
| **23** | `wax_block` | Beeswax Block | **Wax** | `organic_wax.png` | Uniform (6 faces) | Domestic Hives, Workshops | Compact block of rendered virgin beeswax; waterproof sealant and architectural candle craft. |
| **24** | `slime_block` | Slime Block | **Colloid** | `organic_slime.png` | Translucent (Alpha 178) | Swamps, Deep Caves | Elastic green colloidal gel; high viscoelastic resilience providing kinetic bounce. |
| **25** | `jelly_block` | Royal Jelly Block | **Colloid** | `organic_jelly.png` | Translucent (Alpha 191) | Queen Chambers, Orchards | Viscoelastic translucent purple royal gel providing bounce without slime stickiness. |
| **26** | `guano_block` | Guano Block | **Dropping** | `organic_guano.png` | Uniform (6 faces) | Bat Caves, Sea Islands | Compact nitrogen and phosphate-rich natural fertilizer accumulated from bat and seabird guano. |
| **27** | `poop_block` | Manure Block | **Dropping** | `organic_poop.png` | Uniform (6 faces) | Pastures, Livestock Corrals | Rich agricultural farm animal dung with undigested vegetable fiber; high-potency soil fertilizer. |
| **28** | `coprolite_block`| Coprolite Block | **Fossil** | `organic_coprolite.png` | Uniform (6 faces) | Fossil Beds, Dry Caves | Prehistoric petrified fossil dung mineralized into solid calcium phosphate sedimentary rock. |
| **29** | `dust_block` | Organic Dust | **Detritus** | `organic_dust.png` | Uniform (6 faces) | Tombs, Forgotten Crypts | Dense accumulation of inert biological detritus, dead epithelial cells, and micro-arthropod debris. |
| **30** | `seashell_block`| Seashell Block | **Sediment** | `organic_seashell.png` | Uniform (6 faces) | Warm Beaches, Coastlines | Dense coastal coquina formed from crushed whole marine bivalves and iridescent sea shells. |
| **31** | `cobweb` | Cobweb Block | **Silk** | `organic_cobweb.png` | Volumetric 3D Cross | Abandoned Mines, Crypts | Full-voxel 3D intersecting cross of adhesive proteinaceous spider silk threads. |
| **32** | `cobweb_arch` | Cobweb Arch | **Silk** | `organic_cobweb_arch.png` | Flat Vertical Quad | Tunnel Corners, Doorways | Diagonal fan-shaped silk web mounted flush in the upper corner between wall and ceiling. |
| **33** | `cobweb_hanging`| Hanging Cobweb | **Silk** | `organic_cobweb_hanging.png` | Flat Vertical Quad | Tunnel Ceilings, Architraves | Pendant veil of cobweb fringe suspended from top horizontal joints and lintels. |
| **34** | `cobweb_stretched`| Stretched Cobweb | **Silk** | `organic_cobweb_stretched.png` | Flat Vertical Quad | Narrow Corridors, Gaps | Tensioned vertical silk web stretched taut between two parallel vertical surfaces. |

---

## 06. Decorations, Lighting & Architecture (`decorations`) — 160 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/decorations`  
📄 **Comprehensive Specification**: [decorations_design.md](blocks/decorations_design.md)

The architectural, utilitarian, and illumination domain comprises **6 utilitarian containers & structural furniture blocks**, **8 traditional pure masonry, textile, and glass blocks**, **128 dyed finishes in 16 colors** via shader tinting ($8 	imes 16$), **6 agricultural fibers & rural pathways**, and **12 radiant emitters & torches** = **160 Total Architectural Blocks**.

---

### 4.1. Utilitarian Containers & Structural Furniture (`util_`) (6 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `barrel` | Barrel | **Utility** | `util_barrel_top.png`<br>`util_barrel_side.png`<br>`util_barrel_bot.png` | Top top + Bottom bot + 4 sides side |
| **02** | `crate` | Crate | **Utility** | `util_crate_top.png`<br>`util_crate_side.png`<br>`util_crate_bot.png` | Top top + Bottom bot + 4 sides side |
| **03** | `basket` | Basket | **Utility** | `util_basket_top.png`<br>`util_basket_side.png`<br>`util_basket_bot.png` | Top top + Bottom bot + 4 sides side |
| **04** | `bookshelf` | Bookshelf | **Utility** | `util_bookshelf_top.png`<br>`util_bookshelf_side.png`<br>`util_bookshelf_front.png` | Top & Bottom top + 3 sides side + 1 front `util_bookshelf_front.png` |
| **05** | `bookshelf_empty` | Empty Bookshelf | **Utility** | `util_bookshelf_top.png`<br>`util_bookshelf_side.png`<br>`util_bookshelf_front_empty.png` | Top & Bottom top + 3 sides side + 1 front `util_bookshelf_front_empty.png` |
| **06** | `bag` | Burlap Bag | **Utility** | `util_bag_top.png`<br>`util_bag_side.png`<br>`util_bag_side_front.png`<br>`util_bag_bot.png` | Top top + Bottom bot + 3 sides side + 1 front `util_bag_side_front.png` |

---

### 4.2. Basic Pure Traditional Materials & Masonry (8 Original Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **07** | `bricks` | Bricks | **Masonry** | `deco_bricks.png` | 6 uniform faces |
| **08** | `linen` | Linen | **Textile** | `deco_linen.png` | 6 uniform faces |
| **09** | `fabric` | Fabric | **Textile** | `deco_fabric.png` | 6 uniform faces |
| **10** | `glass` | Glass | **Glasswork** | `deco_glass.png` | 6 uniform faces (Clear transparent) |
| **11** | `tinted_glass` | Tinted Glass | **Glasswork** | `deco_tinted_glass.png` | 6 uniform faces (Smoked semi-translucent) |
| **12** | `plaster` | Plaster | **Masonry** | `deco_plaster.png` | 6 uniform faces |
| **13** | `wool` | Wool | **Textile** | `deco_wool.png` | 6 uniform faces |
| **14** | `shingles` | Shingles | **Roofing** | `deco_shingles.png` | 6 uniform faces |

---

### 4.3. Dyed and Glazed Finishes in 16 Colors (128 Blocks)

Each of the 8 master materials multiplies its grayscale luminance texture by the 16 official dye colors via shader, generating **16 colored blocks per material** ($8 \times 16 = 128$ blocks):

> **16 Standard Dye Colors (IDs 1 to 16):**  
> `01. black` · `02. blue` · `03. brown` · `04. dark_blue` · `05. dark_grey` · `06. green` · `07. light_grey` · `08. light_pink` · `09. lime` · `10. orange` · `11. pink` · `12. purple` · `13. red` · `14. turquoise` · `15. white` · `16. yellow`

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **15–30** | `brick_<color>` | (Color) Bricks *(16 colors)* | **Masonry** | `deco_bricks_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **31–46** | `linen_<color>` | (Color) Linen *(16 colors)* | **Textile** | `deco_linen_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **47–62** | `fabric_<color>` | (Color) Fabric *(16 colors)* | **Textile** | `deco_fabric_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **63–78** | `glass_<color>` | (Color) Glass *(16 colors)* | **Glasswork** | `deco_glass_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **79–94** | `tinted_glass_<color>` | (Color) Tinted Glass *(16 colors)* | **Glasswork** | `deco_tinted_glass_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **95–110** | `plaster_<color>` | (Color) Plaster *(16 colors)* | **Masonry** | `deco_plaster_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **111–126** | `wool_<color>` | (Color) Wool *(16 colors)* | **Textile** | `deco_wool_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |
| **127–142** | `shingles_<color>` | (Color) Shingles *(16 colors)* | **Roofing** | `deco_shingles_painted.png` | 6 uniform faces (Shader Tint: colors 01–16) |

---

### 4.4. Agricultural Fibers & Rural Pathways (6 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **143** | `hay_bale` | Hay Bale | **Fibers** | `deco_hay_bale_top.png`<br>`deco_hay_bale_side.png` | Top & Bottom top + 4 sides side *(Orientable)* |
| **144** | `straw_bale` | Straw Bale | **Fibers** | `deco_straw_bale_top.png`<br>`deco_straw_bale_side.png` | Top & Bottom top + 4 sides side *(Orientable)* |
| **145** | `path_stony` | Stony Path | **Paths** | `deco_path_stony.png` | 6 uniform faces |
| **146** | `path_dirty` | Dirty Path | **Paths** | `deco_path_dirty.png` | 6 uniform faces |
| **147** | `path_sandy` | Sandy Path | **Paths** | `deco_path_sandy.png` | 6 uniform faces |
| **148** | `path_snowy` | Snowy Path | **Paths** | `deco_path_snowy.png` | 6 uniform faces |

---

### 4.5. Decorative Lighting & Radiant Emitters (`emit_`) (12 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **149** | `emit_lamp` | Lamp | **Lighting** | `emit_lamp.png`<br>`emit_lamp_top.png`<br>`emit_lamp_on.png`<br>`emit_lamp_top_on.png` | Top top + 4 sides + Bottom (`emit_lamp.png`) *(2 states: Normal / On)* |
| **150** | `emit_torch_fire` | Fire Torch | **Lighting** | `emit_torch_fire.png` | Vertical stem model *(5 animated fire frames)* |
| **151** | `emit_torch_cold` | Cold Torch | **Lighting** | `emit_torch_cold.png` | Vertical stem model *(Cold light 6500K)* |
| **152** | `emit_torch_warm` | Warm Torch | **Lighting** | `emit_torch_warm.png` | Vertical stem model *(Warm light 2700K)* |
| **153** | `emit_torch_red` | Red Torch | **Lighting** | `emit_torch_red.png` | Vertical stem model *(Ruby red diode)* |
| **154** | `emit_torch_green` | Green Torch | **Lighting** | `emit_torch_green.png` | Vertical stem model *(Emerald green diode)* |
| **155** | `emit_torch_blue` | Blue Torch | **Lighting** | `emit_torch_blue.png` | Vertical stem model *(Sapphire blue diode)* |
| **156** | `emit_light_cold` | Cold Light | **Radiant Emitter** | `emit_light_cold.png` | 6 uniform faces |
| **157** | `emit_light_warm` | Warm Light | **Radiant Emitter** | `emit_light_warm.png` | 6 uniform faces |
| **158** | `emit_light_red` | Red Light | **Radiant Emitter** | `emit_light_red.png` | 6 uniform faces |
| **159** | `emit_light_green` | Green Light | **Radiant Emitter** | `emit_light_green.png` | 6 uniform faces |
| **160** | `emit_light_blue` | Blue Light | **Radiant Emitter** | `emit_light_blue.png` | 6 uniform faces |

---

## 07. Botany: Flowers & Flowering Flora (`flowers`) — 32 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/flowers`  
📄 **Comprehensive Specification**: [flowers_design.md](blocks/flowers_design.md)

The flowering flora ecosystem encompasses **12 wild meadow & woodland flowers** (single-block cross-billboards), **4 tulip cultivars**, **10 double-tall flowers & flowering shrubs** (two-block composites), **4 ground cover petal carpets** (sub-voxel layering), and **2 arid flowering succulents** = **32 Total Flowering Blocks**.

---

### 3.1. Wild Meadow & Woodland Flowers (12 Blocks)

Single-voxel tall flowers rendered as intersecting cross-billboards (`#` shape). Feature procedural rotation and random visual variations.

| # | Block ID | Name | Category | Texture File(s) | Variations | Biome Occurrence | Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- | :--- |
| **01** | `flower_allium` | Allium | **Wildflower** | `flower_allium.png` to `3` | 4 | Prairies, Rocky Slopes | Spherical globose purple-magenta bloom on a tall, slender stem. |
| **02** | `flower_azure_bluet` | Azure Bluet | **Wildflower** | `flower_azure_bluet.png` to `3` | 4 | Alpine Meadows, Clearings | Tufted dainty perennial with 4-lobed sky-blue petals and yellow eye. |
| **03** | `flower_blue_orchid` | Blue Orchid | **Wildflower** | `flower_blue_orchid.png`, `1` | 2 | Swamps, Moist Floodplains | Exotic electric-cyan zygomorphic orchid thriving in humid marshlands. |
| **04** | `flower_cornflower` | Cornflower | **Wildflower** | `flower_cornflower.png` to `2` | 3 | Open Fields, Grasslands | Radiant deep-blue fringed ray florets, pioneer of sunlit meadows. |
| **05** | `flower_dandelion` | Dandelion | **Wildflower** | `flower_dandelion.png` to `3` | 4 | Temperate Plains, Forests | Vibrant golden-yellow composite head with deeply toothed basal leaves. |
| **06** | `flower_hibiscus` | Hibiscus | **Wildflower** | `flower_hibiscus.png` | 1 | Tropical Jungles, Coasts | Large tropical scarlet-red actinomorphic flower with yellow staminal column. |
| **07** | `flower_iris` | Iris | **Wildflower** | `flower_iris.png` | 1 | Riverbanks, Marsh Edges | Majestic purple-violet flower with drooping falls and upright standards. |
| **08** | `flower_lavender` | Lavender | **Wildflower** | `flower_lavender.png` | 1 | Mediterranean Hills, Dunes | Aromatic whorled lavender-purple spikes with silver-tinted foliage. |
| **09** | `flower_lily_of_the_valley` | Lily of the Valley | **Wildflower** | `flower_lily_of_the_valley.png` to `2` | 3 | Shaded Woodlands, Glades | Pure white, nodding bell-shaped scented blossoms along a curved stalk. |
| **10** | `flower_marigold` | Marigold | **Wildflower** | `flower_marigold.png`, `1` | 2 | Valleys, Sunlit Plains | Ruffled pom-pom double heads in radiant orange and amber tones. |
| **11** | `flower_oxeye_daisy` | Oxeye Daisy | **Wildflower** | `flower_oxeye_daisy.png` to `2` | 3 | Pastures, Grassy Meadows | Classic meadow daisy with bright white ray petals and golden disc center. |
| **12** | `flower_poppy` | Poppy | **Wildflower** | `flower_poppy.png` to `4` | 5 | Steppes, Dry Hillsides | Delicate paper-thin scarlet-red petals with a dark basal center spot. |

---

### 3.2. Tulip Cultivars (4 Blocks)

Symmetrical cup-shaped perennial blooms on upright stems. Found across sunny plains, temperate slopes, and garden beds.

| # | Block ID | Name | Category | Texture File(s) | Variations | Biome Occurrence | Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- | :--- |
| **13** | `flower_red_tulip` | Red Tulip | **Tulip** | `flower_red_tulip.png` to `2` | 3 | Temperate Plains, Valleys | Rich crimson-red chalice bloom on an erect stalk. |
| **14** | `flower_orange_tulip` | Orange Tulip | **Tulip** | `flower_orange_tulip.png` to `2` | 3 | Sunny Meadows, Savannas | Fiery amber-orange petals with warm golden throat tones. |
| **15** | `flower_white_tulip` | White Tulip | **Tulip** | `flower_white_tulip.png` to `2` | 3 | Cold Plains, Foothills | Immaculate porcelain-white petals with cool soft highlights. |
| **16** | `flower_pink_tulip` | Pink Tulip | **Tulip** | `flower_pink_tulip.png` to `2` | 3 | Flower Forests, Meadows | Delicate blush-pink petals fading gracefully to soft cream. |

---

### 3.3. Double-Tall Flowers & Flowering Shrubs (10 Blocks)

Two-block-high composite plants requiring both a root-anchored lower stem (`_bottom`) and a flowering canopy crown (`_top`).

| # | Block ID | Name | Category | Texture File(s) | Variations | Growth Habit | Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- | :--- |
| **17** | `flower_sunflower` | Sunflower | **Double-Tall** | `flower_sunflower_bottom.png`, `1`<br>`flower_sunflower_top.png`<br>`flower_sunflower_front.png`, `back.png` | 5 | Heliotropic Tall Stalk | Giant golden composite head that tracks the sun azimutally. |
| **18** | `flower_rose_bush` | Rose Bush | **Double-Tall** | `flower_rose_bush_bottom.png`, `1`<br>`flower_rose_bush_top.png` to `3` | 6 | Dense Thorny Shrub | Multi-stemmed shrub densely studded with velvety scarlet roses. |
| **19** | `flower_peony` | Peony | **Double-Tall** | `flower_peony_bottom.png`, `1`<br>`flower_peony_top.png`, `1` | 4 | Lush Bushy Perennial | Heavy ruffly globes of pastel-pink petals nestled among dark foliage. |
| **20** | `flower_lilac` | Lilac | **Double-Tall** | `flower_lilac_bottom.png`, `1`<br>`flower_lilac_top.png`, `1` | 4 | Woody Flowering Shrub | Upright woody shrub crowned with conical lilac-purple fragrant panicles. |
| **21** | `flower_begonia` | Begonia | **Double-Tall** | `flower_begonia_bottom.png`<br>`flower_begonia_top.png` | 2 | Tropical Understory | Succulent-stemmed sub-shrub with asymmetrical leaves and scarlet bloom clusters. |
| **22** | `flower_blue_hydrangea` | Blue Hydrangea | **Double-Tall** | `flower_blue_hydrangea_bottom.png`<br>`flower_blue_hydrangea_top.png` | 2 | Acid-Soil Shrub | Massive globose pompons of saturated cobalt-blue flowers. |
| **23** | `flower_goldenrod` | Goldenrod | **Double-Tall** | `flower_goldenrod_bottom.png`<br>`flower_goldenrod_top.png` | 2 | Tall Feathered Stalk | Arching feathered plumes of golden-yellow sprays swaying in the breeze. |
| **24** | `flower_snapdragon` | Snapdragon | **Double-Tall** | `flower_snapdragon_bottom.png`<br>`flower_snapdragon_top.png` | 2 | Dense Floral Spike | Upright spike of bilabiate blossoms gradienting from magenta to yellow. |
| **25** | `flower_gardenia` | Gardenia | **Double-Tall** | `flower_gardenia_bottom.png`<br>`flower_gardenia_top.png` | 2 | Glossy Evergreen Bush | Glossy dark leaves bearing intensely fragrant waxy ivory-white flowers. |
| **26** | `flower_foxglove` | Foxglove | **Double-Tall** | `flower_foxglove_bottom.png`<br>`flower_foxglove_top.png` | 2 | Tall Woodland Spike | Towering stately raceme of pendulous tubular spotted pink bells. |

---

### 3.4. Ground Cover & Petal Carpets (4 Blocks)

Low-lying sub-voxel floral carpets spread horizontally across soil, grass, or stone surfaces.

| # | Block ID | Name | Category | Texture File(s) | Density Stages | Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- |
| **27** | `flower_pink_petals` | Pink Petals | **Carpet** | `flower_pink_petals.png`<br>`flower_pink_petals_stem.png` | 1–4 layers | Scattered wind-blown cherry blossom petals accumulating on terrain. |
| **28** | `flower_white_petals` | White Petals | **Carpet** | `flower_white_petals.png`<br>`flower_white_petals_stem.png` | 1–4 layers | Soft snowfall of pale deciduous flower petals dusting the forest floor. |
| **29** | `flower_wildflowers` | Wildflowers Carpet | **Carpet** | `flower_wildflowers.png`, `1`<br>`flower_wildflowers_stem.png` | Multi-carpet | Dense multi-colored carpet of tiny creeping flowers covering pasture grass. |
| **30** | `flower_clover` | Clover Carpet | **Carpet** | `flower_clover.png`<br>`flower_clover_stem.png` | Multi-carpet | Ground-hugging mat of trifoliate green clover with occasional white florets. |

---

### 3.5. Arid Flora & Flowering Succulents (2 Blocks)

Drought-tolerant specialized flora thriving in arid deserts, dry mesas, and rocky badlands.

| # | Block ID | Name | Category | Texture File(s) | Growth Habit | Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- |
| **31** | `flower_cactus_flower` | Cactus Flower | **Arid Flora** | `flower_cactus_flower.png` | Stem Crest Perch | Bright yellow multi-petaled bloom perching atop saguaro cactus ribs. |
| **32** | `flower_imperial_succulent` | Imperial Succulent | **Arid Flora** | `flower_imperial_succulent.png` | Desert Rosette | Symmetrical geometric rosette of fleshy blue-green leaves with an erect bloom. |

---

---

## 08. Botany: Groundcover & Herbaceous Flora (`vegetation`) — 38 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/vegetation`  
📄 **Comprehensive Specification**: [vegetation_design.md](blocks/vegetation_design.md)

The groundcover and herbaceous layer encompasses **12 grasses & wild cereals** (6 single carpets + 6 double-tall prairie grasses and grain stalks), **6 ferns & pteridophytes**, **8 shrubs, bushes & cacti** (including 3D columnar saguaro), **4 wetlands & floating pads**, **4 epiphytes, climbers & root hangings**, and **4 subterranean cave flora** = **38 Total Groundcover Blocks**.

---

### 3.1. Grasses & Wild Cereals (12 Blocks)

| # | Block ID | Name | Category | Primary Texture(s) | Variations | Biome & Natural Occurrence | Geometry Profile |
| :-: | :--- | :--- | :--- | :--- | :---: | :--- | :--- |
| **01** | `vege_wildgrass` | Wildgrass | **Grass** | `vege_wildgrass.png` | 4 (`_`, `1`, `2`, `3`) | Plains, Meadows & Temperate Forests | Single Cross-Mesh |
| **02** | `vege_drygrass` | Drygrass | **Grass** | `vege_drygrass.png` | 4 (`_`, `1`, `2`, `3`) | Savannas, Steppes & Scrublands | Single Cross-Mesh |
| **03** | `vege_frigid_grass` | Frigid Grass | **Grass** | `vege_frigid_grass.png` | 2 (`_`, `1`) | Arctic Tundras, Taigas & Permafrost | Single Cross-Mesh |
| **04** | `vege_bluff_grass` | Bluff Grass | **Grass** | `vege_bluff_grass.png` | 3 (`_`, `1`, `2`) | Rocky Crags, Cliffs & Windblown Ridges | Single Cross-Mesh |
| **05** | `vege_mossgrass` | Mossgrass | **Grass** | `vege_mossgrass.png` | 3 (`_`, `1`, `2`) | Damp Woodlands, River Glades & Bogs | Low Dense Tuft |
| **06** | `vege_tall_wildgrass` | Tall Wildgrass | **Tall Grass** | `vege_tall_wildgrass_bottom.png`<br>`vege_tall_wildgrass_top.png` | 4 pairs | High Meadows & Lush River Valleys | Double-Tall (2 Blocks) |
| **07** | `vege_tall_prairie_grass` | Tall Prairie Grass | **Tall Grass** | `vege_tall_prairie_grass_bottom.png`<br>`vege_tall_prairie_grass_top.png` | 1 pair | Central Prairies, Steppes & Pampas | Double-Tall (2 Blocks) |
| **08** | `vege_tall_beach_grass` | Tall Beach Grass | **Tall Grass** | `vege_tall_beach_grass_bottom.png`<br>`vege_tall_beach_grass_top.png` | 1 pair | Coastal Sand Dunes & Shorelines | Double-Tall (2 Blocks) |
| **09** | `vege_sea_oats` | Sea Oats | **Tall Grass** | `vege_sea_oats_bottom.png`<br>`vege_sea_oats_top.png` | 1 pair | Subtropical Coasts & Dune Crests | Double-Tall (2 Blocks) |
| **10** | `vege_barley` | Wild Barley | **Wild Cereal** | `vege_barley_bottom.png`<br>`vege_barley_top.png` | 1 bottom, 3 tops | Alluvial Valleys & Sunlit Hillsides | Double-Tall (2 Blocks) |
| **11** | `vege_tall_bluff_grass` | Tall Bluff Grass | **Tall Grass** | `vege_tall_bluff_grass_bottom.png`<br>`vege_tall_bluff_grass_top.png` | 3 pairs | Mountain Bluffs & Escarpments | Double-Tall (2 Blocks) |
| **12** | `vege_pampas_grass` | Pampas Grass | **Tall Grass** | `vege_pampas_grass_bottom.png`<br>`vege_pampas_grass_top.png` | 2 bottoms, 1 top | Subtropical Plains & Pampas Fields | Double-Tall Plume (2 Blocks) |

---

### 3.2. Ferns & Pteridophytes (6 Blocks)

| # | Block ID | Name | Category | Primary Texture(s) | Variations | Biome & Natural Occurrence | Geometry Profile |
| :-: | :--- | :--- | :--- | :--- | :---: | :--- | :--- |
| **13** | `vege_fern` | Common Fern | **Fern** | `vege_fern.png` | 4 (`_`, `1`, `2`, `3`) | Shaded Woodlands & Taiga Understory | Single Cross-Mesh |
| **14** | `vege_fern_large` | Large Fern | **Fern** | `vege_fern_large.png` | 1 | Temperate Rainwoods & Humid Glades | Robust Cross-Mesh |
| **15** | `vege_fern_large_snowy` | Snowy Large Fern | **Fern** | `vege_fern_large_snowy.png` | 1 | Cold Taigas & Glacial Forest Floors | Snowy Cross-Mesh |
| **16** | `vege_bracken` | Bracken Fern | **Fern** | `vege_bracken.png` | 1 | Temperate Moorlands, Heaths & Woods | Spreading Frond Mesh |
| **17** | `vege_tall_fern` | Tall Fern | **Tall Fern** | `vege_tall_fern_bottom.png`<br>`vege_tall_fern_top.png` | 4 pairs | Ancient Forests & Mountain Ravines | Double-Tall (2 Blocks) |
| **18** | `vege_tall_jungle_fern` | Tall Jungle Fern | **Tall Fern** | `vege_tall_jungle_fern_bottom.png`<br>`vege_tall_jungle_fern_top.png` | 1 pair | Tropical Rainforests & Equatorial Valleys | Double-Tall (2 Blocks) |

---

### 3.3. Shrubs, Bushes & Cacti (8 Blocks)

| # | Block ID | Name | Category | Primary Texture(s) | Variations | Biome & Natural Occurrence | Geometry Profile |
| :-: | :--- | :--- | :--- | :--- | :---: | :--- | :--- |
| **19** | `vege_bush` | Temperate Bush | **Shrub** | `vege_bush.png` | 4 (`_`, `1`, `2`, `3`) | Forest Edges, Meadows & Foothills | Dense Rounded Mesh |
| **20** | `vege_red_shrub` | Red Shrub | **Shrub** | `vege_red_shrub.png` | 2 (`_`, `1`) | Autumn Woods, Acid Peat & Moors | Red Pigmented Mesh |
| **21** | `vege_dead_bush` | Dead Bush | **Shrub** | `vege_dead_bush.png` | 4 (`_`, `1`, `2`, `3`) | Arid Deserts, Badlands & Frozen Tundras | Withered Woody Sticks |
| **22** | `vege_cactus_bush` | Cactus Bush | **Cactus** | `vege_cactus_bush.png` | 2 (`_`, `1`) | Warm Deserts, Scrub & Chaparrals | Spiny Cross-Mesh |
| **23** | `vege_barrel_cactus` | Barrel Cactus | **Cactus** | `vege_barrel_cactus.png` | 2 (`_`, `1`) | Dry Desert Washes & Rocky Arid Flats | Globular Succulent Mesh |
| **24** | `vege_prickly_pear_cactus` | Prickly Pear Cactus | **Cactus** | `vege_prickly_pear_cactus.png` | 1 | Semiarid Slopes, Savannas & Scrub | Fleshy Pad Cross-Mesh |
| **25** | `vege_small_cactus` | Small Column Cactus | **Cactus** | `vege_small_cactus_top.png`<br>`vege_small_cactus_side.png`<br>`vege_small_cactus_bottom.png` | 1 | Rocky Deserts & Gravelly Flats | 3D Compact Column Block |
| **26** | `vege_cactus` | Saguaro Cactus | **Cactus** | `vege_cactus_top.png`<br>`vege_cactus_side.png`<br>`vege_cactus_bottom.png` | 1 | Hot Sand Deserts & Arid Dunes | 3D Full Column Block (Thorns) |

---

### 3.4. Wetlands, Aquatics & Floating Pads (4 Blocks)

| # | Block ID | Name | Category | Primary Texture(s) | Variations | Biome & Natural Occurrence | Geometry Profile |
| :-: | :--- | :--- | :--- | :--- | :---: | :--- | :--- |
| **27** | `vege_sugar_cane` | Sugar Cane | **Wetland** | `vege_sugar_cane.png` | 5 (`_` to `4`) | Riverbanks, Lakeshores & Swamp Edges | Water-Adjacent Reeds |
| **28** | `vege_cattail` | Cattail | **Wetland** | `vege_cattail_bottom.png`<br>`vege_cattail_top.png` | 1 pair | Muddy Marshes, Swamps & Shallows | Double-Tall Emergent (2 Blocks) |
| **29** | `vege_lily_pad` | Water Lily Pad | **Floating Pad** | `vege_lily_pad.png` | 5 (`_` to `4`) | Calm Lakes, Swamps & River Pools | Floating Water Surface Mesh |
| **30** | `vege_helvola_pad` | Pygmy Helvola Pad | **Floating Pad** | `vege_helvola_pad.png` | 1 | Jungle Springs, Ponds & Estuary Backwaters | Miniature Floating Pad |

---

### 3.5. Epiphytes, Climbers & Root Hangings (4 Blocks)

| # | Block ID | Name | Category | Primary Texture(s) | Variations | Biome & Natural Occurrence | Geometry Profile |
| :-: | :--- | :--- | :--- | :--- | :---: | :--- | :--- |
| **31** | `vege_vines` | Vines (Lianas) | **Epiphyte** | `vege_vines.png` | 3 (`_`, `1`, `2`) | Rainforest Canopies & Cliff Walls | Vertical Wall Climber |
| **32** | `vege_hanging_moss` | Hanging Moss | **Epiphyte** | `vege_hanging_moss.png` | 1 | Murky Swamps & Gallery Forests | Ceiling Hanging Decal |
| **33** | `vege_hanging_roots` | Hanging Roots | **Epiphyte** | `vege_hanging_roots.png` | 2 (`_`, `1`) | Dirt Overhangs, Ravines & Cave Ceilings | Ceiling Root Danglers |
| **34** | `vege_lichen` | Tree & Rock Lichen | **Epiphyte** | `vege_lichen.png` | 3 (`_`, `1`, `2`) | Humid Barks, Damp Rocks & Tundra Boulders | Crustose Surface Patch |

---

### 3.6. Subterranean Cave Flora (4 Blocks)

| # | Block ID | Name | Category | Primary Texture(s) | Variations | Biome & Natural Occurrence | Geometry Profile |
| :-: | :--- | :--- | :--- | :--- | :---: | :--- | :--- |
| **35** | `vege_cave_growths` | Cave Growths | **Cave Flora** | `vege_cave_growths.png` | 1 | Damp Subterranean Caverns & Chasms | Ground Umbrophilic Thallus |
| **36** | `vege_grainy_cave_growths` | Grainy Cave Growths | **Cave Flora** | `vege_grainy_cave_growths.png` | 1 | Mineral-Rich Subterranean Walls & Caves | Ferruginous Granular Crust |
| **37** | `vege_lurid_cave_growths` | Lurid Cave Growths | **Cave Flora** | `vege_lurid_cave_growths.png` | 1 | Deep Abyssal Caverns & Sunken Grottoes | Bioluminescent Spectral Moss |
| **38** | `vege_cave_hyssop` | Cave Hyssop | **Cave Flora** | `vege_cave_hyssop.png` | 1 | Geodes, Crystal Caverns & Deep Vaults | Subterranean Flowering Herb |

---

## 09. Mycology: Mushrooms (`mushrooms`) — 18 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/mushrooms`  
📄 **Comprehensive Specification**: [mushrooms_design.md](blocks/mushrooms_design.md)

The mycological domain encompasses **8 wild understory ground mushrooms** (cross-billboards with procedural variations), **8 giant macrofungal caps** (`top`/`side` cap with `bottom` porous gills), and **2 shared macrofungal structural blocks** (`stem` stipe column and `inside` flesh) = **18 Total Mycological Blocks**.

---

### 3.1. Wild Understory Ground Mushrooms (8 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `fungi_amanita` | Amanita (Fly Agaric) | **Ground Mushroom** | `fungi_amanita.png` *(+ 4 variants: `fungi_amanita1.png` to `4`)* | Cross-billboard model (2 intersecting vertical planes) |
| **02** | `fungi_porcini` | Porcini (King Bolete) | **Ground Mushroom** | `fungi_porcini.png` *(+ 4 variants: `fungi_porcini1.png` to `4`)* | Cross-billboard model (2 intersecting vertical planes) |
| **03** | `fungi_shiitake` | Shiitake | **Ground Mushroom** | `fungi_shiitake.png`, `fungi_shiitake1.png` *(2 variants)* | Cross-billboard model (2 intersecting vertical planes) |
| **04** | `fungi_russula` | Russula (Sickener) | **Ground Mushroom** | `fungi_russula.png`, `fungi_russula1.png` *(2 variants)* | Cross-billboard model (2 intersecting vertical planes) |
| **05** | `fungi_verdigris` | Verdigris Agaric | **Ground Mushroom** | `fungi_verdigris.png`, `fungi_verdigris1.png` *(2 variants)* | Cross-billboard model (2 intersecting vertical planes) |
| **06** | `fungi_chanterelle` | Golden Chanterelle | **Ground Mushroom** | `fungi_chanterelle.png`, `fungi_chanterelle1.png` *(2 variants)* | Cross-billboard model (2 intersecting vertical planes) |
| **07** | `fungi_mycena` | Mycena (Bioluminescent) | **Ground Mushroom** | `fungi_mycena.png`, `fungi_mycena1.png` *(2 variants)* | Cross-billboard model (2 intersecting vertical planes) |
| **08** | `fungi_indigo` | Indigo Milk Cap | **Ground Mushroom** | `fungi_indigo.png`, `fungi_indigo1.png` *(2 variants)* | Cross-billboard model (2 intersecting vertical planes) |

---

### 3.2. Giant Macrofungal Caps (8 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **09** | `fungi_amanita_cap` | Amanita Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_amanita_cap.png`<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **10** | `fungi_porcini_cap` | Porcini Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_porcini_cap.png`<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **11** | `fungi_shiitake_cap` | Shiitake Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_shiitake_cap.png`<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **12** | `fungi_russula_cap` | Russula Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_russula_cap.png`<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **13** | `fungi_verdigris_cap` | Verdigris Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_verdigris_cap.png`<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **14** | `fungi_chanterelle_cap` | Chanterelle Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_chanterelle_cap.png`<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **15** | `fungi_mycena_cap` | Mycena Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_mycena_cap.png`<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **16** | `fungi_indigo_cap` | Indigo Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_indigo_cap.png`<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |

---

### 3.3. Shared Macrofungal Structural Blocks (2 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **17** | `fungi_stem` | Fungi Stem | **Macrofungal Structure** | `fungi_stem.png` | 6 uniform faces |
| **18** | `fungi_inside` | Fungi Inside Flesh | **Macrofungal Structure** | `fungi_inside.png` | 6 uniform faces |

---

## 10. Hydrosphere & Cryology (`fluids`) — 18 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/fluids`  
📄 **Comprehensive Specification**: [fluids_design.md](blocks/fluids_design.md)

The hydrosphere, rheology, and cryosphere domain comprises **8 cryogenic ice blocks** (clear, fragile, packed, black, and blue varieties) and **10 dynamic physical liquids** (each backed by static block icon, still surface animation strip, and cascading flow strip) = **18 Total Fluid & Ice Blocks**.

---

### 3.1. Cryology and Ice Blocks (8 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `frost_ice` | Clear Ice | **Cryology** | `frost_ice.png` | 6 uniform faces *(Translucent, alpha=153)* |
| **02** | `frost_ice_fragile` | Fragile Ice | **Cryology** | `frost_ice_fragile.png` | 6 uniform faces *(Translucent with subtle hairline cracks)* |
| **03** | `frost_ice_packed` | Packed Ice | **Cryology** | `frost_ice_packed.png` | 6 uniform faces *(100% Opaque solid firn ice)* |
| **04** | `frost_ice_black` | Black Ice | **Cryology** | `frost_ice_black.png` | 6 uniform faces *(Translucent abyssal dark, alpha=160)* |
| **05** | `frost_ice_black_fragile` | Fragile Black Ice | **Cryology** | `frost_ice_black_fragile.png` | 6 uniform faces *(Translucent dark with bold cleavage fractures)* |
| **06** | `frost_ice_black_packed` | Packed Black Ice | **Cryology** | `frost_ice_black_packed.png` | 6 uniform faces *(100% Opaque solid dark permafrost)* |
| **07** | `frost_blue_ice` | Blue Ice | **Cryology** | `frost_blue_ice.png` | 6 uniform faces *(100% Opaque deep cobalt glacial)* |
| **08** | `frost_blue_ice_cracked` | Cracked Blue Ice | **Cryology** | `frost_blue_ice_cracked.png` | 6 uniform faces *(100% Opaque cellular frost veins & bubbles)* |

---

### 3.2. Dynamic Liquids & Fluids (10 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **09** | `liquid_water` | Water | **Aquatic** | Block: `liquid_water.png`<br>Still: `liquid_water_still.png`<br>Flow: `liquid_water_flow.png` | Top: Still animated strip *(Dynamic biome tint)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **10** | `liquid_spring_water` | Spring Water | **Aquatic** | Block: `liquid_spring_water.png`<br>Still: `liquid_spring_water_still.png`<br>Flow: `liquid_spring_water_flow.png` | Top: Still animated strip *(Turquoise mineral)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **11** | `liquid_lava` | Lava | **Magmatic** | Block: `liquid_lava.png`<br>Still: `liquid_lava_still.png`<br>Flow: `liquid_lava_flow.png` | Top: Still animated strip *(1000°C–1200°C basaltic)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **12** | `liquid_core_magma` | Core Magma | **Magmatic** | Block: `liquid_core_magma.png`<br>Still: `liquid_core_magma_still.png`<br>Flow: `liquid_core_magma_flow.png` | Top: Still animated strip *(Deep mantle incandescent)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **13** | `liquid_molten` | Molten Slag | **Magmatic** | Block: `liquid_molten.png`<br>Still: `liquid_molten_still.png`<br>Flow: `liquid_molten_flow.png` | Top: Still animated strip *(Liquefied metallurgical metal)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **14** | `liquid_acid` | Acid | **Chemical** | Block: `liquid_acid.png`<br>Still: `liquid_acid_still.png`<br>Flow: `liquid_acid_flow.png` | Top: Still animated strip *(Neon green corrosive)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **15** | `liquid_blood` | Blood | **Biological** | Block: `liquid_blood.png`<br>Still: `liquid_blood_still.png`<br>Flow: `liquid_blood_flow.png` | Top: Still animated strip *(Crimson arterial)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **16** | `liquid_ooze` | Ooze | **Biological** | Block: `liquid_ooze.png`<br>Still: `liquid_ooze_still.png`<br>Flow: `liquid_ooze_flow.png` | Top: Still animated strip *(Olive green bubbling slime)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **17** | `liquid_sludge` | Sludge | **Alluvial Mire** | Block: `liquid_sludge.png`<br>Still: `liquid_sludge_still.png`<br>Flow: `liquid_sludge_flow.png` | Top: Still animated strip *(Dark brown muddy mire)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |
| **18** | `liquid_tar` | Tar | **Bituminous** | Block: `liquid_tar.png`<br>Still: `liquid_tar_still.png`<br>Flow: `liquid_tar_flow.png` | Top: Still animated strip *(Viscous black bitumen, alpha=216)*<br>Sides/Falling: Flow animated strip<br>Inventory/Particle: Block icon |

---

## 11. Ocean & Benthic Ecosystems (`oceans`) — 36 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/oceans`  
📄 **Comprehensive Specification**: [oceans_design.md](blocks/oceans_design.md)

The marine benthic domain encompasses **12 living massive aragonite coral blocks**, **12 living coral fans** (cross-quad billboards), **2 dead calcified coral formations**, **8 benthic structures & foundational substrates** (sponges, anemones, vents, algae mats, and bedrocks), and **2 subaquatic pelagic animated flora** (kelp and seagrass) = **36 Total Oceanic Blocks**.

---

### 4.1. Living Massive Coral Blocks (12 Blocks)

Solid colonial anthozoan blocks composed of aragonite skeletons populated by photosynthetic zooxanthellae. Form the primary biological framework of tropical reefs.

| # | Block ID | Name | Category | Texture File(s) | Variations | Biome & Depth | Biological Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- | :--- |
| **01** | `tube_coral` | Tube Coral | **Massive Coral** | `aqua_tube_coral.png` | 1 | Shallow Tropical Seas | Dense cylindrical tubular aragonite corallites in vibrant cobalt blue. |
| **02** | `brain_coral` | Brain Coral | **Massive Coral** | `aqua_brain_coral.png` | 1 | Shallow Warm Lagoons | Hemispherical massive colony with convoluted, meandering ridges in rose pink. |
| **03** | `bubble_coral` | Bubble Coral | **Massive Coral** | `aqua_bubble_coral.png` | 1 | Sunlit Reef Slopes | Inflated vesicular water-filled polyps of vivid violet-purple coloration. |
| **04** | `fire_coral` | Fire Coral | **Massive Coral** | `aqua_fire_coral.png` | 1 | Outer Reef Crests | Calcified hydrocoral in bright scarlet with stinging defensive nematocysts. |
| **05** | `horn_coral` | Horn Coral | **Massive Coral** | `aqua_horn_coral.png` | 1 | Clear Tropical Waters | Upright columnar gold-yellow corallum reaching toward intense sunlight. |
| **06** | `staghorn_coral` | Staghorn Coral | **Massive Coral** | `aqua_staghorn_coral.png` | 1 | Barrier Reef Flats | Fast-growing emerald-green branching acroporid structure with angular antlers. |
| **07** | `heart_coral` | Heart Coral | **Massive Coral** | `aqua_heart_coral.png` | 1 | Tropical Reef Drop-offs | Massive calcium colony displaying intense crimson-magenta polyps. |
| **08** | `leaf_coral` | Leaf Coral | **Massive Coral** | `aqua_leaf_coral.png` | 1 | Calm Sheltered Lagoons | Foliaceous ruffled plates and wavy ridges rich in olive-green pigments. |
| **09** | `reed_coral` | Reed Coral | **Massive Coral** | `aqua_reed_coral.png` | 1 | Mid-Depth Patch Reefs | Articulated vertical clustered stems in deep indigo and violet tones. |
| **10** | `spike_coral` | Spike Coral | **Massive Coral** | `aqua_spike_coral.png` | 1 | Deep Reef Margins | Heavy skeletal base with prominent conical spires in cyan-turquoise hues. |
| **11** | `thorn_coral` | Thorn Coral | **Massive Coral** | `aqua_thorn_coral.png` | 1 | Deep Trenches & Sea Caves | Antipatharian black coral with a tough, thorny obsidian-colored chitin skeleton. |
| **12** | `wax_coral` | Wax Coral | **Massive Coral** | `aqua_wax_coral.png` | 1 | Calcareous Reef Plateaus | Smooth, pale ivory-white colonial formation with a velvety waxy finish. |

---

### 4.2. Living Coral Fans (12 Blocks)

Delicate branching gorgonians and soft corals anchored atop reef blocks. Rendered as cross-quad billboards swaying with ambient water currents.

| # | Block ID | Name | Category | Texture File(s) | Variations | Mount Surface | Morphological Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- | :--- |
| **13** | `tube_coral_fan` | Tube Coral Fan | **Coral Fan** | `vege_tube_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Branching tubular cobalt-blue fan anchored perpendicularly to reef strata. |
| **14** | `brain_coral_fan` | Brain Coral Fan | **Coral Fan** | `vege_brain_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Compact wavy pink frilled fan extending into tidal currents. |
| **15** | `bubble_coral_fan` | Bubble Coral Fan | **Coral Fan** | `vege_bubble_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Clustered delicate purple vesicles on a thin flexible calcareous stem. |
| **16** | `fire_coral_fan` | Fire Coral Fan | **Coral Fan** | `vege_fire_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Upright fiery scarlet branching fan thriving in energetic breaker zones. |
| **17** | `horn_coral_fan` | Horn Coral Fan | **Coral Fan** | `vege_horn_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Slender yellow antlers spreading outward to catch drifting micro-nutrients. |
| **18** | `staghorn_coral_fan` | Staghorn Coral Fan | **Coral Fan** | `vege_staghorn_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Intricate green digitiform branching fan mimicking juvenile staghorns. |
| **19** | `heart_coral_fan` | Heart Coral Fan | **Coral Fan** | `vege_heart_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Broad crimson-pink fan displaying gentle lobed contours. |
| **20** | `leaf_coral_fan` | Leaf Coral Fan | **Coral Fan** | `vege_leaf_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Leaf-like ruffled olive-green fronds undulating with ocean swell. |
| **21** | `reed_coral_fan` | Reed Coral Fan | **Coral Fan** | `vege_reed_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Vertical violet spires and delicate reedy branches rising from a basal cup. |
| **22** | `spike_coral_fan` | Spike Coral Fan | **Coral Fan** | `vege_spike_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Radiant turquoise thorny fan with sharp needle-pointed tips. |
| **23** | `thorn_coral_fan` | Thorn Coral Fan | **Coral Fan** | `vege_thorn_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Delicate black obsidian fan armed with minute microscopic thorns. |
| **24** | `wax_coral_fan` | Wax Coral Fan | **Coral Fan** | `vege_wax_coral_fan.png`, `fan1.png` | 2 | Top of Coral Blocks | Translucent ivory-white fan with velvety waxy texture. |

---

### 4.3. Dead Calcified Coral Formations (2 Blocks)

Bleached, fossilized remnants of destroyed or aged coral reefs that have lost their living symbionts.

| # | Block ID | Name | Category | Texture File(s) | Variations | Occurrence | Geological Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- | :--- |
| **25** | `dead_coral` | Dead Coral Block | **Dead Coral** | `aqua_dead_coral.png` | 1 | Bleached Reefs, Sunken Atolls | Porous, chalky mineralized calcium carbonate block in ghostly grey. |
| **26** | `dead_coral_fan` | Dead Coral Fan | **Dead Coral** | `vege_dead_coral_fan.png`, `fan1.png` | 2 | Top of Dead Reefs | Brittle, desaturated skeletal fan with chalky grey limestone texture. |

---

### 4.4. Benthic Structures & Substrates (8 Blocks)

Sessile marine fauna, thermal geological vents, and foundational oceanic bedrocks supporting benthic biodiversity.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Biome Occurrence | Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- | :--- |
| **27** | `sponge` | Natural Sponge | **Poriferan** | `aqua_sponge.png` | Uniform (6 faces) | Deep Warm Reefs | Multicellular sessile poriferan with porous water-filtration canals. |
| **28** | `algae_mat` | Algae Mat | **Biofilm** | `aqua_algae_mat_top.png`<br>`aqua_algae_mat_side.png` | Top/Side split | Intertidal Shores, Docks | Dense green biofilm of microalgae and bryophytes coating coastal stone. |
| **29** | `geothermal_vent` | Geothermal Vent | **Hydrothermal** | `aqua_geothermal_vent_top.png`<br>`aqua_geothermal_vent_side.png` | Top/Side split | Abyssal Trenches | Submarine volcanic chimney venting superheated mineral-rich water. |
| **30** | `sea_anemone` | Sea Anemone | **Anthozoan** | `aqua_sea_anemone.png`, `1.png` | Cross-billboard | Reef Walls, Tide Pools | Predatory sessile polyp with pedal disc and ring of stinging tentacles. |
| **31** | `coralstone` | Coralstone | **Biogenic Rock** | `aqua_coralstone.png` | Uniform (6 faces) | Shallow Reefs, Sandbars | Porous light-colored calcarenite formed from consolidated coral fragments. |
| **32** | `reefstone` | Reefstone | **Volcanic Rock** | `aqua_reefstone.png` | Uniform (6 faces) | Reef Foundations | Dense, dark basaltic bedrock providing geological support for coral reefs. |
| **33** | `algae_reefstone` | Algae Reefstone | **Encrusted Rock** | `aqua_reefstone_algae.png` | Uniform (6 faces) | Sunlit Reef Slopes | Dark reefstone heavily coated with vibrant green marine algal biofilm. |
| **34** | `barnacles` | Barnacles | **Crustacean** | `aqua_reefstone_barnacles.png` | Uniform (6 faces) | Rocky Coasts, Shipwrecks | Compact colonial crust of conical calcified acorn barnacles. |

---

### 4.5. Subaquatic Pelagic Flora (2 Blocks)

Tall photosynthetic marine plants growing from the seabed toward the sunlit surface. Rendered with custom frame-strip animations.

| # | Block ID | Name | Category | Texture File(s) | Variations | Animation | Biological Description |
| :-: | :--- | :--- | :--- | :--- | :-: | :--- | :--- |
| **35** | `kelp_plant` | Kelp Plant | **Laminar Macroalga** | `vege_kelp_plant.png`, `1`, `2` | 3 | 16×320 px (20 frames) | Giant vertical brown laminar kelp forming dense underwater kelp forests. |
| **36** | `seagrass` | Seagrass | **Marine Angiosperm** | `vege_seagrass.png`, `1` | 2 | 16×240 px (15 frames) | Flowering marine grass anchoring soft sediments in shallow coastal meadows. |

---

## 12. Speleology & Crystals (`caverns`) — 28 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/caverns`  
📄 **Comprehensive Specification**: [caverns_design.md](blocks/caverns_design.md)

The subterranean speleological domain comprises **6 solid mineral crystal blocks**, **6 crystal clusters**, **6 glinted emissive crystal clusters**, **6 modular speleothem/dripstone systems** (stalactites, stalagmites, columns, and soda straws), and **4 paleontological fossil blocks** = **28 Total Cavern Blocks**.

---

### 3.1. Solid Crystal Blocks (6 Blocks)

Dense, solid blocks formed by compacting crystallized mineral geodes. Can be mined and used as luxury building materials.

| # | Block ID | Name | Category | Texture File(s) | Description |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `crystal_amethyst_block` | Amethyst Block | **Crystal** | `crystal_amethyst.png` | Violet quartz block with rich purple crystal facets ($SiO_2$). |
| **02** | `crystal_aquamarine_block` | Aquamarine Block | **Crystal** | `crystal_aquamarine.png` | Translucent cyan-blue beryl block ($Be_3Al_2Si_6O_{18}$). |
| **03** | `crystal_citrine_block` | Citrine Block | **Crystal** | `crystal_citrine.png` | Solar golden-amber quartz block created by geothermal heating ($SiO_2$). |
| **04** | `crystal_fluorite_block` | Fluorite Block | **Crystal** | `crystal_fluorite.png` | Mint-green cubic calcium fluoride block with soft natural sheen ($CaF_2$). |
| **05** | `crystal_pink_salt_block` | Pink Salt Block | **Crystal** | `crystal_pink_salt.png` | Ancient coral-pink evaporite halite salt block ($NaCl$). |
| **06** | `crystal_selenite_block` | Selenite Block | **Crystal** | `crystal_selenite.png` | Pearlescent silvery-white columnar gypsum block ($CaSO_4 \cdot 2H_2O$). |

---

### 3.2. Wild Mineral Crystal Clusters (6 Blocks)

Natural mineral prism clusters growing on geode cavity walls, floors, and ceilings. Rendered as cross-billboard geometry.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **07** | `crystal_amethyst_cluster` | Amethyst Cluster | **Cluster** | `crystal_amethyst_cluster.png` | Cross-billboard | Clustered prismatic violet quartz crystals. |
| **08** | `crystal_aquamarine_cluster` | Aquamarine Cluster | **Cluster** | `crystal_aquamarine_cluster.png` | Cross-billboard | Hexagonal prismatic sea-blue beryl cluster. |
| **09** | `crystal_citrine_cluster` | Citrine Cluster | **Cluster** | `crystal_citrine_cluster.png` | Cross-billboard | Radiant golden-amber hydrothermal quartz cluster. |
| **10** | `crystal_fluorite_cluster` | Fluorite Cluster | **Cluster** | `crystal_fluorite_cluster.png` | Cross-billboard | Cubic mint-green crystal cluster with natural cleavage. |
| **11** | `crystal_pink_salt_cluster` | Pink Salt Cluster | **Cluster** | `crystal_pink_salt_cluster.png` | Cross-billboard | Crystalline halite hopper crystals in coral pink. |
| **12** | `crystal_selenite_cluster` | Selenite Cluster | **Cluster** | `crystal_selenite_cluster.png` | Cross-billboard | Needle-sharp translucent pearlescent gypsum spires. |

---

### 3.3. Glinted Emissive Crystal Clusters (6 Blocks)

Rare, high-grade crystalline clusters featuring radiant optical reflections and emissive glow in dark caves.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **13** | `crystal_amethyst_glinted` | Glinted Amethyst Cluster | **Emissive** | `crystal_amethyst_cluster_glinted.png` | Cross-billboard | Deep purple cluster flashing radiant lavender glints. |
| **14** | `crystal_aquamarine_glinted` | Glinted Aquamarine Cluster | **Emissive** | `crystal_aquamarine_cluster_glinted.png` | Cross-billboard | Bioluminescent sea-foam cyan glints and reflections. |
| **15** | `crystal_citrine_glinted` | Glinted Citrine Cluster | **Emissive** | `crystal_citrine_cluster_glinted.png` | Cross-billboard | Fiery solar amber gleam emanating warm subterranean light. |
| **16** | `crystal_fluorite_glinted` | Glinted Fluorite Cluster | **Emissive** | `crystal_fluorite_cluster_glinted.png` | Cross-billboard | Naturally fluorescent ultraviolet emerald-green sparkle. |
| **17** | `crystal_pink_salt_glinted` | Glinted Pink Salt Cluster | **Emissive** | `crystal_pink_salt_cluster_glinted.png` | Cross-billboard | Warm salmon-pink shimmering salt cluster. |
| **18** | `crystal_selenite_glinted` | Glinted Selenite Cluster | **Emissive** | `crystal_selenite_cluster_glinted.png` | Cross-billboard | Pure silvery lunar brilliance reflecting ambient cavern light. |

---

### 3.4. Modular Speleothems & Dripstone Systems (6 Block Systems)

Dynamic, multi-stage subterranean dripstones. Each material supports procedural growth from ceiling (stalactite) or floor (stalagmite), merging seamlessly into solid cave pillars.

| # | Block ID | Name | Parent Bedrock | Pieces | Visual Identity | Geological Origin & Setting |
| :-: | :--- | :--- | :--- | :---: | :--- | :--- |
| **19** | `dripstone_karst` | Karst Dripstone | `rock_karst.png` | **10** | Rugged dark-grey weathered limestone | Classic dissolution caves formed by slow calcium bicarbonate seepage. |
| **20** | `dripstone_calcite` | Calcite Flowstone | `rock_calcite.png` | **10** | Smooth, pearlescent milk-white flowstone | Pure carbonate precipitation in pristine mineral-rich groundwater caverns. |
| **21** | `speleothem_brimstone` | Brimstone Spike | `rock_brimstone.png` | **10** | Sharp vivid sulfur-yellow needles | Acicular mineral spikes grown from gaseous sublimation around volcanic vents. |
| **22** | `speleothem_basalt` | Lava Icicle | `rock_basalt.png` | **10** | Charcoal basalt with glowing lava cracks | Lava tube stalactites formed by remelting and dripping basaltic magma. |
| **23** | `speleothem_ice` | Glacial Icicle | `frost_ice_packed.png` | **10** | Clear, crystalline glacial ice needles | Water dripping and refreezing inside polar cryosphere and permafrost caves. |
| **24** | `speleothem_salt_straw` | Pink Salt Soda Straw | `crystal_pink_salt.png` | **6** | Ultra-thin hollow coral-pink salt tubes | Fragile tubular soda straws precipitating from the outer rims of saline droplets. |

---

### 3.5. Paleontological Fossil Strata (4 Blocks)

Sedimentary subterranean formations containing ancient organic remnants, petroleum hydrocarbons, and amber inclusions. Can be excavated for paleontology, alchemy, and specialized chemical resources:

| # | Block ID | Name | Category | Texture File(s) | Face Mapping | Description |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **25** | `fossil_bones` | Fossil Bones | **Fossil** | `fossil_bones.png` | 6 uniform faces | Dark sedimentary rock embedding mineralized prehistoric vertebrate bones and skull fragments ($Ca_3(PO_4)_2$). |
| **26** | `fossil_amber` | Fossil Amber | **Fossil** | `fossil_amber.png` | 6 uniform faces | Deep strata embedding golden translucent fossilized tree resin droplets with ancient botanical/insect inclusions. |
| **27** | `fossil_bitumen` | Fossil Bitumen | **Fossil** | `fossil_bitumen.png` | 6 uniform faces | Dark hydrocarbon-saturated sedimentary seam containing viscous lustrous petroleum pitch and asphaltum. |
| **28** | `fossil_matrix` | Fossil Matrix | **Fossil** | `fossil_matrix.png` | 6 uniform faces | Barren host sedimentary siltstone enclosing bonebeds and fossil deposits; thematic structural block. |

---

---

## 13. Technical & System Blocks (`systems`) — 8 Blocks

📂 **Domain Directory**: `docs/worldbuilding/blocks/systems`  
📄 **Comprehensive Specification**: [systems_design.md](blocks/systems_design.md)

The engine internal technical and validation domain comprises **4 engine fallback placeholders** (solid cube, dynamic liquid, cross-billboard, and generic 3D UV checkerboard) and **4 developer pipeline validation markers** (initialization, chunk instance, pass, and fail) = **8 Total System Blocks**.

---

### 3.1. Engine Fallback Placeholders (4 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `null_block` | Null Block | **Engine Fallback** | `null_block.png` | 6 uniform faces *(Rendered when full block texture or ID is missing)* |
| **02** | `null_liquid` | Null Liquid | **Engine Fallback** | `null_liquid.png` | Animated vertical scrolling strip *(Rendered when liquid texture is unassigned)* |
| **03** | `null_crossmesh` | Null Crossmesh | **Engine Fallback** | `null_crossmesh.png` | Cross-billboard model *(Rendered when cross-billboard or flora texture is missing)* |
| **04** | `null_generic` | Null Generic | **Engine Fallback** | `null_generic.png` | UV face mapping *(Rendered when custom 3D model, prop, or entity texture is missing)* |

---

### 3.2. Developer Validation Markers (4 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **05** | `debug_start` | Debug Start Marker | **Development** | `debug_start.png` | 6 uniform faces *(Pipeline initialization validation)* |
| **06** | `debug_instance` | Debug Instance Marker | **Development** | `debug_instance.png` | 6 uniform faces *(Chunk / structure instance validation)* |
| **07** | `debug_accept` | Debug Accept Marker | **Development** | `debug_accept.png` | 6 uniform faces *(Visual indicator of passing assertion / OK)* |
| **08** | `debug_fail` | Debug Fail Marker | **Development** | `debug_fail.png` | 6 uniform faces *(Visual indicator of failed assertion / Error)* |

---
