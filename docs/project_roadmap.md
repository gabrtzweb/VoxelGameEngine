# Voxel Game Engine — Development Roadmap

---

## Roadmap Overview & Milestone Structure

The Voxel Game Engine development lifecycle is structured into major evolutionary versions:

- **Version 0.1 — Foundation Era (Phases 1–20) [Completed]**: Established the core engine architecture, multi-chunk streaming, greedy meshing, procedural terrain, physics, fluid dynamics, celestial environments, survival interfaces, and low-level SIMD/memory optimization.
- **Version 0.2 — Modernization, Worldbuilding & Deep Systems (Phases 1–20) [Completed]**: Delivered deep worldbuilding content, standardized 480×270 Dark Iron UI, Bevy 0.20 & WESL shader migration, 6D multi-noise 44-biome worldgen, subterranean strata & geodes, spatial audio, 8-race custom voxel characters, agriculture, weather, smooth fluids, localization, rebindable controls, GPU procedural grass, and multi-chunk world feature generation.
- **Version 0.3 — Next Development Cycle (Phases 1–20) [Current]**: Active development milestone building on top of the completed Version 0.2 engine and worldbuilding foundation.

---

## Version 0.1: Foundation Era (Completed)

This foundational milestone encapsulates the initial 20 engineering phases that built the engine from a clean slate into a robust, high-performance voxel engine:

| Phase | Milestone Title | Summary of Completed Deliverables |
| :---: | :--- | :--- |
| **01** | **Project Initialization & Core Bevy Engine Architecture** | Initialized Bevy engine application pipeline, coordinate systems, base window management, and tick loop infrastructure. |
| **02** | **Voxel Chunk Data Structures & Raycast Interaction** | Implemented 3D chunk array storage, exposed face detection, free-fly developer camera, and runtime raycast block targeting. |
| **03** | **Multi-Chunk World Streaming & Terrain Noise Prototype** | Added multi-chunk boundary handling, fast block placement/breaking highlights, and initial continuous Perlin/simplex heightmap generation. |
| **04** | **Asynchronous Chunk Generation & Persistence** | Introduced background worker thread chunk generation, vertical column streaming, and disk-ready world modification persistence. |
| **05** | **Player Controller Stances, Physics & Creative Flight** | Built dual-mode player controllers (Creative flight and spectator modes), AABB collision detection, and calibrated flight auto-stepping. |
| **06** | **Greedy Chunk Meshing & Voxel Type Architecture** | Implemented greedy quad-merging mesher reducing vertex counts by ~80%, alongside extensible voxel type enum registries. |
| **07** | **Astronomical Day/Night Cycle & Dynamic Atmosphere** | Added 24-minute celestial clock, rotating additive sun/moon billboards with 8 moon phases, dynamic starfields, and horizon fog. |
| **08** | **Dynamic Cellular Automata Fluid Simulation** | Built real-time water propagation with stepped sub-voxel fluid heights, boundary physics, submerged fog, and surface mesh offsets. |
| **09** | **Runtime Developer Diagnostics & Egui World Inspector** | Created the F1 runtime inspector for live noise parameter tuning, entity diagnostics, performance metrics, and instant world reload. |
| **10** | **Hardware 2D Texture Array & WGSL Voxel Shader** | Engineered custom WGSL material extension with nearest-neighbor 2D texture array sampling and spatial variant randomization. |
| **11** | **Sub-Voxel Block Shaping Tools & Centered Placement** | Developed the real-time block shaping tool (`R`/`T` keys) supporting slabs, stairs, corner stairs, and centered vertical column voxels. |
| **12** | **In-Game Calendar, Seasons & Debug HUD** | Implemented a 4-season astronomical calendar, day tracking, and a comprehensive F3 debug overlay with performance metrics. |
| **13** | **Player Model Rigging, Skins & Cursor State Management** | Integrated first/third-person player avatar rendering with texture skin mapping and context-sensitive gameplay cursor states. |
| **14** | **Multi-Biome World Generation & Procedural Caves** | Implemented 11 distinct biomes driven by 2D continuous temperature/moisture noise, 3D cave/ravine density, and procedural trees. |
| **15** | **In-Game Cartography (Minimap HUD & Interactive World Map)** | Built a real-time top-down minimap HUD with player orientation and a full-screen interactive world map supporting panning and zooming. |
| **16** | **Survival Inventory Architecture & 2D Item Tooltips** | Created dual-card container inventories, hotbar item synchronization, 2D pixel-art icon rendering, and hovering tooltips. |
| **17** | **Engine-Wide Subsystem Modularization & Architecture Cleanup** | Refactored monolithic files into clean domain subsystems (`core`, `environment`, `gameplay`, `generation`, `map`, `menu`, `player`, `simulation`, `voxel`). |
| **18** | **Multi-Tier Chunk Mesh LOD & Distance Scaling** | Implemented distance-based chunk mesh Level-of-Detail (LOD), camera-frustum loading prioritization, and dynamic framerate throttling. |
| **19** | **Screen-Space Post-Processing, Water Shaders & Responsive UI** | Added custom post-process pipeline with depth-of-field blur, reflective/refractive water shaders, and DPI-responsive GUI scaling. |
| **20** | **Performance Engineering & Explicit SIMD Vector Acceleration** | Integrated 4-wide vector SIMD noise (3.2x speedup), `Arc<Chunk>` zero-copy meshing, bitwise hardware math, L1 voxel tables, and LRU chunk caching. |

---

## Version 0.2: Modernization, Worldbuilding & Deep Systems (Completed)

This second major milestone expanded the engine across 20 phases into a full-featured voxel world with modern shaders, rich geology and flora, spatial audio, weather, agriculture, and high-throughput streaming:

| Phase | Milestone Title | Summary of Completed Deliverables |
| :---: | :--- | :--- |
| **01** | **Full Block Catalog & Geological Systems Foundation** | Unified 600+ active voxel catalog, 3-axis directional placement, selective biome foliage tinting, shader de-tiling, extended shapes, and shape locking. |
| **02** | **Standardized 9-Slot Interface Architecture & Menu Modernization** | Standardized 9-slot container/HUD geometry on a 480×270 base canvas with integer scaling, Dark Iron button suite, pause menu bar, and menu DoF/dimming. |
| **03** | **Core Engine Upgrade (Bevy 0.19 to Bevy 0.20 Migration)** | Migrated to Bevy 0.20 and `bevy_egui = "0.43"`, adopted retained UI widgets, native WESL shader framework, and `ScreenSpaceTransmission`. |
| **04** | **Architecture, Graphics Modernization & Soundness** | Added multi-tier hardware profiles, lazy chunk variant memory optimization (~92% reduction), dynamic 3D render scaling, and 16:9 window snapping. |
| **05** | **High-Impact Performance & Graphics Pipeline** | Implemented dynamic FPS limiter, hardware-scaled GPU mesh upload budgets, AMD FSR 1.0 upscaling with RCAS, and subterranean occlusion culling. |
| **06** | **Wild Flora, Herbaceous Vegetation & Botanical Systems** | Registered vegetation and flower blocks with cross-billboards, double-tall plants, petal carpets, leaf litter, marine flora, and connected topology. |
| **07** | **Subterranean Strata, Caverns & Geological Generation** | Built multi-tier geological strata down to Mantle bedrock (`Y = -254`), 54 host-rock ore pairings, crystal geodes, speleothems, and subterranean fluid basins. |
| **08** | **Next-Gen 3D World Generation and 6D Multi-Noise Climate** | Engineered 6D SIMD climate noise with Voronoi matching across 44 biomes, subterranean microbiomes, elevated stepped rivers, and slope-aware cliff strata. |
| **09** | **Main Menu, Clean World Session Lifetime & Deferred World Loading** | Created two-phase title screen, deferred `WorldSession` streaming, progress loading screen with rotating tips, shuffled menu soundtrack, and clean world unload. |
| **10** | **Sound Engine & Spatial Audio System** | Built multi-channel audio mixer, material-based 3D block interaction & footstep acoustics, underwater/cave/biome ambience, weather audio, and 39-track soundtrack. |
| **11** | **Custom Voxel Player Character, 8-Race Silhouettes & Hand Item Rendering** | Replaced legacy skins with modular PBR voxel rigs across 8 fantasy races with bespoke silhouettes, race selector screen, and 1st/3rd-person 3D held items. |
| **12** | **Agricultural Simulation, Cultivated Crops & Farmland Systems** | Implemented 16 domestic 4-stage crop families, 18 wild crops, tilled farmland substrates, growth ticks, bone meal fertilization, and cucurbit vine fruiting. |
| **13** | **Physics Engine, Fluid Dynamics & Item Drops** | Added smooth-flow fluid tessellation across all 9 liquids, downhill river hydrology & current forces, and floating 3D item drops with buoyancy and pickup suction. |
| **14** | **Dynamic Weather System & Meteorological Phenomena** | Built climate-routed weather state machine, 3D harmonic wind field, batched precipitation/surface FX particles, lightning with speed-of-sound thunder, and clean resets. |
| **15** | **Visual Aesthetics, Shaders & Atmospheric FX Overhaul** | Integrated per-vertex Voxel Ambient Occlusion, water refraction/caustics/foam, HDR bloom, volumetric crepuscular god rays, and subterranean lighting stabilization. |
| **16** | **Performance Engineering, Chunk Streaming & LOD Enhancements** | Modularized chunk streaming and shape builders, added 36-bit cave visibility culling, extended LOD & cloud distances, and optimized meshing/GPU upload pipelines. |
| **17** | **Localization & Multi-Language Support** | Added flat-key `assets/lang/` localization (`en_us.json`, `pt_br.json`) covering UI, blocks, and items with live reactive in-game language switching. |
| **18** | **Controls & Keybinding System** | Built categorized rebindable keyboard/mouse controls, real-time conflict detection, mouse sensitivity/invert-Y options, and `local_config.json` persistence. |
| **19** | **Dynamic Foliage & Procedural Grass System** | Engineered GPU-accelerated segmented 3D grass blades with Cubic Bezier wind deformation, Voronoi clumping, entity footprint trails, and multi-tier distance LOD. |
| **20** | **World Surface & Subterranean Feature Generation** | Implemented multi-chunk procedural trees (13 archetypes), connected column branches, arboreal pods, wild crops, leaf litter, surface/cave rubble, pebbles, and coral reefs. |

---

## Milestone Transition: Version 0.2 → Version 0.3

> [!NOTE]
> **Version 0.2 Complete — Advancing to Version 0.3**  
> With all 20 phases of Version 0.2 completed, the project advances to **Version 0.3**, resetting the active roadmap cycle to Phases 01–20 for the next wave of gameplay, world, and engine milestones.

---

## Version 0.3: Next Development Cycle

### Phase Directory

| Phase | Milestone Title | Status | Focus / Key Deliverables |
| :---: | :--- | :---: | :--- |
| **01** | **[Phase 1: TBD](#phase-1-tbd)** | **Planned** | *(To be defined)* |

---

### Phase 1: TBD

To be defined.
