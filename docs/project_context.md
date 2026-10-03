# Voxel Game Engine - Current Project Context

This is an attempt to develop an experimental voxel engine and game, built from scratch using Rust and Bevy.


---

## Core Concepts

- The world is stored using 1.0 m voxels (similar to Minecraft blocks).
- Chunks have 16 × 16 × 16 voxels (16 m × 16 m × 16 m physical sections).
- Native 1.0 m block with multiple shapes: (`Full`, `Slab`, `Stair`, `'Column`, `Torch`, `Basket`) with 3D orientations, custom meshes (3D hollow basket, torches), and passable non-solid collision for torches.
- Comprehensive 244-block palette across 10 categories with multi-face textures, dual-layer moss overlays, organic tinting, and tuned dynamic fluid simulation (wave movement and finite spreading).
- Interactive tactile feedback: subtle 8-particle debris bursts with terrain collision bouncing on block break, and 0.18s elastic scale bounce on block placement.
- Auto-step is calibrated to 0.50 m (50 cm) for smooth future slab stepping, requiring jumping over full 1 m blocks.
- Square gameplay Minimap HUD (North-up, compact XYZ coordinates, live player heading chevron) and full-screen interactive World Map (<kbd>M</kbd>).
- Ambient Environment procedural color noise and climate-driven biome palettes (grass, foliage, water) with smooth 5-point cross-kernel boundary blending.
- Dynamic FPS & VSync: configurable presentation modes (AutoNoVsync default, toggleable in Settings) and intelligent frame throttling (15 FPS unfocused, 30 FPS idle) with unconstrained active gameplay (250+ FPS).
- Custom typography & drop shadows: universal font asset management (`CutePixel.ttf`) applied across all in-game HUDs, menus, and interfaces with drop shadow contrast styling.
- Centered unified textured container interface: Creative palette & Personal storage with top mode toggle buttons, real-time search with hold-to-repeat backspace, Caps Lock support, and text drag/double-click selection, persistent tab memory, Shift-drag multi-slot transfers, and cinematic Depth-of-Field blur.
- Dynamic adaptive UI scaling: responsive resolution-relative `UiScale` system (`AdaptiveUiPlugin`), automatically scaling HUD, minimap, hotbar, and inventory elements proportionally across all screen sizes and resolutions.
- Restructured two-column menus with pixel-art buttons: Pause and Settings menus redesigned into clean two-column floating layouts using authentic textured container buttons (`button.png`, `small_button.png`).
- Item identification tooltips: Item name display centered above the hotbar on slot selection with smooth fade-out, and responsive floating hover tooltips across all Creative and Personal inventory slots.

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
- Vertical world limits are currently:
    - Minimum chunk Y: -16 (-256 blocks)
    - Maximum chunk Y: +16 (+256 blocks)
    - Total playable vertical height: 512 blocks.
- Cloud plane altitude: 220.0 m.
- Current default render distance: 12 chunks with LOD chunks render distance set to 12 as well.
- Chunks are generated and unloaded dynamically as the player moves through the world.

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

## Next Steps / Upcoming Phases

- [x] Phase 1: Core Rendering & Texture-Array Architecture (Completed)
- [x] Phase 2: Atmosphere, Celestial Bodies & Dynamic Sky (Completed)
- [x] Phase 3: Gameplay, Inventory & Sub-Voxel Shaping Tools (Completed)
- [x] Phase 4: Dynamic Fluid Simulation & Boundary Mechanics (Completed)
- [x] Phase 5: General Polish, Revisions & In-Game Interfaces (Completed)
- [x] Phase 6: Advanced World Generation, Biomes & Caves (Completed)
- [x] Phase 7: Project Organization, Architecture Audit & Refactor (Completed)
- [x] Phase 8: Foundational Storage, Meshing & Bitmask Acceleration (Completed)
- [x] Phase 9: Engine-Wide Architecture Modernization, 1m Shapes & Codebase Cleanup (Completed)
- [x] Phase 10: Gameplay Polish, Interaction Feedback & Quality-of-Life (Completed)
- [x] Phase 11: World Generation & Worldbuilding Expansion (High Fantasy & Dark Fantasy Realism) (Completed)
- [ ] Phase 12: Flora, Procedural Trees & Surface Vegetation (Active)
- [x] Phase 13: High-Performance Scaling, Level-of-Detail (LOD) & Engine Optimization (Completed)
- [x] Phase 14: Comprehensive Technical Review, Performance Engineering & Architecture Audit (Completed)

---

## Important Rules for Assistance and Collaboration

- Inspect current files first before replacing systems.
- Current controls and project Structure can be found inside the "README.md" file.
- Keep the engine fully functional during migration.
- Validate changes with `cargo fmt`, `cargo check`, and `cargo clippy`.
- Do not add unnecessary comments to the code.
- Keep the code 100% in English.
