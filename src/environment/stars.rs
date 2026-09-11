use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub const STAR_COUNT: usize = 250;
pub const STAR_DISTANCE: f32 = 140.0;
pub const STAR_BASE_SIZE: f32 = 0.9;

#[derive(Component)]
pub struct StarInstance {
    pub initial_dir: Vec3,
}

#[derive(Resource)]
pub struct StarfieldMaterialHandle(pub Handle<StandardMaterial>);

pub fn setup_starfield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let star_image = generate_star_texture();
    let image_handle = images.add(star_image);

    let quad_mesh = meshes.add(Rectangle::new(1.0, 1.0));

    let material_handle = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        base_color_texture: Some(image_handle),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        fog_enabled: false,
        ..default()
    });

    commands.insert_resource(StarfieldMaterialHandle(material_handle.clone()));

    let star_data = generate_star_directions(STAR_COUNT);

    for (initial_dir, size) in star_data {
        let up = if initial_dir.y.abs() > 0.95 {
            Vec3::Z
        } else {
            Vec3::Y
        };

        commands.spawn((
            StarInstance { initial_dir },
            Mesh3d(quad_mesh.clone()),
            MeshMaterial3d(material_handle.clone()),
            Transform::from_translation(initial_dir * STAR_DISTANCE)
                .looking_to(-initial_dir, up)
                .with_scale(Vec3::splat(size)),
            Visibility::Visible,
            NotShadowCaster,
            NotShadowReceiver,
        ));
    }
}

pub type StarQuery<'w, 's> =
    Query<'w, 's, (&'static StarInstance, &'static mut Transform), Without<Camera3d>>;

pub fn sync_starfield(
    camera: Single<&Transform, With<Camera3d>>,
    mut stars: StarQuery,
    material_handle: Res<StarfieldMaterialHandle>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: f32,
) {
    let camera_translation = camera.translation;
    let rotation = Quat::from_rotation_y(time_of_day * core::f32::consts::TAU);

    for (star, mut transform) in &mut stars {
        let dir = rotation * star.initial_dir;
        let up = if dir.y.abs() > 0.95 { Vec3::Z } else { Vec3::Y };
        let scale = transform.scale;
        *transform = Transform::from_translation(camera_translation + dir * STAR_DISTANCE)
            .looking_to(-dir, up)
            .with_scale(scale);
    }

    let sun_elevation = calculate_sun_elevation(time_of_day);
    let fade = ((0.05 - sun_elevation) / 0.18).clamp(0.0, 1.0);
    let intensity = fade * 1.6;

    if let Some(mut mat) = materials.get_mut(&material_handle.0) {
        mat.base_color = Color::LinearRgba(LinearRgba::new(intensity, intensity, intensity, 1.0));
    }
}

fn calculate_sun_elevation(time_of_day: f32) -> f32 {
    let angle = (time_of_day - 0.25) * core::f32::consts::TAU;
    angle.cos()
}

pub fn generate_star_directions(count: usize) -> Vec<(Vec3, f32)> {
    let mut results = Vec::with_capacity(count);
    let mut seed = 0x57A8_C001_u32;
    let mut next_f32 = || -> f32 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 8) as f32 / 16777216.0
    };

    for _ in 0..count {
        let azimuth = next_f32() * core::f32::consts::TAU;
        let y_norm = 0.07 + next_f32() * 0.90;
        let horizontal_radius = (1.0 - y_norm * y_norm).sqrt();

        let initial_dir = Vec3::new(
            horizontal_radius * azimuth.cos(),
            y_norm,
            horizontal_radius * azimuth.sin(),
        )
        .normalize();

        let size = STAR_BASE_SIZE * (0.75 + next_f32() * 0.50);
        results.push((initial_dir, size));
    }

    results
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

            let circle = (1.0 - dist).max(0.0).powf(1.8);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_distribution_is_valid() {
        let stars = generate_star_directions(STAR_COUNT);
        assert_eq!(stars.len(), STAR_COUNT);
        for (dir, size) in stars {
            assert!(dir.y > 0.0, "Star should be in upper hemisphere");
            assert!(
                (dir.length() - 1.0).abs() < 1e-4,
                "Direction must be normalized"
            );
            assert!(size > 0.0, "Star size must be positive");
        }
    }

    #[test]
    fn star_texture_generates_valid_image() {
        let img = generate_star_texture();
        assert_eq!(img.width(), 16);
        assert_eq!(img.height(), 16);
        let data = img.data.as_ref().unwrap();
        let center_idx = (7 * 16 + 7) * 4;
        assert!(data[center_idx] > 200, "Center should be bright");
    }
}
