# Worldbuilding Architecture & Catalog: Mushrooms (Fungi)

All active textures for wild cross-billboard mushrooms, giant mushroom caps, and macrofungal stems (`fungi_`) are organized in the folder:
📂 **`docs/worldbuilding/blocks/mushrooms`** *(54 active PNG textures on disk)*

Generating **18 unique blocks** in the master catalog: 8 wild understory ground mushrooms (cross-billboards with procedural variations) + 8 giant macrofungal caps + 2 shared macrofungal structural blocks.

Shared surface overlays are located in:
📂 **`docs/worldbuilding/blocks/overlays`**

---

## 1. Mycological and Morphological Classification

In the world's mycological ecosystem, fungal species and macrostructures are classified into **3 morphological categories (18 Unique Blocks / 54 Textures)**:

```mermaid
graph TD
    F["Kingdom Fungi & Mycology (18 Unique Blocks / 54 Textures)"] --> C1["1. Wild Ground Mushrooms (8 Cross-Billboard Blocks)"]
    F --> C2["2. Giant Macrofungal Caps (8 Solid Blocks)"]
    F --> C3["3. Shared Macrofungal Structural Blocks (2 Blocks)"]

    C1 --> C1a["Amanita, Porcini, Shiitake, Russula"]
    C1 --> C1b["Verdigris, Chanterelle, Mycena, Indigo"]

    C2 --> C2a["Amanita Cap, Porcini Cap, Shiitake Cap, Russula Cap"]
    C2 --> C2b["Verdigris Cap, Chanterelle Cap, Mycena Cap, Indigo Cap"]

    C3 --> C3a["Fungi Stem (Fibrous Stipe Trunk)"]
    C3 --> C3b["Fungi Inside (Porous Flesh / Hymenium Tissue)"]
```

---

## 2. Complete Texture Inventory in `worldbuilding/blocks/mushrooms/` (54 Active Textures)

All 32 textures strictly follow the unified prefix `fungi_`:

### A. Wild Understory Ground Mushrooms (24 Files, 16×16 px RGBA)
All 8 understory ground mushrooms strictly adhere to 3 standardized cross-billboard variations (`base`, `1`, `2`):
* `fungi_amanita.png`, `1.png`, `2.png` *(Fly agaric; scarlet pileus with veil dots)* (3)
* `fungi_porcini.png`, `1.png`, `2.png` *(King bolete; velvety earth-brown cap)* (3)
* `fungi_shiitake.png`, `1.png`, `2.png` *(Shiitake; warm wood-brown umbrella cap)* (3)
* `fungi_russula.png`, `1.png`, `2.png` *(Sickener; ruby-crimson brittlegill)* (3)
* `fungi_verdigris.png`, `1.png`, `2.png` *(Verdigris agaric; petrol-teal cap with golden scales)* (3)
* `fungi_chanterelle.png`, `1.png`, `2.png` *(Golden chanterelle; apricot fluted trumpet)* (3)
* `fungi_mycena.png`, `1.png`, `2.png` *(Mycena; bioluminescent green fairy bonnets)* (3)
* `fungi_indigo.png`, `1.png`, `2.png` *(Indigo milk cap; denim-slate indigo blue)* (3)

### B. Giant Macrofungal Caps (`_cap`) (24 Files, 16×16 px)
All 8 giant mushroom caps feature complete 3-fold stochastic variants (`1` and `2`) to break repetition across large fungal canopies:
* `fungi_amanita_cap.png` *(+ `1`, `2`)* *(Scarlet pileus dotted with white veil flecks)*
* `fungi_porcini_cap.png` *(+ `1`, `2`)* *(Deep velvety earth-brown cap)*
* `fungi_shiitake_cap.png` *(+ `1`, `2`)* *(Warm caramel-brown cap with natural fiber fissures)*
* `fungi_russula_cap.png` *(+ `1`, `2`)* *(Deep velvety crimson cuticle)*
* `fungi_verdigris_cap.png` *(+ `1`, `2`)* *(Petrol-teal surface flecked with golden-yellow scales)*
* `fungi_chanterelle_cap.png` *(+ `1`, `2`)* *(Velvety apricot-ochre wavy cap)*
* `fungi_mycena_cap.png` *(+ `1`, `2`)* *(Sage-green bioluminescent subterranean cap)*
* `fungi_indigo_cap.png` *(+ `1`, `2`)* *(Deep indigo-slate cap with concentric denim rings)*

### C. Shared Macrofungal Structural Blocks (6 Files, 16×16 px)
* `fungi_stem.png` *(+ `1`, `2`)* *(Fibrous cream-ivory stipe column)*
* `fungi_inside.png` *(+ `1`, `2`)* *(Porous hymenial interior flesh / underside gills)*

---

## 3. Complete Block Catalog (18 Unique Blocks)

### 3.1. Wild Understory Ground Mushrooms (8 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **01** | `fungi_amanita` | Amanita (Fly Agaric) | **Ground Mushroom** | `fungi_amanita.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **02** | `fungi_porcini` | Porcini (King Bolete) | **Ground Mushroom** | `fungi_porcini.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **03** | `fungi_shiitake` | Shiitake | **Ground Mushroom** | `fungi_shiitake.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **04** | `fungi_russula` | Russula (Sickener) | **Ground Mushroom** | `fungi_russula.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **05** | `fungi_verdigris` | Verdigris Agaric | **Ground Mushroom** | `fungi_verdigris.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **06** | `fungi_chanterelle` | Golden Chanterelle | **Ground Mushroom** | `fungi_chanterelle.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **07** | `fungi_mycena` | Mycena (Bioluminescent) | **Ground Mushroom** | `fungi_mycena.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |
| **08** | `fungi_indigo` | Indigo Milk Cap | **Ground Mushroom** | `fungi_indigo.png` *(+ `1`, `2`)* | 3 (Cross-billboard model: 2 intersecting planes) |

---

### 3.2. Giant Macrofungal Caps (8 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **09** | `fungi_amanita_cap` | Amanita Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_amanita_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **10** | `fungi_porcini_cap` | Porcini Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_porcini_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **11** | `fungi_shiitake_cap` | Shiitake Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_shiitake_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **12** | `fungi_russula_cap` | Russula Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_russula_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **13** | `fungi_verdigris_cap` | Verdigris Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_verdigris_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **14** | `fungi_chanterelle_cap` | Chanterelle Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_chanterelle_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **15** | `fungi_mycena_cap` | Mycena Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_mycena_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |
| **16** | `fungi_indigo_cap` | Indigo Cap | **Giant Mushroom Cap** | Top & Sides: `fungi_indigo_cap.png` *(+ `1`, `2`)*<br>Bottom: `fungi_inside.png` | Top & 4 sides cap + Bottom `fungi_inside.png` |

---

### 3.3. Shared Macrofungal Structural Blocks (2 Blocks)

| # | Block ID | Name | Category | Texture File(s) | Face Mapping |
| :-: | :--- | :--- | :--- | :--- | :--- |
| **17** | `fungi_stem` | Fungi Stem | **Macrofungal Structure** | `fungi_stem.png` *(+ `1`, `2`)* | 6 uniform faces |
| **18** | `fungi_inside` | Fungi Inside Flesh | **Macrofungal Structure** | `fungi_inside.png` *(+ `1`, `2`)* | 6 uniform faces |
