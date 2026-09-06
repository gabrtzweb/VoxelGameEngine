# VoxelGameEngine

An experimental voxel game engine built from scratch with Rust and Bevy.

The project focuses on a fully editable procedural voxel world with a hybrid block structure:

- The world is stored using 0.5 m voxels.
- A traditional 1 m³ logical block is composed of 2 × 2 × 2 voxels.
- Each 1 m³ block therefore contains 8 individually editable voxels.
- Individual 0.5 m voxels can be destroyed and placed at runtime.
- 1 m blocks remain useful as a visual, gameplay and coordinate abstraction.

The long-term goal is to build a performant procedural voxel game with large-world streaming, runtime terrain editing, configurable generation, multiple gameplay modes, dynamic fluids and extensive development tooling.

## License

License not yet defined.
