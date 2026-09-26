# VoxelGameEngine

An experimental voxel game engine built from scratch with Rust and Bevy.

The project focuses on a fully editable procedural voxel world with 1.0 m³ blocks:

- The world is stored using 1.0 m voxels (identical to Minecraft blocks).
- Chunks have 16 × 16 × 16 voxels (16 m × 16 m × 16 m physical sections).
- Complete removal of single 0.5 m sub-voxels; native 1.0 m block shapes (`Full`, `Slab`, `Stair`, `Column`) with 3D orientations.
- Interactive tactile feedback: subtle 8-particle debris bursts with terrain collision bouncing on block break, and 0.18s elastic scale bounce on placement.
- Auto-step is calibrated to 0.50 m (50 cm) for smooth future slab stepping, requiring jumping over full 1 m blocks.
- Square gameplay Minimap HUD (North-up, compact XYZ coordinates, live player heading chevron) and full-screen interactive World Map (<kbd>M</kbd>).
- Ambient Environment procedural color noise and climate-driven biome palettes (grass, foliage, water) with smooth 5-point cross-kernel boundary blending.
- Dynamic FPS & VSync: configurable presentation modes (AutoNoVsync default, toggleable in Settings) and intelligent frame throttling (15 FPS unfocused, 30 FPS idle) with unconstrained active gameplay (250+ FPS).
- Custom typography & drop shadows: universal font asset management (`CutePixel.ttf`) applied across all in-game HUDs, menus, and interfaces with drop shadow contrast styling.
- Centered unified textured container interface: Creative palette & Personal storage with top mode toggle buttons, real-time search with hold-to-repeat backspace, Caps Lock support, and text drag/double-click selection, persistent tab memory, Shift-drag multi-slot transfers, and cinematic Depth-of-Field blur.

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
    C                 Crawl (0.45m prone posture through low openings)
    Z                 Camera Zoom (Hold Z + Mouse Wheel to adjust magnification)

    Left Mouse        Break block
    Right Mouse       Place block
    Middle Mouse      Pick block

    1 - 8             Hotbar slot selection
    Mouse Wheel       Scroll hotbar slots (when not zooming)
    Q                 Clear active hotbar slot
    R                 Block shape (Tap: cycle sequentially / Hold: 4-slice circular radial menu)
    T                 Rotate block shape 90° clockwise

    ESC               Pause Menu (Settings: Screen Mode, Render Dist, FOV, Fog, Bobbing, Fancy Sides, VSync, Dynamic FPS; Restart, Quit)
    E                 Inventory (Unified 190×152 px textured container: Personal & Creative inventory with background blur)
                      • Mode Buttons at top / Tab key: Toggle between Personal and Creative inventory (choice persists across close/reopen)
                      • Personal Inventory: Dedicated title, 8×4 grid (32 slots) storage, hotbar mirror row, action buttons
                      • Creative Palette: Dedicated title, 8×4 grid of all available blocks (including 4 wood plank varieties), pixel-art scroller, hotbar mirror row
                      • Creative Search Bar: Real-time search & filtering with Caps Lock and Shift uppercase support, holding Backspace repeats character deletion, double-click selects all, click & drag selects text range, Ctrl+A select all
                      • Shift + Click: Quick transfer between personal inventory and hotbar
                      • Shift + LMB Drag: Rapid multi-slot transfer between hotbar and personal inventory / hotbar wipe in Creative
                      • LMB Drag: Paint held block across multiple hotbar slots
                      • Click outside / backdrop: Return held item to inventory / deselect
                      • Right Click: Stamp block into slot / deselect on empty space
                      • Q / Middle Click: Clear hovered hotbar slot / drop item
                      • 1 - 8: Quick assign or swap slots
    M                 World Map (Full-screen interactive map, pan & zoom, player tracking)

    F1                Toggle Inspector (bevy_inspector_egui)
    F2                Chunk debug borders
    F3                Toggle HUD (Minimal / Extended debug)
    Shift + F3        Toggle HUD visibility (Show / Hide)
    F4                Creative / Spectator
    F5                First / Third person (64×64 Minecraft skin body model, head tracking, overlay layers, animations)
    F6                Day / Night cycle (Click: step phase / Hold: scrub time)


## Current Project Structure

```text
assets/
├── fonts/
│   └── CutePixel.ttf
├── shaders/
│   └── voxel.wgsl
├── sounds/
│   ├── footsteps/
│   └── music/
│       ├── game/
│       └── menu/
├── textures/
│   ├── blocks/
│   ├── environments/   
│   │   └── celestial/
│   │       └── moon/
│   ├── interfaces/
│   │   ├── atlases/
│   │   │   └── decorations/
│   │   ├── containers/
│   │   │   └── slots/
│   │   └── cursors/
│   ├── items/
│   └── models/
└── icon.ico

docs/
├── context.md
└── roadmap.md

src/
├── core/
│   ├── dev_stats.rs
│   ├── dynamic_fps.rs
│   ├── font.rs
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
│   ├── feedback.rs
│   ├── icon.rs
│   ├── interaction.rs
│   ├── mod.rs
│   ├── radial_menu.rs
│   ├── shaping.rs
│   ├── target_hud.rs
│   └── targeting.rs
├── generation/
│   ├── biome.rs
│   ├── caves.rs
│   ├── generator.rs
│   ├── inspector.rs
│   ├── mod.rs
│   ├── strata.rs
│   └── trees.rs
├── map/
│   ├── cache.rs
│   ├── color.rs
│   ├── minimap.rs
│   ├── mod.rs
│   └── world_map.rs
├── menu/
│   ├── creative_inventory.rs
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
│   ├── inventory.rs
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

## Artificial Intelligence Usage Disclaimer

In the development of this project, I use AI tools as technical assistants. They are utilized across several support workflows, including:

* Code translation and syntax refactoring
* Bug fixing, debugging assistance, and troubleshooting
* Brainstorming technical solutions and architectural planning
* Researching performance optimization strategies and documentation

At the same time, all creative direction, core design concepts, texture creation, and artistic vision remain entirely human-driven.

## Third-Party Assets & Textures Disclaimer

This project is currently an experimental engine and game in active early development. It is not being commercially distributed, sold, or packaged. 

A subset of the block textures currently present in the codebase are used strictly as temporary internal placeholders for prototyping, testing meshing pipelines, and visual debugging. All credit and copyright belong to their respective original creators:

* **Ashen** by [Aim_Boot](https://modrinth.com/resourcepack/ashen) (All Rights Reserved)
* **Excalibur** by [Maffhew](https://modrinth.com/resourcepack/excal) (Licensed under CC BY-NC-ND 3.0)

These assets will be replaced with original artwork or permissively licensed open assets (e.g., CC0) prior to any public demo, distribution, or release.

## License

License not yet defined.
