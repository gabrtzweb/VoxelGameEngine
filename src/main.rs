use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Chão
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.25, 0.45, 0.25))),
    ));

    // Cubo
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.7, 0.7, 0.75))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Luz
    commands.spawn((
        PointLight { ..default() },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Câmera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-4.0, 4.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
