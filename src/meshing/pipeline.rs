use bevy::platform::collections::HashMap;

use bevy::{
    camera::primitives::{Aabb, MeshAabb},
    light::NotShadowCaster,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::*,
    shader::ShaderRef,
};

use super::{
    culling::{CaveCullingState, ChunkCoordinate, SubterraneanChunkMesh},
    greedy::{ChunkMesher, ChunkMeshes},
    textures::{VoxelTextureRegistry, build_voxel_texture_array, build_mask_texture_array},
};
use crate::environment::foliage::GrassChunkMaterial;
use crate::world::{Chunk, VoxelWorld};

pub const OPAQUE_VOXEL_SHADER_PATH: &str = "shaders/voxel_opaque.wesl";
pub const TRANSPARENT_VOXEL_SHADER_PATH: &str = "shaders/voxel_transparent.wesl";
pub const MAX_SHADOW_CASTER_CHUNKS: i32 = 6;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct OpaqueVoxelMaterial {
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    pub texture_array: Handle<Image>,
    #[texture(102)]
    #[sampler(103)]
    pub noise_texture: Handle<Image>,
    #[uniform(104)]
    pub params: Vec4,
    #[uniform(105)]
    pub anim_params: Vec4,
    #[texture(106, dimension = "2d_array")]
    #[sampler(107)]
    pub mask_texture_array: Handle<Image>,
}

impl MaterialExtension for OpaqueVoxelMaterial {
    fn vertex_shader() -> ShaderRef {
        OPAQUE_VOXEL_SHADER_PATH.into()
    }

    fn deferred_vertex_shader() -> ShaderRef {
        OPAQUE_VOXEL_SHADER_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        OPAQUE_VOXEL_SHADER_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        OPAQUE_VOXEL_SHADER_PATH.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "shaders/voxel_prepass.wesl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "shaders/voxel_prepass.wesl".into()
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
    pub grass: Option<ChunkRenderPart>,
    pub visibility_mask: u64,
    pub is_subterranean: bool,
}

impl ChunkRenderData {
    pub fn is_empty(&self) -> bool {
        self.opaque.is_none() && self.transparent.is_none() && self.grass.is_none()
    }

    pub fn vertex_count(&self) -> usize {
        self.opaque.as_ref().map_or(0, |part| part.vertex_count)
            + self
                .transparent
                .as_ref()
                .map_or(0, |part| part.vertex_count)
            + self.grass.as_ref().map_or(0, |part| part.vertex_count)
    }

    pub fn triangle_count(&self) -> usize {
        self.opaque.as_ref().map_or(0, |part| part.triangle_count)
            + self
                .transparent
                .as_ref()
                .map_or(0, |part| part.triangle_count)
            + self.grass.as_ref().map_or(0, |part| part.triangle_count)
    }
}

#[derive(Resource, Default)]
pub struct ChunkMeshRegistry {
    entries: HashMap<IVec3, ChunkRenderData>,
    columns: HashMap<IVec2, usize>,
    surface_columns: HashMap<IVec2, usize>,
    total_vertices: usize,
    total_triangles: usize,
}

impl ChunkMeshRegistry {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn total_vertices(&self) -> usize {
        self.total_vertices
    }

    #[inline]
    pub fn total_triangles(&self) -> usize {
        self.total_triangles
    }

    pub fn contains(&self, coordinate: &IVec3) -> bool {
        self.entries.contains_key(coordinate)
    }

    pub fn get_visibility_mask_opt(&self, coordinate: &IVec3) -> Option<u64> {
        self.entries.get(coordinate).map(|d| d.visibility_mask)
    }

    pub fn get_render_data(&self, coordinate: &IVec3) -> Option<&ChunkRenderData> {
        self.entries.get(coordinate)
    }

    pub fn is_subterranean(&self, coordinate: &IVec3) -> bool {
        self.entries.get(coordinate).is_some_and(|d| d.is_subterranean)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn has_column_mesh(&self, column: IVec2) -> bool {
        self.columns.get(&column).is_some_and(|&count| count > 0)
    }

    #[inline]
    pub fn has_column_surface_mesh(&self, column: IVec2) -> bool {
        self.surface_columns
            .get(&column)
            .is_some_and(|&count| count > 0)
    }

    pub fn iter_coordinates(&self) -> impl Iterator<Item = &IVec3> {
        self.entries.keys()
    }

    pub fn clear(&mut self, commands: &mut Commands) {
        for (_, data) in self.entries.drain() {
            if let Some(part) = data.opaque {
                commands.entity(part.entity).despawn();
            }
            if let Some(part) = data.transparent {
                commands.entity(part.entity).despawn();
            }
            if let Some(part) = data.grass {
                commands.entity(part.entity).despawn();
            }
        }
        self.columns.clear();
        self.surface_columns.clear();
        self.total_vertices = 0;
        self.total_triangles = 0;
    }
}

/// Registry of distant LOD surface meshes for chunk columns (cx, cz).
#[derive(Resource, Default)]
pub struct LodMeshRegistry {
    entries: HashMap<IVec2, ChunkRenderData>,
    lods: HashMap<IVec2, super::lod::ChunkLod>,
    total_vertices: usize,
    total_triangles: usize,
}

impl LodMeshRegistry {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn total_vertices(&self) -> usize {
        self.total_vertices
    }

    #[inline]
    pub fn total_triangles(&self) -> usize {
        self.total_triangles
    }

    pub fn contains(&self, coordinate: &IVec2) -> bool {
        self.entries.contains_key(coordinate)
    }

    pub fn get_lod(&self, coordinate: &IVec2) -> Option<super::lod::ChunkLod> {
        self.lods.get(coordinate).copied()
    }

    pub fn clear(&mut self, commands: &mut Commands) {
        for (_, data) in self.entries.drain() {
            if let Some(part) = data.opaque {
                commands.entity(part.entity).despawn();
            }
            if let Some(part) = data.transparent {
                commands.entity(part.entity).despawn();
            }
        }
        self.lods.clear();
        self.total_vertices = 0;
        self.total_triangles = 0;
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
    pub grass: Option<Handle<GrassChunkMaterial>>,
    pub texture_registry: VoxelTextureRegistry,
}

pub fn setup_chunk_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut opaque_materials: ResMut<Assets<OpaqueChunkMaterial>>,
    mut transparent_materials: ResMut<Assets<TransparentChunkMaterial>>,
    grass_material_handle: Option<Res<crate::environment::foliage::GrassMaterialHandle>>,
    local_config: Option<Res<crate::core::config::LocalConfig>>,
) {
    let (texture_array_image, mut texture_registry) = build_voxel_texture_array();
    if let Some(cfg) = &local_config {
        texture_registry.full_grass = cfg.full_grass;
        texture_registry.leaf_culling = cfg.leaf_culling;
        texture_registry.grass_enabled = cfg.grass_enabled;
        texture_registry.grass_density = cfg.grass_density;
    }
    let texture_array = images.add(texture_array_image);
    let mask_texture_array_image = build_mask_texture_array();
    let mask_texture_array = images.add(mask_texture_array_image);
    let noise_texture = asset_server.load("textures/shaders/noise.png");

    let coat_strength = local_config
        .as_ref()
        .map_or(0.28, |cfg| cfg.coated_textures.strength());
    let waving_foliage_flag = local_config
        .as_ref()
        .map_or(1.0, |cfg| if cfg.waving_foliage { 1.0 } else { 0.0 });
    let pixelated_shadows_flag = local_config
        .as_ref()
        .map_or(1.0, |cfg| if cfg.pixelated_shadows { 1.0 } else { 0.0 });

    let params = Vec4::new(coat_strength, waving_foliage_flag, pixelated_shadows_flag, 0.0);
    let anim_params = Vec4::new(0.0, 1.0, 0.44, 0.0);

    let opaque = opaque_materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Mask(0.5),
            perceptual_roughness: 0.9,
            ..default()
        },
        extension: OpaqueVoxelMaterial {
            texture_array: texture_array.clone(),
            noise_texture,
            params,
            anim_params,
            mask_texture_array,
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

    let grass = grass_material_handle.as_ref().map(|h| h.material.clone());

    commands.insert_resource(texture_registry.clone());
    commands.insert_resource(ChunkMaterial {
        opaque,
        transparent,
        grass,
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
    culling_state: Option<&CaveCullingState>,
    registry: &mut ChunkMeshRegistry,
    meshes: &mut Assets<Mesh>,
    material: &ChunkMaterial,
) {
    let translation = VoxelWorld::chunk_translation(coordinate);
    let column = IVec2::new(coordinate.x, coordinate.z);
    let was_empty;

    let initial_visibility = if is_subterranean {
        if culling_state.is_none()
            || culling_state.is_some_and(|cs| cs.visible_chunks.contains(&coordinate))
        {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    } else {
        Visibility::Inherited
    };

    let old_verts;
    let old_tris;
    let new_verts;
    let new_tris;

    let is_empty = {
        let entry = registry.entries.entry(coordinate).or_default();
        was_empty = entry.is_empty();
        old_verts = entry.vertex_count();
        old_tris = entry.triangle_count();
        entry.is_subterranean = is_subterranean;
        entry.visibility_mask = rebuilt.visibility_mask;

        let chunk_aabb =
            Aabb::from_min_max(Vec3::ZERO, Vec3::splat(crate::world::CHUNK_SIZE as f32));
        let no_shadows = is_subterranean
            || culling_state.is_some_and(|cs| {
                cs.last_player_chunk.x != i32::MIN
                    && ((coordinate.x - cs.last_player_chunk.x).abs() > MAX_SHADOW_CASTER_CHUNKS
                        || (coordinate.z - cs.last_player_chunk.z).abs() > MAX_SHADOW_CASTER_CHUNKS)
            });

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
            no_shadows,
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
            true,
        );

        if let Some(grass_material) = &material.grass {
            let is_near_grass = culling_state.is_none_or(|cs| {
                cs.last_player_chunk.x == i32::MIN
                    || ((coordinate.x - cs.last_player_chunk.x).abs() <= 5
                        && (coordinate.z - cs.last_player_chunk.z).abs() <= 5)
            });
            let grass_visibility = if initial_visibility == Visibility::Inherited && is_near_grass {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            sync_render_part(
                commands,
                meshes,
                &mut entry.grass,
                rebuilt.grass,
                grass_material,
                translation,
                chunk_aabb,
                Some(coordinate),
                is_subterranean,
                grass_visibility,
                true,
            );
        } else {
            remove_render_part(commands, meshes, &mut entry.grass);
        }

        new_verts = entry.vertex_count();
        new_tris = entry.triangle_count();
        entry.is_empty()
    };

    registry.total_vertices = registry
        .total_vertices
        .saturating_sub(old_verts)
        + new_verts;
    registry.total_triangles = registry
        .total_triangles
        .saturating_sub(old_tris)
        + new_tris;

    if is_empty {
        if !was_empty {
            registry.entries.remove(&coordinate);
            if let Some(count) = registry.columns.get_mut(&column) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    registry.columns.remove(&column);
                }
            }
            if !is_subterranean && let Some(count) = registry.surface_columns.get_mut(&column) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    registry.surface_columns.remove(&column);
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
    no_shadows: bool,
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

    let tight_aabb = rebuilt_mesh.get_aabb().unwrap_or(aabb);

    if let Some(existing) = part.as_mut() {
        let new_handle = meshes.add(rebuilt_mesh);
        let old_id = existing.mesh_handle.id();
        existing.mesh_handle = new_handle.clone();
        existing.vertex_count = vertex_count;
        existing.triangle_count = triangle_count;
        if let Ok(mut entity_cmds) = commands.get_entity(existing.entity) {
            entity_cmds.insert((Mesh3d(new_handle), tight_aabb));
            if no_shadows {
                entity_cmds.insert(NotShadowCaster);
            } else {
                entity_cmds.remove::<NotShadowCaster>();
            }
            if !is_subterranean || initial_visibility == Visibility::Inherited {
                entity_cmds.insert(Visibility::Inherited);
            }
        }
        meshes.remove(old_id);
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

    if no_shadows {
        entity_cmds.insert(NotShadowCaster);
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

    registry.total_vertices = registry
        .total_vertices
        .saturating_sub(render_data.vertex_count());
    registry.total_triangles = registry
        .total_triangles
        .saturating_sub(render_data.triangle_count());

    let column = IVec2::new(coordinate.x, coordinate.z);
    if !render_data.is_empty() {
        if let Some(count) = registry.columns.get_mut(&column) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                registry.columns.remove(&column);
            }
        }
        if !render_data.is_subterranean
            && let Some(count) = registry.surface_columns.get_mut(&column)
        {
            *count = count.saturating_sub(1);
            if *count == 0 {
                registry.surface_columns.remove(&column);
            }
        }
    }

    remove_render_part(commands, meshes, &mut render_data.opaque);
    remove_render_part(commands, meshes, &mut render_data.transparent);
    remove_render_part(commands, meshes, &mut render_data.grass);
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

    let old_verts;
    let old_tris;
    let new_verts;
    let new_tris;

    let is_empty = {
        let entry = registry.entries.entry(coordinate).or_default();
        old_verts = entry.vertex_count();
        old_tris = entry.triangle_count();

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
            true,
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
            true,
        );

        new_verts = entry.vertex_count();
        new_tris = entry.triangle_count();
        entry.is_empty()
    };

    registry.total_vertices = registry
        .total_vertices
        .saturating_sub(old_verts)
        + new_verts;
    registry.total_triangles = registry
        .total_triangles
        .saturating_sub(old_tris)
        + new_tris;

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

    registry.total_vertices = registry
        .total_vertices
        .saturating_sub(render_data.vertex_count());
    registry.total_triangles = registry
        .total_triangles
        .saturating_sub(render_data.triangle_count());

    remove_render_part(commands, meshes, &mut render_data.opaque);
    remove_render_part(commands, meshes, &mut render_data.transparent);
}

pub fn sync_voxel_shader_settings(
    local_config: Option<Res<crate::core::LocalConfig>>,
    wind_state: Option<Res<crate::environment::WindState>>,
    chunk_material: Option<Res<ChunkMaterial>>,
    mut opaque_materials: ResMut<Assets<OpaqueChunkMaterial>>,
    mut last_sent: Local<Option<(Vec4, f32, f32)>>,
) {
    let Some(chunk_material) = chunk_material else { return };

    let target_params = if let Some(ref cfg) = local_config {
        Vec4::new(
            cfg.coated_textures.strength(),
            if cfg.waving_foliage { 1.0 } else { 0.0 },
            if cfg.pixelated_shadows { 1.0 } else { 0.0 },
            0.0,
        )
    } else {
        Vec4::new(0.28, 1.0, 1.0, 0.0)
    };

    let (wind_intensity, wind_angle) = if let Some(ref wind) = wind_state {
        ((wind.current_speed / 4.0).clamp(0.4, 2.5), wind.current_heading)
    } else {
        (1.0, 0.44)
    };

    let needs_update = match *last_sent {
        None => true,
        Some((p, wi, wa)) => {
            (p - target_params).abs().max_element() > 0.005
                || (wi - wind_intensity).abs() > 0.005
                || (wa - wind_angle).abs() > 0.005
        }
    };

    if !needs_update {
        return;
    }

    if let Some(mut mat) = opaque_materials.get_mut(&chunk_material.opaque) {
        mat.extension.params = target_params;
        mat.extension.anim_params = Vec4::new(0.0, wind_intensity, wind_angle, 0.0);
        *last_sent = Some((target_params, wind_intensity, wind_angle));
    }
}
