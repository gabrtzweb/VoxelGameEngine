pub mod debug;
pub mod feedback;
pub mod icon;
pub mod interaction;
pub mod radial_menu;
pub mod shaping;
pub mod target_hud;
pub mod targeting;

pub use icon::{BlockIcons, setup_block_icons};
pub use interaction::SelectedVoxel;
pub use radial_menu::RadialMenuState;
pub use targeting::{CurrentTarget, TargetingSet, VoxelTarget};

use bevy::prelude::*;
use debug::VoxelDebugPlugin;
use feedback::InteractionFeedbackPlugin;
use icon::BlockIconPlugin;
use interaction::VoxelInteractionPlugin;
use shaping::ShapingPlugin;
use target_hud::TargetHudPlugin;
use targeting::TargetingPlugin;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TargetingPlugin,
            VoxelInteractionPlugin,
            InteractionFeedbackPlugin,
            ShapingPlugin,
            BlockIconPlugin,
            VoxelDebugPlugin,
            TargetHudPlugin,
        ));
    }
}
