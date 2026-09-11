#![allow(dead_code)]

use bevy::{
    asset::RenderAssetUsages,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

const STAR_COUNT: usize = 1200;
const STAR_DOME_RADIUS: f32 = 135.0;
const STAR_BASE_SIZE: f32 = 0.55;

#[derive(Component)]
pub struct StarfieldVisual;

#[derive(Resource)]
pub struct StarfieldMaterialHandle(pub Handle<StandardMaterial>);

pub fn setup_starfield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = generate_star_dome_mesh(STAR_COUNT, STAR_DOME_RADIUS, STAR_BASE_SIZE);
    let mesh_handle = meshes.add(mesh);

    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    commands.insert_resource(StarfieldMaterialHandle(material_handle.clone()));

    commands.spawn((
        StarfieldVisual,
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::default(),
        Visibility::Visible,
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

pub fn sync_starfield(
    camera: Single<&Transform, With<Camera3d>>,
    starfield: Single<&mut Transform, (With<StarfieldVisual>, Without<Camera3d>)>,
    material_handle: Res<StarfieldMaterialHandle>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: f32,
) {
    let camera_translation = camera.translation;
    let mut star_transform = starfield.into_inner();

    // Center starfield on camera so it behaves like an infinite celestial sphere.
    star_transform.translation = camera_translation;

    // Slow celestial rotation around the polar axis.
    star_transform.rotation = Quat::from_rotation_y(time_of_day * core::f32::consts::TAU);

    // Calculate visibility: completely invisible when sun is above horizon (+0.05),
    // smoothly fades in during twilight, reaching maximum brilliance at night (-0.15).
    let sun_elevation = calculate_sun_elevation(time_of_day);
    let alpha = ((0.05 - sun_elevation) / 0.20).clamp(0.0, 1.0);

    if let Some(mut mat) = materials.get_mut(&material_handle.0) {
        mat.base_color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}

fn calculate_sun_elevation(time_of_day: f32) -> f32 {
    let angle = (time_of_day - 0.25) * core::f32::consts::TAU;
    angle.cos()
}

/// Generates a dome of inward-facing quads scattered pseudo-randomly over the upper hemisphere.
pub fn generate_star_dome_mesh(count: usize, radius: f32, base_size: f32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(count * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(count * 4);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(count * 4);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(count * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(count * 6);

    let mut seed = 0x57A8_C001_u32;
    let mut next_f32 = || -> f32 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 8) as f32 / 16777216.0
    };

    for _ in 0..count {
        // Uniform distribution over upper hemisphere (elevation between 3° and 88°).
        let azimuth = next_f32() * core::f32::consts::TAU;
        let y_norm = 0.05 + next_f32() * 0.93;
        let horizontal_radius = (1.0 - y_norm * y_norm).sqrt();

        let dir = Vec3::new(
            horizontal_radius * azimuth.cos(),
            y_norm,
            horizontal_radius * azimuth.sin(),
        );

        let center = dir * radius;
        let size = base_size * (0.65 + next_f32() * 0.85);

        // Subtle star color temperature tinting.
        let tint_selector = next_f32();
        let star_color = if tint_selector < 0.60 {
            [1.0, 1.0, 1.0, 1.0] // White
        } else if tint_selector < 0.80 {
            [0.85, 0.92, 1.0, 1.0] // Soft Blue (hot star)
        } else if tint_selector < 0.95 {
            [1.0, 0.95, 0.80, 1.0] // Warm Yellow
        } else {
            [1.0, 0.82, 0.72, 1.0] // Subtle Red Giant
        };

        // Construct billboard quad perpendicular to the radial direction towards center.
        let normal = -dir;
        let up = if dir.y.abs() > 0.95 { Vec3::Z } else { Vec3::Y };
        let right = dir.cross(up).normalize();
        let quad_up = right.cross(dir).normalize();

        let half_size = size * 0.5;
        let p0 = center - right * half_size - quad_up * half_size;
        let p1 = center + right * half_size - quad_up * half_size;
        let p2 = center + right * half_size + quad_up * half_size;
        let p3 = center - right * half_size + quad_up * half_size;

        let base_idx = positions.len() as u32;

        positions.push(p0.to_array());
        positions.push(p1.to_array());
        positions.push(p2.to_array());
        positions.push(p3.to_array());

        let norm_arr = normal.to_array();
        normals.push(norm_arr);
        normals.push(norm_arr);
        normals.push(norm_arr);
        normals.push(norm_arr);

        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([1.0, 1.0]);
        uvs.push([0.0, 1.0]);

        colors.push(star_color);
        colors.push(star_color);
        colors.push(star_color);
        colors.push(star_color);

        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_dome_mesh_generation_matches_quad_counts() {
        let count = 50;
        let mesh = generate_star_dome_mesh(count, 100.0, 0.5);
        assert_eq!(mesh.count_vertices(), count * 4);
    }
}
