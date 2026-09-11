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

Current dimensions:
- Width: 0.60 m
- Height: 1.80 m
- Eye height: 1.62 m

Camera FOV:
- 90 degrees

Game Modes Currently Implemented:
- Creative (currently the main gameplay and development mode)
- Spectator

The player uses a custom AABB collision system that directly queries voxel data. Individual physics colliders are not created for terrain voxels.

Player Features:
- Gravity
- Ground detection
- Voxel collisions
- Wall collisions
- Jumping
- Automatic terrain stepping
- Creative flight
- Swimming
- First-person camera
- Third-person camera
- Automatic Step-Up (0.5 meters)

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
- Light-emitting blocks
- Directional shadows
- Distance fog
- Creative and Spectator modes
- Basic movement and physics
- HUD and debug overlay
- 2D Texture-Array voxel rendering with multi-variant randomization (e.g., multiple grass variants)
- Full 4-phase day and night cycle (Morning, Noon, Evening, Night) with smooth continuous atmospheric transitions
- Interactive time control (F6: click to advance between phases, hold to scrub time smoothly)
- Stylized billboard celestial bodies (sun with radiant coronal ring and 8-phase lunar cycle with additive blending)
- Dynamic moving cloud layer with wind drift and atmospheric color tinting
- Sparkling nighttime starfield dome with celestial rotation and smooth twilight fade-in
- 8-slot hotbar GUI with 2D pixel-art item icons, active selection indicator, direct keybinds (1–8), mouse wheel scrolling, and slot clearing (Q)
- Sub-voxel block shaping tool (R key) cycling 1m³ blocks through Full, Stair, Slabs, and Column configurations
- Sub-voxel block rotation tool (T key) rotating shapes 90° clockwise around the Y-axis

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

## Next Steps / To Be Implemented

- Radial UI for selecting sub-voxel shapes
- Custom skybox with procedural sky gradient
- Dynamic water propagation
- Support fluid updates across chunk boundaries
- Improve underwater visuals and water surface rendering
- Better and more realistic terrain
- Add biome generation
- Add caves and underground generation
- Add runtime terrain-generation controls

## Performance improvements to maybe implement in the future
- LOD render distance
- Extremity bound checking
- Noise up-sampling
- Noise Cashing
- RLE based runtime voxel data

---

## Important Rules for Assistance

- Inspect current files first before replacing systems.
- Current controls and project Structure can be found inside the "README.md" file.
- Keep the engine fully functional during migration.
- Validate changes with `cargo fmt`, `cargo check`, and `cargo clippy`.
- Do not add unnecessary comments to the code.
- Keep the code 100% in English.
