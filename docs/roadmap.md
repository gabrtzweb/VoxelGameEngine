# Voxel Game Engine - Development Roadmap

This document outlines the planned development phases for the voxel game engine, with foundational architectural systems, gameplay tools, fluids, and procedural generation.

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
- [x] **8 Moon Phases from Spritesheet**:
  - Automatically slices the 128×64 spritesheet (`assets/textures/environments/moon_phases.png`) into 8 discrete 32×32 pixel textures (Full Moon, Waning Gibbous, Third Quarter, Waning Crescent, New Moon, Waxing Crescent, First Quarter, Waxing Gibbous).
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
    - Classic humanoid limb hierarchy (Head, Torso, Left/Right Arm, Left/Right Leg) mapped to standard Minecraft 64×64 skin textures (`assets/textures/mobs/player_skin.png`), supporting both base skin and 3D outer overlays (hat/hair, jacket, sleeves, pants).
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

## Phase 8: Engine Optimization & Scalability (Next Milestone)

Phase 8 focuses on deep algorithmic and memory optimizations to scale chunk throughput, slash generation latency, and compress the memory footprint for high render distances and smooth 144+ FPS gameplay.

### Strategic Priorities & Recommended Order
1. **Priority 1: Extremity Bound Checking & Chunk Homogeneity Metadata** (Highest ROI / Immediate O(1) skips across meshing, collision, and raycasting)
2. **Priority 2: Noise Up-Sampling & 3D Trilinear Interpolation** (Massive generation speedup, 97% reduction in 3D noise evaluations per chunk)
3. **Priority 3: RLE & Paletted Runtime Voxel Data + Cache Locality Layout** (Drastic RAM reduction, cache-friendly iteration, foundations for disk saves)
4. **Priority 4: Decoupled Simulation Radius vs. Render Distance** (Simulate fluids & ticking only in inner 4–6 chunks; outer 12–16 chunks remain static meshes)
5. **Priority 5: Bitwise Bitmask Acceleration for Face Culling & Greedy Mesher** (64-bit bitboards for sub-millisecond chunk meshing)
6. **Priority 6: Static Lookup Tables (LUTs) for Shape Transforms & Face Offsets** (Eliminates runtime coordinate math in inner loops)
7. **Priority 7: LOD Render Distance** (Lowest priority; deferred or simplified due to T-junction boundary seams and low GPU vertex bottlenecks)

---

### Detailed Analysis & Implementation Breakdown

- [ ] **Stage 8.1: Extremity Bound Checking & Chunk Homogeneity Flags**:
  - **The Problem**: Currently, collision checks, raycasting, and meshing still traverse coordinate ranges inside chunks that are 100% open sky (`Air`) or 100% subterranean rock (`Stone`/`Slate`). Although greedy meshing has early-exit counts, player collision tests (`overlapping_solid_voxels`) and targeting raycasts still query chunk storage coordinate by coordinate.
  - **Architecture**:
    - Introduce chunk state metadata: `ChunkHomogeneity::Empty` (100% Air), `ChunkHomogeneity::Solid(Voxel)` (100% single solid material), or `ChunkHomogeneity::Mixed`.
    - Maintain non-air voxel count and unique voxel variant counters during procedural generation and runtime edits in O(1).
  - **Engine Benefits**:
    - **Meshing**: Completely bypass chunk mesher task spawning for `Empty` chunks and fully occluded `Solid` chunks surrounded by solid neighbors (zero background tasks scheduled, zero memory allocations).
    - **Raycasting**: O(1) skip across empty chunks during line-of-sight ray traversal; O(1) hit on bounding box faces of solid chunks without voxel-level ray marching.
    - **Collision**: Player movement queries can immediately skip empty chunks without iterating over coordinate ranges.
  - **Complexity / Risk**: Low complexity, zero visual trade-offs, immediate CPU saving.

- [ ] **Stage 8.2: Noise Up-Sampling & Caching (Trilinear Interpolation)**:
  - **The Problem**: Procedural chunk generation evaluates complex multi-octave 3D Simplex/Perlin noise (caves, worm tunnels, cheese chambers, strata veins) independently for all 4,096 voxels in a chunk. This is the single largest CPU load on the `AsyncComputeTaskPool`, causing thread pool starvation during fast flight or streaming spikes.
  - **Architecture**:
    - Compute 3D cave/density noise only at a coarse lattice of sample points (e.g. 4×4×4 or 2×2×2 voxel cells) within the chunk grid.
    - For each cell, evaluate the 8 corner lattice points, then interpolate the inner 64 voxel densities using fast trilinear interpolation (`lerp` across X, Y, Z).
    - Leverage SIMD / vectorized math for the interpolation pass.
  - **Engine Benefits**:
    - Reduces expensive 3D noise evaluations from **4,096 down to 125 samples per chunk** (a **97% reduction** in mathematical noise evaluations!).
    - Drastically accelerates async chunk generation speed, eliminating chunk streaming pop-in during flight.
  - **Complexity / Risk**: Moderate complexity. May slightly smooth sharp micro-crevices in caves, but in practice yields more organic and aesthetically pleasing cave tunnels with virtually zero visual degradation.

- [ ] **Stage 8.3: RLE Runtime Voxel Data, Paletted Storage & Cache Locality**:
  - **The Problem**: Every loaded chunk currently stores a flat `[Voxel; 4096]` array (4,096 bytes). At render distance 10–12, several thousand chunks are held in memory simultaneously, consuming tens of megabytes of uncompressed RAM and causing cache pressure during iteration.
  - **Architecture**:
    - Implement a two-tiered paletted chunk representation:
      - `ChunkStorage::Uniform(Voxel)`: 1 byte of data for 100% Air or 100% Stone chunks.
      - `ChunkStorage::Paletted`: Chunks with <= 16 distinct block types use 4-bit indices pointing into a local 16-element palette (shrinking 4,096 bytes down to ~2,048 bytes).
      - `ChunkStorage::Rle(Vec<(Voxel, u16)>)`: Run-Length Encoded runs for layered horizontal strata and cave air pockets.
      - `ChunkStorage::Dense(Box<[Voxel; 4096]>)`: Flat uncompressed buffer used only during active multi-voxel player editing or when complexity warrants.
    - **Cache Locality Optimization**: Ensure chunk indexing order matches CPU L1 cache line stride (64 bytes) during raycast marching and meshing passes.
  - **Engine Benefits**:
    - Cuts overall world memory footprint by **70% to 85%**.
    - Prepares the data structures directly for fast binary disk serialization (world saving and loading).
  - **Complexity / Risk**: Moderate. Needs careful abstraction so `get(x, y, z)` and `set(x, y, z)` remain fast and inline-friendly without branch mispredictions.

- [ ] **Stage 8.4: Decoupled Simulation Radius vs. Render Distance**:
  - **The Problem**: When the player raises render distance to 12 or 16 chunks, the world holds 2,000+ active chunks. Running fluid propagation, cellular automaton ticks, and dynamic updates across all loaded chunks wastes CPU cycles on distant, non-visible activity.
  - **Architecture**:
    - Decouple `simulation_distance` (default: 4–6 chunks, ~32–48m radius around player) from visual `render_distance` (10–16+ chunks).
    - Chunks within the simulation radius actively process fluid ticks, falling sand, and dynamic neighbor updates. Chunks beyond the simulation radius remain purely static mesh renderables.
  - **Engine Benefits**:
    - Caps active fluid/simulation CPU budget to a fixed, small local bubble regardless of how high the player sets their visual render distance.
  - **Complexity / Risk**: Low complexity. Requires a simple radius test when scheduling simulation ticks.

- [ ] **Stage 8.5: Bitwise Bitmask Acceleration for Face Culling & Greedy Mesher**:
  - **The Problem**: Greedy meshing checks adjacent voxel solid/air states through millions of individual 3D index calls in nested loops.
  - **Architecture**:
    - Represent each 16-voxel row or 16×16 slice as 64-bit integer bitboards (`u64`).
    - Compute exposed face visibility across entire rows simultaneously using bitwise boolean operations:
      `visible_faces = current_row & ~neighbor_row`.
    - Extract contiguous runs using CPU intrinsic instructions (`trailing_zeros`, `leading_zeros`) to feed directly into quad generation.
  - **Engine Benefits**:
    - Cuts CPU time spent in face extraction by **4x–8x**, enabling near-instantaneous chunk remeshes when placing or breaking blocks.
  - **Complexity / Risk**: Moderate. Requires low-level bitwise manipulation logic.

- [ ] **Stage 8.6: Static Lookup Tables (LUTs) for Shape Transforms & Face Offsets**:
  - **The Problem**: Sub-voxel shaping tools (<kbd>R</kbd>), block rotations (<kbd>T</kbd>), and face normal transforms perform coordinate arithmetic and rotation matrix operations at runtime.
  - **Architecture**:
    - Precompute static lookup tables for all 10 sub-voxel shapes across 4 rotation orientations (`[SubVoxelMask; 40]`).
    - Precompute face normal vectors, UV quadrant offsets, and neighbor chunk coordinate offsets into `const` LUT arrays.
  - **Engine Benefits**:
    - Replaces runtime trigonometric calculations and branch trees with zero-cost table lookups.
  - **Complexity / Risk**: Very low. Standard compile-time constant arrays.

- [ ] **Stage 8.7: LOD Render Distance (Downsampled Greedy Meshes for Distant Chunks)**:
  - **Status**: Kept as lowest priority / deferred.
  - **Analysis**: Co-planar greedy meshing already merges flat terrain into minimal quads. Geometric LOD introduces T-junction cracks and boundary seam artifacts with minimal performance upside at render distance 12–16. Re-evaluate if render distance expands to 24–32+ chunks in future milestones.

---

## Phase 9: Flora, Procedural Trees & Surface Vegetation

Phase 9 breathes organic life and color into the procedural world by generating biome-specific trees, flowering ground cover, shrubs, and dynamic wind-swayed foliage.

- [ ] **Stage 9.1: Procedural Trees & Canopy Architecture**:
  - **Trunk & Branch Structure**: Multi-block vertical and branching wood logs (`Voxel::OakWood`, `Voxel::BirchWood`, `Voxel::PineWood`, `Voxel::PalmWood`).
  - **Leaf Canopy Generators**:
    - **Oak Trees** (Woodland / Plains): Sturdy 4–6 block trunks topped with rounded spherical/ellipsoid leaf crowns (`Voxel::OakLeaves`).
    - **Birch Trees** (Meadow / Plains): Slender white-barked trunks with light, airy leaf clusters.
    - **Pine & Spruce Trees** (Snowy Tundra / Mountain Foothills): Tall conical/pyramidal needle canopies with snow-dusted variants.
    - **Palm Trees** (Beach / Coastlines): Gently curved, sloped trunks leaning toward the water with fan-like palm fronds.
    - **Swamp Willows** (Wetlands): Wide gnarly trunks with hanging moss and vines draped over marsh water.
    - **Desert Cacti** (Desert): Columnar saguaro cacti with right-angled branching arms.
  - **Spawn Validation**: Trees spawn strictly on compatible soil (Grass, Dirt, Packed Dirt, Sand for palms) with clearance checks preventing growth inside caves or underwater.

- [ ] **Stage 9.2: Ground Flora, Flowers & Biome Foliage**:
  - **Wild Grass & Ferns**: Single and double-tall grass tufts scattered across Plains, Meadows, and Woodlands using cross-quad alpha cutouts.
  - **Flowering Plants**: Biome-specific flowers:
    - Meadows: High-density vibrant carpets of Poppies, Dandelions, Cornflowers, and Blue Orchids.
    - Woodlands: Woodland bluebells and wild ferns.
    - Wetlands: Water lily pads floating on marsh pools, reeds/sugar cane along muddy riverbanks.
    - Caves & Shadows: Red and brown mushrooms flourishing in low-light subterranean grottos and damp overhangs.
    - Deserts: Dead tumbleweeds and dry shrubs.

- [ ] **Stage 9.3: Alpha-Cutout Cross-Quad Meshing & Wind Sway Shader**:
  - **Cross-Quad Plant Geometry**: Efficient 2-quad (X-pattern) billboard meshes for wild grass, flowers, and crops.
  - **Alpha-to-Coverage / Cutout Transparency**: Clean silhouette rendering without sorting artifacts or depth-buffer clipping.
  - **Subtle Wind Sway (WGSL)**: Vertex shader displacement in `voxel.wgsl` using a gentle sine wave driven by `globals.time` to add organic swaying movement to leaves, tall grass, and flowers.

---

## Phase 10: Gameplay Polish, Audio Foundation & Quality-of-Life Tweaks

Phase 10 delivers core game feel improvements, sensory feedback, and quality-of-life additions.

- [ ] **Stage 10.1: Block Interaction Feedback & Particle FX**:
  - **Block Breaking Particle Bursts**: Scattering sub-voxel debris particles matching the texture of the broken block, bouncing briefly before fading out.
  - **Block Placement Feedback**: Subtle scale pop / bounce animation and placement dust puff.
  - **Sound Event Hooks**: Audio trigger events for:
    - Footstep sounds by surface type (Grass, Stone, Sand, Wood, Snow, Water wading).
    - Block breaking and placement audio (crunchy dirt, resonant stone, snappy wood, splashing water).
    - Ambient wind gusts on mountain summits and subterranean cavern echoes.

- [ ] **Stage 10.2: Quality-of-Life & Inspector Live Reload Fix**:
  - **Fix Live "Regenerate World" Chunk Reload**: Ensure that clicking "Regenerate World" in the <kbd>F1</kbd> Inspector cleanly despawns existing chunk mesh entities and re-triggers async mesh generation in real time.
  - **Clean Screenshot Hotkey (<kbd>F11</kbd> / <kbd>F7</kbd>)**: Captures high-res screenshots while temporarily hiding all HUD elements, crosshairs, and inspector windows.
  - **Block Item Drops / Hand Bob**: Floating rotating mini-block pickups when blocks are broken in survival/adventure context, and subtle hand swing animation when placing or breaking blocks.

- [ ] **Stage 10.3: World Persistence & Binary Save/Load Foundation**:
  - **Chunk Region File Format**: Simple binary serialization format storing modified chunk data in a dedicated world save folder.
  - **Save on Exit & Auto-Save**: Seamlessly serializes player block edits and inventory state, restoring the player's world exactly as built upon launch.

