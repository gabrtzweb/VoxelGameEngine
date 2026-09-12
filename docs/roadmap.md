# Voxel Game Engine - Development Roadmap

This document outlines the planned development phases for the voxel game engine, prioritizing foundational architectural systems before gameplay tools, fluids, and procedural generation.

---

## Phase 1: Core Rendering & Texture-Array Architecture (Completed)
- [x] **Texture-Array Shader & Pipeline**:
  - Implemented custom WGSL shader in [assets/shaders/voxel.wgsl](VoxelGameEngine/assets/shaders/voxel.wgsl) via Bevy's `ExtendedMaterial<StandardMaterial, VoxelMaterialExtension>`.
  - Mapped texture array and sampler to `@group(#{MATERIAL_BIND_GROUP})` bindings 100 and 101, preserving standard PBR lighting, directional shadows, and distance fog.
- [x] **Pixel-Art Texture Asset Pipeline & Dynamic Variant Discovery**:
  - Implemented in [src/voxel/texture.rs](VoxelGameEngine/src/voxel/texture.rs) using `build_voxel_texture_array()`.
  - Standardized textures by category prefixes (`terr_`, `rock_`, `liqd_`, `emit_`) in `assets/textures/blocks/`.
  - Loads 16×16 PNG textures with nearest-neighbor sampling (`ImageSampler::nearest()`) into a hardware 2D Texture Array (`TextureDimension::D2`).
  - Auto-discovers multiple texture variants per block type (`{name}.png`, `{name}1.png`, `{name}2.png`, etc.) without code changes, with 4 variants each for grass, dirt, stone, and sand.
  - Ensured all 6 faces of a voxel share the same texture (uniform grass styling).
- [x] **Mesher Integration, Deterministic Spatial Randomization & Color Tinting**:
  - Implemented in [src/voxel/mesher.rs](VoxelGameEngine/src/voxel/mesher.rs).
  - Supplies the layer index per vertex via `Mesh::ATTRIBUTE_UV_1`, with `fract(uv)` in WGSL tiling greedy-meshed quads cleanly without stretching.
  - Selects variants via an integer spatial hash of each voxel's 3D world coordinate (`world_voxel`), guaranteeing consistent random distributions with zero flickering across chunk remeshes.
  - Integrates vertex color tinting (`Mesh::ATTRIBUTE_COLOR`) for grayscale textures (grass and water) to support biome and environmental tint variations while preserving greedy-mesh boundaries.
- [x] **Phase 1 Polish & Additional Refinements**:
  - Replaced legacy textures with 4-letter categorized variants (`terr_`, `rock_`, `liqd_`, `emit_`).
  - Implemented grayscale tinting pipeline for grass and water in the mesher and shader with calibrated brightness.
  - Added vertical-strip animated texture pipeline (e.g. 16×576 `liqd_water_still` with 36 frames) driven seamlessly on the GPU using WGSL `globals.time` and vertex frame-count attributes at 6.0 FPS.
  - Softened shadow contrast by boosting daytime ambient illuminance from 62 to 450 lux and tuning exposure.
  - Calibrated sub-voxel water geometry: surface water voxels have a 40cm height offset (yielding a 90cm water height for 1m blocks matching Minecraft fluids and eliminating shoreline z-fighting).
  - Enabled Creative flight activation (double-tap Space) while swimming and submerged.
  - *(Note: Rendering top water faces from underneath while submerged and fluid flow mechanics are deferred to Phase 4).*

---

## Phase 2: Atmosphere, Celestial Bodies & Dynamic Sky (Completed)
- [x] **4-Phase Day & Night Cycle**:
  - Implemented continuous in-game astronomical clock in [src/environment.rs](VoxelGameEngine/src/environment.rs) (`time_of_day: 0.0..1.0`, default 600s cycle) categorized into 4 discrete phases (`Morning`, `Noon`, `Evening`, `Night`).
  - Automatically tracks day count on midnight-to-morning cycle rollover, advancing the calendar and moon phase.
- [x] **F6 Time Controls**:
  - Implemented dual-mode input handling: tapping/clicking `F6` (<0.25s) steps immediately to the next discrete phase (`Morning` -> `Noon` -> `Evening` -> `Night`).
  - Holding `F6` (>0.25s) continuously scrubs time forward smoothly at an accelerated pace (0.22 day units/sec).
- [x] **Flat Textured Celestial Billboards**:
  - Implemented in [src/environment/celestial.rs](VoxelGameEngine/src/environment/celestial.rs).
  - Celestial bodies are rendered as flat billboard quads facing the camera rather than 3D cubes, scaled with distinct proportions: Sun at 52m and Moon at 40m (at 100m distance).
  - Uses Minecraft-style **Additive Blending** (`AlphaMode::Add`): black pixels (`[0, 0, 0]`) act as mathematical zero (leaving sky colors 100% untouched without dark halos), while luminous RGB values physically add light to the skybox.
  - Enhanced solar corona with HDR base color luminance and non-linear power curve for a prominent, glowing ring and rays against the daytime sky.
  - Set `fog_enabled: false` on celestial materials so distance fog never draws solid boxes over the sun or moon.
  - Sun casts 4-level cascaded directional shadows with daytime sky fill lighting.
- [x] **8 Moon Phases from Spritesheet**:
  - Automatically slices the 128×64 spritesheet (`assets/textures/environments/moon_phases.png`) into 8 discrete 32×32 pixel textures (Full Moon, Waning Gibbous, Third Quarter, Waning Crescent, New Moon, Waxing Crescent, First Quarter, Waxing Gibbous).
  - Uses additive blending to render crisp glowing crescents and phases against the night sky without square artifacts.
  - Active texture swaps dynamically with `day_count % 8`, and directional moonlight intensity/shadows scale based on the active phase's illumination factor.
- [x] **Atmosphere, Dynamic Fog & Color Transitions**:
  - Continuous 4-stop piecewise-linear palette interpolation across Morning, Noon, Evening, and Night.
  - Dynamically blends `ClearColor`, `GlobalAmbientLight` (color and brightness), camera `DistanceFog` (color and directional scattering exponent), and camera `Exposure` (EV100).
- [x] **Night Starfield**:
  - Implemented in [src/environment/stars.rs](VoxelGameEngine/src/environment/stars.rs).
  - 250 procedural stars placed at 140m distance (behind clouds at 80m and celestial bodies at 100m in Bevy's back-to-front transparent render queue), ensuring clouds naturally occlude stars.
  - GPU-batched instanced entities sharing a single quad mesh and procedural 16×16 soft circular texture.
  - Uses additive blending (`AlphaMode::Add`), `fog_enabled: false`, rotating around the celestial polar axis, and smoothly fades from daytime (0.0) to HDR night sparkle (1.6).
- [x] **Stylized Cloud System**:
  - Implemented in [src/environment/clouds.rs](VoxelGameEngine/src/environment/clouds.rs).
  - Renders the 256×256 texture from `assets/textures/environments/clouds.png` on a large horizontal plane (1600m × 1600m) at altitude Y = 80m.
  - Uses native alpha blending, nearest-neighbor sampling, and `fog_enabled: false`.
  - Drifts at a gentle, relaxed speed (1.8 m/s in X, 0.6 m/s in Z) across the sky without popping, with time-of-day color tinting.

---

## Phase 3: Gameplay, Inventory & Sub-Voxel Shaping Tools (Completed)
- [x] **8-Slot Hotbar GUI**:
  - Implemented in [src/player/hotbar.rs](VoxelGameEngine/src/player/hotbar.rs).
  - 8 selectable item slots rendered with dark translucent backing, active gold selection border, slot indices (1..8), and 2D pixel-art icons from `assets/textures/items/`.
  - Seamless input handling: select directly via number keys `1`–`8`, scroll forward/backward via the mouse wheel, and clear active slot with `Q`.
  - Automatically synchronizes with `SelectedVoxel` and middle-click block picking.
- [x] **Sub-Voxel Block Shaping Tool (`R` Key)**:
  - Implemented in [src/voxel/shaping.rs](file:///c:/Users/Rodrigo/Documents/BevyProjects/VoxelGameEngine/src/voxel/shaping.rs).
  - Targets 1m³ blocks in Block interaction mode and cycles their 2×2×2 sub-voxel layout on each `R` press: Full Block (8 voxels) -> Stair (6 voxels) -> Upside-Down Stair (6 voxels) -> Corner Stair (5 voxels) -> Bottom Slab (4 voxels) -> Top Slab (4 voxels) -> Vertical Slab (4 voxels) -> Column (2 voxels) -> Centered Column (2 stacked centered voxels).
  - Preserves the targeted block's material and updates lighting, chunk meshes, and persistence immediately.
  - Centered columns are composed of 2 independent stacked centered voxels (0.5m × 0.5m centered footprint with 0.25m margin on all 4 sides), allowing either voxel to be broken or built on top independently.
  - Bidirectional Voxel-mode centered placement: placing on the floor directly beneath a hanging centered column or stacking vertically automatically aligns and connects centered voxels without falling back to corner grid placement.
- [x] **Connected Block Placement on Slabs & Sub-Voxels**:
  - Implemented in [src/voxel/targeting.rs](file:///c:/Users/Rodrigo/Documents/BevyProjects/VoxelGameEngine/src/voxel/targeting.rs) and [src/voxel/interaction.rs](file:///c:/Users/Rodrigo/Documents/BevyProjects/VoxelGameEngine/src/voxel/interaction.rs).
  - Blocks placed against slabs, stairs, upside-down stairs, or non-full shapes align directly to the hit surface rather than the rigid 2x2x2 block grid, seamlessly completing bottom slabs into full blocks and stacking slabs/blocks above with zero floating air gaps.
  - Full support for composite emergent structures (e.g. half-column / half-slab blocks), with block-mode outline highlights accurately rendering both the centered column sub-voxel and standard sub-voxel layers simultaneously.
- [x] **Block Rotation Tool (`T` Key)**:
  - Rotates the targeted block's sub-voxels 90° clockwise around the vertical Y-axis, allowing stairs, upside-down stairs, corner stairs, slabs, and columns to face in any cardinal direction.

---

## Phase 4: Dynamic Fluid Simulation & Boundary Mechanics
- **Dynamic Water Propagation**: Cellular automaton simulation queue for liquid flow and downward/horizontal spreading.
- **Cross-Chunk Fluid Updates**: Propagate water across chunk boundaries, ensuring neighbor chunks are updated and remeshed cleanly.
- **Boundary Handling for Unloaded Chunks**: Queue fluid flow arriving at unloaded boundaries without stalling streaming threads.
- **Water Surface & Underwater Visuals**: Offset water top faces to eliminate z-fighting, add animated surface flow UVs, and apply underwater camera fog immersion.

---

## Phase 5: Advanced World Generation, Biomes & Caves
- **Biome System**: Macro-scale climate noise (continentalness, temperature, humidity) driving diverse surface palettes and height profiles.
- **3D Caves & Underground Generation**: 3D noise functions for caverns, ravines, and underground aquifers.
- **Realistic Strata**: Deeper rock layers, mineral deposits, and varied surface soil depths.
- **Runtime Generation Controls**: Live tuning of world seeds, frequencies, and cave thresholds via `bevy_inspector_egui`.

---

## Phase 6: Engine Optimization & Scalability (Future Milestone)
- **LOD Render Distance**: Downsampled greedy meshes for distant chunks.
- **Extremity Bound Checking**: Early skipping of completely empty or solid chunks during collision and meshing.
- **Noise Up-sampling & Caching**: Coarse 3D noise sampling with trilinear interpolation.
- **RLE Runtime Voxel Data**: Run-Length Encoded chunk storage to minimize memory footprint.
