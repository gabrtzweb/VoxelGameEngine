use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

pub const CLOUD_ALTITUDE: f32 = 80.0;
pub const CLOUD_PLANE_SIZE: f32 = 1600.0;
pub const CLOUD_TILE_WORLD_SIZE: f32 = 280.0;
const WIND_SPEED_X: f32 = 1.8;
const WIND_SPEED_Z: f32 = 0.6;

#[derive(Component)]
pub struct CloudVisual;

#[derive(Resource)]
pub struct CloudMaterialHandle(pub Handle<StandardMaterial>);

pub fn setup_clouds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let cloud_image = load_cloud_image();
    let image_handle = images.add(cloud_image);

    let uv_repeat = CLOUD_PLANE_SIZE / CLOUD_TILE_WORLD_SIZE;
    let mesh = generate_horizontal_quad_mesh(CLOUD_PLANE_SIZE, uv_repeat);
    let mesh_handle = meshes.add(mesh);

    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.95),
        base_color_texture: Some(image_handle),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        fog_enabled: true,
        ..default()
    });

    commands.insert_resource(CloudMaterialHandle(material_handle.clone()));

    commands.spawn((
        CloudVisual,
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, CLOUD_ALTITUDE, 0.0),
        Visibility::Visible,
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

pub type CloudQuery<'w, 's> = Single<
    'w,
    's,
    (&'static mut Transform, &'static mut Visibility),
    (With<CloudVisual>, Without<Camera3d>),
>;

pub fn sync_clouds(
    time: Res<Time>,
    camera: Single<(&Transform, &GlobalTransform), With<Camera3d>>,
    mut cloud: CloudQuery,
    material_handle: Res<CloudMaterialHandle>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: f32,
    is_underwater: bool,
) {
    let (ref mut cloud_transform, ref mut visibility) = *cloud;
    if is_underwater {
        **visibility = Visibility::Hidden;
        return;
    }
    **visibility = Visibility::Visible;

    let elapsed = time.elapsed_secs();
    let camera_translation = camera.0.translation;

    // Drift clouds horizontally with the wind while keeping the plane centered on the player.
    // Offsetting by the tile modulo gives seamless continuous movement without edge boundary popping.
    let offset_x = (elapsed * WIND_SPEED_X).rem_euclid(CLOUD_TILE_WORLD_SIZE);
    let offset_z = (elapsed * WIND_SPEED_Z).rem_euclid(CLOUD_TILE_WORLD_SIZE);

    cloud_transform.translation = Vec3::new(
        camera_translation.x + offset_x,
        CLOUD_ALTITUDE,
        camera_translation.z + offset_z,
    );

    // Dynamic atmospheric cloud tinting across the day-night cycle.
    if let Some(mut mat) = materials.get_mut(&material_handle.0) {
        mat.base_color = sample_cloud_color(time_of_day);
    }
}

pub fn sample_cloud_color(time_of_day: f32) -> Color {
    // 0.00: Morning / Dawn
    // 0.25: Noon / Midday
    // 0.50: Evening / Sunset
    // 0.75: Night / Midnight
    let t = time_of_day.rem_euclid(1.0);

    let (c1, c2, factor) = if t < 0.25 {
        let f = t / 0.25;
        (
            LinearRgba::new(1.0, 0.84, 0.70, 0.90), // Dawn warm peach
            LinearRgba::new(1.0, 1.0, 1.0, 0.95),   // Midday bright white
            f,
        )
    } else if t < 0.50 {
        let f = (t - 0.25) / 0.25;
        (
            LinearRgba::new(1.0, 1.0, 1.0, 0.95),   // Midday bright white
            LinearRgba::new(1.0, 0.58, 0.44, 0.90), // Sunset vibrant amber/pink
            f,
        )
    } else if t < 0.75 {
        let f = (t - 0.50) / 0.25;
        (
            LinearRgba::new(1.0, 0.58, 0.44, 0.90), // Sunset vibrant amber/pink
            LinearRgba::new(0.14, 0.18, 0.30, 0.75), // Midnight deep navy
            f,
        )
    } else {
        let f = (t - 0.75) / 0.25;
        (
            LinearRgba::new(0.14, 0.18, 0.30, 0.75), // Midnight deep navy
            LinearRgba::new(1.0, 0.84, 0.70, 0.90),  // Dawn warm peach
            f,
        )
    };

    let lerped = LinearRgba::new(
        c1.red + (c2.red - c1.red) * factor,
        c1.green + (c2.green - c1.green) * factor,
        c1.blue + (c2.blue - c1.blue) * factor,
        c1.alpha + (c2.alpha - c1.alpha) * factor,
    );

    Color::LinearRgba(lerped)
}

fn generate_horizontal_quad_mesh(size: f32, uv_repeat: f32) -> Mesh {
    let half = size * 0.5;

    let positions = vec![
        [-half, 0.0, -half],
        [half, 0.0, -half],
        [half, 0.0, half],
        [-half, 0.0, half],
    ];

    let normals = vec![
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
    ];

    let uvs = vec![
        [0.0, 0.0],
        [uv_repeat, 0.0],
        [uv_repeat, uv_repeat],
        [0.0, uv_repeat],
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn load_cloud_image() -> Image {
    let path = "assets/textures/environments/clouds.png";
    if let Ok(opened) = image::open(path) {
        let rgba = opened.into_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        let mut img = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba.into_raw(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: ImageFilterMode::Nearest,
            min_filter: ImageFilterMode::Nearest,
            ..default()
        });
        return img;
    }

    warn!(
        "Failed to load cloud image at {}, using fallback procedural",
        path
    );
    solid_cloud_fallback()
}

fn solid_cloud_fallback() -> Image {
    let mut data = Vec::with_capacity(32 * 32 * 4);
    for _ in 0..(32 * 32) {
        data.extend_from_slice(&[255, 255, 255, 180]);
    }
    let mut img = Image::new(
        Extent3d {
            width: 32,
            height: 32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::nearest();
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_image_loads_with_alpha_channel() {
        let img = load_cloud_image();
        assert_eq!(img.width(), 256);
        assert_eq!(img.height(), 256);
        let data = img.data.as_ref().unwrap();
        assert_eq!(
            data[3], 0,
            "Corner pixel of clouds.png should be transparent"
        );
    }

    #[test]
    fn cloud_color_sampling_smoothness() {
        let noon_color = sample_cloud_color(0.25);
        if let Color::LinearRgba(c) = noon_color {
            assert!(c.red > 0.95 && c.green > 0.95 && c.blue > 0.95);
        } else {
            panic!("Expected LinearRgba");
        }
    }
}
