pub mod chunk;
pub mod chunk_manager;
pub mod debug;
pub mod fluid;
pub mod interaction;
pub mod interaction_mode;
pub mod light;
pub mod mesher;
pub mod modifications;
pub mod render;
pub mod shaping;
pub mod targeting;
pub mod terrain;
pub mod texture;
pub mod world;

pub use chunk::{CHUNK_VOLUME, VOXEL_SIZE};

pub use chunk_manager::ChunkManagerPlugin;

pub use debug::VoxelDebugPlugin;

pub use fluid::FluidSimulationPlugin;

pub use interaction::VoxelInteractionPlugin;

pub use interaction_mode::InteractionMode;

pub use render::{ChunkMeshRegistry, VoxelMaterial};

pub use shaping::ShapingPlugin;

pub use targeting::TargetingPlugin;

pub use world::VoxelWorld;
