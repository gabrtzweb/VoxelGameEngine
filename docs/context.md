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
- The world is stored using 1.0 m voxels (identical to Minecraft blocks).
- Full 1 m³ blocks form the base unit of the world; 0.5 m sub-voxels are completely removed.
- Each chunk contains: 16 × 16 × 16 voxels (logical blocks).
- Total voxel capacity per chunk: 4,096 voxels.
- Physical chunk size: 16 m × 16 m × 16 m.
- The world streams procedurally and is effectively unlimited horizontally:
    - X: procedural streaming
    - Z: procedural streaming
- Vertical world limits are currently:
    - Minimum chunk Y: -16 (-256 blocks)
    - Maximum chunk Y: +16 (+256 blocks)
    - Total playable vertical height: 512 blocks.
- Bottom-most layer of blocks: 100% unbreakable Dreadstone bedrock strictly confined to the bottom 3–4 layers of the world ($Y \le -254$). The very bottom layer ($Y = -256$) is guaranteed solid, smooth Dreadstone bedrock; cave carvers are strictly masked out of the bottom layer to eliminate voids or holes through the floor of the world.
- Upper underground crust ($Y > -120$) is uniform `Stone`, smoothly transitioning at the underground depth midpoint ($Y \le -120$) down to `Blackstone` (`rock_blackstone`). `Slate` and `Cobbleslate` are reserved exclusively for the `Highlands` biome.
- Chunk streaming operates using a horizontal cylindrical distance ($X^2 + Z^2 \le R^2$) within the vertical range of chunk Y $-16$ to $+16$. This guarantees that soaring mountain summits ($Y \le 256$) and deep caverns are never truncated or sliced off by spherical distance clipping.
- Cloud plane altitude: 220.0 m (floating high above the tallest mountain peaks).
- Current default render distance: 12 chunks (distance fog disabled by default).
- Chunks are generated and unloaded dynamically as the Creative player moves through the world.

The engine focuses on a fully editable procedural voxel world composed of full 1.0 m³ blocks. It takes inspiration from voxel games like Minecraft but features its own architecture and mechanics.

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
- Minecraft 64×64 skin compatible humanoid mesh hierarchy (Head, Torso, Left/Right Arm, Left/Right Leg) mapped to `assets/textures/models/player_skin.png`
- Full support for base skin and 3D outer layers (hat/hair, jacket, sleeves, pants) with alpha masking
- Procedural locomotion: dynamic walk/sprint leg & arm pendulum swings, idle breathing sway, crouch torso tilt, prone crawling strokes, streamlined flutter-kick swimming, airborne jump poses, and 4-state flight animations (ascent, descent, fast flight, hover)

Game Modes Currently Implemented:
- Creative (currently the main gameplay and development mode)
- Spectator (noclip flight mode)

The player uses a custom AABB collision system that directly queries voxel data. Individual physics colliders are not created for terrain voxels.

---

## Implemented Architecture & Features by Domain

### 1. Core Voxel Architecture & World Representation
- **1.0m³ Block Architecture**: The world is stored using 1.0 m voxels (identical to Minecraft blocks). Full 1 m³ blocks form the base unit of the world; 0.5 m sub-voxels are completely removed. Native block shapes (`BlockShape::Full`, `BlockShape::Slab`, `BlockShape::Stair`, `BlockShape::Column`) are supported with full 3D orientations and exact sub-box collision raycasting.
- **Chunk Geometry**: 16 × 16 × 16 voxels/blocks (4,096 voxels per chunk) spanning 16 m × 16 m × 16 m physical space.
- **Procedural Cylindrical Streaming**: Dynamic horizontal radius streaming ($X^2 + Z^2 \le R^2$) spanning vertical chunk bounds from chunk $Y = -16$ ($-256$ blocks) up to chunk $Y = +16$ ($+256$ blocks). Default render distance of 12 chunks (configurable 2..=16 chunks in settings). Prevents mountain peaks and subterranean caverns from being truncated.
- **Block Registry & Properties**: Dedicated blocks architecture in `src/world/block.rs` supporting 64 block types, texture IDs, tool tiers (Pickaxe, Shovel, Axe), and material durability values. Includes multi-face blocks such as `Voxel::OakWoodLog` (log rings on top/bottom, bark on sides), `Voxel::SnowyGrass` (snow top, snowy grass sides, dirt bottom), `Voxel::RootedDirt` (dirt with hanging subterranean roots), and `Voxel::RainwoodWoodLog`.

### 2. Meshing & GPU Rendering Pipeline
- **Asynchronous Greedy Meshing**: Chunk meshing offloaded to Bevy's `AsyncComputeTaskPool` with background worker tasks and throttled main-thread mesh uploading (`src/meshing/async_mesher.rs`), eliminating frame-rate drops.
- **Multi-Face Directional Textures**: `VoxelTextureRegistry` and greedy mesher support distinct textures per face direction (e.g. `PositiveY`/`NegativeY` log rings vs `PositiveX`/`Z` side bark), seamlessly integrating with greedy quad merging.
- **Custom WGSL Voxel Shader**: Extended PBR material (`ExtendedMaterial<StandardMaterial, VoxelMaterialExtension>`) preserving PBR lighting, directional shadows, distance fog, and emissive block radiance.
- **Hardware 2D Texture Array with Variant Auto-Discovery**: 16×16 texture array with automated discovery of multi-variant textures (e.g., 8 grass variants, 6 oak bark variants, 4 stone variants, 4 dirt variants).
- **Deterministic Spatial Randomization & Vertex Tinting**: Integer spatial hashing of 3D world coordinates for consistent variant selection across remeshes, and vertex color tinting (`ATTRIBUTE_COLOR`) for biome grass and water.
- **1.0m Block UV Tiling & Face Unification**: Quads merge seamlessly across 1m block faces with 1:1 UV texture coordinate mapping.
- **Animated Liquid Shaders**: GPU-driven vertical strip animation (36 frames for still water, 8 frames for flowing water) driven by `globals.time` in WGSL at 6 FPS.

### 3. Procedural World Generation, Biomes & Caves
- **Continuous Macro-Climate Noise & Geography**: Deterministic 2D gradient noise driving Continentalness, Temperature, and Humidity. Recalibrated continental scale ($0.0012$ frequency) producing vast landmasses (1000+ blocks wide) and grand mountain peaks rising to $Y = 80\text{--}140+$, eliminating abrupt vertical cliffs and harsh elevation cuts across biome borders.
- **Comprehensive 14-Biome Climate Suite & Stratified Geology**:
  - Full suite of 14 active biomes: Plains, Cold Plains, Snowy Tundra & Frost Peaks (featuring multi-face `SnowyGrass` and snow summits), Meadow, Woodland, Wetlands, Highlands (featuring exclusive `Slate` and `Cobbleslate`), Plains Forest, Savanna, Desert, Beach, River, Ocean, and Deep Ocean.
  - Baseline terrain palette uses clean core materials: `Grass`, `Dirt`, `Stone`, `Water`, `Snow`/`SnowyGrass`, `Sand`, with lower subterranean `Blackstone` and bottom bedrock `Dreadstone`.
- **Eliminated Grass Stacking**: Surface grass is strictly placed at `depth == 0` with air exposure above it. Side-exposure dirt promotion has been removed, preventing stacked grass blocks on cliff steps and slopes.
- **Underwater Beach Protection**: Submerged surfaces (`world_y <= water_level`) never generate Grass, placing Sand down to `sea_level - 6` to eliminate offshore green grass rings.
- **Clean Subterranean Strata & Bedrock Floor**:
  - Subsoil is uniform `Dirt` (or `Sand` in Desert/Beach) with gravel/blackstone clutter completely removed.
  - Upper underground crust ($Y > -120$) is uniform `Stone`.
  - Lower crust ($Y \le -120$) transitions at depth midpoint down to `Blackstone` (`rock_blackstone`) via 3D dithered noise.
  - Bedrock (`Dreadstone`) is strictly confined to the bottom 3–4 layers of the world ($Y \le -254$). The bottom-most layer ($Y = -256$) is guaranteed smooth, unbroken Dreadstone bedrock with cave carvers masked out.
- **Natural Mountain Arches, Cave Mouths & Subterranean Rivers**:
  - Spacious 3–5 block wide 3D caves with natural cave mouth breaches on dry hillsides ($0.18$ mask threshold).
  - Horizontal ridge-tunneling arches carving hollow openings through tall mountain ridges ($Y \ge 34$).
  - Rivers flowing into high peaks ($Y > \text{sea\_level} + 14$) preserve the standing mountain mass while tunneling subterranean river caverns at sea level.
  - Removed subterranean water aquifers for clean, walkable cave exploration.
- **Paused Procedural Tree Generation**:
  - Procedural tree and clutter block generation has been paused and cleaned up in terrain chunk building. Trees and multi-face logs remain registered in block/inventory definitions, ready to be reintroduced in Phase 12.
- **Dedicated Live Terrain & World Inspector GUI**: Custom egui tuning window bound to <kbd>F1</kbd> running in `EguiPrimaryContextPass` with full interactive sliders and an instant "Regenerate World" button that cleanly despawns existing chunk mesh entities and re-triggers async mesh generation in real time.

### 4. Player Physics, Collision & Locomotion
- **Custom Voxel AABB Collision**: Zero-allocation AABB collision system querying chunk voxel data directly without rigid bodies or external physics engine overhead.
- **0.50m Auto-Stepping**: Automatically steps up to 0.50m (50cm) elevation changes (calibrated for slab stepping), requiring jumping over full 1m blocks.
- **Stance Hierarchy & Dimensions**:
  - Standing: height 1.80 m, eye height 1.62 m.
  - Crouching (<kbd>Ctrl</kbd>): height 1.30 m, eye height 1.20 m, speed reduced to 55%, with ledge-fall clamping preventing drops off steep edges.
  - Crawling (<kbd>C</kbd>): prone height 0.45 m, eye height 0.40 m, speed reduced to 35%, enables moving through low openings with headroom safety checks.
- **Creative Flight**: Double-tap Space toggle, fast sprint flight, vertical ascent/descent, and drag damping.
- **Fluid Locomotion**: Realistic water wading, swimming buoyancy, drag forces, and submersion detection.
- **Game Modes**: Creative mode (unrestricted flight, instant block edits) and Spectator mode (noclip through voxels).

### 5. Camera System & Procedural Humanoid Model
- **Minecraft 64×64 Skin Pipeline**: Humanoid mesh hierarchy (Head, Torso, Left/Right Arm, Left/Right Leg) mapped to `assets/textures/models/player_skin.png` with full support for base skins and 3D outer layers (hat, jacket, sleeves, pants) with alpha masking.
- **Procedural Locomotion Animations**: Walk/sprint limb swings, idle breathing sway, crouch torso tilt, crawling prone strokes, streamlined flutter-kick swimming, jump poses, and 4-state flight animations.
- **First-Person Body View**: True first-person visibility where the player's head is culled to avoid interior clipping, while looking down naturally reveals animated chest, arms, and legs.
- **Third-Person Orbit Camera (<kbd>F5</kbd>)**: Raycast occlusion prevention preventing camera from clipping underground or through walls, with independent head pitch/yaw tracking.
- **Camera Juice & Ergonomics**: Default 90° FOV (slider 60°..=110°), dynamic FOV kick (+8°) on sprint/fast flight, view bobbing during grounded walking, and continuous zoom (<kbd>Z</kbd> + wheel up to 20x).

### 6. Cellular Automata & Fluid Simulation
- **Wave-Paced Water Propagation**: Cellular automaton simulation running at a calibrated 0.25s tick rate with queued updates.
- **Spread Limits**: Up to 8 blocks spread for 1m³ full block water sources.
- **Downward Waterfall Priority**: Water falls strictly downwards when unsupported by solid ground, preventing mid-air spread on pillars or cliffs.
- **Stepped Water Height & Vertical Step Walls**: Gradient decreasing by 10cm per step down to 10cm, with vertical step quads sealing level transitions without air gaps.
- **Submerged Visibility**: Counter-clockwise ceiling geometry allowing clear upward visibility from below water surfaces, paired with submerged blue fog immersion.

### 7. Atmosphere, Celestial Systems & Calendar
- **Astronomical Clock & Calendar**: 24-minute real-time day cycle, 28-day months, 4 seasons (Spring, Summer, Autumn, Winter) lasting 84 days each, starting on Day 1 Month 1 Spring.
- **8-Phase Synchronized Lunar Cycle**: Dedicated individual 64×64 moon phase textures from `assets/textures/environments/celestial/moon/` with additive blending, synchronized with the 28-day calendar and alternating 3-day and 4-day phase durations.
- **Stylized Billboard Sun**: Flat billboard quad with additive blending and HDR coronal ring, casting 4-cascade directional shadows.
- **Dynamic Clouds & Starfield**: 1600m horizontal cloud plane with wind drift and atmospheric tinting; single-root hierarchical starfield dome with celestial rotation and smooth twilight fade.
- **Atmospheric Transitions & Time Control**: Continuous 4-stop piecewise-linear palette interpolation across Morning, Noon, Evening, and Night. Interactive time control (<kbd>F6</kbd>: tap to advance phase, hold to scrub time).

### 8. Gameplay Tools, Shaping & Interaction Feedback
- **Block Interaction Feedback & Particle FX (Stage 10.1)**:
  - **Block Breaking Particle Bursts**: Scattering 8 subtle sub-voxel debris pebbles ($0.040\text{m}$ half-extent / $8\text{cm}$ cubes) matching the broken block's texture layer and tint color, bouncing realistically against collidable terrain and walls (`check_terrain_collision`), with friction, gravity, and lifetime shrinking before despawning.
  - **Block Placement Feedback**: Punchy 0.18s elastic scale bounce animation ($1.15 \to 1.00 \to 0.95 \to 1.00$) matching placed block multi-face textures and orientation.
  - **Decoupled Observer Architecture**: Driven by Bevy 0.19 `On<BlockBreakEvent>` and `On<BlockPlaceEvent>` observer triggers in `src/gameplay/feedback.rs`.
- **Block Shaping Tool (<kbd>R</kbd> Key)**: Tap <kbd>R</kbd> to sequentially cycle configurations (`Full`, `Slab`, `Stair`, `Column`); held for 4-slice radial menu.
- **Circular Radial Menu (<kbd>Hold R</kbd> >0.2s)**: 4-slice circular wheel with directional mouse selection and center preview card.
- **Block Rotation Tool (<kbd>T</kbd> Key)**: Rotates targeted block shape 90° clockwise around the vertical Y-axis.
- **Block Interactions**: Left-click break, right-click place, middle-click block pick.

### 9. User Interface, Menus & Developer Tooling
- **Textured 8-Slot Hotbar HUD**: Pixel-art skinned hotbar tray (`assets/textures/interfaces/containers/hotbar.png`, 256×32 px centered, scaled 2× integer to 512×64 px) with 36×36 px outer slot frames, 32×32 px native item icons (16px internal slot size), transparent inactive slots, golden active selection frame (`Color::srgb(1.0, 0.85, 0.30)`), slot numbers (1..8) with drop shadow, mouse wheel cycling, and <kbd>Q</kbd> slot clearing.
- **Unified Textured Inventory Interface (<kbd>E</kbd> Key)**: Texture-skinned container interface (`src/menu/creative_inventory.rs`) based on a unified 190×152 px container panel (`assets/textures/interfaces/containers/inventory.png` and `inventory_creative.png`) rendered at 3× integer scale (570×456 px) using nearest-neighbor sampling. Centered horizontally and vertically:
  - **Mode Toggle Buttons**: Two clickable buttons sitting in the top tab area above the main panel (`Personal` at $X=22..91$, `Creative` at $X=98..167$, $Y=2..13$ in base texture space) as well as the <kbd>Tab</kbd> hotkey to toggle between Personal and Creative modes. Features **Persistent Tab Memory** remembering the active mode across closing and reopening.
  - **Personal Inventory Panel**: Displays the `"Inventory"` title ($X=24, Y=32$), two action button placeholders ($X=138, 156$), an 8×4 storage grid (32 slots) mapped to `PlayerInventory`, and a 1×8 hotbar mirror row at $Y=128$.
  - **Creative Inventory Panel**: Displays a compact title, an interactive search bar ($X=99..167, Y=33..42$), an 8×4 palette grid of all registered blocks with live filtering, a pixel-art scrollbar ($X=175..186$) with draggable `inventory_scroller.png` thumb, and the 1×8 hotbar mirror row.
  - **Search Bar Ergonomics & Text Selection**:
    - **Backspace Repeat**: Holding the Backspace key automatically repeats character deletion (0.40s initial delay, 0.04s repeat rate) instead of single-character deletion.
    - **Full Text Selection**: Double-clicking selects all text; clicking and dragging with the mouse selects a precise character range, rendered with a semi-transparent blue highlight box behind the text. Also supports <kbd>Ctrl+A</kbd> to select all.
    - **Selection Replacement & Deletion**: Typing replaces the selected text range; pressing Backspace or Delete removes the selection.
    - Guarded <kbd>E</kbd> key when search input is focused so typing 'e' does not close the inventory.
  - **Slot Ergonomics & Transfers**:
    - **Shift + Click**: Instant transfer between personal storage and hotbar.
    - **Shift + LMB Drag**: Rapid multi-slot sweep transfer / creative hotbar clearance.
    - **LMB Drag**: Multi-slot hotbar painting with held block.
    - **Right Click / Middle Click / Q**: Slot drop, stamp, and clear actions.
  - **Cinematic Background Blur**: Smooth Gaussian camera depth-of-field blur (`DepthOfField`) triggers when opening the inventory, matching the Pause/Settings menu.
- **3D Isometric Pixel-Art Block Icons**: Generated on-the-fly with 1.0 / 0.80 / 0.60 directional face shading, vertex tinting, and silhouette outlines.
- **Pause Menu (<kbd>ESC</kbd> Key)**: Game pause with Resume, Settings, Restart Game, Quit to Desktop, and camera Depth-of-Field blur.
- **In-Game Settings**: Live steppers for Screen Mode (Windowed, Exclusive Fullscreen, Borderless Fullscreen), Render Distance (2..=16 chunks), FOV (60°..=110°), Distance Fog toggle, Camera Bobbing toggle, Fancy Block Sides toggle (broadened to Snowy Grass, Mulch, and Grass sides), VSync toggle (AutoNoVsync default / AutoVsync), Dynamic FPS toggle, and Time Flow toggle.
- **Custom Mouse Cursors**: 9 cursor states including 13-frame animated busy spinner and floating held-block preview.
- **Minecraft/Xaero-Style Minimap HUD**: Top-right square HUD with parchment `map_background.png` frame extending outward around the 192×192 dynamic canvas, bright white cardinal indicators (<kbd>N</kbd>, <kbd>S</kbd>, <kbd>E</kbd>, <kbd>W</kbd>) inset over terrain, real-time readout (`Coordinates: XYZ: ...` and `Biome: ...`), and live directional player `marker_red.png` rotating with camera yaw via GPU `UiTransform`. Powered by an asynchronous 2D cache (`MapCache`) with North-up topographic hill shading and precomputed surface colors for zero-noise 250+ FPS updates.
- **Full-Screen Interactive World Map (<kbd>M</kbd> Key)**: Seamless `MenuState::WorldMap` integration with smooth mouse drag panning, scroll wheel zooming (0.20x to 4.0x), keyboard panning (<kbd>WASD</kbd> / arrows), snap-to-player quick key (<kbd>Space</kbd>), real-time cursor coordinate tracking under pointer, and accurately centered `marker_red.png` tracking player coordinates and rotating with camera yaw.
- **Custom Font & Universal Drop Shadows**: Centralized typography via `AppFont` and `FontPlugin` pointing to `assets/fonts/CutePixel.ttf`, with universal crisp drop shadows (`TextShadow`) applied across all UI and HUD text elements for clear contrast on all backgrounds.
- **Ambient Environment & Biome Coloration**: Procedural 2-octave smooth color noise in `voxel.wgsl` via `world_position.xz` modulating tinted block vertices by $\pm 8\%$ to break up flat monochromatic plains, coupled with calibrated biome palettes (emerald Plains, golden Savanna, pale Desert, icy Tundra, turquoise coastal waters, deep navy oceans) and 5-point cross-kernel boundary blending.
- **Dynamic FPS & Power Throttling**: Automatic background frame throttling (15 FPS), idle/AFK detection (30 FPS after 30s), and physical battery detection. Active input priority and zero thread sleeping during normal gameplay guarantees unthrottled 250–300+ FPS performance.
- **Debug Overlays**: Minimal HUD by default, Extended Technical Debug HUD on <kbd>F3</kbd> (FPS, frame time, power source, dynamic throttle status, player XYZ/chunk, biome climate, chunk stats), HUD toggle on <kbd>Shift+F3</kbd>, and chunk debug borders on <kbd>F2</kbd>.

### 10. Engine Optimizations & Scalability (Phase 7, 8 & 9 Milestones)
- **Async Compute Greedy Meshing**: Decoupled from main thread `Update` loop to `AsyncComputeTaskPool`.
- **Chunk Homogeneity Flags**: `Empty` and fully occluded `Solid` chunks bypass mesher task scheduling, collision queries, and raycasts in O(1).
- **Noise Up-Sampling & Trilinear Interpolation**: 3D cave/density noise sampled at a 4×4×4 lattice with SIMD trilinear interpolation, reducing mathematical noise evaluations by **97%**.
- **Paletted Chunk Storage**: 4-bit indices for chunks with $\le 16$ block types, cutting world memory footprint by 70%–85%.
- **Decoupled Simulation Radius**: Simulation bubble (4–6 chunks) decoupled from visual render distance (10–16+ chunks).
- **64-Bit Bitmask Acceleration**: Row-level bitboards and intrinsic trailing/leading zero counts cutting face extraction time by 4x–8x.
- **Static Lookup Tables (LUTs)**: Compile-time precomputed tables for shapes, orientations, normals, and quadrant UVs.
- **Decoupled Remesh Queues**: Routed fluid simulation, player edits, shaping, and pause reload through chunk streaming queues instead of unbuffered synchronous remeshing.
- **Single-Root Starfield Hierarchy**: Single rotating `StarfieldRoot` entity replacing 250 individual entity transform mutations per frame.
- **Direct-Indexed Texture Registry**: `[Option<VoxelTextureMapping>; 64]` array eliminating SipHash in greedy mesher loops.
- **Fast Chunk Hashing**: High-performance fast hasher (`FxHashMap`) for 3x–5x faster chunk lookups in `VoxelWorld`.
- **Zero Asset Dirtying in Environment**: Checking property changes before `materials.get_mut()` prevents constant GPU bind group invalidations.
- **Zero-Alloc Collision Checks**: Stack-allocated buffer for `overlapping_solid_voxels`.
- **Codebase Modernization**: Complete dead code pruning and modular refactoring across 10 engine subsystems with zero compiler warnings and zero Clippy lints.

---

## Next Steps / Upcoming Phases

- [x] Phase 1: Core Rendering & Texture-Array Architecture (Completed)
- [x] Phase 2: Atmosphere, Celestial Bodies & Dynamic Sky (Completed)
- [x] Phase 3: Gameplay, Inventory & Sub-Voxel Shaping Tools (Completed)
- [x] Phase 4: Dynamic Fluid Simulation & Boundary Mechanics (Completed)
- [x] Phase 5: General Polish, Revisions & In-Game Interfaces (Completed)
- [x] Phase 6: Advanced World Generation, Biomes & Caves (Completed)
- [x] Phase 7: Project Organization, Architecture Audit & Refactor (Completed)
- [x] Phase 8: Foundational Storage, Meshing & Bitmask Acceleration (Completed)
- [x] Phase 9: Engine-Wide Architecture Modernization, 1m Shapes & Codebase Cleanup (Completed)
- [x] Phase 10: Gameplay Polish, Interaction Feedback & Quality-of-Life (Completed)
- [ ] Phase 11: World Generation & Worldbuilding Expansion (High Fantasy & Dark Fantasy Realism) (Active)
- [ ] Phase 12: Flora, Procedural Trees & Surface Vegetation (Upcoming)
- [ ] Phase 13: High-Performance Scaling, Level-of-Detail (LOD) & Engine Optimization (Upcoming)

---

## Known Issues & Backlog for Future Fixes

- **Terrain Polish & Gameplay Tuning**: World generation, cave density, and strata are currently undergoing agile tuning as gameplay testing dictates. Completed milestones include Stage 10.1 block feedback particles/bouncing, Stage 10.3 Minimap & World Map, Stage 10.4 Ambient Environment noise with biome-specific color tinting and blending, Stage 10.5 Screen Modes & Map Polish, Stage 10.6 UI Texture Skinning & Ergonomics, Phase 8 Bitmask/Storage Optimizations, and Phase 9 Codebase Modernization. Current active development is in Phase 11 high fantasy biomes, geological worldbuilding, and terrain climate mapping.

---

## Important Rules for Assistance and Collaboration

- Inspect current files first before replacing systems.
- Current controls and project Structure can be found inside the "README.md" file.
- Keep the engine fully functional during migration.
- Validate changes with `cargo fmt`, `cargo check`, and `cargo clippy`.
- Do not add unnecessary comments to the code.
- Keep the code 100% in English.
