use std::collections::HashMap;

use bevy::prelude::*;

use super::{mesher::ChunkMesher, world::VoxelWorld};

#[derive(Clone)]
pub struct ChunkRenderData {
    pub entity: Entity,
    pub mesh_handle: Handle<Mesh>,

    vertex_count: usize,
    triangle_count: usize,
}

#[derive(Resource, Default)]
pub struct ChunkMeshRegistry {
    entries: HashMap<IVec3, ChunkRenderData>,
}

impl ChunkMeshRegistry {
    pub fn get(&self, coordinate: IVec3) -> Option<&Handle<Mesh>> {
        self.entries
            .get(&coordinate)
            .map(|entry| &entry.mesh_handle)
    }

    pub fn insert(
        &mut self,
        coordinate: IVec3,
        entity: Entity,
        mesh_handle: Handle<Mesh>,
        vertex_count: usize,
        triangle_count: usize,
    ) {
        self.entries.insert(
            coordinate,
            ChunkRenderData {
                entity,
                mesh_handle,
                vertex_count,
                triangle_count,
            },
        );
    }

    pub fn remove(&mut self, coordinate: IVec3) -> Option<ChunkRenderData> {
        self.entries.remove(&coordinate)
    }

    pub fn update_geometry_counts(
        &mut self,
        coordinate: IVec3,
        vertex_count: usize,
        triangle_count: usize,
    ) {
        let Some(entry) = self.entries.get_mut(&coordinate) else {
            return;
        };

        entry.vertex_count = vertex_count;

        entry.triangle_count = triangle_count;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn total_vertices(&self) -> usize {
        self.entries.values().map(|entry| entry.vertex_count).sum()
    }

    pub fn total_triangles(&self) -> usize {
        self.entries
            .values()
            .map(|entry| entry.triangle_count)
            .sum()
    }

    pub fn iter_coordinates(&self) -> impl Iterator<Item = &IVec3> {
        self.entries.keys()
    }
}

#[derive(Resource)]
pub struct ChunkMaterial(pub Handle<StandardMaterial>);

pub fn setup_chunk_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.52, 0.22),
        perceptual_roughness: 0.9,
        ..default()
    });

    commands.insert_resource(ChunkMaterial(material));
}

pub fn sync_chunk_render(
    commands: &mut Commands,
    world: &VoxelWorld,
    coordinate: IVec3,
    registry: &mut ChunkMeshRegistry,
    meshes: &mut Assets<Mesh>,
    material: &ChunkMaterial,
) {
    if world.get_chunk(coordinate).is_none() {
        remove_chunk_render(commands, coordinate, registry, meshes);

        return;
    }

    let Some(rebuilt_mesh) = ChunkMesher::build_mesh(world, coordinate) else {
        remove_chunk_render(commands, coordinate, registry, meshes);

        return;
    };

    let vertex_count = rebuilt_mesh.count_vertices();

    let triangle_count = rebuilt_mesh
        .indices()
        .map(|indices| indices.len() / 3)
        .unwrap_or(0);

    if let Some(mesh_handle) = registry.get(coordinate).cloned() {
        if let Some(mut mesh) = meshes.get_mut(&mesh_handle) {
            *mesh = rebuilt_mesh;

            registry.update_geometry_counts(coordinate, vertex_count, triangle_count);

            return;
        }

        remove_chunk_render(commands, coordinate, registry, meshes);
    }

    let mesh_handle = meshes.add(rebuilt_mesh);

    let entity = commands
        .spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(VoxelWorld::chunk_translation(coordinate)),
        ))
        .id();

    registry.insert(
        coordinate,
        entity,
        mesh_handle,
        vertex_count,
        triangle_count,
    );
}

pub fn remove_chunk_render(
    commands: &mut Commands,
    coordinate: IVec3,
    registry: &mut ChunkMeshRegistry,
    meshes: &mut Assets<Mesh>,
) {
    let Some(render_data) = registry.remove(coordinate) else {
        return;
    };

    commands.entity(render_data.entity).despawn();

    meshes.remove(render_data.mesh_handle.id());
}
