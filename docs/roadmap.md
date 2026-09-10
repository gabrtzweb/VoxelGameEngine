# Voxel Game Engine - Development Roadmap

This document outlines the planned development phases for the voxel game engine, prioritizing foundational architectural systems before gameplay tools, fluids, and procedural generation.

---

## Phase 1: Core Rendering & Texture-Array Architecture (In Progress)
- [x] **Texture-Array Shader & Pipeline**:
  - Implemented custom WGSL shader in [assets/shaders/voxel.wgsl](file:///c:/Users/rodri/OneDrive/Documentos/Rodrigo/Projects/VoxelGameEngine/assets/shaders/voxel.wgsl) via Bevy's `ExtendedMaterial<StandardMaterial, VoxelMaterialExtension>`.
  - Mapped texture array and sampler to `@group(#{MATERIAL_BIND_GROUP})` bindings 100 and 101, preserving standard PBR lighting, directional shadows, and distance fog.
- [x] **Pixel-Art Texture Asset Pipeline & Dynamic Variant Discovery**:
  - Implemented in [src/voxel/texture.rs](file:///c:/Users/rodri/OneDrive/Documentos/Rodrigo/Projects/VoxelGameEngine/src/voxel/texture.rs) using `build_voxel_texture_array()`.
  - Loads 16×16 PNG textures with nearest-neighbor sampling (`ImageSampler::nearest()`) into a hardware 2D Texture Array (`TextureDimension::D2`).
  - Auto-discovers multiple texture variants per block type (`block_{name}.png`, `block_{name}1.png`, `block_{name}2.png`, etc.) without code changes.
  - Ensured all 6 faces of a voxel share the same texture (uniform grass styling).
- [x] **Mesher Integration & Deterministic Spatial Randomization**:
  - Implemented in [src/voxel/mesher.rs](file:///c:/Users/rodri/OneDrive/Documentos/Rodrigo/Projects/VoxelGameEngine/src/voxel/mesher.rs).
  - Supplies the layer index per vertex via `Mesh::ATTRIBUTE_UV_1`, with `fract(uv)` in WGSL tiling greedy-meshed quads cleanly without stretching.
  - Selects variants via an integer spatial hash of each voxel's 3D world coordinate (`world_voxel`), guaranteeing consistent random distributions with zero flickering across chunk remeshes.
- [ ] **Phase 1 Polish & Additional Refinements**:
  - Fine-tuning variant distributions, additional block assets, and visual adjustments before proceeding to Phase 2.

---

## Phase 2: Atmosphere, Celestial Bodies & Dynamic Sky
- **4-Phase Day & Night Cycle**: Continuous in-game clock with 4 discrete phases (`Morning`, `Noon`, `Evening`, `Night`).
- **F6 Time Controls**: Click to step between phases; hold to continuously advance/scrub time.
- **Stylized Celestial Bodies**: Pixelated, cuboid sun and moon meshes honoring the engine's stylized visual identity (no smooth spheres).
- **8 Moon Phases**: Daily moon phase progression cycling every 8 in-game days.
- **Atmosphere & Skybox**: Procedural sky gradient, fading nighttime starfield, and dynamic fog/ambient light color transitions.
- **Stylized Cloud System**: Planar voxel/pixelated cloud grid drifting at constant altitude.

---

## Phase 3: Gameplay, Inventory & Sub-Voxel Shaping Tools
- **10-Slot Hotbar GUI**: 10 selectable item slots mapped to keys `1` through `0` and scrollable via the mouse wheel.
- **Sub-Voxel Block Shaping Tool (`R` Key)**: Cycle targeted 1 m³ blocks between 2×2×2 sub-voxel configurations (Full Block, Stair, Horizontal Slab, Vertical Slab).
- **Block Rotation Tool**: Rotate the sub-voxel matrix around the block origin to align stairs and slabs with player orientation.
- **Collision Calibration**: Validate and tune the custom AABB physics stepper (`AUTO_STEP_HEIGHT = 0.5 m`) against sub-voxel stairs and slabs.

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
