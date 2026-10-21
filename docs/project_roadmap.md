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
| **02** | **[Standardized 9-Slot Interface Architecture & Menu Modernization](#phase-2-standardized-9-slot-interface-architecture--menu-modernization-in-progress)** | **In Progress** | Unified 9-slot standard, 416×256 px windows, 320×44 px hotbar HUD, unified button suite, Dark Iron slot overlays, 480×270 integer scaling |
| **03** | **[Core Engine Upgrade (Bevy 0.19 to Bevy 0.20 Migration)](#phase-3-core-engine-upgrade-bevy-019-to-bevy-020-migration-completed)** | **Completed** | Upgraded to Bevy 0.20, adopted direct `bevy_egui = "0.43"`, eliminated deprecated `Interaction` across all UI, full WESL shader migration, ScreenSpaceTransmission |
| **04** | **[Architecture, Graphics Modernization & Soundness](#phase-4-architecture-graphics-modernization--soundness-completed)** | **Completed** | Hardware profiles, chunk memory optimization, monolithic decomposition, render scale, 16:9 snap, presets, tests |
| **05** | **[High-Impact Performance & Graphics Pipeline](#phase-5-high-impact-performance--graphics-pipeline-completed)** | **Completed** | 5-Step Target FPS Limiter (30/60/120/144/Unlocked), Preset Chunk Mesh Upload Budget, FSR 1.0 Upscaling Filter, Subterranean Culling |
| **06** | **[Flora, Crops & Interactive Mechanical Blocks](#phase-6-flora-crops--interactive-mechanical-blocks-in-progress)** | **In Progress** | Multistage crops, hydration simulation, connected fences/walls/panes bitwise logic, functional doors/gates, containers, and fixtures |
| **07** | **[World Generation & Biome Overhaul](#phase-7-world-generation--biome-overhaul-planned)** | **Planned** | Geological rock strata, 2D continuous climate noise, 3D subterranean cave biomes, organic boundary dithering, cliff overhangs |
| **08** | **[System Menus & Cartography Overhaul](#phase-8-system-menus--cartography-overhaul-planned)** | **Planned** | Dark Iron & Parchment menu polish (Pause, Settings, Keybindings), full-screen world map cartography, fog of war, map markers |
| **09** | **[Sound Engine & Spatial Audio System](#phase-9-sound-engine--spatial-audio-system-planned)** | **Planned** | 3D spatial audio, block break/place audio, material-based footsteps, ambient environmental loops, weather sounds, dynamic music |
| **10** | **[Custom Voxel Player Character & Modular Character Creator](#phase-10-custom-voxel-player-character--modular-character-creator-planned)** | **Planned** | Modular voxel character rig, Character Creator interface, fantasy races (Human, Elf, Dwarf, Orc), custom hair, skin, and eyes |
| **11** | **[First-Person Item Rendering & Creative Utility Tools](#phase-11-first-person-item-rendering--creative-utility-tools-planned)** | **Planned** | 3D held items/blocks in player hand, paint roller, lumberjack tree-felling saw, multi-block sledgehammer, tool animations |
| **12** | **[Physics Engine, Fluid Dynamics & Item Drops](#phase-12-physics-engine-fluid-dynamics--item-drops-planned)** | **Planned** | Overhauled fluid propagation, gravity blocks (sand, gravel), 3D floating item drops with magnetism, projectile trajectories |
| **13** | **[Visual Aesthetics, Shaders & Atmospheric FX Overhaul](#phase-13-visual-aesthetics-shaders--atmospheric-fx-overhaul-planned)** | **Planned** | Voxel Ambient Occlusion (Smooth Lighting), water caustics & refraction, bloom, volumetric fog/god rays, weather particles |
| **14** | **[Performance Engineering, Chunk Streaming & LOD Enhancements](#phase-14-performance-engineering-chunk-streaming--lod-enhancements-planned)** | **Planned** | Worker thread scaling, GPU frustum/occlusion culling, smooth LOD horizon transitions, runtime voxel memory compression |

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

### Phase 2: Standardized 9-Slot Interface Architecture & Menu Modernization (In Progress)

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
- [ ] **Unified 480×270 Base UI Redesign & Integer Scaling**:
  - Standardize all container, inventory, and HUD layouts onto a fixed 480×270 px (16:9) base canvas, scaled up to 4× with integer pixel-perfect alignment.

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

### Phase 4: Architecture, Graphics Modernization & Soundness (Completed)

- [x] **Multi-Machine Hardware Profiles & Calibrated Presets**:
  - Implemented `LocalConfig` in `src/core/config.rs` with automatic first-boot creation of `local_config.json` (isolated from Git).
  - Added 5 calibrated performance profiles (`Potato`, `Low`, `Medium`, `High`, `Ultra`) calibrating render distance, LOD distance, simulation distance, shadows, clouds, full grass, anti-aliasing, and render scale.
- [x] **Chunk Memory Footprint Optimization**:
  - Refactored `src/world/chunk.rs` to compute variant distributions lazily, reducing the inline struct footprint of `Chunk` by ~92% (from 2,192 bytes down to 144 bytes).
- [x] **Voxel Enum Soundness & Safe Transmutation**:
  - Eliminated unsafe memory transmutation risks in `Voxel::from_u16` across all 723 variants, with exhaustive discriminant validation tests.
  - Resolved `BlockShape` radial menu overlapping bug via `BlockShape::shapable()`.
- [x] **Monolithic Block Decomposition**:
  - Modularized the 3,763-line `src/world/block.rs` monolith into clean, cohesive submodules under `src/world/block/` (`catalog.rs`, `connection.rs`, `data.rs`, `props.rs`, `shape.rs`, `voxel.rs`, `mod.rs`).
- [x] **Pure Architectural Test Isolation**:
  - Decoupled engine code into `src/lib.rs` and `src/main.rs`.
  - Removed 100% of `#[cfg(test)]` blocks from `src/`, establishing a unified single-binary integration test suite in `tests/integration_tests.rs` with modular domain suites (`tests/suite/`).
- [x] **Dynamic 3D Render Scale & Native 2D UI Overlay**:
  - Implemented dynamic 3D Render Scale (`src/core/render_scale.rs`) routing `Camera3d` to an off-screen render texture with hardware-accelerated bilinear upscaling (50%, 66%, 75%, 85%, 100%).
  - UI and HUD retain 100% native display crispness. At 100% native scale, offscreen overhead is automatically bypassed.
- [x] **Debounced 16:9 Aspect Ratio Window Snapping**:
  - Implemented 16:9 aspect ratio window snapping (`src/core/window_aspect.rs`) for windowed mode with minimum constraints (640×360 px).
  - Engineered startup grace period and drag-settle debounce timer (250ms) to ensure DirectX 12 / WGPU swapchain buffers never experience re-entrant resize collisions (`0x887A0001`).
- [x] **Hardware Anti-Aliasing Quality Modes**:
  - Added configurable anti-aliasing (`AntiAliasingMode`: `Off`, `Fxaa`, `Msaa2x`, `Msaa4x`), dynamically configuring camera MSAA sample counts and FXAA post-processing passes.
- [x] **Directional Shadow Quality & Cascade Configuration**:
  - Implemented directional shadow cascade control (`ShadowQuality`: `Off`, `Low`, `Medium`, `High`, `Ultra`), adjusting shadow map resolution and 1 to 4 cascade split levels on `SunLight` and `MoonLight`.
- [x] **Dual-Mode Cloud Rendering System**:
  - Implemented dual-mode clouds (`CloudQuality`: `Off`, `Fast2D`, `Fancy3D`), offering flat planar dual-face quads with 70% vertex reduction on lower tiers, and full volumetric 3D blocks on higher tiers.
- [x] **Dynamic FOV Kick & Motion Accessibility Toggle**:
  - Integrated dynamic sprint FOV kick (+8° FOV expansion during sprint) with an accessibility toggle (`dynamic_fov_enabled`) for motion-sensitive players.

---

### Phase 5: High-Impact Performance & Graphics Pipeline (Completed)

- [x] **5-Step Target Frame Rate Limiter (`TargetFpsLimit` / `DynamicFpsSettings`)**:
  - Implemented 5 calibrated display refresh steps: `30 FPS` (battery / entry laptops), `60 FPS` (standard 60Hz), `120 FPS` (high refresh), `144 FPS` (esports standard), and `Unlocked` (unconstrained).
  - Decoupled frame pacing from VSync in `dynamic_fps.rs`, preventing CPU/GPU thermal throttling, fan noise, and excessive battery drain on low-spec laptops even with VSync disabled or on high-refresh panels.
  - Interactive stepper row exposed in the in-game Settings menu (`SettingsAction::DecTargetFps` / `IncTargetFps`), saving to `local_config.json` and adjusting pacing in real time.
- [x] **Preset-Calibrated Chunk Mesh Upload Budgeting**:
  - Integrated per-frame vertex/index buffer GPU transfer throttling in `ChunkStreamingManager` and `async_mesher.rs`, scaling dynamically by performance preset:
    - `Potato`: 3 chunk mesh uploads per frame (eliminates PCIe bus saturation and micro-stutter on weak integrated graphics / laptops).
    - `Low`: 6 chunk mesh uploads per frame.
    - `Medium`: 12 chunk mesh uploads per frame.
    - `High`: 24 chunk mesh uploads per frame.
    - `Ultra`: 64 chunk mesh uploads per frame for instant high-end desktop world streaming.
- [x] **AMD FidelityFX Super Resolution 1.0 (FSR 1.0) Upscaling Filter**:
  - Integrated selectable upscaling reconstruction filter options in `LocalConfig` and `RenderScalePlugin`: `Bilinear` (ultra-fast baseline interpolation for potato laptops) vs `FSR 1.0` (Spatial Edge-Adaptive Filtering + RCAS Robust Contrast-Adaptive Sharpening).
  - Pairs with 3D render scales (`0.50x`, `0.66x`, `0.75x`, `0.85x`) to restore sharp geometric voxel edges and contrast without ringing or haloing.
  - Interactive stepper row exposed directly beneath Render Scale in the in-game Settings menu (`SettingsAction::DecUpscalingFilter` / `IncUpscalingFilter`).
- [x] **Subterranean Cave Occlusion Face Culling**:
  - Implemented surface-to-underground occlusion mask checks, culling complete chunk meshes and interior cavern faces occluded by solid surface terrain when the player is on the surface.
  - Integrated real-time underground state detection (`is_position_underground`) casting vertical occlusion rays and checking subterranean chunk elevation.
  - Enhanced chunk graph traversal (Sodium architecture) to halt horizontal propagation into subterranean cave chambers from surface openings, preventing cave networks from illuminating under solid ground.
  - Added real-time chunk and vertex culling metrics (`rendered_vertices`, `culled_vertices`, `rendered_chunks`, `culled_chunks`) and exposed live culling environment telemetry in the F3 HUD.
  - Validated with automated unit tests in `tests/suite/culling.rs`.

---

### Phase 6: Flora, Crops & Interactive Mechanical Blocks (In Progress)

- [x] **Crop Lifecycle & Agricultural Mechanics (Pure Simulation Foundation)**:
  - Implemented pure simulation data models (`CropType`, `CropStage`, `SoilMoisture`, `CropSimulator` in `src/simulation/crops.rs`) with Chebyshev water hydration distance (4-block radius), weather hydration (rain), growth tick chance evaluation, and harvest yields (`CropHarvestResult`).
  - Validated with automated unit tests in `tests/suite/crops.rs`.
- [ ] **Crop In-World Meshing & Block Placement Hooks**:
  - Connect crop growth stages to world block ticks and visual billboard rendering models.
- [x] **Connected Block Algorithmic Logic (Pure Topology Masks)**:
  - Implemented bitwise connection topology (`ConnectionMask` and `ConnectedCategory` in `src/world/block/connection.rs`) calculating 4-way cardinal adjacency and 6-way adjacency for fences, walls, glass panes, and iron bars.
  - Recognizes isolated posts, straight lines (N-S, E-W), perpendicular L-corners, 3-way T-junctions, and 4-way intersections.
  - Validated with automated unit tests in `tests/suite/connection.rs`.
- [ ] **Wild Flowers, Vegetation & Fungal Colonies**:
  - Deploy wild flower variants with biome-specific color palettes and harvestable botanical seeds.
  - Add decorative undergrowth: forest ferns, desert shrubs, tundra mosses, hanging cavern vines, and shelf fungi.
- [ ] **Interactive Mechanical & Structural Blocks**:
  - Implement functional doors, trapdoors, and fence gates with distinct open/closed states and interaction sounds.
  - Implement interactive container blocks (chests, barrels, baskets) with lid animations and container UI hooks.
  - Support wall-mounted torches, lantern posts, and decorative hanging fixtures.

---

### Phase 7: World Generation & Biome Overhaul (Planned)

- [ ] **Geological Rock Strata & Mineral Distribution**:
  - Restructure underground generation into realistic geological strata layers (Limestone, Slate, Granite, Basalt, Blackstone, Dreadstone Bedrock).
  - Embed mineral veins, ore clusters, and gem geodes naturally within their corresponding host stone layers.
- [ ] **Continuous Multi-Noise Climate Mapping**:
  - Refactor 2D continentalness, temperature, and moisture noise fields to generate coherent high-fantasy biomes with smooth transitional boundaries.
  - Implement organic biome edge dithering to eliminate harsh rectangular biome borders.
- [ ] **3D Subterranean Cave Biomes**:
  - Generate distinct underground cave environments: lush overgrown caverns, crystal grottos, deep magma chambers, and dripstone chasms.

---

### Phase 8: System Menus & Cartography Overhaul (Planned)

- [ ] **Unified Dark Iron & Parchment System Menus**:
  - Overhaul the in-game Pause Menu, Settings Menu, Audio/Video Options, and Keybindings to strictly adhere to the unified Dark Iron design system.
  - Add smooth menu transition animations and responsive input support for gamepad and keyboard navigation.
- [ ] **Stylized Cartography & World Map Overhaul**:
  - Overhaul the full-screen World Map (<kbd>M</kbd>) with hand-drawn parchment aesthetics, smooth zoom/pan controls, and customizable player map pins.
  - Implement dynamic fog of war revealing terrain as the player explores, along with compass navigation integration.

---

### Phase 9: Sound Engine & Spatial Audio System (Planned)

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

### Phase 10: Custom Voxel Player Character & Modular Character Creator (Planned)

- [ ] **Proprietary Modular Voxel Character Rig**:
  - Retire the legacy Minecraft skin player model in favor of a bespoke, modular voxel humanoid character rig.
  - Update first-person hand meshes, tool holding points, and third-person locomotion animations (walk, run, swim, swing, crouch).
- [ ] **In-Game Character Creator**:
  - Build an interactive Character Creator UI accessible from the main menu and new world setup.
  - Implement playable fantasy races (e.g., Human, Elf, Dwarf, Orc) with distinct physical silhouettes and proportions.
  - Add deep visual customization: selectable hairstyles, hair colors, skin tones, eye colors, facial features, and starting outfits.

---

### Phase 11: First-Person Item Rendering & Creative Utility Tools (Planned)

- [ ] **First-Person & Third-Person Hand Rendering**:
  - Render the actively selected hotbar item or block directly in the player's hand in first-person and third-person views.
  - Add smooth idle hand bobbing, interaction swings on click, and block placement recoil animations.
- [ ] **Specialized Creative & Utility Tools**:
  - **Paint Roller**: Real-time coloring and dyeing tool to paint tintable blocks (Linen, painted wood, plaster, chalk) with selected palette dyes.
  - **Lumberjack Saw / Felling Axe**: Rapid forestry tool that fells an entire connected tree trunk upon breaking the base log, with falling log physics.
  - **Heavy Sledgehammer**: Excavation tool that clears a 3×3 grid of blocks simultaneously, accelerating creative sculpting and terrain clearing.
  - **Architectural Rulers & Measuring Tapes**: Visual distance guides and bounding box overlays for large-scale creative construction.

---

### Phase 12: Physics Engine, Fluid Dynamics & Item Drops (Planned)

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

### Phase 13: Visual Aesthetics, Shaders & Atmospheric FX Overhaul (Planned)

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

### Phase 14: Performance Engineering, Chunk Streaming & LOD Enhancements (Planned)

- [ ] **Streaming Throughput & Worker Scaling**:
  - Optimize asynchronous compute worker pools for chunk generation and greedy meshing with dynamic per-frame upload budgets.
- [ ] **Advanced Level-of-Detail (LOD) & Occlusion**:
  - Refine multi-tier chunk mesh LOD transitions to eliminate visible terrain pop-in at high render distances (32+ chunks).
  - Implement GPU-driven frustum culling and occlusion queries to avoid rendering occluded subterranean geometry.
- [ ] **Voxel Memory Footprint & Disk Streaming**:
  - Compress runtime voxel storage structures to maximize cache locality and reduce memory consumption per loaded chunk.
  - Accelerate binary region-based world serialization for seamless disk save and load streaming.
