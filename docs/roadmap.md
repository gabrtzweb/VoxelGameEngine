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

## Phase 6: Advanced World Generation, Biomes & Caves
- [x] **Multi-Noise Biome System**:
  - Macro-scale continuous 2D climate noise in [src/voxel/biome.rs](VoxelGameEngine/src/voxel/biome.rs) for Continentalness, Temperature, and Humidity.
  - 6 distinct biomes with individual surface, subsoil, and elevation profiles:
    - **Plains**: Temperate, moderate humidity, rolling green hills, grass surface, dirt sublayer.
    - **Desert**: Warm, arid, wind-swept sand dunes, sandstone/sand sublayer.
    - **Snowy Tundra & Frost Peaks**: Frigid high altitudes, snow-covered surface, frost-cracked stone.
    - **Wetlands / Swamps**: Low-lying coastal flats, high moisture, mud, packed mud, clay beds, shallow water.
    - **Rocky Highlands**: Rugged mountain ridges, exposed slate, cobbleslate, flint, and scree slopes.
    - **Woodland**: High humidity, mulch forest floors, packed dirt, mossy stone.
- [x] **3D Caves & Underground Caverns**:
  - High-performance native 3D gradient noise in [src/voxel/caves.rs](VoxelGameEngine/src/voxel/caves.rs).
  - Spaghetti worm tunnels via dual-noise zero-crossing intersections ($|\text{NoiseA}| < t \land |\text{NoiseB}| < t$).
  - Cheese caverns creating large subterranean chambers and grottos.
  - Subterranean water aquifers below sea level, and deep magma/lava pools at the lowest crust boundaries.
  - Surface attenuation buffer preserving flat surface plains while allowing cave entrances at steep cliffs.
- [x] **Realistic Geological Strata & Mineral Vein Deposits**:
  - Depth-based geological layering in [src/voxel/strata.rs](VoxelGameEngine/src/voxel/strata.rs):
    - Topsoil & Subsoil: Biome-specific topsoil and subsoil down to 4 logical blocks.
    - Upper Crust: Standard Stone with gravel pockets, flint veins, and cobblestone fractures.
    - Mid Crust: Metamorphic transition into Slate, Cobbleslate, and Flint clusters.
    - Deep Crust: Volcanic plutonic layer of Blackstone, Cobbleblackstone, Magma veins, and molten pools.
  - 3D mineral deposit noise quantized to 1m³ logical blocks for perfect sub-voxel material consistency.
- [] **Runtime Generation Controls & World Inspector Integration**:
  - Registered `TerrainGenerator`, `ClimateGenerator`, `CaveGenerator`, `StrataGenerator`, and `BiomeType` with Bevy's `AppTypeRegistry`.
  - Live parameter tuning in `bevy_inspector_egui` (<kbd>F1</kbd>) with automatic chunk reloading and remeshing while preserving player modifications.
  (NOT WORKING)
- [x] **Extended Debug HUD & Spawn Safety Polish**: 
  - Extended Debug HUD (<kbd>F3</kbd>) displays active Biome name, continentalness, temperature, and moisture metrics.
  - Dynamic spawn height calculation in [src/player/mod.rs](VoxelGameEngine/src/player/mod.rs) ensuring safe arrival on solid surface ground.

---

## Phase 7: Engine Optimization & Scalability (Future Milestone)
- **LOD Render Distance**: Downsampled greedy meshes for distant chunks (maybe not a priority = can be skipped).
- **Extremity Bound Checking**: Early skipping of completely empty or solid chunks during collision and meshing.
- **Noise Up-sampling & Caching**: Coarse 3D noise sampling with trilinear interpolation.
- **RLE Runtime Voxel Data**: Run-Length Encoded chunk storage to minimize memory footprint.

## Phase 8: To be decided
