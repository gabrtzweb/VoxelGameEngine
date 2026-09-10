use std::path::Path;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub const TEXTURE_RESOLUTION: u32 = 16;

pub const LAYER_GRASS_TOP: u16 = 0;
pub const LAYER_GRASS_SIDE: u16 = 1;
pub const LAYER_DIRT: u16 = 2;
pub const LAYER_STONE: u16 = 3;
pub const LAYER_SAND: u16 = 4;
pub const LAYER_WATER: u16 = 5;
pub const LAYER_LIGHT: u16 = 6;
pub const TOTAL_TEXTURE_LAYERS: u32 = 7;

pub fn build_voxel_texture_array() -> Image {
    let mut raw_layers: Vec<u8> = Vec::with_capacity(
        (TEXTURE_RESOLUTION * TEXTURE_RESOLUTION * 4 * TOTAL_TEXTURE_LAYERS) as usize,
    );

    let grass_top = load_or_fallback("assets/textures/blocks/block_grass.png", [82, 158, 64, 255]);

    let dirt = load_or_fallback("assets/textures/blocks/block_dirt.png", [107, 66, 33, 255]);

    let grass_side = load_or_create_grass_side(
        "assets/textures/blocks/block_grass_side.png",
        &grass_top,
        &dirt,
    );

    let stone = load_or_fallback(
        "assets/textures/blocks/block_stone.png",
        [122, 128, 133, 255],
    );

    let sand = load_or_fallback(
        "assets/textures/blocks/block_sand.png",
        [209, 194, 128, 255],
    );

    let water = load_or_fallback("assets/textures/blocks/block_water.png", [20, 89, 199, 180]);

    let light = load_or_fallback(
        "assets/textures/blocks/block_light.png",
        [255, 199, 64, 255],
    );

    raw_layers.extend_from_slice(&grass_top);
    raw_layers.extend_from_slice(&grass_side);
    raw_layers.extend_from_slice(&dirt);
    raw_layers.extend_from_slice(&stone);
    raw_layers.extend_from_slice(&sand);
    raw_layers.extend_from_slice(&water);
    raw_layers.extend_from_slice(&light);

    let mut image = Image::new(
        Extent3d {
            width: TEXTURE_RESOLUTION,
            height: TEXTURE_RESOLUTION,
            depth_or_array_layers: TOTAL_TEXTURE_LAYERS,
        },
        TextureDimension::D2,
        raw_layers,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::nearest();

    image
}

fn load_or_fallback(path: &str, fallback_color: [u8; 4]) -> Vec<u8> {
    if let Ok(opened) = image::open(path) {
        let rgba = opened.into_rgba8();
        if rgba.width() == TEXTURE_RESOLUTION && rgba.height() == TEXTURE_RESOLUTION {
            return rgba.into_raw();
        }

        warn!(
            "Texture at {} was not {}x{}, using fallback",
            path, TEXTURE_RESOLUTION, TEXTURE_RESOLUTION
        );
    } else {
        warn!("Failed to open texture at {}, using fallback", path);
    }

    solid_color_layer(fallback_color)
}

fn solid_color_layer(color: [u8; 4]) -> Vec<u8> {
    let pixel_count = (TEXTURE_RESOLUTION * TEXTURE_RESOLUTION) as usize;
    let mut data = Vec::with_capacity(pixel_count * 4);
    for _ in 0..pixel_count {
        data.extend_from_slice(&color);
    }
    data
}

fn load_or_create_grass_side(path: &str, grass_top: &[u8], dirt: &[u8]) -> Vec<u8> {
    let file_path = Path::new(path);
    if file_path.exists()
        && let Ok(opened) = image::open(file_path)
    {
        let rgba = opened.into_rgba8();
        if rgba.width() == TEXTURE_RESOLUTION && rgba.height() == TEXTURE_RESOLUTION {
            return rgba.into_raw();
        }
    }

    let mut data = dirt.to_vec();

    let fringe_depth: [u32; 16] = [3, 4, 3, 5, 4, 3, 3, 4, 5, 4, 3, 4, 3, 4, 5, 3];

    for x in 0..TEXTURE_RESOLUTION {
        let max_y = fringe_depth[x as usize];
        for y in 0..max_y {
            let index = ((y * TEXTURE_RESOLUTION + x) * 4) as usize;
            data[index..index + 4].copy_from_slice(&grass_top[index..index + 4]);
        }
    }

    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let image_buffer =
        image::RgbaImage::from_raw(TEXTURE_RESOLUTION, TEXTURE_RESOLUTION, data.clone());
    if let Some(buffer) = image_buffer {
        let _ = buffer.save(file_path);
    }

    data
}

#[cfg(test)]
mod tests {
    use super::{TEXTURE_RESOLUTION, TOTAL_TEXTURE_LAYERS, build_voxel_texture_array};

    #[test]
    fn texture_array_builds_with_correct_dimensions_and_layers() {
        let image = build_voxel_texture_array();
        assert_eq!(image.width(), TEXTURE_RESOLUTION);
        assert_eq!(image.height(), TEXTURE_RESOLUTION);
        assert_eq!(
            image.texture_descriptor.size.depth_or_array_layers,
            TOTAL_TEXTURE_LAYERS
        );
        let expected_bytes =
            (TEXTURE_RESOLUTION * TEXTURE_RESOLUTION * 4 * TOTAL_TEXTURE_LAYERS) as usize;
        assert_eq!(image.data.as_ref().map(|d| d.len()), Some(expected_bytes));
    }
}
