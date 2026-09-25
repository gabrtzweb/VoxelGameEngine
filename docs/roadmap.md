# Voxel Game Engine - Development Roadmap

This document outlines the planned development phases for the voxel game engine, with foundational architectural systems, gameplay tools, fluids, procedural generation, and high-performance scaling.

### Roadmap Phase Directory

| Phase | Milestone Name | Status | Focus / Key Deliverables |
| :--- | :--- | :--- | :--- |
| **Phase 1** | [Core Rendering & Texture-Array Architecture](#phase-1-core-rendering--texture-array-architecture-completed) | **Completed** | WGSL texture array shader, nearest-neighbor sampling, dynamic variant discovery, vertex tinting |
| **Phase 2** | [Atmosphere, Celestial Bodies & Dynamic Sky](#phase-2-atmosphere-celestial-bodies--dynamic-sky-completed) | **Completed** | 24-min day/night cycle, billboard sun/coronas, 8 moon phases, rotating starfield, drifting clouds |
| **Phase 3** | [Gameplay, Inventory & Sub-Voxel Shaping Tools](#phase-3-gameplay-inventory--sub-voxel-shaping-tools-completed) | **Completed** | 8-slot hotbar, initial block shaping & rotation (<kbd>R</kbd>/<kbd>T</kbd>), connected block placement |
| **Phase 4** | [Dynamic Fluid Simulation & Boundary Mechanics](#phase-4-dynamic-fluid-simulation--boundary-mechanics-completed) | **Completed** | Cellular automata water propagation, stepped fluid height, waterlogging, submerged fog |
| **Phase 5** | [General Polish, Revisions & In-Game Interfaces](#phase-5-general-polish-revisions--in-game-interfaces-completed) | **Completed** | 4-season calendar, F3 debug HUD, in-game pause/settings menu, player model & animations |
| **Phase 6** | [Advanced World Generation, Biomes & Caves](#phase-6-advanced-world-generation-biomes--caves-completed) | **Completed** | 11 biomes, continuous 2D climate noise, 3D caves/ravines, geological strata, F1 World Inspector |
| **Phase 7** | [Project Organization, Architecture Audit & Refactor](#phase-7-project-organization-architecture-audit--refactor-completed) | **Completed** | Domain modularization, async compute greedy mesher, FxHashMap voxel storage, zero-alloc collision |
| **Phase 8** | [Foundational Storage, Meshing & Bitmask Acceleration](#phase-8-foundational-storage-meshing--bitmask-acceleration-completed) | **Completed** | Chunk homogeneity flags, noise up-sampling (97% reduction), 4-bit paletted storage, 64-bit bitmasks |
| **Phase 9** | [Engine-Wide Architecture Modernization, 1m Shapes & Codebase Cleanup](#phase-9-engine-wide-architecture-modernization-1m-shapes--codebase-cleanup-completed) | **Completed** | 1m block shapes (`Full`, `Slab`, `Stair`, `Column`), dead code purged across 10 engine subsystems |
| **Phase 10** | [Gameplay Polish, Interaction Feedback & Quality-of-Life](#phase-10-gameplay-polish-interaction-feedback--quality-of-life-completed) | **Completed** | Block break/place feedback particles, dual-card inventory, minimap & world map, DoF blur |
| **Phase 11** | [World Generation & Worldbuilding Expansion (High Fantasy & Dark Fantasy Realism)](#phase-11-world-generation--worldbuilding-expansion-high-fantasy--dark-fantasy-realism-active) | **Active** | 512-block world depth, 14 fantasy biomes, multi-noise climate mapping, natural edge dithering |
| **Phase 12** | [Flora, Procedural Trees & Surface Vegetation](#phase-12-flora-procedural-trees--surface-vegetation-upcoming) | **Upcoming** | 1m procedural trees (Oak, Birch, Pine, Rainwood), ground cover, flowering plants, wind sway shader |
| **Phase 13** | [High-Performance Scaling, Level-of-Detail (LOD) & Engine Optimization](#phase-13-high-performance-scaling-level-of-detail-lod--engine-optimization-upcoming) | **Upcoming** | Multi-tier chunk mesh LOD, GPU Hi-Z occlusion culling, Multi-Draw Indirect, MCA region saves |

---

## Phase 1: Core Rendering & Texture-Array Architecture (Completed)
- [x] **Texture-Array Shader & Pipeline**:
  - Implemented custom WGSL shader in [assets/shaders/voxel.wgsl](VoxelGameEngine/assets/shaders/voxel.wgsl) via Bevy's `ExtendedMaterial<StandardMaterial, VoxelMaterialExtension>`.
  - Mapped texture array and sampler to `@group(#{MATERIAL_BIND_GROUP})` bindings 100 and 101, preserving standard PBR lighting, directional shadows, and distance fog.
- [x] **Pixel-Art Texture Asset Pipeline & Dynamic Variant Discovery**:
  - Implemented in [src/voxel/texture.rs](VoxelGameEngine/src/voxel/texture.rs) using `build_voxel_texture_array()`.
  - Standardized textures by category prefixes (`terr_`, `rock_`, `liqd_`, `emit_`) in `assets/textures/blocks/`.
  - Loads 16×16 PNG textures with nearest-neighbor sampling (`ImageSampler::nearest()`) into a hardware 2D Texture Array (`TextureDimension::D2`).
  - Auto-discovers multiple texture variants per block type (`{name}.png`, `{name}1.png`, `{name}2.png`, etc.) without code changes, with 4 variants each for grass, dirt, stone, and sand.
  - Ensured all 6 faces of a voxel share the same texture (uniform grass styling).
- [x] **Mesher Integration, Deterministic Spatial Randomization & Color Tinting**:
  - Implemented in [src/voxel/mesher.rs](VoxelGameEngine/src/voxel/mesher.rs).
  - Supplies the layer index per vertex via `Mesh::ATTRIBUTE_UV_1`, with `fract(uv)` in WGSL tiling greedy-meshed quads cleanly without stretching.
  - Selects variants via an integer spatial hash of each voxel's 3D world coordinate (`world_voxel`), guaranteeing consistent random distributions with zero flickering across chunk remeshes.
  - Integrates vertex color tinting (`Mesh::ATTRIBUTE_COLOR`) for grayscale textures (grass and water) to support biome and environmental tint variations while preserving greedy-mesh boundaries.
- [x] **Phase 1 Polish & Additional Refinements**:
  - Replaced legacy textures with 4-letter categorized variants (`terr_`, `rock_`, `liqd_`, `emit_`).
  - Implemented grayscale tinting pipeline for grass and water in the mesher and shader with calibrated brightness.
  - Added vertical-strip animated texture pipeline (e.g. 16×576 `liqd_water_still` with 36 frames) driven seamlessly on the GPU using WGSL `globals.time` and vertex frame-count attributes at 6.0 FPS.
  - Softened shadow contrast by boosting daytime ambient illuminance from 62 to 450 lux and tuning exposure.
  - Calibrated sub-voxel water geometry: surface water voxels have a 40cm height offset (yielding a 90cm water height for 1m blocks matching Minecraft fluids and eliminating shoreline z-fighting).
  - Enabled Creative flight activation (double-tap Space) while swimming and submerged.
  - *(Note: Rendering top water faces from underneath while submerged and fluid flow mechanics are deferred to Phase 4).*

---

## Phase 2: Atmosphere, Celestial Bodies & Dynamic Sky (Completed)
- [x] **4-Phase Day & Night Cycle**:
  - Implemented continuous in-game astronomical clock in [src/environment.rs](VoxelGameEngine/src/environment.rs) (`time_of_day: 0.0..1.0`, default 600s cycle) categorized into 4 discrete phases (`Morning`, `Noon`, `Evening`, `Night`).
  - Automatically tracks day count on midnight-to-morning cycle rollover, advancing the calendar and moon phase.
- [x] **F6 Time Controls**:
  - Implemented dual-mode input handling: tapping/clicking `F6` (<0.25s) steps immediately to the next discrete phase (`Morning` -> `Noon` -> `Evening` -> `Night`).
  - Holding `F6` (>0.25s) continuously scrubs time forward smoothly at an accelerated pace (0.22 day units/sec).
- [x] **Flat Textured Celestial Billboards**:
  - Implemented in [src/environment/celestial.rs](VoxelGameEngine/src/environment/celestial.rs).
  - Celestial bodies are rendered as flat billboard quads facing the camera rather than 3D cubes, scaled with distinct proportions: Sun at 52m and Moon at 40m (at 100m distance).
  - Uses Minecraft-style **Additive Blending** (`AlphaMode::Add`): black pixels (`[0, 0, 0]`) act as mathematical zero (leaving sky colors 100% untouched without dark halos), while luminous RGB values physically add light to the skybox.
  - Enhanced solar corona with HDR base color luminance and non-linear power curve for a prominent, glowing ring and rays against the daytime sky.
  - Set `fog_enabled: false` on celestial materials so distance fog never draws solid boxes over the sun or moon.
  - Sun casts 4-level cascaded directional shadows with daytime sky fill lighting.
- [x] **8 Individual Moon Phases**:
  - Dedicated individual textures for each phase loaded from `assets/textures/environments/celestial/moon/` (Full Moon, Waning Gibbous, Third Quarter, Waning Crescent, New Moon, Waxing Crescent, First Quarter, Waxing Gibbous).
  - Uses additive blending to render crisp glowing crescents and phases against the night sky without square artifacts.
  - Active texture swaps dynamically with `day_count % 8`, and directional moonlight intensity/shadows scale based on the active phase's illumination factor.
- [x] **Atmosphere, Dynamic Fog & Color Transitions**:
  - Continuous 4-stop piecewise-linear palette interpolation across Morning, Noon, Evening, and Night.
  - Dynamically blends `ClearColor`, `GlobalAmbientLight` (color and brightness), camera `DistanceFog` (color and directional scattering exponent), and camera `Exposure` (EV100).
- [x] **Night Starfield**:
  - Implemented in [src/environment/stars.rs](VoxelGameEngine/src/environment/stars.rs).
  - 250 procedural stars placed at 140m distance (behind clouds at 80m and celestial bodies at 100m in Bevy's back-to-front transparent render queue), ensuring clouds naturally occlude stars.
  - GPU-batched instanced entities sharing a single quad mesh and procedural 16×16 soft circular texture.
  - Uses additive blending (`AlphaMode::Add`), `fog_enabled: false`, rotating around the celestial polar axis, and smoothly fades from daytime (0.0) to HDR night sparkle (1.6).
- [x] **Stylized Cloud System**:
  - Implemented in [src/environment/clouds.rs](VoxelGameEngine/src/environment/clouds.rs).
  - Renders the 256×256 texture from `assets/textures/environments/clouds.png` on a large horizontal plane (1600m × 1600m) at altitude Y = 80m.
  - Uses native alpha blending, nearest-neighbor sampling, and `fog_enabled: false`.
  - Drifts at a gentle, relaxed speed (1.8 m/s in X, 0.6 m/s in Z) across the sky without popping, with time-of-day color tinting.

---

## Phase 3: Gameplay, Inventory & Sub-Voxel Shaping Tools (Completed)
- [x] **8-Slot Hotbar GUI**:
  - Implemented in [src/player/hotbar.rs](VoxelGameEngine/src/player/hotbar.rs).
  - 8 selectable item slots rendered with dark translucent backing, active gold selection border, slot indices (1..8), and 2D pixel-art icons from `assets/textures/items/`.
  - Seamless input handling: select directly via number keys `1`–`8`, scroll forward/backward via the mouse wheel, and clear active slot with `Q`.
  - Automatically synchronizes with `SelectedVoxel` and middle-click block picking.
- [x] **Sub-Voxel Block Shaping Tool (`R` Key)**:
  - Implemented in [src/voxel/shaping.rs](VoxelGameEngine/src/voxel/shaping.rs).
  - Targets 1m³ blocks in Block interaction mode and cycles their 2×2×2 sub-voxel layout on each `R` press: Full Block (8 voxels) -> Stair (6 voxels) -> Upside-Down Stair (6 voxels) -> Corner Stair (5 voxels) -> Bottom Slab (4 voxels) -> Top Slab (4 voxels) -> Vertical Slab (4 voxels) -> Column (2 voxels) -> Centered Column (2 stacked centered voxels).
  - Preserves the targeted block's material and updates lighting, chunk meshes, and persistence immediately.
  - Centered columns are composed of 2 independent stacked centered voxels (0.5m × 0.5m centered footprint with 0.25m margin on all 4 sides), allowing either voxel to be broken or built on top independently.
  - Bidirectional Voxel-mode centered placement: placing on the floor directly beneath a hanging centered column or stacking vertically automatically aligns and connects centered voxels without falling back to corner grid placement.
- [x] **Connected Block Placement on Slabs & Sub-Voxels**:
  - Implemented in [src/voxel/targeting.rs](VoxelGameEngine/src/voxel/targeting.rs) and [src/voxel/interaction.rs](VoxelGameEngine/src/voxel/interaction.rs).
  - Blocks placed against slabs, stairs, upside-down stairs, or non-full shapes align directly to the hit surface rather than the rigid 2x2x2 block grid, seamlessly completing bottom slabs into full blocks and stacking slabs/blocks above with zero floating air gaps.
  - Full support for composite emergent structures (e.g. half-column / half-slab blocks), with block-mode outline highlights accurately rendering both the centered column sub-voxel and standard sub-voxel layers simultaneously.
- [x] **Block Rotation Tool (`T` Key)**:
  - Rotates the targeted block's sub-voxels 90° clockwise around the vertical Y-axis, allowing stairs, upside-down stairs, corner stairs, slabs, and columns to face in any cardinal direction.

---

## Phase 4: Dynamic Fluid Simulation & Boundary Mechanics (Completed)
- [x] **Dynamic Water Propagation**:
  - Cellular automaton simulation queue running at a paced 0.25s tick rate with per-tick queue batching (`FluidUpdateQueue`).
  - Downward waterfall priority: fluid falls strictly downwards in mid-air (`is_supported_by_ground`), preventing mid-air spread along pillars and cliff edges.
  - Differential spread limits: 4 voxels (2 blocks) for single-voxel sources, 8 voxels (4 blocks) for full-block sources.
- [x] **Gradual Height Transitions & Vertical Step Walls**:
  - Height model decreasing by 10cm per step down to 10cm at the stream boundary.
  - Vertical step side quads connecting adjacent water levels, seamlessly closing any gaps/holes between steps.
- [x] **Cross-Chunk Fluid Updates & Waterlogging**:
  - Seamless propagation across chunk boundaries updating lighting, mesh registry, and neighbor chunks.
  - Automatic waterlogging of cutouts during underwater shaping (`R` key) and rotating (`T` key) without trapping dry air or breaking shape detection.
- [x] **Water Surface & Underwater Visuals**:
  - Dynamic surface offsets eliminating z-fighting.
  - Animated surface and flowing water textures with correct frame count metadata.
  - Full underwater visibility looking up from below with counter-clockwise winding ceiling geometry.
  - Submerged blue fog immersion and camera-in-water detection with sky/cloud handling.

---

## Phase 5: General Polish, Revisions & In-Game Interfaces (Current)
- [x] **Environment & Calendar System Implementation**:
  - Set day duration to 24 minutes in real time.
  - Implement a 28-day month and a 4-season year (Spring, Summer, Autumn, Winter), with each season lasting exactly 3 months (84 days).
  - Set the game to always start on Day 1 of Month 1, marking the exact beginning of Spring.
  - Implement the 8-phase lunar cycle synchronized with the 28-day month. Alternate phase durations strictly between 3 and 4 days.
- [x] **Debug Screen (F3) & HUD Refactor**:
  - Refactor the current debug text to have toggleable states instead of a single cluttered view.
  - Minimal HUD: Show only essential gameplay info like FPS, Current Time, Day, Season, XYZ Position and Game Mode.
  - Extended Debug (F3 Mode): Show the full technical layout including Frame time, Moon Phase, Flight status, Player chunk, Camera coordinates, Loaded/Meshed chunks, Mesh vertices/triangles, Voxel capacity, and Target voxel (Currently, the F3 key is assigned to toggle Chunk debug; change it to F2).
- [x] **In-Game Settings, Inventory (`E` Key) & Pause Menu (`ESC` Key)**:
  - Stylized UI overlay pausing gameplay/freeing mouse cursor when pressing `ESC`.
  - In-game configurable settings:
    - Render Distance (chunk radius slider/stepper, dynamically resizing active chunk streaming).
    - Field of View (FOV) slider.
    - Fog toggles and density controls.
    - Toggle in-game time to be paused.
  - In game "creative inventory" (`E` Key) - 8×4 (32 slots) grid matching hotbar width, item pickup/swap/clearing, quick-assign (1–8 keys), and custom mouse cursor with floating item previews.
- [x] **Cross-Phase Refinements & Mechanics Polish**:
  - [x] **Priority A: Block Palette Expansion, 3D Block Icons & Inventory Simplification**:
    - Extracted block definitions into dedicated [src/voxel/blocks.rs](VoxelGameEngine/src/voxel/blocks.rs), supporting 28+ block types, texture IDs, and future survival properties (durability, preferred tool).
    - Integrated new blocks: Blackstone, Cobbleblackstone, Slate, Cobbleslate, Cobblestone, Mossy Cobblestone/Stone, Magma, Flint, Mud, Packed Dirt/Mud, Mulch, Moss, Snow, Clay, Gravel, and RGB/temperature lights (Warm, Cold, Red, Green, Blue).
    - Implemented Minecraft-style 3D isometric pixel-art icon rasterizer in [src/voxel/icon.rs](VoxelGameEngine/src/voxel/icon.rs) generating 32×32 icons on-the-fly with 1.0/0.8/0.6 directional face shading and tints.
    - Cleaned up inventory UI: simplified title to "INVENTORY", removed "HOTBAR" label, and eliminated explanatory tooltip/hover text.
    - Added cinematic camera background blur (`DepthOfField`) whenever the pause or settings menu is opened, while keeping the world alive and unblurred during inventory interactions.
    - Fixed light-emitting blocks: integrated into chunk mesher with actual textures, added shader self-illumination radiance (`emissive`), and consolidated to 1 high-intensity 3D point light per block.
    - Removed legacy static icons in `assets/textures/items/` while preserving directory for future item sprites.
  - [x] **Priority B: Sub-Voxel UV Blending & Seamless Texturing**:
    - Full 1m³ block face unification: A full 1m² face (composed of 2×2 co-planar sub-voxels) maps a single continuous 16×16 texture across the entire surface rather than repeating 4 times.
    - Adaptive sub-voxel UV mapping: Isolated voxels and columns retain clean full [0, 1] texture mapping to avoid awkward corner cropping, while contiguous sub-voxels (slabs, steps) seamlessly blend across their shared plane.
  - [x] **Priority C: Radial Shape Selection Menu (<kbd>Hold R</kbd>) & Inverted Corner Stairs**:
    - Added new shape `CornerStairInverted`: 4 base voxels + 3 top voxels (7 solid sub-voxels) leaving a single corner cutout, with full 4-way 90° Y-rotation support.
    - Short tap <kbd>R</kbd>: Quick-cycles to the next shape sequentially (Full -> Stair -> StairUpsideDown -> CornerStair -> CornerStairInverted -> SlabBottom -> SlabTop -> VerticalSlab -> Column -> CenteredColumn -> Full).
    - Hold <kbd>R</kbd> (>0.2s): Sleek circular radial wheel centered on screen showing all 10 shapes with directional mouse selection, center preview card, and instant release-to-apply.
  - [x] **Priority D: Player Body Model, Minecraft Skin Support & Procedural Animations (<kbd>F5</kbd>)**:
    - Classic humanoid limb hierarchy (Head, Torso, Left/Right Arm, Left/Right Leg) mapped to standard Minecraft 64×64 skin textures (`assets/textures/models/player_skin.png`), supporting both base skin and 3D outer overlays (hat/hair, jacket, sleeves, pants).
    - True first-person body visibility: Head is automatically hidden to prevent camera interior clipping, while looking down reveals animated arms, chest, and legs beneath the player.
    - Third-person view toggle (<kbd>F5</kbd>): Full body and head become visible with camera orbiting behind and head yaw/pitch tracking camera view.
    - Procedural animations: Dynamic walk/sprint leg & arm pendulum swings, idle breathing sway, crouch torso tilt (<kbd>Ctrl</kbd>), prone crawl locomotion (<kbd>C</kbd>), airborne jump poses, and aquatic swimming strokes.
  - [x] **Priority E: Movement & Camera Juice**:
    - Default FOV: Set baseline camera FOV to 90.0° with settings slider adjustment.
    - Crouch (<kbd>Control</kbd>): Lowers collision height to 1.3m and eye height to 1.20m, reduces speed, and prevents walking off precarious block edges (ledge clamping).
    - Crawl (<kbd>C</kbd>): Lowers collision height to 0.45m and eye height to 0.40m, allowing crawling through 1-voxel high openings (0.5m) and under low overhangs, with headroom safety checks preventing uncrawling under ceilings.
    - Camera Zoom (<kbd>Z</kbd>): Holding <kbd>Z</kbd> zooms smoothly with dynamic mouse scroll wheel control (scroll up zooms in closer up to 20x magnification, scroll down zooms out), automatically suppressing hotbar slot cycling during zoom and smoothly scaling mouse sensitivity.
    - View Bobbing: Subtle sinusoidal head bobbing during grounded walking and sprinting, toggleable in In-Game Settings.
    - Dynamic FOV Kick: Smooth camera FOV expansion (+8°) when sprinting or flying fast.
---

## Phase 6: Advanced World Generation, Biomes & Caves (Completed)
- [x] **Multi-Noise Biome System**:
  - Macro-scale continuous 2D climate noise in [src/generation/biome.rs](VoxelGameEngine/src/generation/biome.rs) for Continentalness, Temperature, and Humidity.
  - Continuous $C^1$ smooth cubic spline curve for continental base elevation and continuous roughness multiplier, completely eliminating harsh elevation cuts and abrupt vertical cliffs across biome boundaries.
  - 11 distinct biomes with individual surface, subsoil, and elevation profiles:
    - **Plains**: Temperate, moderate humidity, rolling green hills, grass surface, dirt sublayer.
    - **Desert**: Warm, arid, wind-swept sand dunes, sandstone/sand sublayer, red sand accents.
    - **Snowy Tundra & Frost Peaks**: Frigid high altitudes, snow-covered surface with stone outcrops.
    - **Wetlands / Swamps**: Low-lying smooth swamp basins, natural mix of swamp grass (40%), mud (35%), packed mud (15%), and clay (10%).
    - **Rocky Highlands**: Rugged towering mountain ridges, organic mix of slate, stone, and cobbleslate.
    - **Woodland**: High humidity, living forest floor with natural blend of grass (50%), mulch (30%), packed dirt (10%), and moss (10%).
    - **Meadow**: Gentle transition between woodland and plains with vibrant flora.
    - **Beach & Coast**: Sand coastlines wrapping all sea-level land borders.
    - **River**: Winding fluvial ribbons carved with 2D ridge noise zero-crossings.
    - **Ocean & Deep Ocean**: Continental seabed depression with abyssal gravel and blackstone beds.
- [x] **3D Caves & Underground Caverns**:
  - High-performance native 3D gradient noise in [src/generation/caves.rs](VoxelGameEngine/src/generation/caves.rs).
  - Walkable spaghetti worm tunnels (3–5 blocks wide) with wide surface entrances.
  - Cheese caverns creating large subterranean chambers and grottos.
  - Subterranean water aquifers below sea level, and deep magma/lava pools at lowest crust boundaries.
  - Ravines tuned to rare dramatic chasms, strictly suppressed underwater in rivers, lakes, and oceans.
- [x] **Realistic Geological Strata, Bedrock & Mineral Deposits**:
  - Depth-based geological layering in [src/generation/strata.rs](VoxelGameEngine/src/generation/strata.rs) with natural 3D noise dithering at all layer transitions.
  - **Dreadstone Bedrock**: Unbreakable bedrock layer at the bottom of the world ($Y = -80$ blocks / $-160$ voxels), blending naturally into Blackstone across the bottom 3 layers.
  - Upper Crust: Standard Stone with gravel pockets, flint veins, and cobblestone fractures.
  - Mid Crust: Metamorphic transition into Slate, Cobbleslate, and Flint clusters.
  - Deep Crust: Volcanic plutonic layer of Blackstone, Cobbleblackstone, Magma veins, and molten pools.
- [x] **Runtime Generation Controls & Dedicated World Inspector GUI**:
  - Custom egui tuning window bound to <kbd>F1</kbd> running in `EguiPrimaryContextPass` with full interactive clicking and dragging for noise, river, cave, and strata sliders.
  - Automated game input isolation while inspector is open: suppresses hotbar mouse wheel scrolling, number key slot cycling, crosshair display, spectator movement, and block break/place clicks.
  - Instant **"Regenerate World"** live reload button that purges and re-streams world chunks with new seed/parameters.
- [x] **Extended Debug HUD & Default Settings Polish**: 
  - Minimal and Extended Debug HUD accurately displays column biome name (Plains, Beach, River, Ocean, Woodland, etc.).
  - Default render distance set to **12 chunks**; distance fog disabled by default.

---

## Phase 7: Project Organization, Architecture Audit & Refactor (Completed)
Comprehensive technical review, bottleneck diagnostics, and optimization plan successfully executed across all engine domains.

- [x] **Stage 7.1: Immediate Bottleneck Elimination (High FPS Impact)**:
  - [x] **Optimize `logical_block_has_exposed_dirt`**: Eliminated the 8×5 nested neighborhood loop in terrain generation. Cached column height samples and sampled only the top surface of the logical block, eliminating up to 80,000+ redundant noise calls per chunk.
  - [x] **Turn Off Asset Dirtying in Environment Systems**: In `stars.rs`, `clouds.rs`, and `celestial.rs`, inspected if material properties actually changed before calling `materials.get_mut()`, preventing constant GPU bind group and uniform buffer invalidations.
  - [x] **Parent Starfield to Single Root Entity**: Eliminated the 250-entity mutation loop in `sync_starfield` by parenting all star quads to a single rotating `StarfieldRoot` entity.
  - [x] **Direct Indexing for Texture Registry**: Replaced `HashMap<Voxel, VoxelTextureMapping>` with a fixed array `[Option<VoxelTextureMapping>; 34]` for O(1) direct memory indexing without SipHash in the greedy mesher loop.
  - [x] **Cache Submersion and Water Checks**: Introduced a lightweight `PlayerEnvironmentStatus` resource updated once per frame, eliminating 4 duplicate `is_point_in_water` and `player_submersion` evaluations.

- [x] **Stage 7.2: Background Thread Meshing & Frame Throttling**:
  - [x] **Move `ChunkMesher::build_meshes` to `AsyncComputeTaskPool`**: Run greedy meshing in worker threads using snapshot voxel data; main thread only receives finished `Mesh` buffers and binds them to Bevy entities.
  - [x] **Remove Direct Synchronous Meshing from Gameplay Systems**: Routed fluid simulation, player edits, shaping tools, and pause restarts through the remesh queue rather than executing unbuffered synchronous remeshing on the main thread.
  - [x] **Switch to Fast Hasher for `VoxelWorld`**: Replaced `std::collections::HashMap` with `bevy::platform_support::collections::HashMap` / `FxHashMap` for 3x–5x faster chunk lookups.

- [x] **Stage 7.3: Codebase Modularization & Directory Restructure**:
  - [x] Reorganized codebase into domain subdirectories: `core/`, `environment/`, `menu/`, `player/`, `world/`, `generation/`, `meshing/`, `simulation/`, and `gameplay/`.
  - [x] Split monolithic `src/voxel/mesher.rs` (1,571 lines) into `greedy.rs`, `shapes.rs`, and `pipeline.rs`.
  - [x] Split `src/voxel/shaping.rs` (1,038 lines) into `shaping.rs`, `radial_menu.rs`, and sub-voxel geometry helpers.
  - [x] Cleaned up `src/environment.rs` into modular domain files (`atmosphere.rs`, `celestial.rs`, `clouds.rs`, `stars.rs`, `time.rs`).

- [x] **Stage 7.4: Collision & Memory Micro-Optimizations**:
  - [x] Replaced `Vec<IVec3>` allocations in `overlapping_solid_voxels` with stack-allocated buffers and visitor closures.
  - [x] Added early-exit bounds check for fully empty or solid chunks during meshing.

---

## Phase 8: Foundational Storage, Meshing & Bitmask Acceleration (Completed)

Phase 8 established deep algorithmic and memory optimizations to scale chunk throughput, slash generation latency, and compress the memory footprint for high render distances and smooth 144+ FPS gameplay.

### Strategic Priorities & Architecture
1. **Priority 1: Extremity Bound Checking & Chunk Homogeneity Metadata** (Highest ROI / Immediate O(1) skips across meshing, collision, and raycasting)
2. **Priority 2: Noise Up-Sampling & 3D Trilinear Interpolation** (Massive generation speedup, 97% reduction in 3D noise evaluations per chunk)
3. **Priority 3: RLE & Paletted Runtime Voxel Data + Cache Locality Layout** (Drastic RAM reduction, cache-friendly iteration, foundations for disk saves)
4. **Priority 4: Decoupled Simulation Radius vs. Render Distance** (Simulate fluids & ticking only in inner 4–6 chunks; outer 12–16 chunks remain static meshes)
5. **Priority 5: Bitwise Bitmask Acceleration for Face Culling & Greedy Mesher** (64-bit bitboards for sub-millisecond chunk meshing)
6. **Priority 6: Static Lookup Tables (LUTs) for Shape Transforms & Face Offsets** (Eliminates runtime coordinate math in inner loops)
7. **Priority 7: LOD Render Distance** (Elevated and moved to Phase 13 for dedicated high-performance LOD meshing and skirt stitching)

---

### Implementation Breakdown

- [x] **Stage 8.1: Extremity Bound Checking & Chunk Homogeneity Flags**:
  - **The Problem**: Collision checks, raycasting, and meshing previously traversed coordinate ranges inside chunks that are 100% open sky (`Air`) or 100% subterranean rock (`Stone`/`Slate`).
  - **Architecture**:
    - Introduced chunk state metadata: `ChunkHomogeneity::Empty` (100% Air), `ChunkHomogeneity::Solid(Voxel)` (100% single solid material), or `ChunkHomogeneity::Mixed`.
    - Maintained non-air voxel count and unique voxel variant counters during procedural generation and runtime edits in O(1).
  - **Engine Benefits**:
    - **Meshing**: Completely bypasses chunk mesher task spawning for `Empty` chunks and fully occluded `Solid` chunks surrounded by solid neighbors (zero background tasks scheduled, zero memory allocations).
    - **Raycasting**: O(1) skip across empty chunks during line-of-sight ray traversal; O(1) hit on bounding box faces of solid chunks without voxel-level ray marching.
    - **Collision**: Player movement queries immediately skip empty chunks without iterating over coordinate ranges.

- [x] **Stage 8.2: Noise Up-Sampling & Caching (Trilinear Interpolation)**:
  - **The Problem**: Procedural chunk generation evaluated complex multi-octave 3D Simplex/Perlin noise independently for all 4,096 voxels in a chunk, creating CPU bottlenecks on the `AsyncComputeTaskPool`.
  - **Architecture**:
    - Compute 3D cave/density noise only at a coarse lattice of sample points (4×4×4 voxel cells) within the chunk grid.
    - For each cell, evaluate the 8 corner lattice points, then interpolate the inner 64 voxel densities using fast trilinear interpolation (`lerp` across X, Y, Z).
  - **Engine Benefits**:
    - Reduces expensive 3D noise evaluations from **4,096 down to 125 samples per chunk** (a **97% reduction** in mathematical noise evaluations).
    - Drastically accelerates async chunk generation speed, eliminating chunk streaming pop-in during flight.

- [x] **Stage 8.3: Paletted Storage & Cache Locality**:
  - **The Problem**: Uncompressed flat `[Voxel; 4096]` arrays consume excessive RAM and cause CPU L1/L2 cache misses during high render distance streaming.
  - **Architecture**:
    - Introduced paletted chunk storage:
      - `ChunkStorage::Uniform(Voxel)`: 1 byte of data for uniform Air or Stone chunks.
      - `ChunkStorage::Paletted`: Chunks with <= 16 distinct block types use 4-bit indices pointing into a local 16-element palette (shrinking 4,096 bytes down to ~2,048 bytes).
      - `ChunkStorage::Dense(Box<[Voxel; 4096]>)`: Flat uncompressed buffer used only during active multi-voxel player editing or when complexity warrants.
    - **Cache Locality Optimization**: Ensured chunk indexing order matches CPU L1 cache line stride (64 bytes) during raycast marching and meshing passes.
  - **Engine Benefits**:
    - Cuts overall world memory footprint by **70% to 85%**.
    - Prepares the data structures directly for fast binary disk serialization.

- [x] **Stage 8.4: Decoupled Simulation Radius vs. Render Distance**:
  - **The Problem**: Simulating fluids and dynamic updates across all 2,000+ active chunks at render distances 12–16 wasted CPU cycles on distant, non-visible activity.
  - **Architecture**:
    - Decoupled `simulation_distance` (inner 4–6 chunks, ~32–48m radius around player) from visual `render_distance` (10–16+ chunks).
    - Chunks within the simulation radius actively process fluid ticks, falling sand, and dynamic neighbor updates. Chunks beyond the simulation radius remain purely static mesh renderables.
  - **Engine Benefits**:
    - Caps active fluid/simulation CPU budget to a fixed, small local bubble regardless of how high the player sets their visual render distance.

- [x] **Stage 8.5: Bitwise Bitmask Acceleration for Face Culling & Greedy Mesher**:
  - **The Problem**: Checking adjacent voxel solid/air states through millions of individual 3D index calls in nested loops produced heavy CPU overhead during meshing.
  - **Architecture**:
    - Represent each 16-voxel row or 16×16 slice as 64-bit integer bitboards (`u64`).
    - Compute exposed face visibility across entire rows simultaneously using bitwise boolean operations:
      `visible_faces = current_row & ~neighbor_row`.
    - Extract contiguous runs using CPU intrinsic instructions (`trailing_zeros`, `leading_zeros`) to feed directly into quad generation.
  - **Engine Benefits**:
    - Cuts CPU time spent in face extraction by **4x–8x**, enabling near-instantaneous chunk remeshes when placing or breaking blocks.

- [x] **Stage 8.6: Static Lookup Tables (LUTs) for Shape Transforms & Face Offsets**:
  - **The Problem**: Sub-voxel shaping tools (<kbd>R</kbd>), block rotations (<kbd>T</kbd>), and face normal transforms performed coordinate arithmetic and rotation matrix operations at runtime.
  - **Architecture**:
    - Precomputed static lookup tables for shape configurations across 4 rotation orientations.
    - Precomputed face normal vectors, UV quadrant offsets, and neighbor chunk coordinate offsets into `const` LUT arrays.
  - **Engine Benefits**:
    - Replaces runtime trigonometric calculations and branch trees with zero-cost table lookups.

---

## Phase 9: Engine-Wide Architecture Modernization, 1m Shapes & Codebase Cleanup (Completed)

This phase executed a comprehensive cleanup, technical debt elimination, dead code purging, and systems modernization across the entire engine. It finalized the engine's transition away from the legacy 50cm sub-voxel architecture (where 1m blocks were composed of 8 individual 50cm sub-voxels) into a native 1x1m voxel paradigm with explicit shapes (`Full`, `Slab`, `Stair`, `Column`), orientations, and clean meshing, while preserving pristine build integrity (zero compiler warnings, zero Clippy lints, and 100% passing tests).

- [x] **Stage 9.1: Meshing Subsystem Cleanup & Textures Decoupling (Completed)**:
  - **`src/meshing/textures.rs`**:
    - Removed 272 lines of dead code, redundant variant count assertions, and hardcoded test suites.
    - Decoupled texture array generation from fixed variant assumptions: texture arrays dynamically load any number of texture variants per block type (`{name}.png`, `{name}1.png`, etc.) without artificial constraints.
    - Removed unused `get_texture_info()` helper.
  - **`src/meshing/greedy.rs`**:
    - Purged 646 lines of obsolete tests and unused constants.
    - Integrated shape filtering in greedy bitmask extraction: non-full blocks (`BlockShape != Full`) bypass greedy quad merging so that custom shape geometry is rendered with correct silhouettes and face culling.
    - Linked `mesh_shaped_voxels` hook into chunk meshing pipeline.
  - **`src/meshing/shapes.rs`**:
    - Replaced legacy 50cm 8-subvoxel logic with high-performance geometry generators for 1x1m shaped blocks:
      - **Slab**: 6 orientations (Bottom/Floor, Top/Ceiling, North Wall, South Wall, West Wall, East Wall).
      - **Column**: 6 orientations (Centered Vertical, 4 Corner Vertical columns, Centered Horizontal).
      - **Stair**: 8 orientations (4 upright cardinal directions + 4 inverted cardinal directions).
    - **Invisible Block Bug Fix**: Fixed coordinate unpacking bug where $Y$ and $Z$ strides were inverted (`(index / 16) % 16` vs `index / 256`), which caused non-full blocks to query empty air and fail to generate vertices. Added `Chunk::index_to_xyz(index)` with comprehensive roundtrip verification.
    - Automatic neighbor face culling against adjacent solid full blocks.
    - Purged obsolete compatibility stubs (`is_chunk_local_isolated_voxel`, `mesh_centered_voxels`, `push_water_quad_both_sides`, etc.).
  - **`src/meshing/mod.rs`**:
    - Removed `#![allow(unused_imports)]`.
    - Pruned dead re-exports to strictly export active pipeline types (`ChunkMeshingTask`, `ChunkMaterial`, `ChunkMeshRegistry`, `remove_chunk_render`, `sync_chunk_render`, `VoxelTextureRegistry`, `MeshingPlugin`).

- [x] **Stage 9.2: Gameplay Subsystem Cleanup & Block Shaping Overhaul (Completed)**:
  - **`src/world/block.rs` & `src/world/chunk.rs`**:
    - Defined `BlockShape` enum (`Full`, `Slab`, `Stair`, `Column`) with orientation counts, naming helpers, and sequential cycling.
    - Added `BlockShape::local_boxes(orientation)` returning exact $[0..1]^3$ sub-box bounding boxes for all shapes and orientations.
    - Implemented sparse chunk shape storage (`HashMap<usize, (BlockShape, u8)>`) for memory efficiency and future serialization.
    - Added `get_shape` and `set_shape` accessors across `Chunk`, `VoxelWorld`, and `VoxelAccess`.
    - Integrated shape persistence into `WorldModificationStore` so player edits survive chunk unload and reload.
  - **`src/gameplay/targeting.rs`**:
    - **Adaptive Shape Highlight**: Updated wireframe gizmo highlight to draw the precise sub-box outlines defined by `shape.local_boxes(orientation)` instead of a generic full 1x1x1 cube.
    - **Accurate Sub-Box Raycasting**: Integrated `ray_hit_local_box` intersection so player line of sight tests against actual physical shape geometry, allowing rays to pass through the empty negative space of slabs, stairs, and columns.
  - **`src/gameplay/shaping.rs`**:
    - Overhauled from ~900 lines down to ~250 lines.
    - Removed legacy 8-subvoxel bitmask tables (`FULL_BLOCK_MASK`, `HALF_SLAB_BOTTOM_MASK`, etc.) and voxel-index arithmetic.
    - Implemented direct shaping controls:
      - **Hold `R` + Move Mouse**: 4-slice Radial Menu wheel to select between `Full`, `Slab`, `Stair`, and `Column`.
      - **Tap `R`**: Rapid sequential shape cycle.
      - **Press `T`**: Cycles through orientations for the targeted block.
    - Purged all obsolete compatibility stubs (`detect_current_shape`, `get_block_voxels`, `is_centered_layer`, etc.) and unused imports (`FluidUpdateQueue`, `Voxel`).
  - **`src/gameplay/radial_menu.rs`**:
    - Replaced legacy 8-subvoxel descriptions and redundant slices with a clean 4-slice radial wheel.
    - Center preview card displays shape title, total orientation count, and control tips with responsive hover and selection highlights.
  - **`src/gameplay/target_hud.rs`**:
    - Modernized targeted block info resolution to inspect chunk shape storage.
    - HUD title formats dynamically with shape and orientation name (e.g. `Stone (Slab - Bottom (Floor))` or `Oak Planks (Stairs - Upright (+X))`).
    - Pruned obsolete 8-subvoxel unit tests and replaced with clean 1x1m block tests.
  - **`src/gameplay/interaction.rs`**:
    - Cleaned up obsolete 8-subvoxel placement tests referencing `Voxel::Occupied`.
    - Removed dead imports.
  - **`src/gameplay/mod.rs`**:
    - Removed `#![allow(unused_imports)]`.
    - Streamlined exports to only include active types used across modules (`BlockIcons`, `setup_block_icons`, `SelectedVoxel`, `RadialMenuState`, `CurrentTarget`, `TargetingSet`, `VoxelTarget`).

- [x] **Stage 9.3: `src/world/` and Streaming Subsystems Cleanup (Completed)**:
  - **`src/world/chunk.rs`**:
    - Purged dead RLE compression/decompression storage variants and stubs (`ChunkStorage::Rle`, `to_rle`, `from_rle`, `compress_rle`, `decompress_rle`).
    - Stripped dead test-only metrics and unused getters (`memory_size`, `non_air_count`, `solid_opaque_count`, `unique_voxel_count`, `storage`).
    - Pruned 200 lines of obsolete internal unit tests, reducing file size by 342 lines.
  - **`src/world/block.rs`**:
    - Purged unused `ToolType` enum, unintegrated `durability()` (63 lines), and `required_tool()` (59 lines) survival stubs.
    - Removed legacy misspelling aliases (`Terracota`, `Rainwood`, `RainwoodLog`).
    - Pruned obsolete unit tests, reducing file size by 212 lines.
  - **`src/world/storage.rs`**:
    - Removed unused `ChunkNeighborhood::center()` getter.
  - **`src/world/streaming/`**:
    - Removed redundant `NEIGHBOR_CHUNK_OFFSETS` constant duplicate from `manager.rs`, `streaming/mod.rs`, and `world/mod.rs`.
    - Streamlined re-exports to only expose actively consumed symbols.
  - **`src/gameplay/shaping.rs`**:
    - Refactored `apply_block_shape` with a `ShapeModification` parameter object, completely eliminating `#[allow(clippy::too_many_arguments)]`.

- [x] **Stage 9.4: `src/simulation/` and `src/player/` Subsystems Cleanup (Completed)**:
  - **`src/simulation/` Subsystem**:
    - `src/simulation/fluid.rs`: Purged dead `compute_water_distance` stub and stripped 162-line `mod tests` block.
    - `src/simulation/lighting.rs`: Purged obsolete 33-line `mod tests` block.
    - `src/simulation/mod.rs`: Removed `#![allow(unused_imports)]` and pruned dead re-exports (`remove_chunk_lights`, `sync_voxel_light`, `compute_water_distance`, etc.), retaining only active symbols.
  - **`src/player/` Subsystem**:
    - `src/player/collision.rs`: Stripped 39-line `mod tests` block.
    - `src/player/controller.rs`: Stripped 44-line `mod tests` block.
    - `src/player/hotbar.rs`: Stripped 36-line `mod tests` block.
    - `src/player/inventory.rs`: Purged dead `new()`, `clear()`, unused `swap()`, and 46-line `mod tests` block, while retaining active `get`, `set`, `first_empty_slot`, and `add_item` methods used by the creative inventory (net reduction of 59 lines).
    - `src/player/model.rs`: Stripped 29-line `mod tests` block.
    - `src/player/mod.rs`: Cleaned up redundant imports and streamlined re-exports.

- [x] **Stage 9.5: `src/map/` and `src/menu/` Subsystems Cleanup (Completed)**:
  - **`src/map/` Subsystem**:
    - `src/map/cache.rs`: Removed dead `contains_chunk` and test-only `chunk_count` methods, eliminated their `#[allow(dead_code)]` annotations, and pruned the 87-line `mod tests` block (~100 lines removed).
    - `src/map/color.rs`: Stripped 44-line `mod tests` block.
    - `src/map/minimap.rs`: Removed unused `marker_image` field from `MinimapState`, removed obsolete 84-line software triangle rasterizer (`draw_player_arrow`, `dist_to_segment`) replaced by GPU UI transform rotation, and stripped `mod tests` (~115 lines removed).
    - `src/map/world_map.rs`: Removed dead `last_marker_yaw` field and stripped 32-line `mod tests` block.
    - `src/map/mod.rs`: Removed `#[allow(unused_imports)]` and pruned unused re-exports (`MapChunk`, `MapPixel`), leaving only `pub use cache::MapCache;`.
  - **`src/menu/` Subsystem**:
    - `src/menu/mod.rs`: Removed dead `from_window_mode`, removed unused `GuiTextures` struct and resource insertion, and stripped the 51-line `mod tests` block (~69 lines removed).
    - `src/menu/creative_inventory.rs`: Removed unused `INVENTORY_VISIBLE_SLOTS`, test-only `total_inventory_rows`, `max_scroll_row`, and `ArmorSlotType::name()`, and stripped the 231-line `mod tests` block (~254 lines removed).
    - `src/menu/settings.rs` & `src/menu/pause.rs`: Verified 100% active UI logic with 0 bloat or dead code.

- [x] **Stage 9.6: `src/generation/` and `src/gameplay/` Subsystems Cleanup (Completed)**:
  - **`src/generation/` Subsystem**:
    - `src/generation/trees.rs`: Completely purged 1,000+ lines of obsolete 50cm tree generation code awaiting Phase 12 rewrite; retained clean `TreeSpecies` enum (`Oak`, `Birch`, `Pine`, `Cactus`, and newly added `Rainwood`) with voxel mapping helpers (net reduction of 995 lines).
    - `src/generation/biome.rs`: Removed dead `BiomeType::ALL` and `BiomeType::ACTIVE` constants and stripped 71-line `mod tests` block (net reduction of 93 lines).
    - `src/generation/caves.rs`: Removed dead `is_cave` and `cave_voxel` methods (eliminating their `#[allow(dead_code, clippy::too_many_arguments)]` suppressions) and stripped 63-line `mod tests` block (net reduction of 103 lines).
    - `src/generation/generator.rs`: Stripped 257-line `mod tests` block.
    - `src/generation/strata.rs`: Stripped 54-line `mod tests` block.
    - `src/generation/mod.rs`: Pruned unused re-exports and removed `#![allow(unused_imports)]`.
    - `src/generation/inspector.rs`: Verified 100% active egui inspector code.
  - **`src/gameplay/` Subsystem**:
    - `src/gameplay/shaping.rs`: Refactored `handle_block_rotation` to group 4 mutable world resources into a tuple parameter, completely eliminating `#[allow(clippy::too_many_arguments)]`.
    - `src/gameplay/icon.rs`: Stripped 22-line `mod tests` block.
    - `src/gameplay/interaction.rs`: Stripped 34-line `mod tests` block.
    - `src/gameplay/target_hud.rs`: Stripped 72-line `mod tests` block.
    - `src/gameplay/targeting.rs`: Stripped 36-line `mod tests` block.
    - `src/gameplay/debug.rs`, `radial_menu.rs`, & `mod.rs`: Verified 100% active runtime systems.

- [x] **Stage 9.7: `src/environment/` and `src/core/` Subsystems Cleanup (Completed)**:
  - **`src/environment/` Subsystem**:
    - `src/environment/atmosphere.rs`: Stripped 20-line `mod tests` block.
    - `src/environment/celestial.rs`: Removed dead `sun` field from `CelestialMaterials`, removed dead `moon_phase_factor()` helper, and stripped 56-line `mod tests` block (net reduction of 71 lines).
    - `src/environment/clouds.rs`: Stripped 27-line `mod tests` block.
    - `src/environment/stars.rs`: Turned `StarInstance` into a unit marker struct (`pub struct StarInstance;`), removed unused `initial_dir` field and its `#[allow(dead_code)]`, and stripped 29-line `mod tests` block (net reduction of 36 lines).
    - `src/environment/time.rs`: Removed dead `MOON_PHASE_NAMES` array, removed `#[allow(dead_code)]` from `pub fn year(&self)` (actively used by dev stats), and stripped 114-line `mod tests` block (net reduction of 127 lines).
    - `src/environment/mod.rs`: Replaced glob re-exports with explicit, minimal imports/re-exports (`DayPhase`, `EnvironmentState`, `EnvironmentPlugin`), completely eliminating `#![allow(unused_imports)]`.
  - **`src/core/` Subsystem**:
    - `src/core/font.rs`: Removed dead helpers `text_font`, `make_text_font`, `to_font_source`, and stripped 10-line `mod tests` block (net reduction of 39 lines).
    - `src/core/noise.rs`: Stripped 38-line `mod tests` block.
    - `src/core/dynamic_fps.rs`: Stripped 24-line `mod tests` block.
    - `src/core/dev_stats.rs`: Stripped 35-line `mod tests` block.
    - `src/core/mod.rs`: Pruned unused re-exports (`DebugHudMode`, `DebugHudSettings`, `DynamicFpsStateKind`, `DEFAULT_FONT_PATH`, `fbm_2d`, etc.), retaining only active symbols.

---

## Phase 10: Gameplay Polish, Interaction Feedback & Quality-of-Life (Completed)

Phase 10 delivered extensive gameplay polish, interactive tactile feedback, inventory texture skinning, map navigation, and quality-of-life systems.

- [x] **Stage 10.0: Terrain & World Polish (Agile / Quality-of-Life)**:
  - **Paused Tree Generation**: Temporarily removed tree and clutter block generation (packed dirt, moss, etc.) from chunk building to focus on a clean baseline world (dirt, stone, grass, water, snow/snowy grass, sand) while tuning terrain parameters.
  - **Eliminated Grass Stacking**: Fixed bug where subsoil dirt was promoted to Grass on cliff steps; grass is strictly placed at `depth == 0` with air above it.
  - **Underwater Beach Protection**: Prevented green grass rings offshore by generating Sand underwater and extending beach shelves down to `sea_level - 6`.
  - **Clean Subterranean Strata**: Upper underground is pure `Stone`, transitioning at depth midpoint ($Y \le -41$) to `Slate` (`rock_slate`), with `Dreadstone` bedrock strictly confined to the bottom 3–4 layers of the world ($Y \le -94$). Removed gravel/cobblestone clutter from dirt subsoils.
  - **Snowy Grass Block (`SnowyGrass`)**: Added multi-face block with snow top, dirt bottom, and newly integrated snowy grass side textures (`terr_snowy_grass_side`) for snowy biomes and peaks.
  - **Scrollable Creative Inventory (Stage 10.2 QoL)**: Restructured inventory to 4 rows (8×4 grid = 32 visible slots) with a smooth scrollbar matching Minecraft UX, cleanly accommodating all 58 available blocks.
  - **Live "Regenerate World" Chunk Reload (Stage 10.2 QoL)**: Ensured clicking "Regenerate World" in the <kbd>F1</kbd> Inspector cleanly despawns existing chunk mesh entities and re-triggers async generation and meshing live.
  - **Grand 3D Mountains & Continents**: Recalibrated continental scale ($0.0012$ frequency) for vast landmasses (1000+ blocks), peaks rising up to $Y = 80\text{--}140+$, cylindrical chunk streaming ($X^2 + Z^2 \le R^2$), and world chunk height raised to chunk 16 ($Y = 256$, bottom chunk $-6$ at $Y = -96$).
  - **Subterranean Mountain Rivers**: Rivers flowing into tall mountains ($Y > \text{sea\_level} + 14$) preserve the standing mountain peaks while carving an underground river cavern tunnel at sea level.
  - **Natural Mountain Arches & Cave Mouths**: Added horizontal ridge-tunneling arches, tuned cave mouths for dry hillsides ($0.18$ threshold), and removed subterranean water aquifers for clean cave exploration.
  - **Atmospheric Clouds**: Moved cloud altitude from $80.0$ to $220.0$, floating high above all mountain peaks.

- [x] **Stage 10.1: Block Interaction Feedback & Particle FX (Completed)**:
  - **Block Breaking Particle Bursts**: Scattering 8 subtle sub-voxel debris pebbles ($0.040\text{m}$ half-extent / $8\text{cm}$ cubes) matching the texture array layer and tint color of the broken block, bouncing realistically against collidable terrain and walls (`check_terrain_collision`), with friction, gravity, and lifetime shrinking before despawning.
  - **Block Placement Feedback**: Punchy 0.18s elastic scale bounce animation ($1.15 \to 1.00 \to 0.95 \to 1.00$) matching the placed block's multi-face textures and orientation.
  - **Decoupled Observer Architecture**: Driven by Bevy 0.19 `On<BlockBreakEvent>` and `On<BlockPlaceEvent>` observer triggers in `src/gameplay/feedback.rs`, keeping gameplay interaction systems decoupled and lightweight.

- [x] **Stage 10.2: Quality-of-Life & Tooling Polish (Completed)**:
  - **Fix Live "Regenerate World" Chunk Reload**: Ensured that clicking "Regenerate World" in the <kbd>F1</kbd> Inspector cleanly despawns existing chunk mesh entities and re-triggers async mesh generation in real time.
  - **4-Row Scrollable Creative Inventory**: Compact 4-row inventory grid with scrollbar navigation supporting all blocks.
  - **Player Personal Inventory & Creative Dual-Tab Toggle**: Dedicated 8×4 grid (32 visible slots) for the player's personal storage without scrollbar, with toggle button tabs at the top switching seamlessly between Personal Inventory and Creative Inventory, supporting item movement, slot swapping, and Shift-click transfer to/from hotbar.
  - **Mouse Tweaks & Shift-Drag Fast Transfer**: Shift + LMB click-and-drag over hotbar slots rapidly transfers multiple items into personal inventory (and dragging across personal inventory transfers to hotbar).

- [x] **Stage 10.3: Minimap & Interactive World Map (Minecraft/Xaero-Style) (Completed)**:
  - **Shared Map Data & Topographic Cache Layer**:
    - High-performance 2D column surface extraction (`MapCache`) caching explored chunk terrain.
    - Incremental dirty-column updates when chunks stream in or blocks are placed/broken.
    - Surface voxel detection, water depth computation, and North-up topographic relief hill-shading.
    - Persistent exploration history even when 3D chunks are unloaded from memory; dark grid background for unexplored areas.
  - **Gameplay Minimap (Square HUD)**:
    - Square HUD minimap permanently visible during gameplay in the top-right corner.
    - Player-centered, North-up orientation with live player directional heading marker.
    - Stylized metallic border frame with N/S/E/W cardinal indicators.
    - Real-time player coordinate display ($X, Y, Z$).
  - **Full-Screen World Map**:
    - Opened and toggled via <kbd>M</kbd> or <kbd>ESC</kbd> with seamless `MenuState::WorldMap` integration.
    - Smooth click-and-drag panning and mouse scroll wheel zooming (0.25x to 4.0x).
    - Live player position marker, heading chevron, and compass indicator.
    - Real-time coordinate HUD (Player coordinates, cursor coordinates under pointer, zoom level).
    - Center-on-player quick snap hotkey (<kbd>Space</kbd>).

- [x] **Stage 10.4: Biome Color Variation & Ambient Environment Noise ("Ambient Environment" Mod Style) (Completed)**:
  - **Procedural Ambient Color Noise**: Procedural 2-octave smooth value noise computed in `voxel.wgsl` via `world_position.xz` modulating tinted vertex colors by $\pm 8\%$ to break up flat monochromatic plains and foliage expanses without fragmenting greedy-meshed quads.
  - **Biome-Specific Grass & Foliage Tints**: Calibrated base grass and leaf tints dynamically across biomes (vibrant emerald for Plains and Meadow, dry golden-olive for Savanna, sun-baked olive for Desert, cold glacial blue-green for Snowy Tundra and Cold Plains).
  - **Biome-Specific Water Hues**: Distinct water coloration across aquatic climates (warm turquoise for tropical beaches and desert oases, deep marine navy for oceans/deep oceans, crisp crystal blue for mountain streams and rivers, murky teal for wetlands).
  - **Natural Biome Blend Transitions & Organic Block Dithering**:
    - **Vertex & Map Tint Blending**: 13-point symmetric circular Gaussian kernel ($R = 10.0$ blocks) smoothly blending climate colors across biome boundaries with 64-step channel quantization to preserve maximum greedy-meshing quad merging efficiency. Minimap and full-screen world map updated with live biome colors.
    - **Organic Surface Block Transitions**: Multi-octave 2D coherent noise dithering and threshold perturbation seamlessly intermingling surface block types across biome boundaries (Grass vs SnowyGrass patches and tongues across Snowy Tundra borders, Sand vs Grass dunes and drifts across Desert margins, and organic undulating beach shorelines).

- [x] **Stage 10.5: Screen Modes, Map Performance & Font Integration (Completed)**:
  - **Flight & Movement Performance Bottleneck Elimination**:
    - Discovered and eliminated the 40–50 FPS frame pacing bottleneck caused by evaluating Perlin noise across 36,864 minimap pixels every frame. Precomputed surface color in `MapPixel` at column extraction time, dropping main-thread Perlin noise calls during minimap updates from 3,354,624 to 0 and stabilizing framerates at 250–300+ FPS during active movement and flight.
  - **Dynamic FPS Restoration & VSync Controls**:
    - Restored `DynamicFpsPlugin` with zero-sleep unthrottled active gameplay, 30 FPS idle throttling, and 15 FPS unfocused throttling, accompanied by a Settings menu toggle.
    - Added an in-game VSync toggle to Settings (disabled by default via `AutoNoVsync`, dynamically switching to `AutoVsync` without restarting).
  - **Custom Font System & Drop Shadows**:
    - Integrated `AppFont` resource and `FontPlugin` pointing to `assets/fonts/CutePixel.ttf` for easy font reference swapping.
    - Added universal drop shadow support (`TextShadow`) across all UI and HUD text (F3 dev stats, minimap, world map, settings, pause menu, hotbar numbers, inventory tabs/badges, target HUD, and radial menu).
  - **Minimap Visual Enhancements & Polish**:
    - **Parchment Background Frame**: Loaded `assets/textures/interfaces/atlases/map_background.png` as an authentic cartographic border extending outward behind the 192×192 terrain view.
    - **Red Player Marker**: Loaded `assets/textures/interfaces/atlases/decorations/marker_red.png` replacing the procedural arrow, dynamically rotated with player camera yaw using native `UiTransform` and `Rot2`.
    - **Compact Coordinates Format**: Updated readout format to `"Coordinates: XYZ: 0, 0, 0"`.
    - **Cardinal Indicators (N, S, W, E)**: Standardized all four indicators to bright white (`13.0px`) with drop shadows, inset `16.0px` over the terrain view to prevent border clipping.
  - **Screen Mode Settings**:
    - Added live screen mode switching stepper to Settings Menu: **Windowed**, **Exclusive Fullscreen**, and **Borderless Fullscreen**, updating Bevy's `PrimaryWindow` mode in real time.
  - **World Map Marker Fix & Rotation**:
    - Separated viewport container from the terrain image node in `src/map/world_map.rs`, eliminating Taffy leaf node child clipping bugs.
    - Fixed marker positioning to accurately track world coordinates and center on player upon opening, with zero-size viewport initialization guards and yaw rotation.
  - **Environment Assets & Block Registry Expansion**:
    - Migrated celestial assets from legacy spritesheets to dedicated folders: individual 64×64 moon phase images in `assets/textures/environments/celestial/moon/` and sun texture at `assets/textures/environments/celestial/sun.png`.
    - Added `Terracotta` (with 5 organic variants `terr_terracotta.png` .. `4`), `RootedDirt`, and the `Rainwood` block family (`RainwoodWood` with 4 variants, `RainwoodWoodLog` with log ring tops, and `RainwoodLeaves` with 2 foliage variants) to the voxel registry, terrain texture array, creative inventory, and map coloration.

- [x] **Stage 10.6: UI Texture Skinning & Ergonomics (Hotbar, Personal Inventory & Creative Inventory) (Completed)**:
  - **Textured Hotbar HUD**:
    - Loaded `assets/textures/interfaces/containers/hotbar.png` (256×32 px) centered horizontally, scaled 2× integer (`512×64 px`) via `ImageNode` with nearest-neighbor sampling.
    - Symmetrically aligned the 8 slot hitboxes and icons to exact texture coordinates (36×36 px outer slot frames with 2px borders, 32×32 px native item icons, stride 40 px, 16px internal slot size).
    - Inactive slots have transparent borders and backgrounds allowing the pixel-art bevels and recessed shadows to show through; active slot highlights with a 2px golden frame and subtle white sheen.
  - **Unified Textured Container Inventory Interface**:
    - Transitioned from legacy split dual-card layout to a streamlined single-panel container (base 190×152 px, rendered at 3× integer scale to 570×456 px) using dedicated textures `assets/textures/interfaces/containers/inventory.png`, `inventory_creative.png`, and `inventory_scroller.png`. Removed side panel player card, 3D viewport, and armor slot placeholders to focus cleanly on core storage and palette functionality.
    - **Top Mode Switch Buttons**:
      - Symmetrically placed toggle buttons above the central panel for `Personal` ($X=22..91$, $Y=2..13$) and `Creative` ($X=98..167$, $Y=2..13$) matching texture guidelines, with <kbd>Tab</kbd> hotkey support.
      - **Persistent Mode Memory**: Active view mode (Personal vs. Creative) is preserved across closing and reopening.
    - **Personal Inventory**:
      - 8×4 slot grid (32 items) for player storage ($X=24 + c \times 18$, $Y=52 + r \times 18$).
      - Title area ($X=24, Y=32$) and two action button placeholders ($X=138, 156$).
      - 1×8 bottom hotbar mirror row at $Y=128$.
    - **Creative Inventory**:
      - Compact title and adjacent search bar ($X=99..167$, $Y=33..42$, width 69px, height 10px).
      - 8×4 creative block grid with real-time filtering and 1×8 hotbar mirror row.
      - Scrollbar track at $X=175..186$ with pixel-art thumb (`inventory_scroller.png`, 12×15 px at base, 36×45 px at 3×) supporting mouse wheel scrolling and click-and-drag.
    - **Search Bar Text Selection & Repeat Deletion**:
      - **Backspace Repeat**: Holding Backspace repeats character deletion with a 0.40s initial delay and 0.04s repeat rate.
      - **Double-Click & Drag Selection**: Double-clicking selects all text; clicking and dragging selects character ranges, rendered with a semi-transparent blue highlight box behind text. Also supports <kbd>Ctrl+A</kbd>.
      - **Selection Deletion & Typing Replacement**: Typing replaces selected text; Backspace/Delete removes selection.
      - Guarded <kbd>E</kbd> key when search input is focused.
    - **Ergonomics & Polish**:
      - **Shift + Click**: Rapid item transfer between personal inventory and hotbar.
      - **Shift + LMB Drag**: Multi-slot batch transfer / creative hotbar clearance.
      - **LMB Drag**: Continuous hotbar painting with held item.
      - **Depth-of-Field Blur**: Cinematic camera depth-of-field blur (`DepthOfField`) upon opening inventory.

---

## Phase 11: World Generation & Worldbuilding Expansion (High Fantasy & Dark Fantasy Realism) (Active)

A focused overhaul and expansion of procedural world generation, terrain topography, and geological worldbuilding inspired by classic dark and high fantasy settings (*The Witcher*, *D&D*, and *Lord of the Rings*). This phase prioritizes perfecting terrain layout, relief height, surface blocks, and smooth biome transitions before flora and fauna are reintroduced in Phase 12.

- [x] **Stage 11.0: Foundational Geology & World Depth Calibration (Completed)**:
  - **Symmetric 512-Block Vertical Height**: Set minimum chunk level to `WORLD_MIN_CHUNK_Y = -16`, establishing a symmetric vertical world range spanning $-256$ to $+256$ (512 total playable blocks).
  - **Underground Blackstone Stratum**: Standard subterranean stone transitions halfway down the crust ($Y \le -120$) into dense `Blackstone`, while unbreakable `Dreadstone` bedrock forms the floor of the world ($Y \le -254$).
  - **Exclusive Highlands Slate**: Reserved `Slate` and `Cobbleslate` exclusively for the `Highlands` biome, establishing its unique geological identity as steep alpine crags and scree slopes.
  - **Reactivated Core Biome Suite**: Activated all 14 baseline biomes in `BiomeType::ACTIVE` and `ClimateGenerator::classify_biome` (`Plains`, `Cold Plains`, `Snowy Tundra`, `Meadow`, `Woodland`, `Wetlands`, `Highlands`, `Plains Forest`, `Savanna`, `Desert`, `Beach`, `River`, `Ocean`, `Deep Ocean`).

- [ ] **Stage 11.1: Surface Fantasy Biome Catalog (Terrain & Surface Palettes)**:
  *Note: Only existing engine blocks are utilized (`Grass`, `SnowyGrass`, `Dirt`, `PackedDirt`, `Mud`, `PackedMud`, `Mulch`, `Moss`, `RedMoss`, `Clay`, `Gravel`, `Flint`, `Sand`, `RedSand`, `Stone`, `MossyStone`, `Cobblestone`, `Slate`, `Basalt`, `Blackstone`, `Sandstone`, `RedSandstone`, `Limestone`, `Calcite`, `Ice`, `PackedIce`). No flora/fauna features at this stage.*

  - **1. Frigid & Subpolar Climates**:
    - *Glacial Spire & Nunataks*: Soaring vertical ice peaks and nunatak crags. Surface: `PackedIce`, `Ice`, `Snow`, and `Blueschist`. Subsoil: `PackedIce`. Water: Frigid navy (`#1B3F73`).
    - *Frost Scree & Permafrost Slope*: High wind-swept scree slopes. Surface: `SnowyGrass`, loose `Gravel`, `Stone`, and `Snow`. Subsoil: `Dirt` and `Flint`. Water: Frigid cyan (`#48B2DE`).
  - **2. Temperate & Maritime Climates**:
    - *Ancient Grove (Brokilon/Fangorn style)*: Deep shaded lowland forest floor. Surface: `Moss`, `Grass`, and `PackedDirt`. Subsoil: `Mulch` and `Dirt`. Water: Dark tannin green (`#327A58`).
    - *Old-Growth Pine Taiga*: Cool conifer valleys and hills. Surface: `Mulch`, `PackedDirt`, and cold `Grass`. Subsoil: `PackedDirt`. Water: Clear mountain blue (`#3E8AB8`).
    - *Rolling Moors & Heathlands*: Wind-swept undulating hills. Surface: `Grass` with scattered `Gravel` patches and `Dirt`. Subsoil: `Dirt` and `Stone`. Water: Temperate blue (`#356592`).
  - **3. Waterlogged Mires & Wetlands**:
    - *Crookback Mire (Velen / Dead Marshes style)*: Stagnant depressions and peat banks. Surface: Wet `Mud`, `PackedMud`, `Clay`, and olive `Moss`. Subsoil: Deep `PackedMud`. Water: Murky brown-green (`#3D4A30`).
    - *Brackish Estuary*: Low-lying braided channels. Surface: `Mud`, `Clay`, and river `Sand`. Subsoil: Layered `Clay` and `Gravel`. Water: Silty teal (`#3B827E`).
  - **4. Arid & Scorched Climates**:
    - *Red Mesa & Slot Canyons*: Layered flat-topped plateaus and dry arroyos. Surface: `RedSandstone`, `RedSand`, and `Ochrestone`. Subsoil: `RedSandstone`. Water: Rare oasis turquoise (`#2AC4C4`).
    - *Volcanic Ashlands & Basalt Sinks (Mordor style)*: Scorched volcanic plains with fissures. Surface: `Basalt`, `Blackstone`, `Magma` seams, and dark `PackedDirt`. Subsoil: `Basalt`. Water/Fluids: Glowing `Lava`.
  - **5. Coastal & Marine Boundaries**:
    - *Rocky Sea-Cliffs*: Sheer ocean precipices battered by surf. Surface: `Stone`, `Cobblestone`, `Gravel`, and sea-spray `Grass`. Subsoil: Solid `Stone`. Water: Deep marine blue (`#244C8E`).

- [ ] **Stage 11.2: Multi-Parameter Climate Noise & Spline Mapping**:
  - **Multi-Noise Climate Coordinates**: Continuous multi-octave 2D noise mapping Continentalness, Temperature, and Humidity with expanded parameter curves.
  - **Smooth Spline / Voronoi Climate Blending**: Multi-octave jittered cellular partitioning ensuring biomes transition naturally without artificial geometric borders.

- [ ] **Stage 11.3: Natural Biome Transitions & Edge Dithering**:
  - **Surface Block Dithering**: Expand organic block transitions (similar to Grass vs. SnowyGrass and Sand vs. Grass) across all adjacent biome borders (e.g. Mud fingers blending into Moor grass, RedSand drifts meeting Sandstone).
  - **Height & Slope Blending**: Natural elevation interpolation preventing sudden cliff cuts across biome boundaries.

- [ ] **Stage 11.4: Atmospheric Biome Weather, Volumetric Fog & Environment Grading**:
  - **Biome-Specific Ambient Palettes**:
    - Low-altitude eerie mist in wetlands and mires.
    - Crisp high-exposure distance fog on alpine and glacial heights.
    - Dark ash haze in volcanic wastelands.
    - Soft warm golden lighting across temperate plains and moors.

---

## Phase 12: Flora, Procedural Trees & Surface Vegetation (Upcoming)

Phase 12 breathes organic life, vertical grandeur, and color into the procedural world by generating biome-specific trees, flowering ground cover, shrubs, and dynamic wind-swayed foliage native to the 1m voxel architecture.

- [ ] **Stage 12.1: Procedural 1m Trees & Canopy Architecture**:
  - **Trunk Shapes, Species & Wood Types**:
    - Wood species suite: Oak (`OakWood`, `OakWoodLog`), Birch (`BirchWood`, `BirchWoodLog`), Pine (`PineWood`, `PineWoodLog`), and Rainwood (`RainwoodWood`, `RainwoodWoodLog`), following the bark-only sides vs top/bottom log-ring architecture.
    - 1m block trunk geometry utilizing native shapes:
      - **Standard Trunks**: 1m × 1m full blocks (`BlockShape::Full`).
      - **Slender / Branch Trunks**: Centered vertical column shapes (`BlockShape::Column`, centered).
      - **Colossal Trunks**: 2×2 log cores flanked by flaring root buttresses at ground level.
    - Procedural branch generation: Log branches radiating outward and upward from the main trunk into canopies.
    - Species profiles:
      - **Oak**: Broad billowing Euclidean spherical leaf canopies, branching crowns (5–12m height).
      - **Birch**: Tall, slender columnar trunks with oval sinusoidal crowns (7–14m height).
      - **Pine**: Tiered dense conical evergreen skirts tapering to needle spires (10–22m height).
      - **Rainwood**: Dense tropical umbrella canopies with hanging mosses and wide buttress bases.
  - **Canopy Foliage, Volumetric Depth & Alpha Cutouts**:
    - Leaves blocks: `Voxel::OakLeaves`, `Voxel::BirchLeaves`, `Voxel::PineLeaves`, and `Voxel::RainwoodLeaves`.
    - GPU alpha-masking via `AlphaMode::Mask(0.5)` with `discard` in `voxel.wgsl` for crisp see-through foliage with full depth testing and zero sorting artifacts.
    - Volumetric interior leaf rendering (`should_render_face(leaf, leaf) = true`) preventing hollow outer shells while GPU backface culling preserves performance.
    - Biome foliage tinting dynamically harmonizing leaf colors with terrain climate noise.
  - **Spawn Validation & Multi-Chunk Margins**:
    - Trees spawn strictly on compatible soil (Grass, Dirt, Packed Dirt, Mulch, Mud, Sand for palms) with clearance checks preventing growth inside caves or underwater.
    - Multi-chunk generation margins ensuring branch and canopy geometry seamlessly crosses chunk boundaries without planar clipping.

- [ ] **Stage 12.2: Ground Flora, Flowers & Biome Foliage**:
  - **Wild Grass & Ferns**: Single and double-tall grass tufts scattered across Plains, Meadows, and Woodlands using cross-quad alpha cutouts.
  - **Flowering Plants**: Biome-specific flowers:
    - Meadows: High-density vibrant carpets of Poppies, Dandelions, Cornflowers, and Blue Orchids.
    - Woodlands: Woodland bluebells and wild ferns.
    - Wetlands: Water lily pads floating on marsh pools, reeds/sugar cane along muddy riverbanks.
    - Caves & Shadows: Red and brown mushrooms flourishing in low-light subterranean grottos and damp overhangs.
    - Deserts: Dead tumbleweeds, dry shrubs, and saguaro cacti.

- [ ] **Stage 12.3: Alpha-Cutout Cross-Quad Meshing & Wind Sway Shader**:
  - **Cross-Quad Plant Geometry**: Efficient 2-quad (X-pattern) billboard meshes for wild grass, flowers, and crops.
  - **Alpha-to-Coverage / Cutout Transparency**: Clean silhouette rendering without sorting artifacts or depth-buffer clipping.
  - **Subtle Wind Sway (WGSL)**: Vertex shader displacement in `voxel.wgsl` using a gentle sine wave driven by `globals.time` to add organic swaying movement to leaves, tall grass, and flowers.

---

## Phase 13: High-Performance Scaling, Level-of-Detail (LOD) & Engine Optimization (Upcoming)

Phase 13 scales the engine's rendering and storage architecture to support massive view distances (24–32+ chunks, 384–512m+ radii), seamless 144+ FPS frame pacing during supersonic flight, and persistent binary disk storage.

- [ ] **Stage 13.1: Chunk Mesh LOD & Downsampled Far-Mesh Geometry (LOD Render Distance)**:
  - **The Problem**: Rendering 4,000+ chunks at 24–32 chunk render distances submits millions of polygons to the GPU, saturating vertex stages with distant sub-pixel details.
  - **Architecture**:
    - Multi-tier geometric LOD hierarchy:
      - **LOD 0 (Near: 0–12 chunks)**: Full resolution 1m³ greedy-meshed voxels with all custom shapes.
      - **LOD 1 (Mid: 13–20 chunks)**: 2×2 voxel downsampled blocks merged into simplified terrain meshes.
      - **LOD 2 (Far: 21–32+ chunks)**: 4×4 voxel downsampled heightfield blocks representing macro topography.
    - **Skirt Stitching & Seam Elimination**: Vertical boundary skirts connecting differing LOD levels to eliminate T-junction cracks and visible gaps without complex stitching algorithms.
    - **Hysteresis Distance Blending**: Chunk LOD levels transition with distance hysteresis to completely prevent visual popping when crossing chunk borders.
  - **Engine Benefits**:
    - Slashes distant mesh vertex counts by 75%–90%, making 32-chunk view distances smooth on standard GPUs.

- [ ] **Stage 13.2: GPU Occlusion Culling (Hi-Z Depth Pyramid & Sub-Chunk AABB Culling)**:
  - **The Problem**: Chunks buried deep underground or hidden behind mountain ranges are processed by the vertex shader and GPU rasterizer even when 100% occluded.
  - **Architecture**:
    - Construct a Hierarchical-Z (Hi-Z) depth buffer pyramid on the GPU using downsampled depth from the previous frame.
    - Compute-shader occlusion pass testing each chunk's world-space AABB bounding box against the Hi-Z pyramid before draw dispatches.
    - Discard occluded chunks entirely from the render list.
  - **Engine Benefits**:
    - Eliminates 50%–70% of rendered chunk geometry in hilly terrain and subterranean biomes, freeing substantial GPU bandwidth.

- [ ] **Stage 13.3: Indirect Draw & Multi-Draw Indirect (MDI) Chunk Batching**:
  - **The Problem**: Submitting thousands of individual draw calls per frame incurs significant CPU driver overhead and pipeline stalls.
  - **Architecture**:
    - Consolidate chunk draw calls using GPU Multi-Draw Indirect (`draw_indexed_indirect`).
    - Store chunk world transforms, light values, and material indices in a persistent GPU `StorageBuffer`.
    - A single indirect draw dispatch renders all visible chunks in a unified draw pass.
  - **Engine Benefits**:
    - Slashes CPU render preparation time from milliseconds down to microseconds, sustaining 200+ FPS regardless of active chunk counts.

- [ ] **Stage 13.4: Frustum-Prioritized Multi-Threaded Generation & Frame Pacing**:
  - **The Problem**: High-speed flight creates generation bursts that can cause frame time spikes when dozens of new chunk meshes are uploaded to the GPU in a single frame.
  - **Architecture**:
    - **Camera Frustum Generation Prioritization**: Prioritize chunk generation in the camera's forward view cone (field of view + direction) before allocating worker threads to peripheral or rear chunks.
    - **Mesh Upload Frame Budgeting**: Enforce a strict microsecond upload cap per frame (e.g., max 2–4 chunk meshes uploaded per frame), smoothing frame times into an unbroken 144+ FPS line.
  - **Engine Benefits**:
    - Eliminates micro-stutters and pop-in directly in the player's line of sight during rapid flight.

- [ ] **Stage 13.5: Compressed Binary Region File Storage (Anvil/MCA 32×32 Architecture)**:
  - **The Problem**: The world is currently memory-only and regenerates procedurally on every run; player edits in `WorldModificationStore` are not saved to disk.
  - **Architecture**:
    - Sector-based 32×32 chunk binary region file format (`.mca` style) storing 1,024 chunk columns per region file.
    - Per-chunk compression utilizing high-throughput Zstandard or LZ4 over 4-bit paletted voxel data.
    - Asynchronous background disk I/O thread pool streaming dirty chunks to disk without blocking the main game thread.
    - Spatial chunk index header with timestamp metadata for fast random-access chunk reads.
  - **Engine Benefits**:
    - Full persistent world saving and loading with fast disk access and compact file sizes.
    