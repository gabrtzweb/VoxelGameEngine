use bevy::{
    camera::Exposure,
    light::{CascadeShadowConfigBuilder, NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use crate::{
    player::PlayerCamera,
    voxel::{VOXEL_SIZE, chunk::CHUNK_SIZE, chunk_manager::ChunkStreamingSettings},
};

const DAY_SUN_ILLUMINANCE: f32 = 7_000.0;
const DAY_FILL_ILLUMINANCE: f32 = 2_200.0;
const DAY_AMBIENT_BRIGHTNESS: f32 = 62.0;
const DAY_EXPOSURE_EV100: f32 = 11.4;

const NIGHT_MOON_ILLUMINANCE: f32 = 450.0;
const NIGHT_AMBIENT_BRIGHTNESS: f32 = 8.0;
const NIGHT_EXPOSURE_EV100: f32 = 9.2;

const FOG_START_FACTOR: f32 = 0.50;
const FOG_END_FACTOR: f32 = 0.90;

const SKY_BODY_DISTANCE: f32 = 28.0;
const SUN_VISUAL_RADIUS: f32 = 1.5;
const MOON_VISUAL_RADIUS: f32 = 1.2;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentPhase {
    #[default]
    Day,
    Night,
}

#[derive(Resource, Default)]
pub struct EnvironmentState {
    pub phase: EnvironmentPhase,
}

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct Moon;

#[derive(Component)]
struct SkyFill;

#[derive(Component)]
struct SunVisual;

#[derive(Component)]
struct MoonVisual;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnvironmentState>()
            .insert_resource(GlobalAmbientLight {
                color: day_ambient_color(),
                brightness: DAY_AMBIENT_BRIGHTNESS,
                ..default()
            })
            .insert_resource(ClearColor(day_clear_color()))
            .add_systems(Startup, setup_environment)
            .add_systems(
                Update,
                (toggle_day_night, sync_fog_distance, sync_sky_bodies).chain(),
            );
    }
}

fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sun_mesh = meshes.add(Sphere::new(SUN_VISUAL_RADIUS));

    let moon_mesh = meshes.add(Sphere::new(MOON_VISUAL_RADIUS));

    let sun_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.90, 0.55),

        emissive: LinearRgba::rgb(10.0, 7.0, 2.5),

        unlit: true,

        ..default()
    });

    let moon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.86, 0.90, 1.0),

        emissive: LinearRgba::rgb(1.5, 1.6, 2.2),

        unlit: true,

        ..default()
    });

    // Main sunlight.
    commands.spawn((
        Sun,
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85),

            illuminance: DAY_SUN_ILLUMINANCE,

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
        day_sun_transform(),
    ));

    // Soft blue directional fill light.
    //
    // This approximates indirect sky lighting and
    // prevents sun shadows from becoming pure black.
    commands.spawn((
        SkyFill,
        DirectionalLight {
            color: Color::srgb(0.62, 0.72, 0.90),

            illuminance: DAY_FILL_ILLUMINANCE,

            shadow_maps_enabled: false,

            ..default()
        },
        day_fill_transform(),
    ));

    // Moonlight.
    //
    // The entity stays alive during daytime with zero
    // illuminance so switching phases remains stable.
    commands.spawn((
        Moon,
        DirectionalLight {
            color: Color::srgb(0.48, 0.56, 0.88),

            illuminance: 0.0,

            shadow_maps_enabled: false,

            ..default()
        },
        night_moon_transform(),
    ));

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
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn toggle_day_night(
    keyboard: Res<ButtonInput<KeyCode>>,

    mut state: ResMut<EnvironmentState>,

    mut ambient: ResMut<GlobalAmbientLight>,

    mut clear_color: ResMut<ClearColor>,

    sun: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<Sun>,
            Without<Moon>,
            Without<SkyFill>,
            Without<SunVisual>,
            Without<MoonVisual>,
            Without<PlayerCamera>,
        ),
    >,

    moon: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<Moon>,
            Without<Sun>,
            Without<SkyFill>,
            Without<SunVisual>,
            Without<MoonVisual>,
            Without<PlayerCamera>,
        ),
    >,

    sky_fill: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<SkyFill>,
            Without<Sun>,
            Without<Moon>,
            Without<SunVisual>,
            Without<MoonVisual>,
            Without<PlayerCamera>,
        ),
    >,

    sun_visual: Single<
        &mut Visibility,
        (
            With<SunVisual>,
            Without<MoonVisual>,
            Without<Sun>,
            Without<Moon>,
            Without<SkyFill>,
            Without<PlayerCamera>,
        ),
    >,

    moon_visual: Single<
        &mut Visibility,
        (
            With<MoonVisual>,
            Without<SunVisual>,
            Without<Sun>,
            Without<Moon>,
            Without<SkyFill>,
            Without<PlayerCamera>,
        ),
    >,

    camera: Single<
        (&mut DistanceFog, &mut Exposure),
        (
            With<PlayerCamera>,
            Without<Sun>,
            Without<Moon>,
            Without<SkyFill>,
            Without<SunVisual>,
            Without<MoonVisual>,
        ),
    >,
) {
    if !keyboard.just_pressed(KeyCode::F6) {
        return;
    }

    state.phase = match state.phase {
        EnvironmentPhase::Day => EnvironmentPhase::Night,

        EnvironmentPhase::Night => EnvironmentPhase::Day,
    };

    let (mut sun_light, mut sun_transform) = sun.into_inner();

    let (mut moon_light, mut moon_transform) = moon.into_inner();

    let (mut sky_fill_light, mut sky_fill_transform) = sky_fill.into_inner();

    let mut sun_visual_visibility = sun_visual.into_inner();

    let mut moon_visual_visibility = moon_visual.into_inner();

    let (mut fog, mut exposure) = camera.into_inner();

    match state.phase {
        EnvironmentPhase::Day => {
            sun_light.illuminance = DAY_SUN_ILLUMINANCE;

            sun_light.shadow_maps_enabled = true;

            *sun_transform = day_sun_transform();

            moon_light.illuminance = 0.0;

            *moon_transform = night_moon_transform();

            sky_fill_light.illuminance = DAY_FILL_ILLUMINANCE;

            *sky_fill_transform = day_fill_transform();

            *sun_visual_visibility = Visibility::Visible;

            *moon_visual_visibility = Visibility::Hidden;

            ambient.color = day_ambient_color();

            ambient.brightness = DAY_AMBIENT_BRIGHTNESS;

            fog.color = day_fog_color();

            fog.directional_light_color = Color::srgba(1.0, 0.92, 0.78, 0.12);

            fog.directional_light_exponent = 20.0;

            exposure.ev100 = DAY_EXPOSURE_EV100;

            clear_color.0 = day_clear_color();
        }

        EnvironmentPhase::Night => {
            // Keep the Sun entity alive so its shadow
            // cascade state survives Day -> Night -> Day.
            sun_light.illuminance = 0.0;

            *sun_transform = day_sun_transform();

            moon_light.illuminance = NIGHT_MOON_ILLUMINANCE;

            *moon_transform = night_moon_transform();

            sky_fill_light.illuminance = 0.0;

            *sky_fill_transform = day_fill_transform();

            *sun_visual_visibility = Visibility::Hidden;

            *moon_visual_visibility = Visibility::Visible;

            ambient.color = night_ambient_color();

            ambient.brightness = NIGHT_AMBIENT_BRIGHTNESS;

            fog.color = night_fog_color();

            fog.directional_light_color = Color::srgba(0.40, 0.48, 0.75, 0.06);

            fog.directional_light_exponent = 16.0;

            exposure.ev100 = NIGHT_EXPOSURE_EV100;

            clear_color.0 = night_clear_color();
        }
    }

    info!(
        "Environment: {}",
        match state.phase {
            EnvironmentPhase::Day => {
                "Day"
            }

            EnvironmentPhase::Night => {
                "Night"
            }
        }
    );
}

fn sync_fog_distance(
    settings: Res<ChunkStreamingSettings>,

    fog: Single<&mut DistanceFog, With<PlayerCamera>>,
) {
    let mut fog = fog.into_inner();

    let chunk_world_size = CHUNK_SIZE as f32 * VOXEL_SIZE;

    let render_radius = settings.render_distance.max(1) as f32 * chunk_world_size;

    fog.falloff = FogFalloff::Linear {
        start: render_radius * FOG_START_FACTOR,

        end: render_radius * FOG_END_FACTOR,
    };
}

#[allow(clippy::type_complexity)]
fn sync_sky_bodies(
    camera: Single<&Transform, With<PlayerCamera>>,

    sun_visual: Single<
        &mut Transform,
        (With<SunVisual>, Without<MoonVisual>, Without<PlayerCamera>),
    >,

    moon_visual: Single<
        &mut Transform,
        (With<MoonVisual>, Without<SunVisual>, Without<PlayerCamera>),
    >,
) {
    let camera_transform = camera.into_inner();

    let mut sun_transform = sun_visual.into_inner();

    let mut moon_transform = moon_visual.into_inner();

    sun_transform.translation =
        camera_transform.translation - day_light_direction() * SKY_BODY_DISTANCE;

    moon_transform.translation =
        camera_transform.translation - night_light_direction() * SKY_BODY_DISTANCE;
}

fn day_sun_transform() -> Transform {
    Transform::default().looking_to(day_light_direction(), Vec3::Y)
}

fn day_fill_transform() -> Transform {
    Transform::default().looking_to(Vec3::new(-0.30, -0.45, -0.35).normalize(), Vec3::Y)
}

fn night_moon_transform() -> Transform {
    Transform::default().looking_to(night_light_direction(), Vec3::Y)
}

fn day_light_direction() -> Vec3 {
    Vec3::new(0.35, -0.88, 0.22).normalize()
}

fn night_light_direction() -> Vec3 {
    Vec3::new(-0.22, -0.72, -0.66).normalize()
}

fn day_ambient_color() -> Color {
    Color::srgb(0.80, 0.84, 0.92)
}

fn night_ambient_color() -> Color {
    Color::srgb(0.26, 0.30, 0.42)
}

fn day_clear_color() -> Color {
    Color::srgb(0.44, 0.66, 0.92)
}

fn night_clear_color() -> Color {
    Color::srgb(0.035, 0.055, 0.11)
}

fn day_fog_color() -> Color {
    Color::srgb(0.60, 0.72, 0.84)
}

fn night_fog_color() -> Color {
    Color::srgb(0.065, 0.085, 0.14)
}
