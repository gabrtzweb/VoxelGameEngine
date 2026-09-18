use std::path::Path;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::greedy::FaceDirection;
use crate::world::Voxel;

pub const TEXTURE_RESOLUTION: u32 = 16;
pub const MAX_VOXEL_VARIANTS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceTextureInfo {
    pub start_layer: u16,
    pub variant_count: u16,
    pub frame_count: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoxelTextureMapping {
    pub side: FaceTextureInfo,
    pub top: FaceTextureInfo,
    pub bottom: FaceTextureInfo,
}

impl VoxelTextureMapping {
    #[allow(dead_code)]
    pub fn uniform(start_layer: u16, variant_count: u16, frame_count: u16) -> Self {
        let info = FaceTextureInfo {
            start_layer,
            variant_count,
            frame_count,
        };
        Self {
            side: info,
            top: info,
            bottom: info,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct VoxelTextureRegistry {
    mappings: [Option<VoxelTextureMapping>; MAX_VOXEL_VARIANTS],
    total_layers: u32,
    pub full_grass: bool,
}

impl Default for VoxelTextureRegistry {
    fn default() -> Self {
        Self {
            mappings: [None; MAX_VOXEL_VARIANTS],
            total_layers: 0,
            full_grass: false,
        }
    }
}

impl VoxelTextureRegistry {
    #[allow(dead_code)]
    pub fn register(
        &mut self,
        voxel: Voxel,
        start_layer: u16,
        variant_count: u16,
        frame_count: u16,
    ) {
        let index = voxel as usize;
        if index < MAX_VOXEL_VARIANTS {
            self.mappings[index] = Some(VoxelTextureMapping::uniform(
                start_layer,
                variant_count,
                frame_count,
            ));
        }
    }

    pub fn register_multi_face(
        &mut self,
        voxel: Voxel,
        side: FaceTextureInfo,
        top: FaceTextureInfo,
        bottom: FaceTextureInfo,
    ) {
        let index = voxel as usize;
        if index < MAX_VOXEL_VARIANTS {
            self.mappings[index] = Some(VoxelTextureMapping { side, top, bottom });
        }
    }

    #[allow(dead_code)]
    pub fn total_layers(&self) -> u32 {
        self.total_layers
    }

    #[allow(dead_code)]
    pub fn variant_count(&self, voxel: Voxel) -> u16 {
        self.mappings
            .get(voxel as usize)
            .and_then(|m| *m)
            .map_or(0, |m| m.side.variant_count)
    }

    #[allow(dead_code)]
    pub fn top_variant_count(&self, voxel: Voxel) -> u16 {
        self.mappings
            .get(voxel as usize)
            .and_then(|m| *m)
            .map_or(0, |m| m.top.variant_count)
    }

    #[allow(dead_code)]
    pub fn frame_count(&self, voxel: Voxel) -> u16 {
        self.mappings
            .get(voxel as usize)
            .and_then(|m| *m)
            .map_or(1, |m| m.side.frame_count)
    }

    #[inline(always)]
    pub fn get_face_texture_info(
        &self,
        voxel: Voxel,
        world_voxel: IVec3,
        direction: FaceDirection,
    ) -> (u16, u16) {
        let Some(m) = self.mappings.get(voxel as usize).and_then(|m| *m) else {
            return (0, 1);
        };

        let face = match direction {
            FaceDirection::PositiveY => m.top,
            FaceDirection::NegativeY => m.bottom,
            _ => {
                if voxel == Voxel::Grass && self.full_grass {
                    m.top
                } else {
                    m.side
                }
            }
        };

        if face.variant_count <= 1 {
            return (face.start_layer, face.frame_count);
        }

        let bx = world_voxel.x.div_euclid(2);
        let by = world_voxel.y.div_euclid(2);
        let bz = world_voxel.z.div_euclid(2);

        let mut h = (bx as u32).wrapping_mul(0x85EB_CA6B);
        h ^= (by as u32).wrapping_mul(0xC2B2_AE35);
        h ^= (bz as u32).wrapping_mul(0x27D4_EB2D);
        h ^= h >> 16;
        h = h.wrapping_mul(0x1656_67B1);
        h ^= h >> 13;

        let variant = (h as usize % face.variant_count as usize) as u16;
        let start = face.start_layer + variant * face.frame_count;
        (start, face.frame_count)
    }

    #[inline(always)]
    pub fn get_texture_info(&self, voxel: Voxel, world_voxel: IVec3) -> (u16, u16) {
        self.get_face_texture_info(voxel, world_voxel, FaceDirection::PositiveX)
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

        let side_name = voxel.side_texture_name().unwrap_or(base_name);
        let mut side_variants = load_all_variants_for(side_name, voxel.fallback_color());

        // Composite grass side overlay if this is Grass
        if voxel == Voxel::Grass {
            if let Some(overlay) = try_load_image("assets/textures/blocks/terr_grass_side_overlay.png") {
                if let Some(overlay_frame) = overlay.frames.first() {
                    let tint = voxel.tint_color();
                    for variant in &mut side_variants {
                        for frame in &mut variant.frames {
                            for i in 0..256 {
                                let idx = i * 4;
                                let ov_a = overlay_frame[idx + 3] as f32 / 255.0;
                                if ov_a > 0.0 {
                                    let ov_r = overlay_frame[idx] as f32 * tint[0];
                                    let ov_g = overlay_frame[idx + 1] as f32 * tint[1];
                                    let ov_b = overlay_frame[idx + 2] as f32 * tint[2];

                                    let base_r = frame[idx] as f32;
                                    let base_g = frame[idx + 1] as f32;
                                    let base_b = frame[idx + 2] as f32;

                                    let out_r = (base_r * (1.0 - ov_a) + ov_r * ov_a).round().clamp(0.0, 255.0) as u8;
                                    let out_g = (base_g * (1.0 - ov_a) + ov_g * ov_a).round().clamp(0.0, 255.0) as u8;
                                    let out_b = (base_b * (1.0 - ov_a) + ov_b * ov_a).round().clamp(0.0, 255.0) as u8;

                                    frame[idx] = out_r;
                                    frame[idx + 1] = out_g;
                                    frame[idx + 2] = out_b;
                                }
                            }
                        }
                    }
                }
            }
        }

        let side_variant_count = side_variants.len() as u16;
        let side_frame_count = side_variants.first().map_or(1, |v| v.frames.len() as u16);
        let side_start_layer = current_layer;

        for variant in side_variants {
            for frame in variant.frames {
                raw_layers.extend_from_slice(&frame);
                current_layer += 1;
            }
        }

        let side_info = FaceTextureInfo {
            start_layer: side_start_layer,
            variant_count: side_variant_count,
            frame_count: side_frame_count,
        };

        let top_info = if let Some(top_name) = voxel.top_texture_name() {
            let top_variants = load_all_variants_for(top_name, voxel.fallback_color());
            let top_variant_count = top_variants.len() as u16;
            let top_frame_count = top_variants.first().map_or(1, |v| v.frames.len() as u16);
            let top_start_layer = current_layer;

            for variant in top_variants {
                for frame in variant.frames {
                    raw_layers.extend_from_slice(&frame);
                    current_layer += 1;
                }
            }

            FaceTextureInfo {
                start_layer: top_start_layer,
                variant_count: top_variant_count,
                frame_count: top_frame_count,
            }
        } else if voxel.side_texture_name().is_some() && voxel.side_texture_name() != Some(base_name) {
            // If side was overridden, base_name is used for the top texture (e.g. Basalt, Mulch, Grass)
            let top_variants = load_all_variants_for(base_name, voxel.fallback_color());
            let top_variant_count = top_variants.len() as u16;
            let top_frame_count = top_variants.first().map_or(1, |v| v.frames.len() as u16);
            let top_start_layer = current_layer;

            for variant in top_variants {
                for frame in variant.frames {
                    raw_layers.extend_from_slice(&frame);
                    current_layer += 1;
                }
            }

            FaceTextureInfo {
                start_layer: top_start_layer,
                variant_count: top_variant_count,
                frame_count: top_frame_count,
            }
        } else {
            side_info
        };

        let bottom_info = if let Some(bot_name) = voxel.bottom_texture_name() {
            if Some(bot_name) == voxel.top_texture_name()
                || (voxel.top_texture_name().is_none() && Some(bot_name) == Some(base_name))
            {
                top_info
            } else {
                let bot_variants = load_all_variants_for(bot_name, voxel.fallback_color());
                let bot_variant_count = bot_variants.len() as u16;
                let bot_frame_count = bot_variants.first().map_or(1, |v| v.frames.len() as u16);
                let bot_start_layer = current_layer;

                for variant in bot_variants {
                    for frame in variant.frames {
                        raw_layers.extend_from_slice(&frame);
                        current_layer += 1;
                    }
                }

                FaceTextureInfo {
                    start_layer: bot_start_layer,
                    variant_count: bot_variant_count,
                    frame_count: bot_frame_count,
                }
            }
        } else if voxel.side_texture_name().is_some() {
            top_info
        } else {
            side_info
        };

        registry.register_multi_face(voxel, side_info, top_info, bottom_info);
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

    let mut rgba = opened.into_rgba8();
    let width = rgba.width();
    let height = rgba.height();

    if width == 0 || height == 0 {
        return None;
    }

    if width != TEXTURE_RESOLUTION {
        if height % width == 0 {
            let frame_count = height / width;
            let target_height = frame_count * TEXTURE_RESOLUTION;
            rgba = image::imageops::resize(
                &rgba,
                TEXTURE_RESOLUTION,
                target_height,
                image::imageops::FilterType::Nearest,
            );
        } else {
            warn!(
                "Texture at {} was {}x{}, expected height multiple of width {}, ignoring",
                path, width, height, width
            );
            return None;
        }
    }

    let final_height = rgba.height();
    if final_height == 0 || final_height % TEXTURE_RESOLUTION != 0 {
        warn!(
            "Texture at {} was {}x{}, expected height multiple of {}, ignoring",
            path, width, final_height, TEXTURE_RESOLUTION
        );
        return None;
    }

    let frame_count = (final_height / TEXTURE_RESOLUTION) as usize;
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
    use super::*;
    use crate::world::Voxel;
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

        let grass_side_count = registry.variant_count(Voxel::Grass);
        assert_eq!(grass_side_count, 4, "Grass sides should have 4 variants");

        let grass_top_count = registry.top_variant_count(Voxel::Grass);
        assert_eq!(grass_top_count, 8, "Grass top should have 8 variants");

        let stone_count = registry.variant_count(Voxel::Stone);
        assert_eq!(stone_count, 4, "Stone should have 4 variants");

        let dirt_count = registry.variant_count(Voxel::Dirt);
        assert_eq!(dirt_count, 4, "Dirt should have 4 variants");

        let sand_count = registry.variant_count(Voxel::Sand);
        assert_eq!(sand_count, 4, "Sand should have 4 variants");

        let water_count = registry.variant_count(Voxel::Water);
        assert_eq!(water_count, 1, "Water should have 1 variant");

        let water_flow_count = registry.variant_count(Voxel::WaterFlowing);
        assert_eq!(water_flow_count, 1, "WaterFlowing should have 1 variant");

        let water_flow_frames = registry.frame_count(Voxel::WaterFlowing);
        assert_eq!(water_flow_frames, 8, "WaterFlowing should have 8 frames");

        let water_frames = registry.frame_count(Voxel::Water);
        assert_eq!(water_frames, 36, "Water should have 36 animation frames");

        let (water_layer, frame_count) = registry.get_texture_info(Voxel::Water, IVec3::ZERO);
        assert_eq!(frame_count, 36);
        assert!(water_layer + frame_count <= registry.total_layers() as u16);

        let layer_a = registry.get_layer(Voxel::Grass, IVec3::new(0, 0, 0));
        let layer_b = registry.get_layer(Voxel::Grass, IVec3::new(1, 0, 0));
        assert!(layer_a < registry.total_layers() as u16);
        assert!(layer_b < registry.total_layers() as u16);

        // Verify OakWood has 6 side bark variants and uniform top/side layers
        let oak_wood_variants = registry.variant_count(Voxel::OakWood);
        assert_eq!(oak_wood_variants, 6, "OakWood should have 6 bark variants");
        let (oak_wood_side, _) = registry.get_face_texture_info(Voxel::OakWood, IVec3::ZERO, FaceDirection::PositiveX);
        let (oak_wood_top, _) = registry.get_face_texture_info(Voxel::OakWood, IVec3::ZERO, FaceDirection::PositiveY);
        assert_eq!(oak_wood_side, oak_wood_top, "OakWood bark-only has identical top and side start layer");

        // Verify OakWoodLog has distinct top log ring layer and side bark layer
        let (log_side, _) = registry.get_face_texture_info(Voxel::OakWoodLog, IVec3::ZERO, FaceDirection::PositiveX);
        let (log_top, _) = registry.get_face_texture_info(Voxel::OakWoodLog, IVec3::ZERO, FaceDirection::PositiveY);
        let (log_bot, _) = registry.get_face_texture_info(Voxel::OakWoodLog, IVec3::ZERO, FaceDirection::NegativeY);
        assert_ne!(log_side, log_top, "OakWoodLog top ring layer must differ from side bark layer");
        assert_eq!(log_top, log_bot, "OakWoodLog top and bottom share the log ring layer");

        // Verify BirchWood (4 variants) and BirchWoodLog
        let birch_variants = registry.variant_count(Voxel::BirchWood);
        assert_eq!(birch_variants, 4, "BirchWood should have 4 bark variants");
        let (birch_side, _) = registry.get_face_texture_info(Voxel::BirchWood, IVec3::ZERO, FaceDirection::PositiveX);
        let (birch_top, _) = registry.get_face_texture_info(Voxel::BirchWood, IVec3::ZERO, FaceDirection::PositiveY);
        assert_eq!(birch_side, birch_top, "BirchWood bark-only has identical top and side start layer");
        let (b_log_side, _) = registry.get_face_texture_info(Voxel::BirchWoodLog, IVec3::ZERO, FaceDirection::PositiveX);
        let (b_log_top, _) = registry.get_face_texture_info(Voxel::BirchWoodLog, IVec3::ZERO, FaceDirection::PositiveY);
        assert_ne!(b_log_side, b_log_top, "BirchWoodLog top ring must differ from bark side");

        // Verify PineWood (5 variants) and PineWoodLog
        let pine_variants = registry.variant_count(Voxel::PineWood);
        assert_eq!(pine_variants, 5, "PineWood should have 5 bark variants");
        let (pine_side, _) = registry.get_face_texture_info(Voxel::PineWood, IVec3::ZERO, FaceDirection::PositiveX);
        let (pine_top, _) = registry.get_face_texture_info(Voxel::PineWood, IVec3::ZERO, FaceDirection::PositiveY);
        assert_eq!(pine_side, pine_top, "PineWood bark-only has identical top and side start layer");
        let (p_log_side, _) = registry.get_face_texture_info(Voxel::PineWoodLog, IVec3::ZERO, FaceDirection::PositiveX);
        let (p_log_top, _) = registry.get_face_texture_info(Voxel::PineWoodLog, IVec3::ZERO, FaceDirection::PositiveY);
        assert_ne!(p_log_side, p_log_top, "PineWoodLog top ring must differ from bark side");

        // Verify Leaf textures and variants
        assert_eq!(registry.variant_count(Voxel::OakLeaves), 2, "OakLeaves should have 2 variants");
        assert_eq!(registry.variant_count(Voxel::BirchLeaves), 2, "BirchLeaves should have 2 variants");
        assert_eq!(registry.variant_count(Voxel::PineLeaves), 4, "PineLeaves should have 4 variants");
    }
}
