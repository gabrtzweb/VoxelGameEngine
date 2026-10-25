use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::LocalConfig;

/// Supported language locales in the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Reflect)]
pub enum Locale {
    #[default]
    EnUs,
    PtBr,
}

impl Locale {
    pub fn code(self) -> &'static str {
        match self {
            Self::EnUs => "en_us",
            Self::PtBr => "pt_br",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::EnUs => "English (US)",
            Self::PtBr => "Português (BR)",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().replace(['-', '.'], "_").as_str() {
            "pt_br" | "pt" | "portugues" | "português" => Self::PtBr,
            _ => Self::EnUs,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::EnUs => Self::PtBr,
            Self::PtBr => Self::EnUs,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::EnUs => Self::PtBr,
            Self::PtBr => Self::EnUs,
        }
    }
}

/// Central localization manager holding translation tables for active locale and fallback.
#[derive(Resource, Debug, Clone)]
pub struct Localization {
    pub active_locale: Locale,
    translations: HashMap<String, String>,
    fallback: HashMap<String, String>,
}

impl Default for Localization {
    fn default() -> Self {
        let mut loc = Self {
            active_locale: Locale::EnUs,
            translations: HashMap::new(),
            fallback: HashMap::new(),
        };
        loc.reload();
        loc
    }
}

impl Localization {
    pub fn new(locale: Locale) -> Self {
        let mut loc = Self {
            active_locale: locale,
            translations: HashMap::new(),
            fallback: HashMap::new(),
        };
        loc.reload();
        loc
    }

    /// Switches the active locale and reloads translation mapping immediately.
    pub fn set_locale(&mut self, locale: Locale) {
        if self.active_locale != locale {
            self.active_locale = locale;
            self.reload_active();
        }
    }

    /// Full reload of fallback (`en_us`) and active locale dictionaries from disk.
    pub fn reload(&mut self) {
        self.fallback = Self::load_file("en_us");
        self.reload_active();
    }

    fn reload_active(&mut self) {
        if self.active_locale == Locale::EnUs {
            self.translations = self.fallback.clone();
        } else {
            self.translations = Self::load_file(self.active_locale.code());
        }
    }

    fn load_file(code: &str) -> HashMap<String, String> {
        let paths = [
            format!("assets/lang/{}.json", code),
            format!("assets/lang/{}.json", code.replace('_', ".")),
        ];

        for path in &paths {
            if Path::new(path).exists()
                && let Ok(content) = fs::read_to_string(path)
                && let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content)
            {
                return map;
            }
        }
        HashMap::new()
    }

    /// Translates a key, falling back to English, then returning the key itself.
    #[inline]
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        if let Some(val) = self.translations.get(key) {
            val.as_str()
        } else if let Some(val) = self.fallback.get(key) {
            val.as_str()
        } else {
            key
        }
    }

    /// Formats a template string replacing `{0}`, `{1}`, etc. with corresponding arguments.
    pub fn t_fmt(&self, key: &str, args: &[&str]) -> String {
        let mut text = self.t(key).to_string();
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            text = text.replace(&placeholder, arg);
        }
        text
    }

    /// Alias for `t_fmt`.
    #[inline]
    pub fn t_args(&self, key: &str, args: &[&str]) -> String {
        self.t_fmt(key, args)
    }

    /// Translates a voxel block label (`"block.<id>"`), falling back to voxel.label().
    #[inline]
    pub fn t_block(&self, voxel: crate::world::Voxel) -> &str {
        let key = format!("block.{}", voxel.id());
        if let Some(val) = self.translations.get(&key) {
            val.as_str()
        } else if let Some(val) = self.fallback.get(&key) {
            val.as_str()
        } else {
            voxel.label()
        }
    }
}

pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        let initial_locale = app
            .world()
            .get_resource::<LocalConfig>()
            .map(|c| Locale::from_code(&c.language))
            .unwrap_or_default();

        app.insert_resource(Localization::new(initial_locale));
    }
}

