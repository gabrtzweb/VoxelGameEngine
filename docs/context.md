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

- Introduce texture-array rendering
- Individual texture per voxel
- Pixelated/stylized sun
- Pixelated/stylized moon
- Full day and night cycle divided into 4 phases: (the F6 control can be changed to: hold = advance time, click = change between phases)
    Morning
    Noon
    Evening
    Night
- Implement the 8 moon phases (can be 1 phase per day, resets after every 8 days)
- Custom skybox with sky gradient
- Stars during the night time
- Cloud system
- Better lightning and atmosphere
- Dynamic water propagation
- Support fluid updates across chunk boundaries
- Improve underwater visuals and water surface rendering
- Better and more realistic terrain
- Add biome generation
- Add caves and underground generation
- Add runtime terrain-generation controls
- Create a hotbar GUI with 10 slots (1 through 0 and scrollable)
- Block shaping tool while looking at it (R key: changes the voxel arrangement—i.e., full block, "stair" shape, or "horizontal/vertical slab" shape)
- Block rotation tool while looking at it (I haven't thought of a specific keybind yet)

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
