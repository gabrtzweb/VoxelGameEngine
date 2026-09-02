# VoxelGameEngine

An experimental voxel game engine built from scratch with Rust and Bevy.

The project focuses on a fully editable procedural voxel world with a hybrid block structure:

- The world is stored using 0.5 m voxels.
- A traditional 1 m³ block is composed of 2 × 2 × 2 voxels.
- Each 1 m³ block therefore contains 8 individually editable voxels.
- Players will be able to destroy and place individual 0.5 m voxels while larger 1 m blocks remain useful as a visual and gameplay abstraction.

The long-term goal is to build a performant infinite procedural voxel world with runtime terrain editing, chunk streaming, configurable world generation and extensive development tooling.

## Current Stack

- Rust
- Bevy 0.19
- Custom voxel storage
- Custom chunk meshing
- Custom voxel raycasting
- Custom development camera
- Bevy ECS and rendering
- Git / GitHub

## World Structure

### Voxel

Base voxel resolution:

    0.5 m × 0.5 m × 0.5 m

A 1 m³ block contains:

    2 × 2 × 2 = 8 voxels

Each voxel can currently be individually removed or placed.

### Chunk

Each chunk contains:

    16 × 16 × 16 voxels

Total voxel capacity per chunk:

    4,096 voxels

Physical chunk size:

    8 m × 8 m × 8 m

### Current Prototype World

The current prototype loads:

    8 × 8 chunks

Total:

    64 chunks

Horizontal world area:

    64 m × 64 m

Maximum voxel capacity:

    262,144 voxels

The current world size is temporary and will eventually be replaced by dynamic chunk streaming.

## Current Features

### Voxel World

- 0.5 m voxel resolution
- 16³ voxel chunks
- Integer world-space voxel coordinates
- Correct support for negative chunk coordinates
- World-to-chunk coordinate conversion
- Multiple independent chunks
- Cross-chunk voxel lookup
- Runtime voxel modification

### Chunk Meshing

- Custom mesh generation
- One Bevy mesh per chunk
- Hidden internal voxel faces are removed
- Faces between neighboring chunks are removed
- Neighbor-aware chunk meshing
- Chunk remeshing after runtime edits
- Neighbor chunk remeshing when editing chunk boundaries

The current mesher generates one quad for every exposed voxel face.

Greedy meshing has not been implemented yet.

### Procedural Terrain

The current prototype uses a simple deterministic 2D value-noise terrain generator.

Current generation parameters include:

- Seed
- Base height
- Height amplitude
- Frequency
- Octaves
- Persistence

Terrain generation is continuous across chunk boundaries.

This generator is intentionally simple and currently exists to test the voxel world architecture.

Future terrain generation will include more advanced terrain shaping, biomes, caves and configurable runtime generation parameters.

### Voxel Interaction

Voxel interaction uses a custom grid-based voxel raycast instead of physics colliders.

Controls:

    Left Mouse Button   Break voxel
    Right Mouse Button  Place voxel

Holding either mouse button performs the action continuously.

Current repeat interval:

    0.16 seconds

Interaction distance:

    10 meters

Voxel placement uses the normal of the targeted voxel face to determine the adjacent placement position.

### Targeting

The crosshair continuously performs voxel targeting.

Target highlighting works at the 1 m block level.

A complete block is highlighted as a 1 m³ cube.

If one or more of its internal voxels have been removed, the highlight follows the resulting geometry of the remaining block rather than highlighting every individual voxel.

Target highlighting remains active independently from developer debug visualization.

### Development Camera

The project currently uses a custom free-fly development camera.

Controls:

    Mouse        Look
    W            Forward
    S            Backward
    A            Left
    D            Right
    Space        Move up
    Left Ctrl    Move down
    Shift        Fast movement

Mouse look is always active while the game window is focused.

The cursor is locked and hidden.

### Crosshair

A crosshair is permanently displayed at the center of the screen.

It is used for voxel targeting, breaking and placement.

### Developer Debug Mode

Press:

    F3

to toggle voxel world debug visualization.

Debug mode currently displays:

- 1 m block outlines
- Chunk boundaries

Chunk boundaries use a separate color from block outlines.

Individual 0.5 m voxel debug outlines are intentionally not displayed.

Debug visualization is distance-limited to reduce unnecessary rendering overhead.

### Development Statistics

A development HUD is displayed in the top-left corner.

Current statistics include:

- FPS
- Frame time
- Loaded chunks
- Meshed chunks
- Total voxel capacity
- Camera position
- Camera chunk coordinate
- Target voxel coordinate

Additional profiling statistics will be added as the engine becomes more complex.

## Project Structure

    src/
    ├── main.rs
    ├── dev_camera.rs
    ├── dev_stats.rs
    │
    └── voxel/
        ├── mod.rs
        ├── chunk.rs
        ├── world.rs
        ├── terrain.rs
        ├── mesher.rs
        ├── targeting.rs
        ├── interaction.rs
        └── debug.rs

### Module Responsibilities

`main.rs`

Application bootstrap, initial world creation, rendering setup and lighting.

`dev_camera.rs`

Free-fly development camera, mouse look and crosshair.

`dev_stats.rs`

Runtime development statistics HUD.

`voxel/chunk.rs`

Voxel type, chunk constants and chunk voxel storage.

`voxel/world.rs`

Multi-chunk voxel world, coordinate conversion and world-space voxel access.

`voxel/terrain.rs`

Procedural terrain generation.

`voxel/mesher.rs`

Voxel-to-mesh conversion and hidden-face removal.

`voxel/targeting.rs`

Voxel raycasting, current target state and adaptive block highlighting.

`voxel/interaction.rs`

Voxel breaking, placement and runtime chunk remeshing.

`voxel/debug.rs`

Developer visualization for blocks and chunk boundaries.

## Development Commands

Run the project:

    cargo run

Run an optimized build:

    cargo run --release

Check the project:

    cargo check

Format the project:

    cargo fmt

Run Rust lints:

    cargo clippy

## Current Development Status

The core voxel prototype is functional.

Completed:

- Rust / Bevy development environment
- GitHub repository
- Custom free camera
- Crosshair
- Voxel storage
- Multi-chunk world
- Runtime voxel breaking
- Runtime voxel placement
- Continuous breaking and placement
- Custom voxel raycast
- Chunk mesh generation
- Hidden face removal
- Cross-chunk face removal
- Neighbor chunk remeshing
- Procedural terrain
- Block targeting highlight
- Chunk and block debug visualization
- Development statistics HUD

## Next Technical Milestones

The next major areas of development are:

1. Establish a proper performance baseline and profiling tools.
2. Implement greedy meshing.
3. Add dynamic chunk loading and unloading.
4. Move chunk generation and meshing away from the main thread.
5. Add runtime terrain-generation controls.
6. Implement effectively infinite procedural world streaming.
7. Add persistent storage for player-modified voxels.
8. Expand terrain generation with caves, biomes and larger terrain features.
9. Add multiple voxel and material types.
10. Introduce player gameplay and physics after the voxel engine foundation is stable.

## Performance Goals

Performance is a core goal of the project.

The engine should eventually support:

- Large render distances
- Smooth chunk streaming
- Runtime terrain editing without noticeable frame stalls
- Multithreaded chunk generation
- Efficient chunk remeshing
- Compact voxel storage
- Reduced mesh geometry through greedy meshing
- Stable high-framerate gameplay

Performance targets will be defined using measured CPU, GPU, meshing, memory and rendering diagnostics rather than estimated voxel counts alone.

## License

License not yet defined.
