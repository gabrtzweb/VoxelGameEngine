mod dev_camera;
mod voxel;

use bevy::prelude::*;

use dev_camera::{DevCamera, DevCameraPlugin};
use voxel::{Chunk, ChunkMesher, CHUNK_SIZE, CHUNK_VOLUME, VOXEL_SIZE};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DevCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunk = Chunk::new_half_solid();

    let exposed_faces = ChunkMesher::exposed_face_count(&chunk);

    info!("Voxel size: {} m", VOXEL_SIZE);
    info!("Chunk size: {}³", CHUNK_SIZE);
    info!("Chunk volume: {} voxels", CHUNK_VOLUME);
    info!("Exposed faces: {}", exposed_faces);
    info!("Generated vertices: {}", exposed_faces * 4);
    info!("Generated triangles: {}", exposed_faces * 2);

    // Generate the voxel chunk mesh.
    let chunk_mesh = ChunkMesher::build_mesh(&chunk);

    commands.spawn((
        Mesh3d(meshes.add(chunk_mesh)),
        MeshMaterial3d(
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.28, 0.52, 0.22),
                perceptual_roughness: 0.9,
                ..default()
            }),
        ),
    ));

    // Main scene light.
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(
            Quat::from_euler(
                EulerRot::XYZ,
                -0.8,
                -0.5,
                0.0,
            ),
        ),
    ));

    // Development camera.
    let camera_transform = Transform::from_xyz(-6.0, 6.0, 10.0)
        .looking_at(Vec3::new(4.0, 2.0, 4.0), Vec3::Y);

    let dev_camera = DevCamera::from_transform(&camera_transform);

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        dev_camera,
    ));
}