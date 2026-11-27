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
| **01** | **[Engine-Wide Performance Optimization & LOD Pipeline](#phase-1-engine-wide-performance-optimization--lod-pipeline-completed)** | **Completed** | Terrain/biome generation caching, LOD meshing overhaul, optional leaf culling, shader/grass/celestial optimizations, streaming & chunk cache improvements |
| **02** | **[World Terrain Polish & Surface Detail Features](#phase-2-world-terrain-polish--surface-detail-features-completed)** | **Completed** | Universal slab/stair terrain smoothing, sculpted outcrops & boulders, ground bushes, ocean biomes & coral/reefstone formations, fallen logs & forest moss, sculpted tree canopies, campfire, toggleable lamps, geothermal vents |
| **03** | **[Atmospheric & Gameplay Particle Engine](#phase-3-atmospheric--gameplay-particle-engine-completed)** | **Completed** | Falling canopy leaves (21 sprites, selective tint), underwater geothermal/magma bubbles + pop, footstep/landing dust, blinking night fireflies, daytime pollen, player/item water splashes, campfire smoke |
| **04** | **[Pure-Code UI & Menu System Redesign](#phase-4-pure-code-ui--menu-system-redesign-in-progress)** | **In Progress** | Pure Bevy UI Dark Iron theme (`src/menu/theme.rs`), 6-tab top bar with icons & live status HUD, 3-column cascading System/Settings/Controls drawers, fullscreen World Map with Atlas Overlays, searchable localized Journal Compendium (78 biomes, expandable cards, per-category counts), family-grouped Creative Inventory search, Skills/Quests screens, and expanded Display/Audio/Camera/Controls Settings |
| **05** | **[Races, Subraces, Attributes & Skills Redesign](#phase-5-races-subraces-attributes--skills-redesign-completed)** | **Completed** | 3 classic core races × 3 subraces each (9 playable subraces) with face-portrait icons, 16-field `RaceAttributes` (live jump height, move/swim speed, footstep stealth, mining reach), 3-column Race Selection UI, and 60 localized skills (4 general trees + dynamic racial/subracial tree) with 60 bespoke pixel-art icons |
| **06** | **[Main Menu Redesign](#phase-6-main-menu-redesign-in-progress)** | **In Progress** | 2-side Main Menu with character-gated `New Game` flow (`CharacterReady`), 3-mode World Selection screen (`Laboratory`, `Sandbox`, `Adventure`), right-side character showcase panel, and future 3D character preview |
| **07** | **[Dedicated Character Creator](#phase-7-dedicated-character-creator-planned)** | **Planned** | Interactive character customization (skin tone, hair style/color, beard/features, outfit palette, character naming, and live 3D voxel model preview) |
| **08** | **[Survival Mode Core Systems](#phase-8-survival-mode-core-systems-in-progress)** | **In Progress** | Default Survival mode (`S → C → E`), floating GUI-scaled Vitals HUD, live stamina/drowning/fall damage/respawn, 11 flint tools & crafting materials, 64-item stacking & consumption, block durability with 9-stage crack overlay + shake, Fire Starter/Hoe/Shears actions (hunger/thirst/temperature sim, food buffs & tool durability next) |
| **09** | **[Mobs, Wildlife & Entity AI](#phase-9-mobs-wildlife--entity-ai-planned)** | **Planned** | First creatures and wildlife (passive fauna, hostile mobs, pathfinding, stealth/footstep awareness, combat, taming, and live Bestiary unlocking) |
| **10** | **[Quests, Objectives & Progression Engine](#phase-10-quests-objectives--progression-engine-planned)** | **Planned** | Functional quest state machine, objective tracking, exploration/gathering/combat triggers, XP/leveling, skill point spending, and rewards |
| **11** | **[Subterranean World, Caves & Underground Biomes Refactor](#phase-11-subterranean-world-caves--underground-biomes-refactor-planned)** | **Planned** | Ground-up redesign of subterranean caverns, underground biomes, surface cave entrances, crystal geodes, speleothems, subterranean fluids, and depth-based ore progression (legacy caves removed; world below surface is currently 100% solid rock & ores) |
| **12** | **[Adventure Mode, 78-Biome Overhaul & Tri-Mode World Architecture](#phase-12-adventure-mode-78-biome-overhaul--tri-mode-world-architecture-in-progress)** | **In Progress** | 27 Macro-Biomes & 78 surface Micro-Biomes, pure-code `2,880×2,880` radial **Laboratory** 78-biome atlas, **Sandbox** with random seeds & safe dry-land spawn, 6-layer `3200×1800` **Adventure Mode** loader, elevated lakes & stepped rivers, World Map Atlas Overlays & async Full-World Preload |
| **13** | **[Lore, Worldbuilding & Regional Identity](#phase-13-lore-worldbuilding--regional-identity-planned)** | **Planned** | Official world naming, continents/regions lore, named landmarks, ancient ruins, trade routes, and faction territories |
| **14** | **[Cross-System Revisions, Bug Fixes & Content Polish](#phase-14-cross-system-revisions-bug-fixes--content-polish-completed)** | **Completed** | Display/Fullscreen/Mouse-Lock fixes, Audio Mute-on-Unfocus & UI volume tuning, Movement input lock in all menus, `Crawl` keybind, Creative Inventory localized search & family sorting, Journal Compendium search/cards/counters, Free Cam (`F8`) vs noclip Spectator (`F4`), `Chunk Border Fog` rename, and 19 new blocks (`736` active blocks) |
| **15** | **[Visual Overhaul: Lighting, Skybox, Atmospheric Fog & Shaders](#phase-15-visual-overhaul--lighting-skybox-atmospheric-fog--shaders-completed)** | **Completed** | Complementary/BSL-inspired lighting contrast (warm sun vs cool ambient), `TonyMcMapface` tonemapping + `ColorGrading` vibrance & HDR Bloom tuning, 3-zone procedural Sky Dome, altitude-aware & per-biome `Atmospheric Fog` separate from `Chunk Border Fog`, sky-reflective Fresnel water with active organic foam/caustics/sun speculars, grass depth gradient + foliage SSS + procedural cloud shadows |
| **16** | **[Water, Underwater & Multi-Liquid Visual Overhaul](#phase-16-water-underwater--multi-liquid-visual-overhaul-completed)** | **Completed** | Rain Puddles & pixel-art ripples on puddles/water, projected underwater caustics on all submerged terrain & shapes, underwater radial sunbeams & depth-aware fog, universal Particular-style 3D fluid splashes (`fluid_splash*.png`), animated surface fluid bubbles (`fluid_bubble.png` on Lava/Magma/Molten/Tar/Acid), recolored violet-black Tar, opaque-fluid culling fix, and bespoke physics & submersion visuals across all 9 engine liquids |

---

### Phase 1: Engine-Wide Performance Optimization & LOD Pipeline (Completed)

- [x] **Terrain, Biome & Flora Generation Optimization**:
  - Optimized 6D biome sampling, column terrain evaluation, and flora generation in `src/generation/` to reduce per-chunk CPU work and allocation overhead.
- [x] **LOD Meshing & Streaming Pipeline Overhaul**:
  - Improved LOD chunk generation and meshing throughput (`src/meshing/lod.rs`), refined streaming planner/loader/unloader scheduling, and optimized chunk storage and disk cache paths.
- [x] **Greedy Meshing, Culling & Optional Leaf Culling**:
  - Streamlined greedy meshing, visibility culling, and texture lookup paths (`src/meshing/`); added an optional `leaf_culling` graphics setting (disabled by default) persisted in `local_config.json`.
- [x] **Shaders, Procedural Grass & Per-Frame Systems Cleanup**:
  - Optimized WESL shaders (`grass_blades.wesl`, `voxel_opaque.wesl`, `voxel_prepass.wesl`), grass trail updates, player state queries, celestial/atmosphere updates, and dev stats overhead.

---

### Phase 2: World Terrain Polish & Surface Detail Features (Completed)

- [x] **Universal Slab & Stair Terrain Smoothing (Dry & Underwater)**:
  - Extended smooth slab/stair surface transitions across all dry terrain (sandy beaches, riverbanks, clay, mud) and submerged riverbeds, lakebeds, and seafloors.
- [x] **Waterlogged Slabs & Stairs Meshing**:
  - Extended `is_submerged_fixture` and `mesh_waterlogged_envelope` to seamlessly waterlog submerged slabs and stairs, rendering the water surface over shallow slabs/stairs and eliminating underwater air pockets and side water walls.
- [x] **Sculpted Basalt & 3D Rock Outcrops**:
  - Added a surface overhang shaping pass applying stairs, slabs, and inverted overhang shapes along curves, crests, and edges of 3D rock/basalt mounds seamlessly across chunk borders.
- [x] **Surface Cobble & Mossy Boulder Clusters**:
  - Added multi-chunk regional cobble and mossy cobble rock clusters (`Cobbled*` + `MossyCobbled*`) scattered across forests, valleys, plains, and highlands with slope footing.
- [x] **Ground Leaf Bushes**:
  - Added multi-chunk woodland and meadow bushes (single central biome log wrapped by top and cardinal leaf blocks) scattered across vegetated biomes.
- [x] **Ocean Biome Resolution, Varied Seabed & Multi-Block Coral Reefs**:
  - Resolved open ocean bodies (`terrain_height < sea_level`, non-river/non-lake) to authentic ocean biomes (`DeepWarmOcean`, `ColdOcean`, `CoralReef`, `OceanTrench`) so the minimap HUD and worldgen never misreport terrestrial biomes over open sea.
  - Enriched ocean floors with multi-scale organic sediment variety (white/common/dune sands, sandy/dirty gravel, blue/yellow/white clay, silt/peat mud, loam dirt, reefstone, and abyssal basalt).
  - Added `Coralstone` seabed patches beneath coral colonies and underwater `Coralstone` boulder clusters around reef formations.
- [x] **Coastal Reefstone Boulders & Sea-Stack Spires**:
  - Added multi-chunk sculpted `Reefstone` boulder clusters (concentrated in coastal/submerged waters with sparse dark-rubble-ringed dry beach outcrops) plus column-anchored tapered `Reefstone` sea stacks sculpted with slabs and stairs.
- [x] **Fallen Forest Logs, Forest Moss Patches & Coastal Beach Grass**:
  - Added multi-chunk fallen forest trunks (with optional stumps, top mushrooms, and adjacent moss), organic `MossForest` floor patches across woodlands, `VegeDrygrass` across sandy beaches, and green `VegeWildgrass` tufts around coastal palm trees.
- [x] **Sculpted Tree Canopies (Slabs & Stairs), Giant Mushrooms & LOD `full_grass` Tinting**:
  - Sculpted the outer silhouette rims of deciduous leaf clouds, acacia umbrellas, palm fronds, and conifer boughs with `Slab` and upright/inverted `Stair` shapes while preserving full interior blocks for greedy-meshing performance.
  - Added multi-chunk Giant Mushrooms (`FungiStem` stalk, `FungiInside` interior gills, and domed/umbrella `Fungi*Cap` canopies across all 5 species) with sparse regional spawns in swamps, cloud forests, jungles, taigas, and woodlands (including LOD proxies).
  - Fixed LOD vertical side wall tinting (`push_lod_wall` in `src/meshing/lod.rs`) so `full_grass` renders biome-tinted grass sides on distant LOD chunks instead of grayscale.
  - Upgraded coral reefs into clustered multi-block 3D spires, branching trees, and reef mounds, with 6-direction wall/floor/ceiling attachment support for coral fans in both worldgen and player placement.
- [x] **Underwater Geothermal Vents, Campfire & Toggleable Lamps**:
  - Added natural hydrothermal `GeothermalVent` chimneys and submerged `Magma` clusters in deep oceans and trenches (`src/generation/flora.rs`).
  - Implemented 3D crossed-log `Campfire` (`Campfire` / `CampfireUnlit`) and off-state `EmitLamp*Off` blocks with empty-hand right-click toggling, alongside weather fog/dust/streak textures in `weather_atlas.png`.

---

### Phase 3: Atmospheric & Gameplay Particle Engine (Completed)

- [x] **Falling Canopy Leaf Particles (`src/environment/foliage/particles.rs`)**:
  - Packed all 21 leaf particle sprites into a crisp runtime nearest-neighbor atlas rendered in a single batched draw call (`FallingLeafBatchMesh`), with selective biome/conifer tinting (`common_leaf*`, `conifer_leaf*`), `64×64×24` sampling range, sinusoidal flutter, ground landing, and smooth cubic fade-out.
- [x] **Unified Batched World Particle Engine (`src/environment/particles.rs`)**:
  - Packed all 27 atmospheric and gameplay particle sprites into a single `128×64` nearest-neighbor atlas rendered via `WorldParticleBatchMesh` with `WorldParticleBurstQueue`.
- [x] **Underwater Bubbles & Surface Pop (`bubble*`, `bubble_pop*`)**:
  - Emitted rising wobble bubbles from submerged `GeothermalVent` and `Magma` blocks, automatically transitioning into 3-frame `bubble_pop` animations at the water surface.
- [x] **Footstep & Landing Dust (`dust_gust*`)**:
  - Spawned 6-frame animated dust puffs tinted by the floor block material when walking, sprinting, or landing from falls.
- [x] **Blinking Night Fireflies & Daytime Pollen (`firefly*`, `pollen*`)**:
  - Added dusk/night bioluminescent fireflies in forests and `OrchidSwamp` with pulsing HDR emissive blinking tint, plus sunny daytime pollen motes drifting over floral and forested biomes.
- [x] **Player & 3D Item Drop Water Splashes (`splash*`)**:
  - Emitted ballistic water-tinted splash droplets whenever the player enters/wades in water or dropped 3D items hit water surfaces (`src/audio/footsteps.rs`, `src/gameplay/item_drops.rs`).
- [x] **Campfire Smoke Plumes (`smoke*`)**:
  - Emitted rising, expanding smoke puffs strictly above lit `Campfire` blocks (`Voxel::Campfire`), excluded from torches and unlit campfires.

---

### Phase 4: Pure-Code UI & Menu System Redesign (In Progress)

- [x] **Unified Pure-Code Dark Iron Theme (`src/menu/theme.rs`)**:
  - Introduced a centralized Dark Iron / Forged Obsidian color palette (`palette`), standardized `310px` drawer column dimensions, and reusable `ThemedButton` component via Bevy UI `BackgroundColor`, `BorderColor`, and `BorderRadius`.
  - Removed the legacy `MenuButtonTexture` sprite-swapping component from `src/menu/mod.rs` and deleted `assets/textures/interfaces/containers/` (retaining `main_menu_bg.png`, `cursors/`, `icons/`, and `atlases/`).
- [x] **Shared 6-Tab Top Bar & Direct Shortcuts (`src/menu/menu_bar.rs`)**:
  - Redesigned the top navigation bar with clean text-only tabs, gold active underline, 16×16 pixel-art icons (`Inventory`, `Skills`, `Journal`, `Quests`, `World Map`, `System`), `[Q]`/`[E]` tab cycling, and a live right-side status bar (Level, live Clock with time-of-day icon, Gold Coins, and clickable `[ESC]` close keycap).
  - Added direct rebindable shortcuts: `Tab` (Inventory), `H` (Skills), `J` (Journal), `K` (Quests), `M` (World Map), and `Esc` (System / step-back drawer navigation).
- [x] **Left-Docked 3-Column Cascading System, Settings & Controls Drawers**:
  - Rebuilt `System` (`src/menu/pause.rs`), `Settings` (`src/menu/settings.rs`), and `Controls` (`src/menu/controls.rs`) into full-height left-docked cascading drawers (up to 3 columns), adding `Save World`, `Restart World`, functional `Game Mode` switching inside `World & Session`, and direct `Exit Game`.
  - Reorganized Settings into 6 prioritized categories (`World`, `Graphics` with Display at top, numeric `Fullscreen Resolution`, and conditional `Borderless Mouse Lock`, `Shaders & FX` including `Procedural Grass` & `Grass Density`, `Camera` with `Camera Smoothing`, `Third-Person Distance`, `Over-the-Shoulder` style, and `Free 360° Orbit`, `Audio` with `UI Volume` & `Mute on Unfocus`, and `Accessibility & UI` with `Menu Pause`, `Auto-Save Interval`, 4-tier `FPS / Debug HUD`, functional `High-Contrast Crosshair`, and fade-animated `Dynamic Crosshair`).
- [x] **Interactive Fullscreen World Map, Skills, Journal & Quests Screens**:
  - Rebuilt the World Map (`src/map/world_map.rs`) to fill the full viewport below the top bar with 1:1 aspect-corrected terrain rendering, UI-scale-accurate cursor projection, RMB context menu (`Add Waypoint Here` + Creative-only `Teleport Here`), 7-icon waypoint system with cursor-following placement mode and scrollable/inline-renamable sidebar list, and 5 persisted layers in `local_config.json` (`Player Marker`, `Waypoints`, subtle `16x16 Chunk Grid`, `Day / Night Lighting`, and smoothed `Elevation Contours`).
  - Built dedicated `Skills` (`src/menu/skills.rs`), fully localized scrollable & searchable `Journal` (`src/menu/journal.rs` with F6-synchronized live season/day/clock/weather almanac card, `COMPENDIUM` discovery tracking via `JournalProgress` across all 78 surface biomes with 78 unique 16×16 icons in `icons/biomes/`, `Bestiary & Races` with expandable cards, subcategorized live-unlocked `Flora & Agriculture`, `Blocks & Minerals`, and `Items & Equipment` with per-category counters, plus 27 categorized `Statistics` with 27 bespoke pixel-art icons and 16 live trackers), and localized `Quests` (`src/menu/quests.rs`) screens.
- [x] **Hotbar, Dynamic Input Prompts & 36-Slot Inventory Overhaul**:
  - Added reactive Dark Steel pixel-art input prompts (`src/core/input_prompts.rs`, `assets/textures/interfaces/atlases/inputs_tilemap.png`) replacing top-bar `[Q]`/`[E]`/`[ESC]` text keycaps and automatically syncing with rebindable `ControlSettings` (gamepad-ready).
  - Removed the placeholder XP bar from `src/player/hotbar.rs`, grouped the 2 auxiliary slots (`Off-hand` + `Quick Utility`) into a separated right-side tray, fixed the active selection highlight to a border-only gold frame, and added bottom-right stack count numbers across Hotbar & Inventory.
  - Expanded Personal Inventory (`src/player/inventory.rs`, `src/menu/creative_inventory.rs`) to 36 slots (4×9) with a locked 4th row partially unlocked by subrace (`carry_slots` shown as the 16th attribute in both `src/menu/race_selection.rs` and `src/menu/skills.rs`), and disabled/hid the Creative Catalog toggle during Survival mode.
- [ ] **Remaining UI Polish (Final Step of Phase 4)**:
  - Personal Inventory upper character/equipment/crafting panel redesign, expanding dynamic keycap prompts to World Map/Controls/HUD hints, and cleaning up remaining UI placeholders.

---

### Phase 5: Races, Subraces, Attributes & Skills Redesign (Completed)

- [x] **3 Core Playable Races × 3 Subraces Each (9 Playable Subraces)**:
  - Streamlined playable lineages in `src/player/race.rs` to **Humans** (`Northern Human`, `Southern Human`, `Coastal Human`), **Elves** (`Wood Elf`, `Dark Elf`, `High Elf`), and **Dwarves** (`Mountain Dwarf`, `Deep Dwarf`, `Hill Dwarf`), while retaining `Orc`, `Dragonborn`, and `Tiefling` rigs for future NPCs/mobs and removing `Halfling` and legacy standalone `Drow`.
  - Created 9 dedicated 16×16 face-portrait subrace icons (`subrace_*.png`) separate from the 3 core race icons (`race_human.png`, `race_elf.png`, `race_dwarf.png`), plus full EN/PT-BR lore descriptions and 2 racial traits per subrace.
- [x] **16-Attribute `RaceAttributes` & Live Gameplay Modifiers**:
  - Defined distinct per-subrace profiles covering Health, Stamina, Defense, Move Speed, Swim Speed, Jump Height, Footstep Noise (Stealth), Mining Speed, Mining Reach, Cave Vision, Fall Resistance, Knockback Resistance, Cold Resistance, Heat Resistance, XP Gain, and Carry Slots.
  - Wired immediately playable modifiers directly into gameplay: movement & swim speed (`src/player/movement.rs`), 2-block elven jump impulse (`RaceAttributes::jump_speed`), footstep/landing audio stealth volume (`src/audio/footsteps.rs`), block targeting reach bonus (`src/gameplay/targeting.rs`), and subrace 3D character palettes (`src/player/model.rs`).
- [x] **3-Column Race Selection & 60-Skill Official Catalog**:
  - Redesigned `src/menu/race_selection.rs` into a pure-code Dark Iron 3-column selector (Core Races → Race Lore & 3 Subraces → Subrace Details, 15-stat Attributes Grid & Traits).
  - Officialized 60 localized skills across 4 general trees (`Combat`, `Gathering`, `Crafting`, `Exploration` — 24 skills) and a dynamic `Racial` tree (3 core race base skills + 3 subrace-exclusive Tier III skills — 36 racial/subracial skills), each paired with a bespoke 16×16 pixel-art icon in `assets/textures/interfaces/icons/skills/`.

---

### Phase 6: Main Menu Redesign (In Progress)

- [x] **2-Side Main Menu & Character-Gated New Game Flow**:
  - Redesigned `src/menu/main_menu.rs` Action Phase into a 2-side layout: Left side displays title, primary `New Game` button (locked via `CharacterReady(false)` with localized hint until a character is selected), `Settings`, `Controls`, and `Quit`; Right side displays the Dark Iron Character Panel with subrace portrait, player name, subrace title, stat summary pills, and `Create Character >` / `Change Character >` routing to `MenuState::RaceSelection`.
- [x] **World Mode Selection Screen (`Laboratory`, `Sandbox`, `Adventure`)**:
  - Added `MainMenuPhase::NewGameModeSelect` and `SelectedWorldMode` in `src/menu/main_menu.rs` with three Dark Iron cards presenting **Laboratory** (compact radial 78-biome showcase atlas, Creative default), **Sandbox World** (infinite procedurally generated world with random seeds), and **Adventure Mode** (bounded handcrafted map), complete with EN/PT-BR localization and `ESC` / Back navigation.
- [ ] **Future Main Menu Expansions**:
  - Live 3D voxel character preview render on the Main Menu, multi-character roster slots, and saved world / Continue list.

---

### Phase 7: Dedicated Character Creator (Planned)

- [ ] **Visual Character Customization**:
  - Dedicated character creation screen allowing players to customize skin tone, hairstyle and hair color, facial hair/racial features, outfit palette, and custom character name with a live rotatable 3D voxel preview.

---

### Phase 8: Survival Mode Core Systems (In Progress)

- [x] **Survival Game Mode, HUD & Core Vitals Loop (`src/player/survival_hud.rs`, `src/player/game_mode.rs`)**:
  - Made `GameMode::Survival` the default starting mode with `F4` cycling `Survival → Creative → Spectator` and a centered Dark Iron mode/respawn toast popup (Spectator starts in a 3rd-person offset while keeping the player model in place with idle breathing).
  - Built the floating bottom-left Survival Vitals HUD (horizontally aligned with and dynamically scaled by `gui_scale` to match the hotbar) featuring Health, Stamina, vertical circular progress rings for Hunger/Thirst/Temperature (`assets/textures/interfaces/icons/hud/`), and a dynamic submerged Oxygen ring.
  - Wired live stamina consumption on sprinting (ground & mid-air) and jumping with exhaustion gating, underwater oxygen drain + periodic drowning damage, fall damage mitigated by racial `fall_damage_reduction` and safe-jump height, death & respawn at `WorldSpawnPoint` with live `JournalProgress` stats (`deaths`, `damage_taken`), and Survival-accurate middle-click Pick Block (`src/gameplay/interaction.rs`).
- [x] **Flint Tools, Block Durability, 9-Stage Crack Overlay & 64-Item Stacking (`src/world/block/props.rs`, `src/gameplay/feedback.rs`, `src/gameplay/interaction.rs`, `src/gameplay/item_drops.rs`)**:
  - Registered 11 new inventory-only crafting materials & flint tools (`ItemFlintShard`, `ItemPlantFiber`, `ItemPlantString`, `ToolFireStarter`, `ToolFlintAxe/Hammer/Hoe/Knife/Pickaxe/Shears/Shovel`) with starting hotbar kit (Pickaxe, Axe, Shovel, 8 Torches).
  - Implemented per-material block hardness, tool speed multipliers + racial `mining_speed_multiplier`, 9-stage grayscale crack overlay (`assets/textures/masks/break_stages/`) with subtle block shake/pulse, and an under-crosshair mining progress bar.
  - Wired tool actions (`ToolFireStarter` lights campfires & places fire, required for Survival campfires; `ToolFlintHoe` tills grass/dirt into farmland; `ToolFlintShears` harvests original vegetation instead of `ItemPlantFiber`) and full 64-item stacking (1 for tools) with Survival block consumption, merge/split, and single-item `Q` dropping.
- [ ] **Remaining Survival Systems**:
  - Non-lethal Hunger & Thirst drain with debuffs and consumable food/drink buffs, climate/biome Temperature simulation (applying racial cold/heat resistance and campfire warmth), basic pocket/station crafting, and tool durability.

---

### Phase 9: Mobs, Wildlife & Entity AI (Planned)

- [ ] **First Creatures & Autonomous AI**:
  - Introduce initial passive wildlife and hostile mobs (reusing preserved Orc, Dragonborn, and Tiefling rigs alongside new fauna), voxel pathfinding, perception/stealth noise integration (`footstep_noise_multiplier`), combat, taming, and automatic Bestiary discovery.

---

### Phase 10: Quests, Objectives & Progression Engine (Planned)

- [ ] **Functional Quests & RPG Progression**:
  - Power the `Quests` screen with real quest definitions, multi-step objectives, world/gathering/combat triggers, XP & leveling progression, spendable Skill Points across the 60-skill catalog, and quest rewards.

---

### Phase 11: Subterranean World, Caves & Underground Biomes Refactor (Planned)

> [!NOTE]
> All legacy cave carvers (`src/generation/caves.rs`), surface cave entrances, hollow geodes, speleothems, cave flora, and the 4 legacy subterranean biomes (`ThermalCaves`, `UndergroundJungle`, `IceCaves`, `MushroomCaves`) have been completely removed so the world below the surface is currently 100% solid geological strata and ores. This entire layer will be redesigned from scratch in Phase 11.

- [ ] **Ground-Up Subterranean Cavern & Entrance Architecture**:
  - Design a brand-new 3D subterranean cavern network (cheese/spaghetti/noodle chambers, deliberate surface cave entrances, underground aquifers, and structural stability without breaking rivers/lakes or floating trees).
- [ ] **New Subterranean Biomes & Compendium Integration**:
  - Design and register a cohesive set of subterranean biomes with bespoke surface/wall/ceiling materials, dedicated pixel-art icons, Journal Compendium category, cave flora, lighting, and spatial ambience.
- [ ] **Ore Veins, Crystal Geodes & Mining Progression**:
  - Rebalance procedural ore veins, reintroduce crystal geodes and speleothems tailored to the new cave architecture (`src/generation/ores.rs`), and tie depth-based mineral progression into Survival tool tiers.

---

### Phase 12: Adventure Mode, 78-Biome Overhaul & Tri-Mode World Architecture (In Progress)

- [x] **27 Macro-Biomes & 78 Surface Micro-Biomes Architecture (`src/generation/biome.rs`, `docs/biomes_design.md`)**:
  - Replaced all legacy surface biomes with the official 27 `MacroBiome` and 78 surface `BiomeType` micro-biomes across 9 geographic categories, wired across terrain, trees, flora, strata, weather particles, ambience, 6D Sandbox Voronoi climate, EN/PT-BR localization, and 78 unique 16×16 pixel-art icons in `assets/textures/interfaces/icons/biomes/`.
  - Added new biome-specific blocks and texture variants: `SeagrassSand`, `TallSeagrassBottom`/`Top` (15-frame animated 2-block underwater seagrass), `SiltTundraGrass`, `LoamTundraGrass`, `PeatTundraGrass`, `PermafrostTundraGrass` (with reddish GPU procedural grass blades), `VegeArcticMoss`, and `VegeArcticGrass`.
- [x] **Pure-Code Radial Laboratory World (`src/generation/laboratory_map.rs`)**:
  - Built a 3rd world mode (`WorldMode::Laboratory`) generating a `2,880 × 2,880` circular atlas containing all 78 surface biomes (central spawn hub + moat, 6 terrestrial 60° slices × 9 biomes separated by meandering rivers, 9-biome coastal ring, 6 ocean sectors, and 6 outer islands), starting in Creative Mode with automatic `[M]` World Map preloading.
- [x] **6-Layer `3200×1800` Handcrafted Adventure Mode Loader (`src/generation/adventure_map.rs`)**:
  - Implemented compact `Arc`-shared loading of `heightmap.png`, `macro_biomes.png`, `micro_biomes.png`, `world_continents.png`, `world_regions.png`, and `world_landmass.png` supporting **Full (`12,800 × 7,200`)** and **Medium (`6,400 × 3,600`)** scales, synthetic biome+latitude+altitude climate, inland-only domain warping, 3 validated dry-land spawns, and an in-game top-down rendered `map_preview.png`.
- [x] **Elevated Highland Lakes & Stepped Mountain Rivers (`src/generation/adventure_map.rs`, `src/generation/generator.rs`)**:
  - Implemented per-pixel `water_level_i16` BFS basin flood-fill for elevated `CalmLake` basins and smoothed stepped elevations for inland `Rapids` rivers so highland water bodies sit naturally at their true terrain elevation without carving deep chasms down to `SEA_LEVEL (Y = 12)`.
- [x] **Tri-Mode World Routing (`Laboratory` / `Sandbox` / `Adventure`) & World Map Atlas Overlays (`src/map/world_map.rs`, `src/menu/pause.rs`)**:
  - Wired `SelectedWorldMode` so **Laboratory** loads the radial 78-biome testbed, **Sandbox** uses infinite 6D procedural generation with randomized seeds and `find_safe_land_spawn()` dry-land spawn safety, and **Adventure Mode** loads the bounded handcrafted world.
  - Added interactive **Atlas Overlay** filters on the `[M]` World Map (`Terrain`, `Continents`, `Regions`, `Macro-Biomes`, `Micro-Biomes`), fixed-height hover region/biome cursor info, and persisted `Show Undiscovered Compendium` + async `Preload Full World Map` toggles in System → World & Session (`local_config.json`).
- [ ] **Adventure Biome Visual Tuning, Regional Gameplay & Save-Slot Separation**:
  - Fine-tune individual micro-biome surface blocks/flora/palettes in Adventure Mode, add HUD Region/Continent Discovery Banners, region-specific ambience/weather rules, and separate save-slot metadata per world mode.

---

### Phase 13: Lore, Worldbuilding & Regional Identity (Planned)

- [ ] **Official World Naming & Geography Lore**:
  - Replace generic placeholder names (Western/Central/Eastern Continents, Southern/Volcano Archipelagos, 12 Official Regions) with official lore names, history, and flavor descriptions across `assets/lang/en_us.json`, `assets/lang/pt_br.json`, World Map tooltips, Discovery Banners, and the Journal.
- [ ] **Landmarks, Ruins & Cultural Identity**:
  - Define named points of interest (POIs), ancient ruins, trade routes, faction territories, and regional storytelling hooks tied directly to the Adventure Mode continents and regions.

---

### Phase 14: Cross-System Revisions, Bug Fixes & Content Polish (Completed)

- [x] **Display, Fullscreen & Mouse Lock Fixes (`src/menu/settings.rs`, `src/menu/mod.rs`, `src/core/window_clip.rs`)**:
  - Added a numeric-only **Fullscreen Resolution** selector capped at the monitor's native resolution, resolved native Exclusive Fullscreen Independent Flip FPS/180Hz throttling, added automatic minimize on Alt+Tab/unfocus in Exclusive Fullscreen, and fixed Win32 `ClipCursor` enforcement for `Borderless Mouse Lock`.
- [x] **Audio Mixer & UI Volume Polish (`src/audio/`)**:
  - Fixed **Mute on Unfocus** across all active `AudioSink` channels (music, ambience, weather, footsteps, SFX, UI), wired `UI Volume` slider reactivity, and attenuated base UI click gain for a cleaner soundscape.
- [x] **Controls, System Drawer & Input Locking (`src/core/controls.rs`, `src/menu/controls.rs`, `src/menu/pause.rs`, `src/player/movement.rs`)**:
  - Added `InputAction::Crawl` (`C`) to the Movement keybinds category, moved `Restart World` directly below `Save World`, wired live `Game Mode` switching inside `World & Session`, removed unfinished multiplayer/difficulty/Subtitles placeholders, fixed player Y-coordinate flicker (`minimap.rs`, `dev_stats.rs`, `world_map.rs`), and locked player movement input whenever any menu (`Inventory`, `Skills`, `Journal`, `Quests`, `WorldMap`) is open.
- [x] **Creative Inventory & Journal Compendium Overhaul (`src/menu/creative_inventory.rs`, `src/menu/journal.rs`)**:
  - Upgraded Creative Inventory search to localized, accent-insensitive display-name matching (excluding internal IDs) and replaced raw enum order with language-independent canonical English sorting grouped by structural sub-families (`sub_family_rank`).
  - Standardized all Journal Compendium cards (`235px` / `36×36` icons, subtle `+` expand indicators, uniform expandable descriptions), added `Bestiary & Races` *Native Peoples* category, filtered unlit duplicate variants (`Campfire (Unlit)`), added per-subcategory `(discovered / total)` counters across all 5 Compendium tabs, added a `Statistics` subtitle, and built real-time localized search bars in all 5 Compendium tabs with full menu-hotkey blocking while typing.
- [x] **New Geological, Coastal, Thermal & Palm Blocks (`736` Active Blocks, `docs/blocks_design.md`)**:
  - Added 19 new blocks with full texture variants, 2D icons, and EN/PT-BR localization: `Terracotta`, `WhiteTerracotta`, `YellowTerracotta`, `SandPink`, `SandBlack` (+ Pink/Black `Sandstone`, `CobbledSandstone`, `SandstoneBricks`, `SandstoneRubble`), Yellowstone-toned `Geyserite`, `RedGeyserite`, `WhiteGeyserite`, revamped Palm bark/log/colored leaves + `DeadPalmLeaves` + colored leaf particles, and unlit-default right-click-toggleable `TreeCharred*` / `TreeCharred*Lit` blocks. Synced `docs/blocks_design.md` to all `739/739` voxel entries.

---

### Phase 15: Visual Overhaul — Lighting, Skybox, Atmospheric Fog & Shaders (Completed)

- [x] **Step 1 — Lighting Contrast, Dynamic Warm/Cool Palette, 3-Zone Climate & Post-Processing (`src/environment/atmosphere.rs`, `src/environment/celestial.rs`, `src/player/mod.rs`, `src/environment/post_process.rs`, `assets/shaders/post_process.wesl`)**:
  - Adopted Complementary-inspired warm-sun vs. sky-derived cool-ambient lighting contrast (`DAY_SUN_ILLUMINANCE = 16,500 lx`, `DAY_AMBIENT_BRIGHTNESS = 1,550 lx`, `EV100 = 11.9`), fixed the nocturnal duration plateau (`21:00–04:15` genuinely dark night), and added smooth 3-zone climate lighting (`Cold`, `Temperate`, `Warm`).
  - Switched camera tonemapping to `Tonemapping::TonyMcMapface` + native `ColorGrading` vibrance (`post_saturation = 1.14`) and tuned HDR `Bloom` alongside crepuscular God Rays.
- [x] **Step 2 — 3-Zone Procedural Sky Dome, Post-Rain Rainbow, Aurora Borealis, Twinkling Stars & Nocturnal/Storm Clouds (`src/environment/sky.rs`, `assets/shaders/sky_dome.wesl`, `src/environment/stars.rs`, `src/environment/clouds.rs`)**:
  - Replaced flat `ClearColor` sky with a procedural 3-zone Sky Dome featuring sunrise/sunset horizon ribbons, forward-scattered Sun/Moon Glare, post-rain 42° rainbows, and smooth polar night Aurora Borealis (toggleable in Settings).
  - Upgraded the night sky to 900 batched colored twinkling stars (4 draw calls) and dark nocturnal/storm-tinted clouds driven by weather wind speed and synchronized with `F6` time scrubbing.
- [x] **Step 3 — Altitude-Aware Atmospheric Fog, Per-Biome Fog & Sky-Matched Chunk Border Fog (`src/environment/atmosphere.rs`, `src/menu/settings.rs`, `src/core/config.rs`, `assets/lang/en_us.json`, `assets/lang/pt_br.json`)**:
  - Added an independent **Atmospheric Fog** setting (separate from **Chunk Border Fog**) with altitude-aware aerial perspective and smooth per-biome color/density profiles, while refining **Chunk Border Fog** to match the Sky Dome horizon.
- [x] **Step 4 — Grass Depth Gradient, Foliage Subsurface Scattering, Cloud Shadows & Ambient Particles (`assets/shaders/grass_blades.wesl`, `assets/shaders/voxel_opaque.wesl`, `assets/shaders/voxel_prepass.wesl`, `src/environment/particles.rs`, `src/environment/weather/particles/emitters.rs`)**:
  - Expanded grass blade root-to-tip shading (`mix(0.48, 1.12)`), added backlit fake Subsurface Scattering on leaves/flora/grass, procedural drifting cloud shadows (with Settings toggles), storm wind sway boosts, and player parting/push on cross-mesh flora & crops.
  - Added animated horizontal wind swirl ribbons (`wind*.png`), sky-exposed raindrop ground splashes (`rain_drop*.png`), daytime butterflies (`ambience_butterfly*.png`), and fixed ground fog to stay world-aligned horizontally.
- [x] **Step 5 — Real Water Reflections (SSR), Sky-Matched Fresnel, Organic Foam, Caustics & Specular Sun Glints (`assets/shaders/voxel_transparent.wesl`, `src/meshing/pipeline.rs`, `src/menu/settings.rs`)**:
  - Implemented toggleable Screen-Space Water Reflections (`Water Reflections: Real (SSR) / Sky Only`) falling back cleanly into the procedural Sky Dome reflection at screen edges, activated domain-rotated organic shoreline foam and shallow caustics, and raised `max_fluid_luma` to `3.20` for crisp sunlit speculars.

---

### Phase 16: Water, Underwater & Multi-Liquid Visual Overhaul (Completed)

- [x] **Step 1 — Rain Puddles & Wet Ground Reflections (`assets/shaders/voxel_opaque.wesl`, `assets/shaders/voxel_transparent.wesl`, `src/meshing/pipeline.rs`, `src/core/config.rs`, `src/menu/settings.rs`, `assets/lang/`)**:
  - Added smooth weather-driven ground wetness (`rain_intensity`, `thunder_intensity`, and lingering post-rain drying via `after_weather_timer`) with procedural world-space puddles on upward-facing terrain, non-repetitive 16×16 pixel-art raindrop ripples on both puddles and water blocks, `rain_drop` splashes on exposed water surfaces, and a `Rain Puddles` toggle in *Settings → Shaders*.
- [x] **Step 2 — Underwater Projected Caustics on Submerged Terrain & Shapes (`assets/shaders/voxel_opaque.wesl`, `assets/shaders/voxel_prepass.wesl`, `src/meshing/greedy.rs`, `src/meshing/shapes/mod.rs`)**:
  - Tagged opaque faces and submerged shapes (slabs, stairs, rubbles, pebbles, marine flora, coral fans) via `uv_b.y + 4096.0` (decoded identically in both `voxel_opaque.wesl` and `voxel_prepass.wesl`) and projected dual-layer rotated Voronoi caustic light webs with smooth depth attenuation down to ocean trenches.
- [x] **Step 3 — Underwater Depth Absorption Fog, Smooth Celestial Fade, Radial Sunbeams & Gentle Refraction (`assets/shaders/post_process.wesl`, `src/environment/post_process.rs`, `src/environment/atmosphere.rs`, `src/environment/celestial.rs`, `src/player/water.rs`)**:
  - Added time-of-day and depth-aware turquoise-to-abyssal underwater absorption fog, depth-attenuated sunlight & smooth underwater sun/moon disc fade, real 3D sun-anchored turquoise underwater crepuscular rays, balanced translucent pixel-art water underside view, gentle sinusoidal refraction shimmer, and subtle deep-water vignette.
- [x] **Step 4 — Universal Particular-Inspired 3D Fluid Splashes & Surface Bubbles (`src/environment/particles.rs`, `src/audio/footsteps.rs`, `src/gameplay/item_drops.rs`, `assets/textures/particles/fluid_*.png`)**:
  - Implemented 1-draw-call batched 3D hollow-prism fluid splashes (`fluid_splash*.png` + `fluid_splash_foam*.png` + `fluid_splash_ring*.png` + flying droplets) using per-fluid color palettes (deep water tone, emissive molten lava/magma, toxic lime acid, dark violet tar, crimson blood, sludge, ooze) with impact-speed height scaling and a tall secondary central plume for both the player and dropped items.
  - Added 1-draw-call 10-frame animated grayscale surface bubbles (`fluid_bubble.png`) popping on exposed `Lava`, `CoreMagma`, `Molten`, `Tar`, and `Acid` surfaces, and recolored `liquid_tar*.png` to match the rich violet-black tar palette.
- [x] **Step 5 — All-Liquids Physics, Opaque-Fluid Culling Fix & Submersion Visuals (`src/meshing/greedy.rs`, `src/player/water.rs`, `src/player/state.rs`, `src/player/movement.rs`, `src/environment/atmosphere.rs`)**:
  - Removed opaque-fluid solid face culling so all 7 opaque liquids no longer expose `0.10m` void slits on adjacent blocks.
  - Generalized player fluid detection (`submerged_fluid` / `camera_fluid`) across all 9 liquids with per-liquid swimming viscosity (`Water`/`SpringWater` 1.0×, `Acid`/`Blood` 0.85×, `Lava`/`CoreMagma`/`Molten` 0.48×, `Tar`/`Sludge`/`Ooze` 0.34×) and custom submersion fog distances, colors, ambient lighting, and screen overlays.
