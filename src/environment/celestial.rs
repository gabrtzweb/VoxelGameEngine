use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    light::{CascadeShadowConfigBuilder, NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub const CELESTIAL_DISTANCE: f32 = 100.0;
pub const SUN_SIZE: f32 = 48.0;
pub const MOON_SIZE: f32 = 40.0;

#[derive(Component)]
pub struct SunVisual;

#[derive(Component)]
pub struct MoonVisual;

#[derive(Component)]
pub struct SunLight;

#[derive(Component)]
pub struct MoonLight;

#[derive(Component)]
pub struct SkyFillLight;

#[derive(Resource)]
pub struct MoonPhaseTextures(pub [Handle<Image>; 8]);

#[derive(Resource)]
pub struct CelestialMaterials {
    #[allow(dead_code)]
    pub sun: Handle<StandardMaterial>,
    pub moon: Handle<StandardMaterial>,
}

pub fn setup_celestial(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (sun_image, moon_images) = load_celestial_images();

    let sun_handle = images.add(sun_image);
    let moon_handles = moon_images.map(|img| images.add(img));

    commands.insert_resource(MoonPhaseTextures(moon_handles.clone()));

    let sun_mesh = meshes.add(Rectangle::new(SUN_SIZE, SUN_SIZE));
    let moon_mesh = meshes.add(Rectangle::new(MOON_SIZE, MOON_SIZE));

    let sun_material = materials.add(StandardMaterial {
        base_color_texture: Some(sun_handle),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        fog_enabled: false,
        ..default()
    });

    let moon_material = materials.add(StandardMaterial {
        base_color_texture: Some(moon_handles[0].clone()),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        fog_enabled: false,
        ..default()
    });

    commands.insert_resource(CelestialMaterials {
        sun: sun_material.clone(),
        moon: moon_material.clone(),
    });

    commands.spawn((
        SunVisual,
        Mesh3d(sun_mesh),
        MeshMaterial3d(sun_material),
        Transform::default(),
        Visibility::Visible,
        NotShadowCaster,
        NotShadowReceiver,
    ));

    commands.spawn((
        MoonVisual,
        Mesh3d(moon_mesh),
        MeshMaterial3d(moon_material),
        Transform::default(),
        Visibility::Visible,
        NotShadowCaster,
        NotShadowReceiver,
    ));

    commands.spawn((
        SunLight,
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85),
            illuminance: 6_500.0,
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.1,
            maximum_distance: 96.0,
            first_cascade_far_bound: 14.0,
            overlap_proportion: 0.25,
        }
        .build(),
        Transform::default(),
    ));

    commands.spawn((
        SkyFillLight,
        DirectionalLight {
            color: Color::srgb(0.62, 0.72, 0.90),
            illuminance: 2_200.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::default(),
    ));

    commands.spawn((
        MoonLight,
        DirectionalLight {
            color: Color::srgb(0.48, 0.56, 0.88),
            illuminance: 0.0,
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 3,
            minimum_distance: 0.1,
            maximum_distance: 64.0,
            first_cascade_far_bound: 12.0,
            overlap_proportion: 0.25,
        }
        .build(),
        Transform::default(),
    ));
}

pub fn calculate_sun_direction(time_of_day: f32) -> Vec3 {
    let angle = (time_of_day - 0.25) * core::f32::consts::TAU;
    let tilt = 0.22;
    Vec3::new(-angle.sin(), angle.cos(), -angle.cos() * tilt).normalize()
}

pub fn moon_phase_factor(phase: usize) -> f32 {
    match phase % 8 {
        0 => 1.0,
        1 | 7 => 0.75,
        2 | 6 => 0.50,
        3 | 5 => 0.25,
        4 => 0.05,
        _ => 1.0,
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn sync_celestial_transforms(
    camera: Single<&Transform, With<Camera3d>>,
    sun_visual: Single<
        (&mut Transform, &mut Visibility),
        (With<SunVisual>, Without<MoonVisual>, Without<Camera3d>),
    >,
    moon_visual: Single<
        (&mut Transform, &mut Visibility),
        (With<MoonVisual>, Without<SunVisual>, Without<Camera3d>),
    >,
    sun_light: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<SunLight>,
            Without<MoonLight>,
            Without<SkyFillLight>,
            Without<Camera3d>,
            Without<SunVisual>,
            Without<MoonVisual>,
        ),
    >,
    moon_light: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<MoonLight>,
            Without<SunLight>,
            Without<SkyFillLight>,
            Without<Camera3d>,
            Without<SunVisual>,
            Without<MoonVisual>,
        ),
    >,
    sky_fill: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<SkyFillLight>,
            Without<SunLight>,
            Without<MoonLight>,
            Without<Camera3d>,
            Without<SunVisual>,
            Without<MoonVisual>,
        ),
    >,
    celestial_materials: Res<CelestialMaterials>,
    moon_textures: Res<MoonPhaseTextures>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: f32,
    moon_phase: usize,
    day_sun_illuminance: f32,
    day_fill_illuminance: f32,
    night_moon_illuminance: f32,
) {
    let camera_translation = camera.translation;
    let sun_dir = calculate_sun_direction(time_of_day);
    let moon_dir = -sun_dir;

    let (mut sun_trans, mut sun_vis) = sun_visual.into_inner();
    let (mut moon_trans, mut moon_vis) = moon_visual.into_inner();
    let (mut sun_l, mut sun_lt) = sun_light.into_inner();
    let (mut moon_l, mut moon_lt) = moon_light.into_inner();
    let (mut fill_l, mut fill_lt) = sky_fill.into_inner();

    // Position sun billboard quad on the celestial sphere, facing the camera.
    sun_trans.translation = camera_translation + sun_dir * CELESTIAL_DISTANCE;
    *sun_trans = sun_trans.looking_to(sun_dir, Vec3::Y);
    *sun_vis = if sun_dir.y > -0.20 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    // Position moon billboard quad opposite to the sun, facing the camera.
    moon_trans.translation = camera_translation + moon_dir * CELESTIAL_DISTANCE;
    *moon_trans = moon_trans.looking_to(moon_dir, Vec3::Y);
    *moon_vis = if moon_dir.y > -0.20 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    // Update active moon phase texture if needed.
    if let Some(mut mat) = materials.get_mut(&celestial_materials.moon) {
        let target_handle = &moon_textures.0[moon_phase % 8];
        if mat.base_color_texture.as_ref() != Some(target_handle) {
            mat.base_color_texture = Some(target_handle.clone());
        }
    }

    // Directional sunlight shining from sun towards the world (-sun_dir).
    let sun_elev = sun_dir.y.max(0.0);
    sun_l.illuminance = sun_elev.powf(0.6) * day_sun_illuminance;
    sun_l.shadow_maps_enabled = sun_elev > 0.04;
    *sun_lt = Transform::default().looking_to(-sun_dir, Vec3::Y);

    // Directional moonlight shining from moon towards the world (-moon_dir).
    let moon_elev = moon_dir.y.max(0.0);
    let phase_factor = moon_phase_factor(moon_phase);
    moon_l.illuminance = moon_elev.powf(0.6) * (phase_factor * night_moon_illuminance);
    moon_l.shadow_maps_enabled = moon_elev > 0.04 && phase_factor > 0.20;
    *moon_lt = Transform::default().looking_to(-moon_dir, Vec3::Y);

    // Sky fill light softens shadows during daytime.
    fill_l.illuminance = sun_elev.powf(0.5) * day_fill_illuminance;
    *fill_lt = Transform::default().looking_to(Vec3::new(-0.30, -0.65, -0.30).normalize(), Vec3::Y);
}

fn load_celestial_images() -> (Image, [Image; 8]) {
    let sun_image = load_sun_image();
    let moon_images = load_moon_phase_images();
    (sun_image, moon_images)
}

fn load_sun_image() -> Image {
    let path = "assets/textures/environments/sun.png";
    if let Ok(opened) = image::open(path) {
        let rgba = opened.into_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for pixel in rgba.pixels() {
            let [r, g, b, _] = pixel.0;
            let is_black = r == 0 && g == 0 && b == 0;
            let alpha = if is_black { 0 } else { 255 };
            data.extend_from_slice(&[r, g, b, alpha]);
        }

        let mut img = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        img.sampler = ImageSampler::nearest();
        return img;
    }

    warn!(
        "Failed to load sun texture at {}, using solid fallback",
        path
    );
    solid_color_image(32, 32, [255, 230, 120, 255])
}

fn load_moon_phase_images() -> [Image; 8] {
    let path = "assets/textures/environments/moon_phases.png";
    if let Ok(opened) = image::open(path) {
        let rgba = opened.into_rgba8();
        if rgba.width() >= 128 && rgba.height() >= 64 {
            let mut phases: [Option<Image>; 8] = [None, None, None, None, None, None, None, None];

            for (phase, slot) in phases.iter_mut().enumerate() {
                let row = (phase / 4) as u32;
                let col = (phase % 4) as u32;

                let start_x = col * 32;
                let start_y = row * 32;

                let mut phase_bytes = Vec::with_capacity(32 * 32 * 4);
                for y in start_y..(start_y + 32) {
                    for x in start_x..(start_x + 32) {
                        let pixel = rgba.get_pixel(x, y);
                        let [r, g, b, _] = pixel.0;
                        let is_black = r == 0 && g == 0 && b == 0;
                        let alpha = if is_black { 0 } else { 255 };
                        phase_bytes.extend_from_slice(&[r, g, b, alpha]);
                    }
                }

                let mut img = Image::new(
                    Extent3d {
                        width: 32,
                        height: 32,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    phase_bytes,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                );
                img.sampler = ImageSampler::nearest();
                *slot = Some(img);
            }

            return phases.map(|opt| opt.unwrap());
        }
    }

    warn!(
        "Failed to slice moon phases at {}, using fallback array",
        path
    );
    core::array::from_fn(|_| solid_color_image(32, 32, [220, 230, 255, 255]))
}

fn solid_color_image(width: u32, height: u32, color: [u8; 4]) -> Image {
    let count = (width * height) as usize;
    let mut data = Vec::with_capacity(count * 4);
    for _ in 0..count {
        data.extend_from_slice(&color);
    }

    let mut img = Image::new(
        Extent3d {
            width,
            height,
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
    fn moon_phases_are_sliced_into_eight_discrete_images() {
        let phases = load_moon_phase_images();
        assert_eq!(phases.len(), 8);
        for (idx, img) in phases.iter().enumerate() {
            assert_eq!(img.width(), 32, "Phase {} width should be 32", idx);
            assert_eq!(img.height(), 32, "Phase {} height should be 32", idx);
        }
    }

    #[test]
    fn sun_and_moon_have_additive_transparent_backgrounds() {
        let sun = load_sun_image();
        let sun_data = sun.data.as_ref().unwrap();
        // Corner pixel (0, 0) should be pure black / zero alpha
        assert_eq!(sun_data[0], 0);
        assert_eq!(sun_data[1], 0);
        assert_eq!(sun_data[2], 0);
        assert_eq!(sun_data[3], 0);

        // Center pixel (16, 16) should be bright yellow sun core
        let center_idx = (16 * 32 + 16) * 4;
        assert!(sun_data[center_idx] > 200, "Sun center red should be > 200");
        assert!(
            sun_data[center_idx + 3] == 255,
            "Sun center alpha should be 255"
        );

        let phases = load_moon_phase_images();
        for (idx, phase_img) in phases.iter().enumerate() {
            let data = phase_img.data.as_ref().unwrap();
            // Corner pixel should be pure black (0, 0, 0, 0)
            assert_eq!(data[0], 0, "Phase {} corner red should be 0", idx);
            assert_eq!(data[1], 0, "Phase {} corner green should be 0", idx);
            assert_eq!(data[2], 0, "Phase {} corner blue should be 0", idx);
            assert_eq!(data[3], 0, "Phase {} corner alpha should be 0", idx);
        }
    }
}
