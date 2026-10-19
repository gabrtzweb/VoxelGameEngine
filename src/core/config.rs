use std::fs;
use std::path::Path;
use bevy::prelude::Resource;
use bevy::render::settings::Backends;
use serde::{Deserialize, Serialize};

pub const CONFIG_PATH: &str = "local_config.json";

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// Preferred graphics backend: "auto" (default, uses PRIMARY/Vulkan), "dx12", "vulkan", "gl"
    #[serde(default = "default_backend")]
    pub backend: String,
}

fn default_backend() -> String {
    "auto".to_string()
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
        }
    }
}

impl LocalConfig {
    /// Loads `local_config.json` if it exists, otherwise returns default settings.
    pub fn load_or_default() -> Self {
        if Path::new(CONFIG_PATH).exists() {
            if let Ok(content) = fs::read_to_string(CONFIG_PATH) {
                if let Ok(config) = serde_json::from_str::<LocalConfig>(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    /// Resolves the WGPU graphics backends based on:
    /// 1. `backend` in `local_config.json` (if explicitly set to "dx12", "vulkan", etc.)
    /// 2. `WGPU_BACKEND` environment variable
    /// 3. Default fallback: `Backends::PRIMARY` (Vulkan on desktop/Linux)
    pub fn resolve_backends(&self) -> Backends {
        match self.backend.to_lowercase().trim() {
            "dx12" | "directx12" | "d3d12" => Backends::DX12,
            "vulkan" | "vk" => Backends::VULKAN,
            "gl" | "opengl" => Backends::GL,
            "metal" => Backends::METAL,
            _ => Backends::from_env().unwrap_or(Backends::PRIMARY),
        }
    }
}
