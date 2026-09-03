use std::collections::HashSet;

use bevy::prelude::*;

use super::{
    chunk::VOXEL_SIZE, interaction::ChunkMeshRegistry, mesher::ChunkMesher,
    modifications::WorldModificationStore, targeting::TargetingSet, terrain::TerrainGenerator,
    world::VoxelWorld,
};

const DEFAULT_RENDER_DISTANCE: i32 = 4;

const HORIZONTAL_NEIGHBORS: [IVec3; 4] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

#[derive(Resource)]
pub struct ChunkStreamingSettings {
    pub render_distance: i32,
}

impl Default for ChunkStreamingSettings {
    fn default() -> Self {
        Self {
            render_distance: DEFAULT_RENDER_DISTANCE,
        }
    }
}

#[derive(Resource, Default)]
struct ChunkStreamingState {
    last_camera_chunk: Option<IVec3>,
}

#[derive(Resource)]
struct ChunkMaterial(Handle<StandardMaterial>);

pub struct ChunkManagerPlugin;

impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .init_resource::<ChunkMeshRegistry>()
            .init_resource::<ChunkStreamingSettings>()
            .init_resource::<ChunkStreamingState>()
            .init_resource::<WorldModificationStore>()
            .insert_resource(TerrainGenerator::default())
            .add_systems(Startup, setup_chunk_material)
            .add_systems(Update, stream_chunks.before(TargetingSet::UpdateTarget));
    }
}

fn setup_chunk_material(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.52, 0.22),
        perceptual_roughness: 0.9,
        ..default()
    });

    commands.insert_resource(ChunkMaterial(material));
}

fn stream_chunks(
    mut commands: Commands,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    settings: Res<ChunkStreamingSettings>,
    mut state: ResMut<ChunkStreamingState>,
    terrain_generator: Res<TerrainGenerator>,
    modifications: Res<WorldModificationStore>,
    chunk_material: Res<ChunkMaterial>,
    mut world: ResMut<VoxelWorld>,
    mut chunk_meshes: ResMut<ChunkMeshRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let camera_position = camera.translation();

    let camera_voxel = IVec3::new(
        (camera_position.x / VOXEL_SIZE).floor() as i32,
        (camera_position.y / VOXEL_SIZE).floor() as i32,
        (camera_position.z / VOXEL_SIZE).floor() as i32,
    );

    let (camera_chunk_3d, _) = VoxelWorld::world_voxel_to_chunk(camera_voxel);

    // Horizontal streaming only for now.
    let camera_chunk = IVec3::new(camera_chunk_3d.x, 0, camera_chunk_3d.z);

    if state.last_camera_chunk == Some(camera_chunk) {
        return;
    }

    state.last_camera_chunk = Some(camera_chunk);

    let desired_chunks = desired_chunk_coordinates(camera_chunk, settings.render_distance);

    let loaded_chunks: HashSet<IVec3> = world
        .iter_chunks()
        .map(|(&coordinate, _)| coordinate)
        .collect();

    let chunks_to_load: Vec<IVec3> = desired_chunks.difference(&loaded_chunks).copied().collect();

    let chunks_to_unload: Vec<IVec3> = loaded_chunks.difference(&desired_chunks).copied().collect();

    let mut dirty_chunks = HashSet::new();

    // Chunks remaining beside an unloaded
    // chunk need their newly exposed border
    // faces rebuilt.
    for &coordinate in &chunks_to_unload {
        for neighbor in horizontal_neighbors(coordinate) {
            if desired_chunks.contains(&neighbor) {
                dirty_chunks.insert(neighbor);
            }
        }
    }

    // Generate all incoming voxel data
    // before meshing so neighboring chunks
    // are already available.
    for &coordinate in &chunks_to_load {
        let mut chunk = terrain_generator.generate_chunk(coordinate);

        // Restore player modifications made
        // earlier during this game session.
        modifications.apply_to_chunk(coordinate, &mut chunk);

        world.insert_chunk(coordinate, chunk);

        dirty_chunks.insert(coordinate);

        for neighbor in horizontal_neighbors(coordinate) {
            if desired_chunks.contains(&neighbor) {
                dirty_chunks.insert(neighbor);
            }
        }
    }

    // Remove chunks that left render distance.
    for &coordinate in &chunks_to_unload {
        world.remove_chunk(coordinate);

        if let Some(render_data) = chunk_meshes.remove(coordinate) {
            commands.entity(render_data.entity).despawn();
        }
    }

    // Spawn new chunk meshes or rebuild
    // neighboring chunks affected by streaming.
    for coordinate in dirty_chunks {
        if world.get_chunk(coordinate).is_none() {
            continue;
        }

        let rebuilt_mesh = ChunkMesher::build_mesh(&world, coordinate);

        if let Some(mesh_handle) = chunk_meshes.get(coordinate).cloned() {
            if let Some(mut mesh) = meshes.get_mut(&mesh_handle) {
                *mesh = rebuilt_mesh;
            }

            continue;
        }

        let mesh_handle = meshes.add(rebuilt_mesh);

        let entity = commands
            .spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(chunk_material.0.clone()),
                Transform::from_translation(VoxelWorld::chunk_translation(coordinate)),
            ))
            .id();

        chunk_meshes.insert(coordinate, entity, mesh_handle);
    }

    info!(
        "Streaming update | Camera chunk: {:?} | Loaded: {} | Added: {} | Removed: {}",
        camera_chunk,
        world.iter_chunks().count(),
        chunks_to_load.len(),
        chunks_to_unload.len(),
    );
}

fn desired_chunk_coordinates(center: IVec3, render_distance: i32) -> HashSet<IVec3> {
    let diameter = (render_distance * 2 + 1) as usize;

    let mut chunks = HashSet::with_capacity(diameter * diameter);

    for z in -render_distance..=render_distance {
        for x in -render_distance..=render_distance {
            chunks.insert(center + IVec3::new(x, 0, z));
        }
    }

    chunks
}

fn horizontal_neighbors(coordinate: IVec3) -> impl Iterator<Item = IVec3> {
    HORIZONTAL_NEIGHBORS
        .into_iter()
        .map(move |direction| coordinate + direction)
}
