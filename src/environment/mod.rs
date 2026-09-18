#![allow(unused_imports)]

pub mod atmosphere;
pub mod celestial;
pub mod clouds;
pub mod stars;
pub mod time;

use bevy::prelude::*;

pub use atmosphere::*;
pub use celestial::*;
pub use clouds::*;
pub use stars::*;
pub use time::*;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        let default_state = EnvironmentState::default();
        let initial_clear = sample_sky_color(default_state.time_of_day);
        let initial_ambient = sample_ambient_color(default_state.time_of_day);
        let initial_brightness = sample_ambient_brightness(default_state.time_of_day);

        app.insert_resource(default_state)
            .insert_resource(GlobalAmbientLight {
                color: initial_ambient,
                brightness: initial_brightness,
                ..default()
            })
            .insert_resource(ClearColor(initial_clear))
            .add_systems(
                Startup,
                (
                    celestial::setup_celestial,
                    stars::setup_starfield,
                    clouds::setup_clouds,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_environment_input,
                    advance_environment_clock,
                    update_atmosphere,
                    sync_fog_distance,
                    sync_celestial_system,
                    sync_starfield_system,
                    sync_cloud_system,
                )
                    .chain()
                    .after(crate::player::PlayerSet::Movement),
            );
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_celestial_system(
    state: Res<EnvironmentState>,
    camera: Single<&Transform, With<Camera3d>>,
    sun_visual: Single<
        (&mut Transform, &mut Visibility),
        (
            With<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
            Without<Camera3d>,
        ),
    >,
    moon_visual: Single<
        (&mut Transform, &mut Visibility),
        (
            With<celestial::MoonVisual>,
            Without<celestial::SunVisual>,
            Without<Camera3d>,
        ),
    >,
    sun_light: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<celestial::SunLight>,
            Without<celestial::MoonLight>,
            Without<celestial::SkyFillLight>,
            Without<Camera3d>,
            Without<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
        ),
    >,
    moon_light: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<celestial::MoonLight>,
            Without<celestial::SunLight>,
            Without<celestial::SkyFillLight>,
            Without<Camera3d>,
            Without<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
        ),
    >,
    sky_fill: Single<
        (&mut DirectionalLight, &mut Transform),
        (
            With<celestial::SkyFillLight>,
            Without<celestial::SunLight>,
            Without<celestial::MoonLight>,
            Without<Camera3d>,
            Without<celestial::SunVisual>,
            Without<celestial::MoonVisual>,
        ),
    >,
    celestial_materials: Res<celestial::CelestialMaterials>,
    moon_textures: Res<celestial::MoonPhaseTextures>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    celestial::sync_celestial_transforms(
        camera,
        sun_visual,
        moon_visual,
        sun_light,
        moon_light,
        sky_fill,
        celestial_materials,
        moon_textures,
        materials,
        state.time_of_day,
        state.moon_texture_index(),
        state.moon_phase().illuminance_factor(),
        DAY_SUN_ILLUMINANCE,
        DAY_FILL_ILLUMINANCE,
        NIGHT_MOON_ILLUMINANCE,
    );
}

fn sync_starfield_system(
    state: Res<EnvironmentState>,
    camera: Single<&Transform, With<Camera3d>>,
    starfield_root: stars::StarfieldRootQuery,
    material_handle: Res<stars::StarfieldMaterialHandle>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    stars::sync_starfield(
        camera,
        starfield_root,
        material_handle,
        materials,
        state.time_of_day,
    );
}

fn sync_cloud_system(
    time: Res<Time>,
    state: Res<EnvironmentState>,
    camera: Single<(&Transform, &GlobalTransform), With<Camera3d>>,
    cloud: clouds::CloudQuery,
    material_handle: Res<clouds::CloudMaterialHandle>,
    materials: ResMut<Assets<StandardMaterial>>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
) {
    let is_underwater = env_status.as_ref().is_some_and(|s| s.is_camera_in_water);

    clouds::sync_clouds(
        time,
        camera,
        cloud,
        material_handle,
        materials,
        state.time_of_day,
        is_underwater,
    );
}
