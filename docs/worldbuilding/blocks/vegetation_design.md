# Worldbuilding Architecture & Catalog: Groundcover & Herbaceous Flora (vegetation)

All active textures for herbaceous flora, meadows, wild cereals, ferns, xerophytes, cacti, wetland plants, and epiphytes are organized in the folder:
📂 **`docs/worldbuilding/blocks/vegetation`** *(109 active PNG textures on disk, representing 38 unique blocks across 6 strictly even categories)*

Generating **38 unique blocks** in the master catalog:

* **12 Grasses & Wild Cereals**: 6 single-block carpet grasses + 6 double-tall prairie grasses and grain stalks.
* **6 Ferns & Pteridophytes**: 4 understory fronds + 2 double-tall giant and jungle ferns.
* **8 Shrubs, Bushes & Cacti**: 3 temperate/arid shrubs + 5 xerophytic cacti and succulents.
* **4 Wetlands, Aquatics & Floating Pads**: Riparian reeds and freshwater floating lily pads.
* **4 Epiphytes, Climbers & Root Hangings**: Wall vines, tree lichens, and ceiling hanging root mosses.
* **4 Subterranean Cave Flora**: Umbrophilic mosses, mineral encrustations, and glowing geode herbs.

---

## 1. Ecological Classification & Physiognomic Domains (38 Unique Blocks)

The herbaceous and shrub layer is organized into **6 Major Physiognomic Domains (38 Unique Blocks / 109 Active Textures)**, with every single domain featuring an even number of blocks:

```mermaid
graph TD
    V["Herbaceous Flora & Vegetation (38 Blocks / 109 Textures)"] --> A["1. Grasses & Wild Cereals (12 Blocks)"]
    V --> B["2. Ferns & Pteridophytes (6 Blocks)"]
    V --> C["3. Shrubs, Bushes & Cacti (8 Blocks)"]
    V --> D["4. Wetlands & Aquatics (4 Blocks)"]
    V --> E["5. Epiphytes & Climbers (4 Blocks)"]
    V --> F["6. Subterranean Cave Flora (4 Blocks)"]

    A --> A1["Wildgrass, Drygrass, Frigid Grass, Bluff Grass, Mossgrass"]
    A --> A2["Tall Wildgrass, Tall Prairie, Tall Beach, Sea Oats, Barley, Tall Bluff, Pampas Grass"]

    B --> B1["Common Fern, Large Fern, Snowy Large Fern, Bracken"]
    B --> B2["Tall Fern, Tall Jungle Fern (Double-tall arborescent ferns)"]

    C --> C1["Temperate Bush, Red Shrub, Dead Bush (Deciduous and arid shrubs)"]
    C --> C2["Cactus Bush, Barrel Cactus, Prickly Pear, Small Cactus, Saguaro Cactus"]

    D --> D1["Sugar Cane, Cattail, Water Lily Pad, Pygmy Helvola Pad"]

    E --> E1["Vines, Hanging Moss, Hanging Roots, Tree Lichen"]

    F --> F1["Cave Growths, Grainy Cave Growths, Lurid Cave Growths, Cave Hyssop"]
```

---

## 2. Rendering Profiles & Architectural Geometry

1. **Simple Cross-Mesh / Billboard Flora (`16×16 px RGBA Cutout`)**:
   * Rendered using intersecting diagonal planes (X-shape) or offset vertical billboard quads.
   * Includes procedural variations (`1, 2, 3...`) to break visual tiling repetition across open fields.
2. **Double-Tall Composite Flora (Two-Block Plants)**:
   * **Lower Segment (`vege_*_bottom.png`)**: Dense basal root clusters, clumps, or heavy lower stalks.
   * **Upper Segment (`vege_*_top.png`)**: Feathered plumes, flowering tassels, grain heads, or broad frond arches.
   * **Consolidated Composite Sprites (`32×32 px RGBA`)**: Master full-height textures (`vege_tall_wildgrass.png`, `vege_tall_fern.png`) used for entity icons and unified LOD rendering.
3. **Solid & Columnar 3D Blocks (Cacti & Succulents)**:
   * **Large Saguaro Cactus (`vege_cactus`)**: Full 3D column cube block with directional `top`, `bottom`, and `side` ribs.
   * **Small Cactus (`vege_small_cactus`)**: Compact succulent column block (`top`, `bottom`, `side`).
4. **Surface Overlays & Floating Pads (Epiphytes & Water Pads)**:
   * **Floating Pads (`vege_lily_pad`, `vege_helvola_pad`)**: Thin planar geometry floating on the upper water meniscus.
   * **Wall Climbers & Ceilings (`vege_vines`, `vege_lichen`, `vege_hanging_roots`)**: Oriented planar decals mounted against vertical bark/rock walls or hanging beneath ceiling faces.

---

## 3. Complete Block Catalog (38 Unique Blocks)

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
