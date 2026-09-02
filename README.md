# VoxelGameEngine

A voxel game engine experiment built with Rust and Bevy.

The project focuses on a highly granular destructible voxel world where the base voxel size is 25 cm, allowing significantly more terrain detail than traditional 1-meter voxel systems.

## Goals

- 0.25 m base voxel resolution
- Fully destructible and placeable individual voxels
- Procedural terrain generation
- Infinite world streaming
- Chunk-based world architecture
- Multithreaded terrain generation and meshing
- Real-time terrain generation controls
- Free-fly developer camera
- Runtime voxel editing
- Persistent world modifications

## Initial Technical Targets

- Engine: Bevy 0.19
- Language: Rust
- Voxel size: 0.25 m
- Initial chunk size: 32 x 32 x 32 voxels
- Initial render test: 9 x 9 chunks
- Target: smooth 144 FPS development environment

## Development

Main development commands:

    cargo run
    cargo check
    cargo fmt
    cargo clippy

## Status

Early development.

Current focus:

- Development environment
- Free camera
- Voxel data representation
- Chunk system
- Procedural generation
- Chunk meshing

## License

License not yet defined.
