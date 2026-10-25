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

## Milestone Transition: Version 0.1 → Version 0.2

> [!NOTE]
> **Foundation Complete — Advancing to Version 0.2**  
> Having successfully established the 20 foundational architecture phases of Version 0.1, the engine transitions from structural prototyping into full production worldbuilding, aesthetic refinement, deep survival gameplay systems, and modern platform tooling.

---

## Version 0.2: Modernization, Worldbuilding & Deep Systems

### Phase Directory

| Phase | Milestone Title | Status | Focus / Key Deliverables |
| :---: | :--- | :---: | :--- |
| **01** | **[Full Block Catalog & Geological Systems Foundation](#phase-1-full-block-catalog--geological-systems-foundation-completed)** | **Completed** | 615 active engine voxels, geological strata catalog, 3-axis directional placement, selective foliage tinting, shader de-tiling |
| **02** | **[Standardized 9-Slot Interface Architecture & Menu Modernization](#phase-2-standardized-9-slot-interface-architecture--menu-modernization-completed)** | **Completed** | Standardized 9-slot container and HUD geometry, Dark Iron design suite, 480×270 base integer scaling |
| **03** | **[Core Engine Upgrade (Bevy 0.19 to Bevy 0.20 Migration)](#phase-3-core-engine-upgrade-bevy-019-to-bevy-020-migration-completed)** | **Completed** | Bevy 0.20 migration, direct `bevy_egui = "0.43"`, modernized UI widgets, native WESL shader framework, ScreenSpaceTransmission |
| **04** | **[Architecture, Graphics Modernization & Soundness](#phase-4-architecture-graphics-modernization--soundness-completed)** | **Completed** | Multi-tier hardware profiles, chunk memory optimization, modular domain crates, dynamic 3D render scaling, 16:9 window snapping |
| **05** | **[High-Impact Performance & Graphics Pipeline](#phase-5-high-impact-performance--graphics-pipeline-completed)** | **Completed** | Dynamic frame rate limiter, hardware GPU upload budgets, AMD FSR 1.0 upscaling, subterranean occlusion culling |
| **06** | **[Flora, Crops & Interactive Mechanical Blocks](#phase-6-flora-crops--interactive-mechanical-blocks-in-progress)** | **In Progress** | Agricultural lifecycle simulation, in-world crop growth, connected block topology masks, wild flora, interactive doors and containers |
| **07** | **[Subterranean Strata, Caverns & Geological Generation](#phase-7-subterranean-strata-caverns--geological-generation-in-progress)** | **In Progress** | Multi-tier geological strata, Mantle bedrock at Y = -254, 54 authentic ore vein pairings, crystal geodes, subterranean cave biomes |
| **08** | **[Biome Refactoring, Climate Modeling & Tree/Flora Distribution](#phase-8-biome-refactoring-climate-modeling--treeflora-distribution-planned)** | **Planned** | Multi-noise climate fields (continentalness/temperature/moisture), organic biome dithering, ecological tree dispersion, canopy models |
| **09** | **[Main Menu & World Management](#phase-9-main-menu--world-management-planned)** | **Planned** | Boot to Main Menu title screen, New World creation (seed & name), saved world browser, persistence & save triggers, quit |
| **10** | **[Sound Engine & Spatial Audio System](#phase-10-sound-engine--spatial-audio-system-in-progress)** | **In Progress** | 3D spatial audio, material-based block break/place and footsteps, ambient environmental soundscapes, weather audio, dynamic music |
| **11** | **[Custom Voxel Player Character & Modular Character Creator](#phase-11-custom-voxel-player-character--modular-character-creator-planned)** | **Planned** | Modular voxel character rig, Character Creator interface, fantasy races, custom hair, skin, eye, and cosmetic customization |
| **12** | **[First-Person Item Rendering & Creative Utility Tools](#phase-12-first-person-item-rendering--creative-utility-tools-planned)** | **Planned** | 3D held items/blocks in player hand, paint roller, lumberjack tree-felling saw, multi-block sledgehammer, measurement tools |
| **13** | **[Physics Engine, Fluid Dynamics & Item Drops](#phase-13-physics-engine-fluid-dynamics--item-drops-planned)** | **Planned** | Overhauled fluid propagation, gravity blocks (sand, gravel), 3D floating item drops with magnetism, projectile trajectories |
| **14** | **[Dynamic Weather System & Meteorological Phenomena](#phase-14-dynamic-weather-system--meteorological-phenomena-completed)** | **Completed** | Multi-state weather simulation (rain, snow, fog, storms), precipitation particles, surface wetness, dynamic snow accumulation |
| **15** | **[Visual Aesthetics, Shaders & Atmospheric FX Overhaul](#phase-15-visual-aesthetics-shaders--atmospheric-fx-overhaul-planned)** | **Planned** | Voxel Ambient Occlusion (Smooth Lighting), water caustics & refraction, bloom, volumetric fog/god rays, biome color grading |
| **16** | **[Performance Engineering, Chunk Streaming & LOD Enhancements](#phase-16-performance-engineering-chunk-streaming--lod-enhancements-planned)** | **Planned** | Worker thread scaling, GPU frustum/occlusion culling, smooth LOD horizon transitions, runtime voxel memory compression |
| **17** | **[Localization & Multi-Language Support](#phase-17-localization--multi-language-support-completed)** | **Completed** | `lang/` translation files (`en_us`, `pt_br`), comprehensive UI/block/item localization, natural Portuguese grammar, in-game language selector |
| **18** | **[Controls & Keybinding System](#phase-18-controls--keybinding-system-completed)** | **Completed** | Configurable keybindings for all actions, mouse sensitivity & invert Y options, input conflict detection, Main Menu & Pause integration |
| **19** | **[Dynamic Foliage & Procedural Grass System](#phase-19-dynamic-foliage--procedural-grass-system-completed)** | **Completed** | Procedural 3D grass blades on surface blocks, cubic Bezier wind deformation, Voronoi clumping, entity footprint trails, dynamic LOD |

---

### Phase 1: Full Block Catalog & Geological Systems Foundation (Completed)

- [x] **615-Block Engine Voxel Architecture & Geological Catalog**:
  - Unified catalog of 615 active voxel variants across engine registries, greedy meshing pipelines, and runtime properties.
  - Comprehensive geological foundation comprising 10 host mother rocks, 20 minerals and gemstones, 54 authentic ore vein pairings, subterranean crystal geodes, and indestructible Mantle bedrock.
- [x] **3-Axis Directional Block Placement**:
  - Universal orientation (Vertical Y, Horizontal X, Horizontal Z) for structural blocks, all tree log species, wood bark variants, organic pillars, and agricultural bales.
  - Dynamic face UV mapping ensuring consistent grain continuity and endcap orientation across all placement axes in greedy meshing.
- [x] **Selective Biome Foliage Tinting**:
  - Biome-driven foliage color mapping applied strictly to living grayscale canopy species, preserving natural intrinsic pigments for seasonal, dead, and evergreen foliage.
- [x] **Procedural 2D & Handheld Item Rendering Pipeline**:
  - Real-time procedural 2D icon generation for cubic blocks, planar cross-billboards, animated fluid frames, and handheld items across container interfaces.
- [x] **Stochastic WESL Texture Rotation & De-tiling**:
  - Fragment-level coordinate hashing in the opaque voxel shader, breaking repetitive grid patterns on cliffs, cavern walls, and terrain surfaces with zero CPU meshing overhead.
- [x] **Extended Block Shapes, Cavity Meshing & Shape Locking**:
  - Expanded native engine block shapes to 11 variants, introducing `BlockShape::Cocoon` ($0.75\text{ m} \times 0.875\text{ m}$) for Spider Eggs and Silk Cocoons.
  - Runtime shape-locking protection (`is_shape_locked()`) preventing illegal deformation or rotation on decorative nests, eggs, cocoons, and custom-meshed blocks.
  - Basket interior cavity meshing with precise 2px rims, interior floor quad, and 4 inward-facing side walls.
  - Consolidated Crate as a 100% solid, fully textured opaque wooden container block.
  - Player-facing front orientation for contextual blocks (Bookshelves, Bee Nests, Wasp Nests) and natural horizontal-only rotation constraints for surface pebbles/rocks.

---

### Phase 2: Standardized 9-Slot Interface Architecture & Menu Modernization (Completed)

- [x] **Standardized 9-Slot Container & HUD Geometry**:
  - Unified 9-slot row grid architecture across all container windows and gameplay hotbar HUD, standardizing inventory navigation and equipment readouts with 3px slot spacing.
- [x] **Unified Dark Iron Interface Component Suite**:
  - Standardized multi-tier interactive button suite (`button_large.png`, `button_medium.png`, `button_small.png`, `button_util.png`) and contextual equipment slot overlays across all menus.
- [x] **Unified 480×270 Base UI & Integer Scaling**:
  - Standardized all container, inventory, and HUD layouts onto a fixed 480×270 px (16:9) base canvas, supporting integer pixel-perfect scaling up to 4× across display resolutions (`GUI_SCALE = 4.0`).
  - Integrated Dark Iron Pause Menu Bar architecture with centralized 4-tab navigation: Map, Journal, Inventory, and System.
  - Organized container assets and outlines into structured subdirectories (`interfaces/containers/`, `buttons/`, `outlines/`).
  - Consolidated Hotbar HUD with health bar integration, exact 12px numeric slot font parity, and centered selection tooltips.
  - Procedural 2D/3D icon rendering pipeline with directional front-face projection (bookshelves, hives, nests, bags), planar cross-mesh billboards (plants, beaver dams, corals), and animated texture strips.
- [x] **Configurable GUI Scaling & Interface Polish**:
  - Interactive GUI Scale selector (2×, 3×, 4×) in Settings, resizing container inventories, world map, journal, and system menus dynamically while preserving unscaled top menu bar navigation.
  - Multi-tier Menu Depth-of-Field blur intensity (`Off`, `Low`, `Medium`, `High`, `Ultra`) and dynamic background dimming (`Off`, `Light`, `Medium`, `Dark`, `Heavy`) across all open UI views.

---

### Phase 3: Core Engine Upgrade (Bevy 0.19 to Bevy 0.20 Migration) (Completed)

- [x] **Bevy 0.20 & Direct Egui Migration**:
  - Upgraded engine core to Bevy 0.20 and direct `bevy_egui = "0.43"`, migrating rendering pipelines and adding `ScreenSpaceTransmission` for fluid/glass specular refraction.
- [x] **UI Widget Modernization**:
  - Modernized interface event handling with retained widget components (`Hovered`, `Pressed`, `Button`), eliminating deprecated UI interaction patterns across all menus.
- [x] **Native WESL Shader Framework Adoption**:
  - Fully transitioned shader architecture from legacy WGSL preprocessors to modular WESL shaders (`voxel_opaque.wesl`, `voxel_transparent.wesl`, `voxel_prepass.wesl`, `post_process.wesl`).

---

### Phase 4: Architecture, Graphics Modernization & Soundness (Completed)

- [x] **Multi-Tier Hardware Profiles & Quality Calibration**:
  - Automated hardware config (`Potato` to `Ultra`) managing render scale, shadow cascades, dual-mode clouds, anti-aliasing (FXAA/MSAA), and view distances.
- [x] **Engine Soundness & Memory Footprint Optimization**:
  - Lazy chunk variant distribution reducing memory footprint by ~92%, safe discriminant validation, and decomposition into modular domain crates with pure integration test isolation.
- [x] **Dynamic 3D Render Scaling & Window Presentation**:
  - Decoupled 3D resolution scaling with native 2D interface crispness, and debounced 16:9 window snapping eliminating swapchain resize collisions.
- [x] **Display Initialization & Multi-Monitor Window Management**:
  - Anchored window creation to `WindowPosition::Centered(MonitorSelection::Primary)` and updated Borderless/Exclusive Fullscreen resolution to target the Primary display, eliminating multi-monitor winit detection warnings on Windows startup.
  - Preserved custom preset persistence (`preset: "Custom"`) and calendar time-flow state (`is_time_paused`) in local configuration.
  - Corrected placement pop feedback animation to respect non-full block geometries (slabs/nests) and fixed painted block tint latency during placement bounce.

---

### Phase 5: High-Impact Performance & Graphics Pipeline (Completed)

- [x] **Dynamic Display Frame Pacing & Limiter**:
  - Calibrated target frame rate limiter (30, 60, 120, 144, Unlocked) decoupled from VSync, preventing thermal throttling and excessive power consumption.
- [x] **Hardware-Scaled Chunk Mesh GPU Upload Budgeting**:
  - Per-frame vertex and index buffer upload limits dynamically scaled by hardware profile, eliminating PCIe saturation and frame stutter during rapid world streaming.
- [x] **AMD FidelityFX Super Resolution (FSR 1.0) Pipeline**:
  - Spatial edge-adaptive filtering and contrast-adaptive sharpening (RCAS) paired with 3D render scaling for sharp voxel edges on lower-spec hardware.
- [x] **Subterranean Cave Occlusion Face Culling**:
  - Surface-to-underground occlusion masking and directional chunk graph traversal culling occluded subterranean cave chambers when the player is on the surface.

---

### Phase 6: Flora, Crops & Interactive Mechanical Blocks (In Progress)

- [x] **Agricultural Simulation Engine**:
  - Simulation model for crop lifecycles, soil moisture propagation, rainfall hydration, and harvest yield evaluation.
- [ ] **In-World Crop Meshing & Growth Integration**:
  - Connect crop growth stages to world block ticks, seasonal growth rates, and visual stage models.
- [x] **Connected Block Topology Logic**:
  - Bitwise connection solver evaluating 4-way and 6-way adjacency for fences, walls, glass panes, and iron bars.
- [ ] **Wild Botanical Flora & Fungal Colonies**:
  - Biome-specific wildflowers, harvestable seeds, undergrowth scatter (ferns, shrubs, mosses), and subterranean fungi.
- [ ] **Interactive Mechanical & Structural Blocks**:
  - Functional doors, trapdoors, and fence gates with distinct states, container blocks (chests, barrels) with lid animations, and wall-mounted fixtures.

---

### Phase 7: Subterranean Strata, Caverns & Geological Generation (In Progress)

- [x] **Geological Strata Generation Architecture**:
  - Multi-tier vertical strata system replacing uniform filler stone with realistic geological depth layers (Surface, Upper, Middle, and Deep tectonic crusts) bounded by indestructible Mantle bedrock at Y = -254.
- [x] **Subterranean Mineral & Ore Vein Matrix**:
  - Procedural distribution of 54 authentic host-rock mineral pairings across 10 mother rocks, alongside subterranean crystal geodes with basalt/calcite protective shells.
- [ ] **3D Subterranean Cave Biomes & Unique Caverns**:
  - Distinct subterranean environments including karst caverns, flooded calcite grottoes, dripstone chasms, and volcanic lava tube corridors.

---

### Phase 8: Biome Refactoring, Climate Modeling & Tree/Flora Distribution (Planned)

- [ ] **Continuous Multi-Noise Climate Modeling**:
  - Multi-noise climate fields (continentalness, temperature, moisture) generating coherent macro-biomes with organic edge dithering and transitional micro-climates.
- [ ] **Ecological Tree & Flora Distribution**:
  - Climate-driven procedural tree generation and canopy density modeling across tropical, temperate, boreal, and montane environments, accompanied by fallen logs and natural undergrowth scatter.

---

### Phase 9: Main Menu & World Management (Planned)

- [ ] **Boot Flow Transition & Title Screen Interface**:
  - Re-architect engine startup flow to boot into a dedicated Main Menu / Title Screen instead of spawning directly into active gameplay.
  - Implement Dark Iron visual design suite for the title screen with animated background panorama, logo banner, version indicator, and navigation buttons (*Play*, *Settings*, *Controls*, *Credits*, *Quit*).
- [ ] **New World Creation Interface**:
  - Build a world creation modal allowing players to define World Name, custom World Seed (with seed randomizer), and initial game mode (Survival / Creative).
  - Add procedural generation seed validation and world folder initialization.
- [ ] **Saved Worlds Selection & Loading Architecture**:
  - Implement saved world list UI that queries local save directories (`saves/`).
  - Display rich save metadata: world name, seed, last played timestamp, game mode, and file size.
  - Enable loading, duplicating, and deleting world saves safely with confirmation prompts.
- [ ] **World Persistence & Session Lifecycle**:
  - Connect in-game Pause Menu (*Save and Return to Title Screen*, *Quit to Desktop*) to world state serialization routines.
  - Graceful world unload releasing voxel chunks, simulation threads, and audio channels before returning to the main menu.

---

### Phase 10: Sound Engine & Spatial Audio System (In Progress)

- [x] **Audio Pipeline & Channel Architecture**:
  - Multi-channel audio engine (`AudioSettings`, `EngineAudioPlugin`) with independent volume channels for master, weather, sound effects (SFX), music, and ambient murmurs, integrated with in-game Settings UI and persistent `local_config.json`.
- [ ] **Block Interaction & Footstep Audio**:
  - Implement material-specific block break, place, and impact audio cues (stone, wood, dirt, sand, gravel, glass, foliage).
  - Implement dynamic 3D spatial footstep sounds based on surface block material, movement stance (walking, sprinting, sneaking), and submersion.
- [x] **Dynamic Environment, Weather & Background Music**:
  - Add ambient environmental soundscapes and multi-channel weather audio (rain intensity, wind gusts, snow murmur, sandstorms, physical speed-of-sound thunderclaps).
  - Implement dynamic gameplay background music soundtrack player (`GameplayMusicPlugin`) with seamless cross-fading and organic pauses across all 39 game music tracks.

---

### Phase 11: Custom Voxel Player Character & Modular Character Creator (Planned)

- [ ] **Proprietary Modular Voxel Character Rig**:
  - Retire the legacy Minecraft skin player model in favor of a bespoke, modular voxel humanoid character rig.
  - Update first-person hand meshes, tool holding points, and third-person locomotion animations (walk, run, swim, swing, crouch).
- [ ] **In-Game Character Creator**:
  - Build an interactive Character Creator UI accessible from the main menu and new world setup.
  - Implement playable fantasy races (e.g., Human, Elf, Dwarf, Orc) with distinct physical silhouettes and proportions.
  - Add deep visual customization: selectable hairstyles, hair colors, skin tones, eye colors, facial features, and starting outfits.

---

### Phase 12: First-Person Item Rendering & Creative Utility Tools (Planned)

- [ ] **First-Person & Third-Person Hand Rendering**:
  - Render the actively selected hotbar item or block directly in the player's hand in first-person and third-person views.
  - Add smooth idle hand bobbing, interaction swings on click, and block placement recoil animations.
- [ ] **Specialized Creative & Utility Tools**:
  - **Paint Roller**: Real-time coloring and dyeing tool to paint tintable blocks (Linen, painted wood, plaster, chalk) with selected palette dyes.
  - **Lumberjack Saw / Felling Axe**: Rapid forestry tool that fells an entire connected tree trunk upon breaking the base log, with falling log physics.
  - **Heavy Sledgehammer**: Excavation tool that clears a 3×3 grid of blocks simultaneously, accelerating creative sculpting and terrain clearing.
  - **Architectural Rulers & Measuring Tapes**: Visual distance guides and bounding box overlays for large-scale creative construction.

---

### Phase 13: Physics Engine, Fluid Dynamics & Item Drops (Planned)

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

### Phase 14: Dynamic Weather System & Meteorological Phenomena (Completed)

Inspired by Minecraft's vanilla atmospheric mechanics and advanced particle dynamics from the *Particle Rain* mod, Phase 14 establishes a high-performance, modular weather simulation and particle rendering system fully integrated with the 1.0m voxel world, Bevy 0.20, and WESL shaders.

- [x] **Phase 14.1: Meteorological State Machine & Atmosphere Transitions**:
  - Implement `WeatherManager` resource with cyclic and procedural weather states: `Clear`, `Rain`, `HeavyRain`, `Thunderstorm`, `Snow`, `Blizzard`, and `DustStorm`.
  - Continuous weather interpolation weights ($W_{\text{rain}}, W_{\text{thunder}} \in [0.0, 1.0]$) driving smooth cross-fading of ambient illumination, directional sun/moon intensity, exposure, and sky/fog color palette.
  - Climate-aware weather routing: map weather types dynamically based on biome temperature ($T_{\text{biome}}$), elevation lapse rate ($T(y) = T_{\text{biome}} - (y - y_{\text{sea}}) \times 0.0016$), and humidity ($H_{\text{biome}}$).
  - Atmospheric fog contraction: dynamically contract `DistanceFog` start and end boundaries during rainstorms, thick fog, and snowstorms.
- [x] **Phase 14.2: Dynamic 3D Wind Simulation Field**:
  - Implement `WindManager` generating continuous, multi-octave 3D wind velocity vectors $\vec{W}(x, y, z, t)$ adapted from Particle Rain's harmonic gust formulation (`oneWindPlease`).
  - Altitude-based wind scaling: calibrate wind speed by elevation ($y$-level scaling factor from $Y=50$ to $Y=90$), creating calmer valley breezes and howling mountain gusts.
  - Unify wind integration across the engine: expose global `WindState` driving weather particles, cloud drift acceleration, and future vegetation leaf/grass vertex sway.
- [x] **Phase 14.3: High-Performance GPU/Batched Weather Particle Engine**:
  - High-throughput instanced quad particle renderer (or compact flat DOD particle buffer) avoiding per-particle ECS entity overhead (capable of simulating 10,000+ active precipitation quads at 144+ FPS).
  - Velocity-aligned billboarding (`RELATIVE_VELOCITY`): tilt falling rain streaks and heavy drops dynamically along the composite velocity vector $(\vec{v}_{\text{gravity}} + \vec{W})$.
  - Screen-aligned and axis-locked quad rendering (`COPY_CAMERA`, `HORIZONTAL`, `VERTICAL`) for snowflakes, ground mist, dust clouds, and water ripples.
  - Particle lifetime curves with alpha fade envelopes (`FadeType::In`, `FadeType::Out`, `FadeType::InOut`).
- [x] **Phase 14.4: Precipitation & Ambient Surface Phenomena (Particle Rain Reverse-Engineering)**:
  - **Sky FX Spawning**: 3D cylindrical volume particle emitter centered around the player camera, with spawn rate scaled by storm intensity and player velocity.
  - **Surface FX & Fast Heightmap Sampling**: Sample ground positions using triangular distribution (`RandomSource::triangle`); query cached column surface heights for:
    - Water ripples (`ripple`): animated 8-frame flat horizontal quads on water surfaces with water tint.
    - Ground splatters (`splatter`): tiny bounce droplets on solid terrain blocks.
    - Thermal steam/smoke (`smoke`): vaporization particles when rain strikes hot or flaming blocks.
    - Post-storm & ambient ground fog (`fog_after_rain`, `fog`): soft, low-lying ground mist quads rising over moist soils after rain stops.
    - Arid dust & skittering sand (`dust`, `skittering_dust`, `dust_haze`): windblown dust swirls and tumbling shrubs (`dead_bush`) across desert and dry soils.
  - **Vertical Block Side Streaks (`streak`)**: Raycast-verified water trails sliding down exterior vertical faces of glass and impermeable blocks, culminating in dripping droplets at overhangs.
- [x] **Phase 14.5: Thunder, Lightning & Audio-Visual Spectacle**:
  - Procedural lightning bolt generation during `Thunderstorm` events: striking high-elevation terrain with instantaneous full-scene emissive flash and localized point light.
  - Physical distance-based audio delay: speed-of-sound calculated interval between visual flash and rumble impact.
  - Spatial weather audio pipeline: integration of multi-track ambient loops (rain intensity, ground vs. altitude snow murmur, whistling desert sandstorms using assets from mod references).
- [x] **Phase 14.6: Clean Simulation Reset Integration**:
  - Full synchronization with Pause Menu *Restart Game* action: resets active weather state to Clear Sky, flushes particle ring buffers, clears pending acoustic thunder queues, and despawns active lightning entities.

---

### Phase 15: Visual Aesthetics, Shaders & Atmospheric FX Overhaul (Planned)

- [ ] **Voxel Ambient Occlusion (Smooth Lighting)**:
  - Implement per-vertex ambient occlusion calculating corner and edge block occlusion, adding deep geometric contrast to all surfaces.
- [ ] **Advanced Fluid Shaders & Optical Effects**:
  - Upgrade water and liquid shaders with optical refraction, dynamic water caustics projected onto river/ocean beds, shoreline foam lines, and underwater visual distortion.
- [ ] **Bloom & Emissive Post-Processing**:
  - Integrate HDR bloom post-processing for light-emitting blocks (torches, lanterns, glowing crystals, lava, fireflies, and enchanted ores).
- [ ] **Volumetric Fog & Crepuscular Rays (God Rays)**:
  - Add screen-space or volumetric light shafts piercing through forest foliage, mountain ridges, and cavern ceilings.
- [ ] **Biome Color Grading & Atmospheric Tonemapping**:
  - Implement biome-specific tonemapping and color grading LUTs for tailored environmental atmospheres (e.g. moody caverns vs vibrant sunlit plains).

---

### Phase 16: Performance Engineering, Chunk Streaming & LOD Enhancements (Planned)

- [ ] **Streaming Throughput & Worker Scaling**:
  - Optimize asynchronous compute worker pools for chunk generation and greedy meshing with dynamic per-frame upload budgets.
- [ ] **Advanced Level-of-Detail (LOD) & Occlusion**:
  - Refine multi-tier chunk mesh LOD transitions to eliminate visible terrain pop-in at high render distances (32+ chunks).
  - Implement GPU-driven frustum culling and occlusion queries to avoid rendering occluded subterranean geometry.
- [ ] **Voxel Memory Footprint & Disk Streaming**:
  - Compress runtime voxel storage structures to maximize cache locality and reduce memory consumption per loaded chunk.
  - Accelerate binary region-based world serialization for seamless disk save and load streaming.

---

### Phase 17: Localization & Multi-Language Support (Completed)

- [x] **Language File Architecture (`lang/`)**:
  - Create a dedicated `assets/lang/` directory holding one translation file per locale (`en_us.json` default / fallback, and `pt_br.json`).
  - Use flat, namespaced translation keys (e.g. `menu.tab.system`, `block.sand_common`, `hud.tooltip.empty`) so new locales can be added without code changes.
- [x] **Comprehensive Translation Key Lookup & String Migration**:
  - Introduce a `Localization` resource that loads the active locale on startup and resolves keys at runtime, falling back to `en_us` for missing entries.
  - Migrate all UI strings (Settings options, Audio controls, Menu Bar tabs, Journal provisional view, Target HUD block names, Creative Inventory tabs/search) to translation keys.
  - Ensure all Portuguese translations adhere to natural phrasing, proper grammar, and standard gaming terminology (adjective-noun order, correct prepositions).
- [x] **In-Game Language Selector**:
  - Add a language option inside the Settings menu, persisted in `local_config.json`, with live reactive UI refresh on change (no restart required).
  - Validate font glyph coverage for accented Latin characters (e.g. `ã`, `ç`, `é`, `í`, `õ`) across all supported interfaces.

---

### Phase 18: Controls & Keybinding System (Completed)

- [x] **Unified Controls & Input Customization UI**:
  - Activate the existing `Controls` button across both the in-game Pause Menu and the Main Menu Settings.
  - Provide an intuitive categorization: *Movement*, *Gameplay & Interaction*, *Interface & Menus*, and *Mouse / Camera*.
- [x] **Rebindable Keybindings & Conflict Resolution**:
  - Full keyboard keybinding remapping for all engine actions: movement (`WASD`), jumping/flight (`Space`), crouching/sneaking (`LCtrl`), sprinting (`LShift`), inventory (`E`/`I`), world map (`M`), journal (`J`), shaping tool (`R`/`T`), and hotbar quick-slots (`1`-`9`).
  - Support multi-button mouse binding (Left, Right, Middle, Extra mouse buttons).
  - Real-time conflict detection highlighting overlapping keybinds in amber/red with one-click *Reset to Defaults* button.
- [x] **Mouse & Camera Configuration**:
  - Configurable mouse sensitivity sliders (stepper from 0.2x to 3.0x in 0.1 increments).
  - Invert Mouse Y-axis toggle for flight and first-person camera control.
  - Optional mouse smoothing and raw input toggles for precise cursor and crosshair handling.
- [x] **Input Configuration Persistence**:
  - Save all custom key assignments, mouse settings, and control flags into `local_config.json`.
  - Automatic migration and fallback to default input bindings on format updates or corrupt configurations.

---

### Phase 19: Dynamic Foliage & Procedural Grass System (Completed)

- [x] **Procedural Grass Blade Generation & Packing (Grassier Grass Integration)**:
  - Generate dense 3D grass blades on surface grass blocks and slabs (up to 64 blades/block) with deterministic PRNG seeded by block coordinate hash.
  - Apply slight 8% overhang beyond block edges to seamlessly blend block boundaries into a continuous natural carpet.
  - Compact blade metadata into vertex attributes (`Color.a` packing side, height class, and 32 quantised angle bins; `Normal.xy` packing world noise coords; `Normal.z` packing height fraction).
  - Multi-tier distance LOD: 4 quads/blade ($\le 32\text{m}$), 2 quads/blade ($32\dots 80\text{m}$), and 1 quad/blade ($> 80\text{m}$).
- [x] **Cubic Bezier Wind Deformation & WESL Shader Pipeline**:
  - Implement `shaders/grass_blades.wesl` reconstructing blade centerlines and bending spines along 3D Cubic Bezier splines.
  - Multi-layered wind simulation driven by `WindState`: continuous travelling wind gusts, medium-wind bow curve, high-frequency tip flutter, and calm resting curves.
  - Voronoi tuft clumping: sample packed `noise.png` for regional height swells, inward tuft curl, and coherent tuft lean/width.
  - GPU decimation: snap quad heights on shortened blades to collapse unused geometry into zero-area triangles, cutting rasterizer overdraw.
  - Camera-facing width alignment: subtly rotate edge-on blades towards the camera to preserve perceived fullness.
- [x] **Interactive Entity Footprint Trails (`GrassTrailField`)**:
  - Dynamic 48×48 block footprint texture centered on camera/player tracking entity positions and movement velocities.
  - Physically bend grass blades away from walking player/mobs in the shader, with smooth exponential spring-back recovery after entities pass.
- [x] **Vegetation Balanço & Dense Flower Patches**:
  - Implement `shaders/grass_plant.wesl` providing gentle base-locked head lean and bob to existing flowers, crops, and tall plants.
  - Sprout companion flowers next to adjacent flower blocks to create rich, dense flower patches.
- [x] **Styles, Weather Integration, Partículas & Configurações**:
  - Dual visual styles: *Tapered* (smooth organic gradient using `grass_tapered.png`) and *Segmented* (pixelated retro bands using `grass_bands.png`).
  - Wind-borne loose blade particles (`wind_blade.png`) during howling winds ($\ge 20\text{ m/s}$) and storms.
  - Weather integration: darken blades during rain and blend white snow tint during snowstorms or snow accumulation.
  - In-game Settings menu integration with full Brazilian Portuguese (`pt_br.json`) and English (`en_us.json`) localization, persisted in `local_config.json`.
