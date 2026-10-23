# Voxel Game Engine — User Interface Design & Architecture (`interfaces_design.md`)

This document formally specifies and standardizes the **Fase 2 User Interface (UI / HUD) Architecture** for the Voxel Game Engine.

> [!IMPORTANT]
> **New 16:9 Fullscreen Architecture (480 × 270 px @ 4× Nearest Neighbor)**  
> The entire interface architecture has been refactored around a native base resolution of **480 × 270 pixels**, scaled exactly **4× via Nearest-Neighbor sampling** to render in full **1920 × 1080 (Full HD)** across all modern displays.
>
> Unlike previous floating window cards, all major interfaces now span the complete **16:9 screen frame**, featuring:
>
> 1. A unified, persistent **Top Menu Bar** (`480 × 20 px`) shared across all modal interfaces (Inventory, Creative Catalog, Map, Pause/Settings).
> 2. The persistent **In-Game Hotbar HUD** displayed during active 3D gameplay (when modal screens are closed).
> 3. Standardized **16 × 16 px slot cavities** with 3px gutters (19 px pitch) for pixel-perfect 1:1 texel alignment.

---

## 1. Architectural Principle: Blueprint Skeleton (`_outline`) vs. Clean Render (`.png`)

The engine maintains a **strict dual-layer decoupled architecture** separating interaction logic and spatial layout from cosmetic pixel art:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                      ENGINE UI RENDERING PIPELINE                      │
├───────────────────────────────────┬────────────────────────────────────┤
│   1. Geometric Blueprint Layer    │   2. Clean Cosmetic Render Layer   │
│      (*_outline.png / 480x270_*)  │      (*.png / Clean Plate)         │
├───────────────────────────────────┼────────────────────────────────────┤
│ • Functional geometric skeleton   │ • Final visual game skin           │
│ • Window boundary perimeters      │ • Dark slate / obsidian palette    │
│ • Slot & button hitboxes          │ • 0 guide lines or ID numbers      │
│ • Unique field IDs (purple text)  │ • 1px brushed steel double bevels  │
│ • Parsed on boot / asset bake     │ • Rendered live during gameplay    │
└───────────────────────────────────┴────────────────────────────────────┘
```

### 1.1 Blueprint Textures & Color-Coding Convention

In all blueprint textures in `docs/worldbuilding/interfaces/containers/480x270/`, pixel colors are strictly reserved for parsing semantics:

| Visual Element | Hex Color | RGBA Value | Engine Function |
| :--- | :---: | :---: | :--- |
| **Card / Window Perimeter** | `#ED1C24` | `(237, 28, 36, 255)` | Outer bounding borders, silhouettes of windows, and stylistic decorative borders. |
| **Slot Cavities** | `#880015` | `(136, 0, 21, 255)` | Item receptacles (standard 16×16 px slots and 3D character viewport). |
| **Interactive Fields** | `#FF7F27` | `(255, 127, 39, 255)` | Navigation tabs, action buttons, title plates, text search inputs, scrollbar tracks, and meters. |
| **Field Numbers & Legends** | `#A349A4` | `(163, 73, 164, 255)` | 3×5 px raster pixel typography with 1px tracking used for element indexing and automated parser mapping. |

---

## 2. Shared Top Menu Bar (`480x270_menu_bar`)

📂 **Blueprint**: [`480x270_menu_bar_outline.png`](interfaces/containers/480x270/480x270_menu_bar_outline.png)  
🎨 **Clean Render**: [`480x270_menu_bar.png`](interfaces/containers/480x270/480x270_menu_bar.png)  
📏 **Dimensions**: 480 × 20 px (active upper boundary `y: 0..19`)  
🎯 **Role**: Persistent global navigation bar present at the top of the screen whenever an interface is open (Inventory, Creative, Map, Pause/Settings). Hidden during active 3D gameplay.

```text
(0,0)┌─────────────────────────────────────────────────────────────────────────────────┐
     │ [1:Q] [2:INVENTORY] [3:JOURNAL] [4:WORLD MAP] [5:SYSTEM] [6:E]  [7][8]  [9][10]  │
     └─────────────────────────────────────────────────────────────────────────────────┘(480,20)
```

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **01** | `(11, 6)` | 8 × 8 px | Navigation Button | **Previous Tab Button (`[Q]`)**: Navigates to the previous tab on the left (hotkey shortcut `[Q]` or mouse click). |
| **02** | `(27, 5)` | 48 × 10 px | Navigation Tab | **Inventory Tab**: Opens active inventory dialog (personal inventory, creative catalog, or opened chest). |
| **03** | `(87, 5)` | 48 × 10 px | Navigation Tab | **Journal Tab**: Quest log, bestiary, and expedition notes (reserved, empty placeholder). |
| **04** | `(147, 5)` | 48 × 10 px | Navigation Tab | **World Map Tab**: Opens full-screen interactive World Map. |
| **05** | `(207, 5)` | 48 × 10 px | Navigation Tab | **System Tab (`[SYSTEM]`)**: Opens Pause menu (`MenuState::Pause`) with system options, graphics presets, and resume/quit. |
| **06** | `(263, 6)` | 8 × 8 px | Navigation Button | **Next Tab Button (`[E]`)**: Navigates to the next tab on the right (hotkey shortcut `[E]` or mouse click). |
| **07** | `(399, 6)` | 8 × 8 px | Display Icon | **Currency 1 Icon**: Player money / gold coin currency icon. |
| **08** | `(411, 5)` | 20 × 10 px | Text Readout | **Currency 1 Value**: Dynamic numeric field displaying player balance for Currency 1. |
| **09** | `(437, 6)` | 8 × 8 px | Display Icon | **Currency 2 Icon**: Special currency / gem icon. |
| **10** | `(449, 5)` | 20 × 10 px | Text Readout | **Currency 2 Value**: Dynamic numeric field displaying player balance for Currency 2. |

---

## 3. In-Game Hotbar HUD (`hotbar`)

📂 **Blueprint**: [`outlines/hotbar_outline.png`](interfaces/containers/outlines/hotbar_outline.png)  
🎨 **Clean Render**: [`hotbar.png`](interfaces/containers/hotbar.png)  
📏 **Dimensions**: 480 × 270 px (centered HUD cluster at `x: 127..353, y: 226..255`)  
🎯 **Role**: Active 3D gameplay heads-up display. Displays quick-access inventory slots with standardized 3px gutters (19px pitch, perfectly matching the open inventory grid), offhand, utility accessory, and internal horizontal meter bar.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **01** | `(156, 247)` | 168 × 3 px | Progress Bar | **Bottom Progress Meter**: Horizontal gauge across the bottom of the hotbar slots (Health or Experience / Level meter). |
| **02** | `(134, 231)` | 16 × 16 px | Special Slot | **Offhand Slot (Left Wing)**: Auxiliary offhand slot for shields, torches, or maps. |
| **03** | `(156, 228)` | 16 × 16 px | Hotbar Slot | **Hotbar Slot 1** (Hotkey `[1]`). |
| **04** | `(175, 228)` | 16 × 16 px | Hotbar Slot | **Hotbar Slot 2** (Hotkey `[2]`). |
| **05** | `(194, 228)` | 16 × 16 px | Hotbar Slot | **Hotbar Slot 3** (Hotkey `[3]`). |
| **06** | `(213, 228)` | 16 × 16 px | Hotbar Slot | **Hotbar Slot 4** (Hotkey `[4]`). |
| **07** | `(232, 228)` | 16 × 16 px | Hotbar Slot | **Hotbar Slot 5** (Hotkey `[5]`, screen center `x=240`). |
| **08** | `(251, 228)` | 16 × 16 px | Hotbar Slot | **Hotbar Slot 6** (Hotkey `[6]`). |
| **09** | `(270, 228)` | 16 × 16 px | Hotbar Slot | **Hotbar Slot 7** (Hotkey `[7]`). |
| **10** | `(289, 228)` | 16 × 16 px | Hotbar Slot | **Hotbar Slot 8** (Hotkey `[8]`). |
| **11** | `(308, 228)` | 16 × 16 px | Hotbar Slot | **Hotbar Slot 9** (Hotkey `[9]`). |
| **12** | `(330, 231)` | 16 × 16 px | Special Slot | **Utility / Accessory Slot (Right Wing)**: Auxiliary utility slot for tools, quivers, or trinkets. |

---

## 4. Compact Player Inventory (`480x270_inventory`)

📂 **Blueprint**: [`480x270_inventory_outline.png`](interfaces/containers/480x270/480x270_inventory_outline.png)  
🎨 **Clean Render**: [`480x270_inventory.png`](interfaces/containers/480x270/480x270_inventory.png)  
📏 **Dimensions**: 480 × 270 px  
🎯 **Role**: Default player inventory (equivalent to compact creative / survival inventory). Contains 3D character preview, armor/equipment sockets, 3×3 crafting station, utility management controls, and 36-slot player storage.

### 4.1 Upper Section: Character, Equipment & Crafting (`x: 145..335, y: 35..137`)

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **01** | `(156, 45)` | 54 × 80 px | 3D Viewport | **Live 3D Character Viewport**: Interactive player model with mouse drag rotation. |
| **02** | `(214, 49)` | 16 × 16 px | Equipment Slot | **Helmet / Headgear** slot (`helmet`). Displays translucent overlay when vacant. |
| **03** | `(214, 67)` | 16 × 16 px | Equipment Slot | **Chestplate / Torso Armor** slot (`chestplate`). Displays overlay when vacant. |
| **04** | `(214, 85)` | 16 × 16 px | Equipment Slot | **Leggings / Leg Armor** slot (`leggings`). Displays overlay when vacant. |
| **05** | `(233, 49)` | 16 × 16 px | Equipment Slot | **Boots / Footwear** slot (`boots`). Displays overlay when vacant. |
| **06** | `(233, 67)` | 16 × 16 px | Equipment Slot | **Gloves / Gauntlets** slot (`gloves`). Displays overlay when vacant. |
| **07** | `(233, 85)` | 16 × 16 px | Equipment Slot | **Accessory / Ring / Amulet** slot (`accessory`). Displays overlay when vacant. |
| **08** | `(214, 109)` | 16 × 16 px | Equipment Slot | **Offhand Equipment** slot (`shield`). Directly mirrors Hotbar field 02. |
| **09** | `(233, 109)` | 16 × 16 px | Equipment Slot | **Utility / Tool** slot (`quiver`). Directly mirrors Hotbar field 12. |
| **10** | `(270, 45)` | 16 × 16 px | Crafting Slot | **Crafting Grid (Row 1, Col 1)**. |
| **11** | `(289, 45)` | 16 × 16 px | Crafting Slot | **Crafting Grid (Row 1, Col 2)**. |
| **12** | `(308, 45)` | 16 × 16 px | Crafting Slot | **Crafting Grid (Row 1, Col 3)**. |
| **13** | `(270, 63)` | 16 × 16 px | Crafting Slot | **Crafting Grid (Row 2, Col 1)**. |
| **14** | `(289, 63)` | 16 × 16 px | Crafting Slot | **Crafting Grid (Row 2, Col 2)**. |
| **15** | `(308, 63)` | 16 × 16 px | Crafting Slot | **Crafting Grid (Row 2, Col 3)**. |
| **16** | `(270, 81)` | 16 × 16 px | Crafting Slot | **Crafting Grid (Row 3, Col 1)**. |
| **17** | `(289, 81)` | 16 × 16 px | Crafting Slot | **Crafting Grid (Row 3, Col 2)**. |
| **18** | `(308, 81)` | 16 × 16 px | Crafting Slot | **Crafting Grid (Row 3, Col 3)**. |
| **19** | `(289, 109)` | 16 × 16 px | Crafting Result | **Crafting Output Slot**: Displays crafted item result recipe. |

### 4.2 Lower Section: Inventory Controls & Storage (`x: 127..335, y: 143..255`)

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **20** | `(128, 148)` | 16 × 16 px | Action Button | **Creative Catalog Toggle**: Button on the left of title bar that expands the Creative Catalog. |
| **21** | `(156, 152)` | 64 × 8 px | Title Plate | **Inventory Title Header**: Displays "INVENTORY" or player character name. |
| **22** | `(280, 152)` | 8 × 8 px | Action Button | **Sort Items Button**: Automatically consolidates and sorts storage items. |
| **23** | `(292, 151)` | 8 × 8 px | Action Button | **Quick Stack Button**: Rapidly transfers matching item stacks into containers. |
| **24** | `(304, 151)` | 8 × 8 px | Action Button | **Lock / Deposit Button**: Prevents specific slots from auto-sorting or quick-stacking. |
| **25** | `(316, 152)` | 8 × 8 px | Action Button | **Close Button**: Closes inventory and resumes active 3D gameplay. |
| **26..34** | `(156..308, 168)` | 16 × 16 px | Storage Slot | **Main Inventory Storage — Row 1** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **35..43** | `(156..308, 186)` | 16 × 16 px | Storage Slot | **Main Inventory Storage — Row 2** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **44..52** | `(156..308, 204)` | 16 × 16 px | Storage Slot | **Main Inventory Storage — Row 3** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **53..61** | `(156..308, 228)` | 16 × 16 px | Hotbar Slot | **Active Hotbar Row** (9 slots, mirrors hotbar fields 03..11 during gameplay). |

---

## 5. Expanded Creative Catalog (`480x270_inventory_creative`)

📂 **Blueprint**: [`480x270_inventory_creative_outline.png`](interfaces/containers/480x270/480x270_inventory_creative_outline.png)  
🎨 **Clean Render**: [`480x270_inventory_creative.png`](interfaces/containers/480x270/480x270_inventory_creative.png)  
📏 **Dimensions**: 480 × 270 px (catalog perimeter `x: 127..353, y: 35..255`)  
🎯 **Role**: Full-scale creative block browser. Features a left-hand vertical category sidebar, real-time search filtering, vertical scrollbar paging, 81 catalog slots, and 9 mirrored hotbar slots.

### 5.1 Header, Search & Navigation Sidebar

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **01** | `(128, 40)` | 16 × 16 px | Action Button | **Compact Inventory Toggle**: Button on the left of title bar that collapses back to compact inventory. |
| **02** | `(156, 44)` | 64 × 8 px | Title Plate | **Catalog Title Header**: Displays "CREATIVE CATALOG". |
| **03** | `(260, 44)` | 64 × 8 px | Text Input | **Search Input Box**: Real-time substring filter for blocks and items. |
| **04** | `(128, 65)` | 16 × 16 px | Category Tab | **Category Tab 1 — Building / Rocks**: Stone, sandstones, cobblestones, and bricks. |
| **05** | `(128, 84)` | 16 × 16 px | Category Tab | **Category Tab 2 — Trees & Wood**: Logs, bark, planks, and foliage. |
| **06** | `(128, 103)` | 16 × 16 px | Category Tab | **Category Tab 3 — Soils & Turf**: Grasses, loams, peats, gravels, and sands. |
| **07** | `(128, 122)` | 16 × 16 px | Category Tab | **Category Tab 4 — Ores & Minerals**: Raw ores, gemstone blocks, and deep mineral deposits. |
| **08** | `(128, 141)` | 16 × 16 px | Category Tab | **Category Tab 5 — Flora & Decoration**: Flowers, bushes, reeds, crops, and pebbles. |
| **09** | `(128, 160)` | 16 × 16 px | Category Tab | **Category Tab 6 — Tools & Weapons**: Pickaxes, shovels, axes, blades, and bows. |
| **10** | `(128, 179)` | 16 × 16 px | Category Tab | **Category Tab 7 — Armor & Apparel**: Helmets, chestplates, leggings, boots, and shields. |
| **11** | `(128, 198)` | 16 × 16 px | Category Tab | **Category Tab 8 — Consumables & Alchemy**: Potions, food, seeds, and ingredients. |
| **12** | `(128, 217)` | 16 × 16 px | Category Tab | **Category Tab 9 — Mechanisms & Redstone**: Switches, wiring, lights, and logic gates. |
| **13** | `(339, 67)` | 8 × 164 px | Scrollbar Track | **Vertical Scroll Track**: Scrollbar gutter and draggable thumb for browsing the catalog. |

### 5.2 Catalog Storage Grid & Hotbar Row

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **14..22** | `(156..308, 60)` | 16 × 16 px | Catalog Slot | **Catalog Grid — Row 1** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **23..31** | `(156..308, 78)` | 16 × 16 px | Catalog Slot | **Catalog Grid — Row 2** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **32..40** | `(156..308, 96)` | 16 × 16 px | Catalog Slot | **Catalog Grid — Row 3** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **41..49** | `(156..308, 114)` | 16 × 16 px | Catalog Slot | **Catalog Grid — Row 4** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **50..58** | `(156..308, 132)` | 16 × 16 px | Catalog Slot | **Catalog Grid — Row 5** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **59..67** | `(156..308, 150)` | 16 × 16 px | Catalog Slot | **Catalog Grid — Row 6** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **68..76** | `(156..308, 168)` | 16 × 16 px | Catalog Slot | **Catalog Grid — Row 7** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **77..85** | `(156..308, 186)` | 16 × 16 px | Catalog Slot | **Catalog Grid — Row 8** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **86..94** | `(156..308, 204)` | 16 × 16 px | Catalog Slot | **Catalog Grid — Row 9** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **95..103** | `(156..308, 228)` | 16 × 16 px | Hotbar Slot | **Creative Hotbar Row** (9 slots, mirrors hotbar fields 03..11 during gameplay). |

---

## 6. Chest Container Interface (`480x270_chest`)

📂 **Blueprint**: [`480x270_chest_outline.png`](interfaces/containers/480x270/480x270_chest_outline.png)  
🎨 **Clean Render**: [`480x270_chest.png`](interfaces/containers/480x270/480x270_chest.png)  
📏 **Dimensions**: 480 × 270 px (container cluster at `x: 145..335, y: 35..255`)  
🎯 **Role**: Standard single chest storage dialog. Displays 27 chest storage slots in the upper compartment and 36 player inventory slots in the lower compartment.

### 6.1 Upper Section: Chest Storage Compartment (`x: 145..335, y: 35..122`)

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **01** | `(156, 44)` | 64 × 8 px | Title Plate | **Chest Title Header**: Displays "WOODEN CHEST" or custom container label. |
| **02** | `(280, 44)` | 8 × 8 px | Action Button | **Sort Chest**: Consolidates and sorts items inside the chest. |
| **03** | `(292, 43)` | 8 × 8 px | Action Button | **Take All**: Rapidly transfers all chest items into player inventory. |
| **04** | `(304, 43)` | 8 × 8 px | Action Button | **Lock Chest**: Prevents other players from opening container in multiplayer. |
| **05** | `(316, 44)` | 8 × 8 px | Action Button | **Close Container**: Exits chest interface and returns to 3D gameplay. |
| **06..14** | `(156..308, 60)` | 16 × 16 px | Chest Slot | **Chest Grid — Row 1** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **15..23** | `(156..308, 78)` | 16 × 16 px | Chest Slot | **Chest Grid — Row 2** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |
| **24..32** | `(156..308, 96)` | 16 × 16 px | Chest Slot | **Chest Grid — Row 3** (9 slots, x=156, 175, 194, 213, 232, 251, 270, 289, 308). |

### 6.2 Lower Section: Player Inventory Storage (`x: 145..335, y: 143..255`)

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **33** | `(156, 152)` | 64 × 8 px | Title Plate | **Inventory Title Header**: Displays "INVENTORY". |
| **34** | `(280, 152)` | 8 × 8 px | Action Button | **Sort Inventory**: Consolidates player items. |
| **35** | `(292, 151)` | 8 × 8 px | Action Button | **Quick Stack**: Transfers inventory stacks into chest. |
| **36** | `(304, 151)` | 8 × 8 px | Action Button | **Auto-Store**: Depositor toggle button. |
| **37** | `(316, 152)` | 8 × 8 px | Action Button | **Close Inventory**: Exits container screen. |
| **38..46** | `(156..308, 168)` | 16 × 16 px | Storage Slot | **Player Storage — Row 1** (9 slots, x=156..308). |
| **47..55** | `(156..308, 186)` | 16 × 16 px | Storage Slot | **Player Storage — Row 2** (9 slots, x=156..308). |
| **56..64** | `(156..308, 204)` | 16 × 16 px | Storage Slot | **Player Storage — Row 3** (9 slots, x=156..308). |
| **65..73** | `(156..308, 228)` | 16 × 16 px | Hotbar Slot | **Player Hotbar Row** (9 slots, mirrors hotbar fields 03..11). |

---

## 7. Interactive World Map Interface (`480x270_map`)

📂 **Blueprint**: [`480x270_map_outline.png`](interfaces/containers/480x270/480x270_map_outline.png)  
🎨 **Clean Render**: [`480x270_map.png`](interfaces/containers/480x270/480x270_map.png)  
📏 **Dimensions**: 480 × 270 px  
🎯 **Role**: Full-screen interactive cartographic screen accessible via Menu Bar tab 04 or hotkey `M`.

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **01** | `(15, 35)` | 450 × 220 px | Map Viewport | **Fullscreen Cartographic Viewport**: Interactive world map with real-time player position indicator, pan, zoom, waypoint markers, and climate biome blending. |

---

## 8. System Pause Menu (`480x270_pause`)

📂 **Blueprint**: [`480x270_pause_outline.png`](interfaces/containers/480x270/480x270_pause_outline.png)  
🎨 **Clean Render**: [`480x270_pause.png`](interfaces/containers/480x270/480x270_pause.png)  
📏 **Dimensions**: 480 × 270 px (sidebar menu cluster at `x: 7..147, y: 28..144`)  
🎯 **Role**: Left-hand sidebar pause menu accessed via hotkey `Escape` or system controls.

> [!NOTE]
> The anchor corner at `(7, 28, 7, 7)` is a stylized decorative dark iron corner border and framing bracket, not an interactive button. The interactive elements consist of the 6 stacked buttons (numbered **01** to **06**).

| Field | Coordinates `(x, y)` | Dimensions | Field Type | Description & Engine Function |
| :---: | :---: | :---: | :---: | :--- |
| **01** | `(19, 40)` | 128 × 14 px | Action Button | **Retomar (Resume Game)**: Closes pause screen and returns immediately to active 3D gameplay. |
| **02** | `(19, 58)` | 128 × 14 px | Action Button | **Configurações (Settings & Options)**: Opens audio volume, render scale, graphics presets, and FOV menu. |
| **03** | `(19, 76)` | 128 × 14 px | Action Button | **Controles (Controls & Keybindings - WIP)**: Opens keyboard/mouse remapping and sensitivity screen. |
| **04** | `(19, 94)` | 128 × 14 px | Action Button | **Conquistas / Estatísticas (Achievements & Stats - WIP)**: Displays play time, chunks explored, and unlocked achievements. |
| **05** | `(19, 112)` | 128 × 14 px | Action Button | **Reiniciar Jogo (Restart Game) / Salvar e Sair para o Menu**: Currently functions as restart game (will save and exit to title menu when title menu is added). |
| **06** | `(19, 130)` | 128 × 14 px | Action Button | **Sair para a Área de Trabalho (Quit to Desktop)**: Saves chunk state and safely terminates the game process. |

---

## 9. Shared Interactive UI Components

The following standardized interactive button textures in `docs/worldbuilding/interfaces/containers/` are utilized across all 480×270 dialogs:

| Asset File | Dimensions | Idle State | Hover State | Engine Role & Usage |
| :--- | :---: | :---: | :---: | :--- |
| **Utility Button** | 8 × 8 px | [`button_util.png`](interfaces/containers/button_util.png) | [`button_util_hover.png`](interfaces/containers/button_util_hover.png) | Miniature square buttons for header actions: Sort, Quick Stack, Lock, Close. |
| **Small Button** | 16 × 16 px | [`button_small.png`](interfaces/containers/button_small.png) | [`button_small_hover.png`](interfaces/containers/button_small_hover.png) | Category tabs, toggle buttons, and icon triggers. |
| **Medium Button** | 64 × 16 px | [`button_medium.png`](interfaces/containers/button_medium.png) | [`button_medium_hover.png`](interfaces/containers/button_medium_hover.png) | Standard menu dialog actions (e.g. Back, Confirm, Cancel). |
| **Large Button** | 96 × 16 px | [`button_large.png`](interfaces/containers/button_large.png) | [`button_large_hover.png`](interfaces/containers/button_large_hover.png) | Primary Pause Menu and Title Menu action buttons. |

---

## 10. Equipment Slot Overlays (`interfaces/containers/slots/`)

Translucent 14 × 14 px Dark Iron icons rendered inside empty equipment slots to indicate accepted item categories:

| Target Equipment Slot | Overlay Asset | Slot ID Mapping |
| :--- | :---: | :--- |
| **Helmet / Headgear** | [`helmet.png`](interfaces/containers/slots/helmet.png) | Inventory Slot **02** |
| **Chestplate / Torso** | [`chestplate.png`](interfaces/containers/slots/chestplate.png) | Inventory Slot **03** |
| **Leggings / Legs** | [`leggings.png`](interfaces/containers/slots/leggings.png) | Inventory Slot **04** |
| **Boots / Footwear** | [`boots.png`](interfaces/containers/slots/boots.png) | Inventory Slot **05** |
| **Gloves / Gauntlets** | [`gloves.png`](interfaces/containers/slots/gloves.png) | Inventory Slot **06** |
| **Amulet / Accessory** | [`necklace.png`](interfaces/containers/slots/necklace.png) | Inventory Slot **07** |
| **Offhand / Shield** | [`shield.png`](interfaces/containers/slots/shield.png) | Inventory Slot **08** / Hotbar Slot **02** |
| **Utility / Quiver** | [`quiver.png`](interfaces/containers/slots/quiver.png) | Inventory Slot **09** / Hotbar Slot **12** |
| **Backpack / Caddy** | [`backpack.png`](interfaces/containers/slots/backpack.png) | Expansion slot asset |
| **Tool Belt / Pouch** | [`belt.png`](interfaces/containers/slots/belt.png) | Quick-belt accessory asset |
| **Attribute Ring** | [`ring.png`](interfaces/containers/slots/ring.png) | Jewelry accessory asset |
| **Magic Rune** | [`rune.png`](interfaces/containers/slots/rune.png) | Socket gem asset |
| **Alchemy Potion** | [`potion.png`](interfaces/containers/slots/potion.png) | Potion flask asset |
| **Cape / Mantle / Glider** | [`cape.png`](interfaces/containers/slots/cape.png) | Cosmetic back accessory asset |

---

## 11. Interface Icon Repository (`docs/worldbuilding/interfaces/icons/`)

Pre-staged 16 × 16 px pixel-art icons for HUD readouts, celestial cycles, weather forecasts, biomes, and character creation:

* **Diurnal Phases (`time_*.png`)**: [`time_morning.png`](interfaces/icons/time_morning.png), [`time_noon.png`](interfaces/icons/time_noon.png), [`time_evening.png`](interfaces/icons/time_evening.png), [`time_night.png`](interfaces/icons/time_night.png).
* **Atmospheric Weather (`weather_*.png`)**: [`weather_clear.png`](interfaces/icons/weather_clear.png), [`weather_raining.png`](interfaces/icons/weather_raining.png), [`weather_thunder.png`](interfaces/icons/weather_thunder.png), [`weather_snow.png`](interfaces/icons/weather_snow.png).
* **Biome Indicators (`biome_*.png`)**: [`biome_plains.png`](interfaces/icons/biome_plains.png), [`biome_forest.png`](interfaces/icons/biome_forest.png), [`biome_dark_forest.png`](interfaces/icons/biome_dark_forest.png), [`biome_birch_forest.png`](interfaces/icons/biome_birch_forest.png), [`biome_taiga.png`](interfaces/icons/biome_taiga.png), [`biome_desert.png`](interfaces/icons/biome_desert.png), [`biome_savanna.png`](interfaces/icons/biome_savanna.png), [`biome_beach.png`](interfaces/icons/biome_beach.png), [`biome_river.png`](interfaces/icons/biome_river.png), [`biome_frozen_peaks.png`](interfaces/icons/biome_frozen_peaks.png).
* **Expedition Journal**: [`journal_board.png`](interfaces/icons/journal_board.png).
* **Character Races (`race_*.png`)**: [`race_human.png`](interfaces/icons/race_human.png), [`race_elf.png`](interfaces/icons/race_elf.png), [`race_dwarf.png`](interfaces/icons/race_dwarf.png), [`race_orc.png`](interfaces/icons/race_orc.png), [`race_drow.png`](interfaces/icons/race_drow.png), [`race_halfling.png`](interfaces/icons/race_halfling.png), [`race_tiefling.png`](interfaces/icons/race_tiefling.png), [`race_dragonborn.png`](interfaces/icons/race_dragonborn.png).
