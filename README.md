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
    Ctrl              Crouch (with ledge-fall prevention) / Swim down / Fly down
    C                 Crawl (0.45m prone posture through 1-voxel gaps)
    Z                 Camera Zoom (Hold Z + Mouse Wheel to adjust magnification)

    B                 Toggle Interaction mode (1m³ Block vs 0.5m Sub-voxel)
    Left Mouse        Break voxel / block
    Right Mouse       Place voxel / block
    Middle Mouse      Pick voxel / block

    1 - 8             Hotbar slot selection
    Mouse Wheel       Scroll hotbar slots (when not zooming)
    Q                 Clear active hotbar slot
    R                 Block shape (Tap: cycle sequentially / Hold: 10-shape circular radial menu)
    T                 Rotate block shape 90° clockwise

    ESC               Pause Menu (Settings, Restart Game, Quit) with Depth of Field blur
    E                 Inventory (32-slot palette, 3D isometric icons, Mouse Tweaks controls)
                      • Shift + Click: Quick transfer from palette to hotbar / quick clear
                      • Shift + LMB Drag: Rapid transfer into hotbar / rapid hotbar wipe
                      • LMB Drag: Paint held block across multiple hotbar slots
                      • Click outside / same block: Deselect held item
                      • Right Click: Stamp block into slot / deselect on empty space
                      • Q / Middle Click: Clear hovered hotbar slot
                      • 1 - 8: Quick assign or swap slots

    F1                Toggle Inspector (bevy_inspector_egui)
    F2                Chunk debug borders
    F3                Toggle HUD (Minimal / Extended debug)
    Shift + F3        Toggle HUD visibility (Show / Hide)
    F4                Creative / Spectator
    F5                First / Third person (64×64 Minecraft skin body model, head tracking, overlay layers, animations)
    F6                Day / Night cycle (Click: step phase / Hold: scrub time)


## Current Project Structure

```text
src/
├── core/
│   ├── dev_stats.rs
│   ├── mod.rs
│   └── noise.rs
├── environment/
│   ├── atmosphere.rs
│   ├── celestial.rs
│   ├── clouds.rs
│   ├── mod.rs
│   ├── stars.rs
│   └── time.rs
├── gameplay/
│   ├── debug.rs
│   ├── icon.rs
│   ├── interaction.rs
│   ├── interaction_mode.rs
│   ├── mod.rs
│   ├── radial_menu.rs
│   ├── shaping.rs
│   └── targeting.rs
├── generation/
│   ├── biome.rs
│   ├── caves.rs
│   ├── generator.rs
│   ├── mod.rs
│   └── strata.rs
├── menu/
│   ├── inventory.rs
│   ├── mod.rs
│   ├── pause.rs
│   └── settings.rs
├── meshing/
│   ├── async_mesher.rs
│   ├── greedy.rs
│   ├── mod.rs
│   ├── pipeline.rs
│   ├── shapes.rs
│   └── textures.rs
├── player/
│   ├── collision.rs
│   ├── controller.rs
│   ├── game_mode.rs
│   ├── hotbar.rs
│   ├── mod.rs
│   ├── model.rs
│   ├── spectator.rs
│   ├── state.rs
│   └── water.rs
├── simulation/
│   ├── fluid.rs
│   ├── lighting.rs
│   └── mod.rs
├── world/
│   ├── streaming/
│   │   ├── manager.rs
│   │   ├── mod.rs
│   │   └── queues.rs
│   ├── block.rs
│   ├── chunk.rs
│   ├── mod.rs
│   ├── modifications.rs
│   └── storage.rs
└── main.rs

assets/textures/
├── blocks/
├── environments/
├── gui/
│   └── cursors/
└── mobs/

docs/
├── context.md
└── roadmap.md
```

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
