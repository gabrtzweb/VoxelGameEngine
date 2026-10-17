# Voxel Game Engine — Development Roadmap

---

## Roadmap Overview & Milestone Structure

The Voxel Game Engine development lifecycle is structured into major evolutionary versions:

- **Version 0.1 — Foundation Era (Phases 1–20) [Completed]**: Established the core engine architecture, multi-chunk streaming, greedy meshing, procedural terrain, physics, fluid dynamics, celestial environments, survival interfaces, and low-level SIMD/memory optimization.
- **Version 0.2 — Modernization, Worldbuilding & Deep Systems [Current]**: Focuses on deep worldbuilding content, comprehensive asset pipelines, standardized UI architecture, engine upgrades, flora, interactive blocks, biome overhauls, spatial audio, custom player models, creative tools, advanced physics, visual shaders, and high-performance LOD streaming.

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
| **11** | **Sub-Voxel Block Shaping Tools & Centered Placement** | Developed the real-time block shaping tool (<kbd>R</kbd>/<kbd>T</kbd> keys) supporting slabs, stairs, corner stairs, and centered vertical column voxels. |
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

## Milestone Transition: Version 0.1 → Version 0.2

> [!NOTE]
> **Foundation Complete — Advancing to Version 0.2**  
> Having successfully established the 20 foundational architecture phases of Version 0.1, the engine transitions from structural prototyping into full production worldbuilding, aesthetic refinement, deep survival gameplay systems, and modern platform tooling.

---

## Version 0.2: Modernization, Worldbuilding & Deep Systems

### Phase Directory

| Phase | Milestone Title | Status | Focus / Key Deliverables |
| :---: | :--- | :---: | :--- |
| **01** | **[Full Block Catalog & Texture System Refactoring](#phase-1-full-block-catalog--texture-system-refactoring-completed)** | **Completed** | 720+ worldbuilding block specs, 120+ active blocks, WGSL stochastic rotation, 3-axis directional placement, natural strata variants |
| **02** | **[Standardized 9-Slot Interface Architecture & Menu Modernization](#phase-2-standardized-9-slot-interface-architecture--menu-modernization-completed)** | **Completed** | Unified 9-slot standard, 416×256 px windows, 320×44 px hotbar HUD, unified button suite, Dark Iron slot overlays, clean asset layout |
| **03** | **[Core Engine Upgrade (Bevy 0.19 to Bevy 0.20 Migration)](#phase-3-core-engine-upgrade-bevy-019-to-bevy-020-migration-completed)** | **Completed** | Upgraded to Bevy 0.20, adopted direct `bevy_egui = "0.43"`, eliminated deprecated `Interaction` across all UI, full WESL shader migration, ScreenSpaceTransmission |
| **04** | **[Flora, Crops & Interactive Mechanical Blocks](#phase-4-flora-crops--interactive-mechanical-blocks-planned)** | **Planned** | Multistage crops, wild vegetation, connected fences/walls, functional doors/gates, interactive storage chests, and light fixtures |
| **05** | **[World Generation & Biome Overhaul](#phase-5-world-generation--biome-overhaul-planned)** | **Planned** | Geological rock strata, 2D continuous climate noise, 3D subterranean cave biomes, organic boundary dithering, cliff overhangs |
| **06** | **[System Menus & Cartography Overhaul](#phase-6-system-menus--cartography-overhaul-planned)** | **Planned** | Dark Iron & Parchment menu polish (Pause, Settings, Keybindings), full-screen world map cartography, fog of war, map markers |
| **07** | **[Sound Engine & Spatial Audio System](#phase-7-sound-engine--spatial-audio-system-planned)** | **Planned** | 3D spatial audio, block break/place audio, material-based footsteps, ambient environmental loops, weather sounds, dynamic music |
| **08** | **[Custom Voxel Player Character & Modular Character Creator](#phase-8-custom-voxel-player-character--modular-character-creator-planned)** | **Planned** | Modular voxel character rig, Character Creator interface, fantasy races (Human, Elf, Dwarf, Orc), custom hair, skin, and eyes |
| **09** | **[First-Person Item Rendering & Creative Utility Tools](#phase-9-first-person-item-rendering--creative-utility-tools-planned)** | **Planned** | 3D held items/blocks in player hand, paint roller, lumberjack tree-felling saw, multi-block sledgehammer, tool animations |
| **10** | **[Physics Engine, Fluid Dynamics & Item Drops](#phase-10-physics-engine-fluid-dynamics--item-drops-planned)** | **Planned** | Overhauled fluid propagation, gravity blocks (sand, gravel), 3D floating item drops with magnetism, projectile trajectories |
| **11** | **[Visual Aesthetics, Shaders & Atmospheric FX Overhaul](#phase-11-visual-aesthetics-shaders--atmospheric-fx-overhaul-planned)** | **Planned** | Voxel Ambient Occlusion (Smooth Lighting), water caustics & refraction, bloom, volumetric fog/god rays, weather particles |
| **12** | **[Performance Engineering, Chunk Streaming & LOD Enhancements](#phase-12-performance-engineering-chunk-streaming--lod-enhancements-planned)** | **Planned** | Worker thread scaling, GPU frustum/occlusion culling, smooth LOD horizon transitions, runtime voxel memory compression |

---

### Phase 1: Full Block Catalog & Texture System Refactoring (Completed)

- [x] **Comprehensive 720-Block Worldbuilding Catalog**:
  - Documented complete high-fantasy block catalog across 11 thematic categories (`docs/worldbuilding/blocks_design.md` and `docs/worldbuilding/blocks/*.md`).
  - Standardized 120+ active engine blocks under clean 4-letter prefixes (`terr_`, `rock_`, `wood_`, `deco_`, `liqd_`, `ore_`, `emit_`, `meta_`).
- [x] **Stochastic WGSL Texture Rotation & Flipping**:
  - Implemented fragment-level integer coordinate hashing in `assets/shaders/voxel.wgsl` and `assets/shaders/voxel_opaque.wgsl`.
  - Eliminates visual grid repetition and repetitive tiling across vast cliff walls, cave surfaces, and soil expanses with zero CPU meshing overhead.
- [x] **3-Axis Directional Placement (`AxisAlignedPillar`)**:
  - Implemented in `src/voxel/placement.rs` allowing wood logs, structural columns, and bone pillars to align along X, Y, or Z axes based on raycast hit normal.
- [x] **Natural Strata, Weaves & Grayscale Tinting**:
  - Deployed natural texture variants and grayscale tint masters for blocks including Limestone (`rock_limestone`, `rock_limestone_cobbled`) and Linen (`deco_linen`, `deco_linen_painted`).

---

### Phase 2: Standardized 9-Slot Interface Architecture & Menu Modernization (Completed)

- [x] **Standardized 9-Horizontal-Slot UI Geometry**:
  - Unified all container windows to exact 416×256 px canvas dimensions with consistent 9-slot row grids matching the in-game HUD hotbar.
  - Redesigned gameplay Hotbar HUD to 320×44 px with 9 primary item slots, 2 specialized readout slots (offhand and utility with Dark Iron backdrops), 4 non-clickable wing key badges (`[M]`, `[J]`, `[C]`, `[T]`), static visual meters (XP, HP, Mana), and centered 20×20 px selection frame.
- [x] **Unified Button Component Family**:
  - Created standardized 3-tier button suite (`button_large.png` 96×16 px, `button_medium.png` 64×16 px, `button_small.png` 16×16 px) with normal, hover, and pressed states.
  - Integrated frosted slate-blue metallic highlights (`#3c4552`) across all interactive menus.
- [x] **Dark Iron Specialized Slot Overlays & Clean Asset Layout**:
  - Created 14 bespoke Dark Iron slot overlays in `assets/textures/interfaces/containers/slots/` (Armor Helm, Chest, Legs, Boots; Weapons; Tools; Offhand; Rings; Amulet; Crafting Output; Fuel; Ticking).
  - Purged legacy 8-slot assets and organized UI directories into `containers/`, `containers/slots/`, `containers/outlines/`, `atlases/`, `cursors/`, and `icons/`.
- [x] **Codebase Integration**:
  - Updated container textures and button geometries across `src/menu/creative_inventory.rs`, `src/menu/pause.rs`, `src/menu/settings.rs`, and `src/player/hotbar.rs`.

---

### Phase 3: Core Engine Upgrade (Bevy 0.19 to Bevy 0.20 Migration) (Completed)

- [x] **Bevy 0.20 Dependency & API Migration**:
  - Upgraded engine dependency from Bevy 0.19 to Bevy 0.20 in `Cargo.toml`.
  - Decoupled from `bevy-inspector-egui = "0.37"` and adopted direct `bevy_egui = "0.43"`.
  - Migrated core engine APIs: `compute_aabb()` -> `get_aabb()` in `src/meshing/pipeline.rs`, `Tonemapping` relocated to `bevy::render::view::Tonemapping` in `src/player/mod.rs`, and `#[extract_app(RenderApp)]` on `PostProcessSettings`.
  - Added `bevy::pbr::ScreenSpaceTransmission` to `Camera3d` to preserve water, fluid, and glass specular transmission under Bevy 0.20's opt-in transmission architecture.
- [x] **Full UI Modernization & Deprecation Elimination**:
  - Eliminated deprecated `bevy::ui::Interaction` and `bevy::ui::widget::Button` across `src/menu/creative_inventory.rs`, `src/menu/pause.rs`, `src/menu/settings.rs`, and `src/menu/mod.rs`.
  - Adopted modern `bevy::ui_widgets::Button`, `bevy::picking::hover::Hovered`, and `bevy::ui::Pressed`, guaranteeing zero compiler deprecation warnings across the entire interface codebase.
- [x] **WESL Shader Framework Migration**:
  - Migrated engine shaders to native `.wesl` format (`assets/shaders/voxel_opaque.wesl`, `voxel_transparent.wesl`, `voxel_prepass.wesl`, `post_process.wesl`).
  - Standardized module imports (`import bevy_pbr::render::{...};`, `import bevy_pbr::prepass::io::{...};`, `import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;`).
  - Applied `@group(constants::MATERIAL_BIND_GROUP)` and replaced `#ifdef` preprocessors with `@if(...)` / `@else` attributes.
  - Purged obsolete legacy `.wgsl` files.

---

### Phase 4: Flora, Crops & Interactive Mechanical Blocks (Planned)

- [ ] **Crop Lifecycle & Agricultural Mechanics**:
  - Implement multistage growth cycles (seed -> sprout -> growing -> ripe/harvestable) for agricultural crops (`crop_wheat`, `crop_barley`, `crop_carrot`, `crop_potato`, etc.).
  - Implement farmland hydration mechanics, growth tick schedules, and seed drop harvesting.
- [ ] **Wild Flowers, Vegetation & Fungal Colonies**:
  - Deploy wild flower variants with biome-specific color palettes and harvestable botanical seeds.
  - Add decorative undergrowth: forest ferns, desert shrubs, tundra mosses, hanging cavern vines, and shelf fungi.
- [ ] **Interactive Mechanical & Structural Blocks**:
  - Implement functional doors, trapdoors, and fence gates with distinct open/closed states and interaction sounds.
  - Add connected block logic for wooden fences, stone walls, iron bars, and glass panes.
  - Implement interactive container blocks (chests, barrels, baskets) with lid animations and container UI hooks.
  - Support wall-mounted torches, lantern posts, and decorative hanging fixtures.

---

### Phase 5: World Generation & Biome Overhaul (Planned)

- [ ] **Geological Rock Strata & Mineral Distribution**:
  - Restructure underground generation into realistic geological strata layers (Limestone, Slate, Granite, Basalt, Blackstone, Dreadstone Bedrock).
  - Embed mineral veins, ore clusters, and gem geodes naturally within their corresponding host stone layers.
- [ ] **Continuous Multi-Noise Climate Mapping**:
  - Refactor 2D continentalness, temperature, and moisture noise fields to generate coherent high-fantasy biomes with smooth transitional boundaries.
  - Implement organic biome edge dithering to eliminate harsh rectangular biome borders.
- [ ] **3D Subterranean Cave Biomes**:
  - Generate distinct underground cave environments: lush overgrown caverns, crystal grottos, deep magma chambers, and dripstone chasms.

---

### Phase 6: System Menus & Cartography Overhaul (Planned)

- [ ] **Unified Dark Iron & Parchment System Menus**:
  - Overhaul the in-game Pause Menu, Settings Menu, Audio/Video Options, and Keybindings to strictly adhere to the unified Dark Iron design system.
  - Add smooth menu transition animations and responsive input support for gamepad and keyboard navigation.
- [ ] **Stylized Cartography & World Map Overhaul**:
  - Overhaul the full-screen World Map (<kbd>M</kbd>) with hand-drawn parchment aesthetics, smooth zoom/pan controls, and customizable player map pins.
  - Implement dynamic fog of war revealing terrain as the player explores, along with compass navigation integration.

---

### Phase 7: Sound Engine & Spatial Audio System (Planned)

- [ ] **Audio Pipeline & Category Systems**:
  - Integrate Bevy's audio engine with the categorized audio assets in `assets/sounds/` (`ambience`, `blocks`, `footsteps`, `liquids`, `music`, `weather`).
  - Add independent master, ambient, block, sound effect, and music volume sliders in the Settings Menu.
- [ ] **Block Interaction & Footstep Audio**:
  - Implement material-specific block break, place, and impact audio cues (stone, wood, dirt, sand, gravel, glass, foliage).
  - Implement dynamic 3D spatial footstep sounds based on surface block material, movement stance (walking, sprinting, sneaking), and submersion.
- [ ] **Dynamic Environment, Weather & Background Music**:
  - Add ambient environmental soundscapes (forest wind, cave echoes, shoreline surf, running streams).
  - Implement 3D spatial weather audio (rain intensity, thunderstorm claps, blizzards) and dynamic daytime/nighttime background music playlists.

---

### Phase 8: Custom Voxel Player Character & Modular Character Creator (Planned)

- [ ] **Proprietary Modular Voxel Character Rig**:
  - Retire the legacy Minecraft skin player model in favor of a bespoke, modular voxel humanoid character rig.
  - Update first-person hand meshes, tool holding points, and third-person locomotion animations (walk, run, swim, swing, crouch).
- [ ] **In-Game Character Creator**:
  - Build an interactive Character Creator UI accessible from the main menu and new world setup.
  - Implement playable fantasy races (e.g., Human, Elf, Dwarf, Orc) with distinct physical silhouettes and proportions.
  - Add deep visual customization: selectable hairstyles, hair colors, skin tones, eye colors, facial features, and starting outfits.

---

### Phase 9: First-Person Item Rendering & Creative Utility Tools (Planned)

- [ ] **First-Person & Third-Person Hand Rendering**:
  - Render the actively selected hotbar item or block directly in the player's hand in first-person and third-person views.
  - Add smooth idle hand bobbing, interaction swings on click, and block placement recoil animations.
- [ ] **Specialized Creative & Utility Tools**:
  - **Paint Roller**: Real-time coloring and dyeing tool to paint tintable blocks (Linen, painted wood, plaster, chalk) with selected palette dyes.
  - **Lumberjack Saw / Felling Axe**: Rapid forestry tool that fells an entire connected tree trunk upon breaking the base log, with falling log physics.
  - **Heavy Sledgehammer**: Excavation tool that clears a 3×3 grid of blocks simultaneously, accelerating creative sculpting and terrain clearing.
  - **Architectural Rulers & Measuring Tapes**: Visual distance guides and bounding box overlays for large-scale creative construction.

---

### Phase 10: Physics Engine, Fluid Dynamics & Item Drops (Planned)

- [ ] **Fluid Simulation & Propagation Overhaul**:
  - Overhaul fluid cellular automata mechanics to deliver realistic volume conservation, natural slope cascading, and viscous lava flow.
  - Fix fluid boundary clipping and enhance surface flow animations.
- [ ] **Falling Block & Gravity Mechanics**:
  - Implement dynamic falling physics for unsupported loose blocks (sand, gravel, loose dirt, accumulating snow layers).
  - Add natural slope sliding angles (*angle of repose*) for realistic gravel/sand collapses.
- [ ] **Dynamic 3D Item Drops & Suction Physics**:
  - Spawn floating 3D voxel item drop entities upon block breaking or inventory discarding.
  - Add smooth floating, bobbing, spinning, water buoyancy (floating vs sinking), and player magnetism/suction upon close approach.
- [ ] **Micro-Physics & Projectiles**:
  - Implement physics-driven projectile trajectories for throwable items, arrows, and debris particles with block surface collision.

---

### Phase 11: Visual Aesthetics, Shaders & Atmospheric FX Overhaul (Planned)

- [ ] **Voxel Ambient Occlusion (Smooth Lighting)**:
  - Implement per-vertex ambient occlusion calculating corner and edge block occlusion, adding deep geometric contrast to all surfaces.
- [ ] **Advanced Fluid Shaders & Optical Effects**:
  - Upgrade water and liquid shaders with optical refraction, dynamic water caustics projected onto river/ocean beds, shoreline foam lines, and underwater visual distortion.
- [ ] **Bloom & Emissive Post-Processing**:
  - Integrate HDR bloom post-processing for light-emitting blocks (torches, lanterns, glowing crystals, lava, fireflies, and enchanted ores).
- [ ] **Volumetric Fog & Crepuscular Rays (God Rays)**:
  - Add screen-space or volumetric light shafts piercing through forest foliage, mountain ridges, and cavern ceilings.
- [ ] **Atmospheric Weather Particles & Biome Color Grading**:
  - Implement particle weather effects (rain with block surface splashing, snow flurries, falling autumn leaves, embers near lava).
  - Implement biome-specific tonemapping and color grading LUTs for tailored environmental atmospheres (e.g. moody caverns vs vibrant sunlit plains).

---

### Phase 12: Performance Engineering, Chunk Streaming & LOD Enhancements (Planned)

- [ ] **Streaming Throughput & Worker Scaling**:
  - Optimize asynchronous compute worker pools for chunk generation and greedy meshing with dynamic per-frame upload budgets.
- [ ] **Advanced Level-of-Detail (LOD) & Occlusion**:
  - Refine multi-tier chunk mesh LOD transitions to eliminate visible terrain pop-in at high render distances (32+ chunks).
  - Implement GPU-driven frustum culling and occlusion queries to avoid rendering occluded subterranean geometry.
- [ ] **Voxel Memory Footprint & Disk Streaming**:
  - Compress runtime voxel storage structures to maximize cache locality and reduce memory consumption per loaded chunk.
  - Accelerate binary region-based world serialization for seamless disk save and load streaming.
