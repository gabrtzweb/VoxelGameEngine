use bevy::{
    asset::RenderAssetUsages,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use super::{
    buffer::{
        MAX_WEATHER_PARTICLES, ParticleRotationMode, WeatherParticleKind, WeatherParticlePool,
    },
    emitters::{UV_RIPPLE_0, UV_RIPPLE_1, UV_RIPPLE_2, UV_RIPPLE_3},
};
use crate::player::PlayerCamera;

#[derive(Component)]
pub struct WeatherBatchMesh;

#[derive(Resource)]
pub struct WeatherMeshHandle(pub Handle<Mesh>);

/// Initializes the shared weather dynamic mesh and translucent atlas material.
pub fn setup_weather_renderer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Initialize with a dummy degenerate triangle so Bevy's GPU mesh allocator never encounters 0 vertices
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 0.0]; 3]);
    mesh.insert_indices(Indices::U32(vec![0, 0, 0]));

    let mesh_handle = meshes.add(mesh);
    commands.insert_resource(WeatherMeshHandle(mesh_handle.clone()));

    let texture_handle = asset_server.load("textures/environments/weather_atlas.png");

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    commands.spawn((
        WeatherBatchMesh,
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        NotShadowCaster,
        NotShadowReceiver,
        Transform::default(),
        Visibility::Hidden,
    ));
}

/// Dynamically updates the GPU vertex buffer with all simulated active weather quads.
pub fn sync_weather_particle_mesh(
    pool: Res<WeatherParticlePool>,
    mesh_handle: Option<Res<WeatherMeshHandle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    voxel_world: Res<crate::world::storage::VoxelWorld>,
    mut batch_query: Query<&mut Visibility, With<WeatherBatchMesh>>,
) {
    let Some(ref mh) = mesh_handle else {
        return;
    };
    let Some(mut mesh) = meshes.get_mut(&mh.0) else {
        return;
    };

    let Ok(mut batch_vis) = batch_query.single_mut() else {
        return;
    };

    let Ok(cam_transform) = camera_query.single() else {
        return;
    };

    let cam_block = IVec3::new(
        cam_transform.translation.x.floor() as i32,
        cam_transform.translation.y.floor() as i32,
        cam_transform.translation.z.floor() as i32,
    );
    let is_cam_underwater = voxel_world
        .get_voxel(cam_block)
        .is_some_and(|v| v.is_water());

    // When submerged under water or pool is inactive, hide the mesh to eliminate GPU overhead
    if is_cam_underwater || pool.active_count == 0 {
        if *batch_vis != Visibility::Hidden {
            *batch_vis = Visibility::Hidden;
        }
        return;
    }

    let cam_forward = *cam_transform.forward();
    let cam_right = *cam_transform.right();
    let cam_up = *cam_transform.up();

    let estimated_active = pool.active_count.min(MAX_WEATHER_PARTICLES);
    let mut positions = Vec::with_capacity(estimated_active * 4);
    let mut normals = Vec::with_capacity(estimated_active * 4);
    let mut uvs = Vec::with_capacity(estimated_active * 4);
    let mut colors = Vec::with_capacity(estimated_active * 4);
    let mut indices = Vec::with_capacity(estimated_active * 6);

    let mut vertex_cursor = 0u32;

    for p in pool.particles.iter() {
        if !p.alive {
            continue;
        }

        let alpha = p.current_alpha();
        if alpha <= 0.005 {
            continue;
        }

        let color = [p.color[0], p.color[1], p.color[2], alpha];

        // Animated sprite frame selection for ripples
        let uv_rect = if p.kind == WeatherParticleKind::WaterRipple {
            let frame = ((p.lifetime / p.max_lifetime) * 4.0).floor() as usize;
            match frame {
                0 => UV_RIPPLE_0,
                1 => UV_RIPPLE_1,
                2 => UV_RIPPLE_2,
                _ => UV_RIPPLE_3,
            }
        } else {
            p.uv_min_max
        };

        let half_w = p.size.x * 0.5;
        let half_h = p.size.y * 0.5;

        let (corner_0, corner_1, corner_2, corner_3) = match p.rotation_mode {
            ParticleRotationMode::RelativeVelocity => {
                let v = p.velocity;
                let v_len = v.length();
                let v_dir = if v_len > 0.001 { v / v_len } else { -Vec3::Y };

                // Extrude perpendicular to both velocity and view vector
                let mut side = v_dir.cross(cam_forward);
                if side.length_squared() < 0.001 {
                    side = v_dir.cross(cam_up);
                }
                let side_dir = side.normalize();

                let h_ext = v_dir * half_h;
                let w_ext = side_dir * half_w;

                (
                    p.position - w_ext - h_ext,
                    p.position + w_ext - h_ext,
                    p.position + w_ext + h_ext,
                    p.position - w_ext + h_ext,
                )
            }
            ParticleRotationMode::CopyCamera => {
                let r_ext = cam_right * half_w;
                let u_ext = cam_up * half_h;

                (
                    p.position - r_ext - u_ext,
                    p.position + r_ext - u_ext,
                    p.position + r_ext + u_ext,
                    p.position - r_ext + u_ext,
                )
            }
            ParticleRotationMode::Horizontal => {
                (
                    p.position + Vec3::new(-half_w, 0.0, -half_h),
                    p.position + Vec3::new(half_w, 0.0, -half_h),
                    p.position + Vec3::new(half_w, 0.0, half_h),
                    p.position + Vec3::new(-half_w, 0.0, half_h),
                )
            }
            ParticleRotationMode::Vertical => {
                (
                    p.position + Vec3::new(-half_w, -half_h, 0.0),
                    p.position + Vec3::new(half_w, -half_h, 0.0),
                    p.position + Vec3::new(half_w, half_h, 0.0),
                    p.position + Vec3::new(-half_w, half_h, 0.0),
                )
            }
            ParticleRotationMode::VerticalX => {
                // Aligned with YZ plane (facing ±X direction)
                (
                    p.position + Vec3::new(0.0, -half_h, -half_w),
                    p.position + Vec3::new(0.0, -half_h, half_w),
                    p.position + Vec3::new(0.0, half_h, half_w),
                    p.position + Vec3::new(0.0, half_h, -half_w),
                )
            }
            ParticleRotationMode::VerticalZ => {
                // Aligned with XY plane (facing ±Z direction)
                (
                    p.position + Vec3::new(-half_w, -half_h, 0.0),
                    p.position + Vec3::new(half_w, -half_h, 0.0),
                    p.position + Vec3::new(half_w, half_h, 0.0),
                    p.position + Vec3::new(-half_w, half_h, 0.0),
                )
            }
        };

        // Push 4 quad vertices
        positions.push([corner_0.x, corner_0.y, corner_0.z]);
        positions.push([corner_1.x, corner_1.y, corner_1.z]);
        positions.push([corner_2.x, corner_2.y, corner_2.z]);
        positions.push([corner_3.x, corner_3.y, corner_3.z]);

        let norm = [0.0, 1.0, 0.0];
        normals.push(norm);
        normals.push(norm);
        normals.push(norm);
        normals.push(norm);

        uvs.push([uv_rect[0], uv_rect[3]]);
        uvs.push([uv_rect[2], uv_rect[3]]);
        uvs.push([uv_rect[2], uv_rect[1]]);
        uvs.push([uv_rect[0], uv_rect[1]]);

        colors.push(color);
        colors.push(color);
        colors.push(color);
        colors.push(color);

        // Push 2 triangles (6 indices)
        indices.push(vertex_cursor);
        indices.push(vertex_cursor + 1);
        indices.push(vertex_cursor + 2);
        indices.push(vertex_cursor);
        indices.push(vertex_cursor + 2);
        indices.push(vertex_cursor + 3);

        vertex_cursor += 4;
    }

    if positions.is_empty() {
        if *batch_vis != Visibility::Hidden {
            *batch_vis = Visibility::Hidden;
        }
        return;
    }

    if *batch_vis != Visibility::Visible {
        *batch_vis = Visibility::Visible;
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
}
