mod dev_camera;
mod dev_stats;
mod voxel;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

use dev_camera::{DevCamera, DevCameraPlugin};

use dev_stats::DevStatsPlugin;

use voxel::{
    CHUNK_SIZE, CHUNK_VOLUME, CHUNK_WORLD_SIZE, ChunkMeshRegistry, ChunkMesher, TargetingPlugin,
    TerrainGenerator, VOXEL_SIZE, VoxelDebugPlugin, VoxelInteractionPlugin, VoxelWorld,
};

const INITIAL_WORLD_SIZE: i32 = 8;
const INITIAL_WORLD_HALF_SIZE: i32 = INITIAL_WORLD_SIZE / 2;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DevCameraPlugin)
        .add_plugins(DevStatsPlugin)
        .add_plugins(TargetingPlugin)
        .add_plugins(VoxelInteractionPlugin)
        .add_plugins(VoxelDebugPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut world = VoxelWorld::default();
    let mut chunk_meshes = ChunkMeshRegistry::default();

    let terrain_generator = TerrainGenerator::default();

    let chunk_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.52, 0.22),
        perceptual_roughness: 0.9,
        ..default()
    });

    let mut chunk_coordinates = Vec::new();

    // First pass: generate all chunk voxel data.
    for z in -INITIAL_WORLD_HALF_SIZE..INITIAL_WORLD_HALF_SIZE {
        for x in -INITIAL_WORLD_HALF_SIZE..INITIAL_WORLD_HALF_SIZE {
            let chunk_coordinate = IVec3::new(x, 0, z);

            let chunk = terrain_generator.generate_chunk(chunk_coordinate);

            world.insert_chunk(chunk_coordinate, chunk);

            chunk_coordinates.push(chunk_coordinate);
        }
    }

    // Second pass: build meshes after all neighboring chunks exist.
    for &chunk_coordinate in &chunk_coordinates {
        let chunk_mesh = ChunkMesher::build_mesh(&world, chunk_coordinate);

        let mesh_handle = meshes.add(chunk_mesh);

        commands.spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(chunk_material.clone()),
            Transform::from_translation(VoxelWorld::chunk_translation(chunk_coordinate)),
        ));

        chunk_meshes.insert(chunk_coordinate, mesh_handle);
    }

    let loaded_chunks = chunk_coordinates.len();

    commands.insert_resource(world);
    commands.insert_resource(chunk_meshes);

    info!("Voxel size: {} m", VOXEL_SIZE);
    info!("Chunk size: {}³", CHUNK_SIZE);
    info!("Chunk world size: {} m", CHUNK_WORLD_SIZE);
    info!("Chunk volume: {} voxels", CHUNK_VOLUME);
    info!("Loaded chunks: {}", loaded_chunks);
    info!("Loaded voxel capacity: {}", loaded_chunks * CHUNK_VOLUME);
    info!("Terrain seed: {}", terrain_generator.seed);

    // Main directional light.
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));

    // Development camera.
    let camera_transform =
        Transform::from_xyz(-10.0, 12.0, 14.0).looking_at(Vec3::new(4.0, 3.0, 4.0), Vec3::Y);

    let dev_camera = DevCamera::from_transform(&camera_transform);

    commands.spawn((Camera3d::default(), camera_transform, dev_camera));
}
