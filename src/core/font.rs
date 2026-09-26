use bevy::prelude::*;
pub use bevy::text::FontSource;
pub use bevy::ui::widget::TextShadow;

/// Path to the active game font inside `assets/`.
/// To swap to a different font, change this reference.
pub const DEFAULT_FONT_PATH: &str = "fonts/CutePixel.ttf";

/// Shared game font handle available across all UI and HUD systems.
#[derive(Resource, Clone)]
pub struct AppFont {
    pub handle: Handle<Font>,
}

impl AppFont {
    pub fn source(&self) -> FontSource {
        FontSource::Handle(self.handle.clone())
    }
}

/// Standard crisp drop shadow for game text to guarantee readability on all backgrounds.
pub fn text_shadow_default() -> TextShadow {
    TextShadow {
        offset: Vec2::new(1.0, 1.0),
        color: Color::srgba(0.0, 0.0, 0.0, 0.85),
    }
}

impl FromWorld for AppFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let handle: Handle<Font> = asset_server.load(DEFAULT_FONT_PATH);
        AppFont { handle }
    }
}

pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppFont>();
    }
}
