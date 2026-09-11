use std::{collections::HashMap, path::Path};

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::chunk::Voxel;

pub const TEXTURE_RESOLUTION: u32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoxelTextureMapping {
    pub start_layer: u16,
    pub variant_count: u16,
    pub frame_count: u16,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct VoxelTextureRegistry {
    mappings: HashMap<Voxel, VoxelTextureMapping>,
    total_layers: u32,
}

impl VoxelTextureRegistry {
    pub fn register(
        &mut self,
        voxel: Voxel,
        start_layer: u16,
        variant_count: u16,
        frame_count: u16,
    ) {
        self.mappings.insert(
            voxel,
            VoxelTextureMapping {
                start_layer,
                variant_count,
                frame_count,
            },
        );
    }

    #[allow(dead_code)]
    pub fn total_layers(&self) -> u32 {
        self.total_layers
    }

    #[allow(dead_code)]
    pub fn variant_count(&self, voxel: Voxel) -> u16 {
        self.mappings.get(&voxel).map_or(0, |m| m.variant_count)
    }

    #[allow(dead_code)]
    pub fn frame_count(&self, voxel: Voxel) -> u16 {
        self.mappings.get(&voxel).map_or(1, |m| m.frame_count)
    }

    pub fn get_texture_info(&self, voxel: Voxel, world_voxel: IVec3) -> (u16, u16) {
        let Some(&m) = self.mappings.get(&voxel) else {
            return (0, 1);
        };

        if m.variant_count <= 1 {
            return (m.start_layer, m.frame_count);
        }

        let mut h = (world_voxel.x as u32).wrapping_mul(0x85EB_CA6B);
        h ^= (world_voxel.y as u32).wrapping_mul(0xC2B2_AE35);
        h ^= (world_voxel.z as u32).wrapping_mul(0x27D4_EB2D);
        h ^= h >> 16;
        h = h.wrapping_mul(0x1656_67B1);
        h ^= h >> 13;

        let variant = (h as usize % m.variant_count as usize) as u16;
        let start = m.start_layer + variant * m.frame_count;
        (start, m.frame_count)
    }

    #[allow(dead_code)]
    pub fn get_layer(&self, voxel: Voxel, world_voxel: IVec3) -> u16 {
        self.get_texture_info(voxel, world_voxel).0
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
        let variant_count = variants.len() as u16;
        let frame_count = variants.first().map_or(1, |v| v.frames.len() as u16);

        registry.register(voxel, current_layer, variant_count, frame_count);

        for variant in variants {
            for frame in variant.frames {
                raw_layers.extend_from_slice(&frame);
                current_layer += 1;
            }
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

#[derive(Clone, Debug)]
pub struct LoadedTexture {
    pub frames: Vec<Vec<u8>>,
}

fn load_all_variants_for(base_name: &str, fallback_color: [u8; 4]) -> Vec<LoadedTexture> {
    let mut variants = Vec::new();

    let unprefixed = base_name.split_once('_').map(|(_, rest)| rest);

    let mut primary_paths = vec![
        format!("assets/textures/blocks/{base_name}.png"),
        format!("assets/textures/blocks/{base_name}0.png"),
    ];

    if base_name == "liqd_water_still" {
        primary_paths.push("assets/textures/blocks/liqd_water.png".to_string());
    } else if base_name == "liqd_water" {
        primary_paths.insert(0, "assets/textures/blocks/liqd_water_still.png".to_string());
    }

    primary_paths.push(format!("assets/textures/blocks/block_{base_name}.png"));
    primary_paths.push(format!("assets/textures/blocks/block_{base_name}0.png"));

    let mut found_primary = false;
    for path in &primary_paths {
        if let Some(data) = try_load_image(path) {
            variants.push(data);
            found_primary = true;
            break;
        }
    }

    if !found_primary && let Some(suffix) = unprefixed {
        let legacy_paths = [
            format!("assets/textures/blocks/block_{suffix}.png"),
            format!("assets/textures/blocks/block_{suffix}0.png"),
        ];
        for path in &legacy_paths {
            if let Some(data) = try_load_image(path) {
                variants.push(data);
                break;
            }
        }
    }

    for i in 1..64 {
        let mut candidate_paths = vec![
            format!("assets/textures/blocks/{base_name}{i}.png"),
            format!("assets/textures/blocks/{base_name}_{i}.png"),
            format!("assets/textures/blocks/block_{base_name}{i}.png"),
            format!("assets/textures/blocks/block_{base_name}_{i}.png"),
        ];

        if let Some(suffix) = unprefixed {
            candidate_paths.push(format!("assets/textures/blocks/block_{suffix}{i}.png"));
            candidate_paths.push(format!("assets/textures/blocks/block_{suffix}_{i}.png"));
        }

        let mut found_variant = false;
        for path in &candidate_paths {
            if let Some(data) = try_load_image(path) {
                variants.push(data);
                found_variant = true;
                break;
            }
        }

        if !found_variant {
            break;
        }
    }

    if variants.is_empty() {
        variants.push(solid_color_texture(fallback_color));
    }

    variants
}

fn try_load_image(path: &str) -> Option<LoadedTexture> {
    if !Path::new(path).exists() {
        return None;
    }

    let Ok(opened) = image::open(path) else {
        return None;
    };

    let rgba = opened.into_rgba8();
    let width = rgba.width();
    let height = rgba.height();

    if width != TEXTURE_RESOLUTION || height == 0 || height % TEXTURE_RESOLUTION != 0 {
        warn!(
            "Texture at {} was {}x{}, expected width {} and height multiple of {}, ignoring",
            path, width, height, TEXTURE_RESOLUTION, TEXTURE_RESOLUTION
        );
        return None;
    }

    let frame_count = (height / TEXTURE_RESOLUTION) as usize;
    let mut frames = Vec::with_capacity(frame_count);
    let frame_pixel_count = (TEXTURE_RESOLUTION * TEXTURE_RESOLUTION) as usize;
    let raw = rgba.into_raw();

    for f in 0..frame_count {
        let mut frame_data = Vec::with_capacity(frame_pixel_count * 4);
        let start_y = f * TEXTURE_RESOLUTION as usize;
        for y in 0..TEXTURE_RESOLUTION as usize {
            let row_start = ((start_y + y) * TEXTURE_RESOLUTION as usize) * 4;
            let row_end = row_start + (TEXTURE_RESOLUTION as usize * 4);
            frame_data.extend_from_slice(&raw[row_start..row_end]);
        }
        frames.push(frame_data);
    }

    Some(LoadedTexture { frames })
}

fn solid_color_texture(color: [u8; 4]) -> LoadedTexture {
    let pixel_count = (TEXTURE_RESOLUTION * TEXTURE_RESOLUTION) as usize;
    let mut data = Vec::with_capacity(pixel_count * 4);
    for _ in 0..pixel_count {
        data.extend_from_slice(&color);
    }
    LoadedTexture { frames: vec![data] }
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
        assert_eq!(grass_count, 4, "Grass should have 4 variants");

        let stone_count = registry.variant_count(Voxel::Stone);
        assert_eq!(stone_count, 4, "Stone should have 4 variants");

        let dirt_count = registry.variant_count(Voxel::Dirt);
        assert_eq!(dirt_count, 4, "Dirt should have 4 variants");

        let sand_count = registry.variant_count(Voxel::Sand);
        assert_eq!(sand_count, 4, "Sand should have 4 variants");

        let water_count = registry.variant_count(Voxel::Water);
        assert_eq!(water_count, 1, "Water should have 1 variant");

        let water_frames = registry.frame_count(Voxel::Water);
        assert_eq!(water_frames, 36, "Water should have 36 animation frames");

        let (water_layer, frame_count) = registry.get_texture_info(Voxel::Water, IVec3::ZERO);
        assert_eq!(frame_count, 36);
        assert!(water_layer + frame_count <= registry.total_layers() as u16);

        let layer_a = registry.get_layer(Voxel::Grass, IVec3::new(0, 0, 0));
        let layer_b = registry.get_layer(Voxel::Grass, IVec3::new(1, 0, 0));
        assert!(layer_a < registry.total_layers() as u16);
        assert!(layer_b < registry.total_layers() as u16);
    }
}
