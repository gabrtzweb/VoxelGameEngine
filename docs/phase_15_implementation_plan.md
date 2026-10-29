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

### Phase 15.1: Global Illumination, Sky Light Floor & Flora Normal Corrections

- **Milestone 15.1.1 — Directional Lighting Model**:
  - Update `assets/shaders/voxel_opaque.wesl` with a balanced directional shading floor:
    - Top Face ($+Y$): $1.00$
    - Bottom Face ($-Y$): $0.55$
    - North/South Faces ($\pm Z$): $0.82$
    - East/West Faces ($\pm X$): $0.70$
  - Scale ambient lighting and fill lighting in `src/environment/atmosphere.rs` and `src/environment/celestial.rs` to maintain natural contrast without crushing side faces.
- **Milestone 15.1.2 — Translucent Flora & Billboard Lighting**:
  - Update `src/meshing/shapes.rs` (`mesh_cross_billboard`): orient billboard normals towards $+Y$ ($[0.0, 1.0, 0.0]$) so plants capture ambient sky light and overhead sunlight uniformly.
  - In `assets/shaders/grass_plant.wesl` and `assets/shaders/voxel_opaque.wesl`, apply wrapped two-sided diffuse lighting:
    $$I_{\text{diffuse}} = \max(0.0, N \cdot L) \times 0.55 + 0.45$$
    ensuring flora remains luminous and translucent from all viewing angles.
- **Milestone 15.1.3 — Coated Textures Micro-Variation (`COATED_TEXTURES`)**:
  - Reverse-engineer Complementary's `CoatTextures`: sample low-frequency noise per-texel combined with block world coordinates (`floor(world_pos)`).
  - Apply organic micro-luminance modulation ($color \times noise$) on stone, dirt, wood, and terrain blocks to eliminate sterile repetition and grant a weathered, tactile surface finish.
  - Zero performance cost: a single texture tap (or procedural hash) and scalar multiplier in `voxel_opaque.wesl`.

---

### Phase 15.2: Dynamic Vegetation & Leaf Sway Simulation (Wind & Water Currents)

- **Milestone 15.2.1 — Foliage & Flower Wind Sway (`WAVING_FOLIAGE`)**:
  - Leverage `WindState` and `globals.time` in `assets/shaders/grass_plant.wesl` and `assets/shaders/voxel_opaque.wesl`.
  - Base-anchored displacement: lock $y = 0.0$ at the block root and apply sinusoidal displacement to the upper vertex tier ($y > 0.2$).
  - Introduce gust variation using low-frequency coordinate noise so flower patches undulate cohesively in travelling waves.
- **Milestone 15.2.2 — Tree Leaves Ambient Wind Rustle (`WAVING_LEAVES`)**:
  - Reverse-engineer Complementary's `DoWave_Leaves`:
    $$P_{\text{offset}} = \vec{W}_{\text{dir}} \times \sin(\text{time} \times 2.5 + \text{pos} \cdot 0.75) \times 0.025$$
  - High-frequency subtle fluttering on leaf blocks without separating quad seams between neighboring voxels.
  - Disable leaf sway when covered by snow layers (`in_snow`).
- **Milestone 15.2.3 — Aquatic Flora Fluid Undulation (`WAVING_WATER_PLANTS`)**:
  - Vertical harmonic wave propagation for Kelp and Seagrass:
    $$X_{\text{offset}} = \sin(\text{time} \times 1.8 + y \times 1.2) \times 0.04 \times (y - y_{\text{base}})$$
  - Slower, heavier cadence mimicking underwater currents and tidal surge.

---

### Phase 15.3: Voxel Ambient Occlusion (Smooth Lighting & Anti-Anisotropy)

- **Milestone 15.3.1 — 3-Neighbor Corner Sampling**:
  - In `src/meshing/greedy.rs`, for each vertex of an exposed face, sample the 3 adjacent neighbor blocks:
    - Side 1 ($S_1$)
    - Side 2 ($S_2$)
    - Corner ($C$)
  - Standard AO evaluation formula:
    $$\text{AO level} = \begin{cases} 0 & \text{if } S_1 \text{ and } S_2 \text{ are solid} \\ 3 - (S_1 + S_2 + C) & \text{otherwise} \end{cases}$$
    mapping to intensity multipliers $[0.45, 0.65, 0.82, 1.00]$.
- **Milestone 15.3.2 — Anisotropic Quad Flipping**:
  - Address interpolation distortion across quad triangulation diagonals.
  - If $AO_{00} + AO_{11} > AO_{01} + AO_{10}$, invert triangulation indices from $(0, 1, 2), (0, 2, 3)$ to $(1, 2, 3), (1, 3, 0)$.
- **Milestone 15.3.3 — Vertex Attribute Packing & Shader Integration**:
  - Pack AO into vertex attributes (or vertex color channel `color.a`).
  - Interpolate per-vertex AO smoothly across face fragments in `assets/shaders/voxel_opaque.wesl`.

---

### Phase 15.4: Pixelated Shadows & Shadow Stabilization (Complementary Reverse-Engineering)

- **Milestone 15.4.1 — Screen-Space Texel Offset Calculation**:
  - Implement `ComputeTexelOffset` in WESL using screen-space UV derivatives (`dpdx(uv)`, `dpdy(uv)`):
    $$\Delta UV = (\lfloor UV \times 16.0 \rfloor + 0.5) / 16.0 - UV$$
- **Milestone 15.4.2 — Texel-Snapped Shadow Sampling (`TexelSnap`)**:
  - Snap world position or shadow coordinate by projecting along screen gradients:
    $$\text{Pos}_{\text{snapped}} = \text{Pos} + \text{dpdx}(\text{Pos}) \times \text{offset}_x + \text{dpdy}(\text{Pos}) \times \text{offset}_y$$
  - Eliminate sub-pixel shadow shimmering; shadow silhouettes align strictly to the 16×16 pixel grid of the target voxel.
- **Milestone 15.4.3 — Configurable Pixelated Lighting**:
  - Expose `Pixelated Shadows` and `Pixelated AO` toggles in Settings (`local_config.json`), allowing players to choose between smooth modern shading and Complementary Reimagined retro-pixel styling.

---

### Phase 15.5: HDR Bloom & Emissive Blocks Pipeline

- **Milestone 15.5.1 — Bevy Bloom Integration**:
  - Attach `BloomSettings` to `PlayerCamera` in `src/player/camera.rs` with calibrated thresholds (threshold $\ge 1.0$, soft knee $0.5$).
- **Milestone 15.5.2 — Emissive Block Catalog & HDR Boosting**:
  - Register emission values for torches, lanterns, lava, redstone, glowing geodes, and fireflies.
  - In `assets/shaders/voxel_opaque.wesl`, write emission color values $> 1.0$ into fragment output to trigger bloom thresholding naturally.

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
