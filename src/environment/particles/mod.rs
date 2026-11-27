mod ambient_fish;
mod dragonflies;
mod fluid_bubbles;
mod fluid_splashes;
mod shooting_stars;
mod water_godrays;
mod wind_streaks;
mod world_particles;

pub use ambient_fish::{AmbientFishBatchMesh, AmbientFishPool, setup_ambient_fish, update_ambient_fish};
pub use dragonflies::{DragonflyBatchMesh, DragonflyPool, setup_dragonflies, update_dragonflies};
pub use fluid_bubbles::{FluidBubbleBatchMesh, FluidBubblePool, setup_fluid_bubbles, update_fluid_bubbles};
pub use fluid_splashes::{
    WaterSplash3DBatchMesh, WaterSplash3DPool, setup_water_splashes_3d, update_water_splashes_3d,
};
pub use shooting_stars::{
    ShootingStarBatchMesh, ShootingStarPool, setup_shooting_stars, update_shooting_stars,
};
pub use water_godrays::{
    WaterGodRay3DBatchMesh, WaterGodRay3DPool, setup_water_godrays_3d, update_water_godrays_3d,
};
pub use wind_streaks::{WindStreakBatchMesh, WindStreakPool, setup_wind_streaks, update_wind_streaks};
pub use world_particles::{
    WorldParticleBatchMesh, WorldParticleBurst, WorldParticleBurstQueue, WorldParticlePool,
    setup_world_particles, update_world_particles,
};

pub(crate) use world_particles::smooth_in_out_alpha;
