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
- Vertical world limits are currently (These limits are intentionally configurable and may be expanded later):
    - Minimum chunk Y: -8
    - Maximum chunk Y: +7
- Chunk streaming operates in three dimensions around the player.
- Current default render distance: 8 chunks.
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
- Spectator

The player uses a custom AABB collision system that directly queries voxel data. Individual physics colliders are not created for terrain voxels.

Player Features:
- Gravity & terminal velocity
- Ground detection & ledge clamping (crouching)
- Voxel collisions & wall sliding
- Headroom detection for stance transitions
- Jumping
- Automatic terrain stepping (0.5 meters)
- Creative flight (double-space toggle, fly up/down, fast sprint flight)
- Realistic swimming & wading with buoyancy, drag, and submersion detection
- First-person & third-person cameras with raycast occlusion prevention

---

## Implemented Features / Currently Working

The project already has:
- Procedural terrain generation
- Chunks and streaming
- Terrain with vertical variation
- Lakes and water bodies
- Traversable water and basic physics
- Destructible and placeable blocks
- Dynamic lighting
- Light-emitting blocks (with true textures, shader emissive radiance, and consolidated 3D point lights)
- Directional shadows
- Distance fog
- Creative and Spectator modes
- Basic movement and physics
- HUD and debug overlay
- 2D Texture-Array voxel rendering with multi-variant randomization (e.g., multiple grass variants)
- Sub-voxel UV blending & seamless 1m² face texture unifications (continuous 16×16 texture across 2×2 sub-voxels)
- Full 4-phase day and night cycle (Morning, Noon, Evening, Night) with smooth continuous atmospheric transitions
- Interactive time control (F6: click to advance between phases, hold to scrub time smoothly)
- Stylized billboard celestial bodies (sun with radiant coronal ring and 8-phase lunar cycle with additive blending)
- Dynamic moving cloud layer with wind drift and atmospheric color tinting
- Sparkling nighttime starfield dome with celestial rotation and smooth twilight fade-in
- 8-slot hotbar GUI with 2D pixel-art item icons, active selection indicator, direct keybinds (1–8), mouse wheel scrolling, and slot clearing (Q)
- Sub-voxel block shaping tool with 10 configurations: Full, Stair, Upside-Down Stair, Corner Stair, Inverted Corner Stair, Bottom Slab, Top Slab, Vertical Slab, Column, and Centered Column
- Circular radial shape selection menu (<kbd>Hold R</kbd> >0.2s) with directional slice selection and center preview card
- Sub-voxel block rotation tool (T key) rotating shapes 90° clockwise around the Y-axis
- Connected block placement against non-full sub-voxels (slabs, stairs) without floating gaps, and bi-directional centered column stacking
- Dynamic cellular automaton fluid simulation with 0.25s wave-by-wave propagation pacing
- Differential fluid spread: 4 voxels (2 blocks) for single-voxel sources, 8 voxels (4 blocks) for full-block sources
- Stepped water surface height gradient (10cm steps down to 10cm) with vertical step walls sealing all level transitions
- Straight-down waterfall physics (ground-support verification) preventing mid-air spreading on pillars and cliff drops
- Automatic sub-voxel waterlogging during underwater shaping and rotation
- Full underwater visibility from below with counter-clockwise winding ceiling geometry and submerged fog immersion
- In-game calendar and seasonal progression: 24-minute real-time day cycle, 28-day months, 4 seasons (Spring, Summer, Autumn, Winter) lasting 84 days each, starting on Day 1 Month 1 Spring
- 8-phase lunar cycle strictly synchronized with the 28-day calendar, alternating between 3-day and 4-day phase durations
- Dual-state debug HUD: Minimal non-intrusive HUD by default, Extended technical debug screen on F3, HUD visibility toggle on Shift+F3, and chunk boundary debug remapped to F2
- Pause menu (ESC key) pausing in-game clock and player actions, with Resume, in-game Settings, Restart Game (resets day, teleports to spawn, rolls back placed/destroyed blocks), Quit to desktop, and camera Depth of Field blur
- In-game settings menu with live render distance stepper (2..=16 chunks), FOV stepper (60°..=110°), distance fog toggle, camera bobbing toggle, and time flow toggle
- In-game inventory menu (E key) with clean 8×4 (32-slot) item grid, non-pausing live world interaction, hotbar mirror row, Mouse Tweaks controls (Shift-click quick transfer/clear, Shift+LMB drag, LMB drag painting across slots, RMB stamp/deselect), backdrop click deselect, Q/middle-click clear, and digit hotkeys (1–8)
- Procedural Minecraft-style 3D isometric pixel-art block icon renderer on-the-fly with 1.0/0.8/0.6 directional face shading and tints
- Dedicated blocks architecture (`src/voxel/blocks.rs`) supporting 28+ block types, texture IDs, and future survival properties (durability, tools)
- Custom stylized pixel-art mouse cursor states (default, pointing_hand, grabbing, shift, busy with 13-frame animation, resize, ibeam, crosshair, not_allowed) with floating block preview when holding items
- Minecraft 64×64 skin body model with true first-person visibility, third-person mode, and procedural animations for walk, sprint, idle breathing, crouch, crawl, swim, and flight
- Continuous macro-climate noise generator (Continentalness, Temperature, Humidity) driving 6 distinct biomes (Plains, Desert, Snowy Tundra, Wetlands, Highlands, Woodland) with unique surface materials and elevation profiles
- 3D cave system featuring dual-noise spaghetti worm tunnels, expansive subterranean cheese caverns, surface attenuation buffering, underground water aquifers, and deep magma/lava basins
- Realistic geological strata layers (Topsoil, Subsoil, Upper Stone, Mid Slate/Cobbleslate, Deep Blackstone/Magma) with 3D mineral deposits (Gravel, Flint, Cobblestone, Clay, Magma) quantized to 1m³ logical blocks for sub-voxel material consistency
- Live world generation controls registered in `bevy_inspector_egui` (F1) with real-time chunk reloading and remeshing while preserving player edits
- Extended Debug HUD (F3) displaying live Biome identification and climate parameters

---

## Desired Visual Identity

I do not want a photorealistic look, but rather an aesthetic that is:
- Voxel-based
- Pixel art
- Stylized
- Inspired by voxel games

I want to avoid:
- Perfect circles
- Non-cuboid models
- Photorealism

I want lighting and atmosphere inspired by shaders or Vibrant Visuals, while maintaining a voxel and pixel art aesthetic.

---

## Next Steps / Upcoming Phases

- [x] Phase 6: Advanced World Generation, Biomes & Caves (Completed)
- [ ] Phase 7: Engine Optimization & Scalability (Next Milestone)
- [ ] Phase 8: To be decided

---

## Important Rules for Assistance

- Inspect current files first before replacing systems.
- Current controls and project Structure can be found inside the "README.md" file.
- Keep the engine fully functional during migration.
- Validate changes with `cargo fmt`, `cargo check`, and `cargo clippy`.
- Do not add unnecessary comments to the code.
- Keep the code 100% in English.
