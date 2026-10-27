# Voxel Game Engine - Current Project Context

This is an attempt to develop an experimental voxel engine and game, built from scratch using Rust and Bevy.

---

## Core Concepts

- **1.0 m Voxel World**: The world is stored using 1.0 m voxels (identical to Minecraft blocks). Sub-voxels are completely eliminated; full 1.0 m³ blocks form the atomic spatial unit.
- **Chunk Geometry**: Chunks contain 16 × 16 × 16 voxels (16 m × 16 m × 16 m physical sections), with 4,096 voxels per chunk.
- **Block Shapes & Directional Placement**:
  - Native 1.0 m block shapes (11 total variants): `Full`, `Slab`, `Stair`, `Column`, `Torch`, `Basket`, `Pebble`, `PebbleCluster`, `CrossBillboard`, `Bamboo`, and `Cocoon` (0.75m × 0.875m dedicated bounds for `SpiderEgg` and `SilkCocoon`). Features custom meshes, passable non-solid collision for cross-billboards/mushrooms, low stepping for pebbles, and strict 3px column collision for bamboo stalks and leaves.
  - Runtime shape-locking protection (`is_shape_locked`): blocks such as bird nests, hay nests, bee/wasp hives, spider eggs, cocoons, and custom-meshed flora cannot be cycled into stairs/slabs or rotated via player controls.
  - Basket & Crate architectures: `Basket` features an authentic custom interior cavity with 2px rims, interior floor quad, and 4 inward-facing walls. `Crate` is consolidated as a 100% solid, fully textured opaque wooden container block.
  - 3-axis directional placement (`is_axis_aligned`): all 17 tree species logs (`*Log`) and wood bark (`*Wood`), `BoneBlock`, `CalcinedBone`, `BloodyFlesh`, `FleshBlock`, and bales (`CornBale`, `HayBale`, `RiceBale`, `StrawBale`). Automatically aligns along the X, Y, or Z placement axis based on hit normal, rotating face textures and swapping UVs in greedy meshing. Contextual front-facing blocks (Bookshelves, Bee Nests, Wasp Nests) automatically face the player upon placement.
  - Automatic floor-support cascading: blocks requiring floor support (`requires_floor_support`, including pebbles, mushrooms, floor torches, vertical bamboo stalks/leaves, aquatic vegetation like seagrass/kelp, and coral fans) automatically break when their support block is broken.
- **615-Block Engine Catalog & Geological Systems Foundation**:
  - 615 registered engine voxel definitions (discriminants 0..=613) across 11 thematic categories (`rocks`, `trees`, `soils`, `ores`, `caverns`, `oceans`, `organics`, `decorations`, `mushrooms`, `vegetation`, `fluids`/`systems`).
  - Geological rock strata: 10 primary mother rocks (`Dolomite`, `Slate`, `Gabbro`, `Andesite`, `Limestone`, `Argillite`, `Granite`, `Basalt`, `Sandstone`, `Marble`), 54 authentic host-rock ore veins, 6 crystal geode varieties (Amethyst, Aquamarine, Citrine, Fluorite, Pink Salt, Selenite), and indestructible Mantle bedrock at Y = -254.
- **Strict Grayscale Selective Foliage Tinting**:
  - Biome foliage tinting restricted exclusively to living grayscale canopy species, grass soils, and water.
  - Natural pre-colored pigments for seasonal, dead, and evergreen foliage are preserved without green tint multiplication.
- **Procedural 2D & Handheld Item Rendering Pipeline**:
  - Real-time procedural 2D icon generation for cubic blocks, planar cross-billboards, animated fluid frames, and handheld items across container interfaces.
- **Deterministic Natural Texture Rotation & Flipping**:
  - Implemented in WESL fragment shader (`assets/shaders/voxel_opaque.wesl`) using an integer coordinate hash of `floor(world_position)`.
  - Natural terrain blocks (stone, soils, sands, gravels) rotate and flip pseudo-randomly to eliminate grid tiling patterns over large cliff faces and cave walls without breaking CPU greedy meshing efficiency.
- **Engine Architecture & Bevy 0.20 Platform**:
  - Built on **Bevy 0.20.0** utilizing direct **`bevy_egui = "0.43"`** for developer diagnostics.
  - Complete adoption of Bevy's **WESL shader framework** (`assets/shaders/*.wesl`) replacing legacy WGSL/`naga_oil` preprocessing with explicit package imports (`import bevy_pbr::...`), `@if` conditional compilation, and standard bind groups.
  - Full modernization of UI state handling, transitioning away from deprecated `bevy::ui::Interaction` to retained components (`bevy::ui_widgets::Button`, `bevy::picking::hover::Hovered`, and `bevy::ui::Pressed`).
  - Explicit `ScreenSpaceTransmission` registration for physical fluid and translucent material rendering.
- **Outcrop Dual-Layer Rendering**:
  - Grass and snow outcrop blocks across soils and rocks render top and side layers while ensuring the bottom face (`Face::Bottom` / `Face::Down`) renders the base mineral material (e.g., Dolomite Grass bottom renders Dolomite; Peat Grass bottom renders Peat Dirt).
  - Outcrop blocks are dual-indexed to appear seamlessly in both Soils and Rocks inventory tabs.
- **Tactile Gameplay Feedback**:
  - Subtle 8-particle debris bursts with terrain collision bouncing on block break.
  - 0.18s elastic scale bounce on block placement.
- **Auto-Step**:
  - Calibrated to 0.50 m (50 cm) for smooth future slab stepping, requiring jumping over full 1 m blocks.
- **Navigation & Mapping**:
  - Square gameplay Minimap HUD (North-up, compact XYZ coordinates, live player heading chevron).
  - Full-screen interactive World Map (`M`) with pan, zoom, and player tracking.
- **Atmosphere & Climate**:
  - 24-minute day/night astronomical clock, 28-day lunar calendar (8 moon phases), and 4 seasons.
  - Procedural color noise and climate-driven biome palettes (grass, foliage, water) with smooth 5-point cross-kernel boundary blending.
- **Dynamic FPS & VSync**:
  - Configurable presentation modes (AutoNoVsync default, toggleable in Settings) and intelligent frame throttling (15 FPS unfocused, 30 FPS idle) with unconstrained active gameplay (250+ FPS).
- **Render Scale (3D Resolution Scaling)**:
  - Real-time 3D viewport scaling (`0.50` to `1.00`, presets: 50% Performance, 66% Balanced, 75% Quality, 85% Ultra Quality, 100% Native) in `src/core/render_scale.rs`.
  - Off-screen render buffer with hardware-accelerated bilinear upsampling to the native window, keeping 2D UI, hotbars, menus, and text at crisp 100% native resolution.
- **16:9 Window Aspect Snapping**:
  - Enforced in `src/core/window_aspect.rs` (`WindowAspectPlugin`) maintaining 16:9 aspect ratio in windowed mode with minimum constraints (640×360 px) and auto-disable in fullscreen.
- **Custom Typography**:
  - Universal font asset management (`CutePixel.ttf`) applied across all in-game HUDs, menus, and interfaces with drop shadow contrast styling.
- **Standardized 480×270 Base Canvas & 9-Horizontal-Slot UI Architecture**:
  - **Unified 480×270 (16:9) Base Canvas**: All container, HUD, and menu interfaces are mapped to a fixed 480×270 px native pixel-art canvas, scaling cleanly with integer pixel-perfect alignment up to 4× (`GUI_SCALE = 4.0`, 1920×1080) and seamless borderless / letterbox support.
  - **In-Game Hotbar HUD (320 × 44 px)**: 9 primary item slots with 3px slot spacing, integrated health meter, exact 12px numeric font scale consistent with open inventory, 2 special readout slots (offhand and utility with Dark Iron backdrops), 4 non-clickable wing badges (`[M]`, `[J]`, `[C]`, `[T]`), and centered selection tooltips with smooth fade-out.
  - **Dark Iron Pause Menu Bar**: Centralized top navigation bar with 4 tabs (`Map`, `Journal`, `Inventory`, `System`), allowing seamless mouse navigation between menus while automatically hiding sub-HUDs (like the minimap) on background screens.
  - **Active Player Inventory (`inventory.png`, 480 × 270 px)**: 36 Card B storage slots (4 rows of 9), mirrored hotbar and offhand/utility slots with `inventory_slot_hover.png` feedback, action utility buttons (Sort, Trash, Options, Lock), non-clickable Card A armor placeholders and Card C crafting placeholders.
  - **Creative Catalog (`inventory_creative.png`, 480 × 270 px)**: 11 category tabs rendering scaled 3D isometric block icons (`BlockIcons`), docking selection frame (`inventory_creative_tab_selection.png`), live search filter with cursor, vertical scroll track with draggable thumb (`inventory_creative_scroller.png`), 90-slot grid (10 rows of 9), and mirrored bottom hotbar.
  - **Unified Dark Iron Button Suite**: Menus powered by unified dark obsidian / brushed steel buttons (`button_large.png` 96×16 px, `button_medium.png` 64×16 px, `button_small.png` 16×16 px, `button_util.png` 12×12 px) with frosted slate-blue metallic hover highlights (`#3c4552`).
  - **Item Tooltips**:
  - Centered item name readout above hotbar upon slot selection with smooth fade-out.
  - Responsive floating hover tooltips across all inventory and catalog slots.
- **Procedural Foliage & Grass Blade System**:
  - GPU-accelerated procedural grass blades emitted on top of grass turf blocks, slabs, and stairs (`FoliagePlugin`).
  - Rendered with custom WESL PBR shader (`assets/shaders/grass_blades.wesl`) featuring dynamic wind displacement, player trail trampling and recovery, and exact color matching with the underlying voxel terrain.
- **Multi-Channel Sound Engine & Spatial Audio**:
  - Comprehensive audio engine (`EngineAudioPlugin`) built on Bevy audio with independent volume mixing for Master, Weather, SFX, Music, and Ambience.
  - Material-driven block acoustics (`SoundMaterial`) covering all 614 voxels across 11 material profiles.
  - Spatialized 3D block interaction audio with distance attenuation and dynamic pitch variation.
  - Footstep system (`FootstepAudioPlugin`) with stance cadences, landing impact velocity scaling, water wading, and swimming strokes.
  - Submerged underwater loop with bubble effects, subterranean cave drones, day/night biome nature soundscapes, and multi-channel weather audio.
  - 39-track gameplay soundtrack player (`GameplayMusicPlugin`) with seamless cross-fading and natural Minecraft-style intervals.
- **Main Menu & Title Screen**:
  - Direct engine startup into `MenuState::MainMenu` presenting a full-screen concept art background (`assets/textures/interfaces/main_menu_bg.png`).
  - Interactive two-phase flow:
    - **Splash Phase**: Large retro-pixel title banner ("VOXEL ENGINE"), version readout, and pulsing prompt ("Press any key to start") waiting for any key or mouse trigger.
    - **Action Phase**: Revealing Play, Settings, Controls, and Quit buttons styled with the Dark Iron sliced button suite (`button_large.png` & hover states) with tactile UI audio.
  - Seamless stack navigation via `MenuReturnTarget`, routing Settings and Controls Back buttons and Escape cleanly back to the Main Menu when opened from there, and back to Pause when in-game.

---

## Current World Scale

- The world is stored using 1.0 m voxels (identical to Minecraft blocks).
- Full 1 m³ blocks form the base unit of the world; 0.5 m sub-voxels are completely removed.
- Each chunk contains: 16 × 16 × 16 voxels (logical blocks).
- Total voxel capacity per chunk: 4,096 voxels.
- Physical chunk size: 16 m × 16 m × 16 m.
- The world streams procedurally and is effectively unlimited horizontally:
  - X: procedural streaming
  - Z: procedural streaming
- Vertical world limits:
  - Minimum chunk Y: -16 (-256 blocks)
  - Maximum chunk Y: +16 (+256 blocks)
  - Total playable vertical height: 512 blocks.
- Cloud plane altitude: 220.0 m.
- Current default render distance: 12 chunks (with LOD chunk radius set to 12 as well).
- Chunks are generated, meshed, and unloaded dynamically as the player moves through the world.

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
- Camera Zoom: dynamic magnification (`Z` + Mouse Wheel up to 20x zoom) with mouse sensitivity dampening
- First-Person Camera: true first-person body view (head hidden to prevent interior clipping, looking down reveals animated chest, arms, and legs)
- Third-Person Camera (`F5`): orbiting camera with dynamic raycast block collision prevention to avoid clipping underground/walls, and independent head pitch/yaw tracking

Player Model & Procedural Animations:

- Minecraft 64×64 skin compatible humanoid mesh hierarchy (Head, Torso, Left/Right Arm, Left/Right Leg) mapped to `assets/textures/models/player_skin.png`
- Full support for base skin and 3D outer layers (hat/hair, jacket, sleeves, pants) with alpha masking
- Procedural locomotion: dynamic walk/sprint leg & arm pendulum swings, idle breathing sway, crouch torso tilt, prone crawling strokes, streamlined flutter-kick swimming, airborne jump poses, and 4-state flight animations (ascent, descent, fast flight, hover)

Game Modes Currently Implemented:

- Creative (currently the main gameplay and development mode)
- Spectator (noclip flight mode)

The player uses a custom AABB collision system that directly queries voxel data. Individual physics colliders are not created for terrain voxels.

---

## Roadmap Status & Development Transition

The engine has concluded foundational development across Version 0.1 (Phases 1 through 20: rendering, atmosphere, fluids, greedy mesher optimizations, SIMD noise, 512-block world generation, and interface modernization).

In Version 0.2 (Modernization, Worldbuilding & Deep Systems), the project status:

- **Phase 1 (Completed)**: Full Block Catalog & Geological Systems Foundation (615 active engine voxels, geological strata catalog, 3-axis directional placement, selective foliage tinting, shader de-tiling).
- **Phase 2 (Completed)**: Standardized 9-Slot Interface Architecture & Menu Modernization (Unified 9-slot container and HUD geometry, Dark Iron design suite, 480×270 base integer scaling, pause menu bar navigation, and procedural icon rendering).
- **Phase 3 (Completed)**: Core Engine Upgrade (Bevy 0.20 Migration, direct `bevy_egui = "0.43"`, modern UI widgets, WESL shader framework, and ScreenSpaceTransmission).
- **Phase 4 (Completed)**: Architecture, Graphics Modernization & Soundness (Multi-tier hardware profiles, chunk memory optimization, modular domain crates, dynamic 3D render scaling, 16:9 window snapping, test isolation).
- **Phase 5 (Completed)**: High-Impact Performance & Graphics Pipeline (Dynamic frame rate limiter, hardware GPU upload budgets, AMD FSR 1.0 upscaling, subterranean occlusion culling).
- **Phase 6 (In Progress)**: Flora, Crops & Interactive Mechanical Blocks (Agricultural lifecycle simulation, in-world crop growth, connected block topology masks, wild flora, interactive doors and containers).
- **Phase 7 (In Progress)**: Subterranean Strata, Caverns & Geological Generation (Multi-tier geological strata, Mantle bedrock at Y = -254, 54 authentic ore vein pairings, crystal geodes, subterranean cave biomes).
- **Phase 8 (Planned)**: Biome Refactoring, Climate Modeling & Tree/Flora Distribution (Multi-noise climate fields, organic biome dithering, ecological tree dispersion, canopy density models).
- [Phase 9 (In Progress - Simplified Version)](file:///c:/Users/Rodrigo/Documents/BevyProjects/VoxelGameEngine/docs/project_roadmap.md#phase-9-main-menu-clean-world-session-lifetime--deferred-world-loading-in-progress---simplified-version): Main Menu, Clean World Session Lifetime & Deferred World Loading.
- **Phase 10 (Completed)**: Sound Engine & Spatial Audio System (Multi-channel volume mixer, 11-material acoustic profiles, spatial 3D block interaction audio, realistic footstep cadences, water wading and swimming strokes, ambient biomes, weather audio, and 39-track soundtrack).
- **Procedural Foliage Engine**: Real-time GPU grass blade rendering with wind swaying, player trampling, and terrain color matching.

Detailed progress and milestones are tracked in `docs/project_roadmap.md`.

---

## Important Rules for Assistance and Collaboration

- Inspect current files first before replacing systems.
- Current controls and project structure can be found inside `README.md`.
- Keep the engine fully functional during migration.
- Validate changes with `cargo fmt`, `cargo check`, and `cargo clippy`.
- Do not add unnecessary comments to the code.
- Keep the code 100% in English.
