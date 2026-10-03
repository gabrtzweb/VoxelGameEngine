use bevy::platform::collections::HashMap;

use bevy::{
    camera::primitives::{Aabb, MeshAabb},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::*,
    shader::ShaderRef,
};

use super::{
    culling::{CaveCullingState, ChunkCoordinate, SubterraneanChunkMesh},
    greedy::{ChunkMesher, ChunkMeshes},
    textures::{VoxelTextureRegistry, build_voxel_texture_array},
};
use crate::world::{Chunk, VoxelWorld};

pub const OPAQUE_VOXEL_SHADER_PATH: &str = "shaders/voxel_opaque.wgsl";
pub const TRANSPARENT_VOXEL_SHADER_PATH: &str = "shaders/voxel_transparent.wgsl";

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct OpaqueVoxelMaterial {
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    pub texture_array: Handle<Image>,
}

impl MaterialExtension for OpaqueVoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        OPAQUE_VOXEL_SHADER_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        OPAQUE_VOXEL_SHADER_PATH.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "shaders/voxel_prepass.wgsl".into()
    }
}

pub type OpaqueChunkMaterial = ExtendedMaterial<StandardMaterial, OpaqueVoxelMaterial>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct TransparentVoxelMaterial {
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    pub texture_array: Handle<Image>,
}

impl MaterialExtension for TransparentVoxelMaterial {
    fn vertex_shader() -> ShaderRef {
        TRANSPARENT_VOXEL_SHADER_PATH.into()
    }

    fn deferred_vertex_shader() -> ShaderRef {
        TRANSPARENT_VOXEL_SHADER_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        TRANSPARENT_VOXEL_SHADER_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        TRANSPARENT_VOXEL_SHADER_PATH.into()
    }

    // Transparent water reads the opaque depth prepass and should not write into prepass or shadow maps
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }
}

pub type TransparentChunkMaterial = ExtendedMaterial<StandardMaterial, TransparentVoxelMaterial>;

pub struct ChunkRenderPart {
    pub entity: Entity,
    pub mesh_handle: Handle<Mesh>,
    pub vertex_count: usize,
    pub triangle_count: usize,
}

#[derive(Default)]
pub struct ChunkRenderData {
    pub opaque: Option<ChunkRenderPart>,
    pub transparent: Option<ChunkRenderPart>,
    pub visibility_mask: u64,
    pub is_subterranean: bool,
}

impl ChunkRenderData {
    pub fn is_empty(&self) -> bool {
        self.opaque.is_none() && self.transparent.is_none()
    }

    pub fn vertex_count(&self) -> usize {
        self.opaque.as_ref().map_or(0, |part| part.vertex_count)
            + self
                .transparent
                .as_ref()
                .map_or(0, |part| part.vertex_count)
    }

    pub fn triangle_count(&self) -> usize {
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
    columns: HashMap<IVec2, usize>,
    surface_columns: HashMap<IVec2, usize>,
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

    pub fn contains(&self, coordinate: &IVec3) -> bool {
        self.entries.contains_key(coordinate)
    }

    pub fn get_visibility_mask_opt(&self, coordinate: &IVec3) -> Option<u64> {
        self.entries.get(coordinate).map(|d| d.visibility_mask)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn has_column_mesh(&self, column: IVec2) -> bool {
        self.columns.get(&column).is_some_and(|&count| count > 0)
    }

    #[inline]
    pub fn has_column_surface_mesh(&self, column: IVec2) -> bool {
        self.surface_columns.get(&column).is_some_and(|&count| count > 0)
    }

    pub fn iter_coordinates(&self) -> impl Iterator<Item = &IVec3> {
        self.entries.keys()
    }
}

/// Registry of distant LOD surface meshes for chunk columns (cx, cz).
#[derive(Resource, Default)]
pub struct LodMeshRegistry {
    entries: HashMap<IVec2, ChunkRenderData>,
    lods: HashMap<IVec2, super::lod::ChunkLod>,
}

impl LodMeshRegistry {
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

    pub fn contains(&self, coordinate: &IVec2) -> bool {
        self.entries.contains_key(coordinate)
    }

    pub fn get_lod(&self, coordinate: &IVec2) -> Option<super::lod::ChunkLod> {
        self.lods.get(coordinate).copied()
    }

    pub fn lod_count(&self, lod: super::lod::ChunkLod) -> usize {
        self.lods.values().filter(|&&l| l == lod).count()
    }

    pub fn iter_coordinates(&self) -> impl Iterator<Item = &IVec2> {
        self.entries.keys()
    }
}

#[derive(Resource)]
pub struct ChunkMaterial {
    pub opaque: Handle<OpaqueChunkMaterial>,
    pub transparent: Handle<TransparentChunkMaterial>,
    pub texture_registry: VoxelTextureRegistry,
}

pub fn setup_chunk_material(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut opaque_materials: ResMut<Assets<OpaqueChunkMaterial>>,
    mut transparent_materials: ResMut<Assets<TransparentChunkMaterial>>,
) {
    let (texture_array_image, texture_registry) = build_voxel_texture_array();
    let texture_array = images.add(texture_array_image);

    let opaque = opaque_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Mask(0.5),
            perceptual_roughness: 0.9,
            ..default()
        },
        extension: OpaqueVoxelMaterial {
            texture_array: texture_array.clone(),
        },
    });

    let transparent = transparent_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.85),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            double_sided: false,
            perceptual_roughness: 0.04,
            reflectance: 0.9,
            ..default()
        },
        extension: TransparentVoxelMaterial { texture_array },
    });

    commands.insert_resource(texture_registry.clone());
    commands.insert_resource(ChunkMaterial {
        opaque,
        transparent,
        texture_registry,
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

    let is_subterranean = world
        .get_chunk(coordinate)
        .is_some_and(Chunk::is_subterranean);
    let rebuilt = ChunkMesher::build_meshes(world, coordinate, &material.texture_registry);
    apply_chunk_mesh(
        commands,
        coordinate,
        rebuilt,
        is_subterranean,
        None,
        registry,
        meshes,
        material,
    );
}

pub fn apply_chunk_mesh(
    commands: &mut Commands,
    coordinate: IVec3,
    rebuilt: ChunkMeshes,
    is_subterranean: bool,
    _culling_state: Option<&CaveCullingState>,
    registry: &mut ChunkMeshRegistry,
    meshes: &mut Assets<Mesh>,
    material: &ChunkMaterial,
) {
    let translation = VoxelWorld::chunk_translation(coordinate);
    let column = IVec2::new(coordinate.x, coordinate.z);
    let was_empty;

    let initial_visibility = if is_subterranean {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };

    let is_empty = {
        let entry = registry.entries.entry(coordinate).or_default();
        was_empty = entry.is_empty();
        entry.is_subterranean = is_subterranean;
        entry.visibility_mask = rebuilt.visibility_mask;

        let chunk_aabb =
            Aabb::from_min_max(Vec3::ZERO, Vec3::splat(crate::world::CHUNK_SIZE as f32));

        sync_render_part(
            commands,
            meshes,
            &mut entry.opaque,
            rebuilt.opaque,
            &material.opaque,
            translation,
            chunk_aabb,
            Some(coordinate),
            is_subterranean,
            initial_visibility,
        );

        sync_render_part(
            commands,
            meshes,
            &mut entry.transparent,
            rebuilt.transparent,
            &material.transparent,
            translation,
            chunk_aabb,
            Some(coordinate),
            is_subterranean,
            initial_visibility,
        );

        entry.is_empty()
    };

    if is_empty {
        if !was_empty {
            registry.entries.remove(&coordinate);
            if let Some(count) = registry.columns.get_mut(&column) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    registry.columns.remove(&column);
                }
            }
            if !is_subterranean {
                if let Some(count) = registry.surface_columns.get_mut(&column) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        registry.surface_columns.remove(&column);
                    }
                }
            }
        } else {
            registry.entries.remove(&coordinate);
        }
    } else if was_empty {
        *registry.columns.entry(column).or_default() += 1;
        if !is_subterranean {
            *registry.surface_columns.entry(column).or_default() += 1;
        }
    }
}

fn sync_render_part<M: Material>(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    part: &mut Option<ChunkRenderPart>,
    rebuilt_mesh: Option<Mesh>,
    material: &Handle<M>,
    translation: Vec3,
    aabb: Aabb,
    coordinate: Option<IVec3>,
    is_subterranean: bool,
    initial_visibility: Visibility,
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

    let tight_aabb = rebuilt_mesh.compute_aabb().unwrap_or(aabb);

    if let Some(existing) = part.as_mut()
        && let Some(mut mesh) = meshes.get_mut(&existing.mesh_handle)
    {
        *mesh = rebuilt_mesh;
        existing.vertex_count = vertex_count;
        existing.triangle_count = triangle_count;
        if let Ok(mut entity_cmds) = commands.get_entity(existing.entity) {
            entity_cmds.insert(tight_aabb);
        }
        return;
    }

    remove_render_part(commands, meshes, part);

    let mesh_handle = meshes.add(rebuilt_mesh);

    let mut entity_cmds = commands.spawn((
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(translation),
        tight_aabb,
        initial_visibility,
    ));

    if let Some(coord) = coordinate {
        entity_cmds.insert(ChunkCoordinate(coord));
    }

    if is_subterranean {
        entity_cmds.insert(SubterraneanChunkMesh);
    }

    let entity = entity_cmds.id();

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

    if let Ok(mut entity_cmds) = commands.get_entity(render_part.entity) {
        entity_cmds.despawn();
    }
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

    let column = IVec2::new(coordinate.x, coordinate.z);
    if !render_data.is_empty() {
        if let Some(count) = registry.columns.get_mut(&column) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                registry.columns.remove(&column);
            }
        }
        if !render_data.is_subterranean {
            if let Some(count) = registry.surface_columns.get_mut(&column) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    registry.surface_columns.remove(&column);
                }
            }
        }
    }

    remove_render_part(commands, meshes, &mut render_data.opaque);
    remove_render_part(commands, meshes, &mut render_data.transparent);
}

pub fn apply_lod_mesh(
    commands: &mut Commands,
    coordinate: IVec2,
    lod: super::lod::ChunkLod,
    rebuilt: ChunkMeshes,
    registry: &mut LodMeshRegistry,
    meshes: &mut Assets<Mesh>,
    material: &ChunkMaterial,
) {
    let translation = Vec3::new(
        coordinate.x as f32 * crate::world::CHUNK_SIZE as f32,
        0.0,
        coordinate.y as f32 * crate::world::CHUNK_SIZE as f32,
    );

    let is_empty = {
        let entry = registry.entries.entry(coordinate).or_default();

        let region_size = lod.world_size();
        let lod_aabb = Aabb::from_min_max(
            Vec3::new(0.0, -256.0, 0.0),
            Vec3::new(region_size, 256.0, region_size),
        );

        sync_render_part(
            commands,
            meshes,
            &mut entry.opaque,
            rebuilt.opaque,
            &material.opaque,
            translation,
            lod_aabb,
            None,
            false,
            Visibility::Inherited,
        );

        sync_render_part(
            commands,
            meshes,
            &mut entry.transparent,
            rebuilt.transparent,
            &material.transparent,
            translation,
            lod_aabb,
            None,
            false,
            Visibility::Inherited,
        );

        entry.is_empty()
    };

    if is_empty {
        registry.entries.remove(&coordinate);
        registry.lods.remove(&coordinate);
    } else {
        registry.lods.insert(coordinate, lod);
    }
}

pub fn remove_lod_render(
    commands: &mut Commands,
    coordinate: IVec2,
    registry: &mut LodMeshRegistry,
    meshes: &mut Assets<Mesh>,
) {
    registry.lods.remove(&coordinate);
    let Some(mut render_data) = registry.entries.remove(&coordinate) else {
        return;
    };

    remove_render_part(commands, meshes, &mut render_data.opaque);
    remove_render_part(commands, meshes, &mut render_data.transparent);
}
