# Voxel Game Engine - Current Project Context

This is an attempt to develop an experimental voxel engine and game, built from scratch using Rust and Bevy.

---

## Core Concepts

- **1.0 m Voxel World**: The world is stored using 1.0 m voxels (identical to Minecraft blocks). Sub-voxels are completely eliminated; full 1.0 m³ blocks form the atomic spatial unit.
- **Chunk Geometry**: Chunks contain 16 × 16 × 16 voxels (16 m × 16 m × 16 m physical sections), with 4,096 voxels per chunk.
- **Block Shapes & Directional Placement**:
  - Native 1.0 m block shapes (11 total variants): `Full`, `Slab`, `Stair`, `Column`, `Torch`, `Basket`, `Pebble`, `PebbleCluster`, `CrossBillboard`, `Bamboo`, and `Cocoon` (0.75m × 0.875m dedicated bounds for `SpiderEgg` and `SilkCocoon`). Features custom meshes, passable non-solid collision for cross-billboards/mushrooms, low stepping for pebbles, and strict 3px column collision for bamboo stalks and leaves.
  - Runtime shape-locking protection (`is_shape_locked`): blocks such as bird nests, hay nests, bee/wasp hives, spider eggs, cocoons, and custom-meshed flora cannot be cycled into stairs/slabs or rotated via player controls.
  - Basket & Crate architectures: `Basket` features an authentic custom interior cavity with 2px rims, interior floor quad, and 4 inward-facing walls. `Crate` is consolidated as a 100% solid, fully textured opaque wooden container block.
  - 3-axis directional placement (`is_axis_aligned`): all 12 tree species logs (`*Log`) and wood bark (`*Wood`), `BloodyBone`, `BoneBlock`, `BloodyFlesh`, and `FleshBlock`. Automatically aligns along the X, Y, or Z placement axis based on hit normal, rotating face textures and swapping UVs in greedy meshing. Contextual front-facing blocks (Bookshelves, Bee Nests, Wasp Nests) automatically face the player upon placement.
  - Automatic floor-support cascading: blocks requiring floor support (`requires_floor_support`, including pebbles, mushrooms, floor torches, vertical bamboo stalks/leaves, aquatic vegetation like seagrass/kelp, and coral fans) automatically break when their support block is broken.
- **615-Block Engine Catalog & Geological Systems Foundation**:
  - 615 registered engine voxel definitions (discriminants 0..=613) across 11 thematic categories (`rocks`, `trees`, `soils`, `ores`, `caverns`, `oceans`, `organics`, `decorations`, `mushrooms`, `vegetation`, `fluids`/`systems`).
  - Geological rock strata: 10 primary mother rocks (`Dolomite`, `Slate`, `Gabbro`, `Andesite`, `Limestone`, `Argillite`, `Granite`, `Basalt`, `Sandstone`, `Marble`), 54 authentic host-rock ore veins, 6 crystal geode varieties (Amethyst, Aquamarine, Citrine, Fluorite, Pink Salt, Selenite), and indestructible Mantle bedrock at Y = -254.
- **Strict Grayscale Selective Foliage Tinting**:
  - Biome foliage tinting restricted exclusively to living grayscale canopy species, grass soils, and water.
  - Natural pre-colored pigments for seasonal, dead, and evergreen foliage are preserved without green tint multiplication.
- **Procedural 2D & Handheld Item Rendering Pipeline**:
  - Real-time procedural 2D icon generation for cubic blocks, planar cross-billboards, animated fluid frames, and handheld items across container interfaces.
- **Deterministic Natural Texture Rotation & Flipping**:
  - Implemented in WESL fragment shader (`assets/shaders/voxel_opaque.wesl`) using an integer coordinate hash of `floor(world_position)`.
  - Natural terrain blocks (stone, soils, sands, gravels) rotate and flip pseudo-randomly to eliminate grid tiling patterns over large cliff faces and cave walls without breaking CPU greedy meshing efficiency.
- **Engine Architecture & Bevy 0.20 Platform**:
  - Built on **Bevy 0.20.0** utilizing direct **`bevy_egui = "0.43"`** for developer diagnostics.
  - Complete adoption of Bevy's **WESL shader framework** (`assets/shaders/*.wesl`) replacing legacy WGSL/`naga_oil` preprocessing with explicit package imports (`import bevy_pbr::...`), `@if` conditional compilation, and standard bind groups.
  - Full modernization of UI state handling, transitioning away from deprecated `bevy::ui::Interaction` to retained components (`bevy::ui_widgets::Button`, `bevy::picking::hover::Hovered`, and `bevy::ui::Pressed`).
  - Explicit `ScreenSpaceTransmission` registration for physical fluid and translucent material rendering.
- **Outcrop Dual-Layer Rendering**:
  - Grass and snow outcrop blocks across soils and rocks render top and side layers while ensuring the bottom face (`Face::Bottom` / `Face::Down`) renders the base mineral material (e.g., Dolomite Grass bottom renders Dolomite; Peat Grass bottom renders Peat Dirt).
  - Outcrop blocks are dual-indexed to appear seamlessly in both Soils and Rocks inventory tabs.
- **Tactile Gameplay Feedback**:
  - Subtle 8-particle debris bursts with terrain collision bouncing on block break.
  - 0.18s elastic scale bounce on block placement.
- **Auto-Step**:
  - Calibrated to 0.50 m (50 cm) for smooth future slab stepping, requiring jumping over full 1 m blocks.
- **Navigation & Mapping**:
  - Square gameplay Minimap HUD (North-up, compact XYZ coordinates, live player heading chevron).
  - Full-screen interactive World Map (`M`) filling the viewport below the top bar with 1:1 aspect-corrected terrain rendering, UI-scale-accurate cursor projection, percentage-locked player heading marker, pan/zoom, RMB context menu (`Add Waypoint Here` + Creative-only `Teleport Here`), 7-icon waypoint system (cursor-following placement mode + scrollable & inline-renamable sidebar list), and 5 persisted map layers (`Player Marker`, `Waypoints`, subtle `16x16 Chunk Grid`, `Day / Night Lighting`, and smoothed `Elevation Contours`).
- **Atmosphere & Climate**:
  - 24-minute day/night astronomical clock, 28-day lunar calendar (8 moon phases), and 4 seasons.
  - Procedural color noise and climate-driven biome palettes (grass, foliage, water) with smooth 5-point cross-kernel boundary blending.
- **Dynamic FPS & VSync**:
  - Configurable presentation modes (AutoNoVsync default, toggleable in Settings) and intelligent frame throttling (15 FPS unfocused, 30 FPS idle) with unconstrained active gameplay (250+ FPS).
- **Render Scale (3D Resolution Scaling)**:
  - Real-time 3D viewport scaling (`0.50` to `1.00`, presets: 50% Performance, 66% Balanced, 75% Quality, 85% Ultra Quality, 100% Native) in `src/core/render_scale.rs`.
  - Off-screen render buffer with hardware-accelerated bilinear upsampling to the native window, keeping 2D UI, hotbars, menus, and text at crisp 100% native resolution.
- **16:9 Window Aspect Snapping**:
  - Enforced in `src/core/window_aspect.rs` (`WindowAspectPlugin`) maintaining 16:9 aspect ratio in windowed mode with minimum constraints (640×360 px) and auto-disable in fullscreen.
- **Custom Typography**:
  - Universal font asset management (`ari-w9500-condensed-display.ttf`) applied across all in-game HUDs, menus, and interfaces with drop shadow contrast styling.
- **Pure-Code Dark Iron / Forged Obsidian UI Architecture (`src/menu/theme.rs`)**:
  - **Texture-Free Containers & Buttons**: All menus, HUDs, drawers, slots, and buttons are rendered in pure Bevy UI code using a unified Dark Iron / Forged Obsidian palette (`palette`) and `ThemedButton` component (retaining only pixel-art icons in `assets/textures/interfaces/icons/`, cursors, and `main_menu_bg.png`).
  - **Shared 6-Tab Top Menu Bar & Dynamic Input Prompts (`src/menu/menu_bar.rs`, `src/core/input_prompts.rs`)**: Text-only tabs with 16×16 icons and gold active underlines (`Inventory`, `Skills`, `Journal`, `Quests`, `World Map`, `System`), `[Q]`/`[E]` tab cycling, live right-side status readout (Level, live Clock with time-of-day icon, Gold Coins), and reactive Dark Steel pixel-art keycap prompts (`assets/textures/interfaces/atlases/inputs_tilemap.png`, 34×24 grid) that automatically update when keybinds change in Controls (including 2-tile wide keys and future gamepad readiness).
  - **Left-Docked 3-Column Cascading System, Settings & Controls Drawers**: Standardized `310px` full-height left drawers (`src/menu/pause.rs`, `src/menu/settings.rs`, `src/menu/controls.rs`) supporting up to 3 cascading columns (System → Categories → Detail Panel), `World & Session` submenu, direct `Exit Game`, and 6 prioritized Settings categories (`World`, `Graphics` with Display & conditional `Borderless Mouse Lock`, `Shaders & FX` with `Procedural Grass` & `Grass Density`, `Camera` with `Camera Smoothing`, `Third-Person Distance`, `Over-the-Shoulder`, and `Free 360° Orbit`, `Audio`, and `Accessibility & UI`), including `Connected Textures`, `Menu Pause` behavior (`System Only` / `All Menus` / `Never`), 4-tier `FPS / Debug HUD`, functional `High-Contrast Crosshair`, and fade-animated `Dynamic Crosshair`.
  - **Dedicated `Skills`, `Journal` & `Quests` Screens**: `Skills` (`src/menu/skills.rs`) pairs a scrollable `310px` character & subrace attribute sheet (16 attributes including live Health/Stamina and `Inventory Slots` `27..33/36`, subrace portrait icon, racial traits) alongside 5 localized skill tree tabs (`Combat`, `Gathering`, `Crafting`, `Exploration`, and a dynamic `Racial` tree adapting to the active subrace) featuring 60 official skills with 60 bespoke 16×16 pixel-art icons; `Journal` (`src/menu/journal.rs`) is 100% localized with mouse-wheel + draggable scrollbar support, pairing a live F6-synchronized Season/Day/Clock/Weather almanac card with a subcategorized `COMPENDIUM` (`JournalProgress` live discovery for `Biomes & Regions` with 44 unique icons in `icons/biomes/`, `Bestiary`, `Flora & Agriculture`, `Blocks & Minerals`, and `Items & Equipment` via `BlockIcons`) and 4-group `Statistics` (27 bespoke icons, 18 live gameplay trackers + 9 survival/progression placeholders); `Quests` (`src/menu/quests.rs`) provides a localized active quests list and objective tracker.
  - **In-Game Hotbar, Survival HUD, Stacks & 36-Slot Inventory (`GUI Scale` 2×–4×, default 3×)**: Clean pure-code Dark Iron 9-slot hotbar chassis (border-only gold selection frame, stack count readouts, starting kit: Flint Pickaxe, Flint Axe, Flint Shovel, 8 Torches) with a separated 2-slot right-side auxiliary tray (`Off-hand` + `Quick Utility`), floating bottom-left Survival Vitals HUD (`src/player/survival_hud.rs`) horizontally aligned and scaled with the hotbar (Health, Stamina, vertical circular progress rings for Hunger/Thirst/Temperature + dynamic submerged Oxygen ring), centered Dark Iron mode/respawn toast, 64-item stacking (1 for tools) with merge/split/drop and Survival block consumption, 36-slot (4×9) Player Inventory with racial unlocked 4th-row slots (`27..33/36`, synced from `RaceAttributes::carry_slots`), and Creative-only Catalog (`+` toggle hidden in Survival).
  - **Survival Mode Core Loop, Tools & Harvesting (`GameMode::Survival`, default on world start)**: `F4` cycles `Survival → Creative → Spectator` (Spectator opens in a 3rd-person offset while keeping the character standing in place with idle breathing). Survival disables double-space flight, map Teleport, and creative catalog/conjuring pick-block (middle-click only pulls blocks already in hotbar/unlocked inventory); enforces live stamina drain (sprinting + jumping + exhaustion recovery), oxygen drain + drowning damage, racial-mitigated fall damage (`fall_damage_reduction`), and death/respawn at `WorldSpawnPoint`; adds 11 inventory-only items & flint tools (`ItemFlintShard`, `ItemPlantFiber`, `ItemPlantString`, `ToolFireStarter`, `ToolFlintAxe/Hammer/Hoe/Knife/Pickaxe/Shears/Shovel`), per-material block durability with tool & racial `mining_speed_multiplier`, 9-stage grayscale crack overlay (`assets/textures/masks/break_stages/`) + subtle Hytale-style block shake/pulse + under-crosshair mining bar, Fire Starter ignition (required for Survival campfires & placing fire), Flint Hoe soil tilling, and Shears vs. Plant Fiber vegetation drops.
  - **Item Tooltips**:
    - Centered item name readout above hotbar upon slot selection with smooth fade-out.
    - Responsive floating hover tooltips across all inventory and catalog slots.
- **Procedural Foliage & Grass Blade System**:
  - GPU-accelerated procedural grass blades emitted on top of grass turf blocks, slabs, and stairs (`FoliagePlugin`).
  - Rendered with custom WESL PBR shader (`assets/shaders/grass_blades.wesl`) featuring dynamic wind displacement, player trail trampling and recovery, and exact color matching with the underlying voxel terrain.
- **Multi-Channel Sound Engine & Spatial Audio**:
  - Comprehensive audio engine (`EngineAudioPlugin`) built on Bevy audio with independent volume mixing for Master, Weather, SFX, Music, and Ambience.
  - Material-driven block acoustics (`SoundMaterial`) covering all 614 voxels across 11 material profiles.
  - Spatialized 3D block interaction audio with distance attenuation and dynamic pitch variation.
  - Footstep system (`FootstepAudioPlugin`) with stance cadences, landing impact velocity scaling, water wading, and swimming strokes.
  - Submerged underwater loop with bubble effects, subterranean cave drones, day/night biome nature soundscapes, and multi-channel weather audio.
  - 39-track gameplay soundtrack player (`GameplayMusicPlugin`) with seamless cross-fading and natural Minecraft-style intervals.
- **Main Menu, Loading Screen & Title System**:
  - Direct engine startup into `MenuState::MainMenu` presenting a full-screen concept art background (`assets/textures/interfaces/main_menu_bg.png`).
  - Interactive two-phase flow:
    - **Splash Phase**: Large retro-pixel title banner ("VOXEL ENGINE"), version readout, and pulsing prompt ("Press any key to start") waiting for any key or mouse trigger.
    - **Action Phase**: Enshrouded-inspired 2-side layout — left side holds the game title, `Play` button (gated by `CharacterReady` until a character is chosen, then launches directly into `MenuState::Loading`), `Settings`, `Controls`, and `Quit`; right side presents a Dark Iron Character Panel displaying the selected subrace portrait, player name, stat pills, and a `Create Character >` / `Change Character >` button opening the 3-column Race & Subrace Selector (`src/menu/race_selection.rs`).
  - Seamless stack navigation via `MenuReturnTarget`, routing Settings and Controls Back buttons and Escape cleanly back to the Main Menu when opened from there, and back to Pause when in-game.
  - Smooth loading screen (`MenuState::Loading`) between Main Menu and gameplay, awaiting initial real-chunk generation and meshing with an animated progress bar, real-time chunk counter (`X / Y`), dynamic status messages, and 10 rotating tips.
  - 4 exclusive menu music tracks (`assets/sounds/music/menu/`) with Fisher-Yates non-repeating shuffle on startup, and immediate cutoff of gameplay tracks upon returning to the menu.

---

## Current World Scale

- The world is stored using 1.0 m voxels (identical to Minecraft blocks).
- Full 1 m³ blocks form the base unit of the world; 0.5 m sub-voxels are completely removed.
- Each chunk contains: 16 × 16 × 16 voxels (logical blocks).
- Total voxel capacity per chunk: 4,096 voxels.
- Physical chunk size: 16 m × 16 m × 16 m.
- The world streams procedurally and is effectively unlimited horizontally:
  - X: procedural streaming
  - Z: procedural streaming
- Vertical world limits:
  - Minimum chunk Y: -16 (-256 blocks)
  - Maximum chunk Y: +16 (+256 blocks)
  - Total playable vertical height: 512 blocks.
- Cloud plane altitude: 220.0 m.
- Current default render distance: 12 chunks (with LOD chunk radius set to 12 as well).
- Chunks are generated, meshed, and unloaded dynamically as the player moves through the world.

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

Game Modes Currently Implemented:

- Creative (currently the main gameplay and development mode)
- Spectator (noclip flight mode)

The player uses a custom AABB collision system that directly queries voxel data. Individual physics colliders are not created for terrain voxels.

---

## Roadmap Status & Development Transition

The engine has concluded two full 20-phase development milestones:

- **Version 0.1 — Foundation Era (Phases 1–20, Completed)**: Core engine architecture, multi-chunk streaming, greedy meshing, procedural terrain, physics, fluid dynamics, celestial environments, survival interfaces, and SIMD/memory optimizations.
- **Version 0.2 — Modernization, Worldbuilding & Deep Systems (Phases 1–20, Completed)**: Full voxel catalog and geological strata, 9-slot 480×270 Dark Iron UI, Bevy 0.20 and WESL migration, hardware profiles and FSR 1.0, wild flora and botanical systems, 6D multi-noise 44-biome generation, Main Menu and deferred loading, spatial audio, 8-race custom voxel characters, agricultural crop simulation, smooth fluid dynamics and 3D item drops, dynamic weather, AO/water/bloom/god-ray shaders, streaming and LOD performance engineering, multi-language localization, rebindable controls, GPU procedural grass, and multi-chunk world feature generation.
- **Version 0.3 (Current, In Progress)**: Active development milestone focused on engine-wide performance optimization, world terrain polish, atmospheric/gameplay particles, pure-code UI & Journal Compendium, the 3×3 race/subrace attribute & 60-skill system, and Main Menu modernization.

Individual phases and detailed milestones are tracked exclusively in `docs/project_roadmap.md`.

---

## Important Rules for Assistance and Collaboration

- Inspect current files first before replacing systems.
- Current controls and project structure can be found inside `README.md`.
- Keep the engine fully functional during migration.
- Validate changes with `cargo fmt`, `cargo check`, and `cargo clippy`.
- Do not add unnecessary comments to the code.
- Keep the code 100% in English.
