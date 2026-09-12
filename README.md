# VoxelGameEngine

An experimental voxel game engine built from scratch with Rust and Bevy.

The project focuses on a fully editable procedural voxel world with a hybrid block structure:

- The world is stored using 0.5 m voxels.
- A traditional 1 m³ logical block is composed of 2 × 2 × 2 voxels.
- Each 1 m³ block therefore contains 8 individually editable voxels.
- Individual 0.5 m voxels can be destroyed and placed at runtime.
- 1 m blocks remain useful as a visual, gameplay and coordinate abstraction.

The long-term goal is to build a performant procedural voxel game with large-world streaming, runtime terrain editing, configurable generation, multiple gameplay modes, dynamic fluids and extensive development tooling.

## Current Game Controls

    Mouse             Look

    W                 Forward / Swim forward
    S                 Backward / Swim backward
    A                 Left
    D                 Right

    Shift             Sprint / Fast swim / Fast flight

    Space             Jump / Swim up / Fly up
    Double Space      Toggle Creative flight
    Ctrl              Swim down / Fly down

    B                 Toggle Interaction mode
    Left Mouse        Break voxel
    Right Mouse       Place voxel
    Middle Mouse      Pick voxel

    1 - 8             Hotbar slot selection
    Mouse Wheel       Scroll hotbar slots
    Q                 Clear active hotbar slot
    R                 Cycle block shape (Full, Stairs, Corner Stairs, Slabs, Columns, Centered Columns)
    T                 Rotate block shape 90° clockwise

    F1                Toggle Inspector
    F3                Chunk debug
    F4                Creative / Spectator
    F5                First / Third person
    F6                Day / Night cycle (Click: step phase / Hold: scrub time)


## Current Project Structure

    src/
    ├── environment/
    │   ├── celestial.rs
    │   ├── clouds.rs
    │   └── stars.rs
    │
    ├── player/
    │   ├── collision.rs
    │   ├── controller.rs
    │   ├── game_mode.rs
    │   ├── hotbar.rs
    │   ├── mod.rs
    │   ├── spectator.rs
    │   └── water.rs
    │
    ├── voxel/
    │   ├── chunk_manager.rs
    │   ├── chunk.rs
    │   ├── debug.rs
    │   ├── fluid.rs
    │   ├── interaction_mode.rs
    │   ├── interaction.rs
    │   ├── light.rs
    │   ├── mesher.rs
    │   ├── mod.rs
    │   ├── modifications.rs
    │   ├── render.rs
    │   ├── shaping.rs
    │   ├── targeting.rs
    │   ├── terrain.rs
    │   └── world.rs
    │
    ├── dev_stats.rs
    ├── environment.rs
    └── main.rs

## Development Commands

Run:

    cargo run

Optimized build:

    cargo run --release

Recommended before commits:

    cargo fmt
    cargo check
    cargo clippy

## AI Usage Disclaimer

In the development of this project, I use AI tools as technical assistants. They are utilized across several support workflows, including:

* Code translation and syntax refactoring
* Bug fixing, debugging assistance, and troubleshooting
* Brainstorming technical solutions and architectural planning
* Researching performance optimization strategies and documentation

At the same time, all creative direction, core design concepts, texture creation, and artistic vision remain entirely human-driven.

## License

License not yet defined.
