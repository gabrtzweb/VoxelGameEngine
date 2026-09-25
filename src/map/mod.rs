pub mod cache;
pub mod color;
pub mod minimap;
pub mod world_map;

pub use cache::MapCache;
pub use minimap::MinimapPlugin;
pub use world_map::WorldMapPlugin;

use crate::world::VoxelWorld;
use bevy::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapCache>()
            .add_plugins((MinimapPlugin, WorldMapPlugin))
            .add_systems(Update, update_map_cache);
    }
}

fn update_map_cache(mut map_cache: ResMut<MapCache>, world: Res<VoxelWorld>) {
    map_cache.update_dirty_columns(&world);
}
