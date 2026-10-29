# Phase 15 Implementation Plan: Visual Aesthetics, Shaders & Atmospheric FX Overhaul

## 1. Executive Summary & Aesthetic Objectives

Phase 15 upgrades the visual fidelity of the *VoxelGameEngine* from a raw, flat-lit prototype into a cohesive, atmospheric voxel world inspired by state-of-the-art Minecraft shaders—specifically reverse-engineered techniques from **Complementary Shaders - Reimagined (r5.9.3)**.

### Target Aesthetic Pillars

- **Warm, Cohesive Lighting**: Eliminate pitch-black lateral block faces and glowing/dark flora artifacts by introducing directional sky light ratios and wrapped lighting.
- **Organic Motion**: Gentle wind swaying for surface flora, leaves, and crops, alongside fluid undulation for submerged aquatic vegetation.
- **Deep Geometric Contrast**: Per-vertex Voxel Ambient Occlusion (Smooth Lighting) with anisotropic quad flipping.
- **Pixel-Perfect Stylization**: Crisp pixelated shadows and lighting snapped to the 16×16 block texel grid (`TexelSnap`), eradicating shadow shimmering and crawling.
- **Luminescent Highlights**: HDR Bloom for torches, lanterns, lava, glowing crystals, and enchanted ores.
- **Living Fluids**: Atmospheric water featuring depth-based absorption, shoreline contact foam, and animated underwater caustics.
- **Atmospheric Grandeur**: Volumetric light shafts (God Rays) piercing tree canopies and mountain horizons.

---

## 2. Diagnosis of Current Visual Deficits

| Deficit | Technical Root Cause | Resolution Strategy |
| :--- | :--- | :--- |
| **Harsh Lateral Block Darkening** | Standard Bevy PBR Lambertian cosine law ($N \cdot L \le 0$) drops direct sunlight to zero on faces opposing the sun. Combined with high sun illuminance ($8,500$ lux) vs. low ambient ($1,000$ lux) at $EV_{100} = 11.1$, side faces appear nearly pitch black. | Implement a Minecraft-style directional lighting floor (Top: $1.0$, North/South: $0.82$, East/West: $0.68$, Bottom: $0.52$) blended with diffuse sky illumination in `assets/shaders/voxel_opaque.wesl`. |
| **Dark Cross-Mesh Flora** | `mesh_cross_billboard` in `src/meshing/shapes.rs` assigns pure horizontal normals ($N = [\pm 0.707, 0.0, \pm 0.707]$). Overhead sunlight yields $N_y \approx 0$, leaving plants starved of direct light, with backfaces in shadow. | Reorient billboard normals upwards ($N = [0.0, 1.0, 0.0]$) and implement two-sided wrapped transmission ($N \cdot L \times 0.5 + 0.5$) in `assets/shaders/grass_plant.wesl` and `assets/shaders/voxel_opaque.wesl`. |
| **Shadow Shimmering & Flickering** | Directional sunlight rotates continuously and camera moves across continuous world coordinates. Bevy Cascaded Shadow Maps (CSM) suffer sub-pixel texel crawling against sharp 1.0m block edges. | Implement screen-space texel snapping (`TexelSnap` via `dFdx(uv)` / `dFdy(uv)`) inspired by Complementary Reimagined to lock shadow and light sampling to the block's 16×16 texel grid. |
| **Static World** | Leaves, flowers, and submerged marine flora have no vertex displacement or current simulation. | Introduce wind-driven sinusoidal and Bezier vertex sway for land flora and tree leaves, and fluid harmonic sway for underwater flora in WESL shaders. |
| **Flat Geometry & Absence of Depth** | No Ambient Occlusion currently exists in the engine; crevices and corners have the same flat illumination as open plains. | Build per-vertex Voxel Ambient Occlusion in `src/meshing/greedy.rs` with automatic diagonal quad flipping to avoid triangulation anisotropy. |
| **Primitive Fluid Rendering** | Water lacks depth fogging, shore foam interaction, optical refraction, and sun caustics. | Modernize `assets/shaders/voxel_transparent.wesl` with scene depth comparison for foam, procedural normal ripples, and animated caustic projection. |

---

## 3. Sub-Phase Breakdown & Milestones

### Phase 15.1: Global Illumination, Sky Light Floor & Flora Normal Corrections (Completed)

- [x] **Milestone 15.1.1 — Directional Lighting Model**:
  - Balanced directional shading floor in `assets/shaders/voxel_opaque.wesl`: Top 1.0, Bottom 0.55, N/S 0.86, E/W 0.78.
  - Calibrated ambient lighting (2,000 lux floor) and fill light in `atmosphere.rs` and `celestial.rs` to eliminate overly dark side walls.
- [x] **Milestone 15.1.2 — Translucent Flora & Billboard Lighting**:
  - Reoriented billboard normals to upward $[0.0, 1.0, 0.0]$ in `shapes.rs` (`mesh_cross_billboard`) and `grass_plant.wesl` for uniform two-sided ambient & sun illumination.
- [x] **Milestone 15.1.3 — Coated Textures Micro-Variation (`COATED_TEXTURES`)**:
  - Integrated authentic Complementary `noise.png` via continuous planar world-space mapping (`texel_world = floor(planar_coord * 16.0) / 16.0`).
  - Added real-time strength control in Settings tab (`CoatedTextureMode`: Off, Low, Medium, High, Ultra).

---

### Phase 15.2: Dynamic Vegetation & Leaf Sway Simulation (Wind & Water Currents) (Completed)

- [x] **Milestone 15.2.1 — Foliage & Flower Wind Sway (`WAVING_FOLIAGE`)**:
  - Implemented `@vertex` shader in `assets/shaders/voxel_opaque.wesl` with base-anchored quadratic displacement (`smoothstep(0.1, 0.9, 1.0 - uv.y)`).
  - Synchronized travelling wind gusts and high-frequency flutter driven by `WindState` and `globals.time`.
  - Tied procedural grass in `uniforms.rs` to `local_config.waving_foliage`.
- [x] **Milestone 15.2.2 — Tree Leaves Ambient Wind Rustle (`WAVING_LEAVES`)**:
  - Continuous 3D world-space wave displacement for tree leaf blocks (`anim_type == 3.0`) in `voxel_opaque.wesl`, breathing and rustling without tearing voxel quad seams.
- [x] **Milestone 15.2.3 — Aquatic Flora Fluid Undulation (`WAVING_WATER_PLANTS`)**:
  - Harmonically propagating viscous wave for marine flora (Seagrass, Kelp, `anim_type == 4.0`), mimicking underwater ocean currents.

---

### Phase 15.3: Voxel Ambient Occlusion (Smooth Lighting & Anti-Anisotropy) (Completed)

- [x] **Milestone 15.3.1 — 3-Neighbor Corner Sampling**:
  - In `src/meshing/greedy.rs`, for each vertex of an exposed face, sample the 3 adjacent neighbor blocks:
    - Side 1 ($S_1$)
    - Side 2 ($S_2$)
    - Corner ($C$)
  - Standard AO evaluation formula:
    $$\text{AO level} = \begin{cases} 0 & \text{if } S_1 \text{ and } S_2 \text{ are solid} \\ 3 - (S_1 + S_2 + C) & \text{otherwise} \end{cases}$$
    mapping to intensity multipliers $[0.45, 0.62, 0.80, 1.00]$.
- [x] **Milestone 15.3.2 — Anisotropic Quad Flipping**:
  - Addressed interpolation distortion across quad triangulation diagonals.
  - If $(AO_1 + AO_3) > (AO_0 + AO_2)$, invert triangulation indices from $(0, 1, 2), (0, 2, 3)$ to $(1, 2, 3), (1, 3, 0)$ across the brighter diagonal.
- [x] **Milestone 15.3.3 — Vertex Attribute Packing & Shader Integration**:
  - Baked per-vertex AO into `Mesh::ATTRIBUTE_COLOR` (`color.rgb * ao_multiplier`), maintaining hardware bilinear interpolation and GPU cache efficiency without extra uniform bindings.
  - Preserved greedy meshing optimizations by merging unoccluded `[3, 3, 3, 3]` expanses while isolating corners.

---

### Phase 15.4: Pixelated Shadows & Shadow Stabilization (Complementary Reverse-Engineering)

- [x] **Milestone 15.4.1 — Screen-Space Texel Offset Calculation**:
  - Implemented `compute_texel_offset` in WESL (`assets/shaders/voxel_opaque.wesl`) using screen-space UV derivatives (`dpdx(uv)`, `dpdy(uv)`), solving the pseudo-inverse 2D projection to find the exact sub-pixel screen offset to 16×16 block texel centers.
- [x] **Milestone 15.4.2 — Texel-Snapped Shadow Sampling (`TexelSnap`)**:
  - Implemented `texel_snap_vec3` to project `pbr_input.world_position` along screen position gradients:
    $$\text{Pos}_{\text{snapped}} = \text{Pos} + \text{clamp}(\text{dpdx}(\text{Pos}) \times \text{offset}_x + \text{dpdy}(\text{Pos}) \times \text{offset}_y, -0.25, 0.25)$$
  - Snapped sampling coordinates before `apply_pbr_lighting(pbr_input)`, locking cascaded shadow lookups and PCF filtering strictly to the block's 16×16 texel grid, eliminating camera movement shimmering and sub-pixel shadow crawling.
- [x] **Milestone 15.4.3 — Configurable Pixelated Lighting**:
  - Wired `pixelated_shadows` into `voxel_params.z` in `src/meshing/pipeline.rs`, dynamically synced with `LocalConfig` and toggleable in the Settings menu under the Shaders tab.

---

### Phase 15.5: HDR Bloom, Emissive Blocks & Dynamic Handheld Lighting

- [x] **Milestone 15.5.1 — Bevy Bloom Integration**:
  - Attached calibrated `Bloom` (`Bloom::NATURAL`, `intensity: 0.20`, `prefilter.threshold: 1.0`, `threshold_softness: 0.5`) to `PlayerCamera` in `src/player/mod.rs` operating over `Camera3d` with `bevy::camera::Hdr`.
  - Added runtime synchronization system `sync_camera_bloom` hooked into `LocalConfig::hdr_bloom` and toggleable in Settings.
- [x] **Milestone 15.5.2 — Emissive Block Catalog & HDR Boosting**:
  - Boosted self-illuminating output values for light-emitting blocks (torches, lamps, glowing crystals, redstone, lava, magma) into HDR space ($> 1.0$) in both `assets/shaders/voxel_opaque.wesl` ($2.5\times$) and `assets/shaders/voxel_transparent.wesl` ($2.2\times$), triggering the bloom threshold naturally with vibrant halos.
- [x] **Milestone 15.5.3 — Dynamic Handheld / Entity Lighting**:
  - Implemented `src/player/dynamic_light.rs` (`update_player_dynamic_light`).
  - When the player selects/holds any light-emitting block or item in the active hotbar slot, a real-time point light (`PointLight`) automatically spawns at the player's hand position, matching the item's `light_color()`, emission level, and intensity.
  - Light smoothly tracks the player without expensive shadow map passes (`shadow_maps_enabled: false`), delivering OptiFine/Complementary style dynamic lighting at near-zero GPU overhead.
  - Added `dynamic_lighting: bool` to `LocalConfig` and an interactive toggle in `Settings > Shaders`.

---

### Phase 15.6: Advanced Water & Fluids Overhaul

- **Milestone 15.6.1 — Scene Depth Comparison & Shoreline Contact Foam**:
  - Query scene depth prepass in `assets/shaders/voxel_transparent.wesl`.
  - Where $(\text{SceneDepth} - \text{WaterDepth}) < 0.20\text{m}$, generate animated, turbulent white foam fringes along sand, dirt, and stone coastlines.
- **Milestone 15.6.2 — Animated Surface Caustics**:
  - Dual-layer scrolling Voronoi / simplex caustic pattern projected along underwater floor faces.
  - Sunlight intensity modulation: caustics brighten under clear sun and fade under overcast/rainy weather.
- **Milestone 15.6.3 — Optical Refraction & Depth Color Absorption**:
  - Smooth Beer-Lambert exponential depth absorption: shallow water is crystal-clear turquoise/aquamarine; deep oceans transition to rich navy blue.

---

### Phase 15.7: Volumetric Fog & Crepuscular Rays (God Rays)

- **Milestone 15.7.1 — Screen-Space Sun/Moon Light Shafts**:
  - In `assets/shaders/post_process.wesl`, implement radial occlusion blur radiating outward from sun/moon screen coordinates.
  - Shafts cast natural dark volumetric shadows when occluded by tree trunks, canopies, and terrain peaks.
- **Milestone 15.7.2 — Meteorological Fog Coupling**:
  - Connect ray intensity with Phase 14 weather states: dense sunbursts during clear mornings and damp post-rain ground mists; diffused gloom during blizzards and thunderstorms.

---

### Phase 15.8: Biome Tonemapping & Environmental Grading

- **Milestone 15.8.1 — Biome Color Correction**:
  - Tailored color balance, saturation, and contrast curves per biome (e.g. warm golden highlights in Savannah/Desert, moody cool blues in Taiga/Glaciers, muted sepia in Swamps).
- **Milestone 15.8.2 — Subterranean Darkness Grading**:
  - Dynamic desaturation and deep contrast when descending into deep caverns ($Y < 0$).

---

## 4. Verification & Testing Strategy

1. **Compilation Soundness**: `cargo check` and `cargo test` executed and verified after each sub-phase.
2. **Visual Inspection**:
   - Verify lateral face brightness during noon, sunset, and night.
   - Verify flora brightness with overhead sun (ensuring both sides of cross-mesh quads are well-lit).
   - Test wind sway on flowers, wheat crops, oak leaves, and underwater kelp.
   - Inspect shadow edges with camera stationary and moving to verify zero crawling/trembling.
   - Test Bloom around torches, glowing ores, and lava at night.
   - Test water coastline foam and caustics along riverbanks and ocean shores.
3. **Performance Profiling**:
   - Maintain 144+ FPS in unlocked mode on discrete GPUs (Intel Arc B580 baseline).
   - Zero per-frame memory allocation or vertex buffer reallocation spikes in the render pipeline.
