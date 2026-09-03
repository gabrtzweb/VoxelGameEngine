use std::collections::HashMap;

use bevy::prelude::*;

use super::{mesher::ChunkMesher, world::VoxelWorld};

struct ChunkRenderPart {
    entity: Entity,
    mesh_handle: Handle<Mesh>,

    vertex_count: usize,
    triangle_count: usize,
}

#[derive(Default)]
struct ChunkRenderData {
    opaque: Option<ChunkRenderPart>,

    transparent: Option<ChunkRenderPart>,
}

impl ChunkRenderData {
    fn is_empty(&self) -> bool {
        self.opaque.is_none() && self.transparent.is_none()
    }

    fn vertex_count(&self) -> usize {
        self.opaque.as_ref().map_or(0, |part| part.vertex_count)
            + self
                .transparent
                .as_ref()
                .map_or(0, |part| part.vertex_count)
    }

    fn triangle_count(&self) -> usize {
        self.opaque.as_ref().map_or(0, |part| part.triangle_count)
            + self
                .transparent
                .as_ref()
                .map_or(0, |part| part.triangle_count)
    }
}

#[derive(Resource, Default)]
pub struct ChunkMeshRegistry {
    entries: HashMap<IVec3, ChunkRenderData>,
}

impl ChunkMeshRegistry {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn total_vertices(&self) -> usize {
        self.entries
            .values()
            .map(ChunkRenderData::vertex_count)
            .sum()
    }

    pub fn total_triangles(&self) -> usize {
        self.entries
            .values()
            .map(ChunkRenderData::triangle_count)
            .sum()
    }

    pub fn iter_coordinates(&self) -> impl Iterator<Item = &IVec3> {
        self.entries.keys()
    }
}

#[derive(Resource)]
pub struct ChunkMaterial {
    opaque: Handle<StandardMaterial>,

    transparent: Handle<StandardMaterial>,
}

pub fn setup_chunk_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let opaque = materials.add(StandardMaterial {
        base_color: Color::WHITE,

        perceptual_roughness: 0.9,

        ..default()
    });

    let transparent = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.58),

        alpha_mode: AlphaMode::Blend,

        perceptual_roughness: 0.2,

        ..default()
    });

    commands.insert_resource(ChunkMaterial {
        opaque,
        transparent,
    });
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

    let rebuilt = ChunkMesher::build_meshes(world, coordinate);

    let translation = VoxelWorld::chunk_translation(coordinate);

    let is_empty = {
        let entry = registry.entries.entry(coordinate).or_default();

        sync_render_part(
            commands,
            meshes,
            &mut entry.opaque,
            rebuilt.opaque,
            &material.opaque,
            translation,
        );

        sync_render_part(
            commands,
            meshes,
            &mut entry.transparent,
            rebuilt.transparent,
            &material.transparent,
            translation,
        );

        entry.is_empty()
    };

    if is_empty {
        registry.entries.remove(&coordinate);
    }
}

fn sync_render_part(
    commands: &mut Commands,

    meshes: &mut Assets<Mesh>,

    part: &mut Option<ChunkRenderPart>,

    rebuilt_mesh: Option<Mesh>,

    material: &Handle<StandardMaterial>,

    translation: Vec3,
) {
    let Some(rebuilt_mesh) = rebuilt_mesh else {
        remove_render_part(commands, meshes, part);

        return;
    };

    let vertex_count = rebuilt_mesh.count_vertices();

    let triangle_count = rebuilt_mesh
        .indices()
        .map(|indices| indices.len() / 3)
        .unwrap_or(0);

    if let Some(existing) = part.as_mut()
        && let Some(mut mesh) = meshes.get_mut(&existing.mesh_handle)
    {
        *mesh = rebuilt_mesh;

        existing.vertex_count = vertex_count;

        existing.triangle_count = triangle_count;

        return;
    }

    remove_render_part(commands, meshes, part);

    let mesh_handle = meshes.add(rebuilt_mesh);

    let entity = commands
        .spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(translation),
        ))
        .id();

    *part = Some(ChunkRenderPart {
        entity,
        mesh_handle,
        vertex_count,
        triangle_count,
    });
}

fn remove_render_part(
    commands: &mut Commands,

    meshes: &mut Assets<Mesh>,

    part: &mut Option<ChunkRenderPart>,
) {
    let Some(render_part) = part.take() else {
        return;
    };

    commands.entity(render_part.entity).despawn();

    meshes.remove(render_part.mesh_handle.id());
}

pub fn remove_chunk_render(
    commands: &mut Commands,

    coordinate: IVec3,

    registry: &mut ChunkMeshRegistry,

    meshes: &mut Assets<Mesh>,
) {
    let Some(mut render_data) = registry.entries.remove(&coordinate) else {
        return;
    };

    remove_render_part(commands, meshes, &mut render_data.opaque);

    remove_render_part(commands, meshes, &mut render_data.transparent);
}
