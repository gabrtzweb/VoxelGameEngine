# Worldbuilding Architecture & Catalog: Biomaterials, Fauna & Organic Matter (Organics)

All active textures for animal architecture, nests, living animal tissues, osteology, secretions, natural waxes, biological droppings, and spiderwebs are organized in the folder:
📂 **`docs/worldbuilding/blocks/organics`** *(94 active PNG textures on disk)*

Generating **36 unique blocks** in the master catalog divided into 3 biological categories (all strictly even numbers):
* **14 Fauna Architecture & Animal Nests (`nest_`)**: Hives, colonies, half-height nests, oothecae, cocoons, dams, and multi-soil animal burrows.
* **8 Living Tissues, Organs & Osteology (`bio_`)**: Raw muscle, bloody flesh, adipose blubber, nervous tissue, bloody brain, animal pelt, and axial skeletal bones.
* **14 Secretions, Waxes, Droppings & Spiderwebs (`organic_`)**: Honey, honeycomb, pure beeswax, colloidal gels, guano, manure, fossils, seashells, and modular vertical spiderwebs.

---

## 1. Ecological Classification & Fauna Strata

The biomaterial and zoological domain is structured into **3 Core Categories (36 Unique Blocks / 94 Textures)**:

```mermaid
graph TD
    O["Biomaterials & Organics (36 Unique Blocks / 94 Textures)"] --> A["1. Fauna Architecture & Nests: nest_ (14 Blocks)"]
    O --> B["2. Living Tissues & Osteology: bio_ (8 Blocks)"]
    O --> C["3. Secretions, Droppings & Webs: organic_ (14 Blocks)"]

    A --> A1["Ant Hill (Closed/Open), Termite Mound (Closed/Open)"]
    A --> A2["Bee Nest, Wasp Nest, Bird Nest, Hay Nest, Beaver Dam"]
    A --> A3["Spider Egg Ootheca, Silk Cocoon"]
    A --> A4["Loam Burrow, Silt Burrow, Peat Burrow (Soil-bound animal burrows)"]

    B --> B1["Raw Flesh Block, Bloody Flesh Block, Blubber Block, Brain Block, Bloody Brain Block, Fur Block"]
    B --> B2["Bone Block (Axial marrow), Calcined Bone (Pyromineralized fossil)"]

    C --> C1["Honey Block, Honeycomb Block, Beeswax Block, Slime Block, Jelly Block"]
    C --> C2["Guano Block, Poop/Manure Block, Coprolite Block, Dust Block, Seashell Block"]
    C --> C3["Cobweb (3D Volume), Cobweb Arch, Cobweb Hanging, Cobweb Stretched (Flat Wall Webs)"]
```

---

## 2. Rendering Standards & Texture Prefix Taxonomy

All textures adhere to a clean, 3-prefix functional taxonomy:
* **`nest_`**: Animal colonies, excavated mounds, tree-suspended hives, cocoons, dams, and burrows.
* **`bio_`**: Raw anatomical animal tissues, organs, pelts, and skeletal osseous matter.
* **`organic_`**: Viscous secretions, insect waxes, colloidal gels, natural droppings, and spider silk webs.

### 1. Directional Face Mapping Standards
* Standard cubic and slab directional tags: **`_top`** (upper face), **`_side`** (lateral faces), **`_bot`** (bottom/floor face), **`_front`** (entrance face), and **`_front_honey`** (dripping honey variant).
* **Bone Cylindrical Orientation**: `bio_bone_side.png` wraps the compact cortical bone shaft, while `bio_bone_top.png` represents the porous marrow canal on both axial caps.

### 2. Multi-Soil Burrow Architecture & Layered Rendering
Animal burrows (rabbits, badgers, marmots) utilize a dual-texture composite top face over soil bedrock:
* **`nest_burrow_top.png` (Grayscale Alpha Cutout)**: Renders the outer grass fringe surrounding the burrow opening, receiving dynamic **Biome Tinting** to seamlessly blend with adjacent terrain turf.
* **`nest_burrow_top_layer.png` (Full-Color Earth)**: Un-tinted soil threshold and interior depth shadow rendered directly atop the opening.
* **3 Dedicated Soil Burrows**: To guarantee full runtime simplicity without complex dynamic face replacement, 3 dedicated blocks exist for natural soil habitats:
  * **Loam Burrow (`loam_burrow`)**: Top burrow aperture + Bottom `soil_loam_dirt.png` + Sides `soil_loam_grass_side.png`.
  * **Silt Burrow (`silt_burrow`)**: Top burrow aperture + Bottom `soil_silt_dirt.png` + Sides `soil_silt_grass_side.png`.
  * **Peat Burrow (`peat_burrow`)**: Top burrow aperture + Bottom `soil_peat_dirt.png` + Sides `soil_peat_grass_side.png`.

### 3. Half-Height Fauna Nests (Slabs / 8px Height)
* **Bird Nest (`bird_nest`)**: Shallow bowl woven from twigs, moss, and feathers. Features a concave top cavity woven from fine twigs and soft moss (`nest_bird_nest_top.png`), side texture with transparent upper half (`nest_bird_nest_side.png`), and compact twig bottom (`nest_bird_nest_bot.png`).
* **Hay Nest (`hay_nest`)**: Farm poultry nest woven from dry agricultural hay and straw. Top concave hay bed (`nest_hay_nest_top.png`), side texture with transparent upper half (`nest_hay_nest_side.png`), and dense straw bottom (`nest_hay_nest_bot.png`).

### 4. Spider Silk Webs: Volumetric 3D vs. Flat Vertical Quad Cutouts
* **Volumetric 3D Cobweb (`organic_cobweb.png`)**: Full-voxel intersecting cross-billboard block slowing movement.
* **Flat Wall-Mounted Cobwebs**: 2D vertical quad blocks placed flush against walls, archways, and ceilings:
  * **`organic_cobweb_arch.png`**: Corner arch web hugging the upper joint between wall and ceiling.
  * **`organic_cobweb_hanging.png`**: Hanging curtain of silk fringe suspended from ceilings or lintels.
  * **`organic_cobweb_stretched.png`**: Stretched cobweb trap tensioned between two vertical surfaces.

---

## 3. Technical Texture Inventory in `worldbuilding/blocks/organics/` (85 Active Textures)

### A. Fauna Architecture & Nests (31 Files, 16×16 px)
* **Ant Hill (2)**: `nest_ant_hill.png`, `nest_ant_hill_open.png`
* **Termite Mound (2)**: `nest_termite_mound.png`, `nest_termite_mound_open.png`
* **Bee Nest (5)**: `nest_bee_nest_bot.png`, `nest_bee_nest_front.png`, `nest_bee_nest_front_honey.png`, `nest_bee_nest_side.png`, `nest_bee_nest_top.png`
* **Wasp Nest (3)**: `nest_wasp_nest_bot.png`, `nest_wasp_nest_side.png`, `nest_wasp_nest_top.png`
* **Bird Nest (3)**: `nest_bird_nest_bot.png`, `nest_bird_nest_side.png`, `nest_bird_nest_top.png` *(Slab model)*
* **Hay Nest (3)**: `nest_hay_nest_bot.png`, `nest_hay_nest_side.png`, `nest_hay_nest_top.png` *(Slab model)*
* **Beaver Dam (3)**: `nest_beaver_dam.png` *(+ `1`, `2`)*
* **Spider Egg (3)**: `nest_spider_egg_bot.png`, `nest_spider_egg_side.png`, `nest_spider_egg_top.png`
* **Silk Cocoon (3)**: `nest_silk_cocoon_bot.png`, `nest_silk_cocoon_side.png`, `nest_silk_cocoon_top.png`
* **Burrow (4)**: `nest_burrow_top.png` *(Grayscale tinted turf)*, `nest_burrow_top_layer.png` *(+ `1`, `2`)* *(Soil entry layer)*

### B. Living Tissues, Organs & Osteology (21 Files, 16×16 px)
* **Raw Flesh (4)**: `bio_flesh_top.png`, `bio_flesh_side.png` *(+ `1`, `2`)*
* **Blubber (3)**: `bio_blubber.png` *(+ `1`, `2`)* *(Translucent adipose layer)*
* **Brain (3)**: `bio_brain.png` *(+ `1`, `2`)* *(Convoluted neural cerebral mass)*
* **Animal Fur (3)**: `bio_fur_top.png`, `bio_fur_side.png`, `bio_fur_bot.png` *(Pelt with tanned suede leather backing)*
* **Bone Block (4)**: `bio_bone_top.png`, `bio_bone_side.png` *(+ `1`, `2`)*
* **Calcined Bone (4)**: `bio_calcined_bone_top.png`, `bio_calcined_bone_side.png` *(+ `1`, `2`)*

### C. Secretions, Waxes, Gels, Droppings & Spiderwebs (42 Files, 16×16 px)
* **Honey Block (3)**: `organic_honey_block_bot.png`, `organic_honey_block_side.png`, `organic_honey_block_top.png` *(Translucent)*
* **Honeycomb Block (3)**: `organic_honeycomb_block.png` *(+ `1`, `2`)*
* **Beeswax Block (3)**: `organic_wax.png` *(+ `1`, `2`)*
* **Slime Block (3)**: `organic_slime_top.png`, `organic_slime_side.png`, `organic_slime_bot.png` *(Translucent green)*
* **Jelly Block (3)**: `organic_jelly_top.png`, `organic_jelly_side.png`, `organic_jelly_bot.png` *(Translucent purple)*
* **Guano Block (3)**: `organic_guano.png` *(+ `1`, `2`)*
* **Poop Block (3)**: `organic_poop.png` *(+ `1`, `2`)* *(Natural manure with buzzing flies and straw fibers)*
* **Coprolite Block (3)**: `organic_coprolite.png` *(+ `1`, `2`)* *(Petrified fossil dung with calcified apatite inclusions)*
* **Dust Block (3)**: `organic_dust.png` *(+ `1`, `2`)*
* **Seashell Block (3)**: `organic_seashell.png` *(+ `1`, `2`)*
* **Cobweb (12)**: `organic_cobweb.png` *(+ `1`, `2`)*, `organic_cobweb_arch.png` *(+ `1`, `2`)*, `organic_cobweb_hanging.png` *(+ `1`, `2`)*, `organic_cobweb_stretched.png` *(+ `1`, `2`)*

---

## 4. Complete Block Catalog (36 Unique Blocks)

### 4.1. Fauna Architecture & Animal Nests (14 Blocks)

Structures engineered by colonial insects, avian species, mammals, and arachnids.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `ant_hill` | Ant Hill (Closed) | **Colony** | `nest_ant_hill.png` | Uniform (6 faces) |
| **02** | `ant_hill_open` | Ant Hill (Active) | **Colony** | `nest_ant_hill_open.png` | Uniform (6 faces) |
| **03** | `termite_mound` | Termite Mound (Closed) | **Colony** | `nest_termite_mound.png` | Uniform (6 faces) |
| **04** | `termite_mound_open` | Termite Mound (Active) | **Colony** | `nest_termite_mound_open.png` | Uniform (6 faces) |
| **05** | `bee_nest` | Wild Bee Nest | **Hive** | `nest_bee_nest_bot...top.png`<br>`nest_bee_nest_front_honey.png` | Directional (5 faces) |
| **06** | `wasp_nest` | Paper Wasp Nest | **Hive** | `nest_wasp_nest_bot...top.png` | Directional (3 faces) |
| **07** | `bird_nest` | Songbird Nest | **Nest** | `nest_bird_nest_bot...top.png` | Half-height Slab |
| **08** | `hay_nest` | Hay Poultry Nest | **Nest** | `nest_hay_nest_bot...top.png` | Half-height Slab |
| **09** | `beaver_dam` | Beaver Dam | **Architecture** | `nest_beaver_dam.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **10** | `spider_egg` | Spider Egg Sac | **Ootheca** | `nest_spider_egg_bot...top.png` | Directional (3 faces) |
| **11** | `silk_cocoon` | Wild Silk Cocoon | **Pupa** | `nest_silk_cocoon_bot...top.png` | Directional (3 faces) |
| **12** | `loam_burrow` | Loam Animal Burrow | **Burrow** | Top: `nest_burrow_top.png` + `layer.png` *(+ `1`, `2`)*<br>Bot: `soils/soil_loam_dirt.png`<br>Sides: `soils/soil_loam_grass_side.png` | Composite Top + Loam soil |
| **13** | `silt_burrow` | Silt Animal Burrow | **Burrow** | Top: `nest_burrow_top.png` + `layer.png` *(+ `1`, `2`)*<br>Bot: `soils/soil_silt_dirt.png`<br>Sides: `soils/soil_silt_grass_side.png` | Composite Top + Silt soil |
| **14** | `peat_burrow` | Peat Animal Burrow | **Burrow** | Top: `nest_burrow_top.png` + `layer.png` *(+ `1`, `2`)*<br>Bot: `soils/soil_peat_dirt.png`<br>Sides: `soils/soil_peat_grass_side.png` | Composite Top + Peat soil |

---

### 4.2. Living Tissues, Organs & Osteology (8 Blocks)

Biological anatomical materials harvested from animal carcasses or skeletal deposits.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **15** | `bone_block` | Bone Block | **Osteology** | `bio_bone_top.png`<br>`bio_bone_side.png` *(+ `1`, `2`)* | Axial Column (2 faces) |
| **16** | `calcined_bone` | Calcined Bone | **Osteology** | `bio_calcined_bone_top.png`<br>`bio_calcined_bone_side.png` *(+ `1`, `2`)* | Axial Column (2 faces) |
| **17** | `flesh_block` | Raw Flesh Block | **Tissue** | `bio_flesh_top.png`<br>`bio_flesh_side.png` *(+ `1`, `2`)* | Directional (2 faces) |
| **18** | `bloody_flesh` | Bloody Flesh Block | **Tissue** | Top/Bot: `bio_flesh_top.png`<br>Sides: `bio_flesh_side.png` *(+ `bloody_overlay.png`, `1`, `2`)* | Directional (2 faces) + Lateral Shader Bloody Overlay |
| **19** | `blubber_block` | Blubber Block | **Adipose** | `bio_blubber.png` *(+ `1`, `2`)* | Translucent (Alpha 204) |
| **20** | `brain_block` | Brain Block | **Neural** | `bio_brain.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **21** | `bloody_brain` | Bloody Brain Block | **Neural** | `bio_brain.png` *(+ `bloody_overlay.png`, `1`, `2`)* | Uniform (6 faces) + Shader Bloody Overlay |
| **22** | `fur_block` | Animal Fur Block | **Integument** | `bio_fur_top.png`<br>`bio_fur_side.png`<br>`bio_fur_bot.png` | Directional (3 faces) |

---

### 4.3. Secretions, Waxes, Gels, Droppings & Spiderwebs (14 Blocks)

Physiological secretions, insect products, biological wastes, and structural spider silk.

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **23** | `honey_block` | Honey Block | **Secretion** | `organic_honey_block_bot...top.png` | Translucent (3 faces) |
| **24** | `honeycomb_block` | Honeycomb Block | **Wax Matrix** | `organic_honeycomb_block.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **25** | `wax_block` | Beeswax Block | **Wax** | `organic_wax.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **26** | `slime_block` | Slime Block | **Colloid** | `organic_slime_top.png`<br>`organic_slime_side.png`<br>`organic_slime_bot.png` | Translucent (3 faces) |
| **27** | `jelly_block` | Royal Jelly Block | **Colloid** | `organic_jelly_top.png`<br>`organic_jelly_side.png`<br>`organic_jelly_bot.png` | Translucent (3 faces) |
| **28** | `guano_block` | Guano Block | **Dropping** | `organic_guano.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **29** | `poop_block` | Manure Block | **Dropping** | `organic_poop.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **30** | `coprolite_block` | Coprolite Block | **Fossil** | `organic_coprolite.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **31** | `dust_block` | Organic Dust | **Detritus** | `organic_dust.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **32** | `seashell_block` | Seashell Block | **Sediment** | `organic_seashell.png` *(+ `1`, `2`)* | Uniform (6 faces) |
| **33** | `cobweb` | Cobweb Block | **Silk** | `organic_cobweb.png` *(+ `1`, `2`)* | Volumetric 3D Cross |
| **34** | `cobweb_arch` | Cobweb Arch | **Silk** | `organic_cobweb_arch.png` *(+ `1`, `2`)* | Flat Vertical Quad |
| **35** | `cobweb_hanging` | Hanging Cobweb | **Silk** | `organic_cobweb_hanging.png` *(+ `1`, `2`)* | Flat Vertical Quad |
| **36** | `cobweb_stretched` | Stretched Cobweb | **Silk** | `organic_cobweb_stretched.png` *(+ `1`, `2`)* | Flat Vertical Quad |
