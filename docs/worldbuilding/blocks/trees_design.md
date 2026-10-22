# Worldbuilding Architecture & Catalog: Trees, Forestry & Dendrology (trees)

All active textures for living forest trees, timbers, foliage, and lumber are organized in the folder:
📂 **`docs/worldbuilding/blocks/trees`** *(242 active PNG textures on disk across 18 botanical species)*

Dynamic canopy overlays for seasonal flowers and snowy side accumulations are located in:
📂 **`docs/worldbuilding/blocks/overlays`** *(12 modular flowering overlays across 4 colors)*

Generating **92 unique blocks** in the master catalog:

* **18 Tree Species**: All true, living trees organized across 5 planetary bioclimatic zones.
* **92 Unique Blocks**: 35 log/wood/stalk structural blocks, 18 building plank blocks, 38 foliage blocks (healthy and dead), and 1 aerial root block.

---

## 1. Dendrological & Botanical Classification (18 Species)

Planetary forestry is organized into **5 Major Bioclimatic Zones & Canopy Morphologies (18 Botanical Species)**, with each zone featuring an even number of species:

```mermaid
graph TD
    A["Planetary Dendrology (18 Forest Species)"] --> B["1. Temperate & Deciduous Forests (6)"]
    A --> C["2. Boreal & Coniferous Taigas (4)"]
    A --> D["3. Tropical & Rainforest Jungles (4)"]
    A --> E["4. Arid Savannas & Woodlands (2)"]
    A --> F["5. Coastal & Wetland Groves (2)"]

    B --> B1["Oak: Classical temperate oak (Healthy, Dead)"]
    B --> B2["Birch: White paper birch with slender trunk and ivory lumber (Healthy, Dead)"]
    B --> B3["Maple: Vibrant autumn maple (Orange, Red, Yellow, Dead)"]
    B --> B4["Cherry: Ornamental mountain cherry with delicate blossoms (Sakura, Dead)"]
    B --> B5["Aspen: Mountain trembling aspen (Golden, Dead)"]
    B --> B6["Willow: Weeping riverbank willow with drooping foliage (Healthy, Dead)"]

    C --> C1["Pine: Boreal Scots pine with rugged bark (Healthy, Dead)"]
    C --> C2["Fir: Conical high-altitude silver fir (Healthy, Dead)"]
    C --> C3["Redwood: Colossal ancient sequoia with massive red-brick heartwood (Healthy, Dead)"]
    C --> C4["Yew: Ancient shade yew with deep chocolate-black lumber (Healthy, Dead)"]

    D --> D1["Mahogany: Dense jungle timber tree with deep reddish-brown heartwood (Healthy, Dead)"]
    D --> D2["Bamboo: Woody giant grass with segmented hollow stalks and fan foliage (Large, Small)"]
    D --> D3["Mangrove: Estuarine saltwater tree with stilt aerial root network (Healthy, Dead)"]
    D --> D4["Kapok: Colossal tropical emergent rainforest giant with buttress roots (Healthy, Dead)"]

    E --> E1["Acacia: Flat-topped umbrella savanna tree with vivid orange lumber (Healthy, Dead)"]
    E --> E2["Baobab: Ancient swollen water-storing pachycaul tree (Healthy, Dead)"]

    F --> F1["Palm: Tropical coastal and desert oasis palm with feathered fronds (Healthy, Dead)"]
    F --> F2["Cypress: Slender rot-resistant swamp and Mediterranean cypress (Healthy, Dead)"]
```

---

## 2. Dendrological Species Matrix (18 Species)

| # | Species | Botanical Reference | Native Biome | Trunk & Bark Morphology | Plank Color Profile (`planks`) | Foliage Variants / Seasonality | Ecological Role |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **01** | **Oak** | *Quercus robur* | Temperate Plains & Forests | Rugged brown bark, balanced concentric rings | Neutral warm amber brown | Normal, **Dead** *(+ Floral overlays)* | Universal forest canopy and baseline building timber. |
| **02** | **Birch** | *Betula pendula* | Temperate Forests & Foothills | Chalky white papery bark with horizontal dark lenticels | Pale ivory / cream blond wood | Normal, **Dead** | High-contrast luminous groves and Nordic architecture. |
| **03** | **Maple** | *Acer saccharum* | Deciduous Autumn Forests | Smooth grey-brown bark with warm beige rings | Soft cozy honey brown | 3 seasonal phases (Orange, Red, Yellow), **Dead** | Permanent autumn biomes with sweeping color gradients. |
| **04** | **Cherry** | *Prunus serrulata* | Oriental Groves & Highlands | Polished greyish-brown bark with fine annular rings | Delicate pastel pinkish-beige | Sakura blossom canopy, **Dead** | Ornamental groves, mountain shrines, and gardens. |
| **05** | **Aspen** | *Populus tremuloides* | Alpine Slopes & Mountain Ridges | Pale bone-white bark with vertical black scars | Clean pale chalk beige | Golden autumn, **Snowy (Top/Side)**, **Dead** | Subalpine transition zones and snowcapped mountain slopes. |
| **06** | **Willow** | *Salix babylonica* | Riparian Riverbanks & Swamps | Dark furrowed twisted fibrous bark | Gentle warm tan beige | Drooping pendulous foliage, **Dead** | Riverbanks, murky marshes, and lakeshore wetlands. |
| **07** | **Pine** | *Pinus sylvestris* | Boreal Taigas & Tundras | Rugged reddish-grey scaly bark plates | Deep rustic earthy brown | Needle canopy (tintable), **Snowy Top/Side**, **Dead** | Cold taigas, mountain log cabins, and rustic beams. |
| **08** | **Fir** | *Abies alba* | Mountain Taigas & Glacial Slopes | Dense vertical dark grey bark | Soft elegant coffee brown | Native green needles, **Snowy needles**, **Dead** | Steep alpine peaks, conical fir ridges, and snowy crests. |
| **09** | **Redwood** | *Sequoia sempervirens* | Coastal Fog Giants & Old-Growth | Thick deeply furrowed fibrous reddish-brown bark | Rich noble brick-red (Sequoia) | Evergreen needles, **Dead** | Colossal ancient trees with wide trunks (2×2 to 4×4 blocks). |
| **10** | **Yew** | *Taxus baccata* | Dark Woods & Ancient Sanctuaries | Deep near-black purple-tinged bark | Ultra-dark chocolate / Ebony (Dark Oak) | Dark needle canopy, **Snowy Top/Side**, **Dead** | Gothic cathedrals, shadowy dungeons, and luxury furnishings. |
| **11** | **Mahogany** | *Swietenia macrophylla* | Tropical Jungles & Rainforests | Rugged deep brown rainforest bark | Rich deep reddish-auburn timber | Broad tropical leaves, **Dead** | Lush equatorial canopies, naval shipbuilding, and fine furniture. |
| **12** | **Bamboo** | *Bambusoideae* | Bamboo Jungles & River Valleys | Hollow green jointed nodal culms (`stalk`) | Woven golden straw matting | Large Leaves (high canopy), Small Leaves (shoots) | Scaffolding, lightweight construction, and oriental mats. |
| **13** | **Mangrove** | *Rhizophora mangle* | Tidal Estuaries & Salt Marshes | Olive-grey bark with exposed stilt root arches | Earthy coppery-tan wood | Salt-tolerant waxy leaves, **Stilt Roots**, **Dead** | Intertidal mudflats, brackish deltas, and drowned forests. |
| **14** | **Kapok** | *Ceiba pentandra* | Equatorial Rainforests | Smooth grey bark with buttress roots and thorned nodules | Warm pale tropical clay beige | Broad palmate leaves, **Dead** | Colossal emergent trees piercing the high rainforest canopy. |
| **15** | **Acacia** | *Acacia sensu lato* | Arid Savannas & Scrublands | Striped grey bark with light fissured furrows | Striking vivid terracotta orange | Sparse umbrella foliage, **Dead** | Distinctive horizontal umbrella silhouettes on savanna horizons. |
| **16** | **Baobab** | *Adansonia digitata* | Arid Savannas & Dry Valleys | Smooth grey fibrous bark on swollen water-storing trunk | Fibrous sunny copper orange | Tufted foliage clusters, **Dead** | Colossal pachycaul landmarks enduring severe seasonal drought. |
| **17** | **Palm** | *Arecaceae* | Tropical Beaches, Coasts & Oases | Ringed golden-beige fibrous curved stipe bark | Sunny golden blonde timber | Broad arching green fronds, **Dead** | Tropical coastlines, sandy beaches, and desert oases. |
| **18** | **Cypress** | *Cupressus sempervirens* | Mediterranean Valleys & Swamps | Smooth dark greyish-brown fibrous bark | Resilient neutral warm hazel | Slender columnar emerald foliage, **Dead** | Formal vertical landscaping and desolate dead swamps. |

---

## 3. Botanical Anatomy & Texturing Standards

### 3.1. Logs vs Wood (Bark Blocks)

* **`tree_<name>_log` (Directional Circular Log)**:
  * **Top / Bottom**: Concentric annual growth rings showing heartwood and sapwood (`tree_<name>_log.png`).
  * **Sides**: Textured outer bark (`tree_<name>_bark.png`).
  * Supports 3-axis directional alignment (X, Y, Z axes).
* **`tree_<name>_wood` (6-Sided Bark Block)**:
  * Uses `tree_<name>_bark.png` across all 6 faces.
  * Used for horizontal branches, natural forks, and architectural pillars where cut rings must remain concealed.

### 3.2. Planks (`planks`): Staggered Board Standard

* Every wood species features a standardized **4-board staggered pattern** (16×16 px wood brick format).
* Built strictly using a 5-tone ramp:
  1. `Tone A`: Darkest deep seam / saw cut shadow.
  2. `Tone B`: Groove bevel shade.
  3. `Tone C`: Base lumber tone.
  4. `Tone D`: Mid-lit grain highlight.
  5. `Tone E`: Specular fiber highlight.

### 3.3. Foliage: Healthy Canopies & Dead Canopies

* **Healthy Canopies (`tree_*_leaves.png`)**: Binary alpha cutout transparency representing vibrant, turgid leaves. Features complete 3-fold stochastic variations (`tree_*_leaves.png`, `tree_*_leaves1.png`, `tree_*_leaves2.png`).
* **Dead Canopies (`tree_*_leaves_dead.png`)**:
  * Represents withered, drought-parched, or frost-killed canopies.
  * Strictly standardized to the **Botanical Senescence Palette (5 tones)** with complete 3-fold stochastic variations (`tree_<species>_leaves_dead.png`, `tree_<species>_leaves_dead1.png`, `tree_<species>_leaves_dead2.png`) generated via 1-to-1 tonal mapping from healthy leaf structures.
  * Botanical Senescence Tonal Ramp:
    * `(88, 57, 44)`: Deep withered shadow
    * `(103, 72, 53)`: Earthy brown shade
    * `(124, 92, 57)`: Dry straw / dead leaf midtone
    * `(140, 106, 60)`: Brittle ochre highlight
    * `(154, 125, 83)`: Sun-bleached brittle leaf tips

---

## 4. Technical Texture Inventory in `worldbuilding/blocks/trees/` (242 Active Textures)

> [!NOTE]
> **Stochastic Variations for Dynamic Shader Sampling:**
>
> * **Planks (54 files)**: All 18 wood species feature complete 3-fold plank variations (`tree_<species>_planks.png`, `1`, `2`).
> * **Barks (51 files)**: All 17 bark-bearing tree species feature complete 3-fold bark variations (`tree_<species>_bark.png`, `1`, `2`).
> * **Bamboo Stalk (4 files)**: Lateral column variations (`tree_bamboo_stalk.png`, `1`, `2`) and top cap (`tree_bamboo_stalk_top.png`).
> * **Mangrove Roots (6 files)**: Aerial roots (`tree_mangrove_roots.png`, `1`, `2`) and top waterlogged cap (`tree_mangrove_roots_top.png`, `1`, `2`).
> * **Log End Caps (17 files)**: Annual growth rings for columnar logs (`tree_<species>_log.png`).
> * **Living Foliage (57 files)**: All 19 active leaf canopy types feature complete 3-fold variations (`tree_<species>_leaves.png`, `1`, `2`).
> * **Bamboo Leaves (2 files)**: `tree_bamboo_large_leaves.png` and `tree_bamboo_small_leaves.png`.
> * **Senescent / Dead Foliage (51 files)**: All 17 tree species feature complete 3-fold stochastic variations (`tree_<species>_leaves_dead.png`, `1`, `2`).

### A. Temperate & Deciduous Forests (77 Files, 16×16 px)

* **Oak (13)**: `tree_oak_bark.png` *(+ 1, 2)*, `tree_oak_log.png`, `tree_oak_planks.png` *(+ 1, 2)*, `tree_oak_leaves.png` *(+ 1, 2)*, `tree_oak_leaves_dead.png` *(+ 1, 2)*
* **Birch (13)**: `tree_birch_bark.png` *(+ 1, 2)*, `tree_birch_log.png`, `tree_birch_planks.png` *(+ 1, 2)*, `tree_birch_leaves.png` *(+ 1, 2)*, `tree_birch_leaves_dead.png` *(+ 1, 2)*
* **Maple (19)**: `tree_maple_bark.png` *(+ 1, 2)*, `tree_maple_log.png`, `tree_maple_planks.png` *(+ 1, 2)*, `tree_maple_leaves_orange.png` *(+ 1, 2)*, `tree_maple_leaves_red.png` *(+ 1, 2)*, `tree_maple_leaves_yellow.png` *(+ 1, 2)*, `tree_maple_leaves_dead.png` *(+ 1, 2)*
* **Cherry (13)**: `tree_cherry_bark.png` *(+ 1, 2)*, `tree_cherry_log.png`, `tree_cherry_planks.png` *(+ 1, 2)*, `tree_cherry_leaves.png` *(+ 1, 2)*, `tree_cherry_leaves_dead.png` *(+ 1, 2)*
* **Aspen (13)**: `tree_aspen_bark.png` *(+ 1, 2)*, `tree_aspen_log.png`, `tree_aspen_planks.png` *(+ 1, 2)*, `tree_aspen_leaves.png` *(+ 1, 2)*, `tree_aspen_leaves_dead.png` *(+ 1, 2)*
* **Willow (13)**: `tree_willow_bark.png` *(+ 1, 2)*, `tree_willow_log.png`, `tree_willow_planks.png` *(+ 1, 2)*, `tree_willow_leaves.png` *(+ 1, 2)*, `tree_willow_leaves_dead.png` *(+ 1, 2)*

### B. Boreal & Coniferous Taigas (48 Files, 16×16 px)

* **Pine (13)**: `tree_pine_bark.png` *(+ 1, 2)*, `tree_pine_log.png`, `tree_pine_planks.png` *(+ 1, 2)*, `tree_pine_leaves.png` *(+ 1, 2)*, `tree_pine_leaves_dead.png` *(+ 1, 2)*
* **Fir (13)**: `tree_fir_bark.png` *(+ 1, 2)*, `tree_fir_log.png`, `tree_fir_planks.png` *(+ 1, 2)*, `tree_fir_leaves.png` *(+ 1, 2)*, `tree_fir_leaves_dead.png` *(+ 1, 2)*
* **Redwood (13)**: `tree_redwood_bark.png` *(+ 1, 2)*, `tree_redwood_log.png`, `tree_redwood_planks.png` *(+ 1, 2)*, `tree_redwood_leaves.png` *(+ 1, 2)*, `tree_redwood_leaves_dead.png` *(+ 1, 2)*
* **Yew (13)**: `tree_yew_bark.png` *(+ 1, 2)*, `tree_yew_log.png`, `tree_yew_planks.png` *(+ 1, 2)*, `tree_yew_leaves.png` *(+ 1, 2)*, `tree_yew_leaves_dead.png` *(+ 1, 2)*

### C. Tropical & Rainforest Jungles (54 Files, 16×16 px)

* **Mahogany (13)**: `tree_mahogany_bark.png` *(+ 1, 2)*, `tree_mahogany_log.png`, `tree_mahogany_planks.png` *(+ 1, 2)*, `tree_mahogany_leaves.png` *(+ 1, 2)*, `tree_mahogany_leaves_dead.png` *(+ 1, 2)*
* **Bamboo (9)**: `tree_bamboo_stalk.png` *(+ 1, 2)*, `tree_bamboo_stalk_top.png`, `tree_bamboo_planks.png` *(+ 1, 2)*, `tree_bamboo_large_leaves.png`, `tree_bamboo_small_leaves.png`
* **Mangrove (19)**: `tree_mangrove_bark.png` *(+ 1, 2)*, `tree_mangrove_log.png`, `tree_mangrove_planks.png` *(+ 1, 2)*, `tree_mangrove_leaves.png` *(+ 1, 2)*, `tree_mangrove_roots.png` *(+ 1, 2)*, `tree_mangrove_roots_top.png` *(+ 1, 2)*, `tree_mangrove_leaves_dead.png` *(+ 1, 2)*
* **Kapok (13)**: `tree_kapok_bark.png` *(+ 1, 2)*, `tree_kapok_log.png`, `tree_kapok_planks.png` *(+ 1, 2)*, `tree_kapok_leaves.png` *(+ 1, 2)*, `tree_kapok_leaves_dead.png` *(+ 1, 2)*

### D. Arid Savannas & Woodlands (26 Files, 16×16 px)

* **Acacia (13)**: `tree_acacia_bark.png` *(+ 1, 2)*, `tree_acacia_log.png`, `tree_acacia_planks.png` *(+ 1, 2)*, `tree_acacia_leaves.png` *(+ 1, 2)*, `tree_acacia_leaves_dead.png` *(+ 1, 2)*
* **Baobab (13)**: `tree_baobab_bark.png` *(+ 1, 2)*, `tree_baobab_log.png`, `tree_baobab_planks.png` *(+ 1, 2)*, `tree_baobab_leaves.png` *(+ 1, 2)*, `tree_baobab_leaves_dead.png` *(+ 1, 2)*

### E. Coastal & Wetland Groves (26 Files, 16×16 px)

* **Palm (13)**: `tree_palm_bark.png` *(+ 1, 2)*, `tree_palm_log.png`, `tree_palm_planks.png` *(+ 1, 2)*, `tree_palm_leaves.png` *(+ 1, 2)*, `tree_palm_leaves_dead.png` *(+ 1, 2)*
* **Cypress (13)**: `tree_cypress_bark.png` *(+ 1, 2)*, `tree_cypress_log.png`, `tree_cypress_planks.png` *(+ 1, 2)*, `tree_cypress_leaves.png` *(+ 1, 2)*, `tree_cypress_leaves_dead.png` *(+ 1, 2)*

---

## 5. Canopy Overlays in `worldbuilding/blocks/overlays/` (12 Active Files)

| Overlay File | Function | Primary Associated Species |
| :--- | :--- | :--- |
| `flower_leaves_white_overlay.png` | Tiny white spring blossoms (Var 0) | `Oak`, `Birch`, `Cherry` (white sakura) |
| `flower_leaves_white_overlay1.png` | Clustered white spring floral bouquets (Var 1) | `Oak`, `Birch`, `Willow` (catkins) |
| `flower_leaves_magenta_overlay.png` | Vivid pinkish-magenta floral blossoms (Var 0) | `Cherry` (pink sakura), `Mahogany`, `Kapok` |
| `flower_leaves_magenta_overlay1.png` | Clustered vibrant magenta floral bunches (Var 1) | `Cherry`, `Kapok`, `Oak` |
| `flower_leaves_yellow_overlay.png` | Golden yellow spring inflorescences (Var 0) | `Acacia`, `Oak`, `Birch` |
| `flower_leaves_yellow_overlay1.png` | Clustered golden yellow blossom sprays (Var 1) | `Acacia`, `Willow`, `Kapok` |
| `flower_leaves_blue_overlay.png` | Delicate sky-blue inflorescences (Var 0) | `Willow`, mystical enchanted forest canopies |
| `flower_leaves_blue_overlay1.png` | Clustered blue blossom bouquets (Var 1) | `Willow`, highland alpine groves |

---

## 6. Complete Block Catalog (92 Unique Blocks)

### 6.1. Temperate & Deciduous Forest Blocks (35 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **001** | `oak_log` | Oak Log | **Wood** | Top/Bottom: `tree_oak_log.png`<br>Sides: `tree_oak_bark.png` | Directional (Top/Bottom + 4 sides) |
| **002** | `oak_wood` | Oak Wood | **Wood** | `tree_oak_bark.png` | 6 uniform faces |
| **003** | `oak_planks` | Oak Planks | **Planks** | `tree_oak_planks.png` | 6 uniform faces |
| **004** | `oak_leaves` | Oak Leaves | **Foliage** | `tree_oak_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **005** | `oak_leaves_dead` | Dead Oak Leaves | **Foliage** | `tree_oak_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **006** | `birch_log` | Birch Log | **Wood** | Top/Bottom: `tree_birch_log.png`<br>Sides: `tree_birch_bark.png` | Directional (Top/Bottom + 4 sides) |
| **007** | `birch_wood` | Birch Wood | **Wood** | `tree_birch_bark.png` | 6 uniform faces |
| **008** | `birch_planks` | Birch Planks | **Planks** | `tree_birch_planks.png` | 6 uniform faces |
| **009** | `birch_leaves` | Birch Leaves | **Foliage** | `tree_birch_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **010** | `birch_leaves_dead` | Dead Birch Leaves | **Foliage** | `tree_birch_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **011** | `maple_log` | Maple Log | **Wood** | Top/Bottom: `tree_maple_log.png`<br>Sides: `tree_maple_bark.png` | Directional (Top/Bottom + 4 sides) |
| **012** | `maple_wood` | Maple Wood | **Wood** | `tree_maple_bark.png` | 6 uniform faces |
| **013** | `maple_planks` | Maple Planks | **Planks** | `tree_maple_planks.png` | 6 uniform faces |
| **014** | `maple_leaves_orange` | Orange Maple Leaves | **Foliage** | `tree_maple_leaves_orange.png` | 6 uniform faces (Cutout alpha transparency) |
| **015** | `maple_leaves_red` | Red Maple Leaves | **Foliage** | `tree_maple_leaves_red.png` | 6 uniform faces (Cutout alpha transparency) |
| **016** | `maple_leaves_yellow` | Yellow Maple Leaves | **Foliage** | `tree_maple_leaves_yellow.png` | 6 uniform faces (Cutout alpha transparency) |
| **017** | `maple_leaves_dead` | Dead Maple Leaves | **Foliage** | `tree_maple_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **018** | `cherry_log` | Cherry Log | **Wood** | Top/Bottom: `tree_cherry_log.png`<br>Sides: `tree_cherry_bark.png` | Directional (Top/Bottom + 4 sides) |
| **019** | `cherry_wood` | Cherry Wood | **Wood** | `tree_cherry_bark.png` | 6 uniform faces |
| **020** | `cherry_planks` | Cherry Planks | **Planks** | `tree_cherry_planks.png` | 6 uniform faces |
| **021** | `cherry_leaves` | Cherry Blossom Leaves | **Foliage** | `tree_cherry_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **022** | `cherry_leaves_dead` | Dead Cherry Leaves | **Foliage** | `tree_cherry_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **023** | `aspen_log` | Aspen Log | **Wood** | Top/Bottom: `tree_aspen_log.png`<br>Sides: `tree_aspen_bark.png` | Directional (Top/Bottom + 4 sides) |
| **024** | `aspen_wood` | Aspen Wood | **Wood** | `tree_aspen_bark.png` | 6 uniform faces |
| **025** | `aspen_planks` | Aspen Planks | **Planks** | `tree_aspen_planks.png` | 6 uniform faces |
| **026** | `aspen_leaves` | Golden Aspen Leaves | **Foliage** | `tree_aspen_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **027** | `aspen_leaves_dead` | Dead Aspen Leaves | **Foliage** | `tree_aspen_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **028** | `willow_log` | Willow Log | **Wood** | Top/Bottom: `tree_willow_log.png`<br>Sides: `tree_willow_bark.png` | Directional (Top/Bottom + 4 sides) |
| **029** | `willow_wood` | Willow Wood | **Wood** | `tree_willow_bark.png` | 6 uniform faces |
| **030** | `willow_planks` | Willow Planks | **Planks** | `tree_willow_planks.png` | 6 uniform faces |
| **031** | `willow_leaves` | Weeping Willow Leaves | **Foliage** | `tree_willow_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **032** | `willow_leaves_dead` | Dead Willow Leaves | **Foliage** | `tree_willow_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |

---

### 6.2. Boreal & Coniferous Taiga Blocks (23 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **033** | `pine_log` | Pine Log | **Wood** | Top/Bottom: `tree_pine_log.png`<br>Sides: `tree_pine_bark.png` | Directional (Top/Bottom + 4 sides) |
| **034** | `pine_wood` | Pine Wood | **Wood** | `tree_pine_bark.png` | 6 uniform faces |
| **035** | `pine_planks` | Pine Planks | **Planks** | `tree_pine_planks.png` | 6 uniform faces |
| **036** | `pine_leaves` | Pine Needles | **Foliage** | `tree_pine_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **037** | `pine_leaves_dead` | Dead Pine Needles | **Foliage** | `tree_pine_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **038** | `fir_log` | Fir Log | **Wood** | Top/Bottom: `tree_fir_log.png`<br>Sides: `tree_fir_bark.png` | Directional (Top/Bottom + 4 sides) |
| **039** | `fir_wood` | Fir Wood | **Wood** | `tree_fir_bark.png` | 6 uniform faces |
| **040** | `fir_planks` | Fir Planks | **Planks** | `tree_fir_planks.png` | 6 uniform faces |
| **041** | `fir_leaves` | Fir Needles | **Foliage** | `tree_fir_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **042** | `fir_leaves_dead` | Dead Fir Needles | **Foliage** | `tree_fir_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **043** | `redwood_log` | Redwood Log | **Wood** | Top/Bottom: `tree_redwood_log.png`<br>Sides: `tree_redwood_bark.png` | Directional (Top/Bottom + 4 sides) |
| **044** | `redwood_wood` | Redwood Wood | **Wood** | `tree_redwood_bark.png` | 6 uniform faces |
| **045** | `redwood_planks` | Redwood Planks | **Planks** | `tree_redwood_planks.png` | 6 uniform faces |
| **046** | `redwood_leaves` | Redwood Foliage | **Foliage** | `tree_redwood_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **047** | `redwood_leaves_dead` | Dead Redwood Foliage | **Foliage** | `tree_redwood_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **048** | `yew_log` | Yew Log | **Wood** | Top/Bottom: `tree_yew_log.png`<br>Sides: `tree_yew_bark.png` | Directional (Top/Bottom + 4 sides) |
| **049** | `yew_wood` | Yew Wood | **Wood** | `tree_yew_bark.png` | 6 uniform faces |
| **050** | `yew_planks` | Yew Planks | **Planks** | `tree_yew_planks.png` | 6 uniform faces |
| **051** | `yew_leaves` | Yew Foliage | **Foliage** | `tree_yew_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **052** | `yew_leaves_dead` | Dead Yew Foliage | **Foliage** | `tree_yew_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |

---

### 6.3. Tropical & Rainforest Jungle Blocks (20 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **053** | `mahogany_log` | Mahogany Log | **Wood** | Top/Bottom: `tree_mahogany_log.png`<br>Sides: `tree_mahogany_bark.png` | Directional (Top/Bottom + 4 sides) |
| **054** | `mahogany_wood` | Mahogany Wood | **Wood** | `tree_mahogany_bark.png` | 6 uniform faces |
| **055** | `mahogany_planks` | Mahogany Planks | **Planks** | `tree_mahogany_planks.png` | 6 uniform faces |
| **056** | `mahogany_leaves` | Mahogany Leaves | **Foliage** | `tree_mahogany_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **057** | `mahogany_leaves_dead` | Dead Mahogany Leaves | **Foliage** | `tree_mahogany_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **058** | `bamboo_stalk` | Bamboo Stalk | **Wood / Stalk** | Top/Bottom: `tree_bamboo_stalk_top.png`<br>Sides: `tree_bamboo_stalk.png` | Directional column (3×3 top/bottom, 3×16 sides) |
| **059** | `bamboo_planks` | Bamboo Planks | **Planks** | `tree_bamboo_planks.png` | 6 uniform faces |
| **060** | `bamboo_large_leaves` | Large Bamboo Leaves | **Foliage** | `tree_bamboo_large_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **061** | `bamboo_small_leaves` | Small Bamboo Leaves | **Foliage** | `tree_bamboo_small_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **062** | `mangrove_log` | Mangrove Log | **Wood** | Top/Bottom: `tree_mangrove_log.png`<br>Sides: `tree_mangrove_bark.png` | Directional (Top/Bottom + 4 sides) |
| **063** | `mangrove_wood` | Mangrove Wood | **Wood** | `tree_mangrove_bark.png` | 6 uniform faces |
| **064** | `mangrove_planks` | Mangrove Planks | **Planks** | `tree_mangrove_planks.png` | 6 uniform faces |
| **065** | `mangrove_leaves` | Mangrove Leaves | **Foliage** | `tree_mangrove_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **066** | `mangrove_roots` | Mangrove Aerial Roots | **Root** | Top/Bottom: `tree_mangrove_roots_top.png`<br>Sides: `tree_mangrove_roots.png` | Directional (Top/Bottom + 4 sides, Cutout) |
| **067** | `mangrove_leaves_dead` | Dead Mangrove Leaves | **Foliage** | `tree_mangrove_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **068** | `kapok_log` | Kapok Log | **Wood** | Top/Bottom: `tree_kapok_log.png`<br>Sides: `tree_kapok_bark.png` | Directional (Top/Bottom + 4 sides) |
| **069** | `kapok_wood` | Kapok Wood | **Wood** | `tree_kapok_bark.png` | 6 uniform faces |
| **070** | `kapok_planks` | Kapok Planks | **Planks** | `tree_kapok_planks.png` | 6 uniform faces |
| **071** | `kapok_leaves` | Kapok Leaves | **Foliage** | `tree_kapok_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **072** | `kapok_leaves_dead` | Dead Kapok Leaves | **Foliage** | `tree_kapok_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |

---

### 6.4. Arid Savanna & Woodland Blocks (10 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **073** | `acacia_log` | Acacia Log | **Wood** | Top/Bottom: `tree_acacia_log.png`<br>Sides: `tree_acacia_bark.png` | Directional (Top/Bottom + 4 sides) |
| **074** | `acacia_wood` | Acacia Wood | **Wood** | `tree_acacia_bark.png` | 6 uniform faces |
| **075** | `acacia_planks` | Acacia Planks | **Planks** | `tree_acacia_planks.png` | 6 uniform faces |
| **076** | `acacia_leaves` | Acacia Leaves | **Foliage** | `tree_acacia_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **077** | `acacia_leaves_dead` | Dead Acacia Leaves | **Foliage** | `tree_acacia_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **078** | `baobab_log` | Baobab Log | **Wood** | Top/Bottom: `tree_baobab_log.png`<br>Sides: `tree_baobab_bark.png` | Directional (Top/Bottom + 4 sides) |
| **079** | `baobab_wood` | Baobab Wood | **Wood** | `tree_baobab_bark.png` | 6 uniform faces |
| **080** | `baobab_planks` | Baobab Planks | **Planks** | `tree_baobab_planks.png` | 6 uniform faces |
| **081** | `baobab_leaves` | Baobab Leaves | **Foliage** | `tree_baobab_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **082** | `baobab_leaves_dead` | Dead Baobab Leaves | **Foliage** | `tree_baobab_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |

---

### 6.5. Coastal & Wetland Grove Blocks (10 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **083** | `palm_log` | Palm Log | **Wood** | Top/Bottom: `tree_palm_log.png`<br>Sides: `tree_palm_bark.png` | Directional (Top/Bottom + 4 sides) |
| **084** | `palm_wood` | Palm Wood | **Wood** | `tree_palm_bark.png` | 6 uniform faces |
| **085** | `palm_planks` | Palm Planks | **Planks** | `tree_palm_planks.png` | 6 uniform faces |
| **086** | `palm_leaves` | Palm Fronds | **Foliage** | `tree_palm_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **087** | `palm_leaves_dead` | Dead Palm Fronds | **Foliage** | `tree_palm_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
| **088** | `cypress_log` | Cypress Log | **Wood** | Top/Bottom: `tree_cypress_log.png`<br>Sides: `tree_cypress_bark.png` | Directional (Top/Bottom + 4 sides) |
| **089** | `cypress_wood` | Cypress Wood | **Wood** | `tree_cypress_bark.png` | 6 uniform faces |
| **090** | `cypress_planks` | Cypress Planks | **Planks** | `tree_cypress_planks.png` | 6 uniform faces |
| **091** | `cypress_leaves` | Cypress Foliage | **Foliage** | `tree_cypress_leaves.png` | 6 uniform faces (Cutout alpha transparency) |
| **092** | `cypress_leaves_dead` | Dead Cypress Foliage | **Foliage** | `tree_cypress_leaves_dead.png` | 6 uniform faces (Cutout alpha transparency) |
