# VoxelGameEngine

An experimental voxel game engine built from scratch with Rust and Bevy.

The project focuses on a fully editable procedural voxel world with a hybrid block structure:

- The world is stored using 0.5 m voxels.
- A traditional 1 m³ block is composed of 2 × 2 × 2 voxels.
- Each 1 m³ block therefore contains 8 individually editable voxels.
- Individual 0.5 m voxels can be destroyed and placed at runtime.
- 1 m blocks remain useful as a visual, gameplay and coordinate abstraction.

The long-term goal is to build a performant procedural voxel game with large-world streaming, runtime terrain editing, configurable generation, multiple gameplay modes and extensive development tooling.

## Current Stack

- Rust
- Bevy 0.19
- Bevy ECS and rendering
- Custom voxel storage
- Custom chunk streaming
- Custom chunk meshing
- Custom voxel raycasting
- Custom voxel collision system
- Asynchronous terrain generation
- Git / GitHub

## World Structure

### Voxel

Base voxel resolution:

    0.5 m × 0.5 m × 0.5 m

A traditional 1 m³ logical block contains:

    2 × 2 × 2 = 8 voxels

Every voxel can be independently modified.

### Chunk

Each chunk contains:

    16 × 16 × 16 voxels

Total voxel capacity per chunk:

    4,096 voxels

Physical chunk size:

    8 m × 8 m × 8 m

### World Dimensions

The world currently streams procedurally and is effectively unlimited horizontally:

    X: procedural streaming
    Z: procedural streaming

Vertical world limits are currently:

    Minimum chunk Y: -8
    Maximum chunk Y: +7

Total vertical layers:

    16 chunks

Total vertical world height:

    128 meters

These limits are intentionally configurable and may be expanded later.

## Chunk Streaming

Chunk streaming operates in three dimensions around the player.

Current default render distance:

    8 chunks

The desired chunk region uses a spherical distance rather than a cubic region.

Chunks are generated and unloaded dynamically as the Creative player moves through the world.

### Asynchronous Generation

Terrain generation runs through Bevy's asynchronous compute task pool.

This keeps expensive procedural generation work away from the main gameplay thread and significantly reduces frame-time spikes during exploration.

Generation tasks are:

- Started with a limited number of concurrent jobs
- Prioritized by distance from the player
- Collected without blocking the main thread
- Discarded if the player moves away before the result is needed

Chunk meshing currently remains budgeted on the main thread.

Async meshing may be implemented later if profiling shows it is necessary.

### Render Optimization

Loaded chunks and rendered chunks are intentionally separate concepts.

A chunk may exist in voxel memory without requiring any GPU geometry.

Chunks with no exposed geometry:

- Do not receive a Bevy Mesh
- Do not receive a render Entity
- Do not appear in the F3 chunk debug visualization

This is especially important for completely empty air chunks and fully enclosed solid chunks.

## Procedural Terrain

The current terrain generator uses deterministic multi-octave 2D value noise.

Current parameters include:

- Seed
- Base height
- Height amplitude
- Frequency
- Octaves
- Persistence

Terrain is continuous across chunk boundaries.

The generator also contains fast paths for chunks that are completely above or below the terrain surface.

This avoids unnecessary voxel-by-voxel terrain filling for uniform air or solid chunks.

The current terrain generator exists primarily to validate the world architecture.

Future terrain generation will include:

- Larger terrain features
- Mountains
- Oceans
- Beaches
- Caves
- Biomes
- Runtime generation controls
- Additional noise dimensions and parameters

## Runtime World Modification

Voxel changes are currently persisted in memory for the duration of the game session.

If a modified chunk unloads and later loads again:

    modifications are restored

If the game is restarted:

    modifications reset

Disk-based world saving is intentionally deferred until later development.

## Chunk Meshing

The engine uses a custom world-aware chunk mesher.

Current features:

- One render mesh per visible chunk
- Hidden internal voxel faces are removed
- Faces between neighboring chunks are removed
- Chunk borders correctly query adjacent chunks
- Runtime edits trigger chunk remeshing
- Boundary edits also remesh affected neighboring chunks
- Empty meshes are never spawned

The current mesher generates one quad for every exposed voxel face.

Greedy meshing has not yet been implemented.

## Player System

The engine now has a dedicated player system separate from the voxel engine.

Current player dimensions:

    Width:      0.60 m
    Height:     1.80 m
    Eye height: 1.62 m

Camera FOV:

    90 degrees

The player uses a custom AABB collision system that queries the voxel world directly.

Individual voxel physics colliders are not required.

## Game Modes

The current architecture supports separate gameplay modes.

Implemented:

- Creative
- Spectator

Planned:

- Survival

Survival gameplay is intentionally deferred until the engine and Creative development workflow are more mature.

### Creative Mode

Creative is the primary development and gameplay mode.

Controls:

    Mouse          Look
    W              Forward
    S              Backward
    A              Left
    D              Right
    Shift          Sprint
    Space          Jump

The player has:

- Gravity
- Ground detection
- Voxel collision
- Wall collision
- Jumping
- Smooth terrain traversal
- Creative flight
- First-person camera
- Third-person camera

### Automatic Step-Up

The player automatically climbs terrain differences of:

    0.5 meters

This matches the base voxel height and allows smooth traversal over normal voxel terrain without repeatedly jumping.

Higher obstacles require jumping.

Camera movement is visually smoothed when stepping onto 0.5 m terrain.

### Creative Flight

Double-tapping:

    Space

toggles Creative flight.

While flying:

    W / A / S / D  Move
    Space          Move up
    Ctrl           Move down
    Shift          Fast movement

Creative flight still:

- Uses player collision
- Generates chunks
- Supports voxel interaction

Double-tapping Space again disables flight and restores normal gravity.

### First / Third Person

Press:

    F5

to toggle between first-person and third-person Creative camera modes.

A simple temporary player body is currently rendered as a colored rectangular body.

The body is hidden in first person and visible in third person.

Third-person camera collision with terrain has not yet been implemented.

## Spectator Mode

Press:

    F4

to switch between Creative and Spectator modes.

Spectator mode uses the original free-fly development-camera behavior.

Controls:

    Mouse          Look
    W / A / S / D  Move
    Space          Move up
    Ctrl           Move down
    Shift          Fast movement

Spectator mode:

- Has no gravity
- Has no collisions
- Can pass through terrain
- Does not break or place voxels
- Does not move the Creative player
- Does not generate new chunks

This means the player remains at the last Creative position while the Spectator camera can freely inspect the loaded world.

The player body remains visible while in Spectator mode, making it easy to see where the Creative player was left.

Returning to Creative mode moves the camera back to the player.

## Voxel Interaction

Voxel interaction uses a custom grid-based voxel raycast rather than physics colliders.

Controls:

    Left Mouse Button   Break voxel
    Right Mouse Button  Place voxel

Holding either button performs the action continuously.

Current repeat interval:

    0.16 seconds

Interaction distance:

    10 meters

Voxel placement uses the hit-face normal to determine the adjacent voxel position.

Voxel interaction is disabled in Spectator mode.

## Targeting

The crosshair continuously performs voxel targeting.

Target highlighting operates at the logical 1 m block level while respecting the internal 0.5 m voxel geometry.

A complete block is highlighted as a full 1 m³ shape.

If internal voxels have been removed, the outline adapts to the remaining geometry while suppressing unnecessary internal edges.

Target highlighting remains active independently from developer debug visualization.

## Developer Debug Mode

Press:

    F3

to toggle chunk debug visualization.

Debug mode currently displays only chunk boundaries that have actual rendered geometry.

Completely empty air chunks and other non-rendered chunks are not outlined.

Debug visualization is distance-limited to reduce unnecessary rendering overhead.

## Development Statistics

A development HUD is displayed in the top-left corner.

Current statistics include:

- FPS
- Frame time
- Current game mode
- Creative flight state
- Player logical block coordinates
- Player chunk coordinates
- Camera position
- Loaded chunks
- Meshed chunks
- Total loaded voxel capacity
- Target voxel coordinate

Logical player coordinates use 1 meter per coordinate unit even though the internal voxel resolution is 0.5 meters.

Additional profiling statistics will be added as the engine becomes more complex.

## Project Structure

    src/
    ├── main.rs
    ├── dev_stats.rs
    │
    ├── player/
    │   ├── mod.rs
    │   ├── game_mode.rs
    │   ├── controller.rs
    │   ├── collision.rs
    │   └── spectator.rs
    │
    └── voxel/
        ├── mod.rs
        ├── chunk.rs
        ├── world.rs
        ├── chunk_manager.rs
        ├── terrain.rs
        ├── mesher.rs
        ├── render.rs
        ├── targeting.rs
        ├── interaction.rs
        ├── modifications.rs
        └── debug.rs

## Module Responsibilities

### Player

`player/mod.rs`

Player plugin, player entity, temporary body mesh, camera creation, crosshair and shared player constants.

`player/game_mode.rs`

Creative and Spectator mode state and mode switching.

`player/controller.rs`

Creative movement, gravity, jumping, creative flight, mouse look and first/third-person camera control.

`player/collision.rs`

Custom voxel-aware AABB collision detection, ground detection and automatic 0.5 m terrain stepping.

`player/spectator.rs`

Free-fly noclip Spectator movement.

### Voxel Engine

`voxel/chunk.rs`

Voxel representation, chunk dimensions and chunk-local voxel storage.

`voxel/world.rs`

Loaded chunk storage, world/chunk coordinate conversion and world-space voxel access.

`voxel/chunk_manager.rs`

3D spherical world streaming, asynchronous chunk generation, generation queues, unloading and mesh-update scheduling.

`voxel/terrain.rs`

Procedural terrain generation and terrain noise.

`voxel/mesher.rs`

Voxel-to-mesh conversion and hidden-face removal.

`voxel/render.rs`

Rendered chunk registry, Bevy mesh/entity synchronization and empty-mesh handling.

`voxel/targeting.rs`

Custom voxel raycasting, current target state and adaptive block highlighting.

`voxel/interaction.rs`

Voxel breaking, placement and affected-chunk remeshing.

`voxel/modifications.rs`

In-memory persistence of player voxel modifications across chunk unload/reload cycles.

`voxel/debug.rs`

F3 rendered-chunk boundary visualization.

### Development

`dev_stats.rs`

Runtime development and profiling HUD.

`main.rs`

Application bootstrap, plugin registration, rendering configuration and lighting.

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

The core voxel engine and initial player prototype are functional.

Completed:

- Rust / Bevy development environment
- Custom voxel storage
- 0.5 m voxel resolution
- 16³ voxel chunks
- Multi-chunk world
- Negative world coordinates
- 3D spherical chunk streaming
- Effectively unlimited horizontal procedural world
- Configurable finite vertical world
- Asynchronous terrain generation
- Streaming work budgets
- Procedural terrain
- Runtime voxel breaking
- Runtime voxel placement
- Continuous voxel interaction
- In-memory modification persistence
- Custom voxel raycasting
- Neighbor-aware chunk meshing
- Empty chunk render optimization
- Adaptive block targeting highlight
- Rendered-chunk F3 debug visualization
- Development statistics HUD
- Creative player controller
- Custom voxel collision
- Gravity
- Jumping
- 0.5 m automatic step-up
- Creative flight
- Spectator mode
- First-person camera
- Third-person camera
- Temporary visible player body
- 90° camera FOV

## Next Technical Milestones

Current likely development direction:

1. Implement greedy meshing.
2. Add runtime terrain-generation controls.
3. Improve procedural terrain shaping.
4. Add caves and underground terrain.
5. Add multiple voxel and material types.
6. Add textures and block materials.
7. Improve third-person camera collision.
8. Expand development profiling statistics.
9. Add asynchronous meshing if profiling shows it is needed.
10. Eventually introduce Survival gameplay systems.
11. Add persistent disk-based world saving when required.

## Performance Goals

Performance is a core design goal.

The engine should support:

- Large render distances
- Smooth chunk streaming
- Runtime terrain editing without noticeable frame stalls
- Multithreaded procedural generation
- Efficient chunk remeshing
- Minimal rendering of empty geometry
- Compact voxel storage
- Reduced geometry through greedy meshing
- Stable high-framerate gameplay

Optimization decisions should continue to be driven by profiling and measured bottlenecks rather than theoretical voxel counts alone.

## License

License not yet defined.
