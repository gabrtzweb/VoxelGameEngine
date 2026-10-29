use bevy::asset::RenderAssetUsages;

use bevy::camera::{Camera2d, ClearColorConfig, RenderTarget};

use bevy::image::{Image, ImageSampler};

use bevy::prelude::*;

use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use bevy::window::{PrimaryWindow, WindowRef};



use crate::core::{LocalConfig, UpscalingFilter};

use crate::player::PlayerCamera;



pub struct RenderScalePlugin;



impl Plugin for RenderScalePlugin {

    fn build(&self, app: &mut App) {

        app.init_resource::<RenderScaleState>()

            .add_systems(Startup, setup_render_scale)

            .add_systems(PreUpdate, sync_render_scale);

    }

}



/// Runtime resource storing the off-screen scaled image handle and tracking state.

#[derive(Resource)]

pub struct RenderScaleState {

    pub scale: f32,

    pub upscaling_filter: UpscalingFilter,

    pub target_image: Option<Handle<Image>>,

    pub last_window_size: UVec2,

    pub last_applied_scale: f32,

    pub last_applied_filter: Option<UpscalingFilter>,

}



impl Default for RenderScaleState {

    fn default() -> Self {

        Self {

            scale: 1.0,

            upscaling_filter: UpscalingFilter::Bilinear,

            target_image: None,

            last_window_size: UVec2::ZERO,

            last_applied_scale: -1.0,

            last_applied_filter: None,

        }

    }

}



/// Marker component for the 2D UI Camera that renders UI and the upscaled 3D image.

#[derive(Component)]

pub struct RenderScaleUiCamera;



/// Marker component for the full-screen background node displaying the scaled 3D texture.

#[derive(Component)]

pub struct RenderScaleBackgroundNode;



fn setup_render_scale(

    mut commands: Commands,

    mut images: ResMut<Assets<Image>>,

    local_config: Res<LocalConfig>,

    mut state: ResMut<RenderScaleState>,

) {

    let initial_scale = local_config.render_scale.clamp(0.50, 1.00);

    let initial_filter = local_config.upscaling_filter;

    state.scale = initial_scale;

    state.upscaling_filter = initial_filter;

    state.last_applied_scale = initial_scale;

    state.last_applied_filter = Some(initial_filter);

    state.last_window_size = UVec2::ZERO;



    let is_scaling = initial_scale < 0.999;

    let w = ((1280.0 * initial_scale).round() as u32).max(1);

    let h = ((720.0 * initial_scale).round() as u32).max(1);



    let mut image = Image::new_fill(

        Extent3d {

            width: w,

            height: h,

            depth_or_array_layers: 1,

        },

        TextureDimension::D2,

        &[0, 0, 0, 255],

        TextureFormat::Rgba8UnormSrgb,

        RenderAssetUsages::default(),

    );

    image.texture_descriptor.usage =

        TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;

    image.sampler = ImageSampler::linear();



    let image_handle = images.add(image);

    state.target_image = Some(image_handle.clone());



    // Persistent 2D UI camera: permanently owns UI rendering at native window resolution.

    commands.spawn((

        Camera2d,

        Camera {

            order: 10,

            clear_color: ClearColorConfig::None,

            ..default()

        },

        bevy::ui::IsDefaultUiCamera,

        bevy_egui::PrimaryEguiContext,

        bevy_egui::EguiContext::default(),

        RenderScaleUiCamera,

    ));



    // Persistent background node: displays upscaled 3D image when scaling, hidden at 100%.

    commands.spawn((

        Node {

            position_type: PositionType::Absolute,

            left: Val::Px(0.0),

            top: Val::Px(0.0),

            width: Val::Percent(100.0),

            height: Val::Percent(100.0),

            display: if is_scaling {

                Display::Flex

            } else {

                Display::None

            },

            ..default()

        },

        ImageNode {

            image: image_handle,

            ..default()

        },

        ZIndex(-100),

        bevy::picking::Pickable::IGNORE,

        RenderScaleBackgroundNode,

    ));

}



fn sync_render_scale(

    local_config: Res<LocalConfig>,

    mut state: ResMut<RenderScaleState>,

    mut images: ResMut<Assets<Image>>,

    mut commands: Commands,

    primary_window: Query<&Window, With<PrimaryWindow>>,

    camera_3d_query: Query<Entity, (With<Camera3d>, With<PlayerCamera>)>,

    mut bg_node_query: Query<&mut Node, With<RenderScaleBackgroundNode>>,

) {

    let Ok(window) = primary_window.single() else {

        return;

    };

    let Ok(camera_entity) = camera_3d_query.single() else {

        return;

    };



    let target_scale = local_config.render_scale.clamp(0.50, 1.00);

    let target_filter = local_config.upscaling_filter;

    state.scale = target_scale;

    state.upscaling_filter = target_filter;



    let window_w = window.physical_width().max(1);

    let window_h = window.physical_height().max(1);

    let current_window_size = UVec2::new(window_w, window_h);



    let scale_changed = (target_scale - state.last_applied_scale).abs() > 0.005;

    let size_changed = current_window_size != state.last_window_size;

    let filter_changed = state.last_applied_filter != Some(target_filter);



    if !scale_changed && !size_changed && !filter_changed {

        return;

    }



    state.last_applied_scale = target_scale;

    state.last_applied_filter = Some(target_filter);

    state.last_window_size = current_window_size;



    let is_scaling = target_scale < 0.999;



    if is_scaling {

        let scaled_w = ((window_w as f32 * target_scale).round() as u32).max(1);

        let scaled_h = ((window_h as f32 * target_scale).round() as u32).max(1);



        let image_handle = if let Some(ref handle) = state.target_image {

            if let Some(mut image) = images.get_mut(handle) {

                let current_size = image.size();

                if current_size.x != scaled_w || current_size.y != scaled_h {

                    image.resize(Extent3d {

                        width: scaled_w,

                        height: scaled_h,

                        depth_or_array_layers: 1,

                    });

                }



                match target_filter {

                    UpscalingFilter::Bilinear | UpscalingFilter::Fsr1 => {

                        image.sampler = ImageSampler::linear();

                    }

                }

            }

            handle.clone()

        } else {

            let mut image = Image::new_fill(

                Extent3d {

                    width: scaled_w,

                    height: scaled_h,

                    depth_or_array_layers: 1,

                },

                TextureDimension::D2,

                &[0, 0, 0, 255],

                TextureFormat::Rgba8UnormSrgb,

                RenderAssetUsages::default(),

            );

            image.texture_descriptor.usage =

                TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;

            image.sampler = ImageSampler::linear();

            let handle = images.add(image);

            state.target_image = Some(handle.clone());

            handle

        };



        // Render 3D world into scaled off-screen image

        commands

            .entity(camera_entity)

            .insert(RenderTarget::Image(image_handle.clone().into()));



        for mut node in &mut bg_node_query {

            node.display = Display::Flex;

        }

    } else {

        // 100% Native: Render 3D world directly to primary window

        commands

            .entity(camera_entity)

            .insert(RenderTarget::Window(WindowRef::Primary));



        // Hide upscaled background node; Camera2d remains active to cleanly render UI over Camera3d

        for mut node in &mut bg_node_query {

            node.display = Display::None;

        }

    }

}

