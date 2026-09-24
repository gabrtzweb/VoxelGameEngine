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

    #[allow(dead_code)]
    pub fn text_font(&self, size: f32) -> TextFont {
        TextFont {
            font: self.source(),
            font_size: FontSize::Px(size),
            ..default()
        }
    }
}

/// Creates a TextFont with the given size and the optional custom game font.
#[allow(dead_code)]
pub fn make_text_font(font: Option<&AppFont>, size: f32) -> TextFont {
    if let Some(f) = font {
        f.text_font(size)
    } else {
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        }
    }
}

/// Converts a Font Handle to a FontSource.
#[allow(dead_code)]
pub fn to_font_source(handle: &Handle<Font>) -> FontSource {
    FontSource::Handle(handle.clone())
}

/// Standard crisp drop shadow for game text to guarantee readability on all backgrounds.
pub fn text_shadow_default() -> TextShadow {
    TextShadow {
        offset: Vec2::new(1.0, 1.0),
        color: Color::srgba(0.0, 0.0, 0.0, 0.85),
    }
}

pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_app_font);
    }
}

fn setup_app_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<Font> = asset_server.load(DEFAULT_FONT_PATH);
    commands.insert_resource(AppFont { handle });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_shadow_default() {
        let shadow = text_shadow_default();
        assert_eq!(shadow.offset, Vec2::new(1.0, 1.0));
    }
}
