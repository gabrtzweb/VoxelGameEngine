use std::{collections::HashMap, path::Path};

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::chunk::Voxel;

pub const TEXTURE_RESOLUTION: u32 = 16;

#[derive(Resource, Clone, Debug, Default)]
pub struct VoxelTextureRegistry {
    variants: HashMap<Voxel, (u16, u16)>,
    total_layers: u32,
}

impl VoxelTextureRegistry {
    pub fn register(&mut self, voxel: Voxel, start_layer: u16, count: u16) {
        self.variants.insert(voxel, (start_layer, count));
    }

    #[allow(dead_code)]
    pub fn total_layers(&self) -> u32 {
        self.total_layers
    }

    #[allow(dead_code)]
    pub fn variant_count(&self, voxel: Voxel) -> u16 {
        self.variants.get(&voxel).map_or(0, |&(_, count)| count)
    }

    pub fn get_layer(&self, voxel: Voxel, world_voxel: IVec3) -> u16 {
        let Some(&(start, count)) = self.variants.get(&voxel) else {
            return 0;
        };

        if count <= 1 {
            return start;
        }

        let mut h = (world_voxel.x as u32).wrapping_mul(0x85EB_CA6B);
        h ^= (world_voxel.y as u32).wrapping_mul(0xC2B2_AE35);
        h ^= (world_voxel.z as u32).wrapping_mul(0x27D4_EB2D);
        h ^= h >> 16;
        h = h.wrapping_mul(0x1656_67B1);
        h ^= h >> 13;

        let offset = (h as usize % count as usize) as u16;
        start + offset
    }
}

pub fn build_voxel_texture_array() -> (Image, VoxelTextureRegistry) {
    let mut raw_layers = Vec::new();
    let mut registry = VoxelTextureRegistry::default();
    let mut current_layer: u16 = 0;

    for &voxel in &Voxel::ALL {
        let Some(base_name) = voxel.texture_name() else {
            continue;
        };

        let variants = load_all_variants_for(base_name, voxel.fallback_color());
        let count = variants.len() as u16;

        registry.register(voxel, current_layer, count);
        current_layer += count;

        for layer_data in variants {
            raw_layers.extend_from_slice(&layer_data);
        }
    }

    let total_layers = current_layer.max(1) as u32;
    registry.total_layers = total_layers;

    let mut image = Image::new(
        Extent3d {
            width: TEXTURE_RESOLUTION,
            height: TEXTURE_RESOLUTION,
            depth_or_array_layers: total_layers,
        },
        TextureDimension::D2,
        raw_layers,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::nearest();

    (image, registry)
}

fn load_all_variants_for(base_name: &str, fallback_color: [u8; 4]) -> Vec<Vec<u8>> {
    let mut variants = Vec::new();

    let primary_path = format!("assets/textures/blocks/block_{base_name}.png");
    if let Some(data) = try_load_image(&primary_path) {
        variants.push(data);
    } else {
        let alt_zero = format!("assets/textures/blocks/block_{base_name}0.png");
        if let Some(data) = try_load_image(&alt_zero) {
            variants.push(data);
        }
    }

    for i in 1..64 {
        let path1 = format!("assets/textures/blocks/block_{base_name}{i}.png");
        let path2 = format!("assets/textures/blocks/block_{base_name}_{i}.png");

        if let Some(data) = try_load_image(&path1) {
            variants.push(data);
        } else if let Some(data) = try_load_image(&path2) {
            variants.push(data);
        } else {
            break;
        }
    }

    if variants.is_empty() {
        variants.push(solid_color_layer(fallback_color));
    }

    variants
}

fn try_load_image(path: &str) -> Option<Vec<u8>> {
    if !Path::new(path).exists() {
        return None;
    }

    let Ok(opened) = image::open(path) else {
        return None;
    };

    let rgba = opened.into_rgba8();
    if rgba.width() == TEXTURE_RESOLUTION && rgba.height() == TEXTURE_RESOLUTION {
        Some(rgba.into_raw())
    } else {
        warn!(
            "Texture at {} was not {}x{}, ignoring",
            path, TEXTURE_RESOLUTION, TEXTURE_RESOLUTION
        );
        None
    }
}

fn solid_color_layer(color: [u8; 4]) -> Vec<u8> {
    let pixel_count = (TEXTURE_RESOLUTION * TEXTURE_RESOLUTION) as usize;
    let mut data = Vec::with_capacity(pixel_count * 4);
    for _ in 0..pixel_count {
        data.extend_from_slice(&color);
    }
    data
}

#[cfg(test)]
mod tests {
    use super::{TEXTURE_RESOLUTION, build_voxel_texture_array};
    use crate::voxel::chunk::Voxel;
    use bevy::prelude::IVec3;

    #[test]
    fn texture_array_builds_with_dynamic_variants() {
        let (image, registry) = build_voxel_texture_array();
        assert_eq!(image.width(), TEXTURE_RESOLUTION);
        assert_eq!(image.height(), TEXTURE_RESOLUTION);
        assert!(registry.total_layers() >= 6);
        assert_eq!(
            image.texture_descriptor.size.depth_or_array_layers,
            registry.total_layers()
        );

        let grass_count = registry.variant_count(Voxel::Grass);
        assert!(grass_count >= 2, "Grass should have at least 2 variants");

        let layer_a = registry.get_layer(Voxel::Grass, IVec3::new(0, 0, 0));
        let layer_b = registry.get_layer(Voxel::Grass, IVec3::new(1, 0, 0));
        assert!(layer_a < registry.total_layers() as u16);
        assert!(layer_b < registry.total_layers() as u16);
    }
}
