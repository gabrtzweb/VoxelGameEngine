# Voxel Game Engine - Current Project Context

This is an attempt to develop an experimental voxel engine and game, built from scratch using Rust and Bevy.

---

## Core Concepts

Built from scratch in Rust on **Bevy 0.20** with custom **WESL** shaders, the engine simulates three distinct world modes: a pure-code radial **Laboratory** showcase atlas (`2,880 × 2,880` containing all 78 surface biomes), an infinite procedurally streamed **Sandbox** world (with randomized seeds and safe dry-land spawn resolution), and a handcrafted bounded **Adventure Mode** world (`12,800 × 7,200` Full / `6,400 × 3,600` Medium loaded from 6 `3200 × 1800` map layers) — all built on 1.0 m voxels organized into 16 × 16 × 16 chunks. It pairs greedy meshing with custom multi-shape block geometry across 736 active blocks, directional placement, multi-layer solid geological strata & ores (legacy caves removed pending Phase 11 subterranean refactor), 27 Macro-Biomes / 78 surface Micro-Biomes, elevated lakes & stepped rivers, dynamic fluids, procedural GPU grass, and a continuous day/night, lunar, seasonal, and dynamic weather simulation.

On top of the voxel foundation, the project implements a **Survival-first** gameplay loop (alongside Creative and Spectator modes) featuring tool-based harvesting, block durability, a 36-slot inventory and hotbar, a 3-race / 9-subrace attribute and skill system, an interactive minimap and full-screen world map, a pure-code localized UI with rebindable controls and resolution scaling, and a spatialized multi-channel audio engine.

---

## Current World Scale, Time & Environment

### Spatial & Vertical Scale

- **Voxel & Chunk Geometry**:
  - 1.0 m voxels; full 1 m³ blocks form the base unit of the world (0.5 m sub-voxels removed).
  - Each chunk contains 16 × 16 × 16 voxels (4,096 voxels, 16 m × 16 m × 16 m physical section).
- **Horizontal Streaming**: Effectively unlimited along X and Z in **Sandbox Mode** (random seed per world + safe dry-land spawn search); bounded `2,880 × 2,880` radial 78-biome atlas in **Laboratory Mode** (Creative default, auto-preloaded on World Map); bounded `12,800 × 7,200` (Full) or `6,400 × 3,600` (Medium) handcrafted world in **Adventure Mode**.
- **Vertical Limits**: Chunk Y `-16` to `+16` (`-256` to `+256` blocks; 512 blocks total playable height).
- **Elevation Landmarks**:
  - Indestructible Bedrock / Mantle floor: `Y = -254` (100% solid rock & ore strata below surface; subterranean caves & underground biomes removed until the Phase 11 Subterranean Refactor)
  - Sea level: `Y = 12` (baseline terrain elevation: `Y = 16`; Adventure Mode supports elevated highland lakes and stepped rivers above `Y = 12`)
  - Cloud layers: 2 active layers at `Y = 168.0 m` (cumulus) and `Y = 215.0 m` (altocumulus), with a 3rd configurable cirrus layer at `Y = 275.0 m`
- **Render Distance**: Default 12 chunks (LOD chunk radius also set to 12); chunks are generated, meshed, and unloaded dynamically as the player moves.

### Time, Calendar & Seasons

- **Day / Night Cycle**: 24 real-world minutes (`1440.0 s`, 1 real minute = 1 in-game hour), divided into 4 phases (`Morning`, `Noon`, `Evening`, `Night`). Tap `F6` to step phases or hold `F6` to scrub time continuously.
- **Lunar Cycle & Months**: 28 days per month across 8 moon phases (`New Moon`, `Waxing Crescent`, `First Quarter`, `Waxing Gibbous`, `Full Moon`, `Waning Gibbous`, `Last Quarter`, `Waning Crescent`), each lasting 3–4 days with night illuminance scaling from `0.05` (New Moon) to `1.0` (Full Moon).
- **Seasons & Year**: 4 seasons (`Spring`, `Summer`, `Autumn`, `Winter`) lasting 3 months (84 days) each, totaling 12 months / 336 days per year.

### Weather & Wind

- **Weather States (7 total)**: `Clear`, `Rain`, `HeavyRain`, `Thunderstorm`, `Snow`, `Blizzard`, and `DustStorm` (tap `F7` to cycle manually; `Shift+F7` or hold `F7` to restore automatic simulation).
- **Dynamic Cycle & Transitions**: Smooth ~3 s cross-fades between states, 45 s of post-precipitation ground mist after rain/storms, and automatic scheduling (60% Clear for 8–16 min, 20% Rain for 4–8 min, 10% Heavy Rain, 10% Thunderstorm).
- **3D Harmonic Wind**: Continuous dual-harmonic wind field scaling with elevation (`Y = 50` calm valleys to `Y = 90` mountain gusts) and amplified during storms, blizzards, and dust storms.

### World Physics

- **Gravity & Jumping**: Gravity `24.0 m/s²`, terminal velocity `50.0 m/s`, base jump velocity `8.0 m/s`.
- **Base Movement Speeds**: Walk `5.0 m/s`, Sprint `8.0 m/s`, Crouch `55%` (`2.75 m/s`), Crawl `35%` (`1.75 m/s`), Fly `7.0 m/s`, Fast Fly `15.0 m/s`.
- **Water Physics**: Water gravity `4.5 m/s²`, buoyancy `6.5 m/s²`, swim speed `3.0 m/s` (`4.5 m/s` fast swim), max ascend/descend speed `4.0 m/s`.

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
- Camera Zoom: dynamic magnification (`Z` + Mouse Wheel up to 20x zoom) with mouse sensitivity dampening
- First-Person Camera: true first-person body view (head hidden to prevent interior clipping, looking down reveals animated chest, arms, and legs)
- Third-Person Camera (`F5`): orbiting camera with configurable distance (2.5 m–8.0 m), optional camera smoothing, selectable `Centered` or `Over-the-Shoulder` framing, optional `Free 360° Orbit` (rotates freely around the stationary character without turning the body), dynamic raycast block collision prevention to avoid clipping underground/walls, and independent head pitch/yaw tracking

Player Model, 3-Race / 9-Subrace System & Held Items:

- Modular voxel character rig with reactive PBR materials driven by `SelectedSubrace` palettes and `CharacterRace` silhouettes.
- **3 classic playable core races × 3 subraces each (9 playable subraces)**:
  - **Humans**: `Northern Human` (`Nordling`), `Southern Human` (`Sunlander`), `Coastal Human` (`CoastalHuman`)
  - **Elves**: `Wood Elf` (`WoodElf`), `Dark Elf` (`DarkElf`), `High Elf` (`HighElf`)
  - **Dwarves**: `Mountain Dwarf` (`MountainDwarf`), `Deep Dwarf` (`DeepDwarf`), `Hill Dwarf` (`HillDwarf`)
  - Legacy `Orc`, `Dragonborn`, and `Tiefling` rigs are preserved in `CharacterRace` for future NPC/mob reuse.
- **16-Attribute `RaceAttributes` Profile**: Per-subrace stats covering Health, Stamina, Defense, Move Speed, Swim Speed, Jump Height (2-block elven leap), Footstep Noise (stealth audio scaling), Mining Speed, Mining Reach, Cave Vision, Fall Resistance, Knockback Resistance, Cold/Heat Resistance, XP Gain, and Carry Slots — with movement speed, swim speed, jump height, footstep volume, and block reach actively wired in gameplay.
- 1st-person and 3rd-person held item rendering directly attached to the player's hand with natural resting posture and action swing/recoil animations.
- Procedural locomotion: dynamic walk/sprint leg & arm pendulum swings, idle breathing sway, crouch torso tilt, prone crawling strokes, streamlined flutter-kick swimming, airborne jump poses, and 4-state flight animations (ascent, descent, fast flight, hover).

Game Modes Currently Implemented (`F4` cycles `Survival → Creative → Spectator`):

- **Survival** (default on world start — stamina, oxygen, fall damage, block durability, tool harvesting, item consumption, and respawn at `WorldSpawnPoint`)
- **Creative** (flight, map teleport, unlimited catalog & conjuring pick-block)
- **Spectator** (noclip flight mode in 3rd-person offset while the character remains standing in place)

The player uses a custom AABB collision system that directly queries voxel data. Individual physics colliders are not created for terrain voxels.

---

## Roadmap Status & Development Transition

The engine has concluded two full 20-phase development milestones:

- **Version 0.1 — Foundation Era (Phases 1–20, Completed)**: Core engine architecture, multi-chunk streaming, greedy meshing, procedural terrain, physics, fluid dynamics, celestial environments, survival interfaces, and SIMD/memory optimizations.
- **Version 0.2 — Modernization, Worldbuilding & Deep Systems (Phases 1–20, Completed)**: Full voxel catalog and geological strata, 9-slot 480×270 Dark Iron UI, Bevy 0.20 and WESL migration, hardware profiles and FSR 1.0, wild flora and botanical systems, 6D multi-noise 44-biome generation, Main Menu and deferred loading, spatial audio, 8-race custom voxel characters, agricultural crop simulation, smooth fluid dynamics and 3D item drops, dynamic weather, AO/water/bloom/god-ray shaders, streaming and LOD performance engineering, multi-language localization, rebindable controls, GPU procedural grass, and multi-chunk world feature generation.
- **Version 0.3 (Current, In Progress)**: Active development milestone delivering engine-wide performance optimization, world terrain polish, atmospheric/gameplay particles, pure-code UI & searchable 78-biome Journal Compendium, the 3×3 race/subrace attribute & 60-skill system, Main Menu & Survival core systems, tri-mode world architecture (`Laboratory`, `Sandbox`, `Adventure`) with World Map Atlas Overlays & async Full-World Preload, 736 active blocks, and cross-system display/audio/input/UI polish.

Individual phases and detailed milestones are tracked exclusively in `docs/project_roadmap.md`.

---

## Important Rules for Assistance and Collaboration

- Inspect current files first before replacing systems.
- Current controls and project structure can be found inside `README.md`.
- Keep the engine fully functional during migration.
- Validate changes with `cargo fmt`, `cargo check`, and `cargo clippy`.
- Do not add unnecessary comments to the code.
- Keep the code 100% in English.
