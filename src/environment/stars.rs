use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use crate::player::PlayerCamera;

pub const STAR_COUNT: usize = 900;
pub const STAR_TWINKLE_GROUPS: usize = 4;
pub const STAR_DISTANCE: f32 = 850.0;
pub const STAR_BASE_SIZE: f32 = 4.6;

#[derive(Component)]
pub struct StarfieldRoot;

#[derive(Resource)]
pub struct StarfieldMaterialHandles(pub [Handle<StandardMaterial>; STAR_TWINKLE_GROUPS]);

pub fn setup_starfield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let star_image = generate_star_texture();
    let image_handle = images.add(star_image);

    let material_handles: [Handle<StandardMaterial>; STAR_TWINKLE_GROUPS] = std::array::from_fn(|_| {
        materials.add(StandardMaterial {
            base_color: Color::BLACK,
            base_color_texture: Some(image_handle.clone()),
            unlit: true,
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            double_sided: true,
            fog_enabled: false,
            ..default()
        })
    });

    commands.insert_resource(StarfieldMaterialHandles(material_handles.clone()));

    let star_data = generate_star_field_data(STAR_COUNT);
    let mut group_meshes = Vec::with_capacity(STAR_TWINKLE_GROUPS);

    for group_idx in 0..STAR_TWINKLE_GROUPS {
        let mesh = build_star_group_mesh(&star_data, group_idx);
        let mesh_handle = meshes.add(mesh);
        let entity = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handles[group_idx].clone()),
                Transform::default(),
                Visibility::Inherited,
                NotShadowCaster,
                NotShadowReceiver,
                bevy::camera::visibility::NoFrustumCulling,
            ))
            .id();
        group_meshes.push(entity);
    }

    commands
        .spawn((StarfieldRoot, Transform::default(), Visibility::Visible))
        .add_children(&group_meshes);
}

pub type StarfieldRootQuery<'w, 's> =
    Single<'w, 's, &'static mut Transform, (With<StarfieldRoot>, Without<Camera3d>)>;

pub fn sync_starfield(
    camera: Single<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    mut starfield_root: StarfieldRootQuery,
    material_handles: Res<StarfieldMaterialHandles>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: f32,
    elapsed_secs: f32,
    rain_factor: f32,
) {
    let camera_translation = camera.translation;
    let rotation = Quat::from_rotation_y(time_of_day * core::f32::consts::TAU);

    starfield_root.translation = camera_translation;
    starfield_root.rotation = rotation;

    let sun_elevation = calculate_sun_elevation(time_of_day);
    let night_fade = ((0.06 - sun_elevation) / 0.16).clamp(0.0, 1.0);
    let weather_vis = (1.0 - rain_factor * 0.92).clamp(0.0, 1.0);
    let base_intensity = night_fade * night_fade * weather_vis * 2.1;

    // 4 staggered gentle twinkle frequencies so the night sky feels softly alive
    const FREQS: [f32; STAR_TWINKLE_GROUPS] = [1.35, 2.05, 2.85, 3.60];
    const PHASES: [f32; STAR_TWINKLE_GROUPS] = [0.0, 1.7, 3.4, 5.1];

    for i in 0..STAR_TWINKLE_GROUPS {
        let twinkle = 0.80 + 0.20 * (elapsed_secs * FREQS[i] + PHASES[i]).sin();
        let intensity = base_intensity * twinkle;
        let target_color = Color::LinearRgba(LinearRgba::new(intensity, intensity, intensity, 1.0));
        if let Some(mut mat_mut) = materials.get_mut(&material_handles.0[i]) {
            mat_mut.base_color = target_color;
        }
    }
}

fn calculate_sun_elevation(time_of_day: f32) -> f32 {
    let angle = (time_of_day - 0.25) * core::f32::consts::TAU;
    angle.cos()
}

pub struct StarSpec {
    pub dir: Vec3,
    pub size: f32,
    pub color: [f32; 4],
    pub group: usize,
}

fn generate_star_field_data(count: usize) -> Vec<StarSpec> {
    let mut results = Vec::with_capacity(count);
    let mut seed = 0x57A8_C001_u32;
    let mut next_f32 = || -> f32 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 8) as f32 / 16777216.0
    };

    for idx in 0..count {
        let azimuth = next_f32() * core::f32::consts::TAU;
        // Uniform-ish hemisphere distribution with slightly fewer stars right at horizon
        let y_norm = 0.05 + next_f32().powf(0.85) * 0.94;
        let horizontal_radius = (1.0 - y_norm * y_norm).sqrt();

        let dir = Vec3::new(
            horizontal_radius * azimuth.cos(),
            y_norm,
            horizontal_radius * azimuth.sin(),
        )
        .normalize();

        // Natural magnitude distribution: mostly small crisp stars, occasional brighter beacons
        let mag = next_f32();
        let (size_mult, lum_mult) = if mag > 0.94 {
            (1.35 + next_f32() * 0.40, 1.15)
        } else if mag > 0.65 {
            (0.85 + next_f32() * 0.40, 0.82)
        } else {
            (0.48 + next_f32() * 0.35, 0.50)
        };

        // Subtle astronomical spectral color tints (blue-white, pure white, warm cream, pale gold)
        let hue_roll = next_f32();
        let rgb = if hue_roll < 0.28 {
            [0.78, 0.88, 1.0] // Cool blue-white
        } else if hue_roll < 0.68 {
            [0.96, 0.97, 1.0] // Crisp white
        } else if hue_roll < 0.88 {
            [1.0, 0.92, 0.78] // Warm cream
        } else {
            [1.0, 0.82, 0.62] // Pale golden amber
        };

        results.push(StarSpec {
            dir,
            size: STAR_BASE_SIZE * size_mult,
            color: [rgb[0] * lum_mult, rgb[1] * lum_mult, rgb[2] * lum_mult, 1.0],
            group: idx % STAR_TWINKLE_GROUPS,
        });
    }

    results
}

fn build_star_group_mesh(stars: &[StarSpec], group_idx: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for star in stars.iter().filter(|s| s.group == group_idx) {
        let center = star.dir * STAR_DISTANCE;
        let up_ref = if star.dir.y.abs() > 0.95 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        let right = up_ref.cross(star.dir).normalize() * (star.size * 0.5);
        let up = star.dir.cross(right).normalize() * (star.size * 0.5);
        let n = (-star.dir).to_array();

        let base = positions.len() as u32;
        positions.push((center - right - up).to_array());
        positions.push((center + right - up).to_array());
        positions.push((center + right + up).to_array());
        positions.push((center - right + up).to_array());

        normals.extend_from_slice(&[n, n, n, n]);
        uvs.extend_from_slice(&[[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
        colors.extend_from_slice(&[star.color; 4]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn generate_star_directions(count: usize) -> Vec<(Vec3, f32)> {
    generate_star_field_data(count)
        .into_iter()
        .map(|s| (s.dir, s.size))
        .collect()
}

fn generate_star_texture() -> Image {
    let size = 16;
    let mut data = Vec::with_capacity(size * size * 4);
    let center = (size as f32 - 1.0) * 0.5;

    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 - center).abs() / center;
            let dy = (y as f32 - center).abs() / center;
            let dist = (dx * dx + dy * dy).sqrt();

            let circle = (1.0 - dist).max(0.0).powf(2.0);
            let val = (circle * 255.0) as u8;

            data.extend_from_slice(&[val, val, val, val]);
        }
    }

    let mut img = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::linear();
    img
}
