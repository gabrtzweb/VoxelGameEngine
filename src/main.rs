mod dev_camera;
mod voxel;

use bevy::prelude::*;

use dev_camera::{DevCamera, DevCameraPlugin};

use voxel::{
    CHUNK_SIZE, CHUNK_VOLUME, CHUNK_WORLD_SIZE, Chunk, ChunkMeshRegistry, ChunkMesher,
    TargetingPlugin, VOXEL_SIZE, VoxelDebugPlugin, VoxelInteractionPlugin, VoxelWorld,
};

const INITIAL_WORLD_RADIUS: i32 = 1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DevCameraPlugin)
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

    let chunk_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.52, 0.22),
        perceptual_roughness: 0.9,
        ..default()
    });

    let mut loaded_chunks = 0;

    for z in -INITIAL_WORLD_RADIUS..=INITIAL_WORLD_RADIUS {
        for x in -INITIAL_WORLD_RADIUS..=INITIAL_WORLD_RADIUS {
            let chunk_coordinate = IVec3::new(x, 0, z);

            let chunk = Chunk::new_half_solid();
            let chunk_mesh = ChunkMesher::build_mesh(&chunk);
            let mesh_handle = meshes.add(chunk_mesh);

            commands.spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(chunk_material.clone()),
                Transform::from_translation(VoxelWorld::chunk_translation(chunk_coordinate)),
            ));

            world.insert_chunk(chunk_coordinate, chunk);

            chunk_meshes.insert(chunk_coordinate, mesh_handle);

            loaded_chunks += 1;
        }
    }

    commands.insert_resource(world);
    commands.insert_resource(chunk_meshes);

    info!("Voxel size: {} m", VOXEL_SIZE);
    info!("Chunk size: {}³", CHUNK_SIZE);
    info!("Chunk world size: {} m", CHUNK_WORLD_SIZE);
    info!("Chunk volume: {} voxels", CHUNK_VOLUME);
    info!("Loaded chunks: {}", loaded_chunks);
    info!("Loaded voxel capacity: {}", loaded_chunks * CHUNK_VOLUME);

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));

    let camera_transform =
        Transform::from_xyz(-10.0, 8.0, 14.0).looking_at(Vec3::new(4.0, 2.0, 4.0), Vec3::Y);

    let dev_camera = DevCamera::from_transform(&camera_transform);

    commands.spawn((Camera3d::default(), camera_transform, dev_camera));
}
