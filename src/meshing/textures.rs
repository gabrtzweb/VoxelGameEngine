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
pub const MAX_VOXEL_VARIANTS: usize = 2048;

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
    pub front: FaceTextureInfo,
}

#[derive(Resource, Clone, Debug)]
pub struct VoxelTextureRegistry {
    mappings: [Option<VoxelTextureMapping>; MAX_VOXEL_VARIANTS],
    pub full_grass: bool,
}

impl Default for VoxelTextureRegistry {
    fn default() -> Self {
        Self {
            mappings: [None; MAX_VOXEL_VARIANTS],
            full_grass: false,
        }
    }
}

impl VoxelTextureRegistry {
    pub fn register_multi_face(
        &mut self,
        voxel: Voxel,
        side: FaceTextureInfo,
        top: FaceTextureInfo,
        bottom: FaceTextureInfo,
    ) {
        self.register_multi_face_directional(voxel, side, top, bottom, side);
    }

    pub fn register_multi_face_directional(
        &mut self,
        voxel: Voxel,
        side: FaceTextureInfo,
        top: FaceTextureInfo,
        bottom: FaceTextureInfo,
        front: FaceTextureInfo,
    ) {
        let index = voxel as usize;
        if index < MAX_VOXEL_VARIANTS {
            self.mappings[index] = Some(VoxelTextureMapping {
                side,
                top,
                bottom,
                front,
            });
        }
    }

    #[inline(always)]
    pub fn get_face_texture_info(
        &self,
        voxel: Voxel,
        world_voxel: IVec3,
        direction: FaceDirection,
    ) -> (u16, u16) {
        self.get_face_texture_info_oriented(voxel, world_voxel, direction, 0)
    }

    #[inline(always)]
    pub fn get_face_texture_info_oriented(
        &self,
        voxel: Voxel,
        world_voxel: IVec3,
        direction: FaceDirection,
        orientation: u8,
    ) -> (u16, u16) {
        let Some(m) = self.mappings.get(voxel as usize).and_then(|m| *m) else {
            return (0, 1);
        };

        let face = if voxel.is_axis_aligned() {
            match orientation % 3 {
                1 => match direction {
                    FaceDirection::PositiveX => m.top,
                    FaceDirection::NegativeX => m.bottom,
                    _ => m.side,
                },
                2 => match direction {
                    FaceDirection::PositiveZ => m.top,
                    FaceDirection::NegativeZ => m.bottom,
                    _ => m.side,
                },
                _ => match direction {
                    FaceDirection::PositiveY => m.top,
                    FaceDirection::NegativeY => m.bottom,
                    _ => m.side,
                },
            }
        } else {
            let front_dir = match orientation % 4 {
                1 => FaceDirection::PositiveX,
                2 => FaceDirection::NegativeZ,
                3 => FaceDirection::NegativeX,
                _ => FaceDirection::PositiveZ,
            };

            match direction {
                FaceDirection::PositiveY => m.top,
                FaceDirection::NegativeY => m.bottom,
                d if d == front_dir && voxel.front_texture_name().is_some() => m.front,
                _ => {
                    if (voxel.is_tinted() || voxel.side_texture_name().is_some()) && self.full_grass
                    {
                        m.top
                    } else {
                        m.side
                    }
                }
            }
        };

        if face.variant_count <= 1 {
            return (face.start_layer, face.frame_count);
        }

        let bx = world_voxel.x;
        let by = world_voxel.y;
        let bz = world_voxel.z;

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
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FaceCacheKey {
    texture_name: String,
    overlay_name: Option<String>,
    tint: Option<[u8; 4]>,
}

pub fn build_voxel_texture_array() -> (Image, VoxelTextureRegistry) {
    let mut raw_layers = Vec::new();
    let mut registry = VoxelTextureRegistry::default();
    let mut current_layer: u16 = 0;
    let mut cache: std::collections::HashMap<FaceCacheKey, FaceTextureInfo> =
        std::collections::HashMap::new();

    let mut get_or_load_face = |tex_name: &str,
                                overlay_name: Option<&str>,
                                tint: Option<[f32; 4]>,
                                fallback: [u8; 4],
                                layers: &mut Vec<u8>,
                                layer_counter: &mut u16|
     -> FaceTextureInfo {
        let key = FaceCacheKey {
            texture_name: tex_name.to_string(),
            overlay_name: overlay_name.map(|s| s.to_string()),
            tint: tint.map(|t| {
                [
                    (t[0] * 255.0).round().clamp(0.0, 255.0) as u8,
                    (t[1] * 255.0).round().clamp(0.0, 255.0) as u8,
                    (t[2] * 255.0).round().clamp(0.0, 255.0) as u8,
                    (t[3] * 255.0).round().clamp(0.0, 255.0) as u8,
                ]
            }),
        };

        if let Some(&info) = cache.get(&key) {
            return info;
        }

        let mut variants = load_all_variants_for(tex_name, fallback);

        if let Some(ov_name) = overlay_name
            && let Some(overlay) = try_load_image(&format!("assets/textures/blocks/{ov_name}.png"))
        {
            if ov_name == "soil_grass_side_overlay" {
                if let Some(overlay_frame) = overlay.frames.first() {
                    let t = tint.unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    for variant in &mut variants {
                        for frame in &mut variant.frames {
                            for i in 0..256 {
                                let idx = i * 4;
                                let ov_a = overlay_frame[idx + 3] as f32 / 255.0;
                                if ov_a > 0.0 {
                                    let ov_r = overlay_frame[idx] as f32 * t[0];
                                    let ov_g = overlay_frame[idx + 1] as f32 * t[1];
                                    let ov_b = overlay_frame[idx + 2] as f32 * t[2];

                                    let base_r = frame[idx] as f32;
                                    let base_g = frame[idx + 1] as f32;
                                    let base_b = frame[idx + 2] as f32;

                                    frame[idx] = (base_r * (1.0 - ov_a) + ov_r * ov_a)
                                        .round()
                                        .clamp(0.0, 255.0)
                                        as u8;
                                    frame[idx + 1] = (base_g * (1.0 - ov_a) + ov_g * ov_a)
                                        .round()
                                        .clamp(0.0, 255.0)
                                        as u8;
                                    frame[idx + 2] = (base_b * (1.0 - ov_a) + ov_b * ov_a)
                                        .round()
                                        .clamp(0.0, 255.0)
                                        as u8;
                                }
                            }
                        }
                    }
                }
            } else if overlay.frames.len() > 1 {
                // Multi-frame animated overlay (e.g. Onyx and Opal glimmering over host stone)
                for variant in &mut variants {
                    let base_frame = variant
                        .frames
                        .first()
                        .cloned()
                        .unwrap_or_else(|| solid_color_texture(fallback).frames[0].clone());
                    variant.frames.clear();
                    for ov_frame in &overlay.frames {
                        let mut blended = base_frame.clone();
                        blend_single_overlay_frame(&mut blended, ov_frame);
                        variant.frames.push(blended);
                    }
                }
            } else if let Some(overlay_frame) = overlay.frames.first() {
                composite_overlay(&mut variants, overlay_frame);
            }
        }

        let variant_count = variants.len() as u16;
        let frame_count = variants.first().map_or(1, |v| v.frames.len() as u16);
        let start_layer = *layer_counter;

        for variant in variants {
            for frame in variant.frames {
                layers.extend_from_slice(&frame);
                *layer_counter += 1;
            }
        }

        let info = FaceTextureInfo {
            start_layer,
            variant_count,
            frame_count,
        };

        cache.insert(key, info);
        info
    };

    for &voxel in &Voxel::ALL {
        let Some(base_name) = voxel.texture_name() else {
            continue;
        };

        let side_name = voxel.side_texture_name().unwrap_or(base_name);
        let side_overlay = voxel.overlay_texture_name();
        let side_tint = if side_overlay == Some("soil_grass_side_overlay") {
            Some(voxel.tint_color())
        } else {
            None
        };
        let side_info = get_or_load_face(
            side_name,
            side_overlay,
            side_tint,
            voxel.fallback_color(),
            &mut raw_layers,
            &mut current_layer,
        );

        let top_info = if let Some(top_name) = voxel.top_texture_name() {
            let top_overlay = voxel
                .overlay_texture_name()
                .filter(|&o| o != "soil_grass_side_overlay");
            get_or_load_face(
                top_name,
                top_overlay,
                None,
                voxel.fallback_color(),
                &mut raw_layers,
                &mut current_layer,
            )
        } else if voxel.side_texture_name().is_some()
            && voxel.side_texture_name() != Some(base_name)
        {
            let top_overlay = voxel
                .overlay_texture_name()
                .filter(|&o| o != "soil_grass_side_overlay");
            get_or_load_face(
                base_name,
                top_overlay,
                None,
                voxel.fallback_color(),
                &mut raw_layers,
                &mut current_layer,
            )
        } else {
            side_info
        };

        let bottom_info = if let Some(bot_name) = voxel.bottom_texture_name() {
            if Some(bot_name) == voxel.top_texture_name()
                || (voxel.top_texture_name().is_none() && Some(bot_name) == Some(base_name))
            {
                top_info
            } else if Some(bot_name) == voxel.side_texture_name() {
                side_info
            } else {
                let bot_overlay = voxel
                    .overlay_texture_name()
                    .filter(|&o| o != "soil_grass_side_overlay");
                get_or_load_face(
                    bot_name,
                    bot_overlay,
                    None,
                    voxel.fallback_color(),
                    &mut raw_layers,
                    &mut current_layer,
                )
            }
        } else if voxel.side_texture_name().is_some() || voxel.top_texture_name().is_some() {
            if voxel.side_texture_name() == Some(base_name) && voxel.top_texture_name().is_none() {
                side_info
            } else {
                let bot_overlay = voxel
                    .overlay_texture_name()
                    .filter(|&o| o != "soil_grass_side_overlay");
                get_or_load_face(
                    base_name,
                    bot_overlay,
                    None,
                    voxel.fallback_color(),
                    &mut raw_layers,
                    &mut current_layer,
                )
            }
        } else {
            side_info
        };

        let front_info = if let Some(front_name) = voxel.front_texture_name() {
            get_or_load_face(
                front_name,
                None,
                None,
                voxel.fallback_color(),
                &mut raw_layers,
                &mut current_layer,
            )
        } else {
            side_info
        };

        registry.register_multi_face_directional(
            voxel,
            side_info,
            top_info,
            bottom_info,
            front_info,
        );
    }

    let total_layers = current_layer.max(1) as u32;

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

fn blend_single_overlay_frame(frame: &mut [u8], overlay_frame: &[u8]) {
    for i in 0..256 {
        let idx = i * 4;
        let ov_a = overlay_frame[idx + 3] as f32 / 255.0;
        if ov_a > 0.0 {
            let ov_r = overlay_frame[idx] as f32;
            let ov_g = overlay_frame[idx + 1] as f32;
            let ov_b = overlay_frame[idx + 2] as f32;

            let base_r = frame[idx] as f32;
            let base_g = frame[idx + 1] as f32;
            let base_b = frame[idx + 2] as f32;

            frame[idx] = (base_r * (1.0 - ov_a) + ov_r * ov_a)
                .round()
                .clamp(0.0, 255.0) as u8;
            frame[idx + 1] = (base_g * (1.0 - ov_a) + ov_g * ov_a)
                .round()
                .clamp(0.0, 255.0) as u8;
            frame[idx + 2] = (base_b * (1.0 - ov_a) + ov_b * ov_a)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
}

fn composite_overlay(variants: &mut [LoadedTexture], overlay_frame: &[u8]) {
    for variant in variants {
        for frame in &mut variant.frames {
            blend_single_overlay_frame(frame, overlay_frame);
        }
    }
}

fn load_all_variants_for(base_name: &str, fallback_color: [u8; 4]) -> Vec<LoadedTexture> {
    let mut variants = Vec::new();

    let unprefixed = base_name.split_once('_').map(|(_, rest)| rest);

    let mut primary_paths = vec![
        format!("assets/textures/blocks/{base_name}.png"),
        format!("assets/textures/blocks/{base_name}0.png"),
    ];

    if base_name.ends_with("_still") {
        let stripped = base_name.trim_end_matches("_still");
        primary_paths.push(format!("assets/textures/blocks/{stripped}.png"));
    } else if base_name.starts_with("liquid_") {
        primary_paths.insert(0, format!("assets/textures/blocks/{base_name}_still.png"));
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

    if !variants.is_empty() && variants[0].frames.len() > 1 {
        // Multi-frame animation strips only use their primary animation sequence
        return variants;
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

    // Cap animation frames to max 8 to respect GPU texture array layer limits
    let frame_count = ((final_height / TEXTURE_RESOLUTION) as usize).min(8);
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
