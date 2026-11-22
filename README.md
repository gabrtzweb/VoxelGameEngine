# VoxelGameEngine

**Current Version:** `Alpha 0.3.0`

This is an attempt to develop an experimental voxel game engine built from scratch with Rust and Bevy (`0.20`).

The project focuses on a fully editable procedural voxel world with 1.0 m³ blocks.

The long-term goal is to build a performant procedural voxel game with large-world streaming, runtime terrain editing, configurable generation, multiple gameplay modes, dynamic fluids, and extensive development tooling.

More information about how the project works and future plans can be found in:

- [project_context.md](docs/project_context.md) — Current engine architecture and implemented systems
- [project_roadmap.md](docs/project_roadmap.md) — Development milestones and active roadmap

## Current Game Controls

All primary controls can also be rebound in-game via **Settings / Controls**.

```text
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
Right Mouse       Place block (3-axis directional placement for logs, bark, and pillars; 6-face for crystals & coral fans)
Middle Mouse      Pick block

1 - 9             Hotbar slot selection (9 primary slots)
Mouse Wheel       Scroll hotbar slots (when not zooming)
Q                 Drop active hotbar item (3D item drop)
R                 Block shape (Tap: cycle / Hold: 4-slice radial menu; locked on decorative/custom blocks)
T                 Rotate block shape 90° clockwise (locked on non-rotatable blocks)

ESC               System Menu & 6-Tab Top Bar (Inventory, Skills, Journal, Quests, World Map, System)
Q / E (in menu)   Cycle between top bar menu tabs
Tab               Personal Inventory / Creative Catalog
H                 Skills & Character Sheet (60 skills across 4 general trees + dynamic Racial tree)
J                 Journal (Live World Almanac, Compendium: 44 Biomes, Bestiary, Flora, Blocks, Items, Statistics)
K                 Quests & Objectives
M                 World Map (Full-screen interactive map with right sidebar)

F1                Toggle Inspector (bevy_egui)
F2                Chunk debug borders
F3                Toggle HUD (FPS Only / Compact / Full debug)
Shift + F3        Toggle HUD visibility (Show / Hide)
F4                Creative / Spectator
F5                First / Third person (3-race / 9-subrace custom voxel character rig, head tracking, 3D held items & animations)
F6                Day / Night cycle (Click: step phase / Hold: scrub time)
F7                Weather cycle (Click: step weather / Shift+F7 or Hold: dynamic simulation)
```

## Current Project Structure

```text
assets/
├── fonts/
│   ├── ari-w9500-condensed-bold.ttf
│   ├── ari-w9500-condensed-display.ttf
│   └── ari-w9500-condensed.ttf
├── lang/
│   ├── en_us.json
│   └── pt_br.json
├── shaders/
│   ├── grass_blades.wesl
│   ├── grass_plant.wesl
│   ├── post_process.wesl
│   ├── voxel_opaque.wesl
│   ├── voxel_prepass.wesl
│   └── voxel_transparent.wesl
├── sounds/
│   ├── ambience/
│   ├── blocks/
│   ├── footsteps/
│   ├── liquids/
│   ├── music/
│   ├── ui/
│   └── weather/
├── textures/
│   ├── blocks/
│   ├── environments/
│   ├── foliage/
│   ├── interfaces/
│   ├── items/
│   ├── masks/
│   ├── models/
│   ├── particles/
│   └── shaders/
└── icon.ico

docs/
├── blocks_design.md
├── project_context.md
└── project_roadmap.md

src/
├── audio/
│   ├── ambience.rs
│   ├── block_audio.rs
│   ├── footsteps.rs
│   ├── material.rs
│   ├── mod.rs
│   ├── music.rs
│   ├── settings.rs
│   ├── sfx.rs
│   ├── ui.rs
│   └── weather_audio.rs
├── core/
│   ├── config.rs
│   ├── controls.rs
│   ├── dev_stats.rs
│   ├── dynamic_fps.rs
│   ├── font.rs
│   ├── localization.rs
│   ├── math.rs
│   ├── mod.rs
│   ├── noise.rs
│   ├── render_scale.rs
│   ├── ui_scale.rs
│   ├── window_aspect.rs
│   └── window_clip.rs
├── environment/
│   ├── foliage/
│   │   ├── config.rs
│   │   ├── generator.rs
│   │   ├── material.rs
│   │   ├── mod.rs
│   │   ├── particles.rs
│   │   ├── trail.rs
│   │   └── uniforms.rs
│   ├── weather/
│   │   ├── particles/
│   │   │   ├── buffer.rs
│   │   │   ├── emitters.rs
│   │   │   ├── mod.rs
│   │   │   └── renderer.rs
│   │   ├── lightning.rs
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   └── wind.rs
│   ├── atmosphere.rs
│   ├── celestial.rs
│   ├── clouds.rs
│   ├── mod.rs
│   ├── particles.rs
│   ├── post_process.rs
│   ├── stars.rs
│   └── time.rs
├── gameplay/
│   ├── crops.rs
│   ├── debug.rs
│   ├── feedback.rs
│   ├── icon.rs
│   ├── interaction.rs
│   ├── item_drops.rs
│   ├── mod.rs
│   ├── radial_menu.rs
│   ├── shaping.rs
│   ├── target_hud.rs
│   └── targeting.rs
├── generation/
│   ├── biome.rs
│   ├── caves.rs
│   ├── flora.rs
│   ├── generator.rs
│   ├── inspector.rs
│   ├── mod.rs
│   ├── ores.rs
│   ├── strata.rs
│   └── trees.rs
├── map/
│   ├── cache.rs
│   ├── color.rs
│   ├── minimap.rs
│   ├── mod.rs
│   └── world_map.rs
├── menu/
│   ├── controls.rs
│   ├── creative_inventory.rs
│   ├── journal.rs
│   ├── loading.rs
│   ├── main_menu.rs
│   ├── menu_bar.rs
│   ├── mod.rs
│   ├── pause.rs
│   ├── quests.rs
│   ├── race_selection.rs
│   ├── settings.rs
│   ├── skills.rs
│   └── theme.rs
├── meshing/
│   ├── shapes/
│   │   ├── fixtures.rs
│   │   ├── mod.rs
│   │   ├── pillars_walls.rs
│   │   └── slabs_stairs.rs
│   ├── async_mesher.rs
│   ├── culling.rs
│   ├── greedy.rs
│   ├── lod.rs
│   ├── mod.rs
│   ├── pipeline.rs
│   └── textures.rs
├── player/
│   ├── camera.rs
│   ├── collision.rs
│   ├── controller.rs
│   ├── dynamic_light.rs
│   ├── game_mode.rs
│   ├── hotbar.rs
│   ├── inventory.rs
│   ├── mod.rs
│   ├── model.rs
│   ├── movement.rs
│   ├── race.rs
│   ├── spectator.rs
│   ├── state.rs
│   └── water.rs
├── simulation/
│   ├── crops.rs
│   ├── fluid.rs
│   ├── lighting.rs
│   └── mod.rs
├── world/
│   ├── block/
│   │   ├── catalog.rs
│   │   ├── connection.rs
│   │   ├── data.rs
│   │   ├── mod.rs
│   │   ├── props.rs
│   │   ├── shape.rs
│   │   └── voxel.rs
│   ├── streaming/
│   │   ├── loader.rs
│   │   ├── lod_scheduler.rs
│   │   ├── manager.rs
│   │   ├── mod.rs
│   │   ├── planner.rs
│   │   ├── queues.rs
│   │   └── unloader.rs
│   ├── cache.rs
│   ├── chunk.rs
│   ├── mod.rs
│   ├── modifications.rs
│   └── storage.rs
├── lib.rs
└── main.rs

tests/
├── suite/
│   ├── audio.rs
│   ├── block.rs
│   ├── chunk.rs
│   ├── config.rs
│   ├── connection.rs
│   ├── controls.rs
│   ├── crop_gameplay.rs
│   ├── crops.rs
│   ├── culling.rs
│   ├── foliage.rs
│   ├── localization.rs
│   ├── menu.rs
│   ├── noise.rs
│   ├── ores.rs
│   ├── texture.rs
│   └── weather.rs
└── integration_tests.rs
```

## Development Commands

Run:

```bash
cargo run
```

Fast optimized dev build:

```bash
cargo run --profile dev-fast
```

Release build:

```bash
cargo run --release
```

Run automated test suite:

```bash
cargo test
```

Recommended before commits:

```bash
cargo fmt
cargo check
cargo clippy
```

## Artificial Intelligence Usage Disclaimer

In the development of this project, I use AI tools as technical assistants. They are utilized across several support workflows, including:

- Code translation and syntax refactoring
- Bug fixing, debugging assistance, and troubleshooting
- Brainstorming technical solutions and architectural planning
- Researching performance optimization strategies and documentation

At the same time, all creative direction, core design concepts, texture creation, and artistic vision remain entirely human-driven.

## Third-Party Assets & Textures Disclaimer

This project is an experimental, independent voxel game engine in active early development. It is not being commercially distributed, monetized, or sold.

The texturing aesthetic and art direction of the game are heavily inspired by the visual style and rustic low-fantasy design found in classic 16x16 resource packs, primarily:

- **Ashen** by [Aim_Boot](https://modrinth.com/resourcepack/ashen) (Visual Style & Art Inspiration)
- **Excalibur** by [Maffhew](https://modrinth.com/resourcepack/excal) (Aesthetic Inspiration & Reference)

While the project adopts this distinctive visual language, the textures are not direct or identical 1:1 copies. Virtually all textures in the engine have been custom-crafted, repainted, recolored, expanded with animation frames, or tailored specifically for this engine. Both creators are credited here as a matter of creative inspiration, artistic acknowledgement, and legal safety.

## License

License not yet defined.
