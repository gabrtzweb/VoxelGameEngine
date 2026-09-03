# VoxelGameEngine

An experimental voxel game engine built from scratch with Rust and Bevy.

The project focuses on a fully editable procedural voxel world with a hybrid block structure:

- The world is stored using 0.5 m voxels.
- A traditional 1 m³ logical block is composed of 2 × 2 × 2 voxels.
- Each 1 m³ block therefore contains 8 individually editable voxels.
- Individual 0.5 m voxels can be destroyed and placed at runtime.
- 1 m blocks remain useful as a visual, gameplay and coordinate abstraction.

The long-term goal is to build a performant procedural voxel game with large-world streaming, runtime terrain editing, configurable generation, multiple gameplay modes, dynamic fluids and extensive development tooling.

## Current Stack

- Rust
- Bevy 0.19
- Bevy ECS and rendering
- Custom voxel storage
- Custom 3D chunk streaming
- Asynchronous terrain generation
- Custom greedy meshing
- Custom voxel raycasting
- Custom voxel collision system
- Custom player controller
- Dynamic voxel lights
- Transparent water rendering
- Custom water physics
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

The world streams procedurally and is effectively unlimited horizontally:

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

## Voxel Types

Currently implemented voxel types:

    Air
    Grass
    Dirt
    Stone
    Sand
    Water
    Light

Voxel properties are handled independently.

For example:

- Air is empty and non-collidable.
- Water is transparent and non-collidable.
- Terrain voxels are opaque and collidable.
- Light voxels are collidable and use a dedicated emissive rendering system.

The block system is being structured with future texture arrays and material expansion in mind.

## Chunk Streaming

Chunk streaming operates in three dimensions around the Creative player.

Current default render distance:

    8 chunks

The desired chunk region uses spherical distance rather than loading a full cube.

Chunks are generated and unloaded dynamically as the Creative player moves through the world.

Spectator movement intentionally does not generate additional chunks.

### Asynchronous Terrain Generation

Terrain generation runs through Bevy's asynchronous compute task pool.

Generation tasks are:

- Started with a limited number of concurrent jobs
- Prioritized by distance from the player
- Collected without blocking the main thread
- Discarded if the player moves away before their result is required

Chunk meshing currently uses a frame-budgeted main-thread queue.

Async meshing remains a future option if profiling shows that it is required.

### Streaming Work Budgets

Chunk operations are deliberately spread across frames.

Current systems limit:

- Generation tasks started per frame
- Concurrent terrain generation tasks
- Chunk unloads per frame
- Chunk mesh updates per frame

This avoids large frame-time spikes while continuously exploring the procedural world.

## Render Optimization

Loaded chunks and rendered chunks are separate concepts.

A chunk may exist in voxel memory without requiring GPU geometry.

Chunks with no exposed geometry:

- Do not receive a Bevy mesh
- Do not receive a render entity
- Do not appear in the F3 chunk visualization

This avoids rendering completely empty air chunks and fully enclosed geometry.

## Greedy Meshing

The engine now uses a custom world-aware greedy mesher.

Instead of generating one quad for every exposed voxel face, compatible neighboring faces are merged into larger quads.

The mesher currently supports:

- Hidden internal face removal
- Neighbor-aware chunk boundaries
- Greedy face merging
- Runtime remeshing
- Neighbor remeshing after boundary edits
- Multiple voxel types
- Texture-layer metadata
- Separate opaque and transparent geometry
- Water-specific face rules
- Dedicated rendering for Light voxels

Greedy meshing dramatically reduces generated vertex and triangle counts compared with the original naive mesher.

### Chunk Render Layers

Each rendered chunk may contain:

    Opaque Mesh
    Transparent Mesh

Opaque geometry currently includes:

- Grass
- Dirt
- Stone
- Sand

Transparent geometry currently includes:

- Water

Light voxels are rendered separately as emissive entities and are not included in the normal greedy chunk mesh.

This structure prepares the renderer for future materials such as:

- Glass
- Leaves
- Plants
- Transparent blocks
- Additional fluids

## Texture Array Preparation

Voxel faces already carry texture-layer information during meshing.

Current conceptual layers include:

    Grass top
    Grass side
    Dirt
    Stone
    Sand
    Water
    Light

Actual texture-array rendering has not yet been implemented.

The current vertex-color rendering is intentionally temporary.

## Procedural Terrain v2

Terrain generation is deterministic and continuous across chunk boundaries.

The current generator uses multiple scales of procedural noise.

### Large-Scale Terrain

Terrain combines:

- Macro terrain noise
- Smaller detail noise
- Multiple octaves
- Configurable persistence
- Configurable seed

This creates broader plains, hills and terrain features instead of only small noisy height variations.

### Lakes

Lakes are generated independently from the normal terrain heightmap.

The generator uses a separate low-frequency lake field to create smooth lake basins.

Lake generation includes:

- Wide lake regions
- Gradual shoreline transitions
- Basin carving
- Deep lake centers
- Flat water surfaces
- Sandy lake floors
- Sandy shoreline regions

Current maximum generated lake depth is approximately:

    10 voxels
    ≈ 5 meters

This is intentionally deep enough for the 1.8 m player to swim and fully submerge.

Generated water is currently static.

Dynamic water propagation is the next major fluid milestone.

## Runtime World Modification

Voxel changes are persisted in memory for the duration of the current game session.

If a modified chunk unloads and later reloads:

    modifications are restored

If the game is restarted:

    modifications reset

Disk-based world persistence is intentionally deferred.

## Player System

The engine has a dedicated player system separate from the voxel engine.

Current dimensions:

    Width:      0.60 m
    Height:     1.80 m
    Eye height: 1.62 m

Camera FOV:

    90 degrees

The player uses a custom AABB collision system that directly queries voxel data.

Individual physics colliders are not created for terrain voxels.

## Game Modes

Currently implemented:

- Creative
- Spectator

Planned:

- Survival

Survival gameplay is intentionally deferred while the engine and Creative development workflow are still evolving.

## Creative Mode

Creative is currently the main gameplay and development mode.

Basic controls:

    Mouse           Look
    W               Forward
    S               Backward
    A               Left
    D               Right
    Shift           Sprint
    Space           Jump

The Creative player supports:

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

## Automatic Step-Up

The player automatically climbs terrain differences of:

    0.5 meters

This matches the base voxel height.

Normal voxel terrain can therefore be traversed smoothly without repeatedly jumping.

Higher obstacles still require jumping.

The physical step occurs immediately while the camera transition is smoothed.

## Creative Flight

Double-tapping:

    Space

toggles Creative flight while outside water.

Flight controls:

    W / A / S / D   Move
    Space           Move up
    Ctrl            Move down
    Shift           Fast movement

Creative flight:

- Uses player collision
- Generates chunks
- Supports voxel interaction
- Supports automatic horizontal 0.5 m stepping

Double-tapping Space again disables flight and restores normal movement.

## First / Third Person

Press:

    F5

to toggle between first-person and third-person Creative cameras.

A temporary colored rectangular body represents the player.

The body is:

- Hidden in first person
- Visible in third person
- Visible while the camera is in Spectator mode

Third-person camera collision with terrain has not yet been implemented.

## Spectator Mode

Press:

    F4

to switch between Creative and Spectator modes.

Spectator controls:

    Mouse           Look
    W / A / S / D   Move
    Space           Move up
    Ctrl            Move down
    Shift           Fast movement

Spectator mode:

- Has no gravity
- Has no collision
- Passes through terrain
- Does not modify voxels
- Does not move the Creative player
- Does not generate new chunks

Returning to Creative mode restores the camera to the Creative player.

## Water Rendering

Water uses a dedicated transparent render layer.

Water currently has:

- Transparent rendering
- Alpha blending
- Separate transparent chunk meshes
- Hidden Water-to-Water internal faces
- Visible terrain surfaces behind Water
- Greedy meshing support

Water does not collide with the player.

## Water Physics v1

The player can now physically enter and swim through water.

The engine calculates the approximate percentage of the player's body volume currently submerged.

Submersion can therefore vary continuously from:

    0.0 = completely dry
    1.0 = completely submerged

Water movement includes:

- Reduced horizontal movement speed
- Reduced gravity
- Vertical drag
- Buoyancy
- Reduced falling velocity when entering water
- Controlled ascent
- Controlled descent

Swimming controls:

    W / A / S / D   Swim horizontally
    Shift           Swim faster
    Space           Swim upward
    Ctrl            Swim downward

Creative flight activation is temporarily suppressed while swimming so Space can be used naturally for ascent.

### Underwater Camera Effect

When the camera enters a Water voxel, a simple blue underwater overlay is enabled.

The effect disappears immediately when the camera leaves the water.

This currently works in both:

- Creative
- Spectator

More advanced underwater visuals are planned later.

## Voxel Lighting

The engine has a dedicated Light voxel type.

Select it with:

    6

A Light voxel consists of two independent runtime entities:

    Emissive visual cube
    PointLight

Separating the visual mesh from the PointLight prevents lighting from disappearing when the visible cube leaves the camera frustum.

Light voxels support:

- Emissive visual surfaces
- Dynamic local illumination
- Placement and destruction
- Chunk unload/reload
- Session modification persistence

Point-light shadow maps are currently disabled for performance.

## Environment

The world contains a basic dynamic environment system.

### Day / Night

Press:

    F6

to switch between:

    Day
    Night

Day mode includes:

- Directional sunlight
- Cascaded directional shadows
- Sky fill lighting
- Visible sun
- Distance fog

Night mode includes:

- Moonlight
- Increased nighttime ambient visibility
- Visible moon
- Night fog
- Strong contrast for artificial voxel lights

The Sun entity remains active while switching phases so directional shadow state remains stable.

### Distance Fog

Distance fog is tied to chunk render distance.

Its purpose is to visually hide the edge of the currently streamed terrain before the player can clearly see chunk loading boundaries.

Fog colors change between day and night.

## Voxel Interaction

Voxel interaction uses custom grid-based raycasting instead of physics colliders.

Controls:

    Left Mouse       Break voxel
    Right Mouse      Place voxel
    Middle Mouse     Pick targeted voxel type

Holding Left or Right Mouse repeats the action.

Current repeat interval:

    0.16 seconds

Interaction distance:

    10 meters

### Block Selection

Current quick selection:

    1   Grass
    2   Dirt
    3   Stone
    4   Sand
    5   Water
    6   Light

Middle Mouse performs Minecraft-style Pick Block:

    look at voxel
    press Middle Mouse
    targeted voxel becomes selected

Right Mouse then places the selected voxel type.

## Targeting

The crosshair continuously performs a custom voxel DDA raycast.

Target highlighting operates at the logical 1 m block level while respecting internal 0.5 m voxel geometry.

A complete block appears as a full 1 m³ outline.

If internal voxels are removed, the outline adapts to the remaining shape while suppressing unnecessary internal edges.

Water remains targetable even though it is non-collidable.

Target highlighting remains independent from developer chunk visualization.

## Developer Debug Mode

Press:

    F3

to toggle chunk boundary visualization.

Only chunks with actual rendered geometry are shown.

Completely empty air chunks are not outlined.

Debug rendering is distance-limited.

## Development Statistics

A runtime development HUD is displayed in the top-left corner.

Current statistics include:

- FPS
- Frame time
- Current game mode
- Creative flight state
- Player logical coordinates
- Player chunk coordinates
- Camera position
- Loaded chunks
- Meshed chunks
- Mesh vertex count
- Mesh triangle count
- Loaded voxel capacity
- Current target voxel

Logical player coordinates use:

    1 coordinate unit = 1 meter

even though internal voxel resolution is 0.5 m.

## Current Controls

    Mouse             Look

    W                 Forward / Swim forward
    S                 Backward / Swim backward
    A                 Left
    D                 Right

    Shift             Sprint / Fast swim / Fast flight

    Space             Jump / Swim up / Fly up
    Ctrl              Swim down / Fly down

    Left Mouse        Break voxel
    Right Mouse       Place voxel
    Middle Mouse      Pick voxel

    1                 Grass
    2                 Dirt
    3                 Stone
    4                 Sand
    5                 Water
    6                 Light

    F3                Chunk debug
    F4                Creative / Spectator
    F5                First / Third person
    F6                Day / Night

    Double Space      Toggle Creative flight

## Project Structure

    src/
    ├── main.rs
    ├── dev_stats.rs
    ├── environment.rs
    │
    ├── player/
    │   ├── mod.rs
    │   ├── game_mode.rs
    │   ├── controller.rs
    │   ├── collision.rs
    │   ├── spectator.rs
    │   └── water.rs
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
        ├── light.rs
        └── debug.rs

## Module Responsibilities

`environment.rs`

Day/night state, directional sunlight, moonlight, sky fill lighting, visible sun/moon, shadow configuration and distance fog.

### Player

`player/mod.rs`

Player plugin, player entity, temporary body mesh, camera creation, crosshair and shared dimensions.

`player/game_mode.rs`

Creative and Spectator mode switching.

`player/controller.rs`

Creative movement, gravity, jumping, flight, swimming and first/third-person camera handling.

`player/collision.rs`

Custom voxel-aware AABB collision detection, ground detection and automatic 0.5 m terrain stepping.

`player/spectator.rs`

Free-fly noclip Spectator movement.

`player/water.rs`

Player water-submersion detection and underwater camera effect.

### Voxel Engine

`voxel/chunk.rs`

Voxel types, voxel properties, chunk dimensions and chunk-local storage.

`voxel/world.rs`

Loaded chunk storage, coordinate conversion and world-space voxel access.

`voxel/chunk_manager.rs`

3D spherical streaming, asynchronous generation, unload queues and mesh-update scheduling.

`voxel/terrain.rs`

Terrain v2 generation, macro/detail noise and lake-basin generation.

`voxel/mesher.rs`

World-aware greedy meshing, texture-layer metadata and opaque/transparent geometry generation.

`voxel/render.rs`

Opaque and transparent chunk rendering, Bevy mesh synchronization and render registry.

`voxel/targeting.rs`

Custom DDA voxel raycasting and adaptive logical-block highlighting.

`voxel/interaction.rs`

Voxel breaking, placement, block selection and affected-chunk remeshing.

`voxel/modifications.rs`

In-memory persistence of runtime voxel changes across chunk unload/reload cycles.

`voxel/light.rs`

Voxel-light visual entities, PointLight entities and light lifecycle management.

`voxel/debug.rs`

F3 rendered-chunk boundary visualization.

### Development

`dev_stats.rs`

Runtime profiling and development HUD.

`main.rs`

Application bootstrap and plugin registration.

## Development Commands

Run:

    cargo run

Optimized build:

    cargo run --release

Check:

    cargo check

Format:

    cargo fmt

Lint:

    cargo clippy

Recommended before commits:

    cargo fmt
    cargo check
    cargo clippy

## Current Development Status

The main voxel-engine foundation and Creative gameplay prototype are functional.

Completed:

- Rust / Bevy development environment
- 0.5 m voxel resolution
- 16³ voxel chunks
- Negative world coordinates
- Effectively unlimited horizontal procedural world
- Configurable finite vertical world
- 3D spherical chunk streaming
- Streaming work budgets
- Async terrain generation
- Runtime chunk unloading
- In-memory modification persistence
- Terrain v2
- Broad terrain features
- Procedural deep lakes
- Grass / Dirt / Stone / Sand / Water / Light
- Runtime voxel breaking
- Runtime voxel placement
- Pick Block
- Custom voxel raycasting
- Adaptive block targeting
- World-aware greedy meshing
- Separate opaque and transparent chunk meshes
- Transparent water rendering
- Empty render optimization
- Custom player collision
- Gravity
- Jumping
- 0.5 m automatic step-up
- Creative flight
- Spectator mode
- First-person camera
- Third-person camera
- Swimming
- Buoyancy
- Underwater camera effect
- Day/night switching
- Directional sunlight
- Directional shadows
- Moonlight
- Distance fog
- Visible sun and moon
- Dynamic Light voxels
- Development statistics HUD
- F3 rendered-chunk visualization

## Next Technical Milestones

Immediate development direction:

1. Design dynamic fluid state representation.
2. Implement Water propagation v1.
3. Add active-fluid update queues and per-frame simulation budgets.
4. Support fluid updates across chunk boundaries.
5. Improve underwater visuals and water surface rendering.
6. Add caves and underground generation.
7. Add runtime terrain-generation controls.
8. Introduce texture-array rendering.
9. Add biome generation.
10. Improve third-person camera collision.
11. Profile whether asynchronous meshing is necessary.
12. Eventually introduce Survival systems.
13. Add persistent disk-based world saving when required.

## Performance Philosophy

Performance is a core design goal.

The engine should support:

- Large render distances
- Smooth chunk streaming
- Runtime terrain editing without major frame stalls
- Multithreaded procedural generation
- Efficient chunk remeshing
- Minimal empty geometry
- Compact voxel storage
- Greedy geometry reduction
- Budgeted simulation work
- Stable high-framerate gameplay

Optimization decisions should continue to be driven by profiling and measured bottlenecks rather than theoretical voxel counts alone.

## License

License not yet defined.
