# Voxel Game Engine - Current Project Context

This is an attempt to develop an experimental voxel engine and game, built from scratch using Rust and Bevy.

---

## Core Concepts

- **1.0 m Voxel World**: The world is stored using 1.0 m voxels (identical to Minecraft blocks). Sub-voxels are completely eliminated; full 1.0 m³ blocks form the atomic spatial unit.
- **Chunk Geometry**: Chunks contain 16 × 16 × 16 voxels (16 m × 16 m × 16 m physical sections), with 4,096 voxels per chunk.
- **Block Shapes & Directional Placement**:
  - Native 1.0 m block shapes: `Full`, `Slab`, `Stair`, `Column`, `Torch`, and `Basket`, with 3D orientations, custom meshes, and passable non-solid collision for torches.
  - Minecraft-style 3-axis directional placement for full-sized pillar blocks (`AxisAlignedPillar` on logs, bone blocks, and structural columns) aligning their 6-face texture mapping along the X, Y, or Z placement axis based on hit normal.
- **Comprehensive 720-Block Worldbuilding Catalog**:
  - 720 block definitions across 11 thematic categories (`rocks`, `trees`, `soils`, `ores`, `caverns`, `oceans`, `organics`, `decorations`, `mushrooms`, `vegetation`, `fluids`/`systems`).
  - Documented in `docs/worldbuilding/blocks_design.md` and `docs/worldbuilding/blocks/*.md`.
- **Deterministic Natural Texture Rotation & Flipping**:
  - Implemented in WGSL fragment shader (`assets/shaders/voxel_opaque.wgsl`) using an integer coordinate hash of `floor(world_position)`.
  - Natural terrain blocks (stone, soils, sands, gravels) rotate and flip pseudo-randomly to eliminate grid tiling patterns over large cliff faces and cave walls without breaking CPU greedy meshing efficiency.
- **Outcrop Dual-Layer Rendering**:
  - Grass and snow outcrop blocks across soils and rocks render top and side layers while ensuring the bottom face (`Face::Bottom` / `Face::Down`) renders the base mineral material (e.g., Dolomite Grass bottom renders Dolomite; Peat Grass bottom renders Peat Dirt).
- **Tactile Gameplay Feedback**:
  - Subtle 8-particle debris bursts with terrain collision bouncing on block break.
  - 0.18s elastic scale bounce on block placement.
- **Auto-Step**:
  - Calibrated to 0.50 m (50 cm) for smooth future slab stepping, requiring jumping over full 1 m blocks.
- **Navigation & Mapping**:
  - Square gameplay Minimap HUD (North-up, compact XYZ coordinates, live player heading chevron).
  - Full-screen interactive World Map (<kbd>M</kbd>) with pan, zoom, and player tracking.
- **Atmosphere & Climate**:
  - 24-minute day/night astronomical clock, 28-day lunar calendar (8 moon phases), and 4 seasons.
  - Procedural color noise and climate-driven biome palettes (grass, foliage, water) with smooth 5-point cross-kernel boundary blending.
- **Dynamic FPS & VSync**:
  - Configurable presentation modes (AutoNoVsync default, toggleable in Settings) and intelligent frame throttling (15 FPS unfocused, 30 FPS idle) with unconstrained active gameplay (250+ FPS).
- **Custom Typography**:
  - Universal font asset management (`CutePixel.ttf`) applied across all in-game HUDs, menus, and interfaces with drop shadow contrast styling.
- **Standardized 9-Horizontal-Slot UI Architecture**:
  - **In-Game Hotbar HUD (320 × 44 px)**: 9 primary item slots (hotkeys `1`–`9` and mouse scroll wheel), 2 special readout slots (offhand and utility with Dark Iron backdrops), 4 non-clickable wing badges (`[M]`, `[J]`, `[C]`, `[T]`), static visual meters (XP, HP, Mana), and centered 20×20 px `hotbar_selection.png`. Zero mouse hover/clicks during active 3D gameplay.
  - **Active Player Inventory (`inventory_creative_closed.png`, 416 × 256 px)**: 36 Card B storage slots (4 rows of 9), mirrored hotbar and offhand/utility slots with `inventory_slot_hover.png` feedback, action utility buttons (Sort, Trash, Options, Lock), non-clickable Card A armor placeholders (with Dark Iron overlays) and Card C crafting placeholders, and **B1** toggle tab at `(118, 116)`.
  - **Creative Catalog (`inventory_creative_open.png`, 416 × 256 px)**: 11 category tabs rendering scaled 3D isometric block icons (`BlockIcons`), docking selection frame (`inventory_creative_tab_selection.png`), live search filter with cursor, vertical scroll track with draggable thumb (`inventory_creative_scroller.png`), 90-slot grid (10 rows of 9), and mirrored bottom hotbar.
  - **Unified Button Suite**: Pause and Settings menus powered by unified dark obsidian / brushed steel buttons (`button_large.png` 96×16 px, `button_medium.png` 64×16 px, `button_small.png` 16×16 px) with frosted slate-blue metallic hover highlights (`#3c4552`).
- **Item Tooltips**:
  - Centered item name readout above hotbar upon slot selection with smooth fade-out.
  - Responsive floating hover tooltips across all inventory and catalog slots.

---

## Current World Scale

- The world is stored using 1.0 m voxels (identical to Minecraft blocks).
- Full 1 m³ blocks form the base unit of the world; 0.5 m sub-voxels are completely removed.
- Each chunk contains: 16 × 16 × 16 voxels (logical blocks).
- Total voxel capacity per chunk: 4,096 voxels.
- Physical chunk size: 16 m × 16 m × 16 m.
- The world streams procedurally and is effectively unlimited horizontally:
  - X: procedural streaming
  - Z: procedural streaming
- Vertical world limits:
  - Minimum chunk Y: -16 (-256 blocks)
  - Maximum chunk Y: +16 (+256 blocks)
  - Total playable vertical height: 512 blocks.
- Cloud plane altitude: 220.0 m.
- Current default render distance: 12 chunks (with LOD chunk radius set to 12 as well).
- Chunks are generated, meshed, and unloaded dynamically as the player moves through the world.

---

## Player System

The engine has a dedicated player system separate from the voxel engine.

Current dimensions & stances:
- Width: 0.60 m
- Standing: height 1.80 m, eye height 1.62 m
- Crouching: height 1.30 m, eye height 1.20 m, speed reduced to 55%, edge drop/ledge prevention
- Crawling: height 0.45 m, eye height 0.40 m, speed reduced to 35%, 1-voxel high openings (0.5 m) with headroom safety checks preventing uncrawling under ceilings

Camera System:
- Default FOV: 90 degrees (configurable 60°..=110° via in-game Settings)
- Dynamic FOV Kick: smooth +8° expansion when sprinting or flying fast
- View Bobbing: subtle sinusoidal head bobbing during grounded movement (toggleable in Settings)
- Camera Zoom: dynamic magnification (<kbd>Z</kbd> + Mouse Wheel up to 20x zoom) with mouse sensitivity dampening
- First-Person Camera: true first-person body view (head hidden to prevent interior clipping, looking down reveals animated chest, arms, and legs)
- Third-Person Camera (<kbd>F5</kbd>): orbiting camera with dynamic raycast block collision prevention to avoid clipping underground/walls, and independent head pitch/yaw tracking

Player Model & Procedural Animations:
- Minecraft 64×64 skin compatible humanoid mesh hierarchy (Head, Torso, Left/Right Arm, Left/Right Leg) mapped to `assets/textures/models/player_skin.png`
- Full support for base skin and 3D outer layers (hat/hair, jacket, sleeves, pants) with alpha masking
- Procedural locomotion: dynamic walk/sprint leg & arm pendulum swings, idle breathing sway, crouch torso tilt, prone crawling strokes, streamlined flutter-kick swimming, airborne jump poses, and 4-state flight animations (ascent, descent, fast flight, hover)

Game Modes Currently Implemented:
- Creative (currently the main gameplay and development mode)
- Spectator (noclip flight mode)

The player uses a custom AABB collision system that directly queries voxel data. Individual physics colliders are not created for terrain voxels.

---

## Roadmap Status & Development Transition

The engine has concluded foundational development across Phases 1 through 14 (rendering, atmosphere, fluids, greedy mesher optimizations, SIMD noise, 512-block world generation, and interface modernization).

The legacy linear roadmap has been set aside. A revised roadmap is currently being structured to outline upcoming milestones, focusing on procedural vegetation, world generation refinements, and survival gameplay systems.

---

## Important Rules for Assistance and Collaboration

- Inspect current files first before replacing systems.
- Current controls and project structure can be found inside `README.md`.
- Keep the engine fully functional during migration.
- Validate changes with `cargo fmt`, `cargo check`, and `cargo clippy`.
- Do not add unnecessary comments to the code.
- Keep the code 100% in English.
