# Voxel Game Engine - Current General Project Context

This is an attempt to develop an experimental voxel engine and game, built from scratch using Rust and Bevy.

## Technologies
The stack I am using for my project:
- Rust
- Bevy 0.19
- Bevy ECS
- Bevy Inspector Egui 0.37
- Image 0.25
- Winit 0.30
- Git/GitHub

## Current World Scale
- The world is stored using 0.5 m voxels.
- A traditional 1 m³ logical block is composed of 2 × 2 × 2 voxels.
- Each 1 m³ block therefore contains 8 individually editable voxels.
- Individual 0.5 m voxels can be destroyed and placed at runtime (alternative mode, B key).
- The 1 m³ logical block can also be destroyed and placed at runtime (default mode).
- 1 m blocks remain useful as a visual, gameplay, and coordinate abstraction.
- Each chunk contains: 16 × 16 × 16 voxels
- Total voxel capacity per chunk: 4,096 voxels
- Physical chunk size: 8 m × 8 m × 8 m
- The world streams procedurally and is effectively unlimited horizontally:
    - X: procedural streaming
    - Z: procedural streaming
- Vertical world limits are currently:
    - Minimum chunk Y: -10 (-160 voxels / -80 logical blocks)
    - Maximum chunk Y: +10 (+175 voxels / +87 logical blocks)
- Bottom-most layer of blocks: 100% unbreakable Dreadstone bedrock, blending naturally into Blackstone across the bottom 3 block layers.
- Chunk streaming operates in three dimensions around the player.
- Current default render distance: 12 chunks (distance fog disabled by default).
- The desired chunk region uses spherical distance rather than loading a full cube.
- Chunks are generated and unloaded dynamically as the Creative player moves through the world.

The engine focuses on a fully editable procedural voxel world with a hybrid block structure. It takes inspiration from voxel games like Minecraft but features its own architecture and mechanics.

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
- Minecraft 64×64 skin compatible humanoid mesh hierarchy (Head, Torso, Left/Right Arm, Left/Right Leg) mapped to `assets/textures/mobs/player_skin.png`
- Full support for base skin and 3D outer layers (hat/hair, jacket, sleeves, pants) with alpha masking
- Procedural locomotion: dynamic walk/sprint leg & arm pendulum swings, idle breathing sway, crouch torso tilt, prone crawling strokes, streamlined flutter-kick swimming, airborne jump poses, and 4-state flight animations (ascent, descent, fast flight, hover)

Game Modes Currently Implemented:
- Creative (currently the main gameplay and development mode)
- Spectator (noclip flight mode)

The player uses a custom AABB collision system that directly queries voxel data. Individual physics colliders are not created for terrain voxels.

---

## Implemented Architecture & Features by Domain

### 1. Core Voxel Architecture & World Representation
- **Hybrid Voxel-Block Model**: The world is stored using 0.5 m physical voxels. A traditional 1 m³ logical block is composed of 2 × 2 × 2 (8) individually editable voxels. Supports both 1 m³ block manipulation (default) and 0.5 m sub-voxel editing (<kbd>B</kbd> key).
- **Chunk Geometry**: 16 × 16 × 16 voxels (4,096 voxels per chunk) spanning 8 m × 8 m × 8 m physical space.
- **Procedural 3D Streaming**: Dynamic spherical chunk streaming in X, Y, and Z around the player. Default render distance of 8 chunks (configurable 2..=16 chunks in settings). Configurable vertical limits (currently chunks Y = -8 to +7).
- **Block Registry & Properties**: Dedicated blocks architecture in `src/world/block.rs` supporting 38+ block types, texture IDs, tool tiers (Pickaxe, Shovel, Axe), and material durability values.

### 2. Meshing & GPU Rendering Pipeline
- **Asynchronous Greedy Meshing**: Chunk meshing offloaded to Bevy's `AsyncComputeTaskPool` with background worker tasks and throttled main-thread mesh uploading (`src/meshing/async_mesher.rs`), eliminating frame-rate drops.
- **Custom WGSL Voxel Shader**: Extended PBR material (`ExtendedMaterial<StandardMaterial, VoxelMaterialExtension>`) preserving PBR lighting, directional shadows, distance fog, and emissive block radiance.
- **Hardware 2D Texture Array with Variant Auto-Discovery**: 16×16 texture array with automated discovery of multi-variant textures (e.g., 8 grass variants, 4 stone variants, 4 dirt variants).
- **Deterministic Spatial Randomization & Vertex Tinting**: Integer spatial hashing of 3D world coordinates for consistent variant selection across remeshes, and vertex color tinting (`ATTRIBUTE_COLOR`) for biome grass and water.
- **Sub-Voxel UV Blending & Face Unification**: Contiguous 2×2 sub-voxels merge into a single seamless 16×16 texture across 1m² block faces. Isolated sub-voxels retain complete [0, 1] texture mapping to avoid awkward corner cropping.
- **Animated Liquid Shaders**: GPU-driven vertical strip animation (36 frames for still water, 8 frames for flowing water) driven by `globals.time` in WGSL at 6 FPS.

### 3. Procedural World Generation, Biomes & Caves
- **Continuous Macro-Climate Noise & Geography**: Deterministic 2D gradient noise driving Continentalness, Temperature, and Humidity. Uses a continuous $C^1$ smooth cubic spline curve for continental base elevation and a continuous roughness multiplier, eliminating abrupt vertical cliffs and harsh elevation cuts across biome borders.
- **11 Distinct Biomes**: Plains (rolling hills, grass), Meadow (rich flowering grass transition), Desert (sand dunes, red sand accents, sandstone), Snowy Tundra & Frost Peaks (snow, packed ice, frost stone), Wetlands / Swamps (swamp grass mixed with mud, packed mud, clay), Rocky Highlands (mountain ridges, slate, cobbleslate, scree), Woodland (rich forest floor with grass mixed with mulch, packed dirt, and moss), Beach (sand coastlines), River (winding fluvial ribbons), Ocean (continental seabed), and Deep Ocean (abyssal gravel and blackstone trenches).
- **Surface Material Mixing & Natural Strata Transitions**: Natural multi-material noise blends across biomes (no uniform 100% mulch or mud); 3D noise dithering across all strata boundaries (subsoil-to-stone, slate, blackstone).
- **Dreadstone Bedrock Layer**: Unbreakable Dreadstone bedrock forming the bottom layer ($Y = -80$ blocks / $-160$ voxels), blending naturally into Blackstone across the bottom 3 layers. Completely immune to breaking and shaping.
- **Walkable 3D Caves & Suppressed Water Ravines**: Spacious 3–5 block wide spaghetti tunnels with wide walkable mouths at the surface; rare dramatic ravines that are strictly suppressed underwater in rivers, lakes, and oceans.
- **Dedicated Live Terrain & World Inspector GUI**: Custom egui tuning window bound to <kbd>F1</kbd> running in `EguiPrimaryContextPass` with full interactive sliders and buttons, with automated isolation of hotbar mouse scrolling and block interactions while open, plus an instant "Regenerate World" button.

### 4. Player Physics, Collision & Locomotion
- **Custom Voxel AABB Collision**: Zero-allocation AABB collision system querying chunk voxel data directly without rigid bodies or external physics engine overhead.
- **0.5m Terrain Auto-Stepping**: Automatically steps up 0.5m voxel elevation changes smoothly during grounded traversal.
- **Stance Hierarchy & Dimensions**:
  - Standing: height 1.80 m, eye height 1.62 m.
  - Crouching (<kbd>Ctrl</kbd>): height 1.30 m, eye height 1.20 m, speed reduced to 55%, with ledge-fall clamping preventing drops off steep edges.
  - Crawling (<kbd>C</kbd>): prone height 0.45 m, eye height 0.40 m, speed reduced to 35%, enables moving through 1-voxel high openings (0.5m) with headroom safety checks.
- **Creative Flight**: Double-tap Space toggle, fast sprint flight, vertical ascent/descent, and drag damping.
- **Fluid Locomotion**: Realistic water wading, swimming buoyancy, drag forces, and submersion detection.
- **Game Modes**: Creative mode (unrestricted flight, instant block edits) and Spectator mode (noclip through voxels).

### 5. Camera System & Procedural Humanoid Model
- **Minecraft 64×64 Skin Pipeline**: Humanoid mesh hierarchy (Head, Torso, Left/Right Arm, Left/Right Leg) mapped to `assets/textures/mobs/player_skin.png` with full support for base skins and 3D outer layers (hat, jacket, sleeves, pants) with alpha masking.
- **Procedural Locomotion Animations**: Walk/sprint limb swings, idle breathing sway, crouch torso tilt, crawling prone strokes, streamlined flutter-kick swimming, jump poses, and 4-state flight animations.
- **First-Person Body View**: True first-person visibility where the player's head is culled to avoid interior clipping, while looking down naturally reveals animated chest, arms, and legs.
- **Third-Person Orbit Camera (<kbd>F5</kbd>)**: Raycast occlusion prevention preventing camera from clipping underground or through walls, with independent head pitch/yaw tracking.
- **Camera Juice & Ergonomics**: Default 90° FOV (slider 60°..=110°), dynamic FOV kick (+8°) on sprint/fast flight, view bobbing during grounded walking, and continuous zoom (<kbd>Z</kbd> + wheel up to 20x).

### 6. Cellular Automata & Fluid Simulation
- **Wave-Paced Water Propagation**: Cellular automaton simulation running at a calibrated 0.25s tick rate with queued updates.
- **Differential Spread Limits**: 4 voxels (2 blocks) spread for single-voxel sources; 8 voxels (4 blocks) spread for full 1m³ block sources.
- **Downward Waterfall Priority**: Water falls strictly downwards when unsupported by solid ground, preventing mid-air spread on pillars or cliffs.
- **Stepped Water Height & Vertical Step Walls**: Gradient decreasing by 10cm per step down to 10cm, with vertical step quads sealing level transitions without air gaps.
- **Cross-Chunk Waterlogging**: Dynamic waterlogging during underwater sub-voxel shaping (<kbd>R</kbd>) and rotation (<kbd>T</kbd>).
- **Submerged Visibility**: Counter-clockwise ceiling geometry allowing clear upward visibility from below water surfaces, paired with submerged blue fog immersion.

### 7. Atmosphere, Celestial Systems & Calendar
- **Astronomical Clock & Calendar**: 24-minute real-time day cycle, 28-day months, 4 seasons (Spring, Summer, Autumn, Winter) lasting 84 days each, starting on Day 1 Month 1 Spring.
- **8-Phase Synchronized Lunar Cycle**: Spritesheet-sliced 32×32 moon phases with additive blending, synchronized with the 28-day calendar and alternating 3-day and 4-day phase durations.
- **Stylized Billboard Sun**: Flat billboard quad with additive blending and HDR coronal ring, casting 4-cascade directional shadows.
- **Dynamic Clouds & Starfield**: 1600m horizontal cloud plane with wind drift and atmospheric tinting; single-root hierarchical starfield dome with celestial rotation and smooth twilight fade.
- **Atmospheric Transitions & Time Control**: Continuous 4-stop piecewise-linear palette interpolation across Morning, Noon, Evening, and Night. Interactive time control (<kbd>F6</kbd>: tap to advance phase, hold to scrub time).

### 8. Gameplay Tools & Sub-Voxel Shaping
- **Interaction Modes (<kbd>B</kbd> Key)**: Toggle between 1m³ Logical Block mode (default) and 0.5m Sub-voxel mode.
- **Sub-Voxel Block Shaping Tool (<kbd>R</kbd> Key)**: Tap <kbd>R</kbd> to sequentially cycle 10 configurations (Full, Stair, Upside-Down Stair, Corner Stair, Inverted Corner Stair, Bottom Slab, Top Slab, Vertical Slab, Column, Centered Column).
- **Circular Radial Menu (<kbd>Hold R</kbd> >0.2s)**: 10-slice circular wheel with directional mouse selection and center preview card.
- **Block Rotation Tool (<kbd>T</kbd> Key)**: Rotates targeted block sub-voxels 90° clockwise around the vertical Y-axis.
- **Connected Placement**: Placing against non-full shapes (slabs, stairs) aligns to the hit surface without floating air gaps, with bi-directional centered column stacking.
- **Block Interactions**: Left-click break, right-click place, middle-click block pick.

### 9. User Interface, Menus & Developer Tooling
- **8-Slot Hotbar GUI**: Dark translucent backing, active gold selection border, slot numbers (1..8), mouse wheel scrolling, and <kbd>Q</kbd> slot clearing.
- **In-Game Creative Inventory (<kbd>E</kbd> Key)**: 40-slot item grid (8×5), non-pausing live world interaction, hotbar mirror row, and Mouse Tweaks controls (Shift-click transfer/clear, Shift+LMB drag, LMB drag painting, RMB stamp, digit key quick-assign).
- **3D Isometric Pixel-Art Block Icons**: Generated on-the-fly with 1.0 / 0.80 / 0.60 directional face shading, vertex tinting, and silhouette outlines.
- **Pause Menu (<kbd>ESC</kbd> Key)**: Game pause with Resume, Settings, Restart Game, Quit to Desktop, and camera Depth-of-Field blur.
- **In-Game Settings**: Live steppers for Render Distance (2..=16 chunks), FOV (60°..=110°), Distance Fog toggle, Camera Bobbing toggle, and Time Flow toggle.
- **Custom Mouse Cursors**: 9 cursor states including 13-frame animated busy spinner and floating held-block preview.
- **Debug Overlays**: Minimal HUD by default, Extended Technical Debug HUD on <kbd>F3</kbd> (FPS, frame time, player XYZ/chunk, biome climate, chunk stats), HUD toggle on <kbd>Shift+F3</kbd>, and chunk debug borders on <kbd>F2</kbd>.

### 10. Engine Optimizations & Scalability (Phase 7 Milestones)
- **Async Compute Greedy Meshing**: Decoupled from main thread `Update` loop to `AsyncComputeTaskPool`.
- **Decoupled Remesh Queues**: Routed fluid simulation, player edits, shaping, and pause reload through chunk streaming queues instead of unbuffered synchronous remeshing.
- **Single-Root Starfield Hierarchy**: Single rotating `StarfieldRoot` entity replacing 250 individual entity transform mutations per frame.
- **Direct-Indexed Texture Registry**: `[Option<VoxelTextureMapping>; 64]` array eliminating SipHash in greedy mesher loops.
- **Fast Chunk Hashing**: High-performance fast hasher (`FxHashMap`) for 3x–5x faster chunk lookups in `VoxelWorld`.
- **Optimized Terrain Exposure Sampling**: Eliminated 8×5 nested loops in dirt exposure checks, preventing thread pool starvation.
- **Zero Asset Dirtying in Environment**: Checking property changes before `materials.get_mut()` prevents constant GPU bind group invalidations.
- **Zero-Alloc Collision Checks**: Stack-allocated buffer for `overlapping_solid_voxels`.

---

## Next Steps / Upcoming Phases

- [x] Phase 1: Core Rendering & Texture-Array Architecture (Completed)
- [x] Phase 2: Atmosphere, Celestial Bodies & Dynamic Sky (Completed)
- [x] Phase 3: Player Tools & Block Shapes (Completed)
- [x] Phase 4: Fluid Dynamics, Underwater Visibility & World Persistence (Completed)
- [x] Phase 5: Procedural Voxel Meshing & Texture-Array Optimization (Completed)
- [x] Phase 6: Advanced World Generation, Biomes & Caves (Completed)
- [x] Phase 7: Engine Optimization, Architecture Audit & Scalability (Completed)
- [ ] Phase 8: Engine Optimization & Scalability (Next Milestone)
- [ ] Phase 9: Flora, Procedural Trees & Surface Vegetation
- [ ] Phase 10: Gameplay Polish, Audio Foundation & Quality-of-Life Tweaks

---

## Known Issues & Backlog for Future Fixes

- **Inspector "Regenerate World" Live Reload**: Sliders and options in the <kbd>F1</kbd> Terrain Inspector interact smoothly and update procedural generator parameters in real time. However, clicking the "Regenerate World" button does not yet immediately reload existing chunk meshes on screen because loaded chunk mesh entities need an explicit despawn/re-mesh trigger in `src/world/streaming/manager.rs`. (Paused and noted for a future fix per user direction).

---

## Important Rules for Assistance and Collaboration

- Inspect current files first before replacing systems.
- Current controls and project Structure can be found inside the "README.md" file.
- Keep the engine fully functional during migration.
- Validate changes with `cargo fmt`, `cargo check`, and `cargo clippy`.
- Do not add unnecessary comments to the code.
- Keep the code 100% in English.
