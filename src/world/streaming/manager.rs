use std::collections::{HashMap, HashSet};

use bevy::{
    prelude::*,
    tasks::Task,
};

use crate::{
    generation::{BiomeType, CaveGenerator, ClimateGenerator, StrataGenerator, TerrainGenerator},
    meshing::{
        ChunkLod, ChunkMeshRegistry, ChunkMeshingTask, LodMeshRegistry,
        remove_chunk_render, remove_lod_render,
    },
    player::PlayerSet,
    simulation::lighting::{VoxelLightRegistry, remove_chunk_lights},
    world::{
        chunk::Chunk,
        modifications::WorldModificationStore,
        storage::VoxelWorld,
        streaming::queues::ChunkStreamingQueues,
    },
};

use super::{loader::*, lod_scheduler::*, planner::*, unloader::*};

pub const DEFAULT_RENDER_DISTANCE: i32 = 8;
pub const DEFAULT_LOD_RENDER_DISTANCE: i32 = 8;
pub const DEFAULT_SIMULATION_DISTANCE: i32 = 8;

pub const WORLD_MIN_CHUNK_Y: i32 = -16;
pub const WORLD_MAX_CHUNK_Y: i32 = 16;

pub const MAX_GENERATION_TASKS_IN_FLIGHT: usize = 16;
pub const MAX_GENERATION_TASKS_STARTED_PER_FRAME: usize = 8;

pub const MAX_LOD_TASKS_IN_FLIGHT: usize = 8;
pub const MAX_LOD_TASKS_STARTED_PER_FRAME: usize = 4;
pub const MAX_LOD_MESH_UPLOADS_PER_FRAME: usize = 16;

pub const MAX_CHUNK_UNLOADS_PER_FRAME: usize = 128;
pub const MAX_LOD_UNLOADS_PER_FRAME: usize = 64;

pub const NEIGHBOR_DIRECTIONS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

#[derive(Resource, Clone, Reflect)]
pub struct ChunkStreamingSettings {
    pub render_distance: i32,
    pub lod_render_distance: i32,
    pub simulation_distance: i32,
    pub mesh_upload_budget: usize,
    pub max_meshing_tasks_in_flight: usize,
    pub max_meshing_tasks_started_per_frame: usize,
}

impl Default for ChunkStreamingSettings {
    fn default() -> Self {
        Self {
            render_distance: DEFAULT_RENDER_DISTANCE,
            lod_render_distance: DEFAULT_LOD_RENDER_DISTANCE,
            simulation_distance: DEFAULT_SIMULATION_DISTANCE,
            mesh_upload_budget: 12,
            max_meshing_tasks_in_flight: 16,
            max_meshing_tasks_started_per_frame: 8,
        }
    }
}

#[derive(Resource, Default)]
pub struct ChunkStreamingState {
    pub last_player_chunk: Option<IVec3>,
    pub last_camera_fwd: Option<Vec2>,
    pub desired_chunks: HashSet<IVec3>,
    pub desired_lod_columns: HashMap<IVec2, ChunkLod>,
}

#[derive(Component)]
pub struct ChunkGenerationTask {
    pub coordinate: IVec3,
    pub task: Task<GeneratedChunk>,
}

pub struct GeneratedChunk {
    pub coordinate: IVec3,
    pub chunk: Chunk,
}

#[derive(Component)]
pub struct LodMeshingTask {
    pub coordinate: IVec2,
    pub lod: ChunkLod,
    pub task: Task<CompletedLodMesh>,
}

pub struct CompletedLodMesh {
    pub coordinate: IVec2,
    pub lod: ChunkLod,
    pub meshes: crate::meshing::greedy::ChunkMeshes,
    pub map_chunks: Vec<(IVec2, crate::map::MapChunk)>,
}

pub struct ChunkStreamingPlugin;

impl Plugin for ChunkStreamingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .init_resource::<ChunkStreamingSettings>()
            .init_resource::<ChunkStreamingState>()
            .init_resource::<ChunkStreamingQueues>()
            .init_resource::<WorldModificationStore>()
            .init_resource::<VoxelLightRegistry>()
            .insert_resource(TerrainGenerator::default())
            .register_type::<ChunkStreamingSettings>()
            .register_type::<TerrainGenerator>()
            .register_type::<ClimateGenerator>()
            .register_type::<CaveGenerator>()
            .register_type::<StrataGenerator>()
            .register_type::<BiomeType>()
            .add_systems(
                Update,
                (
                    handle_terrain_generator_reload,
                    plan_chunk_streaming,
                    process_chunk_unloads,
                    process_lod_unloads,
                    start_generation_tasks,
                    collect_generation_tasks,
                    start_lod_meshing_tasks,
                    collect_lod_meshing_tasks,
                )
                    .chain()
                    .after(PlayerSet::Movement)
                    .run_if(|session: Option<Res<crate::world::WorldSession>>| {
                        session.map_or(true, |s| s.is_active)
                    }),
            );
    }
}

pub fn handle_terrain_generator_reload(
    generator: Res<TerrainGenerator>,
    mut last_version: Local<u32>,
    mut state: ResMut<ChunkStreamingState>,
    mut world: ResMut<VoxelWorld>,
    mut modifications: ResMut<WorldModificationStore>,
    mut mesh_registry: ResMut<ChunkMeshRegistry>,
    mut lod_registry: ResMut<LodMeshRegistry>,
    mut light_registry: ResMut<VoxelLightRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    generation_tasks: Query<(Entity, &ChunkGenerationTask)>,
    meshing_tasks: Query<(Entity, &ChunkMeshingTask)>,
    lod_tasks: Query<(Entity, &LodMeshingTask)>,
    mut commands: Commands,
    mut queues: ResMut<ChunkStreamingQueues>,
    mut map_cache: Option<ResMut<crate::map::MapCache>>,
) {
    if generator.version == *last_version {
        return;
    }

    *last_version = generator.version;

    // 1. Despawn all in-flight generation tasks
    for (entity, _) in &generation_tasks {
        if let Ok(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.try_despawn();
        }
    }

    // 2. Despawn all in-flight meshing tasks
    for (entity, _) in &meshing_tasks {
        if let Ok(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.try_despawn();
        }
    }

    // 2b. Despawn all in-flight LOD meshing tasks
    for (entity, _) in &lod_tasks {
        if let Ok(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.try_despawn();
        }
    }

    // 3. Despawn and remove all existing chunk meshes
    let mesh_coords: Vec<IVec3> = mesh_registry.iter_coordinates().copied().collect();
    for coordinate in mesh_coords {
        remove_chunk_render(&mut commands, coordinate, &mut mesh_registry, &mut meshes);
    }

    // 3b. Despawn and remove all existing LOD meshes
    let lod_coords: Vec<IVec2> = lod_registry.iter_coordinates().copied().collect();
    for coordinate in lod_coords {
        remove_lod_render(&mut commands, coordinate, &mut lod_registry, &mut meshes);
    }

    // 4. Remove all chunk point lights
    let light_coords: Vec<IVec3> = world.iter_chunks().map(|(&c, _)| c).collect();
    for coordinate in light_coords {
        remove_chunk_lights(&mut commands, coordinate, &mut light_registry);
    }

    // 5. Purge world storage and recorded modifications
    world.clear();
    modifications.clear();
    if let Some(ref mut cache) = map_cache {
        cache.clear();
    }

    // 6. Clear streaming queues
    queues.load.clear();
    queues.unload.clear();
    queues.remesh.clear();
    queues.remesh_set.clear();
    queues.lod_load.clear();
    queues.lod_unload.clear();

    // 7. Reset player tracking so plan_chunk_streaming immediately queues full reload around player
    state.last_player_chunk = None;
    state.last_camera_fwd = None;
    state.desired_chunks.clear();
    state.desired_lod_columns.clear();
}

pub fn neighbors(coordinate: IVec3) -> impl Iterator<Item = IVec3> {
    NEIGHBOR_DIRECTIONS
        .into_iter()
        .map(move |direction| coordinate + direction)
}
