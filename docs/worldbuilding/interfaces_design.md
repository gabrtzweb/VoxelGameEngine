# Voxel Game Engine — User Interface Design & Architecture (`interfaces_design.md`)

This document formally specifies and standardizes the entire **User Interface (UI / HUD) architecture** for the Voxel Game Engine. It covers the standardized 9-horizontal-slot inventory system, container menus (chests), the modular creative catalog, hotbar HUD, button components, standalone navigation/hover textures, equipment slot overlays, and the interface icon repository located in `docs/worldbuilding/interfaces/`.

> [!IMPORTANT]
> **Standardized 9-Horizontal-Slot Metric (416 × 256 px)**  
> The entire interface suite has been consolidated around the definitive **9-horizontal-slot** standard (168 px slot grid width, unified 416 × 256 px canvas for all inventory/container windows, and 320 × 44 px for the in-game hotbar HUD). All legacy 8-slot layouts have been fully deprecated.

---

## 1. Architectural Principle: Blueprint Skeleton (`_outline`) vs. Clean Render (`.png`)

The engine employs a **strict dual-layer decoupled architecture** to separate geometric logic and interaction mapping from cosmetic pixel-art rendering:

```
┌────────────────────────────────────────────────────────────────────────┐
│                      ENGINE UI RENDERING PIPELINE                      │
├───────────────────────────────────┬────────────────────────────────────┤
│   1. Geometric Blueprint Layer    │   2. Clean Cosmetic Render Layer   │
│      (*_outline.png)              │      (*.png)                       │
├───────────────────────────────────┼────────────────────────────────────┤
│ • Functional geometric skeleton   │ • Final visual game skin           │
│ • Card boundary perimeters        │ • Dark slate / obsidian palette    │
│ • Slot & button hitboxes          │ • 0 guide lines or ID numbers      │
│ • Unique field IDs (A1, B1, etc.) │ • 1px brushed steel bevels         │
│ • Parsed on boot / asset bake     │ • Rendered live during gameplay    │
└───────────────────────────────────┴────────────────────────────────────┘
```

### 1.1 Blueprint Textures (`*_outline.png`)
* Function as the **functional geometry map** and **interaction blueprint** for the engine.
* During boot initialization or asset pre-baking, the UI layout engine parses the `_outline` texture to extract the precise bounding boxes `(x, y, width, height)` of every clickable slot, button, viewport, text box, and scroll track.
* Eliminates hardcoded magic pixel coordinates in source code.
* Unambiguously identifies modular cards with letters (`A`, `B`, `C`, `D`) and internal components with sequential numbers.

### 1.2 Clean Render Textures (`*.png` without `_outline`)
* The **actual texture rendered on screen** during active gameplay.
* Completely clean of any colored guide lines, bounding boxes, or purple numbering.
* Features handcrafted pixel-art visuals: **dark slate/obsidian backgrounds**, **polished brushed steel double-bevels**, recessed inner-shadow slot cavities, and subtle dithered lighting.
* The engine draws this clean background plate and dynamically anchors item icons, stack count typography, health/mana meters, and cursor highlights strictly to the coordinates extracted from the corresponding `_outline` blueprint.

---

## 2. Blueprint Color Coding Convention (`_outline`)

In all `*_outline.png` files, colors are strictly reserved for parsing semantics:

| Visual Element | Hex Color | RGBA Value | Engine Function |
| :--- | :---: | :---: | :--- |
| **Card Boundaries** | `#ED1C24` | `(237, 28, 36, 255)` | Outer perimeter and bounding silhouette of each modular card/window. |
| **Slot Cavities** | `#880015` | `(136, 0, 21, 255)` | Item receptacles (standard 16×16 px or compact 14×14 px equipment slots). |
| **Interactive Fields** | `#FF7F27` | `(255, 127, 39, 255)` | Action buttons, tabs, title plates, text search inputs, scrollbar tracks, and meters. |
| **Field Numbers & Letters** | `#A349A4` | `(163, 73, 164, 255)` | 3×5 px raster typography with 1px tracking used for unique element identification. |

---

## 3. Modular Card Architecture

The player inventory system is assembled from **independent modular cards** aligned horizontally:

```
┌──────────────┐ ┌──────────────────────────────────────┐ ┌──────────────┐
│    CARD A    │ │                CARD B                │ │    CARD C    │
│Player & Armor│ │         Main Inventory & Hotbar      │ │   Crafting   │
│ (88×128 px)  │ │            (222×128 px)              │ │ (88×128 px)  │
└──────────────┘ └──────────────────────────────────────┘ └──────────────┘
                                     ▲
                       [On container / chest screens]
                 ┌──────────────────────────────────────┐
                 │                CARD D                │
                 │        Single or Double Chest        │
                 │       (210×102 px / 372×102 px)      │
                 └──────────────────────────────────────┘
```

---

### Card A — Player Character & Equipment (`A`)
Located on the lower-left quadrant (`x=5..92, y=125..252`), measuring 88 × 128 px. Handles live 3D character preview, equipped armor pieces, and accessory management.

> [!NOTE]
> **Implementation Status**: Card A armor slots (A3..A8) are rendered as **visual / non-interactive placeholders** in the engine for now, because armor equip mechanics and character stats systems are not yet implemented.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **A1** | `(19, 132)` | 60 × 8 px | Interactive / Header | Recessed header plate for player name, class title, or current character level. |
| **A2** | `(22, 140)` | 54 × 64 px | 3D Viewport | Live 3D character viewport with an elliptical stone pedestal at its base. Supports interactive character rotation via mouse drag. |
| **A3** | `(25, 210)` | 14 × 14 px | Equipment Slot (WIP) | **Helmet / Headgear** slot (`helmet`). Displays a translucent 14×14 px overlay when vacant. Non-interactive placeholder. |
| **A4** | `(42, 210)` | 14 × 14 px | Equipment Slot (WIP) | **Chestplate / Torso Armor** slot (`chestplate`). Displays a 14×14 px overlay when vacant. Non-interactive placeholder. |
| **A5** | `(59, 210)` | 14 × 14 px | Equipment Slot (WIP) | **Leggings / Leg Armor** slot (`leggings`). Displays a 14×14 px overlay when vacant. Non-interactive placeholder. |
| **A6** | `(25, 226)` | 14 × 14 px | Equipment Slot (WIP) | **Boots / Footwear** slot (`boots`). Displays a 14×14 px overlay when vacant. Non-interactive placeholder. |
| **A7** | `(42, 226)` | 14 × 14 px | Equipment Slot (WIP) | **Gloves / Gauntlets** slot (`gloves`). Displays a 14×14 px overlay when vacant. Non-interactive placeholder. |
| **A8** | `(59, 226)` | 14 × 14 px | Equipment Slot (WIP) | **Amulet / Ring / Accessory** slot (`accessory`). Displays a 14×14 px overlay when vacant. Non-interactive placeholder. |

---

### Card B — Main Inventory, Hotbar, and Creative Catalog (`B`)
Central card (`x=97..318`). Its internal layout adapts dynamically according to the active game mode and creative toggle state:

#### 1. Variant `inventory_closed` (Standard Survival Inventory — Deferred)
Measures 222 × 128 px (`x=97..318, y=125..252`). Standard survival inventory without the top creative toggle tab.
* **Usage Status**: **Not used currently**. Reserved for future survival mode gameplay when health/resource loops are introduced.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **B1** | `(121, 135)` | 60 × 8 px | Header | Recessed title plate inscribed with "Inventory". |
| **B2** | `(268, 135)` | 8 × 8 px | Action Button | **Auto-Sort**: sorts inventory stacks by item category, rarity, or name. |
| **B3** | `(278, 135)` | 8 × 8 px | Action Button | **Quick Discard / Trash**: quickly incinerates or drops the item stack under the cursor. |
| **B4** | `(288, 135)` | 8 × 8 px | Action Button | **UI Preferences**: toggles extended tooltip stats and numerical durability readouts. |
| **B5** | `(298, 135)` | 8 × 8 px | Action Button | **Inventory Lock**: locks favored slots to prevent accidental transfer or dumping. |
| **B6 to B41** | `y=148, 166, 184, 202` | 16 × 16 px | Item Slot | **36 Main Inventory Slots**: arranged in 4 rows of 9 slots (`x=124..276`). |
| **B42** | `(104, 228)` | 14 × 14 px | Special Slot | **Off-hand / Shield / Torch Slot**: handles secondary equipment. |
| **B43 to B51**| `(124..276, 227)` | 16 × 16 px | Item Slot | **9 Central Hotbar Slots**: primary quickbar mapped to keyboard keys 1 through 9. |
| **B52** | `(298, 228)` | 14 × 14 px | Special Slot | **Utility / Quiver Slot**: dedicated slot for arrows, throwing knives, or grapple hooks. |

---

#### 2. Variant `inventory_creative_closed` (Primary Active Player Inventory)
The **primary standard inventory** used during gameplay in the current engine state. Features the player inventory augmented with the top tab **B1** that toggles the creative catalog.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **B1** | `(118, 116)` | **10 × 10 px** | Button / Tab | **Creative Mode Toggle Tab**: top tab button (10×10 px) that expands into the full creative catalog (`inventory_creative_open`). |
| **B2** | `(121, 135)` | 60 × 8 px | Header | Title plate inscribed with "Inventory". |
| **B3 to B6** | `(268..298, 135)` | 8 × 8 px | Action Buttons | 4 utility action buttons (sort, trash, preferences, lock). |
| **B7 to B42** | `y=148, 166, 184, 202` | 16 × 16 px | Item Slot | **36 Main Inventory Slots** (personal player inventory). Fully interactive with hover & clicks. |
| **B43** | `(104, 228)` | 14 × 14 px | Special Slot | Special left slot (off-hand / shield). |
| **B44 to B52**| `(124..276, 227)` | 16 × 16 px | Item Slot | **9 Central Hotbar Slots** (mirrored hotbar embedded in inventory, fully interactive with mouse). |
| **B53** | `(298, 228)` | 14 × 14 px | Special Slot | Special right slot (utility / quiver). |

---

#### 3. Variant `inventory_creative_open` (Expanded Creative Catalog)
Card B expands vertically to the top of the canvas (`x=97..318, y=6..255`), replacing standard storage with a comprehensive categorized catalog.

> [!TIP]
> **Category Tab Block Icons**: Each category tab (B2..B12) renders an isometric 3D block icon identical in style to the block icons rendered in inventory/hotbar slots, scaled down to fit inside the ~14×14 px tab bounds, representing that category's block type.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Category Representative Block Icon |
| :---: | :---: | :---: | :---: | :--- |
| **B1** | `(118, 8)` | **10 × 10 px** | Button / Tab | **Creative Mode Toggle Tab**: remains B1 (10×10 px), clicked to collapse back to compact mode (`inventory_creative_closed`). |
| **B2** | `(132, 8)` | **12 × 12 px** | Category Tab | **Rocks & Geology (`rocks`)**: renders miniature 3D rock icon (e.g., `Dolomite`). |
| **B3** | `(146, 8)` | **12 × 12 px** | Category Tab | **Trees & Timber (`trees`)**: renders miniature 3D timber icon (e.g., `WoodLogOak`). |
| **B4** | `(160, 8)` | **12 × 12 px** | Category Tab | **Soils & Terrain (`soils`)**: renders miniature 3D soil icon (e.g., `LoamDirt`). |
| **B5** | `(174, 8)` | **12 × 12 px** | Category Tab | **Ores & Minerals (`ores`)**: renders miniature 3D ore icon (e.g., `OreIron`). |
| **B6** | `(188, 8)` | **12 × 12 px** | Category Tab | **Caverns & Subterranea (`caverns`)**: renders miniature 3D cavern icon (e.g., `Dripstone`). |
| **B7** | `(202, 8)` | **12 × 12 px** | Category Tab | **Oceans & Marine (`oceans`)**: renders miniature 3D marine icon (e.g., `Coral`). |
| **B8** | `(216, 8)` | **12 × 12 px** | Category Tab | **Organics & Fauna (`organics`)**: renders miniature 3D bio icon (e.g., `BoneBlock`). |
| **B9** | `(230, 8)` | **12 × 12 px** | Category Tab | **Decorations & Architecture (`decorations`)**: renders miniature 3D building icon (e.g., `Bricks`). |
| **B10**| `(244, 8)` | **12 × 12 px** | Category Tab | **Mycology & Fungi (`mushrooms`)**: renders miniature 3D mushroom icon (e.g., `MushroomCap`). |
| **B11**| `(258, 8)` | **12 × 12 px** | Category Tab | **Vegetation & Flora (`vegetation`)**: renders miniature 3D flora icon (e.g., `Wildflowers`). |
| **B12**| `(272, 8)` | **12 × 12 px** | Category Tab | **Fluids & Systems (`fluids` / `systems`)**: renders miniature 3D fluid icon (e.g., `Water`). |
| **B13**| `(121, 27)` | 60 × 8 px | Header | Category title plate displaying the active category name (e.g., "Rocks", "Trees"). |
| **B14**| `(223, 27)` | 72 × 8 px | Text Input | Search box with instant substring filtering across item localized names and tags. |
| **B15**| `(298, 40..205)` | 6 × 166 px | Scroll Track | Vertical scroll track navigated by the draggable `inventory_creative_scroller.png`. |
| **B16 to B105** | `y=43..205` (10 rows) | 16 × 16 px | Catalog Slot | **90 Creative Catalog Slots**: 10 rows of 9 slots (`x=124..276`). Clicking grants infinite stacks without consuming world resources. |
| **B106**| `(104, 228)` | 14 × 14 px | Special Slot | Special left hotbar slot (off-hand). |
| **B107 to B115** | `(124..276, 227)` | 16 × 16 px | Item Slot | **9 Central Hotbar Slots** (fully editable and accessible inside creative catalog). |
| **B116**| `(298, 228)` | 14 × 14 px | Special Slot | Special right hotbar slot (utility). |

---

### Card C — Crafting Grid & Synthesis (`C`)
Located on the lower-right quadrant (`x=323..410, y=125..252`), measuring 88 × 128 px. Handles recipe synthesis and item crafting.

> [!NOTE]
> **Implementation Status**: Card C crafting matrix and output slot (C2..C11) are rendered as **visual / non-interactive placeholders** in the engine for now, because crafting recipe trees and output evaluation are not yet implemented.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **C1** | `(337, 132)` | 60 × 8 px | Header | Recessed header plate inscribed with "Crafting". |
| **C2 to C10** | `y=148, 166, 184` | 16 × 16 px | Matrix Slot (WIP) | **3×3 Crafting Matrix**: columns at `x=340, 359, 378`. Non-interactive placeholder. |
| **Arrow** | `(363, 206)` | 8 × 12 px | Indicator | Engraved stone directional arrow indicating workflow toward output. |
| **C11** | `(359, 224)` | 16 × 16 px | Output Slot (WIP) | **Crafting Output Slot**: reinforced with polished steel double-bevel. Non-interactive placeholder. |

---

### Card D — Storage Containers & Chests (`D` — Deferred / Not Used)
Card D occupies the upper portion of the interface (`y=20..121`) whenever the player opens an external storage container.

> [!IMPORTANT]
> **Implementation Status**: **Deferred / Not used currently**. Chest blocks, double chests, and external container entities do not exist in the engine yet. Neither `inventory_chest_open` nor `inventory_chest_double_open` will be loaded or hooked up until container systems are developed.

#### 1. Variant `inventory_chest_open` (Single Chest)
Measures 210 × 102 px (`x=103..312, y=20..121`), aligned with Card B.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **D1** | `(121, 27)` | 60 × 8 px | Header | Title plate inscribed with "Chest". |
| **D2** | `(251, 27)` | 8 × 8 px | Container Action | **Quick Deposit**: moves matching items from player inventory into the chest. |
| **D3** | `(265, 27)` | 8 × 8 px | Container Action | **Auto-Sort**: sorts chest contents by category and item count. |
| **D4** | `(279, 27)` | 8 × 8 px | Container Action | **Lock Chest**: toggles protection against unintended removal. |
| **D5** | `(293, 27)` | 8 × 8 px | Container Action | **Close Chest**: quick exit button. |
| **D6 to D41** | `y=43, 61, 79, 97` | 16 × 16 px | Container Slot | **36 Chest Slots**: arranged in 4 rows of 9 slots (`x=124..276`), mirroring player inventory capacity. |

---

#### 2. Variant `inventory_chest_double_open` (Double Chest)
Measures 372 × 102 px (`x=22..393, y=20..121`), expanding horizontally with side wings over Cards A and C.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **D1** | `(45, 27)` | 60 × 8 px | Header | Title plate inscribed with "Double Chest". |
| **D2 to D5** | `(327..370, 27)` | 8 × 8 px | Container Actions | 4 container action buttons (quick deposit, sort, lock, close). |
| **D6 to D77** | `y=43, 61, 79, 97` | 16 × 16 px | Container Slot | **72 Double Chest Slots** (**exactly double** the 36 slots of single chest):<br>• Row 1 (`y=43`): 17 slots (`x=48..352`)<br>• Row 2 (`y=61`): 19 slots (`x=29..371`, expanding over side wings)<br>• Row 3 (`y=79`): 19 slots (`x=29..371`)<br>• Row 4 (`y=97`): 17 slots (`x=48..352`) |

---

## 4. Hotbar HUD (`hotbar.png` and `hotbar_outline.png`)

The persistent in-game quickbar HUD rendered at the bottom of the screen measures **320 × 44 px**:

```
                       ┌────────────────────────────────────────┐
                       │ 1. Experience / Level Progress Bar(XP) │
                       └────────────────────────────────────────┘
  ┌───────┐ ┌──────┐ ┌──┬──┬──┬──┬──┬──┬──┬──┬──┐ ┌──────┐ ┌───────┐
  │ 2 & 3 │ │  4   │ │5 │6 │7 │8 │9 │10│11│12│13│ │  14  │ │15 & 16│
  │Badges │ │Off-H.│ │      9 Primary Hotbar Slots│ │Util. │ │Badges │
  └───────┘ └──────┘ └──┴──┴──┴──┴──┴──┴──┴──┴──┘ └──────┘ └───────┘
              ┌───────────────────┐    ┌───────────────────┐
              │17. Health Bar(HP) │    │18. Mana/Stam Bar  │
              └───────────────────┘    └───────────────────┘
```

> [!IMPORTANT]
> **Gameplay Hotbar Interaction Rules (No Mouse Hover / No Clicks)**:
> 1. **Zero Mouse Hover**: During active 3D gameplay, the mouse cursor is captured by the first-person / third-person camera. Because there is no free cursor on screen, **the gameplay hotbar has no mouse hover states and no mouse clicks**.
> 2. **No Hover Textures on Gameplay Hotbar**: Hover textures like `inventory_slot_hover.png` have no logical purpose on the gameplay hotbar and must **never** be rendered on it.
> 3. **Active Selection Box Only**: The active slot is visually highlighted solely by drawing `hotbar_selection.png` (20 × 20 px) over one of the 9 primary slots (fields 5..13), operated strictly via keyboard keys `1` through `9` and the mouse scroll wheel.
> 4. **Wing Fields (2, 3, 15, 16) are Non-Clickable Badges**: The 4 small 8 × 8 px corner slots (2 on each wing) are **not clickable buttons**. They serve as static HUD indicator badges to display hotkey letters (e.g. `[M]`) or mini icons (e.g. mini map icon) showing keyboard shortcuts to the player.
> 5. **Gameplay Hotbar vs. In-Inventory Hotbar**: When an inventory screen (such as `inventory_creative_closed.png`) is opened, the cursor unlocks, but the background gameplay hotbar stays in the background and is non-clickable / inactive. Item interaction with hotbar slots occurs exclusively inside the mirrored hotbar embedded in Card B of the inventory window.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | HUD Function |
| :---: | :---: | :---: | :---: | :--- |
| **1** | `(59, 4)` | 204 × 4 px | Progress Meter (Static) | Recessed groove for **Experience Bar (XP)**. Rendered as static visual groove (leveling not implemented). |
| **2** | `(18, 25)` | 8 × 8 px | Keybind Badge | Left wing badge 1: non-clickable hotkey indicator (e.g., `[M]` for Map or HUD icon). |
| **3** | `(30, 24)` | 8 × 8 px | Keybind Badge | Left wing badge 2: non-clickable hotkey indicator (e.g., `[J]` for Journal or HUD icon). |
| **4** | `(45, 17)` | 14 × 14 px | Special Display | **Off-hand Slot**: visual readout of equipped shield, torch, or secondary tool (non-clickable in gameplay HUD). |
| **5 to 13** | `y=14` | 16 × 16 px | Item Slot | **9 Primary Hotbar Slots** (`x=65, 87, 109, 131, 153, 175, 197, 219, 241`). Cycled via mouse scroll wheel or number keys 1 through 9. Highlighted by `hotbar_selection.png`. Zero mouse hover. |
| **14** | `(263, 17)` | 14 × 14 px | Special Display | **Quick Utility Slot**: visual readout of quiver, grappling hook, or movement gadget (non-clickable in gameplay HUD). |
| **15** | `(284, 24)` | 8 × 8 px | Keybind Badge | Right wing badge 1: non-clickable hotkey indicator (e.g., `[C]` for Crafting or HUD icon). |
| **16** | `(296, 25)` | 8 × 8 px | Keybind Badge | Right wing badge 2: non-clickable hotkey indicator (e.g., `[T]` for Chat or HUD icon). |
| **17** | `(68, 36)` | 82 × 4 px | Progress Meter (Static) | Recessed groove for **Health Points Bar (HP)**. Rendered as static visual placeholder (health system not implemented). |
| **18** | `(172, 36)` | 82 × 4 px | Progress Meter (Static) | Recessed groove for **Mana / Stamina Bar**. Rendered as static visual placeholder (mana system not implemented). |

---

## 5. Interactive Button Family

System buttons are standardized with 1px pixel-art chamfered corners, brushed-steel double-bevels, and a consistent height metric:

| Component | Default Texture | Hover Texture | Dimensions | Aspect Ratio | Primary Use Case |
| :--- | :--- | :--- | :---: | :---: | :--- |
| **Button Tiny (Util)** | [`button_util.png`](interfaces/containers/button_util.png) | [`button_util_hover.png`](interfaces/containers/button_util_hover.png) | **8 × 8 px** | 1:1 | Card header utility actions (B2..B5, D2..D5). *(Note: Not used as clickable buttons on gameplay hotbar).* |
| **Button Small** | [`button_small.png`](interfaces/containers/button_small.png) | [`button_small_hover.png`](interfaces/containers/button_small_hover.png) | **16 × 16 px** | 1:1 | Compact icon buttons, page pagination, and secondary interface actions. |
| **Button Medium** | [`button_medium.png`](interfaces/containers/button_medium.png) | [`button_medium_hover.png`](interfaces/containers/button_medium_hover.png) | **64 × 16 px** | 4:1 | Confirmation prompts, cancel, save, modal buttons, and major tabs. |
| **Button Large** | [`button_large.png`](interfaces/containers/button_large.png) | [`button_large_hover.png`](interfaces/containers/button_large_hover.png) | **96 × 16 px** | 6:1 | Main menu screens, "Enter World", "Settings", and "Quit Game". |

### 5.1 Button Construction Rules
* **Outer Rim (Layer 0)**: `#2d2e30` top, `#1e1e20` sides, and `#000000` base with 1px cut corners.
* **Metallic Bevel (Layer 1)**: Light brushed steel highlight (`#b4b5af` top, lateral gradient from `#9c9c96` to `#28292b`), and deep shadow base (`#18191b` / `#141416`).
* **Inner Groove (Layer 2)**: 1px dark inset groove (`#38393c` top, `#101012` bottom).
* **Center Surface**: Dark graphite face (`#28292c` transitioning downward to `#151618`) with subtle `(x+y)%4` micro-dithering for tactile texture.
* **Hover State**: Highlights brighten to `#fafcff` / `#dae2ec` along the top rim, `#b4beca` on the left, and the face receives a frosted metallic slate-blue hue (`#3c4552` / `#2c333e`), delivering high tactile contrast.

---

## 6. Standalone Selection, Navigation, and Hover State Textures

In addition to full interface backplates, the engine utilizes standalone modular textures for dynamic interaction states:

```
┌──────────────────┐    ┌──────────────┐    ┌──────────────────┐
│ hotbar_selection │    │ tab_selection│    │ creative_scroller│
│   (20 × 20 px)   │    │ (16 × 16 px) │    │   (12 × 16 px)   │
└──────────────────┘    └──────────────┘    └──────────────────┘
         ▲                     ▲                     ▲
┌──────────────────┐    ┌──────────────┐    ┌──────────────────┐
│slot_hover (16×16)│    │slot_hov(14×14│    │button_util (8×8) │
└──────────────────┘    └──────────────┘    └──────────────────┘
```

### 6.1 Selection & Navigation Overlays
* **`hotbar_selection.png` (20 × 20 px)**:  
  Thick outer highlight frame with polished steel double-bevel (`#ffffff` top, `#b4beca` sides, recessed corners `#1e2026`). Features a hollow center (16×16 px clear) that wraps cleanly around the active hotbar slot, visually indicating the currently held item on the gameplay HUD or in menus.
* **`inventory_creative_tab_selection.png` (16 × 16 px)**:  
  Selection frame with an open base designed to dock directly over the active creative category tab (B2..B12). Visually bridges the active tab into the main catalog body.
* **`inventory_creative_scroller.png` (12 × 16 px)**:  
  Tactile thumb slider for the vertical scrollbar track (B15, 6×166 px). Built with thick slate and chamfered edges, supporting smooth dragging or mouse-wheel scrolling.

### 6.2 Slot & Button Hover States (Menu-Exclusive)
> [!IMPORTANT]
> Hover textures are strictly for **opened inventory and menu windows** where the mouse cursor is active. They are **never** used on the gameplay hotbar HUD.
* **`inventory_slot_hover.png` (16 × 16 px)**:  
  Illuminated hover state for standard 16×16 px slots in opened menus. Retains the 1px recessed cavity with top-left shadow (`#1c2128` / `#202630`), an illuminated slate-blue floor (`#2c333e` / `#343c48`), a bright steel bottom/right inner lip (`#8c96a2` / `#b4beca`), and a specular glint point (`#dae2ec`) at the bottom-right corner.
* **`inventory_slot_special_hover.png` (14 × 14 px)**:  
  Adapted for compact 14×14 px slots in opened menus (off-hand and utility slots), replicating the exact same illuminated cavity effect.
* **`button_util_hover.png` (8 × 8 px)**:  
  Illuminated hover state for the 8×8 px utility action buttons inside menu headers (sort, trash, lock).


---

## 7. Slot Overlay System (`docs/worldbuilding/interfaces/containers/slots/`)

To achieve a compact footprint for player equipment (Card A) and hotbar utility slots, these receptacles are dimensioned to **14 × 14 px**.

Accordingly, **all 14 slot overlays are standardized at 14 × 14 px**, with each pixel-art icon centered with a minimum 1px margin and rendered in translucent **Dark Iron**:

* **Dark Iron Palette**:
  * Top highlight: `#62666E` `(98, 102, 110)`
  * Main body: `#3C3F45` `(60, 63, 69)`
  * Shadow / Silhouette: `#222428` `(34, 36, 40)`
  * Background: Fully Transparent `(0, 0, 0, 0)`

| Overlay (14 × 14 px) | Texture File | Target Interface Slot |
| :--- | :--- | :--- |
| **Helmet / Headgear** | [`helmet.png`](interfaces/containers/slots/helmet.png) | Slot **A3** (Card A) |
| **Chestplate / Torso Armor** | [`chestplate.png`](interfaces/containers/slots/chestplate.png) | Slot **A4** (Card A) |
| **Leggings / Leg Armor** | [`leggings.png`](interfaces/containers/slots/leggings.png) | Slot **A5** (Card A) |
| **Boots / Footwear** | [`boots.png`](interfaces/containers/slots/boots.png) | Slot **A6** (Card A) |
| **Gloves / Gauntlets** | [`gloves.png`](interfaces/containers/slots/gloves.png) | Slot **A7** (Card A) |
| **Amulet / Necklace** | [`necklace.png`](interfaces/containers/slots/necklace.png) | Slot **A8** (Card A) |
| **Shield / Off-hand** | [`shield.png`](interfaces/containers/slots/shield.png) | Slot **B42** (Inventory) / **Field 4** (Hotbar) |
| **Quiver / Projectiles** | [`quiver.png`](interfaces/containers/slots/quiver.png) | Slot **B52** (Inventory) / **Field 14** (Hotbar) |
| **Backpack / Caddy** | [`backpack.png`](interfaces/containers/slots/backpack.png) | Future expansion bag / extra storage slot |
| **Tool Belt / Pouch** | [`belt.png`](interfaces/containers/slots/belt.png) | Utility quick-belt accessory slot |
| **Attribute Ring** | [`ring.png`](interfaces/containers/slots/ring.png) | Finger ring / stat jewel slot |
| **Magic Rune** | [`rune.png`](interfaces/containers/slots/rune.png) | Runic socket / catalyst gem slot |
| **Alchemy Potion** | [`potion.png`](interfaces/containers/slots/potion.png) | Quick potion / elixir flask slot |
| **Cape / Mantle / Glider** | [`cape.png`](interfaces/containers/slots/cape.png) | Cosmetic mantle / cloak / gliding wing slot |

---

## 8. Interface Icon Repository (`docs/worldbuilding/interfaces/icons/`)

The `interfaces/icons/` directory acts as an **asset library of 16 × 16 px pixel-art icons** prepared for current and future interface subsystems (HUD clock, quest log, weather forecast, biome indicators, and character creator).

> [!NOTE]
> **Asset Library Development Status**  
> These icons currently **do not have hardcoded gameplay logic bindings in the engine** (they are intentionally pre-staged assets). Because worldbuilding, climates, biomes, and character creation are actively evolving, the collections below are open-ended and will receive additional icons as new systems are formalized.

### 8.1 Diurnal Phases / Sun Clock (`time_*.png`)
Icons designed for the HUD sun dial / temporal compass:
* [`time_morning.png`](interfaces/icons/time_morning.png): Dawn / Morning (rising sun over dew grass)
* [`time_noon.png`](interfaces/icons/time_noon.png): Noon / Zenith (bright golden sun at its peak)
* [`time_evening.png`](interfaces/icons/time_evening.png): Dusk / Evening (orange setting sun at horizon)
* [`time_night.png`](interfaces/icons/time_night.png): Night / Lunar Zenith (crescent moon against starfield)

### 8.2 Weather Conditions (`weather_*.png`)
Atmospheric icons for weather forecasts and environmental HUD:
* [`weather_clear.png`](interfaces/icons/weather_clear.png): Clear skies / bright sunny day
* [`weather_raining.png`](interfaces/icons/weather_raining.png): Moderate rainfall / storm cloud with water droplets
* [`weather_thunder.png`](interfaces/icons/weather_thunder.png): Thunderstorm / dark tempest cloud with lightning bolt
* [`weather_snow.png`](interfaces/icons/weather_snow.png): Snowfall / cold blizzard cloud with drifting crystalline snowflakes

### 8.3 Biome & Terrain Indicators (`biome_*.png`)
Environmental icons representing vegetation, geology, and terrain:
* [`biome_plains.png`](interfaces/icons/biome_plains.png): Open grassy plains
* [`biome_forest.png`](interfaces/icons/biome_forest.png): Temperate oak/pine forest
* [`biome_dark_forest.png`](interfaces/icons/biome_dark_forest.png): Dense canopy dark forest
* [`biome_birch_forest.png`](interfaces/icons/biome_birch_forest.png): Birch grove
* [`biome_taiga.png`](interfaces/icons/biome_taiga.png): Boreal conifer taiga
* [`biome_desert.png`](interfaces/icons/biome_desert.png): Arid sand dunes
* [`biome_savanna.png`](interfaces/icons/biome_savanna.png): Tropical savanna
* [`biome_beach.png`](interfaces/icons/biome_beach.png): Coastal shore and beaches
* [`biome_river.png`](interfaces/icons/biome_river.png): Freshwater riverbed
* [`biome_frozen_peaks.png`](interfaces/icons/biome_frozen_peaks.png): Glacial peaks and snowy crags

### 8.4 Quest Log & Expedition Journal
* [`journal_board.png`](interfaces/icons/journal_board.png): Notice board / Player expedition quest log icon

### 8.5 Playable Character Races (`race_*.png`)
Blocky, voxel-styled cubic character head portraits (16 × 16 px) designed for the character creation screen:
* [`race_human.png`](interfaces/icons/race_human.png): **Human**: classic adventurer portrait with brown hair, peach skin, expressive eyes, and blue tunic collar.
* [`race_elf.png`](interfaces/icons/race_elf.png): **Elf**: slender cubic head with elongated pointed ears, golden locks, emerald green eyes, and silver circlet.
* [`race_dwarf.png`](interfaces/icons/race_dwarf.png): **Dwarf**: iron-banded helmet, resolute dark eyes, and a massive copper-braided beard adorned with gold clasp rings.
* [`race_orc.png`](interfaces/icons/race_orc.png): **Orc**: olive-green cubic head, topknot hair, amber eyes, pointed ears, and sharp white lower tusks protruding over the lip.
* [`race_drow.png`](interfaces/icons/race_drow.png): **Drow**: obsidian-ashen skin, long pointed ears, snowy platinum hair, and glowing lilac-crimson eyes.
* [`race_halfling.png`](interfaces/icons/race_halfling.png): **Halfling**: cheerful cubic head crowned with curly chestnut hair puffs, bright hazel eyes, and rosy cheeks.
* [`race_tiefling.png`](interfaces/icons/race_tiefling.png): **Tiefling**: deep crimson skin, dark swept-back demonic horns curving from the brow, and solid glowing sulfur-gold eyes.
* [`race_dragonborn.png`](interfaces/icons/race_dragonborn.png): **Dragonborn**: draconic crest horns, bronze reptilian scales, blocky snout with nostril slits, and golden vertical-slit reptilian eyes.

---

## 9. Complete Interface Asset Manifest

The consolidated table below catalogs all assets currently available in the `docs/worldbuilding/interfaces/` directory:

| Asset File | Dimensions | Format | Engine Role & Description |
| :--- | :---: | :---: | :--- |
| [`inventory_closed.png`](interfaces/inventory_closed.png) | 416 × 256 px | RGBA | Final rendered background plate for closed player inventory (Cards A, B, and C). |
| [`inventory_closed_outline.png`](interfaces/inventory_closed_outline.png) | 416 × 256 px | RGBA | Hitbox blueprint and sequential field IDs for `inventory_closed` (A1..A8, B1..B52, C1..C11). |
| [`inventory_creative_closed.png`](interfaces/inventory_creative_closed.png) | 416 × 256 px | RGBA | Final rendered plate for compact creative inventory with top 10×10 px toggle tab B1. |
| [`inventory_creative_closed_outline.png`](interfaces/inventory_creative_closed_outline.png) | 416 × 256 px | RGBA | Hitbox blueprint for `inventory_creative_closed` (Fields B1 to B53). |
| [`inventory_creative_open.png`](interfaces/inventory_creative_open.png) | 416 × 256 px | RGBA | Final rendered plate for expanded creative catalog (10×10 px toggle B1, 11 category tabs B2..B12 of 12×12 px, search B14, and 90 catalog slots B16..B105). |
| [`inventory_creative_open_outline.png`](interfaces/inventory_creative_open_outline.png) | 416 × 256 px | RGBA | Hitbox blueprint for expanded creative catalog (Fields B1 to B116). |
| [`inventory_chest_open.png`](interfaces/inventory_chest_open.png) | 416 × 256 px | RGBA | Final rendered plate for Single Chest (36 chest slots in Card D + lower player inventory). |
| [`inventory_chest_open_outline.png`](interfaces/inventory_chest_open_outline.png) | 416 × 256 px | RGBA | Hitbox blueprint for Single Chest (Card D with fields D1 to D41). |
| [`inventory_chest_double_open.png`](interfaces/inventory_chest_double_open.png) | 416 × 256 px | RGBA | Final rendered plate for Double Chest (72 chest slots in Card D with side wing expansions). |
| [`inventory_chest_double_open_outline.png`](interfaces/inventory_chest_double_open_outline.png) | 416 × 256 px | RGBA | Hitbox blueprint for Double Chest (Card D with fields D1 to D77). |
| [`hotbar.png`](interfaces/hotbar.png) | 320 × 44 px | RGBA | Final rendered texture for in-game persistent Hotbar HUD. |
| [`hotbar_outline.png`](interfaces/hotbar_outline.png) | 320 × 44 px | RGBA | Hitbox blueprint for Hotbar HUD (18 numbered functional fields). |
| [`hotbar_selection.png`](interfaces/hotbar_selection.png) | 20 × 20 px | RGBA | Highlight selection frame with hollow center wrapping the active hotbar slot. |
| [`inventory_creative_scroller.png`](interfaces/inventory_creative_scroller.png) | 12 × 16 px | RGBA | Draggable thumb slider for creative catalog vertical scroll track (track B15). |
| [`inventory_creative_tab_selection.png`](interfaces/inventory_creative_tab_selection.png) | 16 × 16 px | RGBA | Open-base highlight frame docking over the active creative category tab. |
| [`inventory_slot_hover.png`](interfaces/inventory_slot_hover.png) | 16 × 16 px | RGBA | Illuminated hover state texture for standard 16×16 px slots. |
| [`inventory_slot_special_hover.png`](interfaces/inventory_slot_special_hover.png) | 14 × 14 px | RGBA | Illuminated hover state texture for compact 14×14 px equipment and utility slots. |
| [`button_util.png`](interfaces/button_util.png) | 8 × 8 px | RGBA | Miniature utility button in default idle state. |
| [`button_util_hover.png`](interfaces/button_util_hover.png) | 8 × 8 px | RGBA | Miniature utility button in illuminated hover state. |
| [`button_small.png`](interfaces/button_small.png) | 16 × 16 px | RGBA | Small button in default idle state. |
| [`button_small_hover.png`](interfaces/button_small_hover.png) | 16 × 16 px | RGBA | Small button in illuminated hover state. |
| [`button_medium.png`](interfaces/button_medium.png) | 64 × 16 px | RGBA | Medium button in default idle state. |
| [`button_medium_hover.png`](interfaces/button_medium_hover.png) | 64 × 16 px | RGBA | Medium button in illuminated hover state. |
| [`button_large.png`](interfaces/button_large.png) | 96 × 16 px | RGBA | Large button in default idle state. |
| [`button_large_hover.png`](interfaces/button_large_hover.png) | 96 × 16 px | RGBA | Large button in illuminated hover state. |
| **Slot Overlays (14 files)** | 14 × 14 px | RGBA | Translucent Dark Iron equipment icons in `interfaces/slots/`. |
| **Interface Icons (23 files)** | 16 × 16 px | RGBA | Interface icons (time, weather, biomes, quest journal, character races) in `interfaces/icons/`. |
