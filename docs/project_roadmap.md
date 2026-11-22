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
| **04** | **[Pure-Code UI & Menu System Redesign](#phase-4-pure-code-ui--menu-system-redesign-in-progress)** | **In Progress** | Pure Bevy UI Dark Iron theme (`src/menu/theme.rs`), removal of `assets/textures/interfaces/containers/`, 6-tab top bar with icons & live status HUD, 3-column cascading System/Settings/Controls drawers, fullscreen World Map with right sidebar, fully localized Journal Compendium (44 biomes with pixel-art icons), Skills/Quests screens, and expanded Settings |
| **05** | **[Races, Subraces, Attributes & Skills Redesign](#phase-5-races-subraces-attributes--skills-redesign-completed)** | **Completed** | 3 classic core races × 3 subraces each (9 playable subraces) with face-portrait icons, 16-field `RaceAttributes` (live jump height, move/swim speed, footstep stealth, mining reach), 3-column Race Selection UI, and 60 localized skills (4 general trees + dynamic racial/subracial tree) with 60 bespoke pixel-art icons |
| **06** | **[Main Menu Redesign](#phase-6-main-menu-redesign-in-progress)** | **In Progress** | Enshrouded-inspired 2-side Main Menu with character-gated `Play` flow (`CharacterReady`), right-side character showcase panel, and future world selection/creation & 3D character preview |
| **07** | **[Dedicated Character Creator](#phase-7-dedicated-character-creator-planned)** | **Planned** | Interactive character customization (skin tone, hair style/color, beard/features, outfit palette, character naming, and live 3D voxel model preview) |
| **08** | **[Survival Mode Core Systems](#phase-8-survival-mode-core-systems-planned)** | **Planned** | Health, stamina, hunger/thirst, fall damage, climate temperature (cold/heat resistance), tool durability, resource harvesting, and death/respawn loop |
| **09** | **[Mobs, Wildlife & Entity AI](#phase-9-mobs-wildlife--entity-ai-planned)** | **Planned** | First creatures and wildlife (passive fauna, hostile mobs, pathfinding, stealth/footstep awareness, combat, taming, and live Bestiary unlocking) |
| **10** | **[Quests, Objectives & Progression Engine](#phase-10-quests-objectives--progression-engine-planned)** | **Planned** | Functional quest state machine, objective tracking, exploration/gathering/combat triggers, XP/leveling, skill point spending, and rewards |

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
  - Implemented 3D crossed-log `Campfire` (`Campfire` / `CampfireUnlit`) and off-state `EmitLamp*Off` blocks with empty-hand right-click toggling, alongside authentic Particle Rain fog/dust/streak textures in `weather_atlas.png`.

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
  - Rebuilt `System` (`src/menu/pause.rs`), `Settings` (`src/menu/settings.rs`), and `Controls` (`src/menu/controls.rs`) into full-height left-docked cascading drawers (up to 3 columns), adding `Save World`, `World & Session` (multiplayer/difficulty placeholders), and direct `Exit Game`.
  - Reorganized Settings into 6 prioritized categories (`World`, `Graphics` with Display at top + conditional `Borderless Mouse Lock`, `Shaders & FX`, `Camera`, `Audio`, `Accessibility & UI`), adding `Connected Textures`, `Menu Pause` (`System Only` / `All Menus` / `Never`), `Camera Smoothing`, `Third-Person Distance`, `FPS / Debug HUD` (`Off` / `FPS Only` / `Compact` / `Full`), `UI Volume`, `Mute on Unfocus`, `Auto-Save Interval`, `High-Contrast Crosshair`, and `Subtitles`.
- [x] **Fullscreen World Map, Skills, Journal & Quests Screens**:
  - Rebuilt the World Map (`src/map/world_map.rs`) to fill the full viewport below the top bar with 1:1 aspect-corrected terrain rendering, percentage-locked player marker, and a right-docked `310px` tools/waypoints/layers sidebar.
  - Built dedicated `Skills` (`src/menu/skills.rs`), fully localized scrollable `Journal` (`src/menu/journal.rs` with live season/day/clock/weather almanac card + all 7 weather icons, `COMPENDIUM` discovery tracking via `JournalProgress` for discovered `Biomes & Regions` across all 44 biomes with 44 unique 16×16 icons in `icons/biomes/`, `Bestiary`, live-unlocked `Flora & Agriculture`, `Blocks & Minerals`, and `Items & Equipment` using `BlockIcons`, plus 25 categorized `Statistics` with 14 live trackers), and localized `Quests` (`src/menu/quests.rs`) screens.
- [ ] **Remaining UI Polish**:
  - Inventory & Hotbar visual cohesion polish and World Map right-sidebar tools (waypoint creation/editing & layer toggles).

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

- [x] **Enshrouded-Inspired 2-Side Main Menu & Character-Gated Play Flow**:
  - Redesigned `src/menu/main_menu.rs` Action Phase into a 2-side layout: Left side displays title, primary `Play` button (locked via `CharacterReady(false)` with localized hint until a character is selected, then launches directly into `MenuState::Loading`), `Settings`, `Controls`, and `Quit`; Right side displays the Dark Iron Character Panel with subrace portrait, player name, subrace title, stat summary pills, and `Create Character >` / `Change Character >` routing to `MenuState::RaceSelection`.
- [ ] **Future Main Menu Expansions**:
  - Live 3D voxel character preview render on the Main Menu, multi-character roster slots, and multi-world selection/creation flow.

---

### Phase 7: Dedicated Character Creator (Planned)

- [ ] **Visual Character Customization**:
  - Dedicated character creation screen allowing players to customize skin tone, hairstyle and hair color, facial hair/racial features, outfit palette, and custom character name with a live rotatable 3D voxel preview.

---

### Phase 8: Survival Mode Core Systems (Planned)

- [ ] **Core Survival Loop**:
  - Implement playable Survival Mode with Health, Stamina, Hunger/Thirst, Fall Damage (applying racial fall resistance), Climate Temperature (applying cold/heat resistance across seasons and biomes), tool durability, block harvesting rules, and death/respawn flow.

---

### Phase 9: Mobs, Wildlife & Entity AI (Planned)

- [ ] **First Creatures & Autonomous AI**:
  - Introduce initial passive wildlife and hostile mobs (reusing preserved Orc, Dragonborn, and Tiefling rigs alongside new fauna), voxel pathfinding, perception/stealth noise integration (`footstep_noise_multiplier`), combat, taming, and automatic Bestiary discovery.

---

### Phase 10: Quests, Objectives & Progression Engine (Planned)

- [ ] **Functional Quests & RPG Progression**:
  - Power the `Quests` screen with real quest definitions, multi-step objectives, world/gathering/combat triggers, XP & leveling progression, spendable Skill Points across the 60-skill catalog, and quest rewards.
