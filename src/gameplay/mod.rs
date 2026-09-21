#![allow(unused_imports)]

pub mod debug;
pub mod icon;
pub mod interaction;
pub mod radial_menu;
pub mod shaping;
pub mod target_hud;
pub mod targeting;

pub use debug::{VoxelDebugPlugin, VoxelDebugSettings};
pub use icon::{
    BlockIconPlugin, BlockIcons, ICON_SIZE, render_isometric_block_icon, setup_block_icons,
};
pub use interaction::{SelectedVoxel, VoxelInteractionPlugin, place_block};
pub use radial_menu::{
    RadialMenuRoot, RadialMenuSlice, RadialMenuState, RadialMenuSubtitleText, RadialMenuTitleText,
};
pub use shaping::{
    BlockShape, ShapingPlugin, centered_layer_coordinates, detect_current_shape,
    generate_shape_voxels, get_block_voxels, get_centered_layer_material, is_centered_column,
    is_centered_layer, is_layer_centered, rotate_block_90_y,
};
pub use target_hud::{
    TargetHudPlugin, TargetHudRoot, format_target_hud_title, resolve_target_block_info,
};
pub use targeting::{
    CurrentTarget, TargetingPlugin, TargetingSet, VoxelTarget, adjacent_block_origin,
    block_origin_from_voxel,
};

use bevy::prelude::*;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TargetingPlugin,
            VoxelInteractionPlugin,
            ShapingPlugin,
            BlockIconPlugin,
            VoxelDebugPlugin,
            TargetHudPlugin,
        ));
    }
}
