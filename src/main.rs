mod voxel;

use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};

use voxel::{Chunk, ChunkMesher, CHUNK_SIZE, CHUNK_VOLUME, VOXEL_SIZE};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunk = Chunk::new_half_solid();

    info!("Voxel size: {} m", VOXEL_SIZE);
    info!("Chunk size: {}³", CHUNK_SIZE);
    info!("Chunk volume: {} voxels", CHUNK_VOLUME);
    info!(
        "Exposed faces: {}",
        ChunkMesher::exposed_face_count(&chunk)
    );

    // Temporary ground plane
    commands.spawn((
        Mesh3d(meshes.add(
            Plane3d::default()
                .mesh()
                .size(20.0, 20.0),
        )),
        MeshMaterial3d(
            materials.add(Color::srgb(0.25, 0.45, 0.25))
        ),
    ));

    // Temporary reference cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(
            materials.add(Color::srgb(0.7, 0.7, 0.75))
        ),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Free-fly development camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-4.0, 4.0, 6.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera {
            walk_speed: 5.0,
            run_speed: 15.0,
            sensitivity: 0.15,
            ..default()
        },
    ));
}